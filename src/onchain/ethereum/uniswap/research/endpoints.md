# Uniswap API Endpoints

## Overview

Uniswap is a decentralized exchange (DEX) on Ethereum and other EVM chains. It provides multiple API interfaces:
- **REST API**: Uniswap Labs Trading API (hosted service)
- **GraphQL API**: The Graph subgraph endpoints
- **Smart Contracts**: Direct on-chain interaction via RPC

---

## 1. Uniswap Labs Trading API (REST)

### Base URLs

**Production:**
```
https://trade-api.gateway.uniswap.org/v1
```

**Beta/Testing:**
```
https://beta.trade-api.gateway.uniswap.org/v1
```

### Authentication

All endpoints require API key authentication:
```
Headers:
  x-api-key: <YOUR_API_KEY>
```

Get API key from: [Uniswap Developer Portal](https://api-docs.uniswap.org/)

---

## 2. Swapping Endpoints

### 2.1 Check Approval

**Endpoint:** `POST /check_approval`

**Purpose:** Verify if token is approved for trading

**Request Body:**
```json
{
  "token": "0x...",
  "walletAddress": "0x...",
  "chainId": 1
}
```

**Response:**
```json
{
  "isApproved": true
}
```

---

### 2.2 Get Quote

**Endpoint:** `POST /quote`

**Purpose:** Get quote for swap, bridge, or wrap/unwrap operation

**Request Body:**
```json
{
  "type": "EXACT_INPUT",
  "amount": "1000000000000000000",
  "tokenInChainId": 1,
  "tokenOutChainId": 1,
  "tokenIn": "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
  "tokenOut": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
  "swapper": "0x...",
  "slippageTolerance": 0.5,
  "routingPreference": "BEST_PRICE"
}
```

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `type` | enum | Yes | `"EXACT_INPUT"` or `"EXACT_OUTPUT"` |
| `amount` | string | Yes | Token quantity in smallest unit (wei for ETH) |
| `tokenInChainId` | number | Yes | Chain ID (1=Ethereum, 137=Polygon, etc.) |
| `tokenOutChainId` | number | Yes | Output token chain ID |
| `tokenIn` | string | Yes | Input token address (0x format) |
| `tokenOut` | string | Yes | Output token address (0x format) |
| `swapper` | string | Yes | Wallet address executing swap |
| `slippageTolerance` | number | No | Max slippage % (0-100, 2 decimals) |
| `autoSlippage` | enum | No | `"DEFAULT"` for auto-calculation |
| `routingPreference` | enum | No | `"BEST_PRICE"` or `"FASTEST"` |
| `protocols` | array | No | `["V2", "V3", "V4", "UNISWAPX", "UNISWAPX_V2", "UNISWAPX_V3"]` |
| `hooksOptions` | enum | No | `"V4_HOOKS_INCLUSIVE"`, `"V4_HOOKS_ONLY"`, `"V4_NO_HOOKS"` |
| `urgency` | enum | No | `"normal"`, `"fast"`, `"urgent"` |

**Response:**
```json
{
  "requestId": "uuid",
  "quote": {
    "quoteId": "uuid",
    "portionBips": 0,
    "portionAmount": "0",
    "slippageTolerance": 0.5,
    "classicGasUseEstimateUSD": "2.50",
    "aggregatedOutputs": [
      {
        "token": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
        "amount": "1000000000",
        "recipient": "0x...",
        "minAmount": "995000000"
      }
    ]
  },
  "routing": "DUTCH_V2",
  "permitData": { },
  "permitGasFee": "0"
}
```

---

### 2.3 Execute Swap

**Endpoint:** `POST /swap`

**Purpose:** Get transaction calldata for executing swap

**Request Body:** Similar to `/quote` with additional execution parameters

**Response:**
```json
{
  "transaction": {
    "to": "0x...",
    "from": "0x...",
    "data": "0x...",
    "value": "0",
    "chainId": 1,
    "gasLimit": "200000",
    "maxFeePerGas": "50000000000",
    "maxPriorityFeePerGas": "2000000000"
  }
}
```

---

### 2.4 Get Order Status

**Endpoint:** `GET /orders?orderHash={hash}`

**Purpose:** Check status of UniswapX gasless order

**Response:**
```json
{
  "orders": [
    {
      "orderHash": "0x...",
      "status": "FILLED",
      "filledAt": 1640000000
    }
  ]
}
```

---

### 2.5 Get Swap Status

**Endpoint:** `GET /swaps?transactionHash={hash}`

**Purpose:** Check swap or bridge transaction status

**Response:**
```json
{
  "swaps": [
    {
      "transactionHash": "0x...",
      "status": "CONFIRMED",
      "chainId": 1
    }
  ]
}
```

---

## 3. Liquidity Provisioning Endpoints

### 3.1 Create Pool & Position

**Endpoint:** `POST /lp/create`

**Purpose:** Create new pool and initial liquidity position

**Request Body:**
```json
{
  "chainId": 1,
  "token0": "0x...",
  "token1": "0x...",
  "fee": 3000,
  "tickLower": -887220,
  "tickUpper": 887220,
  "amount0Desired": "1000000000000000000",
  "amount1Desired": "1000000000"
}
```

**Fee Tiers:**
- `100` = 0.01% (stablecoins)
- `500` = 0.05% (low volatility)
- `3000` = 0.30% (standard)
- `10000` = 1.00% (exotic pairs)

---

### 3.2 Increase Position

**Endpoint:** `POST /lp/increase`

**Purpose:** Add liquidity to existing position

---

### 3.3 Decrease Position

**Endpoint:** `POST /lp/decrease`

**Purpose:** Remove liquidity from position

---

### 3.4 Claim Fees

**Endpoint:** `POST /lp/claim`

**Purpose:** Collect earned trading fees

---

## 4. The Graph Subgraph API (GraphQL)

### Base URLs

All endpoints use The Graph's decentralized gateway:
```
https://gateway.thegraph.com/api/<YOUR_API_KEY>/subgraphs/id/<SUBGRAPH_ID>
```

### Subgraph IDs

**Ethereum Mainnet:**
- V4: `DiYPVdygkfjDWhbxGSqAQxwBKmfKnkWQojqeM2rkLb3G`
- V3: `5zvR82QoaXYFyDEKLZ9t6v9adgnptxYpKpSbxtgVENFV`
- V2: `A3Np3RQbaBA6oKJgiwDJeo5T3zrYfGHPWFYayMwtNDum`
- V1: `ESnjgAG9NjfmHypk4Huu4PVvz55fUwpyrRqHF21thoLJ`

**Multi-chain V3 Support:**
- Arbitrum
- Base
- Optimism
- Polygon
- BSC (Binance Smart Chain)
- Avalanche
- Celo
- Blast

### Authentication

API keys required from [The Graph Studio](https://thegraph.com/studio/apikeys/)

---

### 4.1 Query Pools

**GraphQL Query:**
```graphql
{
  pools(
    first: 10
    orderBy: totalValueLockedUSD
    orderDirection: desc
    where: { volumeUSD_gt: "1000000" }
  ) {
    id
    token0 {
      id
      symbol
      name
      decimals
    }
    token1 {
      id
      symbol
      name
      decimals
    }
    feeTier
    liquidity
    sqrtPrice
    tick
    volumeUSD
    totalValueLockedUSD
  }
}
```

---

### 4.2 Query Swaps

**GraphQL Query:**
```graphql
{
  swaps(
    first: 100
    orderBy: timestamp
    orderDirection: desc
    where: { pool: "0x..." }
  ) {
    id
    transaction {
      id
      timestamp
    }
    sender
    recipient
    amount0
    amount1
    amountUSD
    token0 {
      symbol
    }
    token1 {
      symbol
    }
  }
}
```

---

### 4.3 Query Tokens

**GraphQL Query:**
```graphql
{
  tokens(
    first: 20
    orderBy: volumeUSD
    orderDirection: desc
  ) {
    id
    symbol
    name
    decimals
    volumeUSD
    totalValueLockedUSD
    poolCount
  }
}
```

---

### 4.4 Query Factory Stats

**GraphQL Query:**
```graphql
{
  factory(id: "0x1F98431c8aD98523631AE4a59f267346ea31F984") {
    poolCount
    txCount
    totalVolumeUSD
    totalVolumeETH
    totalFeesUSD
  }
}
```

---

### 4.5 Query User Positions

**GraphQL Query:**
```graphql
{
  positions(
    first: 10
    where: { owner: "0x..." }
  ) {
    id
    owner
    liquidity
    token0 {
      symbol
    }
    token1 {
      symbol
    }
    collectedFeesToken0
    collectedFeesToken1
  }
}
```

---

## 5. On-Chain Contract Endpoints (via RPC)

### 5.1 Smart Contract Addresses

**Ethereum Mainnet:**

```
V2 Router:    0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D
V2 Factory:   0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f
V3 Router:    0xE592427A0AEce92De3Edee1F18E0157C05861564
V3 Factory:   0x1F98431c8aD98523631AE4a59f267346ea31F984
V3 Quoter:    0xb27308f9F90D607463BB33eA1BeBb41C27CE5AB6
V4 Universal: 0x66a9893Cc07D91D95644aEDd05d03f95E1DBa8Af
```

---

### 5.2 Pool Contract Methods

**Get Pool State:**
```solidity
function slot0() external view returns (
  uint160 sqrtPriceX96,
  int24 tick,
  uint16 observationIndex,
  uint16 observationCardinality,
  uint16 observationCardinalityNext,
  uint8 feeProtocol,
  bool unlocked
)
```

**Get Liquidity:**
```solidity
function liquidity() external view returns (uint128)
```

**Get Tick Data:**
```solidity
function ticks(int24 tick) external view returns (
  uint128 liquidityGross,
  int128 liquidityNet,
  uint256 feeGrowthOutside0X128,
  uint256 feeGrowthOutside1X128,
  int56 tickCumulativeOutside,
  uint160 secondsPerLiquidityOutsideX128,
  uint32 secondsOutside,
  bool initialized
)
```

---

### 5.3 Quoter Contract Methods

**Single Pool Quote:**
```solidity
function quoteExactInputSingle(
  address tokenIn,
  address tokenOut,
  uint24 fee,
  uint256 amountIn,
  uint160 sqrtPriceLimitX96
) external returns (
  uint256 amountOut,
  uint160 sqrtPriceX96After,
  uint32 initializedTicksCrossed,
  uint256 gasEstimate
)
```

**Multi-Pool Quote:**
```solidity
function quoteExactInput(
  bytes memory path,
  uint256 amountIn
) external returns (
  uint256 amountOut,
  uint160[] memory sqrtPriceX96AfterList,
  uint32[] memory initializedTicksCrossedList,
  uint256 gasEstimate
)
```

---

## 6. Routing API

### GitHub Repository
```
https://github.com/Uniswap/routing-api
https://github.com/Uniswap/unified-routing-api
```

**Purpose:** Finds optimal swap routes using `@uniswap/smart-order-router`

**Features:**
- Multi-hop routing (up to 7 paths)
- Split orders across multiple pools
- Gas cost optimization
- Support for V2, V3, V4, and UniswapX

---

## 7. Reference Data Endpoints

### 7.1 Get Swappable Tokens

**Endpoint:** `GET /swappable_tokens`

**Purpose:** List all tokens available for swapping on supported chains

**Response:**
```json
{
  "tokens": [
    {
      "chainId": 1,
      "address": "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
      "symbol": "WETH",
      "name": "Wrapped Ether",
      "decimals": 18
    }
  ]
}
```

---

## Summary Table

| API Type | Use Case | Authentication | Format |
|----------|----------|----------------|--------|
| Trading API | Swaps, quotes, orders | API key header | REST/JSON |
| Subgraph | Historical data, analytics | API key in URL | GraphQL |
| Smart Contracts | Direct on-chain | Wallet signature | Solidity ABI |
| Routing API | Optimal path finding | API key | REST/JSON |

---

## Notes

1. **Token Amounts:** Always use smallest unit (wei for ETH, 10^decimals for ERC20)
2. **Chain IDs:** 1=Ethereum, 137=Polygon, 42161=Arbitrum, 10=Optimism, etc.
3. **Fee Calculation:** Protocol may charge service fees (see `portionBips` in responses)
4. **Simulation:** All quotes are simulated; failures include `txFailureReason`
5. **Multi-chain:** V3 supports 8+ chains; V4 is Ethereum mainnet only (as of 2026)
