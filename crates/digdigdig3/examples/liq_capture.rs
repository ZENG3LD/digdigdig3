//! # liq_capture — multi-exchange liquidation feed validator / long-capture harness.
//!
//! Connects to one or more exchanges via WS, subscribes to the public
//! Liquidation stream, collects events for `--duration <secs>`, then
//! prints a per-exchange summary table.
//!
//! ## Usage
//!
//! ```
//! # Single exchange, BTC-USDT, 5 min (default)
//! cargo run --example liq_capture --release -- --exchanges Binance
//!
//! # All 4 exchanges, all-symbols variants, 60s smoke
//! cargo run --example liq_capture --release -- \
//!     --exchanges Binance,Bybit,OKX,GateIO --all --duration 60
//!
//! # Specific symbol, 2h
//! cargo run --example liq_capture --release -- \
//!     --exchanges Bybit --symbol ETH-USDT --duration 7200
//!
//! # All-symbols, 2h background capture
//! cargo run --example liq_capture --release -- \
//!     --exchanges Binance,Bybit,OKX,GateIO --all --duration 7200
//! ```
//!
//! ## Flags
//!
//! `--exchanges <CSV>`  Comma-separated list: Binance, Bybit, OKX, GateIO, Bitget.
//!                      Default: Binance.
//!
//! `--symbol <SYM>`     Base-quote pair, dash-separated. Default: BTC-USDT.
//!                      Ignored by exchanges that are already all-symbols (OKX).
//!
//! `--all`              All-symbols variant where supported:
//!                        Binance  → `!forceOrder@arr` (market-wide)
//!                        GateIO   → `["!all"]` payload (all contracts)
//!                        OKX      → no-op (already all-instType)
//!                        Bybit    → no-op (per-symbol by API design)
//!
//! `--duration <secs>`  Capture window in seconds. Default: 300. Max: 7200.
//!
//! Log files are written to `liq_capture_<exchange>_<ts>.jsonl` via the
//! DIG3_WS_TRACE mechanism (set per task).

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
    exchanges: Vec<ExchangeId>,
    /// Base-quote symbol, dash-separated, e.g. "BTC-USDT"
    symbol: String,
    all_symbols: bool,
    duration_secs: u64,
}

fn parse_exchange(s: &str) -> Option<ExchangeId> {
    match s {
        "Binance" => Some(ExchangeId::Binance),
        "Bybit" => Some(ExchangeId::Bybit),
        "OKX" => Some(ExchangeId::OKX),
        "GateIO" => Some(ExchangeId::GateIO),
        "Bitget" => Some(ExchangeId::Bitget),
        _ => None,
    }
}

fn parse_args() -> Args {
    let argv: Vec<String> = std::env::args().collect();
    let mut exchange_csv: Option<String> = None;
    let mut symbol = "BTC-USDT".to_string();
    let mut all_symbols = false;
    let mut duration_secs: u64 = 300;
    let mut i = 1usize;
    while i < argv.len() {
        match argv[i].as_str() {
            "--exchanges" | "--exchange" => {
                i += 1;
                if i < argv.len() {
                    exchange_csv = Some(argv[i].clone());
                }
            }
            "--symbol" => {
                i += 1;
                if i < argv.len() {
                    symbol = argv[i].clone();
                }
            }
            "--all" => {
                all_symbols = true;
            }
            "--duration" => {
                i += 1;
                if i < argv.len() {
                    duration_secs = argv[i].parse().unwrap_or(300).min(7200);
                }
            }
            _ => {}
        }
        i += 1;
    }

    let csv = exchange_csv.unwrap_or_else(|| "Binance".to_string());
    let exchanges: Vec<ExchangeId> = csv
        .split(',')
        .filter_map(|s| {
            let s = s.trim();
            let id = parse_exchange(s);
            if id.is_none() {
                eprintln!("[liq_capture] unknown exchange {:?} — skipping", s);
            }
            id
        })
        .collect();

    if exchanges.is_empty() {
        eprintln!("[liq_capture] no valid exchanges. Valid: Binance, Bybit, OKX, GateIO, Bitget");
        std::process::exit(1);
    }

    Args { exchanges, symbol, all_symbols, duration_secs }
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

/// Build the Symbol to subscribe with, taking --all into account.
///
/// - Binance + all_symbols → empty raw ("") → triggers !forceOrder@arr in protocol.rs
/// - GateIO  + all_symbols → raw "!all"     → payload ["!all"] in protocol.rs
/// - OKX                   → always all-symbols (instType=SWAP), symbol ignored
/// - Bybit                 → per-symbol; use whatever the caller specified
/// - Others                → use exchange-native raw from normalizer
fn build_symbol(id: ExchangeId, sym_str: &str, all_symbols: bool) -> Symbol {
    match id {
        ExchangeId::Binance if all_symbols => {
            // Empty raw → stream_name returns "!forceOrder@arr"
            Symbol::with_raw("", "", String::new())
        }
        ExchangeId::GateIO if all_symbols => {
            // GateIO public_liquidates accepts ["!all"] as wildcard payload
            Symbol::with_raw("", "", "!all".to_string())
        }
        ExchangeId::OKX => {
            // OKX liquidation-orders with instType=SWAP is already all-symbols.
            // Symbol field is ignored by the protocol; pass a placeholder.
            Symbol::with_raw("BTC", "USDT", "BTC-USDT-SWAP".to_string())
        }
        _ => {
            // Parse "BASE-QUOTE" → Symbol, then normalise to exchange-native raw.
            let (base, quote) = sym_str
                .split_once('-')
                .unwrap_or(("BTC", "USDT"));
            let canonical = Symbol::new(base, quote);
            let raw = SymbolNormalizer::to_exchange(id, &canonical, AccountType::FuturesCross)
                .unwrap_or_else(|_| format!("{}{}", base, quote));
            Symbol::with_raw(base, quote, raw)
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Per-exchange capture task
// ─────────────────────────────────────────────────────────────────────────────

struct CaptureResult {
    exchange: String,
    event_count: u64,
    elapsed_secs: f64,
    first_event_ms: Option<u64>,
    last_event_ms: Option<u64>,
    sample: Option<String>,
    error: Option<String>,
}

async fn capture_one(
    id: ExchangeId,
    sym: Symbol,
    duration: Duration,
) -> CaptureResult {
    let exchange_name = format!("{:?}", id);

    // Safety: multiple tasks each set a different trace tag.
    // All set_var calls happen before any spawn (in main), so there is no
    // concurrent mutation. Using unsafe is required by the std API.
    // NOTE: trace tag set in main() before spawning — nothing to do here.

    let hub = ExchangeHub::new();
    match timeout(
        Duration::from_secs(15),
        hub.connect_websocket(id, AccountType::FuturesCross, false),
    )
    .await
    {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            return CaptureResult {
                exchange: exchange_name,
                event_count: 0,
                elapsed_secs: 0.0,
                first_event_ms: None,
                last_event_ms: None,
                sample: None,
                error: Some(format!("connect_websocket: {}", e)),
            };
        }
        Err(_) => {
            return CaptureResult {
                exchange: exchange_name,
                event_count: 0,
                elapsed_secs: 0.0,
                first_event_ms: None,
                last_event_ms: None,
                sample: None,
                error: Some("connect_websocket timeout (15s)".to_string()),
            };
        }
    }

    let ws = match hub.ws(id, AccountType::FuturesCross) {
        Some(w) => w,
        None => {
            return CaptureResult {
                exchange: exchange_name,
                event_count: 0,
                elapsed_secs: 0.0,
                first_event_ms: None,
                last_event_ms: None,
                sample: None,
                error: Some("hub.ws() returned None".to_string()),
            }
        }
    };

    match timeout(
        Duration::from_secs(15),
        ws.connect(AccountType::FuturesCross),
    )
    .await
    {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            return CaptureResult {
                exchange: exchange_name,
                event_count: 0,
                elapsed_secs: 0.0,
                first_event_ms: None,
                last_event_ms: None,
                sample: None,
                error: Some(format!("ws.connect: {}", e)),
            };
        }
        Err(_) => {
            return CaptureResult {
                exchange: exchange_name,
                event_count: 0,
                elapsed_secs: 0.0,
                first_event_ms: None,
                last_event_ms: None,
                sample: None,
                error: Some("ws.connect timeout (15s)".to_string()),
            };
        }
    }

    let sub = SubscriptionRequest {
        symbol: sym.clone(),
        stream_type: StreamType::Liquidation,
        account_type: AccountType::FuturesCross,
        depth: None,
        update_speed_ms: None,
    };

    println!(
        "[{}] subscribing Liquidation sym={}...",
        exchange_name,
        sym.raw.as_deref().unwrap_or("(canonical)")
    );

    match timeout(Duration::from_secs(10), ws.subscribe(sub)).await {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            return CaptureResult {
                exchange: exchange_name,
                event_count: 0,
                elapsed_secs: 0.0,
                first_event_ms: None,
                last_event_ms: None,
                sample: None,
                error: Some(format!("subscribe: {}", e)),
            };
        }
        Err(_) => {
            return CaptureResult {
                exchange: exchange_name,
                event_count: 0,
                elapsed_secs: 0.0,
                first_event_ms: None,
                last_event_ms: None,
                sample: None,
                error: Some("subscribe timeout (10s)".to_string()),
            };
        }
    }

    println!("[{}] subscribed OK — collecting for {}s", exchange_name, duration.as_secs());

    let mut stream = ws.event_stream();
    let mut event_count: u64 = 0;
    let mut first_event_ms: Option<u64> = None;
    let mut last_event_ms: Option<u64> = None;
    let mut sample: Option<String> = None;
    let start = Instant::now();

    loop {
        let remaining = duration.saturating_sub(start.elapsed());
        if remaining.is_zero() {
            break;
        }
        match timeout(remaining, stream.next()).await {
            Ok(Some(Ok(event))) => {
                let ts_now = now_ms();
                if let StreamEvent::Liquidation { symbol, liquidation } =
                    &event
                {
                    event_count += 1;
                    if first_event_ms.is_none() {
                        first_event_ms = Some(ts_now);
                    }
                    last_event_ms = Some(ts_now);
                    if sample.is_none() {
                        sample = Some(format!(
                            "Liquidation {{ sym={} side={:?} px={:.4} qty={:.6} ts={} }}",
                            symbol, liquidation.side, liquidation.price, liquidation.quantity, liquidation.timestamp
                        ));
                    }
                    if event_count % 10 == 1 {
                        println!(
                            "[{}] #{}: sym={} side={:?} px={:.4} qty={:.6}",
                            exchange_name, event_count, symbol, liquidation.side, liquidation.price, liquidation.quantity
                        );
                    }
                }
            }
            Ok(Some(Err(e))) => {
                return CaptureResult {
                    exchange: exchange_name,
                    event_count,
                    elapsed_secs: start.elapsed().as_secs_f64(),
                    first_event_ms,
                    last_event_ms,
                    sample,
                    error: Some(format!("stream error: {}", e)),
                };
            }
            Ok(None) => break,
            Err(_) => break, // timeout = window elapsed
        }
    }

    CaptureResult {
        exchange: exchange_name,
        event_count,
        elapsed_secs: start.elapsed().as_secs_f64(),
        first_event_ms,
        last_event_ms,
        sample,
        error: None,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Main
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    let args = parse_args();
    let capture_dur = Duration::from_secs(args.duration_secs);
    let start_ts = now_ms();

    println!(
        "[liq_capture] exchanges={} symbol={} all={} duration={}s",
        args.exchanges
            .iter()
            .map(|id| format!("{:?}", id))
            .collect::<Vec<_>>()
            .join(","),
        args.symbol,
        args.all_symbols,
        args.duration_secs,
    );

    // Set DIG3_WS_TRACE per exchange before spawning tasks.
    // Each exchange gets its own trace file via a unique tag.
    for id in &args.exchanges {
        let tag = format!("liq_capture/{:?}_{}", id, start_ts);
        // Safety: all set_var calls happen here, single-threaded, before any
        // tokio::spawn — no concurrent env access.
        unsafe { std::env::set_var(format!("DIG3_WS_TRACE_{:?}", id), &tag) };
    }
    // Also set the generic trace key (last exchange wins — acceptable for single-exchange runs).
    if let Some(id) = args.exchanges.first() {
        let tag = format!("liq_capture/{:?}_{}", id, start_ts);
        unsafe { std::env::set_var("DIG3_WS_TRACE", &tag) };
    }

    // Spawn one task per exchange.
    let mut handles = Vec::new();
    for id in args.exchanges {
        let sym = build_symbol(id, &args.symbol, args.all_symbols);
        let dur = capture_dur;
        let handle = tokio::spawn(async move { capture_one(id, sym, dur).await });
        handles.push(handle);
    }

    // Wait for all tasks.
    let results: Vec<CaptureResult> = futures_util::future::join_all(handles)
        .await
        .into_iter()
        .filter_map(|r| r.ok())
        .collect();

    // Print summary table.
    println!();
    println!("══════════════════════════════════════════════════════════════════");
    println!("  SUMMARY  (duration_requested={}s)", args.duration_secs);
    println!("══════════════════════════════════════════════════════════════════");
    for r in &results {
        let verdict = if r.error.is_some() {
            "ERROR"
        } else if r.event_count == 0 {
            "ZERO_EVENTS"
        } else {
            "OK"
        };
        println!("  [{:<8}]  events={:<6}  elapsed={:.1}s  verdict={}",
            r.exchange, r.event_count, r.elapsed_secs, verdict);
        if let (Some(first), Some(last)) = (r.first_event_ms, r.last_event_ms) {
            println!("             first_event_ms={}  last_event_ms={}", first, last);
        }
        if let Some(e) = &r.error {
            println!("             error: {}", e);
        }
        if let Some(s) = &r.sample {
            println!("             sample: {}", s);
        }
        if r.event_count == 0 && r.error.is_none() {
            println!("             hint: check trace file for subscribe ACK / silent stream");
        }
    }
    println!("══════════════════════════════════════════════════════════════════");
}
