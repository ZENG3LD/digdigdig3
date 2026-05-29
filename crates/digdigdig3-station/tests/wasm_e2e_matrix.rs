//! # wasm_e2e_matrix — full WS coverage matrix in headless Chrome
//!
//! Mirrors `examples/e2e_smoke.rs` for the wasm32 target: subscribes every
//! crypto venue available on wasm × every public WS stream kind and reports an
//! OK / SILENT / ERR / unsupported matrix to the Chrome console.
//!
//! ## Wasm WS venues (from factory.rs `#[cfg(target_arch = "wasm32")]`)
//!
//! Binance, Bybit, OKX, HyperLiquid (onchain-evm feature),
//! Gemini, CryptoCom, Bitfinex, BingX, Upbit, Dydx, Lighter.
//!
//! ## Stream kinds tested
//!
//! Core public: Ticker, Trade, Orderbook, Kline.
//! Futures-capable exchanges (Groups A/B): additionally MarkPrice, FundingRate,
//! OpenInterest, AggTrade, Liquidation.
//!
//! ## Architecture — single hub per venue
//!
//! To avoid the 9+7+8 = 24s overhead of reconnecting for each stream, each
//! venue test creates ONE hub + ONE WS connection, then subscribes to each
//! stream kind sequentially on that connection. This reduces per-venue cost to
//! ~9s connect + 7s × N subscriptions + 8s × N windows.
//!
//! ## Budget (all 4 tests run in one browser session, 1200s total)
//!
//! Group A (Binance/Bybit/OKX, 9 streams): 3 venues × (9+63+72)s ≈ 432s
//! Group B (HyperLiquid/Dydx/Lighter, 9 streams): 3 venues × ~144s ≈ 432s
//! Group C (Gemini/CryptoCom/Bitfinex, 4 streams): 3 venues × ~65s ≈ 195s
//! Group D (BingX/Upbit, 4 streams): 2 venues × ~65s ≈ 130s
//! Total: ~1189s ≈ 1200s (borderline — set timeout 1800s or run groups individually)
//!
//! ## IMPORTANT
//!
//! To keep total runtime < 1200s, Groups A+B are run separately from C+D.
//! Run Group A+B with a higher timeout or split into individual tests:
//!   WASM_BINDGEN_TEST_TIMEOUT=1800 cargo test ... -- wasm_ws_matrix_group_a
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
use std::sync::Arc;

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{AccountType, ExchangeId, StreamType, SubscriptionRequest, Symbol};
use digdigdig3::core::traits::WebSocketConnector;
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

fn truncate(s: &str, n: usize) -> String {
    match s.char_indices().nth(n) {
        Some((i, _)) => format!("{}…", &s[..i]),
        None => s.to_string(),
    }
}

// ─── One-shot stream probe (reuses existing WS handle) ───────────────────────

/// Subscribe to `stream_type` on `ws`, collect for `window`, return Cell.
///
/// Does NOT reconnect — reuses the handle passed in. Each call subscribes
/// once and reads one event. gloo_timers provides the wasm-safe deadline.
async fn probe_on_ws(
    ws: &Arc<dyn WebSocketConnector>,
    stream_type: StreamType,
    symbol: Symbol,
    account_type: AccountType,
    window: Duration,
) -> Cell {
    use futures_util::future::{select, Either};
    use futures_util::pin_mut;

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

// ─── Connect helper ───────────────────────────────────────────────────────────

/// Connect hub for `id` + `account_type`. Returns Ok(ws) or Err(Cell).
async fn connect_hub(
    id: ExchangeId,
    account_type: AccountType,
) -> Result<Arc<dyn WebSocketConnector>, Cell> {
    use futures_util::future::{select, Either};
    use futures_util::pin_mut;

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
                    return Err(Cell::Unsupported);
                }
                return Err(Cell::Err(format!("connect:{}", truncate(&msg, 40))));
            }
            Either::Right(_) => return Err(Cell::Err("connect_timeout".into())),
        }
    }
    match hub.ws(id, account_type) {
        Some(ws) => Ok(ws),
        None => Err(Cell::Err("ws_none".into())),
    }
}

// ─── Per-venue runner (core streams, one connection) ─────────────────────────

/// Test 4 core streams on one connection: Ticker / Trade / Orderbook / Kline.
/// Connect cost paid once. Total per venue: ~10s connect + 4×(7+8)s = ~70s.
async fn test_venue_core(id: ExchangeId) -> VenueRow {
    let name = venue_name(id);
    let (spot_sym, _, _) = venue_symbols(id);
    let spot_at = match id {
        ExchangeId::HyperLiquid | ExchangeId::Dydx | ExchangeId::Lighter => AccountType::FuturesCross,
        _ => AccountType::Spot,
    };
    let w = Duration::from_secs(8);

    let ws = match connect_hub(id, spot_at).await {
        Ok(ws) => ws,
        Err(cell) => {
            return VenueRow {
                name,
                ticker: cell.clone(), trade: cell.clone(), orderbook: cell.clone(), kline: cell,
                mark_price: Cell::Skipped, funding_rate: Cell::Skipped, open_interest: Cell::Skipped,
                agg_trade: Cell::Skipped, liquidation: Cell::Skipped,
            };
        }
    };

    let ticker = probe_on_ws(&ws, StreamType::Ticker, spot_sym.clone(), spot_at, w).await;
    let trade = probe_on_ws(&ws, StreamType::Trade, spot_sym.clone(), spot_at, w).await;
    let orderbook = probe_on_ws(&ws, StreamType::Orderbook, spot_sym.clone(), spot_at, w).await;
    let kline = probe_on_ws(&ws, StreamType::Kline { interval: "1m".into() }, spot_sym.clone(), spot_at, w).await;

    VenueRow {
        name, ticker, trade, orderbook, kline,
        mark_price: Cell::Skipped, funding_rate: Cell::Skipped, open_interest: Cell::Skipped,
        agg_trade: Cell::Skipped, liquidation: Cell::Skipped,
    }
}

// ─── Per-venue runner (full: core + futures, two connections) ─────────────────

/// Test 9 streams using two connections: one spot, one futures.
/// Spot conn: Ticker/Trade/OB/Kline. Futures conn: Mark/Fund/OI/Agg/Liq.
/// Total per venue: 2×10s connect + 4×15s spot + 5×15s futures ≈ 155s.
async fn test_venue_full(id: ExchangeId) -> VenueRow {
    let name = venue_name(id);
    let (spot_sym, fut_sym, fut_at) = venue_symbols(id);
    let spot_at = match id {
        ExchangeId::HyperLiquid | ExchangeId::Dydx | ExchangeId::Lighter => fut_at,
        _ => AccountType::Spot,
    };
    let w = Duration::from_secs(8);
    let liq_w = Duration::from_secs(12);

    // ── Spot connection ──
    let (ticker, trade, orderbook, kline) = match connect_hub(id, spot_at).await {
        Ok(ws) => {
            let ticker = probe_on_ws(&ws, StreamType::Ticker, spot_sym.clone(), spot_at, w).await;
            let trade = probe_on_ws(&ws, StreamType::Trade, spot_sym.clone(), spot_at, w).await;
            let orderbook = probe_on_ws(&ws, StreamType::Orderbook, spot_sym.clone(), spot_at, w).await;
            let kline = probe_on_ws(&ws, StreamType::Kline { interval: "1m".into() }, spot_sym.clone(), spot_at, w).await;
            (ticker, trade, orderbook, kline)
        }
        Err(cell) => (cell.clone(), cell.clone(), cell.clone(), cell),
    };

    // ── Futures connection ──
    let (mark_price, funding_rate, open_interest, agg_trade, liquidation) =
        match connect_hub(id, fut_at).await {
            Ok(ws) => {
                let mark = probe_on_ws(&ws, StreamType::MarkPrice, fut_sym.clone(), fut_at, w).await;
                let fund = probe_on_ws(&ws, StreamType::FundingRate, fut_sym.clone(), fut_at, w).await;
                let oi = probe_on_ws(&ws, StreamType::OpenInterest, fut_sym.clone(), fut_at, w).await;
                let agg = probe_on_ws(&ws, StreamType::AggTrade, fut_sym.clone(), fut_at, w).await;
                let liq_sym = match id {
                    ExchangeId::Binance => Symbol::with_raw("", "", "".to_string()),
                    _ => fut_sym.clone(),
                };
                let liq = probe_on_ws(&ws, StreamType::Liquidation, liq_sym, fut_at, liq_w).await;
                (mark, fund, oi, agg, liq)
            }
            Err(cell) => (cell.clone(), cell.clone(), cell.clone(), cell.clone(), cell),
        };

    VenueRow { name, ticker, trade, orderbook, kline, mark_price, funding_rate, open_interest, agg_trade, liquidation }
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

// ─── Test Group A: Binance, Bybit, OKX ───────────────────────────────────────
//
// Budget: 3 venues × ~155s = ~465s

/// Wasm WS matrix — Group A: Binance / Bybit / OKX.
///
/// Full futures CEX. Two WS connections per venue: spot+futures.
/// Expected: Ticker+Trade+OB+Kline OK, MarkPrice/FundingRate OK.
/// Liq/OI may be SILENT (sparse in 12s window).
#[wasm_bindgen_test]
async fn wasm_ws_matrix_group_a_binance_bybit_okx() {
    let venues = [ExchangeId::Binance, ExchangeId::Bybit, ExchangeId::OKX];
    let mut rows = Vec::new();
    for id in venues {
        web_sys::console::log_1(&format!("[matrix-A] {}...", venue_name(id)).into());
        rows.push(test_venue_full(id).await);
    }
    print_matrix_block("Group A: Binance/Bybit/OKX", &rows);
    let trade_ob_ok = rows.iter().filter(|r| r.trade_ob_ok()).count();
    assert!(
        trade_ob_ok >= 2,
        "Expected ≥2/3 (Binance/Bybit/OKX) Trade+OB in wasm; got {}/3\n{}",
        trade_ob_ok,
        rows.iter().map(|r| r.console_line()).collect::<Vec<_>>().join("\n")
    );
}

// ─── Test Group B: HyperLiquid, Dydx, Lighter ────────────────────────────────
//
// Budget: 3 venues × ~155s = ~465s

/// Wasm WS matrix — Group B: HyperLiquid / dYdX / Lighter.
///
/// DEX/futures-only. Two connections per venue. ≥1 must deliver Trade+OB.
#[wasm_bindgen_test]
async fn wasm_ws_matrix_group_b_hyperliquid_dydx_lighter() {
    let venues = [ExchangeId::HyperLiquid, ExchangeId::Dydx, ExchangeId::Lighter];
    let mut rows = Vec::new();
    for id in venues {
        web_sys::console::log_1(&format!("[matrix-B] {}...", venue_name(id)).into());
        rows.push(test_venue_full(id).await);
    }
    print_matrix_block("Group B: HyperLiquid/Dydx/Lighter", &rows);
    let trade_ob_ok = rows.iter().filter(|r| r.trade_ob_ok()).count();
    assert!(
        trade_ob_ok >= 1,
        "Expected ≥1/3 DEX venues Trade+OB in wasm; got {}/3\n{}",
        trade_ob_ok,
        rows.iter().map(|r| r.console_line()).collect::<Vec<_>>().join("\n")
    );
}

// ─── Test Group C: Gemini, CryptoCom, Bitfinex ───────────────────────────────
//
// Budget: 3 venues × ~70s = ~210s

/// Wasm WS matrix — Group C: Gemini / CryptoCom / Bitfinex.
///
/// Spot CEX. Core streams only (one connection per venue).
#[wasm_bindgen_test]
async fn wasm_ws_matrix_group_c_gemini_cryptocom_bitfinex() {
    let venues = [ExchangeId::Gemini, ExchangeId::CryptoCom, ExchangeId::Bitfinex];
    let mut rows = Vec::new();
    for id in venues {
        web_sys::console::log_1(&format!("[matrix-C] {}...", venue_name(id)).into());
        rows.push(test_venue_core(id).await);
    }
    print_matrix_block("Group C: Gemini/CryptoCom/Bitfinex", &rows);
    let any = rows.iter().filter(|r| r.trade.is_data_ok() || r.ticker.is_data_ok()).count();
    assert!(
        any >= 1,
        "Expected ≥1/3 (Gemini/CryptoCom/Bitfinex) Trade or Ticker; got {}/3\n{}",
        any,
        rows.iter().map(|r| r.console_line()).collect::<Vec<_>>().join("\n")
    );
}

// ─── Test Group D: BingX, Upbit ──────────────────────────────────────────────
//
// Budget: 2 venues × ~70s = ~140s

/// Wasm WS matrix — Group D: BingX / Upbit.
///
/// BingX: BTC-USDT spot. Upbit: KRW-BTC spot.
/// Core streams only. Last group — prints matrix completion summary.
#[wasm_bindgen_test]
async fn wasm_ws_matrix_group_d_bingx_upbit() {
    let venues = [ExchangeId::BingX, ExchangeId::Upbit];
    let mut rows = Vec::new();
    for id in venues {
        web_sys::console::log_1(&format!("[matrix-D] {}...", venue_name(id)).into());
        rows.push(test_venue_core(id).await);
    }
    print_matrix_block("Group D: BingX/Upbit", &rows);

    web_sys::console::log_1(&"=== WASM E2E MATRIX COMPLETE (mirrors native e2e_smoke) ===".into());
    web_sys::console::log_1(
        &"Venues: Binance/Bybit/OKX (A) + HyperLiquid/Dydx/Lighter (B) + Gemini/CryptoCom/Bitfinex (C) + BingX/Upbit (D) = 11 total".into()
    );
    web_sys::console::log_1(
        &"Streams: Ticker/Trade/Orderbook/Kline (+futures: Mark/Fund/OI/Agg/Liq for Groups A+B)".into()
    );
    web_sys::console::log_1(&"Window: 8s normal / 12s liquidation. SILENT = no events in window.".into());

    let any = rows.iter().filter(|r| r.trade.is_data_ok() || r.ticker.is_data_ok()).count();
    assert!(
        any >= 1,
        "Expected ≥1/2 (BingX/Upbit) Trade or Ticker; got {}/2\n{}",
        any,
        rows.iter().map(|r| r.console_line()).collect::<Vec<_>>().join("\n")
    );
}

// ─── Test Group E: Kraken, KuCoin, GateIO, HTX ───────────────────────────────
//
// Futures-capable CEX (KuCoin/GateIO/HTX) + Kraken spot. Full stream set where
// applicable. Completes parity with native e2e_smoke for these venues.

/// Wasm WS matrix — Group E: Kraken / KuCoin / GateIO / HTX.
#[wasm_bindgen_test]
async fn wasm_ws_matrix_group_e_kraken_kucoin_gateio_htx() {
    let venues = [
        ExchangeId::Kraken,
        ExchangeId::KuCoin,
        ExchangeId::GateIO,
        ExchangeId::HTX,
    ];
    let mut rows = Vec::new();
    for id in venues {
        web_sys::console::log_1(&format!("[matrix-E] {}...", venue_name(id)).into());
        rows.push(test_venue_full(id).await);
    }
    print_matrix_block("Group E: Kraken/KuCoin/GateIO/HTX", &rows);
    let trade_ob_ok = rows.iter().filter(|r| r.trade_ob_ok()).count();
    assert!(
        trade_ob_ok >= 2,
        "Expected ≥2/4 (Kraken/KuCoin/GateIO/HTX) Trade+OB in wasm; got {}/4\n{}",
        trade_ob_ok,
        rows.iter().map(|r| r.console_line()).collect::<Vec<_>>().join("\n")
    );
}

// ─── Test Group F: Deribit, MEXC, Bitget, Bitstamp, Coinbase, Bitmex ─────────
//
// Remaining CEX venues — completes the full crypto roster parity with native.

/// Wasm WS matrix — Group F: Deribit / MEXC / Bitget / Bitstamp / Coinbase / Bitmex.
#[wasm_bindgen_test]
async fn wasm_ws_matrix_group_f_deribit_mexc_bitget_bitstamp_coinbase_bitmex() {
    let venues = [
        ExchangeId::Deribit,
        ExchangeId::MEXC,
        ExchangeId::Bitget,
        ExchangeId::Bitstamp,
        ExchangeId::Coinbase,
        ExchangeId::Bitmex,
    ];
    let mut rows = Vec::new();
    for id in venues {
        web_sys::console::log_1(&format!("[matrix-F] {}...", venue_name(id)).into());
        rows.push(test_venue_full(id).await);
    }
    print_matrix_block("Group F: Deribit/MEXC/Bitget/Bitstamp/Coinbase/Bitmex", &rows);

    web_sys::console::log_1(&"=== WASM E2E MATRIX — FULL CRYPTO ROSTER (21 venues) ===".into());
    web_sys::console::log_1(
        &"A: Binance/Bybit/OKX  B: HyperLiquid/Dydx/Lighter  C: Gemini/CryptoCom/Bitfinex  \
          D: BingX/Upbit  E: Kraken/KuCoin/GateIO/HTX  F: Deribit/MEXC/Bitget/Bitstamp/Coinbase/Bitmex".into()
    );

    let trade_ob_ok = rows.iter().filter(|r| r.trade_ob_ok()).count();
    assert!(
        trade_ob_ok >= 3,
        "Expected ≥3/6 of Group F Trade+OB in wasm; got {}/6\n{}",
        trade_ob_ok,
        rows.iter().map(|r| r.console_line()).collect::<Vec<_>>().join("\n")
    );
}
