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
use digdigdig3::l3::open::crypto::cex::htx::{HtxConnector, HtxWebSocket};
use digdigdig3::l3::open::crypto::cex::kucoin::{KuCoinConnector, KuCoinWebSocket};
use digdigdig3::l3::open::crypto::cex::gateio::{GateioConnector, GateioWebSocket};
use digdigdig3::l3::open::crypto::cex::bitfinex::{BitfinexConnector, BitfinexWebSocket};
use digdigdig3::l3::open::crypto::cex::kraken::{KrakenConnector, KrakenWebSocket};
use digdigdig3::l3::open::crypto::cex::gemini::{GeminiConnector, GeminiWebSocket};
use digdigdig3::l3::open::crypto::cex::bitstamp::{BitstampConnector, BitstampWebSocket};
use digdigdig3::l3::open::crypto::cex::upbit::UpbitConnector;
use digdigdig3::l3::open::crypto::cex::crypto_com::{CryptoComConnector, CryptoComWebSocket};
use digdigdig3::l3::open::crypto::cex::bingx::BingxConnector;
use digdigdig3::l3::open::crypto::cex::coinbase::CoinbaseWebSocket;
use digdigdig3::l3::open::crypto::dex::dydx::DydxConnector;

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

    // NEW: get_basis_history
    tally.tested += 1;
    match conn.get_basis_history("BTCUSDT", "PERPETUAL", "5m", Some(5), None, None).await {
        Ok(v) => { let (p, _) = ok_rest_single!("get_basis_history(BTCUSDT, PERPETUAL, 5m)", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("get_basis_history", e); tally.failed += 1; }
    }

    // NEW: get_open_interest_cm (coin-margined)
    tally.tested += 1;
    match conn.get_open_interest_cm("BTCUSD_PERP").await {
        Ok(v) => { let (p, _) = ok_rest_single!("get_open_interest_cm(BTCUSD_PERP)", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("get_open_interest_cm", e); tally.failed += 1; }
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

    // NEW: get_risk_limit
    tally.tested += 1;
    match conn.get_risk_limit("linear", "BTCUSDT").await {
        Ok(v) => { let (p, _) = ok_rest_single!("get_risk_limit(linear, BTCUSDT)", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("get_risk_limit", e); tally.failed += 1; }
    }

    // NEW: get_delivery_price (linear, BTCUSDT)
    tally.tested += 1;
    match conn.get_delivery_price("linear", "BTCUSDT", Some(5)).await {
        Ok(v) => { let (p, _) = ok_rest_single!("get_delivery_price(linear, BTCUSDT)", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("get_delivery_price", e); tally.failed += 1; }
    }

    // NEW: get_institutional_loan_products
    tally.tested += 1;
    match conn.get_institutional_loan_products().await {
        Ok(v) => { let (p, _) = ok_rest_single!("get_institutional_loan_products()", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("get_institutional_loan_products", e); tally.failed += 1; }
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

    // get_liquidation_orders
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

    // NEW: get_position_tiers
    tally.tested += 1;
    match conn.get_position_tiers("SWAP", "isolated", None, Some("BTC-USD"), Some("BTC-USD-SWAP"), None, None).await {
        Ok(v) => { let (p, _) = ok_rest_single!("get_position_tiers(SWAP, isolated, BTC-USD-SWAP)", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("get_position_tiers", e); tally.failed += 1; }
    }

    // NEW: get_funding_rate_history
    tally.tested += 1;
    match conn.get_funding_rate_history("BTC-USDT-SWAP", None, None, Some(5)).await {
        Ok(v) => { let (p, _) = ok_rest_single!("get_funding_rate_history(BTC-USDT-SWAP)", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("get_funding_rate_history", e); tally.failed += 1; }
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

    // NEW: get_spot_meta_and_asset_ctxs
    tally.tested += 1;
    match conn.get_spot_meta_and_asset_ctxs().await {
        Ok(v) => {
            let summary = if v.is_array() {
                format!("[{} elements]", v.as_array().map(|a| a.len()).unwrap_or(0))
            } else {
                abbrev(&v)
            };
            println!("  OK:   get_spot_meta_and_asset_ctxs -> {}", summary);
            tally.passed += 1;
        }
        Err(e) => { fail_rest!("get_spot_meta_and_asset_ctxs", e); tally.failed += 1; }
    }

    // NEW: get_non_funding_ledger_updates (zero address — expect empty array)
    tally.tested += 1;
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    match conn.get_non_funding_ledger_updates(
        "0x0000000000000000000000000000000000000000",
        0,
        Some(now_ms),
    ).await {
        Ok(v) => {
            let summary = if v.is_array() {
                format!("[{} entries]", v.as_array().map(|a| a.len()).unwrap_or(0))
            } else {
                abbrev(&v)
            };
            println!("  OK:   get_non_funding_ledger_updates(zero_addr) -> {}", summary);
            tally.passed += 1;
        }
        Err(e) => { fail_rest!("get_non_funding_ledger_updates", e); tally.failed += 1; }
    }

    // NEW: get_vault_details (HLP vault)
    tally.tested += 1;
    match conn.get_vault_details("0xa15099a30bbf2e68942d6f4c43d70d04faeab0a0").await {
        Ok(v) => {
            let summary = if v.is_null() { "null (unknown vault)".to_string() } else { abbrev(&v) };
            println!("  OK:   get_vault_details(HLP) -> {}", summary);
            tally.passed += 1;
        }
        Err(e) => { fail_rest!("get_vault_details", e); tally.failed += 1; }
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

    // NEW: get_futures_market_fills
    tally.tested += 1;
    match conn.get_futures_market_fills("BTCUSDT", "USDT-FUTURES", Some(10)).await {
        Ok(v) => { let (p, _) = ok_rest_single!("get_futures_market_fills(BTCUSDT, USDT-FUTURES)", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("get_futures_market_fills", e); tally.failed += 1; }
    }

    // NEW: get_futures_mark_candles
    tally.tested += 1;
    match conn.get_futures_mark_candles("BTCUSDT", "USDT-FUTURES", "1H", None, None, Some(5)).await {
        Ok(v) => { let (p, _) = ok_rest_single!("get_futures_mark_candles(BTCUSDT, USDT-FUTURES, 1H)", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("get_futures_mark_candles", e); tally.failed += 1; }
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

    // NEW: get_elite_account_ratio
    tally.tested += 1;
    match conn.get_elite_account_ratio("BTC-USDT", "1hour").await {
        Ok(v) => { let (p, _) = ok_rest_single!("get_elite_account_ratio(BTC-USDT, 1hour)", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("get_elite_account_ratio", e); tally.failed += 1; }
    }

    // NEW: get_historical_funding_rate
    tally.tested += 1;
    match conn.get_historical_funding_rate("BTC-USDT", Some(1), Some(5)).await {
        Ok(v) => { let (p, _) = ok_rest_single!("get_historical_funding_rate(BTC-USDT)", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("get_historical_funding_rate", e); tally.failed += 1; }
    }

    tally
}

async fn test_kucoin_rest() -> RestTally {
    println!("\n── KuCoin REST ──────────────────────────────────────────────");
    let mut tally = RestTally { exchange: "KuCoin".into(), tested: 0, passed: 0, failed: 0 };

    let conn = match KuCoinConnector::public(false).await {
        Ok(c) => c,
        Err(e) => {
            println!("  FAIL: connector init -> {}", e);
            tally.failed += 1;
            tally.tested += 1;
            return tally;
        }
    };

    // get_risk_limit
    tally.tested += 1;
    match conn.get_risk_limit("XBTUSDTM").await {
        Ok(v) => { let (p, _) = ok_rest_single!("get_risk_limit(XBTUSDTM)", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("get_risk_limit", e); tally.failed += 1; }
    }

    // get_historical_funding_rates (last 24h)
    tally.tested += 1;
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    let from_ms = now_ms - 86_400_000;
    match conn.get_historical_funding_rates("XBTUSDTM", Some(from_ms), Some(now_ms)).await {
        Ok(v) => { let (p, _) = ok_rest_single!("get_historical_funding_rates(XBTUSDTM)", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("get_historical_funding_rates", e); tally.failed += 1; }
    }

    tally
}

async fn test_gateio_rest() -> RestTally {
    println!("\n── Gate.io REST ─────────────────────────────────────────────");
    let mut tally = RestTally { exchange: "Gate.io".into(), tested: 0, passed: 0, failed: 0 };

    let conn = match GateioConnector::public(false).await {
        Ok(c) => c,
        Err(e) => {
            println!("  FAIL: connector init -> {}", e);
            tally.failed += 1;
            tally.tested += 1;
            return tally;
        }
    };

    // get_contract_stats (contract, from, to, interval, limit)
    tally.tested += 1;
    match conn.get_contract_stats("BTC_USDT", None, None, Some("1h"), Some(5)).await {
        Ok(v) => { let (p, _) = ok_rest_single!("get_contract_stats(BTC_USDT, 1h)", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("get_contract_stats", e); tally.failed += 1; }
    }

    // get_insurance_fund (limit)
    tally.tested += 1;
    match conn.get_insurance_fund(Some(5)).await {
        Ok(v) => { let (p, _) = ok_rest_single!("get_insurance_fund(5)", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("get_insurance_fund", e); tally.failed += 1; }
    }

    tally
}

async fn test_dydx_rest() -> RestTally {
    println!("\n── dYdX REST ────────────────────────────────────────────────");
    let mut tally = RestTally { exchange: "dYdX".into(), tested: 0, passed: 0, failed: 0 };

    let conn = match DydxConnector::public(false).await {
        Ok(c) => c,
        Err(e) => {
            println!("  FAIL: connector init -> {}", e);
            tally.failed += 1;
            tally.tested += 1;
            return tally;
        }
    };

    // get_markets
    tally.tested += 1;
    match conn.get_markets().await {
        Ok(v) => { let (p, _) = ok_rest_single!("get_markets()", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("get_markets", e); tally.failed += 1; }
    }

    // get_historical_funding
    tally.tested += 1;
    match conn.get_historical_funding("BTC-USD", Some(5)).await {
        Ok(v) => { let (p, _) = ok_rest!("get_historical_funding(BTC-USD, 5)", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("get_historical_funding", e); tally.failed += 1; }
    }

    tally
}

async fn test_lighter_rest() -> RestTally {
    println!("\n── Lighter REST ─────────────────────────────────────────────");
    println!("  SKIP: Lighter mainnet TCP unreachable from this host (geo/firewall) — skipping all REST");
    RestTally { exchange: "Lighter".into(), tested: 0, passed: 0, failed: 0 }
}

async fn test_bitfinex_rest() -> RestTally {
    println!("\n── Bitfinex REST ────────────────────────────────────────────");
    let mut tally = RestTally { exchange: "Bitfinex".into(), tested: 0, passed: 0, failed: 0 };

    let conn = match BitfinexConnector::public(false).await {
        Ok(c) => c,
        Err(e) => {
            println!("  FAIL: connector init -> {}", e);
            tally.failed += 1;
            tally.tested += 1;
            return tally;
        }
    };

    // get_derivative_status_history(symbol, start, end, limit, sort)
    tally.tested += 1;
    match conn.get_derivative_status_history("tBTCF0:USTF0", None, None, Some(3), None).await {
        Ok(v) => { let (p, _) = ok_rest_single!("get_derivative_status_history(tBTCF0:USTF0, 3)", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("get_derivative_status_history", e); tally.failed += 1; }
    }

    // get_funding_stats(symbol, limit, start, end)
    tally.tested += 1;
    match conn.get_funding_stats("fUSD", Some(3), None, None).await {
        Ok(v) => { let (p, _) = ok_rest_single!("get_funding_stats(fUSD, 3)", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("get_funding_stats", e); tally.failed += 1; }
    }

    tally
}

async fn test_kraken_rest() -> RestTally {
    println!("\n── Kraken REST ──────────────────────────────────────────────");
    let mut tally = RestTally { exchange: "Kraken".into(), tested: 0, passed: 0, failed: 0 };

    let conn = match KrakenConnector::public(false).await {
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
    match conn.get_futures_open_interest(Some("PF_XBTUSD")).await {
        Ok(v) => { let (p, _) = ok_rest_single!("get_futures_open_interest(PF_XBTUSD)", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("get_futures_open_interest", e); tally.failed += 1; }
    }

    tally
}

async fn test_gemini_rest() -> RestTally {
    println!("\n── Gemini REST ──────────────────────────────────────────────");
    let mut tally = RestTally { exchange: "Gemini".into(), tested: 0, passed: 0, failed: 0 };

    let conn = match GeminiConnector::public(false).await {
        Ok(c) => c,
        Err(e) => {
            println!("  FAIL: connector init -> {}", e);
            tally.failed += 1;
            tally.tested += 1;
            return tally;
        }
    };

    // get_trades_with_breaks(symbol, limit, since_tid)
    tally.tested += 1;
    match conn.get_trades_with_breaks("btcusd", Some(3), None).await {
        Ok(v) => { let (p, _) = ok_rest_single!("get_trades_with_breaks(btcusd, 3)", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("get_trades_with_breaks", e); tally.failed += 1; }
    }

    tally
}

async fn test_bitstamp_rest() -> RestTally {
    println!("\n── Bitstamp REST ────────────────────────────────────────────");
    let mut tally = RestTally { exchange: "Bitstamp".into(), tested: 0, passed: 0, failed: 0 };

    let conn = match BitstampConnector::public().await {
        Ok(c) => c,
        Err(e) => {
            println!("  FAIL: connector init -> {}", e);
            tally.failed += 1;
            tally.tested += 1;
            return tally;
        }
    };

    // get_markets — confirm existing
    tally.tested += 1;
    match conn.get_markets().await {
        Ok(v) => { let (p, _) = ok_rest_single!("get_markets()", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("get_markets", e); tally.failed += 1; }
    }

    tally
}

async fn test_upbit_rest() -> RestTally {
    println!("\n── Upbit REST ───────────────────────────────────────────────");
    let mut tally = RestTally { exchange: "Upbit".into(), tested: 0, passed: 0, failed: 0 };

    let conn = match UpbitConnector::public().await {
        Ok(c) => c,
        Err(e) => {
            println!("  FAIL: connector init -> {}", e);
            tally.failed += 1;
            tally.tested += 1;
            return tally;
        }
    };

    // get_markets_with_warnings — returns Vec<StreamEvent> with caution flags
    tally.tested += 1;
    match conn.get_markets_with_warnings().await {
        Ok(v) => {
            println!("  OK:   get_markets_with_warnings() -> {} items", v.len());
            tally.passed += 1;
        }
        Err(e) => { fail_rest!("get_markets_with_warnings", e); tally.failed += 1; }
    }

    tally
}

async fn test_crypto_com_rest() -> RestTally {
    println!("\n── Crypto.com REST ──────────────────────────────────────────");
    let mut tally = RestTally { exchange: "Crypto.com".into(), tested: 0, passed: 0, failed: 0 };

    let conn = match CryptoComConnector::public(false).await {
        Ok(c) => c,
        Err(e) => {
            println!("  FAIL: connector init -> {}", e);
            tally.failed += 1;
            tally.tested += 1;
            return tally;
        }
    };

    // get_expired_settlement_price
    tally.tested += 1;
    match conn.get_expired_settlement_price("PERPETUAL_SWAP").await {
        Ok(v) => { let (p, _) = ok_rest_single!("get_expired_settlement_price(PERPETUAL_SWAP)", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("get_expired_settlement_price", e); tally.failed += 1; }
    }

    // get_insurance — requires instrument_name (not instrument_type)
    tally.tested += 1;
    match conn.get_insurance("BTCUSD-PERP").await {
        Ok(v) => { let (p, _) = ok_rest_single!("get_insurance(BTCUSD-PERP)", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("get_insurance", e); tally.failed += 1; }
    }

    tally
}

async fn test_bingx_rest() -> RestTally {
    println!("\n── BingX REST ───────────────────────────────────────────────");
    let mut tally = RestTally { exchange: "BingX".into(), tested: 0, passed: 0, failed: 0 };

    let conn = match BingxConnector::public(false).await {
        Ok(c) => c,
        Err(e) => {
            println!("  FAIL: connector init -> {}", e);
            tally.failed += 1;
            tally.tested += 1;
            return tally;
        }
    };

    // swap_open_interest
    tally.tested += 1;
    match conn.swap_open_interest("BTC-USDT").await {
        Ok(v) => { let (p, _) = ok_rest_single!("swap_open_interest(BTC-USDT)", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("swap_open_interest", e); tally.failed += 1; }
    }

    // swap_premium_index
    tally.tested += 1;
    match conn.swap_premium_index(Some("BTC-USDT")).await {
        Ok(v) => { let (p, _) = ok_rest_single!("swap_premium_index(BTC-USDT)", v); tally.passed += p as usize; }
        Err(e) => { fail_rest!("swap_premium_index", e); tally.failed += 1; }
    }

    tally
}

fn test_mexc_note() -> RestTally {
    println!("\n── MEXC REST ────────────────────────────────────────────────");
    println!("  SKIPPED: MEXC geo-blocked from this IP (confirmed by E2D agent)");
    RestTally { exchange: "MEXC".into(), tested: 0, passed: 0, failed: 0 }
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

    let duration = Duration::from_secs(5);
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
            let req = SubscriptionRequest::new(Symbol::empty(), StreamType::CompositeIndex);
            let (ok, n, err, label) = ws_listen(&mut ws, req, duration, "!compositeIndex@arr").await;
            if ok { tally.subscribed += 1; }
            tally.events += n;
            tally.parse_errors += err;
            if ok && n == 0 { tally.zero_event_channels.push(label); }
            let _ = ws.disconnect().await;
        }
    }

    // NEW Channel 5: !forceOrder@arr global liquidation stream (empty symbol)
    {
        tally.channels += 1;
        let mut ws = match BinanceWebSocket::new(None, false, AccountType::FuturesCross).await {
            Ok(w) => w,
            Err(e) => { println!("  FAIL WS init -> {}", e); return tally; }
        };
        if ws.connect(AccountType::FuturesCross).await.is_ok() {
            // Global liquidation: subscribe with empty symbol triggers !forceOrder@arr
            let req = SubscriptionRequest::new(Symbol::empty(), StreamType::Liquidation);
            let (ok, n, err, label) = ws_listen(&mut ws, req, duration, "!forceOrder@arr (global)").await;
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

    let duration = Duration::from_secs(5);
    let btc = Symbol::new("BTC", "USDT");

    // Channel 1: tickers.BTCUSDT (linear)
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
            let req = SubscriptionRequest::new(Symbol::new("USDT", ""), StreamType::InsuranceFund);
            let (ok, n, err, label) = ws_listen(&mut ws, req, duration, "insurance.USDT").await;
            if ok { tally.subscribed += 1; }
            tally.events += n;
            tally.parse_errors += err;
            if ok && n == 0 { tally.zero_event_channels.push(label); }
            let _ = ws.disconnect().await;
        }
    }

    // NEW Channel 4: adlAlert.BTCUSDT (RiskLimit emission)
    // adlAlert uses settlement coin not symbol — use USDT coin
    {
        tally.channels += 1;
        let mut ws = match BybitWebSocket::new(None, false, AccountType::FuturesCross).await {
            Ok(w) => w,
            Err(e) => { println!("  FAIL WS init -> {}", e); return tally; }
        };
        if ws.connect(AccountType::FuturesCross).await.is_ok() {
            // RiskLimit maps to adlAlert.<coin> — use USDT settlement coin
            let req = SubscriptionRequest::new(Symbol::new("USDT", ""), StreamType::RiskLimit);
            let (ok, n, err, label) = ws_listen(&mut ws, req, duration, "adlAlert.USDT").await;
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

    let duration = Duration::from_secs(5);
    let btc_swap = Symbol::new("BTC", "USDT");

    // Channel 1: tickers BTC-USDT-SWAP
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

    // Channel 4: mark-price-candle1m BTC-USDT-SWAP (business WS endpoint)
    {
        tally.channels += 1;
        let mut ws = match OkxWebSocket::new_business(None, false).await {
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

    // NEW Channel 5: block-trades instType=SWAP
    {
        tally.channels += 1;
        let mut ws = match OkxWebSocket::new(None, false).await {
            Ok(w) => w,
            Err(e) => { println!("  FAIL WS init -> {}", e); return tally; }
        };
        let mut req = SubscriptionRequest::new(btc_swap.clone(), StreamType::BlockTrade);
        req.account_type = AccountType::FuturesCross;
        if ws.connect(AccountType::FuturesCross).await.is_ok() {
            let (ok, n, err, label) = ws_listen(&mut ws, req, duration, "block-trades BTC-USDT-SWAP").await;
            if ok { tally.subscribed += 1; }
            tally.events += n;
            tally.parse_errors += err;
            if ok && n == 0 { tally.zero_event_channels.push(label); }
            let _ = ws.disconnect().await;
        }
    }

    // NEW Channel 6: estimated-price instType=OPTION uly=BTC-USD (SettlementEvent)
    {
        tally.channels += 1;
        let mut ws = match OkxWebSocket::new(None, false).await {
            Ok(w) => w,
            Err(e) => { println!("  FAIL WS init -> {}", e); return tally; }
        };
        // SettlementEvent maps to estimated-price; for OPTIONS use base=BTC, quote=USD
        let mut req = SubscriptionRequest::new(Symbol::new("BTC", "USD"), StreamType::SettlementEvent);
        req.account_type = AccountType::FuturesCross;
        if ws.connect(AccountType::FuturesCross).await.is_ok() {
            let (ok, n, err, label) = ws_listen(&mut ws, req, duration, "estimated-price BTC-USD (OPTIONS)").await;
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

    let duration = Duration::from_secs(5);
    let btc = Symbol::new("BTC", "");

    // Channel 1: activeAssetCtx coin=BTC
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

    // NEW Channel 2: allMids — should emit Vec<Ticker>
    {
        tally.channels += 1;
        let mut ws = HyperliquidWebSocket::new(false);
        if ws.connect(AccountType::FuturesCross).await.is_ok() {
            match ws.subscribe_all_mids().await {
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
                    let label = "allMids".to_string();
                    println!(
                        "    CH {} -> events={}, errors={}{}",
                        label, n, errors,
                        if n == 0 { " [ZERO EVENTS]" } else { "" }
                    );
                    if n == 0 { tally.zero_event_channels.push(label); }
                }
                Err(e) => {
                    println!("    FAIL subscribe allMids -> {}", e);
                }
            }
            let _ = ws.disconnect().await;
        } else {
            println!("  FAIL: Hyperliquid WS connect (allMids)");
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

    let duration = Duration::from_secs(5);
    let btc_perp = Symbol::new("BTC", "PERPETUAL");

    // Channel 1: ticker.BTC-PERPETUAL.raw
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

    // NEW Channel 2: deribit_volatility_index.btc_usd
    {
        tally.channels += 1;
        let mut ws = match DeribitWebSocket::new(None, false, AccountType::FuturesCross).await {
            Ok(w) => w,
            Err(e) => { println!("  FAIL WS init -> {}", e); return tally; }
        };
        if ws.connect(AccountType::FuturesCross).await.is_ok() {
            match ws.subscribe_volatility_index("btc_usd").await {
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
                    let label = "deribit_volatility_index.btc_usd".to_string();
                    println!(
                        "    CH {} -> events={}, errors={}{}",
                        label, n, errors,
                        if n == 0 { " [ZERO EVENTS]" } else { "" }
                    );
                    if n == 0 { tally.zero_event_channels.push(label); }
                }
                Err(e) => println!("    FAIL subscribe deribit_volatility_index.btc_usd -> {}", e),
            }
            let _ = ws.disconnect().await;
        }
    }

    // NEW Channel 3: markprice.options.btc_usd
    {
        tally.channels += 1;
        let mut ws = match DeribitWebSocket::new(None, false, AccountType::FuturesCross).await {
            Ok(w) => w,
            Err(e) => { println!("  FAIL WS init -> {}", e); return tally; }
        };
        if ws.connect(AccountType::FuturesCross).await.is_ok() {
            match ws.subscribe_options_mark_prices("btc_usd").await {
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
                    let label = "markprice.options.btc_usd".to_string();
                    println!(
                        "    CH {} -> events={}, errors={}{}",
                        label, n, errors,
                        if n == 0 { " [ZERO EVENTS]" } else { "" }
                    );
                    if n == 0 { tally.zero_event_channels.push(label); }
                }
                Err(e) => println!("    FAIL subscribe markprice.options.btc_usd -> {}", e),
            }
            let _ = ws.disconnect().await;
        }
    }

    // NEW Channel 4: block_trade_confirmations
    {
        tally.channels += 1;
        let mut ws = match DeribitWebSocket::new(None, false, AccountType::FuturesCross).await {
            Ok(w) => w,
            Err(e) => { println!("  FAIL WS init -> {}", e); return tally; }
        };
        if ws.connect(AccountType::FuturesCross).await.is_ok() {
            match ws.subscribe_block_trades().await {
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
                    let label = "block_trade_confirmations".to_string();
                    println!(
                        "    CH {} -> events={}, errors={}{}",
                        label, n, errors,
                        if n == 0 { " [ZERO EVENTS]" } else { "" }
                    );
                    if n == 0 { tally.zero_event_channels.push(label); }
                }
                Err(e) => println!("    FAIL subscribe block_trade_confirmations -> {}", e),
            }
            let _ = ws.disconnect().await;
        }
    }

    tally
}

async fn test_htx_ws() -> WsTally {
    println!("\n── HTX WS ───────────────────────────────────────────────────");
    let mut tally = WsTally {
        exchange: "HTX".into(),
        channels: 0,
        subscribed: 0,
        events: 0,
        parse_errors: 0,
        zero_event_channels: Vec::new(),
    };

    // HTX IndexPriceKline: no valid WS topic exists (verified 2026-05-15).
    // subscribe() now returns WebSocketError::Subscription. Documented as REST-only.
    // Test replaced with HTX kline (market.BTC-USDT.kline.1min) to keep channel count.
    let duration = Duration::from_secs(10);

    // Channel 1: market.BTC-USDT.kline.1min — regular kline on linear-swap-ws
    {
        tally.channels += 1;
        let ws_result = HtxWebSocket::new(None, false, AccountType::FuturesCross);
        let mut ws = match ws_result {
            Ok(w) => w,
            Err(e) => { println!("  FAIL WS init -> {}", e); return tally; }
        };
        if ws.connect(AccountType::FuturesCross).await.is_ok() {
            let req = SubscriptionRequest::new(
                Symbol::new("BTC", "USDT"),
                StreamType::Kline { interval: "1min".to_string() },
            );
            let (ok, n, err, label) = ws_listen(&mut ws, req, duration, "market.BTC-USDT.kline.1min").await;
            if ok { tally.subscribed += 1; }
            tally.events += n;
            tally.parse_errors += err;
            if ok && n == 0 { tally.zero_event_channels.push(label); }
            let _ = ws.disconnect().await;
        } else {
            println!("  FAIL: HTX WS connect");
        }
    }

    tally
}

async fn test_kucoin_ws() -> WsTally {
    println!("\n── KuCoin WS ────────────────────────────────────────────────");
    let mut tally = WsTally {
        exchange: "KuCoin".into(),
        channels: 0,
        subscribed: 0,
        events: 0,
        parse_errors: 0,
        zero_event_channels: Vec::new(),
    };

    let duration = Duration::from_secs(5);

    // NEW Channel 1: /contractMarket/indexPrice:XBTUSDTM
    {
        tally.channels += 1;
        let ws_result = KuCoinWebSocket::new(None, false, AccountType::FuturesCross).await;
        let mut ws = match ws_result {
            Ok(w) => w,
            Err(e) => { println!("  FAIL WS init -> {}", e); return tally; }
        };
        if ws.connect(AccountType::FuturesCross).await.is_ok() {
            // BTC/USDT with FuturesCross maps to XBTUSDTM via format_symbol.
            let mut req = SubscriptionRequest::new(
                Symbol::new("BTC", "USDT"),
                StreamType::IndexPrice,
            );
            req.account_type = AccountType::FuturesCross;
            let (ok, n, err, label) = ws_listen(&mut ws, req, duration, "/contractMarket/indexPrice:XBTUSDTM").await;
            if ok { tally.subscribed += 1; }
            tally.events += n;
            tally.parse_errors += err;
            if ok && n == 0 { tally.zero_event_channels.push(label); }
            let _ = ws.disconnect().await;
        } else {
            println!("  FAIL: KuCoin WS connect");
        }
    }

    tally
}

async fn test_gateio_ws() -> WsTally {
    println!("\n── Gate.io WS ───────────────────────────────────────────────");
    let mut tally = WsTally {
        exchange: "Gate.io".into(),
        channels: 0,
        subscribed: 0,
        events: 0,
        parse_errors: 0,
        zero_event_channels: Vec::new(),
    };

    // Gate.io PremiumIndexKline via WS: not available (verified 2026-05-15).
    // "futures.premium_index" removed; "premium_index_CONTRACT" on futures.candlesticks
    // returns "unknown currency pair". Replaced with futures.candlesticks BTC_USDT.
    let duration = Duration::from_secs(10);

    // Channel 1: futures.candlesticks BTC_USDT 1m — regular futures kline
    {
        tally.channels += 1;
        let ws_result = GateioWebSocket::new(None, false, AccountType::FuturesCross).await;
        let mut ws = match ws_result {
            Ok(w) => w,
            Err(e) => { println!("  FAIL WS init -> {}", e); return tally; }
        };
        if ws.connect(AccountType::FuturesCross).await.is_ok() {
            let req = SubscriptionRequest::new(
                Symbol::new("BTC", "USDT"),
                StreamType::Kline { interval: "1m".to_string() },
            );
            let (ok, n, err, label) = ws_listen(&mut ws, req, duration, "futures.candlesticks BTC_USDT 1m").await;
            if ok { tally.subscribed += 1; }
            tally.events += n;
            tally.parse_errors += err;
            if ok && n == 0 { tally.zero_event_channels.push(label); }
            let _ = ws.disconnect().await;
        } else {
            println!("  FAIL: Gate.io WS connect");
        }
    }

    tally
}

async fn test_crypto_com_ws() -> WsTally {
    println!("\n── Crypto.com WS ────────────────────────────────────────────");
    let mut tally = WsTally {
        exchange: "Crypto.com".into(),
        channels: 0,
        subscribed: 0,
        events: 0,
        parse_errors: 0,
        zero_event_channels: Vec::new(),
    };

    let duration = Duration::from_secs(5);
    let btcusd_perp = Symbol::new("BTCUSD", "PERP");

    // NEW Channel 1: estimatedfunding.BTCUSD-PERP — PredictedFunding
    {
        tally.channels += 1;
        let mut ws = CryptoComWebSocket::new(None, false);
        // CryptoComWebSocket::connect() takes no AccountType arg
        if ws.connect().await.is_ok() {
            let req = SubscriptionRequest::new(btcusd_perp.clone(), StreamType::PredictedFunding);
            let (ok, n, err, label) = ws_listen(&mut ws, req, duration, "estimatedfunding.BTCUSD-PERP").await;
            if ok { tally.subscribed += 1; }
            tally.events += n;
            tally.parse_errors += err;
            if ok && n == 0 { tally.zero_event_channels.push(label); }
            let _ = ws.disconnect().await;
        } else {
            println!("  FAIL: Crypto.com WS connect");
        }
    }

    // NEW Channel 2: settlement.BTCUSD-PERP — SettlementEvent (likely quiet)
    {
        tally.channels += 1;
        let mut ws = CryptoComWebSocket::new(None, false);
        if ws.connect().await.is_ok() {
            let req = SubscriptionRequest::new(btcusd_perp.clone(), StreamType::SettlementEvent);
            let (ok, n, err, label) = ws_listen(&mut ws, req, duration, "settlement.BTCUSD-PERP").await;
            if ok { tally.subscribed += 1; }
            tally.events += n;
            tally.parse_errors += err;
            if ok && n == 0 { tally.zero_event_channels.push(label); }
            let _ = ws.disconnect().await;
        }
    }

    tally
}

async fn test_bitfinex_ws() -> WsTally {
    println!("\n── Bitfinex WS ──────────────────────────────────────────────");
    let mut tally = WsTally {
        exchange: "Bitfinex".into(),
        channels: 0,
        subscribed: 0,
        events: 0,
        parse_errors: 0,
        zero_event_channels: Vec::new(),
    };

    let duration = Duration::from_secs(5);

    // NEW Channel 1: L3 book R0 for tBTCUSD
    {
        tally.channels += 1;
        let ws_result = BitfinexWebSocket::new(None, false, AccountType::Spot).await;
        let mut ws = match ws_result {
            Ok(w) => w,
            Err(e) => { println!("  FAIL WS init -> {}", e); return tally; }
        };
        if ws.connect(AccountType::Spot).await.is_ok() {
            match ws.subscribe_l3_book(Symbol::new("BTC", "USD"), 25).await {
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
                    let label = "book R0 tBTCUSD (L3)".to_string();
                    println!(
                        "    CH {} -> events={}, errors={}{}",
                        label, n, errors,
                        if n == 0 { " [ZERO EVENTS]" } else { "" }
                    );
                    if n == 0 { tally.zero_event_channels.push(label); }
                }
                Err(e) => println!("    FAIL subscribe book R0 tBTCUSD -> {}", e),
            }
            let _ = ws.disconnect().await;
        } else {
            println!("  FAIL: Bitfinex WS connect");
        }
    }

    // NEW Channel 2: funding book fUSD
    {
        tally.channels += 1;
        let ws_result = BitfinexWebSocket::new(None, false, AccountType::Spot).await;
        let mut ws = match ws_result {
            Ok(w) => w,
            Err(e) => { println!("  FAIL WS init -> {}", e); return tally; }
        };
        if ws.connect(AccountType::Spot).await.is_ok() {
            match ws.subscribe_funding_book("fUSD").await {
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
                    let label = "funding book fUSD".to_string();
                    println!(
                        "    CH {} -> events={}, errors={}{}",
                        label, n, errors,
                        if n == 0 { " [ZERO EVENTS]" } else { "" }
                    );
                    if n == 0 { tally.zero_event_channels.push(label); }
                }
                Err(e) => println!("    FAIL subscribe funding book fUSD -> {}", e),
            }
            let _ = ws.disconnect().await;
        }
    }

    // Channel 3: status deriv:tBTCF0:USTF0 (multi-emit: MarkPrice + FundingRate + OI + InsuranceFund)
    // Use Symbol("BTC", "USDT") + FuturesCross so format_symbol produces "tBTCF0:USTF0" correctly.
    {
        tally.channels += 1;
        let ws_result = BitfinexWebSocket::new(None, false, AccountType::FuturesCross).await;
        let mut ws = match ws_result {
            Ok(w) => w,
            Err(e) => { println!("  FAIL WS init -> {}", e); return tally; }
        };
        if ws.connect(AccountType::FuturesCross).await.is_ok() {
            // FundingRate StreamType maps to status channel with key "deriv:tBTCF0:USTF0".
            // format_symbol("BTC", "USDT", FuturesCross) = "tBTCF0:USTF0" (correct).
            let mut req = SubscriptionRequest::new(Symbol::new("BTC", "USDT"), StreamType::FundingRate);
            req.account_type = AccountType::FuturesCross;
            let (ok, n, err, label) = ws_listen(&mut ws, req, duration, "status deriv:tBTCF0:USTF0").await;
            if ok { tally.subscribed += 1; }
            tally.events += n;
            tally.parse_errors += err;
            if ok && n == 0 { tally.zero_event_channels.push(label); }
            let _ = ws.disconnect().await;
        }
    }

    tally
}

async fn test_gemini_ws() -> WsTally {
    println!("\n── Gemini WS ────────────────────────────────────────────────");
    let mut tally = WsTally {
        exchange: "Gemini".into(),
        channels: 0,
        subscribed: 0,
        events: 0,
        parse_errors: 0,
        zero_event_channels: Vec::new(),
    };

    let duration = Duration::from_secs(5);

    // NEW Channel 1: auction for btcusd — AuctionEvent
    {
        tally.channels += 1;
        let ws_result = GeminiWebSocket::new_market_data(false).await;
        let ws = match ws_result {
            Ok(w) => w,
            Err(e) => { println!("  FAIL WS init -> {}", e); return tally; }
        };
        // GeminiWebSocket has inherent connect() with no args (inherent method shadows trait)
        if ws.connect().await.is_ok() {
            match ws.subscribe_auction(Symbol::new("BTC", "USD")).await {
                Ok(_) => {
                    tally.subscribed += 1;
                    // Use trait event_stream (returns Pin<Box<dyn Stream>>) via UFCS
                    let mut stream = <GeminiWebSocket as WebSocketConnector>::event_stream(&ws);
                    let mut n = 0usize;
                    let mut errors = 0usize;
                    let _ = timeout(duration, async {
                        while let Some(item) = stream.next().await {
                            match item { Ok(_) => n += 1, Err(_) => errors += 1 }
                        }
                    }).await;
                    tally.events += n;
                    tally.parse_errors += errors;
                    let label = "auction btcusd".to_string();
                    println!(
                        "    CH {} -> events={}, errors={}{}",
                        label, n, errors,
                        if n == 0 { " [ZERO EVENTS]" } else { "" }
                    );
                    if n == 0 { tally.zero_event_channels.push(label); }
                }
                Err(e) => println!("    FAIL subscribe auction btcusd -> {}", e),
            }
            let _ = ws.disconnect().await;
        } else {
            println!("  FAIL: Gemini WS connect");
        }
    }

    tally
}

async fn test_bitstamp_ws() -> WsTally {
    println!("\n── Bitstamp WS ──────────────────────────────────────────────");
    let mut tally = WsTally {
        exchange: "Bitstamp".into(),
        channels: 0,
        subscribed: 0,
        events: 0,
        parse_errors: 0,
        zero_event_channels: Vec::new(),
    };

    let duration = Duration::from_secs(5);

    // NEW Channel 1: detail_order_book_btcusd — OrderbookL3
    {
        tally.channels += 1;
        let ws_result = BitstampWebSocket::new().await;
        let mut ws = match ws_result {
            Ok(w) => w,
            Err(e) => { println!("  FAIL WS init -> {}", e); return tally; }
        };
        if ws.connect(AccountType::Spot).await.is_ok() {
            let req = SubscriptionRequest::new(Symbol::new("BTC", "USD"), StreamType::OrderbookL3);
            let (ok, n, err, label) = ws_listen(&mut ws, req, duration, "detail_order_book_btcusd").await;
            if ok { tally.subscribed += 1; }
            tally.events += n;
            tally.parse_errors += err;
            if ok && n == 0 { tally.zero_event_channels.push(label); }
            let _ = ws.disconnect().await;
        } else {
            println!("  FAIL: Bitstamp WS connect");
        }
    }

    tally
}

async fn test_coinbase_ws() -> WsTally {
    println!("\n── Coinbase WS ──────────────────────────────────────────────");
    let mut tally = WsTally {
        exchange: "Coinbase".into(),
        channels: 0,
        subscribed: 0,
        events: 0,
        parse_errors: 0,
        zero_event_channels: Vec::new(),
    };

    let duration = Duration::from_secs(5);

    // NEW Channel 1: rfq_matches — BlockTrade
    // Note: Coinbase subscribe() doesn't map BlockTrade/rfq_matches via StreamType.
    // The rfq_matches parser exists in the WS handler but subscription is not exposed
    // through the standard trait interface. Report as architectural gap.
    tally.channels += 1;
    println!("    NOTE: rfq_matches/BlockTrade not subscriptable via standard StreamType trait on Coinbase WS.");
    println!("    NOTE: Parser exists for rfq_matches channel but no subscribe() mapping — architectural gap.");

    // Subscribe to Ticker to confirm WS connectivity is working.
    {
        let ws_result = CoinbaseWebSocket::new(None).await;
        let mut ws = match ws_result {
            Ok(w) => w,
            Err(e) => { println!("  FAIL WS init -> {}", e); return tally; }
        };
        if ws.connect(AccountType::Spot).await.is_ok() {
            let req = SubscriptionRequest::new(Symbol::new("BTC", "USD"), StreamType::Ticker);
            let (ok, n, err, label) = ws_listen(&mut ws, req, duration, "ticker BTC-USD (connectivity check)").await;
            if ok { tally.subscribed += 1; }
            tally.events += n;
            tally.parse_errors += err;
            if ok && n == 0 { tally.zero_event_channels.push(label); }
            let _ = ws.disconnect().await;
        } else {
            println!("  FAIL: Coinbase WS connect");
        }
    }

    tally
}

async fn test_kraken_ws() -> WsTally {
    println!("\n── Kraken WS ────────────────────────────────────────────────");
    let mut tally = WsTally {
        exchange: "Kraken".into(),
        channels: 0,
        subscribed: 0,
        events: 0,
        parse_errors: 0,
        zero_event_channels: Vec::new(),
    };

    let duration = Duration::from_secs(5);

    // NEW Channel 1: instrument channel — MarketWarning
    {
        tally.channels += 1;
        let ws_result = KrakenWebSocket::new(None, AccountType::Spot).await;
        let mut ws = match ws_result {
            Ok(w) => w,
            Err(e) => { println!("  FAIL WS init -> {}", e); return tally; }
        };
        if ws.connect(AccountType::Spot).await.is_ok() {
            // MarketWarning maps to the "instrument" channel on Kraken
            let req = SubscriptionRequest::new(Symbol::new("BTC", "USD"), StreamType::MarketWarning);
            let (ok, n, err, label) = ws_listen(&mut ws, req, duration, "instrument (MarketWarning)").await;
            if ok { tally.subscribed += 1; }
            tally.events += n;
            tally.parse_errors += err;
            if ok && n == 0 { tally.zero_event_channels.push(label); }
            let _ = ws.disconnect().await;
        } else {
            println!("  FAIL: Kraken WS connect");
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
    println!("WS channels run for 8 seconds each (sequential).");

    // ── Section A: REST ──────────────────────────────────────────────────────
    println!("\n══════════════════ Section A: REST ══════════════════");

    let binance_rest   = test_binance_rest().await;
    let bybit_rest     = test_bybit_rest().await;
    let okx_rest       = test_okx_rest().await;
    let hl_rest        = test_hyperliquid_rest().await;
    let deribit_rest   = test_deribit_rest().await;
    let bitget_rest    = test_bitget_rest().await;
    let htx_rest       = test_htx_rest().await;
    let kucoin_rest    = test_kucoin_rest().await;
    let gateio_rest    = test_gateio_rest().await;
    let dydx_rest      = test_dydx_rest().await;
    let lighter_rest   = test_lighter_rest().await;
    let bitfinex_rest  = test_bitfinex_rest().await;
    let kraken_rest    = test_kraken_rest().await;
    let gemini_rest    = test_gemini_rest().await;
    let bitstamp_rest  = test_bitstamp_rest().await;
    let upbit_rest     = test_upbit_rest().await;
    let crypto_com_rest = test_crypto_com_rest().await;
    let bingx_rest     = test_bingx_rest().await;
    let mexc_note      = test_mexc_note();

    let rest_tallies = vec![
        binance_rest,
        bybit_rest,
        okx_rest,
        hl_rest,
        deribit_rest,
        bitget_rest,
        htx_rest,
        kucoin_rest,
        gateio_rest,
        dydx_rest,
        lighter_rest,
        bitfinex_rest,
        kraken_rest,
        gemini_rest,
        bitstamp_rest,
        upbit_rest,
        crypto_com_rest,
        bingx_rest,
        mexc_note,
    ];

    // ── Section B: WebSocket ─────────────────────────────────────────────────
    println!("\n══════════════════ Section B: WebSocket ══════════════════");
    println!("(each channel listens 8 s — sequential to avoid port exhaustion)");

    let binance_ws   = test_binance_ws().await;
    let bybit_ws     = test_bybit_ws().await;
    let okx_ws       = test_okx_ws().await;
    let hl_ws        = test_hyperliquid_ws().await;
    let deribit_ws   = test_deribit_ws().await;
    let htx_ws       = test_htx_ws().await;
    let kucoin_ws    = test_kucoin_ws().await;
    let gateio_ws    = test_gateio_ws().await;
    let crypto_com_ws = test_crypto_com_ws().await;
    let bitfinex_ws  = test_bitfinex_ws().await;
    let gemini_ws    = test_gemini_ws().await;
    let bitstamp_ws  = test_bitstamp_ws().await;
    let coinbase_ws  = test_coinbase_ws().await;
    let kraken_ws    = test_kraken_ws().await;

    let ws_tallies = vec![
        binance_ws,
        bybit_ws,
        okx_ws,
        hl_ws,
        deribit_ws,
        htx_ws,
        kucoin_ws,
        gateio_ws,
        crypto_com_ws,
        bitfinex_ws,
        gemini_ws,
        bitstamp_ws,
        coinbase_ws,
        kraken_ws,
    ];

    // ── Section C: Summary ───────────────────────────────────────────────────
    println!("\n══════════════════ Section C: Summary ══════════════════");
    print_rest_summary(&rest_tallies);
    print_ws_summary(&ws_tallies);

    println!("\nDone.");
}
