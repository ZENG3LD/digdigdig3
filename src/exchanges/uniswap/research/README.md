# Uniswap V5 Connector Research

## Overview

Comprehensive research documentation for implementing a Uniswap connector in the V5 architecture.

**Date:** 2026-01-20
**Exchange:** Uniswap (Decentralized Exchange)
**Blockchain:** Ethereum + Multi-chain (Arbitrum, Polygon, Optimism, Base, etc.)
**Protocol Versions:** V2, V3, V4

---

## Research Documents

### 1. [endpoints.md](./endpoints.md)
Complete API endpoint documentation covering:
- Trading API (REST) - Quote, swap, order status
- The Graph Subgraph (GraphQL) - Historical data, analytics
- Smart Contract methods - On-chain interactions
- Routing API - Optimal path finding

**Key Endpoints:**
- `POST /quote` - Get swap quotes
- `POST /swap` - Execute swaps
- GraphQL pools/swaps/tokens queries
- Contract methods: `slot0()`, `quoteExactInputSingle()`

---

### 2. [authentication.md](./authentication.md)
Authentication methods for all API types:
- Trading API: `x-api-key` header
- The Graph: API key in URL path
- Smart Contracts: Transaction signing with private keys
- Permit2: Off-chain EIP-712 signatures
- WebSocket: Node provider authentication

**Security Best Practices:**
- Environment variables for keys
- Separate keys per environment
- Hardware wallets for manual operations
- Never commit secrets to version control

---

### 3. [response_formats.md](./response_formats.md)
Detailed response structures for all APIs:
- Trading API JSON responses
- GraphQL query results
- Smart contract return values
- WebSocket event formats
- Error responses (400, 401, 429, 500, 504)

**Data Types:**
- Token amounts: String (to avoid precision loss)
- Addresses: Checksummed hex strings
- Timestamps: Unix seconds
- All amounts in smallest unit (wei, 10^decimals)

---

### 4. [symbols.md](./symbols.md)
Token addressing and symbol formatting:
- ERC-20 token addresses (not symbol-based like CEX)
- Common token addresses (WETH, USDC, USDT, DAI, etc.)
- Pool address computation
- Native ETH vs WETH distinction
- Fee tiers (0.01%, 0.05%, 0.30%, 1.00%)
- Multi-chain token addresses

**Key Concept:**
Uniswap uses **contract addresses** for tokens, not symbols. Multiple pools exist per pair with different fee tiers.

---

### 5. [rate_limits.md](./rate_limits.md)
Rate limits and usage restrictions:
- Trading API: 12 requests/second (default)
- The Graph: Budget-based limits
- JSON-RPC providers: Variable (Infura, Alchemy, Chainstack)
- Smart contracts: Gas limits only

**Best Practices:**
- Request queuing with token bucket
- Exponential backoff for 429 errors
- Caching static data
- Batch requests (Multicall, GraphQL)
- WebSocket subscriptions over polling

---

### 6. [websocket.md](./websocket.md)
Real-time event monitoring:
- Ethereum WebSocket subscriptions (no native Uniswap WS)
- Event types: Swap, Mint, Burn, Collect
- Subscription methods: `newHeads`, `logs`, `newPendingTransactions`
- Event decoding with alloy/ethers
- Reconnection strategies
- Performance considerations

**Use Cases:**
- Price tracking
- Arbitrage detection
- Transaction monitoring
- Volume analysis

---

## Architecture Differences from CEX

### Uniswap (DEX)
- **Token IDs**: Contract addresses (0x...)
- **Pairs**: Token0 + Token1 + Fee tier
- **Multiple pools** per pair (different fees)
- **On-chain execution**: Requires gas
- **No order book**: AMM (Automated Market Maker)
- **Permissionless**: Anyone can create pools
- **Wallet required**: Direct blockchain interaction

### Centralized Exchange (CEX)
- **Symbols**: BTC, ETH, USDT
- **Pairs**: BTC/USDT, ETH/USD
- **Single market** per pair
- **Free execution** (exchange fees only)
- **Order book**: Limit/market orders
- **Exchange-controlled**: Listing process
- **Account-based**: API keys only

---

## Implementation Checklist

### Required Components

#### 1. Endpoints Module (`endpoints.rs`)
- [ ] Define API base URLs (Trading API, The Graph)
- [ ] Endpoint enum for all operations
- [ ] Token address formatting helpers
- [ ] Pool address computation
- [ ] Multi-chain support

#### 2. Authentication Module (`auth.rs`)
- [ ] API key management
- [ ] Transaction signing (ethers/alloy)
- [ ] Permit2 signature generation
- [ ] Nonce tracking
- [ ] Gas price estimation

#### 3. Parser Module (`parser.rs`)
- [ ] JSON response parsing (Trading API)
- [ ] GraphQL response parsing (The Graph)
- [ ] Event log decoding (Swap, Mint, Burn)
- [ ] ABI encoding/decoding
- [ ] Error response handling

#### 4. Connector Module (`connector.rs`)
- [ ] Implement `MarketData` trait
  - [ ] `get_orderbook()` - N/A for AMM, use pool liquidity
  - [ ] `get_ticker()` - Current pool price/liquidity
  - [ ] `get_recent_trades()` - Recent swaps from subgraph
  - [ ] `get_klines()` - Historical OHLCV (aggregate swaps)
- [ ] Implement `Trading` trait (if applicable)
  - [ ] `place_order()` - Execute swap
  - [ ] `cancel_order()` - N/A (no pending orders)
  - [ ] `get_order_status()` - Check transaction status
- [ ] Implement `Account` trait
  - [ ] `get_balances()` - Token balances via RPC
  - [ ] `get_positions()` - LP positions from subgraph

#### 5. WebSocket Module (`websocket.rs`)
- [ ] Ethereum WebSocket connection
- [ ] Subscribe to pool events (Swap, Mint, Burn)
- [ ] Event decoding and dispatching
- [ ] Auto-reconnection logic
- [ ] Heartbeat/ping-pong

#### 6. Utils Module
- [ ] Rate limiter (token bucket)
- [ ] Retry logic with exponential backoff
- [ ] Cache for token metadata
- [ ] Multicall batching
- [ ] Price calculation helpers (sqrtPriceX96 → decimal)

---

## Smart Contract Addresses

### Ethereum Mainnet

```rust
pub const V2_ROUTER: &str = "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D";
pub const V2_FACTORY: &str = "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f";
pub const V3_ROUTER: &str = "0xE592427A0AEce92De3Edee1F18E0157C05861564";
pub const V3_FACTORY: &str = "0x1F98431c8aD98523631AE4a59f267346ea31F984";
pub const V3_QUOTER: &str = "0xb27308f9F90D607463BB33eA1BeBb41C27CE5AB6";
pub const V4_UNIVERSAL: &str = "0x66a9893Cc07D91D95644aEDd05d03f95E1DBa8Af";
```

### Common Tokens

```rust
pub const WETH: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
pub const USDC: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
pub const USDT: &str = "0xdAC17F958D2ee523a2206206994597C13D831ec7";
pub const DAI: &str = "0x6B175474E89094C44Da98b954EedeAC495271d0F";
```

---

## Configuration Example

```toml
[uniswap]
trading_api_url = "https://trade-api.gateway.uniswap.org/v1"
trading_api_key = "${UNISWAP_API_KEY}"

subgraph_url = "https://gateway.thegraph.com/api/${THE_GRAPH_API_KEY}/subgraphs/id/5zvR82QoaXYFyDEKLZ9t6v9adgnptxYpKpSbxtgVENFV"

rpc_url = "https://mainnet.infura.io/v3/${INFURA_PROJECT_ID}"
ws_url = "wss://mainnet.infura.io/ws/v3/${INFURA_PROJECT_ID}"

chain_id = 1  # Ethereum mainnet
wallet_private_key = "${WALLET_PRIVATE_KEY}"  # For signing transactions

rate_limit = 12  # requests per second
```

---

## Special Considerations

### 1. Gas Costs
Every transaction requires gas payment in ETH. Must account for:
- Gas price fluctuations
- Failed transactions still cost gas
- Approval transactions (first-time token use)
- Permit2 can save gas on approvals

### 2. Slippage
AMM pools have price impact. Large trades move price:
- Calculate slippage tolerance
- Set minimum output amount
- Monitor pool liquidity depth

### 3. MEV (Miner Extractable Value)
Transactions visible in mempool before execution:
- Front-running risk
- Consider private mempools (Flashbots)
- Use deadline parameters

### 4. Multiple Fee Tiers
Same pair has 4 different pools:
- 0.01% (100) - Stablecoins
- 0.05% (500) - Low volatility
- 0.30% (3000) - Standard
- 1.00% (10000) - Exotic
- Routing API finds best automatically

### 5. Native ETH Handling
ETH is not ERC-20:
- Must wrap to WETH for trading
- Router can auto-wrap if `msg.value > 0`
- Remember to unwrap back if needed

---

## Testing Strategy

### Unit Tests
- Token address validation
- Event decoding
- Price calculations (sqrtPriceX96)
- Error handling

### Integration Tests (Testnet)
- Connect to Goerli/Sepolia testnet
- Execute test swaps
- Monitor events
- Handle failures

### Mainnet Testing
- Start with read-only operations
- Use small amounts initially
- Monitor gas costs
- Test error recovery

---

## Resources

### Official Documentation
- [Uniswap Docs](https://docs.uniswap.org/)
- [Trading API Docs](https://api-docs.uniswap.org/)
- [The Graph Docs](https://thegraph.com/docs/)

### GitHub Repositories
- [Uniswap V3 Core](https://github.com/Uniswap/v3-core)
- [Uniswap V3 SDK](https://github.com/Uniswap/v3-sdk)
- [Routing API](https://github.com/Uniswap/routing-api)

### Tools
- [Uniswap Info](https://info.uniswap.org/) - Analytics
- [Etherscan](https://etherscan.io/) - Block explorer
- [The Graph Studio](https://thegraph.com/studio/) - API keys

---

## Next Steps

1. **Review Research**: Read all 6 documents thoroughly
2. **Setup Development Environment**:
   - Get API keys (Uniswap, The Graph, Infura/Alchemy)
   - Create test wallet (testnet only)
   - Install Rust dependencies (alloy, reqwest, tokio)
3. **Implement Core Modules**:
   - Start with `endpoints.rs` and `parser.rs`
   - Add authentication and signing
   - Implement connector trait methods
4. **Test on Testnet**:
   - Use Goerli or Sepolia
   - Execute test swaps
   - Monitor events
5. **Production Deployment**:
   - Mainnet testing with small amounts
   - Production rate limiting
   - Monitoring and alerting

---

## Questions or Issues

For questions about this research or implementation:
1. Review the specific document (endpoints, auth, etc.)
2. Check official Uniswap documentation
3. Search GitHub issues in Uniswap repositories
4. Ask in Uniswap Discord/Forum

---

**Research Complete**: All 6 required documents created with comprehensive technical details for V5 connector implementation.
