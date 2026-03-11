# GMX Response Formats

This document details all response formats for GMX REST API endpoints and smart contract queries.

## REST API Response Formats

### 1. Ping

**Endpoint:** `GET /{chain}-api.gmxinfra.io/ping`

**Response:**
```json
{
  "status": "ok"
}
```

**Fields:**
- `status` (string): Always "ok" if endpoint is operational

---

### 2. Tickers (Price Display)

**Endpoint:** `GET /{chain}-api.gmxinfra.io/prices/tickers`

**Response:**
```json
{
  "ETH": {
    "minPrice": "2500000000000000000000000000000000",
    "maxPrice": "2501000000000000000000000000000000",
    "timestamp": 1674567890
  },
  "BTC": {
    "minPrice": "40000000000000000000000000000000000",
    "maxPrice": "40010000000000000000000000000000000",
    "timestamp": 1674567890
  },
  "USDC": {
    "minPrice": "1000000000000000000000000000000",
    "maxPrice": "1000000000000000000000000000000",
    "timestamp": 1674567890
  }
}
```

**Field Details:**
- Token symbol (string): Top-level keys (e.g., "ETH", "BTC")
  - `minPrice` (string): Minimum price with 30 decimals precision
  - `maxPrice` (string): Maximum price with 30 decimals precision
  - `timestamp` (integer): Unix timestamp in seconds

**Price Precision:**
- All prices have **30 decimals** of precision
- Example: "2500000000000000000000000000000000" = $2,500.00
- Conversion: `value / 10^30` = USD price

**Min/Max Price Explanation:**
- `minPrice`: Used for calculating position losses (conservative)
- `maxPrice`: Used for calculating position gains (conservative)
- Spread: Typically 0.01% - 0.05% between min and max

---

### 3. Signed Prices

**Endpoint:** `GET /{chain}-api.gmxinfra.io/signed_prices/latest`

**Response:**
```json
{
  "signedPrices": [
    {
      "tokenSymbol": "ETH",
      "tokenAddress": "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1",
      "minPrice": "2500123456789012345678901234567890",
      "maxPrice": "2500234567890123456789012345678901",
      "timestamp": 1674567890,
      "signature": "0x1234567890abcdef..."
    },
    {
      "tokenSymbol": "BTC",
      "tokenAddress": "0x2f2a2543B76A4166549F7aaB2e75Bef0aefC5B0f",
      "minPrice": "40000123456789012345678901234567890",
      "maxPrice": "40001234567890123456789012345678901",
      "timestamp": 1674567890,
      "signature": "0xabcdef1234567890..."
    }
  ],
  "updatedAt": 1674567890
}
```

**Field Details:**
- `signedPrices` (array): Array of signed price objects
  - `tokenSymbol` (string): Token symbol (e.g., "ETH", "BTC")
  - `tokenAddress` (string): Token contract address (hex)
  - `minPrice` (string): Minimum price with 30 decimals
  - `maxPrice` (string): Maximum price with 30 decimals
  - `timestamp` (integer): Unix timestamp when price was signed
  - `signature` (string): ECDSA signature (hex, 65 bytes)
- `updatedAt` (integer): Unix timestamp of response generation

**Signature Format:**
- 65 bytes hex string (130 characters + "0x")
- ECDSA signature: `r (32 bytes) + s (32 bytes) + v (1 byte)`
- Can be verified using `ecrecover` on-chain

---

### 4. Candlesticks (OHLC)

**Endpoint:** `GET /{chain}-api.gmxinfra.io/prices/candles?tokenSymbol=ETH&period=1h&limit=100`

**Response:**
```json
[
  [1674567890, "2503.45", "2508.92", "2501.23", "2505.67"],
  [1674564290, "2498.12", "2504.56", "2495.78", "2503.45"],
  [1674560690, "2492.34", "2499.89", "2490.12", "2498.12"]
]
```

**Array Format:** `[timestamp, open, high, low, close]`

**Field Details:**
- Index 0: `timestamp` (integer) - Unix timestamp in seconds
- Index 1: `open` (string) - Opening price in USD (2 decimal precision)
- Index 2: `high` (string) - Highest price in period
- Index 3: `low` (string) - Lowest price in period
- Index 4: `close` (string) - Closing price

**Ordering:** Descending by timestamp (newest first)

**Supported Periods:**
- `1m` - 1 minute
- `5m` - 5 minutes
- `15m` - 15 minutes
- `1h` - 1 hour
- `4h` - 4 hours
- `1d` - 1 day

**Limit:** 1 to 10,000 candles (default: 1,000)

---

### 5. Tokens List

**Endpoint:** `GET /{chain}-api.gmxinfra.io/tokens`

**Response:**
```json
[
  {
    "symbol": "ETH",
    "name": "Ethereum",
    "address": "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1",
    "decimals": 18,
    "priceDecimals": 30,
    "isNative": false,
    "isShortable": true,
    "isStable": false
  },
  {
    "symbol": "USDC",
    "name": "USD Coin",
    "address": "0xaf88d065e77c8cC2239327C5EDb3A432268e5831",
    "decimals": 6,
    "priceDecimals": 30,
    "isNative": false,
    "isShortable": false,
    "isStable": true
  },
  {
    "symbol": "BTC",
    "name": "Bitcoin",
    "address": "0x2f2a2543B76A4166549F7aaB2e75Bef0aefC5B0f",
    "decimals": 8,
    "priceDecimals": 30,
    "isNative": false,
    "isShortable": true,
    "isStable": false
  }
]
```

**Field Details:**
- `symbol` (string): Token ticker symbol
- `name` (string): Full token name
- `address` (string): Token contract address (hex)
- `decimals` (integer): Token decimal precision (6, 8, or 18)
- `priceDecimals` (integer): Oracle price decimal precision (always 30)
- `isNative` (boolean): Whether token is native to chain
- `isShortable` (boolean): Can be used as index token for shorts
- `isStable` (boolean): Is a stablecoin

---

### 6. Markets List

**Endpoint:** `GET /{chain}-api.gmxinfra.io/markets`

**Response:**
```json
[
  {
    "marketToken": "0x70d95587d40A2caf56bd97485aB3Eec10Bee6336",
    "indexToken": "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1",
    "longToken": "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1",
    "shortToken": "0xaf88d065e77c8cC2239327C5EDb3A432268e5831",
    "indexTokenSymbol": "ETH",
    "longTokenSymbol": "ETH",
    "shortTokenSymbol": "USDC",
    "marketSymbol": "ETH/USD"
  },
  {
    "marketToken": "0x47c031236e19d024b42f8AE6780E44A573170703",
    "indexToken": "0x2f2a2543B76A4166549F7aaB2e75Bef0aefC5B0f",
    "longToken": "0x2f2a2543B76A4166549F7aaB2e75Bef0aefC5B0f",
    "shortToken": "0xaf88d065e77c8cC2239327C5EDb3A432268e5831",
    "indexTokenSymbol": "BTC",
    "longTokenSymbol": "BTC",
    "shortTokenSymbol": "USDC",
    "marketSymbol": "BTC/USD"
  }
]
```

**Field Details:**
- `marketToken` (string): GM token address (liquidity pool token)
- `indexToken` (string): Index token address (asset being traded)
- `longToken` (string): Collateral token for long positions
- `shortToken` (string): Collateral token for short positions
- `indexTokenSymbol` (string): Index token symbol
- `longTokenSymbol` (string): Long collateral symbol
- `shortTokenSymbol` (string): Short collateral symbol
- `marketSymbol` (string): Human-readable market name

**Market Naming:**
- Format: `{INDEX}/{QUOTE}` (e.g., "ETH/USD")
- Full format: `{INDEX}/{QUOTE} [{LONG}-{SHORT}]` (e.g., "ETH/USD [ETH-USDC]")

---

### 7. Markets Info (Detailed)

**Endpoint:** `GET /{chain}-api.gmxinfra.io/markets/info`

**Response:**
```json
{
  "0x70d95587d40A2caf56bd97485aB3Eec10Bee6336": {
    "marketToken": "0x70d95587d40A2caf56bd97485aB3Eec10Bee6336",
    "indexToken": "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1",
    "longToken": "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1",
    "shortToken": "0xaf88d065e77c8cC2239327C5EDb3A432268e5831",
    "marketSymbol": "ETH/USD [ETH-USDC]",
    "poolValueInfo": {
      "poolValue": "45678901234567890123456789012",
      "longTokenAmount": "8765432109876543210",
      "shortTokenAmount": "12345678901234",
      "longTokenUsd": "22839450617283945061728394506",
      "shortTokenUsd": "22839450617283945061728394506",
      "totalBorrowingFees": "123456789012345678",
      "borrowingFeePoolFactor": "500000000000000000000000000000",
      "impactPoolAmount": "1234567890123456",
      "longPnl": "-500000000000000000000000000000",
      "shortPnl": "300000000000000000000000000000",
      "netPnl": "-200000000000000000000000000000",
      "pnlAfterCap": "-200000000000000000000000000000"
    },
    "openInterestLong": "15000000000000000000000000000000",
    "openInterestShort": "12000000000000000000000000000000",
    "longInterestUsd": "15000000000000000000000000000000",
    "shortInterestUsd": "12000000000000000000000000000000",
    "longInterestInTokens": "6000000000000000000",
    "shortInterestInTokens": "4800000000000000000",
    "maxOpenInterestUsd": "50000000000000000000000000000000",
    "maxOpenInterestLong": "25000000000000000000000000000000",
    "maxOpenInterestShort": "25000000000000000000000000000000",
    "fundingFactorPerSecond": "380517503805175",
    "longsPayShorts": true,
    "borrowingFactorPerSecondForLongs": "190258751902587",
    "borrowingFactorPerSecondForShorts": "190258751902587",
    "virtualInventoryForLongPositions": "5000000000000000000",
    "virtualInventoryForShortPositions": "0",
    "isDisabled": false,
    "listingDate": 1700000000
  }
}
```

**Field Details:**

**Top-level keys:** Market token addresses (hex strings)

**Market Object:**
- `marketToken` (string): GM token address
- `indexToken` (string): Index token address
- `longToken` (string): Long collateral token address
- `shortToken` (string): Short collateral token address
- `marketSymbol` (string): Full market name with pool

**Pool Value Info:**
- `poolValue` (string): Total pool value in USD (30 decimals)
- `longTokenAmount` (string): Amount of long token in pool (token decimals)
- `shortTokenAmount` (string): Amount of short token in pool (token decimals)
- `longTokenUsd` (string): USD value of long tokens (30 decimals)
- `shortTokenUsd` (string): USD value of short tokens (30 decimals)
- `totalBorrowingFees` (string): Cumulative borrowing fees (30 decimals)
- `borrowingFeePoolFactor` (string): Borrowing fee multiplier (30 decimals)
- `impactPoolAmount` (string): Price impact pool reserves
- `longPnl` (string): Unrealized PnL for all long positions (30 decimals)
- `shortPnl` (string): Unrealized PnL for all short positions (30 decimals)
- `netPnl` (string): Net PnL (longPnl + shortPnl)
- `pnlAfterCap` (string): PnL after max cap applied

**Open Interest:**
- `openInterestLong` (string): Total long open interest in USD (30 decimals)
- `openInterestShort` (string): Total short open interest in USD (30 decimals)
- `longInterestUsd` (string): Long interest in USD (30 decimals)
- `shortInterestUsd` (string): Short interest in USD (30 decimals)
- `longInterestInTokens` (string): Long interest in index tokens (token decimals)
- `shortInterestInTokens` (string): Short interest in index tokens (token decimals)
- `maxOpenInterestUsd` (string): Maximum total OI allowed (30 decimals)
- `maxOpenInterestLong` (string): Maximum long OI allowed (30 decimals)
- `maxOpenInterestShort` (string): Maximum short OI allowed (30 decimals)

**Funding & Borrowing:**
- `fundingFactorPerSecond` (string): Funding rate per second (30 decimals)
- `longsPayShorts` (boolean): Direction of funding (true = longs pay shorts)
- `borrowingFactorPerSecondForLongs` (string): Borrow rate for longs (30 decimals)
- `borrowingFactorPerSecondForShorts` (string): Borrow rate for shorts (30 decimals)

**Virtual Inventory (Price Impact):**
- `virtualInventoryForLongPositions` (string): Virtual long inventory (token decimals)
- `virtualInventoryForShortPositions` (string): Virtual short inventory (token decimals)

**Status:**
- `isDisabled` (boolean): Whether market is disabled for trading
- `listingDate` (integer): Unix timestamp of market launch

---

### 8. Fee APYs

**Endpoint:** `GET /{chain}-api.gmxinfra.io/apy?period=7d`

**Response:**
```json
{
  "0x70d95587d40A2caf56bd97485aB3Eec10Bee6336": {
    "marketSymbol": "ETH/USD [ETH-USDC]",
    "apy": "0.0854",
    "apr": "0.0820",
    "totalFees": "123456789012345678901234567890",
    "totalVolume": "9876543210987654321098765432109876"
  },
  "0x47c031236e19d024b42f8AE6780E44A573170703": {
    "marketSymbol": "BTC/USD [BTC-USDC]",
    "apy": "0.0623",
    "apr": "0.0605",
    "totalFees": "98765432109876543210987654321",
    "totalVolume": "7654321098765432109876543210987654"
  }
}
```

**Field Details:**
- Market token address (string): Top-level keys
  - `marketSymbol` (string): Market name
  - `apy` (string): Annual Percentage Yield (decimal, e.g., "0.0854" = 8.54%)
  - `apr` (string): Annual Percentage Rate (decimal, e.g., "0.0820" = 8.20%)
  - `totalFees` (string): Total fees collected in period (30 decimals)
  - `totalVolume` (string): Total trading volume in period (30 decimals)

**Period Options:** `1d`, `7d`, `30d`, `90d`, `180d`, `1y`, `total`

---

### 9. Performance

**Endpoint:** `GET /{chain}-api.gmxinfra.io/performance/annualized?period=30d`

**Response:**
```json
{
  "0x70d95587d40A2caf56bd97485aB3Eec10Bee6336": {
    "marketSymbol": "ETH/USD [ETH-USDC]",
    "annualizedReturn": "0.1245",
    "totalReturn": "0.0312",
    "startValue": "45000000000000000000000000000000",
    "endValue": "46404000000000000000000000000000",
    "fees": "1404000000000000000000000000000"
  }
}
```

**Field Details:**
- `marketSymbol` (string): Market name
- `annualizedReturn` (string): Annualized return rate (decimal)
- `totalReturn` (string): Total return for period (decimal)
- `startValue` (string): Pool value at period start (30 decimals)
- `endValue` (string): Pool value at period end (30 decimals)
- `fees` (string): Total fees earned in period (30 decimals)

---

### 10. GLV Tokens

**Endpoint:** `GET /{chain}-api.gmxinfra.io/glvs`

**Response:**
```json
[
  {
    "glvToken": "0x...",
    "name": "GLV-ETH/USD",
    "symbol": "GLV-ETH",
    "markets": [
      "0x70d95587d40A2caf56bd97485aB3Eec10Bee6336"
    ]
  }
]
```

**Field Details:**
- `glvToken` (string): GLV token address
- `name` (string): GLV token name
- `symbol` (string): GLV token symbol
- `markets` (array): Array of underlying market addresses

---

## Smart Contract Response Formats

### 1. Reader.getAccountPositions()

**Call:**
```solidity
function getAccountPositions(
    DataStore dataStore,
    address account,
    uint256 start,
    uint256 end
) external view returns (Position.Props[] memory)
```

**Return Type:** Array of Position structs

**Position Struct:**
```solidity
struct Props {
    Addresses addresses;
    Numbers numbers;
    Flags flags;
}

struct Addresses {
    address account;
    address market;
    address collateralToken;
}

struct Numbers {
    uint256 sizeInUsd;
    uint256 sizeInTokens;
    uint256 collateralAmount;
    uint256 borrowingFactor;
    uint256 fundingFeeAmountPerSize;
    uint256 longTokenClaimableFundingAmountPerSize;
    uint256 shortTokenClaimableFundingAmountPerSize;
    uint256 increasedAtBlock;
    uint256 decreasedAtBlock;
}

struct Flags {
    bool isLong;
}
```

**Example Decoded Response:**
```json
[
  {
    "addresses": {
      "account": "0x...",
      "market": "0x70d95587d40A2caf56bd97485aB3Eec10Bee6336",
      "collateralToken": "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1"
    },
    "numbers": {
      "sizeInUsd": "5000000000000000000000000000000000",
      "sizeInTokens": "2000000000000000000",
      "collateralAmount": "500000000000000000",
      "borrowingFactor": "1000000000000000000000000000000",
      "fundingFeeAmountPerSize": "500000000000000000000000000",
      "longTokenClaimableFundingAmountPerSize": "100000000000000000000000000",
      "shortTokenClaimableFundingAmountPerSize": "0",
      "increasedAtBlock": 123456789,
      "decreasedAtBlock": 123456790
    },
    "flags": {
      "isLong": true
    }
  }
]
```

---

### 2. Reader.getAccountOrders()

**Call:**
```solidity
function getAccountOrders(
    DataStore dataStore,
    address account,
    uint256 start,
    uint256 end
) external view returns (Order.Props[] memory)
```

**Order Struct:**
```solidity
struct Props {
    Addresses addresses;
    Numbers numbers;
    Flags flags;
}

struct Addresses {
    address account;
    address receiver;
    address callbackContract;
    address uiFeeReceiver;
    address market;
    address initialCollateralToken;
    address[] swapPath;
}

struct Numbers {
    OrderType orderType;
    DecreasePositionSwapType decreasePositionSwapType;
    uint256 sizeDeltaUsd;
    uint256 initialCollateralDeltaAmount;
    uint256 triggerPrice;
    uint256 acceptablePrice;
    uint256 executionFee;
    uint256 callbackGasLimit;
    uint256 minOutputAmount;
    uint256 updatedAtBlock;
}

struct Flags {
    bool isLong;
    bool shouldUnwrapNativeToken;
    bool isFrozen;
}
```

**Example Decoded Response:**
```json
[
  {
    "addresses": {
      "account": "0x...",
      "receiver": "0x...",
      "callbackContract": "0x0000000000000000000000000000000000000000",
      "uiFeeReceiver": "0x0000000000000000000000000000000000000000",
      "market": "0x70d95587d40A2caf56bd97485aB3Eec10Bee6336",
      "initialCollateralToken": "0xaf88d065e77c8cC2239327C5EDb3A432268e5831",
      "swapPath": []
    },
    "numbers": {
      "orderType": 2,
      "decreasePositionSwapType": 0,
      "sizeDeltaUsd": "1000000000000000000000000000000000",
      "initialCollateralDeltaAmount": "1000000000",
      "triggerPrice": "2500000000000000000000000000000000",
      "acceptablePrice": "2512500000000000000000000000000000",
      "executionFee": "5000000000000000",
      "callbackGasLimit": 0,
      "minOutputAmount": 0,
      "updatedAtBlock": 123456789
    },
    "flags": {
      "isLong": true,
      "shouldUnwrapNativeToken": false,
      "isFrozen": false
    }
  }
]
```

**Order Types:**
- 0: MarketSwap
- 1: LimitSwap
- 2: MarketIncrease
- 3: LimitIncrease
- 4: MarketDecrease
- 5: LimitDecrease
- 6: StopLossDecrease
- 7: Liquidation

---

## GraphQL Response Formats (Subsquid)

### Position Query

**Query:**
```graphql
query GetPositions($account: String!) {
  positions(where: { account: $account, isOpen: true }) {
    id
    account
    market
    collateralToken
    isLong
    sizeInUsd
    sizeInTokens
    collateralAmount
    entryPrice
    averagePrice
    realizedPnl
    unrealizedPnl
    createdAt
    updatedAt
  }
}
```

**Response:**
```json
{
  "data": {
    "positions": [
      {
        "id": "0x...-0x...",
        "account": "0x...",
        "market": "0x70d95587d40A2caf56bd97485aB3Eec10Bee6336",
        "collateralToken": "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1",
        "isLong": true,
        "sizeInUsd": "5000000000000000000000000000000000",
        "sizeInTokens": "2000000000000000000",
        "collateralAmount": "500000000000000000",
        "entryPrice": "2500000000000000000000000000000000",
        "averagePrice": "2500000000000000000000000000000000",
        "realizedPnl": "50000000000000000000000000000000",
        "unrealizedPnl": "100000000000000000000000000000000",
        "createdAt": 1674567890,
        "updatedAt": 1674568990
      }
    ]
  }
}
```

---

## Error Response Formats

### REST API Errors

**HTTP 404 - Not Found:**
```json
{
  "error": "Resource not found"
}
```

**HTTP 500 - Internal Server Error:**
```json
{
  "error": "Internal server error",
  "message": "Failed to fetch data from blockchain"
}
```

### Smart Contract Errors

**Reverted Transaction:**
```json
{
  "code": -32000,
  "message": "execution reverted: InsufficientCollateral",
  "data": "0x..."
}
```

Common revert reasons:
- `InsufficientCollateral`
- `MaxLeverageExceeded`
- `PriceImpactTooHigh`
- `InsufficientLiquidity`
- `MarketDisabled`
- `OrderNotFound`

---

## Sources

- [GMX REST API Documentation](https://docs.gmx.io/docs/api/rest/)
- [GMX Synthetics Contracts](https://github.com/gmx-io/gmx-synthetics)
- [GMX Reader Contract](https://github.com/gmx-io/gmx-synthetics/blob/main/contracts/reader/Reader.sol)
- [Web3 Ethereum DeFi - GMX API](https://web3-ethereum-defi.readthedocs.io/api/gmx/_autosummary_gmx/eth_defi.gmx.api.html)
