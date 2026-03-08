# Zerodha Kite Connect - Implementation Brief for rust-implementer

## Context

You are implementing a **Zerodha Kite Connect connector** for the V5 connectors architecture.

**Key Facts:**
- **Provider**: Zerodha Kite Connect (India's #1 stock broker)
- **Category**: stocks/india
- **Type**: FULL-SERVICE BROKER (not just data provider)
- **Must Implement**: MarketData + Trading + Account + Positions traits

## Research Available

All research documentation is in: `src/stocks/india/zerodha/research/`

Files:
- `01_api_overview.md` - API architecture, base URLs, features
- `02_authentication.md` - Custom OAuth-like flow with SHA-256 checksum
- `03_market_data.md` - Instruments, quotes, historical candles
- `04_trading.md` - Order placement, modification, cancellation, GTT
- `05_account.md` - User profile, margins, holdings, positions
- `06_websocket.md` - Binary WebSocket streaming (complex)
- `07_rate_limits.md` - Rate limits and constraints
- `08_error_handling.md` - Error types and handling

## Reference Implementation

Use KuCoin connector as reference pattern:
`src/exchanges/kucoin/` - Study the structure and trait implementations

## Critical Differences from Standard Exchanges

### 1. Authentication (MOST IMPORTANT!)

Zerodha uses **custom OAuth-like authentication** (NOT standard OAuth 2.0):

**Flow:**
1. User navigates to login URL: `https://kite.zerodha.com/connect/login?v=3&api_key={api_key}`
2. After login, receives `request_token` via redirect
3. Calculate checksum: `SHA256(api_key + request_token + api_secret)`
4. Exchange for access_token via POST to `/session/token`
5. Use access_token in ALL requests

**Authorization Header Format:**
```
Authorization: token {api_key}:{access_token}
```

NOT `Bearer {token}` - it's `token {key}:{token}`!

**Token Lifetime:**
- access_token expires DAILY at 6:00 AM IST
- NO refresh token mechanism
- Must re-authenticate daily

### 2. Symbol Format

Zerodha uses **instrument_token** (integer) AND **exchange:tradingsymbol** format:

```
NSE:INFY - Infosys equity on NSE
NFO:NIFTY26FEB20000CE - Nifty Call Option
408065 - instrument_token for INFY
```

**Quote endpoints**: Use `exchange:tradingsymbol`
**Historical data**: Use `instrument_token`
**WebSocket**: Use `instrument_token`

### 3. WebSocket (COMPLEX!)

- **Binary data format** (not JSON)
- **Custom binary encoding** with instrument tokens
- **3 streaming modes**: ltp (8 bytes), quote (44 bytes), full (184 bytes)
- **Prices in PAISE** (divide by 100 for INR)
- **Market depth**: 5 levels each side
- **Order postbacks**: JSON text messages
- **1-byte heartbeat** from server (no ping/pong required)

### 4. Broker-Specific Features

- **Holdings**: Long-term delivery portfolio
- **Positions**: Intraday + derivatives (net and day views)
- **Products**: CNC (delivery), MIS (intraday), NRML (F&O), MTF (margin)
- **GTT Orders**: Good Till Triggered (1 year validity)
- **Position conversion**: MIS ↔ CNC ↔ NRML
- **Margin calculator**: Pre-calculate margins for orders

## Implementation Structure

Create files in: `src/stocks/india/zerodha/`

### File 1: mod.rs

```rust
//! Zerodha Kite Connect connector
//!
//! Category: stocks/india
//! Type: Full-service broker
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: Yes (Binary + JSON)
//! - Authentication: Custom OAuth-like with SHA-256
//! - Free tier: Limited (Personal API - no WS, no historical)
//! - Paid tier: ₹500/month (Connect API - full access)
//!
//! ## Supported Exchanges
//! - NSE - National Stock Exchange (Equities)
//! - BSE - Bombay Stock Exchange (Equities)
//! - NFO - NSE Futures & Options
//! - BFO - BSE Futures & Options
//! - MCX - Multi Commodity Exchange
//! - CDS - Currency Derivatives (NSE)
//! - BCD - BSE Currency Derivatives
//!
//! ## Data Types
//! - Market quotes: Yes (LTP, OHLC, Full with depth)
//! - Historical candles: Yes (minute to day intervals)
//! - Order management: Yes (regular, AMO, GTT, iceberg)
//! - Account data: Yes (profile, margins, holdings, positions)
//! - Trading: Yes (all order types, products)

mod endpoints;
mod auth;
mod parser;
mod connector;
mod websocket;

pub use connector::ZerodhaConnector;
pub use websocket::ZerodhaWebSocket;
```

### File 2: endpoints.rs

**Key points:**
- Base URL: `https://api.kite.trade`
- WebSocket: `wss://ws.kite.trade`
- Use enum for endpoints like KuCoin
- Symbol format: `{exchange}:{tradingsymbol}` for REST
- Historical data uses `/instruments/historical/{token}/{interval}`

**Endpoints to implement:**
- `/instruments` and `/instruments/{exchange}` - instrument list (CSV)
- `/quote`, `/quote/ohlc`, `/quote/ltp` - market quotes
- `/instruments/historical/{token}/{interval}` - candles
- `/orders/{variety}` - place/modify/cancel orders
- `/portfolio/holdings` - holdings
- `/portfolio/positions` - positions
- `/user/margins` - margins
- `/user/profile` - user profile

### File 3: auth.rs

**Critical implementation:**

```rust
pub struct ZerodhaAuth {
    pub api_key: String,
    pub api_secret: String,
    pub access_token: Option<String>,
}

impl ZerodhaAuth {
    /// Generate SHA-256 checksum for token exchange
    /// message = api_key + request_token + api_secret
    /// checksum = SHA256(message)
    pub fn generate_checksum(&self, request_token: &str) -> String {
        use sha2::{Sha256, Digest};
        let message = format!("{}{}{}", self.api_key, request_token, self.api_secret);
        let mut hasher = Sha256::new();
        hasher.update(message.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Sign headers for authenticated requests
    /// Format: "Authorization: token {api_key}:{access_token}"
    pub fn sign_headers(&self, headers: &mut HashMap<String, String>) {
        if let Some(token) = &self.access_token {
            headers.insert(
                "Authorization".to_string(),
                format!("token {}:{}", self.api_key, token)
            );
        }
    }

    /// Exchange request_token for access_token
    /// POST /session/token with form-encoded body
    pub async fn exchange_token(&mut self, request_token: &str) -> Result<String, ExchangeError> {
        // Implementation needed
    }
}
```

### File 4: parser.rs

**Parse these response types:**

1. **Instruments CSV** - gzipped CSV with columns: instrument_token, exchange_token, tradingsymbol, name, last_price, expiry, strike, tick_size, lot_size, instrument_type, segment, exchange

2. **Quote JSON** - nested structure with instrument_token, last_price, ohlc {open, high, low, close}, depth {buy: [{price, quantity, orders}], sell: [...]}

3. **Order JSON** - order_id, status, tradingsymbol, exchange, quantity, filled_quantity, price, average_price, etc.

4. **Holdings JSON** - tradingsymbol, exchange, quantity, average_price, last_price, pnl

5. **Positions JSON** - {net: [...], day: [...]} with quantity, average_price, pnl, m2m

6. **Margins JSON** - {equity: {enabled, net, available {...}, utilised {...}}, commodity: {...}}

7. **WebSocket Binary** - Complex binary format (start simple, use official SDKs as reference)

### File 5: connector.rs

**Implement ALL these traits:**

1. **ExchangeIdentity** - exchange_name() = "zerodha", exchange_id = ExchangeId::Zerodha

2. **MarketData** - All methods:
   - get_price() - use `/quote/ltp`
   - get_ticker() - use `/quote`
   - get_orderbook() - use `/quote` (depth field)
   - get_klines() - use `/instruments/historical/{token}/{interval}`
   - get_symbols() - use `/instruments` or `/instruments/{exchange}`

3. **Trading** - All methods:
   - place_order() - POST `/orders/regular`
   - cancel_order() - DELETE `/orders/regular/{order_id}`
   - modify_order() - PUT `/orders/regular/{order_id}`
   - get_order() - GET `/orders/{order_id}`
   - get_open_orders() - GET `/orders` then filter

4. **Account** - All methods:
   - get_balance() - GET `/user/margins`
   - get_account_info() - GET `/user/profile`

5. **Positions** - All methods:
   - get_positions() - GET `/portfolio/positions`
   - get_position() - GET `/portfolio/positions` then filter

**Extended methods** (Zerodha-specific):
- `get_holdings()` - GET `/portfolio/holdings`
- `convert_position()` - PUT `/portfolio/positions`
- `place_gtt()` - POST `/gtt/triggers`
- `get_margin_for_orders()` - POST `/margins/orders`

### File 6: websocket.rs

**Start with basic implementation:**

1. Connection: `wss://ws.kite.trade?api_key={key}&access_token={token}`
2. Subscribe: Send JSON `{"a": "subscribe", "v": [token1, token2]}`
3. Set mode: Send JSON `{"a": "mode", "v": ["full", [token1, token2]]}`
4. Receive:
   - Binary messages: Market data (parse struct with instrument_token + fields)
   - Text messages: Order postbacks, errors

**For Phase 2/3, can implement basic binary parsing:**
- Read packet count (2 bytes)
- Read packet length (2 bytes)
- Read instrument_token (4 bytes)
- Read mode-specific fields

**Advanced binary parsing can be refined in Phase 4.**

## Phase 2 Checklist

- [ ] mod.rs with documentation
- [ ] endpoints.rs with all endpoint enums
- [ ] auth.rs with SHA-256 checksum and "token" authorization
- [ ] parser.rs with JSON parsers (CSV optional for Phase 2)
- [ ] connector.rs with ALL trait implementations:
  - [ ] ExchangeIdentity
  - [ ] MarketData (all methods)
  - [ ] Trading (all methods)
  - [ ] Account (all methods)
  - [ ] Positions (all methods)
- [ ] websocket.rs with basic binary parsing
- [ ] Add to `src/stocks/india/mod.rs`
- [ ] Add ExchangeId::Zerodha to `src/core/types/common.rs`
- [ ] `cargo check --package digdigdig3` passes

## Phase 3: Testing

Create files in `tests/`:

1. **tests/zerodha_integration.rs**:
   - test_exchange_identity
   - test_get_price (NSE:INFY)
   - test_get_ticker
   - test_get_klines
   - test_get_symbols
   - test_get_orderbook
   - test_place_order (CAREFUL - real order!)
   - test_get_balance
   - test_get_positions
   - test_get_holdings (Zerodha-specific)

2. **tests/zerodha_websocket.rs**:
   - test_websocket_connect
   - test_subscribe_ticker
   - test_receive_events
   - test_order_postback

**Test Data:**
- Symbol: NSE:INFY (Infosys)
- Token: 408065
- Use small quantities for orders!

## Phase 4: Debugging

Common issues to watch for:

1. **Authentication errors**: Check "token" format, not "Bearer"
2. **Symbol format**: Use "NSE:INFY", not "INFY-INR"
3. **Instrument tokens**: Must fetch from `/instruments` CSV
4. **Binary parsing**: Prices in PAISE (divide by 100)
5. **WebSocket auth**: Must be in URL query params
6. **Content-Type**: Some endpoints use form-encoded, some JSON
7. **Order parameters**: Exchange, tradingsymbol, transaction_type, order_type, quantity, product all required

## Environment Variables

```bash
export ZERODHA_API_KEY="your_api_key"
export ZERODHA_API_SECRET="your_api_secret"
export ZERODHA_ACCESS_TOKEN="your_access_token"  # After login flow
```

## Important Notes

1. **No testnet** - Must use production with small orders
2. **Token expires daily at 6 AM IST** - Tests will fail after expiry
3. **Rate limits**: 10 req/sec per API key
4. **Order limits**: 3,000 orders/day, 200 orders/minute
5. **WebSocket limits**: 3,000 instruments per connection, 3 connections max
6. **Paid tier required** for WebSocket and historical data

## Success Criteria

Phase 2 complete when:
- All 6 files created
- Code compiles without errors
- All traits implemented (even if returning mock data initially)

Phase 3 complete when:
- Test files created
- Tests compile
- Tests have graceful error handling

Phase 4 complete when:
- At least one test returns REAL data
- Trading tests return UnsupportedOperation OR execute safely
- No panics or crashes

## Resources

- Official Docs: https://kite.trade/docs/connect/v3/
- Research folder: `src/stocks/india/zerodha/research/`
- KuCoin reference: `src/exchanges/kucoin/`
- Python SDK (reference): https://github.com/zerodha/pykiteconnect

## Next Steps

1. Start with Phase 2: Implement the 6 Rust files
2. Run `cargo check` frequently
3. Fix compilation errors as you go
4. Once Phase 2 compiles, move to Phase 3 (tests)
5. Once tests compile, move to Phase 4 (debug with real API)

Good luck! This is a comprehensive implementation but the research is thorough and the pattern is clear from KuCoin.
