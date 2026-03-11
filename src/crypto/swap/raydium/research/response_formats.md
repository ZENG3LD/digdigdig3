# Raydium API Response Formats - Complete Reference

**Research Date**: 2026-01-20

This document details the JSON response structures for all major Raydium API endpoints.

---

## Table of Contents

1. [General Response Structure](#general-response-structure)
2. [Token List Response](#token-list-response)
3. [Token Info Response](#token-info-response)
4. [Token Price Response](#token-price-response)
5. [Pool List Response](#pool-list-response)
6. [Pool Info Response](#pool-info-response)
7. [Farm Info Response](#farm-info-response)
8. [Swap Quote Response](#swap-quote-response)
9. [Priority Fee Response](#priority-fee-response)
10. [Error Response](#error-response)

---

## General Response Structure

### Success Response

**Format**:
```json
{
  "id": "string",
  "success": true,
  "data": {
    // endpoint-specific data
  }
}
```

**Fields**:
- `id` (string): Unique request identifier
- `success` (boolean): Always `true` for successful responses
- `data` (object/array): Response payload (varies by endpoint)

### Pagination Response

**Format**:
```json
{
  "id": "string",
  "success": true,
  "data": {
    "count": 1234,
    "hasNextPage": true,
    "data": [
      // array of items
    ]
  }
}
```

**Fields**:
- `count` (number): Total number of items
- `hasNextPage` (boolean): Whether more pages exist
- `data` (array): Current page items

---

## Token List Response

### Endpoint

`GET /mint/list`

### Response Format

```json
{
  "id": "req-12345",
  "success": true,
  "data": [
    {
      "address": "So11111111111111111111111111111111111111112",
      "chainId": 101,
      "symbol": "SOL",
      "name": "Wrapped SOL",
      "decimals": 9,
      "logoURI": "https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/So11111111111111111111111111111111111111112/logo.png",
      "tags": ["wrapped", "native"],
      "extensions": {
        "coingeckoId": "solana"
      }
    },
    {
      "address": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
      "chainId": 101,
      "symbol": "USDC",
      "name": "USD Coin",
      "decimals": 6,
      "logoURI": "https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v/logo.png",
      "tags": ["stablecoin"],
      "extensions": {
        "coingeckoId": "usd-coin"
      }
    }
  ]
}
```

### Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| `address` | string | Token mint address (Solana pubkey) |
| `chainId` | number | 101 for Solana mainnet, 102 for testnet, 103 for devnet |
| `symbol` | string | Token ticker symbol (e.g., "SOL", "USDC") |
| `name` | string | Full token name |
| `decimals` | number | Number of decimal places (6-9 typical) |
| `logoURI` | string | URL to token logo image |
| `tags` | array | Category tags (optional) |
| `extensions` | object | Additional metadata (optional) |

---

## Token Info Response

### Endpoint

`GET /mint/ids?mints=So11111111111111111111111111111111111111112,EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v`

### Response Format

```json
{
  "id": "req-12346",
  "success": true,
  "data": {
    "So11111111111111111111111111111111111111112": {
      "address": "So11111111111111111111111111111111111111112",
      "chainId": 101,
      "symbol": "SOL",
      "name": "Wrapped SOL",
      "decimals": 9,
      "logoURI": "https://...",
      "tags": ["wrapped", "native"],
      "extensions": {
        "coingeckoId": "solana"
      }
    },
    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v": {
      "address": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
      "chainId": 101,
      "symbol": "USDC",
      "name": "USD Coin",
      "decimals": 6,
      "logoURI": "https://...",
      "tags": ["stablecoin"]
    }
  }
}
```

**Key Structure**: Object with mint addresses as keys, token info as values.

---

## Token Price Response

### Endpoint

`GET /mint/price?mints=So11111111111111111111111111111111111111112,EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v`

### Response Format

```json
{
  "id": "req-12347",
  "success": true,
  "data": {
    "So11111111111111111111111111111111111111112": 145.67,
    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v": 1.0
  }
}
```

**Key Structure**: Object with mint addresses as keys, prices (in USD) as values.

**Price Format**:
- `number`: Price in USD as floating point
- Example: `145.67` means $145.67 per token

---

## Pool List Response

### Endpoint

`GET /pools/info/list?type=All&sort=liquidity&order=desc&page=1&pageSize=10`

### Response Format

```json
{
  "id": "req-12348",
  "success": true,
  "data": {
    "count": 1234,
    "hasNextPage": true,
    "data": [
      {
        "id": "AVs9TA4nWDzfPJE9gGVNJMVhcQy3V9PGazuz33BfG2RA",
        "type": "Standard",
        "programId": "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8",
        "mintA": {
          "address": "So11111111111111111111111111111111111111112",
          "symbol": "SOL",
          "decimals": 9,
          "logoURI": "https://..."
        },
        "mintB": {
          "address": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
          "symbol": "USDC",
          "decimals": 6,
          "logoURI": "https://..."
        },
        "price": 145.67,
        "mintAmountA": 12345.678901234,
        "mintAmountB": 1800000.123456,
        "feeRate": 0.0025,
        "openTime": "0",
        "tvl": 2634567.89,
        "day": {
          "volume": 1234567.89,
          "volumeQuote": 1234567.89,
          "volumeFee": 3086.42,
          "apr": 12.34,
          "feeApr": 12.34,
          "priceMin": 142.15,
          "priceMax": 148.92
        },
        "week": {
          "volume": 8641975.32,
          "volumeQuote": 8641975.32,
          "volumeFee": 21604.94,
          "apr": 11.87,
          "feeApr": 11.87,
          "priceMin": 138.45,
          "priceMax": 151.23
        },
        "month": {
          "volume": 37123456.78,
          "volumeQuote": 37123456.78,
          "volumeFee": 92808.64,
          "apr": 10.98,
          "feeApr": 10.98,
          "priceMin": 125.67,
          "priceMax": 158.91
        },
        "pooltype": ["AMM"],
        "rewardDefaultInfos": [],
        "farmUpcomingCount": 0,
        "farmOngoingCount": 0,
        "farmFinishedCount": 0
      }
    ]
  }
}
```

### Pool Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Pool ID (Solana pubkey) |
| `type` | string | "Standard" (constant product AMM) or "Concentrated" (CLMM) |
| `programId` | string | Raydium program address |
| `mintA` | object | First token in pair |
| `mintB` | object | Second token in pair |
| `price` | number | Current price (mintA/mintB ratio) |
| `mintAmountA` | number | Reserve amount of token A |
| `mintAmountB` | number | Reserve amount of token B |
| `feeRate` | number | Swap fee rate (0.0025 = 0.25%) |
| `openTime` | string | Pool opening timestamp ("0" = always open) |
| `tvl` | number | Total value locked in USD |
| `day`/`week`/`month` | object | Time-period statistics |

### Time Period Statistics

| Field | Type | Description |
|-------|------|-------------|
| `volume` | number | Trading volume in base token |
| `volumeQuote` | number | Trading volume in quote token (USD) |
| `volumeFee` | number | Total fees collected |
| `apr` | number | Annual percentage rate |
| `feeApr` | number | APR from fees only |
| `priceMin` | number | Minimum price in period |
| `priceMax` | number | Maximum price in period |

---

## Pool Info Response

### Endpoint

`GET /pools/info/ids?ids=AVs9TA4nWDzfPJE9gGVNJMVhcQy3V9PGazuz33BfG2RA`

### Response Format

```json
{
  "id": "req-12349",
  "success": true,
  "data": [
    {
      "id": "AVs9TA4nWDzfPJE9gGVNJMVhcQy3V9PGazuz33BfG2RA",
      "type": "Standard",
      "programId": "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8",
      "mintA": {
        "address": "So11111111111111111111111111111111111111112",
        "symbol": "SOL",
        "decimals": 9,
        "logoURI": "https://..."
      },
      "mintB": {
        "address": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        "symbol": "USDC",
        "decimals": 6,
        "logoURI": "https://..."
      },
      "price": 145.67,
      "mintAmountA": 12345.678901234,
      "mintAmountB": 1800000.123456,
      "feeRate": 0.0025,
      "lpMint": {
        "address": "8HoQnePLqPj4M7PUDzfw8e3Ymdwgc7NLGnaTUapubyvu",
        "decimals": 9,
        "supply": 1234567.89
      },
      "lpPrice": 2.134,
      "marketId": "9wFFyRfZBsuAha4YcuxcXLKwMxJR43S7fPfQLusDBzvT",
      "marketProgramId": "srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX",
      "tvl": 2634567.89,
      "day": {
        // same as pool list
      },
      "week": {
        // same as pool list
      },
      "month": {
        // same as pool list
      }
    }
  ]
}
```

**Additional Fields**:
- `lpMint`: LP token information
  - `address`: LP token mint address
  - `decimals`: LP token decimals
  - `supply`: Circulating LP token supply
- `lpPrice`: LP token price in USD
- `marketId`: OpenBook market ID (if applicable)
- `marketProgramId`: OpenBook program address

---

## Farm Info Response

### Endpoint

`GET /farms/info/ids?ids=4EwbZo8BZXP5313z5A2H11MRBP15M5n6YxfmkjXESKAW`

### Response Format

```json
{
  "id": "req-12350",
  "success": true,
  "data": [
    {
      "id": "4EwbZo8BZXP5313z5A2H11MRBP15M5n6YxfmkjXESKAW",
      "lpMint": {
        "address": "8HoQnePLqPj4M7PUDzfw8e3Ymdwgc7NLGnaTUapubyvu",
        "symbol": "SOL-USDC",
        "decimals": 9
      },
      "rewardInfos": [
        {
          "mint": {
            "address": "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R",
            "symbol": "RAY",
            "decimals": 6
          },
          "openTime": "1640000000",
          "endTime": "1672000000",
          "perSecond": "0.123456",
          "apr": 15.67
        }
      ],
      "tvl": 5678912.34,
      "apr": 15.67,
      "totalApr": 28.01
    }
  ]
}
```

### Farm Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Farm ID |
| `lpMint` | object | LP token being staked |
| `rewardInfos` | array | Reward token information |
| `tvl` | number | Total value locked in farm (USD) |
| `apr` | number | APR from trading fees |
| `totalApr` | number | Total APR (fees + rewards) |

### Reward Info Fields

| Field | Type | Description |
|-------|------|-------------|
| `mint` | object | Reward token details |
| `openTime` | string | Reward distribution start time (unix timestamp) |
| `endTime` | string | Reward distribution end time |
| `perSecond` | string | Tokens distributed per second |
| `apr` | number | APR contribution from this reward |

---

## Swap Quote Response

### Endpoint

`GET /compute/swap-base-in?inputMint=So11111111111111111111111111111111111111112&outputMint=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v&amount=1000000000&slippageBps=50&txVersion=V0`

### Response Format

```json
{
  "id": "unique-compute-id",
  "success": true,
  "data": {
    "swapType": "BaseIn",
    "inputMint": "So11111111111111111111111111111111111111112",
    "inputAmount": "1000000000",
    "outputMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    "outputAmount": "145670000",
    "otherAmountThreshold": "145597650",
    "slippageBps": 50,
    "priceImpactPct": 0.0123,
    "routePlan": [
      {
        "poolId": "AVs9TA4nWDzfPJE9gGVNJMVhcQy3V9PGazuz33BfG2RA",
        "inputMint": "So11111111111111111111111111111111111111112",
        "outputMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        "feeMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        "feeRate": 25,
        "feeAmount": "364175"
      }
    ]
  }
}
```

### Swap Quote Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| `swapType` | string | "BaseIn" or "BaseOut" |
| `inputMint` | string | Source token address |
| `inputAmount` | string | Input amount in base units (lamports/smallest unit) |
| `outputMint` | string | Destination token address |
| `outputAmount` | string | Expected output amount |
| `otherAmountThreshold` | string | Minimum output after slippage (BaseIn) or max input (BaseOut) |
| `slippageBps` | number | Slippage tolerance in basis points |
| `priceImpactPct` | number | Estimated price impact percentage |
| `routePlan` | array | Route through pools (can be multi-hop) |

### Route Plan Fields

| Field | Type | Description |
|-------|------|-------------|
| `poolId` | string | Pool address for this hop |
| `inputMint` | string | Input token for this hop |
| `outputMint` | string | Output token for this hop |
| `feeMint` | string | Token in which fee is charged |
| `feeRate` | number | Fee rate in basis points (25 = 0.25%) |
| `feeAmount` | string | Fee amount in base units |

---

## Priority Fee Response

### Endpoint

`GET /main/auto-fee`

### Response Format

```json
{
  "id": "req-12351",
  "success": true,
  "data": {
    "default": {
      "vh": 1000000,
      "h": 500000,
      "m": 100000
    }
  }
}
```

### Priority Fee Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| `vh` | number | Very high priority fee (microLamports) |
| `h` | number | High priority fee (microLamports) |
| `m` | number | Medium priority fee (microLamports) |

**Conversion**: 1 SOL = 1,000,000,000 microLamports
- `vh: 1000000` = 0.001 SOL = ~$0.15 priority fee

---

## Error Response

### Format

When an error occurs:

```json
{
  "id": "req-error-12345",
  "success": false,
  "error": {
    "code": "ERROR_CODE",
    "message": "Human readable error message"
  }
}
```

**Field Definitions**:
- `success` (boolean): `false` for errors
- `error` (object): Error details
  - `code` (string): Error code identifier
  - `message` (string): Error description

### Common Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `INVALID_PARAM` | 400 | Invalid query parameter |
| `NOT_FOUND` | 404 | Resource not found |
| `TOO_MANY_REQUESTS` | 429 | Rate limit exceeded |
| `INTERNAL_ERROR` | 500 | Server error |
| `SERVICE_UNAVAILABLE` | 503 | Service temporarily unavailable |

**Note**: Specific error codes are not publicly documented. These are inferred from typical REST API patterns.

---

## Response Parsing Notes

### Numeric Precision

**Amounts in Base Units**:
- All token amounts returned in **smallest unit** (lamports for SOL)
- Must divide by `10^decimals` to get human-readable amount
- Example: `1000000000` lamports ÷ 10^9 = 1.0 SOL

**Conversion Formula**:
```rust
fn to_human_readable(amount_string: &str, decimals: u8) -> f64 {
    let amount = amount_string.parse::<u64>().unwrap();
    amount as f64 / 10f64.powi(decimals as i32)
}
```

**Example**:
- `mintAmountA: 12345.678901234` → Already human-readable (pool endpoint)
- `inputAmount: "1000000000"` → Raw units (quote endpoint)

**Inconsistency**: Pool endpoints return human-readable numbers, quote endpoints return string-encoded integers in base units.

### String vs Number

**Numbers as Strings**:
- Large amounts: `"1000000000"` (to avoid precision loss)
- Small amounts: `"0.123456"` (high precision decimals)

**Numbers as Numbers**:
- Prices: `145.67` (JSON number)
- Percentages: `0.0025` (JSON number)
- APR: `12.34` (JSON number)

**Parsing Strategy**:
```rust
// For string amounts
let amount = response["inputAmount"].as_str().unwrap().parse::<u64>()?;

// For numeric prices
let price = response["price"].as_f64().unwrap();
```

### Optional Fields

**Fields that may be `null` or missing**:
- `logoURI` (if token not verified)
- `tags` (if no tags assigned)
- `extensions` (if no metadata)
- `marketId` (for pools without OpenBook integration)
- `rewardDefaultInfos` (if no active rewards)

**Safe Parsing**:
```rust
let logo = data["logoURI"].as_str(); // Returns Option<&str>
let tags = data["tags"].as_array().unwrap_or(&vec![]); // Default to empty
```

### Array vs Object Response

**Array Response** (list endpoints):
```json
{
  "data": [
    { "id": "1", ... },
    { "id": "2", ... }
  ]
}
```

**Object Response** (keyed by ID):
```json
{
  "data": {
    "mint_address_1": { ... },
    "mint_address_2": { ... }
  }
}
```

**Pagination Response** (nested data):
```json
{
  "data": {
    "count": 100,
    "data": [ /* items */ ]
  }
}
```

---

## Legacy V2 Response Formats

### Liquidity JSON (Static File)

**URL**: `https://api.raydium.io/v2/sdk/liquidity/mainnet.json`

**Format**:
```json
{
  "official": [
    {
      "id": "AVs9TA4nWDzfPJE9gGVNJMVhcQy3V9PGazuz33BfG2RA",
      "baseMint": "So11111111111111111111111111111111111111112",
      "quoteMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
      "lpMint": "8HoQnePLqPj4M7PUDzfw8e3Ymdwgc7NLGnaTUapubyvu",
      "baseDecimals": 9,
      "quoteDecimals": 6,
      "lpDecimals": 9,
      "version": 4,
      "programId": "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8",
      "authority": "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1",
      "openOrders": "HRk9CMrpq7Jn9sh7mzxE8CChHG8dneX9p475QKz4Fsfc",
      "targetOrders": "CZza3Ej4Mc58MnxWA385itCC9jCo3L1D7zc3LKy1bZMR",
      "baseVault": "DQyrAcCrDXQ7NeoqGgDCZwBvWDcYmFCjSb9JtteuvPpz",
      "quoteVault": "HLmqeL62xR1QoZ1HKKbXRrdN1p3phKpxRMb2VVopvBBz",
      "withdrawQueue": "G7xeGGLevkRwB5f44QNgQtrPKBdMfkT6ZZwpS9xcC97n",
      "lpVault": "Awpt6N7ZYPBa4vG4BQNFhFxDj4sxExAA9rpBAoBw2uok",
      "marketVersion": 3,
      "marketProgramId": "srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX",
      "marketId": "9wFFyRfZBsuAha4YcuxcXLKwMxJR43S7fPfQLusDBzvT",
      "marketAuthority": "F8Vyqk3unwxkXukZFQeYyGmFfTG3CAX4v24iyrjEYBJV",
      "marketBaseVault": "36c6YqAwyGKQG66XEp2dJc5JqjaBNv7sVghEtJv4c7u6",
      "marketQuoteVault": "8CFo8bL8mZQK8abbFyypFMwEDd8tVJjHTTojMLgQTUSZ",
      "marketBids": "14ivtgssEBoBjuZJtSAPKYgpUK7DmnSwuPMqJoVTSgKJ",
      "marketAsks": "CEQdAFKdycHugujQg9k2wbmxjcpdYZyVLfV9WerTnafJ",
      "marketEventQueue": "5KKsLVU6TcbVDK4BS6K1DGDxnh4Q9xjYJ8XaDCG5t8ht",
      "lookupTableAccount": "5DDNNv1z1PXhG2S7FeihHhvBz1hPXuEHqYbMq3aqJ2Ce"
    }
  ],
  "unOfficial": []
}
```

**All Pubkeys**: This format includes ALL on-chain addresses needed for direct program interaction.

---

## Data Type Reference

### Solana Address (Pubkey)

**Format**: Base58-encoded string, 32-44 characters

**Examples**:
- `So11111111111111111111111111111111111111112` (SOL)
- `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v` (USDC)
- `675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8` (Raydium Program)

**Validation**: Always 32 bytes when decoded from Base58.

### Amount (Lamports)

**Format**: String-encoded unsigned 64-bit integer

**Examples**:
- `"1000000000"` = 1 SOL (9 decimals)
- `"1000000"` = 1 USDC (6 decimals)
- `"364175"` = 0.364175 USDC fee

**Range**: 0 to 18,446,744,073,709,551,615 (u64::MAX)

### Decimal Number

**Format**: JSON number with fractional part

**Examples**:
- `145.67` (price in USD)
- `0.0025` (fee rate = 0.25%)
- `12.34` (APR percentage)

**Precision**: JavaScript number precision (~15-17 significant digits)

### Basis Points

**Format**: Integer representing 1/100th of a percent

**Examples**:
- `50` = 0.50%
- `25` = 0.25%
- `100` = 1.00%

**Formula**: `percentage = basis_points / 100.0`

---

## SDK Response Normalization

The Raydium SDK V2 normalizes API responses:

**Raw API Response**:
```json
{
  "data": [{ "id": "pool1" }]
}
```

**Or Paginated**:
```json
{
  "data": {
    "data": [{ "id": "pool1" }]
  }
}
```

**SDK Normalization**:
```typescript
function normalizeRaydiumBetaPoolInfoResponse(
  data: raydium.ApiV3PoolInfoItem[] | raydium.ApiV3PageIns<raydium.ApiV3PoolInfoItem>
): raydium.ApiV3PoolInfoItem[] {
  return Array.isArray(data) ? data : data.data;
}
```

**Why?**: Some endpoints return arrays, others return paginated objects. SDK abstracts this inconsistency.

---

## Parser Implementation Hints

### Required Validations

When parsing Raydium responses:

```rust
// 1. Check success flag
if !response["success"].as_bool().unwrap_or(false) {
    return Err(parse_error_response(response));
}

// 2. Validate required fields exist
let price = response["data"]["price"]
    .as_f64()
    .ok_or(ParseError::MissingField("price"))?;

// 3. Validate mint addresses are valid Base58
let mint = response["data"]["mintA"]["address"]
    .as_str()
    .ok_or(ParseError::MissingField("mintA.address"))?;

validate_solana_address(mint)?;

// 4. Convert amount strings to integers
let amount = response["data"]["mintAmountA"]
    .as_str()
    .ok_or(ParseError::MissingField("mintAmountA"))?
    .parse::<u64>()
    .map_err(|e| ParseError::InvalidAmount(e))?;

// 5. Handle nullable fields gracefully
let logo_uri = response["data"]["logoURI"]
    .as_str()
    .unwrap_or("https://placeholder.com/token.png");
```

### Common Parsing Patterns

**Pool Data**:
```rust
pub struct PoolInfo {
    pub id: String,
    pub mint_a: TokenInfo,
    pub mint_b: TokenInfo,
    pub price: f64,
    pub reserve_a: f64, // human-readable
    pub reserve_b: f64, // human-readable
    pub tvl: f64,
    pub volume_24h: f64,
    pub apr_24h: f64,
}

impl PoolInfo {
    fn from_json(data: &serde_json::Value) -> Result<Self> {
        Ok(Self {
            id: data["id"].as_str().ok_or(ParseError)?.to_string(),
            mint_a: TokenInfo::from_json(&data["mintA"])?,
            mint_b: TokenInfo::from_json(&data["mintB"])?,
            price: data["price"].as_f64().ok_or(ParseError)?,
            reserve_a: data["mintAmountA"].as_f64().ok_or(ParseError)?,
            reserve_b: data["mintAmountB"].as_f64().ok_or(ParseError)?,
            tvl: data["tvl"].as_f64().ok_or(ParseError)?,
            volume_24h: data["day"]["volume"].as_f64().ok_or(ParseError)?,
            apr_24h: data["day"]["apr"].as_f64().ok_or(ParseError)?,
        })
    }
}
```

**Quote Data**:
```rust
pub struct SwapQuote {
    pub input_mint: String,
    pub input_amount: u64, // base units
    pub output_mint: String,
    pub output_amount: u64, // base units
    pub minimum_received: u64, // after slippage
    pub price_impact: f64,
    pub route: Vec<RouteHop>,
}

impl SwapQuote {
    fn from_json(data: &serde_json::Value) -> Result<Self> {
        Ok(Self {
            input_mint: data["inputMint"].as_str().ok_or(ParseError)?.to_string(),
            input_amount: data["inputAmount"].as_str().ok_or(ParseError)?.parse()?,
            output_mint: data["outputMint"].as_str().ok_or(ParseError)?.to_string(),
            output_amount: data["outputAmount"].as_str().ok_or(ParseError)?.parse()?,
            minimum_received: data["otherAmountThreshold"].as_str().ok_or(ParseError)?.parse()?,
            price_impact: data["priceImpactPct"].as_f64().ok_or(ParseError)?,
            route: data["routePlan"].as_array()
                .ok_or(ParseError)?
                .iter()
                .map(RouteHop::from_json)
                .collect::<Result<Vec<_>>>()?,
        })
    }
}
```

---

## Sources

Research compiled from the following sources:

- [Raydium API V3 Documentation](https://api-v3.raydium.io/docs/)
- [Raydium SDK V2 GitHub](https://github.com/raydium-io/raydium-sdk-V2)
- [Raydium Pool Info Gist](https://gist.github.com/rubpy/2ba5d409181675c0e49341ce150ac498)
- [Raydium SDK TypeScript Source](https://github.com/raydium-io/raydium-sdk-V2/blob/master/src/api/api.ts)
- [Solana Token List Standard](https://github.com/solana-labs/token-list)

---

**Document Version**: 1.0
**Last Updated**: 2026-01-20
**Research Completed By**: Claude Code Research Agent
