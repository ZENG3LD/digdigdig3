//! Alpha Vantage API endpoints

/// Base URLs for Alpha Vantage API
pub struct AlphaVantageEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for AlphaVantageEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://www.alphavantage.co/query",
            ws_base: None, // Alpha Vantage does not support WebSocket
        }
    }
}

/// Alpha Vantage API endpoint enum
#[derive(Debug, Clone)]
pub enum AlphaVantageEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // STOCK/EQUITY ENDPOINTS (6)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get real-time quote for a stock
    GlobalQuote,
    /// Get intraday time series (1min, 5min, 15min, 30min, 60min)
    TimeSeriesIntraday,
    /// Get daily time series
    TimeSeriesDaily,
    /// Get weekly time series
    TimeSeriesWeekly,
    /// Get monthly time series
    TimeSeriesMonthly,
    /// Search for symbols by keywords
    SymbolSearch,

    // ═══════════════════════════════════════════════════════════════════════
    // FOREX ENDPOINTS (2)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get real-time forex exchange rate
    CurrencyExchangeRate,
    /// Get daily forex time series
    FxDaily,

    // ═══════════════════════════════════════════════════════════════════════
    // CRYPTO ENDPOINTS (2)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get crypto rating/health score
    CryptoRating,
    /// Get daily crypto time series
    DigitalCurrencyDaily,

    // ═══════════════════════════════════════════════════════════════════════
    // ECONOMIC INDICATORS ENDPOINTS (9)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get real GDP data
    RealGdp,
    /// Get real GDP per capita
    RealGdpPerCapita,
    /// Get Treasury yield rates
    TreasuryYield,
    /// Get federal funds rate
    FederalFundsRate,
    /// Get Consumer Price Index
    Cpi,
    /// Get inflation rate
    Inflation,
    /// Get retail sales
    RetailSales,
    /// Get unemployment rate
    Unemployment,
    /// Get nonfarm payroll
    NonfarmPayroll,

    // ═══════════════════════════════════════════════════════════════════════
    // TECHNICAL INDICATORS ENDPOINTS (4)
    // ═══════════════════════════════════════════════════════════════════════
    /// Simple Moving Average
    Sma,
    /// Exponential Moving Average
    Ema,
    /// Relative Strength Index
    Rsi,
    /// Moving Average Convergence Divergence
    Macd,

    // ═══════════════════════════════════════════════════════════════════════
    // COMMODITIES ENDPOINTS (4)
    // ═══════════════════════════════════════════════════════════════════════
    /// WTI crude oil prices
    Wti,
    /// Brent crude oil prices
    Brent,
    /// Natural gas prices
    NaturalGas,
    /// Copper prices
    Copper,
}

impl AlphaVantageEndpoint {
    /// Get function name for Alpha Vantage API
    ///
    /// Alpha Vantage uses a single endpoint with `function` parameter
    pub fn function(&self) -> &'static str {
        match self {
            // Stock/Equity
            Self::GlobalQuote => "GLOBAL_QUOTE",
            Self::TimeSeriesIntraday => "TIME_SERIES_INTRADAY",
            Self::TimeSeriesDaily => "TIME_SERIES_DAILY",
            Self::TimeSeriesWeekly => "TIME_SERIES_WEEKLY",
            Self::TimeSeriesMonthly => "TIME_SERIES_MONTHLY",
            Self::SymbolSearch => "SYMBOL_SEARCH",

            // Forex
            Self::CurrencyExchangeRate => "CURRENCY_EXCHANGE_RATE",
            Self::FxDaily => "FX_DAILY",

            // Crypto
            Self::CryptoRating => "CRYPTO_RATING",
            Self::DigitalCurrencyDaily => "DIGITAL_CURRENCY_DAILY",

            // Economic Indicators
            Self::RealGdp => "REAL_GDP",
            Self::RealGdpPerCapita => "REAL_GDP_PER_CAPITA",
            Self::TreasuryYield => "TREASURY_YIELD",
            Self::FederalFundsRate => "FEDERAL_FUNDS_RATE",
            Self::Cpi => "CPI",
            Self::Inflation => "INFLATION",
            Self::RetailSales => "RETAIL_SALES",
            Self::Unemployment => "UNEMPLOYMENT",
            Self::NonfarmPayroll => "NONFARM_PAYROLL",

            // Technical Indicators
            Self::Sma => "SMA",
            Self::Ema => "EMA",
            Self::Rsi => "RSI",
            Self::Macd => "MACD",

            // Commodities
            Self::Wti => "WTI",
            Self::Brent => "BRENT",
            Self::NaturalGas => "NATURAL_GAS",
            Self::Copper => "COPPER",
        }
    }
}

/// Format symbol for Alpha Vantage API
///
/// Alpha Vantage uses plain symbol strings like "IBM", "AAPL", "MSFT"
pub fn format_symbol(symbol: &crate::core::types::Symbol) -> String {
    symbol.base.to_uppercase()
}

/// Parse symbol from Alpha Vantage response to domain Symbol
pub fn parse_symbol(symbol_str: &str) -> crate::core::types::Symbol {
    crate::core::types::Symbol::new(symbol_str, "")
}
