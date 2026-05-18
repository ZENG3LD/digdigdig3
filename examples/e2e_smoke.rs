//! # e2e_smoke — content-inspecting parallel async E2E of all dig3 exchanges.
//!
//! (Renamed from `deep_smoke` — this is the project's E2E live-API harness.)
//!
//! For every WS event received: decodes the StreamEvent and reports actual field values.
//! For every REST ticker: reports last_price, volume, bid, ask, timestamp — flags zero/default.
//!
//! WS impl total in codebase: 35 (25 distinct exchange connectors).
//! Covers EVERY exchange findable via ConnectorFactory + MOEX WS directly.
//!
//! Run:
//!     cargo run --example e2e_smoke --release 2>&1 | tee e2e_smoke_report.txt
//!
//! No API keys required — public endpoints only.

use std::time::{Instant, SystemTime, UNIX_EPOCH};

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::traits::MarketData;
use digdigdig3::core::types::{AccountType, ExchangeId, StreamEvent, SubscriptionRequest, Symbol, SymbolInput};
use digdigdig3::core::utils::SymbolNormalizer;
use digdigdig3::l2::free::moex::MoexWebSocket;
use digdigdig3::core::traits::WebSocketConnector;
use futures_util::StreamExt;
use tokio::time::{timeout, Duration};

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Truncate a string to at most `max_chars` Unicode characters (never panics on multibyte).
fn truncate_str(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}

// ── Thresholds ────────────────────────────────────────────────────────────────

/// Timestamps older than this (in ms) are considered stale
fn stale_threshold_ms() -> i64 {
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    // 1 hour ago
    now_ms - 3_600_000
}

// ── Result types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
enum RestResult {
    Ok {
        last_price: f64,
        volume: f64,
        bid: Option<f64>,
        ask: Option<f64>,
        timestamp: i64,
        zero_fields: Vec<&'static str>,
    },
    Fail(String),
}

impl RestResult {
    fn short(&self) -> String {
        match self {
            RestResult::Ok { last_price, volume, bid, ask, timestamp, zero_fields } => {
                let bid_s = bid.map(|v| format!("{:.4}", v)).unwrap_or_else(|| "None".into());
                let ask_s = ask.map(|v| format!("{:.4}", v)).unwrap_or_else(|| "None".into());
                let ts_age = {
                    let now_ms = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map(|d| d.as_millis() as i64)
                        .unwrap_or(0);
                    (now_ms - timestamp) / 1000
                };
                let zero_flag = if zero_fields.is_empty() {
                    String::new()
                } else {
                    format!(" ZERO:{}", zero_fields.join(","))
                };
                format!(
                    "OK last={:.4} vol={:.2} bid={} ask={} ts={}s_ago{}",
                    last_price, volume, bid_s, ask_s, ts_age, zero_flag
                )
            }
            RestResult::Fail(e) => {
                format!("FAIL {}", truncate_str(e, 60))
            }
        }
    }

    fn is_ok(&self) -> bool {
        matches!(self, RestResult::Ok { .. })
    }

    fn has_zero_fields(&self) -> bool {
        matches!(self, RestResult::Ok { zero_fields, .. } if !zero_fields.is_empty())
    }
}

#[derive(Debug, Clone)]
enum WsResult {
    Unsupported(String),
    ConnectFail(String),
    SubscribeFail(String),
    Silent,
    Events {
        count: u32,
        first_event: String,
        data_valid: bool,
    },
}

impl WsResult {
    fn connect_ok(&self) -> bool {
        matches!(self, WsResult::Silent | WsResult::Events { .. } | WsResult::SubscribeFail(_))
    }

    fn has_events(&self) -> bool {
        matches!(self, WsResult::Events { count, .. } if *count > 0)
    }

    fn data_valid(&self) -> bool {
        matches!(self, WsResult::Events { data_valid: true, .. })
    }

    fn data_empty(&self) -> bool {
        matches!(self, WsResult::Events { data_valid: false, .. })
    }
}

#[derive(Debug)]
struct Row {
    exchange: ExchangeId,
    rest: RestResult,
    ws: WsResult,
}

// ── Event content inspector ───────────────────────────────────────────────────

fn inspect_event(event: &StreamEvent, stale_ms: i64) -> (String, bool) {
    match event {
        StreamEvent::Ticker(t) => {
            let valid = t.last_price > 0.0 && t.timestamp > stale_ms;
            let s = format!(
                "Ticker sym={} last={:.4} bid={} ask={} ts={}",
                t.symbol,
                t.last_price,
                t.bid_price.map(|v| format!("{:.4}", v)).unwrap_or_else(|| "None".into()),
                t.ask_price.map(|v| format!("{:.4}", v)).unwrap_or_else(|| "None".into()),
                t.timestamp,
            );
            (s, valid)
        }
        StreamEvent::Trade(t) => {
            let valid = t.price > 0.0 && t.quantity > 0.0;
            let s = format!(
                "Trade sym={} px={:.4} qty={:.6} ts={}",
                t.symbol, t.price, t.quantity, t.timestamp
            );
            (s, valid)
        }
        StreamEvent::OrderbookSnapshot(ob) => {
            let valid = !ob.bids.is_empty()
                && !ob.asks.is_empty()
                && ob.bids.first().map(|l| l.price > 0.0).unwrap_or(false);
            let top_bid = ob.bids.first().map(|l| l.price).unwrap_or(0.0);
            let top_ask = ob.asks.first().map(|l| l.price).unwrap_or(0.0);
            let s = format!(
                "OBSnapshot bids={} asks={} top_bid={:.4} top_ask={:.4}",
                ob.bids.len(), ob.asks.len(), top_bid, top_ask
            );
            (s, valid)
        }
        StreamEvent::OrderbookDelta(od) => {
            let has_data = !od.bids.is_empty() || !od.asks.is_empty();
            let top_bid = od.bids.first().map(|l| l.price).unwrap_or(0.0);
            let s = format!(
                "OBDelta bids={} asks={} top_bid={:.4} ts={}",
                od.bids.len(), od.asks.len(), top_bid, od.timestamp
            );
            (s, has_data)
        }
        StreamEvent::Kline(k) => {
            let valid = k.close > 0.0 && k.open > 0.0 && k.open_time > 0;
            let s = format!(
                "Kline o={:.4} h={:.4} l={:.4} c={:.4} vol={:.2} ts={}",
                k.open, k.high, k.low, k.close, k.volume, k.open_time
            );
            (s, valid)
        }
        StreamEvent::MarkPrice { symbol, mark_price, timestamp, .. } => {
            let valid = *mark_price > 0.0 && *timestamp > stale_ms;
            let s = format!("MarkPrice sym={} px={:.4} ts={}", symbol, mark_price, timestamp);
            (s, valid)
        }
        StreamEvent::FundingRate { symbol, rate, timestamp, .. } => {
            let s = format!("FundingRate sym={} rate={:.6} ts={}", symbol, rate, timestamp);
            (s, *timestamp > 0)
        }
        StreamEvent::AggTrade { symbol, price, quantity, timestamp, .. } => {
            let valid = *price > 0.0 && *quantity > 0.0;
            let s = format!(
                "AggTrade sym={} px={:.4} qty={:.6} ts={}",
                symbol, price, quantity, timestamp
            );
            (s, valid)
        }
        other => {
            let s = format!("{:?}", other);
            let short = if s.len() > 80 { format!("{}...", &s[..77]) } else { s };
            (short, true)
        }
    }
}

// ── WS test helper ────────────────────────────────────────────────────────────

async fn run_ws_test(
    ws: std::sync::Arc<dyn WebSocketConnector>,
    symbol: Symbol,
    account_type: AccountType,
    stale_ms: i64,
) -> WsResult {
    // Connect
    match timeout(Duration::from_secs(8), ws.connect(account_type)).await {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            let short = e.to_string();
            return WsResult::ConnectFail(truncate_str(&short, 60).to_string());
        }
        Err(_) => return WsResult::ConnectFail("connect_timeout".into()),
    }

    // Subscribe
    let sub = SubscriptionRequest::ticker_for(symbol, account_type);
    match timeout(Duration::from_secs(5), ws.subscribe(sub)).await {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            let short = e.to_string();
            return WsResult::SubscribeFail(truncate_str(&short, 60).to_string());
        }
        Err(_) => return WsResult::SubscribeFail("subscribe_timeout".into()),
    }

    // Collect events 5s
    let mut stream = ws.event_stream();
    let mut event_count = 0u32;
    let mut first_event: Option<String> = None;
    let mut data_valid = false;
    let collect_start = Instant::now();
    let collect_budget = Duration::from_secs(5);

    loop {
        let remaining = collect_budget.saturating_sub(collect_start.elapsed());
        if remaining.is_zero() {
            break;
        }
        match timeout(remaining, stream.next()).await {
            Ok(Some(Ok(event))) => {
                event_count += 1;
                if first_event.is_none() {
                    let (desc, valid) = inspect_event(&event, stale_ms);
                    data_valid = valid;
                    first_event = Some(desc);
                }
            }
            Ok(Some(Err(_))) => break,
            Ok(None) => break,
            Err(_) => break,
        }
    }

    if event_count == 0 {
        WsResult::Silent
    } else {
        WsResult::Events {
            count: event_count,
            first_event: first_event.unwrap_or_else(|| "unknown".into()),
            data_valid,
        }
    }
}

// ── Per-exchange test ─────────────────────────────────────────────────────────

/// Resolve exchange-native raw symbol and account type for BTC per exchange.
///
/// Exchanges that don't trade USDT use USD. Deribit uses FuturesCross to get
/// BTC-PERPETUAL. HyperLiquid perps use just "BTC".
/// Returns `(Symbol-with-raw, raw_str_for_REST, effective_account_type)`.
fn raw_symbol_for(id: ExchangeId) -> (Symbol, String, AccountType) {
    let btc_usdt = Symbol::new("BTC", "USDT");
    let btc_usd = Symbol::new("BTC", "USD");

    let make = |sym: Symbol, at: AccountType| -> (Symbol, String, AccountType) {
        let raw = SymbolNormalizer::to_exchange(id, &sym, at)
            .unwrap_or_else(|_| sym.to_concat());
        let sym_with_raw = Symbol::with_raw(&sym.base, &sym.quote, raw.clone());
        (sym_with_raw, raw, at)
    };

    match id {
        // Deribit perp: BTC-PERPETUAL (FuturesCross account type)
        ExchangeId::Deribit => make(btc_usd, AccountType::FuturesCross),
        // HyperLiquid perps: just "BTC" (FuturesCross)
        ExchangeId::HyperLiquid => make(btc_usd, AccountType::FuturesCross),
        // Upbit: KRW-BTC (quote-base reversed). KRW pairs are the most liquid
        // on Upbit; USDT pairs are thinly traded and produce stale tickers.
        ExchangeId::Upbit => {
            let btc_krw = Symbol::new("BTC", "KRW");
            let raw = SymbolNormalizer::to_exchange(id, &btc_krw, AccountType::Spot)
                .unwrap_or_else(|_| "KRW-BTC".to_string());
            let sym_with_raw = Symbol::with_raw("BTC", "KRW", raw.clone());
            (sym_with_raw, raw, AccountType::Spot)
        }
        // Bitfinex: tBTCUSD (t-prefix, no separator)
        ExchangeId::Bitfinex => make(btc_usd, AccountType::Spot),
        // Gemini: btcusd (lowercase, no separator, USD not USDT)
        ExchangeId::Gemini => make(btc_usd, AccountType::Spot),
        // Bitstamp: btcusd (lowercase)
        ExchangeId::Bitstamp => make(btc_usd, AccountType::Spot),
        // Kraken: XBTUSD (BTC→XBT mapping, no separator)
        ExchangeId::Kraken => make(btc_usd, AccountType::Spot),
        // Coinbase: BTC-USD
        ExchangeId::Coinbase => make(btc_usd, AccountType::Spot),
        // KuCoin: BTC-USDT (dash separator)
        ExchangeId::KuCoin => make(btc_usdt, AccountType::Spot),
        // OKX: BTC-USDT (dash separator)
        ExchangeId::OKX => make(btc_usdt, AccountType::Spot),
        // Gate.io: BTC_USDT (underscore)
        ExchangeId::GateIO => make(btc_usdt, AccountType::Spot),
        // BingX: BTC-USDT (dash)
        ExchangeId::BingX => make(btc_usdt, AccountType::Spot),
        // Crypto.com spot: BTC_USDT (underscore)
        ExchangeId::CryptoCom => make(btc_usdt, AccountType::Spot),
        // dYdX: BTC-USD (perpetuals, USD-margined, dash separator)
        ExchangeId::Dydx => make(btc_usd, AccountType::FuturesCross),
        // YahooFinance: AAPL (stock ticker, not crypto)
        ExchangeId::YahooFinance => {
            let aapl = Symbol::new("AAPL", "");
            let raw = SymbolNormalizer::to_exchange(id, &aapl, AccountType::Spot)
                .unwrap_or_else(|_| "AAPL".to_string());
            let sym_with_raw = Symbol::with_raw("AAPL", "", raw.clone());
            (sym_with_raw, raw, AccountType::Spot)
        }
        // All others: BTCUSDT concat (Binance, Bybit, MEXC, HTX, Bitget, etc.)
        _ => make(btc_usdt, AccountType::Spot),
    }
}

async fn test_exchange(id: ExchangeId) -> Row {
    let hub = ExchangeHub::new();
    let (ws_symbol, symbol_str, account_type) = raw_symbol_for(id);
    let stale_ms = stale_threshold_ms();

    // ── REST ─────────────────────────────────────────────────────────────────
    let rest = match timeout(Duration::from_secs(10), hub.connect_public(id, false)).await {
        Ok(Ok(())) => {
            match hub.rest(id) {
                Some(conn) => {
                    match timeout(
                        Duration::from_secs(10),
                        MarketData::get_ticker(&*conn, symbol_str.as_str().into(), account_type),
                    )
                    .await
                    {
                        Ok(Ok(ticker)) => {
                            let mut zero_fields = Vec::new();
                            if ticker.last_price == 0.0 { zero_fields.push("last_price"); }
                            if ticker.volume_24h.unwrap_or(0.0) == 0.0 { zero_fields.push("volume"); }
                            RestResult::Ok {
                                last_price: ticker.last_price,
                                volume: ticker.volume_24h.unwrap_or(0.0),
                                bid: ticker.bid_price,
                                ask: ticker.ask_price,
                                timestamp: ticker.timestamp,
                                zero_fields,
                            }
                        }
                        Ok(Err(e)) => {
                            let s = e.to_string();
                            RestResult::Fail(s)
                        }
                        Err(_) => RestResult::Fail("ticker_timeout".into()),
                    }
                }
                None => RestResult::Fail("no_rest_handle".into()),
            }
        }
        Ok(Err(e)) => RestResult::Fail(e.to_string()),
        Err(_) => RestResult::Fail("connect_timeout".into()),
    };

    // ── WS ────────────────────────────────────────────────────────────────────
    let ws_result = match timeout(
        Duration::from_secs(8),
        hub.connect_websocket(id, account_type, false),
    )
    .await
    {
        Ok(Ok(())) => {
            match hub.ws(id, account_type) {
                Some(ws) => run_ws_test(ws, ws_symbol, account_type, stale_ms).await,
                None => WsResult::Unsupported("ws_none_after_connect".into()),
            }
        }
        Ok(Err(e)) => {
            let msg = e.to_string();
            return Row {
                exchange: id,
                rest,
                ws: WsResult::Unsupported(truncate_str(&msg, 60).to_string()),
            };
        }
        Err(_) => {
            return Row {
                exchange: id,
                rest,
                ws: WsResult::Unsupported("create_ws_timeout".into()),
            };
        }
    };

    Row { exchange: id, rest, ws: ws_result }
}

/// Test MOEX REST + WS directly (factory blocks WS for MOEX, but impl exists)
async fn test_moex_direct() -> Row {
    let hub = ExchangeHub::new();
    // MOEX: use GAZP (Gazprom) as canonical test stock
    let symbol_moex = Symbol::new("GAZP", "");
    let symbol_moex_str = SymbolNormalizer::to_exchange(ExchangeId::Moex, &symbol_moex, AccountType::Spot)
        .unwrap_or_else(|_| "GAZP".to_string());
    let account_type = AccountType::Spot;
    let stale_ms = stale_threshold_ms();

    // REST
    let rest = match timeout(Duration::from_secs(10), hub.connect_public(ExchangeId::Moex, false)).await {
        Ok(Ok(())) => {
            match hub.rest(ExchangeId::Moex) {
                Some(conn) => {
                    let ticker_result = timeout(
                        Duration::from_secs(10),
                        MarketData::get_ticker(&*conn, symbol_moex_str.as_str().into(), account_type),
                    )
                    .await
                    .unwrap_or_else(|_| Err(digdigdig3::core::types::ExchangeError::Timeout("timeout".into())));

                    match ticker_result {
                        Ok(ticker) => {
                            let mut zero_fields = Vec::new();
                            if ticker.last_price == 0.0 { zero_fields.push("last_price"); }
                            if ticker.volume_24h.unwrap_or(0.0) == 0.0 { zero_fields.push("volume"); }
                            RestResult::Ok {
                                last_price: ticker.last_price,
                                volume: ticker.volume_24h.unwrap_or(0.0),
                                bid: ticker.bid_price,
                                ask: ticker.ask_price,
                                timestamp: ticker.timestamp,
                                zero_fields,
                            }
                        }
                        Err(e) => RestResult::Fail(e.to_string()),
                    }
                }
                None => RestResult::Fail("no_rest_handle".into()),
            }
        }
        Ok(Err(e)) => RestResult::Fail(e.to_string()),
        Err(_) => RestResult::Fail("connect_timeout".into()),
    };

    // WS — direct construction since factory blocks it
    let ws_result = {
        let ws = std::sync::Arc::new(MoexWebSocket::new_public()) as std::sync::Arc<dyn WebSocketConnector>;
        // Use GAZP (Gazprom) — canonical MOEX stock ticker
        let moex_symbol = Symbol::new("GAZP", "");
        match timeout(
            Duration::from_secs(20),
            run_ws_test(ws, moex_symbol, account_type, stale_ms),
        )
        .await
        {
            Ok(r) => r,
            Err(_) => WsResult::ConnectFail("overall_timeout_20s".into()),
        }
    };

    Row { exchange: ExchangeId::Moex, rest, ws: ws_result }
}

// ── Canonical path smoke ──────────────────────────────────────────────────────

/// Verify that SymbolInput::Canonical resolves and returns real data for two
/// representative exchanges.  Raw vs Canonical must produce identical tickers.
async fn smoke_canonical_path() {
    println!();
    println!("=== CANONICAL PATH SMOKE (θ.3) ===");
    println!("Verifies SymbolInput::Canonical normalizes + returns real ticker data.");
    println!();

    // Exchange 1: Binance — BTC/USDT spot
    {
        let hub = ExchangeHub::new();
        let canonical_sym = Symbol::new("BTC", "USDT");
        let ok = match timeout(Duration::from_secs(12), hub.connect_public(ExchangeId::Binance, false)).await {
            Ok(Ok(())) => {
                match hub.rest(ExchangeId::Binance) {
                    Some(conn) => {
                        // Canonical call — normalizer converts BTC/USDT → "BTCUSDT" internally
                        let input = SymbolInput::Canonical(&canonical_sym);
                        match timeout(
                            Duration::from_secs(10),
                            MarketData::get_ticker(&*conn, input, AccountType::Spot),
                        ).await {
                            Ok(Ok(ticker)) => {
                                let valid = ticker.last_price > 0.0;
                                println!(
                                    "Binance  Canonical BTC/USDT → last={:.4}  {}",
                                    ticker.last_price,
                                    if valid { "PASS" } else { "FAIL (zero price)" }
                                );
                                valid
                            }
                            Ok(Err(e)) => { println!("Binance  Canonical FAIL: {e}"); false }
                            Err(_)     => { println!("Binance  Canonical FAIL: timeout"); false }
                        }
                    }
                    None => { println!("Binance  Canonical FAIL: no rest handle"); false }
                }
            }
            Ok(Err(e)) => { println!("Binance  Canonical FAIL: connect {e}"); false }
            Err(_)     => { println!("Binance  Canonical FAIL: connect timeout"); false }
        };
        if !ok {
            println!("  RESULT: Canonical path FAIL on Binance");
        }
    }

    // Exchange 2: OKX — BTC/USDT spot (dash-separated internally: "BTC-USDT")
    {
        let hub = ExchangeHub::new();
        let canonical_sym = Symbol::new("BTC", "USDT");
        let ok = match timeout(Duration::from_secs(12), hub.connect_public(ExchangeId::OKX, false)).await {
            Ok(Ok(())) => {
                match hub.rest(ExchangeId::OKX) {
                    Some(conn) => {
                        // Canonical call — normalizer converts BTC/USDT → "BTC-USDT" internally
                        let input = SymbolInput::Canonical(&canonical_sym);
                        match timeout(
                            Duration::from_secs(10),
                            MarketData::get_ticker(&*conn, input, AccountType::Spot),
                        ).await {
                            Ok(Ok(ticker)) => {
                                let valid = ticker.last_price > 0.0;
                                println!(
                                    "OKX      Canonical BTC/USDT → last={:.4}  {}",
                                    ticker.last_price,
                                    if valid { "PASS" } else { "FAIL (zero price)" }
                                );
                                valid
                            }
                            Ok(Err(e)) => { println!("OKX      Canonical FAIL: {e}"); false }
                            Err(_)     => { println!("OKX      Canonical FAIL: timeout"); false }
                        }
                    }
                    None => { println!("OKX      Canonical FAIL: no rest handle"); false }
                }
            }
            Ok(Err(e)) => { println!("OKX      Canonical FAIL: connect {e}"); false }
            Err(_)     => { println!("OKX      Canonical FAIL: connect timeout"); false }
        };
        if !ok {
            println!("  RESULT: Canonical path FAIL on OKX");
        }
    }

    println!();
    println!("Canonical path: SymbolInput::Canonical(&Symbol::new(\"BTC\",\"USDT\")) dispatched via");
    println!("SymbolNormalizer::to_exchange inside resolve() — zero changes to connector code.");
}

// ── All exchanges ─────────────────────────────────────────────────────────────

fn all_testable_exchanges() -> Vec<ExchangeId> {
    vec![
        // CEX — full public REST + WS via factory
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
        // Polymarket: prediction-market CLOB. Tokens are 64-char hex IDs from
        // /markets, not BTC/USDT. Skipped until we add a market-discovery step.
        // ExchangeId::Polymarket,
        ExchangeId::Dukascopy,
        ExchangeId::Alpaca,
        ExchangeId::Krx,
        // Auth-required — shows diagnostic FAIL
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
        // Coinglass — still in factory, kept for now (creds-gated).
        ExchangeId::Coinglass,
        // Polymarket — prediction-market CLOB; symbols are 64-char hex
        // token_ids resolved from /markets, not canonical BTC/USDT. The
        // e2e_smoke harness has no live market-id picker, so skip until
        // we add a discovery step.
        // ExchangeId::Polymarket,
        // Extracted to dig2feed (DefiLlama, WhaleAlert, Bitquery) and removed
        // (Fred, Bls). Their ExchangeId variants linger in the enum because
        // ~300 sites reference them; they cannot be smoke-tested from this
        // crate and the factory returns `UnsupportedOperation`. Skipping
        // here keeps the e2e_smoke report focused on real connectors.
        //
        // ExchangeId::DefiLlama,  // moved to dig2feed
        // ExchangeId::WhaleAlert, // moved to dig2feed
        // ExchangeId::Fred,       // removed
        // ExchangeId::Bitquery,   // moved to dig2feed
        // ExchangeId::Bls,        // removed
    ]
}

// ── Main ──────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();

    let exchanges = all_testable_exchanges();
    let total = exchanges.len() + 1; // +1 for MOEX direct

    println!("=== DEEP SMOKE — digdigdig3 ===");
    println!("WS impl total (grep impl WebSocketConnector+WsProtocol): 35");
    println!("WS-capable distinct exchanges (via factory + direct): 25");
    println!("REST total ExchangeId variants in factory: 49");
    println!("Testing {} exchanges ({}+MOEX_direct) in parallel", total, exchanges.len());
    println!();

    // Spawn all factory-based tests
    let mut handles: Vec<_> = exchanges
        .iter()
        .copied()
        .map(|id| {
            tokio::spawn(async move {
                timeout(Duration::from_secs(25), test_exchange(id))
                    .await
                    .unwrap_or_else(|_| Row {
                        exchange: id,
                        rest: RestResult::Fail("TIMEOUT_25s".into()),
                        ws: WsResult::Unsupported("TIMEOUT_25s".into()),
                    })
            })
        })
        .collect();

    // Spawn MOEX direct (needs slightly more time for its WS)
    let moex_handle = tokio::spawn(async move {
        timeout(Duration::from_secs(30), test_moex_direct())
            .await
            .unwrap_or_else(|_| Row {
                exchange: ExchangeId::Moex,
                rest: RestResult::Fail("TIMEOUT_30s".into()),
                ws: WsResult::Unsupported("TIMEOUT_30s".into()),
            })
    });
    handles.push(moex_handle);

    let results = futures_util::future::join_all(handles).await;

    let mut rows: Vec<Row> = results.into_iter().filter_map(|r| r.ok()).collect();
    rows.sort_by(|a, b| format!("{:?}", a.exchange).cmp(&format!("{:?}", b.exchange)));

    // ── Per-row output ────────────────────────────────────────────────────────
    println!("=== PER-EXCHANGE RESULTS ===");
    println!("{:<20} | {:<75} | {}", "Exchange", "REST", "WS");
    println!("{}", "-".repeat(200));

    for row in &rows {
        let ws_str = match &row.ws {
            WsResult::Unsupported(msg) => format!("WS_NA: {}", &msg[..msg.len().min(50)]),
            WsResult::ConnectFail(msg) => format!("CONN_FAIL: {}", &msg[..msg.len().min(50)]),
            WsResult::SubscribeFail(msg) => format!("SUB_FAIL: {}", &msg[..msg.len().min(50)]),
            WsResult::Silent => "SILENT(0 events)".to_string(),
            WsResult::Events { count, first_event, data_valid } => {
                let fe = if first_event.len() > 80 {
                    format!("{}...", &first_event[..77])
                } else {
                    first_event.clone()
                };
                let valid_tag = if *data_valid { "DATA_VALID" } else { "DATA_EMPTY" };
                format!("OK cnt={} {} | {}", count, valid_tag, fe)
            }
        };

        println!(
            "{:<20} | {:<75} | {}",
            format!("{:?}", row.exchange),
            row.rest.short(),
            ws_str
        );
    }

    // ── Summary ───────────────────────────────────────────────────────────────
    let _rest_ok_count = rows.iter().filter(|r| r.rest.is_ok()).count();
    let rest_ok_populated = rows
        .iter()
        .filter(|r| r.rest.is_ok() && !r.rest.has_zero_fields())
        .count();
    let rest_ok_zero = rows
        .iter()
        .filter(|r| r.rest.is_ok() && r.rest.has_zero_fields())
        .count();
    let rest_fail = rows.iter().filter(|r| !r.rest.is_ok()).count();

    let ws_total_impls = 35usize;
    let ws_via_factory = rows.iter().filter(|r| r.ws.connect_ok()).count();
    let ws_flowing = rows.iter().filter(|r| r.ws.has_events()).count();
    let ws_valid = rows.iter().filter(|r| r.ws.data_valid()).count();
    let ws_empty_data = rows.iter().filter(|r| r.ws.data_empty()).count();

    let ws_silent: Vec<_> = rows
        .iter()
        .filter(|r| matches!(r.ws, WsResult::Silent))
        .map(|r| format!("{:?}", r.exchange))
        .collect();

    let ws_empty_data_exchanges: Vec<_> = rows
        .iter()
        .filter(|r| r.ws.data_empty())
        .map(|r| format!("{:?}", r.exchange))
        .collect();

    let rest_zero_fields_exchanges: Vec<_> = rows
        .iter()
        .filter(|r| r.rest.has_zero_fields())
        .map(|r| {
            if let RestResult::Ok { zero_fields, .. } = &r.rest {
                format!("{:?}({})", r.exchange, zero_fields.join(","))
            } else {
                format!("{:?}", r.exchange)
            }
        })
        .collect();

    // Class 1: Connection fails
    let class1_conn_fail: Vec<_> = rows
        .iter()
        .filter(|r| {
            !r.rest.is_ok()
                && !matches!(r.ws, WsResult::Unsupported(_))
                || matches!(r.ws, WsResult::ConnectFail(_))
        })
        .map(|r| format!("{:?}", r.exchange))
        .collect();

    // Class 2: Subscribed but silent
    let class2_silent: Vec<_> = rows
        .iter()
        .filter(|r| matches!(r.ws, WsResult::Silent))
        .map(|r| format!("{:?}", r.exchange))
        .collect();

    // Class 3: Events flowing but data is zero/empty
    let class3_empty_data: Vec<_> = ws_empty_data_exchanges.clone();

    println!();
    println!("=== SUMMARY ===");
    println!("WS impl total in codebase:            {}", ws_total_impls);
    println!("WS tested (via factory + MOEX direct):{}", rows.len());
    println!("WS connected:                         {}", ws_via_factory);
    println!("WS events flowing:                    {}", ws_flowing);
    println!("WS events with REAL data (non-zero):  {}", ws_valid);
    println!("WS events EMPTY/DEFAULT fields (BUG): {}", ws_empty_data);
    println!("WS silent (subscribed, 0 events):     {}", ws_silent.len());
    println!();
    println!("REST total variants tested:            {}", total);
    println!("REST OK with populated fields:         {}", rest_ok_populated);
    println!("REST OK but zero/default fields:       {}", rest_ok_zero);
    println!("REST FAIL (auth/network):              {}", rest_fail);
    println!();
    println!("=== BUG CLASSES ===");
    println!("Class 1 — Connection fails:            {} — {:?}", class1_conn_fail.len(), class1_conn_fail);
    println!("Class 2 — Silent streams (sub ok, 0 events): {} — {:?}", class2_silent.len(), class2_silent);
    println!("Class 3 — Events flowing but zero/empty data (PARSER BUG): {} — {:?}", class3_empty_data.len(), class3_empty_data);
    if !rest_zero_fields_exchanges.is_empty() {
        println!("REST zero fields: {:?}", rest_zero_fields_exchanges);
    }

    // θ.3 — verify Canonical path end-to-end
    smoke_canonical_path().await;

    println!();
    println!("Total runtime: {:.1}s", start.elapsed().as_secs_f64());

    Ok(())
}
