//! Stub `MarketDataPublic` impls for every connector type.
//!
//! Each empty body inherits the trait's default `UnsupportedOperation` methods.
//! Per-exchange overrides arrive in subsequent phases and replace these stubs
//! in their respective connector files.

use super::MarketDataPublic;

// CEX — Binance, Bybit, OKX, Lighter, Bitstamp, Coinbase, Deribit, Gemini, Upbit have real impls in their connector.rs files
use crate::l3::open::crypto::cex::kucoin::KuCoinConnector;
// GateioConnector — real impl in connector.rs
use crate::l3::open::crypto::cex::bitfinex::BitfinexConnector;
use crate::l3::open::crypto::cex::mexc::MexcConnector;
// HtxConnector — real impl in connector.rs
// BitgetConnector — real impl in connector.rs
use crate::l3::open::crypto::cex::bingx::BingxConnector;
use crate::l3::open::crypto::cex::crypto_com::CryptoComConnector;
// DEX — Lighter, dYdX, HyperLiquid have real impls in their connector.rs files

// Stocks US
use crate::l2::paid::polygon::PolygonConnector;
use crate::l1::free::finnhub::FinnhubConnector;
use crate::l1::paid::tiingo::TiingoConnector;
use crate::l1::paid::twelvedata::TwelvedataConnector;
use crate::l3::gated::stocks::us::alpaca::AlpacaConnector;

// Stocks India
use crate::l3::gated::stocks::india::angel_one::AngelOneConnector;
use crate::l3::gated::stocks::india::zerodha::ZerodhaConnector;
use crate::l3::gated::stocks::india::upstox::UpstoxConnector;
use crate::l3::gated::stocks::india::dhan::DhanConnector;
use crate::l3::gated::stocks::india::fyers::FyersConnector;

// Stocks Other
use crate::l1::paid::jquants::JQuantsConnector;
use crate::l1::free::krx::KrxConnector;
use crate::l2::free::moex::MoexConnector;
use crate::l3::gated::stocks::russia::tinkoff::TinkoffConnector;

// Forex
use crate::l3::gated::forex::oanda::OandaConnector;
use crate::l3::gated::forex::dukascopy::DukascopyConnector;
use crate::l1::paid::alphavantage::AlphaVantageConnector;

// Prediction — PolymarketConnector has real impl in connector.rs

// Brokers
#[cfg(not(target_arch = "wasm32"))]
use crate::l3::gated::multi::ib::IBConnector;

// Data Feeds
use crate::l1::free::yahoo::YahooFinanceConnector;
use crate::l2::paid::cryptocompare::CryptoCompareConnector;

// ═══════════════════════════════════════════════════════════════════════════════
// CEX stubs
// ═══════════════════════════════════════════════════════════════════════════════

impl MarketDataPublic for KuCoinConnector {}
// KrakenConnector — real impl in connector.rs
// CoinbaseConnector — real impl in connector.rs
// GateioConnector — real impl in connector.rs
impl MarketDataPublic for BitfinexConnector {}
// BitstampConnector — real impl in connector.rs
// GeminiConnector — real impl in connector.rs
impl MarketDataPublic for MexcConnector {}
// HtxConnector — real impl in connector.rs
// BitgetConnector — real impl in connector.rs
impl MarketDataPublic for BingxConnector {}
impl MarketDataPublic for CryptoComConnector {}
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

impl MarketDataPublic for PolygonConnector {}
impl MarketDataPublic for FinnhubConnector {}
impl MarketDataPublic for TiingoConnector {}
impl MarketDataPublic for TwelvedataConnector {}
impl MarketDataPublic for AlpacaConnector {}

// ═══════════════════════════════════════════════════════════════════════════════
// Stocks India stubs
// ═══════════════════════════════════════════════════════════════════════════════

impl MarketDataPublic for AngelOneConnector {}
impl MarketDataPublic for ZerodhaConnector {}
impl MarketDataPublic for UpstoxConnector {}
impl MarketDataPublic for DhanConnector {}
impl MarketDataPublic for FyersConnector {}

// ═══════════════════════════════════════════════════════════════════════════════
// Stocks Other stubs
// ═══════════════════════════════════════════════════════════════════════════════

impl MarketDataPublic for JQuantsConnector {}
impl MarketDataPublic for KrxConnector {}
impl MarketDataPublic for MoexConnector {}
impl MarketDataPublic for TinkoffConnector {}

// ═══════════════════════════════════════════════════════════════════════════════
// Forex stubs
// ═══════════════════════════════════════════════════════════════════════════════

impl MarketDataPublic for OandaConnector {}
impl MarketDataPublic for DukascopyConnector {}
impl MarketDataPublic for AlphaVantageConnector {}

// ═══════════════════════════════════════════════════════════════════════════════
// Prediction stubs
// ═══════════════════════════════════════════════════════════════════════════════

// PolymarketConnector — real impl in connector.rs

// ═══════════════════════════════════════════════════════════════════════════════
// Brokers stubs
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(not(target_arch = "wasm32"))]
impl MarketDataPublic for IBConnector {}

// ═══════════════════════════════════════════════════════════════════════════════
// Data Feeds stubs
// ═══════════════════════════════════════════════════════════════════════════════

impl MarketDataPublic for YahooFinanceConnector {}

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl MarketDataPublic for CryptoCompareConnector {
    async fn get_recent_trades(
        &self,
        _symbol: crate::core::types::SymbolInput<'_>,
        _limit: Option<u32>,
        _account_type: crate::core::types::AccountType,
    ) -> crate::core::types::ExchangeResult<Vec<crate::core::types::PublicTrade>> {
        Err(crate::core::types::ExchangeError::NotSupported(
            "CryptoCompare WS-only for raw trades; no public REST endpoint — use REST OHLCV /data/v2/histominute for aggregated data".into(),
        ))
    }
}
