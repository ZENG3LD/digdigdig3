# GMX V2 Exchange Research

Complete API research documentation for implementing GMX V2 connector in the V5 architecture.

## Overview

**GMX** is a decentralized perpetual exchange operating on Arbitrum, Avalanche, and Botanix. Unlike centralized exchanges, GMX:
- Operates entirely on-chain via smart contracts
- Requires wallet signatures for trading (no API keys)
- Uses REST APIs only for read-only market data
- Executes trades through blockchain transactions

## Research Documents

### 1. [endpoints.md](./endpoints.md)
Complete documentation of all REST API endpoints and smart contract interaction methods.

**Covers:**
- REST API base URLs and fallback endpoints
- MarketData trait endpoints (tickers, candles, markets)
- Trading trait contract methods (ExchangeRouter)
- Account trait position/order queries (Reader contract)
- Positions trait via GraphQL (Subsquid)
- Contract addresses for Arbitrum and Avalanche
- Request/response formats for all endpoints

**Key Endpoints:**
- `GET /prices/tickers` - Current prices
- `GET /prices/candles` - OHLC data
- `GET /markets/info` - Market liquidity, OI, rates
- `GET /signed_prices/latest` - Signed prices for trading
- Smart contracts for order creation and position management

### 2. [authentication.md](./authentication.md)
Wallet-based authentication and transaction signing documentation.

**Covers:**
- No traditional API authentication (public REST endpoints)
- Wallet integration requirements (Ethereum-compatible)
- Transaction signing flow (ERC20 approval, order creation)
- EIP-712 typed data signing
- Oracle price signatures
- Gas management and execution fees
- Security best practices
- Private key management

**Key Concepts:**
- Trading requires blockchain wallet with private key
- Token approval before trading
- Two-step order flow: create order → keeper execution
- Execution fees paid in ETH/AVAX

### 3. [response_formats.md](./response_formats.md)
Detailed response formats for all API endpoints and contract calls.

**Covers:**
- REST API JSON response structures
- Smart contract return types (structs)
- GraphQL query response formats
- Price precision (30 decimals)
- Position and order data structures
- Error response formats
- Event log structures

**Key Details:**
- All prices use 30-decimal precision
- Min/max price spread for conservative PnL calculations
- Complex nested structures for market info
- Position struct with addresses, numbers, flags

### 4. [symbols.md](./symbols.md)
Market naming conventions and token symbol mapping.

**Covers:**
- Market naming format: `{INDEX}/{QUOTE} [{LONG}-{SHORT}]`
- Index tokens vs collateral tokens vs market tokens (GM)
- Token address mapping for Arbitrum and Avalanche
- Symbol normalization (WETH → ETH, bridged tokens)
- Perpetual vs spot markets
- Single-token vs dual-token markets
- Collateral selection for long/short positions
- GLV tokens (liquidity vaults)

**Examples:**
- `ETH/USD [ETH-USDC]` - Long ETH with ETH collateral, short with USDC
- `BTC/USD [BTC-USDT]` - Long BTC with BTC collateral, short with USDT
- Multiple pools per index token (different collateral pairs)

### 5. [rate_limits.md](./rate_limits.md)
Rate limiting strategies and best practices.

**Covers:**
- No published GMX API rate limits
- Recommended conservative limits (10 req/s for tickers)
- Blockchain transaction rate limits
- RPC node limitations (public vs private)
- Fallback URL rotation strategy
- Retry and backoff strategies
- Caching strategies for different data types
- Circuit breaker pattern

**Recommendations:**
- REST API: 5-10 req/s per endpoint
- Smart contract reads: 20 req/s (RPC dependent)
- Smart contract writes: 1 tx per 2-3 seconds
- Use private RPC providers for production
- Implement caching for static/semi-static data

### 6. [websocket.md](./websocket.md)
Real-time data strategies (GMX has no native WebSocket API).

**Covers:**
- No native WebSocket API available
- Alternative 1: Polling REST endpoints (1-5 second intervals)
- Alternative 2: Blockchain event subscriptions (WebSocket RPC)
- Alternative 3: GraphQL subscriptions (if supported by Subsquid)
- Alternative 4: Oracle price feed events
- Hybrid approach combining multiple methods
- WebSocket reconnection and heartbeat strategies
- Candlestick streaming via polling or tick aggregation

**Recommended Approach:**
- WebSocket RPC for order/position events (real-time)
- REST polling for prices (2-5 seconds)
- REST polling for market data (30-60 seconds)
- GraphQL polling for historical data (5-15 minutes)

## Architecture Summary

### GMX V2 System Components

```
┌─────────────────────────────────────────────────────────────┐
│                        User (Trader)                        │
└────────────┬──────────────────────────┬─────────────────────┘
             │                          │
             │ 1. Query data            │ 2. Submit trades
             │    (REST API)            │    (Blockchain txs)
             │                          │
    ┌────────▼────────┐        ┌────────▼──────────┐
    │  GMX REST API   │        │  Smart Contracts  │
    │  (Read-only)    │        │  (ExchangeRouter) │
    └────────┬────────┘        └────────┬──────────┘
             │                          │
             │ - Tickers                │ - createOrder()
             │ - Candles                │ - cancelOrder()
             │ - Markets                │ - updateOrder()
             │ - Signed prices          │
             │                          │
    ┌────────▼────────┐        ┌────────▼──────────┐
    │   Oracle        │        │   Order Keepers   │
    │   Keepers       │───────▶│   (Off-chain)     │
    └─────────────────┘        └────────┬──────────┘
       - Price feeds                    │
       - Sign prices             3. Execute orders
                                        │ with oracle prices
                               ┌────────▼──────────┐
                               │   Order Vault     │
                               │   DataStore       │
                               │   Markets         │
                               └───────────────────┘
```

### Data Flow

**Market Data (Read):**
1. Query GMX REST API endpoints
2. Parse JSON responses
3. Handle 30-decimal price precision
4. Cache appropriately

**Trading (Write):**
1. Approve tokens (ERC20)
2. Transfer collateral to OrderVault
3. Create order via ExchangeRouter
4. Wait for keeper execution (~5-30 seconds)
5. Monitor OrderExecuted events

**Position Monitoring:**
1. Query Reader contract for positions
2. Subscribe to blockchain events (PositionIncrease/Decrease)
3. Poll GraphQL for historical data
4. Calculate unrealized PnL client-side

## Implementation Roadmap

### Phase 1: Market Data (MarketData Trait)

**Priority: High**

- [ ] REST client with fallback URLs
- [ ] Endpoint: Get tickers
- [ ] Endpoint: Get candlesticks (OHLC)
- [ ] Endpoint: Get markets list
- [ ] Endpoint: Get market info (detailed)
- [ ] Endpoint: Get tokens list
- [ ] 30-decimal price parsing
- [ ] Symbol mapping (address ↔ symbol)
- [ ] Caching layer for static data

**Estimated Effort:** 2-3 days

### Phase 2: Authentication & Wallet Integration

**Priority: High**

- [ ] Wallet initialization from private key
- [ ] Transaction signing utilities
- [ ] Gas price estimation
- [ ] Execution fee calculation
- [ ] Nonce management
- [ ] Environment-based key loading

**Estimated Effort:** 1-2 days

### Phase 3: Trading (Trading Trait)

**Priority: Medium**

- [ ] Token approval function
- [ ] CreateOrder parameter builder
- [ ] Market order creation
- [ ] Limit order creation
- [ ] Order cancellation
- [ ] Transaction monitoring
- [ ] Event parsing (OrderCreated, OrderExecuted)
- [ ] Error handling (revert reasons)

**Estimated Effort:** 3-4 days

### Phase 4: Positions & Account (Positions + Account Traits)

**Priority: Medium**

- [ ] Reader contract integration
- [ ] Get account positions
- [ ] Get account orders
- [ ] Get position details
- [ ] Calculate unrealized PnL
- [ ] Position monitoring via events
- [ ] GraphQL client for historical data

**Estimated Effort:** 2-3 days

### Phase 5: Real-Time Data (WebSocket Alternative)

**Priority: Low**

- [ ] WebSocket RPC connection
- [ ] Event subscription (orders, positions)
- [ ] Event filtering by account
- [ ] Reconnection logic
- [ ] Missed event synchronization
- [ ] Price polling loop
- [ ] Unified event channel

**Estimated Effort:** 2-3 days

### Phase 6: Advanced Features

**Priority: Low**

- [ ] Stop-loss/take-profit orders
- [ ] Conditional orders
- [ ] Position modification
- [ ] Swap functionality
- [ ] Liquidity provision (deposits/withdrawals)
- [ ] GLV token operations

**Estimated Effort:** 3-5 days

## Key Differences from Centralized Exchanges

### 1. No API Keys
- REST endpoints are public (no auth)
- Trading uses wallet signatures
- No rate limit quotas per user

### 2. Asynchronous Order Execution
- Orders don't execute immediately
- Keepers execute orders off-chain
- 5-30 second delay typical
- Order keys for tracking

### 3. On-Chain State
- All positions stored on blockchain
- Query via smart contracts or indexers
- No centralized database
- Blockchain events for updates

### 4. Gas Fees
- Every trade costs gas (ETH/AVAX)
- Execution fees for keepers
- Variable costs based on network congestion

### 5. Decentralized Pricing
- Oracle-based prices (not orderbook)
- Signed price feeds
- Min/max price spread
- Price impact based on pool utilization

### 6. Market Structure
- Markets = liquidity pools (GM tokens)
- Multiple pools per asset (different collateral)
- Long/short collateral separation
- Isolated pool risk

## Testing Considerations

### Testnet Support

**Arbitrum Sepolia:**
- Use for development testing
- Faucet available for test ETH
- Separate contract addresses

**Testnet Limitations:**
- Lower liquidity
- Fewer markets
- Different oracle behavior
- May have different features

### Mainnet Testing

**Start Small:**
- Test with minimum position sizes
- Use stable markets (ETH/USD, BTC/USD)
- Monitor gas costs carefully
- Verify all calculations

**Risk Management:**
- Implement strict position limits
- Use stop-losses
- Monitor liquidation risk
- Track execution fees

## Common Pitfalls

### 1. Decimal Precision
- GMX uses **30 decimals** for prices
- Token amounts use **token decimals** (6, 8, 18)
- Must convert correctly

### 2. Two-Step Order Process
- Transfer collateral AND create order in one transaction
- Use multicall to avoid front-running
- Order creation ≠ order execution

### 3. Oracle Price Requirements
- Signed prices required for transactions
- Prices must be recent (<60 seconds)
- Fetch from `/signed_prices/latest`

### 4. Market Selection
- Same index can have multiple markets
- Choose based on collateral preference
- Check liquidity and OI limits

### 5. Gas Management
- Dynamic gas prices
- Execution fees separate from gas
- Failed transactions still cost gas

## Useful Resources

### Official Documentation
- [GMX Docs](https://docs.gmx.io/)
- [GMX Trading Guide](https://docs.gmx.io/docs/trading/v2/)
- [GMX SDK](https://docs.gmx.io/docs/sdk/)
- [GMX REST API](https://docs.gmx.io/docs/api/rest/)

### GitHub Repositories
- [gmx-synthetics](https://github.com/gmx-io/gmx-synthetics) - Smart contracts
- [gmx-interface](https://github.com/gmx-io/gmx-interface) - Frontend (SDK reference)
- [gmx-subgraph](https://github.com/gmx-io/gmx-subgraph) - GraphQL indexer

### Tools & Explorers
- [Arbiscan](https://arbiscan.io/) - Arbitrum block explorer
- [SnowTrace](https://snowtrace.io/) - Avalanche block explorer
- [GMX Stats](https://stats.gmx.io/) - Protocol analytics
- [The Graph Explorer](https://thegraph.com/explorer/) - Subgraph queries

### Community
- Discord: [GMX Discord](https://discord.gg/gmx)
- Twitter: [@GMX_IO](https://twitter.com/GMX_IO)
- Forum: [gov.gmx.io](https://gov.gmx.io/)

## Next Steps

1. **Review all research documents** to understand GMX architecture
2. **Set up development environment** with Arbitrum Sepolia testnet
3. **Implement MarketData trait** for read-only functionality
4. **Test wallet integration** with small transactions
5. **Implement Trading trait** with order creation
6. **Add position monitoring** via Reader contract
7. **Optimize with caching** and real-time updates
8. **Production deployment** with risk management

## Questions & Clarifications

If implementing the GMX connector, consider:

**Architecture Decisions:**
- Which blockchain library? (ethers-rs, web3, alloy)
- How to handle async execution model?
- Event-driven vs polling for positions?

**Trait Coverage:**
- Full MarketData support? (Yes - REST API complete)
- Full Trading support? (Yes - via smart contracts)
- Full Account support? (Yes - Reader contract + GraphQL)
- Full Positions support? (Yes - Reader + events + GraphQL)

**Special Features:**
- GLV token operations?
- Liquidity provision?
- Cross-chain operations?

## Conclusion

GMX V2 is a fully decentralized perpetual exchange requiring a different integration approach than centralized exchanges. The connector must:

1. Use REST API for market data only
2. Use blockchain transactions for trading
3. Integrate wallet signing for authentication
4. Handle asynchronous order execution
5. Monitor blockchain events for real-time updates
6. Manage gas fees and execution costs

All necessary information for implementation is documented in the research files. Follow the V5 architecture pattern established by KuCoin reference implementation, adapting for blockchain-based trading.

---

**Research completed:** 2026-01-20
**GMX Version:** V2 (Synthetics)
**Chains Covered:** Arbitrum, Avalanche, Botanix
**Documentation Status:** Complete
