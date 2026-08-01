//! # zero_trade_probe — catch public trade events that carry a zero price.
//!
//! Live-caught 2026-08-01 by the mylittlechart stand: binance USD-M
//! `BTCUSDT` delivered `Trade` events with `price == 0 && quantity == 0` at
//! roughly 10/min, mixed into an otherwise healthy stream. A consumer that
//! folds ticks into a candle takes the zero as a real print, so the bar's
//! `low` collapses to zero permanently.
//!
//! This probe subscribes the raw WS trade stream (no Station, no cache) and
//! prints every event whose price or quantity is not strictly positive,
//! alongside a running total, so the defect is provable at the connector
//! boundary rather than inferred from a downstream chart.
//!
//! Usage:
//! ```
//! cargo run --example zero_trade_probe --release -- --duration 180
//! cargo run --example zero_trade_probe --release -- --stream agg --spot
//! ```
//! Flags: `--duration <secs>` (default 120), `--symbol <BASE-QUOTE>`
//! (default BTC-USDT), `--stream trade|agg` (default trade), `--spot`
//! (default is USD-M futures).

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{
    AccountType, ExchangeId, StreamEvent, StreamType, SubscriptionRequest, Symbol,
};
use futures_util::StreamExt;
use tokio::time::{timeout, Duration, Instant};

#[tokio::main]
async fn main() {
    let mut duration_secs = 120u64;
    let mut symbol_arg = "BTC-USDT".to_string();
    let mut agg = false;
    let mut spot = false;

    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--duration" => {
                if let Some(v) = args.get(i + 1).and_then(|s| s.parse().ok()) {
                    duration_secs = v;
                }
                i += 2;
            }
            "--symbol" => {
                if let Some(v) = args.get(i + 1) {
                    symbol_arg = v.clone();
                }
                i += 2;
            }
            "--stream" => {
                agg = args.get(i + 1).map(|s| s == "agg").unwrap_or(false);
                i += 2;
            }
            "--spot" => {
                spot = true;
                i += 1;
            }
            _ => i += 1,
        }
    }

    let account = if spot { AccountType::Spot } else { AccountType::FuturesCross };
    let (base, quote) = symbol_arg.split_once('-').unwrap_or(("BTC", "USDT"));
    let sym = Symbol::new(base, quote);
    let stream_type = if agg { StreamType::AggTrade } else { StreamType::Trade };

    println!(
        "zero_trade_probe: binance {:?} {} stream={:?} for {}s",
        account, symbol_arg, stream_type, duration_secs
    );

    let hub = ExchangeHub::new();
    if let Err(e) = hub.connect_websocket(ExchangeId::Binance, account, false).await {
        eprintln!("connect_websocket failed: {e}");
        return;
    }
    let Some(ws) = hub.ws(ExchangeId::Binance, account) else {
        eprintln!("no ws handle after connect");
        return;
    };
    if let Err(e) = ws.connect(account).await {
        eprintln!("ws.connect failed: {e}");
        return;
    }

    let sub = SubscriptionRequest {
        symbol: sym.clone(),
        stream_type,
        account_type: account,
        depth: None,
        update_speed_ms: None,
    };
    if let Err(e) = ws.subscribe(sub).await {
        eprintln!("subscribe failed: {e}");
        return;
    }
    println!("subscribed — collecting…");

    let mut stream = ws.event_stream();
    let deadline = Duration::from_secs(duration_secs);
    let start = Instant::now();
    let (mut total, mut zeros) = (0u64, 0u64);

    loop {
        let remaining = deadline.saturating_sub(start.elapsed());
        if remaining.is_zero() {
            break;
        }
        let Ok(Some(Ok(ev))) = timeout(remaining, stream.next()).await else {
            continue;
        };
        let bad = match &ev {
            StreamEvent::Trade { symbol, trade } => {
                total += 1;
                (!(trade.price > 0.0) || !(trade.quantity > 0.0))
                    .then(|| format!("Trade sym={symbol} {trade:?}"))
            }
            StreamEvent::AggTrade { symbol, agg } => {
                total += 1;
                (!(agg.price > 0.0) || !(agg.quantity > 0.0))
                    .then(|| format!("AggTrade sym={symbol} {agg:?}"))
            }
            _ => None,
        };
        if let Some(detail) = bad {
            zeros += 1;
            println!("[ZERO #{zeros} @ {:.1}s] {detail}", start.elapsed().as_secs_f64());
        }
    }

    println!(
        "\nRESULT: {total} trade events, {zeros} with non-positive price/qty ({:.2}%) over {}s",
        if total == 0 { 0.0 } else { zeros as f64 * 100.0 / total as f64 },
        duration_secs,
    );
    if zeros > 0 {
        println!("VERDICT: connector emits zero-valued public trades — parser/dispatch defect.");
    } else {
        println!("VERDICT: clean — every event carried a positive price and quantity.");
    }
}
