# GMX V2 API Endpoints

GMX is a decentralized perpetual exchange on Arbitrum, Avalanche, and Botanix. Unlike centralized exchanges, GMX operates through smart contracts and provides REST API endpoints for read-only data retrieval. Trading operations require direct blockchain interaction via contracts.

## Base URLs

### Arbitrum
- Primary: `https://arbitrum-api.gmxinfra.io`
- Fallback 1: `https://arbitrum-api-fallback.gmxinfra.io`
- Fallback 2: `https://arbitrum-api-fallback2.gmxinfra.io`

### Avalanche
- Primary: `https://avalanche-api.gmxinfra.io`
- Fallback 1: `https://avalanche-api-fallback.gmxinfra.io`
- Fallback 2: `https://avalanche-api-fallback2.gmxinfra.io`

### Botanix
- Primary: `https://botanix-api.gmxinfra.io`
- Fallback 1: `https://botanix-api-fallback.gmxinfra.io`
- Fallback 2: `https://botanix-api-fallback2.gmxinfra.io`

## MarketData Trait Endpoints

### 1. Ping
**GET** `/{chain}-api.gmxinfra.io/ping`

Check endpoint status and connectivity.

**Response:**
```json
{
  "status": "ok"
}
```

---

### 2. Tickers (Price Display)
**GET** `/{chain}-api.gmxinfra.io/prices/tickers`

Retrieve latest price information for pricing display across all tokens.

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
  }
}
```

**Note:** Prices are stored with 30 decimals of precision.

---

### 3. Signed Prices (Transaction Submission)
**GET** `/{chain}-api.gmxinfra.io/signed_prices/latest`

Retrieve cryptographically signed price information for sending on-chain transactions.

**Response:**
```json
{
  "signedPrices": [
    {
      "tokenSymbol": "ETH",
      "minPrice": "2500000000000000000000000000000000",
      "maxPrice": "2501000000000000000000000000000000",
      "timestamp": 1674567890,
      "signature": "0x..."
    }
  ]
}
```

**Usage:** Required when creating orders via smart contracts to ensure price authenticity.

---

### 4. Candlesticks (OHLC Data)
**GET** `/{chain}-api.gmxinfra.io/prices/candles`

**Query Parameters:**
- `tokenSymbol` (required): Token symbol (e.g., "ETH", "BTC")
- `period` (required): Timeframe - one of: `1m`, `5m`, `15m`, `1h`, `4h`, `1d`
- `limit` (optional): Number of candles to return (1-10,000; default: 1,000)

**Example:**
```
GET /prices/candles?tokenSymbol=ETH&period=1h&limit=100
```

**Response:**
```json
[
  [1674567890, "2500.50", "2505.75", "2498.25", "2503.00"],
  [1674564290, "2495.00", "2501.00", "2493.50", "2500.50"]
]
```

**Format:** `[timestamp, open, high, low, close]` in descending order (newest first).

---

### 5. Tokens List
**GET** `/{chain}-api.gmxinfra.io/tokens`

List all supported tokens on the network.

**Response:**
```json
[
  {
    "symbol": "ETH",
    "address": "0x...",
    "decimals": 18,
    "isNative": false
  },
  {
    "symbol": "BTC",
    "address": "0x...",
    "decimals": 8,
    "isNative": false
  }
]
```

---

### 6. Markets List
**GET** `/{chain}-api.gmxinfra.io/markets`

List all available markets and their GM tokens.

**Response:**
```json
[
  {
    "marketToken": "0x...",
    "indexToken": "0x...",
    "longToken": "0x...",
    "shortToken": "0x...",
    "marketSymbol": "ETH/USD"
  }
]
```

---

### 7. Markets Info (Detailed)
**GET** `/{chain}-api.gmxinfra.io/markets/info`

Comprehensive market information including:
- Liquidity (pool balances)
- Open interest (long/short)
- Token amounts
- Funding rates
- Borrowing rates
- Net rates
- Market status (isDisabled)
- Listing date

**Response:**
```json
{
  "markets": [
    {
      "marketToken": "0x...",
      "indexToken": "0x...",
      "longToken": "0x...",
      "shortToken": "0x...",
      "marketSymbol": "ETH/USD [ETH-USDC]",
      "poolValueInfo": {
        "poolValue": "1000000000000000000000000",
        "longTokenAmount": "500000000000000000",
        "shortTokenAmount": "1000000000000",
        "longTokenUsd": "500000000000000000000000",
        "shortTokenUsd": "500000000000000000000000"
      },
      "openInterestLong": "250000000000000000000000",
      "openInterestShort": "200000000000000000000000",
      "fundingFactorPerSecond": "1000000000000",
      "borrowingFactorPerSecondForLongs": "500000000000",
      "borrowingFactorPerSecondForShorts": "500000000000",
      "isDisabled": false,
      "virtualInventoryForLongPositions": "0",
      "virtualInventoryForShortPositions": "0"
    }
  ]
}
```

---

### 8. Fee APYs
**GET** `/{chain}-api.gmxinfra.io/apy`

**Query Parameters:**
- `period` (optional): `1d`, `7d`, `30d`, `90d`, `180d`, `1y`, `total` (default: `30d`)

**Example:**
```
GET /apy?period=7d
```

**Response:**
```json
{
  "markets": {
    "0x...": {
      "marketSymbol": "ETH/USD [ETH-USDC]",
      "apy": "0.0542",
      "apr": "0.0528"
    }
  }
}
```

---

### 9. Performance (Annualized Returns)
**GET** `/{chain}-api.gmxinfra.io/performance/annualized`

**Query Parameters:**
- `period` (optional): `7d`, `30d`, `90d`, `180d`, `1y`, `total` (default: `90d`)
- `address` (optional): Specific GM token address

**Example:**
```
GET /performance/annualized?period=30d
GET /performance/annualized?address=0x...&period=90d
```

**Response:**
```json
{
  "performance": [
    {
      "marketToken": "0x...",
      "marketSymbol": "ETH/USD [ETH-USDC]",
      "annualizedReturn": "0.0842",
      "totalReturn": "0.0210"
    }
  ]
}
```

---

### 10. GLV Tokens
**GET** `/{chain}-api.gmxinfra.io/glvs`

List GLV (GMX Liquidity Vault) tokens.

**Response:**
```json
[
  {
    "glvToken": "0x...",
    "name": "GLV-ETH/USD",
    "symbol": "GLV-ETH"
  }
]
```

---

### 11. GLV Info
**GET** `/{chain}-api.gmxinfra.io/glvs/info`

Detailed information about GLV tokens including underlying markets and composition.

---

## Trading Trait - Smart Contract Interaction

GMX trading operations **do NOT use REST endpoints**. All trading requires direct smart contract interaction on-chain.

### Contract Addresses

#### Arbitrum (Chain ID: 42161)
- **ExchangeRouter**: `0x602b805EedddBbD9ddff44A7dcBD46cb07849685`
- **DataStore**: `0xFD70de6b91282D8017aA4E741e9Ae325CAb992d8`
- **Reader**: `0x470fbC46bcC0f16532691Df360A07d8Bf5ee0789`
- **OrderVault**: `0x31eF83a530Fde1B38EE9A18093A333D8Bbbc40D5`

#### Avalanche (Chain ID: 43114)
- Contract addresses available in gmx-synthetics/deployments/avalanche folder
- Refer to official GMX documentation for current addresses

### Trading Operations

All trading operations use the **ExchangeRouter** contract:

1. **Create Order** (Market/Limit)
   - Function: `createOrder(CreateOrderParams)`
   - Requires: Token approval, collateral transfer to OrderVault, signed prices

2. **Increase Position**
   - Function: `createOrder()` with increase position parameters
   - Order type: Market, Limit

3. **Decrease Position**
   - Function: `createOrder()` with decrease position parameters
   - Order type: Market, Limit, Stop Loss, Take Profit

4. **Cancel Order**
   - Function: `cancelOrder(bytes32 key)`
   - Only unfilled orders can be cancelled

5. **Execute Order**
   - Performed by off-chain keepers
   - Users create orders, keepers execute with oracle prices

### Order Flow

1. **User submits transaction** → Creates order request via ExchangeRouter
2. **Collateral transferred** → Tokens moved to OrderVault (same transaction)
3. **Order stored** → Request stored in DataStore
4. **Keeper monitors** → Off-chain keepers listen for new orders
5. **Keeper executes** → Bundles oracle prices and executes order

---

## Account Trait - Position & Balance Queries

### Query via Smart Contracts

#### 1. Get Account Positions
**Contract:** Reader (`0x470fbC46bcC0f16532691Df360A07d8Bf5ee0789` on Arbitrum)

**Function:** `getAccountPositions(DataStore dataStore, address account, uint256 start, uint256 end)`

**Returns:** Array of position data including:
- Position key
- Market address
- Collateral token
- Size in USD
- Size in tokens
- Collateral amount
- Entry price
- Realized PnL
- Unrealized PnL

#### 2. Get Account Orders
**Function:** `getAccountOrders(DataStore dataStore, address account, uint256 start, uint256 end)`

**Returns:** Array of order data including:
- Order key
- Order type
- Market address
- Size delta
- Trigger price
- Acceptable price
- Execution fee

#### 3. Get Account Balances
**Function:** Standard ERC20 `balanceOf(address account)` for each token

For GM token balances:
**Function:** `MarketToken.balanceOf(address account)`

For staking/rewards:
**Function:** `RewardReader.getStakingInfo(address account, address[] rewardTrackers)`

**Returns:** Array of claimable reward amounts

---

## Positions Trait - Position Management

### Query Open Positions

Use Reader contract methods:
- `getAccountPositions()` - Get all positions for an account
- `getPosition()` - Get specific position by key

### Position Data Structure

```solidity
struct Position {
    bytes32 key;              // Unique position identifier
    address market;           // Market address
    address collateralToken;  // Collateral token address
    bool isLong;              // Long or short
    uint256 sizeInUsd;        // Position size in USD
    uint256 sizeInTokens;     // Position size in tokens
    uint256 collateralAmount; // Collateral amount
    uint256 borrowingFactor;  // Cumulative borrowing factor
    uint256 fundingFeeAmountPerSize; // Funding fee per size
    int256 realizedPnlUsd;    // Realized profit/loss
}
```

### Position Queries via GraphQL (Subsquid)

**Arbitrum Subsquid:** `https://gmx.squids.live/gmx-synthetics-arbitrum:prod/api/graphql`

**Avalanche Subsquid:** `https://gmx.squids.live/gmx-synthetics-avalanche:prod/api/graphql`

**Example GraphQL Query:**
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
    realizedPnl
    unrealizedPnl
    entryPrice
    averagePrice
  }
}
```

---

## Subsquid GraphQL Endpoints

### Arbitrum
- Production: `https://gmx.squids.live/gmx-synthetics-arbitrum:prod/api/graphql`
- Alternative: `https://gmx.squids.live/gmx-synthetics-arbitrum@e27d93/api/graphql`

### Avalanche
- Production: `https://gmx.squids.live/gmx-synthetics-avalanche:prod/api/graphql`
- Alternative: `https://gmx.squids.live/gmx-synthetics-avalanche@e27d93/api/graphql`

### Fuji Testnet
- `https://gmx.squids.live/gmx-synthetics-fuji@e27d93/api/graphql`

### Supported Queries
- Markets and tokens
- Positions (open/closed)
- Orders (pending/executed/cancelled)
- Trades and executions
- User statistics
- Volume and fees
- Liquidations

---

## Implementation Notes for V5 Connector

### MarketData Trait
- Use REST endpoints for all market data queries
- Implement fallback URL rotation for reliability
- Parse 30-decimal precision prices correctly
- Cache token and market lists

### Trading Trait
- Requires Web3/Ethers library for contract interaction
- Must implement wallet signing for transactions
- Handle two-step order process (create → keeper execution)
- Monitor transaction status via events

### Account Trait
- Use Reader contract for position/order queries
- Support multicall for batch queries
- Parse complex struct returns from contracts

### Positions Trait
- Prefer GraphQL (Subsquid) for historical data
- Use Reader contract for real-time position state
- Calculate unrealized PnL client-side using current prices

### Error Handling
- Implement fallback URL logic
- Handle blockchain RPC errors
- Parse contract revert reasons
- Retry failed transactions with adjusted gas

---

## Sources

- [GMX REST API Documentation](https://docs.gmx.io/docs/api/rest/)
- [GMX SDK Documentation](https://docs.gmx.io/docs/sdk/)
- [GMX Synthetics Contracts](https://github.com/gmx-io/gmx-synthetics)
- [GMX Trading V2 Guide](https://docs.gmx.io/docs/trading/v2/)
- [GMX Contracts Documentation](https://docs.gmx.io/docs/api/contracts-v2/)
- [Web3 Ethereum DeFi - GMX API](https://web3-ethereum-defi.readthedocs.io/api/gmx/_autosummary_gmx/eth_defi.gmx.api.html)
