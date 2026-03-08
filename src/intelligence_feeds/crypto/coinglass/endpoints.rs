//! # Coinglass Endpoints
//!
//! URL'ы и endpoint enum для Coinglass API V4.
//!
//! Coinglass is a derivatives analytics provider specializing in:
//! - Liquidations (real-time and historical)
//! - Open Interest (aggregated across exchanges)
//! - Funding Rates (current and historical)
//! - Long/Short Ratios (multiple calculation methods)

// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URL'ы для Coinglass API
#[derive(Debug, Clone)]
pub struct CoinglassUrls {
    pub rest: &'static str,
    pub ws: &'static str,
}

impl CoinglassUrls {
    /// Production URLs
    pub const MAINNET: Self = Self {
        rest: "https://open-api-v4.coinglass.com",
        ws: "wss://open-ws.coinglass.com/ws-api",
    };

    /// Get REST base URL
    pub fn rest_url(&self) -> &str {
        self.rest
    }

    /// Get WebSocket URL
    pub fn ws_url(&self) -> &str {
        self.ws
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENDPOINTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Coinglass API endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoinglassEndpoint {
    // === MARKET DISCOVERY ===
    SupportedCoins,
    SupportedExchangePairs,
    PairsMarkets,
    CoinsMarkets,

    // === LIQUIDATIONS ===
    LiquidationHistory,
    LiquidationHeatmap,
    LiquidationMap,
    LiquidationMaxPain,

    // === OPEN INTEREST ===
    OpenInterestOhlc,
    OpenInterestAggregated,
    OpenInterestHistory,
    OpenInterestVolRatio,
    OpenInterestByCoin,

    // === FUNDING RATES ===
    FundingRateHistory,
    FundingRateCurrent,
    FundingRateAggregated,

    // === LONG/SHORT RATIOS ===
    LongShortRateHistory,
    LongShortAccountRatio,
    LongShortGlobalAccountRatio,
    TopLongShortPositionRatio,
    TopLongShortAccountRatio,
    TakerBuySellVolume,

    // === ORDER BOOK ANALYTICS ===
    BidAskRange,
    OrderbookHeatmap,
    LargeOrders,

    // === VOLUME & FLOWS ===
    CumulativeVolumeDelta,
    NetFlowIndicator,
    FootprintChart,

    // === OPTIONS ===
    OptionsMaxPain,
    OptionsOiHistory,
    OptionsVolumeHistory,

    // === ON-CHAIN ===
    ExchangeReserve,
    ExchangeBalanceHistory,
    Erc20Transfers,
    WhaleTransfers,
    TokenUnlocks,
    TokenVesting,

    // === ETF ===
    BtcEtfFlow,
    EthEtfFlow,
    SolEtfFlow,
    XrpEtfFlow,
    HkEtfFlow,
    GrayscalePremium,

    // === HYPERLIQUID ===
    HyperLiquidWhaleAlert,
    HyperLiquidWhalePositions,
    HyperLiquidWalletPositions,
    HyperLiquidPositionDistribution,

    // === TECHNICAL INDICATORS ===
    Rsi,
    MovingAverage,
}

impl CoinglassEndpoint {
    /// Получить путь endpoint'а
    pub fn path(&self) -> &'static str {
        match self {
            // Market Discovery
            Self::SupportedCoins => "/api/futures/supported-coins",
            Self::SupportedExchangePairs => "/api/futures/supported-exchange-pairs",
            Self::PairsMarkets => "/api/futures/pairs-markets",
            Self::CoinsMarkets => "/api/futures/coins-markets",

            // Liquidations
            Self::LiquidationHistory => "/api/futures/liquidation/history",
            Self::LiquidationHeatmap => "/api/futures/liquidation/heatmap",
            Self::LiquidationMap => "/api/futures/liquidation/map",
            Self::LiquidationMaxPain => "/api/futures/liquidation/max-pain",

            // Open Interest
            Self::OpenInterestOhlc => "/api/futures/openInterest/ohlc-aggregated-history",
            Self::OpenInterestAggregated => "/api/futures/openInterest/ohlc-aggregated",
            Self::OpenInterestHistory => "/api/futures/openInterest/history",
            Self::OpenInterestVolRatio => "/api/futures/openInterest/vol-ratio",
            Self::OpenInterestByCoin => "/api/futures/openInterest/chart",

            // Funding Rates
            Self::FundingRateHistory => "/api/futures/funding/history",
            Self::FundingRateCurrent => "/api/futures/funding/rates",
            Self::FundingRateAggregated => "/api/futures/funding/ohlc",

            // Long/Short Ratios
            Self::LongShortRateHistory => "/api/futures/longShortRate/history",
            Self::LongShortAccountRatio => "/api/futures/longShort/accounts",
            Self::LongShortGlobalAccountRatio => "/api/futures/globalLongShortAccountRatio/chart",
            Self::TopLongShortPositionRatio => "/api/futures/topLongShortPositionRatio/chart",
            Self::TopLongShortAccountRatio => "/api/futures/topLongShortAccountRatio/chart",
            Self::TakerBuySellVolume => "/api/futures/takerBuySellVolume/chart",

            // Order Book Analytics
            Self::BidAskRange => "/api/futures/bid-ask-range",
            Self::OrderbookHeatmap => "/api/futures/orderbook/heatmap",
            Self::LargeOrders => "/api/futures/large-orders",

            // Volume & Flows
            Self::CumulativeVolumeDelta => "/api/futures/cvd/chart",
            Self::NetFlowIndicator => "/api/futures/net-flow",
            Self::FootprintChart => "/api/futures/footprint",

            // Options
            Self::OptionsMaxPain => "/api/options/max-pain",
            Self::OptionsOiHistory => "/api/options/oi-history",
            Self::OptionsVolumeHistory => "/api/options/volume-history",

            // On-Chain
            Self::ExchangeReserve => "/api/onchain/exchange-reserve",
            Self::ExchangeBalanceHistory => "/api/onchain/exchange-balance-history",
            Self::Erc20Transfers => "/api/onchain/erc20-transfers",
            Self::WhaleTransfers => "/api/onchain/whale-transfers",
            Self::TokenUnlocks => "/api/onchain/token-unlocks",
            Self::TokenVesting => "/api/onchain/token-vesting",

            // ETF
            Self::BtcEtfFlow => "/api/etf/btc-flow",
            Self::EthEtfFlow => "/api/etf/eth-flow",
            Self::SolEtfFlow => "/api/etf/sol-flow",
            Self::XrpEtfFlow => "/api/etf/xrp-flow",
            Self::HkEtfFlow => "/api/etf/hk-flow",
            Self::GrayscalePremium => "/api/etf/grayscale-premium",

            // Hyperliquid
            Self::HyperLiquidWhaleAlert => "/api/hyperliquid/whale-alert",
            Self::HyperLiquidWhalePositions => "/api/hyperliquid/whale-positions",
            Self::HyperLiquidWalletPositions => "/api/hyperliquid/wallet-positions",
            Self::HyperLiquidPositionDistribution => "/api/hyperliquid/position-distribution",

            // Technical Indicators
            Self::Rsi => "/api/indicator/rsi",
            Self::MovingAverage => "/api/indicator/ma",
        }
    }

    /// Требует ли endpoint авторизации (все endpoints требуют API key)
    pub fn requires_auth(&self) -> bool {
        true // All Coinglass endpoints require CG-API-KEY
    }

    /// HTTP метод для endpoint'а
    pub fn method(&self) -> &'static str {
        "GET" // All Coinglass endpoints use GET
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Форматирование символа для Coinglass
///
/// Coinglass uses simple crypto symbols (not pairs):
/// - "BTC", "ETH", "SOL", etc.
/// - No quote currency (USDT/USD is implied for derivatives)
///
/// # Examples
/// - Input: ("BTC", "USDT") → Output: "BTC"
/// - Input: ("ETH", "USD") → Output: "ETH"
pub fn _format_symbol(base: &str, _quote: &str) -> String {
    base.to_uppercase()
}

/// Map time interval to Coinglass interval format
///
/// Coinglass supports various intervals for historical data:
/// - "1m", "5m", "15m", "30m" (minutes)
/// - "1h", "2h", "4h", "12h" (hours)
/// - "1d" (days)
///
/// # Examples
/// - "1m" → "1m"
/// - "1h" → "1h"
/// - "1d" → "1d"
pub fn _map_interval(interval: &str) -> &'static str {
    match interval {
        "1m" => "1m",
        "5m" => "5m",
        "15m" => "15m",
        "30m" => "30m",
        "1h" => "1h",
        "2h" => "2h",
        "4h" => "4h",
        "12h" => "12h",
        "1d" => "1d",
        _ => "1h", // default to 1 hour
    }
}
