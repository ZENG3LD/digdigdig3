//! # liq_capture — 5-minute live liquidation feed validator.
//!
//! Connects to a single exchange via WS, subscribes to the public
//! market-wide Liquidation stream for BTC-USDT perp, collects events
//! for `--duration <secs>` (default 300), then prints a summary.
//!
//! Usage:
//!     cargo run --example liq_capture --release -- --exchange Binance
//!     cargo run --example liq_capture --release -- --exchange Bybit --duration 60
//!
//! Supported: Binance, Bybit, Bitget, GateIO

use std::time::{SystemTime, UNIX_EPOCH};

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{
    AccountType, ExchangeId, StreamEvent, StreamType, SubscriptionRequest, Symbol,
};
use digdigdig3::core::utils::SymbolNormalizer;
use futures_util::StreamExt;
use tokio::time::{timeout, Duration, Instant};

// ─────────────────────────────────────────────────────────────────────────────
// CLI
// ─────────────────────────────────────────────────────────────────────────────

struct Args {
    exchange: ExchangeId,
    duration_secs: u64,
}

fn parse_args() -> Args {
    let argv: Vec<String> = std::env::args().collect();
    let mut exchange_str: Option<String> = None;
    let mut duration_secs: u64 = 300;
    let mut i = 1usize;
    while i < argv.len() {
        match argv[i].as_str() {
            "--exchange" => {
                i += 1;
                if i < argv.len() {
                    exchange_str = Some(argv[i].clone());
                }
            }
            "--duration" => {
                i += 1;
                if i < argv.len() {
                    duration_secs = argv[i].parse().unwrap_or(300);
                }
            }
            _ => {}
        }
        i += 1;
    }
    let exchange = match exchange_str.as_deref() {
        Some("Binance") => ExchangeId::Binance,
        Some("Bybit") => ExchangeId::Bybit,
        Some("Bitget") => ExchangeId::Bitget,
        Some("GateIO") => ExchangeId::GateIO,
        other => {
            eprintln!("Unknown or missing --exchange. Got: {:?}", other);
            eprintln!("Valid: Binance, Bybit, Bitget, GateIO");
            std::process::exit(1);
        }
    };
    Args { exchange, duration_secs }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn btc_usdt_futures_symbol(id: ExchangeId) -> Symbol {
    let base = Symbol::new("BTC", "USDT");
    let raw = SymbolNormalizer::to_exchange(id, &base, AccountType::FuturesCross)
        .unwrap_or_else(|_| "BTCUSDT".to_string());
    Symbol::with_raw("BTC", "USDT", raw)
}

// ─────────────────────────────────────────────────────────────────────────────
// Main
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    let args = parse_args();
    let id = args.exchange;
    let capture_dur = Duration::from_secs(args.duration_secs);

    // Set WS trace env so the jsonl file gets written.
    let exchange_name = format!("{:?}", id);
    let trace_tag = format!("liq_capture/{}", exchange_name);
    // Safety: single-threaded startup, no concurrent env access at this point.
    unsafe { std::env::set_var("DIG3_WS_TRACE", &trace_tag) };

    println!("[liq_capture] exchange={} duration={}s trace={}", exchange_name, args.duration_secs, trace_tag);

    let sym = btc_usdt_futures_symbol(id);
    println!("[liq_capture] symbol raw={}", sym.raw.as_deref().unwrap_or("(none)"));

    // Connect hub WS.
    let hub = ExchangeHub::new();
    println!("[liq_capture] connecting WS...");
    match timeout(Duration::from_secs(15), hub.connect_websocket(id, AccountType::FuturesCross, false)).await {
        Ok(Ok(())) => println!("[liq_capture] WS connected"),
        Ok(Err(e)) => {
            println!("[liq_capture] CONNECT_FAIL: {}", e);
            return;
        }
        Err(_) => {
            println!("[liq_capture] CONNECT_TIMEOUT (15s)");
            return;
        }
    }

    let ws = match hub.ws(id, AccountType::FuturesCross) {
        Some(w) => w,
        None => {
            println!("[liq_capture] ERROR: hub.ws() returned None after successful connect");
            return;
        }
    };

    // Connect the WS transport.
    println!("[liq_capture] ws.connect()...");
    match timeout(Duration::from_secs(15), ws.connect(AccountType::FuturesCross)).await {
        Ok(Ok(())) => println!("[liq_capture] ws transport connected"),
        Ok(Err(e)) => {
            println!("[liq_capture] WS_TRANSPORT_FAIL: {}", e);
            return;
        }
        Err(_) => {
            println!("[liq_capture] WS_TRANSPORT_TIMEOUT (15s)");
            return;
        }
    }

    // Subscribe to Liquidation.
    let sub = SubscriptionRequest {
        symbol: sym.clone(),
        stream_type: StreamType::Liquidation,
        account_type: AccountType::FuturesCross,
        depth: None,
        update_speed_ms: None,
    };

    println!("[liq_capture] subscribing Liquidation...");
    match timeout(Duration::from_secs(10), ws.subscribe(sub)).await {
        Ok(Ok(())) => println!("[liq_capture] subscribed OK"),
        Ok(Err(e)) => {
            println!("[liq_capture] SUBSCRIBE_FAIL: {}", e);
            return;
        }
        Err(_) => {
            println!("[liq_capture] SUBSCRIBE_TIMEOUT (10s)");
            return;
        }
    }

    // Collect events for capture_dur.
    println!("[liq_capture] collecting for {}s...", args.duration_secs);
    let mut stream = ws.event_stream();
    let mut event_count: u64 = 0;
    let mut first_event_ms: Option<u64> = None;
    let mut last_event_ms: Option<u64> = None;
    let mut sample_event: Option<String> = None;
    let start = Instant::now();

    loop {
        let remaining = capture_dur.saturating_sub(start.elapsed());
        if remaining.is_zero() {
            break;
        }
        match timeout(remaining, stream.next()).await {
            Ok(Some(Ok(event))) => {
                let ts_now = now_ms();
                if let StreamEvent::Liquidation { symbol, price, quantity, side, timestamp, .. } = &event {
                    event_count += 1;
                    if first_event_ms.is_none() {
                        first_event_ms = Some(ts_now);
                    }
                    last_event_ms = Some(ts_now);
                    if sample_event.is_none() {
                        sample_event = Some(format!(
                            "Liquidation {{ sym={} side={:?} px={:.4} qty={:.6} ts={} }}",
                            symbol, side, price, quantity, timestamp
                        ));
                    }
                    if event_count % 10 == 1 {
                        println!(
                            "[liq_capture] event #{}: sym={} side={:?} px={:.4} qty={:.6}",
                            event_count, symbol, side, price, quantity
                        );
                    }
                }
                // Non-liquidation events: ignore (routing noise on shared WS)
            }
            Ok(Some(Err(e))) => {
                println!("[liq_capture] stream error: {}", e);
                break;
            }
            Ok(None) => {
                println!("[liq_capture] stream ended (None)");
                break;
            }
            Err(_) => {
                // timeout = capture window elapsed
                break;
            }
        }
    }

    // Summary.
    println!("=== SUMMARY: {} ===", exchange_name);
    println!("  duration_requested: {}s", args.duration_secs);
    println!("  actual_elapsed:     {:.1}s", start.elapsed().as_secs_f64());
    println!("  liquidation_events: {}", event_count);
    println!("  first_event_at_ms:  {:?}", first_event_ms);
    println!("  last_event_at_ms:   {:?}", last_event_ms);
    println!("  sample_event:       {:?}", sample_event);
    if event_count == 0 {
        println!("  verdict: ZERO_EVENTS — check trace file for subscribe ACK / error frames");
    } else {
        println!("  verdict: OK — feed is live");
    }
}
