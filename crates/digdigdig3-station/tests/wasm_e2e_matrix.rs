//! # wasm_e2e_matrix — per-venue WS job-model harness for headless Chrome
//!
//! Mirrors `examples/e2e_smoke.rs` for the wasm32 target: subscribes every
//! crypto venue available on wasm × every public WS stream kind and reports an
//! OK / SILENT / ERR / unsupported matrix.
//!
//! ## Runner constraints (cite: `docs/research/dig2-wasm-test-capabilities.md`)
//!
//! - **SEAM(GAP-1)**: `cargo test -- <filter>` is ignored by dig2-wasm-test — all
//!   tests always run. Venue selection therefore uses `option_env!("DIG3_WASM_VENUES")`
//!   (compile-time, comma-separated). Example:
//!   `DIG3_WASM_VENUES=binance,bybit cargo test ...` compiles a binary that only
//!   probes those two venues. When GAP-1 lands (test-name filter forwarding in
//!   dig2browser), switch to selecting per-venue `#[wasm_bindgen_test]` functions
//!   at the CLI and delete the `DIG3_WASM_VENUES` env logic.
//!
//! - **SEAM(GAP-2)**: `web_sys::console::log_1(...)` output goes to `#console_log`
//!   which the runner NEVER reads. Results are therefore surfaced via `panic!` so
//!   the failure message lands in `#output` (which the runner DOES print). When
//!   GAP-2 lands (console capture in dig2browser), switch `ws_matrix` to print
//!   via `console::log_1` and remove the `panic!` fallback.
//!
//! ## Concurrency model
//!
//! Mirrors native `e2e_smoke.rs` (`tokio::spawn` + `join_all`):
//! - Each channel probe is SELF-CONTAINED: own `ExchangeHub`, own WS connection,
//!   own `event_stream()`. No shared handles between concurrent probes.
//! - All channels of one venue run CONCURRENTLY via `futures_util::join_all`.
//! - Venues run in concurrent CHUNKS of 7 (3 waves for 21 venues). Chunk size 7
//!   chosen to cap simultaneous WS connections at ~63 (7 venues × 9 channels),
//!   staying well under Chrome's per-host limit (~256) and avoiding exchange
//!   rate-limit storms from a single burst of 189 simultaneous connections.
//!
//! ## Test functions
//!
//! - `ws_matrix` — aggregate test usable NOW. Reads `DIG3_WASM_VENUES` for
//!   subset selection; falls back to all 21 venues. Runs venues in parallel
//!   via `join_all`. Always surfaces the full matrix through `#output` via the
//!   GAP-2 workaround (panic on non-green).
//!
//! - `ws_<venue>` (21 functions) — per-venue tests targeting the future GAP-1
//!   world where `cargo test -- binance` selects only `ws_binance`. Currently
//!   all 21 run sequentially (~21 × venue_budget ≈ 21 min total if none filter).
//!   Each panics with its row's `console_line()` on a non-OK result so the
//!   failure message is visible in `#output` today.
//!
//! ## Run (aggregate, all venues)
//!
//! ```sh
//! WASM_BINDGEN_TEST_TIMEOUT=1800 \
//! cargo test --target wasm32-unknown-unknown -p digdigdig3-station \
//!     --test wasm_e2e_matrix -- --nocapture
//! ```
//!
//! ## Run (subset — compile-time selection via SEAM(GAP-1) workaround)
//!
//! ```sh
//! WASM_BINDGEN_TEST_TIMEOUT=600 \
//! DIG3_WASM_VENUES=binance,bybit,okx \
//! cargo test --target wasm32-unknown-unknown -p digdigdig3-station \
//!     --test wasm_e2e_matrix ws_matrix -- --nocapture
//! ```

#![cfg(target_arch = "wasm32")]

use std::time::Duration;

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{AccountType, ExchangeId, StreamType, SubscriptionRequest, Symbol};
use futures_util::StreamExt;

// ─── Window budgets (mirrors native e2e_smoke.rs budgets) ────────────────────
//
// Native source: docs/research/native-e2e-job-model.md §2 "WS budget per stream type"
//
// Ticker: 60s — aggregate/low-freq channels like dYdX v4_markets push ~once/min.
// Liquidation: 60s — sparse by nature (Bybit: 5-symbol parallel × 60s in native).
// All others: 30s — matching native 30s budget.

const WINDOW_TICKER_SECS: u64 = 60;
const WINDOW_LIQUIDATION_SECS: u64 = 60;
const WINDOW_DEFAULT_SECS: u64 = 30;

// Connect / subscribe timeouts (unchanged from previous harness — conservative).
const CONNECT_TIMEOUT_SECS: u64 = 10;
const SUBSCRIBE_TIMEOUT_SECS: u64 = 7;

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
    fn is_data_ok(&self) -> bool {
        matches!(self, Cell::Ok)
    }
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
    fn all_core_ok(&self) -> bool {
        self.ticker.is_data_ok()
            && self.trade.is_data_ok()
            && self.orderbook.is_data_ok()
            && self.kline.is_data_ok()
    }
    fn console_line(&self) -> String {
        format!(
            "{:<18} | tick={} trad={} ob={} klin={} mark={} fund={} OI={} agg={} liq={}",
            self.name,
            self.ticker.label(),
            self.trade.label(),
            self.orderbook.label(),
            self.kline.label(),
            self.mark_price.label(),
            self.funding_rate.label(),
            self.open_interest.label(),
            self.agg_trade.label(),
            self.liquidation.label(),
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

// ─── Parse DIG3_WASM_VENUES compile-time env into ExchangeId list ────────────
//
// SEAM(GAP-1): Venue selection at compile time via env var until dig2browser
// ships test-name filter forwarding. Once GAP-1 lands, replace with CLI filter.

fn parse_venue_env(raw: &str) -> Vec<ExchangeId> {
    raw.split(',')
        .filter_map(|s| {
            let name = s.trim().to_lowercase();
            match name.as_str() {
                "binance" => Some(ExchangeId::Binance),
                "bybit" => Some(ExchangeId::Bybit),
                "okx" => Some(ExchangeId::OKX),
                "hyperliquid" => Some(ExchangeId::HyperLiquid),
                "gemini" => Some(ExchangeId::Gemini),
                "cryptocom" | "crypto_com" | "crypto.com" => Some(ExchangeId::CryptoCom),
                "bitfinex" => Some(ExchangeId::Bitfinex),
                "bingx" => Some(ExchangeId::BingX),
                "upbit" => Some(ExchangeId::Upbit),
                "dydx" => Some(ExchangeId::Dydx),
                "lighter" => Some(ExchangeId::Lighter),
                "kraken" => Some(ExchangeId::Kraken),
                "kucoin" => Some(ExchangeId::KuCoin),
                "gateio" | "gate" => Some(ExchangeId::GateIO),
                "htx" | "huobi" => Some(ExchangeId::HTX),
                "deribit" => Some(ExchangeId::Deribit),
                "mexc" => Some(ExchangeId::MEXC),
                "bitget" => Some(ExchangeId::Bitget),
                "bitstamp" => Some(ExchangeId::Bitstamp),
                "coinbase" => Some(ExchangeId::Coinbase),
                "bitmex" => Some(ExchangeId::Bitmex),
                _ => None,
            }
        })
        .collect()
}

fn all_venues() -> Vec<ExchangeId> {
    vec![
        ExchangeId::Binance,
        ExchangeId::Bybit,
        ExchangeId::OKX,
        ExchangeId::HyperLiquid,
        ExchangeId::Dydx,
        ExchangeId::Lighter,
        ExchangeId::Gemini,
        ExchangeId::CryptoCom,
        ExchangeId::Bitfinex,
        ExchangeId::BingX,
        ExchangeId::Upbit,
        ExchangeId::Kraken,
        ExchangeId::KuCoin,
        ExchangeId::GateIO,
        ExchangeId::HTX,
        ExchangeId::Deribit,
        ExchangeId::MEXC,
        ExchangeId::Bitget,
        ExchangeId::Bitstamp,
        ExchangeId::Coinbase,
        ExchangeId::Bitmex,
    ]
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
        let timeout_fut =
            gloo_timers::future::sleep(Duration::from_secs(CONNECT_TIMEOUT_SECS));
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
        let timeout_fut =
            gloo_timers::future::sleep(Duration::from_secs(SUBSCRIBE_TIMEOUT_SECS));
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
            Either::Left((Some(Err(_)), _)) | Either::Left((None, _)) => {
                return Cell::Silent
            }
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

    let w_ticker = Duration::from_secs(WINDOW_TICKER_SECS);
    let w = Duration::from_secs(WINDOW_DEFAULT_SECS);
    let liq_w = Duration::from_secs(WINDOW_LIQUIDATION_SECS);

    // Binance liquidation uses empty symbol — exchange broadcasts all symbols
    // on one channel when subscribed with empty symbol.
    let liq_sym = match id {
        ExchangeId::Binance => Symbol::with_raw("", "", "".to_string()),
        _ => fut_sym.clone(),
    };

    if is_spot_only(id) {
        // Only core channels — no futures connections opened.
        let (ticker, trade, orderbook, kline) = futures_util::join!(
            probe_channel(
                id,
                StreamType::Ticker,
                spot_sym.clone(),
                spot_at,
                w_ticker
            ),
            probe_channel(id, StreamType::Trade, spot_sym.clone(), spot_at, w),
            probe_channel(id, StreamType::Orderbook, spot_sym.clone(), spot_at, w),
            probe_channel(
                id,
                StreamType::Kline { interval: "1m".into() },
                spot_sym,
                spot_at,
                w
            ),
        );
        VenueRow {
            name,
            ticker,
            trade,
            orderbook,
            kline,
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
            probe_channel(
                id,
                StreamType::Ticker,
                spot_sym.clone(),
                spot_at,
                w_ticker,
            ),
            probe_channel(id, StreamType::Trade, spot_sym.clone(), spot_at, w),
            probe_channel(id, StreamType::Orderbook, spot_sym.clone(), spot_at, w),
            probe_channel(
                id,
                StreamType::Kline { interval: "1m".into() },
                spot_sym,
                spot_at,
                w,
            ),
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
        let agg_trade = futs_res.pop().unwrap_or(Cell::Err("missing".into()));
        let open_interest = futs_res.pop().unwrap_or(Cell::Err("missing".into()));
        let funding_rate = futs_res.pop().unwrap_or(Cell::Err("missing".into()));
        let mark_price = futs_res.pop().unwrap_or(Cell::Err("missing".into()));
        let kline = core_res.pop().unwrap_or(Cell::Err("missing".into()));
        let orderbook = core_res.pop().unwrap_or(Cell::Err("missing".into()));
        let trade = core_res.pop().unwrap_or(Cell::Err("missing".into()));
        let ticker = core_res.pop().unwrap_or(Cell::Err("missing".into()));
        VenueRow {
            name,
            ticker,
            trade,
            orderbook,
            kline,
            mark_price,
            funding_rate,
            open_interest,
            agg_trade,
            liquidation,
        }
    }
}

// ─── Matrix builder ───────────────────────────────────────────────────────────

fn build_matrix_string(rows: &[VenueRow]) -> String {
    let header =
        "Exchange           | tick  trad  ob    klin  mark  fund  OI    agg   liq";
    let body: Vec<String> = rows.iter().map(|r| r.console_line()).collect();
    format!("{}\n{}", header, body.join("\n"))
}

// ─── Aggregate test: ws_matrix ────────────────────────────────────────────────
//
// Runs all selected venues in parallel (chunks of 7) and surfaces the full
// matrix. SEAM(GAP-1): reads DIG3_WASM_VENUES at compile time for subset
// selection. SEAM(GAP-2): panics with full matrix on non-green so the runner
// captures it in #output.

#[wasm_bindgen_test]
async fn ws_matrix() {
    // SEAM(GAP-1): compile-time venue selection until dig2browser GAP-1 lands.
    let venues: Vec<ExchangeId> = match option_env!("DIG3_WASM_VENUES") {
        Some(raw) if !raw.trim().is_empty() => {
            let parsed = parse_venue_env(raw);
            if parsed.is_empty() {
                all_venues()
            } else {
                parsed
            }
        }
        _ => all_venues(),
    };

    let total = venues.len();
    let mut all_rows: Vec<VenueRow> = Vec::with_capacity(total);

    // Run in chunks of 7 — keeps simultaneous WS connections at ~63 max
    // (7 venues × 9 channels), well under Chrome's per-host limit (~256).
    for chunk in venues.chunks(7) {
        let names: Vec<&str> = chunk.iter().map(|id| venue_name(*id)).collect();
        // SEAM(GAP-2): console.log is lost — this line is informational only,
        // not visible to the runner. The matrix panic below is what surfaces.
        web_sys::console::log_1(
            &format!("[ws_matrix] wave starting: {}", names.join(", ")).into(),
        );
        let chunk_rows =
            futures_util::future::join_all(chunk.iter().copied().map(test_venue))
                .await;
        for row in &chunk_rows {
            web_sys::console::log_1(&row.console_line().into());
        }
        all_rows.extend(chunk_rows);
    }

    let matrix_str = build_matrix_string(&all_rows);

    let trusted = all_rows.iter().filter(|r| r.all_core_ok()).count();
    let trade_ob = all_rows.iter().filter(|r| r.trade_ob_ok()).count();

    web_sys::console::log_1(
        &format!(
            "=== WASM WS MATRIX ({} venues, {}s/{}s/{}s windows) ===\n{}\nTRUSTED(all-core): {}/{} | Trade+OB: {}/{}",
            total,
            WINDOW_TICKER_SECS,
            WINDOW_DEFAULT_SECS,
            WINDOW_LIQUIDATION_SECS,
            matrix_str,
            trusted,
            total,
            trade_ob,
            total,
        )
        .into(),
    );

    // SEAM(GAP-2): runner reads #output but not #console_log. Always embed
    // the full matrix in the panic message so a human reading runner stdout
    // can see per-venue results. On a perfectly green run (≥ threshold) we
    // still assert but don't need to panic with the matrix.
    //
    // "Perfect" threshold: ≥ 66 % trade+OB (same as old harness). When
    // GAP-2 lands, remove the panic-path and print via console::log_1 only.
    let threshold = total * 2 / 3; // ≥ 66 %
    assert!(
        trade_ob >= threshold,
        "WASM WS MATRIX: expected ≥{}/{} venues Trade+OB; got {}/{}\n\n{}\n\nTRUSTED(all-core): {}/{}",
        threshold,
        total,
        trade_ob,
        total,
        matrix_str,
        trusted,
        total,
    );
}

// ─── Per-venue test functions (GAP-1 target) ──────────────────────────────────
//
// One #[wasm_bindgen_test] per venue. These exist so that once dig2browser
// ships GAP-1 (test-name filter forwarding), `cargo test -- ws_binance` will
// run only Binance. Today all 21 run sequentially (tests run strictly
// sequentially in the JS event loop — wasm is single-threaded).
//
// Each function panics with the venue's console_line() on a non-OK result
// so the failure message is visible in #output. SEAM(GAP-2): when GAP-2 lands,
// switch to console::log_1 and remove the panic.
//
// Naming: `ws_<venue_lowercase>` — matches the GAP-1 filter pattern.

macro_rules! venue_test {
    ($fn_name:ident, $exchange_id:expr) => {
        #[wasm_bindgen_test]
        async fn $fn_name() {
            let row = test_venue($exchange_id).await;
            // SEAM(GAP-2): panic embeds the row so the runner captures it in #output.
            if !row.trade_ob_ok() {
                panic!(
                    "WASM WS {}: {}\n(Trade+OB not OK — see row above)",
                    row.name,
                    row.console_line()
                );
            }
        }
    };
}

venue_test!(ws_binance, ExchangeId::Binance);
venue_test!(ws_bybit, ExchangeId::Bybit);
venue_test!(ws_okx, ExchangeId::OKX);
venue_test!(ws_hyperliquid, ExchangeId::HyperLiquid);
venue_test!(ws_dydx, ExchangeId::Dydx);
venue_test!(ws_lighter, ExchangeId::Lighter);
venue_test!(ws_gemini, ExchangeId::Gemini);
venue_test!(ws_cryptocom, ExchangeId::CryptoCom);
venue_test!(ws_bitfinex, ExchangeId::Bitfinex);
venue_test!(ws_bingx, ExchangeId::BingX);
venue_test!(ws_upbit, ExchangeId::Upbit);
venue_test!(ws_kraken, ExchangeId::Kraken);
venue_test!(ws_kucoin, ExchangeId::KuCoin);
venue_test!(ws_gateio, ExchangeId::GateIO);
venue_test!(ws_htx, ExchangeId::HTX);
venue_test!(ws_deribit, ExchangeId::Deribit);
venue_test!(ws_mexc, ExchangeId::MEXC);
venue_test!(ws_bitget, ExchangeId::Bitget);
venue_test!(ws_bitstamp, ExchangeId::Bitstamp);
venue_test!(ws_coinbase, ExchangeId::Coinbase);
venue_test!(ws_bitmex, ExchangeId::Bitmex);
