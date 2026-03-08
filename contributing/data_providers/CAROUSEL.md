# Data Providers Agent Carousel

**Automated system for creating data provider connectors through a sequence of specialized agents.**

**Version:** 1.0 (adapted from Exchange Carousel for data providers)

---

## Quick Links

| Document | Description |
|----------|-------------|
| `prompts/01_research.md` | Phase 1: Research agent prompt |
| `prompts/02_implement.md` | Phase 2: Implementation agent prompt |
| `prompts/03_test.md` | Phase 3: Test agent prompt |
| `prompts/04_debug.md` | Phase 4: Debug agent prompt |
| `MANAGER.md` | Manager guide for 26 providers |
| `DATA_PROVIDERS_CAROUSEL_ANALYSIS.md` | Analysis of differences vs exchanges |

---

## Pipeline Overview

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                    DATA PROVIDER CONNECTOR PIPELINE                           │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                               │
│   Phase 1          Phase 2          Phase 3          Phase 4                 │
│  ┌─────────┐      ┌─────────┐      ┌─────────┐      ┌─────────┐             │
│  │RESEARCH │─────►│IMPLEMENT│─────►│  TEST   │─────►│  DEBUG  │────► DONE   │
│  │         │      │         │      │         │      │         │       ✓     │
│  └─────────┘      └─────────┘      └─────────┘      └────┬────┘             │
│       │                │                │                │                   │
│       │                │                │                │ failures          │
│       ▼                ▼                ▼                └─────────┐        │
│   research/        src/code         tests/                        │        │
│   - api_overview   - REST           - REST integration           ▼        │
│   - endpoints      - WebSocket      - WebSocket                [loop]     │
│   - auth           - Simple API key - Data quality                         │
│   - formats        - UnsupportedOp  - Error handling                       │
│   - tiers/limits   - Parser         - Real data verification              │
│   - data_types                                                              │
│   - coverage                                                                │
│   - websocket                                                               │
│                                                                               │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## Key Differences: Data Providers vs Exchanges

| Aspect | Crypto Exchanges | Data Providers |
|--------|------------------|----------------|
| **Primary Function** | Trading + Data | Data Only (mostly) |
| **Research Files** | 6 files | 8 files (+ api_overview, tiers, data_types, coverage) |
| **Trading Methods** | Required | ❌ UnsupportedOperation |
| **Account/Balance** | Required | ❌ UnsupportedOperation (unless broker) |
| **Authentication** | HMAC signatures | Simple API keys |
| **Symbol Format** | BTC-USDT, BTCUSDT | Varies (AAPL, EUR_USD, series codes) |
| **Rate Limits** | Very strict | Often generous free tiers |
| **Data Types** | Price, orderbook, trades | Depends on provider type |

---

## Reference Implementation

**KuCoin** (crypto exchange) is the reference, but adapt for data providers:

```
src/exchanges/kucoin/      # Exchange pattern
├── mod.rs                 # Exports
├── endpoints.rs           # URLs, endpoint enum
├── auth.rs                # HMAC signature
├── parser.rs              # JSON → domain types
├── connector.rs           # ALL traits implemented
├── websocket.rs           # WebSocket
└── research/              # 6 research files

src/{category}/{provider}/ # Data provider pattern
├── mod.rs                 # Exports
├── endpoints.rs           # URLs, endpoint enum, formatters
├── auth.rs                # SIMPLE API key (not HMAC)
├── parser.rs              # JSON → domain types
├── connector.rs           # Implement what makes sense, UnsupportedOperation for rest
├── websocket.rs           # WebSocket (if available)
└── research/              # 8 research files
```

**Key Adaptation:** Don't force-fit into all traits. Use `UnsupportedOperation` liberally.

---

## Phase 1: Research Agent

### Agent Type
`research-agent`

### Task
**EXHAUSTIVE RESEARCH** - Document EVERYTHING this provider offers.

Unlike crypto exchanges (which are similar), data providers vary wildly.
Map out the ENTIRE API surface, not just standard endpoints.

### Output Files (8 files)

```
src/{CATEGORY}/{PROVIDER}/research/
├── api_overview.md         # Provider info, API type, docs quality
├── endpoints_full.md       # ALL endpoints (not just standard ones)
├── websocket_full.md       # WebSocket details (or "Not Available")
├── authentication.md       # API key/OAuth (simpler than HMAC)
├── tiers_and_limits.md     # Free/paid tiers, rate limits, quotas
├── data_types.md           # CATALOG of all data offered
├── response_formats.md     # Exact JSON examples
└── coverage.md             # Geographic/market coverage
```

### Full Prompt Template

```markdown
Research {PROVIDER} API for V5 connector implementation.

Documentation: {DOCS_URL}
Category: {CATEGORY}

Create folder: src/{CATEGORY}/{PROVIDER}/research/

═══════════════════════════════════════════════════════════════════════════════
FILE 1: api_overview.md
═══════════════════════════════════════════════════════════════════════════════

# {PROVIDER} API Overview

## Provider Information
- Full name: ...
- Website: ...
- Documentation: {DOCS_URL}
- Category: {CATEGORY}

## API Type
- REST: Yes/No (base URL: ...)
- WebSocket: Yes/No (URL: ...)
- GraphQL: Yes/No (endpoint: ...)
- Other protocols: ...

## Base URLs
- Production: https://...
- Testnet/Sandbox: https://... (if exists)
- Regional endpoints: ...
- API version: v2 / v1 / etc.

## Documentation Quality
- Official docs: [URL]
- Quality rating: Excellent / Good / Adequate / Poor
- Code examples: Yes/No (languages: ...)
- OpenAPI/Swagger spec: Available? [URL if yes]
- SDKs available: Python, JavaScript, etc.

## Licensing & Terms
- Free tier: Yes/No
- Paid tiers: Yes/No
- Commercial use: Allowed / Requires license
- Data redistribution: Allowed / Prohibited / Attribution required
- Terms of Service: [URL]

═══════════════════════════════════════════════════════════════════════════════
FILE 2: endpoints_full.md
═══════════════════════════════════════════════════════════════════════════════

**CRITICAL:** Document EVERY endpoint, grouped by category.
**Don't skip anything** - even beta/experimental/paid-only endpoints.

# {PROVIDER} - Complete Endpoint Reference

## Category: Standard Market Data

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v1/price | Current price | Yes | No | 60/min | Real-time |
| GET | /v1/ticker | 24h stats | Yes | No | 60/min | |
| GET | /v1/candles | OHLC bars | Yes | No | 60/min | Max 5000 bars |

## Category: Historical Data
...

## Category: Derivatives Analytics (if applicable)
Document endpoints for:
- Liquidations
- Open Interest
- Funding Rates
- Long/Short Ratios
- Options data

## Category: Fundamental Data (if applicable - stocks)
Document endpoints for:
- Company profiles
- Financial statements
- Earnings
- Dividends
- Analyst ratings

## Category: On-chain Data (if applicable - crypto)
Document endpoints for:
- Wallet balances
- Transactions
- DEX trades
- Token transfers

## Category: Macro/Economic Data (if applicable)
Document endpoints for:
- Interest rates
- GDP
- Inflation metrics
- Economic calendars

## Category: Metadata
Document endpoints for:
- Symbol/instrument lists
- Exchange information
- Market hours
- Reference data

## Parameters Reference
For complex endpoints, document all parameters with types, defaults, descriptions.

═══════════════════════════════════════════════════════════════════════════════
FILE 3: websocket_full.md
═══════════════════════════════════════════════════════════════════════════════

**If WebSocket NOT available:** Create file with "WebSocket: Not Available" and skip.

# {PROVIDER} - WebSocket Documentation

## Availability: Yes / No

## Connection
### URLs
- Public streams: wss://...
- Private streams: wss://...

### Connection Process
1. Connect to URL
2. Handshake: ...
3. Welcome message: ...
4. Auth: ...

## ALL Available Channels/Topics

**CRITICAL:** List EVERY channel, don't skip specialized ones.

| Channel/Topic | Type | Description | Auth? | Free? | Update Frequency | Example |
|---------------|------|-------------|-------|-------|------------------|---------|
| ticker | Public | Price updates | No | Yes | Real-time | {"type":"subscribe","channel":"ticker","symbol":"AAPL"} |
| trades | Public | Trade updates | No | Yes | Real-time | ... |
| orderbook | Public | L2 updates | No | Paid | Real-time | ... |

## Subscription Format
Document subscribe/unsubscribe messages with exact JSON examples.

## Message Formats (for EVERY channel)
Document exact JSON format for each channel's updates.

## Heartbeat / Ping-Pong

**CRITICAL:** Document exactly!

### Who initiates?
- Server → Client ping: Yes/No
- Client → Server ping: Yes/No

### Message Format
- Binary ping/pong frames: Yes/No
- Text messages: "ping"/"pong"
- JSON messages: {"op":"ping"}

### Timing
- Ping interval: X seconds
- Timeout: X seconds
- Client must send: Every X seconds

### Example
```
Server → Client: {"op":"ping","ts":1234567890}
Client → Server: {"op":"pong","ts":1234567890}
```

═══════════════════════════════════════════════════════════════════════════════
FILE 4: authentication.md
═══════════════════════════════════════════════════════════════════════════════

# {PROVIDER} - Authentication

## Public Endpoints
- Public endpoints exist: Yes/No
- Require authentication: Yes/No
- Rate limits without auth: X req/min

## API Key

### Required For
- All endpoints: Yes/No
- Paid tier only: Yes/No
- Rate limit increase: Yes/No

### How to Obtain
- Sign up: [URL]
- API key management: [URL]
- Free tier includes key: Yes/No

### API Key Format
- Header: `X-API-Key: your_api_key_here`
- OR Query param: `?apiKey=xxx`
- OR Bearer token: `Authorization: Bearer xxx`

### Multiple Keys
- Multiple keys allowed: Yes/No
- Rate limits per key: Yes/No

## OAuth (if applicable)
Document OAuth 2.0 flow if provider uses it.

## Signature/HMAC (if applicable)
**Usually NOT needed** - most data providers use simple API keys.
Only document if provider requires signature.

## Authentication Examples

### REST with API Key
```bash
curl -H "X-API-Key: your_key" https://api.example.com/v1/price?symbol=AAPL
```

### WebSocket with API Key
```javascript
const ws = new WebSocket('wss://ws.example.com?apiKey=your_key');
```

═══════════════════════════════════════════════════════════════════════════════
FILE 5: tiers_and_limits.md
═══════════════════════════════════════════════════════════════════════════════

**CRITICAL:** This is very important - impacts what we can do.

# {PROVIDER} - Tiers, Pricing, and Rate Limits

## Free Tier

### Access Level
- Requires sign-up: Yes/No
- API key required: Yes/No
- Credit card required: No (hopefully)

### Rate Limits
- Requests per second: X
- Requests per minute: X
- Requests per hour: X
- Requests per day: X
- Burst allowed: Yes/No (X requests burst)

### Data Access
- Real-time data: Yes/No
- Delayed data: Yes/No (delay: 15min, 1h, etc.)
- Historical data: Yes/No (depth: X months/years)
- WebSocket: Allowed (Yes/No, limits: X connections)
- Data types: (list what's available)

### Limitations
- Symbols: Limited to X / Unlimited
- Endpoints: Some restricted / All available
- Features: (list restrictions)

## Paid Tiers

| Tier Name | Price | Rate Limit | Additional Features | WebSocket | Historical | Support |
|-----------|-------|------------|---------------------|-----------|------------|---------|
| Free | $0 | 60/min | Basic data | 1 conn | 1 year | Community |
| Starter | $29/mo | 300/min | + Real-time | 5 conn | 5 years | Email |
| Professional | $99/mo | 1000/min | + Extended data | Unlimited | Unlimited | Priority |
| Enterprise | Contact | Custom | Everything | Custom | Unlimited | Dedicated |

## Rate Limit Details

### How Measured
- Window: Per second / minute / hour
- Rolling window: Yes/No
- Fixed window: Yes/No

### Limit Scope
- Per IP address: Yes/No
- Per API key: Yes/No
- Shared across: ...

### Response Headers
```
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 45
X-RateLimit-Reset: 1234567890
Retry-After: 30 (on 429 error)
```

### Error Response (HTTP 429)
```json
{
  "error": "Rate limit exceeded",
  "limit": 60,
  "remaining": 0,
  "reset": 1234567890
}
```

## WebSocket Specific Limits
- Max connections per IP: X
- Max subscriptions per connection: X
- Message rate limits: X msg/sec

═══════════════════════════════════════════════════════════════════════════════
FILE 6: data_types.md
═══════════════════════════════════════════════════════════════════════════════

**CRITICAL:** Catalog EVERYTHING this provider offers.

# {PROVIDER} - Data Types Catalog

## Standard Market Data
- [x] Current Price
- [x] Bid/Ask Spread
- [x] 24h Ticker Stats
- [x] OHLC/Candlesticks
- [ ] Level 2 Orderbook
- [x] Recent Trades
- [x] Volume (24h, intraday)

## Historical Data
- [x] Historical prices (depth: X years)
- [x] Minute bars
- [x] Daily bars
- [ ] Tick data
- [x] Adjusted prices

## Derivatives Data (Crypto/Futures)
If applicable:
- [x] Open Interest
- [x] Funding Rates
- [x] Liquidations
- [x] Long/Short Ratios
- [x] Mark Price
- [x] Index Price

## Options Data (if applicable)
- [x] Options Chains
- [x] Implied Volatility
- [x] Greeks
- [x] Open Interest per strike

## Fundamental Data (Stocks)
If applicable:
- [x] Company Profile
- [x] Financial Statements
- [x] Earnings
- [x] Dividends
- [x] Analyst Ratings
- [x] Insider Trading
- [x] Financial Ratios

## On-chain Data (Crypto)
If applicable:
- [x] Wallet Balances
- [x] Transaction History
- [x] DEX Trades
- [x] Token Transfers
- [x] Smart Contract Events

## Macro/Economic Data (Economics)
If applicable:
- [x] Interest Rates
- [x] GDP
- [x] Inflation (CPI, PPI)
- [x] Employment (NFP, unemployment)
- [x] Economic Calendar

## Forex Specific
If applicable:
- [x] Currency Pairs
- [x] Bid/Ask Spreads
- [x] Historical FX rates

## Metadata & Reference
- [x] Symbol/Instrument Lists
- [x] Exchange Information
- [x] Market Hours
- [x] Trading Calendars
- [x] Timezone Info

## Unique/Custom Data
**What makes this provider special?**
Document any unique data this provider offers.

═══════════════════════════════════════════════════════════════════════════════
FILE 7: response_formats.md
═══════════════════════════════════════════════════════════════════════════════

**EXACT JSON examples from official docs** - don't invent.

# {PROVIDER} - Response Formats

## For EVERY important endpoint

### GET /v1/price
```json
{
  "symbol": "AAPL",
  "price": 150.25,
  "timestamp": 1234567890000
}
```

### GET /v1/ticker
```json
{
  "symbol": "AAPL",
  "last": 150.25,
  "bid": 150.24,
  "ask": 150.26,
  "high_24h": 152.50,
  "low_24h": 148.00,
  "volume_24h": 12345678,
  "change_24h": 2.50,
  "change_percent_24h": 1.69,
  "timestamp": 1234567890000
}
```

### GET /v1/candles
```json
[
  {
    "timestamp": 1234567890000,
    "open": 150.00,
    "high": 150.50,
    "low": 149.80,
    "close": 150.25,
    "volume": 1234567
  }
]
```

Document for EVERY endpoint category with EXACT field names from docs.

═══════════════════════════════════════════════════════════════════════════════
FILE 8: coverage.md
═══════════════════════════════════════════════════════════════════════════════

# {PROVIDER} - Data Coverage

## Geographic Coverage
### Regions Supported
- North America: Yes/No
- Europe: Yes/No
- Asia: Yes/No

### Country-Specific
- US: Yes/No
- UK: Yes/No
- Japan: Yes/No
- India: Yes/No

### Restricted Regions
- Blocked countries: ...
- VPN detection: Yes/No

## Markets/Exchanges Covered

### Stock Markets
- US: NYSE, NASDAQ, AMEX (Yes/No)
- UK: LSE (Yes/No)
- Japan: TSE (Yes/No)
- India: NSE, BSE (Yes/No)

### Crypto Exchanges (if aggregator)
- Binance: Yes/No
- Coinbase: Yes/No
- Kraken: Yes/No
(list ALL exchanges aggregated)

### Forex Brokers (if aggregator)
(list)

## Instrument Coverage

### Stocks
- Total symbols: ~X,XXX
- US stocks: X,XXX
- International: X,XXX
- OTC: Yes/No

### Crypto
- Total coins: XXX
- Spot pairs: XXX
- Futures: XXX

### Forex
- Currency pairs: XX
- Majors: 7 pairs
- Minors: ~XX pairs
- Exotics: ~XX pairs

## Data History
### Historical Depth
- Stocks: From year XXXX (X years)
- Crypto: From year XXXX
- Forex: From year XXXX

### Granularity Available
- Tick data: Yes/No
- 1-minute bars: Yes/No
- Daily: Yes/No (depth: X years)

### Real-time vs Delayed
- Real-time: Yes/No (free tier?)
- Delayed: Yes/No (delay: 15min, 1h, EOD)

## Data Quality
### Accuracy
- Source: Direct from exchange / Aggregated / Calculated
- Validation: Yes/No

### Completeness
- Missing data: Common / Rare
- Gaps: How handled?
```

### Exit Criteria

- [x] All 8 research files created
- [x] Every file has EXACT data from official docs (no guessing)
- [x] All endpoints documented (including specialized ones)
- [x] All data types cataloged
- [x] Tier/pricing clearly documented
- [x] WebSocket documented (or noted as unavailable)
- [x] Coverage/limits understood

---

## Phase 2: Implementation Agent

### Agent Type
`rust-implementer`

### Task
Implement Rust connector for {PROVIDER} based on research documentation.

**Key Difference from Exchanges:**
- ❌ DON'T force-fit into all traits
- ✅ Implement what makes sense
- ✅ Return `UnsupportedOperation` for irrelevant methods
- ✅ Focus on DATA access (REST + WebSocket)

### Output Files (5-6 files)

```
src/{CATEGORY}/{PROVIDER}/
├── mod.rs          # Module exports
├── endpoints.rs    # URLs, endpoint enum, formatters
├── auth.rs         # Authentication (usually simple API key)
├── parser.rs       # JSON → domain types
├── connector.rs    # Trait implementations
└── websocket.rs    # WebSocket connector (if WS available)
```

### Full Prompt Template

```markdown
Implement {PROVIDER} connector for V5 architecture.

═══════════════════════════════════════════════════════════════════════════════
REFERENCE
═══════════════════════════════════════════════════════════════════════════════

Reference implementation: src/exchanges/kucoin/
Research docs: src/{CATEGORY}/{PROVIDER}/research/

Study KuCoin code carefully, but ADAPT for data providers:
- Simpler authentication (API key, not HMAC)
- UnsupportedOperation for trading methods
- Symbol formatting depends on provider type

═══════════════════════════════════════════════════════════════════════════════
FILE 1: mod.rs
═══════════════════════════════════════════════════════════════════════════════

//! {PROVIDER} connector
//!
//! Category: {CATEGORY}
//! Type: [Data Provider / Broker / Aggregator]
//!
//! ## Features
//! - REST API: Yes/No
//! - WebSocket: Yes/No
//! - Authentication: API Key / OAuth / None
//! - Free tier: Yes/No

mod endpoints;
mod auth;
mod parser;
mod connector;

#[cfg(feature = "websocket")]
mod websocket;

pub use connector::{ProviderNameConnector};

#[cfg(feature = "websocket")]
pub use websocket::{ProviderNameWebSocket};

═══════════════════════════════════════════════════════════════════════════════
FILE 2: endpoints.rs
═══════════════════════════════════════════════════════════════════════════════

//! {PROVIDER} API endpoints

/// Base URLs
pub struct ProviderNameEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for ProviderNameEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://api.example.com",  // from api_overview.md
            ws_base: Some("wss://ws.example.com"), // or None
        }
    }
}

/// API endpoint enum
#[derive(Debug, Clone)]
pub enum ProviderNameEndpoint {
    // Standard market data
    Price,
    Ticker,
    Candles,
    Symbols,

    // Extended endpoints (from endpoints_full.md)
    Liquidations,  // if applicable
    Fundamentals,  // if applicable
    // ... Add ALL endpoints from research
}

impl ProviderNameEndpoint {
    pub fn path(&self) -> &'static str {
        match self {
            Self::Price => "/v1/price",
            Self::Ticker => "/v1/ticker",
            // ... map all endpoints from research
        }
    }
}

/// Format symbol/ticker for API
///
/// Adapt based on provider type:
/// - Stocks: "AAPL" (just base)
/// - Forex: "EUR_USD" or "EUR/USD"
/// - Crypto: "BTCUSDT" or "BTC-USDT"
pub fn format_symbol(symbol: &crate::core::types::Symbol) -> String {
    // Check data_formats.md research for correct format
    match {CATEGORY} {
        "stocks" => symbol.base.to_uppercase(),
        "forex" => format!("{}_{}", symbol.base, symbol.quote),
        _ => format!("{}{}", symbol.base, symbol.quote),
    }
}

═══════════════════════════════════════════════════════════════════════════════
FILE 3: auth.rs
═══════════════════════════════════════════════════════════════════════════════

//! {PROVIDER} authentication
//!
//! Authentication type: [API Key / OAuth / None]
//! (from authentication.md research)

use std::collections::HashMap;

#[derive(Clone)]
pub struct ProviderNameAuth {
    pub api_key: Option<String>,
}

impl ProviderNameAuth {
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("PROVIDER_API_KEY").ok(),
        }
    }

    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(api_key.into()),
        }
    }

    /// Add authentication headers to request
    pub fn sign_headers(&self, headers: &mut HashMap<String, String>) {
        if let Some(key) = &self.api_key {
            // Check authentication.md for correct header name
            headers.insert("X-API-Key".to_string(), key.clone());
            // OR: headers.insert("Authorization".to_string(), format!("Bearer {}", key));
        }
    }

    /// Add authentication to query params (if provider uses this)
    pub fn sign_query(&self, params: &mut HashMap<String, String>) {
        if let Some(key) = &self.api_key {
            params.insert("apiKey".to_string(), key.clone());
        }
    }
}

═══════════════════════════════════════════════════════════════════════════════
FILE 4: parser.rs
═══════════════════════════════════════════════════════════════════════════════

//! {PROVIDER} response parsers
//!
//! Parse JSON responses to domain types based on response_formats.md

use serde_json::Value;
use crate::core::types::*;
use crate::core::error::{ExchangeError, ExchangeResult};

pub struct ProviderNameParser;

impl ProviderNameParser {
    // ═══════════════════════════════════════════════════════════════════════
    // STANDARD MARKET DATA
    // ═══════════════════════════════════════════════════════════════════════

    pub fn parse_price(response: &Value) -> ExchangeResult<f64> {
        response
            .get("price")  // Check response_formats.md for exact field name
            .and_then(|v| v.as_f64())
            .ok_or_else(|| ExchangeError::Parse("Missing 'price' field".to_string()))
    }

    pub fn parse_ticker(response: &Value, symbol: &str) -> ExchangeResult<Ticker> {
        Ok(Ticker {
            symbol: symbol.to_string(),
            last_price: Self::require_f64(response, "last")?,
            bid_price: Self::get_f64(response, "bid"),
            ask_price: Self::get_f64(response, "ask"),
            high_24h: Self::get_f64(response, "high_24h"),
            low_24h: Self::get_f64(response, "low_24h"),
            volume_24h: Self::get_f64(response, "volume_24h"),
            quote_volume_24h: Self::get_f64(response, "quote_volume_24h"),
            price_change_24h: Self::get_f64(response, "change_24h"),
            price_change_percent_24h: Self::get_f64(response, "change_percent_24h"),
            timestamp: Self::require_i64(response, "timestamp")?,
        })
    }

    pub fn parse_klines(response: &Value) -> ExchangeResult<Vec<Kline>> {
        let array = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array".to_string()))?;

        array.iter().map(|candle| {
            Ok(Kline {
                open_time: Self::require_i64(candle, "timestamp")?,
                open: Self::require_f64(candle, "open")?,
                high: Self::require_f64(candle, "high")?,
                low: Self::require_f64(candle, "low")?,
                close: Self::require_f64(candle, "close")?,
                volume: Self::require_f64(candle, "volume")?,
                quote_volume: Self::get_f64(candle, "quote_volume"),
                close_time: None,
                trades: None,
            })
        }).collect()
    }

    pub fn parse_orderbook(response: &Value) -> ExchangeResult<OrderBook> {
        // NOTE: Many data providers DON'T provide orderbook
        // Check data_types.md - if not available, connector should return UnsupportedOperation
        let bids = Self::parse_order_levels(response.get("bids"))?;
        let asks = Self::parse_order_levels(response.get("asks"))?;

        Ok(OrderBook {
            bids,
            asks,
            timestamp: Self::require_i64(response, "timestamp")?,
            sequence: Self::get_str(response, "sequence").map(|s| s.to_string()),
        })
    }

    pub fn parse_symbols(response: &Value) -> ExchangeResult<Vec<String>> {
        let array = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array".to_string()))?;

        Ok(array.iter()
            .filter_map(|v| v.get("symbol").and_then(|s| s.as_str()))
            .map(|s| s.to_string())
            .collect())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // EXTENDED DATA TYPES (from data_types.md)
    // ═══════════════════════════════════════════════════════════════════════

    // Add parsers for provider-specific data types:
    // - Liquidations (derivatives feeds)
    // - Fundamentals (stock providers)
    // - Macro data (economic feeds)
    // - On-chain data (crypto feeds)

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    fn require_f64(obj: &Value, field: &str) -> ExchangeResult<f64> {
        obj.get(field)
            .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn get_f64(obj: &Value, field: &str) -> Option<f64> {
        obj.get(field)
            .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
    }

    fn require_i64(obj: &Value, field: &str) -> ExchangeResult<i64> {
        obj.get(field)
            .and_then(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn get_i64(obj: &Value, field: &str) -> Option<i64> {
        obj.get(field)
            .and_then(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
    }

    fn get_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field).and_then(|v| v.as_str())
    }

    fn parse_order_levels(value: Option<&Value>) -> ExchangeResult<Vec<(f64, f64)>> {
        let array = value
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Invalid order levels".to_string()))?;

        array.iter().map(|level| {
            let arr = level.as_array()
                .ok_or_else(|| ExchangeError::Parse("Invalid level format".to_string()))?;

            let price = arr.get(0)
                .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
                .ok_or_else(|| ExchangeError::Parse("Invalid price".to_string()))?;

            let size = arr.get(1)
                .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
                .ok_or_else(|| ExchangeError::Parse("Invalid size".to_string()))?;

            Ok((price, size))
        }).collect()
    }
}

═══════════════════════════════════════════════════════════════════════════════
FILE 5: connector.rs
═══════════════════════════════════════════════════════════════════════════════

**CRITICAL:** This is where trait implementation decisions happen.

//! {PROVIDER} connector implementation

use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::*;
use crate::core::error::{ExchangeError, ExchangeResult};
use crate::core::traits::*;

use super::endpoints::*;
use super::auth::*;
use super::parser::*;

pub struct ProviderNameConnector {
    client: Client,
    auth: ProviderNameAuth,
    endpoints: ProviderNameEndpoints,
    testnet: bool,
}

impl ProviderNameConnector {
    pub fn new(auth: ProviderNameAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: ProviderNameEndpoints::default(),
            testnet: false,
        }
    }

    pub fn from_env() -> Self {
        Self::new(ProviderNameAuth::from_env())
    }

    async fn get(
        &self,
        endpoint: ProviderNameEndpoint,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        // Add authentication
        let mut headers = HashMap::new();
        self.auth.sign_headers(&mut headers);

        let mut request = self.client.get(&url);

        for (key, value) in headers {
            request = request.header(key, value);
        }

        if !params.is_empty() {
            request = request.query(&params);
        }

        let response = request.send().await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ExchangeError::Api(format!("HTTP {}", response.status())));
        }

        response.json().await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: ExchangeIdentity (ALWAYS implement)
// ═══════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for ProviderNameConnector {
    fn exchange_name(&self) -> &'static str {
        "{PROVIDER}"
    }

    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::ProviderName  // Add to ExchangeId enum
    }

    fn is_testnet(&self) -> bool {
        self.testnet
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        vec![AccountType::Spot]  // Data providers usually only support Spot equivalent
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: MarketData (Implement what makes sense)
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for ProviderNameConnector {
    async fn get_price(&self, symbol: Symbol, _account_type: AccountType) -> ExchangeResult<Price> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol));

        let response = self.get(ProviderNameEndpoint::Price, params).await?;
        let price = ProviderNameParser::parse_price(&response)?;

        Ok(price)
    }

    async fn get_ticker(&self, symbol: Symbol, _account_type: AccountType) -> ExchangeResult<Ticker> {
        let symbol_str = format_symbol(&symbol);
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol_str.clone());

        let response = self.get(ProviderNameEndpoint::Ticker, params).await?;
        ProviderNameParser::parse_ticker(&response, &symbol_str)
    }

    async fn get_orderbook(
        &self,
        _symbol: Symbol,
        _depth: Option<u16>,
        _account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        // Check data_types.md research:
        // If orderbook is NOT available, return:
        Err(ExchangeError::UnsupportedOperation(
            "{PROVIDER} does not provide orderbook data - data feed only".to_string()
        ))

        // If available, implement:
        // let response = self.get(...).await?;
        // ProviderNameParser::parse_orderbook(&response)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Kline>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol));
        params.insert("interval".to_string(), interval.to_string());
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }

        let response = self.get(ProviderNameEndpoint::Candles, params).await?;
        ProviderNameParser::parse_klines(&response)
    }

    async fn get_symbols(&self, _account_type: AccountType) -> ExchangeResult<Vec<String>> {
        let response = self.get(ProviderNameEndpoint::Symbols, HashMap::new()).await?;
        ProviderNameParser::parse_symbols(&response)
    }

    async fn get_all_tickers(&self) -> ExchangeResult<Vec<Ticker>> {
        Err(ExchangeError::UnsupportedOperation(
            "get_all_tickers not implemented for this provider".to_string()
        ))
    }

    async fn get_funding_rate(&self, _symbol: Symbol, _account_type: AccountType) -> ExchangeResult<FundingRate> {
        Err(ExchangeError::UnsupportedOperation(
            "Funding rate not available - not a derivatives platform".to_string()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Trading (Usually UnsupportedOperation for data providers)
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for ProviderNameConnector {
    async fn place_order(&self, _order: Order) -> ExchangeResult<OrderResult> {
        Err(ExchangeError::UnsupportedOperation(
            "{PROVIDER} is a data provider - trading not supported".to_string()
        ))
    }

    async fn cancel_order(&self, _order_id: &str, _symbol: Option<Symbol>, _account_type: AccountType) -> ExchangeResult<OrderResult> {
        Err(ExchangeError::UnsupportedOperation(
            "{PROVIDER} is a data provider - trading not supported".to_string()
        ))
    }

    async fn get_order(&self, _order_id: &str, _symbol: Option<Symbol>, _account_type: AccountType) -> ExchangeResult<OrderResult> {
        Err(ExchangeError::UnsupportedOperation(
            "{PROVIDER} is a data provider - trading not supported".to_string()
        ))
    }

    async fn get_open_orders(&self, _symbol: Option<Symbol>, _account_type: AccountType) -> ExchangeResult<Vec<OrderResult>> {
        Err(ExchangeError::UnsupportedOperation(
            "{PROVIDER} is a data provider - trading not supported".to_string()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Account (Usually UnsupportedOperation unless broker)
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for ProviderNameConnector {
    async fn get_balance(&self, _account_type: AccountType) -> ExchangeResult<Vec<Balance>> {
        // If provider is a BROKER (Alpaca, OANDA, Zerodha): Implement
        // If provider is DATA ONLY: UnsupportedOperation
        Err(ExchangeError::UnsupportedOperation(
            "{PROVIDER} is a data provider - account operations not supported".to_string()
        ))
    }

    async fn get_account_info(&self, _account_type: AccountType) -> ExchangeResult<AccountInfo> {
        Err(ExchangeError::UnsupportedOperation(
            "{PROVIDER} is a data provider - account operations not supported".to_string()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Positions (Usually UnsupportedOperation unless broker)
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for ProviderNameConnector {
    async fn get_positions(&self, _account_type: AccountType) -> ExchangeResult<Vec<Position>> {
        Err(ExchangeError::UnsupportedOperation(
            "{PROVIDER} is a data provider - position tracking not supported".to_string()
        ))
    }

    async fn get_position(&self, _symbol: Symbol, _account_type: AccountType) -> ExchangeResult<Position> {
        Err(ExchangeError::UnsupportedOperation(
            "{PROVIDER} is a data provider - position tracking not supported".to_string()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// EXTENDED METHODS (Provider-specific, not from traits)
// ═══════════════════════════════════════════════════════════════════════════

impl ProviderNameConnector {
    // Add provider-specific methods from data_types.md:
    // - get_liquidations() for derivatives feeds
    // - get_company_profile() for stock providers
    // - get_economic_series() for macro data providers
}

═══════════════════════════════════════════════════════════════════════════════
FILE 6: websocket.rs (Optional - only if WS available)
═══════════════════════════════════════════════════════════════════════════════

**Skip if WebSocket not available** (check api_overview.md).

Follow KuCoin websocket.rs pattern but adapt for provider-specific format from websocket_full.md.

═══════════════════════════════════════════════════════════════════════════════
AFTER EACH FILE
═══════════════════════════════════════════════════════════════════════════════

cargo check --package digdigdig3

═══════════════════════════════════════════════════════════════════════════════
FINALLY: Add to src/{CATEGORY}/mod.rs
═══════════════════════════════════════════════════════════════════════════════

pub mod {provider};
```

### Exit Criteria

- [x] All 5-6 files created
- [x] `cargo check --package digdigdig3` passes
- [x] Provider added to category mod.rs
- [x] ExchangeId variant added

---

## Phase 3: Test Agent

### Agent Type
`rust-implementer`

### Task
Create integration and WebSocket tests for {PROVIDER} connector.

**Key Differences from Exchange Tests:**
- ❌ NO trading tests
- ❌ NO account tests (unless broker)
- ✅ Focus on data retrieval
- ✅ Verify data quality (realistic prices, valid responses)
- ✅ Graceful handling of rate limits and errors

### Output Files (1-2 files)

```
tests/
├── {provider}_integration.rs    # REST API tests (ALWAYS)
└── {provider}_websocket.rs       # WebSocket tests (if WS available)
```

### Full Prompt Template

```markdown
Write comprehensive tests for {PROVIDER} connector.

═══════════════════════════════════════════════════════════════════════════════
REFERENCES
═══════════════════════════════════════════════════════════════════════════════

REST tests reference: tests/kucoin_integration.rs
WebSocket tests reference: tests/kucoin_websocket.rs

BUT adapt for data providers - NO trading/account tests.

═══════════════════════════════════════════════════════════════════════════════
FILE 1: tests/{provider}_integration.rs
═══════════════════════════════════════════════════════════════════════════════

//! {PROVIDER} integration tests
//!
//! NOTE: These tests make REAL API calls.
//! - Rate limits apply
//! - API key may be required (set PROVIDER_API_KEY env var)

#[cfg(test)]
mod tests {
    use digdigdig3::core::types::*;
    use digdigdig3::core::traits::*;
    use digdigdig3::{CATEGORY}::{PROVIDER}::*;

    fn create_connector() -> ProviderNameConnector {
        ProviderNameConnector::from_env()
    }

    fn test_symbol() -> Symbol {
        // Adapt to provider type
        match "{CATEGORY}" {
            "stocks" => Symbol { base: "AAPL".to_string(), quote: "USD".to_string() },
            "forex" => Symbol { base: "EUR".to_string(), quote: "USD".to_string() },
            _ => Symbol { base: "BTC".to_string(), quote: "USDT".to_string() },
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // IDENTITY TESTS
    // ═══════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_exchange_identity() {
        let connector = create_connector();
        assert_eq!(connector.exchange_name(), "{PROVIDER}");
        println!("✓ Exchange name: {}", connector.exchange_name());
    }

    // ═══════════════════════════════════════════════════════════════════════
    // MARKET DATA TESTS
    // ═══════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_get_price() {
        let connector = create_connector();
        let symbol = test_symbol();

        match connector.get_price(symbol.clone(), AccountType::Spot).await {
            Ok(price) => {
                println!("✓ Price for {}/{}: ${}", symbol.base, symbol.quote, price);

                // Validate price is realistic
                match "{CATEGORY}" {
                    "stocks" => {
                        assert!(price > 1.0 && price < 10000.0, "Stock price unrealistic");
                    }
                    "forex" => {
                        assert!(price > 0.01 && price < 1000.0, "FX rate unrealistic");
                    }
                    _ => {
                        assert!(price > 0.0, "Price must be positive");
                    }
                }
            }
            Err(e) => {
                println!("⚠ Price test failed: {:?}", e);
                println!("  This may be due to:");
                println!("  - Missing API key (set PROVIDER_API_KEY env var)");
                println!("  - Rate limit (free tier exhausted)");
                println!("  - Network issue");
                println!("✓ Test completed (with expected error)");
            }
        }
    }

    #[tokio::test]
    async fn test_get_ticker() {
        let connector = create_connector();
        let symbol = test_symbol();

        match connector.get_ticker(symbol.clone(), AccountType::Spot).await {
            Ok(ticker) => {
                println!("✓ Ticker for {}/{}:", symbol.base, symbol.quote);
                println!("  Last: ${}", ticker.last_price);
                println!("  Volume 24h: {:?}", ticker.volume_24h);

                assert!(ticker.last_price > 0.0);
                if let (Some(bid), Some(ask)) = (ticker.bid_price, ticker.ask_price) {
                    assert!(bid < ask, "Bid must be < Ask");
                }
            }
            Err(e) => {
                println!("⚠ Ticker test failed: {:?}", e);
                println!("✓ Test completed (with expected error)");
            }
        }
    }

    #[tokio::test]
    async fn test_get_klines() {
        let connector = create_connector();
        let symbol = test_symbol();

        match connector.get_klines(symbol.clone(), "1h", Some(10), AccountType::Spot).await {
            Ok(klines) => {
                println!("✓ Retrieved {} klines", klines.len());
                assert!(!klines.is_empty());

                if let Some(first) = klines.first() {
                    // Validate OHLC relationships
                    assert!(first.high >= first.low);
                    assert!(first.high >= first.open);
                    assert!(first.high >= first.close);
                    assert!(first.low <= first.open);
                    assert!(first.low <= first.close);
                }
            }
            Err(e) => {
                println!("⚠ Klines test failed: {:?}", e);
                println!("✓ Test completed (with expected error)");
            }
        }
    }

    #[tokio::test]
    async fn test_get_orderbook() {
        let connector = create_connector();
        let symbol = test_symbol();

        match connector.get_orderbook(symbol.clone(), Some(10), AccountType::Spot).await {
            Ok(orderbook) => {
                println!("✓ Orderbook retrieved");
                if let (Some(best_bid), Some(best_ask)) = (orderbook.bids.first(), orderbook.asks.first()) {
                    assert!(best_bid.0 < best_ask.0, "Bid must be < Ask");
                }
            }
            Err(ExchangeError::UnsupportedOperation(msg)) => {
                println!("✓ Orderbook not supported: {}", msg);
            }
            Err(e) => {
                println!("⚠ Orderbook test failed: {:?}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_get_symbols() {
        let connector = create_connector();

        match connector.get_symbols(AccountType::Spot).await {
            Ok(symbols) => {
                println!("✓ Retrieved {} symbols", symbols.len());
                assert!(!symbols.is_empty());
            }
            Err(e) => {
                println!("⚠ Symbols test failed: {:?}", e);
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // TRADING TESTS (should return UnsupportedOperation)
    // ═══════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_trading_not_supported() {
        let connector = create_connector();

        let order = Order {
            symbol: test_symbol(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            quantity: 1.0,
            price: Some(100.0),
            ..Default::default()
        };

        match connector.place_order(order).await {
            Err(ExchangeError::UnsupportedOperation(msg)) => {
                println!("✓ Trading correctly marked as unsupported");
                println!("  Message: {}", msg);
            }
            Ok(_) => {
                panic!("Trading should not be supported for data providers!");
            }
            Err(e) => {
                println!("⚠ Unexpected error: {:?}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_account_not_supported() {
        let connector = create_connector();

        match connector.get_balance(AccountType::Spot).await {
            Err(ExchangeError::UnsupportedOperation(msg)) => {
                println!("✓ Account operations correctly marked as unsupported");
            }
            Ok(_) => {
                println!("⚠ Account supported - this may be a broker API");
            }
            Err(e) => {
                println!("⚠ Unexpected error: {:?}", e);
            }
        }
    }
}

═══════════════════════════════════════════════════════════════════════════════
FILE 2: tests/{provider}_websocket.rs (if WS available)
═══════════════════════════════════════════════════════════════════════════════

**Skip if WebSocket not available.**

Follow kucoin_websocket.rs pattern with graceful error handling.

═══════════════════════════════════════════════════════════════════════════════
RUN TESTS
═══════════════════════════════════════════════════════════════════════════════

# REST tests
cargo test --package digdigdig3 --test {provider}_integration -- --nocapture

# WebSocket tests
cargo test --package digdigdig3 --test {provider}_websocket -- --nocapture
```

### Exit Criteria

- [x] Test files created
- [x] Tests compile
- [x] Tests use graceful error handling
- [x] Clear console output

---

## Phase 4: Debug Agent

### Agent Type
`rust-implementer`

### Task
Debug and fix tests until they return REAL data from {PROVIDER}.

**Goal:** All tests should either:
- ✅ Return real data (prices, tickers, events)
- ✅ Return UnsupportedOperation (for trading/account methods)
- ✅ Handle errors gracefully (API key, rate limits)

**NOT acceptable:**
- ❌ Tests panic or crash
- ❌ Tests return fake/stub data
- ❌ Compilation errors

### Full Prompt Template

```markdown
Debug and fix failing tests for {PROVIDER} connector.

═══════════════════════════════════════════════════════════════════════════════
PROCESS
═══════════════════════════════════════════════════════════════════════════════

1. Run all tests:
   cargo test --package digdigdig3 --test {provider}_integration -- --nocapture
   cargo test --package digdigdig3 --test {provider}_websocket -- --nocapture

2. For EACH failure, identify error type and fix:

═══════════════════════════════════════════════════════════════════════════════
COMMON ERRORS AND FIXES
═══════════════════════════════════════════════════════════════════════════════

## Missing API Key (401 Unauthorized)
Location: auth.rs

Fix:
1. Set environment variable: export PROVIDER_API_KEY="xxx"
2. Verify auth.rs sign_headers() uses correct header name
3. Check authentication.md for exact format

## Rate Limit Exceeded (HTTP 429)
Location: connector.rs

Fix:
1. Check tiers_and_limits.md for actual limits
2. Add delays between tests:
   tokio::time::sleep(Duration::from_millis(1000)).await;
3. Tests should handle 429 gracefully

## Wrong Endpoint URL (404 Not Found)
Location: endpoints.rs

Fix:
1. Re-check endpoints_full.md
2. Verify endpoint paths match exactly
3. Check base URL (trailing slash?)

## JSON Parse Error
Location: parser.rs

Fix:
1. Re-check response_formats.md
2. Make actual API call to see real response:
   curl -H "X-API-Key: xxx" "https://api.example.com/v1/price?symbol=AAPL"
3. Update parser to match actual structure
4. Common issues:
   - Field name differs: lastPrice vs last_price
   - Value type differs: "150.25" vs 150.25
   - Nested structure: {data: {price: 150.25}}

## Symbol Format Wrong
Location: endpoints.rs

Fix:
1. Re-check data_formats.md
2. Update format_symbol():
   - Stocks: "AAPL" (just base)
   - Forex: "EUR/USD" or "EUR_USD"
   - Crypto: "btcusdt" (lowercase?)

## WebSocket Connection Fails
Location: websocket.rs

Fix:
1. Re-check websocket_full.md for correct URL
2. Check if auth required in URL params or message
3. Test connection from command line: websocat "wss://..."

## WebSocket No Events Received
Location: websocket.rs

Fix:
1. Re-check subscription format in websocket_full.md
2. Add debug logging: println!("Received: {}", msg)
3. Check ping/pong handling
4. Check if messages are compressed (gzip)

## Price Values Unrealistic
Location: parser.rs

Fix:
1. Check if price needs scaling:
   - Provider returns cents: price / 100.0
   - Different precision: price / 10000.0
2. Re-check response_formats.md

═══════════════════════════════════════════════════════════════════════════════
VERIFICATION CHECKLIST
═══════════════════════════════════════════════════════════════════════════════

Integration Tests (REST API):
- [ ] test_exchange_identity passes
- [ ] test_get_price returns realistic price
- [ ] test_get_ticker returns full ticker
- [ ] test_get_klines returns valid OHLC
- [ ] test_get_orderbook either works or returns UnsupportedOperation
- [ ] test_get_symbols returns non-empty list
- [ ] test_trading_not_supported returns UnsupportedOperation
- [ ] test_account_not_supported returns UnsupportedOperation (unless broker)

WebSocket Tests (if applicable):
- [ ] test_websocket_connect succeeds or handles timeout gracefully
- [ ] test_receive_ticker_events receives REAL events
- [ ] test_connection_persistence doesn't disconnect early

Code Quality:
- [ ] No compilation errors
- [ ] No panics in tests
- [ ] Clear println! output

═══════════════════════════════════════════════════════════════════════════════
EXIT CRITERIA
═══════════════════════════════════════════════════════════════════════════════

Phase 4 is COMPLETE when:

1. ✅ All tests compile
2. ✅ At least one data test returns REAL data
3. ✅ Trading tests return UnsupportedOperation
4. ✅ Account tests return UnsupportedOperation (unless broker)
5. ✅ No panics/crashes
6. ✅ Error handling is graceful

Output should look like:
```
✓ Exchange name: polygon
✓ Price for AAPL/USD: $150.25
✓ Ticker for AAPL/USD: last=$150.25
✓ Retrieved 10 klines
✓ Trading correctly marked as unsupported
...
test result: ok. 9 passed; 0 failed
```
```

### Exit Criteria

- [x] All tests compile
- [x] At least one data test returns REAL data
- [x] Trading/account tests return UnsupportedOperation
- [x] No panics
- [x] Graceful error handling

---

## Coordinator Script (Pseudocode)

```python
PROVIDERS = [
    ("polygon", "stocks/us", "https://polygon.io/docs"),
    ("oanda", "forex", "https://developer.oanda.com"),
    ("finnhub", "stocks/us", "https://finnhub.io/docs"),
    ("coinglass", "data_feeds", "https://coinglass-api.com"),
    # ... 22 more providers
]

for provider, category, docs_url in PROVIDERS:

    # Phase 1: Research
    print(f"[{provider}] Phase 1: Research")
    task = Task(
        agent="research-agent",
        prompt=RESEARCH_PROMPT.format(
            PROVIDER=provider,
            CATEGORY=category,
            DOCS_URL=docs_url
        )
    )
    await task
    verify_files_exist(f"src/{category}/{provider}/research/*.md")
    assert count_files == 8  # 8 research files

    # Phase 2: Implement
    print(f"[{provider}] Phase 2: Implement")
    task = Task(
        agent="rust-implementer",
        prompt=IMPLEMENT_PROMPT.format(
            PROVIDER=provider,
            CATEGORY=category
        )
    )
    await task
    assert cargo_check_passes()

    # Phase 3: Test
    print(f"[{provider}] Phase 3: Test")
    task = Task(
        agent="rust-implementer",
        prompt=TEST_PROMPT.format(
            PROVIDER=provider,
            CATEGORY=category
        )
    )
    await task

    # Phase 4: Debug loop
    print(f"[{provider}] Phase 4: Debug")
    max_iterations = 10
    for i in range(max_iterations):
        result = run_tests(provider)
        if result.has_real_data and result.unsupported_ops_work:
            break
        task = Task(
            agent="rust-implementer",
            prompt=DEBUG_PROMPT.format(
                PROVIDER=provider,
                CATEGORY=category,
                failures=result.failures
            )
        )
        await task

    # Commit
    git_add(f"src/{category}/{provider}/")
    git_add(f"tests/{provider}_*.rs")
    git_commit(f"feat(v5): add {provider} connector ({category})")

    print(f"[{provider}] DONE ✓")
```

---

## Parallel Execution

Independent providers can be processed in parallel:

```
┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│   Polygon   │  │   OANDA     │  │  Finnhub    │
│  (stocks)   │  │  (forex)    │  │  (stocks)   │
└─────────────┘  └─────────────┘  └─────────────┘
      │                │                │
      ▼                ▼                ▼
  [research]       [research]      [research]
      │                │                │
      ▼                ▼                ▼
 [implement]      [implement]     [implement]
      │                │                │
      ▼                ▼                ▼
   [test]           [test]          [test]
      │                │                │
      ▼                ▼                ▼
  [debug]          [debug]         [debug]
      │                │                │
      ▼                ▼                ▼
   DONE ✓           DONE ✓          DONE ✓
```

---

## Provider Registry (26 Providers)

### Aggregators (4)

| Provider | Docs URL | Priority | Status |
|----------|----------|----------|--------|
| cryptocompare | https://min-api.cryptocompare.com/documentation | HIGH | Not started |
| defillama | https://defillama.com/docs/api | HIGH | Not started |
| ib (Interactive Brokers) | https://interactivebrokers.github.io/cpwebapi/ | MEDIUM | Not started |
| yahoo | https://finance.yahoo.com/ | MEDIUM | Not started |

### Forex (3)

| Provider | Docs URL | Priority | Status |
|----------|----------|----------|--------|
| alphavantage | https://www.alphavantage.co/documentation/ | MEDIUM | Not started |
| dukascopy | https://www.dukascopy.com/trading-tools/widgets/api/ | LOW | Not started |
| oanda | https://developer.oanda.com/rest-live-v20/introduction/ | HIGH | Not started |

### Stocks - US (5)

| Provider | Docs URL | Priority | Status |
|----------|----------|----------|--------|
| polygon | https://polygon.io/docs/stocks | CRITICAL | Not started |
| alpaca | https://docs.alpaca.markets/docs | HIGH | Not started |
| finnhub | https://finnhub.io/docs/api | HIGH | Not started |
| tiingo | https://www.tiingo.com/documentation/general/overview | MEDIUM | Not started |
| twelvedata | https://twelvedata.com/docs | MEDIUM | Not started |

### Stocks - Regional (8)

| Provider | Region | Docs URL | Priority | Status |
|----------|--------|----------|----------|--------|
| futu | China | https://openapi.futunn.com/futu-api-doc/ | LOW | Not started |
| upstox | India | https://upstox.com/developer/api-documentation | MEDIUM | Not started |
| angel_one | India | https://smartapi.angelbroking.com/docs | MEDIUM | Not started |
| zerodha | India | https://kite.trade/docs/connect/v3/ | HIGH | Not started |
| dhan | India | https://dhanhq.co/docs/ | MEDIUM | Not started |
| fyers | India | https://myapi.fyers.in/docsv3 | MEDIUM | Not started |
| jquants | Japan | https://jpx.gitbook.io/j-quants-en/ | MEDIUM | Not started |
| krx | Korea | https://global.krx.co.kr/ | LOW | Not started |

### Stocks - Russia (2)

| Provider | Docs URL | Priority | Status |
|----------|----------|----------|--------|
| moex | https://www.moex.com/a2193 | LOW | Not started |
| tinkoff | https://tinkoff.github.io/investAPI/ | LOW | Not started |

### Data Feeds (4)

| Provider | Docs URL | Priority | Status |
|----------|----------|----------|--------|
| coinglass | https://coinglass-api.com/ | HIGH | Not started |
| fred | https://fred.stlouisfed.org/docs/api/ | MEDIUM | Not started |
| bitquery | https://docs.bitquery.io/ | MEDIUM | Not started |
| whale_alert | https://docs.whale-alert.io/ | LOW | Not started |

**Total: 26 providers across 4 categories**

---

## Quick Start for Coordinator

### Single Provider

```
User: "Implement Polygon.io connector"

Coordinator:
1. Read this file (CAROUSEL.md)
2. Task(research-agent): Research Polygon.io using Phase 1 prompt
3. Wait → verify 8 research files created
4. Task(rust-implementer): Implement using Phase 2 prompt
5. Wait → verify cargo check passes
6. Task(rust-implementer): Write tests using Phase 3 prompt
7. Wait → verify test files created
8. Loop: Task(rust-implementer): Fix failures using Phase 4 prompt
9. Until: at least one test returns real data
10. Commit
11. Report: "Polygon connector done, X tests passing with real data"
```

### Batch (5 providers)

```
User: "Implement batch: Polygon, OANDA, Finnhub, Coinglass, DefiLlama"

Coordinator:
Week 1:
1. Launch 5 research agents in parallel (Phase 1)
2. Review all research (verify 8 files × 5 = 40 files)
3. Launch 5 implementation agents in parallel (Phase 2)
4. Review all implementations (verify cargo check × 5)
5. Launch 5 test agents in parallel (Phase 3)
6. Review all tests (verify compilation × 5)
7. Launch 5 debug agents in parallel (Phase 4)
8. Monitor progress, commit working connectors
9. Report: "X/5 connectors complete with real data"
```

---

## Lessons Learned

### From Exchange Carousel

1. **REST vs WebSocket field names are DIFFERENT** - Always check both
2. **event_stream() needs broadcast channel** - mpsc alone doesn't work
3. **Ping/pong varies wildly** - Some text, some JSON, some gzip
4. **Graceful test handling** - Use match, not assert for network ops
5. **Connection persistence test is CRITICAL** - Tests heartbeat

### Data Provider Specific

1. **8 research files instead of 6** - Need more context for varied providers
2. **Authentication is simpler** - API keys vs HMAC signatures
3. **UnsupportedOperation pattern is KEY** - Don't force-fit into all traits
4. **Focus on EXHAUSTIVE research** - Providers vary wildly in features
5. **Account aspect = API quotas/tiers** - Not trading accounts
6. **Data quality validation critical** - Realistic price ranges matter
7. **Symbol formatting varies greatly** - AAPL vs EUR_USD vs BTCUSDT
8. **Free tier limits affect testing** - Tests must handle 429 gracefully

---

## Success Metrics

**Minimum Viable:**
- 15/26 providers working (60%)
- All CRITICAL + HIGH priority working
- Real data verified for each

**Target:**
- 21/26 providers working (80%)
- All categories represented
- Documentation complete

**Stretch Goal:**
- 26/26 providers working (100%)
- All with WebSocket support (where available)
- Full test coverage

---

## After Completion

**You will have:**
- 26 data provider connectors
- 4 categories covered (aggregators, forex, stocks, feeds)
- 208 research documents (26 × 8)
- 130+ implementation files (26 × 5)
- 52+ test files (26 × 2)
- Production-ready data infrastructure

**Use cases:**
- Multi-asset trading systems
- Market data aggregation
- Research platforms
- Real-time monitoring
- Historical backtesting

---

## End of Data Providers Agent Carousel

Use this guide to execute the full pipeline for 26 data provider connectors.

**Ready to start?** Begin with pilot provider (Polygon.io).
