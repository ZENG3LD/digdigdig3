# Vertex Protocol API Endpoints

Vertex Protocol provides a hybrid orderbook-AMM DEX built on Arbitrum with WebSocket/REST gateway API for executes, queries, and subscriptions.

## Base URLs

### Production (Mainnet)
- **Gateway REST**: `https://gateway.prod.vertexprotocol.com/v1`
- **Gateway WebSocket**: `wss://gateway.prod.vertexprotocol.com/v1/ws`
- **Gateway Subscribe**: `wss://gateway.prod.vertexprotocol.com/v1/subscribe`
- **Archive Indexer**: `https://archive.prod.vertexprotocol.com/v1`

### Testnet (Sepolia)
- **Gateway REST**: `https://gateway.sepolia-test.vertexprotocol.com/v1`
- **Gateway WebSocket**: `wss://gateway.sepolia-test.vertexprotocol.com/v1/ws`
- **Archive Indexer**: `https://archive.sepolia-test.vertexprotocol.com/v1`

### Supported Networks
- Arbitrum One (Chain ID: 42161)
- Arbitrum Sepolia (Chain ID: 421613)
- Blast, Mantle, Sei, Base mainnets

## Endpoint Structure

### Execute Endpoints (Trading Operations)
**Base Path**: `POST /execute`

All execute operations require EIP-712 signed transactions.

### Query Endpoints (Market Data)
**Base Paths**:
- `GET /query` (URL-encoded parameters)
- `POST /query` (JSON payload)

### Indexer Endpoints (Historical Data)
**Base Path**: `POST [ARCHIVE_ENDPOINT]` (JSON payload)

### Symbols Endpoint
**Path**: `GET /symbols`

---

## MarketData Trait Endpoints

### 1. Get All Products
**Query Type**: `all_products`
**Method**: `GET/POST /query`
**Parameters**:
- `type`: "all_products"

**Response Fields**:
- `spot_products[]`: Array of spot products
  - `product_id`: uint32
  - `oracle_price_x18`: int128 (18 decimal precision)
  - `risk`: Risk configuration object
  - `config`: Product configuration object
  - `state`: Product state object
  - `lp_state`: Liquidity provider state
  - `book_info`: Orderbook information

- `perp_products[]`: Array of perpetual products (same structure)

**Rate Limit**: 12 requests/second (weight: 5)

### 2. Get Product Symbols
**Path**: `GET /symbols`
**Method**: GET
**Parameters**: None

**Response**:
```json
{
  "status": "success",
  "symbols": {
    "0": "USDC",
    "2": "BTC-PERP",
    "4": "ETH-PERP",
    "6": "ARB-PERP"
  }
}
```

**Rate Limit**: 60 requests/second

### 3. Get Market Liquidity (Orderbook Depth)
**Query Type**: `market_liquidity`
**Method**: `GET/POST /query`
**Parameters**:
- `type`: "market_liquidity"
- `product_id`: uint32

**Response**:
- `product_id`: uint32
- `bids[]`: Array of bid levels
  - `price_x18`: int128
  - `size`: int128
- `asks[]`: Array of ask levels
  - `price_x18`: int128
  - `size`: int128

**Rate Limit**: 60 requests/second (40 req/sec per IP)

### 4. Get Latest Market Price
**Query Type**: `market_price`
**Method**: `GET/POST /query`
**Parameters**:
- `type`: "market_price"
- `product_id`: uint32

**Response**:
- `product_id`: uint32
- `bid_x18`: int128 (best bid price)
- `ask_x18`: int128 (best ask price)
- `last_updated`: uint64 (timestamp)

**Rate Limit**: 60 requests/second

### 5. Get Candlesticks (OHLCV)
**Endpoint**: Archive Indexer
**Method**: `POST [ARCHIVE_ENDPOINT]`
**Payload**:
```json
{
  "candlesticks": {
    "product_id": 2,
    "granularity": 3600,
    "limit": 100,
    "max_time": 1234567890
  }
}
```

**Parameters**:
- `product_id`: uint32
- `granularity`: Candle period in seconds (60, 300, 900, 3600, 14400, 86400)
- `limit`: Max number of candles (default 100)
- `max_time`: Latest timestamp (optional)

**Response**:
```json
{
  "candlesticks": [
    {
      "product_id": 2,
      "open_x18": "30000000000000000000000",
      "high_x18": "31000000000000000000000",
      "low_x18": "29500000000000000000000",
      "close_x18": "30500000000000000000000",
      "volume": "1500000000000000000000",
      "timestamp": 1234567890
    }
  ]
}
```

**Rate Limit**: Archive endpoint limits apply

### 6. Get Product Snapshots (24hr Stats)
**Endpoint**: Archive Indexer
**Method**: `POST [ARCHIVE_ENDPOINT]`
**Payload**:
```json
{
  "product_snapshots": {
    "product_id": 2
  }
}
```

**Response**:
- `snapshots[]`: Historical product state snapshots
- `txs[]`: Related transaction data

**Rate Limit**: Archive endpoint limits apply

### 7. Get Contracts Info
**Query Type**: `contracts`
**Method**: `GET/POST /query`
**Parameters**:
- `type`: "contracts"

**Response**:
- `chain_id`: uint64
- `endpoint_addr`: Address of endpoint contract
- `book_addrs[]`: Array of orderbook contract addresses
- Other deployed contract addresses

**Rate Limit**: 60 requests/second (weight: 1)

### 8. Get Status
**Query Type**: `status`
**Method**: `GET/POST /query`
**Parameters**:
- `type`: "status"

**Response**:
- `status`: "online" | "offline"
- `server_time`: uint64 (current server timestamp)

**Rate Limit**: 60 requests/second

---

## Trading Trait Endpoints

### 1. Place Order
**Execute Type**: `place_order`
**Method**: `POST /execute`
**Payload**:
```json
{
  "place_order": {
    "product_id": 2,
    "order": {
      "sender": "0x7a5ec...000000000000000000000000",
      "priceX18": "30000000000000000000000",
      "amount": "1000000000000000000",
      "expiration": 4611686018427387904,
      "nonce": 1234567890123
    },
    "signature": "0x..."
  }
}
```

**Order Fields**:
- `sender`: bytes32 (address + subaccount identifier)
- `priceX18`: int128 (price with 18 decimal precision)
- `amount`: int128 (positive for buy, negative for sell)
- `expiration`: uint64 (timestamp + time-in-force bits)
- `nonce`: uint64 (unique order identifier)
- `signature`: EIP-712 signature

**Time-in-Force Encoding** (in expiration field):
- GTC (Good-Till-Cancel): bits 62-63 = 0
- IOC (Immediate-or-Cancel): bit 62 = 1
- FOK (Fill-or-Kill): bit 63 = 1
- POST_ONLY: bits 62-63 = 1

**Response**:
```json
{
  "status": "success",
  "data": {
    "digest": "0x123abc..."
  }
}
```

**Rate Limits**:
- 10 requests/second (leveraged positions)
- 5 requests/10 seconds (spot with no leverage)

### 2. Place Market Order
Use `place_order` with specific parameters:
- Set `priceX18` to extreme value (e.g., max int128 for buy, min int128 for sell)
- Set time-in-force to IOC or FOK

### 3. Cancel Orders
**Execute Type**: `cancel_orders`
**Method**: `POST /execute`
**Payload**:
```json
{
  "cancel_orders": {
    "sender": "0x7a5ec...000000000000000000000000",
    "productIds": [2, 4],
    "digests": ["0x123abc...", "0x456def..."],
    "nonce": 1234567890123,
    "signature": "0x..."
  }
}
```

**Fields**:
- `sender`: bytes32 (subaccount identifier)
- `productIds`: uint32[] (product IDs of orders)
- `digests`: bytes32[] (order digests to cancel)
- `nonce`: uint64
- `signature`: EIP-712 signature

**Response**:
```json
{
  "status": "success"
}
```

**Rate Limit**: 600 requests/second

### 4. Cancel Product Orders (Cancel All for Product)
**Execute Type**: `cancel_product_orders`
**Method**: `POST /execute`
**Payload**:
```json
{
  "cancel_product_orders": {
    "sender": "0x7a5ec...000000000000000000000000",
    "productIds": [2],
    "nonce": 1234567890123,
    "signature": "0x..."
  }
}
```

**Rate Limit**: 2 requests/second

### 5. Cancel and Place (Atomic Replace)
**Execute Type**: `cancel_and_place`
**Method**: `POST /execute`
**Payload**:
```json
{
  "cancel_and_place": {
    "cancel": {
      "sender": "0x...",
      "productIds": [2],
      "digests": ["0x..."],
      "nonce": 123,
      "signature": "0x..."
    },
    "place": {
      "product_id": 2,
      "order": { /* Order object */ },
      "signature": "0x..."
    }
  }
}
```

**Important**: Both cancel and place signatures must be signed by the same wallet.

**Rate Limit**: Same as place_order

---

## Account Trait Endpoints

### 1. Get Subaccount Info (Balances)
**Query Type**: `subaccount_info`
**Method**: `GET/POST /query`
**Parameters**:
- `type`: "subaccount_info"
- `sender`: bytes32 (subaccount identifier)
- `txs`: Optional transaction data for estimated view

**Response**:
```json
{
  "status": "success",
  "data": {
    "subaccount": "0x7a5ec...",
    "exists": true,
    "healths": [
      {
        "assets": "75323297691833342306",
        "liabilities": "46329556869051092241",
        "health": "28993740822782250065"
      }
    ],
    "health_contributions": [
      ["75323297691833340000", "75323297691833340000", "75323297691833340000"]
    ],
    "spot_count": 3,
    "perp_count": 2,
    "spot_balances": [
      {
        "product_id": 0,
        "balance": {
          "amount": "100000000000000000000",
          "last_cumulative_multiplier_x18": "1003419811982007193"
        },
        "lp_balance": {
          "amount": "0"
        }
      }
    ],
    "perp_balances": [
      {
        "product_id": 2,
        "balance": {
          "amount": "5000000000000000000",
          "v_quote_balance": "150000000000000000000",
          "last_cumulative_funding_x18": "1000000000000000000"
        },
        "lp_balance": {
          "amount": "0"
        }
      }
    ]
  }
}
```

**Health Indices**:
- `healths[0]`: Initial health (long_weight_initial_x18, short_weight_initial_x18)
- `healths[1]`: Maintenance health (long_weight_maintenance_x18, short_weight_maintenance_x18)

**Rate Limit**: 40 requests/10 seconds per IP (weight: 10)

### 2. Get Fee Rates
**Query Type**: `fee_rates`
**Method**: `GET/POST /query`
**Parameters**:
- `type`: "fee_rates"
- `sender`: bytes32 (subaccount identifier)

**Response**:
```json
{
  "status": "success",
  "data": {
    "subaccount": "0x7a5ec...",
    "taker_rate_x18": "500000000000000",
    "maker_rate_x18": "-100000000000000"
  }
}
```

**Rate Limit**: 30 requests/second (weight: 2)

### 3. Get Max Withdrawable
**Query Type**: `max_withdrawable`
**Method**: `GET/POST /query`
**Parameters**:
- `type`: "max_withdrawable"
- `sender`: bytes32
- `product_id`: uint32
- `spot_leverage`: bool (optional)

**Response**:
```json
{
  "status": "success",
  "data": {
    "max_withdrawable_x18": "50000000000000000000"
  }
}
```

**Rate Limit**: 120 requests/10 seconds (weight: 5)

### 4. Deposit Collateral
**Execute Type**: `deposit_collateral`
**Method**: Smart contract interaction (on-chain)

Deposits are done via blockchain transactions, not REST API. Use Web3 library to interact with Vertex contracts.

### 5. Withdraw Collateral
**Execute Type**: `withdraw_collateral`
**Method**: `POST /execute`
**Payload**:
```json
{
  "withdraw_collateral": {
    "sender": "0x7a5ec...",
    "productId": 0,
    "amount": "10000000000000000000",
    "nonce": 1234567890123,
    "signature": "0x..."
  }
}
```

**Fields**:
- `sender`: bytes32
- `productId`: uint32 (collateral product)
- `amount`: uint128
- `nonce`: uint64
- `signature`: EIP-712 signature

**Rate Limit**: Standard execute limit (600 req/10s aggregate)

---

## Positions Trait Endpoints

### 1. Get Open Orders
**Query Type**: `subaccount_orders`
**Method**: `GET/POST /query`
**Parameters**:
- `type`: "subaccount_orders"
- `sender`: bytes32
- `product_id`: uint32 (optional, filter by product)

**Response**:
```json
{
  "status": "success",
  "data": {
    "orders": [
      {
        "product_id": 2,
        "sender": "0x7a5ec...",
        "priceX18": "30000000000000000000000",
        "amount": "1000000000000000000",
        "expiration": 4611686018427387904,
        "nonce": 1234567890123,
        "unfilled_amount": "500000000000000000",
        "digest": "0x123abc...",
        "placed_at": 1234567890
      }
    ]
  }
}
```

**Rate Limit**: 30 requests/second (weight: 2)

### 2. Get Single Order
**Query Type**: `order`
**Method**: `GET/POST /query`
**Parameters**:
- `type`: "order"
- `product_id`: uint32
- `digest`: bytes32 (order digest/hash)

**Response**:
```json
{
  "status": "success",
  "data": {
    "order": {
      "product_id": 2,
      "sender": "0x7a5ec...",
      "priceX18": "30000000000000000000000",
      "amount": "1000000000000000000",
      "expiration": 4611686018427387904,
      "nonce": 1234567890123,
      "unfilled_amount": "500000000000000000",
      "digest": "0x123abc...",
      "placed_at": 1234567890
    }
  }
}
```

**Rate Limit**: 60 requests/second

### 3. Get Max Order Size
**Query Type**: `max_order_size`
**Method**: `GET/POST /query`
**Parameters**:
- `type`: "max_order_size"
- `sender`: bytes32
- `product_id`: uint32
- `price_x18`: int128
- `direction`: 1 (long) or -1 (short)

**Response**:
```json
{
  "status": "success",
  "data": {
    "max_order_size_x18": "10000000000000000000"
  }
}
```

**Rate Limit**: 40 requests/second

### 4. Get Perpetual Positions
Positions are included in the `subaccount_info` response under `perp_balances`:

```json
{
  "perp_balances": [
    {
      "product_id": 2,
      "balance": {
        "amount": "5000000000000000000",
        "v_quote_balance": "150000000000000000000",
        "last_cumulative_funding_x18": "1000000000000000000"
      },
      "lp_balance": {
        "amount": "0"
      }
    }
  ]
}
```

**Fields**:
- `amount`: Position size (positive = long, negative = short)
- `v_quote_balance`: Virtual quote balance
- `last_cumulative_funding_x18`: Last funding payment checkpoint

### 5. Get Funding Rate
**Endpoint**: Archive Indexer
**Method**: `POST [ARCHIVE_ENDPOINT]`
**Payload**:
```json
{
  "funding_rate": {
    "product_id": 2
  }
}
```

**Response**:
- Current funding rate for perpetual product
- Historical funding rate data

### 6. Liquidate Subaccount
**Execute Type**: `liquidate_subaccount`
**Method**: `POST /execute`

Used to liquidate underwater positions. Typically executed by liquidation bots.

---

## Additional Endpoints

### Health Groups
**Query Type**: `health_groups`
**Method**: `GET/POST /query`

Returns all available health groups (risk groups for cross-margin).

### Link Signer
**Execute Type**: `link_signer`
**Method**: `POST /execute`

Authorize additional signing keys for subaccount.

### Mint/Burn LP Tokens
**Execute Types**: `mint_lp`, `burn_lp`
**Method**: `POST /execute`

Add or remove liquidity provider positions.

---

## Rate Limit Summary

| Endpoint | Limit | Window | Weight |
|----------|-------|--------|--------|
| **Aggregate (all endpoints)** | 600 | 10s | - |
| **Indexer** | 60 | 1s | - |
| **Status** | 60 | 1s | 1 |
| **Order** | 60 | 1s | 1 |
| **Subaccount Info** | 40 | 10s | 10 |
| **Market Liquidity** | 40 | 1s | 1 |
| **All Products** | 12 | 1s | 5 |
| **Market Price** | 60 | 1s | 1 |
| **Fee Rates** | 30 | 1s | 2 |
| **Contracts** | 60 | 1s | 1 |
| **Subaccount Orders** | 30 | 1s | 2 |
| **Max Withdrawable** | 12 | 1s | 5 |
| **Place Order (leveraged)** | 10 | 1s | - |
| **Place Order (no leverage)** | 5 | 10s | - |
| **Cancel Orders** | 600 | 1s | - |
| **Cancel All** | 2 | 1s | - |

---

## Data Format Notes

### X18 Precision
All prices and amounts use 18 decimal precision (X18 format):
- 1 USDC = `1000000000000000000` (1e18)
- $20,000 BTC = `20000000000000000000000` (20000e18)

### Sender Format
Subaccount identifier is a bytes32 combining:
- First 20 bytes: Ethereum address
- Last 12 bytes: Subaccount name (e.g., "default" = `64656661756c740000000000`)

Example: `0x7a5ec2748e9065794491a8d29dcf3f9edb8d7c43000000000000000000000000`

### Order Digest
Order hash calculated as: `keccak256(EIP712_order_struct)`

---

## Error Responses

```json
{
  "status": "error",
  "error": {
    "code": "INVALID_SIGNATURE",
    "message": "Signature verification failed"
  }
}
```

Common error codes:
- `INVALID_SIGNATURE`
- `INSUFFICIENT_BALANCE`
- `RATE_LIMIT_EXCEEDED`
- `INVALID_PRODUCT_ID`
- `ORDER_NOT_FOUND`
