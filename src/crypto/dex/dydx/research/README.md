# dYdX v4 Research Documentation

Complete API research for implementing dYdX v4 connector in the V5 architecture.

## Overview

dYdX v4 is a **fully decentralized perpetual futures exchange** built on the Cosmos blockchain. Unlike traditional centralized exchanges and even dYdX v3, v4 operates as a standalone blockchain with:

- **No centralized API keys** - Uses blockchain wallet authentication
- **Dual API architecture** - Indexer (read-only) and Node (write operations)
- **Integer-based pricing** - Quantums and subticks for precision
- **Protocol-level rate limits** - Blockchain-enforced order limits
- **USDC settlement** - All markets quote in USDC

## Architecture Differences

### Traditional CEX (Binance, Bybit, etc.)
- REST API for all operations
- API key + secret authentication
- Centralized order matching
- Direct database queries

### dYdX v4
- **Indexer API** (REST/WebSocket) - Read-only queries
- **Node API** (gRPC/Protobuf) - Write operations via blockchain transactions
- **Cosmos wallet** - Mnemonic-based authentication
- **Decentralized** - Validators run matching engine

## Documentation Structure

### 1. [endpoints.md](./endpoints.md)
Complete API endpoint documentation covering all trait requirements:

- **MarketData Trait**: Markets, orderbook, trades, candles, funding rates
- **Account Trait**: Subaccounts, balances, transfers, rewards
- **Positions Trait**: Perpetual positions, PnL, funding payments
- **Trading Trait**: Place/cancel orders, fills, order status

**Key Points**:
- REST endpoints use ticker format: `BTC-USD`
- gRPC orders use `clobPairId` (internal integer ID)
- Separate endpoints for parent and child subaccounts
- Base URLs differ for mainnet vs testnet

### 2. [authentication.md](./authentication.md)
Authentication and wallet management:

- **No API keys** - Blockchain wallet-based
- **Mnemonic phrases** - 24-word recovery seed
- **Main accounts** - Hold gas tokens, manage subaccounts
- **Subaccounts** - Separate trading accounts (0-128,000)
- **Transaction signing** - All write operations are blockchain transactions

**Key Points**:
- Indexer API requires no authentication (public)
- Node API requires signed transactions
- Gas fees paid in DYDX tokens
- Private key = your authentication

### 3. [response_formats.md](./response_formats.md)
Complete JSON response structures for all endpoints:

- Market data responses
- Account/position responses
- Order and fill responses
- WebSocket message formats
- Error response structures

**Key Points**:
- All prices/sizes are **strings** (not numbers)
- Timestamps in ISO 8601 format
- Block heights as strings
- Null vs missing fields

### 4. [symbols.md](./symbols.md)
Symbol format and market structure:

- **Format**: `{BASE}-USD` (e.g., `BTC-USD`)
- **Quote asset**: All markets use USDC
- **Market types**: Cross-margin (default) and Isolated
- **Market identifiers**: ticker (human) vs clobPairId (protocol)

**Key Points**:
- Case-sensitive uppercase required
- 100+ active markets, 800+ with isolated margin
- ticker ↔ clobPairId mapping required
- Leverage determined by `initialMarginFraction`

### 5. [rate_limits.md](./rate_limits.md)
Multi-layered rate limiting:

- **Blockchain limits**: 200 short-term orders/block, 2 stateful orders/block
- **Indexer limits**: Not publicly documented, monitor 429 responses
- **Withdrawal limits**: max(1% TVL, $1M)/hour, max(10% TVL, $10M)/day

**Key Points**:
- Rate limits per account (shared across subaccounts)
- Block time ~1-2 seconds
- Use WebSockets to reduce HTTP requests
- Consider self-hosted indexer for HFT

### 6. [websocket.md](./websocket.md)
Real-time WebSocket data feeds:

- **6+ channels**: orderbook, trades, markets, candles, subaccounts, block_height
- **Subscription-based** model
- **No authentication** required
- **Lower latency** than REST API

**Key Points**:
- WSS endpoints for mainnet/testnet
- Message IDs for ordering
- Batched mode available
- Automatic reconnection recommended

### 7. [quantums_and_subticks.md](./quantums_and_subticks.md)
Critical protocol-level price/size encoding:

- **Quantums**: Integer representation of position size
- **Subticks**: Integer representation of price
- **Conversion formulas** between human-readable and protocol values
- **atomicResolution**, **quantumConversionExponent**, **subticksPerTick**

**Key Points**:
- All gRPC orders use quantums/subticks (not decimal)
- Indexer API returns human-readable values
- Market-specific parameters required
- Use Decimal types to avoid precision loss

## Implementation Checklist

### Phase 1: Read-Only (Indexer API)
- [ ] Implement REST client for Indexer API
- [ ] Fetch and cache market info (ticker ↔ clobPairId mapping)
- [ ] Implement MarketData trait methods
  - [ ] Get markets
  - [ ] Get orderbook
  - [ ] Get trades
  - [ ] Get candles
  - [ ] Get server time
- [ ] Implement Account trait methods (read-only)
  - [ ] Get subaccounts
  - [ ] Get balances
  - [ ] Get transfers
- [ ] Implement Positions trait methods
  - [ ] Get positions
  - [ ] Get PnL
  - [ ] Get funding payments
- [ ] Symbol normalization (uppercase, validate format)
- [ ] Error handling for 404, 429, etc.

### Phase 2: WebSocket (Real-time Data)
- [ ] Implement WebSocket client
- [ ] Subscribe to channels (orderbook, trades, markets)
- [ ] Handle message types (subscribed, channel_data, error)
- [ ] Automatic reconnection with backoff
- [ ] Message ID tracking
- [ ] Health monitoring

### Phase 3: Write Operations (Node API)
- [ ] Implement gRPC client for Node API
- [ ] Wallet management
  - [ ] Mnemonic handling
  - [ ] Private key derivation
  - [ ] Transaction signing
- [ ] Implement quantums/subticks conversion
  - [ ] Market converter helper struct
  - [ ] Size ↔ quantums
  - [ ] Price ↔ subticks
  - [ ] Validation against step sizes
- [ ] Implement Trading trait methods
  - [ ] Place order (MsgPlaceOrder)
  - [ ] Cancel order (MsgCancelOrder)
  - [ ] Get order status (via Indexer)
- [ ] Sequence number management
- [ ] Gas fee handling
- [ ] Block height tracking (for short-term orders)

### Phase 4: Advanced Features
- [ ] Parent/child subaccount management
- [ ] Isolated margin positions
- [ ] Conditional orders (stop-loss, take-profit)
- [ ] Order batching (multiple orders per block)
- [ ] Local rate limiting
- [ ] Withdrawal operations
- [ ] Compliance screening

## Code Structure (V5 Pattern)

Following KuCoin reference implementation:

```
exchanges/dydx/
├── mod.rs              # Exports
├── endpoints.rs        # URL constants, endpoint enum, symbol formatting
├── auth.rs             # Transaction signing (Cosmos wallet)
├── parser.rs           # JSON/Protobuf parsing
├── connector.rs        # Trait implementations
├── websocket.rs        # WebSocket client
├── quantums.rs         # Quantum/subtick conversion utilities
└── research/           # This documentation
    ├── README.md
    ├── endpoints.md
    ├── authentication.md
    ├── response_formats.md
    ├── symbols.md
    ├── rate_limits.md
    ├── websocket.md
    └── quantums_and_subticks.md
```

## Key Dependencies

```toml
[dependencies]
# HTTP client
reqwest = { version = "0.11", features = ["json"] }

# WebSocket
tokio-tungstenite = "0.21"
futures-util = "0.3"

# JSON
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Decimal math
rust_decimal = "1.33"

# gRPC (for Node API)
tonic = "0.11"
prost = "0.12"

# Cosmos wallet
cosmos-sdk-proto = "0.20"
bip39 = "2.0"  # Mnemonic handling
secp256k1 = "0.27"  # Key derivation

# Async runtime
tokio = { version = "1", features = ["full"] }

# Error handling
thiserror = "1.0"
anyhow = "1.0"
```

## Important Gotchas

### 1. Dual API Architecture
- **Don't confuse** Indexer (read) and Node (write) APIs
- Indexer returns human-readable values
- Node requires quantums/subticks
- Different base URLs and protocols

### 2. Symbol Format
- **Always uppercase**: `BTC-USD` not `btc-usd`
- **Hyphen separator**: `BTC-USD` not `BTC/USD` or `BTCUSD`
- **ticker ≠ clobPairId**: Must map between them

### 3. Quantums/Subticks
- **Don't send decimal values** to gRPC API
- **Convert first**: Use market parameters
- **Validate**: Check stepBaseQuantums and tickSize
- **Use Decimal type**: Avoid floating-point errors

### 4. Authentication
- **No API keys**: Use wallet private key
- **Every write = transaction**: Costs gas
- **Sequence numbers**: Must increment for each transaction
- **Gas balance**: Main account needs DYDX tokens

### 5. Rate Limits
- **Blockchain-level**: 200 short-term orders/block
- **Shared across subaccounts**: All subaccounts under same account
- **Monitor block height**: Schedule orders across blocks
- **Use WebSocket**: Reduce HTTP API load

### 6. Subaccounts
- **0-127**: Parent subaccounts (cross-margin)
- **128-128,000**: Child subaccounts (isolated)
- **Mapping formula**: `parent_id = child_id % 128`
- **Frontend uses 0**: Default subaccount

### 7. Order Types
- **Short-term** (flags=0): High rate limit, 20 blocks lifetime
- **Stateful** (flags=32/64): Low rate limit, long-lived
- **Choose wisely**: Based on strategy needs

## Testing Strategy

### 1. Testnet First
- **Testnet Indexer**: `https://indexer.v4testnet.dydx.exchange/v4`
- **Testnet WebSocket**: `wss://indexer.v4testnet.dydx.exchange/v4/ws`
- **Testnet gRPC**: `oegs-testnet.dydx.exchange:443`
- **Faucet**: `https://faucet.v4testnet.dydx.exchange`

### 2. Read Operations First
- Test market data queries
- Test account queries
- Verify symbol normalization
- Check response parsing

### 3. WebSocket Testing
- Connect and subscribe
- Verify message handling
- Test reconnection logic
- Monitor message latency

### 4. Write Operations (Careful!)
- Start with testnet
- Small order sizes
- Verify quantums/subticks conversion
- Check order appears in Indexer
- Test cancellation

### 5. Integration Testing
- End-to-end order flow
- Market making simulation
- Error handling (network failures, rate limits)
- Reconnection scenarios

## Example Flow: Place Order

```rust
// 1. Fetch market info (cache this)
let markets = indexer.get_perpetual_markets().await?;
let btc_market = markets.get("BTC-USD").unwrap();

// 2. Create market converter
let converter = MarketConverter::new(btc_market);

// 3. Convert human values to protocol values
let size = Decimal::from_str("0.1")?; // 0.1 BTC
let price = Decimal::from_str("50000")?; // 50,000 USD

let quantums = converter.size_to_quantums(size)?;
let subticks = converter.price_to_subticks(price)?;

// 4. Get current block height
let block_info = indexer.get_height().await?;
let good_til_block = block_info.height + 10; // Expires in 10 blocks

// 5. Create and sign order
let order = MsgPlaceOrder {
    subaccount: Subaccount {
        owner: wallet.address(),
        number: 0,
    },
    client_id: generate_client_id(),
    clob_pair_id: btc_market.clob_pair_id.parse()?,
    side: OrderSide::Buy,
    quantums,
    subticks,
    good_til_block,
    order_flags: 0, // Short-term
    time_in_force: TimeInForce::IOC,
    reduce_only: false,
};

let signed_tx = wallet.sign_transaction(order)?;

// 6. Broadcast to Node
let response = node_client.broadcast_transaction(signed_tx).await?;

// 7. Query Indexer for confirmation
tokio::time::sleep(Duration::from_secs(2)).await;
let orders = indexer.get_orders(wallet.address(), 0).await?;
let our_order = orders.iter().find(|o| o.client_id == order.client_id);
```

## Resources

### Official Documentation
- **Main Docs**: https://docs.dydx.xyz
- **API Docs**: https://docs.dydx.exchange
- **Indexer API**: https://docs.dydx.xyz/indexer-client/http
- **WebSocket**: https://docs.dydx.xyz/indexer-client/websockets

### GitHub Repositories
- **v4-chain**: https://github.com/dydxprotocol/v4-chain
- **v4-clients**: https://github.com/dydxprotocol/v4-clients
- **TypeScript Client**: https://github.com/dydxprotocol/v4-clients/tree/main/v4-client-js
- **Python Client**: https://github.com/dydxprotocol/v4-clients/tree/main/v4-client-py

### Community
- **Discord**: https://discord.gg/dydx
- **Forums**: https://forums.dydx.community
- **Governance**: https://dydx.community

### Cosmos Resources
- **Cosmos SDK**: https://docs.cosmos.network
- **CosmJS** (TypeScript): https://github.com/cosmos/cosmjs
- **Cosmos Rust**: https://github.com/cosmos/cosmos-rust

## Next Steps

1. **Read all documentation files** in this directory
2. **Study KuCoin V5 implementation** as reference
3. **Start with Indexer API** (read-only, easier)
4. **Test on testnet** before mainnet
5. **Implement incrementally** (MarketData → Positions → Account → Trading)
6. **Use official clients as reference** (TypeScript/Python)
7. **Ask questions** in dYdX Discord #developers channel

## Notes for Rust Implementation

### Critical Differences from Traditional CEX
1. **No HMAC signatures** - Use Cosmos transaction signing instead
2. **Integer protocol values** - Always convert decimal ↔ quantums/subticks
3. **Two separate clients** - Indexer (HTTP) and Node (gRPC)
4. **Blockchain state** - Track block height, sequence numbers
5. **Gas management** - Need DYDX tokens for transactions

### Recommended Approach
1. Implement Indexer client first (simpler, no auth)
2. Add WebSocket for real-time data
3. Implement wallet/signing (most complex part)
4. Add gRPC Node client
5. Implement order placement with careful testing

### Testing Emphasis
- **More testing needed than typical CEX** due to:
  - Quantum/subtick conversions
  - Blockchain transaction complexity
  - Rate limit calculations
  - Sequence number management

### Performance Considerations
- Cache market info (doesn't change often)
- Use WebSocket for real-time data
- Batch orders within same block when possible
- Consider self-hosted indexer for production HFT

---

**Last Updated**: 2026-01-20

**Status**: Complete research for V5 connector implementation

**Next**: Begin implementation following V5 architecture pattern
