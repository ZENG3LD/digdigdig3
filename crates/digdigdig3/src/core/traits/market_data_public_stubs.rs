//! Stub `MarketDataPublic` impls for every connector type.
//!
//! Each empty body inherits the trait's default `NotImplemented` methods.
//! Per-exchange overrides arrive in subsequent phases and replace these stubs
//! in their respective connector files.

// Every impl in this file is a stub for a non-CEX/DEX exchange (CEX/DEX real
// impls live in their own connector.rs files) — with a narrow crypto-only
// build every impl below compiles out, which would make this otherwise unused.
#[cfg(any(
    feature = "exchange-polygon", feature = "exchange-finnhub", feature = "exchange-tiingo",
    feature = "exchange-twelvedata", feature = "exchange-alpaca", feature = "exchange-angel-one",
    feature = "exchange-zerodha", feature = "exchange-upstox", feature = "exchange-dhan",
    feature = "exchange-fyers", feature = "exchange-jquants", feature = "exchange-krx",
    feature = "exchange-moex", feature = "exchange-tinkoff", feature = "exchange-oanda",
    feature = "exchange-dukascopy", feature = "exchange-alphavantage", feature = "exchange-ib",
    feature = "exchange-yahoo", feature = "exchange-cryptocompare",
))]
use super::MarketDataPublic;

// CEX — Binance, Bybit, OKX, Lighter, Bitstamp, Coinbase, Deribit, Gemini, Upbit, KuCoin have real impls in their connector.rs files
// GateioConnector — real impl in connector.rs
// BitfinexConnector — real impl in connector.rs (funding_rate_history, open_interest_history, long_short_ratio_history)
// MexcConnector — real impl in connector.rs
// HtxConnector — real impl in connector.rs
// BitgetConnector — real impl in connector.rs
// BingxConnector — real impl in connector.rs
// CryptoComConnector — real impl in connector.rs
// DEX — Lighter, dYdX, HyperLiquid have real impls in their connector.rs files

// Stocks US
#[cfg(feature = "exchange-polygon")]
use crate::l2::paid::polygon::PolygonConnector;
#[cfg(feature = "exchange-finnhub")]
use crate::l1::free::finnhub::FinnhubConnector;
#[cfg(feature = "exchange-tiingo")]
use crate::l1::paid::tiingo::TiingoConnector;
#[cfg(feature = "exchange-twelvedata")]
use crate::l1::paid::twelvedata::TwelvedataConnector;
#[cfg(feature = "exchange-alpaca")]
use crate::l3::gated::stocks::us::alpaca::AlpacaConnector;

// Stocks India
#[cfg(feature = "exchange-angel-one")]
use crate::l3::gated::stocks::india::angel_one::AngelOneConnector;
#[cfg(feature = "exchange-zerodha")]
use crate::l3::gated::stocks::india::zerodha::ZerodhaConnector;
#[cfg(feature = "exchange-upstox")]
use crate::l3::gated::stocks::india::upstox::UpstoxConnector;
#[cfg(feature = "exchange-dhan")]
use crate::l3::gated::stocks::india::dhan::DhanConnector;
#[cfg(feature = "exchange-fyers")]
use crate::l3::gated::stocks::india::fyers::FyersConnector;

// Stocks Other
#[cfg(feature = "exchange-jquants")]
use crate::l1::paid::jquants::JQuantsConnector;
#[cfg(feature = "exchange-krx")]
use crate::l1::free::krx::KrxConnector;
#[cfg(feature = "exchange-moex")]
use crate::l2::free::moex::MoexConnector;
#[cfg(feature = "exchange-tinkoff")]
use crate::l3::gated::stocks::russia::tinkoff::TinkoffConnector;

// Forex
#[cfg(feature = "exchange-oanda")]
use crate::l3::gated::forex::oanda::OandaConnector;
#[cfg(feature = "exchange-dukascopy")]
use crate::l3::gated::forex::dukascopy::DukascopyConnector;
#[cfg(feature = "exchange-alphavantage")]
use crate::l1::paid::alphavantage::AlphaVantageConnector;

// Prediction — PolymarketConnector has real impl in connector.rs

// Brokers
#[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "exchange-ib")]
use crate::l3::gated::multi::ib::IBConnector;

// Data Feeds
#[cfg(feature = "exchange-yahoo")]
use crate::l1::free::yahoo::YahooFinanceConnector;
#[cfg(feature = "exchange-cryptocompare")]
use crate::l2::paid::cryptocompare::CryptoCompareConnector;

// ═══════════════════════════════════════════════════════════════════════════════
// CEX stubs
// ═══════════════════════════════════════════════════════════════════════════════

// KuCoinConnector — real impl in connector.rs (get_recent_trades, get_funding_rate_history)
// KrakenConnector — real impl in connector.rs
// CoinbaseConnector — real impl in connector.rs
// GateioConnector — real impl in connector.rs
// BitfinexConnector — real impl in connector.rs
// BitstampConnector — real impl in connector.rs
// GeminiConnector — real impl in connector.rs
// MexcConnector — real impl in connector.rs
// HtxConnector — real impl in connector.rs
// BitgetConnector — real impl in connector.rs
// BingxConnector — real impl in connector.rs
// CryptoComConnector — real impl in connector.rs (mark_price_klines, index_price_klines, funding_rate_history)
// UpbitConnector — real impl in connector.rs
// DeribitConnector — real impl in connector.rs
// HyperliquidConnector — real impl in connector.rs (get_recent_trades overridden)

// ═══════════════════════════════════════════════════════════════════════════════
// DEX stubs
// ═══════════════════════════════════════════════════════════════════════════════

// DydxConnector — real impl in connector.rs

// ═══════════════════════════════════════════════════════════════════════════════
// Stocks US stubs
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(feature = "exchange-polygon")]
impl MarketDataPublic for PolygonConnector {}
#[cfg(feature = "exchange-finnhub")]
impl MarketDataPublic for FinnhubConnector {}
#[cfg(feature = "exchange-tiingo")]
impl MarketDataPublic for TiingoConnector {}
#[cfg(feature = "exchange-twelvedata")]
impl MarketDataPublic for TwelvedataConnector {}
#[cfg(feature = "exchange-alpaca")]
impl MarketDataPublic for AlpacaConnector {}

// ═══════════════════════════════════════════════════════════════════════════════
// Stocks India stubs
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(feature = "exchange-angel-one")]
impl MarketDataPublic for AngelOneConnector {}
#[cfg(feature = "exchange-zerodha")]
impl MarketDataPublic for ZerodhaConnector {}
#[cfg(feature = "exchange-upstox")]
impl MarketDataPublic for UpstoxConnector {}
#[cfg(feature = "exchange-dhan")]
impl MarketDataPublic for DhanConnector {}
#[cfg(feature = "exchange-fyers")]
impl MarketDataPublic for FyersConnector {}

// ═══════════════════════════════════════════════════════════════════════════════
// Stocks Other stubs
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(feature = "exchange-jquants")]
impl MarketDataPublic for JQuantsConnector {}
#[cfg(feature = "exchange-krx")]
impl MarketDataPublic for KrxConnector {}
#[cfg(feature = "exchange-moex")]
impl MarketDataPublic for MoexConnector {}
#[cfg(feature = "exchange-tinkoff")]
impl MarketDataPublic for TinkoffConnector {}

// ═══════════════════════════════════════════════════════════════════════════════
// Forex stubs
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(feature = "exchange-oanda")]
impl MarketDataPublic for OandaConnector {}
#[cfg(feature = "exchange-dukascopy")]
impl MarketDataPublic for DukascopyConnector {}
#[cfg(feature = "exchange-alphavantage")]
impl MarketDataPublic for AlphaVantageConnector {}

// ═══════════════════════════════════════════════════════════════════════════════
// Prediction stubs
// ═══════════════════════════════════════════════════════════════════════════════

// PolymarketConnector — real impl in connector.rs

// ═══════════════════════════════════════════════════════════════════════════════
// Brokers stubs
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "exchange-ib")]
impl MarketDataPublic for IBConnector {}

// ═══════════════════════════════════════════════════════════════════════════════
// Data Feeds stubs
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(feature = "exchange-yahoo")]
impl MarketDataPublic for YahooFinanceConnector {}

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg(feature = "exchange-cryptocompare")]
impl MarketDataPublic for CryptoCompareConnector {
    async fn get_recent_trades(
        &self,
        _symbol: crate::core::types::SymbolInput<'_>,
        _limit: Option<u32>,
        _account_type: crate::core::types::AccountType,
    ) -> crate::core::types::ExchangeResult<Vec<crate::core::types::PublicTrade>> {
        Err(crate::core::types::ExchangeError::WireAbsent(
            "CryptoCompare WS-only for raw trades; no public REST endpoint — use REST OHLCV /data/v2/histominute for aggregated data".into(),
        ))
    }
}
