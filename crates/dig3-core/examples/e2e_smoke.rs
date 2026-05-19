//! # e2e_smoke — full-coverage parallel E2E harness for all dig3 exchanges.
//!
//! Covers EVERY declared method per exchange:
//!   REST: ping, get_price, get_ticker, get_orderbook, get_klines, get_recent_trades,
//!         get_exchange_info, + futures: get_funding_rate, get_open_interest,
//!         get_mark_price, get_long_short_ratio, get_liquidation_history, get_premium_index
//!   WS:   Ticker, Trade, Orderbook, Kline, + futures: MarkPrice, FundingRate,
//!         Liquidation, OpenInterest, AggTrade
//!   Trading (if credentials in ENV): get_balance, get_account_info, get_open_orders,
//!         get_user_trades, get_positions
//!
//! CLI flags (parsed manually — no extra crates):
//!   --exchange <id>     filter to one exchange
//!   --market            run only market-data section (default)
//!   --trading           run only trading/account section (needs ENV creds)
//!   --all               run both market + trading
//!   --json-out <path>   write JSON report to file
//!
//! Run:
//!     cargo run --example e2e_smoke --release 2>&1 | tee e2e_smoke_report.txt
//!
//! No API keys required for market-data. Trading section auto-skips when no creds.

use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use digdigdig3_core::connector_manager::ExchangeHub;
use digdigdig3_core::core::traits::{Credentials, MarketData, WebSocketConnector};
use digdigdig3_core::core::types::{
    AccountType, BalanceQuery, ExchangeId, PositionQuery,
    StreamEvent, StreamType, SubscriptionRequest, Symbol, SymbolInput,
    UserTradeFilter,
};
use digdigdig3_core::core::utils::SymbolNormalizer;
use digdigdig3_core::l2::free::moex::MoexWebSocket;
use digdigdig3_core::testing::harness::TestHarness;
use futures_util::StreamExt;
use tokio::time::{timeout, Duration};

// ─────────────────────────────────────────────────────────────────────────────
// mod cli
// ─────────────────────────────────────────────────────────────────────────────

mod cli {
    #[derive(Debug, Clone)]
    pub struct Args {
        pub exchange_filter: Option<String>,
        pub run_market: bool,
        pub run_trading: bool,
        pub json_out: Option<String>,
    }

    impl Args {
        pub fn parse() -> Self {
            let argv: Vec<String> = std::env::args().collect();
            let mut filter = None;
            let mut market = false;
            let mut trading = false;
            let mut all = false;
            let mut json_out = None;
            let mut i = 1usize;
            while i < argv.len() {
                match argv[i].as_str() {
                    "--exchange" => {
                        i += 1;
                        if i < argv.len() { filter = Some(argv[i].clone()); }
                    }
                    "--market" => { market = true; }
                    "--trading" => { trading = true; }
                    "--all" => { all = true; }
                    "--json-out" => {
                        i += 1;
                        if i < argv.len() { json_out = Some(argv[i].clone()); }
                    }
                    _ => {}
                }
                i += 1;
            }
            if all { market = true; trading = true; }
            // default: market only
            if !market && !trading { market = true; }
            Self { exchange_filter: filter, run_market: market, run_trading: trading, json_out }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// mod result_types
// ─────────────────────────────────────────────────────────────────────────────

mod result_types {
    use serde::Serialize;

    /// Result of a single REST or WS method call.
    #[derive(Debug, Clone, Serialize)]
    #[serde(tag = "status", content = "detail")]
    pub enum MethodResult {
        Ok(String),
        Empty,
        Err(String),
        Timeout,
        Unsupported(String),
        Skipped,
    }

    impl MethodResult {
        pub fn cell(&self) -> &'static str {
            match self {
                MethodResult::Ok(_) => "OK  ",
                MethodResult::Empty => "EMPT",
                MethodResult::Err(_) => "ERR ",
                MethodResult::Timeout => "TIME",
                MethodResult::Unsupported(_) => "-- ",
                MethodResult::Skipped => "SKIP",
            }
        }
        pub fn is_ok(&self) -> bool { matches!(self, MethodResult::Ok(_)) }
        pub fn is_issue(&self) -> bool { matches!(self, MethodResult::Err(_) | MethodResult::Empty | MethodResult::Timeout) }
        pub fn detail(&self) -> Option<&str> {
            match self {
                MethodResult::Ok(s) | MethodResult::Err(s) | MethodResult::Unsupported(s) => Some(s.as_str()),
                _ => None,
            }
        }
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct MarketRow {
        pub exchange: String,
        // REST
        pub ping: MethodResult,
        pub price: MethodResult,
        pub ticker: MethodResult,
        pub orderbook: MethodResult,
        pub klines: MethodResult,
        pub trades: MethodResult,
        pub exch_info: MethodResult,
        // Futures REST
        pub funding: MethodResult,
        pub open_interest: MethodResult,
        pub mark_price: MethodResult,
        pub long_short: MethodResult,
        pub liquidations: MethodResult,
        pub premium_index: MethodResult,
        // WS
        pub ws_ticker: MethodResult,
        pub ws_trade: MethodResult,
        pub ws_orderbook: MethodResult,
        pub ws_kline: MethodResult,
        pub ws_mark_price: MethodResult,
        pub ws_funding: MethodResult,
        pub ws_liquidation: MethodResult,
        pub ws_oi: MethodResult,
        pub ws_agg_trade: MethodResult,
        // issues collected for ISSUES block
        pub issues: Vec<String>,
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct TradingRow {
        pub exchange: String,
        pub balance: MethodResult,
        pub account_info: MethodResult,
        pub open_orders: MethodResult,
        pub user_trades: MethodResult,
        pub positions: MethodResult,
        pub fees: MethodResult,
        pub issues: Vec<String>,
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct ExchangeReport {
        pub market: Option<MarketRow>,
        pub trading: Option<TradingRow>,
    }
}

use result_types::{ExchangeReport, MarketRow, MethodResult, TradingRow};

// ─────────────────────────────────────────────────────────────────────────────
// Timestamp helpers (shared)
// ─────────────────────────────────────────────────────────────────────────────

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn stale_threshold_ms() -> i64 { now_ms() - 5 * 60_000 }

fn timestamp_unit_bug(ts: i64) -> bool {
    let now = now_ms();
    ts > 0 && ts < now / 100
}

fn timestamp_future_bug(ts: i64) -> bool { ts > now_ms() + 60_000 }

fn truncate(s: &str, n: usize) -> String {
    match s.char_indices().nth(n) {
        Some((i, _)) => format!("{}…", &s[..i]),
        None => s.to_string(),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// mod market — WS event inspector (reused from original e2e_smoke)
// ─────────────────────────────────────────────────────────────────────────────

mod market {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ExpectedKind { Ticker, Trade, Orderbook, Kline, MarkPrice, FundingRate, Liquidation, OpenInterest, AggTrade }

    /// Returns (description, valid, issues).
    pub fn inspect_event(event: &StreamEvent, stale_ms: i64, expected_kind: ExpectedKind) -> (String, bool, Vec<String>) {
        let mut issues: Vec<String> = Vec::new();

        let (s, valid) = match event {
            StreamEvent::Ticker(t) => {
                if expected_kind != ExpectedKind::Ticker {
                    issues.push(format!("WRONG_TYPE: got Ticker, expected {:?}", expected_kind));
                }
                if t.last_price <= 0.0 { issues.push("last_price<=0".into()); }
                if timestamp_unit_bug(t.timestamp) {
                    issues.push(format!("ts_unit_bug(seconds): {}", t.timestamp));
                } else if timestamp_future_bug(t.timestamp) {
                    issues.push(format!("ts_future_bug: {}", t.timestamp));
                } else if t.timestamp <= stale_ms {
                    issues.push(format!("ts_stale: {}min ago", (now_ms() - t.timestamp) / 60_000));
                }
                match (t.bid_price, t.ask_price) {
                    (Some(b), Some(a)) if b > a => issues.push(format!("bid>ask: {:.4}>{:.4}", b, a)),
                    // bid/ask may be None for allMids-style tickers (exchange pushes mid-prices
                    // without order-book data).  Not a structural defect — omit from issues.
                    _ => {}
                }
                let valid = t.last_price > 0.0
                    && !timestamp_unit_bug(t.timestamp)
                    && !timestamp_future_bug(t.timestamp)
                    && t.timestamp > stale_ms;
                (format!("Ticker sym={} last={:.4} bid={} ask={} ts={}",
                    t.symbol, t.last_price,
                    t.bid_price.map(|v| format!("{:.4}", v)).unwrap_or_else(|| "None".into()),
                    t.ask_price.map(|v| format!("{:.4}", v)).unwrap_or_else(|| "None".into()),
                    t.timestamp), valid)
            }
            StreamEvent::Trade(t) => {
                if expected_kind != ExpectedKind::Trade {
                    issues.push(format!("WRONG_TYPE: got Trade, expected {:?}", expected_kind));
                }
                let valid = t.price > 0.0 && t.quantity > 0.0;
                if !valid { issues.push("price<=0 or qty<=0".into()); }
                (format!("Trade sym={} px={:.4} qty={:.6} ts={}", t.symbol, t.price, t.quantity, t.timestamp), valid)
            }
            StreamEvent::OrderbookSnapshot(ob) => {
                let top_bid = ob.bids.first().map(|l| l.price).unwrap_or(0.0);
                let top_ask = ob.asks.first().map(|l| l.price).unwrap_or(0.0);
                // Only flag empty/zero when BOTH sides are absent or both tops are zero.
                // A populated side (bids OR asks) with non-zero price means the book is live.
                let truly_empty = (ob.bids.is_empty() && ob.asks.is_empty())
                    || (top_bid <= 0.0 && top_ask <= 0.0);
                if truly_empty { issues.push("orderbook empty/zero".into()); }
                let valid = !truly_empty;
                (format!("OBSnapshot bids={} asks={} top_bid={:.4} top_ask={:.4}",
                    ob.bids.len(), ob.asks.len(), top_bid, top_ask), valid)
            }
            StreamEvent::OrderbookDelta(od) => {
                // Empty deltas are normal (heartbeats / zero-qty level removals) — not an issue.
                let has_data = !od.bids.is_empty() || !od.asks.is_empty();
                let top_bid = od.bids.first().map(|l| l.price).unwrap_or(0.0);
                (format!("OBDelta bids={} asks={} top_bid={:.4} ts={}",
                    od.bids.len(), od.asks.len(), top_bid, od.timestamp), has_data)
            }
            StreamEvent::Kline(k) => {
                let valid = k.close > 0.0 && k.open > 0.0 && k.open_time > 0;
                if !valid { issues.push("kline o/c<=0 or no open_time".into()); }
                (format!("Kline o={:.4} h={:.4} l={:.4} c={:.4} vol={:.2} ts={}",
                    k.open, k.high, k.low, k.close, k.volume, k.open_time), valid)
            }
            StreamEvent::MarkPrice { symbol, mark_price, timestamp, .. } => {
                if expected_kind != ExpectedKind::MarkPrice {
                    issues.push(format!("WRONG_TYPE: got MarkPrice, expected {:?}", expected_kind));
                }
                let valid = *mark_price > 0.0 && *timestamp > stale_ms;
                if !valid { issues.push("mark_price<=0 or stale".into()); }
                (format!("MarkPrice sym={} px={:.4} ts={}", symbol, mark_price, timestamp), valid)
            }
            StreamEvent::FundingRate { symbol, rate, timestamp, .. } => {
                if expected_kind == ExpectedKind::Ticker {
                    issues.push("WRONG_TYPE: got FundingRate while subscribed to Ticker".into());
                }
                (format!("FundingRate sym={} rate={:.6} ts={}", symbol, rate, timestamp), *timestamp > 0)
            }
            StreamEvent::Liquidation { symbol, price, quantity, timestamp, .. } => {
                let valid = *price > 0.0 && *quantity > 0.0;
                if !valid { issues.push("liquidation px/qty<=0".into()); }
                (format!("Liquidation sym={} px={:.4} qty={:.6} ts={}", symbol, price, quantity, timestamp), valid)
            }
            StreamEvent::OpenInterestUpdate { symbol, open_interest, timestamp, .. } => {
                let valid = *open_interest > 0.0;
                if !valid { issues.push("open_interest<=0".into()); }
                (format!("OI sym={} oi={:.2} ts={}", symbol, open_interest, timestamp), valid)
            }
            StreamEvent::AggTrade { symbol, price, quantity, timestamp, .. } => {
                let valid = *price > 0.0 && *quantity > 0.0;
                if !valid { issues.push("aggtrade px/qty<=0".into()); }
                (format!("AggTrade sym={} px={:.4} qty={:.6} ts={}", symbol, price, quantity, timestamp), valid)
            }
            other => {
                let s = format!("{:?}", other);
                let short = truncate(&s, 80);
                (short, true)
            }
        };
        (s, valid, issues)
    }

    /// Collect events from a single WS subscription.
    /// `budget_secs` — collection window after subscribe ACK.
    pub async fn collect_ws_stream(
        ws: Arc<dyn WebSocketConnector>,
        sub: SubscriptionRequest,
        expected_kind: ExpectedKind,
        stale_ms: i64,
        budget_secs: u64,
    ) -> MethodResult {
        let account_type = sub.account_type;
        match timeout(Duration::from_secs(8), ws.connect(account_type)).await {
            Ok(Ok(())) => {}
            Ok(Err(e)) => return MethodResult::Err(truncate(&e.to_string(), 60)),
            Err(_) => return MethodResult::Err("connect_timeout".into()),
        }
        match timeout(Duration::from_secs(5), ws.subscribe(sub)).await {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                let msg = e.to_string();
                if msg.contains("UnsupportedOperation") || msg.contains("not support")
                    || msg.contains("Not supported")
                {
                    return MethodResult::Unsupported(truncate(&msg, 60));
                }
                return MethodResult::Err(format!("sub_fail: {}", truncate(&msg, 60)));
            }
            Err(_) => return MethodResult::Err("subscribe_timeout".into()),
        }

        let mut stream = ws.event_stream();
        let mut event_count = 0u32;
        let mut first_desc: Option<String> = None;
        let mut all_issues: Vec<String> = Vec::new();
        let mut wrong_type: Vec<String> = Vec::new();
        let mut saw_expected = false;
        let collect_start = Instant::now();
        let budget = Duration::from_secs(budget_secs);

        loop {
            let remaining = budget.saturating_sub(collect_start.elapsed());
            if remaining.is_zero() { break; }
            match timeout(remaining, stream.next()).await {
                Ok(Some(Ok(event))) => {
                    event_count += 1;
                    let is_expected = matches!((&event, expected_kind),
                        (StreamEvent::Ticker(_), ExpectedKind::Ticker) |
                        (StreamEvent::Trade(_), ExpectedKind::Trade) |
                        (StreamEvent::OrderbookSnapshot(_) | StreamEvent::OrderbookDelta(_), ExpectedKind::Orderbook) |
                        (StreamEvent::Kline(_), ExpectedKind::Kline) |
                        (StreamEvent::MarkPrice { .. }, ExpectedKind::MarkPrice) |
                        (StreamEvent::FundingRate { .. }, ExpectedKind::FundingRate) |
                        (StreamEvent::Liquidation { .. }, ExpectedKind::Liquidation) |
                        (StreamEvent::OpenInterestUpdate { .. }, ExpectedKind::OpenInterest) |
                        (StreamEvent::AggTrade { .. }, ExpectedKind::AggTrade)
                    );
                    if is_expected { saw_expected = true; }
                    let (desc, _valid, issues) = inspect_event(&event, stale_ms, expected_kind);
                    if first_desc.is_none() { first_desc = Some(desc); }
                    for iss in issues {
                        if iss.starts_with("WRONG_TYPE") {
                            if !wrong_type.contains(&iss) { wrong_type.push(iss); }
                        } else if !all_issues.contains(&iss) {
                            all_issues.push(iss);
                        }
                    }
                }
                Ok(Some(Err(_))) | Ok(None) | Err(_) => break,
            }
        }

        if !saw_expected { all_issues.extend(wrong_type); }

        if event_count == 0 {
            MethodResult::Err("silent_0_events".into())
        } else {
            let desc = first_desc.unwrap_or_else(|| "?".into());
            if all_issues.is_empty() {
                MethodResult::Ok(format!("cnt={} {}", event_count, truncate(&desc, 80)))
            } else {
                MethodResult::Err(format!("cnt={} ISSUES[{}] {}", event_count, all_issues.join(";"), truncate(&desc, 60)))
            }
        }
    }

    /// Run a single WS subscription through the hub (creates its own WS connection).
    /// `budget_secs` — collection window after subscribe ACK (10s for regular streams,
    /// 20s for low-freq streams like Liquidation / OpenInterest).
    pub async fn run_ws_sub(
        exchange: ExchangeId,
        account_type: AccountType,
        stream_type: StreamType,
        symbol: Symbol,
        expected_kind: ExpectedKind,
        stale_ms: i64,
        budget_secs: u64,
    ) -> MethodResult {
        let hub = ExchangeHub::new();
        match timeout(Duration::from_secs(8), hub.connect_websocket(exchange, account_type, false)).await {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                let msg = e.to_string();
                if msg.contains("UnsupportedOperation") || msg.contains("not support")
                    || msg.contains("Not supported")
                {
                    return MethodResult::Unsupported(truncate(&msg, 60));
                }
                return MethodResult::Err(format!("connect_fail: {}", truncate(&msg, 60)));
            }
            Err(_) => return MethodResult::Err("ws_connect_timeout".into()),
        }
        match hub.ws(exchange, account_type) {
            Some(ws) => {
                let sub = SubscriptionRequest {
                    symbol,
                    stream_type,
                    account_type,
                    depth: None,
                    update_speed_ms: None,
                };
                collect_ws_stream(ws, sub, expected_kind, stale_ms, budget_secs).await
            }
            None => MethodResult::Err("ws_none_after_connect".into()),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Symbol resolution (per-exchange BTC mapping)
// ─────────────────────────────────────────────────────────────────────────────

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
        ExchangeId::Deribit => make(btc_usd, AccountType::FuturesCross),
        ExchangeId::HyperLiquid => make(btc_usd, AccountType::FuturesCross),
        ExchangeId::Upbit => {
            let btc_krw = Symbol::new("BTC", "KRW");
            let raw = SymbolNormalizer::to_exchange(id, &btc_krw, AccountType::Spot)
                .unwrap_or_else(|_| "KRW-BTC".to_string());
            let sym_with_raw = Symbol::with_raw("BTC", "KRW", raw.clone());
            (sym_with_raw, raw, AccountType::Spot)
        }
        ExchangeId::Bitfinex => make(btc_usd, AccountType::Spot),
        ExchangeId::Gemini => make(btc_usd, AccountType::Spot),
        ExchangeId::Bitstamp => make(btc_usd, AccountType::Spot),
        ExchangeId::Kraken => make(btc_usd, AccountType::Spot),
        ExchangeId::Coinbase => make(btc_usd, AccountType::Spot),
        ExchangeId::KuCoin => make(btc_usdt, AccountType::Spot),
        ExchangeId::OKX => make(btc_usdt, AccountType::Spot),
        ExchangeId::GateIO => make(btc_usdt, AccountType::Spot),
        ExchangeId::BingX => make(btc_usdt, AccountType::Spot),
        ExchangeId::CryptoCom => make(btc_usdt, AccountType::Spot),
        ExchangeId::Dydx => make(btc_usd, AccountType::FuturesCross),
        ExchangeId::YahooFinance => {
            let btc = Symbol::new("BTC", "USD");
            let raw = SymbolNormalizer::to_exchange(id, &btc, AccountType::Spot)
                .unwrap_or_else(|_| "BTC-USD".to_string());
            (Symbol::with_raw("BTC", "USD", raw.clone()), raw, AccountType::Spot)
        }
        ExchangeId::Polymarket => {
            let sym = Symbol::with_raw("DISCOVER", "USDC", "DISCOVER".to_string());
            (sym, "DISCOVER".to_string(), AccountType::Spot)
        }
        _ => make(btc_usdt, AccountType::Spot),
    }
}

/// Symbol to use for WS Liquidation subscription.
///
/// Binance: empty raw symbol → `!forceOrder@arr` all-symbols stream.
/// GateIO:  raw `"!all"` → `!all` all-symbols liquidation stream.
/// All others: same as the per-exchange BTC perp symbol from `raw_symbol_for`.
fn liq_symbol_for(id: ExchangeId) -> Symbol {
    match id {
        ExchangeId::Binance => Symbol::with_raw("", "", "".to_string()),
        ExchangeId::GateIO  => Symbol::with_raw("", "", "!all".to_string()),
        _ => {
            // Use the standard BTC perp symbol for this exchange.
            let (sym, ..) = raw_symbol_for(id);
            sym
        }
    }
}

/// Data providers that aggregate prices but do NOT expose per-exchange bid/ask
/// on the free tier.  bid_price=None and ask_price=None is expected and correct
/// for these sources — not a wire bug.
///
/// CryptoCompare: `pricemultifull` (CCCAGG aggregate) omits BID/ASK entirely.
/// ob/l1/top (which would have bid/ask) requires a paid API key.
fn no_bid_ask_by_design(id: ExchangeId) -> bool {
    matches!(id,
        ExchangeId::CryptoCompare
        | ExchangeId::YahooFinance
        | ExchangeId::Twelvedata
        | ExchangeId::AlphaVantage
        | ExchangeId::Tiingo
        | ExchangeId::Fred
        | ExchangeId::DefiLlama
        | ExchangeId::Coinglass
        | ExchangeId::Dukascopy
        | ExchangeId::Moex
        | ExchangeId::Krx
        | ExchangeId::JQuants
        | ExchangeId::Bls
    )
}

/// Is this exchange account_type futures-capable (perps/perpetuals)?
fn is_futures(id: ExchangeId, at: AccountType) -> bool {
    matches!(at, AccountType::FuturesCross | AccountType::FuturesIsolated)
        || matches!(id, ExchangeId::Binance | ExchangeId::Bybit | ExchangeId::OKX
            | ExchangeId::KuCoin | ExchangeId::GateIO | ExchangeId::MEXC
            | ExchangeId::HTX | ExchangeId::Bitget | ExchangeId::BingX
            | ExchangeId::CryptoCom | ExchangeId::Deribit | ExchangeId::HyperLiquid
            | ExchangeId::Lighter | ExchangeId::Dydx | ExchangeId::Coinglass)
}

// ─────────────────────────────────────────────────────────────────────────────
// mod market::test_market
// ─────────────────────────────────────────────────────────────────────────────

async fn test_market(id: ExchangeId) -> MarketRow {
    let (sym, raw_str, account_type) = raw_symbol_for(id);
    let stale_ms = stale_threshold_ms();
    let futures_capable = is_futures(id, account_type);

    // ── Connect REST ─────────────────────────────────────────────────────────
    let hub = ExchangeHub::new();
    let connected = match timeout(Duration::from_secs(12), hub.connect_public(id, false)).await {
        Ok(Ok(())) => true,
        Ok(Err(e)) => {
            let err_msg = truncate(&e.to_string(), 70);
            return MarketRow {
                exchange: format!("{:?}", id),
                ping: MethodResult::Err(format!("connect_fail: {}", err_msg)),
                price: MethodResult::Skipped,
                ticker: MethodResult::Skipped,
                orderbook: MethodResult::Skipped,
                klines: MethodResult::Skipped,
                trades: MethodResult::Skipped,
                exch_info: MethodResult::Skipped,
                funding: MethodResult::Skipped,
                open_interest: MethodResult::Skipped,
                mark_price: MethodResult::Skipped,
                long_short: MethodResult::Skipped,
                liquidations: MethodResult::Skipped,
                premium_index: MethodResult::Skipped,
                ws_ticker: MethodResult::Skipped,
                ws_trade: MethodResult::Skipped,
                ws_orderbook: MethodResult::Skipped,
                ws_kline: MethodResult::Skipped,
                ws_mark_price: MethodResult::Skipped,
                ws_funding: MethodResult::Skipped,
                ws_liquidation: MethodResult::Skipped,
                ws_oi: MethodResult::Skipped,
                ws_agg_trade: MethodResult::Skipped,
                issues: vec![format!("connect_fail: {}", err_msg)],
            };
        }
        Err(_) => {
            return MarketRow {
                exchange: format!("{:?}", id),
                ping: MethodResult::Err("connect_timeout".into()),
                price: MethodResult::Skipped, ticker: MethodResult::Skipped,
                orderbook: MethodResult::Skipped, klines: MethodResult::Skipped,
                trades: MethodResult::Skipped, exch_info: MethodResult::Skipped,
                funding: MethodResult::Skipped, open_interest: MethodResult::Skipped,
                mark_price: MethodResult::Skipped, long_short: MethodResult::Skipped,
                liquidations: MethodResult::Skipped, premium_index: MethodResult::Skipped,
                ws_ticker: MethodResult::Skipped, ws_trade: MethodResult::Skipped,
                ws_orderbook: MethodResult::Skipped, ws_kline: MethodResult::Skipped,
                ws_mark_price: MethodResult::Skipped, ws_funding: MethodResult::Skipped,
                ws_liquidation: MethodResult::Skipped, ws_oi: MethodResult::Skipped,
                ws_agg_trade: MethodResult::Skipped,
                issues: vec!["connect_timeout".into()],
            };
        }
    };
    let _ = connected;

    let conn = match hub.rest(id) {
        Some(c) => c,
        None => {
            return MarketRow {
                exchange: format!("{:?}", id),
                ping: MethodResult::Err("no_rest_handle".into()),
                price: MethodResult::Skipped, ticker: MethodResult::Skipped,
                orderbook: MethodResult::Skipped, klines: MethodResult::Skipped,
                trades: MethodResult::Skipped, exch_info: MethodResult::Skipped,
                funding: MethodResult::Skipped, open_interest: MethodResult::Skipped,
                mark_price: MethodResult::Skipped, long_short: MethodResult::Skipped,
                liquidations: MethodResult::Skipped, premium_index: MethodResult::Skipped,
                ws_ticker: MethodResult::Skipped, ws_trade: MethodResult::Skipped,
                ws_orderbook: MethodResult::Skipped, ws_kline: MethodResult::Skipped,
                ws_mark_price: MethodResult::Skipped, ws_funding: MethodResult::Skipped,
                ws_liquidation: MethodResult::Skipped, ws_oi: MethodResult::Skipped,
                ws_agg_trade: MethodResult::Skipped,
                issues: vec!["no_rest_handle".into()],
            };
        }
    };

    let caps = hub.capabilities(id).unwrap_or_default();
    let sym_input_str = raw_str.clone();

    // ── Helper macros for REST calls ─────────────────────────────────────────

    // ping
    let ping = {
        let conn = conn.clone();
        match timeout(Duration::from_secs(10), conn.ping()).await {
            Ok(Ok(())) => MethodResult::Ok("pong".into()),
            Ok(Err(e)) => {
                let msg = e.to_string();
                if msg.contains("UnsupportedOperation") || msg.contains("Not supported:") { MethodResult::Unsupported(truncate(&msg, 50)) }
                else { MethodResult::Err(truncate(&msg, 60)) }
            }
            Err(_) => MethodResult::Timeout,
        }
    };

    // price
    let price = if !caps.has_ticker {
        MethodResult::Skipped
    } else {
        let conn = conn.clone();
        let sym_str = sym_input_str.clone();
        match timeout(Duration::from_secs(10),
            conn.get_price(sym_str.as_str().into(), account_type)).await {
            Ok(Ok(p)) if p > 0.0 => MethodResult::Ok(format!("price={:.4}", p)),
            Ok(Ok(_)) => MethodResult::Empty,
            Ok(Err(e)) => {
                let msg = e.to_string();
                if msg.contains("UnsupportedOperation") || msg.contains("Not supported:") { MethodResult::Unsupported(truncate(&msg, 50)) }
                else { MethodResult::Err(truncate(&msg, 60)) }
            }
            Err(_) => MethodResult::Timeout,
        }
    };

    // ticker
    let ticker = if !caps.has_ticker {
        MethodResult::Skipped
    } else {
        let conn = conn.clone();
        let sym_str = sym_input_str.clone();
        match timeout(Duration::from_secs(10),
            MarketData::get_ticker(&*conn, sym_str.as_str().into(), account_type)).await {
            Ok(Ok(t)) => {
                let mut issues: Vec<String> = Vec::new();
                if t.last_price <= 0.0 { issues.push("last=0".into()); }
                if timestamp_unit_bug(t.timestamp) { issues.push(format!("ts_unit_bug:{}", t.timestamp)); }
                else if timestamp_future_bug(t.timestamp) { issues.push(format!("ts_future_bug:{}", t.timestamp)); }
                else if t.timestamp == 0 { issues.push("ts_missing".into()); }
                match (t.bid_price, t.ask_price) {
                    (Some(b), Some(a)) if b > a => issues.push(format!("bid>ask")),
                    (None, None) if !no_bid_ask_by_design(id) => issues.push("bid/ask None".into()),
                    _ => {}
                }
                let desc = format!("last={:.4} bid={} ask={} ts={}",
                    t.last_price,
                    t.bid_price.map(|v| format!("{:.4}", v)).unwrap_or_else(|| "None".into()),
                    t.ask_price.map(|v| format!("{:.4}", v)).unwrap_or_else(|| "None".into()),
                    if t.timestamp == 0 { "MISSING".to_string() } else { format!("{}s_ago", (now_ms() - t.timestamp) / 1000) });
                if issues.is_empty() && t.last_price > 0.0 {
                    MethodResult::Ok(desc)
                } else if t.last_price > 0.0 {
                    MethodResult::Err(format!("{} ISSUES:{}", desc, issues.join(",")))
                } else {
                    MethodResult::Empty
                }
            }
            Ok(Err(e)) => {
                let msg = e.to_string();
                if msg.contains("UnsupportedOperation") || msg.contains("Not supported:") { MethodResult::Unsupported(truncate(&msg, 50)) }
                else { MethodResult::Err(truncate(&msg, 60)) }
            }
            Err(_) => MethodResult::Timeout,
        }
    };

    // orderbook
    let orderbook = if !caps.has_orderbook {
        MethodResult::Skipped
    } else {
        let conn = conn.clone();
        let sym_str = sym_input_str.clone();
        match timeout(Duration::from_secs(10),
            conn.get_orderbook(sym_str.as_str().into(), Some(10), account_type)).await {
            Ok(Ok(ob)) => {
                if ob.bids.is_empty() || ob.asks.is_empty() {
                    MethodResult::Empty
                } else {
                    let top_bid = ob.bids.first().map(|l| l.price).unwrap_or(0.0);
                    let top_ask = ob.asks.first().map(|l| l.price).unwrap_or(0.0);
                    if top_bid >= top_ask && top_ask > 0.0 {
                        MethodResult::Err(format!("bid={:.4}>=ask={:.4}", top_bid, top_ask))
                    } else {
                        MethodResult::Ok(format!("bids={} asks={} top_bid={:.4} top_ask={:.4}",
                            ob.bids.len(), ob.asks.len(), top_bid, top_ask))
                    }
                }
            }
            Ok(Err(e)) => {
                let msg = e.to_string();
                if msg.contains("UnsupportedOperation") || msg.contains("Not supported:") { MethodResult::Unsupported(truncate(&msg, 50)) }
                else { MethodResult::Err(truncate(&msg, 60)) }
            }
            Err(_) => MethodResult::Timeout,
        }
    };

    // klines
    let klines = if !caps.has_klines {
        MethodResult::Skipped
    } else {
        let conn = conn.clone();
        let sym_str = sym_input_str.clone();
        match timeout(Duration::from_secs(12),
            conn.get_klines(sym_str.as_str().into(), "1m", Some(5), account_type, None)).await {
            Ok(Ok(ks)) if ks.is_empty() => MethodResult::Empty,
            Ok(Ok(ks)) => {
                let last = ks.last().unwrap();
                if last.close <= 0.0 {
                    MethodResult::Err(format!("close={}", last.close))
                } else {
                    MethodResult::Ok(format!("len={} last_close={:.4}", ks.len(), last.close))
                }
            }
            Ok(Err(e)) => {
                let msg = e.to_string();
                if msg.contains("UnsupportedOperation") || msg.contains("Not supported:") { MethodResult::Unsupported(truncate(&msg, 50)) }
                else { MethodResult::Err(truncate(&msg, 60)) }
            }
            Err(_) => MethodResult::Timeout,
        }
    };

    // recent trades
    let trades = if !caps.has_recent_trades {
        MethodResult::Skipped
    } else {
        let conn = conn.clone();
        let sym_str = sym_input_str.clone();
        match timeout(Duration::from_secs(10),
            conn.get_recent_trades(sym_str.as_str().into(), Some(10), account_type)).await {
            Ok(Ok(ts)) if ts.is_empty() => MethodResult::Empty,
            Ok(Ok(ts)) => {
                let first = ts.first().unwrap();
                if first.price <= 0.0 {
                    MethodResult::Err(format!("price={}", first.price))
                } else {
                    MethodResult::Ok(format!("len={} first_px={:.4}", ts.len(), first.price))
                }
            }
            Ok(Err(e)) => {
                let msg = e.to_string();
                if msg.contains("UnsupportedOperation") || msg.contains("Not supported:") { MethodResult::Unsupported(truncate(&msg, 50)) }
                else { MethodResult::Err(truncate(&msg, 60)) }
            }
            Err(_) => MethodResult::Timeout,
        }
    };

    // exchange_info
    let exch_info = if !caps.has_exchange_info {
        MethodResult::Skipped
    } else {
        let conn = conn.clone();
        match timeout(Duration::from_secs(15),
            conn.get_exchange_info(account_type)).await {
            Ok(Ok(infos)) if infos.is_empty() => MethodResult::Empty,
            Ok(Ok(infos)) => MethodResult::Ok(format!("symbols={}", infos.len())),
            Ok(Err(e)) => {
                let msg = e.to_string();
                if msg.contains("UnsupportedOperation") || msg.contains("Not supported:") { MethodResult::Unsupported(truncate(&msg, 50)) }
                else { MethodResult::Err(truncate(&msg, 60)) }
            }
            Err(_) => MethodResult::Timeout,
        }
    };

    // ── Futures-only REST ─────────────────────────────────────────────────────

    let fut_sym = raw_str.clone();
    let futures_at = if matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated) {
        account_type
    } else {
        AccountType::FuturesCross
    };

    // funding_rate
    let funding = if !futures_capable || !caps.has_funding_payments {
        MethodResult::Skipped
    } else {
        let conn = conn.clone();
        let s = fut_sym.clone();
        match timeout(Duration::from_secs(10),
            conn.get_funding_rate(&s, futures_at)).await {
            Ok(Ok(fr)) => MethodResult::Ok(format!("rate={:.6} next={:?}", fr.rate, fr.next_funding_time)),
            Ok(Err(e)) => {
                let msg = e.to_string();
                if msg.contains("UnsupportedOperation") || msg.contains("Not supported:") { MethodResult::Unsupported(truncate(&msg, 50)) }
                else { MethodResult::Err(truncate(&msg, 60)) }
            }
            Err(_) => MethodResult::Timeout,
        }
    };

    // open_interest
    let open_interest = if !futures_capable {
        MethodResult::Skipped
    } else {
        let conn = conn.clone();
        let s = fut_sym.clone();
        match timeout(Duration::from_secs(10),
            conn.get_open_interest(&s, futures_at)).await {
            Ok(Ok(oi)) if oi.open_interest <= 0.0 => MethodResult::Empty,
            Ok(Ok(oi)) => MethodResult::Ok(format!("oi={:.2}", oi.open_interest)),
            Ok(Err(e)) => {
                let msg = e.to_string();
                if msg.contains("UnsupportedOperation") || msg.contains("Not supported:") { MethodResult::Unsupported(truncate(&msg, 50)) }
                else { MethodResult::Err(truncate(&msg, 60)) }
            }
            Err(_) => MethodResult::Timeout,
        }
    };

    // mark_price
    let mark_price = if !futures_capable || !caps.has_mark_price {
        MethodResult::Skipped
    } else {
        let conn = conn.clone();
        let s = fut_sym.clone();
        match timeout(Duration::from_secs(10),
            conn.get_mark_price(&s)).await {
            Ok(Ok(mp)) if mp.mark_price <= 0.0 => MethodResult::Empty,
            Ok(Ok(mp)) => MethodResult::Ok(format!("mark={:.4}", mp.mark_price)),
            Ok(Err(e)) => {
                let msg = e.to_string();
                if msg.contains("UnsupportedOperation") || msg.contains("Not supported:") { MethodResult::Unsupported(truncate(&msg, 50)) }
                else { MethodResult::Err(truncate(&msg, 60)) }
            }
            Err(_) => MethodResult::Timeout,
        }
    };

    // long_short_ratio
    let long_short = if !futures_capable || !caps.has_long_short_ratio {
        MethodResult::Skipped
    } else {
        let conn = conn.clone();
        let s = fut_sym.clone();
        match timeout(Duration::from_secs(10),
            conn.get_long_short_ratio(&s, futures_at)).await {
            Ok(Ok(ls)) => MethodResult::Ok(format!("long={:.4} short={:.4}", ls.long_ratio, ls.short_ratio)),
            Ok(Err(e)) => {
                let msg = e.to_string();
                if msg.contains("UnsupportedOperation") || msg.contains("Not supported:") { MethodResult::Unsupported(truncate(&msg, 50)) }
                else { MethodResult::Err(truncate(&msg, 60)) }
            }
            Err(_) => MethodResult::Timeout,
        }
    };

    // liquidations (history)
    let liquidations = if !futures_capable || !caps.has_liquidation_history {
        MethodResult::Skipped
    } else {
        let conn = conn.clone();
        let sym_str = sym_input_str.clone();
        match timeout(Duration::from_secs(10),
            conn.get_liquidation_history(
                Some(SymbolInput::Raw(&sym_str)),
                None, None, Some(5), futures_at)).await {
            Ok(Ok(ls)) if ls.is_empty() => MethodResult::Empty,
            Ok(Ok(ls)) => MethodResult::Ok(format!("len={}", ls.len())),
            Ok(Err(e)) => {
                let msg = e.to_string();
                if msg.contains("UnsupportedOperation") || msg.contains("Not supported:") { MethodResult::Unsupported(truncate(&msg, 50)) }
                else { MethodResult::Err(truncate(&msg, 60)) }
            }
            Err(_) => MethodResult::Timeout,
        }
    };

    // premium_index (mark+index price)
    let premium_index = if !futures_capable || !caps.has_premium_index {
        MethodResult::Skipped
    } else {
        let conn = conn.clone();
        let sym_str = sym_input_str.clone();
        match timeout(Duration::from_secs(10),
            conn.get_premium_index(Some(SymbolInput::Raw(&sym_str)), futures_at)).await {
            Ok(Ok(ps)) if ps.is_empty() => MethodResult::Empty,
            Ok(Ok(ps)) => MethodResult::Ok(format!("len={} mark={:.4}", ps.len(),
                ps.first().map(|p| p.mark_price).unwrap_or(0.0))),
            Ok(Err(e)) => {
                let msg = e.to_string();
                if msg.contains("UnsupportedOperation") || msg.contains("Not supported:") { MethodResult::Unsupported(truncate(&msg, 50)) }
                else { MethodResult::Err(truncate(&msg, 60)) }
            }
            Err(_) => MethodResult::Timeout,
        }
    };

    // ── WS subscriptions ──────────────────────────────────────────────────────
    // Each WS sub creates its own hub+WS connection to avoid sharing state.

    let sym_ws = sym.clone();
    let sym_ws2 = sym.clone();
    let sym_ws3 = sym.clone();
    let sym_ws4 = sym.clone();
    let sym_ws5 = sym.clone();
    let sym_ws6 = sym.clone();
    let sym_ws7 = sym.clone();
    let sym_ws8 = sym.clone();
    let sym_ws9 = sym.clone();

    let ws_ticker_fut = async {
        if !caps.has_ws_ticker { return MethodResult::Skipped; }
        market::run_ws_sub(id, account_type, StreamType::Ticker, sym_ws, market::ExpectedKind::Ticker, stale_ms, 10).await
    };
    let ws_trade_fut = async {
        if !caps.has_ws_trades { return MethodResult::Skipped; }
        market::run_ws_sub(id, account_type, StreamType::Trade, sym_ws2, market::ExpectedKind::Trade, stale_ms, 10).await
    };
    let ws_ob_fut = async {
        if !caps.has_ws_orderbook { return MethodResult::Skipped; }
        market::run_ws_sub(id, account_type, StreamType::Orderbook, sym_ws3, market::ExpectedKind::Orderbook, stale_ms, 10).await
    };
    let ws_kline_fut = async {
        if !caps.has_ws_klines { return MethodResult::Skipped; }
        market::run_ws_sub(id, account_type, StreamType::Kline { interval: "1m".into() }, sym_ws4, market::ExpectedKind::Kline, stale_ms, 10).await
    };
    let ws_mark_fut = async {
        if !futures_capable || !caps.has_ws_mark_price { return MethodResult::Skipped; }
        market::run_ws_sub(id, futures_at, StreamType::MarkPrice, sym_ws5, market::ExpectedKind::MarkPrice, stale_ms, 10).await
    };
    let ws_funding_fut = async {
        if !futures_capable || !caps.has_ws_funding_rate { return MethodResult::Skipped; }
        market::run_ws_sub(id, futures_at, StreamType::FundingRate, sym_ws6, market::ExpectedKind::FundingRate, stale_ms, 10).await
    };
    let ws_liq_fut = async {
        if !futures_capable { return MethodResult::Skipped; }
        // Liquidation fires at market events (not periodic) — use 30s window.
        // Binance: empty symbol → !forceOrder@arr all-symbols feed (high freq).
        // GateIO: "!all" → all-symbols public_liquidates feed (low freq ~25/hr).
        // Bybit: per-symbol only (no all-symbols variant in V5).  Single-symbol
        //   windows of 30s are too short — spawn 5 high-volume symbols in
        //   parallel and take the first non-silent result within 45s.
        let liq_sym = liq_symbol_for(id);
        drop(sym_ws7); // replaced by liq_sym above
        if id == ExchangeId::Bybit {
            // Subscribe to 5 high-volume perp symbols concurrently. Based on a 1-hour
            // raw capture against Bybit V5 on 2026-05-19:
            //   BTCUSDT  29 liqs / 60min (1 per ~2.1 min)
            //   ETHUSDT   8 liqs / 60min
            //   SOLUSDT   7 liqs / 60min
            //   XRPUSDT   3 liqs / 60min
            //   DOGEUSDT  4 liqs / 60min
            //   Total 51 liqs / 60min = 1 per ~70s across 5 symbols.
            // With a 60s window across 5 parallel subs, hit probability ≈ 75-80%.
            let bybit_liq_syms: &[&str] = &["BTCUSDT", "ETHUSDT", "SOLUSDT", "XRPUSDT", "DOGEUSDT"];
            let mut handles = Vec::new();
            for &sym_str in bybit_liq_syms {
                let sym = Symbol::with_raw("", "", sym_str.to_string());
                let h = tokio::spawn(market::run_ws_sub(
                    ExchangeId::Bybit,
                    futures_at,
                    StreamType::Liquidation,
                    sym,
                    market::ExpectedKind::Liquidation,
                    stale_ms,
                    60,
                ));
                handles.push(h);
            }
            // Wait for all; return first OK or the last result if all silent.
            let mut last = MethodResult::Err("silent_0_events".into());
            for h in handles {
                if let Ok(r) = h.await {
                    match &r {
                        MethodResult::Ok(_) => return r,
                        other => last = other.clone(),
                    }
                }
            }
            last
        } else {
            market::run_ws_sub(id, futures_at, StreamType::Liquidation, liq_sym, market::ExpectedKind::Liquidation, stale_ms, 30).await
        }
    };
    let ws_oi_fut = async {
        if !futures_capable { return MethodResult::Skipped; }
        // OpenInterest update cadence varies by exchange — use 20s window.
        market::run_ws_sub(id, futures_at, StreamType::OpenInterest, sym_ws8, market::ExpectedKind::OpenInterest, stale_ms, 20).await
    };
    let ws_agg_fut = async {
        if !futures_capable { return MethodResult::Skipped; }
        market::run_ws_sub(id, futures_at, StreamType::AggTrade, sym_ws9, market::ExpectedKind::AggTrade, stale_ms, 10).await
    };

    let (ws_ticker, ws_trade, ws_orderbook, ws_kline, ws_mark_price, ws_funding, ws_liquidation, ws_oi, ws_agg_trade) =
        tokio::join!(
            ws_ticker_fut, ws_trade_fut, ws_ob_fut, ws_kline_fut,
            ws_mark_fut, ws_funding_fut, ws_liq_fut, ws_oi_fut, ws_agg_fut
        );

    // ── Collect issues ────────────────────────────────────────────────────────
    let mut issues: Vec<String> = Vec::new();
    let method_cells = [
        ("ping", &ping), ("price", &price), ("ticker", &ticker),
        ("orderbook", &orderbook), ("klines", &klines), ("trades", &trades),
        ("exch_info", &exch_info), ("funding", &funding), ("OI", &open_interest),
        ("mark_px", &mark_price), ("ls_ratio", &long_short), ("liquidations", &liquidations),
        ("premium_idx", &premium_index),
        ("WS_ticker", &ws_ticker), ("WS_trade", &ws_trade), ("WS_ob", &ws_orderbook),
        ("WS_kline", &ws_kline), ("WS_mark", &ws_mark_price), ("WS_funding", &ws_funding),
        ("WS_liq", &ws_liquidation), ("WS_oi", &ws_oi), ("WS_agg", &ws_agg_trade),
    ];
    for (name, result) in &method_cells {
        if result.is_issue() {
            if let Some(d) = result.detail() {
                issues.push(format!("{}: {}", name, d));
            } else {
                issues.push(format!("{}: {:?}", name, result));
            }
        }
    }

    MarketRow {
        exchange: format!("{:?}", id),
        ping, price, ticker, orderbook, klines, trades, exch_info,
        funding, open_interest, mark_price, long_short, liquidations, premium_index,
        ws_ticker, ws_trade, ws_orderbook: ws_orderbook, ws_kline, ws_mark_price, ws_funding,
        ws_liquidation, ws_oi, ws_agg_trade,
        issues,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// mod trading
// ─────────────────────────────────────────────────────────────────────────────

mod trading {
    use super::*;

    /// ENV var names for each exchange's credentials.
    pub fn load_credentials(id: ExchangeId) -> Option<Credentials> {
        let (key_env, secret_env, pass_env): (&str, &str, Option<&str>) = match id {
            ExchangeId::Binance  => ("BINANCE_API_KEY", "BINANCE_API_SECRET", None),
            ExchangeId::Bybit    => ("BYBIT_API_KEY", "BYBIT_API_SECRET", None),
            ExchangeId::OKX      => ("OKX_API_KEY", "OKX_API_SECRET", Some("OKX_PASSPHRASE")),
            ExchangeId::KuCoin   => ("KUCOIN_API_KEY", "KUCOIN_API_SECRET", Some("KUCOIN_PASSPHRASE")),
            ExchangeId::GateIO   => ("GATEIO_API_KEY", "GATEIO_API_SECRET", None),
            ExchangeId::MEXC     => ("MEXC_API_KEY", "MEXC_API_SECRET", None),
            ExchangeId::HTX      => ("HTX_API_KEY", "HTX_API_SECRET", None),
            ExchangeId::Bitget   => ("BITGET_API_KEY", "BITGET_API_SECRET", Some("BITGET_PASSPHRASE")),
            ExchangeId::BingX    => ("BINGX_API_KEY", "BINGX_API_SECRET", None),
            ExchangeId::CryptoCom => ("CRYPTOCOM_API_KEY", "CRYPTOCOM_API_SECRET", None),
            ExchangeId::Bitfinex => ("BITFINEX_API_KEY", "BITFINEX_API_SECRET", None),
            ExchangeId::Gemini   => ("GEMINI_API_KEY", "GEMINI_API_SECRET", None),
            ExchangeId::Bitstamp => ("BITSTAMP_API_KEY", "BITSTAMP_API_SECRET", None),
            ExchangeId::Kraken   => ("KRAKEN_API_KEY", "KRAKEN_API_SECRET", None),
            ExchangeId::Coinbase => ("COINBASE_API_KEY", "COINBASE_API_SECRET", None),
            ExchangeId::Deribit  => ("DERIBIT_API_KEY", "DERIBIT_API_SECRET", None),
            ExchangeId::HyperLiquid => ("HYPERLIQUID_API_KEY", "HYPERLIQUID_API_SECRET", None),
            ExchangeId::Dydx     => ("DYDX_API_KEY", "DYDX_API_SECRET", None),
            ExchangeId::Upbit    => ("UPBIT_API_KEY", "UPBIT_API_SECRET", None),
            _ => return None,
        };

        let api_key = std::env::var(key_env).ok()?;
        let api_secret = std::env::var(secret_env).ok()?;
        let passphrase = pass_env.and_then(|e| std::env::var(e).ok());

        if api_key.is_empty() || api_secret.is_empty() { return None; }

        Some(Credentials { api_key, api_secret, passphrase, testnet: false })
    }
}

async fn test_trading(id: ExchangeId) -> TradingRow {
    let harness = TestHarness::new();
    let conn = match harness.create_authenticated(id).await {
        None => {
            // Also check direct ENV vars as fallback
            match trading::load_credentials(id) {
                None => {
                    return TradingRow {
                        exchange: format!("{:?}", id),
                        balance: MethodResult::Skipped,
                        account_info: MethodResult::Skipped,
                        open_orders: MethodResult::Skipped,
                        user_trades: MethodResult::Skipped,
                        positions: MethodResult::Skipped,
                        fees: MethodResult::Skipped,
                        issues: vec!["no_credentials_in_env".into()],
                    };
                }
                Some(_) => {
                    // Has ENV creds but TestHarness didn't pick them up (.env file missing)
                    return TradingRow {
                        exchange: format!("{:?}", id),
                        balance: MethodResult::Skipped,
                        account_info: MethodResult::Skipped,
                        open_orders: MethodResult::Skipped,
                        user_trades: MethodResult::Skipped,
                        positions: MethodResult::Skipped,
                        fees: MethodResult::Skipped,
                        issues: vec!["creds_in_env_but_not_dotenv".into()],
                    };
                }
            }
        }
        Some(Err(e)) => {
            let msg = truncate(&e.to_string(), 70);
            return TradingRow {
                exchange: format!("{:?}", id),
                balance: MethodResult::Err(format!("auth_connect_fail: {}", msg)),
                account_info: MethodResult::Skipped,
                open_orders: MethodResult::Skipped,
                user_trades: MethodResult::Skipped,
                positions: MethodResult::Skipped,
                fees: MethodResult::Skipped,
                issues: vec![format!("auth_fail: {}", msg)],
            };
        }
        Some(Ok(c)) => c,
    };

    let (_, raw_str, account_type) = raw_symbol_for(id);
    let futures_at = if matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated) {
        account_type
    } else {
        AccountType::FuturesCross
    };

    // balance
    let balance = {
        let conn = conn.clone();
        match timeout(Duration::from_secs(10),
            conn.get_balance(BalanceQuery { asset: None, account_type })).await {
            Ok(Ok(bs)) => MethodResult::Ok(format!("assets={}", bs.len())),
            Ok(Err(e)) => {
                let msg = e.to_string();
                if msg.contains("UnsupportedOperation") || msg.contains("Not supported:") { MethodResult::Unsupported(truncate(&msg, 50)) }
                else { MethodResult::Err(truncate(&msg, 60)) }
            }
            Err(_) => MethodResult::Timeout,
        }
    };

    // account_info
    let account_info = {
        let conn = conn.clone();
        match timeout(Duration::from_secs(10),
            conn.get_account_info(account_type)).await {
            Ok(Ok(_)) => MethodResult::Ok("ok".into()),
            Ok(Err(e)) => {
                let msg = e.to_string();
                if msg.contains("UnsupportedOperation") || msg.contains("Not supported:") { MethodResult::Unsupported(truncate(&msg, 50)) }
                else { MethodResult::Err(truncate(&msg, 60)) }
            }
            Err(_) => MethodResult::Timeout,
        }
    };

    // open_orders
    let open_orders = {
        let conn = conn.clone();
        let s = raw_str.clone();
        match timeout(Duration::from_secs(10),
            conn.get_open_orders(Some(&s), account_type)).await {
            Ok(Ok(os)) => MethodResult::Ok(format!("count={}", os.len())),
            Ok(Err(e)) => {
                let msg = e.to_string();
                if msg.contains("UnsupportedOperation") || msg.contains("Not supported:") { MethodResult::Unsupported(truncate(&msg, 50)) }
                else { MethodResult::Err(truncate(&msg, 60)) }
            }
            Err(_) => MethodResult::Timeout,
        }
    };

    // user_trades
    let user_trades = {
        let conn = conn.clone();
        let s = raw_str.clone();
        match timeout(Duration::from_secs(10),
            conn.get_user_trades(
                UserTradeFilter { symbol: Some(s), order_id: None, start_time: None, end_time: None, limit: Some(5) },
                account_type)).await {
            Ok(Ok(ts)) => MethodResult::Ok(format!("count={}", ts.len())),
            Ok(Err(e)) => {
                let msg = e.to_string();
                if msg.contains("UnsupportedOperation") || msg.contains("Not supported:") { MethodResult::Unsupported(truncate(&msg, 50)) }
                else { MethodResult::Err(truncate(&msg, 60)) }
            }
            Err(_) => MethodResult::Timeout,
        }
    };

    // positions (futures)
    let positions = if !is_futures(id, account_type) {
        MethodResult::Skipped
    } else {
        let conn = conn.clone();
        match timeout(Duration::from_secs(10),
            conn.get_positions(PositionQuery { symbol: None, account_type: futures_at })).await {
            Ok(Ok(ps)) => MethodResult::Ok(format!("count={}", ps.len())),
            Ok(Err(e)) => {
                let msg = e.to_string();
                if msg.contains("UnsupportedOperation") || msg.contains("Not supported:") { MethodResult::Unsupported(truncate(&msg, 50)) }
                else { MethodResult::Err(truncate(&msg, 60)) }
            }
            Err(_) => MethodResult::Timeout,
        }
    };

    // fees
    let fees = {
        let conn = conn.clone();
        let s = raw_str.clone();
        match timeout(Duration::from_secs(10), conn.get_fees(Some(&s))).await {
            Ok(Ok(f)) => MethodResult::Ok(format!("maker={:.6} taker={:.6}", f.maker_rate, f.taker_rate)),
            Ok(Err(e)) => {
                let msg = e.to_string();
                if msg.contains("UnsupportedOperation") || msg.contains("Not supported:") { MethodResult::Unsupported(truncate(&msg, 50)) }
                else { MethodResult::Err(truncate(&msg, 60)) }
            }
            Err(_) => MethodResult::Timeout,
        }
    };

    let mut issues: Vec<String> = Vec::new();
    for (name, result) in [
        ("balance", &balance), ("account_info", &account_info),
        ("open_orders", &open_orders), ("user_trades", &user_trades),
        ("positions", &positions), ("fees", &fees),
    ] {
        if result.is_issue() {
            if let Some(d) = result.detail() {
                issues.push(format!("{}: {}", name, d));
            } else {
                issues.push(format!("{}: {:?}", name, result));
            }
        }
    }

    TradingRow {
        exchange: format!("{:?}", id),
        balance, account_info, open_orders, user_trades, positions, fees,
        issues,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// MOEX direct test (factory blocks WS, but impl exists)
// ─────────────────────────────────────────────────────────────────────────────

async fn test_moex_market() -> MarketRow {
    let hub = ExchangeHub::new();
    let symbol_moex = Symbol::new("GAZP", "");
    let symbol_moex_str = SymbolNormalizer::to_exchange(ExchangeId::Moex, &symbol_moex, AccountType::Spot)
        .unwrap_or_else(|_| "GAZP".to_string());
    let stale_ms = stale_threshold_ms();
    let account_type = AccountType::Spot;

    let connected = match timeout(Duration::from_secs(10), hub.connect_public(ExchangeId::Moex, false)).await {
        Ok(Ok(())) => true,
        _ => false,
    };

    let (ping, price, ticker, orderbook, klines, trades, exch_info) = if !connected {
        (MethodResult::Err("connect_fail".into()), MethodResult::Skipped, MethodResult::Skipped,
         MethodResult::Skipped, MethodResult::Skipped, MethodResult::Skipped, MethodResult::Skipped)
    } else {
        let conn = match hub.rest(ExchangeId::Moex) {
            Some(c) => c,
            None => {
                return MarketRow {
                    exchange: "Moex".into(),
                    ping: MethodResult::Err("no_rest_handle".into()),
                    price: MethodResult::Skipped, ticker: MethodResult::Skipped,
                    orderbook: MethodResult::Skipped, klines: MethodResult::Skipped,
                    trades: MethodResult::Skipped, exch_info: MethodResult::Skipped,
                    funding: MethodResult::Skipped, open_interest: MethodResult::Skipped,
                    mark_price: MethodResult::Skipped, long_short: MethodResult::Skipped,
                    liquidations: MethodResult::Skipped, premium_index: MethodResult::Skipped,
                    ws_ticker: MethodResult::Skipped, ws_trade: MethodResult::Skipped,
                    ws_orderbook: MethodResult::Skipped, ws_kline: MethodResult::Skipped,
                    ws_mark_price: MethodResult::Skipped, ws_funding: MethodResult::Skipped,
                    ws_liquidation: MethodResult::Skipped, ws_oi: MethodResult::Skipped,
                    ws_agg_trade: MethodResult::Skipped,
                    issues: vec!["no_rest_handle".into()],
                };
            }
        };
        let sym_str = symbol_moex_str.clone();
        let ping = match timeout(Duration::from_secs(8), conn.ping()).await {
            Ok(Ok(())) => MethodResult::Ok("pong".into()),
            Ok(Err(e)) => MethodResult::Err(truncate(&e.to_string(), 60)),
            Err(_) => MethodResult::Timeout,
        };
        let ticker = match timeout(Duration::from_secs(10),
            MarketData::get_ticker(&*conn, sym_str.as_str().into(), account_type)).await {
            Ok(Ok(t)) => {
                let mut issues = Vec::new();
                if t.last_price <= 0.0 { issues.push("last=0"); }
                if timestamp_unit_bug(t.timestamp) { issues.push("ts_unit_bug"); }
                if issues.is_empty() { MethodResult::Ok(format!("last={:.4}", t.last_price)) }
                else { MethodResult::Err(format!("last={:.4} ISSUES:{}", t.last_price, issues.join(","))) }
            }
            Ok(Err(e)) => MethodResult::Err(truncate(&e.to_string(), 60)),
            Err(_) => MethodResult::Timeout,
        };
        (ping, MethodResult::Skipped, ticker, MethodResult::Skipped, MethodResult::Skipped, MethodResult::Skipped, MethodResult::Skipped)
    };

    // MOEX WS — direct construction
    let ws_ticker = {
        let ws = Arc::new(MoexWebSocket::new_public()) as Arc<dyn WebSocketConnector>;
        let moex_sym = Symbol::new("GAZP", "");
        let sub = SubscriptionRequest::ticker_for(moex_sym, AccountType::Spot);
        match timeout(Duration::from_secs(20),
            market::collect_ws_stream(ws, sub, market::ExpectedKind::Ticker, stale_ms, 5)).await {
            Ok(r) => r,
            Err(_) => MethodResult::Err("overall_timeout_20s".into()),
        }
    };

    let mut issues: Vec<String> = Vec::new();
    for (name, result) in [("ping", &ping), ("ticker", &ticker), ("WS_ticker", &ws_ticker)] {
        if result.is_issue() {
            if let Some(d) = result.detail() { issues.push(format!("{}: {}", name, d)); }
        }
    }

    MarketRow {
        exchange: "Moex".into(),
        ping, price, ticker, orderbook, klines, trades, exch_info,
        funding: MethodResult::Skipped, open_interest: MethodResult::Skipped,
        mark_price: MethodResult::Skipped, long_short: MethodResult::Skipped,
        liquidations: MethodResult::Skipped, premium_index: MethodResult::Skipped,
        ws_ticker, ws_trade: MethodResult::Skipped, ws_orderbook: MethodResult::Skipped,
        ws_kline: MethodResult::Skipped, ws_mark_price: MethodResult::Skipped,
        ws_funding: MethodResult::Skipped, ws_liquidation: MethodResult::Skipped,
        ws_oi: MethodResult::Skipped, ws_agg_trade: MethodResult::Skipped,
        issues,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// mod report — pretty print
// ─────────────────────────────────────────────────────────────────────────────

mod report {
    use super::*;

    pub fn print_market_matrix(rows: &[MarketRow]) {
        println!();
        println!("=== MARKET COVERAGE MATRIX ===");
        println!("{:<18} | REST                                                          | WS", "");
        println!("{:<18} | ping pric tick ob   klin trad exch fund OI   mark ls   liq  px | tick trad ob   klin mark fund liq  OI   agg", "Exchange");
        println!("{}", "-".repeat(170));
        for row in rows {
            let rest_cells = [
                row.ping.cell(), row.price.cell(), row.ticker.cell(), row.orderbook.cell(),
                row.klines.cell(), row.trades.cell(), row.exch_info.cell(),
                row.funding.cell(), row.open_interest.cell(), row.mark_price.cell(),
                row.long_short.cell(), row.liquidations.cell(), row.premium_index.cell(),
            ];
            let ws_cells = [
                row.ws_ticker.cell(), row.ws_trade.cell(), row.ws_orderbook.cell(),
                row.ws_kline.cell(), row.ws_mark_price.cell(), row.ws_funding.cell(),
                row.ws_liquidation.cell(), row.ws_oi.cell(), row.ws_agg_trade.cell(),
            ];
            let rest_str = rest_cells.join(" ");
            let ws_str = ws_cells.join(" ");
            println!("{:<18} | {} | {}", row.exchange, rest_str, ws_str);
        }
    }

    pub fn print_trading_matrix(rows: &[TradingRow]) {
        println!();
        println!("=== TRADING COVERAGE MATRIX ===");
        println!("{:<18} | balance  acc_info open_ord usr_trd  positions fees", "Exchange");
        println!("{}", "-".repeat(80));
        for row in rows {
            let cells = [
                row.balance.cell(), row.account_info.cell(), row.open_orders.cell(),
                row.user_trades.cell(), row.positions.cell(), row.fees.cell(),
            ];
            println!("{:<18} | {}", row.exchange, cells.join("  "));
        }
    }

    pub fn print_summaries(market_rows: &[MarketRow], trading_rows: &[TradingRow]) {
        // TRUSTED
        let trusted: Vec<&str> = market_rows.iter()
            .filter(|r| {
                r.ping.is_ok() && r.ticker.is_ok() && r.orderbook.is_ok() && r.klines.is_ok()
                && r.ws_ticker.is_ok()
                && r.issues.is_empty()
            })
            .map(|r| r.exchange.as_str())
            .collect();
        println!();
        println!("=== TRUSTED (ping+ticker+ob+klines OK, WS_ticker OK, no issues) ===");
        println!("Count: {}", trusted.len());
        for ex in &trusted { println!("  + {}", ex); }

        // PARTIAL
        let partial: Vec<&MarketRow> = market_rows.iter()
            .filter(|r| !r.issues.is_empty() && (r.ping.is_ok() || r.ticker.is_ok()))
            .collect();
        println!();
        println!("=== PARTIAL (some methods fail) ===");
        for row in &partial {
            println!("  {} | {}", row.exchange, row.issues.first().map(|s| s.as_str()).unwrap_or(""));
        }

        // ISSUES BY EXCHANGE
        let has_issues: Vec<&MarketRow> = market_rows.iter().filter(|r| !r.issues.is_empty()).collect();
        println!();
        println!("=== ISSUES BY EXCHANGE ===");
        for row in &has_issues {
            for iss in &row.issues {
                println!("  {:18} | {}", row.exchange, iss);
            }
        }

        // Trading
        if !trading_rows.is_empty() {
            let trading_issues: Vec<&TradingRow> = trading_rows.iter().filter(|r| !r.issues.is_empty()).collect();
            if !trading_issues.is_empty() {
                println!();
                println!("=== TRADING ISSUES ===");
                for row in &trading_issues {
                    for iss in &row.issues {
                        println!("  {:18} | {}", row.exchange, iss);
                    }
                }
            }
        }
    }

    pub fn write_json(path: &str, reports: &[(ExchangeId, ExchangeReport)]) {
        let json = serde_json::json!({
            "timestamp": now_ms(),
            "exchanges": reports.iter().map(|(id, r)| {
                serde_json::json!({
                    "exchange": format!("{:?}", id),
                    "report": r,
                })
            }).collect::<Vec<_>>(),
        });
        match std::fs::write(path, serde_json::to_string_pretty(&json).unwrap_or_default()) {
            Ok(()) => println!("JSON report written to {}", path),
            Err(e) => println!("Failed to write JSON: {}", e),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// All testable exchanges
// ─────────────────────────────────────────────────────────────────────────────

fn all_testable_exchanges() -> Vec<ExchangeId> {
    vec![
        ExchangeId::Binance, ExchangeId::Bybit, ExchangeId::OKX, ExchangeId::KuCoin,
        ExchangeId::Kraken, ExchangeId::GateIO, ExchangeId::Bitfinex, ExchangeId::MEXC,
        ExchangeId::HTX, ExchangeId::BingX, ExchangeId::CryptoCom, ExchangeId::Upbit,
        ExchangeId::Deribit, ExchangeId::HyperLiquid, ExchangeId::Bitget,
        ExchangeId::Bitstamp, ExchangeId::Coinbase, ExchangeId::Gemini,
        ExchangeId::Dydx, ExchangeId::Lighter,
        ExchangeId::YahooFinance, ExchangeId::CryptoCompare, ExchangeId::Twelvedata,
        ExchangeId::Polymarket, ExchangeId::Dukascopy, ExchangeId::Alpaca,
        ExchangeId::Krx,
        ExchangeId::Polygon, ExchangeId::Finnhub, ExchangeId::Tiingo,
        ExchangeId::AlphaVantage, ExchangeId::AngelOne, ExchangeId::Zerodha,
        ExchangeId::Upstox, ExchangeId::Dhan, ExchangeId::Fyers,
        ExchangeId::Oanda, ExchangeId::JQuants, ExchangeId::Tinkoff,
        ExchangeId::Ib, ExchangeId::Futu, ExchangeId::Coinglass,
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
// main
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::Args::parse();
    let start = Instant::now();

    let mut exchanges = all_testable_exchanges();

    // Apply exchange filter
    if let Some(ref filter) = args.exchange_filter {
        let filter_lc = filter.to_lowercase();
        exchanges.retain(|id| format!("{:?}", id).to_lowercase().contains(&filter_lc));
        if exchanges.is_empty() {
            eprintln!("No exchange matched filter '{}'", filter);
            return Ok(());
        }
    }

    println!("=== e2e_smoke — digdigdig3 ===");
    println!("Exchanges: {} | market={} trading={}", exchanges.len(), args.run_market, args.run_trading);
    if let Some(ref f) = args.exchange_filter { println!("Filter: {}", f); }
    println!();

    let mut all_reports: Vec<(ExchangeId, ExchangeReport)> = Vec::new();
    let mut market_rows: Vec<MarketRow> = Vec::new();
    let mut trading_rows: Vec<TradingRow> = Vec::new();

    // ── Market parallel ───────────────────────────────────────────────────────
    if args.run_market {
        // Add MOEX direct
        let include_moex = args.exchange_filter.as_deref()
            .map(|f| "moex".contains(&f.to_lowercase()))
            .unwrap_or(true);

        let mut market_handles: Vec<tokio::task::JoinHandle<MarketRow>> = exchanges
            .iter()
            .copied()
            .map(|id| {
                tokio::spawn(async move {
                    // Per-exchange wall-time cap. Was 60s but Bybit liquidation
                    // needs 60s alone (5 parallel symbols × 60s window each); the
                    // 90s cap leaves comfortable headroom for slower exchanges to
                    // finish all WS budgets in parallel.
                    timeout(Duration::from_secs(90), test_market(id))
                        .await
                        .unwrap_or_else(|_| MarketRow {
                            exchange: format!("{:?}", id),
                            ping: MethodResult::Err("HARD_TIMEOUT_90s".into()),
                            price: MethodResult::Skipped, ticker: MethodResult::Skipped,
                            orderbook: MethodResult::Skipped, klines: MethodResult::Skipped,
                            trades: MethodResult::Skipped, exch_info: MethodResult::Skipped,
                            funding: MethodResult::Skipped, open_interest: MethodResult::Skipped,
                            mark_price: MethodResult::Skipped, long_short: MethodResult::Skipped,
                            liquidations: MethodResult::Skipped, premium_index: MethodResult::Skipped,
                            ws_ticker: MethodResult::Skipped, ws_trade: MethodResult::Skipped,
                            ws_orderbook: MethodResult::Skipped, ws_kline: MethodResult::Skipped,
                            ws_mark_price: MethodResult::Skipped, ws_funding: MethodResult::Skipped,
                            ws_liquidation: MethodResult::Skipped, ws_oi: MethodResult::Skipped,
                            ws_agg_trade: MethodResult::Skipped,
                            issues: vec!["HARD_TIMEOUT_90s".into()],
                        })
                })
            })
            .collect();

        if include_moex {
            market_handles.push(tokio::spawn(async move {
                timeout(Duration::from_secs(35), test_moex_market())
                    .await
                    .unwrap_or_else(|_| MarketRow {
                        exchange: "Moex".into(),
                        ping: MethodResult::Err("HARD_TIMEOUT_35s".into()),
                        price: MethodResult::Skipped, ticker: MethodResult::Skipped,
                        orderbook: MethodResult::Skipped, klines: MethodResult::Skipped,
                        trades: MethodResult::Skipped, exch_info: MethodResult::Skipped,
                        funding: MethodResult::Skipped, open_interest: MethodResult::Skipped,
                        mark_price: MethodResult::Skipped, long_short: MethodResult::Skipped,
                        liquidations: MethodResult::Skipped, premium_index: MethodResult::Skipped,
                        ws_ticker: MethodResult::Skipped, ws_trade: MethodResult::Skipped,
                        ws_orderbook: MethodResult::Skipped, ws_kline: MethodResult::Skipped,
                        ws_mark_price: MethodResult::Skipped, ws_funding: MethodResult::Skipped,
                        ws_liquidation: MethodResult::Skipped, ws_oi: MethodResult::Skipped,
                        ws_agg_trade: MethodResult::Skipped,
                        issues: vec!["HARD_TIMEOUT_35s".into()],
                    })
            }));
        }

        let results = futures_util::future::join_all(market_handles).await;
        market_rows = results.into_iter().filter_map(|r| r.ok()).collect();
        market_rows.sort_by_key(|r| r.exchange.clone());

        // Print detailed per-method issues for rows with problems
        println!("=== PER-EXCHANGE DETAILS (issues only) ===");
        for row in &market_rows {
            if !row.issues.is_empty() {
                println!("{:18} | ISSUES: {}", row.exchange, row.issues.join(" | "));
            }
        }

        report::print_market_matrix(&market_rows);

        // Populate all_reports
        for row in &market_rows {
            let id = exchanges.iter().find(|&&id| format!("{:?}", id) == row.exchange)
                .copied()
                .unwrap_or(ExchangeId::Moex);
            all_reports.push((id, ExchangeReport { market: Some(row.clone()), trading: None }));
        }
    }

    // ── Trading parallel ──────────────────────────────────────────────────────
    if args.run_trading {
        let trading_handles: Vec<tokio::task::JoinHandle<TradingRow>> = exchanges
            .iter()
            .copied()
            .map(|id| {
                tokio::spawn(async move {
                    timeout(Duration::from_secs(30), test_trading(id))
                        .await
                        .unwrap_or_else(|_| TradingRow {
                            exchange: format!("{:?}", id),
                            balance: MethodResult::Err("HARD_TIMEOUT_30s".into()),
                            account_info: MethodResult::Skipped, open_orders: MethodResult::Skipped,
                            user_trades: MethodResult::Skipped, positions: MethodResult::Skipped,
                            fees: MethodResult::Skipped,
                            issues: vec!["HARD_TIMEOUT_30s".into()],
                        })
                })
            })
            .collect();

        let results = futures_util::future::join_all(trading_handles).await;
        trading_rows = results.into_iter().filter_map(|r| r.ok()).collect();
        trading_rows.sort_by_key(|r| r.exchange.clone());

        report::print_trading_matrix(&trading_rows);

        // Merge into all_reports
        for tr in &trading_rows {
            let id = exchanges.iter().find(|&&id| format!("{:?}", id) == tr.exchange).copied();
            if let Some(id) = id {
                if let Some(entry) = all_reports.iter_mut().find(|(eid, _)| *eid == id) {
                    entry.1.trading = Some(tr.clone());
                } else {
                    all_reports.push((id, ExchangeReport { market: None, trading: Some(tr.clone()) }));
                }
            }
        }
    }

    // ── Summaries ─────────────────────────────────────────────────────────────
    report::print_summaries(&market_rows, &trading_rows);

    // ── JSON output ───────────────────────────────────────────────────────────
    if let Some(ref path) = args.json_out {
        report::write_json(path, &all_reports);
    }

    println!();
    println!("Total runtime: {:.1}s", start.elapsed().as_secs_f64());

    Ok(())
}
