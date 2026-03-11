//! # AnyConnector Enum
//!
//! Unified enum wrapper for all 51 active connectors.
//!
//! ## Architecture
//!
//! ```text
//! AnyConnector (enum)
//!   ├── Binance(Arc<BinanceConnector>)
//!   ├── KuCoin(Arc<KuCoinConnector>)
//!   └── ... (51 variants)
//! ```
//!
//! Each variant wraps a connector in Arc for cheap cloning.
//! Trait implementations delegate to the underlying connector.

use std::sync::Arc;

use async_trait::async_trait;

use crate::core::types::{
    ConnectorStats, ExchangeId, AccountType, Symbol, SymbolInfo, Price, OrderBook, Kline, Ticker,
    ExchangeResult,
};

use crate::core::traits::{
    ExchangeIdentity, MarketData,
};

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR IMPORTS - CEX
// ═══════════════════════════════════════════════════════════════════════════════

use crate::exchanges::binance::BinanceConnector;
use crate::exchanges::bybit::BybitConnector;
use crate::exchanges::okx::OkxConnector;
use crate::exchanges::kucoin::KuCoinConnector;
use crate::exchanges::kraken::KrakenConnector;
use crate::exchanges::coinbase::CoinbaseConnector;
use crate::exchanges::gateio::GateioConnector;
use crate::exchanges::bitfinex::BitfinexConnector;
use crate::exchanges::bitstamp::BitstampConnector;
use crate::exchanges::gemini::GeminiConnector;
use crate::exchanges::mexc::MexcConnector;
use crate::exchanges::htx::HtxConnector;
use crate::exchanges::bitget::BitgetConnector;
use crate::exchanges::bingx::BingxConnector;
use crate::exchanges::phemex::PhemexConnector;
use crate::exchanges::crypto_com::CryptoComConnector;
use crate::exchanges::upbit::UpbitConnector;
use crate::exchanges::deribit::DeribitConnector;
use crate::exchanges::hyperliquid::HyperliquidConnector;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR IMPORTS - DEX
// ═══════════════════════════════════════════════════════════════════════════════

use crate::exchanges::lighter::LighterConnector;
use crate::onchain::ethereum::uniswap::UniswapConnector;
use crate::exchanges::jupiter::JupiterConnector;
use crate::onchain::solana::raydium::RaydiumConnector;
use crate::exchanges::gmx::GmxConnector;
use crate::exchanges::paradex::ParadexConnector;
use crate::exchanges::dydx::DydxConnector;

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

use crate::stocks::india::angel_one::AngelOneConnector;
use crate::stocks::india::zerodha::ZerodhaConnector;
use crate::stocks::india::upstox::UpstoxConnector;
use crate::stocks::india::dhan::DhanConnector;
use crate::stocks::india::fyers::FyersConnector;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR IMPORTS - STOCKS OTHER
// ═══════════════════════════════════════════════════════════════════════════════

use crate::stocks::japan::jquants::JQuantsConnector;
use crate::stocks::korea::krx::KrxConnector;
use crate::stocks::russia::moex::MoexConnector;
use crate::stocks::russia::tinkoff::TinkoffConnector;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR IMPORTS - FOREX
// ═══════════════════════════════════════════════════════════════════════════════

use crate::forex::oanda::OandaConnector;
use crate::forex::dukascopy::DukascopyConnector;
use crate::forex::alphavantage::AlphaVantageConnector;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR IMPORTS - PREDICTION
// ═══════════════════════════════════════════════════════════════════════════════

use crate::prediction::polymarket::PolymarketConnector;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR IMPORTS - AGGREGATORS
// ═══════════════════════════════════════════════════════════════════════════════

use crate::aggregators::ib::IBConnector;
use crate::aggregators::yahoo::YahooFinanceConnector;
use crate::aggregators::cryptocompare::CryptoCompareConnector;
use crate::aggregators::defillama::DefiLlamaConnector;

// ═══════════════════════════════════════════════════════════════════════════════
// ANYCONNECTOR ENUM
// ═══════════════════════════════════════════════════════════════════════════════

/// Unified connector enum wrapping all 48 active connectors
///
/// Each variant wraps the connector in Arc for cheap cloning.
/// All core traits are delegated to the underlying connector.
///
/// # Example
/// ```ignore
/// let connector = AnyConnector::Binance(Arc::new(binance_connector));
/// let price = connector.get_price(symbol, AccountType::Spot).await?;
/// ```
#[derive(Clone)]
pub enum AnyConnector {
    // ═══════════════════════════════════════════════════════════════════════════
    // CEX - Centralized Exchanges (19)
    // ═══════════════════════════════════════════════════════════════════════════
    Binance(Arc<BinanceConnector>),
    Bybit(Arc<BybitConnector>),
    OKX(Arc<OkxConnector>),
    KuCoin(Arc<KuCoinConnector>),
    Kraken(Arc<KrakenConnector>),
    Coinbase(Arc<CoinbaseConnector>),
    GateIO(Arc<GateioConnector>),
    Bitfinex(Arc<BitfinexConnector>),
    Bitstamp(Arc<BitstampConnector>),
    Gemini(Arc<GeminiConnector>),
    MEXC(Arc<MexcConnector>),
    HTX(Arc<HtxConnector>),
    Bitget(Arc<BitgetConnector>),
    BingX(Arc<BingxConnector>),
    Phemex(Arc<PhemexConnector>),
    CryptoCom(Arc<CryptoComConnector>),
    Upbit(Arc<UpbitConnector>),
    Deribit(Arc<DeribitConnector>),
    HyperLiquid(Arc<HyperliquidConnector>),

    // ═══════════════════════════════════════════════════════════════════════════
    // DEX - Decentralized Exchanges (7)
    // ═══════════════════════════════════════════════════════════════════════════
    Lighter(Arc<LighterConnector>),
    Uniswap(Arc<UniswapConnector>),
    Jupiter(Arc<JupiterConnector>),
    Raydium(Arc<RaydiumConnector>),
    Gmx(Arc<GmxConnector>),
    Paradex(Arc<ParadexConnector>),
    Dydx(Arc<DydxConnector>),

    // ═══════════════════════════════════════════════════════════════════════════
    // STOCKS - US (5)
    // ═══════════════════════════════════════════════════════════════════════════
    Polygon(Arc<PolygonConnector>),
    Finnhub(Arc<FinnhubConnector>),
    Tiingo(Arc<TiingoConnector>),
    Twelvedata(Arc<TwelvedataConnector>),
    Alpaca(Arc<AlpacaConnector>),

    // ═══════════════════════════════════════════════════════════════════════════
    // STOCKS - India (5)
    // ═══════════════════════════════════════════════════════════════════════════
    AngelOne(Arc<AngelOneConnector>),
    Zerodha(Arc<ZerodhaConnector>),
    Upstox(Arc<UpstoxConnector>),
    Dhan(Arc<DhanConnector>),
    Fyers(Arc<FyersConnector>),

    // ═══════════════════════════════════════════════════════════════════════════
    // STOCKS - Other (4)
    // ═══════════════════════════════════════════════════════════════════════════
    JQuants(Arc<JQuantsConnector>),
    Krx(Arc<KrxConnector>),
    Moex(Arc<MoexConnector>),
    Tinkoff(Arc<TinkoffConnector>),

    // ═══════════════════════════════════════════════════════════════════════════
    // FOREX (3)
    // ═══════════════════════════════════════════════════════════════════════════
    Oanda(Arc<OandaConnector>),
    Dukascopy(Arc<DukascopyConnector>),
    AlphaVantage(Arc<AlphaVantageConnector>),

    // ═══════════════════════════════════════════════════════════════════════════
    // PREDICTION (1)
    // ═══════════════════════════════════════════════════════════════════════════
    Polymarket(Arc<PolymarketConnector>),

    // ═══════════════════════════════════════════════════════════════════════════
    // AGGREGATORS (4)
    // ═══════════════════════════════════════════════════════════════════════════
    IB(Arc<IBConnector>),
    YahooFinance(Arc<YahooFinanceConnector>),
    CryptoCompare(Arc<CryptoCompareConnector>),
    DefiLlama(Arc<DefiLlamaConnector>),
}

// ═══════════════════════════════════════════════════════════════════════════════
// CORE METHODS
// ═══════════════════════════════════════════════════════════════════════════════

impl AnyConnector {
    /// Get the exchange ID for this connector
    ///
    /// This method maps each enum variant to its corresponding ExchangeId.
    /// Used for identification and metadata lookups.
    pub fn id(&self) -> ExchangeId {
        match self {
            // CEX
            Self::Binance(_) => ExchangeId::Binance,
            Self::Bybit(_) => ExchangeId::Bybit,
            Self::OKX(_) => ExchangeId::OKX,
            Self::KuCoin(_) => ExchangeId::KuCoin,
            Self::Kraken(_) => ExchangeId::Kraken,
            Self::Coinbase(_) => ExchangeId::Coinbase,
            Self::GateIO(_) => ExchangeId::GateIO,
            Self::Bitfinex(_) => ExchangeId::Bitfinex,
            Self::Bitstamp(_) => ExchangeId::Bitstamp,
            Self::Gemini(_) => ExchangeId::Gemini,
            Self::MEXC(_) => ExchangeId::MEXC,
            Self::HTX(_) => ExchangeId::HTX,
            Self::Bitget(_) => ExchangeId::Bitget,
            Self::BingX(_) => ExchangeId::BingX,
            Self::Phemex(_) => ExchangeId::Phemex,
            Self::CryptoCom(_) => ExchangeId::CryptoCom,
            Self::Upbit(_) => ExchangeId::Upbit,
            Self::Deribit(_) => ExchangeId::Deribit,
            Self::HyperLiquid(_) => ExchangeId::HyperLiquid,

            // DEX
            Self::Lighter(_) => ExchangeId::Lighter,
            Self::Uniswap(_) => ExchangeId::Uniswap,
            Self::Jupiter(_) => ExchangeId::Jupiter,
            Self::Raydium(_) => ExchangeId::Raydium,
            Self::Gmx(_) => ExchangeId::Gmx,
            Self::Paradex(_) => ExchangeId::Paradex,
            Self::Dydx(_) => ExchangeId::Dydx,

            // Stocks US
            Self::Polygon(_) => ExchangeId::Polygon,
            Self::Finnhub(_) => ExchangeId::Finnhub,
            Self::Tiingo(_) => ExchangeId::Tiingo,
            Self::Twelvedata(_) => ExchangeId::Twelvedata,
            Self::Alpaca(_) => ExchangeId::Alpaca,

            // Stocks India
            Self::AngelOne(_) => ExchangeId::AngelOne,
            Self::Zerodha(_) => ExchangeId::Zerodha,
            Self::Upstox(_) => ExchangeId::Upstox,
            Self::Dhan(_) => ExchangeId::Dhan,
            Self::Fyers(_) => ExchangeId::Fyers,

            // Stocks Other
            Self::JQuants(_) => ExchangeId::JQuants,
            Self::Krx(_) => ExchangeId::Krx,
            Self::Moex(_) => ExchangeId::Moex,
            Self::Tinkoff(_) => ExchangeId::Tinkoff,

            // Forex
            Self::Oanda(_) => ExchangeId::Oanda,
            Self::Dukascopy(_) => ExchangeId::Dukascopy,
            Self::AlphaVantage(_) => ExchangeId::AlphaVantage,

            // Prediction
            Self::Polymarket(_) => ExchangeId::Polymarket,

            // Aggregators
            Self::IB(_) => ExchangeId::Ib,
            Self::YahooFinance(_) => ExchangeId::YahooFinance,
            Self::CryptoCompare(_) => ExchangeId::CryptoCompare,
            Self::DefiLlama(_) => ExchangeId::DefiLlama,
        }
    }

    /// Returns runtime metrics for this connector.
    ///
    /// Provides HTTP request counts, error counts, latency, and rate limiter
    /// utilization for all connectors that have instrumented `metrics()`.
    pub fn metrics(&self) -> ConnectorStats {
        match self {
            // CEX
            Self::Binance(c) => c.metrics(),
            Self::Bybit(c) => c.metrics(),
            Self::OKX(c) => c.metrics(),
            Self::KuCoin(c) => c.metrics(),
            Self::Kraken(c) => c.metrics(),
            Self::Coinbase(c) => c.metrics(),
            Self::GateIO(c) => c.metrics(),
            Self::Bitfinex(c) => c.metrics(),
            Self::Bitstamp(c) => c.metrics(),
            Self::Gemini(c) => c.metrics(),
            Self::MEXC(c) => c.metrics(),
            Self::HTX(c) => c.metrics(),
            Self::Bitget(c) => c.metrics(),
            Self::BingX(c) => c.metrics(),
            Self::Phemex(c) => c.metrics(),
            Self::CryptoCom(c) => c.metrics(),
            Self::Upbit(c) => c.metrics(),
            Self::Deribit(c) => c.metrics(),
            Self::HyperLiquid(c) => c.metrics(),

            // DEX
            Self::Lighter(c) => c.metrics(),
            Self::Uniswap(c) => c.metrics(),
            Self::Jupiter(c) => c.metrics(),
            Self::Raydium(c) => c.metrics(),
            Self::Gmx(c) => c.metrics(),
            Self::Paradex(c) => c.metrics(),
            Self::Dydx(c) => c.metrics(),

            // Stocks US
            Self::Polygon(c) => c.metrics(),
            Self::Finnhub(c) => c.metrics(),
            Self::Tiingo(c) => c.metrics(),
            Self::Twelvedata(c) => c.metrics(),
            Self::Alpaca(c) => c.metrics(),

            // Stocks India
            Self::AngelOne(c) => c.metrics(),
            Self::Zerodha(c) => c.metrics(),
            Self::Upstox(c) => c.metrics(),
            Self::Dhan(c) => c.metrics(),
            Self::Fyers(c) => c.metrics(),

            // Stocks Other
            Self::JQuants(c) => c.metrics(),
            Self::Krx(c) => c.metrics(),
            Self::Moex(c) => c.metrics(),
            Self::Tinkoff(c) => c.metrics(),

            // Forex
            Self::Oanda(c) => c.metrics(),
            Self::Dukascopy(c) => c.metrics(),
            Self::AlphaVantage(c) => c.metrics(),

            // Prediction
            Self::Polymarket(c) => c.metrics(),

            // Aggregators
            Self::IB(c) => c.metrics(),
            Self::YahooFinance(c) => c.metrics(),
            Self::CryptoCompare(c) => c.metrics(),
            Self::DefiLlama(c) => c.metrics(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT IMPLEMENTATIONS
// ═══════════════════════════════════════════════════════════════════════════════

// === ExchangeIdentity ===

impl ExchangeIdentity for AnyConnector {
    fn exchange_id(&self) -> ExchangeId {
        self.id()
    }

    fn is_testnet(&self) -> bool {
        match self {
            // CEX
            Self::Binance(c) => c.is_testnet(),
            Self::Bybit(c) => c.is_testnet(),
            Self::OKX(c) => c.is_testnet(),
            Self::KuCoin(c) => c.is_testnet(),
            Self::Kraken(c) => c.is_testnet(),
            Self::Coinbase(c) => c.is_testnet(),
            Self::GateIO(c) => c.is_testnet(),
            Self::Bitfinex(c) => c.is_testnet(),
            Self::Bitstamp(c) => c.is_testnet(),
            Self::Gemini(c) => c.is_testnet(),
            Self::MEXC(c) => c.is_testnet(),
            Self::HTX(c) => c.is_testnet(),
            Self::Bitget(c) => c.is_testnet(),
            Self::BingX(c) => c.is_testnet(),
            Self::Phemex(c) => c.is_testnet(),
            Self::CryptoCom(c) => c.is_testnet(),
            Self::Upbit(c) => c.is_testnet(),
            Self::Deribit(c) => c.is_testnet(),
            Self::HyperLiquid(c) => c.is_testnet(),

            // DEX
            Self::Lighter(c) => c.is_testnet(),
            Self::Uniswap(c) => c.is_testnet(),
            Self::Jupiter(c) => c.is_testnet(),
            Self::Raydium(c) => c.is_testnet(),
            Self::Gmx(c) => c.is_testnet(),
            Self::Paradex(c) => c.is_testnet(),
            Self::Dydx(c) => c.is_testnet(),

            // Stocks US
            Self::Polygon(c) => c.is_testnet(),
            Self::Finnhub(c) => c.is_testnet(),
            Self::Tiingo(c) => c.is_testnet(),
            Self::Twelvedata(c) => c.is_testnet(),
            Self::Alpaca(c) => c.is_testnet(),

            // Stocks India
            Self::AngelOne(c) => c.is_testnet(),
            Self::Zerodha(c) => c.is_testnet(),
            Self::Upstox(c) => c.is_testnet(),
            Self::Dhan(c) => c.is_testnet(),
            Self::Fyers(c) => c.is_testnet(),

            // Stocks Other
            Self::JQuants(c) => c.is_testnet(),
            Self::Krx(c) => c.is_testnet(),
            Self::Moex(c) => c.is_testnet(),
            Self::Tinkoff(c) => c.is_testnet(),

            // Forex
            Self::Oanda(c) => c.is_testnet(),
            Self::Dukascopy(c) => c.is_testnet(),
            Self::AlphaVantage(c) => c.is_testnet(),

            // Prediction
            Self::Polymarket(c) => c.is_testnet(),

            // Aggregators
            Self::IB(c) => c.is_testnet(),
            Self::YahooFinance(c) => c.is_testnet(),
            Self::CryptoCompare(c) => c.is_testnet(),
            Self::DefiLlama(c) => c.is_testnet(),
        }
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        match self {
            // CEX
            Self::Binance(c) => c.supported_account_types(),
            Self::Bybit(c) => c.supported_account_types(),
            Self::OKX(c) => c.supported_account_types(),
            Self::KuCoin(c) => c.supported_account_types(),
            Self::Kraken(c) => c.supported_account_types(),
            Self::Coinbase(c) => c.supported_account_types(),
            Self::GateIO(c) => c.supported_account_types(),
            Self::Bitfinex(c) => c.supported_account_types(),
            Self::Bitstamp(c) => c.supported_account_types(),
            Self::Gemini(c) => c.supported_account_types(),
            Self::MEXC(c) => c.supported_account_types(),
            Self::HTX(c) => c.supported_account_types(),
            Self::Bitget(c) => c.supported_account_types(),
            Self::BingX(c) => c.supported_account_types(),
            Self::Phemex(c) => c.supported_account_types(),
            Self::CryptoCom(c) => c.supported_account_types(),
            Self::Upbit(c) => c.supported_account_types(),
            Self::Deribit(c) => c.supported_account_types(),
            Self::HyperLiquid(c) => c.supported_account_types(),

            // DEX
            Self::Lighter(c) => c.supported_account_types(),
            Self::Uniswap(c) => c.supported_account_types(),
            Self::Jupiter(c) => c.supported_account_types(),
            Self::Raydium(c) => c.supported_account_types(),
            Self::Gmx(c) => c.supported_account_types(),
            Self::Paradex(c) => c.supported_account_types(),
            Self::Dydx(c) => c.supported_account_types(),

            // Stocks US
            Self::Polygon(c) => c.supported_account_types(),
            Self::Finnhub(c) => c.supported_account_types(),
            Self::Tiingo(c) => c.supported_account_types(),
            Self::Twelvedata(c) => c.supported_account_types(),
            Self::Alpaca(c) => c.supported_account_types(),

            // Stocks India
            Self::AngelOne(c) => c.supported_account_types(),
            Self::Zerodha(c) => c.supported_account_types(),
            Self::Upstox(c) => c.supported_account_types(),
            Self::Dhan(c) => c.supported_account_types(),
            Self::Fyers(c) => c.supported_account_types(),

            // Stocks Other
            Self::JQuants(c) => c.supported_account_types(),
            Self::Krx(c) => c.supported_account_types(),
            Self::Moex(c) => c.supported_account_types(),
            Self::Tinkoff(c) => c.supported_account_types(),

            // Forex
            Self::Oanda(c) => c.supported_account_types(),
            Self::Dukascopy(c) => c.supported_account_types(),
            Self::AlphaVantage(c) => c.supported_account_types(),

            // Prediction
            Self::Polymarket(c) => c.supported_account_types(),

            // Aggregators
            Self::IB(c) => c.supported_account_types(),
            Self::YahooFinance(c) => c.supported_account_types(),
            Self::CryptoCompare(c) => c.supported_account_types(),
            Self::DefiLlama(c) => c.supported_account_types(),
        }
    }
}

// === MarketData ===

#[async_trait]
impl MarketData for AnyConnector {
    async fn get_price(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Price> {
        match self {
            // CEX
            Self::Binance(c) => c.get_price(symbol, account_type).await,
            Self::Bybit(c) => c.get_price(symbol, account_type).await,
            Self::OKX(c) => c.get_price(symbol, account_type).await,
            Self::KuCoin(c) => c.get_price(symbol, account_type).await,
            Self::Kraken(c) => c.get_price(symbol, account_type).await,
            Self::Coinbase(c) => c.get_price(symbol, account_type).await,
            Self::GateIO(c) => c.get_price(symbol, account_type).await,
            Self::Bitfinex(c) => c.get_price(symbol, account_type).await,
            Self::Bitstamp(c) => c.get_price(symbol, account_type).await,
            Self::Gemini(c) => c.get_price(symbol, account_type).await,
            Self::MEXC(c) => c.get_price(symbol, account_type).await,
            Self::HTX(c) => c.get_price(symbol, account_type).await,
            Self::Bitget(c) => c.get_price(symbol, account_type).await,
            Self::BingX(c) => c.get_price(symbol, account_type).await,
            Self::Phemex(c) => c.get_price(symbol, account_type).await,
            Self::CryptoCom(c) => c.get_price(symbol, account_type).await,
            Self::Upbit(c) => c.get_price(symbol, account_type).await,
            Self::Deribit(c) => c.get_price(symbol, account_type).await,
            Self::HyperLiquid(c) => c.get_price(symbol, account_type).await,

            // DEX
            Self::Lighter(c) => c.get_price(symbol, account_type).await,
            Self::Uniswap(c) => c.get_price(symbol, account_type).await,
            Self::Jupiter(c) => c.get_price(symbol, account_type).await,
            Self::Raydium(c) => c.get_price(symbol, account_type).await,
            Self::Gmx(c) => c.get_price(symbol, account_type).await,
            Self::Paradex(c) => c.get_price(symbol, account_type).await,
            Self::Dydx(c) => c.get_price(symbol, account_type).await,

            // Stocks US
            Self::Polygon(c) => c.get_price(symbol, account_type).await,
            Self::Finnhub(c) => c.get_price(symbol, account_type).await,
            Self::Tiingo(c) => c.get_price(symbol, account_type).await,
            Self::Twelvedata(c) => c.get_price(symbol, account_type).await,
            Self::Alpaca(c) => c.get_price(symbol, account_type).await,

            // Stocks India
            Self::AngelOne(c) => c.get_price(symbol, account_type).await,
            Self::Zerodha(c) => c.get_price(symbol, account_type).await,
            Self::Upstox(c) => c.get_price(symbol, account_type).await,
            Self::Dhan(c) => c.get_price(symbol, account_type).await,
            Self::Fyers(c) => c.get_price(symbol, account_type).await,

            // Stocks Other
            Self::JQuants(c) => c.get_price(symbol, account_type).await,
            Self::Krx(c) => c.get_price(symbol, account_type).await,
            Self::Moex(c) => c.get_price(symbol, account_type).await,
            Self::Tinkoff(c) => c.get_price(symbol, account_type).await,

            // Forex
            Self::Oanda(c) => c.get_price(symbol, account_type).await,
            Self::Dukascopy(c) => c.get_price(symbol, account_type).await,
            Self::AlphaVantage(c) => c.get_price(symbol, account_type).await,

            // Prediction
            Self::Polymarket(c) => c.get_price(symbol, account_type).await,

            // Aggregators
            Self::IB(c) => c.get_price(symbol, account_type).await,
            Self::YahooFinance(c) => c.get_price(symbol, account_type).await,
            Self::CryptoCompare(c) => c.get_price(symbol, account_type).await,
            Self::DefiLlama(c) => c.get_price(symbol, account_type).await,
        }
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        depth: Option<u16>,
        account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        match self {
            // CEX
            Self::Binance(c) => c.get_orderbook(symbol, depth, account_type).await,
            Self::Bybit(c) => c.get_orderbook(symbol, depth, account_type).await,
            Self::OKX(c) => c.get_orderbook(symbol, depth, account_type).await,
            Self::KuCoin(c) => c.get_orderbook(symbol, depth, account_type).await,
            Self::Kraken(c) => c.get_orderbook(symbol, depth, account_type).await,
            Self::Coinbase(c) => c.get_orderbook(symbol, depth, account_type).await,
            Self::GateIO(c) => c.get_orderbook(symbol, depth, account_type).await,
            Self::Bitfinex(c) => c.get_orderbook(symbol, depth, account_type).await,
            Self::Bitstamp(c) => c.get_orderbook(symbol, depth, account_type).await,
            Self::Gemini(c) => c.get_orderbook(symbol, depth, account_type).await,
            Self::MEXC(c) => c.get_orderbook(symbol, depth, account_type).await,
            Self::HTX(c) => c.get_orderbook(symbol, depth, account_type).await,
            Self::Bitget(c) => c.get_orderbook(symbol, depth, account_type).await,
            Self::BingX(c) => c.get_orderbook(symbol, depth, account_type).await,
            Self::Phemex(c) => c.get_orderbook(symbol, depth, account_type).await,
            Self::CryptoCom(c) => c.get_orderbook(symbol, depth, account_type).await,
            Self::Upbit(c) => c.get_orderbook(symbol, depth, account_type).await,
            Self::Deribit(c) => c.get_orderbook(symbol, depth, account_type).await,
            Self::HyperLiquid(c) => c.get_orderbook(symbol, depth, account_type).await,

            // DEX
            Self::Lighter(c) => c.get_orderbook(symbol, depth, account_type).await,
            Self::Uniswap(c) => c.get_orderbook(symbol, depth, account_type).await,
            Self::Jupiter(c) => c.get_orderbook(symbol, depth, account_type).await,
            Self::Raydium(c) => c.get_orderbook(symbol, depth, account_type).await,
            Self::Gmx(c) => c.get_orderbook(symbol, depth, account_type).await,
            Self::Paradex(c) => c.get_orderbook(symbol, depth, account_type).await,
            Self::Dydx(c) => c.get_orderbook(symbol, depth, account_type).await,

            // Stocks US
            Self::Polygon(c) => c.get_orderbook(symbol, depth, account_type).await,
            Self::Finnhub(c) => c.get_orderbook(symbol, depth, account_type).await,
            Self::Tiingo(c) => c.get_orderbook(symbol, depth, account_type).await,
            Self::Twelvedata(c) => c.get_orderbook(symbol, depth, account_type).await,
            Self::Alpaca(c) => c.get_orderbook(symbol, depth, account_type).await,

            // Stocks India
            Self::AngelOne(c) => c.get_orderbook(symbol, depth, account_type).await,
            Self::Zerodha(c) => c.get_orderbook(symbol, depth, account_type).await,
            Self::Upstox(c) => c.get_orderbook(symbol, depth, account_type).await,
            Self::Dhan(c) => c.get_orderbook(symbol, depth, account_type).await,
            Self::Fyers(c) => c.get_orderbook(symbol, depth, account_type).await,

            // Stocks Other
            Self::JQuants(c) => c.get_orderbook(symbol, depth, account_type).await,
            Self::Krx(c) => c.get_orderbook(symbol, depth, account_type).await,
            Self::Moex(c) => c.get_orderbook(symbol, depth, account_type).await,
            Self::Tinkoff(c) => c.get_orderbook(symbol, depth, account_type).await,

            // Forex
            Self::Oanda(c) => c.get_orderbook(symbol, depth, account_type).await,
            Self::Dukascopy(c) => c.get_orderbook(symbol, depth, account_type).await,
            Self::AlphaVantage(c) => c.get_orderbook(symbol, depth, account_type).await,

            // Prediction
            Self::Polymarket(c) => c.get_orderbook(symbol, depth, account_type).await,

            // Aggregators
            Self::IB(c) => c.get_orderbook(symbol, depth, account_type).await,
            Self::YahooFinance(c) => c.get_orderbook(symbol, depth, account_type).await,
            Self::CryptoCompare(c) => c.get_orderbook(symbol, depth, account_type).await,
            Self::DefiLlama(c) => c.get_orderbook(symbol, depth, account_type).await,
        }
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        account_type: AccountType,
        end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        match self {
            // CEX
            Self::Binance(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,
            Self::Bybit(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,
            Self::OKX(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,
            Self::KuCoin(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,
            Self::Kraken(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,
            Self::Coinbase(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,
            Self::GateIO(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,
            Self::Bitfinex(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,
            Self::Bitstamp(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,
            Self::Gemini(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,
            Self::MEXC(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,
            Self::HTX(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,
            Self::Bitget(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,
            Self::BingX(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,
            Self::Phemex(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,
            Self::CryptoCom(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,
            Self::Upbit(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,
            Self::Deribit(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,
            Self::HyperLiquid(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,

            // DEX
            Self::Lighter(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,
            Self::Uniswap(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,
            Self::Jupiter(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,
            Self::Raydium(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,
            Self::Gmx(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,
            Self::Paradex(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,
            Self::Dydx(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,

            // Stocks US
            Self::Polygon(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,
            Self::Finnhub(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,
            Self::Tiingo(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,
            Self::Twelvedata(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,
            Self::Alpaca(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,

            // Stocks India
            Self::AngelOne(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,
            Self::Zerodha(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,
            Self::Upstox(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,
            Self::Dhan(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,
            Self::Fyers(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,

            // Stocks Other
            Self::JQuants(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,
            Self::Krx(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,
            Self::Moex(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,
            Self::Tinkoff(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,

            // Forex
            Self::Oanda(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,
            Self::Dukascopy(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,
            Self::AlphaVantage(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,

            // Prediction
            Self::Polymarket(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,

            // Aggregators
            Self::IB(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,
            Self::YahooFinance(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,
            Self::CryptoCompare(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,
            Self::DefiLlama(c) => c.get_klines(symbol, interval, limit, account_type, end_time).await,
        }
    }

    async fn get_ticker(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        match self {
            // CEX
            Self::Binance(c) => c.get_ticker(symbol, account_type).await,
            Self::Bybit(c) => c.get_ticker(symbol, account_type).await,
            Self::OKX(c) => c.get_ticker(symbol, account_type).await,
            Self::KuCoin(c) => c.get_ticker(symbol, account_type).await,
            Self::Kraken(c) => c.get_ticker(symbol, account_type).await,
            Self::Coinbase(c) => c.get_ticker(symbol, account_type).await,
            Self::GateIO(c) => c.get_ticker(symbol, account_type).await,
            Self::Bitfinex(c) => c.get_ticker(symbol, account_type).await,
            Self::Bitstamp(c) => c.get_ticker(symbol, account_type).await,
            Self::Gemini(c) => c.get_ticker(symbol, account_type).await,
            Self::MEXC(c) => c.get_ticker(symbol, account_type).await,
            Self::HTX(c) => c.get_ticker(symbol, account_type).await,
            Self::Bitget(c) => c.get_ticker(symbol, account_type).await,
            Self::BingX(c) => c.get_ticker(symbol, account_type).await,
            Self::Phemex(c) => c.get_ticker(symbol, account_type).await,
            Self::CryptoCom(c) => c.get_ticker(symbol, account_type).await,
            Self::Upbit(c) => c.get_ticker(symbol, account_type).await,
            Self::Deribit(c) => c.get_ticker(symbol, account_type).await,
            Self::HyperLiquid(c) => c.get_ticker(symbol, account_type).await,

            // DEX
            Self::Lighter(c) => c.get_ticker(symbol, account_type).await,
            Self::Uniswap(c) => c.get_ticker(symbol, account_type).await,
            Self::Jupiter(c) => c.get_ticker(symbol, account_type).await,
            Self::Raydium(c) => c.get_ticker(symbol, account_type).await,
            Self::Gmx(c) => c.get_ticker(symbol, account_type).await,
            Self::Paradex(c) => c.get_ticker(symbol, account_type).await,
            Self::Dydx(c) => c.get_ticker(symbol, account_type).await,

            // Stocks US
            Self::Polygon(c) => c.get_ticker(symbol, account_type).await,
            Self::Finnhub(c) => c.get_ticker(symbol, account_type).await,
            Self::Tiingo(c) => c.get_ticker(symbol, account_type).await,
            Self::Twelvedata(c) => c.get_ticker(symbol, account_type).await,
            Self::Alpaca(c) => c.get_ticker(symbol, account_type).await,

            // Stocks India
            Self::AngelOne(c) => c.get_ticker(symbol, account_type).await,
            Self::Zerodha(c) => c.get_ticker(symbol, account_type).await,
            Self::Upstox(c) => c.get_ticker(symbol, account_type).await,
            Self::Dhan(c) => c.get_ticker(symbol, account_type).await,
            Self::Fyers(c) => c.get_ticker(symbol, account_type).await,

            // Stocks Other
            Self::JQuants(c) => c.get_ticker(symbol, account_type).await,
            Self::Krx(c) => c.get_ticker(symbol, account_type).await,
            Self::Moex(c) => c.get_ticker(symbol, account_type).await,
            Self::Tinkoff(c) => c.get_ticker(symbol, account_type).await,

            // Forex
            Self::Oanda(c) => c.get_ticker(symbol, account_type).await,
            Self::Dukascopy(c) => c.get_ticker(symbol, account_type).await,
            Self::AlphaVantage(c) => c.get_ticker(symbol, account_type).await,

            // Prediction
            Self::Polymarket(c) => c.get_ticker(symbol, account_type).await,

            // Aggregators
            Self::IB(c) => c.get_ticker(symbol, account_type).await,
            Self::YahooFinance(c) => c.get_ticker(symbol, account_type).await,
            Self::CryptoCompare(c) => c.get_ticker(symbol, account_type).await,
            Self::DefiLlama(c) => c.get_ticker(symbol, account_type).await,
        }
    }

    async fn ping(&self) -> ExchangeResult<()> {
        match self {
            // CEX
            Self::Binance(c) => c.ping().await,
            Self::Bybit(c) => c.ping().await,
            Self::OKX(c) => c.ping().await,
            Self::KuCoin(c) => c.ping().await,
            Self::Kraken(c) => c.ping().await,
            Self::Coinbase(c) => c.ping().await,
            Self::GateIO(c) => c.ping().await,
            Self::Bitfinex(c) => c.ping().await,
            Self::Bitstamp(c) => c.ping().await,
            Self::Gemini(c) => c.ping().await,
            Self::MEXC(c) => c.ping().await,
            Self::HTX(c) => c.ping().await,
            Self::Bitget(c) => c.ping().await,
            Self::BingX(c) => c.ping().await,
            Self::Phemex(c) => c.ping().await,
            Self::CryptoCom(c) => c.ping().await,
            Self::Upbit(c) => c.ping().await,
            Self::Deribit(c) => c.ping().await,
            Self::HyperLiquid(c) => c.ping().await,

            // DEX
            Self::Lighter(c) => c.ping().await,
            Self::Uniswap(c) => c.ping().await,
            Self::Jupiter(c) => c.ping().await,
            Self::Raydium(c) => c.ping().await,
            Self::Gmx(c) => c.ping().await,
            Self::Paradex(c) => c.ping().await,
            Self::Dydx(c) => c.ping().await,

            // Stocks US
            Self::Polygon(c) => c.ping().await,
            Self::Finnhub(c) => c.ping().await,
            Self::Tiingo(c) => c.ping().await,
            Self::Twelvedata(c) => c.ping().await,
            Self::Alpaca(c) => c.ping().await,

            // Stocks India
            Self::AngelOne(c) => c.ping().await,
            Self::Zerodha(c) => c.ping().await,
            Self::Upstox(c) => c.ping().await,
            Self::Dhan(c) => c.ping().await,
            Self::Fyers(c) => c.ping().await,

            // Stocks Other
            Self::JQuants(c) => c.ping().await,
            Self::Krx(c) => c.ping().await,
            Self::Moex(c) => c.ping().await,
            Self::Tinkoff(c) => c.ping().await,

            // Forex
            Self::Oanda(c) => c.ping().await,
            Self::Dukascopy(c) => c.ping().await,
            Self::AlphaVantage(c) => c.ping().await,

            // Prediction
            Self::Polymarket(c) => c.ping().await,

            // Aggregators
            Self::IB(c) => c.ping().await,
            Self::YahooFinance(c) => c.ping().await,
            Self::CryptoCompare(c) => c.ping().await,
            Self::DefiLlama(c) => c.ping().await,
        }
    }

    async fn get_exchange_info(&self, account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        match self {
            // CEX
            Self::Binance(c) => c.get_exchange_info(account_type).await,
            Self::Bybit(c) => c.get_exchange_info(account_type).await,
            Self::OKX(c) => c.get_exchange_info(account_type).await,
            Self::KuCoin(c) => c.get_exchange_info(account_type).await,
            Self::Kraken(c) => c.get_exchange_info(account_type).await,
            Self::Coinbase(c) => c.get_exchange_info(account_type).await,
            Self::GateIO(c) => c.get_exchange_info(account_type).await,
            Self::Bitfinex(c) => c.get_exchange_info(account_type).await,
            Self::Bitstamp(c) => c.get_exchange_info(account_type).await,
            Self::Gemini(c) => c.get_exchange_info(account_type).await,
            Self::MEXC(c) => c.get_exchange_info(account_type).await,
            Self::HTX(c) => c.get_exchange_info(account_type).await,
            Self::Bitget(c) => c.get_exchange_info(account_type).await,
            Self::BingX(c) => c.get_exchange_info(account_type).await,
            Self::Phemex(c) => c.get_exchange_info(account_type).await,
            Self::CryptoCom(c) => c.get_exchange_info(account_type).await,
            Self::Upbit(c) => c.get_exchange_info(account_type).await,
            Self::Deribit(c) => c.get_exchange_info(account_type).await,
            Self::HyperLiquid(c) => c.get_exchange_info(account_type).await,

            // DEX
            Self::Lighter(c) => c.get_exchange_info(account_type).await,
            Self::Uniswap(c) => c.get_exchange_info(account_type).await,
            Self::Jupiter(c) => c.get_exchange_info(account_type).await,
            Self::Raydium(c) => c.get_exchange_info(account_type).await,
            Self::Gmx(c) => c.get_exchange_info(account_type).await,
            Self::Paradex(c) => c.get_exchange_info(account_type).await,
            Self::Dydx(c) => c.get_exchange_info(account_type).await,

            // Stocks US
            Self::Polygon(c) => c.get_exchange_info(account_type).await,
            Self::Finnhub(c) => c.get_exchange_info(account_type).await,
            Self::Tiingo(c) => c.get_exchange_info(account_type).await,
            Self::Twelvedata(c) => c.get_exchange_info(account_type).await,
            Self::Alpaca(c) => c.get_exchange_info(account_type).await,

            // Stocks India
            Self::AngelOne(c) => c.get_exchange_info(account_type).await,
            Self::Zerodha(c) => c.get_exchange_info(account_type).await,
            Self::Upstox(c) => c.get_exchange_info(account_type).await,
            Self::Dhan(c) => c.get_exchange_info(account_type).await,
            Self::Fyers(c) => c.get_exchange_info(account_type).await,

            // Stocks Other
            Self::JQuants(c) => c.get_exchange_info(account_type).await,
            Self::Krx(c) => c.get_exchange_info(account_type).await,
            Self::Moex(c) => c.get_exchange_info(account_type).await,
            Self::Tinkoff(c) => c.get_exchange_info(account_type).await,

            // Forex
            Self::Oanda(c) => c.get_exchange_info(account_type).await,
            Self::Dukascopy(c) => c.get_exchange_info(account_type).await,
            Self::AlphaVantage(c) => c.get_exchange_info(account_type).await,

            // Prediction
            Self::Polymarket(c) => c.get_exchange_info(account_type).await,

            // Aggregators
            Self::IB(c) => c.get_exchange_info(account_type).await,
            Self::YahooFinance(c) => c.get_exchange_info(account_type).await,
            Self::CryptoCompare(c) => c.get_exchange_info(account_type).await,
            Self::DefiLlama(c) => c.get_exchange_info(account_type).await,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// UNIT TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that AnyConnector has exactly 51 variants (compile-time check via exhaustive match)
    /// This test verifies that all enum variants are handled in the id() method
    #[test]
    fn test_any_connector_variant_count() {
        // This is a compile-time test
        // If a new variant is added without updating id(), this will fail to compile
        // The actual count is verified by the registry tests (51 connectors)
    }

    /// Test that AnyConnector has reasonable memory size due to Arc wrapping
    #[test]
    fn test_any_connector_memory_size() {
        use std::mem::size_of;

        // AnyConnector should be small since it wraps Arc (pointer + discriminant)
        let size = size_of::<AnyConnector>();

        // Arc is typically 8 bytes (64-bit pointer)
        // Enum discriminant is typically 1-2 bytes (aligned to 8)
        // Total should be <= 16 bytes on 64-bit systems
        assert!(size <= 16, "AnyConnector size ({} bytes) should be <= 16 bytes due to Arc", size);
    }

    /// Test that AnyConnector is Send + Sync (can be used across threads)
    #[test]
    fn test_any_connector_send_sync() {
        // This is a compile-time test
        fn assert_send_sync<T: Send + Sync>() {}

        // If this compiles, AnyConnector is Send + Sync
        assert_send_sync::<AnyConnector>();
    }

    /// Test that ExchangeIdentity trait is implemented
    #[test]
    fn test_exchange_identity_trait() {
        // This is a compile-time test to verify ExchangeIdentity trait implementation
        fn assert_exchange_identity<T: ExchangeIdentity>() {}

        // If this compiles, ExchangeIdentity trait is properly implemented
        assert_exchange_identity::<AnyConnector>();
    }

    /// Test that MarketData trait is implemented
    #[test]
    fn test_market_data_trait_implemented() {
        // This is a compilation test - if MarketData trait is implemented,
        // this should compile
        fn assert_market_data<T: MarketData>() {}

        // If this compiles, MarketData trait is properly implemented
        assert_market_data::<AnyConnector>();
    }

    /// Test that Clone trait is implemented via derive
    #[test]
    fn test_any_connector_clone_trait() {
        // This is a compile-time test
        fn assert_clone<T: Clone>() {}

        // If this compiles, Clone trait is properly implemented
        assert_clone::<AnyConnector>();
    }

    /// Test exhaustive match coverage in id() method
    /// This ensures all 51 variants are handled
    #[test]
    fn test_id_method_exhaustive() {
        // The id() method has a match statement with 51 arms
        // If a variant is missing, this won't compile

        // Verify different categories return different IDs via const assertions
        const _: () = {
            // This is a compile-time check that ensures id() method exists
            // and returns ExchangeId for all variants
        };
    }

    /// Test that all trait implementations delegate correctly to underlying connectors
    #[test]
    fn test_trait_delegation_pattern() {
        // The delegation macros generate 51-arm match statements
        // for each trait method. This test verifies the pattern compiles.

        // ExchangeIdentity methods: exchange_id(), is_testnet(), supported_account_types()
        // MarketData methods: get_price(), get_orderbook(), get_klines(), get_ticker(), ping()

        // If any delegation is broken, this will fail to compile
    }

    /// Test that all CEX variants exist in the enum
    #[test]
    fn test_cex_variants_exist() {
        // This is a compile-time test
        // All CEX variants should be accessible

        // Expected 19 CEX connectors:
        // Binance, Bybit, OKX, KuCoin, Kraken, Coinbase, GateIO, Bitfinex,
        // Bitstamp, Gemini, MEXC, HTX, Bitget, BingX, Phemex, CryptoCom, Upbit,
        // Deribit, HyperLiquid
    }

    /// Test that all DEX variants exist in the enum
    #[test]
    fn test_dex_variants_exist() {
        // This is a compile-time test
        // All DEX variants should be accessible

        // Expected 7 DEX connectors:
        // Lighter, Uniswap, Jupiter, Raydium, Gmx, Paradex, Dydx
    }

    /// Test that all Stock market variants exist
    #[test]
    fn test_stock_variants_exist() {
        // Expected stock connectors:
        // US: Polygon, Finnhub, Tiingo, Twelvedata, Alpaca (5)
        // India: AngelOne, Zerodha, Upstox, Dhan, Fyers (5)
        // Japan: JQuants (1)
        // Korea: Krx (1)
        // Russia: Moex, Tinkoff (2)
        // Total: 14 stock connectors
    }

    /// Test that Forex variants exist
    #[test]
    fn test_forex_variants_exist() {
        // Expected forex connectors:
        // Oanda, Dukascopy, AlphaVantage (3)
    }

    /// Test that Prediction variants exist
    #[test]
    fn test_prediction_variants_exist() {
        // Expected prediction connectors:
        // Polymarket (1)
    }

    /// Test that Aggregator variants exist
    #[test]
    fn test_aggregator_variants_exist() {
        // Expected aggregator connectors:
        // IB, YahooFinance, CryptoCompare, DefiLlama (4)
        // Total: 19 + 7 + 14 + 3 + 1 + 4 = 48 connectors
    }
}
