# Jupiter API Endpoints

## Overview

Jupiter is Solana's leading DEX aggregator that consolidates liquidity from multiple automated market makers (AMMs) to provide optimal token swap rates. The API provides quote, swap, price, and token information endpoints.

## Base URLs

### V6 Swap API (Current)
```
https://quote-api.jup.ag/v6
```

### Metis Swap API (Higher Performance)
```
https://api.jup.ag/swap/v1
```
*Requires API key via `x-api-key` header*

### Price API V3
```
https://api.jup.ag/price/v3
```

### Tokens API V2
```
https://api.jup.ag/tokens/v2
```

### Deprecated Endpoints
- `lite-api.jup.ag` - Will be deprecated on **January 31, 2026**
- `https://quote-api.jup.ag/v6` - Being phased out
- `https://tokens.jup.ag` - Deprecated
- `https://price.jup.ag` - Deprecated

## Swap API Endpoints

### 1. GET /quote

Request a quote for token swap routing.

**Endpoint:**
```
GET https://quote-api.jup.ag/v6/quote
GET https://api.jup.ag/swap/v1/quote (Metis)
```

**Query Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `inputMint` | string | Yes | - | Input token mint address |
| `outputMint` | string | Yes | - | Output token mint address |
| `amount` | integer (u64) | Yes | - | Raw amount to swap (before decimals) |
| `slippageBps` | integer (u16) | No | 50 | Slippage tolerance in basis points (0.5% = 50) |
| `swapMode` | enum | No | ExactIn | "ExactIn" or "ExactOut" |
| `dexes` | string[] | No | - | Comma-separated DEX names to include |
| `excludeDexes` | string[] | No | - | Comma-separated DEX names to exclude |
| `restrictIntermediateTokens` | boolean | No | true | Restrict intermediate tokens to stables |
| `onlyDirectRoutes` | boolean | No | false | Limit to single-hop routes only |
| `asLegacyTransaction` | boolean | No | false | Use legacy transaction format |
| `platformFeeBps` | integer (u16) | No | - | Platform fee in basis points |
| `maxAccounts` | integer (u64) | No | 64 | Max accounts for quote calculation |
| `instructionVersion` | enum | No | V1 | "V1" or "V2" instruction version |

**Example Request:**
```
GET /quote?inputMint=So11111111111111111111111111111111111111112&outputMint=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v&amount=100000000&slippageBps=50
```

**Response Format:**
```json
{
  "inputMint": "string",
  "inAmount": "string",
  "outputMint": "string",
  "outAmount": "string",
  "otherAmountThreshold": "string",
  "swapMode": "ExactIn|ExactOut",
  "slippageBps": 1,
  "priceImpactPct": "string",
  "routePlan": [
    {
      "swapInfo": {
        "ammKey": "string",
        "inputMint": "string",
        "outputMint": "string",
        "inAmount": "string",
        "outAmount": "string",
        "label": "string",
        "feeAmount": "string",
        "feeMint": "string"
      },
      "percent": 123,
      "bps": 123
    }
  ],
  "platformFee": {
    "amount": "string",
    "feeBps": 123
  },
  "contextSlot": 123,
  "timeTaken": 123
}
```

**Key Response Fields:**
- `outAmount`: Calculated output amount including platform and DEX fees
- `otherAmountThreshold`: Minimum output accounting for slippageBps
- `routePlan`: Array detailing swap routes and liquidity sources

---

### 2. POST /swap

Generate serialized transaction for on-chain execution.

**Endpoint:**
```
POST https://quote-api.jup.ag/v6/swap
POST https://api.jup.ag/swap/v1/swap (Metis)
```

**Request Body Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `userPublicKey` | string | Yes | - | User's wallet public key |
| `quoteResponse` | object | Yes | - | Complete quote object from /quote |
| `payer` | string | No | - | Custom payer for transaction fees and rent |
| `wrapAndUnwrapSol` | boolean | No | true | Auto wrap/unwrap SOL in transactions |
| `useSharedAccounts` | boolean | No | false | Enable shared program accounts for complex routing |
| `feeAccount` | string | No | - | Initialized token account for fee collection |
| `trackingAccount` | string | No | - | Public key to track swap transactions |
| `prioritizationFeeLamports` | object | No | - | Priority fee configuration |
| `asLegacyTransaction` | boolean | No | false | Build legacy transaction format |
| `destinationTokenAccount` | string | No | - | Token account to receive output |
| `nativeDestinationAccount` | string | No | - | Account for native SOL output |
| `dynamicComputeUnitLimit` | boolean | No | false | Simulate to determine compute units |
| `skipUserAccountsRpcCalls` | boolean | No | false | Skip RPC calls checking accounts |
| `computeUnitPriceMicroLamports` | integer | No | - | Exact compute unit price |
| `blockhashSlotsToExpiry` | integer | No | - | Slots before transaction expires |

**Example Request:**
```json
{
  "userPublicKey": "5Z3EqYQo9HiCEs3R84RCDMu2n7anpDMxRhdK8PSWmrRC",
  "quoteResponse": { ... },
  "wrapAndUnwrapSol": true,
  "dynamicComputeUnitLimit": true,
  "prioritizationFeeLamports": {
    "priorityLevelWithMaxLamports": {
      "maxLamports": 1000000,
      "priorityLevel": "veryHigh"
    }
  }
}
```

**Response Format:**
```json
{
  "swapTransaction": "base64_encoded_transaction",
  "lastValidBlockHeight": 123456789,
  "prioritizationFeeLamports": 5000
}
```

**Response Fields:**
- `swapTransaction`: Base64-encoded unsigned transaction
- `lastValidBlockHeight`: Block height when transaction expires
- `prioritizationFeeLamports`: Priority fee in lamports applied

---

### 3. POST /swap-instructions

Retrieve individual instructions for custom transaction composition.

**Endpoint:**
```
POST https://quote-api.jup.ag/v6/swap-instructions
POST https://api.jup.ag/swap/v1/swap-instructions (Metis)
```

**Request Body:**
Same parameters as `/swap` endpoint.

**Response Format:**
```json
{
  "tokenLedgerInstruction": {...},
  "computeBudgetInstructions": [...],
  "setupInstructions": [...],
  "swapInstruction": {...},
  "cleanupInstruction": {...},
  "addressLookupTableAddresses": [...]
}
```

**Response Fields:**
- `tokenLedgerInstruction`: Token ledger instruction (if `useTokenLedger=true`)
- `computeBudgetInstructions`: Compute budget setup instructions
- `setupInstructions`: ATA initialization instructions
- `swapInstruction`: Core swap instruction
- `cleanupInstruction`: SOL unwrapping instruction
- `addressLookupTableAddresses`: ALT addresses for versioned transactions

**Instruction Object Format:**
```json
{
  "programId": "string",
  "accounts": [
    {
      "pubkey": "string",
      "isSigner": false,
      "isWritable": true
    }
  ],
  "data": "base64_encoded_data"
}
```

---

## Price API Endpoints

### GET /price/v3

Get current USD prices for up to 50 tokens.

**Endpoint:**
```
GET https://api.jup.ag/price/v3
```

**Headers:**
```
x-api-key: your-api-key
```

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `ids` | string | Yes | Comma-separated mint addresses (max 50) |

**Example Request:**
```
GET /price/v3?ids=So11111111111111111111111111111111111111112,JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN
```

**Response Format:**
```json
{
  "So11111111111111111111111111111111111111112": {
    "usdPrice": 147.4789340738336,
    "blockId": 348004023,
    "decimals": 9,
    "priceChange24h": 1.2907622140620008
  },
  "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN": {
    "usdPrice": 0.4056018512541055,
    "blockId": 348004026,
    "decimals": 6,
    "priceChange24h": 0.5292887924920519
  }
}
```

**Response Fields:**
- `usdPrice`: Current price in USD
- `blockId`: Block identifier for price recency verification
- `decimals`: Token decimal places
- `priceChange24h`: 24-hour percentage change

**Note:** Tokens without recent trades (within 7 days) or failing quality checks return `null`.

---

## Tokens API Endpoints

### 1. GET /search

Search tokens by symbol, name, or mint address.

**Endpoint:**
```
GET https://api.jup.ag/tokens/v2/search
```

**Headers:**
```
x-api-key: your-api-key
```

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `query` | string | Yes | Symbol, name, or mint address (comma-separated, max 100) |

**Example Request:**
```
GET /tokens/v2/search?query=So11111111111111111111111111111111111111112
GET /tokens/v2/search?query=SOL
```

**Behavior:**
- Returns default 20 results when searching by symbol/name
- Returns exact matches when searching by mint address

---

### 2. GET /tag

Query tokens by classification tags.

**Endpoint:**
```
GET https://api.jup.ag/tokens/v2/tag
```

**Headers:**
```
x-api-key: your-api-key
```

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `query` | string | Yes | Tag name: "lst" or "verified" |

**Example Request:**
```
GET /tokens/v2/tag?query=verified
GET /tokens/v2/tag?query=lst
```

**Supported Tags:**
- `verified`: Verified tokens
- `lst`: Liquid-staked tokens

---

### 3. GET /{category}/{interval}

Get top tokens by category and time interval.

**Endpoint:**
```
GET https://api.jup.ag/tokens/v2/{category}/{interval}
```

**Headers:**
```
x-api-key: your-api-key
```

**Path Parameters:**

| Parameter | Values | Description |
|-----------|--------|-------------|
| `category` | toporganicscore, toptraded, toptrending | Category type |
| `interval` | 5m, 1h, 6h, 24h | Time interval |

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `limit` | integer | 50 | Number of results (max 100) |

**Example Request:**
```
GET /tokens/v2/toporganicscore/5m?limit=100
GET /tokens/v2/toptraded/24h?limit=50
```

**Note:** Filters out generic top tokens like SOL and USDC.

---

### 4. GET /recent

Get recently created tokens by first pool creation time.

**Endpoint:**
```
GET https://api.jup.ag/tokens/v2/recent
```

**Headers:**
```
x-api-key: your-api-key
```

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `limit` | integer | 30 | Number of results |

**Example Request:**
```
GET /tokens/v2/recent?limit=50
```

---

## Token Response Format

All token endpoints return comprehensive metadata:

```json
{
  "id": "string",
  "name": "string",
  "symbol": "string",
  "icon": "string",
  "decimals": 9,
  "circSupply": 1000000000,
  "totalSupply": 1000000000,
  "tokenProgram": "string",
  "firstPool": {
    "timestamp": 1234567890
  },
  "holderCount": 5000,
  "audit": {
    "authority": "enabled|disabled",
    "holderConcentration": 0.15
  },
  "organicScore": 95.5,
  "organicScoreLabel": "high",
  "isVerified": true,
  "cexes": ["binance", "coinbase"],
  "tags": ["verified", "lst"],
  "fdv": 100000000,
  "mcap": 80000000,
  "usdPrice": 1.23,
  "liquidity": 5000000,
  "stats5m": {
    "volume": 50000,
    "priceChange": 0.5,
    "traders": 100
  },
  "stats1h": { ... },
  "stats6h": { ... },
  "stats24h": { ... }
}
```

**Key Fields:**
- `id`: Token mint address
- `organicScore`: Quality score (0-100)
- `isVerified`: Jupiter verification status
- `audit.authority`: Token authority status
- `audit.holderConcentration`: Top holder percentage
- `stats{interval}`: Trading metrics by time window

---

## Symbol Formatting

### Token Mint Addresses

Jupiter uses Solana SPL token mint addresses. Common examples:

| Symbol | Mint Address |
|--------|-------------|
| SOL (Wrapped) | `So11111111111111111111111111111111111111112` |
| USDC | `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v` |
| JUP | `JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN` |

**Format:** Base58 encoded string (32-44 characters)

### Amount Formatting

Amounts are specified in the token's smallest unit (raw amount before decimals):

```
For SOL (9 decimals):
  1 SOL = 1000000000 (1e9)
  0.1 SOL = 100000000

For USDC (6 decimals):
  1 USDC = 1000000 (1e6)
  0.1 USDC = 100000
```

**Formula:** `raw_amount = human_amount * 10^decimals`

---

## Transaction Types

### Versioned Transactions

Default format using address lookup tables (ALTs) for efficiency:
- Requires wallet support for versioned transactions
- More compact, lower fees
- Most wallets support as of 2024

### Legacy Transactions

Fallback format for older wallets:
- Set `asLegacyTransaction: true`
- Larger transaction size
- Higher fees
- Universal wallet support

---

## Notes

1. **Deprecation Timeline**: Migrate from `lite-api.jup.ag` before January 31, 2026
2. **API Key**: Required for Metis endpoints and Price/Tokens APIs
3. **Token List V3**: Jupiter Verify system replaced old token lists
4. **Price Methodology**: Uses last swapped price across all transactions
5. **Self-Hosting**: V6 API can be self-hosted using `jupiter-quote-api-node`
