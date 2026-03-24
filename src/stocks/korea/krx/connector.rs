//! KRX connector implementation

use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::*;
use crate::core::traits::*;

use super::endpoints::*;
use super::auth::*;
use super::parser::*;

/// Detect the most likely KRX market for a given 6-digit stock code.
///
/// KOSDAQ codes predominantly start with 2 or 3 (e.g. 035720 Kakao, 293490 Kakao Pay).
/// All other first digits (0, 1, 4, 5, 6, 7, 8, 9) are KOSPI by convention.
/// This is a heuristic — authoritative classification requires querying the KRX base-info
/// endpoint, but this covers the vast majority of real-world cases without a network round-trip.
fn detect_market(code: &str) -> MarketId {
    let first_digit = code.chars().next().unwrap_or('0');
    match first_digit {
        '2' | '3' => MarketId::Kosdaq,
        _ => MarketId::Kospi,
    }
}

/// KRX (Korea Exchange) connector
pub struct KrxConnector {
    client: Client,
    auth: KrxAuth,
    endpoints: KrxEndpoints,
}

impl KrxConnector {
    /// Create new KRX connector with authentication
    pub fn new(auth: KrxAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: KrxEndpoints::default(),
        }
    }

    /// Create public connector without API keys
    ///
    /// WARNING: The new KRX Open API requires authentication.
    /// This constructor is kept for backward compatibility but most methods
    /// will fail with Auth error unless PUBLIC_DATA_PORTAL_KEY is set.
    ///
    /// The old web-scraping pattern (data.krx.co.kr) is DEPRECATED and returns "LOGOUT".
    /// Users must register at https://openapi.krx.co.kr/ to obtain AUTH_KEY.
    #[deprecated(
        since = "0.1.0",
        note = "KRX now requires authentication. Use new() with KrxAuth::from_env() or obtain keys from openapi.krx.co.kr"
    )]
    pub fn new_public() -> Self {
        Self {
            client: Client::new(),
            auth: KrxAuth {
                auth_key: None,
                public_data_portal_key: None,
            },
            endpoints: KrxEndpoints::default(),
        }
    }

    /// Create connector from environment variables
    pub fn from_env() -> Self {
        Self::new(KrxAuth::from_env())
    }

    /// Make POST request to Open API
    ///
    /// All Open API requests are JSON POST with {"basDd": "YYYYMMDD"} body format
    async fn post_openapi(
        &self,
        endpoint: KrxEndpoint,
        body: serde_json::Value,
    ) -> ExchangeResult<serde_json::Value> {
        // Check for authentication
        if !self.auth.has_openapi_auth() {
            return Err(ExchangeError::Auth(
                "KRX Open API requires AUTH_KEY. Register at https://openapi.krx.co.kr/ and set KRX_AUTH_KEY environment variable".to_string(),
            ));
        }

        let url = format!("{}{}", self.endpoints.openapi_base, endpoint.path());

        // Prepare headers with auth
        let mut headers = HashMap::new();
        self.auth.sign_openapi_headers(&mut headers);

        // Build request
        let mut request = self.client.post(&url);

        // Add headers
        for (key, value) in headers {
            request = request.header(key, value);
        }

        // Add JSON body
        request = request.json(&body);

        // Send request
        let response = request
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        // Check HTTP status
        let status = response.status();

        // Get response text for parsing
        let response_text = response
            .text()
            .await
            .map_err(|e| ExchangeError::Network(format!("Failed to read response: {}", e)))?;

        if !status.is_success() {
            return Err(match status.as_u16() {
                401 => ExchangeError::Auth(format!("API key not authorized: {}", response_text)),
                403 => ExchangeError::PermissionDenied(format!("Access forbidden - check API permissions: {}", response_text)),
                429 => ExchangeError::RateLimit,
                _ => ExchangeError::Http(format!("HTTP {}: {}", status, response_text)),
            });
        }

        // Parse JSON response
        let json: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}. Response: {}", e, response_text)))?;

        // Check for API errors in new format
        KrxParser::check_api_error(&json)?;

        Ok(json)
    }

    /// Make GET request to Public Data Portal API
    async fn get_portal(&self, mut params: HashMap<String, String>) -> ExchangeResult<serde_json::Value> {
        let url = self.endpoints.public_data_portal;

        // Add authentication
        self.auth.sign_portal_query(&mut params);

        // Add default params
        params.entry("resultType".to_string()).or_insert("json".to_string());
        params.entry("numOfRows".to_string()).or_insert("100".to_string());
        params.entry("pageNo".to_string()).or_insert("1".to_string());

        // Send request
        let response = self
            .client
            .get(url)
            .query(&params)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        // Check HTTP status
        if !response.status().is_success() {
            return Err(ExchangeError::Http(format!("HTTP {}", response.status())));
        }

        // Parse JSON response
        let json = response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        // Check for API errors
        KrxParser::check_api_error(&json)?;

        Ok(json)
    }

    /// Get daily trading data for a specific date
    ///
    /// Internal helper that fetches data for a single date.
    /// The new API only supports single-date queries.
    async fn get_daily_data(
        &self,
        _symbol: &Symbol,
        date: &str,
        market: MarketId,
    ) -> ExchangeResult<serde_json::Value> {
        let endpoint = market.daily_trading_endpoint();

        // Build request body
        let body = serde_json::json!({
            "basDd": date
        });

        self.post_openapi(endpoint, body).await
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: ExchangeIdentity
// ═══════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for KrxConnector {
    fn exchange_name(&self) -> &'static str {
        "krx"
    }

    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Krx
    }

    fn is_testnet(&self) -> bool {
        false
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        // KRX is data provider only - use Spot as equivalent
        vec![AccountType::Spot]
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: MarketData
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for KrxConnector {
    /// Get current price
    ///
    /// Note: KRX data is delayed by 1 business day
    async fn get_price(&self, symbol: Symbol, _account_type: AccountType) -> ExchangeResult<Price> {
        // Get latest OHLCV to extract current price
        let klines = self.get_klines(symbol, "1d", Some(1), AccountType::Spot, None).await?;

        if let Some(latest) = klines.first() {
            Ok(latest.close)
        } else {
            Err(ExchangeError::NotFound("No price data available".to_string()))
        }
    }

    /// Get ticker (24h stats)
    async fn get_ticker(&self, symbol: Symbol, _account_type: AccountType) -> ExchangeResult<Ticker> {
        // Get latest OHLCV to construct ticker
        let klines = self.get_klines(symbol.clone(), "1d", Some(1), AccountType::Spot, None).await?;

        if let Some(latest) = klines.first() {
            Ok(Ticker {
                symbol: symbol.base.clone(),
                last_price: latest.close,
                bid_price: None,
                ask_price: None,
                high_24h: Some(latest.high),
                low_24h: Some(latest.low),
                volume_24h: Some(latest.volume),
                quote_volume_24h: latest.quote_volume,
                price_change_24h: None,
                price_change_percent_24h: None,
                timestamp: latest.open_time,
            })
        } else {
            Err(ExchangeError::NotFound("No ticker data available".to_string()))
        }
    }

    /// Get orderbook
    ///
    /// KRX does not provide orderbook data through public API
    async fn get_orderbook(
        &self,
        _symbol: Symbol,
        _depth: Option<u16>,
        _account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        Err(ExchangeError::UnsupportedOperation(
            "KRX does not provide orderbook data - data feed only".to_string(),
        ))
    }

    /// Get klines/candles (historical OHLCV data)
    ///
    /// Note: The new API only supports single-date queries.
    /// For date ranges, we must loop over each date.
    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        _account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        // KRX only provides daily data
        if interval != "1d" && interval != "1day" {
            return Err(ExchangeError::InvalidRequest(
                "KRX only provides daily (1d) candles".to_string(),
            ));
        }

        // Calculate date range
        let limit = limit.unwrap_or(30) as i64;

        use chrono::{Duration, Local, Datelike};
        let end = Local::now();
        let start = end - Duration::days(limit - 1);

        // Collect dates to fetch
        let mut dates = Vec::new();
        let mut current = start;
        while current <= end {
            dates.push(format_date(current.year(), current.month(), current.day()));
            current += Duration::days(1);
        }

        // Detect market from the stock code.
        // Korean 6-digit codes have a conventional first-digit breakdown:
        //   0, 1        → KOSPI (e.g. 005930 Samsung, 000660 SK Hynix)
        //   2 (2xxxxx)  → KOSDAQ (most 2-series codes)
        //   3 (3xxxxx)  → KOSDAQ (e.g. 035720 Kakao also classified here)
        //   4, 5        → KOSPI preferred (ETFs, preferred shares)
        //   6, 7, 8, 9  → KOSPI
        // This is a best-effort heuristic; exact classification requires
        // querying the KRX base info endpoint.
        let market = detect_market(&symbol.base);

        // Fetch all dates and collect klines
        // Note: In production, you may want to add rate limiting and concurrency control
        let mut all_klines = Vec::new();

        for date in dates {
            match self.get_daily_data(&symbol, &date, market).await {
                Ok(response) => {
                    // Parse klines from response for this specific date and symbol
                    match KrxParser::parse_klines(&response, &symbol.base) {
                        Ok(mut klines) => all_klines.append(&mut klines),
                        Err(_) => continue, // Skip dates with no data
                    }
                }
                Err(_) => continue, // Skip dates that fail (weekends, holidays, etc.)
            }
        }

        // Sort by timestamp
        all_klines.sort_by_key(|k| k.open_time);

        Ok(all_klines)
    }

    /// Ping the API
    async fn ping(&self) -> ExchangeResult<()> {
        // Try to fetch today's data as ping
        let today = format_today();
        let body = serde_json::json!({
            "basDd": today
        });

        let _ = self.post_openapi(KrxEndpoint::KospiDailyTrading, body).await?;
        Ok(())
    }

    /// Get exchange info — returns KOSPI listed stocks from KRX
    async fn get_exchange_info(&self, _account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        // KospiBaseInfo returns list of all listed securities on KOSPI
        let today = format_today();
        let body = serde_json::json!({
            "basDd": today
        });

        let response = self.post_openapi(KrxEndpoint::KospiBaseInfo, body).await?;
        let symbols = KrxParser::parse_symbols(&response)?;

        let infos = symbols.into_iter().map(|code| SymbolInfo {
            symbol: code.clone(),
            base_asset: code,
            quote_asset: "KRW".to_string(),
            status: "TRADING".to_string(),
            price_precision: 0,
            quantity_precision: 0,
            min_quantity: Some(1.0),
            max_quantity: None,
            tick_size: None,
            step_size: Some(1.0),
            min_notional: None,
            account_type: Default::default(),
        }).collect();

        Ok(infos)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Trading (NOT SUPPORTED - DATA PROVIDER ONLY)
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for KrxConnector {
    async fn place_order(&self, _req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        Err(ExchangeError::UnsupportedOperation(
            "KRX is a data provider - trading not supported".to_string()
        ))
    }

    async fn cancel_order(&self, _req: CancelRequest) -> ExchangeResult<Order> {
        Err(ExchangeError::UnsupportedOperation(
            "KRX is a data provider - trading not supported".to_string()
        ))
    }

    async fn get_order(
        &self,
        _symbol: &str,
        _order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        Err(ExchangeError::UnsupportedOperation(
            "KRX is a data provider - trading not supported".to_string()
        ))
    }

    async fn get_open_orders(
        &self,
        _symbol: Option<&str>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        Err(ExchangeError::UnsupportedOperation(
            "KRX is a data provider - trading not supported".to_string()
        ))
    }

    async fn get_order_history(
        &self,
        _filter: OrderHistoryFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        Err(ExchangeError::UnsupportedOperation(
            "KRX is a data provider - trading not supported".to_string()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Account (NOT SUPPORTED - DATA PROVIDER ONLY)
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for KrxConnector {
    async fn get_balance(&self, _query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        Err(ExchangeError::UnsupportedOperation(
            "KRX is a data provider - account operations not supported".to_string(),
        ))
    
    }

    async fn get_account_info(&self, _account_type: AccountType) -> ExchangeResult<AccountInfo> {
        Err(ExchangeError::UnsupportedOperation(
            "KRX is a data provider - account operations not supported".to_string(),
        ))
    }

    async fn get_fees(&self, _symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        Err(ExchangeError::UnsupportedOperation(
            "KRX is a data provider - account operations not supported".to_string()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Positions (NOT SUPPORTED - DATA PROVIDER ONLY)
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for KrxConnector {
    async fn get_positions(&self, _query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        Err(ExchangeError::UnsupportedOperation(
            "KRX is a data provider - position tracking not supported".to_string()
        ))
    }

    async fn get_funding_rate(
        &self,
        _symbol: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        Err(ExchangeError::UnsupportedOperation(
            "KRX is a data provider - position tracking not supported".to_string()
        ))
    }

    async fn modify_position(&self, _req: PositionModification) -> ExchangeResult<()> {
        Err(ExchangeError::UnsupportedOperation(
            "KRX is a data provider - position tracking not supported".to_string()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// EXTENDED METHODS (KRX-SPECIFIC)
// ═══════════════════════════════════════════════════════════════════════════

impl KrxConnector {
    /// Get stock information from Public Data Portal
    ///
    /// Returns detailed company information including name, market, ISIN, etc.
    pub async fn get_stock_info(&self, ticker: &str) -> ExchangeResult<serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("likeSrtnCd".to_string(), ticker.to_string());

        let response = self.get_portal(params).await?;
        let items = KrxParser::parse_stock_info(&response)?;

        if let Some(first) = items.first() {
            Ok(first.clone())
        } else {
            Err(ExchangeError::NotFound(format!("Stock '{}' not found", ticker)))
        }
    }

    /// Get base info for a stock
    ///
    /// Uses the new Open API base info endpoint
    pub async fn get_base_info(
        &self,
        date: &str,
        market: MarketId,
    ) -> ExchangeResult<serde_json::Value> {
        let endpoint = match market {
            MarketId::Kospi => KrxEndpoint::KospiBaseInfo,
            MarketId::Kosdaq => KrxEndpoint::KosdaqBaseInfo,
            MarketId::Konex => KrxEndpoint::KonexBaseInfo,
            MarketId::All => KrxEndpoint::KospiBaseInfo,
        };

        let body = serde_json::json!({
            "basDd": date
        });

        self.post_openapi(endpoint, body).await
    }

    /// Get market index data
    pub async fn get_index_data(&self, date: &str) -> ExchangeResult<serde_json::Value> {
        let body = serde_json::json!({
            "basDd": date
        });

        self.post_openapi(KrxEndpoint::IndexDailyTrading, body).await
    }
}
