//! # Common Types
//!
//! Базовые типы для V5 коннекторов.

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTIFICATION
// ═══════════════════════════════════════════════════════════════════════════════

/// Идентификатор биржи
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExchangeId {
    // CEX - Global
    Binance,
    Bybit,
    OKX,
    KuCoin,
    Kraken,
    Coinbase,
    GateIO,
    Bitfinex,
    Bitstamp,
    Gemini,
    MEXC,
    HTX,
    Bitget,
    BingX,
    CryptoCom,
    // Bithumb,  // DISABLED: Infrastructure issues (see exchanges/mod.rs)
    Upbit,

    // Derivatives
    Deribit,
    HyperLiquid,

    // DEX
    Lighter,
    Paradex,
    Dydx,

    // Prediction Markets
    Polymarket,     // Prediction market (probability-based trading on real-world events)

    // Data Providers
    Polygon,        // US stocks data provider (Massive.com)
    Finnhub,        // US/Global stocks data provider
    Tiingo,         // Multi-asset data provider (stocks, crypto, forex, fundamentals, news)
    Twelvedata,     // Multi-asset data provider (stocks, forex, crypto, ETFs, commodities, 100+ indicators)
    Coinglass,      // Derivatives analytics provider (liquidations, OI, funding rates)
    CryptoCompare,  // Crypto aggregator (5,700+ coins, 170+ exchanges, CCCAGG index)
    WhaleAlert,     // Blockchain transaction tracker (whale movements, on-chain analytics, 11+ blockchains)
    Bitquery,       // Blockchain data provider (GraphQL API, on-chain analytics)

    // DeFi Aggregators
    DefiLlama,   // DeFi TVL aggregator (protocols, prices, yields)

    // Forex Brokers & Data Providers
    Oanda,       // Forex broker (data + trading)
    AlphaVantage, // Multi-asset data provider (forex, stocks, crypto, commodities, economic data)
    Dukascopy,   // Forex data provider (historical tick data from 2003+, Swiss bank)

    // Stock Brokers & Data Providers
    AngelOne,    // Angel One (Angel Broking) - Indian stock broker (NSE, BSE, MCX, CDS) - FREE API, full broker with F&O, TOTP auth
    Zerodha,     // Indian stock broker (NSE, BSE, NFO, BFO, MCX, CDS, BCD)
    Fyers,       // Indian stock broker (NSE, BSE, MCX, NCDEX) - FREE API, F&O specialization, 100k req/day
    Dhan,        // Indian stock broker (NSE, BSE, MCX) - 200-level depth, free trading APIs
    Upstox,      // Indian stock broker (NSE, BSE, MCX) - OAuth 2.0, Rs 499/mo subscription, HFT endpoints
    Alpaca,      // US stock broker (NYSE, NASDAQ, options, crypto) - commission-free, paper trading
    JQuants,     // Japan stock data provider (Tokyo Stock Exchange official data, JPX)
    Tinkoff,     // Russian stock broker (MOEX) - FREE API, stocks, bonds, ETFs, futures, options
    Moex,        // Moscow Exchange (MOEX) ISS API - FREE delayed data, real-time with subscription, Russia's largest exchange
    Krx,         // Korea Exchange data provider (KOSPI, KOSDAQ, KONEX) - FREE API with approval, daily data
    Futu,        // Futu Securities (HK, US, CN stocks) - TCP + Protocol Buffers via OpenD gateway

    // Economic Data Feeds
    Fred,        // Federal Reserve Economic Data (FRED) - 840,000+ economic time series, free API
    Bls,         // Bureau of Labor Statistics (BLS) - US labor market & economic indicators, optional API key

    // Multi-Asset Aggregators
    YahooFinance, // Yahoo Finance data aggregator (stocks, crypto, forex, options, fundamentals) - unofficial API
    Ib,           // Interactive Brokers (multi-asset broker, stocks, forex, futures, options)

    // Other
    Custom(u16),
}

impl ExchangeId {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Binance => "binance",
            Self::Bybit => "bybit",
            Self::OKX => "okx",
            Self::KuCoin => "kucoin",
            Self::Kraken => "kraken",
            Self::Coinbase => "coinbase",
            Self::GateIO => "gateio",
            Self::Bitfinex => "bitfinex",
            Self::Bitstamp => "bitstamp",
            Self::Gemini => "gemini",
            Self::MEXC => "mexc",
            Self::HTX => "htx",
            Self::Bitget => "bitget",
            Self::BingX => "bingx",
            Self::CryptoCom => "crypto_com",
            // Self::Bithumb => "bithumb",  // DISABLED
            Self::Upbit => "upbit",
            Self::Deribit => "deribit",
            Self::HyperLiquid => "hyperliquid",
            Self::Lighter => "lighter",
            Self::Paradex => "paradex",
            Self::Dydx => "dydx",
            Self::Polymarket => "polymarket",
            Self::Polygon => "polygon",
            Self::Finnhub => "finnhub",
            Self::Tiingo => "tiingo",
            Self::Twelvedata => "twelvedata",
            Self::Coinglass => "coinglass",
            Self::CryptoCompare => "cryptocompare",
            Self::WhaleAlert => "whale_alert",
            Self::Bitquery => "bitquery",
            Self::DefiLlama => "defillama",
            Self::Oanda => "oanda",
            Self::AlphaVantage => "alphavantage",
            Self::Dukascopy => "dukascopy",
            Self::AngelOne => "angel_one",
            Self::Zerodha => "zerodha",
            Self::Fyers => "fyers",
            Self::Dhan => "dhan",
            Self::Upstox => "upstox",
            Self::Alpaca => "alpaca",
            Self::JQuants => "jquants",
            Self::Tinkoff => "tinkoff",
            Self::Moex => "moex",
            Self::Krx => "krx",
            Self::Futu => "futu",
            Self::Fred => "fred",
            Self::Bls => "bls",
            Self::YahooFinance => "yahoo_finance",
            Self::Ib => "ib",
            Self::Custom(_) => "custom",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "binance" => Some(Self::Binance),
            "bybit" => Some(Self::Bybit),
            "okx" => Some(Self::OKX),
            "kucoin" => Some(Self::KuCoin),
            "kraken" => Some(Self::Kraken),
            "coinbase" => Some(Self::Coinbase),
            "gateio" => Some(Self::GateIO),
            "bitfinex" => Some(Self::Bitfinex),
            "bitstamp" => Some(Self::Bitstamp),
            "gemini" => Some(Self::Gemini),
            "mexc" => Some(Self::MEXC),
            "htx" => Some(Self::HTX),
            "bitget" => Some(Self::Bitget),
            "bingx" => Some(Self::BingX),
            "crypto_com" => Some(Self::CryptoCom),
            "upbit" => Some(Self::Upbit),
            "deribit" => Some(Self::Deribit),
            "hyperliquid" => Some(Self::HyperLiquid),
            "lighter" => Some(Self::Lighter),
            "paradex" => Some(Self::Paradex),
            "dydx" => Some(Self::Dydx),
            "polymarket" => Some(Self::Polymarket),
            "polygon" => Some(Self::Polygon),
            "finnhub" => Some(Self::Finnhub),
            "tiingo" => Some(Self::Tiingo),
            "twelvedata" => Some(Self::Twelvedata),
            "coinglass" => Some(Self::Coinglass),
            "cryptocompare" => Some(Self::CryptoCompare),
            "whale_alert" => Some(Self::WhaleAlert),
            "bitquery" => Some(Self::Bitquery),
            "defillama" => Some(Self::DefiLlama),
            "oanda" => Some(Self::Oanda),
            "alphavantage" => Some(Self::AlphaVantage),
            "dukascopy" => Some(Self::Dukascopy),
            "angel_one" => Some(Self::AngelOne),
            "zerodha" => Some(Self::Zerodha),
            "fyers" => Some(Self::Fyers),
            "dhan" => Some(Self::Dhan),
            "upstox" => Some(Self::Upstox),
            "alpaca" => Some(Self::Alpaca),
            "jquants" => Some(Self::JQuants),
            "tinkoff" => Some(Self::Tinkoff),
            "moex" => Some(Self::Moex),
            "krx" => Some(Self::Krx),
            "futu" => Some(Self::Futu),
            "fred" => Some(Self::Fred),
            "bls" => Some(Self::Bls),
            "yahoo_finance" => Some(Self::YahooFinance),
            "ib" => Some(Self::Ib),
            _ => None,
        }
    }

    pub fn exchange_type(&self) -> ExchangeType {
        match self {
            Self::HyperLiquid => ExchangeType::Hybrid,
            Self::Lighter | Self::Paradex | Self::Dydx => ExchangeType::Dex,
            Self::Polymarket | Self::Polygon | Self::Finnhub | Self::Tiingo | Self::Twelvedata | Self::Coinglass | Self::CryptoCompare | Self::WhaleAlert | Self::Bitquery | Self::DefiLlama | Self::Dukascopy | Self::JQuants | Self::Krx | Self::Fred | Self::Bls | Self::YahooFinance => ExchangeType::DataProvider,
            Self::Alpaca | Self::Oanda | Self::AngelOne | Self::Zerodha | Self::Fyers | Self::Dhan | Self::Upstox | Self::Tinkoff | Self::AlphaVantage | Self::Moex | Self::Ib | Self::Futu => ExchangeType::Cex, // Brokers/providers with trading capabilities
            Self::Custom(_) => ExchangeType::Cex, // default
            _ => ExchangeType::Cex,
        }
    }
}

/// Тип биржи
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExchangeType {
    /// Централизованная биржа
    Cex,
    /// Децентрализованная биржа
    Dex,
    /// Гибрид (например HyperLiquid)
    Hybrid,
    /// Брокер (традиционные рынки с полной брокерской функциональностью)
    Broker,
    /// Провайдер данных (не торговая биржа)
    DataProvider,
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/// Тип аккаунта/рынка
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AccountType {
    /// Спотовая торговля
    #[default]
    Spot,
    /// Маржинальная торговля
    Margin,
    /// Фьючерсы с кросс-маржой
    FuturesCross,
    /// Фьючерсы с изолированной маржой
    FuturesIsolated,
    /// Earn / savings / staking account.
    ///
    /// ~8/24: Binance, Bybit, OKX, KuCoin, GateIO, HTX, MEXC, Bitget.
    Earn,
    /// Lending / margin lending account.
    ///
    /// ~6/24: Binance, Bybit, OKX, KuCoin, GateIO, Bitfinex.
    Lending,
    /// Options account.
    ///
    /// ~5/24: Binance, Bybit, OKX, Deribit, KuCoin.
    Options,
    /// Convert / swap sub-account (instant conversion without order book).
    ///
    /// ~7/24: Binance, Bybit, OKX, KuCoin, GateIO, HTX, CryptoCom.
    Convert,
}

impl AccountType {
    /// Short display label for UI use.
    pub fn short_label(&self) -> &'static str {
        match self {
            Self::Spot => "S",
            Self::Margin => "M",
            Self::FuturesCross => "F",
            Self::FuturesIsolated => "FI",
            Self::Earn => "E",
            Self::Lending => "L",
            Self::Options => "O",
            Self::Convert => "CV",
        }
    }

    /// Stable lowercase key string for serialization keys and map lookups.
    pub fn as_key_str(&self) -> &'static str {
        match self {
            Self::Spot => "spot",
            Self::Margin => "margin",
            Self::FuturesCross => "futures_cross",
            Self::FuturesIsolated => "futures_isolated",
            Self::Earn => "earn",
            Self::Lending => "lending",
            Self::Options => "options",
            Self::Convert => "convert",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL
// ═══════════════════════════════════════════════════════════════════════════════

/// Торговая пара
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Symbol {
    /// Базовый актив (например BTC)
    pub base: String,
    /// Котируемый актив (например USDT)
    pub quote: String,
    /// Original raw symbol string from the exchange, if available.
    /// Connectors should prefer this over reconstructing from base/quote.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub raw: Option<String>,
}

impl Symbol {
    pub fn new(base: impl Into<String>, quote: impl Into<String>) -> Self {
        Self {
            base: base.into().to_uppercase(),
            quote: quote.into().to_uppercase(),
            raw: None,
        }
    }

    /// Construct a Symbol preserving the original raw exchange string.
    pub fn with_raw(base: &str, quote: &str, raw: String) -> Self {
        Self {
            base: base.to_uppercase(),
            quote: quote.to_uppercase(),
            raw: Some(raw),
        }
    }

    /// Get the raw symbol string if available.
    pub fn raw(&self) -> Option<&str> {
        self.raw.as_deref()
    }

    /// Пустой символ (для private streams без конкретного символа)
    pub fn empty() -> Self {
        Self {
            base: String::new(),
            quote: String::new(),
            raw: None,
        }
    }

    /// Проверить, пустой ли символ
    pub fn is_empty(&self) -> bool {
        self.base.is_empty() && self.quote.is_empty()
    }

    /// Форматировать как "BTCUSDT"
    pub fn to_concat(&self) -> String {
        format!("{}{}", self.base, self.quote)
    }

    /// Форматировать как "BTC-USDT"
    pub fn to_dash(&self) -> String {
        format!("{}-{}", self.base, self.quote)
    }

    /// Форматировать как "BTC_USDT"
    pub fn to_underscore(&self) -> String {
        format!("{}_{}", self.base, self.quote)
    }

    /// Распарсить из строки (пытается разные форматы)
    pub fn parse(s: &str) -> Option<Self> {
        // Попробовать разные разделители
        if let Some((base, quote)) = s.split_once('-') {
            return Some(Self::new(base, quote));
        }
        if let Some((base, quote)) = s.split_once('_') {
            return Some(Self::new(base, quote));
        }
        if let Some((base, quote)) = s.split_once('/') {
            return Some(Self::new(base, quote));
        }
        // Для формата BTCUSDT нужно знать где разделять
        None
    }
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.base, self.quote)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ERRORS
// ═══════════════════════════════════════════════════════════════════════════════

/// Ошибки Exchange операций
#[derive(Debug, Error)]
pub enum ExchangeError {
    #[error("HTTP error: {0}")]
    Http(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("API error: {code} - {message}")]
    Api { code: i32, message: String },

    #[error("Rate limit exceeded")]
    RateLimit,

    #[error("Rate limit exceeded: {message}")]
    RateLimitExceeded { retry_after: Option<u64>, message: String },

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Invalid credentials: {0}")]
    InvalidCredentials(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Not supported: {0}")]
    NotSupported(String),

    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Not found: {0}")]
    NotFound(String),
}

/// Результат Exchange операции
pub type ExchangeResult<T> = Result<T, ExchangeError>;

/// Ошибки WebSocket
#[derive(Debug, Clone, Error)]
pub enum WebSocketError {
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Not connected")]
    NotConnected,

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Subscription error: {0}")]
    Subscription(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Send error: {0}")]
    SendError(String),

    #[error("Receive error: {0}")]
    ReceiveError(String),

    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    #[error("Timeout")]
    Timeout,
}

/// Результат WebSocket операции
pub type WebSocketResult<T> = Result<T, WebSocketError>;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR METRICS
// ═══════════════════════════════════════════════════════════════════════════════

/// Runtime metrics snapshot for a connector instance.
///
/// Returned by `ExchangeIdentity::metrics()`. Provides at-a-glance visibility
/// into HTTP activity and rate-limiter utilization.
#[derive(Debug, Clone, Default)]
pub struct ConnectorStats {
    /// Total number of HTTP requests attempted (including retries)
    pub http_requests: u64,
    /// Total number of HTTP errors (network errors, non-2xx responses)
    pub http_errors: u64,
    /// Latency of the most recently completed request in milliseconds
    pub last_latency_ms: u64,
    /// Current consumed rate-limiter weight / request count
    pub rate_used: u32,
    /// Maximum rate-limiter weight / request count per window
    pub rate_max: u32,
    /// Per-group rate limit stats (name, used, max). Empty for single-limiter connectors.
    pub rate_groups: Vec<(String, u32, u32)>,
    /// WebSocket ping round-trip time in milliseconds (0 = not measured yet).
    pub ws_ping_rtt_ms: u64,
}
