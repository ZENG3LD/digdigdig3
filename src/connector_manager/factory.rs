//! # Connector Factory
//!
//! Factory for creating connectors by ExchangeId.
//!
//! ## Overview
//!
//! This module provides a unified factory interface for creating any of the 48
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
//! ### Pattern D: `::new(api_key)` (async)
//! - DEX: Jupiter (requires API key since Oct 2025)
//! - Special cases requiring specific parameters
//!
//! ### Pattern F: `::from_env()` (sync)
//! - Data feeds: Alpaca
//! - Load credentials from environment variables

use std::sync::Arc;
use crate::core::types::{ExchangeId, ExchangeResult, ExchangeError};
use crate::core::traits::Credentials;
use crate::connector_manager::AnyConnector;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR IMPORTS - CEX
// ═══════════════════════════════════════════════════════════════════════════════

use crate::crypto::cex::binance::BinanceConnector;
use crate::crypto::cex::bybit::BybitConnector;
use crate::crypto::cex::okx::OkxConnector;
use crate::crypto::cex::kucoin::KuCoinConnector;
use crate::crypto::cex::kraken::KrakenConnector;
use crate::crypto::cex::coinbase::CoinbaseConnector;
use crate::crypto::cex::gateio::GateioConnector;
use crate::crypto::cex::bitfinex::BitfinexConnector;
use crate::crypto::cex::bitstamp::BitstampConnector;
use crate::crypto::cex::gemini::GeminiConnector;
use crate::crypto::cex::mexc::MexcConnector;
use crate::crypto::cex::htx::HtxConnector;
use crate::crypto::cex::bitget::BitgetConnector;
use crate::crypto::cex::bingx::BingxConnector;
use crate::crypto::cex::phemex::PhemexConnector;
use crate::crypto::cex::crypto_com::CryptoComConnector;
use crate::crypto::cex::upbit::UpbitConnector;
use crate::crypto::cex::deribit::DeribitConnector;
use crate::crypto::cex::hyperliquid::HyperliquidConnector;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR IMPORTS - DEX
// ═══════════════════════════════════════════════════════════════════════════════

use crate::crypto::dex::lighter::LighterConnector;
#[cfg(feature = "onchain-evm")]
use crate::crypto::swap::uniswap::UniswapConnector;
use crate::crypto::dex::jupiter::JupiterConnector;
use crate::crypto::swap::raydium::RaydiumConnector;
use crate::crypto::dex::gmx::GmxConnector;
use crate::crypto::dex::paradex::ParadexConnector;
use crate::crypto::dex::dydx::DydxConnector;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR IMPORTS - STOCKS US
// ═══════════════════════════════════════════════════════════════════════════════

use crate::stocks::us::polygon::PolygonConnector;
use crate::stocks::us::finnhub::FinnhubConnector;
use crate::stocks::us::tiingo::TiingoConnector;
use crate::stocks::us::twelvedata::TwelvedataConnector;
use crate::stocks::us::alpaca::AlpacaConnector;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR IMPORTS - STOCKS INDIA
// ═══════════════════════════════════════════════════════════════════════════════

use crate::stocks::india::dhan::DhanConnector;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR IMPORTS - STOCKS OTHER
// ═══════════════════════════════════════════════════════════════════════════════

use crate::stocks::korea::krx::KrxConnector;
use crate::stocks::russia::moex::MoexConnector;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR IMPORTS - FOREX
// ═══════════════════════════════════════════════════════════════════════════════

use crate::forex::dukascopy::DukascopyConnector;
use crate::forex::alphavantage::{AlphaVantageConnector, AlphaVantageAuth};

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR IMPORTS - PREDICTION
// ═══════════════════════════════════════════════════════════════════════════════

use crate::prediction::polymarket::PolymarketConnector;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR IMPORTS - DATA FEEDS
// ═══════════════════════════════════════════════════════════════════════════════

use crate::data_feeds::yahoo::YahooFinanceConnector;
use crate::data_feeds::cryptocompare::CryptoCompareConnector;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR IMPORTS - ON-CHAIN ANALYTICS
// ═══════════════════════════════════════════════════════════════════════════════

use crate::onchain::analytics::whale_alert::{WhaleAlertConnector, WhaleAlertAuth};
use crate::onchain::analytics::bitquery::BitqueryConnector;

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
pub struct ConnectorFactory;

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
    /// * `Ok(Arc<AnyConnector>)` - Wrapped connector instance
    /// * `Err(ExchangeError)` - If connector creation fails or requires auth
    ///
    /// # Example
    ///
    /// ```ignore
    /// let connector = ConnectorFactory::create_public(ExchangeId::Binance, false).await?;
    /// let price = connector.get_price(symbol, AccountType::Spot).await?;
    /// ```
    pub async fn create_public(id: ExchangeId, testnet: bool) -> ExchangeResult<Arc<AnyConnector>> {
        match id {
            // ═══════════════════════════════════════════════════════════════════════
            // CEX - Pattern A: ::public(testnet: bool)
            // ═══════════════════════════════════════════════════════════════════════
            ExchangeId::Binance => {
                let c = BinanceConnector::public(testnet).await?;
                Ok(Arc::new(AnyConnector::Binance(Arc::new(c))))
            }
            ExchangeId::Bybit => {
                let c = BybitConnector::public(testnet).await?;
                Ok(Arc::new(AnyConnector::Bybit(Arc::new(c))))
            }
            ExchangeId::OKX => {
                let c = OkxConnector::public(testnet).await?;
                Ok(Arc::new(AnyConnector::OKX(Arc::new(c))))
            }
            ExchangeId::KuCoin => {
                let c = KuCoinConnector::public(testnet).await?;
                Ok(Arc::new(AnyConnector::KuCoin(Arc::new(c))))
            }
            ExchangeId::Kraken => {
                let c = KrakenConnector::public(testnet).await?;
                Ok(Arc::new(AnyConnector::Kraken(Arc::new(c))))
            }
            ExchangeId::GateIO => {
                let c = GateioConnector::public(testnet).await?;
                Ok(Arc::new(AnyConnector::GateIO(Arc::new(c))))
            }
            ExchangeId::Bitfinex => {
                let c = BitfinexConnector::public(testnet).await?;
                Ok(Arc::new(AnyConnector::Bitfinex(Arc::new(c))))
            }
            ExchangeId::MEXC => {
                let c = MexcConnector::public().await?;
                Ok(Arc::new(AnyConnector::MEXC(Arc::new(c))))
            }
            ExchangeId::HTX => {
                let c = HtxConnector::public(testnet).await?;
                Ok(Arc::new(AnyConnector::HTX(Arc::new(c))))
            }
            ExchangeId::BingX => {
                let c = BingxConnector::public(testnet).await?;
                Ok(Arc::new(AnyConnector::BingX(Arc::new(c))))
            }
            ExchangeId::Phemex => {
                let c = PhemexConnector::public(testnet).await?;
                Ok(Arc::new(AnyConnector::Phemex(Arc::new(c))))
            }
            ExchangeId::CryptoCom => {
                let c = CryptoComConnector::public(testnet).await?;
                Ok(Arc::new(AnyConnector::CryptoCom(Arc::new(c))))
            }
            ExchangeId::Upbit => {
                let c = UpbitConnector::public().await?;
                Ok(Arc::new(AnyConnector::Upbit(Arc::new(c))))
            }
            ExchangeId::Deribit => {
                let c = DeribitConnector::public(testnet).await?;
                Ok(Arc::new(AnyConnector::Deribit(Arc::new(c))))
            }
            ExchangeId::HyperLiquid => {
                let c = HyperliquidConnector::public(testnet).await?;
                Ok(Arc::new(AnyConnector::HyperLiquid(Arc::new(c))))
            }
            ExchangeId::Dydx => {
                let c = DydxConnector::public(testnet).await?;
                Ok(Arc::new(AnyConnector::Dydx(Arc::new(c))))
            }
            ExchangeId::Paradex => {
                let c = ParadexConnector::public(testnet).await?;
                Ok(Arc::new(AnyConnector::Paradex(Arc::new(c))))
            }

            // ═══════════════════════════════════════════════════════════════════════
            // CEX - Pattern B: ::public() (no testnet param)
            // ═══════════════════════════════════════════════════════════════════════
            ExchangeId::Bitget => {
                let c = BitgetConnector::public().await?;
                Ok(Arc::new(AnyConnector::Bitget(Arc::new(c))))
            }
            ExchangeId::Bitstamp => {
                let c = BitstampConnector::public().await?;
                Ok(Arc::new(AnyConnector::Bitstamp(Arc::new(c))))
            }
            ExchangeId::Coinbase => {
                let c = CoinbaseConnector::public().await?;
                Ok(Arc::new(AnyConnector::Coinbase(Arc::new(c))))
            }
            ExchangeId::Gemini => {
                let c = GeminiConnector::public(testnet).await?;
                Ok(Arc::new(AnyConnector::Gemini(Arc::new(c))))
            }

            // ═══════════════════════════════════════════════════════════════════════
            // DEX - Pattern B: ::public() or ::new()
            // ═══════════════════════════════════════════════════════════════════════
            ExchangeId::Lighter => {
                let c = LighterConnector::public(testnet).await?;
                Ok(Arc::new(AnyConnector::Lighter(Arc::new(c))))
            }
            ExchangeId::Uniswap => {
                let c = UniswapConnector::public(testnet).await?;
                Ok(Arc::new(AnyConnector::Uniswap(Arc::new(c))))
            }
            ExchangeId::Raydium => {
                let c = RaydiumConnector::new(testnet).await?;
                Ok(Arc::new(AnyConnector::Raydium(Arc::new(c))))
            }
            ExchangeId::Gmx => {
                let c = GmxConnector::arbitrum().await?;
                Ok(Arc::new(AnyConnector::Gmx(Arc::new(c))))
            }

            // ═══════════════════════════════════════════════════════════════════════
            // DEX - Special: Jupiter requires API key
            // ═══════════════════════════════════════════════════════════════════════
            ExchangeId::Jupiter => {
                Err(ExchangeError::Auth(
                    "Jupiter requires API key (use create_authenticated with api_key in Credentials)".into()
                ))
            }

            // ═══════════════════════════════════════════════════════════════════════
            // STOCKS - Pattern C: ::crypto_only() for public, ::from_env() for auth
            // ═══════════════════════════════════════════════════════════════════════
            ExchangeId::Alpaca => {
                // Create crypto-only connector (works without API keys)
                let c = AlpacaConnector::crypto_only();
                Ok(Arc::new(AnyConnector::Alpaca(Arc::new(c))))
            }

            // ═══════════════════════════════════════════════════════════════════════
            // AGGREGATORS & PREDICTION - Pattern D: ::public()
            // ═══════════════════════════════════════════════════════════════════════
            ExchangeId::YahooFinance => {
                let c = YahooFinanceConnector::new();
                Ok(Arc::new(AnyConnector::YahooFinance(Arc::new(c))))
            }
            ExchangeId::DefiLlama => {
                Err(ExchangeError::UnsupportedOperation(
                    "DefiLlama has moved to dig2feed crate".into()
                ))
            }
            ExchangeId::Polymarket => {
                let c = PolymarketConnector::public();
                Ok(Arc::new(AnyConnector::Polymarket(Arc::new(c))))
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
                Ok(Arc::new(AnyConnector::Twelvedata(Arc::new(c))))
            }
            ExchangeId::AlphaVantage => {
                Err(ExchangeError::Auth(
                    "AlphaVantage requires API key".into()
                ))
            }
            ExchangeId::CryptoCompare => {
                let c = CryptoCompareConnector::public();
                Ok(Arc::new(AnyConnector::CryptoCompare(Arc::new(c))))
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
                Ok(Arc::new(AnyConnector::Dukascopy(Arc::new(c))))
            }
            ExchangeId::JQuants => {
                Err(ExchangeError::Auth(
                    "JQuants requires credentials".into()
                ))
            }
            ExchangeId::Krx => {
                #[allow(deprecated)]
                let c = KrxConnector::new_public();
                Ok(Arc::new(AnyConnector::Krx(Arc::new(c))))
            }
            ExchangeId::Moex => {
                let c = MoexConnector::new_public();
                Ok(Arc::new(AnyConnector::Moex(Arc::new(c))))
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
            // ON-CHAIN ANALYTICS
            // ═══════════════════════════════════════════════════════════════════════
            ExchangeId::Coinglass => {
                Err(ExchangeError::Auth(
                    "Coinglass requires API key".into()
                ))
            }
            ExchangeId::WhaleAlert => {
                // WhaleAlert can work without API key at reduced rate limits (v1 API)
                let auth = WhaleAlertAuth::none();
                let c = WhaleAlertConnector::new(auth);
                Ok(Arc::new(AnyConnector::WhaleAlert(Arc::new(c))))
            }
            ExchangeId::Fred => {
                Err(ExchangeError::Auth(
                    "Fred connector has been removed - intelligence_feeds module is no longer available".into()
                ))
            }
            ExchangeId::Bitquery => {
                Err(ExchangeError::Auth(
                    "Bitquery requires OAuth token (use create_authenticated with api_key in Credentials)".into()
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
    /// * `Ok(Arc<AnyConnector>)` - Wrapped authenticated connector instance
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
    pub async fn create_authenticated(
        id: ExchangeId,
        credentials: Credentials,
    ) -> ExchangeResult<Arc<AnyConnector>> {
        let testnet = credentials.testnet;
        match id {
            // ═══════════════════════════════════════════════════════════════════════
            // CEX - Pattern: ::new(Some(credentials), testnet)
            // ═══════════════════════════════════════════════════════════════════════
            ExchangeId::Binance => {
                let c = BinanceConnector::new(Some(credentials), testnet).await?;
                Ok(Arc::new(AnyConnector::Binance(Arc::new(c))))
            }
            ExchangeId::Bybit => {
                let c = BybitConnector::new(Some(credentials), testnet).await?;
                Ok(Arc::new(AnyConnector::Bybit(Arc::new(c))))
            }
            ExchangeId::OKX => {
                let c = OkxConnector::new(Some(credentials), testnet).await?;
                Ok(Arc::new(AnyConnector::OKX(Arc::new(c))))
            }
            ExchangeId::KuCoin => {
                let c = KuCoinConnector::new(Some(credentials), testnet).await?;
                Ok(Arc::new(AnyConnector::KuCoin(Arc::new(c))))
            }
            ExchangeId::Kraken => {
                let c = KrakenConnector::new(Some(credentials), testnet).await?;
                Ok(Arc::new(AnyConnector::Kraken(Arc::new(c))))
            }
            ExchangeId::GateIO => {
                let c = GateioConnector::new(Some(credentials), testnet).await?;
                Ok(Arc::new(AnyConnector::GateIO(Arc::new(c))))
            }
            ExchangeId::Bitfinex => {
                let c = BitfinexConnector::new(Some(credentials), testnet).await?;
                Ok(Arc::new(AnyConnector::Bitfinex(Arc::new(c))))
            }
            ExchangeId::MEXC => {
                let c = MexcConnector::new(Some(credentials)).await?;
                Ok(Arc::new(AnyConnector::MEXC(Arc::new(c))))
            }
            ExchangeId::HTX => {
                let c = HtxConnector::new(Some(credentials), testnet).await?;
                Ok(Arc::new(AnyConnector::HTX(Arc::new(c))))
            }
            ExchangeId::BingX => {
                let c = BingxConnector::new(Some(credentials), testnet).await?;
                Ok(Arc::new(AnyConnector::BingX(Arc::new(c))))
            }
            ExchangeId::Phemex => {
                let c = PhemexConnector::new(Some(credentials), testnet).await?;
                Ok(Arc::new(AnyConnector::Phemex(Arc::new(c))))
            }
            ExchangeId::CryptoCom => {
                let c = CryptoComConnector::new(Some(credentials), testnet).await?;
                Ok(Arc::new(AnyConnector::CryptoCom(Arc::new(c))))
            }
            ExchangeId::Upbit => {
                // Upbit requires region parameter: "kr" (Korea) or "sg" (Singapore)
                // Default to "kr" for authenticated
                let c = UpbitConnector::new(Some(credentials), "kr").await?;
                Ok(Arc::new(AnyConnector::Upbit(Arc::new(c))))
            }
            ExchangeId::Deribit => {
                let c = DeribitConnector::new(Some(credentials), testnet).await?;
                Ok(Arc::new(AnyConnector::Deribit(Arc::new(c))))
            }
            ExchangeId::HyperLiquid => {
                let c = HyperliquidConnector::new(Some(credentials), testnet).await?;
                Ok(Arc::new(AnyConnector::HyperLiquid(Arc::new(c))))
            }

            // ═══════════════════════════════════════════════════════════════════════
            // CEX - Pattern: ::new(Some(credentials), testnet) (no testnet param)
            // ═══════════════════════════════════════════════════════════════════════
            ExchangeId::Bitget => {
                let c = BitgetConnector::new(Some(credentials), testnet).await?;
                Ok(Arc::new(AnyConnector::Bitget(Arc::new(c))))
            }
            ExchangeId::Bitstamp => {
                let c = BitstampConnector::new(Some(credentials)).await?;
                Ok(Arc::new(AnyConnector::Bitstamp(Arc::new(c))))
            }
            ExchangeId::Coinbase => {
                let c = CoinbaseConnector::new(Some(credentials)).await?;
                Ok(Arc::new(AnyConnector::Coinbase(Arc::new(c))))
            }
            ExchangeId::Gemini => {
                let c = GeminiConnector::new(Some(credentials), testnet).await?;
                Ok(Arc::new(AnyConnector::Gemini(Arc::new(c))))
            }

            // ═══════════════════════════════════════════════════════════════════════
            // DEX - Pattern: ::new(Some(credentials), testnet)
            // ═══════════════════════════════════════════════════════════════════════
            ExchangeId::Dydx => {
                let c = DydxConnector::new(Some(credentials), testnet).await?;
                Ok(Arc::new(AnyConnector::Dydx(Arc::new(c))))
            }
            ExchangeId::Paradex => {
                // Paradex::new takes credentials directly (not Option)
                let c = ParadexConnector::new(credentials, testnet).await?;
                Ok(Arc::new(AnyConnector::Paradex(Arc::new(c))))
            }

            // ═══════════════════════════════════════════════════════════════════════
            // DEX - Pattern: ::new(api_key)
            // ═══════════════════════════════════════════════════════════════════════
            ExchangeId::Jupiter => {
                let c = JupiterConnector::new(credentials.api_key).await?;
                Ok(Arc::new(AnyConnector::Jupiter(Arc::new(c))))
            }

            // ═══════════════════════════════════════════════════════════════════════
            // DEX - No auth supported (public only)
            // ═══════════════════════════════════════════════════════════════════════
            ExchangeId::Lighter |
            ExchangeId::Uniswap |
            ExchangeId::Raydium |
            ExchangeId::Gmx => {
                // These DEXs don't support authentication, use public connector
                Self::create_public(id, testnet).await
            }

            // ═══════════════════════════════════════════════════════════════════════
            // FOREX - Pattern: ::new(AlphaVantageAuth)
            // ═══════════════════════════════════════════════════════════════════════
            ExchangeId::AlphaVantage => {
                let auth = AlphaVantageAuth::new(credentials.api_key);
                let c = AlphaVantageConnector::new(auth);
                Ok(Arc::new(AnyConnector::AlphaVantage(Arc::new(c))))
            }

            // ═══════════════════════════════════════════════════════════════════════
            // AGGREGATORS - Pattern: ::new(api_key)
            // ═══════════════════════════════════════════════════════════════════════
            ExchangeId::CryptoCompare => {
                // CryptoCompare constructor is sync and needs CryptoCompareAuth
                let auth = crate::data_feeds::cryptocompare::CryptoCompareAuth::new(credentials.api_key);
                let c = CryptoCompareConnector::new(auth);
                Ok(Arc::new(AnyConnector::CryptoCompare(Arc::new(c))))
            }
            // ═══════════════════════════════════════════════════════════════════════
            // STOCKS US - Pattern: ::new(api_key)
            // ═══════════════════════════════════════════════════════════════════════
            ExchangeId::Polygon => {
                // Polygon::new takes (credentials, testnet)
                let c = PolygonConnector::new(credentials, testnet).await?;
                Ok(Arc::new(AnyConnector::Polygon(Arc::new(c))))
            }
            ExchangeId::Finnhub => {
                // Finnhub::new takes full credentials
                let c = FinnhubConnector::new(credentials).await?;
                Ok(Arc::new(AnyConnector::Finnhub(Arc::new(c))))
            }
            ExchangeId::Tiingo => {
                // Tiingo::new takes full credentials
                let c = TiingoConnector::new(credentials).await?;
                Ok(Arc::new(AnyConnector::Tiingo(Arc::new(c))))
            }
            ExchangeId::Twelvedata => {
                // Twelvedata constructor is sync (not async)
                let c = TwelvedataConnector::new(credentials.api_key);
                Ok(Arc::new(AnyConnector::Twelvedata(Arc::new(c))))
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
                Ok(Arc::new(AnyConnector::Dhan(Arc::new(c))))
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
                let poly_creds = crate::prediction::polymarket::PolymarketCredentials::new(
                    credentials.api_key.clone(),
                    credentials.api_secret.clone(),
                    credentials.passphrase.clone().unwrap_or_default(),
                    String::new(), // passphrase field — not available in standard Credentials
                );
                let c = PolymarketConnector::authenticated(poly_creds);
                Ok(Arc::new(AnyConnector::Polymarket(Arc::new(c))))
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
                let auth = WhaleAlertAuth::new(credentials.api_key);
                let c = WhaleAlertConnector::new(auth);
                Ok(Arc::new(AnyConnector::WhaleAlert(Arc::new(c))))
            }
            ExchangeId::Fred => {
                Err(ExchangeError::Auth(
                    "Fred connector has been removed - intelligence_feeds module is no longer available".into()
                ))
            }
            ExchangeId::Bitquery => {
                let c = BitqueryConnector::new(credentials).await?;
                Ok(Arc::new(AnyConnector::Bitquery(Arc::new(c))))
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
        assert_eq!(connector.id(), ExchangeId::Binance);
    }

    /// Test that exchanges requiring auth return appropriate error
    #[tokio::test]
    async fn test_create_public_requires_auth() {
        let result = ConnectorFactory::create_public(ExchangeId::Jupiter, false).await;
        assert!(result.is_err(), "Jupiter requires API key");

        if let Err(ExchangeError::Auth(msg)) = result {
            assert!(msg.contains("Jupiter"), "Error message should mention Jupiter");
        } else {
            panic!("Expected Auth error");
        }
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
                assert_eq!(connector.id(), ExchangeId::Binance);
            }
            Err(e) => {
                println!("Expected error with test credentials: {:?}", e);
            }
        }
    }

    /// Test that all 48 exchanges can be instantiated (either public or with auth)
    #[tokio::test]
    async fn test_factory_coverage_all_51_exchanges() {
        let registry = ConnectorRegistry::default();
        let all_metas = registry.list_all();

        assert_eq!(all_metas.len(), 48, "Registry should have 48 connectors");

        println!("\n=== Testing Factory Coverage for 48 Exchanges ===\n");

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
        println!("Total coverage: {}/48", public_success + public_requires_auth);

        assert_eq!(
            public_success + public_requires_auth,
            48,
            "Factory should handle all 48 exchanges"
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
                assert_eq!(connector.id(), ExchangeId::OKX);
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
        assert_eq!(connector.id(), ExchangeId::Bybit);
        // Note: is_testnet() is on ExchangeIdentity trait, which AnyConnector implements
    }

    /// Test multiple connector creation (cloning Arc is cheap)
    #[tokio::test]
    async fn test_factory_multiple_creation() {
        let conn1 = ConnectorFactory::create_public(ExchangeId::Binance, false).await.unwrap();
        let conn2 = ConnectorFactory::create_public(ExchangeId::Bybit, false).await.unwrap();
        let conn3 = ConnectorFactory::create_public(ExchangeId::OKX, false).await.unwrap();

        assert_eq!(conn1.id(), ExchangeId::Binance);
        assert_eq!(conn2.id(), ExchangeId::Bybit);
        assert_eq!(conn3.id(), ExchangeId::OKX);
    }
}
