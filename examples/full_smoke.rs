//! # full_smoke — parallel async validation of all dig3 exchanges.
//!
//! Spawns one tokio task per exchange. Each task has a 25-second total
//! budget (REST connect + ticker + WS connect + 5s event collect).
//! join_all aggregates. No exchange can stall the harness.
//!
//! Run:
//!     cargo run --example full_smoke --release 2>&1 | tee smoke_full_report.txt
//!
//! No API keys required — public endpoints only.

use std::time::Instant;

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::traits::MarketData;
use digdigdig3::core::types::{AccountType, ExchangeId, SubscriptionRequest, Symbol};
use futures_util::StreamExt;
use tokio::time::{timeout, Duration};

// ── Exchange list ─────────────────────────────────────────────────────────────

/// All ExchangeId variants handled by ConnectorFactory::create_public.
/// Order matters only for display.
fn all_exchanges() -> Vec<ExchangeId> {
    vec![
        // CEX — full public REST + WS
        ExchangeId::Binance,
        ExchangeId::Bybit,
        ExchangeId::OKX,
        ExchangeId::KuCoin,
        ExchangeId::Kraken,
        ExchangeId::GateIO,
        ExchangeId::Bitfinex,
        ExchangeId::MEXC,
        ExchangeId::HTX,
        ExchangeId::BingX,
        ExchangeId::CryptoCom,
        ExchangeId::Upbit,
        ExchangeId::Deribit,
        ExchangeId::HyperLiquid,
        ExchangeId::Bitget,
        ExchangeId::Bitstamp,
        ExchangeId::Coinbase,
        ExchangeId::Gemini,
        // DEX
        ExchangeId::Dydx,
        ExchangeId::Lighter,
        // Data feeds with public access
        ExchangeId::YahooFinance,
        ExchangeId::CryptoCompare,
        ExchangeId::Twelvedata,
        ExchangeId::Polymarket,
        ExchangeId::Dukascopy,
        ExchangeId::Alpaca,
        ExchangeId::Krx,
        ExchangeId::Moex,
        // Auth-required — will show FAIL connect (useful diagnostic)
        ExchangeId::Polygon,
        ExchangeId::Finnhub,
        ExchangeId::Tiingo,
        ExchangeId::AlphaVantage,
        ExchangeId::AngelOne,
        ExchangeId::Zerodha,
        ExchangeId::Upstox,
        ExchangeId::Dhan,
        ExchangeId::Fyers,
        ExchangeId::Oanda,
        ExchangeId::JQuants,
        ExchangeId::Tinkoff,
        ExchangeId::Ib,
        ExchangeId::Futu,
        ExchangeId::Coinglass,
        ExchangeId::DefiLlama,
        ExchangeId::WhaleAlert,
        ExchangeId::Fred,
        ExchangeId::Bitquery,
        ExchangeId::Bls,
    ]
}

// ── Row ───────────────────────────────────────────────────────────────────────

#[derive(Debug)]
struct Row {
    exchange: ExchangeId,
    rest_status: String,
    ws_connect: String,
    ws_events: String,
}

// ── Per-exchange test ─────────────────────────────────────────────────────────

async fn test_exchange(id: ExchangeId) -> Row {
    let hub = ExchangeHub::new();
    let symbol = Symbol::new("BTC", "USDT");
    let symbol_str = symbol.to_concat();

    // ── REST: connect + ticker ───────────────────────────────────────────────
    let rest_status = match timeout(
        Duration::from_secs(10),
        hub.connect_public(id, false),
    )
    .await
    {
        Ok(Ok(())) => {
            match hub.rest(id) {
                Some(conn) => {
                    match timeout(
                        Duration::from_secs(10),
                        MarketData::get_ticker(&*conn, &symbol_str, AccountType::Spot),
                    )
                    .await
                    {
                        Ok(Ok(ticker)) => format!("OK {:.4}", ticker.last_price),
                        Ok(Err(e)) => {
                            // If ticker fails, try ping instead
                            let short = e.to_string();
                            let short = if short.len() > 60 { &short[..60] } else { &short };
                            format!("FAIL ticker: {}", short)
                        }
                        Err(_) => "FAIL ticker_timeout".to_string(),
                    }
                }
                None => "FAIL no_rest".to_string(),
            }
        }
        Ok(Err(e)) => {
            let short = e.to_string();
            let short = if short.len() > 60 { &short[..60] } else { &short };
            format!("FAIL connect: {}", short)
        }
        Err(_) => "FAIL connect_timeout".to_string(),
    };

    // ── WS: connect + subscribe + collect 5s ────────────────────────────────
    let (ws_connect, ws_events) = 'ws: {
        // Wire WS through hub
        let ws_result = timeout(
            Duration::from_secs(8),
            hub.connect_websocket(id, AccountType::Spot, false),
        )
        .await;

        match ws_result {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                let msg = e.to_string();
                let short = if msg.len() > 50 { &msg[..50] } else { &msg };
                break 'ws (format!("Unsupported: {}", short), "n/a".to_string());
            }
            Err(_) => break 'ws ("create_timeout".to_string(), "n/a".to_string()),
        }

        let ws = match hub.ws(id, AccountType::Spot) {
            Some(w) => w,
            None => break 'ws ("ws_none_after_connect".to_string(), "n/a".to_string()),
        };

        // Connect
        match timeout(Duration::from_secs(8), ws.connect(AccountType::Spot)).await {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                let short = e.to_string();
                let short = if short.len() > 50 { &short[..50] } else { &short };
                break 'ws (format!("FAIL connect: {}", short), "n/a".to_string());
            }
            Err(_) => break 'ws ("FAIL connect_timeout".to_string(), "n/a".to_string()),
        }

        // Subscribe to ticker
        let sub = SubscriptionRequest::ticker_for(symbol.clone(), AccountType::Spot);
        match timeout(Duration::from_secs(5), ws.subscribe(sub)).await {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                let short = e.to_string();
                let short = if short.len() > 50 { &short[..50] } else { &short };
                break 'ws ("Connected".to_string(), format!("FAIL subscribe: {}", short));
            }
            Err(_) => break 'ws ("Connected".to_string(), "FAIL subscribe_timeout".to_string()),
        }

        // Collect events for 5 seconds
        let mut stream = ws.event_stream();
        let mut event_count = 0u32;
        let collect_start = Instant::now();
        let collect_budget = Duration::from_secs(5);

        loop {
            let remaining = collect_budget.saturating_sub(collect_start.elapsed());
            if remaining.is_zero() {
                break;
            }
            match timeout(remaining, stream.next()).await {
                Ok(Some(Ok(_event))) => {
                    event_count += 1;
                }
                Ok(Some(Err(_))) => {
                    // Stream error — stop collecting
                    break;
                }
                Ok(None) => break, // stream ended
                Err(_) => break,   // 5s budget exhausted
            }
        }

        let events_str = if event_count == 0 {
            "0 events (silent!)".to_string()
        } else {
            format!("{} events in 5s", event_count)
        };

        ("Connected".to_string(), events_str)
    };

    Row {
        exchange: id,
        rest_status,
        ws_connect,
        ws_events,
    }
}

// ── Main ──────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    let exchanges = all_exchanges();
    let total = exchanges.len();

    println!("=== FULL SMOKE — {} exchanges in parallel ===", total);
    println!("Each exchange: 25s total budget (REST + WS + 5s collect)");
    println!();

    // Spawn all jobs concurrently — one hang must NOT block others.
    let handles: Vec<_> = exchanges
        .into_iter()
        .map(|id| {
            tokio::spawn(async move {
                // Per-job 25s hard cap covering REST + WS + collect
                timeout(Duration::from_secs(25), test_exchange(id))
                    .await
                    .unwrap_or_else(|_| Row {
                        exchange: id,
                        rest_status: "TIMEOUT 25s".to_string(),
                        ws_connect: "TIMEOUT".to_string(),
                        ws_events: "n/a".to_string(),
                    })
            })
        })
        .collect();

    // join_all waits for ALL tasks, regardless of which finish first.
    let results = futures_util::future::join_all(handles).await;

    let mut rows: Vec<Row> = results
        .into_iter()
        .filter_map(|r| r.ok())
        .collect();

    // Sort by exchange name for stable output
    rows.sort_by(|a, b| format!("{:?}", a.exchange).cmp(&format!("{:?}", b.exchange)));

    // Print per-exchange results
    println!("=== PER-EXCHANGE RESULTS ===");
    for row in &rows {
        println!(
            "[{:?}] REST: {} | WS: {} | Events: {}",
            row.exchange, row.rest_status, row.ws_connect, row.ws_events
        );
    }

    // Compute summary
    let runtime = start.elapsed();

    let rest_ok: Vec<&Row> = rows.iter().filter(|r| r.rest_status.starts_with("OK")).collect();
    let rest_ok_count = rest_ok.len();

    let ws_connected: Vec<&Row> = rows
        .iter()
        .filter(|r| r.ws_connect == "Connected")
        .collect();
    let ws_connected_count = ws_connected.len();

    let ws_flowing: Vec<&Row> = rows
        .iter()
        .filter(|r| {
            r.ws_connect == "Connected"
                && !r.ws_events.starts_with("0 events")
                && !r.ws_events.starts_with("FAIL")
                && r.ws_events != "n/a"
        })
        .collect();
    let ws_flowing_count = ws_flowing.len();

    let ws_silent: Vec<&str> = rows
        .iter()
        .filter(|r| r.ws_connect == "Connected" && r.ws_events.starts_with("0 events"))
        .map(|r| r.exchange.as_str())
        .collect();

    let rest_failures: Vec<String> = rows
        .iter()
        .filter(|r| !r.rest_status.starts_with("OK") && r.rest_status != "TIMEOUT 25s")
        .map(|r| format!("{:?}({})", r.exchange, &r.rest_status))
        .collect();

    let timeouts: Vec<String> = rows
        .iter()
        .filter(|r| r.rest_status == "TIMEOUT 25s")
        .map(|r| format!("{:?}", r.exchange))
        .collect();

    println!();
    println!("=== SUMMARY ===");
    println!("Total: {} exchanges", total);
    println!("REST OK: {}/{}", rest_ok_count, total);
    println!("WS Connected: {}/{}", ws_connected_count, total);
    println!("WS events flowing: {}/{}", ws_flowing_count, ws_connected_count);

    if !ws_silent.is_empty() {
        println!("Silent streams (subscribe ok, 0 events): {} — {:?}", ws_silent.len(), ws_silent);
    }
    if !timeouts.is_empty() {
        println!("Timeouts (25s budget): {} — {:?}", timeouts.len(), timeouts);
    }
    if !rest_failures.is_empty() {
        println!(
            "REST failures: {} — [{}]",
            rest_failures.len(),
            rest_failures.join(", ")
        );
    }

    println!("Total runtime: {:.1}s", runtime.as_secs_f64());

    Ok(())
}
