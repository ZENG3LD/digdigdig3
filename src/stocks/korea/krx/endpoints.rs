//! KRX API endpoints and URL configuration

/// Base URLs for KRX APIs
pub struct KrxEndpoints {
    /// Open API base URL (new authentication-required API)
    pub openapi_base: &'static str,
    /// Public Data Portal API (government, still works with serviceKey)
    pub public_data_portal: &'static str,
}

impl Default for KrxEndpoints {
    fn default() -> Self {
        Self {
            openapi_base: "https://data-dbg.krx.co.kr",
            public_data_portal: "https://apis.data.go.kr/1160100/service/GetKrxListedInfoService/getItemInfo",
        }
    }
}

/// KRX Open API endpoints (path-based system)
#[derive(Debug, Clone)]
pub enum KrxEndpoint {
    /// KOSPI daily trading data
    KospiDailyTrading,
    /// KOSPI stock base information
    KospiBaseInfo,
    /// KOSDAQ daily trading data
    KosdaqDailyTrading,
    /// KOSDAQ stock base information
    KosdaqBaseInfo,
    /// KONEX daily trading data
    KonexDailyTrading,
    /// KONEX stock base information
    KonexBaseInfo,
    /// Warrant daily trading data
    WarrantDailyTrading,
    /// Subscription warrant daily trading data
    SubscriptionWarrantDailyTrading,
    /// Market index daily trading data
    IndexDailyTrading,
}

impl KrxEndpoint {
    /// Get endpoint path for Open API
    pub fn path(&self) -> &'static str {
        match self {
            Self::KospiDailyTrading => "/svc/apis/sto/stk_bydd_trd.json",
            Self::KospiBaseInfo => "/svc/apis/sto/stk_isu_base_info.json",
            Self::KosdaqDailyTrading => "/svc/apis/sto/ksq_bydd_trd.json",
            Self::KosdaqBaseInfo => "/svc/apis/sto/ksq_isu_base_info.json",
            Self::KonexDailyTrading => "/svc/apis/sto/knx_bydd_trd.json",
            Self::KonexBaseInfo => "/svc/apis/sto/knx_isu_base_info.json",
            Self::WarrantDailyTrading => "/svc/apis/sto/sw_bydd_trd.json",
            Self::SubscriptionWarrantDailyTrading => "/svc/apis/sto/sr_bydd_trd.json",
            Self::IndexDailyTrading => "/svc/apis/idx/idx_bydd_trd.json",
        }
    }
}

/// Format symbol for KRX API
///
/// KRX uses different formats:
/// - Short code (ticker): "005930" (Samsung Electronics)
/// - ISIN code: "KR7005930003" (full 12-character ISIN)
///
/// For stock data, we use short code format (6 digits).
/// The Symbol struct from core types is crypto-centric (base/quote),
/// so for stocks we only use the base field.
pub fn _format_symbol(symbol: &crate::core::types::Symbol) -> String {
    // For KRX stocks, use only the base (ticker code)
    // Quote is not applicable to stocks
    symbol.base.to_uppercase()
}

/// Format ISIN code for KRX API
///
/// Convert short code (6 digits) to full ISIN (12 characters)
/// Format: KR7 + short_code + check_digit (3 chars)
///
/// Note: This is a simplified conversion. For production use,
/// proper ISIN validation and check digit calculation may be needed.
pub fn _format_isin(short_code: &str) -> String {
    if short_code.starts_with("KR") {
        // Already an ISIN
        short_code.to_string()
    } else {
        // Convert short code to ISIN
        // KR7 prefix + 6-digit code + 3-digit suffix
        format!("KR7{}003", short_code)
    }
}

/// Parse symbol from KRX API format back to domain Symbol
///
/// KRX returns short codes like "005930"
/// We convert to Symbol with ticker as base and empty quote
pub fn _parse_symbol(ticker: &str) -> crate::core::types::Symbol {
    crate::core::types::Symbol::new(ticker, "")
}

/// Market ID codes for KRX
#[derive(Debug, Clone, Copy)]
pub enum MarketId {
    /// KOSPI (Korea Stock Exchange)
    Kospi,
    /// KOSDAQ (Korean Securities Dealers Automated Quotations)
    Kosdaq,
    /// KONEX (Korea New Exchange)
    Konex,
    /// All markets
    All,
}

impl MarketId {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Kospi => "STK",
            Self::Kosdaq => "KSQ",
            Self::Konex => "KNX",
            Self::All => "ALL",
        }
    }

    /// Get corresponding endpoint for daily trading data
    pub fn daily_trading_endpoint(&self) -> KrxEndpoint {
        match self {
            Self::Kospi => KrxEndpoint::KospiDailyTrading,
            Self::Kosdaq => KrxEndpoint::KosdaqDailyTrading,
            Self::Konex => KrxEndpoint::KonexDailyTrading,
            Self::All => KrxEndpoint::KospiDailyTrading, // Default to KOSPI for "All"
        }
    }
}

/// Format date for KRX API
///
/// KRX expects YYYYMMDD format (e.g., "20260120")
pub fn format_date(year: i32, month: u32, day: u32) -> String {
    format!("{:04}{:02}{:02}", year, month, day)
}

/// Format current date for KRX API
#[cfg(not(target_arch = "wasm32"))]
pub fn format_today() -> String {
    use chrono::{Local, Datelike};
    let now = Local::now();
    format!("{:04}{:02}{:02}", now.year(), now.month(), now.day())
}
