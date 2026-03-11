# Whale Alert - Response Formats

All examples are from official documentation and represent actual API responses.

---

## REST API Responses

### GET /status (Enterprise API v2)

Returns supported blockchains and currencies.

```json
{
  "blockchains": {
    "bitcoin": ["BTC", "USDT", "EURT"],
    "ethereum": ["ETH", "USDT", "USDC", "WBTC", "DAI", "LINK", "UNI", "AAVE", "SHIB", "MATIC"],
    "tron": ["USDD", "TRX", "BTT", "USDT", "USDC", "TUSD", "USDJ", "WBTC"],
    "dogecoin": ["DOGE"],
    "algorand": ["ALGO"],
    "bitcoincash": ["BCH"],
    "litecoin": ["LTC"],
    "polygon": ["MATIC", "USDT", "USDC"],
    "solana": ["SOL", "USDT", "USDC"],
    "ripple": ["XRP"],
    "cardano": ["ADA"]
  },
  "status": {
    "bitcoin": "connected",
    "ethereum": "connected",
    "tron": "connected",
    "dogecoin": "connected",
    "algorand": "connected",
    "bitcoincash": "connected",
    "litecoin": "connected",
    "polygon": "connected",
    "solana": "connected",
    "ripple": "connected",
    "cardano": "connected"
  }
}
```

**Fields:**
- `blockchains` (object): Map of blockchain name to array of supported currency symbols
- `status` (object): Map of blockchain name to connection status ("connected" or other)

---

### GET /{blockchain}/status

Returns block height range for a specific blockchain.

**Request:**
```
GET https://leviathan.whale-alert.io/ethereum/status?api_key=YOUR_KEY
```

**Response:**
```json
{
  "blockchain": "ethereum",
  "newest_block": 18500000,
  "oldest_block": 18000000,
  "status": "connected"
}
```

**Fields:**
- `blockchain` (string): Blockchain name
- `newest_block` (int): Highest block height available
- `oldest_block` (int): Oldest block height available (30-day retention)
- `status` (string): Connection status

---

### GET /{blockchain}/transaction/{hash}

Returns complete transaction details.

**Request:**
```
GET https://leviathan.whale-alert.io/ethereum/transaction/0x1234567890abcdef?api_key=YOUR_KEY
```

**Response:**
```json
{
  "height": 17887234,
  "index_in_block": 145,
  "timestamp": 1692724660,
  "hash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
  "fee": "0.003456789",
  "fee_symbol": "ETH",
  "fee_symbol_price": "1650.50",
  "sub_transactions": [
    {
      "symbol": "USDC",
      "unit_price_usd": "1.00",
      "transaction_type": "transfer",
      "inputs": [
        {
          "amount": "50000000.0",
          "address": "0xabc123def456abc123def456abc123def456abc1",
          "balance": "250000000.0",
          "locked": "0",
          "is_frozen": false,
          "owner": "Binance",
          "owner_type": "exchange",
          "address_type": "hot_wallet"
        }
      ],
      "outputs": [
        {
          "amount": "50000000.0",
          "address": "0xdef456abc123def456abc123def456abc123def4",
          "balance": "50000000.0",
          "locked": "0",
          "is_frozen": false,
          "owner": "",
          "owner_type": "unknown",
          "address_type": "unknown"
        }
      ]
    }
  ]
}
```

**Root Transaction Object Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `height` | int | Block height containing the transaction |
| `index_in_block` | int | Position within the block (0-indexed) |
| `timestamp` | int | Unix timestamp of the block |
| `hash` | string | Transaction hash/identifier |
| `fee` | string | Transaction cost paid (string for precision) |
| `fee_symbol` | string | Currency of the fee (e.g., "ETH", "BTC") |
| `fee_symbol_price` | string | USD price per unit at block time |
| `sub_transactions` | array | Array of sub-transaction objects |

**SubTransaction Object Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `symbol` | string | Currency ticker (e.g., "USDC", "ETH") |
| `unit_price_usd` | string | USD conversion rate at block time |
| `transaction_type` | string | "transfer", "mint", "burn", "freeze", "unfreeze", "lock", "unlock" |
| `inputs` | array | Array of source addresses (FROM) |
| `outputs` | array | Array of destination addresses (TO) |

**Address Object Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `amount` | string | Balance change quantity (string for precision) |
| `address` | string | Wallet identifier hash |
| `balance` | string | Post-transaction balance (string for precision) |
| `locked` | string | Non-transferable balance amount |
| `is_frozen` | bool | Frozen status indicator |
| `owner` | string | Attributed entity name (empty "" if unknown) |
| `owner_type` | string | Entity classification (exchange, unknown, etc.) |
| `address_type` | string | Wallet category (hot_wallet, cold_wallet, etc.) |

---

### GET /{blockchain}/transactions

Stream transactions from a starting block height.

**Request:**
```
GET https://leviathan.whale-alert.io/ethereum/transactions?start_height=17000000&symbol=USDC&limit=2&api_key=YOUR_KEY
```

**Response:**
```json
[
  {
    "height": 17000012,
    "index_in_block": 56,
    "timestamp": 1678901234,
    "hash": "0xabc...",
    "fee": "0.002",
    "fee_symbol": "ETH",
    "fee_symbol_price": "1600.00",
    "sub_transactions": [
      {
        "symbol": "USDC",
        "unit_price_usd": "1.00",
        "transaction_type": "transfer",
        "inputs": [...],
        "outputs": [...]
      }
    ]
  },
  {
    "height": 17000025,
    "index_in_block": 103,
    "timestamp": 1678901456,
    "hash": "0xdef...",
    "fee": "0.0015",
    "fee_symbol": "ETH",
    "fee_symbol_price": "1601.50",
    "sub_transactions": [
      {
        "symbol": "USDC",
        "unit_price_usd": "1.00",
        "transaction_type": "transfer",
        "inputs": [...],
        "outputs": [...]
      }
    ]
  }
]
```

**Response:** Array of transaction objects (same structure as single transaction endpoint)

---

### GET /{blockchain}/block/{height}

Get complete block data with all transactions.

**Request:**
```
GET https://leviathan.whale-alert.io/ethereum/block/17000000?api_key=YOUR_KEY
```

**Response:**
```json
{
  "blockchain": "ethereum",
  "height": 17000000,
  "timestamp": 1678900000,
  "transaction_count": 234,
  "transactions": [
    {
      "height": 17000000,
      "index_in_block": 0,
      "timestamp": 1678900000,
      "hash": "0x...",
      "fee": "0.001",
      "fee_symbol": "ETH",
      "fee_symbol_price": "1600.00",
      "sub_transactions": [...]
    },
    {
      "height": 17000000,
      "index_in_block": 1,
      "timestamp": 1678900000,
      "hash": "0x...",
      "fee": "0.0008",
      "fee_symbol": "ETH",
      "fee_symbol_price": "1600.00",
      "sub_transactions": [...]
    }
    // ... more transactions
  ]
}
```

**Block Object Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `blockchain` | string | Blockchain name |
| `height` | int | Block height/number |
| `timestamp` | int | Unix timestamp of block |
| `transaction_count` | int | Number of transactions in block |
| `transactions` | array | Array of all transactions in block |

---

### GET /{blockchain}/address/{hash}/transactions

Get transaction history for an address (30-day limit).

**Request:**
```
GET https://leviathan.whale-alert.io/ethereum/address/0xabc123.../transactions?api_key=YOUR_KEY
```

**Response:**
```json
[
  {
    "height": 17887234,
    "index_in_block": 145,
    "timestamp": 1692724660,
    "hash": "0x...",
    "fee": "0.003",
    "fee_symbol": "ETH",
    "fee_symbol_price": "1650.50",
    "sub_transactions": [
      {
        "symbol": "USDC",
        "unit_price_usd": "1.00",
        "transaction_type": "transfer",
        "inputs": [
          {
            "amount": "50000000.0",
            "address": "0xabc123...",
            "balance": "200000000.0",
            "locked": "0",
            "is_frozen": false,
            "owner": "Binance",
            "owner_type": "exchange",
            "address_type": "hot_wallet"
          }
        ],
        "outputs": [...]
      }
    ]
  }
  // ... more transactions involving this address
]
```

**Response:** Array of transaction objects where the specified address appears in inputs or outputs

---

### GET /{blockchain}/address/{hash}/owner_attributions

Get owner attribution with confidence scores.

**Request:**
```
GET https://leviathan.whale-alert.io/ethereum/address/0xabc123.../owner_attributions?api_key=YOUR_KEY
```

**Response:**
```json
{
  "address": "0xabc123def456abc123def456abc123def456abc1",
  "blockchain": "ethereum",
  "attributions": [
    {
      "owner": "Binance",
      "owner_type": "exchange",
      "address_type": "hot_wallet",
      "confidence": 0.95
    },
    {
      "owner": "Binance Exchange",
      "owner_type": "exchange",
      "address_type": "deposit_wallet",
      "confidence": 0.82
    }
  ]
}
```

**Attribution Response Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `address` | string | The queried address hash |
| `blockchain` | string | Blockchain name |
| `attributions` | array | Array of attribution objects |

**Attribution Object Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `owner` | string | Entity name (e.g., "Binance", "Coinbase") |
| `owner_type` | string | Entity classification |
| `address_type` | string | Wallet category |
| `confidence` | float | Confidence score (0.0 to 1.0 scale) |

---

## Developer API v1 Responses (Deprecated)

### GET /v1/status

**Request:**
```
GET https://api.whale-alert.io/v1/status?api_key=YOUR_KEY
```

**Response:**
```json
{
  "blockchains": {
    "bitcoin": {
      "symbols": ["BTC", "USDT"],
      "status": "connected"
    },
    "ethereum": {
      "symbols": ["ETH", "USDT", "USDC", "WBTC"],
      "status": "connected"
    },
    "tron": {
      "symbols": ["TRX", "USDT"],
      "status": "connected"
    }
  }
}
```

---

### GET /v1/transaction/{blockchain}/{hash}

**Request:**
```
GET https://api.whale-alert.io/v1/transaction/bitcoin/abc123def456?api_key=YOUR_KEY
```

**Response:**
```json
{
  "blockchain": "bitcoin",
  "symbol": "btc",
  "id": "abc123def456",
  "transaction_type": "transfer",
  "hash": "abc123def456abc123def456abc123def456abc123def456abc123def456abc1",
  "from": {
    "address": "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
    "owner": "unknown",
    "owner_type": "unknown"
  },
  "to": {
    "address": "1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2",
    "owner": "Binance",
    "owner_type": "exchange"
  },
  "timestamp": 1640000000,
  "amount": 10.5,
  "amount_usd": 500000.00,
  "transaction_count": 1
}
```

**v1 API Transaction Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `blockchain` | string | Blockchain name |
| `symbol` | string | Currency symbol (lowercase) |
| `id` | string | Transaction identifier |
| `transaction_type` | string | "transfer", "mint", "burn", etc. |
| `hash` | string | Transaction hash |
| `from` | object | Sender address object |
| `to` | object | Recipient address object |
| `timestamp` | int | Unix timestamp |
| `amount` | float | Transaction amount |
| `amount_usd` | float | USD value |
| `transaction_count` | int | Number of transactions (1 for single, >1 for grouped) |

**Address Object (v1):**

| Field | Type | Description |
|-------|------|-------------|
| `address` | string | Address hash |
| `owner` | string | Entity name or "unknown" |
| `owner_type` | string | Entity type or "unknown" |

---

### GET /v1/transactions

**Request:**
```
GET https://api.whale-alert.io/v1/transactions?start=1640000000&min_value=1000000&api_key=YOUR_KEY
```

**Response:**
```json
{
  "count": 2,
  "cursor": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "transactions": [
    {
      "blockchain": "ethereum",
      "symbol": "usdt",
      "id": "eth_tx_123",
      "transaction_type": "transfer",
      "hash": "0x123...",
      "from": {
        "address": "0xabc...",
        "owner": "Binance",
        "owner_type": "exchange"
      },
      "to": {
        "address": "0xdef...",
        "owner": "unknown",
        "owner_type": "unknown"
      },
      "timestamp": 1640001000,
      "amount": 50000000.0,
      "amount_usd": 50000000.0,
      "transaction_count": 1
    },
    {
      "blockchain": "bitcoin",
      "symbol": "btc",
      "id": "btc_tx_456",
      "transaction_type": "transfer",
      "hash": "abc123...",
      "from": {
        "address": "1ABC...",
        "owner": "Coinbase",
        "owner_type": "exchange"
      },
      "to": {
        "address": "1DEF...",
        "owner": "unknown",
        "owner_type": "unknown"
      },
      "timestamp": 1640002000,
      "amount": 25.5,
      "amount_usd": 1200000.0,
      "transaction_count": 1
    }
  ]
}
```

**Transactions Response Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `count` | int | Number of transactions returned |
| `cursor` | string | Pagination cursor for next page (use in next request) |
| `transactions` | array | Array of transaction objects |

---

## WebSocket Message Formats

### Subscription Confirmation (Alerts)

**Sent by server after successful subscription:**

```json
{
  "type": "subscribed_alerts",
  "channel_id": "8QFdN74g",
  "blockchains": ["ethereum", "bitcoin"],
  "symbols": ["eth", "weth", "btc"],
  "tx_types": ["transfer"],
  "min_value_usd": 1000000
}
```

**Fields:**
- `type` (string): "subscribed_alerts"
- `channel_id` (string): Assigned channel identifier
- `blockchains` (array): Confirmed blockchain filters
- `symbols` (array): Confirmed symbol filters
- `tx_types` (array): Confirmed transaction type filters
- `min_value_usd` (float): Confirmed minimum value threshold

---

### Subscription Confirmation (Socials)

**Sent by server after successful social subscription:**

```json
{
  "type": "subscribed_socials",
  "channel_id": "xlLZ7tJq"
}
```

**Fields:**
- `type` (string): "subscribed_socials"
- `channel_id` (string): Assigned channel identifier

---

### Alert Message (Transaction Alert)

**Real-time transaction alert:**

```json
{
  "channel_id": "8QFdN74g",
  "timestamp": 1692724660,
  "blockchain": "ethereum",
  "transaction_type": "transfer",
  "from": "Binance",
  "to": "unknown",
  "amounts": [
    {
      "symbol": "USDC",
      "amount": 50000000.0,
      "value_usd": 50000000.0
    }
  ],
  "text": "🔥 50,000,000 #USDC (50,000,000 USD) transferred from Binance to unknown wallet\n\nhttps://whale-alert.io/transaction/ethereum/0x1234567890abcdef",
  "transaction": {
    "height": 17887234,
    "index_in_block": 145,
    "timestamp": 1692724660,
    "hash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
    "fee": "0.003456789",
    "fee_symbol": "ETH",
    "fee_symbol_price": "1650.50",
    "sub_transactions": [
      {
        "symbol": "USDC",
        "unit_price_usd": "1.00",
        "transaction_type": "transfer",
        "inputs": [
          {
            "amount": "50000000.0",
            "address": "0xabc123def456abc123def456abc123def456abc1",
            "balance": "250000000.0",
            "locked": "0",
            "is_frozen": false,
            "owner": "Binance",
            "owner_type": "exchange",
            "address_type": "hot_wallet"
          }
        ],
        "outputs": [
          {
            "amount": "50000000.0",
            "address": "0xdef456abc123def456abc123def456abc123def4",
            "balance": "50000000.0",
            "locked": "0",
            "is_frozen": false,
            "owner": "",
            "owner_type": "unknown",
            "address_type": "unknown"
          }
        ]
      }
    ]
  }
}
```

**Root Alert Message Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `channel_id` | string | Subscription channel identifier |
| `timestamp` | int | Unix timestamp of transaction |
| `blockchain` | string | Blockchain name (ethereum, bitcoin, etc.) |
| `transaction_type` | string | Type of transaction |
| `from` | string | Owner name of sender (or "unknown") |
| `to` | string | Owner name of recipient (or "unknown") |
| `amounts` | array | Array of Amount objects (for multi-currency tx) |
| `text` | string | Human-readable description |
| `transaction` | object | Complete transaction object (same as REST API) |

**Amount Object:**

| Field | Type | Description |
|-------|------|-------------|
| `symbol` | string | Currency symbol |
| `amount` | float | Amount of tokens/coins |
| `value_usd` | float | USD value |

**Transaction object:** Same structure as REST API transaction response (see above)

---

### Social Media Alert

**Whale Alert social post:**

```json
{
  "channel_id": "xlLZ7tJq",
  "timestamp": 1692724660,
  "blockchain": "tron",
  "text": "🔥 🔥 🔥 🔥 🔥 🔥 🔥 🔥 🔥 🔥 1,200,000,000 #USDT (1,200,398,999 USD) burned at Tether Treasury\n\nhttps://whale-alert.io/transaction/tron/cf5b1ae18be3d3596a9920c0dffce82c5247e9672b4ff7b1194d0355e5bec470",
  "urls": [
    "https://twitter.com/whale_alert/status/1694036126422450598",
    "https://t.me/whale_alert_io/72364"
  ]
}
```

**Social Alert Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `channel_id` | string | Subscription channel identifier |
| `timestamp` | int | Unix timestamp when posted |
| `blockchain` | string | Related blockchain |
| `text` | string | Full post text (includes emojis and links) |
| `urls` | array[string] | URLs to original posts (Twitter, Telegram) |

---

## Error Responses

### REST API Error

**HTTP 401 Unauthorized:**
```json
{
  "error_code": 401,
  "error_message": "Invalid API key",
  "details": "The provided API key is not valid or has been revoked"
}
```

**HTTP 429 Rate Limit:**
```json
{
  "error": "Rate limit exceeded",
  "error_code": 429,
  "limit": 1000,
  "remaining": 0,
  "reset": 1692800000,
  "retry_after": 30
}
```

**HTTP 404 Not Found:**
```json
{
  "error_code": 404,
  "error_message": "Transaction not found",
  "details": "No transaction found with the specified hash on this blockchain"
}
```

**HTTP 400 Bad Request:**
```json
{
  "error_code": 400,
  "error_message": "Invalid parameters",
  "details": "start_height must be a positive integer"
}
```

---

### WebSocket Error (Inferred)

**Invalid Subscription:**
```json
{
  "type": "error",
  "error_code": "INVALID_SUBSCRIPTION",
  "message": "min_value_usd must be at least 100000",
  "channel_id": "8QFdN74g"
}
```

**Note:** WebSocket error format not explicitly documented. Errors likely result in connection closure or lack of subscription confirmation.

---

## Data Type Notes

### Precision Handling

**Strings for high-precision numbers:**
- `fee` - String (prevents floating-point precision loss)
- `fee_symbol_price` - String
- `unit_price_usd` - String
- `amount` (in Address object) - String
- `balance` - String
- `locked` - String

**Floats for display values:**
- `amount_usd` (v1 API) - Float
- `value_usd` (WebSocket Amount) - Float
- `amount` (WebSocket Amount) - Float

**Recommendation:** When implementing, parse strings as Decimal/BigDecimal to preserve precision.

### Timestamp Format

**All timestamps are Unix seconds (integer):**
```json
"timestamp": 1692724660
```

**NOT milliseconds** (unlike some APIs)

Convert to datetime:
```javascript
// JavaScript
new Date(timestamp * 1000)
```

```python
# Python
from datetime import datetime
datetime.fromtimestamp(timestamp)
```

```rust
// Rust
use chrono::{DateTime, Utc, NaiveDateTime};
let dt = DateTime::<Utc>::from_utc(
    NaiveDateTime::from_timestamp(timestamp, 0),
    Utc
);
```

### Empty vs Null

**Empty string for unknown:**
```json
"owner": ""
```

**NOT null:**
```json
"owner": null  // Not used
```

**Whale Alert uses empty strings ("") for unknown/missing string values.**

---

## Summary

- **REST API v2 (Enterprise):** Transaction objects with sub_transactions array, precision strings
- **REST API v1 (Deprecated):** Simpler format, from/to objects, float amounts
- **WebSocket Alerts:** Combines high-level summary (amounts, text) with full transaction object
- **WebSocket Socials:** Simple format with text and URLs
- **Precision:** Critical values are strings to prevent precision loss
- **Timestamps:** Unix seconds (integer), not milliseconds
- **Unknown values:** Empty string "", not null
