# Uniswap API Response Formats

## Overview

This document details all response formats for Uniswap APIs:
- Trading API (REST)
- The Graph Subgraph (GraphQL)
- Smart Contract returns
- WebSocket events

---

## 1. Trading API Responses (REST/JSON)

### 1.1 Quote Response

**Endpoint:** `POST /quote`

**Success Response (200 OK):**

```json
{
  "requestId": "550e8400-e29b-41d4-a716-446655440000",
  "quote": {
    "encodedOrder": "0x000000000000000000000000...",
    "orderId": "0xabcd...",
    "orderInfo": {
      "chainId": 1,
      "nonce": "1234567890",
      "reactor": "0x00000011F84B9aa48e5f8aA8B9897600006289Be",
      "swapper": "0x1234567890123456789012345678901234567890",
      "deadline": 1735689600,
      "exclusiveFiller": "0x0000000000000000000000000000000000000000",
      "exclusivityOverrideBps": "0",
      "input": {
        "startAmount": "1000000000000000000",
        "endAmount": "1000000000000000000",
        "token": "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
      },
      "outputs": [
        {
          "startAmount": "1000000000",
          "endAmount": "995000000",
          "token": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
          "recipient": "0x1234567890123456789012345678901234567890"
        }
      ],
      "additionalValidationContract": "0x0000000000000000000000000000000000000000",
      "additionalValidationData": "0x",
      "decayStartTime": 1735686000,
      "decayEndTime": 1735689600
    },
    "portionBips": 0,
    "portionAmount": "0",
    "portionRecipient": "0x0000000000000000000000000000000000000000",
    "quoteId": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
    "slippageTolerance": 0.5,
    "classicGasUseEstimateUSD": "2.50",
    "aggregatedOutputs": [
      {
        "token": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
        "amount": "1000000000",
        "recipient": "0x1234567890123456789012345678901234567890",
        "bps": 10000,
        "minAmount": "995000000"
      }
    ]
  },
  "routing": "DUTCH_V2",
  "permitData": {
    "domain": {
      "name": "Permit2",
      "chainId": 1,
      "verifyingContract": "0x000000000022D473030F116dDEE9F6B43aC78BA3"
    },
    "types": {
      "PermitSingle": [
        { "name": "details", "type": "PermitDetails" },
        { "name": "spender", "type": "address" },
        { "name": "sigDeadline", "type": "uint256" }
      ],
      "PermitDetails": [
        { "name": "token", "type": "address" },
        { "name": "amount", "type": "uint160" },
        { "name": "expiration", "type": "uint48" },
        { "name": "nonce", "type": "uint48" }
      ]
    },
    "values": {
      "details": {
        "token": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
        "amount": "1461501637330902918203684832716283019655932542975",
        "expiration": 1735689600,
        "nonce": 0
      },
      "spender": "0x3fC91A3afd70395Cd496C647d5a6CC9D4B2b7FAD",
      "sigDeadline": 1735689600
    }
  },
  "permitTransaction": null,
  "permitGasFee": "0"
}
```

**Field Descriptions:**

| Field | Type | Description |
|-------|------|-------------|
| `requestId` | string (UUID) | Unique request identifier |
| `quote.quoteId` | string (UUID) | Quote identifier for tracking |
| `quote.portionBips` | number | Service fee in basis points (100 = 1%) |
| `quote.portionAmount` | string | Fee amount in output token |
| `quote.slippageTolerance` | number | Applied slippage tolerance (%) |
| `quote.classicGasUseEstimateUSD` | string | Estimated gas cost in USD |
| `quote.aggregatedOutputs[].token` | string | Output token address |
| `quote.aggregatedOutputs[].amount` | string | Expected output amount |
| `quote.aggregatedOutputs[].minAmount` | string | Minimum output after slippage |
| `routing` | enum | Routing type used |
| `permitData` | object | EIP-712 signature data (if requested) |

**Routing Types:**
- `DUTCH_LIMIT` - Dutch auction limit order
- `DUTCH_V2` - Dutch auction V2
- `DUTCH_V3` - Dutch auction V3
- `CLASSIC` - Traditional AMM swap
- `BRIDGE` - Cross-chain bridge
- `LIMIT_ORDER` - Limit order
- `PRIORITY` - Priority order
- `WRAP` - ETH wrapping
- `UNWRAP` - ETH unwrapping
- `CHAINED` - Multi-step operation

---

### 1.2 Swap Response

**Endpoint:** `POST /swap`

**Success Response (200 OK):**

```json
{
  "requestId": "550e8400-e29b-41d4-a716-446655440000",
  "transaction": {
    "to": "0x3fC91A3afd70395Cd496C647d5a6CC9D4B2b7FAD",
    "from": "0x1234567890123456789012345678901234567890",
    "data": "0x3593564c000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000006790a8c0...",
    "value": "0",
    "chainId": 1,
    "gasLimit": "200000",
    "maxFeePerGas": "50000000000",
    "maxPriorityFeePerGas": "2000000000",
    "gasPrice": null
  },
  "quote": {
    "quoteId": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
    "slippageTolerance": 0.5,
    "aggregatedOutputs": [
      {
        "token": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
        "amount": "1000000000",
        "minAmount": "995000000"
      }
    ]
  }
}
```

**Field Descriptions:**

| Field | Type | Description |
|-------|------|-------------|
| `transaction.to` | string | Contract address to call |
| `transaction.from` | string | Sender address |
| `transaction.data` | string (hex) | Encoded function call |
| `transaction.value` | string | ETH amount to send (wei) |
| `transaction.chainId` | number | Blockchain network ID |
| `transaction.gasLimit` | string | Maximum gas units |
| `transaction.maxFeePerGas` | string | Max gas price (wei) |
| `transaction.maxPriorityFeePerGas` | string | Miner tip (wei) |

---

### 1.3 Check Approval Response

**Endpoint:** `POST /check_approval`

**Success Response (200 OK):**

```json
{
  "isApproved": true,
  "allowance": "115792089237316195423570985008687907853269984665640564039457584007913129639935"
}
```

Or if not approved:

```json
{
  "isApproved": false,
  "allowance": "0"
}
```

---

### 1.4 Order Status Response

**Endpoint:** `GET /orders?orderHash={hash}`

**Success Response (200 OK):**

```json
{
  "orders": [
    {
      "orderHash": "0xabcdef1234567890...",
      "status": "FILLED",
      "createdAt": 1735680000,
      "filledAt": 1735680120,
      "txHash": "0x123456789abcdef..."
    }
  ]
}
```

**Order Statuses:**
- `OPEN` - Order created, waiting for fill
- `FILLED` - Order successfully filled
- `CANCELLED` - Order cancelled by user
- `EXPIRED` - Order deadline passed
- `INSUFFICIENT_FUNDS` - Not enough balance

---

### 1.5 Swap Status Response

**Endpoint:** `GET /swaps?transactionHash={hash}`

**Success Response (200 OK):**

```json
{
  "swaps": [
    {
      "transactionHash": "0x123456789abcdef...",
      "status": "CONFIRMED",
      "chainId": 1,
      "blockNumber": 19000000,
      "timestamp": 1735680000
    }
  ]
}
```

**Swap Statuses:**
- `PENDING` - Transaction submitted, not confirmed
- `CONFIRMED` - Transaction confirmed on-chain
- `FAILED` - Transaction reverted
- `UNKNOWN` - Status not available

---

### 1.6 Error Responses

**400 Bad Request:**
```json
{
  "error": "VALIDATION_ERROR",
  "message": "Invalid token address",
  "details": {
    "field": "tokenIn",
    "value": "0xinvalid",
    "reason": "Address checksum failed"
  }
}
```

**401 Unauthorized:**
```json
{
  "error": "UNAUTHORIZED",
  "message": "Invalid or missing API key"
}
```

**404 Not Found:**
```json
{
  "error": "NOT_FOUND",
  "message": "Order not found",
  "orderHash": "0x..."
}
```

**429 Too Many Requests:**
```json
{
  "error": "RATE_LIMIT_EXCEEDED",
  "message": "Rate limit exceeded. Try again in 10 seconds.",
  "retryAfter": 10
}
```

**500 Internal Server Error:**
```json
{
  "error": "INTERNAL_ERROR",
  "message": "An unexpected error occurred",
  "requestId": "550e8400-e29b-41d4-a716-446655440000"
}
```

**504 Gateway Timeout:**
```json
{
  "error": "TIMEOUT",
  "message": "Request timed out after 30 seconds"
}
```

---

## 2. The Graph Subgraph Responses (GraphQL)

### 2.1 Pool Query Response

**Query:**
```graphql
{
  pools(first: 2, orderBy: totalValueLockedUSD, orderDirection: desc) {
    id
    token0 { id symbol name decimals }
    token1 { id symbol name decimals }
    feeTier
    liquidity
    sqrtPrice
    tick
    volumeUSD
    totalValueLockedUSD
  }
}
```

**Response:**
```json
{
  "data": {
    "pools": [
      {
        "id": "0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640",
        "token0": {
          "id": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
          "symbol": "USDC",
          "name": "USD Coin",
          "decimals": "6"
        },
        "token1": {
          "id": "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
          "symbol": "WETH",
          "name": "Wrapped Ether",
          "decimals": "18"
        },
        "feeTier": "500",
        "liquidity": "12345678901234567890",
        "sqrtPrice": "1234567890123456789012345678",
        "tick": "-197320",
        "volumeUSD": "123456789.50",
        "totalValueLockedUSD": "98765432.10"
      },
      {
        "id": "0x8ad599c3a0ff1de082011efddc58f1908eb6e6d8",
        "token0": {
          "id": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
          "symbol": "USDC",
          "name": "USD Coin",
          "decimals": "6"
        },
        "token1": {
          "id": "0xdac17f958d2ee523a2206206994597c13d831ec7",
          "symbol": "USDT",
          "name": "Tether USD",
          "decimals": "6"
        },
        "feeTier": "100",
        "liquidity": "987654321098765432",
        "sqrtPrice": "79228162514264337593543950336",
        "tick": "0",
        "volumeUSD": "87654321.00",
        "totalValueLockedUSD": "65432109.87"
      }
    ]
  }
}
```

**Field Types:**

| Field | Type | Description |
|-------|------|-------------|
| `id` | string (hex) | Pool contract address |
| `feeTier` | string | Fee in hundredths of bips (500 = 0.05%) |
| `liquidity` | string (decimal) | Total liquidity in pool |
| `sqrtPrice` | string (decimal) | Square root price Q64.96 format |
| `tick` | string (int) | Current active tick |
| `volumeUSD` | string (decimal) | Total volume in USD |
| `totalValueLockedUSD` | string (decimal) | TVL in USD |

---

### 2.2 Swap Query Response

**Query:**
```graphql
{
  swaps(first: 3, orderBy: timestamp, orderDirection: desc) {
    id
    transaction { id timestamp }
    sender
    recipient
    amount0
    amount1
    amountUSD
    token0 { symbol decimals }
    token1 { symbol decimals }
  }
}
```

**Response:**
```json
{
  "data": {
    "swaps": [
      {
        "id": "0x1234...#123",
        "transaction": {
          "id": "0x123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
          "timestamp": "1735680000"
        },
        "sender": "0x1111111111111111111111111111111111111111",
        "recipient": "0x2222222222222222222222222222222222222222",
        "amount0": "-1000000000",
        "amount1": "500000000000000000",
        "amountUSD": "1000.50",
        "token0": {
          "symbol": "USDC",
          "decimals": "6"
        },
        "token1": {
          "symbol": "WETH",
          "decimals": "18"
        }
      },
      {
        "id": "0x5678...#456",
        "transaction": {
          "id": "0xabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
          "timestamp": "1735679940"
        },
        "sender": "0x3333333333333333333333333333333333333333",
        "recipient": "0x4444444444444444444444444444444444444444",
        "amount0": "2000000000000000000",
        "amount1": "-4000000000",
        "amountUSD": "4000.00",
        "token0": {
          "symbol": "WETH",
          "decimals": "18"
        },
        "token1": {
          "symbol": "USDC",
          "decimals": "6"
        }
      }
    ]
  }
}
```

**Field Descriptions:**

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Unique swap ID (txHash#logIndex) |
| `transaction.id` | string (hex) | Transaction hash |
| `transaction.timestamp` | string (unix) | Block timestamp |
| `sender` | string (hex) | Swap initiator address |
| `recipient` | string (hex) | Token recipient address |
| `amount0` | string (decimal) | Token0 delta (negative = sent) |
| `amount1` | string (decimal) | Token1 delta (positive = received) |
| `amountUSD` | string (decimal) | USD value of swap |

---

### 2.3 Token Query Response

**Query:**
```graphql
{
  tokens(first: 2, orderBy: volumeUSD, orderDirection: desc) {
    id
    symbol
    name
    decimals
    volumeUSD
    totalValueLockedUSD
    poolCount
    txCount
  }
}
```

**Response:**
```json
{
  "data": {
    "tokens": [
      {
        "id": "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
        "symbol": "WETH",
        "name": "Wrapped Ether",
        "decimals": "18",
        "volumeUSD": "9876543210.50",
        "totalValueLockedUSD": "123456789.00",
        "poolCount": "1234",
        "txCount": "5678901"
      },
      {
        "id": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
        "symbol": "USDC",
        "name": "USD Coin",
        "decimals": "6",
        "volumeUSD": "8765432109.75",
        "totalValueLockedUSD": "98765432.10",
        "poolCount": "987",
        "txCount": "4567890"
      }
    ]
  }
}
```

---

### 2.4 Factory Query Response

**Query:**
```graphql
{
  factory(id: "0x1F98431c8aD98523631AE4a59f267346ea31F984") {
    poolCount
    txCount
    totalVolumeUSD
    totalVolumeETH
    totalFeesUSD
    totalValueLockedUSD
  }
}
```

**Response:**
```json
{
  "data": {
    "factory": {
      "poolCount": "12345",
      "txCount": "98765432",
      "totalVolumeUSD": "987654321098.50",
      "totalVolumeETH": "123456789.123456789",
      "totalFeesUSD": "9876543210.98",
      "totalValueLockedUSD": "9876543210.50"
    }
  }
}
```

---

### 2.5 Position Query Response

**Query:**
```graphql
{
  positions(first: 2, where: { owner: "0x..." }) {
    id
    owner
    liquidity
    token0 { symbol }
    token1 { symbol }
    tickLower { tickIdx }
    tickUpper { tickIdx }
    collectedFeesToken0
    collectedFeesToken1
  }
}
```

**Response:**
```json
{
  "data": {
    "positions": [
      {
        "id": "12345",
        "owner": "0x1234567890123456789012345678901234567890",
        "liquidity": "1234567890123456",
        "token0": {
          "symbol": "USDC"
        },
        "token1": {
          "symbol": "WETH"
        },
        "tickLower": {
          "tickIdx": "-887220"
        },
        "tickUpper": {
          "tickIdx": "887220"
        },
        "collectedFeesToken0": "123456789",
        "collectedFeesToken1": "987654321098765432"
      }
    ]
  }
}
```

---

### 2.6 GraphQL Error Response

**Query Error:**
```json
{
  "errors": [
    {
      "message": "Cannot query field \"invalidField\" on type \"Pool\".",
      "locations": [
        {
          "line": 3,
          "column": 5
        }
      ]
    }
  ]
}
```

**Rate Limit Error:**
```json
{
  "errors": [
    {
      "message": "Rate limit exceeded"
    }
  ]
}
```

---

## 3. Smart Contract Responses

### 3.1 slot0() Response

**Function:** Get pool state

**Returns:**
```solidity
(
  uint160 sqrtPriceX96,           // 1234567890123456789012345678
  int24 tick,                      // -197320
  uint16 observationIndex,         // 123
  uint16 observationCardinality,   // 1000
  uint16 observationCardinalityNext, // 1000
  uint8 feeProtocol,               // 0
  bool unlocked                    // true
)
```

**JSON RPC Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": "0x0000000000000000000000000000000000000000004e5f42b4b91a2bfffba60000000000000000000000000000000000000000000000000000000000fffcfd8800000000000000000000000000000000000000000000000000000000000000007b00000000000000000000000000000000000000000000000000000000000003e800000000000000000000000000000000000000000000000000000000000003e8000000000000000000000000000000000000000000000000000000000000000001"
}
```

---

### 3.2 liquidity() Response

**Function:** Get total pool liquidity

**Returns:**
```solidity
uint128 liquidity  // 12345678901234567890
```

**JSON RPC Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": "0x00000000000000000000000000000000000000000000000000ab54a98ceb1f0ea"
}
```

---

### 3.3 quoteExactInputSingle() Response

**Function:** Get swap quote from Quoter contract

**Returns:**
```solidity
(
  uint256 amountOut,                 // 1000000000
  uint160 sqrtPriceX96After,         // 1234567890123456789012345678
  uint32 initializedTicksCrossed,    // 5
  uint256 gasEstimate                // 120000
)
```

**Rust Parsing:**
```rust
struct QuoteResult {
    amount_out: U256,
    sqrt_price_x96_after: U256,
    initialized_ticks_crossed: u32,
    gas_estimate: U256,
}
```

---

### 3.4 Token Balance Response

**Function:** `balanceOf(address)`

**Returns:**
```solidity
uint256 balance  // 1000000000000000000
```

**JSON RPC Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": "0x0de0b6b3a7640000"
}
```

---

## 4. WebSocket Event Responses

### 4.1 New Block Header

**Subscription:** `eth_subscribe("newHeads")`

**Response:**
```json
{
  "jsonrpc": "2.0",
  "method": "eth_subscription",
  "params": {
    "subscription": "0x1234567890abcdef",
    "result": {
      "number": "0x121a7b0",
      "hash": "0x123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
      "parentHash": "0xabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
      "timestamp": "0x679e3e40",
      "gasLimit": "0x1c9c380",
      "gasUsed": "0xf4240",
      "baseFeePerGas": "0xba43b7400"
    }
  }
}
```

---

### 4.2 Swap Event Log

**Event Signature:**
```solidity
event Swap(
    address indexed sender,
    address indexed recipient,
    int256 amount0,
    int256 amount1,
    uint160 sqrtPriceX96,
    uint128 liquidity,
    int24 tick
);
```

**Log Response:**
```json
{
  "address": "0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640",
  "topics": [
    "0xc42079f94a6350d7e6235f29174924f928cc2ac818eb64fed8004e115fbcca67",
    "0x000000000000000000000000e592427a0aece92de3edee1f18e0157c05861564",
    "0x0000000000000000000000001234567890123456789012345678901234567890"
  ],
  "data": "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffc465e60000000000000000000000000000000000000000000000000000016345785d8a000000000000000000000000000000000000000000004e5f42b4b91a2bfffba60000000000000000000000000000000000000000000000000000ab54a98ceb1f0eafffffffffffffffffffffffffffffffffffffffffffffffffffffffffffcfd88",
  "blockNumber": "0x121a7b0",
  "transactionHash": "0x123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
  "transactionIndex": "0x5a",
  "blockHash": "0xabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
  "logIndex": "0x12",
  "removed": false
}
```

---

## 5. Data Type Conversion

### 5.1 Token Amounts

**On-Chain:** Smallest unit (uint256)
```
1 ETH = 1000000000000000000 wei
1 USDC = 1000000 (6 decimals)
```

**API:** String representation
```json
{
  "amount": "1000000000000000000"  // 1 ETH
}
```

**Rust Conversion:**
```rust
use ethers::types::U256;

let wei = U256::from_dec_str("1000000000000000000")?;
let eth = wei / U256::from(10).pow(U256::from(18));
```

---

### 5.2 Addresses

**Format:** Checksummed hex string
```
0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2
```

**Rust Type:**
```rust
use alloy::primitives::Address;

let addr: Address = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".parse()?;
```

---

### 5.3 Timestamps

**Unix Timestamp (seconds):**
```json
{
  "timestamp": 1735680000
}
```

**Rust Conversion:**
```rust
use chrono::{DateTime, Utc};

let dt = DateTime::from_timestamp(1735680000, 0)?;
```

---

## Summary

| API Type | Format | Root Key | Pagination |
|----------|--------|----------|------------|
| Trading API | JSON | `quote`, `transaction` | N/A |
| Subgraph | JSON | `data` | `first`, `skip` |
| Smart Contracts | Hex bytes | N/A (decoded) | N/A |
| WebSocket | JSON | `result` | Stream |

**Key Points:**
1. All amounts are strings to avoid precision loss
2. Addresses are checksummed hex strings
3. Timestamps are Unix seconds (not milliseconds)
4. GraphQL always wraps in `data` object
5. Errors use standard HTTP status codes + JSON body
