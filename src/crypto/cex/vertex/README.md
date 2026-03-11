# Vertex Protocol Connector

> ## ⚠️ **SERVICE PERMANENTLY SHUT DOWN** ⚠️
>
> **Vertex Protocol was acquired by Ink Foundation (Kraken-backed L2) and completely shut down on August 14, 2025.**
>
> - **All endpoints are offline** (REST, WebSocket, Indexer)
> - **This connector will not work** - kept for reference only
> - **Timeline:**
>   - July 8, 2025: Acquisition announced
>   - August 14, 2025: Complete service termination
>
> **Alternatives for perpetuals trading:**
> - GMX (Arbitrum perpetuals DEX)
> - dYdX V4 (standalone L1)
> - Hyperliquid (perpetuals L1)
>
> See: `research/vertex/ENDPOINTS_DEEP_RESEARCH.md` for full shutdown details

---

Vertex Protocol is a decentralized exchange (DEX) on Arbitrum that offers perpetual futures and spot trading with off-chain order matching and on-chain settlement.

## Features

- ✅ **Market Data**: Real-time prices, orderbook, klines, ticker
- ✅ **Trading**: Market orders, limit orders, order management
- ✅ **Account**: Balance queries, account info
- ✅ **Positions**: Position tracking, funding rates
- ✅ **WebSocket**: Real-time market data and user data streams
- ✅ **EIP-712 Signatures**: Full support for Vertex's Ethereum-based authentication

## Architecture

Vertex uses a unique hybrid architecture:
- **Off-chain order matching** (fast, low latency)
- **On-chain settlement** (decentralized, trustless)
- **EIP-712 signatures** (Ethereum wallet authentication)

## Usage

### Public API (No Authentication)

```rust
use digdigdig3::exchanges::vertex::VertexConnector;
use digdigdig3::core::{Symbol, AccountType, MarketData};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create public connector (mainnet)
    let connector = VertexConnector::public(false).await?;

    // Get BTC-PERP price
    let symbol = Symbol {
        base: "BTC".to_string(),
        quote: "PERP".to_string(),
    };

    let price = connector.get_price(symbol, AccountType::FuturesCross).await?;
    println!("BTC-PERP price: ${:.2}", price);

    Ok(())
}
```

### Private API (With Authentication)

```rust
use digdigdig3::exchanges::vertex::VertexConnector;
use digdigdig3::core::{Credentials, Symbol, AccountType, Trading, OrderSide};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create credentials from Ethereum private key
    let credentials = Credentials {
        api_key: "0x...".to_string(),      // Ethereum address
        api_secret: "0x...".to_string(),   // Private key (without 0x prefix)
        passphrase: None,
    };

    // Create authenticated connector
    let connector = VertexConnector::new(Some(credentials), false).await?;

    // Place market order
    let symbol = Symbol {
        base: "BTC".to_string(),
        quote: "PERP".to_string(),
    };

    let order = connector.market_order(
        symbol,
        OrderSide::Buy,
        0.01, // quantity in BTC
        AccountType::FuturesCross,
    ).await?;

    println!("Order placed: {:?}", order);

    Ok(())
}
```

### WebSocket Streams

```rust
use digdigdig3::exchanges::vertex::VertexWebSocket;
use digdigdig3::core::{Symbol, AccountType, StreamType, WebSocketConnector};
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create WebSocket connection
    let mut ws = VertexWebSocket::new(None, false).await?;

    // Connect to server
    ws.connect().await?;

    // Subscribe to BTC-PERP trades
    let symbol = Symbol {
        base: "BTC".to_string(),
        quote: "PERP".to_string(),
    };

    ws.subscribe(vec![StreamType::Trade], vec![symbol], AccountType::FuturesCross).await?;

    // Listen for events
    while let Some(event) = ws.next().await {
        match event {
            Ok(stream_event) => {
                println!("Event: {:?}", stream_event);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        }
    }

    Ok(())
}
```

## Symbol Format

Vertex uses a simple symbol format:
- **Spot**: `BTC`, `ETH`, `SOL` (base asset only)
- **Perpetuals**: `BTC-PERP`, `ETH-PERP`, `SOL-PERP`

All markets quote in USDC.

## API Endpoints

### Mainnet (Arbitrum One)
- **REST**: `https://gateway.prod.vertexprotocol.com/v1`
- **WebSocket**: `wss://gateway.prod.vertexprotocol.com/v1/ws`
- **Indexer**: `https://archive.prod.vertexprotocol.com/v1`

### Testnet (Arbitrum Sepolia)
- **REST**: `https://gateway.sepolia-test.vertexprotocol.com/v1`
- **WebSocket**: `wss://gateway.sepolia-test.vertexprotocol.com/v1/ws`
- **Indexer**: `https://archive.sepolia-test.vertexprotocol.com/v1`

## Rate Limits

- **Default**: 100 requests per 10 seconds
- **Weight**: Each request counts as 1 weight
- **Automatic retry**: Enabled with exponential backoff

## Authentication

Vertex uses **EIP-712** (Ethereum) signatures for authentication:

1. Generate a nonce (timestamp-based)
2. Create EIP-712 typed data structure
3. Sign with Ethereum private key
4. Include signature in request

The connector handles all signature generation automatically.

### Required Credentials

- **API Key**: Ethereum address (0x...)
- **API Secret**: Ethereum private key (without 0x prefix)

## Status

### Service Shutdown (August 14, 2025)

Vertex Protocol has been **permanently shut down** following its acquisition by Ink Foundation.

**All API endpoints are permanently offline:**
- REST API: `gateway.prod.vertexprotocol.com` - DEAD
- WebSocket: `wss://gateway.prod.vertexprotocol.com/v1/ws` - DEAD
- Indexer: `archive.prod.vertexprotocol.com` - DEAD
- All testnet endpoints - DEAD

**What happened:**
- July 8, 2025: Vertex acquired by Ink Foundation (Kraken-backed L2)
- August 14, 2025: Complete service termination
- Protocol migrated to Ink L2 architecture

**This connector:**
- Will not work (all endpoints dead)
- Kept for reference and code patterns
- Tests verify graceful error handling

See `research/vertex/ENDPOINTS_DEEP_RESEARCH.md` for complete timeline and details.

## Testing

```bash
# Run all tests (handles connectivity issues gracefully)
cargo test --package digdigdig3 --test vertex_integration -- --nocapture
cargo test --package digdigdig3 --test vertex_websocket -- --nocapture

# Run specific test
cargo test --package digdigdig3 --test vertex_integration test_get_price -- --nocapture
```

## Module Structure

```
vertex/
├── mod.rs          # Public exports
├── endpoints.rs    # URL constants, endpoint enum, symbol formatting
├── auth.rs         # EIP-712 signature implementation
├── parser.rs       # JSON response parsing
├── connector.rs    # Trait implementations (MarketData, Trading, Account, Positions)
└── websocket.rs    # WebSocket implementation
```

## References

- **Documentation**: https://docs.vertexprotocol.com
- **API Docs**: https://vertex-protocol.gitbook.io/docs
- **Website**: https://vertexprotocol.com
- **Network**: Arbitrum One (Chain ID: 42161)

## Implementation Notes

### Market Orders

Vertex doesn't have true market orders. Market orders are simulated using IOC (Immediate-Or-Cancel) limit orders at extreme prices:
- **Buy**: 10% above current market price
- **Sell**: 10% below current market price

### Leverage

Vertex uses **dynamic cross-margin** by default. The `set_leverage` method is not supported as leverage is automatically calculated based on collateral.

### Product IDs

Vertex uses numeric product IDs internally. The connector automatically resolves symbols to product IDs by querying the `/query` endpoint with `type=all_products`.

### x18 Encoding

Prices and quantities are encoded as "x18" integers (value × 10^18) for on-chain compatibility. The connector handles this conversion automatically.

## Error Handling

The connector uses `ExchangeResult<T>` which returns:
- `ExchangeError::Network` - Network/connectivity issues
- `ExchangeError::Api` - API errors (with code and message)
- `ExchangeError::Auth` - Authentication failures
- `ExchangeError::RateLimitExceeded` - Rate limit hit
- `ExchangeError::Parse` - JSON parsing errors

All errors include descriptive messages for debugging.
