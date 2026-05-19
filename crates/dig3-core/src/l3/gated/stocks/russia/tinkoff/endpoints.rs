//! Tinkoff Invest API endpoints
//!
//! REST proxy format:
//! `https://invest-public-api.tbank.ru/rest/tinkoff.public.invest.api.contract.v1.{ServiceName}/{MethodName}`

/// Base URLs for Tinkoff Invest API
pub struct TinkoffEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for TinkoffEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://invest-public-api.tbank.ru/rest",
            ws_base: Some("wss://invest-public-api.tinkoff.ru/ws/"),
        }
    }
}

impl TinkoffEndpoints {
    /// Create endpoints for sandbox environment
    pub fn sandbox() -> Self {
        Self {
            rest_base: "https://sandbox-invest-public-api.tinkoff.ru/rest",
            ws_base: Some("wss://sandbox-invest-public-api.tinkoff.ru/ws/"),
        }
    }
}

/// API endpoint enum for Tinkoff Invest
#[derive(Debug, Clone)]
pub enum TinkoffEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // MarketDataService - Market data retrieval
    // ═══════════════════════════════════════════════════════════════════════
    /// Get historical OHLC candles
    GetCandles,
    /// Get last trade prices for instruments
    GetLastPrices,
    /// Get order book snapshot (L2)
    GetOrderBook,
    /// Get single instrument trading status
    GetTradingStatus,
    /// Get batch trading status
    GetTradingStatuses,
    /// Get anonymous trades (last hour)
    GetLastTrades,
    /// Get session closing prices
    GetClosePrices,

    // ═══════════════════════════════════════════════════════════════════════
    // InstrumentsService - Instrument information and metadata
    // ═══════════════════════════════════════════════════════════════════════
    /// Get trading schedules
    TradingSchedules,
    /// Get single bond by identifier
    BondBy,
    /// List all bonds
    Bonds,
    /// Get bond coupon payment schedule
    GetBondCoupons,
    /// Get single currency by identifier
    CurrencyBy,
    /// List all currencies
    Currencies,
    /// Get single ETF by identifier
    EtfBy,
    /// List all ETFs
    Etfs,
    /// Get single futures contract
    FutureBy,
    /// List all futures
    Futures,
    /// Get single option by identifier
    OptionBy,
    /// Get options filtered by underlying
    OptionsBy,
    /// Get single stock by identifier
    ShareBy,
    /// List all stocks
    Shares,
    /// Get accrued coupon income for bonds
    GetAccruedInterests,
    /// Get margin requirements for futures
    GetFuturesMargin,
    /// Generic instrument lookup
    GetInstrumentBy,
    /// Get dividend payment events
    GetDividends,
    /// Get single asset by UID
    GetAssetBy,
    /// List assets
    GetAssets,
    /// Get user's favorite instruments
    GetFavorites,
    /// Add/remove favorites
    EditFavorites,
    /// Get country reference data
    GetCountries,
    /// Search instruments
    FindInstrument,
    /// List brands/companies
    GetBrands,
    /// Get single brand by UID
    GetBrandBy,

    // ═══════════════════════════════════════════════════════════════════════
    // OrdersService - Order placement and management
    // ═══════════════════════════════════════════════════════════════════════
    /// Place market/limit order
    PostOrder,
    /// Cancel active order
    CancelOrder,
    /// Get single order status
    GetOrderState,
    /// List active orders
    GetOrders,
    /// Modify existing order
    ReplaceOrder,

    // ═══════════════════════════════════════════════════════════════════════
    // StopOrdersService - Stop orders (conditional orders)
    // ═══════════════════════════════════════════════════════════════════════
    /// Place stop order
    PostStopOrder,
    /// List active stop orders
    GetStopOrders,
    /// Cancel stop order
    CancelStopOrder,

    // ═══════════════════════════════════════════════════════════════════════
    // OperationsService - Account operations and portfolio
    // ═══════════════════════════════════════════════════════════════════════
    /// List account operations
    GetOperations,
    /// Get current portfolio holdings
    GetPortfolio,
    /// Get current positions
    GetPositions,
    /// Get available balance for withdrawal
    GetWithdrawLimits,
    /// Get broker statement
    GetBrokerReport,
    /// Get foreign dividend report
    GetDividendsForeignIssuer,
    /// Get operations with pagination
    GetOperationsByCursor,

    // ═══════════════════════════════════════════════════════════════════════
    // UsersService - Account management and user information
    // ═══════════════════════════════════════════════════════════════════════
    /// List trading accounts
    GetAccounts,
    /// Get margin account attributes
    GetMarginAttributes,
    /// Get user tariff/commission plan
    GetUserTariff,
    /// Get user profile information
    GetInfo,

    // ═══════════════════════════════════════════════════════════════════════
    // SandboxService - Testing environment
    // ═══════════════════════════════════════════════════════════════════════
    /// Create sandbox account
    OpenSandboxAccount,
    /// List sandbox accounts
    GetSandboxAccounts,
    /// Delete sandbox account
    CloseSandboxAccount,
    /// Place order in sandbox
    PostSandboxOrder,
    /// List sandbox orders
    GetSandboxOrders,
    /// Cancel sandbox order
    CancelSandboxOrder,
    /// Get sandbox order status
    GetSandboxOrderState,
    /// Get sandbox positions
    GetSandboxPositions,
    /// Get sandbox operations
    GetSandboxOperations,
    /// Get sandbox portfolio
    GetSandboxPortfolio,
    /// Add virtual funds to sandbox
    SandboxPayIn,
}

impl TinkoffEndpoint {
    /// Get full endpoint path for REST proxy
    ///
    /// Format: `/rest/tinkoff.public.invest.api.contract.v1.{ServiceName}/{MethodName}`
    pub fn path(&self) -> &'static str {
        match self {
            // MarketDataService
            Self::GetCandles => "/tinkoff.public.invest.api.contract.v1.MarketDataService/GetCandles",
            Self::GetLastPrices => "/tinkoff.public.invest.api.contract.v1.MarketDataService/GetLastPrices",
            Self::GetOrderBook => "/tinkoff.public.invest.api.contract.v1.MarketDataService/GetOrderBook",
            Self::GetTradingStatus => "/tinkoff.public.invest.api.contract.v1.MarketDataService/GetTradingStatus",
            Self::GetTradingStatuses => "/tinkoff.public.invest.api.contract.v1.MarketDataService/GetTradingStatuses",
            Self::GetLastTrades => "/tinkoff.public.invest.api.contract.v1.MarketDataService/GetLastTrades",
            Self::GetClosePrices => "/tinkoff.public.invest.api.contract.v1.MarketDataService/GetClosePrices",

            // InstrumentsService
            Self::TradingSchedules => "/tinkoff.public.invest.api.contract.v1.InstrumentsService/TradingSchedules",
            Self::BondBy => "/tinkoff.public.invest.api.contract.v1.InstrumentsService/BondBy",
            Self::Bonds => "/tinkoff.public.invest.api.contract.v1.InstrumentsService/Bonds",
            Self::GetBondCoupons => "/tinkoff.public.invest.api.contract.v1.InstrumentsService/GetBondCoupons",
            Self::CurrencyBy => "/tinkoff.public.invest.api.contract.v1.InstrumentsService/CurrencyBy",
            Self::Currencies => "/tinkoff.public.invest.api.contract.v1.InstrumentsService/Currencies",
            Self::EtfBy => "/tinkoff.public.invest.api.contract.v1.InstrumentsService/EtfBy",
            Self::Etfs => "/tinkoff.public.invest.api.contract.v1.InstrumentsService/Etfs",
            Self::FutureBy => "/tinkoff.public.invest.api.contract.v1.InstrumentsService/FutureBy",
            Self::Futures => "/tinkoff.public.invest.api.contract.v1.InstrumentsService/Futures",
            Self::OptionBy => "/tinkoff.public.invest.api.contract.v1.InstrumentsService/OptionBy",
            Self::OptionsBy => "/tinkoff.public.invest.api.contract.v1.InstrumentsService/OptionsBy",
            Self::ShareBy => "/tinkoff.public.invest.api.contract.v1.InstrumentsService/ShareBy",
            Self::Shares => "/tinkoff.public.invest.api.contract.v1.InstrumentsService/Shares",
            Self::GetAccruedInterests => "/tinkoff.public.invest.api.contract.v1.InstrumentsService/GetAccruedInterests",
            Self::GetFuturesMargin => "/tinkoff.public.invest.api.contract.v1.InstrumentsService/GetFuturesMargin",
            Self::GetInstrumentBy => "/tinkoff.public.invest.api.contract.v1.InstrumentsService/GetInstrumentBy",
            Self::GetDividends => "/tinkoff.public.invest.api.contract.v1.InstrumentsService/GetDividends",
            Self::GetAssetBy => "/tinkoff.public.invest.api.contract.v1.InstrumentsService/GetAssetBy",
            Self::GetAssets => "/tinkoff.public.invest.api.contract.v1.InstrumentsService/GetAssets",
            Self::GetFavorites => "/tinkoff.public.invest.api.contract.v1.InstrumentsService/GetFavorites",
            Self::EditFavorites => "/tinkoff.public.invest.api.contract.v1.InstrumentsService/EditFavorites",
            Self::GetCountries => "/tinkoff.public.invest.api.contract.v1.InstrumentsService/GetCountries",
            Self::FindInstrument => "/tinkoff.public.invest.api.contract.v1.InstrumentsService/FindInstrument",
            Self::GetBrands => "/tinkoff.public.invest.api.contract.v1.InstrumentsService/GetBrands",
            Self::GetBrandBy => "/tinkoff.public.invest.api.contract.v1.InstrumentsService/GetBrandBy",

            // OrdersService
            Self::PostOrder => "/tinkoff.public.invest.api.contract.v1.OrdersService/PostOrder",
            Self::CancelOrder => "/tinkoff.public.invest.api.contract.v1.OrdersService/CancelOrder",
            Self::GetOrderState => "/tinkoff.public.invest.api.contract.v1.OrdersService/GetOrderState",
            Self::GetOrders => "/tinkoff.public.invest.api.contract.v1.OrdersService/GetOrders",
            Self::ReplaceOrder => "/tinkoff.public.invest.api.contract.v1.OrdersService/ReplaceOrder",

            // StopOrdersService
            Self::PostStopOrder => "/tinkoff.public.invest.api.contract.v1.StopOrdersService/PostStopOrder",
            Self::GetStopOrders => "/tinkoff.public.invest.api.contract.v1.StopOrdersService/GetStopOrders",
            Self::CancelStopOrder => "/tinkoff.public.invest.api.contract.v1.StopOrdersService/CancelStopOrder",

            // OperationsService
            Self::GetOperations => "/tinkoff.public.invest.api.contract.v1.OperationsService/GetOperations",
            Self::GetPortfolio => "/tinkoff.public.invest.api.contract.v1.OperationsService/GetPortfolio",
            Self::GetPositions => "/tinkoff.public.invest.api.contract.v1.OperationsService/GetPositions",
            Self::GetWithdrawLimits => "/tinkoff.public.invest.api.contract.v1.OperationsService/GetWithdrawLimits",
            Self::GetBrokerReport => "/tinkoff.public.invest.api.contract.v1.OperationsService/GetBrokerReport",
            Self::GetDividendsForeignIssuer => "/tinkoff.public.invest.api.contract.v1.OperationsService/GetDividendsForeignIssuer",
            Self::GetOperationsByCursor => "/tinkoff.public.invest.api.contract.v1.OperationsService/GetOperationsByCursor",

            // UsersService
            Self::GetAccounts => "/tinkoff.public.invest.api.contract.v1.UsersService/GetAccounts",
            Self::GetMarginAttributes => "/tinkoff.public.invest.api.contract.v1.UsersService/GetMarginAttributes",
            Self::GetUserTariff => "/tinkoff.public.invest.api.contract.v1.UsersService/GetUserTariff",
            Self::GetInfo => "/tinkoff.public.invest.api.contract.v1.UsersService/GetInfo",

            // SandboxService
            Self::OpenSandboxAccount => "/tinkoff.public.invest.api.contract.v1.SandboxService/OpenSandboxAccount",
            Self::GetSandboxAccounts => "/tinkoff.public.invest.api.contract.v1.SandboxService/GetSandboxAccounts",
            Self::CloseSandboxAccount => "/tinkoff.public.invest.api.contract.v1.SandboxService/CloseSandboxAccount",
            Self::PostSandboxOrder => "/tinkoff.public.invest.api.contract.v1.SandboxService/PostSandboxOrder",
            Self::GetSandboxOrders => "/tinkoff.public.invest.api.contract.v1.SandboxService/GetSandboxOrders",
            Self::CancelSandboxOrder => "/tinkoff.public.invest.api.contract.v1.SandboxService/CancelSandboxOrder",
            Self::GetSandboxOrderState => "/tinkoff.public.invest.api.contract.v1.SandboxService/GetSandboxOrderState",
            Self::GetSandboxPositions => "/tinkoff.public.invest.api.contract.v1.SandboxService/GetSandboxPositions",
            Self::GetSandboxOperations => "/tinkoff.public.invest.api.contract.v1.SandboxService/GetSandboxOperations",
            Self::GetSandboxPortfolio => "/tinkoff.public.invest.api.contract.v1.SandboxService/GetSandboxPortfolio",
            Self::SandboxPayIn => "/tinkoff.public.invest.api.contract.v1.SandboxService/SandboxPayIn",
        }
    }
}

/// Format ticker symbol for Tinkoff API
///
/// Tinkoff uses FIGI (Financial Instrument Global Identifier) or ticker+class_code.
/// For stock tickers, we return just the base (ticker symbol).
///
/// Examples:
/// - Symbol{base: "SBER", quote: "RUB"} -> "SBER"
/// - Symbol{base: "GAZP", quote: "RUB"} -> "GAZP"
pub fn format_ticker(symbol: &crate::core::types::Symbol) -> String {
    // For Russian stocks, typically just use the base ticker
    // The quote currency is usually implicit (RUB for MOEX)
    symbol.base.to_uppercase()
}

/// Parse ticker from API format back to domain Symbol
///
/// Tinkoff returns tickers directly. We assume RUB as quote currency
/// for Russian stocks traded on MOEX.
pub fn _parse_ticker(api_ticker: &str) -> crate::core::types::Symbol {
    crate::core::types::Symbol::new(api_ticker, "RUB")
}
