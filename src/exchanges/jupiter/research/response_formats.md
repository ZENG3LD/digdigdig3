# Jupiter API Response Formats

## Overview

This document details all response formats, data structures, and error codes for Jupiter API endpoints.

---

## HTTP Status Codes

### Success Codes

#### 200 OK
**Description:** Request completed successfully

**Example:**
```json
{
  "inputMint": "So11111111111111111111111111111111111111112",
  "outputMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
  "outAmount": "100000000"
}
```

---

### Client Error Codes

#### 400 Bad Request
**Description:** Problem with request parameters or syntax

**Common Causes:**
- Invalid mint address format
- Missing required parameters
- Invalid parameter values
- Malformed JSON body

**Example Response:**
```json
{
  "error": "Invalid input mint address"
}
```

#### 401 Unauthorized
**Description:** API key issue

**Common Causes:**
- Missing `x-api-key` header
- Invalid API key
- Expired API key

**Example Response:**
```json
{
  "error": "Invalid API key"
}
```

#### 404 Not Found
**Description:** Broken or invalid endpoint

**Common Causes:**
- Incorrect endpoint URL
- Typo in path
- Using deprecated endpoint

**Example Response:**
```json
{
  "error": "Endpoint not found"
}
```

#### 429 Rate Limited
**Description:** Rate limit exceeded

**Common Causes:**
- Too many requests in time window
- Burst limit exceeded
- Need to upgrade tier

**Example Response:**
```json
{
  "error": "Rate limit exceeded. Please slow down requests or upgrade your plan."
}
```

**Headers:**
```http
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1640000000
Retry-After: 10
```

---

### Server Error Codes

#### 500 Internal Server Error
**Description:** Server-side error

**Action:** Contact support via Discord

#### 502 Bad Gateway
**Description:** Gateway error

**Action:** Contact support via Discord

#### 503 Service Unavailable
**Description:** Service temporarily unavailable

**Action:** Contact support via Discord

#### 504 Gateway Timeout
**Description:** Gateway timeout

**Action:** Contact support via Discord

---

## Swap API Responses

### Quote Response

**Endpoint:** `GET /quote`

**Success Response (200):**
```json
{
  "inputMint": "So11111111111111111111111111111111111111112",
  "inAmount": "100000000",
  "outputMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
  "outAmount": "14747893407",
  "otherAmountThreshold": "14673955873",
  "swapMode": "ExactIn",
  "slippageBps": 50,
  "priceImpactPct": "0.0123",
  "routePlan": [
    {
      "swapInfo": {
        "ammKey": "8BnEgHoWFysVcuFFX7QztDmzuH8r5ZFvyP3sYwn1XTh6",
        "inputMint": "So11111111111111111111111111111111111111112",
        "outputMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        "inAmount": "100000000",
        "outAmount": "14747893407",
        "label": "Orca",
        "feeAmount": "25000",
        "feeMint": "So11111111111111111111111111111111111111112"
      },
      "percent": 100,
      "bps": 10000
    }
  ],
  "platformFee": {
    "amount": "147478",
    "feeBps": 10
  },
  "contextSlot": 348004026,
  "timeTaken": 0.123
}
```

**Field Descriptions:**

| Field | Type | Description |
|-------|------|-------------|
| `inputMint` | string | Input token mint address |
| `inAmount` | string | Input amount in smallest unit |
| `outputMint` | string | Output token mint address |
| `outAmount` | string | Calculated output amount (includes fees) |
| `otherAmountThreshold` | string | Minimum output with slippage applied |
| `swapMode` | enum | "ExactIn" or "ExactOut" |
| `slippageBps` | integer | Slippage tolerance in basis points |
| `priceImpactPct` | string | Estimated price impact percentage |
| `routePlan` | array | Array of swap route steps |
| `platformFee` | object | Platform fee details |
| `contextSlot` | integer | Solana slot number when quote was generated |
| `timeTaken` | number | Time taken to calculate route (seconds) |

**Route Plan Structure:**

```typescript
interface RoutePlan {
  swapInfo: {
    ammKey: string;           // DEX pool public key
    inputMint: string;        // Input token for this step
    outputMint: string;       // Output token for this step
    inAmount: string;         // Input amount for this step
    outAmount: string;        // Output amount for this step
    label: string;            // DEX name (e.g., "Orca", "Raydium")
    feeAmount: string;        // Fee amount
    feeMint: string;          // Token used for fee
  };
  percent: number;            // Percentage of total routed through this path
  bps: number;                // Basis points (percent * 100)
}
```

**Platform Fee Structure:**

```typescript
interface PlatformFee {
  amount: string;             // Fee amount in output token
  feeBps: number;             // Fee in basis points
}
```

**Error Response (400):**
```json
{
  "error": "No route found",
  "data": {
    "inputMint": "So11111111111111111111111111111111111111112",
    "outputMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    "reason": "Insufficient liquidity"
  }
}
```

---

### Swap Response

**Endpoint:** `POST /swap`

**Success Response (200):**
```json
{
  "swapTransaction": "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACAAQAHEAqc...",
  "lastValidBlockHeight": 348004500,
  "prioritizationFeeLamports": 5000,
  "computeUnitLimit": 200000,
  "dynamicSlippageReport": {
    "slippageBps": 50,
    "otherAmount": "14673955873",
    "simulatedIncurredSlippageBps": 12,
    "amplificationFactor": 1.0
  }
}
```

**Field Descriptions:**

| Field | Type | Description |
|-------|------|-------------|
| `swapTransaction` | string | Base64-encoded serialized transaction |
| `lastValidBlockHeight` | integer | Block height when transaction expires |
| `prioritizationFeeLamports` | integer | Priority fee applied (lamports) |
| `computeUnitLimit` | integer | Compute units allocated |
| `dynamicSlippageReport` | object | Slippage analysis (if enabled) |

**Dynamic Slippage Report:**

```typescript
interface DynamicSlippageReport {
  slippageBps: number;                    // Applied slippage (bps)
  otherAmount: string;                    // Minimum output with slippage
  simulatedIncurredSlippageBps: number;   // Simulated actual slippage
  amplificationFactor: number;            // Slippage amplification factor
}
```

---

### Swap Instructions Response

**Endpoint:** `POST /swap-instructions`

**Success Response (200):**
```json
{
  "tokenLedgerInstruction": null,
  "computeBudgetInstructions": [
    {
      "programId": "ComputeBudget111111111111111111111111111111",
      "accounts": [],
      "data": "AwAAgD0AAA=="
    }
  ],
  "setupInstructions": [
    {
      "programId": "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",
      "accounts": [
        {
          "pubkey": "5Z3EqYQo9HiCEs3R84RCDMu2n7anpDMxRhdK8PSWmrRC",
          "isSigner": false,
          "isWritable": true
        }
      ],
      "data": "AQ=="
    }
  ],
  "swapInstruction": {
    "programId": "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4",
    "accounts": [
      {
        "pubkey": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "isSigner": false,
        "isWritable": false
      }
    ],
    "data": "BQAAAAAAAAAQJwAAAAAAAA=="
  },
  "cleanupInstruction": {
    "programId": "So11111111111111111111111111111111111111111",
    "accounts": [],
    "data": ""
  },
  "addressLookupTableAddresses": [
    "D9pNYUKyF3Xm5wdMhqsJNQqNnqZVmHZGbMRMqg5CaSqK",
    "4uRre8j1GEn8F5z5YDKX6ViZ1FvXMYMuEwqLBKaPwPHh"
  ]
}
```

**Field Descriptions:**

| Field | Type | Description |
|-------|------|-------------|
| `tokenLedgerInstruction` | object/null | Token ledger instruction (if `useTokenLedger=true`) |
| `computeBudgetInstructions` | array | Compute budget setup instructions |
| `setupInstructions` | array | ATA initialization instructions |
| `swapInstruction` | object | Core swap instruction |
| `cleanupInstruction` | object | SOL unwrap instruction |
| `addressLookupTableAddresses` | array | ALT addresses for versioned transactions |

**Instruction Structure:**

```typescript
interface Instruction {
  programId: string;          // Program public key
  accounts: Account[];        // Account metadata array
  data: string;              // Base64-encoded instruction data
}

interface Account {
  pubkey: string;            // Account public key
  isSigner: boolean;         // Account must sign transaction
  isWritable: boolean;       // Account will be modified
}
```

---

## Price API Responses

### Get Prices Response

**Endpoint:** `GET /price/v3`

**Success Response (200):**
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
  },
  "unknownToken111111111111111111111111111111": null
}
```

**Field Descriptions:**

| Field | Type | Description |
|-------|------|-------------|
| `usdPrice` | number | Current price in USD |
| `blockId` | integer | Solana block number of price data |
| `decimals` | integer | Token decimal places |
| `priceChange24h` | number | 24-hour price change percentage |

**Note:** Tokens without recent trades or failing quality checks return `null`.

---

## Tokens API Responses

### Token Metadata Structure

**Endpoints:** `/search`, `/tag`, `/{category}/{interval}`, `/recent`

**Success Response (200):**
```json
[
  {
    "id": "So11111111111111111111111111111111111111112",
    "name": "Wrapped SOL",
    "symbol": "SOL",
    "icon": "https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/So11111111111111111111111111111111111111112/logo.png",
    "decimals": 9,
    "circSupply": 412000000,
    "totalSupply": 581000000,
    "tokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
    "firstPool": {
      "timestamp": 1640000000,
      "poolAddress": "8BnEgHoWFysVcuFFX7QztDmzuH8r5ZFvyP3sYwn1XTh6"
    },
    "holderCount": 5000000,
    "audit": {
      "authority": "disabled",
      "holderConcentration": 0.05,
      "top10HolderPercent": 0.25,
      "isFreezable": false,
      "isMintable": false
    },
    "organicScore": 98.5,
    "organicScoreLabel": "high",
    "isVerified": true,
    "cexes": ["binance", "coinbase", "kraken", "okx"],
    "tags": ["verified", "blue-chip"],
    "fdv": 85000000000,
    "mcap": 60000000000,
    "usdPrice": 147.48,
    "liquidity": 500000000,
    "volume24h": 50000000,
    "stats5m": {
      "volume": 1000000,
      "priceChange": 0.1,
      "priceChangePercent": 0.07,
      "buys": 150,
      "sells": 120,
      "traders": 200,
      "buyVolume": 600000,
      "sellVolume": 400000
    },
    "stats1h": {
      "volume": 5000000,
      "priceChange": 0.5,
      "priceChangePercent": 0.34,
      "buys": 800,
      "sells": 700,
      "traders": 1200,
      "buyVolume": 3000000,
      "sellVolume": 2000000
    },
    "stats6h": {
      "volume": 20000000,
      "priceChange": 2.0,
      "priceChangePercent": 1.36,
      "buys": 4000,
      "sells": 3500,
      "traders": 5000,
      "buyVolume": 12000000,
      "sellVolume": 8000000
    },
    "stats24h": {
      "volume": 50000000,
      "priceChange": 1.89,
      "priceChangePercent": 1.29,
      "buys": 15000,
      "sells": 13000,
      "traders": 18000,
      "buyVolume": 30000000,
      "sellVolume": 20000000
    }
  }
]
```

**Field Descriptions:**

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Token mint address |
| `name` | string | Token full name |
| `symbol` | string | Token symbol/ticker |
| `icon` | string | Token icon URL |
| `decimals` | integer | Decimal places |
| `circSupply` | number | Circulating supply |
| `totalSupply` | number | Total supply |
| `tokenProgram` | string | Token program ID |
| `firstPool` | object | First pool creation info |
| `holderCount` | integer | Number of holders |
| `audit` | object | Token audit information |
| `organicScore` | number | Quality score (0-100) |
| `organicScoreLabel` | string | "low", "medium", "high", "very-high" |
| `isVerified` | boolean | Jupiter verification status |
| `cexes` | array | List of CEXs listing token |
| `tags` | array | Token tags/categories |
| `fdv` | number | Fully diluted valuation (USD) |
| `mcap` | number | Market capitalization (USD) |
| `usdPrice` | number | Current USD price |
| `liquidity` | number | Total liquidity (USD) |
| `volume24h` | number | 24-hour volume (USD) |
| `stats{interval}` | object | Trading stats by time window |

**Audit Object:**

```typescript
interface Audit {
  authority: "enabled" | "disabled";      // Token authority status
  holderConcentration: number;            // Top holder percentage
  top10HolderPercent: number;             // Top 10 holders percentage
  isFreezable: boolean;                   // Can token be frozen
  isMintable: boolean;                    // Can token be minted
}
```

**Stats Object:**

```typescript
interface Stats {
  volume: number;                         // Trading volume
  priceChange: number;                    // Absolute price change
  priceChangePercent: number;             // Percentage price change
  buys: number;                           // Number of buy transactions
  sells: number;                          // Number of sell transactions
  traders: number;                        // Unique traders
  buyVolume: number;                      // Buy volume
  sellVolume: number;                     // Sell volume
}
```

---

## Program Error Codes

### Jupiter V6 Aggregator Errors

These errors may appear in failed transactions:

| Code | Error | Description |
|------|-------|-------------|
| 6000 | SlippageToleranceExceeded | Output amount below minimum |
| 6001 | InvalidCalculation | Route calculation error |
| 6002 | MissingPlatformFeeAccount | Fee account not provided |
| 6003 | InvalidSlippage | Slippage value out of range |
| 6004 | NotEnoughAccountKeys | Insufficient accounts |

**Error in Transaction:**
```json
{
  "err": {
    "InstructionError": [
      0,
      {
        "Custom": 6000
      }
    ]
  }
}
```

---

## Common Error Scenarios

### No Route Found

**Error:**
```json
{
  "error": "No route found",
  "data": {
    "inputMint": "TokenA",
    "outputMint": "TokenB",
    "reason": "Insufficient liquidity"
  }
}
```

**Causes:**
- No liquidity pools exist for pair
- Insufficient liquidity for amount
- Token not supported
- New token without pools yet

**Solution:**
- Reduce trade amount
- Try different route (enable more DEXs)
- Check token has liquidity

---

### Transaction Expired

**Error Code:** -1005

**Cause:** Transaction not sent within valid block height window

**Solution:**
- Get fresh quote
- Build new transaction
- Submit faster

---

### Slippage Exceeded

**Error Code:** 6000

**Cause:** Price moved beyond slippage tolerance

**Solution:**
- Increase `slippageBps` parameter
- Use dynamic slippage
- Reduce trade size

---

## Response Headers

### Rate Limit Headers

```http
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1640000010
```

| Header | Description |
|--------|-------------|
| `X-RateLimit-Limit` | Total requests allowed in window |
| `X-RateLimit-Remaining` | Requests remaining in current window |
| `X-RateLimit-Reset` | Unix timestamp when limit resets |

### Standard Headers

```http
Content-Type: application/json
Content-Length: 1234
Date: Mon, 20 Jan 2026 12:00:00 GMT
```

---

## Data Types Reference

### String Types

- **Mint Address**: Base58-encoded string (32-44 chars)
- **Amount**: String representation of u64 number
- **Transaction**: Base64-encoded binary data
- **Price Impact**: Decimal string (e.g., "0.0123")

### Number Types

- **Basis Points (bps)**: Integer (0-10000)
  - 1 bps = 0.01%
  - 50 bps = 0.5%
  - 100 bps = 1%

- **Lamports**: Integer (1 lamport = 0.000000001 SOL)

- **Block Height**: Integer (current Solana block number)

### Enums

**Swap Mode:**
- `ExactIn`: Specify exact input amount
- `ExactOut`: Specify exact output amount

**Priority Level:**
- `none`
- `low`
- `medium`
- `high`
- `veryHigh`

**Organic Score Label:**
- `low`: 0-25
- `medium`: 26-50
- `high`: 51-75
- `very-high`: 76-100

---

## TypeScript Type Definitions

```typescript
// Quote Response
interface QuoteResponse {
  inputMint: string;
  inAmount: string;
  outputMint: string;
  outAmount: string;
  otherAmountThreshold: string;
  swapMode: "ExactIn" | "ExactOut";
  slippageBps: number;
  priceImpactPct: string;
  routePlan: RoutePlan[];
  platformFee?: PlatformFee;
  contextSlot: number;
  timeTaken: number;
}

// Swap Response
interface SwapResponse {
  swapTransaction: string;
  lastValidBlockHeight: number;
  prioritizationFeeLamports: number;
  computeUnitLimit?: number;
  dynamicSlippageReport?: DynamicSlippageReport;
}

// Price Response
type PriceResponse = Record<string, PriceData | null>;

interface PriceData {
  usdPrice: number;
  blockId: number;
  decimals: number;
  priceChange24h: number;
}

// Token Response
interface TokenData {
  id: string;
  name: string;
  symbol: string;
  decimals: number;
  organicScore: number;
  isVerified: boolean;
  usdPrice: number;
  liquidity: number;
  stats24h: Stats;
}
```

---

## Notes

1. All amount fields are strings to prevent precision loss
2. Prices are always in USD
3. Timestamps are Unix timestamps (seconds since epoch)
4. Block IDs are Solana slot numbers
5. `null` values indicate missing or unreliable data
6. Response times typically < 1 second for quotes
7. Transaction expiry is ~150 blocks (~1 minute)
