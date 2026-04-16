# dYdX v4 API Endpoints

## Base URLs

### Production (Mainnet)
- **Indexer HTTP**: `https://indexer.dydx.trade/v4`
- **Indexer WebSocket**: `wss://indexer.dydx.trade/v4/ws`
- **Node gRPC**: Multiple providers available
  - OEGS: `grpc://oegs.dydx.trade:443`
  - Polkachu: `https://dydx-dao-grpc-1.polkachu.com:443` through `-5`
  - KingNodes: `https://dydx-ops-grpc.kingnodes.com:443`
  - Enigma: `https://dydx-dao-grpc.enigma-validator.com:443`
  - Lavender.Five: `https://dydx.lavendarfive.com:443`
  - PublicNode: `https://dydx-grpc.publicnode.com:443`
- **Node RPC**: `https://oegs.dydx.trade:443` (and other providers)
- **Node REST**: Available from multiple providers

### Testnet
- **Indexer HTTP**: `https://indexer.v4testnet.dydx.exchange/v4`
- **Indexer WebSocket**: `wss://indexer.v4testnet.dydx.exchange/v4/ws`
- **Node gRPC**: `oegs-testnet.dydx.exchange:443`
- **Node RPC**: `https://oegs-testnet.dydx.exchange:443`
- **Faucet**: `https://faucet.v4testnet.dydx.exchange`

## Architecture Overview

dYdX v4 has a dual API architecture:

1. **Indexer API** (REST/WebSocket) - Read-only queries for market data, account info, orders, positions
2. **Node API** (gRPC) - Write operations requiring authentication (place/cancel orders, transfers)

---

## MarketData Trait Endpoints

All MarketData endpoints are READ-ONLY and use the **Indexer HTTP API**.

### Get Perpetual Markets
**Endpoint**: `GET /v4/perpetualMarkets`

**Parameters**:
- `market` (optional) - Specific market ticker (e.g., "BTC-USD")

**Response**: List of all perpetual markets or specific market details

**Fields**:
- `ticker` - Market symbol (e.g., "BTC-USD")
- `clobPairId` - Internal market identifier
- `status` - Market status
- `baseAsset` - Base asset symbol
- `quoteAsset` - Quote asset symbol (always USDC)
- `stepSize` - Minimum order size increment
- `tickSize` - Minimum price increment
- `indexPrice` - Current index price
- `oraclePrice` - Oracle price
- `priceChange24H` - 24h price change
- `volume24H` - 24h trading volume
- `trades24H` - 24h number of trades
- `nextFundingRate` - Next funding rate
- `initialMarginFraction` - Initial margin requirement
- `maintenanceMarginFraction` - Maintenance margin requirement
- `openInterest` - Total open interest
- `atomicResolution` - Quantum resolution for size
- `quantumConversionExponent` - Price conversion exponent
- `subticksPerTick` - Subticks per tick

### Get Market Orderbook
**Endpoint**: `GET /v4/orderbooks/perpetualMarket/{market}`

**Parameters**:
- `market` (path, required) - Market ticker (e.g., "BTC-USD")

**Response**: Current order book with bids and asks

**Fields**:
- `bids` - Array of bid levels
  - `price` - Bid price (string)
  - `size` - Bid size (string)
- `asks` - Array of ask levels
  - `price` - Ask price (string)
  - `size` - Ask size (string)

### Get Market Trades
**Endpoint**: `GET /v4/trades/perpetualMarket/{market}`

**Parameters**:
- `market` (path, required) - Market ticker
- `limit` (optional) - Number of trades to return
- `startingBeforeOrAtHeight` (optional) - Block height filter

**Response**: Recent executed trades

**Fields**:
- `id` - Trade ID
- `side` - "BUY" or "SELL"
- `size` - Trade size
- `price` - Execution price
- `type` - Trade type
- `createdAt` - Timestamp
- `createdAtHeight` - Block height

### Get Candles (OHLCV)
**Endpoint**: `GET /v4/candles/perpetualMarkets/{market}`

**Parameters**:
- `market` (path, required) - Market ticker
- `resolution` (required) - One of: "1MIN", "5MINS", "15MINS", "30MINS", "1HOUR", "4HOURS", "1DAY"
- `limit` (optional) - Number of candles
- `fromISO` (optional) - Start time (ISO 8601)
- `toISO` (optional) - End time (ISO 8601)

**Response**: OHLCV candle data

**Fields**:
- `startedAt` - Candle start timestamp
- `ticker` - Market ticker
- `resolution` - Time resolution
- `low` - Lowest price
- `high` - Highest price
- `open` - Opening price
- `close` - Closing price
- `baseTokenVolume` - Volume in base asset
- `usdVolume` - Volume in USD
- `trades` - Number of trades
- `startingOpenInterest` - Open interest at start

### Get Historical Funding Rates
**Endpoint**: `GET /v4/historicalFunding/{market}`

**Parameters**:
- `market` (path, required) - Market ticker
- `limit` (optional) - Number of records
- `effectiveBeforeOrAt` (optional) - Time filter
- `effectiveBeforeOrAtHeight` (optional) - Block height filter

**Response**: Historical funding rate data

**Fields**:
- `ticker` - Market ticker
- `rate` - Funding rate
- `price` - Index price at the time
- `effectiveAt` - Effective timestamp
- `effectiveAtHeight` - Block height

### Get Server Time
**Endpoint**: `GET /v4/time`

**Parameters**: None

**Response**: Current server timestamp

**Fields**:
- `iso` - ISO 8601 timestamp
- `epoch` - Unix epoch timestamp (seconds)

### Get Block Height
**Endpoint**: `GET /v4/height`

**Parameters**: None

**Response**: Current blockchain height and time

**Fields**:
- `height` - Current block height
- `time` - Block timestamp

### Get Market Sparklines
**Endpoint**: `GET /v4/sparklines`

**Parameters**:
- `timePeriod` (required) - Time period for sparkline data

**Response**: Compact price trend visualization data for all markets

---

## Account Trait Endpoints

All Account endpoints are READ-ONLY and use the **Indexer HTTP API**.

### Get Subaccounts
**Endpoint**: `GET /v4/addresses/{address}`

**Parameters**:
- `address` (path, required) - dYdX Chain address
- `limit` (optional) - Number of subaccounts to return

**Response**: List of subaccounts for an address

**Fields**:
- `address` - dYdX Chain address
- `subaccountNumber` - Subaccount number (0-128000)
- `equity` - Total equity (string)
- `freeCollateral` - Available collateral (string)
- `marginEnabled` - Whether margin is enabled (boolean)

### Get Specific Subaccount
**Endpoint**: `GET /v4/addresses/{address}/subaccountNumber/{subaccount_number}`

**Parameters**:
- `address` (path, required) - dYdX Chain address
- `subaccount_number` (path, required) - Subaccount number

**Response**: Detailed subaccount information

**Fields**:
- `address` - dYdX Chain address
- `subaccountNumber` - Subaccount number
- `equity` - Total equity
- `freeCollateral` - Free collateral
- `marginEnabled` - Margin enabled flag
- `openPerpetualPositions` - Map of open positions
- `assetPositions` - Map of asset positions
- `pendingDeposits` - Pending deposits
- `pendingWithdrawals` - Pending withdrawals

### Get Parent Subaccount
**Endpoint**: `GET /v4/addresses/{address}/parentSubaccountNumber/{number}`

**Parameters**:
- `address` (path, required) - dYdX Chain address
- `number` (path, required) - Parent subaccount number (0-127)

**Response**: Parent subaccount with aggregated data from child subaccounts

### Get Asset Positions
**Endpoint**: `GET /v4/assetPositions`

**Parameters**:
- `address` (required) - dYdX Chain address
- `subaccountNumber` (required) - Subaccount number
- `status` (optional) - Position status filter
- `limit` (optional) - Number of records
- `createdBeforeOrAtHeight` (optional) - Block height filter
- `createdBeforeOrAt` (optional) - Time filter

**Response**: Asset position details (USDC balances)

**Fields**:
- `symbol` - Asset symbol (e.g., "USDC")
- `side` - "LONG" (positive balance)
- `size` - Position size
- `assetId` - Asset identifier

### Get Transfer History
**Endpoint**: `GET /v4/transfers`

**Parameters**:
- `address` (required) - dYdX Chain address
- `subaccount_number` (required) - Subaccount number
- `limit` (optional) - Number of records
- `createdBeforeOrAtHeight` (optional) - Block height filter
- `createdBeforeOrAt` (optional) - Time filter
- `page` (optional) - Page number

**Response**: Transfer records (deposits, withdrawals, transfers between subaccounts)

**Fields**:
- `id` - Transfer ID
- `sender` - Sender details
  - `address` - Sender address
  - `subaccountNumber` - Sender subaccount
- `recipient` - Recipient details
  - `address` - Recipient address
  - `subaccountNumber` - Recipient subaccount
- `size` - Transfer amount
- `symbol` - Asset symbol
- `type` - Transfer type ("DEPOSIT", "WITHDRAWAL", "TRANSFER_OUT", "TRANSFER_IN")
- `createdAt` - Timestamp
- `createdAtHeight` - Block height
- `transactionHash` - Transaction hash

### Get Trading Rewards
**Endpoint**: `GET /v4/historicalBlockTradingRewards/{address}`

**Parameters**:
- `address` (path, required) - dYdX Chain address
- `limit` (optional) - Number of records
- `startingBeforeOrAtHeight` (optional) - Block height filter
- `startingBeforeOrAt` (optional) - Time filter

**Response**: Historical block trading rewards

**Fields**:
- `tradingReward` - Reward amount
- `createdAt` - Timestamp
- `createdAtHeight` - Block height

### Get Aggregated Rewards
**Endpoint**: `GET /v4/historicalTradingRewardAggregations/{address}`

**Parameters**:
- `address` (path, required) - dYdX Chain address
- `period` (optional) - Aggregation period
- `limit` (optional) - Number of records
- `startingBeforeOrAt` (optional) - Time filter
- `startingBeforeOrAtHeight` (optional) - Block height filter

**Response**: Aggregated trading rewards by period

**Fields**:
- `tradingReward` - Total reward amount
- `startedAt` - Period start timestamp
- `endedAt` - Period end timestamp
- `period` - Aggregation period

---

## Positions Trait Endpoints

All Positions endpoints are READ-ONLY and use the **Indexer HTTP API**.

### List Perpetual Positions
**Endpoint**: `GET /v4/perpetualPositions`

**Parameters**:
- `address` (required) - dYdX Chain address
- `subaccountNumber` (required) - Subaccount number
- `status` (optional) - "OPEN", "CLOSED", "LIQUIDATED"
- `limit` (optional) - Number of records
- `createdBeforeOrAtHeight` (optional) - Block height filter
- `createdBeforeOrAt` (optional) - Time filter

**Response**: Current and historical perpetual positions

**Fields**:
- `market` - Market ticker (e.g., "BTC-USD")
- `status` - Position status
- `side` - "LONG" or "SHORT"
- `size` - Position size
- `maxSize` - Maximum position size reached
- `entryPrice` - Average entry price
- `exitPrice` - Average exit price (for closed positions)
- `realizedPnl` - Realized profit/loss
- `unrealizedPnl` - Unrealized profit/loss
- `createdAt` - Position open timestamp
- `createdAtHeight` - Block height when opened
- `closedAt` - Position close timestamp (if closed)
- `sumOpen` - Sum of opening trades
- `sumClose` - Sum of closing trades
- `netFunding` - Net funding payments

### List Parent Positions
**Endpoint**: `GET /v4/perpetualPositions/parentSubaccountNumber`

**Parameters**:
- `address` (required) - dYdX Chain address
- `parentSubaccountNumber` (required) - Parent subaccount number (0-127)
- `limit` (optional) - Number of records

**Response**: Aggregated positions from parent subaccount (includes child subaccounts)

### Get Historical PnL
**Endpoint**: `GET /v4/historical-pnl`

**Parameters**:
- `address` (required) - dYdX Chain address
- `subaccount_number` (required) - Subaccount number
- `limit` (optional) - Number of records
- `createdBeforeOrAtHeight` (optional) - Block height filter
- `createdBeforeOrAt` (optional) - Time filter
- `createdOnOrAfterHeight` (optional) - Block height filter (start)
- `createdOnOrAfter` (optional) - Time filter (start)
- `page` (optional) - Page number

**Response**: Historical profit/loss data

**Fields**:
- `id` - Record ID
- `equity` - Account equity at the time
- `totalPnl` - Total P&L
- `netTransfers` - Net transfers
- `createdAt` - Timestamp
- `blockHeight` - Block height
- `blockTime` - Block timestamp

### Get Parent Historical PnL
**Endpoint**: `GET /v4/historical-pnl/parentSubaccountNumber`

**Parameters**:
- `address` (required) - dYdX Chain address
- `parentSubaccountNumber` (required) - Parent subaccount number
- `limit` (optional) - Number of records
- `createdBeforeOrAtHeight` (optional) - Block height filter
- `createdBeforeOrAt` (optional) - Time filter

**Response**: Combined profit/loss data for parent and child subaccounts

### Get Funding Payments
**Endpoint**: `GET /v4/fundingPayments`

**Parameters**:
- `address` (required) - dYdX Chain address
- `subaccountNumber` (required) - Subaccount number
- `limit` (optional) - Number of records
- `ticker` (optional) - Filter by market ticker
- `afterOrAt` (optional) - Time filter
- `page` (optional) - Page number

**Response**: Periodic funding rate settlement records

**Fields**:
- `market` - Market ticker
- `payment` - Funding payment amount (positive = received, negative = paid)
- `rate` - Funding rate applied
- `price` - Index price at the time
- `positionSize` - Position size
- `effectiveAt` - Effective timestamp

### Get Parent Funding Payments
**Endpoint**: `GET /v4/fundingPayments/parentSubaccount`

**Parameters**:
- `address` (required) - dYdX Chain address
- `parentSubaccountNumber` (required) - Parent subaccount number
- `limit` (optional) - Number of records
- `afterOrAt` (optional) - Time filter
- `page` (optional) - Page number

**Response**: Aggregated funding payments for parent and child subaccounts

---

## Trading Trait Endpoints

Trading operations in dYdX v4 use the **Node gRPC API** (NOT the Indexer API). These are WRITE operations requiring authentication.

### Place Order
**Protocol**: gRPC (Protobuf)

**Method**: `MsgPlaceOrder`

**Authentication**: Required (signed transaction)

**Parameters**:
- `subaccount` - Subaccount identifier
  - `owner` - dYdX Chain address
  - `number` - Subaccount number
- `clientId` - Client-assigned order ID (uint32)
- `orderFlags` - Order type flags
  - `0` - Short-term order
  - `32` - Conditional order
  - `64` - Long-term order
- `clobPairId` - Market identifier (internal ID)
- `side` - Order side
  - `ORDER_SIDE_BUY` (1)
  - `ORDER_SIDE_SELL` (2)
- `quantums` - Order size in quantums (uint64)
- `subticks` - Price in subticks (uint64)
- `goodTilBlock` - Expiry block height (for short-term orders)
- `goodTilBlockTime` - Expiry unix timestamp (for stateful orders)
- `timeInForce` - Time in force
  - `TIME_IN_FORCE_UNSPECIFIED` (0)
  - `TIME_IN_FORCE_IOC` (1) - Immediate or Cancel
  - `TIME_IN_FORCE_POST_ONLY` (2) - Post only
  - `TIME_IN_FORCE_FILL_OR_KILL` (3) - Fill or Kill
- `reduceOnly` - Reduce-only flag (boolean)
- `clientMetadata` - Optional client metadata

**Notes**:
- Short-term orders must have `goodTilBlock` within current block + 20 blocks (~30 seconds)
- Stateful orders use `goodTilBlockTime` with max window of current time + 95 days
- Use Composite Client (TypeScript) or Node Client for easier integration
- Price and size must be converted to subticks and quantums (see quantums.md)

### Cancel Order
**Protocol**: gRPC (Protobuf)

**Method**: `MsgCancelOrder`

**Authentication**: Required (signed transaction)

**Parameters**:
- `subaccount` - Subaccount identifier
  - `owner` - dYdX Chain address
  - `number` - Subaccount number
- `clientId` - Client-assigned order ID to cancel
- `orderFlags` - Order type flags (same as place order)
- `clobPairId` - Market identifier
- `goodTilBlock` - Original GTB (for short-term orders)
- `goodTilBlockTime` - Original GTBT (for stateful orders)

**Notes**:
- Short-term order cancellations are best-effort (gossip-based)
- Stateful order cancellations are guaranteed through consensus
- Order is only truly canceled after it expires (for short-term) or after cancellation is included in a block (for stateful)

### Get Orders (Read-Only)
**Endpoint**: `GET /v4/orders`

**Parameters**:
- `address` (required) - dYdX Chain address
- `subaccountNumber` (required) - Subaccount number
- `ticker` (optional) - Filter by market ticker
- `tickerType` (optional) - Ticker type filter
- `side` (optional) - "BUY" or "SELL"
- `status` (optional) - "OPEN", "FILLED", "CANCELED", "BEST_EFFORT_CANCELED", "UNTRIGGERED"
- `type` (optional) - Order type filter
- `limit` (optional) - Number of records
- `goodTilBlockBeforeOrAt` (optional) - GTB filter
- `goodTilBlockTimeBeforeOrAt` (optional) - GTBT filter
- `returnLatestOrders` (optional) - Return latest orders (boolean)

**Response**: Filtered orders

**Fields**:
- `id` - Unique order ID
- `subaccountId` - Subaccount identifier
- `clientId` - Client-assigned ID
- `clobPairId` - Market ID
- `side` - "BUY" or "SELL"
- `size` - Order size
- `totalFilled` - Amount filled
- `price` - Order price
- `type` - Order type
- `status` - Order status
- `timeInForce` - Time in force
- `postOnly` - Post only flag
- `reduceOnly` - Reduce only flag
- `orderFlags` - Order flags (0, 32, or 64)
- `goodTilBlock` - Expiry block
- `goodTilBlockTime` - Expiry timestamp
- `createdAtHeight` - Creation block height
- `clientMetadata` - Client metadata
- `triggerPrice` - Trigger price (for conditional orders)
- `updatedAt` - Last update timestamp
- `updatedAtHeight` - Last update block height

### Get Specific Order
**Endpoint**: `GET /v4/orders/{orderId}`

**Parameters**:
- `orderId` (path, required) - Unique order ID

**Response**: Individual order details (same fields as above)

### Get Fills (Trade Executions)
**Endpoint**: `GET /v4/fills`

**Parameters**:
- `address` (required) - dYdX Chain address
- `subaccountNumber` (required) - Subaccount number
- `ticker` (optional) - Filter by market ticker
- `tickerType` (optional) - Ticker type filter
- `limit` (optional) - Number of records
- `createdBeforeOrAtHeight` (optional) - Block height filter
- `createdBeforeOrAt` (optional) - Time filter
- `page` (optional) - Page number

**Response**: Trade execution records

**Fields**:
- `id` - Fill ID
- `side` - "BUY" or "SELL"
- `liquidity` - "TAKER" or "MAKER"
- `type` - Fill type
- `market` - Market ticker
- `marketType` - Market type
- `price` - Execution price
- `size` - Fill size
- `fee` - Fee amount
- `createdAt` - Execution timestamp
- `createdAtHeight` - Execution block height
- `orderId` - Associated order ID
- `clientMetadata` - Client metadata

### List Parent Orders
**Endpoint**: `GET /v4/orders/parentSubaccountNumber`

**Parameters**:
- `address` (required) - dYdX Chain address
- `parentSubaccountNumber` (required) - Parent subaccount number
- `limit` (optional) - Number of records
- `ticker` (optional) - Filter by market ticker
- `side` (optional) - "BUY" or "SELL"
- `status` (optional) - Order status filter
- `order_type` (optional) - Order type filter

**Response**: All orders under parent subaccount (includes child subaccounts)

### Get Parent Fills
**Endpoint**: `GET /v4/fills/parentSubaccountNumber`

**Parameters**:
- `address` (required) - dYdX Chain address
- `parentSubaccountNumber` (required) - Parent subaccount number
- `limit` (optional) - Number of records
- `market` (optional) - Filter by market ticker
- `marketType` (optional) - Market type filter

**Response**: Executed trades across child accounts

---

## Additional Utility Endpoints

### Compliance Screen
**Endpoint**: `GET /v4/screen`

**Parameters**:
- `address` (required) - Address to screen

**Response**: Compliance screening results

### Compliance Screen v2
**Endpoint**: `GET /v4/compliance/screen/{address}`

**Parameters**:
- `address` (path, required) - Address to screen

**Response**: Address restriction status

**Fields**:
- `restricted` - Whether address is restricted (boolean)
- `reason` - Restriction reason (if applicable)

---

## Vaults Endpoints (Optional)

### Get MegaVault Historical PnL
**Endpoint**: `GET /v4/vault/v1/megavault/historicalPnl`

**Parameters**:
- `resolution` (required) - Time resolution

**Response**: Vault profit/loss data

### Get Vaults Historical PnL
**Endpoint**: `GET /v4/vault/v1/vaults/historicalPnl`

**Parameters**:
- `resolution` (required) - Time resolution

**Response**: Multi-vault aggregated performance

### Get MegaVault Positions
**Endpoint**: `GET /v4/vault/v1/megavault/positions`

**Parameters**: None

**Response**: Current vault position holdings

---

## Important Notes

### Indexer vs Node API
- **Indexer API**: Read-only, REST/WebSocket, no authentication, query market data and account info
- **Node API**: Write operations, gRPC/Protobuf, requires authentication, place/cancel orders

### Data Freshness
WebSocket feeds typically provide more recent data than REST API responses due to read replica lag, often under one second but potentially longer during high load periods.

### Order Placement
- Use gRPC endpoints (validator nodes) for placing/canceling orders
- Use TypeScript Composite Client for easier integration
- Python and Rust must use explicit Node and Indexer clients separately

### Rate Limits
- Public Indexer API has rate limits (specific numbers not documented)
- Consider running your own indexer for high-frequency trading
- Blockchain-level rate limits: 200 short-term orders per block, 2 stateful orders per block

### Deposits and Withdrawals
- USDC deposits via CCTP (Circle's Cross-Chain Transfer Protocol) from Ethereum and other chains
- Deposits routed through Noble blockchain via IBC
- Withdrawals rate limited: max(1% of TVL, $1mm) per hour, max(10% of TVL, $10mm) per day
- Deposit/withdrawal operations use Node API (gRPC)
