//! Stub `MarketDataPublic` impls for every connector type.
//!
//! Each empty body inherits the trait's default `UnsupportedOperation` methods.
//! Per-exchange overrides arrive in subsequent phases and replace these stubs
//! in their respective connector files.

use super::MarketDataPublic;

// CEX — Binance, Bybit, OKX, Lighter have real impls in their connector.rs files
use crate::l3::open::crypto::cex::kucoin::KuCoinConnector;
use crate::l3::open::crypto::cex::kraken::KrakenConnector;
use crate::l3::open::crypto::cex::coinbase::CoinbaseConnector;
use crate::l3::open::crypto::cex::gateio::GateioConnector;
use crate::l3::open::crypto::cex::bitfinex::BitfinexConnector;
use crate::l3::open::crypto::cex::bitstamp::BitstampConnector;
use crate::l3::open::crypto::cex::gemini::GeminiConnector;
use crate::l3::open::crypto::cex::mexc::MexcConnector;
use crate::l3::open::crypto::cex::htx::HtxConnector;
use crate::l3::open::crypto::cex::bitget::BitgetConnector;
use crate::l3::open::crypto::cex::bingx::BingxConnector;
use crate::l3::open::crypto::cex::crypto_com::CryptoComConnector;
use crate::l3::open::crypto::cex::upbit::UpbitConnector;
use crate::l3::open::crypto::cex::deribit::DeribitConnector;
use crate::l3::open::crypto::cex::hyperliquid::HyperliquidConnector;

// DEX — Lighter has real impl in its connector.rs
use crate::l3::open::crypto::dex::dydx::DydxConnector;

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

// Prediction
use crate::l3::open::prediction::polymarket::PolymarketConnector;

// Brokers
use crate::l3::gated::multi::ib::IBConnector;

// Data Feeds
use crate::l1::free::yahoo::YahooFinanceConnector;
use crate::l2::paid::cryptocompare::CryptoCompareConnector;

// ═══════════════════════════════════════════════════════════════════════════════
// CEX stubs
// ═══════════════════════════════════════════════════════════════════════════════

impl MarketDataPublic for KuCoinConnector {}
impl MarketDataPublic for KrakenConnector {}
impl MarketDataPublic for CoinbaseConnector {}
impl MarketDataPublic for GateioConnector {}
impl MarketDataPublic for BitfinexConnector {}
impl MarketDataPublic for BitstampConnector {}
impl MarketDataPublic for GeminiConnector {}
impl MarketDataPublic for MexcConnector {}
impl MarketDataPublic for HtxConnector {}
impl MarketDataPublic for BitgetConnector {}
impl MarketDataPublic for BingxConnector {}
impl MarketDataPublic for CryptoComConnector {}
impl MarketDataPublic for UpbitConnector {}
impl MarketDataPublic for DeribitConnector {}
impl MarketDataPublic for HyperliquidConnector {}

// ═══════════════════════════════════════════════════════════════════════════════
// DEX stubs
// ═══════════════════════════════════════════════════════════════════════════════

impl MarketDataPublic for DydxConnector {}

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

impl MarketDataPublic for PolymarketConnector {}

// ═══════════════════════════════════════════════════════════════════════════════
// Brokers stubs
// ═══════════════════════════════════════════════════════════════════════════════

impl MarketDataPublic for IBConnector {}

// ═══════════════════════════════════════════════════════════════════════════════
// Data Feeds stubs
// ═══════════════════════════════════════════════════════════════════════════════

impl MarketDataPublic for YahooFinanceConnector {}
impl MarketDataPublic for CryptoCompareConnector {}
