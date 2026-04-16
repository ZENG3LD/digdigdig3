# Interactive Brokers Client Portal Web API - Implementation Notes for V5 Connector

## Architecture Recommendations

### Module Structure

Following the V5 pattern (reference: KuCoin implementation):

```
v5/src/aggregators/ib/
├── mod.rs              # Module exports
├── endpoints.rs        # URL constants, endpoint enum, symbol formatting
├── auth.rs             # Authentication (Gateway session management)
├── parser.rs           # JSON response parsing
├── connector.rs        # Trait implementations
└── websocket.rs        # WebSocket streaming
```

### Trait Implementation Requirements

**From V5 architecture:**
- `MarketData` trait - Market data operations
- `Trading` trait - Order management
- `Account` trait - Account and position queries

**IB-Specific Considerations:**
- Session management (tickle mechanism)
- Contract ID (conid) resolution
- Multi-step order confirmation

## Key Implementation Challenges

### 1. Session Management

**Challenge:** Gateway requires periodic tickle to maintain session

**Solution:**
```rust
use tokio::time::{interval, Duration};

pub struct SessionManager {
    http_client: reqwest::Client,
    base_url: String,
    tickle_interval: Duration,
}

impl SessionManager {
    pub fn new(base_url: String) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            base_url,
            tickle_interval: Duration::from_secs(30),
        }
    }

    pub async fn start_tickle_loop(&self) {
        let mut interval = interval(self.tickle_interval);

        loop {
            interval.tick().await;

            if let Err(e) = self.tickle().await {
                eprintln!("Tickle failed: {}", e);
            }
        }
    }

    async fn tickle(&self) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}/tickle", self.base_url);
        let response = self.http_client.get(&url).send().await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("Tickle failed: {}", response.status()).into())
        }
    }

    pub async fn check_auth_status(&self) -> Result<AuthStatus, Box<dyn std::error::Error>> {
        let url = format!("{}/iserver/auth/status", self.base_url);
        let response = self.http_client.get(&url).send().await?;
        let auth_status: AuthStatus = response.json().await?;
        Ok(auth_status)
    }

    pub async fn initialize_session(&self) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}/iserver/auth/ssodh/init", self.base_url);
        self.http_client.post(&url).send().await?;
        Ok(())
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct AuthStatus {
    pub authenticated: bool,
    pub competing: bool,
    pub connected: bool,
    pub message: String,
}
```

**Integration:**
```rust
// In connector initialization
let session_manager = Arc::new(SessionManager::new(base_url));
let session_clone = session_manager.clone();

// Spawn tickle loop
tokio::spawn(async move {
    session_clone.start_tickle_loop().await;
});
```

### 2. Contract ID Resolution

**Challenge:** IB uses conid instead of symbols for all operations

**Solution: Symbol Cache**
```rust
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct SymbolCache {
    symbol_to_conid: RwLock<HashMap<String, i64>>,
    conid_to_info: RwLock<HashMap<i64, ContractInfo>>,
}

impl SymbolCache {
    pub fn new() -> Self {
        Self {
            symbol_to_conid: RwLock::new(HashMap::new()),
            conid_to_info: RwLock::new(HashMap::new()),
        }
    }

    pub async fn get_conid(&self, symbol: &str) -> Option<i64> {
        let cache = self.symbol_to_conid.read().await;
        cache.get(symbol).copied()
    }

    pub async fn cache_contract(&self, symbol: String, conid: i64, info: ContractInfo) {
        let mut symbol_cache = self.symbol_to_conid.write().await;
        let mut info_cache = self.conid_to_info.write().await;

        symbol_cache.insert(symbol, conid);
        info_cache.insert(conid, info);
    }

    pub async fn get_contract_info(&self, conid: i64) -> Option<ContractInfo> {
        let cache = self.conid_to_info.read().await;
        cache.get(&conid).cloned()
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ContractInfo {
    pub conid: i64,
    pub symbol: String,
    pub sec_type: String,
    pub exchange: String,
    pub currency: String,
}

// Contract search implementation
pub async fn search_contract(
    client: &reqwest::Client,
    base_url: &str,
    symbol: &str,
    sec_type: &str,
) -> Result<Vec<ContractInfo>, ExchangeError> {
    let url = format!("{}/iserver/secdef/search", base_url);

    let body = serde_json::json!({
        "symbol": symbol,
        "name": false,
        "secType": sec_type
    });

    let response = client.post(&url).json(&body).send().await?;

    if !response.status().is_success() {
        return Err(ExchangeError::RequestFailed(
            format!("Contract search failed: {}", response.status())
        ));
    }

    let contracts: Vec<ContractInfo> = response.json().await?;
    Ok(contracts)
}
```

**Usage in MarketData trait:**
```rust
async fn get_ticker(&self, symbol: &str) -> Result<Ticker, ExchangeError> {
    // Try cache first
    let conid = match self.symbol_cache.get_conid(symbol).await {
        Some(id) => id,
        None => {
            // Search contract
            let contracts = search_contract(&self.http_client, &self.base_url, symbol, "STK").await?;

            if contracts.is_empty() {
                return Err(ExchangeError::SymbolNotFound(symbol.to_string()));
            }

            let contract = &contracts[0];
            self.symbol_cache.cache_contract(
                symbol.to_string(),
                contract.conid,
                contract.clone()
            ).await;

            contract.conid
        }
    };

    // Fetch market data using conid
    self.fetch_market_data_by_conid(conid).await
}
```

### 3. Order Confirmation Flow

**Challenge:** Some orders require explicit confirmation

**Solution:**
```rust
pub async fn place_order_with_confirmation(
    client: &reqwest::Client,
    base_url: &str,
    account_id: &str,
    order: &OrderRequest,
) -> Result<OrderResponse, ExchangeError> {
    let url = format!("{}/iserver/account/{}/orders", base_url, account_id);

    // Initial order submission
    let response = client.post(&url).json(order).send().await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(ExchangeError::OrderRejected(error_text));
    }

    let initial_response: serde_json::Value = response.json().await?;

    // Check if confirmation needed
    if initial_response.get("id").is_some() && initial_response.get("message").is_some() {
        let reply_id = initial_response["id"].as_str()
            .ok_or(ExchangeError::ParseError("Missing reply ID".into()))?;

        // Send confirmation
        let confirm_url = format!("{}/iserver/reply/{}", base_url, reply_id);
        let confirm_body = serde_json::json!({ "confirmed": true });

        let confirm_response = client.post(&confirm_url).json(&confirm_body).send().await?;

        if !confirm_response.status().is_success() {
            return Err(ExchangeError::OrderRejected("Confirmation failed".into()));
        }

        let final_response: OrderResponse = confirm_response.json().await?;
        Ok(final_response)
    } else {
        // No confirmation needed, parse initial response
        let order_response: OrderResponse = serde_json::from_value(initial_response)?;
        Ok(order_response)
    }
}
```

### 4. WebSocket Subscription Format

**Challenge:** IB uses text-based subscription format, not JSON

**Solution:**
```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};

pub struct IBWebSocket {
    url: String,
}

impl IBWebSocket {
    pub fn new(url: String) -> Self {
        Self { url }
    }

    pub async fn connect(&self) -> Result<(), Box<dyn std::error::Error>> {
        let (ws_stream, _) = connect_async(&self.url).await?;
        let (mut write, mut read) = ws_stream.split();

        // Subscribe to market data
        let conid = 265598;
        let subscription = format!(
            r#"smd+{}+{{"fields":["31","84","86","87","88","85"]}}"#,
            conid
        );

        write.send(Message::Text(subscription)).await?;

        // Subscribe to order updates
        let order_subscription = "sor+{}".to_string();
        write.send(Message::Text(order_subscription)).await?;

        // Handle incoming messages
        while let Some(message) = read.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    self.handle_message(&text)?;
                }
                Ok(Message::Close(_)) => {
                    println!("WebSocket closed");
                    break;
                }
                Err(e) => {
                    eprintln!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn handle_message(&self, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        let data: serde_json::Value = serde_json::from_str(text)?;

        match data.get("topic").and_then(|t| t.as_str()) {
            Some("smd") => {
                // Market data update
                self.handle_market_data(&data)?;
            }
            Some("sor") => {
                // Order update
                self.handle_order_update(&data)?;
            }
            Some("system") => {
                // System message (heartbeat, etc.)
                if data.get("heartbeat").is_some() {
                    println!("Heartbeat received");
                }
            }
            _ => {
                println!("Unknown message type: {}", text);
            }
        }

        Ok(())
    }

    fn handle_market_data(&self, data: &serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
        let conid = data["conid"].as_i64().unwrap_or(0);
        let last_price = data.get("31").and_then(|v| v.as_f64());
        let bid = data.get("84").and_then(|v| v.as_f64());
        let ask = data.get("86").and_then(|v| v.as_f64());

        println!("Market Data [{}]: Last={:?}, Bid={:?}, Ask={:?}", conid, last_price, bid, ask);
        Ok(())
    }

    fn handle_order_update(&self, data: &serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
        let order_id = data.get("orderId").and_then(|v| v.as_i64()).unwrap_or(0);
        let status = data.get("orderStatus").and_then(|v| v.as_str()).unwrap_or("Unknown");

        println!("Order Update [{}]: Status={}", order_id, status);
        Ok(())
    }
}
```

### 5. Rate Limiting

**Challenge:** 10 req/s global limit (Gateway), 50 req/s (OAuth)

**Solution:**
```rust
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use std::collections::VecDeque;

pub struct RateLimiter {
    max_requests: usize,
    time_window: Duration,
    requests: Mutex<VecDeque<Instant>>,
}

impl RateLimiter {
    pub fn new(max_requests: usize, time_window_secs: u64) -> Self {
        Self {
            max_requests,
            time_window: Duration::from_secs(time_window_secs),
            requests: Mutex::new(VecDeque::new()),
        }
    }

    pub async fn acquire(&self) {
        let mut requests = self.requests.lock().await;
        let now = Instant::now();

        // Remove old requests outside time window
        while let Some(&oldest) = requests.front() {
            if now.duration_since(oldest) > self.time_window {
                requests.pop_front();
            } else {
                break;
            }
        }

        // Wait if rate limit reached
        if requests.len() >= self.max_requests {
            if let Some(&oldest) = requests.front() {
                let wait_duration = self.time_window
                    .saturating_sub(now.duration_since(oldest));

                if !wait_duration.is_zero() {
                    drop(requests); // Release lock before sleeping
                    tokio::time::sleep(wait_duration).await;
                    return self.acquire().await; // Retry after wait
                }
            }
        }

        // Record this request
        requests.push_back(now);
    }
}

// Integration in connector
pub struct IBConnector {
    rate_limiter: RateLimiter,
    // ... other fields
}

impl IBConnector {
    pub fn new(base_url: String) -> Self {
        Self {
            rate_limiter: RateLimiter::new(10, 1), // 10 requests per second
            // ...
        }
    }

    async fn request<T>(&self, f: impl Future<Output = Result<T, ExchangeError>>) -> Result<T, ExchangeError> {
        self.rate_limiter.acquire().await;
        f.await
    }
}
```

## Data Structure Mapping

### Market Data Ticker

```rust
#[derive(Debug, Clone)]
pub struct Ticker {
    pub symbol: String,
    pub last_price: Option<f64>,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub volume: Option<f64>,
    pub timestamp: i64,
}

impl From<IBMarketDataSnapshot> for Ticker {
    fn from(snapshot: IBMarketDataSnapshot) -> Self {
        Self {
            symbol: snapshot.symbol.unwrap_or_default(),
            last_price: snapshot.last_price,
            bid: snapshot.bid,
            ask: snapshot.ask,
            volume: snapshot.volume,
            timestamp: snapshot.updated,
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct IBMarketDataSnapshot {
    pub conid: i64,
    #[serde(rename = "31")]
    pub last_price: Option<f64>,
    #[serde(rename = "84")]
    pub bid: Option<f64>,
    #[serde(rename = "86")]
    pub ask: Option<f64>,
    #[serde(rename = "87")]
    pub volume: Option<f64>,
    #[serde(rename = "55")]
    pub symbol: Option<String>,
    #[serde(rename = "_updated")]
    pub updated: i64,
}
```

### Order Request

```rust
#[derive(Debug, serde::Serialize)]
pub struct IBOrderRequest {
    pub orders: Vec<IBOrder>,
}

#[derive(Debug, serde::Serialize)]
pub struct IBOrder {
    pub conid: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sec_type: Option<String>,
    pub order_type: String,
    pub side: String,
    pub tif: String,
    pub quantity: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aux_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outside_rth: Option<bool>,
}

// Builder pattern for orders
pub struct IBOrderBuilder {
    conid: i64,
    order_type: String,
    side: String,
    tif: String,
    quantity: f64,
    price: Option<f64>,
    aux_price: Option<f64>,
    outside_rth: Option<bool>,
}

impl IBOrderBuilder {
    pub fn new(conid: i64) -> Self {
        Self {
            conid,
            order_type: "MKT".to_string(),
            side: "BUY".to_string(),
            tif: "DAY".to_string(),
            quantity: 0.0,
            price: None,
            aux_price: None,
            outside_rth: None,
        }
    }

    pub fn limit(mut self, price: f64) -> Self {
        self.order_type = "LMT".to_string();
        self.price = Some(price);
        self
    }

    pub fn stop(mut self, stop_price: f64) -> Self {
        self.order_type = "STP".to_string();
        self.price = Some(stop_price);
        self
    }

    pub fn side(mut self, side: &str) -> Self {
        self.side = side.to_uppercase();
        self
    }

    pub fn quantity(mut self, qty: f64) -> Self {
        self.quantity = qty;
        self
    }

    pub fn tif(mut self, tif: &str) -> Self {
        self.tif = tif.to_uppercase();
        self
    }

    pub fn build(self) -> IBOrder {
        IBOrder {
            conid: self.conid,
            sec_type: Some(format!("{}:STK", self.conid)),
            order_type: self.order_type,
            side: self.side,
            tif: self.tif,
            quantity: self.quantity,
            price: self.price,
            aux_price: self.aux_price,
            outside_rth: self.outside_rth,
        }
    }
}
```

### Position Mapping

```rust
#[derive(Debug, Clone)]
pub struct Position {
    pub symbol: String,
    pub quantity: f64,
    pub avg_price: f64,
    pub market_price: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
}

impl From<IBPosition> for Position {
    fn from(pos: IBPosition) -> Self {
        Self {
            symbol: pos.contract_desc,
            quantity: pos.position,
            avg_price: pos.avg_price,
            market_price: pos.mkt_price,
            unrealized_pnl: pos.unrealized_pnl,
            realized_pnl: pos.realized_pnl,
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct IBPosition {
    pub conid: i64,
    #[serde(rename = "contractDesc")]
    pub contract_desc: String,
    pub position: f64,
    #[serde(rename = "avgPrice")]
    pub avg_price: f64,
    #[serde(rename = "mktPrice")]
    pub mkt_price: f64,
    #[serde(rename = "unrealizedPnl")]
    pub unrealized_pnl: f64,
    #[serde(rename = "realizedPnl")]
    pub realized_pnl: f64,
}
```

## Error Handling Strategy

```rust
#[derive(Debug, thiserror::Error)]
pub enum IBError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Session timeout")]
    SessionTimeout,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Contract not found: {0}")]
    ContractNotFound(String),

    #[error("Order rejected: {0}")]
    OrderRejected(String),

    #[error("Insufficient funds")]
    InsufficientFunds,

    #[error("Market closed")]
    MarketClosed,

    #[error("Invalid price: {0}")]
    InvalidPrice(String),

    #[error("Market data unavailable")]
    MarketDataUnavailable,

    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("JSON parse error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("WebSocket error: {0}")]
    WebSocketError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<IBError> for ExchangeError {
    fn from(err: IBError) -> Self {
        match err {
            IBError::AuthenticationFailed(msg) => ExchangeError::AuthenticationFailed(msg),
            IBError::SessionTimeout => ExchangeError::SessionExpired,
            IBError::RateLimitExceeded => ExchangeError::RateLimitExceeded,
            IBError::ContractNotFound(symbol) => ExchangeError::SymbolNotFound(symbol),
            IBError::OrderRejected(msg) => ExchangeError::OrderRejected(msg),
            IBError::InsufficientFunds => ExchangeError::InsufficientFunds,
            IBError::InvalidPrice(msg) => ExchangeError::InvalidPrice(msg),
            IBError::HttpError(e) => ExchangeError::NetworkError(e.to_string()),
            IBError::JsonError(e) => ExchangeError::ParseError(e.to_string()),
            _ => ExchangeError::Unknown(err.to_string()),
        }
    }
}
```

## Testing Considerations

### Mock Gateway for Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};

    async fn setup_mock_server() -> MockServer {
        let mock_server = MockServer::start().await;

        // Mock auth status endpoint
        Mock::given(method("GET"))
            .and(path("/iserver/auth/status"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({
                    "authenticated": true,
                    "competing": false,
                    "connected": true
                })
            ))
            .mount(&mock_server)
            .await;

        // Mock contract search
        Mock::given(method("POST"))
            .and(path("/iserver/secdef/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!([
                    {
                        "conid": 265598,
                        "symbol": "AAPL",
                        "companyName": "Apple Inc"
                    }
                ])
            ))
            .mount(&mock_server)
            .await;

        mock_server
    }

    #[tokio::test]
    async fn test_contract_search() {
        let mock_server = setup_mock_server().await;
        let connector = IBConnector::new(mock_server.uri());

        let result = connector.search_contract("AAPL", "STK").await;
        assert!(result.is_ok());

        let contracts = result.unwrap();
        assert_eq!(contracts.len(), 1);
        assert_eq!(contracts[0].conid, 265598);
    }
}
```

## Configuration Structure

```rust
#[derive(Debug, Clone, serde::Deserialize)]
pub struct IBConfig {
    /// Base URL (Gateway: https://localhost:5000/v1/api, OAuth: https://api.ibkr.com/v1/api)
    pub base_url: String,

    /// Account ID
    pub account_id: String,

    /// Enable SSL verification (false for localhost Gateway)
    #[serde(default = "default_ssl_verify")]
    pub ssl_verify: bool,

    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,

    /// Tickle interval in seconds
    #[serde(default = "default_tickle_interval")]
    pub tickle_interval_secs: u64,

    /// Rate limit: max requests per second
    #[serde(default = "default_rate_limit")]
    pub rate_limit: usize,
}

fn default_ssl_verify() -> bool {
    false // Default false for Gateway
}

fn default_timeout() -> u64 {
    30
}

fn default_tickle_interval() -> u64 {
    30
}

fn default_rate_limit() -> usize {
    10
}

impl Default for IBConfig {
    fn default() -> Self {
        Self {
            base_url: "https://localhost:5000/v1/api".to_string(),
            account_id: String::new(),
            ssl_verify: false,
            timeout_secs: 30,
            tickle_interval_secs: 30,
            rate_limit: 10,
        }
    }
}
```

## Known Limitations to Handle

### 1. Manual Authentication

**Limitation:** Individual accounts require manual browser login to Gateway

**Implementation Impact:**
- Cannot fully automate startup
- User must authenticate via browser before connector starts
- Document this requirement clearly

**Mitigation:**
```rust
pub async fn wait_for_authentication(&self, timeout_secs: u64) -> Result<(), IBError> {
    let start = Instant::now();
    let timeout = Duration::from_secs(timeout_secs);

    loop {
        let status = self.session_manager.check_auth_status().await?;

        if status.authenticated {
            return Ok(());
        }

        if start.elapsed() > timeout {
            return Err(IBError::AuthenticationFailed(
                "Timeout waiting for authentication. Please login via browser.".into()
            ));
        }

        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}
```

### 2. Contract ID Requirement

**Limitation:** All operations require conid, not symbol

**Implementation Impact:**
- Additional contract search step before trading
- Cache conids to avoid repeated searches
- Handle contract search failures gracefully

**Mitigation:** Implement robust symbol cache (shown above)

### 3. Limited Order Types

**Limitation:** Not all TWS API order types available

**Implementation Impact:**
- Document supported order types
- Return clear errors for unsupported types
- Map generic order types to IB equivalents

**Mapping:**
```rust
pub fn map_order_type(generic_type: &str) -> Result<String, IBError> {
    match generic_type.to_uppercase().as_str() {
        "MARKET" => Ok("MKT".to_string()),
        "LIMIT" => Ok("LMT".to_string()),
        "STOP" => Ok("STP".to_string()),
        "STOP_LIMIT" => Ok("STP_LMT".to_string()),
        "TRAILING_STOP" => Ok("TRAIL".to_string()),
        unsupported => Err(IBError::OrderRejected(
            format!("Order type '{}' not supported by IB Client Portal API", unsupported)
        )),
    }
}
```

### 4. Single Concurrent Session

**Limitation:** Only one session per username

**Implementation Impact:**
- Handle competing session errors
- Cannot run multiple connectors with same account
- Document this limitation

**Detection:**
```rust
pub async fn detect_competing_session(&self) -> Result<bool, IBError> {
    let status = self.session_manager.check_auth_status().await?;
    Ok(status.competing)
}
```

## Performance Optimization

### Connection Pooling

```rust
use reqwest::Client;

pub fn create_http_client(config: &IBConfig) -> Result<Client, IBError> {
    let mut client_builder = Client::builder()
        .timeout(Duration::from_secs(config.timeout_secs))
        .pool_max_idle_per_host(10)
        .pool_idle_timeout(Duration::from_secs(90));

    if !config.ssl_verify {
        // Disable SSL verification for Gateway
        client_builder = client_builder.danger_accept_invalid_certs(true);
    }

    client_builder
        .build()
        .map_err(|e| IBError::Unknown(format!("Failed to create HTTP client: {}", e)))
}
```

### Async Batch Operations

```rust
pub async fn get_multiple_tickers(&self, symbols: &[String]) -> Vec<Result<Ticker, IBError>> {
    // Fetch multiple tickers concurrently
    let futures: Vec<_> = symbols
        .iter()
        .map(|symbol| self.get_ticker(symbol))
        .collect();

    futures::future::join_all(futures).await
}
```

## Logging Strategy

```rust
use tracing::{info, warn, error, debug};

pub async fn place_order(&self, order: &OrderRequest) -> Result<OrderResponse, IBError> {
    info!(
        account = %self.config.account_id,
        symbol = %order.symbol,
        side = %order.side,
        quantity = %order.quantity,
        "Placing order"
    );

    // Rate limit
    self.rate_limiter.acquire().await;
    debug!("Rate limit acquired");

    // Resolve conid
    let conid = match self.symbol_cache.get_conid(&order.symbol).await {
        Some(id) => {
            debug!(symbol = %order.symbol, conid = %id, "Using cached conid");
            id
        }
        None => {
            debug!(symbol = %order.symbol, "Searching for contract");
            // ... search logic
        }
    };

    // Place order
    match self.place_order_internal(conid, order).await {
        Ok(response) => {
            info!(
                order_id = %response.order_id,
                status = %response.order_status,
                "Order placed successfully"
            );
            Ok(response)
        }
        Err(e) => {
            error!(
                symbol = %order.symbol,
                error = %e,
                "Order placement failed"
            );
            Err(e)
        }
    }
}
```

## Documentation Requirements

**User-Facing Documentation:**
1. Gateway setup instructions
2. Authentication process (manual browser login)
3. Account requirements (IBKR Pro, funded, activated)
4. Market data subscription requirements
5. Supported order types
6. Rate limits and best practices
7. Known limitations

**Developer Documentation:**
1. Module structure and organization
2. Trait implementation details
3. Error handling conventions
4. Testing approach
5. Configuration options
6. Performance considerations

## Final Checklist

**Core Functionality:**
- [ ] Session management with tickle loop
- [ ] Contract search and caching
- [ ] Order placement with confirmation flow
- [ ] Order status tracking
- [ ] Position queries
- [ ] Account balance queries
- [ ] Market data snapshot
- [ ] Historical data (OHLCV)
- [ ] WebSocket streaming

**Error Handling:**
- [ ] All HTTP status codes handled
- [ ] Order rejection reasons mapped
- [ ] Session timeout recovery
- [ ] Rate limit handling with backoff
- [ ] WebSocket reconnection logic

**Testing:**
- [ ] Unit tests for parsers
- [ ] Integration tests with mock server
- [ ] Manual testing with live Gateway
- [ ] Error scenario testing
- [ ] Rate limit testing

**Documentation:**
- [ ] Setup guide
- [ ] API reference
- [ ] Examples
- [ ] Known limitations
- [ ] Troubleshooting guide

---

**Research Date:** 2026-01-26
**Target:** V5 Connector Implementation
**Reference Implementation:** KuCoin (v5/src/exchanges/kucoin/)
**Status:** Ready for Implementation

## Sources

- [Web API v1.0 Documentation](https://www.interactivebrokers.com/campus/ibkr-api-page/cpapi-v1/)
- [Web API Documentation](https://www.interactivebrokers.com/campus/ibkr-api-page/webapi-doc/)
- [Client Portal API Documentation](https://interactivebrokers.github.io/cpwebapi/)
- [WebSocket Trading Lesson](https://www.interactivebrokers.com/campus/trading-lessons/websockets/)
- [Trading Web API](https://www.interactivebrokers.com/campus/ibkr-api-page/web-api-trading/)
- [Order Types](https://www.interactivebrokers.com/campus/ibkr-api-page/order-types/)
- [Launching and Authenticating the Gateway](https://www.interactivebrokers.com/campus/trading-lessons/launching-and-authenticating-the-gateway/)
