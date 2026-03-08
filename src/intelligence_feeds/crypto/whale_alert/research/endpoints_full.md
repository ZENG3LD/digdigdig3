# Whale Alert - Complete Endpoint Reference

## Category: Enterprise REST API (v2)

**Base URL:** https://leviathan.whale-alert.io

**Rate Limit:** 1,000 calls per minute

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /status | Returns supported blockchains and currencies | No | Yes | 1000/min | Full overview of available data |
| GET | /{blockchain}/status | Get block height range for specific blockchain | No | Yes | 1000/min | Returns newest and oldest available blocks |
| GET | /{blockchain}/transaction/{hash} | Retrieve single transaction by hash | No | Yes | 1000/min | 30-day history limit |
| GET | /{blockchain}/transactions | Stream transactions from start height | No | Yes | 1000/min | Supports filtering and pagination |
| GET | /{blockchain}/block/{height} | Get complete block data with all transactions | No | Yes | 1000/min | Full block information |
| GET | /{blockchain}/address/{hash}/transactions | Get transaction history for an address | No | Yes | 1000/min | 30-day history limit |
| GET | /{blockchain}/address/{hash}/owner_attributions | Get owner attribution with confidence scores | No | Yes | 1000/min | Confidence scores on 0-1 scale |

### Parameters Reference

#### GET /status
**Parameters:** None (requires api_key only)

**Returns:** List of supported blockchains with available currency symbols

**Example Response:**
```json
{
  "blockchains": {
    "bitcoin": ["BTC", "USDT", "EURT"],
    "ethereum": ["ETH", "USDT", "USDC", "WBTC", ...],
    "tron": ["USDD", "TRX", "BTT", "USDT", "USDC", "TUSD", "USDJ", "WBTC"],
    "dogecoin": ["DOGE"],
    ...
  }
}
```

#### GET /{blockchain}/status
**Path Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| blockchain | string | Yes | Blockchain name (bitcoin, ethereum, tron, etc.) |

**Query Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| api_key | string | Yes | - | Your API key |

**Returns:** Block height range (newest_block, oldest_block)

#### GET /{blockchain}/transaction/{hash}
**Path Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| blockchain | string | Yes | Blockchain name |
| hash | string | Yes | Transaction hash/identifier |

**Query Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| api_key | string | Yes | - | Your API key |

**Returns:** Complete transaction object with all sub-transactions

**History Limit:** 30 days

#### GET /{blockchain}/transactions
**Path Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| blockchain | string | Yes | Blockchain name |

**Query Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| api_key | string | Yes | - | Your API key |
| start_height | int | Yes | - | Starting block height |
| symbol | string | No | All | Filter by currency symbol |
| transaction_type | string | No | All | Filter by type (transfer, mint, burn, freeze, unfreeze, lock, unlock) |
| limit | int | No | 256 | Number of transactions to return |
| order | string | No | asc | Sort order (asc/desc) |
| format | string | No | json | Response format |

**Returns:** Array of transaction objects

#### GET /{blockchain}/block/{height}
**Path Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| blockchain | string | Yes | Blockchain name |
| height | int | Yes | Block height/number |

**Query Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| api_key | string | Yes | - | Your API key |

**Returns:** Complete block data with all transactions

#### GET /{blockchain}/address/{hash}/transactions
**Path Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| blockchain | string | Yes | Blockchain name |
| hash | string | Yes | Address hash |

**Query Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| api_key | string | Yes | - | Your API key |

**Returns:** Array of transactions involving this address

**History Limit:** 30 days

#### GET /{blockchain}/address/{hash}/owner_attributions
**Path Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| blockchain | string | Yes | Blockchain name |
| hash | string | Yes | Address hash |

**Query Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| api_key | string | Yes | - | Your API key |

**Returns:** Owner attribution data with confidence scores (0-1 scale)

---

## Category: Developer API v1 (Deprecated)

**Base URL:** https://api.whale-alert.io/v1

**Note:** This API is deprecated but still functional. Free tier: 10 req/min, Personal tier: 60 req/min

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /status | Returns supported blockchains and currencies | Yes | No | 10/min | Free tier available |
| GET | /transaction/{blockchain}/{hash} | Retrieve single transaction | Yes | Yes | 10/min (free) | Personal tier: 60/min |
| GET | /transactions | Get multiple transactions with filters | Yes | Yes | 10/min (free) | Supports cursor pagination |

### Parameters Reference (v1 API)

#### GET /v1/status
**Query Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| api_key | string | Yes | - | Your API key |

**Returns:** List of supported blockchains with connection status

#### GET /v1/transaction/{blockchain}/{hash}
**Path Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| blockchain | string | Yes | Blockchain name |
| hash | string | Yes | Transaction hash |

**Query Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| api_key | string | Yes | - | Your API key |

#### GET /v1/transactions
**Query Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| api_key | string | Yes | - | Your API key |
| min_value | int | No | None | Minimum transaction value in USD |
| start | int | Yes | - | Unix timestamp for start time |
| end | int | No | Current | Unix timestamp for end time |
| cursor | string | No | None | Pagination cursor from previous response |
| limit | int | No | 100 | Number of results (max varies by tier) |

**Returns:** Paginated array of transactions

---

## Category: WebSocket Control

**Note:** WebSocket connections are managed via the ws:// protocol. No REST endpoints for WebSocket management.

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| N/A | WebSocket only | No REST endpoints for WS control | - | - | - | See websocket_full.md |

---

## Supported Blockchains

Current supported blockchains (as of 2026):
- Bitcoin (bitcoin)
- Ethereum (ethereum)
- Algorand (algorand)
- Bitcoin Cash (bitcoincash)
- Dogecoin (dogecoin)
- Litecoin (litecoin)
- Polygon (polygon)
- Solana (solana)
- Ripple (ripple)
- Cardano (cardano)
- Tron (tron)

Additional blockchains can be requested through support.

---

## Transaction Types

All endpoints support filtering by transaction type:

| Type | Description |
|------|-------------|
| transfer | Standard value transfer between addresses |
| mint | New tokens/coins created |
| burn | Tokens/coins destroyed |
| freeze | Assets frozen (cannot be transferred) |
| unfreeze | Assets unfrozen (can be transferred again) |
| lock | Assets locked (time-locked or contract-locked) |
| unlock | Assets unlocked |

---

## Common Query Parameters

All endpoints require authentication via query parameter:

```
?api_key=YOUR_API_KEY
```

or appended to existing parameters:

```
&api_key=YOUR_API_KEY
```

---

## Notes

- Enterprise API provides 30 days of historical data access
- Developer API v1 is deprecated but still functional for existing users
- All responses are in JSON format
- Transactions with values under $10 USD may be grouped together
- All prices are in USD at the time of the transaction
- Address attribution includes over 400 known entities (exchanges, institutions, etc.)
