//! # wasm_rest_parity — one REST call per REST-override-wired venue, in-browser.
//!
//! Proves that every venue whose `ConnectorFactory::create_public` accepts
//! `rest_override: Option<String>` can serve real REST data through a CORS proxy
//! from a browser context.
//!
//! Each test is INDEPENDENT: its own `ExchangeHub`, its own override, its own
//! `connect_public` call.  No shared state between tests.
//!
//! The `rest_lighter` test is an exception: Lighter has `Access-Control-Allow-Origin: *`
//! confirmed on every endpoint, so no override is needed — it dials the real API
//! directly from the browser.
//!
//! ## CORS proxy template
//!
//! Configurable at build time via `DIG3_CORS_PROXY`:
//!
//! ```sh
//! DIG3_CORS_PROXY="https://my-proxy.example.com/?url={url}" \
//!   cargo test --target wasm32-unknown-unknown -p digdigdig3-station \
//!       --test wasm_rest_parity
//! ```
//!
//! The `{url}` placeholder is percent-encoded by `assemble_rest_url` and replaced
//! with the full exchange target before the request is sent.
//! Default: `https://api.codetabs.com/v1/proxy/?quest={url}`.
//!
//! ## Run
//!
//! ```sh
//! WASM_BINDGEN_TEST_TIMEOUT=600 \
//! cargo test --target wasm32-unknown-unknown -p digdigdig3-station \
//!     --test wasm_rest_parity -- --nocapture
//! ```

#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{AccountType, ExchangeId, Symbol, SymbolInput};

// ─── CORS proxy template (verbatim copy from wasm_wave3_e2e.rs) ──────────────

/// Returns the CORS proxy URL template for REST calls in browser context.
///
/// The template MUST contain `{url}` — `assemble_rest_url` percent-encodes the
/// full target URL and substitutes it into `{url}` before the request is made.
///
/// ## Configuring a custom proxy (compile-time):
///
///   ```sh
///   DIG3_CORS_PROXY="https://my-proxy.example.com/?url={url}" \
///     cargo test --target wasm32-unknown-unknown -p digdigdig3-station
///   ```
///
/// Production consumers should pass their own backend proxy via
/// `DIG3_CORS_PROXY` at build time, or call `ExchangeHub::set_rest_base_override`
/// at runtime. The public codetabs.com default is for unattended CI only and
/// may be rate-limited.
fn cors_proxy_template() -> &'static str {
    option_env!("DIG3_CORS_PROXY").unwrap_or("https://api.codetabs.com/v1/proxy/?quest={url}")
}

// ─── Helper: canonical BTC/USDT spot symbol ──────────────────────────────────

fn btc_usdt() -> Symbol {
    Symbol::new("BTC", "USDT")
}

fn btc_usd() -> Symbol {
    Symbol::new("BTC", "USD")
}

// ─── Binance ─────────────────────────────────────────────────────────────────
//
// Canonical BTC/USDT Spot → normalizer → "BTCUSDT".
// Use get_klines (proven path: wasm_wave3_e2e.rs rest_via_encoded_proxy_get_klines).
// Ticker also works but klines is the battle-tested proxy path.

#[wasm_bindgen_test]
async fn rest_binance() {
    let hub = ExchangeHub::new();
    hub.set_rest_base_override(ExchangeId::Binance, cors_proxy_template().to_string());
    hub.connect_public(ExchangeId::Binance, false)
        .await
        .expect("Binance connect_public via CORS proxy");

    let rest = hub
        .rest(ExchangeId::Binance)
        .expect("Binance REST connector present after connect_public");

    let sym = btc_usdt();
    let klines = rest
        .get_klines(
            SymbolInput::Canonical(&sym),
            "1m",
            Some(1),
            AccountType::Spot,
            None,
        )
        .await
        .expect("Binance get_klines via CORS proxy");

    web_sys::console::log_1(
        &format!("rest_binance: {} kline(s), open={}", klines.len(), klines.first().map(|k| k.open).unwrap_or(0.0)).into(),
    );

    assert!(!klines.is_empty(), "Binance: expected ≥1 kline; got 0");
    assert!(
        klines[0].open > 0.0,
        "Binance: kline.open must be positive; got {}",
        klines[0].open
    );
}

// ─── Bybit ───────────────────────────────────────────────────────────────────
//
// Canonical BTC/USDT Spot → normalizer → "BTCUSDT".
// get_ticker: spot ticker is well-supported and returns last/bid/ask.

#[wasm_bindgen_test]
async fn rest_bybit() {
    let hub = ExchangeHub::new();
    hub.set_rest_base_override(ExchangeId::Bybit, cors_proxy_template().to_string());
    hub.connect_public(ExchangeId::Bybit, false)
        .await
        .expect("Bybit connect_public via CORS proxy");

    let rest = hub
        .rest(ExchangeId::Bybit)
        .expect("Bybit REST connector present after connect_public");

    let sym = btc_usdt();
    let ticker = rest
        .get_ticker(SymbolInput::Canonical(&sym), AccountType::Spot)
        .await
        .expect("Bybit get_ticker via CORS proxy");

    web_sys::console::log_1(
        &format!("rest_bybit: last={:.4}", ticker.last_price).into(),
    );

    assert!(
        ticker.last_price > 0.0,
        "Bybit: last_price must be positive; got {}",
        ticker.last_price
    );
}

// ─── OKX ─────────────────────────────────────────────────────────────────────
//
// Canonical BTC/USDT Spot → normalizer → "BTC-USDT" (OKX uses dash separator).
// get_ticker Spot.

#[wasm_bindgen_test]
async fn rest_okx() {
    let hub = ExchangeHub::new();
    hub.set_rest_base_override(ExchangeId::OKX, cors_proxy_template().to_string());
    hub.connect_public(ExchangeId::OKX, false)
        .await
        .expect("OKX connect_public via CORS proxy");

    let rest = hub
        .rest(ExchangeId::OKX)
        .expect("OKX REST connector present after connect_public");

    let sym = btc_usdt();
    let ticker = rest
        .get_ticker(SymbolInput::Canonical(&sym), AccountType::Spot)
        .await
        .expect("OKX get_ticker via CORS proxy");

    web_sys::console::log_1(
        &format!("rest_okx: last={:.4}", ticker.last_price).into(),
    );

    assert!(
        ticker.last_price > 0.0,
        "OKX: last_price must be positive; got {}",
        ticker.last_price
    );
}

// ─── Bitget ──────────────────────────────────────────────────────────────────
//
// Canonical BTC/USDT Spot → normalizer → "BTCUSDT".
// e2e_smoke raw_symbol_for default: BTC/USDT Spot.
// get_ticker Spot.

#[wasm_bindgen_test]
async fn rest_bitget() {
    let hub = ExchangeHub::new();
    hub.set_rest_base_override(ExchangeId::Bitget, cors_proxy_template().to_string());
    hub.connect_public(ExchangeId::Bitget, false)
        .await
        .expect("Bitget connect_public via CORS proxy");

    let rest = hub
        .rest(ExchangeId::Bitget)
        .expect("Bitget REST connector present after connect_public");

    let sym = btc_usdt();
    let ticker = rest
        .get_ticker(SymbolInput::Canonical(&sym), AccountType::Spot)
        .await
        .expect("Bitget get_ticker via CORS proxy");

    web_sys::console::log_1(
        &format!("rest_bitget: last={:.4}", ticker.last_price).into(),
    );

    assert!(
        ticker.last_price > 0.0,
        "Bitget: last_price must be positive; got {}",
        ticker.last_price
    );
}

// ─── Bitstamp ────────────────────────────────────────────────────────────────
//
// Bitstamp is BTC/USD (no USDT pairs at top level).
// e2e_smoke: `make(btc_usd, AccountType::Spot)` → normalizer → "btcusd".
// Canonical BTC/USD Spot.

#[wasm_bindgen_test]
async fn rest_bitstamp() {
    let hub = ExchangeHub::new();
    hub.set_rest_base_override(ExchangeId::Bitstamp, cors_proxy_template().to_string());
    hub.connect_public(ExchangeId::Bitstamp, false)
        .await
        .expect("Bitstamp connect_public via CORS proxy");

    let rest = hub
        .rest(ExchangeId::Bitstamp)
        .expect("Bitstamp REST connector present after connect_public");

    let sym = btc_usd();
    let ticker = rest
        .get_ticker(SymbolInput::Canonical(&sym), AccountType::Spot)
        .await
        .expect("Bitstamp get_ticker via CORS proxy");

    web_sys::console::log_1(
        &format!("rest_bitstamp: last={:.4}", ticker.last_price).into(),
    );

    assert!(
        ticker.last_price > 0.0,
        "Bitstamp: last_price must be positive; got {}",
        ticker.last_price
    );
}

// ─── Coinbase ─────────────────────────────────────────────────────────────────
//
// Coinbase uses BTC/USD (USD, not USDT).
// e2e_smoke: `make(btc_usd, AccountType::Spot)` → normalizer → "BTC-USD".
// Canonical BTC/USD Spot.

#[wasm_bindgen_test]
async fn rest_coinbase() {
    let hub = ExchangeHub::new();
    hub.set_rest_base_override(ExchangeId::Coinbase, cors_proxy_template().to_string());
    hub.connect_public(ExchangeId::Coinbase, false)
        .await
        .expect("Coinbase connect_public via CORS proxy");

    let rest = hub
        .rest(ExchangeId::Coinbase)
        .expect("Coinbase REST connector present after connect_public");

    let sym = btc_usd();
    let ticker = rest
        .get_ticker(SymbolInput::Canonical(&sym), AccountType::Spot)
        .await
        .expect("Coinbase get_ticker via CORS proxy");

    web_sys::console::log_1(
        &format!("rest_coinbase: last={:.4}", ticker.last_price).into(),
    );

    assert!(
        ticker.last_price > 0.0,
        "Coinbase: last_price must be positive; got {}",
        ticker.last_price
    );
}

// ─── Kraken ───────────────────────────────────────────────────────────────────
//
// Kraken REST uses "XBTUSD" format (SymbolNormalizer::to_exchange for REST).
// e2e_smoke raw_symbol_for: BTC/USD Spot (normalizer produces XBTUSD for REST).
// Note: the WS override in wasm_e2e_matrix uses "BTC/USD" (slash format) but
// that's WS-only. For REST, canonical BTC/USD Spot → normalizer → "XBTUSD".

#[wasm_bindgen_test]
async fn rest_kraken() {
    let hub = ExchangeHub::new();
    hub.set_rest_base_override(ExchangeId::Kraken, cors_proxy_template().to_string());
    hub.connect_public(ExchangeId::Kraken, false)
        .await
        .expect("Kraken connect_public via CORS proxy");

    let rest = hub
        .rest(ExchangeId::Kraken)
        .expect("Kraken REST connector present after connect_public");

    let sym = btc_usd();
    let ticker = rest
        .get_ticker(SymbolInput::Canonical(&sym), AccountType::Spot)
        .await
        .expect("Kraken get_ticker via CORS proxy");

    web_sys::console::log_1(
        &format!("rest_kraken: last={:.4}", ticker.last_price).into(),
    );

    assert!(
        ticker.last_price > 0.0,
        "Kraken: last_price must be positive; got {}",
        ticker.last_price
    );
}

// ─── Deribit ─────────────────────────────────────────────────────────────────
//
// Deribit is a derivatives exchange. No BTC/USDT spot ticker.
// e2e_smoke: `make(btc_usd, AccountType::FuturesCross)` → normalizer → "BTC-PERPETUAL".
// Use get_klines FuturesCross: perpetual contract kline is always available.
// Canonical BTC/USD FuturesCross.

#[wasm_bindgen_test]
async fn rest_deribit() {
    let hub = ExchangeHub::new();
    hub.set_rest_base_override(ExchangeId::Deribit, cors_proxy_template().to_string());
    hub.connect_public(ExchangeId::Deribit, false)
        .await
        .expect("Deribit connect_public via CORS proxy");

    let rest = hub
        .rest(ExchangeId::Deribit)
        .expect("Deribit REST connector present after connect_public");

    // Deribit: BTC/USD FuturesCross → normalizer → "BTC-PERPETUAL"
    let sym = btc_usd();
    let klines = rest
        .get_klines(
            SymbolInput::Canonical(&sym),
            "1m",
            Some(1),
            AccountType::FuturesCross,
            None,
        )
        .await
        .expect("Deribit get_klines (BTC-PERPETUAL) via CORS proxy");

    web_sys::console::log_1(
        &format!("rest_deribit: {} kline(s), open={}", klines.len(), klines.first().map(|k| k.open).unwrap_or(0.0)).into(),
    );

    assert!(!klines.is_empty(), "Deribit: expected ≥1 kline; got 0");
    assert!(
        klines[0].open > 0.0,
        "Deribit: kline.open must be positive; got {}",
        klines[0].open
    );
}

// ─── HTX (formerly Huobi) ────────────────────────────────────────────────────
//
// Canonical BTC/USDT Spot → normalizer → "btcusdt" (HTX lowercases).
// e2e_smoke raw_symbol_for default: BTC/USDT Spot.

#[wasm_bindgen_test]
async fn rest_htx() {
    let hub = ExchangeHub::new();
    hub.set_rest_base_override(ExchangeId::HTX, cors_proxy_template().to_string());
    hub.connect_public(ExchangeId::HTX, false)
        .await
        .expect("HTX connect_public via CORS proxy");

    let rest = hub
        .rest(ExchangeId::HTX)
        .expect("HTX REST connector present after connect_public");

    let sym = btc_usdt();
    let ticker = rest
        .get_ticker(SymbolInput::Canonical(&sym), AccountType::Spot)
        .await
        .expect("HTX get_ticker via CORS proxy");

    web_sys::console::log_1(
        &format!("rest_htx: last={:.4}", ticker.last_price).into(),
    );

    assert!(
        ticker.last_price > 0.0,
        "HTX: last_price must be positive; got {}",
        ticker.last_price
    );
}

// ─── KuCoin ──────────────────────────────────────────────────────────────────
//
// Canonical BTC/USDT Spot → normalizer → "BTC-USDT" (KuCoin dash separator).
// e2e_smoke: `make(btc_usdt, AccountType::Spot)`.
// get_ticker Spot.

#[wasm_bindgen_test]
async fn rest_kucoin() {
    let hub = ExchangeHub::new();
    hub.set_rest_base_override(ExchangeId::KuCoin, cors_proxy_template().to_string());
    hub.connect_public(ExchangeId::KuCoin, false)
        .await
        .expect("KuCoin connect_public via CORS proxy");

    let rest = hub
        .rest(ExchangeId::KuCoin)
        .expect("KuCoin REST connector present after connect_public");

    let sym = btc_usdt();
    let ticker = rest
        .get_ticker(SymbolInput::Canonical(&sym), AccountType::Spot)
        .await
        .expect("KuCoin get_ticker via CORS proxy");

    web_sys::console::log_1(
        &format!("rest_kucoin: last={:.4}", ticker.last_price).into(),
    );

    assert!(
        ticker.last_price > 0.0,
        "KuCoin: last_price must be positive; got {}",
        ticker.last_price
    );
}

// ─── MEXC ────────────────────────────────────────────────────────────────────
//
// Canonical BTC/USDT Spot → normalizer → "BTCUSDT".
// e2e_smoke raw_symbol_for default: BTC/USDT Spot.

#[wasm_bindgen_test]
async fn rest_mexc() {
    let hub = ExchangeHub::new();
    hub.set_rest_base_override(ExchangeId::MEXC, cors_proxy_template().to_string());
    hub.connect_public(ExchangeId::MEXC, false)
        .await
        .expect("MEXC connect_public via CORS proxy");

    let rest = hub
        .rest(ExchangeId::MEXC)
        .expect("MEXC REST connector present after connect_public");

    let sym = btc_usdt();
    let ticker = rest
        .get_ticker(SymbolInput::Canonical(&sym), AccountType::Spot)
        .await
        .expect("MEXC get_ticker via CORS proxy");

    web_sys::console::log_1(
        &format!("rest_mexc: last={:.4}", ticker.last_price).into(),
    );

    assert!(
        ticker.last_price > 0.0,
        "MEXC: last_price must be positive; got {}",
        ticker.last_price
    );
}

// ─── GateIO ──────────────────────────────────────────────────────────────────
//
// Canonical BTC/USDT Spot → normalizer → "BTC_USDT" (GateIO underscore separator).
// e2e_smoke: `make(btc_usdt, AccountType::Spot)`.

#[wasm_bindgen_test]
async fn rest_gateio() {
    let hub = ExchangeHub::new();
    hub.set_rest_base_override(ExchangeId::GateIO, cors_proxy_template().to_string());
    hub.connect_public(ExchangeId::GateIO, false)
        .await
        .expect("GateIO connect_public via CORS proxy");

    let rest = hub
        .rest(ExchangeId::GateIO)
        .expect("GateIO REST connector present after connect_public");

    let sym = btc_usdt();
    let ticker = rest
        .get_ticker(SymbolInput::Canonical(&sym), AccountType::Spot)
        .await
        .expect("GateIO get_ticker via CORS proxy");

    web_sys::console::log_1(
        &format!("rest_gateio: last={:.4}", ticker.last_price).into(),
    );

    assert!(
        ticker.last_price > 0.0,
        "GateIO: last_price must be positive; got {}",
        ticker.last_price
    );
}

// ─── Gemini ──────────────────────────────────────────────────────────────────
//
// Gemini is BTC/USD (USD pairs only on the standard API).
// e2e_smoke: `make(btc_usd, AccountType::Spot)` → normalizer → "BTCUSD".
// Canonical BTC/USD Spot.

#[wasm_bindgen_test]
async fn rest_gemini() {
    let hub = ExchangeHub::new();
    hub.set_rest_base_override(ExchangeId::Gemini, cors_proxy_template().to_string());
    hub.connect_public(ExchangeId::Gemini, false)
        .await
        .expect("Gemini connect_public via CORS proxy");

    let rest = hub
        .rest(ExchangeId::Gemini)
        .expect("Gemini REST connector present after connect_public");

    let sym = btc_usd();
    let ticker = rest
        .get_ticker(SymbolInput::Canonical(&sym), AccountType::Spot)
        .await
        .expect("Gemini get_ticker via CORS proxy");

    web_sys::console::log_1(
        &format!("rest_gemini: last={:.4}", ticker.last_price).into(),
    );

    assert!(
        ticker.last_price > 0.0,
        "Gemini: last_price must be positive; got {}",
        ticker.last_price
    );
}

// ─── BingX ───────────────────────────────────────────────────────────────────
//
// Canonical BTC/USDT Spot → normalizer → "BTC-USDT" (BingX dash separator).
// e2e_smoke: `make(btc_usdt, AccountType::Spot)`.

#[wasm_bindgen_test]
async fn rest_bingx() {
    let hub = ExchangeHub::new();
    hub.set_rest_base_override(ExchangeId::BingX, cors_proxy_template().to_string());
    hub.connect_public(ExchangeId::BingX, false)
        .await
        .expect("BingX connect_public via CORS proxy");

    let rest = hub
        .rest(ExchangeId::BingX)
        .expect("BingX REST connector present after connect_public");

    let sym = btc_usdt();
    let ticker = rest
        .get_ticker(SymbolInput::Canonical(&sym), AccountType::Spot)
        .await
        .expect("BingX get_ticker via CORS proxy");

    web_sys::console::log_1(
        &format!("rest_bingx: last={:.4}", ticker.last_price).into(),
    );

    assert!(
        ticker.last_price > 0.0,
        "BingX: last_price must be positive; got {}",
        ticker.last_price
    );
}

// ─── CryptoCom ───────────────────────────────────────────────────────────────
//
// Canonical BTC/USDT Spot → normalizer → "BTC_USDT" (Crypto.com underscore).
// e2e_smoke: `make(btc_usdt, AccountType::Spot)`.

#[wasm_bindgen_test]
async fn rest_cryptocom() {
    let hub = ExchangeHub::new();
    hub.set_rest_base_override(ExchangeId::CryptoCom, cors_proxy_template().to_string());
    hub.connect_public(ExchangeId::CryptoCom, false)
        .await
        .expect("CryptoCom connect_public via CORS proxy");

    let rest = hub
        .rest(ExchangeId::CryptoCom)
        .expect("CryptoCom REST connector present after connect_public");

    let sym = btc_usdt();
    let ticker = rest
        .get_ticker(SymbolInput::Canonical(&sym), AccountType::Spot)
        .await
        .expect("CryptoCom get_ticker via CORS proxy");

    web_sys::console::log_1(
        &format!("rest_cryptocom: last={:.4}", ticker.last_price).into(),
    );

    assert!(
        ticker.last_price > 0.0,
        "CryptoCom: last_price must be positive; got {}",
        ticker.last_price
    );
}

// ─── Upbit ───────────────────────────────────────────────────────────────────
//
// Upbit is KRW-denominated (Korean Won). BTC/KRW Spot.
// e2e_smoke: `Symbol::new("BTC","KRW")` → normalizer → "KRW-BTC" (Upbit format).
// Canonical BTC/KRW Spot.

#[wasm_bindgen_test]
async fn rest_upbit() {
    let hub = ExchangeHub::new();
    hub.set_rest_base_override(ExchangeId::Upbit, cors_proxy_template().to_string());
    hub.connect_public(ExchangeId::Upbit, false)
        .await
        .expect("Upbit connect_public via CORS proxy");

    let rest = hub
        .rest(ExchangeId::Upbit)
        .expect("Upbit REST connector present after connect_public");

    // Upbit: BTC/KRW Spot → normalizer → "KRW-BTC"
    let sym = Symbol::new("BTC", "KRW");
    let ticker = rest
        .get_ticker(SymbolInput::Canonical(&sym), AccountType::Spot)
        .await
        .expect("Upbit get_ticker via CORS proxy");

    web_sys::console::log_1(
        &format!("rest_upbit: last={:.4}", ticker.last_price).into(),
    );

    assert!(
        ticker.last_price > 0.0,
        "Upbit: last_price must be positive (KRW price ~100M); got {}",
        ticker.last_price
    );
}

// ─── Bitfinex ────────────────────────────────────────────────────────────────
//
// Bitfinex is BTC/USD (USD-denominated). No USDT at the standard spot level.
// e2e_smoke: `make(btc_usd, AccountType::Spot)` → normalizer → "tBTCUSD".
// Canonical BTC/USD Spot.

#[wasm_bindgen_test]
async fn rest_bitfinex() {
    let hub = ExchangeHub::new();
    hub.set_rest_base_override(ExchangeId::Bitfinex, cors_proxy_template().to_string());
    hub.connect_public(ExchangeId::Bitfinex, false)
        .await
        .expect("Bitfinex connect_public via CORS proxy");

    let rest = hub
        .rest(ExchangeId::Bitfinex)
        .expect("Bitfinex REST connector present after connect_public");

    let sym = btc_usd();
    let ticker = rest
        .get_ticker(SymbolInput::Canonical(&sym), AccountType::Spot)
        .await
        .expect("Bitfinex get_ticker via CORS proxy");

    web_sys::console::log_1(
        &format!("rest_bitfinex: last={:.4}", ticker.last_price).into(),
    );

    assert!(
        ticker.last_price > 0.0,
        "Bitfinex: last_price must be positive; got {}",
        ticker.last_price
    );
}

// ─── HyperLiquid ─────────────────────────────────────────────────────────────
//
// HyperLiquid is a DEX (perps only). BTC/USD FuturesCross.
// e2e_smoke: `make(btc_usd, AccountType::FuturesCross)` → normalizer → "BTC".
// Use get_klines FuturesCross since ticker for perp DEX may not be a 24h candle-stat ticker.
// Canonical BTC/USD FuturesCross.

#[wasm_bindgen_test]
async fn rest_hyperliquid() {
    let hub = ExchangeHub::new();
    hub.set_rest_base_override(ExchangeId::HyperLiquid, cors_proxy_template().to_string());
    hub.connect_public(ExchangeId::HyperLiquid, false)
        .await
        .expect("HyperLiquid connect_public via CORS proxy");

    let rest = hub
        .rest(ExchangeId::HyperLiquid)
        .expect("HyperLiquid REST connector present after connect_public");

    // HyperLiquid: BTC/USD FuturesCross → normalizer → "BTC"
    let sym = btc_usd();
    let klines = rest
        .get_klines(
            SymbolInput::Canonical(&sym),
            "1m",
            Some(1),
            AccountType::FuturesCross,
            None,
        )
        .await
        .expect("HyperLiquid get_klines via CORS proxy");

    web_sys::console::log_1(
        &format!("rest_hyperliquid: {} kline(s), open={}", klines.len(), klines.first().map(|k| k.open).unwrap_or(0.0)).into(),
    );

    assert!(!klines.is_empty(), "HyperLiquid: expected ≥1 kline; got 0");
    assert!(
        klines[0].open > 0.0,
        "HyperLiquid: kline.open must be positive; got {}",
        klines[0].open
    );
}

// ─── Lighter (no override — CORS * confirmed) ─────────────────────────────────
//
// Lighter has `Access-Control-Allow-Origin: *` on all endpoints (confirmed live
// 2026-05-28). No CORS proxy needed — the browser can dial the API directly.
//
// Canonical BTC/USDT Spot → normalizer → Lighter market_id (e.g. "1" or "BTC").
// e2e_smoke venue_symbols: `SymbolNormalizer::to_exchange(id, &Symbol::new("BTC","USDT"), Spot)`.
// Use get_klines Spot (Lighter kline confirmed live: open=73665, close=73633).

#[wasm_bindgen_test]
async fn rest_lighter() {
    let hub = ExchangeHub::new();
    // NO override — Lighter is CORS * and can be dialled directly from the browser.
    hub.connect_public(ExchangeId::Lighter, false)
        .await
        .expect("Lighter connect_public (direct, no proxy needed)");

    let rest = hub
        .rest(ExchangeId::Lighter)
        .expect("Lighter REST connector present after connect_public");

    // Lighter: BTC/USDT Spot → normalizer → market_id
    let sym = btc_usdt();
    let klines = rest
        .get_klines(
            SymbolInput::Canonical(&sym),
            "1m",
            Some(1),
            AccountType::Spot,
            None,
        )
        .await
        .expect("Lighter get_klines (direct)");

    web_sys::console::log_1(
        &format!("rest_lighter: {} kline(s), open={}", klines.len(), klines.first().map(|k| k.open).unwrap_or(0.0)).into(),
    );

    assert!(!klines.is_empty(), "Lighter: expected ≥1 kline; got 0");
    assert!(
        klines[0].open > 0.0,
        "Lighter: kline.open must be positive; got {}",
        klines[0].open
    );
}

// ─── dYdX v4 (Indexer REST via CORS proxy) ───────────────────────────────────
//
// dYdX v4 is a perps DEX; its public Indexer REST (https://indexer.dydx.trade/v4)
// is CORS-blocked in-browser like any CEX, so it needs the proxy override. Until
// 2026-05-30 the DydxConnector had NO rest_override field at all (the factory
// dropped the override on the Dydx arm) — wired now so dYdX reaches REST parity.
// Canonical BTC/USD FuturesCross → normalizer → "BTC-USD" perpetual market.

#[wasm_bindgen_test]
async fn rest_dydx() {
    let hub = ExchangeHub::new();
    hub.set_rest_base_override(ExchangeId::Dydx, cors_proxy_template().to_string());
    hub.connect_public(ExchangeId::Dydx, false)
        .await
        .expect("dYdX connect_public via CORS proxy");

    let rest = hub
        .rest(ExchangeId::Dydx)
        .expect("dYdX REST connector present after connect_public");

    let sym = btc_usd();
    let ticker = rest
        .get_ticker(SymbolInput::Canonical(&sym), AccountType::FuturesCross)
        .await
        .expect("dYdX get_ticker via CORS proxy");

    web_sys::console::log_1(
        &format!("rest_dydx: last={:.4}", ticker.last_price).into(),
    );

    assert!(
        ticker.last_price > 0.0,
        "dYdX: ticker.last_price must be positive; got {}",
        ticker.last_price
    );
}
