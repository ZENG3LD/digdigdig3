# Jupiter Aggregator Connector

## Architecture

Jupiter is a **DEX aggregator** that routes trades across 20+ Solana DEXes to find the best prices.

### Aggregated DEXes
Jupiter aggregates liquidity from:
- AMM Pools: Raydium, Orca, Meteora, Lifinity, Cropper
- Orderbooks: Phoenix, OpenBook
- Hybrid: Invariant, GooseFX, Balansol
- 50%+ of all Solana DEX volume

### Supported Operations

| Operation | Status | Notes |
|-----------|--------|-------|
| `get_price()` | ✅ Supported | Via Price API v3 |
| `get_ticker()` | ✅ Supported | Via Price API v3 |
| `get_orderbook()` | ❌ Unsupported | Aggregator has no orderbook |
| `get_klines()` | ❌ Unsupported | No historical data |
| WebSocket | ❌ Not Available | REST-only API |

### Real-time Updates

Jupiter API is **REST-only** with no WebSocket support.

**Recommended approach:**
- Poll Price API: every 5-10 seconds
- Poll Quote API: every 1-2 seconds
- Implement caching with 2-5 second TTL
- Use exponential backoff on rate limits

**Rate Limits:**
- Free tier: 60 requests / 60 seconds (~1/sec)
- Pro I: 100 requests / 10 seconds (~10/sec)
- Pro IV: 5,000 requests / 10 seconds (~500/sec)

### API Migration (Oct 2025)

⚠️ **Important**: Quote API v6 was deprecated on October 1, 2025.

- Old: `https://quote-api.jup.ag/v6`
- New: `https://api.jup.ag/swap/v1`
- **All endpoints require API key** (free tier available)

### Authentication

All API requests require `x-api-key` header:
```bash
export JUPITER_API_KEY=your_key_here
```

Get API key: https://portal.jup.ag/api-keys

### Alternative Real-time Options

Since Jupiter has no native WebSocket:
1. **Solana RPC**: Monitor Jupiter program transactions
2. **QuickNode Metis**: Third-party streaming API (p99 <500ms)
3. **Bitquery**: Jupiter event subscriptions via GraphQL WebSocket

### References
- API Portal: https://portal.jup.ag
- Documentation: https://dev.jup.ag
- Status: https://status.jup.ag
- GitHub: https://github.com/jup-ag/jupiter-swap-api-client
