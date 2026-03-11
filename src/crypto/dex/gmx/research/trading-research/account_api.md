# GMX V2 Account and Position Data API

Source date: 2026-03-11
Official docs: https://docs.gmx.io/
GitHub: https://github.com/gmx-io/gmx-synthetics

---

## Overview

GMX V2 does not have a REST API for account data (positions, balances, trade history). Account data is read from:

1. **On-chain** via the `Reader.sol` contract (direct EVM calls)
2. **Subsquid GraphQL** for indexed historical data (trade history, past orders)
3. **REST API** for market-level data (not account-specific)

There is no REST endpoint like `/account/positions` or `/account/balance`.

---

## On-Chain Account Queries: Reader Contract

The `Reader.sol` contract is the primary interface for querying account-specific on-chain state. It provides convenience functions over the `DataStore`.

Reader contract address on Arbitrum: NOT DOCUMENTED in official sources at time of research. Discoverable via the DataStore and deployment scripts in the gmx-synthetics repo.

### Key Reader Functions (from `Reader.sol` / `ReaderUtils.sol`)

```solidity
// Get all positions for an account
function getAccountPositions(
    DataStore dataStore,
    address account,
    uint256 start,
    uint256 end
) external view returns (Position.Props[] memory);

// Get position count for an account
function getAccountPositionCount(
    DataStore dataStore,
    address account
) external view returns (uint256);

// Get all orders for an account
function getAccountOrders(
    DataStore dataStore,
    address account,
    uint256 start,
    uint256 end
) external view returns (Order.Props[] memory);

// Get order count for an account
function getAccountOrderCount(
    DataStore dataStore,
    address account
) external view returns (uint256);

// Get a single position by key
function getPosition(
    DataStore dataStore,
    bytes32 key
) external view returns (Position.Props memory);

// Get a single order by key
function getOrder(
    DataStore dataStore,
    bytes32 key
) external view returns (Order.Props memory);

// Get market information
function getMarket(
    DataStore dataStore,
    address market
) external view returns (Market.Props memory);

// Get all markets
function getMarkets(
    DataStore dataStore,
    uint256 start,
    uint256 end
) external view returns (Market.Props[] memory);

// Get position value in USD
function getPositionInfo(
    DataStore dataStore,
    IReferralStorage referralStorage,
    bytes32 positionKey,
    MarketUtils.MarketPrices memory prices,
    uint256 sizeDeltaUsd,
    address uiFeeReceiver,
    bool useMaxSizeDeltaForProfit
) external view returns (ReaderUtils.PositionInfo memory);
```

### Position.Props Structure

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
    uint256 sizeInUsd;              // Position size in USD (30 decimals)
    uint256 sizeInTokens;           // Position size in index tokens
    uint256 collateralAmount;       // Collateral token amount
    uint256 borrowingFactor;        // Accumulated borrowing factor
    uint256 fundingFeeAmountPerSize;
    uint256 longTokenClaimableFundingAmountPerSize;
    uint256 shortTokenClaimableFundingAmountPerSize;
    uint256 increasedAtTime;        // Timestamp of last increase
    uint256 decreasedAtTime;        // Timestamp of last decrease
    uint256 increasedAtBlock;
    uint256 decreasedAtBlock;
}

struct Flags {
    bool isLong;                    // true = long, false = short
}
```

### Usage Pattern

```
// Read positions using pagination
uint256 count = reader.getAccountPositionCount(dataStore, account);
Position.Props[] memory positions = reader.getAccountPositions(dataStore, account, 0, count);
```

---

## Balance Queries

### Token Balances

GMX V2 does not have a proprietary balance API. Token balances are standard ERC-20:

```solidity
// For any collateral token (e.g. USDC, WETH)
IERC20(tokenAddress).balanceOf(account)
```

Collateral is held inside open positions as `collateralAmount` in `Position.Props`. There is no separate "margin balance" concept — collateral is committed per-position.

### Profit/Loss

Unrealized PnL is calculated by the `Reader.getPositionInfo()` function using current oracle prices. This requires passing live `MarketPrices` (obtained from `signed_prices/latest`).

---

## Trade History: Subsquid GraphQL API

Historical trade data (past orders, position changes, swaps) is indexed via Subsquid.

### Endpoint URLs

| Chain | URL |
|-------|-----|
| Arbitrum One | `https://gmx.squids.live/gmx-synthetics-arbitrum:prod/api/graphql` |
| Avalanche C-Chain | `https://gmx.squids.live/gmx-synthetics-avalanche:prod/api/graphql` |
| Botanix | `https://gmx.squids.live/gmx-synthetics-botanix:prod/api/graphql` |
| MegaETH | `https://gmx.squids.live/gmx-synthetics-megaeth:prod/api/graphql` |

GraphQL playground: `https://gmx.squids.live/gmx-synthetics-arbitrum/graphql`

### Available Entity Types

As documented at `https://docs.gmx.io/docs/api/graphql/`:

| Entity | Description |
|--------|-------------|
| `TradeAction` | Historical trade events (order executions, position changes) |
| `ClaimAction` | Funding fee and collateral claim events |
| `Order` | Order state snapshots |
| `SwapFeesInfo` | Swap fee data per transaction |
| `SwapInfo` | Swap execution details |
| `PositionFeesEntity` | Position fee breakdown |
| `Distribution` | Fee distribution events |

### Schema Change (February 2026)

As of **2026-02-24**, the `Transaction` entity was **removed**. Breaking changes:

- Entities now expose `transactionHash: String!` directly
- Top-level `timestamp` field replaces nested `transaction.timestamp`
- Sort operations changed from `transaction_timestamp_DESC` to `timestamp_DESC`
- Backward-compatible endpoint available until **2026-03-01**

### Example GraphQL Query: Trade History for Account

```graphql
query TradeHistory($account: String!) {
  tradeActions(
    where: { account_eq: $account }
    orderBy: timestamp_DESC
  ) {
    id
    transactionHash
    timestamp
    account
    marketAddress
    isLong
    orderType
    sizeDeltaUsd
    collateralDeltaAmount
    executionPrice
    priceImpactUsd
    reason
  }
}
```

Note: Exact field names are NOT FULLY DOCUMENTED in official sources. The above is based on SDK usage patterns and partial documentation. Verify via the GraphQL playground introspection.

### SDK Wrapper

The `@gmx-io/sdk` provides a typed wrapper:

```typescript
// Trade history
const trades: TradeAction[] = await sdk.account.getTradeHistory({
  account: "0x...",
  // additional filters...
});
```

---

## Deposit / Withdrawal Flows

GMX V2 uses **two-step** execution for all state changes. There is no REST API for deposits or withdrawals.

### GM Pool Liquidity (LP Deposits/Withdrawals)

These are for liquidity providers, not traders. Done via:

- `ExchangeRouter.createDeposit()` — submit deposit request for GM tokens
- `ExchangeRouter.createWithdrawal()` — submit withdrawal request to redeem GM tokens
- Keepers execute both within seconds

### Position Collateral Management

See `trading_api.md` for details on adding/removing collateral via `MarketIncrease`/`MarketDecrease` with `sizeDeltaUsd = 0`.

---

## Subgraph vs Subsquid

GMX V2 uses **Subsquid** (not The Graph) for V2 indexing. The `gmx-io/gmx-subgraph` repo contains legacy V1 subgraphs for Arbitrum and Avalanche stats. V2 data is exclusively on Subsquid.

The subgraph repo structure:
```
gmx-subgraph/
├── gmx-arbitrum-stats/    # V1 stats subgraph
├── gmx-avalanche-stats/   # V1 stats subgraph
├── synthetics-stats/      # V2 stats (still uses The Graph infrastructure)
├── gmx-arbitrum-prices/   # Price data
├── gmx-arbitrum-raw/      # Raw event data
└── gmx-referrals/         # Referral program
```

The `synthetics-stats` branch within gmx-subgraph indexes V2 stats data. The main trading/positions/orders indexing for V2 is done by Subsquid.

---

## REST API Market Data (Account-Unaware)

The REST API at `gmxinfra.io` provides **market-level** data only. It has no account-level queries:

| Endpoint | Returns |
|----------|---------|
| `GET /markets/info` | All markets with OI, liquidity, rates |
| `GET /prices/tickers` | Current token prices |
| `GET /tokens` | Token list with decimals |
| `GET /prices/candles?tokenSymbol=X&period=Y` | OHLC candle data |
| `GET /glvs/info` | GLV vault data |
| `GET /signed_prices/latest` | Oracle-signed prices (for on-chain use) |

These endpoints require no authentication and have no per-account filtering.

---

## Sources

- [GitHub - gmx-io/gmx-synthetics: Reader.sol](https://github.com/gmx-io/gmx-synthetics/blob/main/contracts/reader/Reader.sol)
- [GitHub - gmx-io/gmx-synthetics: ReaderUtils.sol](https://github.com/gmx-io/gmx-synthetics/blob/main/contracts/reader/ReaderUtils.sol)
- [GitHub - gmx-io/gmx-subgraph](https://github.com/gmx-io/gmx-subgraph)
- [Subsquid | GMX Docs](https://docs.gmx.io/docs/api/subsquid/)
- [SDK README | GMX Docs](https://docs.gmx.io/docs/sdk/)
- [@gmx-io/sdk on npm](https://www.npmjs.com/package/@gmx-io/sdk)
- [REST V2 | GMX Docs](https://docs.gmx.io/docs/api/rest-v2/)
- [GraphQL playground](https://gmx.squids.live/gmx-synthetics-arbitrum/graphql)
