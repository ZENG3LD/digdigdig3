# Jupiter DEX Trading API Specification

Source: https://dev.jup.ag/ (official Jupiter developer documentation)
Research date: 2026-03-11

---

## Overview

Jupiter offers three distinct trading paradigms, each with separate API families:

| Trading Type | API Family | Base URL |
|---|---|---|
| Instant swap (market) | Ultra Swap API | `https://api.jup.ag/ultra/v1/` |
| Instant swap (manual RPC) | Metis Swap API | `https://api.jup.ag/swap/v1/` |
| Limit / trigger orders | Trigger API | `https://api.jup.ag/trigger/v1/` |
| DCA / recurring | Recurring API | `https://api.jup.ag/recurring/v1/` |
| Perpetuals | On-chain program (Anchor IDL) | Program-based, not REST |

There is also a `lite-api.jup.ag` mirror (e.g. `https://lite-api.jup.ag/ultra/v1/`) available for free-tier usage without API key in some contexts, though `api.jup.ag` is the canonical production host.

---

## 1. Instant Swap

### 1A. Ultra Swap API (recommended)

The Ultra Swap API is Jupiter's recommended swap path. It is RPC-less: Jupiter handles broadcasting, priority fees, slippage estimation, and transaction landing internally. The developer only signs the transaction.

#### Step 1 — Get Order (Quote + Transaction)

```
GET https://api.jup.ag/ultra/v1/order
```

**Required headers:**
- `x-api-key: <your-api-key>`

**Required query parameters:**

| Parameter | Type | Description |
|---|---|---|
| `inputMint` | string | Source token mint address (base58) |
| `outputMint` | string | Destination token mint address (base58) |
| `amount` | uint64 | Amount in smallest token unit (lamports/base units) |
| `taker` | string | User wallet public key (base58) |

**Optional query parameters:**

| Parameter | Type | Description |
|---|---|---|
| `referralAccount` | string | Referral account for fee distribution |
| `referralFee` | number | Fee in basis points |

**Example request:**
```
GET https://api.jup.ag/ultra/v1/order
  ?inputMint=So11111111111111111111111111111111111111112
  &outputMint=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
  &amount=100000000
  &taker=jdocuPgEAjMfihABsPgKEvYtsmMzjUHeq9LX4Hvs7f3
```

**Response (200):**
```json
{
  "transaction": "<base64-encoded unsigned transaction>",
  "requestId": "<UUID string>",
  "swapType": "...",
  "slippageBps": 50
}
```

- `transaction`: Must be deserialized, signed with user keypair, re-serialized to base64
- `requestId`: Must be passed to `/execute` endpoint

---

#### Step 2 — Execute Order

```
POST https://api.jup.ag/ultra/v1/execute
```

**Required headers:**
- `Content-Type: application/json`
- `x-api-key: <your-api-key>`

**Request body:**
```json
{
  "signedTransaction": "<base64-encoded signed transaction>",
  "requestId": "<UUID from /order response>"
}
```

**Response (success):**
```json
{
  "status": "Success",
  "signature": "<base58 transaction signature>"
}
```

**Response (failure):**
```json
{
  "status": "Failed",
  "error": "<error description>",
  "signature": "<base58 transaction signature>"
}
```

**Notes:**
- Same signed transaction may be resubmitted within 2 minutes for status polling without risk of duplicate execution
- Jupiter handles priority fees, slippage, and transaction landing automatically
- Typical end-to-end latency: under 2 seconds (P95)

---

### 1B. Metis Swap API (manual RPC path)

Use when: CPI (Cross Program Invocation) is needed, custom transaction composition is required, or existing RPC infrastructure is preferred. Requires developer to manage their own RPC connection and transaction sending. Development effort: 3-6 months vs hours for Ultra.

#### Step 1 — Get Quote

```
GET https://api.jup.ag/swap/v1/quote
```

**Required query parameters:**

| Parameter | Type | Description |
|---|---|---|
| `inputMint` | string | Source token mint address |
| `outputMint` | string | Destination token mint address |
| `amount` | uint64 | Raw amount; input for ExactIn mode, output for ExactOut |

**Optional query parameters:**

| Parameter | Type | Default | Description |
|---|---|---|---|
| `slippageBps` | uint16 | 50 | Slippage tolerance in basis points |
| `swapMode` | string | `ExactIn` | `ExactIn` or `ExactOut` |
| `dexes` | array | all | Comma-separated DEX names to include |
| `excludeDexes` | array | none | Comma-separated DEX names to exclude |
| `restrictIntermediateTokens` | boolean | true | Limit routing to stable intermediate tokens |
| `onlyDirectRoutes` | boolean | false | Single-hop routes only |
| `asLegacyTransaction` | boolean | false | Use legacy vs versioned transactions |
| `platformFeeBps` | uint16 | — | Platform fee (requires `feeAccount` in `/swap`) |
| `maxAccounts` | uint64 | 64 | Max account estimate |
| `instructionVersion` | string | `V1` | `V1` or `V2` |
| `forJitoBundle` | boolean | false | Exclude Jito-incompatible DEXes |

**Response (200):**
```json
{
  "inputMint": "string",
  "inAmount": "string",
  "outputMint": "string",
  "outAmount": "string",
  "otherAmountThreshold": "string",
  "swapMode": "ExactIn",
  "slippageBps": 50,
  "priceImpactPct": "string",
  "platformFee": {
    "amount": "string",
    "feeBps": 0
  },
  "routePlan": [
    {
      "swapInfo": {
        "ammKey": "string",
        "label": "string",
        "inputMint": "string",
        "outputMint": "string",
        "inAmount": "string",
        "outAmount": "string",
        "feeAmount": "string | null",
        "feeMint": "string | null"
      },
      "percent": 100,
      "bps": 0
    }
  ],
  "contextSlot": 0,
  "timeTaken": 0.0,
  "mostReliableAmmsQuoteReport": null
}
```

**Key fields:**
- `outAmount`: Best output amount after fees; does NOT include slippage effects
- `otherAmountThreshold`: Minimum acceptable output with slippage applied
- `routePlan`: Array of DEX hops used to construct the route

---

#### Step 2 — Build Swap Transaction

```
POST https://api.jup.ag/swap/v1/swap
```

**Required headers:**
- `Content-Type: application/json`

**Required body fields:**

| Field | Type | Description |
|---|---|---|
| `userPublicKey` | string | User's wallet public key (base58) |
| `quoteResponse` | object | Complete response object from `/quote` |

**Optional body fields:**

| Field | Type | Description |
|---|---|---|
| `payer` | string | Account covering transaction fees and token account rent |
| `wrapAndUnwrapSol` | boolean | Auto-wrap/unwrap SOL (default: true) |
| `feeAccount` | string | Initialized token account for fee collection |
| `prioritizationFeeLamports` | object | Priority fee spec; accepts `"auto"` or lamport value |
| `dynamicComputeUnitLimit` | boolean | Enable simulation for accurate CU estimation |
| `destinationTokenAccount` | string | Custom token account for receiving output |
| `nativeDestinationAccount` | string | Account for receiving native SOL output |
| `asLegacyTransaction` | boolean | Build legacy instead of versioned transaction |

**Response (200):**
```json
{
  "swapTransaction": "<base64-encoded unsigned transaction>",
  "lastValidBlockHeight": 0,
  "prioritizationFeeLamports": 0
}
```

After receiving: deserialize → sign with wallet → broadcast via own RPC.

**Swap Instructions variant:**
```
POST https://api.jup.ag/swap/v1/swap-instructions
```
Returns individual Solana instructions instead of a pre-built transaction, for use in custom transaction composition.

---

## 2. Trigger Orders (Limit Orders)

Jupiter's limit/trigger orders allow execution when price conditions are met. The API is called "Trigger API". Orders remain on-chain until filled or canceled.

### 2A. Create Trigger Order

```
POST https://api.jup.ag/trigger/v1/createOrder
```

**Required headers:**
- `Content-Type: application/json`
- `x-api-key: <your-api-key>`

**Request body:**

| Field | Type | Required | Description |
|---|---|---|---|
| `inputMint` | string | Yes | Source token mint address |
| `outputMint` | string | Yes | Destination token mint address |
| `maker` | string | Yes | Wallet address creating the order |
| `payer` | string | Yes | Wallet address paying transaction fees |
| `makingAmount` | string | Yes | Amount of input tokens (smallest unit, as string) |
| `takingAmount` | string | Yes | Amount of output tokens expected (smallest unit, as string) |
| `slippageBps` | number | No | Slippage tolerance in basis points |
| `expiredAt` | number | No | Unix timestamp for order expiration |
| `feeBps` | number | No | Referral fee in basis points |
| `feeAccount` | string | No | Referral token account for output mint |
| `computeUnitPrice` | string | No | `"auto"` or numeric lamports value |
| `wrapAndUnwrapSol` | boolean | No | Auto-wrap/unwrap SOL (default: true) |

**Implied price:** `takingAmount / makingAmount` defines the limit price. The order executes when the market price reaches this ratio.

**Response (200):**
```json
{
  "order": "<order account ID>",
  "transaction": "<base64-encoded unsigned transaction>",
  "requestId": "<UUID>"
}
```

**Error response:**
```json
{
  "error": "string",
  "cause": "string",
  "code": 400
}
```

---

### 2B. Execute Trigger Order (on-chain submission)

```
POST https://api.jup.ag/trigger/v1/execute
```

**Required headers:**
- `Content-Type: application/json`
- `x-api-key: <your-api-key>`

**Request body:**
```json
{
  "signedTransaction": "<base64-encoded signed transaction>",
  "requestId": "<UUID from createOrder response>"
}
```

**Response (success):**
```json
{
  "status": "Success",
  "signature": "<base58 transaction signature>"
}
```

**Response (failure):**
```json
{
  "status": "Failed",
  "error": "custom program error code: 1",
  "code": 500,
  "signature": "<base58 transaction signature>"
}
```

---

### 2C. Cancel Single Trigger Order

```
POST https://api.jup.ag/trigger/v1/cancelOrder
```

**Required headers:**
- `x-api-key: <your-api-key>`

**Request body:**
```json
{
  "maker": "<wallet public key>",
  "order": "<order account ID>",
  "computeUnitPrice": "auto"
}
```

**Response (200):**
```json
{
  "transaction": "<base64-encoded transaction>",
  "requestId": "<UUID>"
}
```

**Error response:**
```json
{
  "error": "string",
  "code": 400
}
```

---

### 2D. Cancel Multiple Trigger Orders

```
POST https://api.jup.ag/trigger/v1/cancelOrders
```

**Request body:**
```json
{
  "maker": "<wallet public key>",
  "orders": ["<order_id_1>", "<order_id_2>"],
  "computeUnitPrice": "auto"
}
```

Note: If `orders` array is omitted, ALL open orders for `maker` are canceled.

**Response (200):**
```json
{
  "transactions": ["<base64>", "<base64>"],
  "requestId": "<UUID>"
}
```

Orders are batched in groups of 5; multiple transactions may be returned.

---

### 2E. Get Trigger Orders

```
GET https://api.jup.ag/trigger/v1/getTriggerOrders
```

**Required headers:**
- `x-api-key: <your-api-key>`

**Query parameters:**

| Parameter | Type | Required | Description |
|---|---|---|---|
| `user` | string | Yes | Wallet address to query |
| `orderStatus` | string | Yes | `active` or `history` |
| `page` | number | No | Pagination (10 orders per page) |
| `inputMint` | string | No | Filter by input token mint |
| `outputMint` | string | No | Filter by output token mint |

**Response (200):**
```json
{
  "orders": [ ... ],
  "hasMoreData": false
}
```

`hasMoreData: true` means more pages exist. Increment `page` to fetch next set.

**Important:** This endpoint uses a different data format than the deprecated `openOrders`/`orderHistory` endpoints from previous API versions.

---

## 3. Recurring Orders (DCA — Dollar Cost Averaging)

The Recurring API executes automatic periodic swaps. Fee: 0.1% per execution. Integrator fees are not currently supported.

Supports two order types:
- **Time-based**: Execute a fixed amount every N seconds (active, recommended)
- **Price-based**: Execute based on price thresholds (DEPRECATED)

### 3A. Create Recurring Order

```
POST https://api.jup.ag/recurring/v1/createOrder
```

**Required headers:**
- `Content-Type: application/json`
- `x-api-key: <your-api-key>`

**Request body (time-based):**
```json
{
  "user": "<wallet public key>",
  "inputMint": "<source token mint>",
  "outputMint": "<destination token mint>",
  "params": {
    "time": {
      "inAmount": 1000000000,
      "numberOfOrders": 10,
      "interval": 86400,
      "minPrice": null,
      "maxPrice": null,
      "startAt": null
    }
  }
}
```

**Time-based params fields:**

| Field | Type | Description |
|---|---|---|
| `inAmount` | number | Total raw input amount to deposit (pre-decimals) |
| `numberOfOrders` | number | Total number of executions |
| `interval` | number | Seconds between each execution |
| `minPrice` | number\|null | Optional minimum price threshold |
| `maxPrice` | number\|null | Optional maximum price threshold |
| `startAt` | number\|null | Unix timestamp for start; null = start immediately |

**Formula:** Amount per cycle = `inAmount / numberOfOrders`
Example: 1,000 USDC / 10 orders = 100 USDC per order

**Request body (price-based, DEPRECATED):**
```json
{
  "user": "<wallet public key>",
  "inputMint": "<source token mint>",
  "outputMint": "<destination token mint>",
  "params": {
    "price": {
      "depositAmount": 1000000000,
      "incrementUsdcValue": 100000000,
      "interval": 86400,
      "startAt": null
    }
  }
}
```

**Response (200, both order types):**
```json
{
  "requestId": "<UUID>",
  "transaction": "<base64-encoded unsigned transaction>"
}
```

**Error response:**
```json
{
  "code": 400,
  "error": "Error description",
  "status": "Status message"
}
```

---

### 3B. Cancel Recurring Order

```
POST https://api.jup.ag/recurring/v1/cancelOrder
```

**Required headers:**
- `x-api-key: <your-api-key>`

**Request body:**
```json
{
  "order": "<order account address>",
  "user": "<wallet public key>",
  "recurringType": "time"
}
```

**Limitation:** Supports only 1 cancellation per transaction.

**Response:** Returns `requestId` and base64-encoded transaction for signing.

---

### 3C. Get Recurring Orders

```
GET https://api.jup.ag/recurring/v1/getRecurringOrders
```

**Required headers:**
- `x-api-key: <your-api-key>`

**Query parameters:**

| Parameter | Type | Required | Description |
|---|---|---|---|
| `user` | string | Yes | Wallet public key |
| `orderStatus` | string | Yes | `active` or `history` |
| `recurringType` | string | Yes | `time` (price-based deprecated) |
| `includeFailedTx` | boolean | No | Include failed transactions in result |
| `page` | number | No | Pagination (10 orders per page) |
| `inputMint` | string | No | Filter by input token mint |
| `outputMint` | string | No | Filter by output token mint |

**Response:** Paginated list of recurring order objects.

---

### 3D. Deposit into Price-Based Order (DEPRECATED)

```
POST https://api.jup.ag/recurring/v1/deposit
```

Used to add funds to an existing price-based recurring order.

---

### 3E. Withdraw from Price-Based Order (DEPRECATED)

```
POST https://api.jup.ag/recurring/v1/withdraw
```

Used to remove funds from an existing price-based recurring order.

---

## 4. Jupiter Perpetuals

Jupiter Perps is a leveraged perpetuals trading platform on Solana. The REST API is documented as **still a work in progress** as of early 2026. Position management is done via direct on-chain program interaction, not via a REST endpoint.

### 4A. Architecture

Jupiter Perps operates via the **keeper model**:
1. Trader submits a request transaction to the Solana chain
2. Jupiter keeper monitors and executes it as a separate transaction

This is fundamentally different from REST-based order placement.

### 4B. On-Chain Program Access

| Property | Value |
|---|---|
| Program type | Solana Anchor program |
| Access method | Direct Solana RPC + Anchor IDL |
| Official REST API status | Work in progress, not yet available |
| Community SDK reference | https://github.com/julianfssen/jupiter-perps-anchor-idl-parsing |
| C# SDK | https://github.com/Bifrost-Technologies/Solnet.JupiterPerps |

### 4C. Supported Assets

- SOL
- ETH (wETH)
- wBTC

### 4D. Trading Parameters

| Parameter | Value |
|---|---|
| Max leverage | Up to 100x (long/short on SOL, ETH, wBTC) |
| Position types | Long, Short |
| Order types | Market orders, Take-Profit (TP), Stop-Loss (SL) |
| Collateral tokens | SOL, ETH, wBTC, USDC, USDT (via JLP pool) |

### 4E. PositionRequest Account (on-chain data structure)

The `PositionRequest` account is a Program Derived Address (PDA) representing a request to open, close, or modify a position.

**Derivation:** From the underlying Position address + constant seeds + random integer seed

**Key fields:**

| Field | Type | Description |
|---|---|---|
| `owner` | Pubkey | Trader account |
| `pool` | Pubkey | JLP liquidity pool |
| `custody` | Pubkey | Token custody account |
| `sizeUsdDelta` | u64 | Position size change in atomic units |
| `collateralDelta` | u64 | Collateral change in atomic units |
| `requestChange` | enum | Increase or Decrease |
| `requestType` | enum | Market or Trigger (TP/SL) |
| `side` | enum | Long or Short |
| `priceSlippage` | u64 | Max acceptable price deviation |
| `triggerPrice` | u64 | TP/SL trigger price |
| `triggerAboveThreshold` | bool | Direction of trigger condition |
| `entirePosition` | bool | Full close vs. partial reduce |
| `executed` | bool | Execution status flag |
| `counter` | u64 | PDA derivation seed |
| `bump` | u8 | PDA bump seed |

**Lifecycle:**
- Market orders and deposits: Closed immediately after execution or rejection
- TP/SL requests: Remain on-chain until trigger condition is met

### 4F. Available On-Chain Instructions (via Solnet SDK enumeration)

- `IncreasePosition` (market open/increase)
- `DecreasePosition` (market close/reduce)
- `InstantCreateLimitOrder` (TP/SL placement)
- Collateral deposit/withdraw
- Liquidation

Position data (pool state, custody state, open positions, oracle prices) is all readable from the Solana blockchain via standard RPC `getAccountInfo` calls using Anchor deserialization.

---

## API Base URLs Summary

| API | Production URL | Lite URL (no-key) |
|---|---|---|
| Ultra Swap | `https://api.jup.ag/ultra/v1/` | `https://lite-api.jup.ag/ultra/v1/` |
| Metis Swap (Quote) | `https://api.jup.ag/swap/v1/` | `https://lite-api.jup.ag/swap/v1/` |
| Trigger (Limit) | `https://api.jup.ag/trigger/v1/` | `https://lite-api.jup.ag/trigger/v1/` |
| Recurring (DCA) | `https://api.jup.ag/recurring/v1/` | `https://lite-api.jup.ag/recurring/v1/` |
| Perps | On-chain Anchor program | N/A |

---

## Sources

- [Ultra Swap API Overview](https://dev.jup.ag/docs/ultra)
- [Get Order Endpoint](https://dev.jup.ag/docs/ultra-api/get-order)
- [Execute Order Endpoint](https://dev.jup.ag/docs/ultra-api/execute-order)
- [Get Quote Endpoint](https://dev.jup.ag/api-reference/swap/quote)
- [Build Swap Transaction](https://dev.jup.ag/docs/swap/build-swap-transaction)
- [Trigger API - Create Order](https://dev.jup.ag/docs/trigger-api/create-order)
- [Trigger API - Cancel Order](https://dev.jup.ag/docs/trigger-api/cancel-order)
- [Trigger API - Execute Order](https://dev.jup.ag/docs/trigger-api/execute-order)
- [Trigger API - Get Trigger Orders](https://dev.jup.ag/docs/trigger-api/get-trigger-orders)
- [Recurring API Overview](https://dev.jup.ag/docs/recurring)
- [Recurring API - Create Order](https://dev.jup.ag/docs/recurring-api/create-order)
- [About Perps API](https://dev.jup.ag/docs/perps)
- [PositionRequest Account](https://dev.jup.ag/docs/perp-api/position-request-account)
- [Jupiter Perps IDL Parsing SDK](https://github.com/julianfssen/jupiter-perps-anchor-idl-parsing)
- [Solnet.JupiterPerps C# SDK](https://github.com/Bifrost-Technologies/Solnet.JupiterPerps)
