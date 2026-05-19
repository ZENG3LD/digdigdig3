# Deribit API Research - Complete Documentation

Research documentation for implementing Deribit V5 connector.

## Overview

Deribit is a **cryptocurrency derivatives exchange** specializing in:
- Bitcoin (BTC) and Ethereum (ETH) options and futures
- USDC-settled linear instruments (SOL, XRP, BNB)
- Perpetual contracts (inverse and linear)
- European-style options
- Combo instruments (multi-leg strategies)

**Key Characteristics**:
- JSON-RPC 2.0 protocol (over HTTP and WebSocket)
- OAuth 2.0 style authentication
- Credit-based rate limiting
- Cash settlement only (no physical delivery)
- WebSocket-first architecture

---

## Document Structure

### 1. [endpoints.md](./endpoints.md)
Complete REST and WebSocket endpoint reference organized by trait:
- **MarketData Trait**: Instruments, orderbook, ticker, trades
- **Trading Trait**: Buy, sell, cancel, edit orders
- **Account Trait**: Account summary, user trades, settlement history
- **Positions Trait**: Position queries
- **Wallet Endpoints**: Transfers, deposits, withdrawals

**Key Points**:
- All endpoints use JSON-RPC 2.0 format
- Method format: `{scope}/{method_name}` (e.g., `public/get_instruments`, `private/buy`)
- Parameters must be named objects (no positional parameters)
- Prefer WebSocket over HTTP for real-time data

---

### 2. [authentication.md](./authentication.md)
Complete OAuth 2.0 authentication specification:
- **Grant Types**: Client credentials, client signature, refresh token
- **Token Lifecycle**: Access tokens (15 min), refresh tokens (extended)
- **Scopes**: `trade:read`, `trade:write`, `wallet:read_write`, `account:read`
- **Security**: HMAC-SHA256 signature generation, 2FA support

**Key Points**:
- Use `public/auth` endpoint to obtain access token
- Client signature grant is more secure (doesn't send secret over network)
- Refresh tokens proactively before expiration
- Support connection-scope and session-scope tokens

---

### 3. [response_formats.md](./response_formats.md)
Complete JSON-RPC response specifications:
- **Success Response**: `result` field with method-specific data
- **Error Response**: `error` field with code and message
- **Deribit Extensions**: `testnet`, `usIn`, `usOut`, `usDiff` (latency tracking)
- **Error Codes**: 100+ error codes documented

**Key Points**:
- All responses include performance metrics (`usDiff` for server-side latency)
- `result` and `error` are mutually exclusive
- WebSocket notifications have no `id` field
- Detailed error codes for debugging (10028 = rate limit, 13004 = invalid credentials)

---

### 4. [symbols.md](./symbols.md)
Instrument naming conventions and symbol formats:
- **Perpetuals**: `BTC-PERPETUAL`, `ETH-PERPETUAL`
- **Linear Perpetuals**: `SOL_USDC-PERPETUAL`, `XRP_USDC-PERPETUAL`
- **Futures**: `BTC-29MAR24`, `ETH-27DEC24`
- **Options**: `BTC-27DEC24-50000-C`, `ETH-29MAR24-3000-P`
- **Combos**: `BTC-FS-29MAR24_27SEP24`, `BTC-STRADDLE-29MAR24-50000`

**Key Points**:
- Deribit is derivatives-focused (limited spot trading)
- Inverse instruments (BTC/ETH-settled) vs Linear instruments (USDC-settled)
- Options use `DDMMMYY` date format and integer strikes
- Use `public/get_instruments` to retrieve metadata

---

### 5. [rate_limits.md](./rate_limits.md)
Credit-based rate limiting system:
- **Credit System**: Each request consumes credits, credits refill continuously
- **Burst Capacity**: ~50,000 credits (matching), ~200,000 (non-matching)
- **Refill Rate**: ~10,000 credits/sec (matching), ~20,000 (non-matching)
- **Sustained Rate**: ~20 req/sec for order operations
- **Separate Pools**: Matching engine (orders) vs non-matching engine (data)

**Key Points**:
- Error 10028 = rate limit exceeded
- Public requests (per IP) have lower limits than authenticated requests (per sub-account)
- WebSocket subscriptions don't consume ongoing credits (only initial subscribe)
- Production and testnet have separate rate limit pools

---

### 6. [websocket.md](./websocket.md)
Complete WebSocket API specification:
- **Endpoints**: `wss://www.deribit.com/ws/api/v2` (production), `wss://test.deribit.com/ws/api/v2` (test)
- **Public Channels**: `book.*`, `ticker.*`, `trades.*`, `deribit_price_index.*`
- **Private Channels**: `user.orders.*`, `user.trades.*`, `user.portfolio.*`, `user.changes.*`
- **Subscriptions**: Batch up to 500 channels per request
- **Intervals**: `raw`, `100ms`, `agg2`

**Key Points**:
- WebSocket is preferred over HTTP (faster, supports subscriptions, cancel-on-disconnect)
- Max 32 connections per IP, 16 sessions per API key
- Heartbeat required (send `public/test` every 30s)
- Reconnection with exponential backoff
- `raw` feeds require authenticated connection

---

## Implementation Priorities

### Phase 1: Core REST API
1. ✅ Authentication (`public/auth` with client credentials and signature)
2. ✅ Market Data (`public/get_instruments`, `public/get_order_book`, `public/ticker`)
3. ✅ Trading (`private/buy`, `private/sell`, `private/cancel`)
4. ✅ Account (`private/get_account_summary`, `private/get_positions`)

### Phase 2: WebSocket
1. ✅ WebSocket connection and authentication
2. ✅ Public subscriptions (`book.*`, `ticker.*`, `trades.*`)
3. ✅ Private subscriptions (`user.orders.*`, `user.trades.*`)
4. ✅ Heartbeat and reconnection logic

### Phase 3: Advanced Features
1. Rate limit tracking and client-side throttling
2. Token refresh mechanism
3. Cancel-on-disconnect support
4. Combo instruments support
5. Block trading support

---

## Key Differences from Other Exchanges

### vs Binance/Bybit/OKX (Spot/Perpetuals)
- **Deribit**: Derivatives-focused (options + futures)
- **Protocol**: JSON-RPC 2.0 (not REST-like)
- **Rate Limits**: Credit-based (not simple req/sec)
- **Authentication**: OAuth 2.0 (not HMAC-SHA256 on each request)
- **Settlement**: Cash only (no physical delivery)

### vs Traditional Options Exchanges
- **Crypto-native**: BTC/ETH as base assets and settlement currencies
- **Perpetuals**: Funding rate mechanism (not traditional futures)
- **Linear Options**: USDC-settled options (like fiat-margined)
- **Mark Price**: Fair value calculated by exchange (prevents manipulation)

---

## Testing Strategy

### On Testnet
1. **Create test account** at test.deribit.com
2. **Generate API keys** with appropriate scopes
3. **Test endpoints**:
   - Authentication flow
   - Market data queries
   - Order placement (test orders, no real money)
   - Position management
4. **Test WebSocket**:
   - Connection and authentication
   - Subscriptions (public and private)
   - Reconnection logic
5. **Test rate limits**:
   - Send burst requests to measure limits
   - Verify error 10028 handling
   - Measure credit refill rate

### Environment Variables
```bash
# Test environment
DERIBIT_ENV=test
DERIBIT_BASE_URL=https://test.deribit.com/api/v2
DERIBIT_WS_URL=wss://test.deribit.com/ws/api/v2
DERIBIT_CLIENT_ID=your_test_client_id
DERIBIT_CLIENT_SECRET=your_test_client_secret

# Production environment
DERIBIT_ENV=production
DERIBIT_BASE_URL=https://www.deribit.com/api/v2
DERIBIT_WS_URL=wss://www.deribit.com/ws/api/v2
DERIBIT_CLIENT_ID=your_prod_client_id
DERIBIT_CLIENT_SECRET=your_prod_client_secret
```

---

## V5 Connector Module Structure

Following KuCoin reference structure:

```
exchanges/deribit/
├── mod.rs              # Module exports
├── endpoints.rs        # Endpoint URLs, request building, symbol formatting
├── auth.rs             # OAuth 2.0 authentication, signature generation
├── parser.rs           # JSON-RPC response parsing
├── connector.rs        # Trait implementations (MarketData, Trading, Account, Positions)
├── websocket.rs        # WebSocket client, subscriptions, notifications
└── research/           # This documentation
    ├── README.md       # This file
    ├── endpoints.md
    ├── authentication.md
    ├── response_formats.md
    ├── symbols.md
    ├── rate_limits.md
    └── websocket.md
```

---

## Key Implementation Notes

### JSON-RPC Request Format
```rust
#[derive(Serialize)]
struct JsonRpcRequest<T> {
    jsonrpc: &'static str, // Always "2.0"
    id: u64,
    method: String,
    params: T,
}

// Example
let request = JsonRpcRequest {
    jsonrpc: "2.0",
    id: 1,
    method: "public/get_instruments".to_string(),
    params: json!({
        "currency": "BTC",
        "kind": "future"
    }),
};
```

### Authentication Header
```rust
// After obtaining access_token via public/auth
let mut headers = HeaderMap::new();
headers.insert(
    "Authorization",
    format!("Bearer {}", access_token).parse()?
);
```

### Symbol Parsing
```rust
fn parse_instrument_name(name: &str) -> InstrumentType {
    if name.ends_with("-PERPETUAL") {
        InstrumentType::Perpetual
    } else if name.ends_with("_USDC-PERPETUAL") {
        InstrumentType::LinearPerpetual
    } else if name.matches('-').count() == 3 {
        // BTC-27DEC24-50000-C
        InstrumentType::Option
    } else if name.matches('-').count() == 1 {
        // BTC-29MAR24
        InstrumentType::Future
    } else {
        InstrumentType::Combo
    }
}
```

### Rate Limiter
```rust
struct RateLimiter {
    matching_credits: Arc<Mutex<u64>>,
    non_matching_credits: Arc<Mutex<u64>>,
    refill_rate_matching: u64,
    refill_rate_non_matching: u64,
    last_refill: Arc<Mutex<Instant>>,
}

impl RateLimiter {
    async fn consume(&self, credits: u64, is_matching: bool) -> Result<()> {
        self.refill().await;

        let mut credits_pool = if is_matching {
            self.matching_credits.lock().await
        } else {
            self.non_matching_credits.lock().await
        };

        if *credits_pool >= credits {
            *credits_pool -= credits;
            Ok(())
        } else {
            Err(RateLimitError::InsufficientCredits)
        }
    }
}
```

---

## Common Pitfalls

1. **Positional Parameters**: Deribit does NOT support positional parameters (only named)
   ```rust
   // WRONG
   params: [currency, kind]

   // CORRECT
   params: { "currency": "BTC", "kind": "future" }
   ```

2. **Instrument Name Case**: Instrument names are case-sensitive (use uppercase)
   ```rust
   // WRONG: "btc-perpetual"
   // CORRECT: "BTC-PERPETUAL"
   ```

3. **Timestamp Units**: Timestamps are in **milliseconds** (not seconds)
   ```rust
   let timestamp = SystemTime::now()
       .duration_since(UNIX_EPOCH)?
       .as_millis() as u64;
   ```

4. **Amount Units**: For perpetuals/futures, amount is in **USD**, not contracts
   ```rust
   // To trade 1 BTC worth of BTC-PERPETUAL at $50,000:
   // amount = $50,000 (not 1.0)
   ```

5. **HTTP Method**: Use POST for JSON-RPC (even for queries)
   ```rust
   client.post(url).json(&request).send().await?
   ```

---

## Sources & References

All information in this research documentation is sourced from:

- [Deribit API Documentation](https://docs.deribit.com/)
- [JSON-RPC 2.0 Protocol Overview](https://docs.deribit.com/articles/json-rpc-overview)
- [API Authentication Guide](https://support.deribit.com/hc/en-us/articles/29748629634205-API-Authentication-Guide)
- [Rate Limits Support Article](https://support.deribit.com/hc/en-us/articles/25944617523357-Rate-Limits)
- [Market Data Best Practices](https://support.deribit.com/hc/en-us/articles/29592500256669-Market-Data-Collection-Best-Practices)
- [Order Management Best Practices](https://support.deribit.com/hc/en-us/articles/29514039279773-Order-Management-Best-Practices)
- [Settlement Information](https://support.deribit.com/hc/en-us/articles/29734325712413-Settlement)
- [Linear Perpetuals](https://support.deribit.com/hc/en-us/articles/31424969384605-Linear-Perpetual)
- [Trading Combos](https://insights.deribit.com/education/trading-combos-on-deribit/)
- [WebSocket Connection Guide](https://insights.deribit.com/dev-hub/how-to-maintain-and-authenticate-a-websocket-connection-to-deribit-python/)

**Research Date**: 2026-01-20

---

## Next Steps

1. Review this research documentation
2. Set up test account and API keys on test.deribit.com
3. Implement V5 connector following KuCoin reference structure
4. Test on testnet thoroughly
5. Deploy to production with proper monitoring

---

## Questions & Clarifications

If you need clarification on any aspect:

1. **Endpoints**: Check `endpoints.md` for detailed parameter specs
2. **Authentication**: Check `authentication.md` for OAuth 2.0 flow
3. **Response Format**: Check `response_formats.md` for parsing details
4. **Symbols**: Check `symbols.md` for instrument name formats
5. **Rate Limits**: Check `rate_limits.md` for credit system mechanics
6. **WebSocket**: Check `websocket.md` for subscription channels

For any gaps, consult the official Deribit API documentation at https://docs.deribit.com/

---

**Research Complete** ✅

All required documentation has been created:
- ✅ endpoints.md
- ✅ authentication.md
- ✅ response_formats.md
- ✅ symbols.md
- ✅ rate_limits.md
- ✅ websocket.md
- ✅ README.md (this file)
