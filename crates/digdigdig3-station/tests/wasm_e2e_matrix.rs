//! # wasm_e2e_matrix — full WS coverage matrix in headless Chrome
//!
//! Mirrors `examples/e2e_smoke.rs` for the wasm32 target: subscribes every
//! crypto venue available on wasm × every public WS stream kind and reports an
//! OK / SILENT / ERR / unsupported matrix to the Chrome console.
//!
//! ## Concurrency model
//!
//! Mirrors native `e2e_smoke.rs` (`tokio::spawn` + `join_all`):
//! - Each channel probe is SELF-CONTAINED: own `ExchangeHub`, own WS connection,
//!   own `event_stream()`. No shared handles between concurrent probes.
//! - All channels of one venue run CONCURRENTLY via `futures_util::join!`.
//! - Venues run in concurrent CHUNKS of 7 (3 waves for 21 venues). Chunk size 7
//!   chosen to cap simultaneous WS connections at ~63 (7 venues × 9 channels),
//!   staying well under Chrome's per-host limit (~256) and avoiding exchange
//!   rate-limit storms from a single burst of 189 simultaneous connections.
//!
//! ## Single test: `wasm_ws_matrix_all`
//!
//! Replaces the 6 sequential group tests (A-F). Runs all 21 venues, logs a
//! full matrix, reports TRUSTED count (all 4 core streams OK).
//!
//! ## Run
//!
//!   ```sh
//!   WASM_BINDGEN_TEST_TIMEOUT=1800 \
//!   cargo test --target wasm32-unknown-unknown -p digdigdig3-station \
//!       --test wasm_e2e_matrix -- --nocapture
//!   ```

#![cfg(target_arch = "wasm32")]

use std::time::Duration;

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{AccountType, ExchangeId, StreamType, SubscriptionRequest, Symbol};
use futures_util::StreamExt;

// ─── Cell tag ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
enum Cell {
    Ok,
    Silent,
    Err(String),
    Unsupported,
    Skipped,
}

impl Cell {
    fn label(&self) -> &'static str {
        match self {
            Cell::Ok => "OK  ",
            Cell::Silent => "SLNT",
            Cell::Err(_) => "ERR ",
            Cell::Unsupported => "--  ",
            Cell::Skipped => "SKIP",
        }
    }
    fn is_data_ok(&self) -> bool { matches!(self, Cell::Ok) }
}

// ─── Per-venue result row ─────────────────────────────────────────────────────

struct VenueRow {
    name: &'static str,
    ticker: Cell,
    trade: Cell,
    orderbook: Cell,
    kline: Cell,
    mark_price: Cell,
    funding_rate: Cell,
    open_interest: Cell,
    agg_trade: Cell,
    liquidation: Cell,
}

impl VenueRow {
    fn trade_ob_ok(&self) -> bool {
        self.trade.is_data_ok() && self.orderbook.is_data_ok()
    }
    fn console_line(&self) -> String {
        format!(
            "{:<18} | tick={} trad={} ob={} klin={} mark={} fund={} OI={} agg={} liq={}",
            self.name,
            self.ticker.label(), self.trade.label(), self.orderbook.label(), self.kline.label(),
            self.mark_price.label(), self.funding_rate.label(), self.open_interest.label(),
            self.agg_trade.label(), self.liquidation.label(),
        )
    }
}

// ─── Symbol helper (mirrors e2e_smoke raw_symbol_for) ────────────────────────

fn venue_symbols(id: ExchangeId) -> (Symbol, Symbol, AccountType) {
    let btc_usdt_spot = Symbol::with_raw("BTC", "USDT", "BTCUSDT".to_string());
    let btc_usdt_fut = Symbol::with_raw("BTC", "USDT", "BTCUSDT".to_string());
    let btc_usd_fut = Symbol::with_raw("BTC", "USD", "BTC-USD".to_string());
    match id {
        ExchangeId::Binance => (btc_usdt_spot, btc_usdt_fut, AccountType::FuturesCross),
        ExchangeId::Bybit => (btc_usdt_spot, btc_usdt_fut, AccountType::FuturesCross),
        ExchangeId::OKX => {
            let spot = Symbol::with_raw("BTC", "USDT", "BTC-USDT".to_string());
            let fut = Symbol::with_raw("BTC", "USDT", "BTC-USDT-SWAP".to_string());
            (spot, fut, AccountType::FuturesCross)
        }
        ExchangeId::HyperLiquid => {
            let sym = Symbol::with_raw("BTC", "USD", "BTC".to_string());
            (sym.clone(), sym, AccountType::FuturesCross)
        }
        ExchangeId::Gemini => {
            let spot = Symbol::with_raw("BTC", "USD", "BTCUSD".to_string());
            (spot.clone(), spot, AccountType::Spot)
        }
        ExchangeId::CryptoCom => (btc_usdt_spot, btc_usdt_fut, AccountType::FuturesCross),
        ExchangeId::Bitfinex => {
            let spot = Symbol::with_raw("BTC", "USD", "tBTCUSD".to_string());
            (spot.clone(), spot, AccountType::Spot)
        }
        ExchangeId::BingX => {
            let spot = Symbol::with_raw("BTC", "USDT", "BTC-USDT".to_string());
            let fut = Symbol::with_raw("BTC", "USDT", "BTC-USDT".to_string());
            (spot, fut, AccountType::FuturesCross)
        }
        ExchangeId::Upbit => {
            let spot = Symbol::with_raw("BTC", "KRW", "KRW-BTC".to_string());
            (spot.clone(), spot, AccountType::Spot)
        }
        ExchangeId::Dydx => {
            let fut = Symbol::with_raw("BTC", "USD", "BTC-USD".to_string());
            (fut.clone(), fut, AccountType::FuturesCross)
        }
        ExchangeId::Lighter => (btc_usd_fut.clone(), btc_usd_fut, AccountType::FuturesCross),
        ExchangeId::Kraken => {
            // Kraken WS v2 requires BASE/QUOTE slash format (NOT REST XBTUSD).
            let spot = Symbol::with_raw("BTC", "USD", "BTC/USD".to_string());
            (spot.clone(), spot, AccountType::Spot)
        }
        ExchangeId::KuCoin => {
            let spot = Symbol::with_raw("BTC", "USDT", "BTC-USDT".to_string());
            let fut = Symbol::with_raw("BTC", "USDT", "XBTUSDTM".to_string());
            (spot, fut, AccountType::FuturesCross)
        }
        ExchangeId::GateIO => {
            let spot = Symbol::with_raw("BTC", "USDT", "BTC_USDT".to_string());
            let fut = Symbol::with_raw("BTC", "USDT", "BTC_USDT".to_string());
            (spot, fut, AccountType::FuturesCross)
        }
        ExchangeId::HTX => {
            let spot = Symbol::with_raw("BTC", "USDT", "btcusdt".to_string());
            let fut = Symbol::with_raw("BTC", "USDT", "BTC-USDT".to_string());
            (spot, fut, AccountType::FuturesCross)
        }
        ExchangeId::Deribit => {
            let fut = Symbol::with_raw("BTC", "USD", "BTC-PERPETUAL".to_string());
            (fut.clone(), fut, AccountType::FuturesCross)
        }
        ExchangeId::MEXC => {
            let spot = Symbol::with_raw("BTC", "USDT", "BTCUSDT".to_string());
            let fut = Symbol::with_raw("BTC", "USDT", "BTC_USDT".to_string());
            (spot, fut, AccountType::FuturesCross)
        }
        ExchangeId::Bitget => {
            let spot = Symbol::with_raw("BTC", "USDT", "BTCUSDT".to_string());
            let fut = Symbol::with_raw("BTC", "USDT", "BTCUSDT".to_string());
            (spot, fut, AccountType::FuturesCross)
        }
        ExchangeId::Bitstamp => {
            let spot = Symbol::with_raw("BTC", "USD", "btcusd".to_string());
            (spot.clone(), spot, AccountType::Spot)
        }
        ExchangeId::Coinbase => {
            let spot = Symbol::with_raw("BTC", "USD", "BTC-USD".to_string());
            (spot.clone(), spot, AccountType::Spot)
        }
        ExchangeId::Bitmex => {
            let fut = Symbol::with_raw("BTC", "USD", "XBTUSD".to_string());
            (fut.clone(), fut, AccountType::FuturesCross)
        }
        _ => (btc_usdt_spot, btc_usdt_fut, AccountType::Spot),
    }
}

fn venue_name(id: ExchangeId) -> &'static str {
    match id {
        ExchangeId::Binance => "Binance",
        ExchangeId::Bybit => "Bybit",
        ExchangeId::OKX => "OKX",
        ExchangeId::HyperLiquid => "HyperLiquid",
        ExchangeId::Gemini => "Gemini",
        ExchangeId::CryptoCom => "CryptoCom",
        ExchangeId::Bitfinex => "Bitfinex",
        ExchangeId::BingX => "BingX",
        ExchangeId::Upbit => "Upbit",
        ExchangeId::Dydx => "Dydx",
        ExchangeId::Lighter => "Lighter",
        ExchangeId::Kraken => "Kraken",
        ExchangeId::KuCoin => "KuCoin",
        ExchangeId::GateIO => "GateIO",
        ExchangeId::HTX => "HTX",
        ExchangeId::Deribit => "Deribit",
        ExchangeId::MEXC => "MEXC",
        ExchangeId::Bitget => "Bitget",
        ExchangeId::Bitstamp => "Bitstamp",
        ExchangeId::Coinbase => "Coinbase",
        ExchangeId::Bitmex => "Bitmex",
        _ => "Unknown",
    }
}

/// Returns true for venues that have no public futures WS channels.
/// These get Cell::Skipped for all futures-side probes without connecting.
fn is_spot_only(id: ExchangeId) -> bool {
    matches!(
        id,
        ExchangeId::Gemini
            | ExchangeId::Upbit
            | ExchangeId::Bitstamp
            | ExchangeId::Coinbase
            | ExchangeId::Kraken
            | ExchangeId::Bitfinex
    )
}

fn truncate(s: &str, n: usize) -> String {
    match s.char_indices().nth(n) {
        Some((i, _)) => format!("{}…", &s[..i]),
        None => s.to_string(),
    }
}

// ─── Self-contained channel probe ────────────────────────────────────────────
//
// Creates its OWN ExchangeHub + OWN WS connection for a single stream kind.
// This is the wasm analogue of native `run_ws_sub` (each native stream gets
// its own independent tokio task + connection). Self-contained = safe to run
// N of these concurrently via join_all without contention on a shared handle.

async fn probe_channel(
    id: ExchangeId,
    stream_type: StreamType,
    symbol: Symbol,
    account_type: AccountType,
    window: Duration,
) -> Cell {
    use futures_util::future::{select, Either};
    use futures_util::pin_mut;

    // ── Connect ──
    let hub = ExchangeHub::new();
    {
        let connect_fut = hub.connect_websocket(id, account_type, false);
        let timeout_fut = gloo_timers::future::sleep(Duration::from_secs(10));
        pin_mut!(connect_fut, timeout_fut);
        match select(connect_fut, timeout_fut).await {
            Either::Left((Ok(()), _)) => {}
            Either::Left((Err(e), _)) => {
                let msg = e.to_string();
                if msg.contains("UnsupportedOperation")
                    || msg.contains("not support")
                    || msg.contains("Not supported")
                {
                    return Cell::Unsupported;
                }
                return Cell::Err(format!("connect:{}", truncate(&msg, 40)));
            }
            Either::Right(_) => return Cell::Err("connect_timeout".into()),
        }
    }

    let ws = match hub.ws(id, account_type) {
        Some(ws) => ws,
        None => return Cell::Err("ws_none".into()),
    };

    // ── Subscribe ──
    let sub = SubscriptionRequest {
        symbol,
        stream_type,
        account_type,
        depth: None,
        update_speed_ms: None,
    };
    {
        let sub_fut = ws.subscribe(sub);
        let timeout_fut = gloo_timers::future::sleep(Duration::from_secs(7));
        pin_mut!(sub_fut, timeout_fut);
        match select(sub_fut, timeout_fut).await {
            Either::Left((Ok(()), _)) => {}
            Either::Left((Err(e), _)) => {
                let msg = e.to_string();
                if msg.contains("UnsupportedOperation")
                    || msg.contains("not support")
                    || msg.contains("Not supported")
                {
                    return Cell::Unsupported;
                }
                return Cell::Err(format!("sub:{}", truncate(&msg, 40)));
            }
            Either::Right(_) => return Cell::Err("sub_timeout".into()),
        }
    }

    // ── Collect ──
    let mut stream = ws.event_stream();
    let deadline = gloo_timers::future::sleep(window);
    pin_mut!(deadline);

    loop {
        let next_fut = stream.next();
        pin_mut!(next_fut);
        match select(next_fut, &mut deadline).await {
            Either::Left((Some(Ok(_)), _)) => return Cell::Ok,
            Either::Left((Some(Err(_)), _)) | Either::Left((None, _)) => return Cell::Silent,
            Either::Right(_) => return Cell::Silent,
        }
    }
}

// ─── Per-venue runner ─────────────────────────────────────────────────────────
//
// All channels run CONCURRENTLY — each has its own connection (probe_channel).
// Core (spot) and futures channel sets run as two parallel join groups then
// merged. For spot-only venues, futures cells are set to Skipped immediately
// without opening any connections.

async fn test_venue(id: ExchangeId) -> VenueRow {
    let name = venue_name(id);
    let (spot_sym, fut_sym, fut_at) = venue_symbols(id);
    let spot_at = match id {
        ExchangeId::HyperLiquid | ExchangeId::Dydx | ExchangeId::Lighter => fut_at,
        _ => AccountType::Spot,
    };
    // 8s window for normal channels; 12s for liquidation (sparse by nature).
    let w = Duration::from_secs(8);
    let liq_w = Duration::from_secs(12);

    // Binance liquidation uses empty symbol — exchange broadcasts all symbols
    // on one channel when subscribed with empty symbol.
    let liq_sym = match id {
        ExchangeId::Binance => Symbol::with_raw("", "", "".to_string()),
        _ => fut_sym.clone(),
    };

    if is_spot_only(id) {
        // Only core channels — no futures connections opened.
        let (ticker, trade, orderbook, kline) = futures_util::join!(
            probe_channel(id, StreamType::Ticker, spot_sym.clone(), spot_at, w),
            probe_channel(id, StreamType::Trade, spot_sym.clone(), spot_at, w),
            probe_channel(id, StreamType::Orderbook, spot_sym.clone(), spot_at, w),
            probe_channel(id, StreamType::Kline { interval: "1m".into() }, spot_sym, spot_at, w),
        );
        VenueRow {
            name,
            ticker, trade, orderbook, kline,
            mark_price: Cell::Skipped,
            funding_rate: Cell::Skipped,
            open_interest: Cell::Skipped,
            agg_trade: Cell::Skipped,
            liquidation: Cell::Skipped,
        }
    } else {
        // Core + futures — all 9 channels concurrent across two join groups,
        // then both groups run concurrently via join!.
        let core_fut = futures_util::future::join_all(vec![
            probe_channel(id, StreamType::Ticker, spot_sym.clone(), spot_at, w),
            probe_channel(id, StreamType::Trade, spot_sym.clone(), spot_at, w),
            probe_channel(id, StreamType::Orderbook, spot_sym.clone(), spot_at, w),
            probe_channel(id, StreamType::Kline { interval: "1m".into() }, spot_sym, spot_at, w),
        ]);
        let futs_fut = futures_util::future::join_all(vec![
            probe_channel(id, StreamType::MarkPrice, fut_sym.clone(), fut_at, w),
            probe_channel(id, StreamType::FundingRate, fut_sym.clone(), fut_at, w),
            probe_channel(id, StreamType::OpenInterest, fut_sym.clone(), fut_at, w),
            probe_channel(id, StreamType::AggTrade, fut_sym.clone(), fut_at, w),
            probe_channel(id, StreamType::Liquidation, liq_sym, fut_at, liq_w),
        ]);
        let (mut core_res, mut futs_res) = futures_util::join!(core_fut, futs_fut);
        // drain in order: core[0..4] = ticker/trade/ob/kline, futs[0..5] = mark/fund/oi/agg/liq
        let liquidation = futs_res.pop().unwrap_or(Cell::Err("missing".into()));
        let agg_trade   = futs_res.pop().unwrap_or(Cell::Err("missing".into()));
        let open_interest = futs_res.pop().unwrap_or(Cell::Err("missing".into()));
        let funding_rate  = futs_res.pop().unwrap_or(Cell::Err("missing".into()));
        let mark_price    = futs_res.pop().unwrap_or(Cell::Err("missing".into()));
        let kline     = core_res.pop().unwrap_or(Cell::Err("missing".into()));
        let orderbook = core_res.pop().unwrap_or(Cell::Err("missing".into()));
        let trade     = core_res.pop().unwrap_or(Cell::Err("missing".into()));
        let ticker    = core_res.pop().unwrap_or(Cell::Err("missing".into()));
        VenueRow { name, ticker, trade, orderbook, kline, mark_price, funding_rate, open_interest, agg_trade, liquidation }
    }
}

// ─── Console printer ──────────────────────────────────────────────────────────

fn print_matrix_block(group_name: &str, rows: &[VenueRow]) {
    web_sys::console::log_1(
        &format!("=== WASM WS MATRIX: {} (8s/12s liq window) ===", group_name).into()
    );
    web_sys::console::log_1(
        &"Exchange           | tick  trad  ob    klin  mark  fund  OI    agg   liq".into()
    );
    for row in rows {
        web_sys::console::log_1(&row.console_line().into());
    }
    web_sys::console::log_1(&"".into());
}

// ─── Full concurrent matrix test ─────────────────────────────────────────────
//
// All 21 venues run in CHUNKS of 7 (3 waves). Within each chunk, all venues
// run concurrently. Within each venue, all channels run concurrently.
//
// Chunk size 7 chosen to limit simultaneous WS connections to ~63
// (7 venues × 9 channels max), staying well under Chrome's per-host
// connection limit (~256) and avoiding rate-limit storms from a single
// burst of all 189 connections at once. 3 waves × 7 venues = 21 total.

#[wasm_bindgen_test]
async fn wasm_ws_matrix_all() {
    // 21 venues — Groups A through F combined.
    // Group A: Binance, Bybit, OKX
    // Group B: HyperLiquid, Dydx, Lighter
    // Group C: Gemini, CryptoCom, Bitfinex
    // Group D: BingX, Upbit
    // Group E: Kraken, KuCoin, GateIO, HTX
    // Group F: Deribit, MEXC, Bitget, Bitstamp, Coinbase, Bitmex
    let venues: &[ExchangeId] = &[
        ExchangeId::Binance,
        ExchangeId::Bybit,
        ExchangeId::OKX,
        ExchangeId::HyperLiquid,
        ExchangeId::Dydx,
        ExchangeId::Lighter,
        ExchangeId::Gemini,
        // --- wave 2 ---
        ExchangeId::CryptoCom,
        ExchangeId::Bitfinex,
        ExchangeId::BingX,
        ExchangeId::Upbit,
        ExchangeId::Kraken,
        ExchangeId::KuCoin,
        ExchangeId::GateIO,
        // --- wave 3 ---
        ExchangeId::HTX,
        ExchangeId::Deribit,
        ExchangeId::MEXC,
        ExchangeId::Bitget,
        ExchangeId::Bitstamp,
        ExchangeId::Coinbase,
        ExchangeId::Bitmex,
    ];

    let mut all_rows: Vec<VenueRow> = Vec::with_capacity(venues.len());

    // Run in chunks of 7 — 3 waves for 21 venues.
    // Each chunk is fully concurrent; chunks run sequentially.
    for chunk in venues.chunks(7) {
        let names: Vec<&str> = chunk.iter().map(|id| venue_name(*id)).collect();
        web_sys::console::log_1(
            &format!("[matrix] wave starting: {}", names.join(", ")).into()
        );
        let chunk_rows = futures_util::future::join_all(
            chunk.iter().copied().map(test_venue)
        ).await;
        for row in &chunk_rows {
            web_sys::console::log_1(&row.console_line().into());
        }
        all_rows.extend(chunk_rows);
    }

    print_matrix_block("ALL 21 VENUES", &all_rows);

    // TRUSTED = all 4 core streams OK
    let trusted = all_rows.iter().filter(|r| {
        r.ticker.is_data_ok()
            && r.trade.is_data_ok()
            && r.orderbook.is_data_ok()
            && r.kline.is_data_ok()
    }).count();
    web_sys::console::log_1(
        &format!("=== WASM TRUSTED (all 4 core OK): {}/{} ===", trusted, all_rows.len()).into()
    );

    let matrix_str = all_rows.iter()
        .map(|r| r.console_line())
        .collect::<Vec<_>>()
        .join("\n");

    // Majority must flow Trade+OB. Rate-limits and sparse channels make
    // all-green brittle, so threshold is ≥14/21 (66 %).
    let trade_ob = all_rows.iter().filter(|r| r.trade_ob_ok()).count();
    assert!(
        trade_ob >= 14,
        "expected ≥14/21 venues Trade+OB in wasm; got {}\n{}",
        trade_ob,
        matrix_str
    );
}
