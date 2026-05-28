//! # Connector Factory
//!
//! Factory for creating connectors by ExchangeId.
//!
//! ## Overview
//!
//! This module provides a unified factory interface for creating any of the
//! supported connectors. It handles all constructor variations across different
//! exchanges and categories.
//!
//! ## Usage
//!
//! ```ignore
//! use connectors_v5::connector_manager::ConnectorFactory;
//! use connectors_v5::core::types::ExchangeId;
//!
//! // Create public connector (no auth)
//! let connector = ConnectorFactory::create_public(ExchangeId::Binance, false).await?;
//!
//! // Create authenticated connector
//! let credentials = Credentials::new("api_key", "api_secret");
//! let connector = ConnectorFactory::create_authenticated(
//!     ExchangeId::Binance,
//!     credentials
//! ).await?;
//! ```
//!
//! ## Constructor Patterns
//!
//! Different connectors use different constructor patterns:
//!
//! ### Pattern A: `::public(testnet: bool)` (async)
//! - CEX: Binance, Bybit, OKX, BingX, Bitfinex, Deribit, Dydx, etc.
//! - These support testnet mode for public access
//!
//! ### Pattern B: `::public()` (async)
//! - CEX: Bitget, Bitstamp, Coinbase, Gemini, etc.
//! - Simpler constructors without testnet parameter
//!
//! ### Pattern C: `::new()` (sync)
//! - Data feeds: AlphaVantage (with auth), etc.
//! - Lightweight connectors that don't need async setup
//!
//! ### Pattern F: `::from_env()` (sync)
//! - Data feeds: Alpaca
//! - Load credentials from environment variables

use std::sync::Arc;
use crate::core::types::{AccountType, ExchangeId, ExchangeResult, ExchangeError};
use crate::core::traits::{Credentials, CoreConnector, WebSocketConnector};

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR IMPORTS - CEX
// ═══════════════════════════════════════════════════════════════════════════════

use crate::l3::open::crypto::cex::bitmex::{BitmexConnector, BitmexWebSocket};
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
#[cfg(feature = "onchain-evm")]
use crate::l3::open::crypto::cex::hyperliquid::HyperliquidConnector;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR IMPORTS - DEX
// ═══════════════════════════════════════════════════════════════════════════════

use crate::l3::open::crypto::dex::lighter::LighterConnector;
use crate::l3::open::crypto::dex::dydx::DydxConnector;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR IMPORTS - STOCKS US
// ═══════════════════════════════════════════════════════════════════════════════

use crate::l2::paid::polygon::PolygonConnector;
use crate::l1::free::finnhub::FinnhubConnector;
use crate::l1::paid::tiingo::TiingoConnector;
use crate::l1::paid::twelvedata::TwelvedataConnector;
use crate::l3::gated::stocks::us::alpaca::AlpacaConnector;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR IMPORTS - STOCKS INDIA
// ═══════════════════════════════════════════════════════════════════════════════

use crate::l3::gated::stocks::india::dhan::DhanConnector;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR IMPORTS - STOCKS OTHER
// ═══════════════════════════════════════════════════════════════════════════════

use crate::l1::free::krx::KrxConnector;
use crate::l2::free::moex::MoexConnector;
#[cfg(not(target_arch = "wasm32"))]
use crate::l2::free::moex::MoexWebSocket;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR IMPORTS - FOREX
// ═══════════════════════════════════════════════════════════════════════════════

use crate::l3::gated::forex::dukascopy::DukascopyConnector;
use crate::l1::paid::alphavantage::{AlphaVantageConnector, AlphaVantageAuth};

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR IMPORTS - PREDICTION
// ═══════════════════════════════════════════════════════════════════════════════

use crate::l3::open::prediction::polymarket::PolymarketConnector;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR IMPORTS - DATA FEEDS
// ═══════════════════════════════════════════════════════════════════════════════

use crate::l1::free::yahoo::YahooFinanceConnector;
use crate::l2::paid::cryptocompare::CryptoCompareConnector;

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET IMPORTS - CEX (native-only)
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(not(target_arch = "wasm32"))]
use crate::l3::open::crypto::cex::binance::BinanceWebSocket;
#[cfg(not(target_arch = "wasm32"))]
use crate::l3::open::crypto::cex::bybit::BybitWebSocket;
#[cfg(not(target_arch = "wasm32"))]
use crate::l3::open::crypto::cex::okx::OkxWebSocket;
#[cfg(not(target_arch = "wasm32"))]
use crate::l3::open::crypto::cex::kucoin::KuCoinWebSocket;
#[cfg(not(target_arch = "wasm32"))]
use crate::l3::open::crypto::cex::kraken::KrakenWebSocket;
#[cfg(not(target_arch = "wasm32"))]
use crate::l3::open::crypto::cex::gateio::GateioWebSocket;
use crate::l3::open::crypto::cex::bitfinex::BitfinexWebSocket;
#[cfg(not(target_arch = "wasm32"))]
use crate::l3::open::crypto::cex::bitstamp::BitstampWebSocket;
use crate::l3::open::crypto::cex::gemini::GeminiWebSocket;
#[cfg(not(target_arch = "wasm32"))]
use crate::l3::open::crypto::cex::mexc::MexcWebSocket;
#[cfg(not(target_arch = "wasm32"))]
use crate::l3::open::crypto::cex::htx::HtxWebSocket;
#[cfg(not(target_arch = "wasm32"))]
use crate::l3::open::crypto::cex::bitget::BitgetWebSocket;
use crate::l3::open::crypto::cex::bingx::BingxWebSocket;
use crate::l3::open::crypto::cex::crypto_com::CryptoComWebSocket;
#[cfg(not(target_arch = "wasm32"))]
use crate::l3::open::crypto::cex::upbit::UpbitWebSocket;
#[cfg(not(target_arch = "wasm32"))]
use crate::l3::open::crypto::cex::deribit::DeribitWebSocket;
// HyperLiquid WS: public subscribe (`l2Book`/`trades`/`candle`) sends NO auth,
// `EvmWallet` is only built inside `HyperliquidConnector::new(Some(creds), _)`.
// Public path is wasm-eligible; k256+sha3 from `onchain-evm` compile to wasm.
#[cfg(feature = "onchain-evm")]
use crate::l3::open::crypto::cex::hyperliquid::HyperliquidWebSocket;
#[cfg(not(target_arch = "wasm32"))]
use crate::l3::open::crypto::cex::coinbase::CoinbaseWebSocket;

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET IMPORTS - DEX (native-only)
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(not(target_arch = "wasm32"))]
use crate::l3::open::crypto::dex::dydx::DydxWebSocket;
#[cfg(not(target_arch = "wasm32"))]
use crate::l3::open::crypto::dex::lighter::LighterWebSocket;

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET IMPORTS - DATA FEEDS (native-only)
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(not(target_arch = "wasm32"))]
use crate::l1::free::yahoo::YahooFinanceWebSocket;
#[cfg(not(target_arch = "wasm32"))]
use crate::l2::paid::cryptocompare::CryptoCompareWebSocket;

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET IMPORTS — wasm32 (subset: Binance/Bybit/OKX via UniversalWsTransport)
// ═══════════════════════════════════════════════════════════════════════════════
#[cfg(target_arch = "wasm32")]
use crate::l3::open::crypto::cex::binance::BinanceWebSocket;
#[cfg(target_arch = "wasm32")]
use crate::l3::open::crypto::cex::bybit::BybitWebSocket;
#[cfg(target_arch = "wasm32")]
use crate::l3::open::crypto::cex::okx::OkxWebSocket;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR FACTORY
// ═══════════════════════════════════════════════════════════════════════════════

/// Factory for creating connectors by ExchangeId
///
/// Provides two main methods:
/// - `create_public()` - Create public connector (no authentication)
/// - `create_authenticated()` - Create authenticated connector with credentials
///
/// # Examples
///
/// ```ignore
/// // Create public connector
/// let connector = ConnectorFactory::create_public(ExchangeId::Binance, false).await?;
///
/// // Create authenticated connector
/// let credentials = Credentials::new("key", "secret");
/// let connector = ConnectorFactory::create_authenticated(
///     ExchangeId::Binance,
///     credentials
/// ).await?;
/// ```
pub(crate) struct ConnectorFactory;

impl ConnectorFactory {
    /// Create a public (no auth) connector for any exchange
    ///
    /// For most connectors, this creates a read-only instance that can access
    /// public market data but cannot perform trading or access account info.
    ///
    /// Some connectors (like data feeds) may require API keys even for public
    /// access - in those cases, this method will return an error indicating
    /// that authentication is required.
    ///
    /// # Arguments
    ///
    /// * `id` - The exchange identifier
    ///
    /// # Returns
    ///
    /// * `Ok(Arc<dyn CoreConnector>)` - Wrapped connector instance
    /// * `Err(ExchangeError)` - If connector creation fails or requires auth
    ///
    /// # Example
    ///
    /// ```ignore
    /// let connector = ConnectorFactory::create_public(ExchangeId::Binance, false).await?;
    /// let price = connector.get_price(symbol, AccountType::Spot).await?;
    /// ```
    pub(crate) async fn create_public(id: ExchangeId, testnet: bool) -> ExchangeResult<Arc<dyn CoreConnector>> {
        match id {
            // ═══════════════════════════════════════════════════════════════════════
            // CEX - Pattern A: ::public(testnet: bool)
            // ═══════════════════════════════════════════════════════════════════════
            ExchangeId::Binance => {
                let c = BinanceConnector::public(testnet).await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            ExchangeId::Bybit => {
                let c = BybitConnector::public(testnet).await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            ExchangeId::OKX => {
                let c = OkxConnector::public(testnet).await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            ExchangeId::KuCoin => {
                let c = KuCoinConnector::public(testnet).await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            ExchangeId::Kraken => {
                let c = KrakenConnector::public(testnet).await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            ExchangeId::GateIO => {
                let c = GateioConnector::public(testnet).await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            ExchangeId::Bitfinex => {
                let c = BitfinexConnector::public(testnet).await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            ExchangeId::MEXC => {
                let c = MexcConnector::public().await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            ExchangeId::HTX => {
                let c = HtxConnector::public(testnet).await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            ExchangeId::BingX => {
                let c = BingxConnector::public(testnet).await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            ExchangeId::CryptoCom => {
                let c = CryptoComConnector::public(testnet).await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            ExchangeId::Upbit => {
                let c = UpbitConnector::public().await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            ExchangeId::Deribit => {
                let c = DeribitConnector::public(testnet).await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            #[cfg(feature = "onchain-evm")]
            ExchangeId::HyperLiquid => {
                let c = HyperliquidConnector::public(testnet).await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            #[cfg(not(feature = "onchain-evm"))]
            ExchangeId::HyperLiquid => {
                Err(ExchangeError::UnsupportedOperation(
                    "HyperLiquid requires the onchain-evm feature".into()
                ))
            }
            ExchangeId::Dydx => {
                let c = DydxConnector::public(testnet).await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            ExchangeId::Bitmex => {
                let c = BitmexConnector::new(testnet);
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }

            // ═══════════════════════════════════════════════════════════════════════
            // CEX - Pattern B: ::public() (no testnet param)
            // ═══════════════════════════════════════════════════════════════════════
            ExchangeId::Bitget => {
                let c = BitgetConnector::public().await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            ExchangeId::Bitstamp => {
                let c = BitstampConnector::public().await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            ExchangeId::Coinbase => {
                let c = CoinbaseConnector::public().await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            ExchangeId::Gemini => {
                let c = GeminiConnector::public(testnet).await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }

            // ═══════════════════════════════════════════════════════════════════════
            // DEX - Pattern B: ::public() or ::new()
            // ═══════════════════════════════════════════════════════════════════════
            ExchangeId::Lighter => {
                let c = LighterConnector::public(testnet).await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            // ═══════════════════════════════════════════════════════════════════════
            // STOCKS - Pattern C: ::crypto_only() for public, ::from_env() for auth
            // ═══════════════════════════════════════════════════════════════════════
            ExchangeId::Alpaca => {
                // Create crypto-only connector (works without API keys)
                let c = AlpacaConnector::crypto_only();
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }

            // ═══════════════════════════════════════════════════════════════════════
            // AGGREGATORS & PREDICTION - Pattern D: ::public()
            // ═══════════════════════════════════════════════════════════════════════
            ExchangeId::YahooFinance => {
                let c = YahooFinanceConnector::new();
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            ExchangeId::DefiLlama => {
                Err(ExchangeError::UnsupportedOperation(
                    "DefiLlama has moved to dig2feed crate".into()
                ))
            }
            ExchangeId::Polymarket => {
                let c = PolymarketConnector::public();
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }

            // ═══════════════════════════════════════════════════════════════════════
            // AGGREGATORS & DATA FEEDS - Require API Key
            // ═══════════════════════════════════════════════════════════════════════
            ExchangeId::Polygon => {
                Err(ExchangeError::Auth(
                    "Polygon requires API key".into()
                ))
            }
            ExchangeId::Finnhub => {
                Err(ExchangeError::Auth(
                    "Finnhub requires API key".into()
                ))
            }
            ExchangeId::Tiingo => {
                Err(ExchangeError::Auth(
                    "Tiingo requires API key".into()
                ))
            }
            ExchangeId::Twelvedata => {
                let c = TwelvedataConnector::demo();
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            ExchangeId::AlphaVantage => {
                Err(ExchangeError::Auth(
                    "AlphaVantage requires API key".into()
                ))
            }
            ExchangeId::CryptoCompare => {
                let c = CryptoCompareConnector::public();
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            // ═══════════════════════════════════════════════════════════════════════
            // BROKERS - Require Authentication
            // ═══════════════════════════════════════════════════════════════════════
            ExchangeId::AngelOne => {
                Err(ExchangeError::Auth(
                    "AngelOne requires credentials".into()
                ))
            }
            ExchangeId::Zerodha => {
                Err(ExchangeError::Auth(
                    "Zerodha requires credentials".into()
                ))
            }
            ExchangeId::Upstox => {
                Err(ExchangeError::Auth(
                    "Upstox requires credentials".into()
                ))
            }
            ExchangeId::Dhan => {
                Err(ExchangeError::Auth(
                    "Dhan requires credentials".into()
                ))
            }
            ExchangeId::Fyers => {
                Err(ExchangeError::Auth(
                    "Fyers requires credentials".into()
                ))
            }
            ExchangeId::Oanda => {
                Err(ExchangeError::Auth(
                    "Oanda requires credentials".into()
                ))
            }
            ExchangeId::Dukascopy => {
                let c = DukascopyConnector::new();
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            ExchangeId::JQuants => {
                Err(ExchangeError::Auth(
                    "JQuants requires credentials".into()
                ))
            }
            ExchangeId::Krx => {
                #[allow(deprecated)]
                let c = KrxConnector::new_public();
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            ExchangeId::Moex => {
                let c = MoexConnector::new_public();
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            ExchangeId::Tinkoff => {
                Err(ExchangeError::Auth(
                    "Tinkoff requires credentials".into()
                ))
            }
            ExchangeId::Ib => {
                Err(ExchangeError::Auth(
                    "Interactive Brokers requires credentials".into()
                ))
            }
            ExchangeId::Futu => {
                Err(ExchangeError::Auth(
                    "Futu requires OpenD TCP+Protobuf connection - create FutuConnector manually".into()
                ))
            }
            ExchangeId::Bls => {
                Err(ExchangeError::Auth(
                    "BLS is a data feed - use BlsConnector directly".into()
                ))
            }

            // ═══════════════════════════════════════════════════════════════════════
            // ON-CHAIN ANALYTICS — extracted to dig2onchain-data crate
            // ═══════════════════════════════════════════════════════════════════════
            ExchangeId::Coinglass => {
                Err(ExchangeError::Auth(
                    "Coinglass requires API key".into()
                ))
            }
            ExchangeId::WhaleAlert => {
                Err(ExchangeError::UnsupportedOperation(
                    "WhaleAlert has been extracted to the dig2onchain-data crate".into()
                ))
            }
            ExchangeId::Fred => {
                Err(ExchangeError::Auth(
                    "Fred connector has been removed - intelligence_feeds module is no longer available".into()
                ))
            }
            ExchangeId::Bitquery => {
                Err(ExchangeError::UnsupportedOperation(
                    "Bitquery has been extracted to the dig2onchain-data crate".into()
                ))
            }

            // Handle custom exchange IDs
            ExchangeId::Custom(_) => {
                Err(ExchangeError::Auth(
                    "Custom exchange IDs not supported by factory - create manually".into()
                ))
            }
        }
    }

    /// Create an authenticated connector with credentials
    ///
    /// Creates a connector instance with authentication credentials, enabling
    /// access to private endpoints for trading, account info, and positions.
    ///
    /// # Arguments
    ///
    /// * `id` - The exchange identifier
    /// * `credentials` - API credentials (api_key, api_secret, optional passphrase)
    ///
    /// # Returns
    ///
    /// * `Ok(Arc<dyn CoreConnector>)` - Wrapped authenticated connector instance
    /// * `Err(ExchangeError)` - If connector creation fails
    ///
    /// # Example
    ///
    /// ```ignore
    /// let credentials = Credentials::new("api_key", "api_secret")
    ///     .with_passphrase("passphrase"); // Optional for OKX, KuCoin
    ///
    /// let connector = ConnectorFactory::create_authenticated(
    ///     ExchangeId::Binance,
    ///     credentials
    /// ).await?;
    ///
    /// let balance = connector.get_balance(crate::core::types::BalanceQuery { asset: None, account_type: AccountType::Spot }).await?;
    /// ```
    pub(crate) async fn create_authenticated(
        id: ExchangeId,
        credentials: Credentials,
    ) -> ExchangeResult<Arc<dyn CoreConnector>> {
        let testnet = credentials.testnet;
        match id {
            // ═══════════════════════════════════════════════════════════════════════
            // CEX - Pattern: ::new(Some(credentials), testnet)
            // ═══════════════════════════════════════════════════════════════════════
            ExchangeId::Binance => {
                let c = BinanceConnector::new(Some(credentials), testnet).await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            ExchangeId::Bybit => {
                let c = BybitConnector::new(Some(credentials), testnet).await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            ExchangeId::OKX => {
                let c = OkxConnector::new(Some(credentials), testnet).await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            ExchangeId::KuCoin => {
                let c = KuCoinConnector::new(Some(credentials), testnet).await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            ExchangeId::Kraken => {
                let c = KrakenConnector::new(Some(credentials), testnet).await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            ExchangeId::GateIO => {
                let c = GateioConnector::new(Some(credentials), testnet).await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            ExchangeId::Bitfinex => {
                let c = BitfinexConnector::new(Some(credentials), testnet).await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            ExchangeId::MEXC => {
                let c = MexcConnector::new(Some(credentials)).await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            ExchangeId::HTX => {
                let c = HtxConnector::new(Some(credentials), testnet).await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            ExchangeId::BingX => {
                let c = BingxConnector::new(Some(credentials), testnet).await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            ExchangeId::CryptoCom => {
                let c = CryptoComConnector::new(Some(credentials), testnet).await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            ExchangeId::Upbit => {
                // Upbit requires region parameter: "kr" (Korea) or "sg" (Singapore)
                // Default to "kr" for authenticated
                let c = UpbitConnector::new(Some(credentials), "kr").await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            ExchangeId::Deribit => {
                let c = DeribitConnector::new(Some(credentials), testnet).await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            #[cfg(feature = "onchain-evm")]
            ExchangeId::HyperLiquid => {
                let c = HyperliquidConnector::new(Some(credentials), testnet).await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            #[cfg(not(feature = "onchain-evm"))]
            ExchangeId::HyperLiquid => {
                Err(ExchangeError::UnsupportedOperation(
                    "HyperLiquid requires the onchain-evm feature".into()
                ))
            }

            // ═══════════════════════════════════════════════════════════════════════
            // CEX - Pattern: ::new(Some(credentials), testnet) (no testnet param)
            // ═══════════════════════════════════════════════════════════════════════
            ExchangeId::Bitget => {
                let c = BitgetConnector::new(Some(credentials), testnet).await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            ExchangeId::Bitstamp => {
                let c = BitstampConnector::new(Some(credentials)).await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            ExchangeId::Coinbase => {
                let c = CoinbaseConnector::new(Some(credentials)).await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            ExchangeId::Gemini => {
                let c = GeminiConnector::new(Some(credentials), testnet).await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }

            // ═══════════════════════════════════════════════════════════════════════
            // DEX - Pattern: ::new(Some(credentials), testnet)
            // ═══════════════════════════════════════════════════════════════════════
            ExchangeId::Dydx => {
                let c = DydxConnector::new(Some(credentials), testnet).await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }

            // ═══════════════════════════════════════════════════════════════════════
            // CEX - Public-only (no auth supported)
            // ═══════════════════════════════════════════════════════════════════════
            ExchangeId::Bitmex => {
                // BitMEX public market data only — no authenticated connector
                Self::create_public(id, testnet).await
            }

            // ═══════════════════════════════════════════════════════════════════════
            // DEX - No auth supported (public only)
            // ═══════════════════════════════════════════════════════════════════════
            ExchangeId::Lighter => {
                // Lighter doesn't support authentication, use public connector
                Self::create_public(id, testnet).await
            }

            // ═══════════════════════════════════════════════════════════════════════
            // FOREX - Pattern: ::new(AlphaVantageAuth)
            // ═══════════════════════════════════════════════════════════════════════
            ExchangeId::AlphaVantage => {
                let auth = AlphaVantageAuth::new(credentials.api_key);
                let c = AlphaVantageConnector::new(auth);
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }

            // ═══════════════════════════════════════════════════════════════════════
            // AGGREGATORS - Pattern: ::new(api_key)
            // ═══════════════════════════════════════════════════════════════════════
            ExchangeId::CryptoCompare => {
                // CryptoCompare constructor is sync and needs CryptoCompareAuth
                let auth = crate::l2::paid::cryptocompare::CryptoCompareAuth::new(credentials.api_key);
                let c = CryptoCompareConnector::new(auth);
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            // ═══════════════════════════════════════════════════════════════════════
            // STOCKS US - Pattern: ::new(api_key)
            // ═══════════════════════════════════════════════════════════════════════
            ExchangeId::Polygon => {
                // Polygon::new takes (credentials, testnet)
                let c = PolygonConnector::new(credentials, testnet).await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            ExchangeId::Finnhub => {
                // Finnhub::new takes full credentials
                let c = FinnhubConnector::new(credentials).await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            ExchangeId::Tiingo => {
                // Tiingo::new takes full credentials
                let c = TiingoConnector::new(credentials).await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            ExchangeId::Twelvedata => {
                // Twelvedata constructor is sync (not async)
                let c = TwelvedataConnector::new(credentials.api_key);
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }

            // ═══════════════════════════════════════════════════════════════════════
            // STOCKS US - Pattern: ::new(credentials)
            // ═══════════════════════════════════════════════════════════════════════
            ExchangeId::Alpaca => {
                // Alpaca uses from_env() - factory with custom credentials not supported
                Err(ExchangeError::Auth(
                    "Alpaca connector uses from_env() - set environment variables instead".into()
                ))
            }

            // ═══════════════════════════════════════════════════════════════════════
            // STOCKS INDIA - Pattern: ::new(credentials)
            // ═══════════════════════════════════════════════════════════════════════
            ExchangeId::AngelOne => {
                // AngelOne has complex auth requiring TOTP
                // Factory cannot create - requires manual setup
                Err(ExchangeError::Auth(
                    "AngelOne requires complex authentication with TOTP - create manually".into()
                ))
            }
            ExchangeId::Zerodha => {
                // Zerodha uses OAuth - factory cannot create
                Err(ExchangeError::Auth(
                    "Zerodha requires OAuth authentication - create manually".into()
                ))
            }
            ExchangeId::Upstox => {
                // Upstox requires OAuth - factory not supported
                Err(ExchangeError::Auth(
                    "Upstox requires OAuth authentication - create manually".into()
                ))
            }
            ExchangeId::Dhan => {
                let c = DhanConnector::new(credentials, testnet).await?;
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }
            ExchangeId::Fyers => {
                // Fyers uses special auth type
                Err(ExchangeError::Auth(
                    "Fyers requires special authentication setup - create manually".into()
                ))
            }

            // ═══════════════════════════════════════════════════════════════════════
            // STOCKS OTHER - Pattern: ::new(credentials)
            // ═══════════════════════════════════════════════════════════════════════
            ExchangeId::JQuants => {
                // JQuants requires email/password - not standard Credentials
                Err(ExchangeError::Auth(
                    "JQuants requires email/password authentication - create manually".into()
                ))
            }
            ExchangeId::Krx => {
                // KRX requires special auth
                Err(ExchangeError::Auth(
                    "KRX requires special authentication - create manually".into()
                ))
            }
            ExchangeId::Moex => {
                // Moex requires special setup
                Err(ExchangeError::Auth(
                    "Moex connector requires special authentication - create manually".into()
                ))
            }
            ExchangeId::Tinkoff => {
                // Tinkoff requires token authentication
                Err(ExchangeError::Auth(
                    "Tinkoff requires token authentication - create manually".into()
                ))
            }

            // ═══════════════════════════════════════════════════════════════════════
            // FOREX - Pattern: ::new(credentials)
            // ═══════════════════════════════════════════════════════════════════════
            ExchangeId::Oanda => {
                // Oanda requires (credentials, practice) parameters
                Err(ExchangeError::Auth(
                    "Oanda requires practice mode specification - create manually".into()
                ))
            }
            ExchangeId::Dukascopy => {
                // Dukascopy is a data provider - no authentication supported
                Self::create_public(id, testnet).await
            }

            // ═══════════════════════════════════════════════════════════════════════
            // AGGREGATORS - Pattern: ::new(credentials)
            // ═══════════════════════════════════════════════════════════════════════
            ExchangeId::Ib => {
                // IB (Interactive Brokers) requires complex setup
                Err(ExchangeError::Auth(
                    "Interactive Brokers requires TWS/Gateway connection - create manually".into()
                ))
            }
            ExchangeId::Futu => {
                // Futu requires OpenD TCP+Protobuf connection — not constructable from HTTP credentials
                Err(ExchangeError::Auth(
                    "Futu requires OpenD TCP+Protobuf connection - create FutuConnector manually with FutuAuth".into()
                ))
            }

            // ═══════════════════════════════════════════════════════════════════════
            // DATA FEEDS - No auth required
            // ═══════════════════════════════════════════════════════════════════════
            ExchangeId::YahooFinance => {
                // YahooFinance doesn't need authentication, use public connector
                Self::create_public(id, testnet).await
            }
            ExchangeId::DefiLlama => {
                Err(ExchangeError::UnsupportedOperation(
                    "DefiLlama has moved to dig2feed crate".into()
                ))
            }

            // ═══════════════════════════════════════════════════════════════════════
            // PREDICTION MARKETS - Polymarket L2 auth (optional)
            // ═══════════════════════════════════════════════════════════════════════
            ExchangeId::Polymarket => {
                // Build PolymarketCredentials from the V5 Credentials struct.
                // Polymarket uses: address (api_key), api_key (api_secret), secret (passphrase field),
                // passphrase (extra field). Map best-effort from standard Credentials:
                //   credentials.api_key    → address (Polygon wallet)
                //   credentials.api_secret → api_key (UUID)
                //   credentials.passphrase → secret (base64 HMAC key)
                // Because Polymarket requires a 4th field (passphrase), and standard
                // Credentials only carries 3, callers should use PolymarketConnector::authenticated()
                // directly for full auth. Factory provides best-effort mapping.
                let poly_creds = crate::l3::open::prediction::polymarket::PolymarketCredentials::new(
                    credentials.api_key.clone(),
                    credentials.api_secret.clone(),
                    credentials.passphrase.clone().unwrap_or_default(),
                    String::new(), // passphrase field — not available in standard Credentials
                );
                let c = PolymarketConnector::authenticated(poly_creds);
                Ok(Arc::new(c) as Arc<dyn CoreConnector>)
            }

            // ═══════════════════════════════════════════════════════════════════════
            // ON-CHAIN ANALYTICS - Authenticated
            // ═══════════════════════════════════════════════════════════════════════
            ExchangeId::Coinglass => {
                Err(ExchangeError::Auth(
                    "Coinglass connector has been removed - intelligence_feeds module is no longer available".into()
                ))
            }
            ExchangeId::WhaleAlert => {
                Err(ExchangeError::UnsupportedOperation(
                    "WhaleAlert has been extracted to the dig2onchain-data crate".into()
                ))
            }
            ExchangeId::Fred => {
                Err(ExchangeError::Auth(
                    "Fred connector has been removed - intelligence_feeds module is no longer available".into()
                ))
            }
            ExchangeId::Bitquery => {
                Err(ExchangeError::UnsupportedOperation(
                    "Bitquery has been extracted to the dig2onchain-data crate".into()
                ))
            }
            ExchangeId::Bls => {
                Err(ExchangeError::UnsupportedOperation(
                    "BLS is a data feed without standard connector traits - use BlsConnector directly".into()
                ))
            }

            // Handle custom exchange IDs
            ExchangeId::Custom(_) => {
                Err(ExchangeError::Auth(
                    "Custom exchange IDs not supported by factory - create manually".into()
                ))
            }
        }
    }

    /// Create a public WebSocket connector for any supported exchange.
    ///
    /// Returns `Arc<dyn WebSocketConnector>` ready to `connect()`. All
    /// connectors are created with `None` credentials (public streams only).
    /// For private streams, construct the concrete `*WebSocket` struct directly.
    ///
    /// Exchanges that require credentials for WS construction (Alpaca, Dhan, IB,
    /// Tiingo, Polygon) return `Err(ExchangeError::UnsupportedOperation)`.
    ///
    /// # Arguments
    ///
    /// * `id` - The exchange identifier
    /// * `account_type` - Account type (spot, futures, etc.) — affects WS URL on
    ///   many exchanges (Binance, Bybit, Bitfinex, Gate.io, …)
    /// * `testnet` - Whether to connect to testnet endpoint
    ///
    /// # Example
    ///
    /// ```ignore
    /// let ws = ConnectorFactory::create_websocket(
    ///     ExchangeId::Binance,
    ///     AccountType::Spot,
    ///     false,
    /// ).await?;
    /// ws.connect(AccountType::Spot).await?;
    /// ```
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) async fn create_websocket(
        id: ExchangeId,
        account_type: AccountType,
        testnet: bool,
    ) -> ExchangeResult<Arc<dyn WebSocketConnector>> {
        match id {
            // ═══════════════════════════════════════════════════════════════════
            // CEX — new(credentials, testnet, account_type)
            // ═══════════════════════════════════════════════════════════════════
            ExchangeId::Binance => {
                let ws = BinanceWebSocket::new(None, testnet, account_type).await?;
                Ok(Arc::new(ws) as Arc<dyn WebSocketConnector>)
            }
            ExchangeId::Bybit => {
                let ws = BybitWebSocket::new(None, testnet, account_type).await?;
                Ok(Arc::new(ws) as Arc<dyn WebSocketConnector>)
            }
            ExchangeId::GateIO => {
                let ws = GateioWebSocket::new(None, testnet, account_type).await?;
                Ok(Arc::new(ws) as Arc<dyn WebSocketConnector>)
            }
            ExchangeId::Bitfinex => {
                // Sync constructor — UniversalWsTransport connects lazily on first subscribe.
                let _ = account_type;
                Ok(Arc::new(BitfinexWebSocket::new(testnet)) as Arc<dyn WebSocketConnector>)
            }
            ExchangeId::Bitget => {
                let ws = BitgetWebSocket::new(None, testnet, account_type).await?;
                Ok(Arc::new(ws) as Arc<dyn WebSocketConnector>)
            }
            ExchangeId::BingX => {
                // Sync constructor — UniversalWsTransport connects lazily on first subscribe.
                let ws = BingxWebSocket::new(None, testnet, account_type);
                Ok(Arc::new(ws) as Arc<dyn WebSocketConnector>)
            }
            ExchangeId::Deribit => {
                let ws = DeribitWebSocket::new(None, testnet, account_type).await?;
                Ok(Arc::new(ws) as Arc<dyn WebSocketConnector>)
            }
            ExchangeId::KuCoin => {
                let ws = KuCoinWebSocket::new(None, testnet, account_type).await?;
                Ok(Arc::new(ws) as Arc<dyn WebSocketConnector>)
            }
            // ═══════════════════════════════════════════════════════════════════
            // CEX — new(credentials, testnet) — no account_type param
            // ═══════════════════════════════════════════════════════════════════
            ExchangeId::OKX => {
                // OKX V5 splits channels across two endpoints (NOT supersets):
                //   /ws/v5/public  — tickers, mark-price, funding-rate, open-interest,
                //                    trades, liquidation-orders, books
                //   /ws/v5/business — candle*, opt-summary, ...
                // We connect to public here — kline/candle is documented as
                // NotSupported in the connector with a citation pointing to the
                // business endpoint. Multi-endpoint per connector is a Wave-4 item.
                let ws = OkxWebSocket::new(None, testnet, account_type).await?;
                Ok(Arc::new(ws) as Arc<dyn WebSocketConnector>)
            }
            // ═══════════════════════════════════════════════════════════════════
            // CEX — sync new(credentials, testnet, account_type)
            // ═══════════════════════════════════════════════════════════════════
            ExchangeId::HTX => {
                let ws = HtxWebSocket::new(None, testnet, account_type)?;
                Ok(Arc::new(ws) as Arc<dyn WebSocketConnector>)
            }
            // ═══════════════════════════════════════════════════════════════════
            // CEX — new(credentials) — no testnet, no account_type
            // ═══════════════════════════════════════════════════════════════════
            ExchangeId::Coinbase => {
                // Sync constructor — UniversalWsTransport connects lazily on first subscribe.
                Ok(Arc::new(CoinbaseWebSocket::public()) as Arc<dyn WebSocketConnector>)
            }
            // ═══════════════════════════════════════════════════════════════════
            // CEX — new() — no credentials
            // ═══════════════════════════════════════════════════════════════════
            ExchangeId::Bitstamp => {
                // Sync constructor — UniversalWsTransport connects lazily on first subscribe.
                Ok(Arc::new(BitstampWebSocket::new()) as Arc<dyn WebSocketConnector>)
            }
            ExchangeId::MEXC => {
                let ws = MexcWebSocket::new(None, account_type).await?;
                Ok(Arc::new(ws) as Arc<dyn WebSocketConnector>)
            }
            // ═══════════════════════════════════════════════════════════════════
            // CEX — Gemini: new(testnet) — sync, UniversalWsTransport connects lazily
            // ═══════════════════════════════════════════════════════════════════
            ExchangeId::Gemini => {
                let ws = GeminiWebSocket::new(testnet);
                Ok(Arc::new(ws) as Arc<dyn WebSocketConnector>)
            }
            // ═══════════════════════════════════════════════════════════════════
            // CEX — Kraken: new(token, account_type) — no credentials for public
            // ═══════════════════════════════════════════════════════════════════
            ExchangeId::Kraken => {
                // Sync constructor — UniversalWsTransport connects lazily on first subscribe.
                Ok(Arc::new(KrakenWebSocket::new()) as Arc<dyn WebSocketConnector>)
            }
            // ═══════════════════════════════════════════════════════════════════
            // CEX — Upbit: new(credentials, region)
            // ═══════════════════════════════════════════════════════════════════
            ExchangeId::Upbit => {
                // KRW markets live on the Korean endpoint (api.upbit.com).
                // sg-api.upbit.com handles international accounts but does not
                // serve KRW-* pairs over WebSocket.
                let ws = UpbitWebSocket::new(None, "kr").await?;
                Ok(Arc::new(ws) as Arc<dyn WebSocketConnector>)
            }
            // ═══════════════════════════════════════════════════════════════════
            // CEX — BitMEX: sync new(testnet)
            // ═══════════════════════════════════════════════════════════════════
            ExchangeId::Bitmex => {
                let ws = BitmexWebSocket::new(testnet);
                Ok(Arc::new(ws) as Arc<dyn WebSocketConnector>)
            }
            // ═══════════════════════════════════════════════════════════════════
            // CEX — HyperLiquid: sync new(is_testnet)
            // ═══════════════════════════════════════════════════════════════════
            #[cfg(feature = "onchain-evm")]
            ExchangeId::HyperLiquid => {
                let ws = HyperliquidWebSocket::new(testnet);
                Ok(Arc::new(ws) as Arc<dyn WebSocketConnector>)
            }
            #[cfg(not(feature = "onchain-evm"))]
            ExchangeId::HyperLiquid => {
                Err(crate::core::types::WebSocketError::UnsupportedOperation(
                    "HyperLiquid requires the onchain-evm feature".into()
                ))
            }
            // ═══════════════════════════════════════════════════════════════════
            // CEX — CryptoCom: sync new(testnet) — UniversalWsTransport wrapper
            // ═══════════════════════════════════════════════════════════════════
            ExchangeId::CryptoCom => {
                let ws = CryptoComWebSocket::new(testnet);
                Ok(Arc::new(ws) as Arc<dyn WebSocketConnector>)
            }
            // ═══════════════════════════════════════════════════════════════════
            // DEX — DyDx: new(testnet, account_type)
            // ═══════════════════════════════════════════════════════════════════
            ExchangeId::Dydx => {
                let ws = DydxWebSocket::new(testnet, account_type).await?;
                Ok(Arc::new(ws) as Arc<dyn WebSocketConnector>)
            }
            // ═══════════════════════════════════════════════════════════════════
            // DEX — Lighter: public(testnet)
            // ═══════════════════════════════════════════════════════════════════
            ExchangeId::Lighter => {
                let ws = LighterWebSocket::public(testnet).await?;
                Ok(Arc::new(ws) as Arc<dyn WebSocketConnector>)
            }
            // ═══════════════════════════════════════════════════════════════════
            // PREDICTION — Polymarket: ClobWebSocket does not impl WebSocketConnector
            // ═══════════════════════════════════════════════════════════════════
            ExchangeId::Polymarket => {
                Err(ExchangeError::UnsupportedOperation(
                    "Polymarket ClobWebSocket does not implement WebSocketConnector — use ClobWebSocket directly".into()
                ))
            }
            // ═══════════════════════════════════════════════════════════════════
            // DATA FEEDS — public constructors
            // ═══════════════════════════════════════════════════════════════════
            ExchangeId::YahooFinance => {
                let ws = YahooFinanceWebSocket::new();
                Ok(Arc::new(ws) as Arc<dyn WebSocketConnector>)
            }
            ExchangeId::CryptoCompare => {
                let ws = CryptoCompareWebSocket::new();
                Ok(Arc::new(ws) as Arc<dyn WebSocketConnector>)
            }
            ExchangeId::Twelvedata => {
                // TwelvedataWebSocket requires an API key — no public no-key constructor
                Err(ExchangeError::UnsupportedOperation(
                    "TwelvedataWebSocket requires an API key — construct directly with TwelvedataWebSocket::new(api_key)".into()
                ))
            }
            // ═══════════════════════════════════════════════════════════════════
            // DATA FEEDS — require credentials (auth not accessible from factory)
            // ═══════════════════════════════════════════════════════════════════
            ExchangeId::Finnhub => {
                // FinnhubWebSocket::new(credentials) — requires API key
                Err(ExchangeError::UnsupportedOperation(
                    "FinnhubWebSocket requires credentials — construct directly with FinnhubWebSocket::new(credentials)".into()
                ))
            }
            ExchangeId::Polygon => {
                // PolygonWebSocket::new(credentials, realtime) — requires API key
                Err(ExchangeError::UnsupportedOperation(
                    "PolygonWebSocket requires credentials — construct directly with PolygonWebSocket::new(credentials, realtime)".into()
                ))
            }
            ExchangeId::Tiingo => {
                // TiingoWebSocket requires TiingoAuth (internal type, not re-exported)
                Err(ExchangeError::UnsupportedOperation(
                    "TiingoWebSocket requires TiingoAuth — construct directly via TiingoWebSocket::new_iex/new_forex/new_crypto".into()
                ))
            }
            ExchangeId::Moex => {
                let ws = MoexWebSocket::new_public();
                Ok(Arc::new(ws) as Arc<dyn WebSocketConnector>)
            }
            // ═══════════════════════════════════════════════════════════════════
            // GATED — require credentials, no public WS
            // ═══════════════════════════════════════════════════════════════════
            ExchangeId::Alpaca => {
                // AlpacaWebSocket::new(AlpacaAuth) — auth not re-exported
                Err(ExchangeError::UnsupportedOperation(
                    "AlpacaWebSocket requires AlpacaAuth — construct directly with AlpacaWebSocket::new(auth)".into()
                ))
            }
            ExchangeId::Dhan => {
                // DhanWebSocket::new(access_token)
                Err(ExchangeError::UnsupportedOperation(
                    "DhanWebSocket requires access token — construct directly with DhanWebSocket::new(token)".into()
                ))
            }
            ExchangeId::Ib => {
                // IBWebSocket::new(ws_url)
                Err(ExchangeError::UnsupportedOperation(
                    "IBWebSocket requires a TWS/Gateway URL — construct directly with IBWebSocket::new(url)".into()
                ))
            }
            // ═══════════════════════════════════════════════════════════════════
            // No WebSocket implementation
            // ═══════════════════════════════════════════════════════════════════
            ExchangeId::AlphaVantage
            | ExchangeId::AngelOne
            | ExchangeId::Zerodha
            | ExchangeId::Upstox
            | ExchangeId::Fyers
            | ExchangeId::Oanda
            | ExchangeId::Dukascopy
            | ExchangeId::JQuants
            | ExchangeId::Krx
            | ExchangeId::Tinkoff
            | ExchangeId::Futu
            | ExchangeId::Bls
            | ExchangeId::Coinglass
            | ExchangeId::WhaleAlert
            | ExchangeId::Fred
            | ExchangeId::Bitquery
            | ExchangeId::DefiLlama => Err(ExchangeError::UnsupportedOperation(
                format!("{id:?} has no WebSocket implementation in digdigdig3")
            )),
            ExchangeId::Custom(_) => Err(ExchangeError::UnsupportedOperation(
                "Custom exchange IDs not supported by WebSocket factory".into()
            )),
        }
    }

    /// Create a WebSocket connector for browser (wasm32) targets.
    ///
    /// Supports Binance, Bybit, OKX — the three exchanges whose WS
    /// implementations use `UniversalWsTransport` with browser-native WS
    /// (`web-sys`). Other exchanges return `UnsupportedOperation`.
    #[cfg(target_arch = "wasm32")]
    pub(crate) async fn create_websocket(
        id: ExchangeId,
        account_type: AccountType,
        testnet: bool,
    ) -> ExchangeResult<Arc<dyn WebSocketConnector>> {
        match id {
            ExchangeId::Binance => {
                let ws = BinanceWebSocket::new(None, testnet, account_type).await?;
                Ok(Arc::new(ws) as Arc<dyn WebSocketConnector>)
            }
            ExchangeId::Bybit => {
                let ws = BybitWebSocket::new(None, testnet, account_type).await?;
                Ok(Arc::new(ws) as Arc<dyn WebSocketConnector>)
            }
            ExchangeId::OKX => {
                let ws = OkxWebSocket::new(None, testnet, account_type).await?;
                Ok(Arc::new(ws) as Arc<dyn WebSocketConnector>)
            }
            #[cfg(feature = "onchain-evm")]
            ExchangeId::HyperLiquid => {
                let _ = account_type;
                let ws = HyperliquidWebSocket::new(testnet);
                Ok(Arc::new(ws) as Arc<dyn WebSocketConnector>)
            }
            ExchangeId::Gemini => {
                let _ = account_type;
                let ws = GeminiWebSocket::new(testnet);
                Ok(Arc::new(ws) as Arc<dyn WebSocketConnector>)
            }
            ExchangeId::CryptoCom => {
                let _ = account_type;
                let ws = CryptoComWebSocket::new(testnet);
                Ok(Arc::new(ws) as Arc<dyn WebSocketConnector>)
            }
            ExchangeId::Bitfinex => {
                let _ = account_type;
                let ws = BitfinexWebSocket::new(testnet);
                Ok(Arc::new(ws) as Arc<dyn WebSocketConnector>)
            }
            ExchangeId::BingX => {
                let ws = BingxWebSocket::new(None, testnet, account_type);
                Ok(Arc::new(ws) as Arc<dyn WebSocketConnector>)
            }
            other => Err(ExchangeError::UnsupportedOperation(format!(
                "{other:?} WebSocket not enabled on wasm32; \
                 only Binance/Bybit/OKX/HyperLiquid/Gemini/CryptoCom/Bitfinex/BingX use UniversalWsTransport+browser-WS"
            ))),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// UNIT TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connector_manager::ConnectorRegistry;

    /// Test that we can create public connectors for exchanges that support it
    #[tokio::test]
    async fn test_create_public_binance() {
        let result = ConnectorFactory::create_public(ExchangeId::Binance, false).await;
        assert!(result.is_ok(), "Should create Binance public connector");

        let connector = result.unwrap();
        assert_eq!(connector.exchange_id(), ExchangeId::Binance);
    }

    /// Test creating authenticated connector
    #[tokio::test]
    async fn test_create_authenticated_binance() {
        let credentials = Credentials::new("test_key", "test_secret");
        let result = ConnectorFactory::create_authenticated(
            ExchangeId::Binance,
            credentials
        ).await;

        // May fail due to invalid credentials, but should not panic
        match result {
            Ok(connector) => {
                assert_eq!(connector.exchange_id(), ExchangeId::Binance);
            }
            Err(e) => {
                println!("Expected error with test credentials: {:?}", e);
            }
        }
    }

    /// Test that every registered exchange can be instantiated (either public or with auth).
    /// Count comes from CONNECTOR_METADATA_ARRAY — no hardcoded target.
    #[tokio::test]
    async fn test_factory_coverage_all_registered_exchanges() {
        let registry = ConnectorRegistry::default();
        let all_metas = registry.list_all();
        let total = all_metas.len();

        assert!(total >= 30, "registry shrunk below 30 — accidental wipe?");

        println!("\n=== Testing Factory Coverage for {} Exchanges ===\n", total);

        let mut public_success = 0;
        let mut public_requires_auth = 0;
        let mut auth_attempted = 0;

        for meta in all_metas {
            println!("Testing {}", meta.name);

            // Try public first
            let public_result = ConnectorFactory::create_public(meta.id, false).await;
            match public_result {
                Ok(_) => {
                    println!("  ✓ Public connector created");
                    public_success += 1;
                }
                Err(ExchangeError::Auth(_)) => {
                    println!("  ⚠ Requires authentication");
                    public_requires_auth += 1;

                    // Try with dummy credentials
                    let credentials = Credentials::new("dummy_key", "dummy_secret");
                    let auth_result = ConnectorFactory::create_authenticated(
                        meta.id,
                        credentials
                    ).await;

                    match auth_result {
                        Ok(_) => {
                            println!("  ✓ Authenticated connector created (may fail at runtime)");
                            auth_attempted += 1;
                        }
                        Err(e) => {
                            println!("  ✗ Auth failed (expected with dummy creds): {:?}", e);
                            auth_attempted += 1;
                        }
                    }
                }
                Err(e) => {
                    panic!("Unexpected error creating public connector for {}: {:?}", meta.name, e);
                }
            }
        }

        println!("\n=== Factory Coverage Summary ===");
        println!("Public connectors created: {}", public_success);
        println!("Require authentication: {}", public_requires_auth);
        println!("Auth connectors attempted: {}", auth_attempted);
        println!("Total coverage: {}/{}", public_success + public_requires_auth, total);

        assert_eq!(
            public_success + public_requires_auth,
            total,
            "Factory should handle all {} registered exchanges",
            total
        );
    }

    /// Test factory with passphrase-based exchanges (OKX, KuCoin)
    #[tokio::test]
    async fn test_create_authenticated_with_passphrase() {
        let credentials = Credentials::new("test_key", "test_secret")
            .with_passphrase("test_passphrase");

        let result = ConnectorFactory::create_authenticated(
            ExchangeId::OKX,
            credentials
        ).await;

        // May fail due to invalid credentials, but should not panic
        match result {
            Ok(connector) => {
                assert_eq!(connector.exchange_id(), ExchangeId::OKX);
            }
            Err(e) => {
                println!("Expected error with test credentials: {:?}", e);
            }
        }
    }

    /// Test that factory creates correct connector type
    #[tokio::test]
    async fn test_factory_creates_correct_type() {
        let result = ConnectorFactory::create_public(ExchangeId::Bybit, false).await;
        assert!(result.is_ok());

        let connector = result.unwrap();
        assert_eq!(connector.exchange_id(), ExchangeId::Bybit);
        // Note: is_testnet() is on ExchangeIdentity, available via Arc<dyn CoreConnector>
    }

    /// Test multiple connector creation (cloning Arc is cheap)
    #[tokio::test]
    async fn test_factory_multiple_creation() {
        let conn1 = ConnectorFactory::create_public(ExchangeId::Binance, false).await.unwrap();
        let conn2 = ConnectorFactory::create_public(ExchangeId::Bybit, false).await.unwrap();
        let conn3 = ConnectorFactory::create_public(ExchangeId::OKX, false).await.unwrap();

        assert_eq!(conn1.exchange_id(), ExchangeId::Binance);
        assert_eq!(conn2.exchange_id(), ExchangeId::Bybit);
        assert_eq!(conn3.exchange_id(), ExchangeId::OKX);
    }
}
