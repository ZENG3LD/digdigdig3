# Bitget API V2 Endpoints

This document describes the NEW V2 REST endpoints for Bitget API.

**CRITICAL**: V1 API was officially decommissioned on November 28, 2025. All V1 endpoints now return error: "The V1 API has been decommissioned. Please migrate to a newer version."

## Base URLs

All V2 endpoints use the same base URL:

```
Production: https://api.bitget.com
Testnet:    https://api.bitget.com (same for now)
```

## V1 vs V2 Path Comparison

### Spot Endpoints

| Endpoint Type | V1 Path | V2 Path |
|--------------|---------|---------|
| Server Time | `/api/spot/v1/public/time` | `/api/v2/public/time` |
| Ticker (Single) | `/api/spot/v1/market/ticker` | `/api/v2/spot/market/tickers` (with symbol param) |
| Tickers (All) | `/api/spot/v1/market/tickers` | `/api/v2/spot/market/tickers` |
| Orderbook | `/api/spot/v1/market/depth` | `/api/v2/spot/market/orderbook` |
| Klines/Candles | `/api/spot/v1/market/candles` | `/api/v2/spot/market/candles` |
| History Candles | `/api/spot/v1/market/history-candles` | `/api/v2/spot/market/history-candles` |
| Recent Fills | `/api/spot/v1/market/fills` | `/api/v2/spot/market/fills` |
| Fills History | `/api/spot/v1/market/fills-history` | `/api/v2/spot/market/fills-history` |
| Symbols/Products | `/api/spot/v1/public/products` | `/api/v2/spot/public/symbols` |
| Coins Info | N/A | `/api/v2/spot/public/coins` |
| Merged Depth | `/api/spot/v1/market/merge-depth` | `/api/v2/spot/market/merge-depth` |

### Spot Trading Endpoints

| Endpoint Type | V1 Path | V2 Path |
|--------------|---------|---------|
| Place Order | `/api/spot/v1/trade/orders` | `/api/v2/spot/trade/place-order` |
| Cancel Order | `/api/spot/v1/trade/cancel-order` | `/api/v2/spot/trade/cancel-order` |
| Batch Orders | `/api/spot/v1/trade/batch-orders` | `/api/v2/spot/trade/batch-orders` |
| Batch Cancel | `/api/spot/v1/trade/cancel-batch-orders` | `/api/v2/spot/trade/batch-cancel-order` |
| Cancel All (Symbol) | N/A | `/api/v2/spot/trade/cancel-symbol-order` |
| Order Info | `/api/spot/v1/trade/orderInfo` | `/api/v2/spot/trade/orderInfo` |
| Open Orders | `/api/spot/v1/trade/open-orders` | `/api/v2/spot/trade/unfilled-orders` |
| Order History | `/api/spot/v1/trade/history` | `/api/v2/spot/trade/history-orders` |
| Fills | `/api/spot/v1/trade/fills` | `/api/v2/spot/trade/fills` |
| Plan Order (Place) | N/A | `/api/v2/spot/trade/place-plan-order` |
| Plan Order (Cancel) | N/A | `/api/v2/spot/trade/cancel-plan-order` |
| Plan Order (Modify) | N/A | `/api/v2/spot/trade/modify-plan-order` |

### Spot Account Endpoints

| Endpoint Type | V1 Path | V2 Path |
|--------------|---------|---------|
| Account Assets | `/api/spot/v1/account/assets` | `/api/v2/spot/account/assets` |
| Account Info | `/api/spot/v1/account/getInfo` | `/api/v2/spot/account/info` |
| Subaccount Assets | `/api/spot/v1/account/sub-account-spot-assets` | `/api/v2/spot/account/subaccount-assets` |
| Bills | `/api/spot/v1/account/bills` | `/api/v2/spot/account/bills` |
| Transfer | `/api/spot/v1/wallet/transfer` | `/api/v2/spot/wallet/transfer` |
| Withdrawal | `/api/spot/v1/wallet/withdrawal` | `/api/v2/spot/wallet/withdrawal` |
| Deposit Address | `/api/spot/v1/wallet/deposit-address` | `/api/v2/spot/wallet/deposit-address` |
| Deposit Records | `/api/spot/v1/wallet/deposit-records` | `/api/v2/spot/wallet/deposit-records` |
| Withdrawal Records | `/api/spot/v1/wallet/withdrawal-records` | `/api/v2/spot/wallet/withdrawal-records` |

### Futures Endpoints

| Endpoint Type | V1 Path | V2 Path |
|--------------|---------|---------|
| Ticker (Single) | `/api/mix/v1/market/ticker` | `/api/v2/mix/market/ticker` |
| Tickers (All) | `/api/mix/v1/market/tickers` | `/api/v2/mix/market/tickers` |
| Orderbook | `/api/mix/v1/market/depth` | `/api/v2/mix/market/merge-depth` |
| Candles | `/api/mix/v1/market/candles` | `/api/v2/mix/market/candles` |
| History Candles | N/A | `/api/v2/mix/market/history-candles` |
| Fills | `/api/mix/v1/market/fills` | `/api/v2/mix/market/fills` |
| Contracts Info | `/api/mix/v1/market/contracts` | `/api/v2/mix/market/contracts` |
| Funding Rate | `/api/mix/v1/market/funding-rate` | `/api/v2/mix/market/current-fund-rate` |
| Funding Rate History | N/A | `/api/v2/mix/market/history-fund-rate` |
| Open Interest | N/A | `/api/v2/mix/market/open-interest` |
| Symbol Price | N/A | `/api/v2/mix/market/symbol-price` |

### Futures Trading Endpoints

| Endpoint Type | V1 Path | V2 Path |
|--------------|---------|---------|
| Place Order | `/api/mix/v1/order/placeOrder` | `/api/v2/mix/order/place-order` |
| Cancel Order | `/api/mix/v1/order/cancel-order` | `/api/v2/mix/order/cancel-order` |
| Batch Place | `/api/mix/v1/order/batch-orders` | `/api/v2/mix/order/batch-place-order` |
| Batch Cancel | `/api/mix/v1/order/cancel-batch-orders` | `/api/v2/mix/order/batch-cancel-orders` |
| Order Detail | `/api/mix/v1/order/detail` | `/api/v2/mix/order/detail` |
| Pending Orders | `/api/mix/v1/order/current` | `/api/v2/mix/order/orders-pending` |
| Order History | `/api/mix/v1/order/history` | `/api/v2/mix/order/orders-history` |
| Fills | `/api/mix/v1/order/fills` | `/api/v2/mix/order/fills` |
| Close Positions | N/A | `/api/v2/mix/order/close-positions` |
| Place TPSL | N/A | `/api/v2/mix/order/place-tpsl-order` |
| Place Plan Order | N/A | `/api/v2/mix/order/place-plan-order` |

### Futures Account/Position Endpoints

| Endpoint Type | V1 Path | V2 Path |
|--------------|---------|---------|
| Account (Single) | `/api/mix/v1/account/account` | `/api/v2/mix/account/account` |
| Accounts (All) | `/api/mix/v1/account/accounts` | `/api/v2/mix/account/accounts` |
| All Positions | `/api/mix/v1/position/allPosition` | `/api/v2/mix/position/all-position` |
| Single Position | `/api/mix/v1/position/singlePosition` | `/api/v2/mix/position/single-position` |
| Set Leverage | `/api/mix/v1/account/setLeverage` | `/api/v2/mix/account/set-leverage` |
| Set Margin | `/api/mix/v1/account/setMargin` | `/api/v2/mix/account/set-margin` |
| Set Margin Mode | `/api/mix/v1/account/setMarginMode` | `/api/v2/mix/account/set-margin-mode` |
| Set Position Mode | N/A | `/api/v2/mix/account/set-position-mode` |
| Account Bills | `/api/mix/v1/account/accountBill` | `/api/v2/mix/account/bill` |

## Complete V2 Endpoint List

### Public Endpoints (No Auth Required)

#### Common
- `GET /api/v2/public/time` - Server time
- `GET /api/v2/public/announcements` - Public announcements

#### Spot Market Data
- `GET /api/v2/spot/public/coins` - Coin info
- `GET /api/v2/spot/public/symbols` - Symbol/trading pair info
- `GET /api/v2/spot/market/vip-fee-rate` - VIP fee rates
- `GET /api/v2/spot/market/tickers` - Ticker(s) - single or all
- `GET /api/v2/spot/market/merge-depth` - Merged orderbook depth
- `GET /api/v2/spot/market/orderbook` - Full orderbook
- `GET /api/v2/spot/market/candles` - Candlestick data
- `GET /api/v2/spot/market/history-candles` - Historical candles
- `GET /api/v2/spot/market/fills` - Recent trades/fills
- `GET /api/v2/spot/market/fills-history` - Historical trades

#### Futures Market Data
- `GET /api/v2/mix/market/vip-fee-rate` - VIP fee rates
- `GET /api/v2/mix/market/ticker` - Single ticker
- `GET /api/v2/mix/market/tickers` - All tickers
- `GET /api/v2/mix/market/merge-depth` - Orderbook depth
- `GET /api/v2/mix/market/candles` - Candlestick data
- `GET /api/v2/mix/market/history-candles` - Historical candles
- `GET /api/v2/mix/market/history-index-candles` - Index price candles
- `GET /api/v2/mix/market/history-mark-candles` - Mark price candles
- `GET /api/v2/mix/market/fills` - Recent trades
- `GET /api/v2/mix/market/fills-history` - Historical trades
- `GET /api/v2/mix/market/contracts` - Contract specifications
- `GET /api/v2/mix/market/current-fund-rate` - Current funding rate
- `GET /api/v2/mix/market/history-fund-rate` - Historical funding rates
- `GET /api/v2/mix/market/open-interest` - Open interest
- `GET /api/v2/mix/market/symbol-price` - Symbol prices (mark/index/last)
- `GET /api/v2/mix/market/funding-time` - Next funding time
- `GET /api/v2/mix/market/query-position-lever` - Position leverage info

### Private Endpoints (Auth Required)

#### Spot Trading
- `POST /api/v2/spot/trade/place-order` - Place order
- `POST /api/v2/spot/trade/cancel-order` - Cancel order
- `POST /api/v2/spot/trade/batch-orders` - Batch place orders
- `POST /api/v2/spot/trade/batch-cancel-order` - Batch cancel orders
- `POST /api/v2/spot/trade/cancel-symbol-order` - Cancel all orders for symbol
- `POST /api/v2/spot/trade/cancel-replace-order` - Modify order (cancel + replace)
- `POST /api/v2/spot/trade/batch-cancel-replace-order` - Batch modify
- `GET /api/v2/spot/trade/orderInfo` - Get order details
- `GET /api/v2/spot/trade/unfilled-orders` - Open orders
- `GET /api/v2/spot/trade/history-orders` - Order history
- `GET /api/v2/spot/trade/fills` - Trade fills
- `POST /api/v2/spot/trade/place-plan-order` - Place plan/trigger order
- `POST /api/v2/spot/trade/modify-plan-order` - Modify plan order
- `POST /api/v2/spot/trade/cancel-plan-order` - Cancel plan order
- `POST /api/v2/spot/trade/batch-cancel-plan-order` - Batch cancel plan orders
- `GET /api/v2/spot/trade/current-plan-order` - Current plan orders
- `GET /api/v2/spot/trade/history-plan-order` - Plan order history
- `GET /api/v2/spot/trade/plan-sub-order` - Plan sub-orders

#### Spot Account
- `GET /api/v2/spot/account/info` - Account info
- `GET /api/v2/spot/account/assets` - Account assets/balances
- `GET /api/v2/spot/account/subaccount-assets` - Subaccount assets
- `GET /api/v2/spot/account/bills` - Account bills/transactions
- `GET /api/v2/spot/account/transfer-records` - Transfer records
- `GET /api/v2/spot/account/deduct-info` - Deduction info
- `POST /api/v2/spot/account/switch-deduct` - Switch deduction
- `GET /api/v2/spot/account/upgrade-status` - Account upgrade status
- `POST /api/v2/spot/account/upgrade` - Upgrade account

#### Spot Wallet
- `POST /api/v2/spot/wallet/transfer` - Internal transfer
- `GET /api/v2/spot/wallet/transfer-coin-info` - Transfer coin info
- `POST /api/v2/spot/wallet/subaccount-transfer` - Subaccount transfer
- `POST /api/v2/spot/wallet/withdrawal` - Withdraw
- `POST /api/v2/spot/wallet/cancel-withdrawal` - Cancel withdrawal
- `GET /api/v2/spot/wallet/deposit-address` - Get deposit address
- `GET /api/v2/spot/wallet/subaccount-deposit-address` - Subaccount deposit address
- `GET /api/v2/spot/wallet/deposit-records` - Deposit records
- `GET /api/v2/spot/wallet/subaccount-deposit-records` - Subaccount deposit records
- `GET /api/v2/spot/wallet/withdrawal-records` - Withdrawal records
- `POST /api/v2/spot/wallet/modify-deposit-account` - Modify deposit account

#### Futures Trading
- `POST /api/v2/mix/order/place-order` - Place order
- `POST /api/v2/mix/order/cancel-order` - Cancel order
- `POST /api/v2/mix/order/modify-order` - Modify order
- `POST /api/v2/mix/order/batch-place-order` - Batch place
- `POST /api/v2/mix/order/batch-cancel-orders` - Batch cancel
- `POST /api/v2/mix/order/cancel-all-orders` - Cancel all
- `POST /api/v2/mix/order/close-positions` - Close positions
- `POST /api/v2/mix/order/click-backhand` - Flash close (reverse position)
- `GET /api/v2/mix/order/detail` - Order details
- `GET /api/v2/mix/order/fills` - Order fills
- `GET /api/v2/mix/order/fill-history` - Fill history
- `GET /api/v2/mix/order/orders-pending` - Pending orders
- `GET /api/v2/mix/order/orders-history` - Order history
- `POST /api/v2/mix/order/place-plan-order` - Place plan/trigger order
- `POST /api/v2/mix/order/modify-plan-order` - Modify plan order
- `POST /api/v2/mix/order/cancel-plan-order` - Cancel plan order
- `GET /api/v2/mix/order/orders-plan-pending` - Pending plan orders
- `GET /api/v2/mix/order/orders-plan-history` - Plan order history
- `GET /api/v2/mix/order/plan-sub-order` - Plan sub-orders
- `POST /api/v2/mix/order/place-tpsl-order` - Place TP/SL order
- `POST /api/v2/mix/order/modify-tpsl-order` - Modify TP/SL order

#### Futures Account
- `GET /api/v2/mix/account/account` - Single account info
- `GET /api/v2/mix/account/accounts` - All accounts
- `GET /api/v2/mix/account/sub-account-assets` - Subaccount assets
- `GET /api/v2/mix/account/interest-history` - Interest history (USDT-M)
- `GET /api/v2/mix/account/open-count` - Estimated open count
- `GET /api/v2/mix/account/bill` - Account bills
- `GET /api/v2/mix/account/transfer-limits` - Transfer limits
- `GET /api/v2/mix/account/max-open` - Max open size
- `GET /api/v2/mix/account/liq-price` - Liquidation price
- `GET /api/v2/mix/account/isolated-symbols` - Isolated margin symbols
- `POST /api/v2/mix/account/set-leverage` - Set leverage
- `POST /api/v2/mix/account/set-margin` - Adjust margin
- `POST /api/v2/mix/account/set-margin-mode` - Set margin mode
- `POST /api/v2/mix/account/set-position-mode` - Set position mode
- `POST /api/v2/mix/account/set-auto-margin` - Set auto-margin
- `POST /api/v2/mix/account/set-asset-mode` - Set asset mode

#### Futures Position
- `GET /api/v2/mix/position/single-position` - Single position
- `GET /api/v2/mix/position/all-position` - All positions
- `GET /api/v2/mix/position/history-position` - Position history

## Key Changes from V1 to V2

1. **Path Structure**: V2 uses `/api/v2/` prefix instead of `/api/spot/v1/` or `/api/mix/v1/`

2. **Symbol Format**:
   - V1: `BTCUSDT_SPBL` (spot), `BTCUSDT_UMCBL` (futures)
   - V2: `BTCUSDT` (simple format without suffix)

3. **Endpoint Names**: More consistent naming
   - V1: `open-orders` → V2: `unfilled-orders`
   - V1: `allPosition` → V2: `all-position` (kebab-case)

4. **Pagination**:
   - V1: `pageSize` and `pageNo`
   - V2: Cursor-based with `idLessThan` and `limit`

5. **New Endpoints**: V2 adds many new endpoints not available in V1
   - Plan/trigger orders
   - Cancel-replace (modify) operations
   - History candles for index/mark prices
   - More granular position management

## Rate Limits

Rate limits are documented per endpoint:
- Most public endpoints: 20 requests/second per IP
- Most private endpoints: 10 requests/second per UID
- Batch operations: Lower rate limits (e.g., 5 req/s)

Check official docs for specific limits per endpoint.

## Product Types (Futures)

V2 uses `productType` parameter for futures:
- `USDT-FUTURES` or `usdt-futures` - USDT-margined perpetual (umcbl)
- `COIN-FUTURES` or `coin-futures` - Coin-margined perpetual (dmcbl)
- `USDC-FUTURES` or `usdc-futures` - USDC-margined perpetual (cmcbl)
- `SUSDT-FUTURES` - Simulated USDT futures (testnet)
- `SCOIN-FUTURES` - Simulated coin futures (testnet)

## Migration Priority

For current V1 connector, focus on migrating these endpoints first:

### High Priority (Market Data - Used by current connector)
1. `/api/v2/spot/market/tickers` - Ticker data
2. `/api/v2/spot/market/orderbook` - Orderbook
3. `/api/v2/spot/market/candles` - Klines
4. `/api/v2/spot/public/symbols` - Symbol info

### Medium Priority (Trading)
5. `/api/v2/spot/trade/place-order` - Place order
6. `/api/v2/spot/trade/cancel-order` - Cancel order
7. `/api/v2/spot/trade/unfilled-orders` - Open orders

### Low Priority (Account)
8. `/api/v2/spot/account/assets` - Balance
9. `/api/v2/spot/account/info` - Account info

## Sources

- [Bitget API V2 Update Guide](https://www.bitget.com/api-doc/common/release-note)
- [Bitget API Introduction](https://www.bitget.com/api-doc/common/intro)
- [Bitget V1 API Deprecation Notice](https://www.bitget.com/support/articles/12560603838361)
- [Bitget API V2 Release Announcement](https://www.bitget.com/support/articles/12560603798900)
- [Official Bitget API Docs](https://bitgetlimited.github.io/apidoc/en/spot/)
- [tiagosiebler/bitget-api GitHub](https://github.com/tiagosiebler/bitget-api)
