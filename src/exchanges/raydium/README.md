# Raydium DEX Connector

## Architecture

Raydium is a **pure AMM (Automated Market Maker)** on Solana using the constant product formula (x × y = k).

### Pool Types
- **AMM v4**: Legacy standard pools, 0.25% fee
- **CPMM**: New constant product pools, 4 fee tiers (0.25%, 1%, 2%, 4%)
- **CLMM**: Concentrated liquidity (like Uniswap v3)

### Supported Operations

| Operation | Status | Notes |
|-----------|--------|-------|
| `get_price()` | ✅ Supported | Via API v3 `/main/price` |
| `get_ticker()` | ✅ Supported | Via API v3 |
| `get_orderbook()` | ❌ Unsupported | AMM pools have no orderbooks |
| `get_klines()` | ❌ Unsupported | Not provided by API |
| WebSocket | ⚠️ Limited | Via Solana RPC `accountSubscribe` |

### WebSocket Implementation

Current implementation uses Solana RPC WebSocket to subscribe to pool account updates:
- Protocol: `wss://api.mainnet-beta.solana.com`
- Method: `accountSubscribe` on AMM pool addresses
- Timeout: 5 minutes (requires reconnection logic)
- Recommended: Use Yellowstone Geyser gRPC for production

### API Stability

⚠️ **Warning**: Raydium API v3 is not suitable for real-time or development purposes.
- Frequent 500 errors
- Rate limiting
- Unstable pool queries

For production, use:
1. Solana RPC to query pool state directly
2. gRPC (Yellowstone Geyser) for real-time updates
3. On-chain program calls

### References
- API v3: https://api-v3.raydium.io
- Docs: https://docs.raydium.io
- SDK: https://github.com/raydium-io/raydium-sdk-V2
