//! Stub `WebSocketConnector` impls for every connector type that doesn't
//! natively support WebSocket. Each connector inherits the trait's default
//! `UnsupportedOperation` methods.
//!
//! Connectors with real WS support (Binance, Bybit, OKX, KuCoin, Kraken, etc.)
//! implement WebSocketConnector on a dedicated `*WebSocket` struct, not on the
//! primary `*Connector` struct. These stubs ensure every `*Connector` satisfies
//! the `CoreConnector` bound.

use super::WebSocketConnector;

// CEX
use crate::l3::open::crypto::cex::binance::BinanceConnector;
use crate::l3::open::crypto::cex::bybit::BybitConnector;
use crate::l3::open::crypto::cex::okx::OkxConnector;
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

// DEX
use crate::l3::open::crypto::dex::dydx::DydxConnector;
use crate::l3::open::crypto::dex::lighter::LighterConnector;

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

impl WebSocketConnector for BinanceConnector {}
impl WebSocketConnector for BybitConnector {}
impl WebSocketConnector for OkxConnector {}
impl WebSocketConnector for KuCoinConnector {}
impl WebSocketConnector for KrakenConnector {}
impl WebSocketConnector for CoinbaseConnector {}
impl WebSocketConnector for GateioConnector {}
impl WebSocketConnector for BitfinexConnector {}
impl WebSocketConnector for BitstampConnector {}
impl WebSocketConnector for GeminiConnector {}
impl WebSocketConnector for MexcConnector {}
impl WebSocketConnector for HtxConnector {}
impl WebSocketConnector for BitgetConnector {}
impl WebSocketConnector for BingxConnector {}
impl WebSocketConnector for CryptoComConnector {}
impl WebSocketConnector for UpbitConnector {}
impl WebSocketConnector for DeribitConnector {}
impl WebSocketConnector for HyperliquidConnector {}

// ═══════════════════════════════════════════════════════════════════════════════
// DEX stubs
// ═══════════════════════════════════════════════════════════════════════════════

impl WebSocketConnector for DydxConnector {}
impl WebSocketConnector for LighterConnector {}

// ═══════════════════════════════════════════════════════════════════════════════
// Stocks US stubs
// ═══════════════════════════════════════════════════════════════════════════════

impl WebSocketConnector for PolygonConnector {}
impl WebSocketConnector for FinnhubConnector {}
impl WebSocketConnector for TiingoConnector {}
impl WebSocketConnector for TwelvedataConnector {}
impl WebSocketConnector for AlpacaConnector {}

// ═══════════════════════════════════════════════════════════════════════════════
// Stocks India stubs
// ═══════════════════════════════════════════════════════════════════════════════

impl WebSocketConnector for AngelOneConnector {}
impl WebSocketConnector for ZerodhaConnector {}
impl WebSocketConnector for UpstoxConnector {}
impl WebSocketConnector for DhanConnector {}
impl WebSocketConnector for FyersConnector {}

// ═══════════════════════════════════════════════════════════════════════════════
// Stocks Other stubs
// ═══════════════════════════════════════════════════════════════════════════════

impl WebSocketConnector for JQuantsConnector {}
impl WebSocketConnector for KrxConnector {}
impl WebSocketConnector for MoexConnector {}
impl WebSocketConnector for TinkoffConnector {}

// ═══════════════════════════════════════════════════════════════════════════════
// Forex stubs
// ═══════════════════════════════════════════════════════════════════════════════

impl WebSocketConnector for OandaConnector {}
impl WebSocketConnector for DukascopyConnector {}
impl WebSocketConnector for AlphaVantageConnector {}

// ═══════════════════════════════════════════════════════════════════════════════
// Prediction stubs
// ═══════════════════════════════════════════════════════════════════════════════

impl WebSocketConnector for PolymarketConnector {}

// ═══════════════════════════════════════════════════════════════════════════════
// Brokers stubs
// ═══════════════════════════════════════════════════════════════════════════════

impl WebSocketConnector for IBConnector {}

// ═══════════════════════════════════════════════════════════════════════════════
// Data Feeds stubs
// ═══════════════════════════════════════════════════════════════════════════════

impl WebSocketConnector for YahooFinanceConnector {}
impl WebSocketConnector for CryptoCompareConnector {}
