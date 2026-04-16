# Futu OpenAPI Connector

Connector for Futu (Futubull/Moomoo) broker via the OpenD gateway.

## Status

⚠️ **STUB** - Business logic complete; TCP+Protobuf transport not yet wired

All trait methods are implemented with correct Futu protocol logic. They return
`UnsupportedOperation` with a diagnostic message (protocol ID + request JSON)
until `proto_call()` is connected to a real OpenD TCP socket.

## Architecture

Futu OpenAPI does **not** use HTTP REST. The architecture is:

```
Your App
   ↓  TCP + Protocol Buffers (port 11111)
OpenD (local daemon / remote server)
   ↓  Futu proprietary protocol
Futu Servers
```

OpenD is a gateway daemon you must run alongside your application. It handles
authentication with Futu's servers; your code talks only to OpenD.

### Integration Path

To make this connector fully operational:

1. Download and run OpenD: https://www.futunn.com/en/download/OpenAPI
2. Implement a TCP client that sends Futu framed protobuf packets
3. Replace the body of `proto_call()` in `connector.rs` — all business logic above it is complete

Alternatively:
- Use Futu's Python SDK via FFI
- Run a Python adapter that exposes a local REST API

## Quick Start

### 1. Install OpenD

```
https://www.futunn.com/en/download/OpenAPI
→ Download OpenD for your platform
→ Launch OpenD and log in with your Futu account credentials
→ OpenD listens on 127.0.0.1:11111 by default
```

### 2. Set Environment Variables

**Linux/macOS/Git Bash:**
```bash
export FUTU_OPEND_HOST="127.0.0.1"
export FUTU_OPEND_PORT="11111"
export FUTU_TRADE_PASSWORD="your_trade_password"  # optional, for trading
```

**Windows PowerShell:**
```powershell
$env:FUTU_OPEND_HOST="127.0.0.1"
$env:FUTU_OPEND_PORT="11111"
$env:FUTU_TRADE_PASSWORD="your_trade_password"
```

### 3. Test

```bash
cargo test --test futu_integration -- --nocapture
```

## Usage

```rust
use digdigdig3::stocks::china::futu::FutuConnector;
use digdigdig3::core::{Symbol, AccountType};
use digdigdig3::core::traits::MarketData;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connects to OpenD at 127.0.0.1:11111 by default
    let connector = FutuConnector::from_env();

    // Get current price (returns UnsupportedOperation until transport is wired)
    let symbol = Symbol::new("00700", "HKD"); // Tencent on HKEX
    let price = connector.get_price(symbol, AccountType::Spot).await?;
    println!("Tencent: HKD{}", price);

    Ok(())
}
```

## Features

### Market Data
- ✅ Real-time snapshots (price, OHLC, bid/ask)
- ✅ Historical OHLCV (20 years daily, minute bars)
- ✅ Order book (HK LV2 with broker queue)
- ✅ Options chains (Greeks, IV)
- ✅ Futures data (open interest, positions)
- ✅ Capital flow data (HK market)
- ❌ Transport not wired (all return UnsupportedOperation)

### Trading
- ✅ Market and limit orders (business logic complete)
- ✅ Order modification and cancellation
- ✅ Simulate (paper) and real trading environments
- ❌ Transport not wired (all return UnsupportedOperation)

### Account
- ✅ Funds query (cash, securities value, buying power)
- ✅ Position list
- ✅ Order history
- ✅ Trade fills
- ❌ Transport not wired (all return UnsupportedOperation)

### NOT Supported
- ❌ HTTP REST API (Futu does not expose one)
- ❌ Standard WebSocket (uses TCP push subscriptions instead)

## Authentication

**Type:** Two-tier — OpenD handles Futu auth; client connects to OpenD

| Layer | Description |
|-------|-------------|
| OpenD ↔ Futu Servers | OpenD logs in with your Futu account during startup |
| Client ↔ OpenD (local) | No authentication needed |
| Client ↔ OpenD (remote) | RSA public key required (`FUTU_RSA_KEY`) |
| Trade unlock | Trade password sent via `Trd_UnlockTrade` protobuf call |

There are no API keys. Credentials live inside OpenD.

## Supported Markets

| Market | Exchanges | Description |
|--------|-----------|-------------|
| Hong Kong | HKEX | Stocks, ETFs, warrants |
| China A-Shares | SSE, SZSE | via Stock Connect |
| US | NYSE, NASDAQ, AMEX | Stocks, ETFs, options |
| Singapore | SGX | Stocks |
| Australia | ASX | Stocks |

## Protocol IDs (for debugging)

| Proto ID | Name | Category |
|----------|------|----------|
| 1004 | KeepAlive | Connection |
| 2001 | Trd_GetAccList | Account |
| 2004 | Trd_UnlockTrade | Account |
| 2101 | Trd_GetFunds | Account |
| 2102 | Trd_GetPositionList | Positions |
| 2201 | Trd_GetOrderList | Trading |
| 2202 | Trd_PlaceOrder | Trading |
| 2205 | Trd_ModifyOrder | Trading |
| 2211 | Trd_GetOrderFillList | Trading |
| 3005 | Qot_GetSecuritySnapshot | Market Data |
| 3012 | Qot_GetOrderBook | Market Data |
| 3103 | Qot_RequestHistoryKL | Market Data |

## Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `FUTU_OPEND_HOST` | No | `127.0.0.1` | OpenD host |
| `FUTU_OPEND_PORT` | No | `11111` | OpenD TCP port |
| `FUTU_TRADE_PASSWORD` | No | — | Trade unlock password |
| `FUTU_RSA_KEY` | No | — | RSA key for remote OpenD |

## Files

```
futu/
├── README.md           # This file
├── mod.rs              # Module exports
├── auth.rs             # OpenD connection parameters
├── endpoints.rs        # Protocol IDs, market enums, symbol formatting
├── connector.rs        # Trait implementations (proto_call stub)
├── parser.rs           # Protobuf response parsing helpers
├── tests.rs            # Unit tests
└── research/           # OpenD protocol research notes
```

## Testing

```bash
# Unit tests (no OpenD required)
cargo test --lib futu

# Integration tests (requires running OpenD)
cargo test --test futu_integration -- --nocapture

# Ignored tests (require live OpenD connection)
cargo test --test futu_integration -- --nocapture --ignored
```

## Documentation

- **OpenAPI docs:** https://openapi.futunn.com/futu-api-doc/en/
- **OpenD download:** https://www.futunn.com/en/download/OpenAPI
- **Protocol Buffer definitions:** https://github.com/FutunnOpen/py-futu-api/tree/master/futu/common/pb
- **Official SDKs:** Python, Java, C#, C++, JavaScript
