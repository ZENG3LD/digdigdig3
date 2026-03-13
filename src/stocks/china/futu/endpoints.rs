//! Futu OpenAPI connection parameters and protocol identifiers
//!
//! Futu does NOT use HTTP REST endpoints. Instead, it uses:
//! - TCP connection to OpenD gateway daemon
//! - Protocol Buffer messages
//! - Push-based subscriptions
//!
//! ## Protocol IDs reference
//!
//! | Proto ID | Name                  | Category     |
//! |----------|-----------------------|--------------|
//! | 1001     | InitConnect           | Connection   |
//! | 1002     | GetGlobalState        | Connection   |
//! | 1004     | KeepAlive             | Connection   |
//! | 2001     | Trd_GetAccList        | Account      |
//! | 2004     | Trd_UnlockTrade       | Account      |
//! | 2101     | Trd_GetFunds          | Account      |
//! | 2102     | Trd_GetPositionList   | Positions    |
//! | 2201     | Trd_GetOrderList      | Trading      |
//! | 2202     | Trd_PlaceOrder        | Trading      |
//! | 2205     | Trd_ModifyOrder       | Trading      |
//! | 2211     | Trd_GetOrderFillList  | Trading      |
//! | 2221     | Trd_GetHistOrderList  | Trading      |
//! | 2231     | Trd_GetHistOrderFill  | Trading      |
//! | 3001     | Qot_Sub               | Subscription |
//! | 3004     | Qot_GetStaticInfo     | Market Data  |
//! | 3005     | Qot_GetSecuritySnapshot| Market Data |
//! | 3006     | Qot_GetPlateSet       | Market Data  |
//! | 3012     | Qot_GetOrderBook      | Market Data  |
//! | 3100     | Qot_GetKL             | Market Data  |
//! | 3103     | Qot_RequestHistoryKL  | Market Data  |

use crate::core::types::Symbol;

// ═══════════════════════════════════════════════════════════════════════════════
// PROTOCOL IDs
// ═══════════════════════════════════════════════════════════════════════════════

/// Futu OpenD protocol identifiers for Protocol Buffer messages.
///
/// These are the top-level proto IDs used in Futu's TCP packet header.
pub mod proto_id {
    // --- Connection ---
    pub const INIT_CONNECT: u32 = 1001;
    pub const GET_GLOBAL_STATE: u32 = 1002;
    pub const KEEP_ALIVE: u32 = 1004;

    // --- Account ---
    /// Trd_GetAccList — list all trading accounts linked to OpenD
    pub const TRD_GET_ACC_LIST: u32 = 2001;
    /// Trd_UnlockTrade — unlock trading with trade password
    pub const TRD_UNLOCK_TRADE: u32 = 2004;
    /// Trd_GetFunds — get account funds (cash, securities value, buying power)
    pub const TRD_GET_FUNDS: u32 = 2101;

    // --- Positions ---
    /// Trd_GetPositionList — list open stock/futures positions
    pub const TRD_GET_POSITION_LIST: u32 = 2102;

    // --- Trading: Orders ---
    /// Trd_GetOrderList — get open orders
    pub const TRD_GET_ORDER_LIST: u32 = 2201;
    /// Trd_PlaceOrder — place a new order
    pub const TRD_PLACE_ORDER: u32 = 2202;
    /// Trd_ModifyOrder — amend or cancel an existing order
    pub const TRD_MODIFY_ORDER: u32 = 2205;
    /// Trd_GetOrderFillList — get recent order fills (deal list)
    pub const TRD_GET_ORDER_FILL_LIST: u32 = 2211;
    /// Trd_GetHistOrderList — get historical order list
    pub const TRD_GET_HIST_ORDER_LIST: u32 = 2221;
    /// Trd_GetHistOrderFillList — get historical fill list
    pub const TRD_GET_HIST_ORDER_FILL_LIST: u32 = 2231;

    // --- Subscriptions ---
    /// Qot_Sub — subscribe / unsubscribe to real-time data streams
    pub const QOT_SUB: u32 = 3001;

    // --- Market Data ---
    /// Qot_GetStaticInfo — static symbol metadata
    pub const QOT_GET_STATIC_INFO: u32 = 3004;
    /// Qot_GetSecuritySnapshot — real-time snapshot (ticker)
    pub const QOT_GET_SECURITY_SNAPSHOT: u32 = 3005;
    /// Qot_GetPlateSet — plate (sector) list
    pub const QOT_GET_PLATE_SET: u32 = 3006;
    /// Qot_GetOrderBook — order book snapshot
    pub const QOT_GET_ORDER_BOOK: u32 = 3012;
    /// Qot_GetKL — real-time klines (current day)
    pub const QOT_GET_KL: u32 = 3100;
    /// Qot_RequestHistoryKL — historical klines with date range
    pub const QOT_REQUEST_HISTORY_KL: u32 = 3103;
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING ENVIRONMENT
// ═══════════════════════════════════════════════════════════════════════════════

/// Futu trading environment (TrdEnv)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrdEnv {
    /// Real money (live trading)
    Real = 1,
    /// Paper trading / simulation
    Simulate = 0,
}

impl TrdEnv {
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING MARKET
// ═══════════════════════════════════════════════════════════════════════════════

/// Futu trading market (TrdMarket)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrdMarket {
    /// Hong Kong Exchange (HKEX)
    Hk = 1,
    /// US markets (NYSE, NASDAQ, AMEX)
    Us = 2,
    /// China A-shares Shanghai (SSE)
    CnSh = 3,
    /// China A-shares Shenzhen (SZSE)
    CnSz = 4,
    /// Singapore Exchange (SGX)
    Sg = 11,
}

impl TrdMarket {
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SECURITY MARKET
// ═══════════════════════════════════════════════════════════════════════════════

/// Futu security market code (QotMarket / SecMarket)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecMarket {
    Hk = 1,
    Us = 2,
    CnSh = 31,
    CnSz = 32,
    Sg = 41,
}

impl SecMarket {
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ORDER TYPE (Futu native)
// ═══════════════════════════════════════════════════════════════════════════════

/// Futu native order type enum (OrderType in Trd_Common.proto)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FutuOrderType {
    /// Normal limit order (also used for market orders with price=0)
    Normal = 1,
    /// Market order (special case — not all markets support)
    Market = 2,
    /// Enhanced limit (approximates stop market)
    EnhancedLimit = 3,
    /// Stop limit order
    StopLimit = 4,
    /// Stop market order
    StopMarket = 5,
    /// Auction order (HK-specific)
    Auction = 6,
    /// Special limit order (HK-specific, must fill all at once)
    SpecialLimit = 7,
}

impl FutuOrderType {
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ORDER SIDE (Futu native)
// ═══════════════════════════════════════════════════════════════════════════════

/// Futu native order side (TrdSide in Trd_Common.proto)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FutuTrdSide {
    Buy = 1,
    Sell = 2,
    /// Sell short (borrowing to sell)
    SellShort = 3,
    /// Buy back (covering short)
    BuyBack = 4,
}

impl FutuTrdSide {
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MODIFY ORDER OPERATION
// ═══════════════════════════════════════════════════════════════════════════════

/// Futu modify order operation (ModifyOrderOp in Trd_Common.proto)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifyOrderOp {
    /// Amend — modify price/qty of a live order
    Normal = 1,
    /// Cancel the order
    Cancel = 2,
    /// Disable (pause) the order
    Disable = 3,
    /// Re-enable a disabled order
    Enable = 4,
    /// Delete the order record
    Delete = 5,
}

impl ModifyOrderOp {
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ORDER STATUS (Futu native)
// ═══════════════════════════════════════════════════════════════════════════════

/// Futu order status codes (OrderStatus in Trd_Common.proto)
pub mod order_status {
    pub const UNSUBMITTED: i32 = 0;
    pub const UNKNOWN: i32 = 1;
    pub const WAITING_SUBMIT: i32 = 2;
    pub const SUBMITTING: i32 = 3;
    pub const SUBMIT_FAILED: i32 = 4;
    pub const TIMEOUT: i32 = 5;
    pub const SUBMITTED: i32 = 6;
    pub const FILLED_PART: i32 = 7;
    pub const FILLED_ALL: i32 = 8;
    pub const CANCELLING_PART: i32 = 9;
    pub const CANCELLING_ALL: i32 = 10;
    pub const CANCELLED_PART: i32 = 11;
    pub const CANCELLED_ALL: i32 = 12;
    pub const FAILED: i32 = 13;
    pub const DISABLED: i32 = 14;
    pub const DELETED: i32 = 15;
    pub const FILL_CANCELLED: i32 = 21;
}

// ═══════════════════════════════════════════════════════════════════════════════
// OPEND CONNECTION PARAMETERS
// ═══════════════════════════════════════════════════════════════════════════════

/// OpenD connection parameters
pub struct FutuEndpoints {
    /// OpenD host (usually 127.0.0.1 for local)
    pub host: String,
    /// OpenD port (default: 11111)
    pub port: u16,
    /// Enable encryption (required for remote OpenD)
    pub use_encryption: bool,
}

impl Default for FutuEndpoints {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 11111,
            use_encryption: false,
        }
    }
}

impl FutuEndpoints {
    /// Build the OpenD address string
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Format symbol for Futu API ("MARKET.CODE" format, e.g. "US.AAPL", "HK.00700")
pub fn format_symbol(symbol: &Symbol, market: SecMarket) -> String {
    let market_prefix = match market {
        SecMarket::Hk => "HK",
        SecMarket::Us => "US",
        SecMarket::CnSh => "SH",
        SecMarket::CnSz => "SZ",
        SecMarket::Sg => "SG",
    };
    format!("{}.{}", market_prefix, symbol.base.to_uppercase())
}

/// Parse a Futu-format code string into a Symbol
pub fn parse_symbol(futu_code: &str) -> Result<Symbol, String> {
    let parts: Vec<&str> = futu_code.splitn(2, '.').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid Futu code format: {}", futu_code));
    }
    let quote = match parts[0] {
        "HK" => "HKD",
        "US" => "USD",
        "SH" | "SZ" => "CNY",
        "SG" => "SGD",
        _ => "USD",
    };
    Ok(Symbol {
        base: parts[1].to_string(),
        quote: quote.to_string(),
        raw: Some(futu_code.to_string()),
    })
}

/// Infer Futu SecMarket from account type context (best-effort)
pub fn infer_sec_market(symbol: &Symbol) -> SecMarket {
    // HK stocks are typically 4-6 digit numbers
    if symbol.base.chars().all(|c| c.is_ascii_digit()) {
        return SecMarket::Hk;
    }
    // Default to US for alpha tickers
    SecMarket::Us
}
