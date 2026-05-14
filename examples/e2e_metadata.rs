//! # e2e_metadata — Live trading-metadata smoke test
//!
//! Hits live exchange REST APIs and WebSocket streams to verify
//! the trading-metadata coverage works end-to-end against real servers.
//!
//! Run:
//!   cargo run --example e2e_metadata
//!
//! No API keys required — all tested endpoints are public.

use std::time::Duration;

use futures_util::StreamExt;
use tokio::time::timeout;

use digdigdig3::l3::open::crypto::cex::binance::{BinanceConnector, BinanceWebSocket};
use digdigdig3::l3::open::crypto::cex::bybit::{BybitConnector, BybitWebSocket};
use digdigdig3::l3::open::crypto::cex::okx::{OkxConnector, OkxWebSocket};
use digdigdig3::l3::open::crypto::cex::hyperliquid::{HyperliquidConnector, HyperliquidWebSocket};
use digdigdig3::l3::open::crypto::cex::deribit::{DeribitConnector, DeribitWebSocket};
use digdigdig3::l3::open::crypto::cex::bitget::BitgetConnector;
use digdigdig3::l3::open::crypto::cex::htx::HtxConnector;

use digdigdig3::core::{
    AccountType, Symbol, StreamType, SubscriptionRequest,
};
use digdigdig3::core::traits::WebSocketConnector;

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Pretty-print first ~80 chars of a debug representation.
fn abbrev<T: std::fmt::Debug>(val: &T) -> String {
    let s = format!("{:?}", val);
    if s.len() > 80 {
        format!("{}…", &s[..80])
    } else {
        s
    }
}

/// Print "OK" line with count and first sample.
macro_rules! ok_rest {
    ($method:expr, $vec:expr) => {{
        let n = $vec.len();
        if n > 0 {
            println!("  OK:   {} -> {} items, first: {}", $method, n, abbrev(&$vec[0]));
        } else {
            println!("  OK:   {} -> 0 items (empty but no error)", $method);
        }
        (true, n)
    }};
}

macro_rules! ok_rest_single {
    ($method:expr, $val:expr) => {{
        println!("  OK:   {} -> {}", $method, abbrev(&$val));
        (true, 1usize)
    }};
}

macro_rules! fail_rest {
    ($method:expr, $err:expr) => {{
        println!("  FAIL: {} -> {}", $method, $err);
        (false, 0usize)
    }};
}

// ─── Tally ──────────────────────────────────────────────────────────────────

struct RestTally {
    exchange: String,
    tested: usize,
    passed: usize,
    failed: usize,
}

struct WsTally {
    exchange: String,
    channels: usize,
    subscribed: usize,
    events: usize,
    parse_errors: usize,
    zero_event_channels: Vec<String>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// SECTION A — REST
// ═══════════════════════════════════════════════════════════════════════════════

async fn test_binance_rest() -> RestTally {
    println!("\n── Binance REST ─────────────────────────────────────────────");
    let mut tally = RestTally { exchange: "Binance".into(), tested: 0, passed: 0, failed: 0 };

    let conn = match BinanceConnector::new(None, false).await {
        Ok(c) => c,
        Err(e) => {
            println!("  FAIL: connector init -> {}", e);
            tally.failed += 1;
            tally.tested += 1;
            return tally;
        }
    };

    // get_open_interest
    tally.tested += 1;
    match conn.get_open_interest("BTCUSDT").await {
        Ok(v) => { let (p, _) = ok_rest_single!("get_open_interest(BTCUSDT)", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("get_open_interest(BTCUSDT)", e); tally.failed += 1; }
    }

    // get_premium_index
    tally.tested += 1;
    match conn.get_premium_index(Some("BTCUSDT")).await {
        Ok(v) => { let (p, _) = ok_rest_single!("get_premium_index(BTCUSDT)", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("get_premium_index(BTCUSDT)", e); tally.failed += 1; }
    }

    // get_force_orders — public endpoint (no auth needed for historical data)
    tally.tested += 1;
    match conn.get_force_orders(Some("BTCUSDT"), None, None, None, Some(10)).await {
        Ok(v) => { let (p, _) = ok_rest!("get_force_orders(BTCUSDT)", v); tally.passed += p as usize; }
        Err(e) => {
            let msg = e.to_string();
            // 400 with code -1003 = "too many requests", -2014 = needs key
            if msg.contains("key") || msg.contains("signature") || msg.contains("apiKey") {
                println!("  SKIPPED: get_force_orders -> needs API key");
            } else {
                fail_rest!("get_force_orders(BTCUSDT)", e);
                tally.failed += 1;
            }
        }
    }

    // get_top_long_short_account_ratio
    tally.tested += 1;
    match conn.get_top_long_short_account_ratio("BTCUSDT", "1h", Some(10), None, None).await {
        Ok(v) => { let (p, _) = ok_rest!("get_top_long_short_account_ratio", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("get_top_long_short_account_ratio", e); tally.failed += 1; }
    }

    // get_open_interest_history
    tally.tested += 1;
    match conn.get_open_interest_history("BTCUSDT", "1h", Some(10), None, None).await {
        Ok(v) => { let (p, _) = ok_rest!("get_open_interest_history", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("get_open_interest_history", e); tally.failed += 1; }
    }

    tally
}

async fn test_bybit_rest() -> RestTally {
    println!("\n── Bybit REST ───────────────────────────────────────────────");
    let mut tally = RestTally { exchange: "Bybit".into(), tested: 0, passed: 0, failed: 0 };

    let conn = match BybitConnector::public(false).await {
        Ok(c) => c,
        Err(e) => {
            println!("  FAIL: connector init -> {}", e);
            tally.failed += 1;
            tally.tested += 1;
            return tally;
        }
    };

    // get_open_interest
    tally.tested += 1;
    match conn.get_open_interest("linear", "BTCUSDT", "1h", Some(10), None, None).await {
        Ok(v) => { let (p, _) = ok_rest!("get_open_interest(linear, BTCUSDT, 1h)", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("get_open_interest", e); tally.failed += 1; }
    }

    // get_long_short_ratio
    tally.tested += 1;
    match conn.get_long_short_ratio("linear", "BTCUSDT", "1h", Some(10)).await {
        Ok(v) => { let (p, _) = ok_rest!("get_long_short_ratio(linear, BTCUSDT, 1h)", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("get_long_short_ratio", e); tally.failed += 1; }
    }

    // get_mark_price_kline
    tally.tested += 1;
    match conn.get_mark_price_kline("linear", "BTCUSDT", "60", Some(10), None, None).await {
        Ok(v) => { let (p, _) = ok_rest!("get_mark_price_kline(linear, BTCUSDT, 60min)", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("get_mark_price_kline", e); tally.failed += 1; }
    }

    tally
}

async fn test_okx_rest() -> RestTally {
    println!("\n── OKX REST ─────────────────────────────────────────────────");
    let mut tally = RestTally { exchange: "OKX".into(), tested: 0, passed: 0, failed: 0 };

    let conn = match OkxConnector::public(false).await {
        Ok(c) => c,
        Err(e) => {
            println!("  FAIL: connector init -> {}", e);
            tally.failed += 1;
            tally.tested += 1;
            return tally;
        }
    };

    // get_open_interest
    tally.tested += 1;
    match conn.get_open_interest("SWAP", None, Some("BTC-USDT-SWAP")).await {
        Ok(v) => { let (p, _) = ok_rest!("get_open_interest(SWAP, BTC-USDT-SWAP)", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("get_open_interest", e); tally.failed += 1; }
    }

    // get_long_short_ratio
    tally.tested += 1;
    match conn.get_long_short_ratio("BTC", Some("1H"), None, None, Some(10)).await {
        Ok(v) => { let (p, _) = ok_rest!("get_long_short_ratio(BTC, 1H)", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("get_long_short_ratio", e); tally.failed += 1; }
    }

    // get_liquidation_orders — state="filled" required by OKX (unfilled also accepted)
    tally.tested += 1;
    match conn.get_liquidation_orders("SWAP", Some("BTC-USDT"), Some("BTC-USDT-SWAP"), Some("filled"), None, None, Some(10)).await {
        Ok(v) => { let (p, _) = ok_rest!("get_liquidation_orders(SWAP, BTC-USDT-SWAP)", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("get_liquidation_orders", e); tally.failed += 1; }
    }

    // get_mark_price
    tally.tested += 1;
    match conn.get_mark_price("BTC-USDT-SWAP", "SWAP").await {
        Ok(v) => { let (p, _) = ok_rest_single!("get_mark_price(BTC-USDT-SWAP)", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("get_mark_price", e); tally.failed += 1; }
    }

    tally
}

async fn test_hyperliquid_rest() -> RestTally {
    println!("\n── Hyperliquid REST ─────────────────────────────────────────");
    let mut tally = RestTally { exchange: "Hyperliquid".into(), tested: 0, passed: 0, failed: 0 };

    let conn = match HyperliquidConnector::public(false).await {
        Ok(c) => c,
        Err(e) => {
            println!("  FAIL: connector init -> {}", e);
            tally.failed += 1;
            tally.tested += 1;
            return tally;
        }
    };

    // get_meta_and_asset_ctxs
    tally.tested += 1;
    match conn.get_meta_and_asset_ctxs().await {
        Ok(v) => {
            let summary = if v.is_array() {
                format!("[{} elements]", v.as_array().map(|a| a.len()).unwrap_or(0))
            } else {
                abbrev(&v)
            };
            println!("  OK:   get_meta_and_asset_ctxs -> {}", summary);
            tally.passed += 1;
        }
        Err(e) => { fail_rest!("get_meta_and_asset_ctxs", e); tally.failed += 1; }
    }

    // get_predicted_fundings
    tally.tested += 1;
    match conn.get_predicted_fundings().await {
        Ok(v) => {
            let summary = if v.is_array() {
                format!("[{} entries]", v.as_array().map(|a| a.len()).unwrap_or(0))
            } else {
                abbrev(&v)
            };
            println!("  OK:   get_predicted_fundings -> {}", summary);
            tally.passed += 1;
        }
        Err(e) => { fail_rest!("get_predicted_fundings", e); tally.failed += 1; }
    }

    tally
}

async fn test_deribit_rest() -> RestTally {
    println!("\n── Deribit REST ─────────────────────────────────────────────");
    let mut tally = RestTally { exchange: "Deribit".into(), tested: 0, passed: 0, failed: 0 };

    let conn = match DeribitConnector::public(false).await {
        Ok(c) => c,
        Err(e) => {
            println!("  FAIL: connector init -> {}", e);
            tally.failed += 1;
            tally.tested += 1;
            return tally;
        }
    };

    // get_index_price
    tally.tested += 1;
    match conn.get_index_price("btc_usd").await {
        Ok(v) => { let (p, _) = ok_rest_single!("get_index_price(btc_usd)", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("get_index_price", e); tally.failed += 1; }
    }

    // get_historical_volatility
    tally.tested += 1;
    match conn.get_historical_volatility("BTC").await {
        Ok(v) => { let (p, _) = ok_rest_single!("get_historical_volatility(BTC)", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("get_historical_volatility", e); tally.failed += 1; }
    }

    // get_funding_rate_history (last 24h window)
    tally.tested += 1;
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    let start_ms = now_ms - 24 * 3600 * 1000;
    match conn.get_funding_rate_history("BTC-PERPETUAL", start_ms, now_ms).await {
        Ok(v) => { let (p, _) = ok_rest_single!("get_funding_rate_history(BTC-PERPETUAL, 24h)", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("get_funding_rate_history", e); tally.failed += 1; }
    }

    tally
}

async fn test_bitget_rest() -> RestTally {
    println!("\n── Bitget REST ──────────────────────────────────────────────");
    let mut tally = RestTally { exchange: "Bitget".into(), tested: 0, passed: 0, failed: 0 };

    let conn = match BitgetConnector::public().await {
        Ok(c) => c,
        Err(e) => {
            println!("  FAIL: connector init -> {}", e);
            tally.failed += 1;
            tally.tested += 1;
            return tally;
        }
    };

    // get_futures_open_interest
    tally.tested += 1;
    match conn.get_futures_open_interest("BTCUSDT", "USDT-FUTURES").await {
        Ok(v) => { let (p, _) = ok_rest_single!("get_futures_open_interest(BTCUSDT, USDT-FUTURES)", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("get_futures_open_interest", e); tally.failed += 1; }
    }

    tally
}

async fn test_htx_rest() -> RestTally {
    println!("\n── HTX REST ─────────────────────────────────────────────────");
    let mut tally = RestTally { exchange: "HTX".into(), tested: 0, passed: 0, failed: 0 };

    let conn = match HtxConnector::public(false).await {
        Ok(c) => c,
        Err(e) => {
            println!("  FAIL: connector init -> {}", e);
            tally.failed += 1;
            tally.tested += 1;
            return tally;
        }
    };

    // get_open_interest
    tally.tested += 1;
    match conn.get_open_interest(Some("BTC-USDT")).await {
        Ok(v) => { let (p, _) = ok_rest_single!("get_open_interest(BTC-USDT)", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("get_open_interest", e); tally.failed += 1; }
    }

    // get_mark_price
    tally.tested += 1;
    match conn.get_mark_price("BTC-USDT").await {
        Ok(v) => { let (p, _) = ok_rest_single!("get_mark_price(BTC-USDT)", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("get_mark_price", e); tally.failed += 1; }
    }

    tally
}

// Lighter get_liquidations needs auth — skip with message
fn test_lighter_rest_note() -> RestTally {
    println!("\n── Lighter REST ─────────────────────────────────────────────");
    println!("  SKIPPED: get_liquidations -> requires account credentials (account_index or l1_address)");
    RestTally { exchange: "Lighter".into(), tested: 1, passed: 0, failed: 0 }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SECTION B — WEBSOCKET
// ═══════════════════════════════════════════════════════════════════════════════

/// Subscribe to a channel, listen for `duration`, count events by variant name.
/// Returns (subscribed_ok, event_count, parse_error_count, channel_label).
async fn ws_listen<W>(
    ws: &mut W,
    request: SubscriptionRequest,
    duration: Duration,
    channel_label: &str,
) -> (bool, usize, usize, String)
where
    W: WebSocketConnector,
{
    match ws.subscribe(request).await {
        Err(e) => {
            println!("    FAIL subscribe {} -> {}", channel_label, e);
            return (false, 0, 0, channel_label.to_string());
        }
        Ok(_) => {}
    }

    let mut stream = ws.event_stream();
    let mut count = 0usize;
    let mut errors = 0usize;

    let result = timeout(duration, async {
        while let Some(item) = stream.next().await {
            match item {
                Ok(_) => count += 1,
                Err(_) => errors += 1,
            }
        }
    }).await;

    // timeout is expected (Ok = stream ended early, Err = timeout reached)
    let _ = result;

    println!(
        "    CH {} -> events={}, errors={}{}",
        channel_label,
        count,
        errors,
        if count == 0 { " [ZERO EVENTS]" } else { "" }
    );

    (true, count, errors, channel_label.to_string())
}

async fn test_binance_ws() -> WsTally {
    println!("\n── Binance WS ───────────────────────────────────────────────");
    let mut tally = WsTally {
        exchange: "Binance".into(),
        channels: 0,
        subscribed: 0,
        events: 0,
        parse_errors: 0,
        zero_event_channels: Vec::new(),
    };

    let duration = Duration::from_secs(10);
    let btc_futures = Symbol::new("BTC", "USDT");

    // Channel 1: forceOrder (liquidations)
    {
        tally.channels += 1;
        let mut ws = match BinanceWebSocket::new(None, false, AccountType::FuturesCross).await {
            Ok(w) => w,
            Err(e) => { println!("  FAIL WS init -> {}", e); return tally; }
        };
        if ws.connect(AccountType::FuturesCross).await.is_ok() {
            let req = SubscriptionRequest::new(btc_futures.clone(), StreamType::Liquidation);
            let (ok, n, err, label) = ws_listen(&mut ws, req, duration, "btcusdt@forceOrder").await;
            if ok { tally.subscribed += 1; }
            tally.events += n;
            tally.parse_errors += err;
            if ok && n == 0 { tally.zero_event_channels.push(label); }
            let _ = ws.disconnect().await;
        } else {
            println!("  FAIL: Binance WS connect (futures)");
        }
    }

    // Channel 2: aggTrade
    {
        tally.channels += 1;
        let mut ws = match BinanceWebSocket::new(None, false, AccountType::FuturesCross).await {
            Ok(w) => w,
            Err(e) => { println!("  FAIL WS init -> {}", e); return tally; }
        };
        if ws.connect(AccountType::FuturesCross).await.is_ok() {
            let req = SubscriptionRequest::new(btc_futures.clone(), StreamType::AggTrade);
            let (ok, n, err, label) = ws_listen(&mut ws, req, duration, "btcusdt@aggTrade").await;
            if ok { tally.subscribed += 1; }
            tally.events += n;
            tally.parse_errors += err;
            if ok && n == 0 { tally.zero_event_channels.push(label); }
            let _ = ws.disconnect().await;
        }
    }

    // Channel 3: markPriceKline_1m
    {
        tally.channels += 1;
        let mut ws = match BinanceWebSocket::new(None, false, AccountType::FuturesCross).await {
            Ok(w) => w,
            Err(e) => { println!("  FAIL WS init -> {}", e); return tally; }
        };
        if ws.connect(AccountType::FuturesCross).await.is_ok() {
            let req = SubscriptionRequest::new(btc_futures.clone(), StreamType::MarkPriceKline { interval: "1m".to_string() });
            let (ok, n, err, label) = ws_listen(&mut ws, req, duration, "btcusdt@markPriceKline_1m").await;
            if ok { tally.subscribed += 1; }
            tally.events += n;
            tally.parse_errors += err;
            if ok && n == 0 { tally.zero_event_channels.push(label); }
            let _ = ws.disconnect().await;
        }
    }

    // Channel 4: !compositeIndex@arr
    {
        tally.channels += 1;
        let mut ws = match BinanceWebSocket::new(None, false, AccountType::FuturesCross).await {
            Ok(w) => w,
            Err(e) => { println!("  FAIL WS init -> {}", e); return tally; }
        };
        if ws.connect(AccountType::FuturesCross).await.is_ok() {
            // CompositeIndex uses empty symbol
            let req = SubscriptionRequest::new(Symbol::empty(), StreamType::CompositeIndex);
            let (ok, n, err, label) = ws_listen(&mut ws, req, duration, "!compositeIndex@arr").await;
            if ok { tally.subscribed += 1; }
            tally.events += n;
            tally.parse_errors += err;
            if ok && n == 0 { tally.zero_event_channels.push(label); }
            let _ = ws.disconnect().await;
        }
    }

    tally
}

async fn test_bybit_ws() -> WsTally {
    println!("\n── Bybit WS ─────────────────────────────────────────────────");
    let mut tally = WsTally {
        exchange: "Bybit".into(),
        channels: 0,
        subscribed: 0,
        events: 0,
        parse_errors: 0,
        zero_event_channels: Vec::new(),
    };

    let duration = Duration::from_secs(10);
    let btc = Symbol::new("BTC", "USDT");

    // Channel 1: tickers.BTCUSDT (linear) — multi-emit: Ticker + FundingRate + MarkPrice + OpenInterestUpdate
    {
        tally.channels += 1;
        let mut ws = match BybitWebSocket::new(None, false, AccountType::FuturesCross).await {
            Ok(w) => w,
            Err(e) => { println!("  FAIL WS init -> {}", e); return tally; }
        };
        if ws.connect(AccountType::FuturesCross).await.is_ok() {
            let req = SubscriptionRequest::new(btc.clone(), StreamType::Ticker);
            let (ok, n, err, label) = ws_listen(&mut ws, req, duration, "tickers.BTCUSDT(linear)").await;
            if ok { tally.subscribed += 1; }
            tally.events += n;
            tally.parse_errors += err;
            if ok && n == 0 { tally.zero_event_channels.push(label); }
            let _ = ws.disconnect().await;
        }
    }

    // Channel 2: liquidation.BTCUSDT
    {
        tally.channels += 1;
        let mut ws = match BybitWebSocket::new(None, false, AccountType::FuturesCross).await {
            Ok(w) => w,
            Err(e) => { println!("  FAIL WS init -> {}", e); return tally; }
        };
        if ws.connect(AccountType::FuturesCross).await.is_ok() {
            let req = SubscriptionRequest::new(btc.clone(), StreamType::Liquidation);
            let (ok, n, err, label) = ws_listen(&mut ws, req, duration, "liquidation.BTCUSDT").await;
            if ok { tally.subscribed += 1; }
            tally.events += n;
            tally.parse_errors += err;
            if ok && n == 0 { tally.zero_event_channels.push(label); }
            let _ = ws.disconnect().await;
        }
    }

    // Channel 3: insurance.USDT
    {
        tally.channels += 1;
        let mut ws = match BybitWebSocket::new(None, false, AccountType::FuturesCross).await {
            Ok(w) => w,
            Err(e) => { println!("  FAIL WS init -> {}", e); return tally; }
        };
        if ws.connect(AccountType::FuturesCross).await.is_ok() {
            // InsuranceFund uses symbol base=coin, quote="" for Bybit
            let req = SubscriptionRequest::new(Symbol::new("USDT", ""), StreamType::InsuranceFund);
            let (ok, n, err, label) = ws_listen(&mut ws, req, duration, "insurance.USDT").await;
            if ok { tally.subscribed += 1; }
            tally.events += n;
            tally.parse_errors += err;
            if ok && n == 0 { tally.zero_event_channels.push(label); }
            let _ = ws.disconnect().await;
        }
    }

    tally
}

async fn test_okx_ws() -> WsTally {
    println!("\n── OKX WS ───────────────────────────────────────────────────");
    let mut tally = WsTally {
        exchange: "OKX".into(),
        channels: 0,
        subscribed: 0,
        events: 0,
        parse_errors: 0,
        zero_event_channels: Vec::new(),
    };

    let duration = Duration::from_secs(10);
    // OKX SWAP inst_id is built from base+quote+account_type inside the WS connector.
    // Using FuturesCross → "BTC-USDT-SWAP"
    let btc_swap = Symbol::new("BTC", "USDT");

    // Channel 1: tickers BTC-USDT-SWAP — multi-emit
    {
        tally.channels += 1;
        let mut ws = match OkxWebSocket::new(None, false).await {
            Ok(w) => w,
            Err(e) => { println!("  FAIL WS init -> {}", e); return tally; }
        };
        let mut req = SubscriptionRequest::new(btc_swap.clone(), StreamType::Ticker);
        req.account_type = AccountType::FuturesCross;
        if ws.connect(AccountType::FuturesCross).await.is_ok() {
            let (ok, n, err, label) = ws_listen(&mut ws, req, duration, "tickers BTC-USDT-SWAP").await;
            if ok { tally.subscribed += 1; }
            tally.events += n;
            tally.parse_errors += err;
            if ok && n == 0 { tally.zero_event_channels.push(label); }
            let _ = ws.disconnect().await;
        }
    }

    // Channel 2: liquidation-orders instType=SWAP
    {
        tally.channels += 1;
        let mut ws = match OkxWebSocket::new(None, false).await {
            Ok(w) => w,
            Err(e) => { println!("  FAIL WS init -> {}", e); return tally; }
        };
        let mut req = SubscriptionRequest::new(btc_swap.clone(), StreamType::Liquidation);
        req.account_type = AccountType::FuturesCross;
        if ws.connect(AccountType::FuturesCross).await.is_ok() {
            let (ok, n, err, label) = ws_listen(&mut ws, req, duration, "liquidation-orders BTC-USDT-SWAP").await;
            if ok { tally.subscribed += 1; }
            tally.events += n;
            tally.parse_errors += err;
            if ok && n == 0 { tally.zero_event_channels.push(label); }
            let _ = ws.disconnect().await;
        }
    }

    // Channel 3: index-tickers BTC-USDT
    {
        tally.channels += 1;
        let mut ws = match OkxWebSocket::new(None, false).await {
            Ok(w) => w,
            Err(e) => { println!("  FAIL WS init -> {}", e); return tally; }
        };
        // IndexPrice on OKX uses spot inst_id: BTC-USDT
        let mut req = SubscriptionRequest::new(btc_swap.clone(), StreamType::IndexPrice);
        req.account_type = AccountType::Spot;
        if ws.connect(AccountType::Spot).await.is_ok() {
            let (ok, n, err, label) = ws_listen(&mut ws, req, duration, "index-tickers BTC-USDT").await;
            if ok { tally.subscribed += 1; }
            tally.events += n;
            tally.parse_errors += err;
            if ok && n == 0 { tally.zero_event_channels.push(label); }
            let _ = ws.disconnect().await;
        }
    }

    // Channel 4: mark-price-candle1m BTC-USDT-SWAP
    {
        tally.channels += 1;
        let mut ws = match OkxWebSocket::new(None, false).await {
            Ok(w) => w,
            Err(e) => { println!("  FAIL WS init -> {}", e); return tally; }
        };
        let mut req = SubscriptionRequest::new(btc_swap.clone(), StreamType::MarkPriceKline { interval: "1m".to_string() });
        req.account_type = AccountType::FuturesCross;
        if ws.connect(AccountType::FuturesCross).await.is_ok() {
            let (ok, n, err, label) = ws_listen(&mut ws, req, duration, "mark-price-candle1m BTC-USDT-SWAP").await;
            if ok { tally.subscribed += 1; }
            tally.events += n;
            tally.parse_errors += err;
            if ok && n == 0 { tally.zero_event_channels.push(label); }
            let _ = ws.disconnect().await;
        }
    }

    tally
}

async fn test_hyperliquid_ws() -> WsTally {
    println!("\n── Hyperliquid WS ───────────────────────────────────────────");
    let mut tally = WsTally {
        exchange: "Hyperliquid".into(),
        channels: 0,
        subscribed: 0,
        events: 0,
        parse_errors: 0,
        zero_event_channels: Vec::new(),
    };

    let duration = Duration::from_secs(10);
    // Hyperliquid uses coin name in `base`, no quote
    let btc = Symbol::new("BTC", "");

    // Channel 1: activeAssetCtx coin=BTC — multi-emit: Ticker + MarkPrice + FundingRate + OpenInterestUpdate + IndexPrice
    // NOTE: HyperliquidWebSocket message handler holds ws_stream Mutex across .next().await,
    // so subscribe() (which also needs the lock) deadlocks / gets "closed connection".
    // This is a known architectural limitation of the HL WS implementation.
    {
        tally.channels += 1;
        let mut ws = HyperliquidWebSocket::new(false);
        if ws.connect(AccountType::FuturesCross).await.is_ok() {
            let req = SubscriptionRequest::new(btc.clone(), StreamType::Ticker);
            match ws.subscribe(req).await {
                Ok(_) => {
                    tally.subscribed += 1;
                    let mut stream = ws.event_stream();
                    let mut n = 0usize;
                    let mut errors = 0usize;
                    let _ = timeout(duration, async {
                        while let Some(item) = stream.next().await {
                            match item { Ok(_) => n += 1, Err(_) => errors += 1 }
                        }
                    }).await;
                    tally.events += n;
                    tally.parse_errors += errors;
                    let label = "activeAssetCtx BTC".to_string();
                    println!(
                        "    CH {} -> events={}, errors={}{}",
                        label, n, errors,
                        if n == 0 { " [ZERO EVENTS]" } else { "" }
                    );
                    if n == 0 { tally.zero_event_channels.push(label); }
                }
                Err(e) => {
                    println!("    FAIL subscribe activeAssetCtx BTC -> {} [known: mutex deadlock in HL WS]", e);
                }
            }
            let _ = ws.disconnect().await;
        } else {
            println!("  FAIL: Hyperliquid WS connect");
        }
    }

    // Channel 2: bbo coin=BTC
    {
        tally.channels += 1;
        let mut ws = HyperliquidWebSocket::new(false);
        if ws.connect(AccountType::FuturesCross).await.is_ok() {
            // bbo is not a standard StreamType — Hyperliquid maps Trade to bbo in some configs.
            // Use SubscriptionRequest with custom stream type not matching any HL type
            // so it falls back to allMids, OR we use AggTrade if mapped.
            // Check websocket.rs: StreamType::Trade → "trades", no "bbo" StreamType mapping exists.
            // The bbo channel is Hyperliquid-specific and not exposed via standard SubscriptionRequest.
            // We note this and skip.
            println!("    NOTE: 'bbo' channel has no standard StreamType mapping in HL WS — skipped (falls back to allMids)");
            tally.channels -= 1; // don't count it
            let _ = ws.disconnect().await;
        }
    }

    tally
}

async fn test_deribit_ws() -> WsTally {
    println!("\n── Deribit WS ───────────────────────────────────────────────");
    let mut tally = WsTally {
        exchange: "Deribit".into(),
        channels: 0,
        subscribed: 0,
        events: 0,
        parse_errors: 0,
        zero_event_channels: Vec::new(),
    };

    let duration = Duration::from_secs(10);
    let btc_perp = Symbol::new("BTC", "PERPETUAL");

    // Channel 1: ticker.BTC-PERPETUAL.raw — Ticker + FundingRate + MarkPrice multi-emit
    {
        tally.channels += 1;
        let mut ws = match DeribitWebSocket::new(None, false, AccountType::FuturesCross).await {
            Ok(w) => w,
            Err(e) => { println!("  FAIL WS init -> {}", e); return tally; }
        };
        if ws.connect(AccountType::FuturesCross).await.is_ok() {
            let req = SubscriptionRequest::new(btc_perp.clone(), StreamType::Ticker);
            let (ok, n, err, label) = ws_listen(&mut ws, req, duration, "ticker.BTC-PERPETUAL.raw").await;
            if ok { tally.subscribed += 1; }
            tally.events += n;
            tally.parse_errors += err;
            if ok && n == 0 { tally.zero_event_channels.push(label); }
            let _ = ws.disconnect().await;
        } else {
            println!("  FAIL: Deribit WS connect");
        }
    }

    tally
}

// ═══════════════════════════════════════════════════════════════════════════════
// SECTION C — TALLY
// ═══════════════════════════════════════════════════════════════════════════════

fn print_rest_summary(tallies: &[RestTally]) {
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║  Section A — REST Trading-Metadata Tally                         ║");
    println!("╠═══════════════╦═════════╦═════════╦═════════╗");
    println!("║ Exchange      ║ Tested  ║ Passed  ║ Failed  ║");
    println!("╠═══════════════╬═════════╬═════════╬═════════╣");
    for t in tallies {
        println!(
            "║ {:13} ║ {:7} ║ {:7} ║ {:7} ║",
            t.exchange, t.tested, t.passed, t.failed
        );
    }
    println!("╚═══════════════╩═════════╩═════════╩═════════╝");
    let total_tested: usize = tallies.iter().map(|t| t.tested).sum();
    let total_passed: usize = tallies.iter().map(|t| t.passed).sum();
    let total_failed: usize = tallies.iter().map(|t| t.failed).sum();
    println!(
        "  Total: tested={} passed={} failed={} (skipped={})",
        total_tested,
        total_passed,
        total_failed,
        total_tested.saturating_sub(total_passed + total_failed)
    );
}

fn print_ws_summary(tallies: &[WsTally]) {
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║  Section B — WebSocket Channel Tally                             ║");
    println!("╠═══════════════╦══════╦══════╦══════════╦════════╗");
    println!("║ Exchange      ║ Ch   ║ SubOK║ Events   ║ Errors ║");
    println!("╠═══════════════╬══════╬══════╬══════════╬════════╣");
    for t in tallies {
        println!(
            "║ {:13} ║ {:4} ║ {:4} ║ {:8} ║ {:6} ║",
            t.exchange, t.channels, t.subscribed, t.events, t.parse_errors
        );
    }
    println!("╚═══════════════╩══════╩══════╩══════════╩════════╝");

    for t in tallies {
        if !t.zero_event_channels.is_empty() {
            println!(
                "  WARN [{}] zero-event channels (parse fail or quiet market): {:?}",
                t.exchange, t.zero_event_channels
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MAIN
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::main]
async fn main() {
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║  e2e_metadata — Live Trading-Metadata Smoke Test                 ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!("Hitting live exchange APIs — no keys required for public endpoints.");
    println!("WS channels run for 10 seconds each.");

    // ── Section A: REST ──────────────────────────────────────────────────────
    println!("\n══════════════════ Section A: REST ══════════════════");

    let binance_rest = test_binance_rest().await;
    let bybit_rest   = test_bybit_rest().await;
    let okx_rest     = test_okx_rest().await;
    let hl_rest      = test_hyperliquid_rest().await;
    let deribit_rest = test_deribit_rest().await;
    let bitget_rest  = test_bitget_rest().await;
    let htx_rest     = test_htx_rest().await;
    let lighter_rest = test_lighter_rest_note();

    let rest_tallies = vec![
        binance_rest,
        bybit_rest,
        okx_rest,
        hl_rest,
        deribit_rest,
        bitget_rest,
        htx_rest,
        lighter_rest,
    ];

    // ── Section B: WebSocket ─────────────────────────────────────────────────
    println!("\n══════════════════ Section B: WebSocket ══════════════════");
    println!("(each channel listens 10 s — sequential to avoid port exhaustion)");

    let binance_ws  = test_binance_ws().await;
    let bybit_ws    = test_bybit_ws().await;
    let okx_ws      = test_okx_ws().await;
    let hl_ws       = test_hyperliquid_ws().await;
    let deribit_ws  = test_deribit_ws().await;

    let ws_tallies = vec![
        binance_ws,
        bybit_ws,
        okx_ws,
        hl_ws,
        deribit_ws,
    ];

    // ── Section C: Summary ───────────────────────────────────────────────────
    println!("\n══════════════════ Section C: Summary ══════════════════");
    print_rest_summary(&rest_tallies);
    print_ws_summary(&ws_tallies);

    println!("\nDone.");
}
