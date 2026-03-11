# Vertex Protocol API Research

Complete API documentation for implementing Vertex Protocol V5 connector.

## Overview

Vertex Protocol is a hybrid orderbook-AMM DEX built on Arbitrum with:
- **Spot Markets**: Spot trading with deposits/borrows
- **Perpetual Futures**: Up to 20x leverage with funding rates
- **Universal Cross-Margin**: Unified margin across all products
- **EIP-712 Authentication**: Ethereum wallet-based signing
- **Low Latency**: 15-30ms order matching

## Research Documents

### 1. [endpoints.md](./endpoints.md)
Complete REST API endpoint documentation covering:
- **MarketData Trait**: Products, symbols, orderbook, candlesticks, ticker
- **Trading Trait**: Place order, cancel orders, market orders
- **Account Trait**: Balances, fee rates, deposits, withdrawals
- **Positions Trait**: Open orders, positions, funding rates

All endpoints include:
- Full URL paths and parameters
- Request/response formats
- Rate limits
- Error codes

### 2. [authentication.md](./authentication.md)
EIP-712 signature-based authentication:
- Domain separator configuration
- Struct definitions (Order, Cancellation, WithdrawCollateral)
- Signature generation process
- Nonce and expiration handling
- Subaccount format (bytes32)
- WebSocket authentication
- Code examples in Rust

### 3. [response_formats.md](./response_formats.md)
JSON response structures for all endpoints:
- Success/error wrapper format
- Market data responses (products, orderbook, candlesticks)
- Trading responses (order confirmation, errors)
- Account responses (balances, positions, health)
- WebSocket message formats
- X18 precision conversion
- Complete field descriptions

### 4. [symbols.md](./symbols.md)
Symbol and product ID mapping:
- Product ID structure (spot vs perpetuals)
- Symbol format (BTC, BTC-PERP)
- /symbols endpoint usage
- Symbol caching strategies
- Product type detection
- Trading pair normalization
- Multi-network support

### 5. [rate_limits.md](./rate_limits.md)
Comprehensive rate limiting documentation:
- Per-endpoint limits with weights
- Aggregate limit (600 req/10s)
- Per-IP restrictions
- WebSocket connection limits (5 per wallet)
- Rate limiter implementation
- Caching strategies
- Retry mechanisms
- Production optimization

### 6. [websocket.md](./websocket.md)
Real-time WebSocket API:
- Connection management (heartbeat, limits)
- Authentication for private streams
- Available streams (Trade, BBO, BookDepth, OrderUpdate, Fill, PositionChange)
- Subscription/unsubscription
- Message formats
- Reconnection strategies
- Implementation examples

## API Base URLs

### Production (Arbitrum One)
```
REST:      https://gateway.prod.vertexprotocol.com/v1
WebSocket: wss://gateway.prod.vertexprotocol.com/v1/ws
Subscribe: wss://gateway.prod.vertexprotocol.com/v1/subscribe
Indexer:   https://archive.prod.vertexprotocol.com/v1
```

### Testnet (Arbitrum Sepolia)
```
REST:      https://gateway.sepolia-test.vertexprotocol.com/v1
WebSocket: wss://gateway.sepolia-test.vertexprotocol.com/v1/ws
Indexer:   https://archive.sepolia-test.vertexprotocol.com/v1
```

## Key Characteristics

### Authentication
- **Method**: EIP-712 typed structured data signing
- **No API Keys**: Uses Ethereum wallet private keys
- **Signature Format**: 65-byte hex string (r + s + v)
- **Domain**: name="Vertex", version="0.0.1", chainId, verifyingContract

### Data Format
- **Precision**: All numbers in X18 format (18 decimal places)
- **Prices**: `priceX18` = actual_price * 1e18
- **Amounts**: Positive = buy/long, Negative = sell/short
- **Timestamps**: Unix seconds (queries), milliseconds (WebSocket auth)

### Subaccount Format
```
bytes32 = address (20 bytes) + subaccount_name (12 bytes)
Example: 0x7a5ec...c43 + "default" = 0x7a5ec...c43000000000000000000000
```

### Order Types
- **Limit Orders**: Specify priceX18 and amount
- **Market Orders**: Use extreme price + IOC/FOK time-in-force
- **Time-in-Force**: GTC, IOC, FOK, POST_ONLY (encoded in expiration field)

### Rate Limits
- **Aggregate**: 600 requests per 10 seconds (all endpoints)
- **Place Order**: 10 req/s (leveraged), 5 req/10s (spot)
- **Cancel Orders**: 600 req/s
- **Queries**: Variable (see rate_limits.md)
- **WebSocket**: 5 connections per wallet, ping every 30s

## Implementation Checklist

### Core Functionality
- [ ] EIP-712 domain and signature generation
- [ ] Subaccount (sender) format conversion
- [ ] X18 precision conversion utilities
- [ ] Nonce and expiration generation
- [ ] Order digest calculation

### MarketData Trait
- [ ] get_products() - All products endpoint
- [ ] get_symbols() - Symbols endpoint with caching
- [ ] get_orderbook() - Market liquidity query
- [ ] get_ticker() - Market price query
- [ ] get_candlesticks() - Archive indexer candlesticks
- [ ] get_recent_trades() - Trade stream via WebSocket

### Trading Trait
- [ ] place_order() - Place order execute
- [ ] place_market_order() - Market order (extreme price + IOC)
- [ ] cancel_order() - Cancel specific orders
- [ ] cancel_all_orders() - Cancel product orders
- [ ] get_order_status() - Order query

### Account Trait
- [ ] get_balances() - Subaccount info query
- [ ] get_fee_rates() - Fee rates query
- [ ] get_max_withdrawable() - Max withdrawable query
- [ ] withdraw() - Withdraw collateral execute

### Positions Trait
- [ ] get_open_orders() - Subaccount orders query
- [ ] get_positions() - Perp balances from subaccount_info
- [ ] get_position() - Single product position
- [ ] get_funding_rate() - Archive indexer funding rate

### WebSocket
- [ ] Connection management with heartbeat
- [ ] Authentication for private streams
- [ ] Trade stream subscription
- [ ] Orderbook (BookDepth) stream
- [ ] Order update stream
- [ ] Fill stream
- [ ] Position change stream
- [ ] Reconnection with exponential backoff

### Rate Limiting
- [ ] Per-endpoint rate limiters
- [ ] Aggregate rate limiter with weights
- [ ] Symbol cache (1 hour TTL)
- [ ] Orderbook cache (configurable)
- [ ] Retry logic with backoff

### Error Handling
- [ ] Parse error responses
- [ ] Handle rate limit errors
- [ ] Signature validation errors
- [ ] Insufficient balance errors
- [ ] WebSocket disconnection handling

## Code Structure

Following V5 architecture pattern (like KuCoin):

```
exchanges/vertex/
├── mod.rs          # Module exports
├── endpoints.rs    # URL constants, endpoint enum, symbol formatting
├── auth.rs         # EIP-712 signing implementation
├── parser.rs       # JSON parsing utilities
├── connector.rs    # Trait implementations
└── websocket.rs    # WebSocket client
```

## Dependencies

```toml
[dependencies]
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
tokio-tungstenite = "0.20"
futures = "0.3"
sha3 = "0.10"  # Keccak-256
secp256k1 = "0.27"  # ECDSA signing
hex = "0.4"
```

## Testing Strategy

1. **Unit Tests**: Signature generation, format conversion, parsing
2. **Integration Tests**: API calls against testnet
3. **WebSocket Tests**: Connection, subscription, message handling
4. **Rate Limit Tests**: Verify limiter logic
5. **Error Tests**: Invalid signatures, rate limits, etc.

## References

### Official Documentation
- [Vertex Protocol Docs](https://docs.vertexprotocol.com/)
- [API Gateway](https://docs.vertexprotocol.com/developer-resources/api/gateway)
- [Python SDK](https://vertex-protocol.github.io/vertex-python-sdk/api-reference.html)
- [TypeScript SDK](https://vertex-protocol.github.io/vertex-typescript-sdk/)

### Reference Implementation
- [Hummingbot Vertex Connector](https://github.com/hummingbot/hummingbot/tree/master/hummingbot/connector/exchange/vertex)
- [Vertex Python SDK](https://github.com/vertex-protocol/vertex-python-sdk)

### EIP-712 Resources
- [EIP-712 Specification](https://eips.ethereum.org/EIPS/eip-712)
- [Vertex Signing Examples](https://docs.vertexprotocol.com/developer-resources/api/gateway/signing/examples)

## Implementation Notes

### Critical Details

1. **Sender Format**: Must be exactly 32 bytes (address + subaccount)
2. **Signature**: Must use Keccak-256 (not SHA-256)
3. **Recovery ID**: v = signature[64] + 27
4. **Nonce**: Must be unique (use timestamp + random)
5. **Expiration**: Includes TIF flags in bits 62-63
6. **X18 Conversion**: All prices/amounts need conversion
7. **Heartbeat**: WebSocket requires ping every 30 seconds
8. **Connection Limit**: Max 5 WebSocket connections per wallet

### Common Pitfalls

- Using wrong hash function (SHA-256 instead of Keccak-256)
- Incorrect sender format (not 32 bytes)
- Missing X18 conversion
- Forgetting WebSocket heartbeat
- Exceeding connection limits
- Not handling rate limits properly
- Incorrect time-in-force encoding

## Next Steps

1. Review all documentation files
2. Set up development environment
3. Implement auth.rs (EIP-712 signing)
4. Implement endpoints.rs (URL constants)
5. Implement parser.rs (JSON parsing)
6. Implement connector.rs (trait methods)
7. Implement websocket.rs (real-time data)
8. Write unit tests
9. Test against testnet
10. Deploy to production

## Support

For questions or issues during implementation:
- Vertex Protocol Discord: [Join](https://discord.gg/vertexprotocol)
- GitHub Issues: [Vertex SDK](https://github.com/vertex-protocol/vertex-python-sdk/issues)
- Documentation: [Vertex Docs](https://docs.vertexprotocol.com/)
