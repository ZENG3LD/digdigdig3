# GMX V2 (Synthetics) Trading API

Source date: 2026-03-11
Official docs: https://docs.gmx.io/
GitHub: https://github.com/gmx-io/gmx-synthetics

---

## Overview: How Trading Works on GMX V2

GMX V2 is a **purely on-chain decentralized exchange**. There is NO centralized REST trading API for placing or managing orders. All trading is done through smart contracts on Arbitrum or Avalanche.

Trading follows a **two-step execution model**:
1. User submits a transaction to create an order (via `ExchangeRouter.createOrder()`), paying an upfront `executionFee`.
2. **Keepers** (off-chain bots) observe the blockchain, bundle signed oracle prices with the order, and execute the final state change via `OrderHandler.executeOrder()`.

The "Max Network Fee" shown in the UI is this execution fee — it is overestimated, and the excess is refunded on execution.

---

## Order Types

GMX V2 supports the following order types (defined in `Order.sol`):

| Value | OrderType Name        | Description |
|-------|-----------------------|-------------|
| 0     | MarketSwap            | Immediate swap at current market price |
| 1     | LimitSwap             | Swap executed when minOutputAmount is achievable |
| 2     | MarketIncrease        | Open or increase a long/short position at market price |
| 3     | LimitIncrease         | Open or increase position when trigger price is reached |
| 4     | MarketDecrease        | Close or reduce position at market price |
| 5     | LimitDecrease         | Close/reduce position (Take Profit) when trigger price is reached |
| 6     | StopLossDecrease      | Close/reduce position (Stop Loss) when trigger price is reached |
| 7     | Liquidation           | Forced liquidation (system-only) |

GMX V2.1+ additional types (from `BaseOrderUtils.sol`):
- `StopIncrease` — increase position when trigger price is reached (stop entry)
- `TakeProfitIncrease` — increase position at take-profit trigger
- `TakeProfitDecrease` — decrease position at take-profit trigger

Key behavioral notes:
- **Limit orders do NOT rest on an order book.** They are stored on-chain and executed by keepers when the oracle price crosses the trigger price.
- For LimitDecrease (take profit) on a long: order executes when index token price >= acceptablePrice.
- For StopLossDecrease on a long: order executes when index token price <= acceptablePrice.
- Multiple TP/SL orders can coexist. A typical position open uses a `multicall` with three `createOrder` calls: position + stop-loss + take-profit.

---

## Order Placement: `ExchangeRouter.createOrder()`

All orders are placed by calling `ExchangeRouter.createOrder()` on-chain. There is no REST endpoint for placing orders.

### Contract Addresses (Arbitrum One)

| Contract | Address |
|----------|---------|
| ExchangeRouter (current) | `0x87d66368cD08a7Ca42252f5ab44B2fb6d1Fb8d15` |
| ExchangeRouter (older/deprecated) | `0x602b805EedddBbD9ddff44A7dcBD46cb07849685` |
| OrderVault | `0x31eF83a530Fde1B38EE9A18093A333D8Bbbc40D5` |
| DataStore | `0xFD70de6b91282D8017aA4E741e9Ae325CAb992d8` |

Note: Always use the current ExchangeRouter address. Older addresses cause transaction reversion.

### Pre-call Requirement

Before calling `createOrder`, collateral tokens must be transferred to the `OrderVault` in the **same transaction**. This is typically done via a multicall:

```
ExchangeRouter.multicall([
  sendWnt(executionFeeAmount),       // Send execution fee as wrapped native token
  sendTokens(collateralToken, amount, OrderVault),  // Send collateral
  createOrder(params)                // Create the order
])
```

If the transfer and `createOrder` are not in the same transaction, tokens may be swept by other users.

### `CreateOrderParams` Struct

```solidity
struct CreateOrderParams {
    CreateOrderParamsAddresses addresses;
    CreateOrderParamsNumbers numbers;
    Order.OrderType orderType;
    Order.DecreasePositionSwapType decreasePositionSwapType;
    bool isLong;
    bool shouldUnwrapNativeToken;
    bool autoCancel;
    bytes32 referralCode;
}

struct CreateOrderParamsAddresses {
    address receiver;                // Receives output tokens / refund
    address cancellationReceiver;    // Receives collateral + gas fee if cancelled (0x0 = receiver)
    address callbackContract;        // Called on execution/cancellation (0x0 = no callback)
    address uiFeeReceiver;           // For UI fee collection (0x0 = none)
    address market;                  // GM market token address
    address initialCollateralToken;  // Token used as collateral
    address[] swapPath;              // Markets to route collateral through
}

struct CreateOrderParamsNumbers {
    uint256 sizeDeltaUsd;            // Position size change in USD (30 decimal precision)
    uint256 initialCollateralDeltaAmount;  // Collateral amount
    uint256 triggerPrice;            // Trigger price for limit/stop orders (30 decimals)
    uint256 acceptablePrice;         // Worst acceptable execution price (30 decimals)
    uint256 executionFee;            // Fee paid to keepers for execution
    uint256 callbackGasLimit;        // Gas limit for callback (0 = no callback)
    uint256 minOutputAmount;         // Minimum output for swap orders
    uint256 validFromTime;           // Earliest execution timestamp
}
```

Key fields:
- `sizeDeltaUsd`: 30-decimal USD value. E.g., $1000 position = `1000 * 10^30`
- `isLong`: `true` for long positions, `false` for short
- `autoCancel`: Set `true` for LimitDecrease/StopLossDecrease to auto-cancel when position is closed
- `callbackContract`: Set to `0x0` for EOA-style execution — **no whitelisting required**

### Whitelisting Note

**No whitelist is required for standard order creation.** Smart contracts and bots can call `createOrder()` directly without governance approval. The `ROUTER_PLUGIN` restriction only applies to legacy callback-based integrations. Setting `callbackContract = 0x0` bypasses any such requirements.

---

## Order Management

### Update Order: `ExchangeRouter.updateOrder()`

```solidity
function updateOrder(
    bytes32 key,              // Order key (returned at creation)
    uint256 sizeDeltaUsd,     // New size delta
    uint256 acceptablePrice,  // New acceptable price
    uint256 triggerPrice,     // New trigger price
    uint256 minOutputAmount,  // New minimum output
    uint256 validFromTime,    // New valid-from timestamp
    bool autoCancel           // New autoCancel setting
) external payable nonReentrant;
```

Constraints:
- Only order owner (the `receiver` address) can update.
- Cannot update a MarketOrder.
- The `updateOrder` feature must be enabled for the given orderType in the DataStore.

### Cancel Order: `ExchangeRouter.cancelOrder()`

```solidity
function cancelOrder(
    bytes32 key    // Order key
) external payable nonReentrant;
```

Constraints:
- Only order owner can cancel.
- `cancelOrder` feature must be enabled for the orderType.
- Collateral and remaining execution gas are returned to `cancellationReceiver`.

---

## Position Management

### Opening a Position

Call `ExchangeRouter.createOrder()` with:
- `orderType = MarketIncrease` (or `LimitIncrease`)
- `sizeDeltaUsd` = desired notional size in USD
- `isLong` = true/false
- `initialCollateralToken` = e.g. USDC or ETH
- `market` = GM market token address (e.g. ETH-USDC market)
- `initialCollateralDeltaAmount` = collateral amount

Leverage is implicit: leverage = sizeDeltaUsd / collateralValue. Max leverage: up to 100x.

### Closing / Reducing a Position

Call `ExchangeRouter.createOrder()` with:
- `orderType = MarketDecrease` (or `LimitDecrease`)
- `sizeDeltaUsd` = amount to reduce (full position size = full close)
- `initialCollateralDeltaAmount` = collateral to withdraw (0 = proportional)

### Modifying Collateral (Deposit / Withdraw)

To add collateral: create a `MarketIncrease` order with `sizeDeltaUsd = 0` and `initialCollateralDeltaAmount > 0`.

To withdraw collateral: create a `MarketDecrease` order with `sizeDeltaUsd = 0` and `initialCollateralDeltaAmount > 0`.

This is the two-step: request is submitted, then keepers execute it.

---

## Querying Funding and Borrowing Rates

### Via REST API (read-only)

The `markets/info` REST endpoint provides current rates per market:

```
GET https://arbitrum-api.gmxinfra.io/markets/info
```

Response fields per market (relevant to rates):
```json
{
  "fundingRateLong":  "string (numeric)",  // Current funding rate for longs (per hour)
  "fundingRateShort": "string (numeric)",  // Current funding rate for shorts (per hour)
  "borrowingRateLong":  "string (numeric)",  // Borrowing rate for longs
  "borrowingRateShort": "string (numeric)",  // Borrowing rate for shorts
  "netRateLong":  "string (numeric)",  // Net effective rate for longs
  "netRateShort": "string (numeric)"   // Net effective rate for shorts
}
```

All numeric fields are returned as strings to preserve precision.

### How Rates Work

- **Funding rate**: Dynamic. The dominant side (more OI) pays the weaker side. Rate changes based on long/short OI imbalance.
- **Borrowing rate**: If longs > shorts, longs pay borrowing fee. If shorts > longs, shorts pay. Rate changes based on pool utilization.
- **Reading rates on-chain**: Use `Reader.getMarketInfo()` or query DataStore keys directly via the Keys contract.

---

## REST API (Read-Only, No Authentication)

The GMX infrastructure REST API at `gmxinfra.io` is **read-only** (no trading). It requires **no API key** and **no authentication**.

### Base URLs

| Chain | Primary | Fallback 1 | Fallback 2 |
|-------|---------|------------|------------|
| Arbitrum | `https://arbitrum-api.gmxinfra.io` | `https://arbitrum-api-fallback.gmxinfra.io` | `https://arbitrum-api-fallback.gmxinfra2.io` |
| Avalanche | `https://avalanche-api.gmxinfra.io` | `https://avalanche-api-fallback.gmxinfra.io` | `https://avalanche-api-fallback.gmxinfra2.io` |
| Botanix | `https://botanix-api.gmxinfra.io` | NOT DOCUMENTED | NOT DOCUMENTED |

### Endpoints

#### `GET /ping`
Health check. Returns server status.

#### `GET /tokens`
Returns list of all tokens supported on the chain.

Response structure:
```json
{
  "tokens": [
    {
      "symbol": "ETH",
      "address": "0x...",
      "decimals": 18,
      "synthetic": true    // Optional field, present on synthetic tokens
    }
  ]
}
```

#### `GET /prices/tickers`
Returns current bid/ask prices for all tokens.

Response: Array of price objects:
```json
[
  {
    "tokenAddress": "0x...",
    "tokenSymbol": "ETH",
    "minPrice": "1900000000000000000000000000000000",   // string, 30-decimal precision
    "maxPrice": "1901000000000000000000000000000000",   // string, 30-decimal precision
    "updatedAt": 1773251933256,    // milliseconds
    "timestamp": 1773251932       // seconds
  }
]
```

#### `GET /signed_prices/latest`
Returns oracle-signed prices suitable for use in on-chain transactions. Keepers use this to bundle price signatures with order execution transactions.

Response: Array of signed price objects:
```json
[
  {
    "id": "...",
    "tokenSymbol": "ETH",
    "tokenAddress": "0x...",
    "minPrice": null,            // Deprecated
    "maxPrice": null,            // Deprecated
    "minPriceFull": "...",       // Full precision min price (string)
    "maxPriceFull": "...",       // Full precision max price (string)
    "minBlockTimestamp": 1773251900,
    "maxBlockTimestamp": 1773251933,
    "createdAt": "...",
    "oracleKeeperKey": "realtimeFeed",
    "oracleType": "realtimeFeed2",
    "oracleKeeperFetchType": "ws",   // Sourced via WebSocket
    "blob": "0x...",                 // Encoded price signature (hex)
    "isValid": true,
    "invalidReason": null,
    // Deprecated fields (all null):
    "minBlockNumber": null, "minBlockHash": null,
    "maxBlockNumber": null, "maxBlockHash": null,
    "signer": null, "signature": null,
    "signatureWithoutBlockHash": null,
    "oracleDecimals": null, "oracleKeeperRecordId": null
  }
]
```

Note: These signed prices are used by keepers — end-users/bots typically do NOT need to call this directly unless building a custom keeper.

#### `GET /markets/info`
Returns detailed market data for all GM pools.

Response: `{"markets": [...]}`

Per-market fields:
```json
{
  "name": "ETH/USD [ETH-USDC]",
  "marketToken": "0x...",          // GM token address
  "indexToken": "0x...",           // Index token (e.g. ETH)
  "longToken": "0x...",            // Long side token
  "shortToken": "0x...",           // Short side token (usually USDC)
  "isListed": true,
  "listingDate": "2023-07-04T...",
  "openInterestLong": "1234567890...",   // string, USD with 30 decimals
  "openInterestShort": "987654321...",
  "availableLiquidityLong": "...",
  "availableLiquidityShort": "...",
  "poolAmountLong": "...",
  "poolAmountShort": "...",
  "fundingRateLong": "...",        // Per-hour rate (string, fractional)
  "fundingRateShort": "...",
  "borrowingRateLong": "...",      // Per-hour rate (string, fractional)
  "borrowingRateShort": "...",
  "netRateLong": "...",
  "netRateShort": "..."
}
```

#### `GET /glvs/info`
Returns information about GLV (GMX Liquidity Vault) tokens.
Response format: NOT DOCUMENTED in detail (similar structure to markets).

#### `GET /prices/candles`
Returns OHLC candlestick data for a token.

Query parameters:
- `tokenSymbol` (required): e.g. `ETH`, `BTC`
- `period` (required): `1m`, `5m`, `15m`, `1h`, `4h`, `1d`

Response:
```json
{
  "period": "1d",
  "candles": [
    [1773187200, 2036.94, 2086.11, 2007.30, 2068.88],
    // [timestamp, open, high, low, close]
  ]
}
```
Candles are returned in descending order (most recent first). ~346 entries available for 1d period.

---

## SDK V2 (TypeScript)

The official TypeScript SDK (`@gmx-io/sdk` on npm) wraps on-chain contract calls and REST queries.

### Installation
```
npm install @gmx-io/sdk
```

### Configuration
```typescript
const sdk = new GmxSdk({
  chainId: 42161,                     // Arbitrum
  rpcUrl: "https://arb1.arbitrum.io/rpc",
  oracleUrl: "https://arbitrum-api.gmxinfra.io",
  walletClient: ...,                  // viem WalletClient
  subsquidUrl: "https://gmx.squids.live/gmx-synthetics-arbitrum:prod/api/graphql"
});
```

### Trading Methods

```typescript
// Open long position
sdk.orders.createIncreaseOrder({
  marketAddress: "0x...",
  collateralToken: "0x...",
  leverage: 5,                        // 5x
  payAmount: BigInt("1000000000"),    // Collateral amount
  allowedSlippageBps: 100,           // 1% slippage
  isLong: true,
  // ...
})

// Shorthand methods
sdk.orders.long(...)
sdk.orders.short(...)
sdk.orders.swap(...)

// Cancel order
sdk.orders.cancelOrder(orderKey: string)
```

### Query Methods

```typescript
sdk.markets.getMarketsInfo()          // Market data + token info
sdk.markets.getDailyVolumes()         // Daily volume per market
sdk.positions.getPositions({
  marketsInfoData,
  tokensData,
  start,
  end
})
sdk.account.getTradeHistory(params)   // Returns TradeAction[]
```

---

## Integration API (CoinGecko-compatible)

Separate lightweight API providing CoinGecko exchange integration data:

```
GET https://gmx-integration-cg.vercel.app/api/arbitrum/pairs
GET https://gmx-integration-cg.vercel.app/api/avalanche/pairs
```

Response per pair:
```json
{
  "ticker_id": "string",
  "base_currency": "string",
  "target_currency": "string",
  "product_type": "Perpetual",   // or "Spot"
  "last_price": 1900.00,
  "low": 1850.00,
  "high": 1950.00,
  "base_volume": 1234567.0,
  "target_volume": 9876543.0,
  "open_interest": 50000000.0
}
```

---

## Trading Fees Summary

| Fee Type | Amount |
|----------|--------|
| Open/close fee (balanced side) | 0.05% of position size |
| Open/close fee (unbalanced side) | 0.07% of position size |
| Price impact fee | Variable — increases with position size and OI imbalance |
| Borrowing fee | Variable — based on pool utilization, charged per hour |
| Funding fee | Variable — dominant side pays minority side, changes with OI ratio |
| Execution fee | Variable — gas cost estimate for keeper execution (excess refunded) |

---

## Sources

- [Trading on V2 | GMX Docs](https://docs.gmx.io/docs/trading/v2/)
- [GitHub - gmx-io/gmx-synthetics](https://github.com/gmx-io/gmx-synthetics)
- [GMX Contracts V2 | GMX Docs](https://docs.gmx.io/docs/api/contracts-v2/)
- [REST V2 | GMX Docs](https://docs.gmx.io/docs/api/rest-v2/)
- [SDK README | GMX Docs](https://docs.gmx.io/docs/sdk/)
- [GitHub - gmx-io/gmx-integration-api](https://github.com/gmx-io/gmx-integration-api)
- [arbitrum-api.gmxinfra.io/signed_prices/latest](https://arbitrum-api.gmxinfra.io/signed_prices/latest) (live endpoint)
- [arbitrum-api.gmxinfra.io/prices/tickers](https://arbitrum-api.gmxinfra.io/prices/tickers) (live endpoint)
- [arbitrum-api.gmxinfra.io/markets/info](https://arbitrum-api.gmxinfra.io/markets/info) (live endpoint)
- [Cyfrin Updraft - GMX Perpetuals Trading Course](https://updraft.cyfrin.io/courses/gmx-perpetuals-trading/)
- [GMX Whitelist Governance Thread](https://gov.gmx.io/t/whitelist-request-aifuturestradingbot-arbitrum-owned-by-my-eoa/4879/1)
