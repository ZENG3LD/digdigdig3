//! Zerodha Kite Connect API endpoints

/// Base URLs for Zerodha Kite Connect
pub struct ZerodhaEndpoints {
    pub rest_base: &'static str,
    pub _ws_base: Option<&'static str>,
    pub _login_url: &'static str,
}

impl Default for ZerodhaEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://api.kite.trade",
            _ws_base: Some("wss://ws.kite.trade"),
            _login_url: "https://kite.zerodha.com/connect/login",
        }
    }
}

/// API endpoint enum
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ZerodhaEndpoint {
    // Authentication
    SessionToken,           // POST /session/token - exchange request_token for access_token
    SessionLogout,          // DELETE /session/token - invalidate session

    // User Profile
    UserProfile,            // GET /user/profile

    // Market Data
    Instruments,            // GET /instruments - all instruments (CSV)
    InstrumentsExchange(String), // GET /instruments/{exchange} - exchange-specific (CSV)
    Quote,                  // GET /quote - full market quotes
    QuoteOhlc,              // GET /quote/ohlc - OHLC quotes
    QuoteLtp,               // GET /quote/ltp - Last traded price only
    HistoricalCandles(u32, String), // GET /instruments/historical/{token}/{interval}

    // Trading
    PlaceOrder(String),     // POST /orders/{variety}
    ModifyOrder(String, String), // PUT /orders/{variety}/{order_id}
    CancelOrder(String, String), // DELETE /orders/{variety}/{order_id}
    GetOrders,              // GET /orders
    GetOrder(String),       // GET /orders/{order_id}
    GetTrades,              // GET /trades
    GetOrderTrades(String), // GET /orders/{order_id}/trades

    // GTT (Good Till Triggered)
    PlaceGtt,               // POST /gtt/triggers
    ModifyGtt(u64),         // PUT /gtt/triggers/{trigger_id}
    DeleteGtt(u64),         // DELETE /gtt/triggers/{trigger_id}
    GetGtts,                // GET /gtt/triggers
    GetGtt(u64),            // GET /gtt/triggers/{trigger_id}

    // Account & Margins
    GetMargins,             // GET /user/margins
    GetMarginsSegment(String), // GET /user/margins/{segment}
    OrderMargins,           // POST /margins/orders
    BasketMargins,          // POST /margins/basket

    // Portfolio
    Holdings,               // GET /portfolio/holdings
    HoldingsAuctions,       // GET /portfolio/holdings/auctions
    AuthorizeHoldings,      // POST /portfolio/holdings/authorise
    Positions,              // GET /portfolio/positions
    ConvertPosition,        // PUT /portfolio/positions

    // Basket Orders
    BasketOrders,           // POST /orders/baskets — place multiple orders atomically
}

impl ZerodhaEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> String {
        match self {
            // Authentication
            Self::SessionToken => "/session/token".to_string(),
            Self::SessionLogout => "/session/token".to_string(),

            // User Profile
            Self::UserProfile => "/user/profile".to_string(),

            // Market Data
            Self::Instruments => "/instruments".to_string(),
            Self::InstrumentsExchange(exchange) => format!("/instruments/{}", exchange),
            Self::Quote => "/quote".to_string(),
            Self::QuoteOhlc => "/quote/ohlc".to_string(),
            Self::QuoteLtp => "/quote/ltp".to_string(),
            Self::HistoricalCandles(token, interval) => {
                format!("/instruments/historical/{}/{}", token, interval)
            }

            // Trading
            Self::PlaceOrder(variety) => format!("/orders/{}", variety),
            Self::ModifyOrder(variety, order_id) => format!("/orders/{}/{}", variety, order_id),
            Self::CancelOrder(variety, order_id) => format!("/orders/{}/{}", variety, order_id),
            Self::GetOrders => "/orders".to_string(),
            Self::GetOrder(order_id) => format!("/orders/{}", order_id),
            Self::GetTrades => "/trades".to_string(),
            Self::GetOrderTrades(order_id) => format!("/orders/{}/trades", order_id),

            // GTT
            Self::PlaceGtt => "/gtt/triggers".to_string(),
            Self::ModifyGtt(trigger_id) => format!("/gtt/triggers/{}", trigger_id),
            Self::DeleteGtt(trigger_id) => format!("/gtt/triggers/{}", trigger_id),
            Self::GetGtts => "/gtt/triggers".to_string(),
            Self::GetGtt(trigger_id) => format!("/gtt/triggers/{}", trigger_id),

            // Account & Margins
            Self::GetMargins => "/user/margins".to_string(),
            Self::GetMarginsSegment(segment) => format!("/user/margins/{}", segment),
            Self::OrderMargins => "/margins/orders".to_string(),
            Self::BasketMargins => "/margins/basket".to_string(),

            // Portfolio
            Self::Holdings => "/portfolio/holdings".to_string(),
            Self::HoldingsAuctions => "/portfolio/holdings/auctions".to_string(),
            Self::AuthorizeHoldings => "/portfolio/holdings/authorise".to_string(),
            Self::Positions => "/portfolio/positions".to_string(),
            Self::ConvertPosition => "/portfolio/positions".to_string(),

            // Basket Orders
            Self::BasketOrders => "/orders/baskets".to_string(),
        }
    }
}

/// Format symbol for Zerodha API
///
/// Zerodha uses format: EXCHANGE:TRADINGSYMBOL
/// Examples:
/// - NSE:INFY (Infosys equity on NSE)
/// - BSE:INFY (Infosys equity on BSE)
/// - NFO:NIFTY26FEB20000CE (Nifty Call Option)
pub fn format_symbol(symbol: &crate::core::types::Symbol) -> String {
    // For Indian stocks, the "base" is the trading symbol
    // and "quote" typically represents the exchange or is empty
    // We'll use a convention: if quote is empty/INR, use NSE as default
    let exchange = if symbol.quote.is_empty() || symbol.quote == "INR" {
        "NSE"
    } else {
        &symbol.quote
    };

    format!("{}:{}", exchange, symbol.base.to_uppercase())
}

/// Parse symbol from Zerodha format back to domain Symbol
pub fn _parse_symbol(api_symbol: &str) -> crate::core::types::Symbol {
    if let Some((exchange, tradingsymbol)) = api_symbol.split_once(':') {
        crate::core::types::Symbol {
            base: tradingsymbol.to_string(),
            quote: exchange.to_string(),
            raw: Some(api_symbol.to_string()),
        }
    } else {
        // Fallback: assume no exchange prefix
        crate::core::types::Symbol {
            base: api_symbol.to_string(),
            quote: "INR".to_string(),
            raw: Some(api_symbol.to_string()),
        }
    }
}

