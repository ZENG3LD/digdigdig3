# WAVE4 Gaps — Batch 3: CEX Endpoint Coverage Analysis

> Generated: 2026-03-13
> Scope: Bitget, BingX, Crypto.com, Gemini, Phemex
> Method: Current `endpoints.rs` vs. official API documentation

---

## 1. Bitget

**Official docs:** https://www.bitget.com/api-doc/common/intro
**Base URL:** `https://api.bitget.com`
**API version:** V2

### 1.1 Spot Market Data

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Market Data | Spot symbols | `GET /api/v2/spot/public/symbols` | YES | `SpotSymbols` |
| Market Data | Spot ticker(s) | `GET /api/v2/spot/market/tickers` | YES | `SpotTicker` / `SpotAllTickers` / `SpotPrice` |
| Market Data | Spot orderbook | `GET /api/v2/spot/market/orderbook` | YES | `SpotOrderbook` |
| Market Data | Spot klines | `GET /api/v2/spot/market/candles` | YES | `SpotKlines` |
| Market Data | Spot history candles | `GET /api/v2/spot/market/history-candles` | **NO** | Paginated historical candles beyond the main endpoint |
| Market Data | Recent trades | `GET /api/v2/spot/market/fills` | **NO** | Public recent fills (not user fills) |
| Market Data | VIP fee rate | `GET /api/v2/spot/market/vip-fee-rate` | YES | `VipFeeRate` |
| Market Data | Merge depth | `GET /api/v2/spot/market/merge-depth` | **NO** | Aggregated orderbook depth |

### 1.2 Spot Trading

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Spot Trading | Place order | `POST /api/v2/spot/trade/place-order` | YES | `SpotCreateOrder` |
| Spot Trading | Cancel order | `POST /api/v2/spot/trade/cancel-order` | YES | `SpotCancelOrder` |
| Spot Trading | Modify order | `POST /api/v2/spot/trade/modify-order` | YES | `SpotModifyOrder` |
| Spot Trading | Batch place | `POST /api/v2/spot/trade/batch-orders` | YES | `SpotBatchPlaceOrders` |
| Spot Trading | Batch cancel | `POST /api/v2/spot/trade/batch-cancel-order` | YES | `SpotBatchCancelOrders` |
| Spot Trading | Cancel by symbol | `POST /api/v2/spot/trade/cancel-symbol-orders` | YES | `SpotCancelBySymbol` |
| Spot Trading | Get order info | `GET /api/v2/spot/trade/orderInfo` | YES | `SpotGetOrder` |
| Spot Trading | Open orders | `GET /api/v2/spot/trade/unfilled-orders` | YES | `SpotOpenOrders` |
| Spot Trading | Order history | `GET /api/v2/spot/trade/history-orders` | YES | `SpotAllOrders` |
| Spot Trading | User fills | `GET /api/v2/spot/trade/fills` | YES | `SpotFills` |
| Spot Trading | Place plan order | `POST /api/v2/spot/trade/place-plan-order` | **NO** | Spot trigger/plan orders |
| Spot Trading | Cancel plan order | `POST /api/v2/spot/trade/cancel-plan-order` | **NO** | Spot trigger order cancel |
| Spot Trading | Get plan orders | `GET /api/v2/spot/trade/current-plan-order` | **NO** | Current spot plan orders |
| Spot Trading | Plan order history | `GET /api/v2/spot/trade/history-plan-order` | **NO** | Historical spot plan orders |

### 1.3 Spot Account

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Spot Account | Assets | `GET /api/v2/spot/account/assets` | YES | `SpotAccounts` |
| Spot Account | Account info | `GET /api/v2/spot/account/info` | YES | `SpotAccountInfo` |
| Spot Account | Bills | `GET /api/v2/spot/account/bills` | **NO** | Trade billing history |
| Spot Account | Transfer records | `GET /api/v2/spot/account/transferRecords` | YES | `TransferHistory` |
| Spot Account | Account upgrade | `POST /api/v2/spot/account/upgrade` | **NO** | Upgrade to UTA |
| Spot Account | Upgrade status | `GET /api/v2/spot/account/upgrade-status` | **NO** | Check upgrade status |
| Spot Account | Isolated symbols | `GET /api/v2/spot/account/isolated-symbols` | **NO** | Isolated margin symbol list |

### 1.4 Futures (Mix) Market Data

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Futures Market | Contracts | `GET /api/v2/mix/market/contracts` | YES | `FuturesContracts` |
| Futures Market | Ticker | `GET /api/v2/mix/market/ticker` | YES | `FuturesTicker` |
| Futures Market | All tickers | `GET /api/v2/mix/market/tickers` | YES | `FuturesAllTickers` |
| Futures Market | Orderbook | `GET /api/v2/mix/market/merge-depth` | YES | `FuturesOrderbook` |
| Futures Market | Klines | `GET /api/v2/mix/market/candles` | YES | `FuturesKlines` |
| Futures Market | History candles | `GET /api/v2/mix/market/history-candles` | **NO** | Paginated historical candles |
| Futures Market | Mark/Index price | `GET /api/v2/mix/market/symbol-price` | **NO** | Returns mark + index + last price |
| Futures Market | Open interest | `GET /api/v2/mix/market/open-interest` | **NO** | OI for a symbol |
| Futures Market | OI limit | `GET /api/v2/mix/market/oi-limit` | **NO** | Max OI per contract |
| Futures Market | Funding rate | `GET /api/v2/mix/market/current-fund-rate` | YES | `FundingRate` |
| Futures Market | Funding rate history | `GET /api/v2/mix/market/history-fund-rate` | **NO** | Historical funding rates |
| Futures Market | History index candles | `GET /api/v2/mix/market/history-index-candles` | **NO** | Historical index candles |
| Futures Market | History mark candles | `GET /api/v2/mix/market/history-mark-candles` | **NO** | Historical mark price candles |

### 1.5 Futures (Mix) Trading

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Futures Trading | Place order | `POST /api/v2/mix/order/place-order` | YES | `FuturesCreateOrder` |
| Futures Trading | Cancel order | `POST /api/v2/mix/order/cancel-order` | YES | `FuturesCancelOrder` |
| Futures Trading | Modify order | `POST /api/v2/mix/order/modify-order` | YES | `FuturesModifyOrder` |
| Futures Trading | Batch place | `POST /api/v2/mix/order/batch-place-order` | YES | `FuturesBatchPlaceOrders` |
| Futures Trading | Batch cancel | `POST /api/v2/mix/order/batch-cancel-orders` | YES | `FuturesBatchCancelOrders` |
| Futures Trading | Cancel by symbol | `POST /api/v2/mix/order/cancel-all-orders` | YES | `FuturesCancelBySymbol` |
| Futures Trading | Close positions | `POST /api/v2/mix/order/close-positions` | YES | `FuturesClosePositions` |
| Futures Trading | Get order | `GET /api/v2/mix/order/detail` | YES | `FuturesGetOrder` |
| Futures Trading | Open orders | `GET /api/v2/mix/order/orders-pending` | YES | `FuturesOpenOrders` |
| Futures Trading | Order history | `GET /api/v2/mix/order/orders-history` | YES | `FuturesAllOrders` |
| Futures Trading | Fill history | `GET /api/v2/mix/order/fill-history` | **NO** | Trade fill history |
| Futures Trading | Plan order | `POST /api/v2/mix/order/place-plan-order` | YES | `FuturesPlanOrder` |
| Futures Trading | Pos TP/SL | `POST /api/v2/mix/order/place-tpsl-order` | YES | `FuturesPosTpSl` |
| Futures Trading | TWAP order | `POST /api/v2/mix/order/place-twap-order` | YES | `FuturesTwapOrder` |
| Futures Trading | Cancel plan order | `POST /api/v2/mix/order/cancel-plan-order` | **NO** | Cancel trigger/plan order |
| Futures Trading | Get plan orders | `GET /api/v2/mix/order/orders-plan-pending` | **NO** | Current plan orders |
| Futures Trading | Plan order history | `GET /api/v2/mix/order/orders-plan-history` | **NO** | Historical plan orders |

### 1.6 Futures (Mix) Account

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Futures Account | Single account | `GET /api/v2/mix/account/account` | YES | `FuturesAccount` |
| Futures Account | All accounts | `GET /api/v2/mix/account/accounts` | YES | `FuturesAllAccounts` |
| Futures Account | All positions | `GET /api/v2/mix/position/all-position` | YES | `FuturesPositions` |
| Futures Account | Single position | `GET /api/v2/mix/position/single-position` | YES | `FuturesPosition` |
| Futures Account | Set leverage | `POST /api/v2/mix/account/set-leverage` | YES | `FuturesSetLeverage` |
| Futures Account | Set margin mode | `POST /api/v2/mix/account/set-margin-mode` | YES | `FuturesSetMarginMode` |
| Futures Account | Set margin | `POST /api/v2/mix/account/set-margin` | YES | `FuturesSetMargin` |
| Futures Account | Max open qty | `GET /api/v2/mix/account/max-open` | **NO** | Max openable quantity |
| Futures Account | Liq price | `GET /api/v2/mix/account/liq-price` | **NO** | Estimated liquidation price |
| Futures Account | ADL rank | `GET /api/v2/mix/position/adlRank` | **NO** | Auto-deleverage ranking |
| Futures Account | Bills | `GET /api/v2/mix/account/bill` | **NO** | Account billing/fee history |

### 1.7 Margin Trading

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Cross Margin | Account assets | `GET /api/v2/margin/crossed/account/assets` | **NO** | Cross margin balances |
| Cross Margin | Borrow | `POST /api/v2/margin/crossed/account/borrow` | **NO** | Borrow funds |
| Cross Margin | Repay | `POST /api/v2/margin/crossed/account/repay` | **NO** | Repay loan |
| Cross Margin | Borrow history | `GET /api/v2/margin/crossed/borrow-history` | **NO** | Borrow records |
| Cross Margin | Repay history | `GET /api/v2/margin/crossed/repay-history` | **NO** | Repay records |
| Cross Margin | Place order | `POST /api/v2/margin/crossed/place-order` | **NO** | Cross margin orders |
| Cross Margin | Interest history | `GET /api/v2/margin/crossed/interest-history` | **NO** | Interest charges |
| Isolated Margin | Account assets | `GET /api/v2/margin/isolated/account/assets` | **NO** | Isolated margin balances |
| Isolated Margin | Borrow | `POST /api/v2/margin/isolated/account/borrow` | **NO** | Borrow funds |
| Isolated Margin | Repay | `POST /api/v2/margin/isolated/account/repay` | **NO** | Repay loan |
| Isolated Margin | Place order | `POST /api/v2/margin/isolated/place-order` | **NO** | Isolated margin orders |

### 1.8 Copy Trading

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Copy Trade (Futures Trader) | Create copy API | `POST /api/v2/copy/mix-trader/create-copy-api` | **NO** | Trader creates API key |
| Copy Trade (Futures Trader) | Query followers | `GET /api/v2/copy/mix-trader/config-query-followers` | **NO** | List followers |
| Copy Trade (Futures Trader) | Remove follower | `POST /api/v2/copy/mix-trader/config-remove-follower` | **NO** | Remove follower |
| Copy Trade (Futures Trader) | Current orders | `GET /api/v2/copy/mix-trader/order-current-track` | **NO** | Trader's open orders |
| Copy Trade (Futures Trader) | History orders | `GET /api/v2/copy/mix-trader/order-history-track` | **NO** | Trader's closed orders |
| Copy Trade (Futures Follower) | Copy settings | `POST /api/v2/copy/mix-follower/copy-settings` | **NO** | Configure copy trade |
| Copy Trade (Futures Follower) | Query settings | `GET /api/v2/copy/mix-follower/query-settings` | **NO** | Get copy settings |
| Copy Trade (Futures Follower) | Close positions | `POST /api/v2/copy/mix-follower/close-positions` | **NO** | Force close copied positions |
| Copy Trade (Futures Follower) | Query traders | `GET /api/v2/copy/mix-follower/query-traders` | **NO** | List followed traders |
| Copy Trade (Futures Follower) | Current orders | `GET /api/v2/copy/mix-follower/query-current-orders` | **NO** | Follower's open orders |
| Copy Trade (Futures Follower) | History orders | `GET /api/v2/copy/mix-follower/query-history-orders` | **NO** | Follower's closed orders |
| Copy Trade (Spot) | Spot trader endpoints | `/api/v2/copy/spot-trader/*` | **NO** | Entire spot copy trading category |
| Copy Trade (Spot) | Spot follower endpoints | `/api/v2/copy/spot-follower/*` | **NO** | Entire spot copy following category |

### 1.9 Transfers & Custody

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Transfers | Transfer | `POST /api/v2/spot/wallet/transfer` | YES | `Transfer` |
| Transfers | Transfer history | `GET /api/v2/spot/account/transferRecords` | YES | `TransferHistory` |
| Custody | Deposit address | `GET /api/v2/spot/wallet/deposit-address` | YES | `DepositAddress` |
| Custody | Withdraw | `POST /api/v2/spot/wallet/withdrawal` | YES | `Withdraw` |
| Custody | Deposit history | `GET /api/v2/spot/wallet/deposit-records` | YES | `DepositHistory` |
| Custody | Withdraw history | `GET /api/v2/spot/wallet/withdrawal-records` | YES | `WithdrawHistory` |
| Custody | Coin list | `GET /api/v2/common/coin-list` | **NO** | Supported coins and networks |
| Custody | Withdraw risk | `GET /api/v2/spot/wallet/withdrawal-risk` | **NO** | Withdrawal risk assessment |

### 1.10 Sub-Accounts

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Sub-Accounts | Create | `POST /api/v2/user/create-virtual-subaccount` | YES | `SubAccountCreate` |
| Sub-Accounts | List | `GET /api/v2/user/virtual-subaccount-list` | YES | `SubAccountList` |
| Sub-Accounts | Transfer | `POST /api/v2/user/virtual-subaccount-transfer` | YES | `SubAccountTransfer` |
| Sub-Accounts | Assets | `GET /api/v2/user/virtual-subaccount-assets` | YES | `SubAccountAssets` |
| Sub-Accounts | Batch create | `POST /api/v2/user/create-virtual-subaccount-and-apikey` | **NO** | Create sub-account + API key |
| Sub-Accounts | API key list | `GET /api/v2/user/virtual-subaccount-apikey-list` | **NO** | Sub-account API keys |

### 1.11 Earn / Broker

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Earn | Loan borrow | `POST /api/v2/earn/loan/borrow` | **NO** | Crypto loan origination |
| Earn | Hour interest | `GET /api/v2/earn/loan/public/hour-interest` | **NO** | Current loan interest rates |
| Earn | Savings products | `GET /api/v2/earn/savings/product` | **NO** | Flexible/fixed savings |
| Broker | Commission | `GET /api/v2/broker/total-commission` | **NO** | Broker total commission |
| Broker | Rebate info | `GET /api/v2/broker/rebate-info` | **NO** | Broker rebate data |
| Broker | Sub deposits/withdrawals | `GET /api/v2/broker/all-sub-deposit-withdrawal` | **NO** | Sub-account fund history |

### 1.12 WebSocket Streams

| Category | Stream | We Have? | Notes |
|----------|--------|----------|-------|
| WS Public | Ticker | **NO** | `ticker` channel |
| WS Public | Orderbook | **NO** | `books` / `books5` channel |
| WS Public | Trades | **NO** | `trade` channel |
| WS Public | Candles | **NO** | `candle{period}` channel |
| WS Public | Funding rate | **NO** | `fundRate` channel |
| WS Private | Orders | **NO** | `orders` channel |
| WS Private | Account | **NO** | `account` channel |
| WS Private | Positions | **NO** | `positions` channel |
| WS Private | Copy orders | **NO** | Copy trading order events |

---

## 2. BingX

**Official docs:** https://bingx-api.github.io/docs/
**Base URL:** `https://open-api.bingx.com`

### 2.1 Spot Market Data

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Spot Market | Symbols | `GET /openApi/spot/v1/common/symbols` | YES | `SpotSymbols` |
| Spot Market | Orderbook | `GET /openApi/spot/v1/market/depth` | YES | `SpotDepth` |
| Spot Market | Recent trades | `GET /openApi/spot/v1/market/trades` | YES | `SpotTrades` |
| Spot Market | Klines | `GET /openApi/spot/v1/market/kline` | YES | `SpotKlines` |
| Spot Market | 24hr ticker | `GET /openApi/spot/v1/ticker/24hr` | YES | `SpotTicker24hr` |
| Spot Market | Book ticker | `GET /openApi/spot/v1/ticker/bookTicker` | YES | `SpotTickerBookTicker` |
| Spot Market | Price ticker | `GET /openApi/spot/v1/ticker/price` | YES | `SpotTickerPrice` |
| Spot Market | Historical klines | `GET /openApi/spot/v1/market/kline` (with params) | YES | Covered by `SpotKlines` |

### 2.2 Spot Trading

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Spot Trading | Place/cancel order | `POST/DELETE /openApi/spot/v1/trade/order` | YES | `SpotOrder` |
| Spot Trading | Open orders | `GET /openApi/spot/v1/trade/openOrders` | YES | `SpotOpenOrders` |
| Spot Trading | Order history | `GET /openApi/spot/v1/trade/historyOrders` | YES | `SpotHistoryOrders` |
| Spot Trading | Cancel all orders | `DELETE /openApi/spot/v1/trade/cancelAllOrders` | YES | `SpotCancelAllOrders` |
| Spot Trading | Query order | `GET /openApi/spot/v1/trade/query` | **NO** | Get specific order by ID |
| Spot Trading | My trades | `GET /openApi/spot/v1/trade/myTrades` | **NO** | User trade history |
| Spot Trading | Batch place orders | `POST /openApi/spot/v1/trade/batchOrders` | **NO** | Place multiple orders at once |
| Spot Trading | OCO order | `POST /openApi/spot/v1/trade/oco/order` | **NO** | OCO order type |
| Spot Trading | Query OCO | `GET /openApi/spot/v1/trade/oco/orderList` | **NO** | Query OCO orders |

### 2.3 Spot Account

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Spot Account | Balance | `GET /openApi/spot/v1/account/balance` | YES | `SpotBalance` |
| Spot Account | Commission rate | `GET /openApi/spot/v1/account/commissionRate` | YES | `SpotCommissionRate` |
| Spot Account | Account config | `GET /openApi/spot/v1/account/config` | **NO** | Account configuration |

### 2.4 Perpetual Swap (V2) Market Data

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Swap Market | Contracts | `GET /openApi/swap/v2/quote/contracts` | YES | `SwapContracts` |
| Swap Market | Orderbook | `GET /openApi/swap/v2/quote/depth` | YES | `SwapDepth` |
| Swap Market | Recent trades | `GET /openApi/swap/v2/quote/trades` | YES | `SwapTrades` |
| Swap Market | Klines | `GET /openApi/swap/v2/quote/klines` | YES | `SwapKlines` |
| Swap Market | Ticker | `GET /openApi/swap/v2/quote/ticker` | YES | `SwapTicker` |
| Swap Market | Funding rate | `GET /openApi/swap/v2/quote/fundingRate` | YES | `SwapFundingRate` |
| Swap Market | Open interest | `GET /openApi/swap/v2/quote/openInterest` | **NO** | Contract open interest |
| Swap Market | Index price | `GET /openApi/swap/v2/quote/premiumIndex` | **NO** | Mark price + index price |
| Swap Market | Funding rate history | `GET /openApi/swap/v2/quote/fundingRateHistory` | **NO** | Historical funding rates |
| Swap Market | Mark price klines | `GET /openApi/swap/v2/quote/markPriceKlines` | **NO** | Mark price OHLC |
| Swap Market | Index price klines | `GET /openApi/swap/v2/quote/indexPriceKlines` | **NO** | Index price OHLC |

### 2.5 Perpetual Swap (V2) Trading

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Swap Trading | Place/cancel order | `POST/DELETE /openApi/swap/v2/trade/order` | YES | `SwapOrder` |
| Swap Trading | Open orders | `GET /openApi/swap/v2/trade/openOrders` | YES | `SwapOpenOrders` |
| Swap Trading | All orders | `GET /openApi/swap/v2/trade/allOrders` | YES | `SwapAllOrders` |
| Swap Trading | Cancel all | `DELETE /openApi/swap/v2/trade/allOpenOrders` | YES | `SwapCancelAllOrders` |
| Swap Trading | Batch orders | `POST /openApi/swap/v2/trade/batchOrders` | YES | `SwapBatchOrders` |
| Swap Trading | Batch cancel | `DELETE /openApi/swap/v2/trade/batchOrders` | YES | `SwapBatchCancelOrders` |
| Swap Trading | Close all positions | `POST /openApi/swap/v2/trade/closeAllPositions` | YES | `SwapCloseAllPositions` |
| Swap Trading | Amend order | `PUT /openApi/swap/v1/trade/amend` | YES | `SwapAmend` |
| Swap Trading | Query order | `GET /openApi/swap/v2/trade/order` | **NO** | Get specific order by ID |
| Swap Trading | User trades | `GET /openApi/swap/v2/trade/allFillOrders` | **NO** | Trade fill history |
| Swap Trading | Force orders | `GET /openApi/swap/v2/trade/forceOrders` | **NO** | Liquidation orders |
| Swap Trading | TWAP order | `POST /openApi/swap/v1/twap/order` | **NO** | TWAP algo order |

### 2.6 Perpetual Swap Account & Positions

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Swap Account | Balance | `GET /openApi/swap/v2/user/balance` | YES | `SwapBalance` |
| Swap Account | Commission rate | `GET /openApi/swap/v2/user/commissionRate` | YES | `SwapCommissionRate` |
| Swap Account | Income history | `GET /openApi/swap/v2/user/income` | YES | `SwapIncome` |
| Swap Account | Income export | `GET /openApi/swap/v2/user/income/export` | **NO** | Export income as file |
| Swap Account | Trading fee rate | `GET /openApi/swap/v2/user/feeTier` | **NO** | Current fee tier |
| Swap Positions | Positions | `GET /openApi/swap/v2/user/positions` | YES | `SwapPositions` |
| Swap Positions | Set leverage | `POST /openApi/swap/v2/trade/leverage` | YES | `SwapLeverage` |
| Swap Positions | Set margin type | `POST /openApi/swap/v2/trade/marginType` | YES | `SwapMarginType` |
| Swap Positions | Adjust margin | `POST /openApi/swap/v2/trade/positionMargin` | **NO** | Manually adjust position margin |
| Swap Positions | Position history | `GET /openApi/swap/v2/trade/positionHistory` | **NO** | Closed position records |

### 2.7 Standard Futures (Coin-M)

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Standard Futures | Contracts | `GET /openApi/cswap/v1/market/contracts` | **NO** | Entire category missing |
| Standard Futures | Depth | `GET /openApi/cswap/v1/quote/depth` | **NO** | Entire category missing |
| Standard Futures | Klines | `GET /openApi/cswap/v1/quote/klines` | **NO** | Entire category missing |
| Standard Futures | Ticker | `GET /openApi/cswap/v1/quote/ticker` | **NO** | Entire category missing |
| Standard Futures | Place order | `POST /openApi/cswap/v1/trade/order` | **NO** | Entire category missing |
| Standard Futures | Positions | `GET /openApi/cswap/v1/user/positions` | **NO** | Entire category missing |
| Standard Futures | Balance | `GET /openApi/cswap/v1/user/balance` | **NO** | Entire category missing |

### 2.8 Account Transfers & Custody

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Transfers | Inner transfer | `POST /openApi/api/v3/post/account/innerTransfer` | YES | `InnerTransfer` |
| Transfers | Transfer history | `GET /openApi/api/v3/get/asset/transfer` | YES | `TransferHistory` |
| Custody | Deposit address | `GET /openApi/wallets/v1/capital/deposit/address` | YES | `DepositAddress` |
| Custody | Withdraw | `POST /openApi/wallets/v1/capital/withdraw/apply` | YES | `Withdraw` |
| Custody | Deposit history | `GET /openApi/api/v3/capital/deposit/hisrec` | YES | `DepositHistory` |
| Custody | Withdraw history | `GET /openApi/api/v3/capital/withdraw/history` | YES | `WithdrawHistory` |
| Custody | Coin info | `GET /openApi/wallets/v1/capital/config/getall` | **NO** | Supported coins/networks |

### 2.9 Sub-Accounts

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Sub-Accounts | Create | `POST /openApi/subAccount/v1/create` | YES | `SubAccountCreate` |
| Sub-Accounts | List | `GET /openApi/subAccount/v1/list` | YES | `SubAccountList` |
| Sub-Accounts | Transfer | `POST /openApi/subAccount/v1/transfer` | YES | `SubAccountTransfer` |
| Sub-Accounts | Assets | `GET /openApi/subAccount/v1/assets` | YES | `SubAccountAssets` |
| Sub-Accounts | Deposit address | `GET /openApi/subAccount/v1/depositAddress` | **NO** | Sub-account deposit address |
| Sub-Accounts | Deposit records | `GET /openApi/subAccount/v1/deposit/records` | **NO** | Sub-account deposits |

### 2.10 Copy Trading

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Copy Trading | Follower positions | `GET /openApi/copyTrading/v1/follower/positions` | **NO** | Entire category missing |
| Copy Trading | Trader profits | `GET /openApi/copyTrading/v1/trader/profits` | **NO** | Entire category missing |
| Copy Trading | Copy settings | `POST /openApi/copyTrading/v1/follower/settings` | **NO** | Entire category missing |
| Copy Trading | Stop copying | `POST /openApi/copyTrading/v1/follower/stopCopying` | **NO** | Entire category missing |

### 2.11 WebSocket Streams

| Category | Stream | We Have? | Notes |
|----------|--------|----------|-------|
| WS Public | Ticker | **NO** | `market.{symbol}@ticker` |
| WS Public | Orderbook | **NO** | `market.{symbol}@depth` |
| WS Public | Trades | **NO** | `market.{symbol}@trade` |
| WS Public | Klines | **NO** | `market.{symbol}@kline_{interval}` |
| WS Private | Account updates | **NO** | `listenKey`-based user data stream |
| WS Private | Order updates | **NO** | Order execution reports |
| WS Private | Position updates | **NO** | Position change events |

---

## 3. Crypto.com

**Official docs:** https://exchange-docs.crypto.com/exchange/v1/rest-ws/index.html
**Base URL:** `https://api.crypto.com/exchange/v1`

### 3.1 Public Market Data

| Category | Endpoint | Method | We Have? | Notes |
|----------|----------|--------|----------|-------|
| Market Data | Get instruments | `public/get-instruments` | YES | `GetInstruments` |
| Market Data | Get book | `public/get-book` | YES | `GetBook` |
| Market Data | Get candlestick | `public/get-candlestick` | YES | `GetCandlestick` |
| Market Data | Get trades | `public/get-trades` | YES | `GetTrades` |
| Market Data | Get tickers | `public/get-tickers` | YES | `GetTickers` |
| Market Data | Get valuations | `public/get-valuations` | YES | `GetValuations` |
| Market Data | Get expired settlement price | `public/get-expired-settlement-price` | **NO** | Settled futures prices |
| Market Data | Get insurance | `public/get-insurance` | **NO** | Insurance fund balance |
| Market Data | Get announcements | `public/get-announcements` | **NO** | Exchange announcements |
| Market Data | Get risk parameters | `public/get-risk-parameters` | **NO** | Exchange risk parameters |

### 3.2 Trading

| Category | Endpoint | Method | We Have? | Notes |
|----------|----------|--------|----------|-------|
| Trading | Create order | `private/create-order` | YES | `CreateOrder` |
| Trading | Create order list | `private/create-order-list` | YES | `CreateOrderList` |
| Trading | Cancel order list | `private/cancel-order-list` | YES | `CancelOrderList` |
| Trading | Amend order | `private/amend-order` | YES | `AmendOrder` |
| Trading | Cancel order | `private/cancel-order` | YES | `CancelOrder` |
| Trading | Cancel all orders | `private/cancel-all-orders` | YES | `CancelAllOrders` |
| Trading | Close position | `private/close-position` | YES | `ClosePosition` |
| Trading | Get open orders | `private/get-open-orders` | YES | `GetOpenOrders` |
| Trading | Get order detail | `private/get-order-detail` | YES | `GetOrderDetail` |
| Trading | Get order history | `private/get-order-history` | YES | `GetOrderHistory` |
| Trading | Get user trades | `private/get-trades` | YES | `GetUserTrades` |

### 3.3 Advanced Order Management

| Category | Endpoint | Method | We Have? | Notes |
|----------|----------|--------|----------|-------|
| Advanced Orders | Create order | `private/advanced/create-order` | YES | `AdvancedCreateOrder` |
| Advanced Orders | Create OCO | `private/advanced/create-oco` | YES | `AdvancedCreateOco` |
| Advanced Orders | Cancel OCO | `private/advanced/cancel-oco` | **NO** | Cancel OCO order |
| Advanced Orders | Create OTO | `private/advanced/create-oto` | YES | `AdvancedCreateOto` |
| Advanced Orders | Cancel OTO | `private/advanced/cancel-oto` | **NO** | Cancel OTO order |
| Advanced Orders | Create OTOCO | `private/advanced/create-otoco` | YES | `AdvancedCreateOtoco` |
| Advanced Orders | Cancel OTOCO | `private/advanced/cancel-otoco` | **NO** | Cancel OTOCO order |
| Advanced Orders | Cancel advanced order | `private/advanced/cancel-order` | **NO** | Cancel any advanced order |
| Advanced Orders | Cancel all advanced | `private/advanced/cancel-all-orders` | **NO** | Cancel all advanced orders |
| Advanced Orders | Get open advanced orders | `private/advanced/get-open-orders` | **NO** | List advanced open orders |
| Advanced Orders | Get advanced order detail | `private/advanced/get-order-detail` | **NO** | Advanced order detail |
| Advanced Orders | Get advanced order history | `private/advanced/get-order-history` | **NO** | Advanced order history |

### 3.4 Account

| Category | Endpoint | Method | We Have? | Notes |
|----------|----------|--------|----------|-------|
| Account | User balance | `private/user-balance` | YES | `UserBalance` |
| Account | User balance history | `private/user-balance-history` | **NO** | Historical balance snapshots |
| Account | Get accounts | `private/get-accounts` | YES | `GetAccounts` |
| Account | Get fee rate | `private/get-fee-rate` | YES | `GetFeeRate` |
| Account | Get instrument fee rate | `private/get-instrument-fee-rate` | YES | `GetInstrumentFeeRate` |
| Account | Get transactions | `private/get-transactions` | YES | `GetTransactions` |
| Account | Get account settings | `private/get-account-settings` | **NO** | Account configuration |
| Account | Change account settings | `private/change-account-settings` | **NO** | Modify account settings |

### 3.5 Positions

| Category | Endpoint | Method | We Have? | Notes |
|----------|----------|--------|----------|-------|
| Positions | Get positions | `private/get-positions` | YES | `GetPositions` |
| Positions | Change account leverage | `private/change-account-leverage` | YES | `ChangeAccountLeverage` |
| Positions | Change isolated margin leverage | `private/change-isolated-margin-leverage` | YES | `ChangeIsolatedMarginLeverage` |
| Positions | Create isolated margin transfer | `private/create-isolated-margin-transfer` | **NO** | Transfer margin to/from isolated position |

### 3.6 Wallet / Custody

| Category | Endpoint | Method | We Have? | Notes |
|----------|----------|--------|----------|-------|
| Wallet | Get deposit address | `private/get-deposit-address` | YES | `GetDepositAddress` |
| Wallet | Create withdrawal | `private/create-withdrawal` | YES | `CreateWithdrawal` |
| Wallet | Get deposit history | `private/get-deposit-history` | YES | `GetDepositHistory` |
| Wallet | Get withdrawal history | `private/get-withdrawal-history` | YES | `GetWithdrawalHistory` |
| Wallet | Get currency networks | `private/get-currency-networks` | **NO** | Supported coins and networks |

### 3.7 Sub-Accounts

| Category | Endpoint | Method | We Have? | Notes |
|----------|----------|--------|----------|-------|
| Sub-Accounts | Create subaccount | `private/subaccount/create` | YES | `SubAccountCreate` |
| Sub-Accounts | List subaccounts | `private/subaccount/get-subaccounts` | YES | `SubAccountList` |
| Sub-Accounts | Transfer | `private/create-subaccount-transfer` | YES | `SubAccountTransfer` (note: method name differs from mapping) |
| Sub-Accounts | Get subaccount balances | `private/subaccount/get-balances` | YES | `SubAccountGetBalances` |

> **Note:** `SubAccountTransfer` maps to `private/create-subaccount-transfer` (top-level), but the enum incorrectly has path `private/subaccount/transfer`. Verify this.

### 3.8 Fiat Wallet

| Category | Endpoint | Method | We Have? | Notes |
|----------|----------|--------|----------|-------|
| Fiat | Fiat deposit info | `private/fiat/fiat-deposit-info` | **NO** | Entire category missing |
| Fiat | Fiat deposit history | `private/fiat/fiat-deposit-history` | **NO** | Entire category missing |
| Fiat | Fiat withdraw history | `private/fiat/fiat-withdraw-history` | **NO** | Entire category missing |
| Fiat | Fiat create withdraw | `private/fiat/fiat-create-withdraw` | **NO** | Entire category missing |
| Fiat | Fiat transaction quota | `private/fiat/fiat-transaction-quota` | **NO** | Entire category missing |
| Fiat | Fiat transaction limit | `private/fiat/fiat-transaction-limit` | **NO** | Entire category missing |
| Fiat | Fiat get bank accounts | `private/fiat/fiat-get-bank-accounts` | **NO** | Entire category missing |

### 3.9 Staking

| Category | Endpoint | Method | We Have? | Notes |
|----------|----------|--------|----------|-------|
| Staking | Stake | `private/staking/stake` | **NO** | Entire category missing |
| Staking | Unstake | `private/staking/unstake` | **NO** | Entire category missing |
| Staking | Get staking position | `private/staking/get-staking-position` | **NO** | Entire category missing |
| Staking | Get staking instruments | `private/staking/get-staking-instruments` | **NO** | Entire category missing |
| Staking | Get open stake | `private/staking/get-open-stake` | **NO** | Entire category missing |
| Staking | Get stake history | `private/staking/get-stake-history` | **NO** | Entire category missing |
| Staking | Get reward history | `private/staking/get-reward-history` | **NO** | Entire category missing |
| Staking | Convert | `private/staking/convert` | **NO** | Entire category missing |
| Staking | Get open convert | `private/staking/get-open-convert` | **NO** | Entire category missing |
| Staking | Get convert history | `private/staking/get-convert-history` | **NO** | Entire category missing |
| Staking | Get conversion rate (public) | `public/staking/get-conversion-rate` | **NO** | Entire category missing |

### 3.10 WebSocket Streams

| Category | Stream | We Have? | Notes |
|----------|--------|----------|-------|
| WS Market | Orderbook | **NO** | `book.{instrument_name}` |
| WS Market | Ticker | **NO** | `ticker.{instrument_name}` |
| WS Market | Trades | **NO** | `trade.{instrument_name}` |
| WS Market | Candlestick | **NO** | `candlestick.{interval}.{instrument_name}` |
| WS User | Balance updates | **NO** | `user.balance` |
| WS User | Order updates | **NO** | `user.order.{instrument_name}` |
| WS User | Trade updates | **NO** | `user.trade.{instrument_name}` |
| WS User | Position updates | **NO** | `user.position.{instrument_name}` |

---

## 4. Gemini

**Official docs:** https://docs.gemini.com/rest-api/
**Base URLs:** `https://api.gemini.com` (production), `https://api.sandbox.gemini.com` (sandbox)

### 4.1 Market Data (Public)

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Market Data | Symbols | `GET /v1/symbols` | YES | `Symbols` |
| Market Data | Symbol details | `GET /v1/symbols/details/{symbol}` | YES | `SymbolDetails` |
| Market Data | Ticker v1 | `GET /v1/pubticker/{symbol}` | YES | `Ticker` |
| Market Data | Ticker v2 | `GET /v2/ticker/{symbol}` | YES | `TickerV2` |
| Market Data | Orderbook | `GET /v1/book/{symbol}` | YES | `OrderBook` |
| Market Data | Trades | `GET /v1/trades/{symbol}` | YES | `Trades` |
| Market Data | Candles | `GET /v2/candles/{symbol}/{time_frame}` | YES | `Candles` |
| Market Data | Derivative candles | `GET /v2/derivatives/candles/{symbol}/{time_frame}` | YES | `DerivativeCandles` |
| Market Data | Price feed | `GET /v1/pricefeed` | YES | `PriceFeed` |
| Market Data | Network info | `GET /v1/network/{token}` | YES | `NetworkInfo` |
| Market Data | Funding amount | `GET /v1/fundingamount/{symbol}` | YES | `FundingAmount` |
| Market Data | Fee promos | `GET /v1/feepromos` | YES | `FeePromos` |
| Market Data | Risk stats | `GET /v1/riskstats/{symbol}` | YES | `RiskStats` |
| Market Data | FX rate | `GET /v2/fxrate/{symbol}/{timestamp}` | **NO** | Historical FX rates |
| Market Data | Funding report file | `GET /v1/fundingamountreport/records.xlsx` | **NO** | Download funding history as XLSX |

### 4.2 Trading (Private Orders)

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Orders | New order | `POST /v1/order/new` | YES | `NewOrder` |
| Orders | Cancel order | `POST /v1/order/cancel` | YES | `CancelOrder` |
| Orders | Cancel all orders | `POST /v1/order/cancel/all` | YES | `CancelAllOrders` |
| Orders | Cancel session orders | `POST /v1/order/cancel/session` | YES | `CancelSessionOrders` |
| Orders | Order status | `POST /v1/order/status` | YES | `OrderStatus` |
| Orders | Active orders | `POST /v1/orders` | YES | `ActiveOrders` |
| Orders | Past trades | `POST /v1/mytrades` | YES | `PastTrades` |
| Orders | Order history | `POST /v1/orders/history` | **NO** | Full order history (not just trades) |
| Orders | Trading volume | `POST /v1/tradevolume` | YES | `TradingVolume` |
| Orders | Notional volume | `POST /v1/notionalvolume` | YES | `NotionalVolume` |
| Orders | Wrap order | `POST /v1/wrap/{symbol}` | YES | `WrapOrder` |
| Orders | Session heartbeat | `POST /v1/heartbeat` | **NO** | Keep session alive |

### 4.3 Fund Management (Private)

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Fund Management | Balances | `POST /v1/balances` | YES | `Balances` |
| Fund Management | Notional balances | `POST /v1/notionalbalances/{currency}` | YES | `NotionalBalances` |
| Fund Management | Staking balances | `POST /v1/balances/staking` | YES | `StakingBalances` |
| Fund Management | Deposit addresses | `POST /v1/addresses/{network}` | YES | `DepositAddresses` |
| Fund Management | New deposit address | `POST /v1/deposit/{network}/newAddress` | YES | `NewDepositAddress` |
| Fund Management | Withdraw | `POST /v1/withdraw/{currency}` | YES | `Withdraw` |
| Fund Management | Withdraw fee estimate | `POST /v1/withdraw/{currency}/feeEstimate` | YES | `WithdrawFeeEstimate` |
| Fund Management | Transfers | `POST /v1/transfers` | YES | `Transfers` |
| Fund Management | Custody fee transfers | `POST /v1/custodyaccountfees` | **NO** | Custody account fee records |
| Fund Management | Account transfer | `POST /v1/account/transfer/{currency}` | YES | `AccountTransfer` |
| Fund Management | Transactions | `POST /v1/transactions` | YES | `Transactions` |
| Fund Management | Payment methods | `POST /v1/payments/methods` | YES | `PaymentMethods` |
| Fund Management | Add bank | `POST /v1/payments/addbank` | **NO** | Register US bank account |
| Fund Management | Add bank CAD | `POST /v1/payments/addbank/cad` | **NO** | Register Canadian bank |
| Fund Management | Approved addresses | `POST /v1/approvedAddresses/account/{network}` | **NO** | List approved withdrawal addresses |
| Fund Management | New approved address | `POST /v1/approvedAddresses/{network}/request` | **NO** | Add to approved address list |
| Fund Management | Remove approved address | `POST /v1/approvedAddresses/{network}/remove` | **NO** | Remove from approved list |

### 4.4 Account Administration

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Administration | Account detail | `POST /v1/account` | YES | `AccountDetail` |
| Administration | Create account | `POST /v1/account/create` | **NO** | Create sub-account in group |
| Administration | Rename account | `POST /v1/account/rename` | **NO** | Rename account/shortname |
| Administration | List accounts | `POST /v1/account/list` | **NO** | List all accounts in group |
| Administration | Roles | `POST /v1/roles` | **NO** | Get current API key role |

### 4.5 Positions & Derivatives

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Positions | Positions | `POST /v1/positions` | YES | `Positions` |
| Positions | Margin | `POST /v1/margin` | YES | `Margin` |
| Positions | Margin account | `POST /v1/margin/account` | YES | `MarginAccount` |
| Positions | Margin rates | `POST /v1/margin/rates` | YES | `MarginRates` |
| Positions | Margin order preview | `POST /v1/margin/order/preview` | YES | `MarginOrderPreview` |
| Positions | Funding payments | `POST /v1/perpetuals/fundingPayment` | YES | `FundingPayments` |
| Positions | Funding payment report | `GET /v1/perpetuals/fundingpaymentreport/records.json` | YES | `FundingPaymentReport` |

### 4.6 Staking

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Staking | Stake | `POST /v1/staking/stake` | **NO** | Entire category missing |
| Staking | Unstake | `POST /v1/staking/unstake` | **NO** | Entire category missing |
| Staking | Staking history | `POST /v1/staking/history` | **NO** | Entire category missing |
| Staking | Staking rates | `GET /v1/staking/rates` | **NO** | Entire category missing |
| Staking | Staking rewards | `POST /v1/staking/rewards` | **NO** | Entire category missing |

### 4.7 Clearing

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Clearing | New clearing order | `POST /v1/clearing/new` | **NO** | Entire category missing |
| Clearing | Get clearing order | `POST /v1/clearing/status` | **NO** | Entire category missing |
| Clearing | Cancel clearing order | `POST /v1/clearing/cancel` | **NO** | Entire category missing |
| Clearing | Confirm clearing order | `POST /v1/clearing/confirm` | **NO** | Entire category missing |
| Clearing | List clearing orders | `POST /v1/clearing/list` | **NO** | Entire category missing |
| Clearing | List clearing brokers | `POST /v1/clearing/broker/list` | **NO** | Entire category missing |
| Clearing | New broker order | `POST /v1/clearing/broker/new` | **NO** | Entire category missing |
| Clearing | List clearing trades | `POST /v1/clearing/trades` | **NO** | Entire category missing |

### 4.8 WebSocket Streams

| Category | Stream | We Have? | Notes |
|----------|--------|----------|-------|
| WS Market (v2) | Multi-market data | `wss://api.gemini.com/v2/marketdata` | **Partial** | URL defined but no stream logic |
| WS Private | Order events | `wss://api.gemini.com/v1/order/events` | **Partial** | URL defined but no stream logic |
| WS Market | Level 2 orderbook | `book_changes` event type | **NO** | Real-time book updates |
| WS Market | Trades | `trade` event type | **NO** | Live trade stream |
| WS Market | Candles | `candles_{interval}` event type | **NO** | Live candle updates |
| WS Private | Order status | `order_pending`, `order_filled`, etc. | **NO** | Order lifecycle events |

---

## 5. Phemex

**Official docs:** https://phemex-docs.github.io/
**Base URL:** `https://api.phemex.com`

### 5.1 General / Public

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| General | Server time | `GET /public/time` | YES | `ServerTime` |
| General | Products | `GET /public/products` | YES | `Products` |

### 5.2 Spot Market Data

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Spot Market | Orderbook | `GET /md/orderbook` | YES | `SpotOrderbook` |
| Spot Market | Recent trades | `GET /md/trade` | YES | `SpotTrades` |
| Spot Market | 24hr ticker | `GET /md/spot/ticker/24hr` | YES | `SpotTicker24h` |
| Spot Market | Klines | `GET /exchange/public/md/v2/kline` | YES | `SpotKlines` |
| Spot Market | All tickers | `GET /md/spot/ticker/24hr/all` | **NO** | All spot tickers at once |

### 5.3 Spot Trading

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Spot Trading | Place order | `PUT /spot/orders` | YES | `SpotCreateOrder` |
| Spot Trading | Amend order | `PUT /spot/orders` (with amend params) | YES | `SpotAmendOrder` |
| Spot Trading | Cancel order | `DELETE /spot/orders` | YES | `SpotCancelOrder` |
| Spot Trading | Cancel all orders | `DELETE /spot/orders/all` | YES | `SpotCancelAllOrders` |
| Spot Trading | Open orders | `GET /spot/orders/active` | YES | `SpotOpenOrders` |
| Spot Trading | Get order by ID | `GET /spot/orders/active?clOrdID=` | **NO** | Query specific open order |
| Spot Trading | Closed orders | `GET /exchange/spot/order` | **NO** | Historical closed spot orders |
| Spot Trading | Trade history | `GET /exchange/spot/order/trades` | **NO** | User's spot trade fills |
| Spot Trading | Spot PnL | `GET /api-data/spots/spotPnlHistory` | **NO** | Spot profit/loss history |

### 5.4 Contract Market Data (COIN-M Perpetual)

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Contract Market | Orderbook | `GET /md/orderbook` | YES | `ContractOrderbook` |
| Contract Market | Recent trades | `GET /md/trade` | YES | `ContractTrades` |
| Contract Market | 24hr ticker | `GET /md/ticker/24hr` | YES | `ContractTicker24h` |
| Contract Market | Klines | `GET /exchange/public/md/v2/kline` | YES | `ContractKlines` |
| Contract Market | Funding rate history | `GET /api-data/public/data/funding-rate-history` | YES | `FundingRateHistory` |
| Contract Market | Full orderbook | `GET /md/fullbook` | **NO** | Complete orderbook snapshot |
| Contract Market | Fee rate | `GET /api-data/futures/fee-rate` | **NO** | Contract fee rates |
| Contract Market | Index price | `GET /md/ticker/24hr` (field) | YES (partial) | Included in ticker response |

### 5.5 Contract Trading (COIN-M Perpetual)

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Contract Trading | Place order | `POST /orders` | YES | `ContractCreateOrder` |
| Contract Trading | Amend order | `PUT /orders/replace` | YES | `ContractAmendOrder` |
| Contract Trading | Cancel order | `DELETE /orders` | YES | `ContractCancelOrder` |
| Contract Trading | Cancel all | `DELETE /orders/all` | YES | `ContractCancelAllOrders` |
| Contract Trading | Open orders | `GET /orders/activeList` | YES | `ContractOpenOrders` |
| Contract Trading | Closed orders | `GET /exchange/order/list` | YES | `ContractClosedOrders` |
| Contract Trading | Get order | `GET /exchange/order` | YES | `ContractGetOrder` |
| Contract Trading | Get trades | `GET /exchange/order/trade` | YES | `ContractGetTrades` |
| Contract Trading | Funding fees | `GET /api-data/futures/funding-fees` | **NO** | Personal funding fee history |
| Contract Trading | Trade account detail | `GET /api-data/futures/v2/tradeAccountDetail` | **NO** | Detailed account ledger |

### 5.6 Hedged Contract Trading (USD-M Perpetual)

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Hedged | Place order | `POST /g-orders` | YES | `HedgedCreateOrder` |
| Hedged | Amend order | `PUT /g-orders/replace` | YES | `HedgedAmendOrder` |
| Hedged | Cancel order | `DELETE /g-orders/cancel` | YES | `HedgedCancelOrder` |
| Hedged | Bulk cancel | `DELETE /g-orders` | **NO** | Cancel multiple by ID list |
| Hedged | Cancel all | `DELETE /g-orders/all` | **NO** | Cancel all hedged orders |
| Hedged | Open orders | `GET /g-orders/activeList` | **NO** | Active hedged orders |
| Hedged | Closed orders | `GET /exchange/order/v2/orderList` | **NO** | Hedged order history |
| Hedged | Trades | `GET /exchange/order/v2/tradingList` | **NO** | Hedged trade history |
| Hedged | Account positions | `GET /g-accounts/accountPositions` | **NO** | Hedged account + positions |
| Hedged | Positions | `GET /g-accounts/positions` | **NO** | Hedged positions with PnL |
| Hedged | Switch pos mode | `PUT /g-positions/switch-pos-mode-sync` | **NO** | OneWay vs. Hedged mode |
| Hedged | Set leverage | `PUT /g-positions/leverage` | **NO** | Hedged position leverage |
| Hedged | Assign balance | `POST /g-positions/assign` | **NO** | Assign margin to position |

### 5.7 Contract Account (COIN-M)

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Contract Account | Account + positions | `GET /accounts/accountPositions` | YES | `ContractAccount` |
| Contract Account | Positions w/ unrealized PnL | `GET /accounts/positions` | **NO** | Separate positions endpoint |
| Contract Account | Set leverage | `PUT /positions/leverage` | YES | `SetLeverage` |
| Contract Account | Set risk limit | `PUT /positions/riskLimit` | YES | `SetRiskLimit` |
| Contract Account | Assign balance | `POST /positions/assign` | YES | `AssignBalance` |

### 5.8 Margin Trading

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Margin | Place order | `PUT /margin/orders` | **NO** | Entire category missing |
| Margin | Cancel order | `DELETE /margin/orders` | **NO** | Entire category missing |
| Margin | Cancel all orders | `DELETE /margin/orders/all` | **NO** | Entire category missing |
| Margin | Open orders | `GET /margin/orders/activeList` | **NO** | Entire category missing |
| Margin | Closed orders | `GET /exchange/margin/order/list` | **NO** | Entire category missing |
| Margin | Trade history | `GET /exchange/margin/order/trades` | **NO** | Entire category missing |
| Margin | Wallets | `GET /margin/wallets` | **NO** | Entire category missing |
| Margin | Borrow | `POST /margin/borrow` | **NO** | Entire category missing |
| Margin | Repay | `POST /margin/repay` | **NO** | Entire category missing |
| Margin | Borrow history | `GET /margin/borrowHistory` | **NO** | Entire category missing |
| Margin | Repay history | `GET /margin/repayHistory` | **NO** | Entire category missing |
| Margin | Interest history | `GET /margin/interestHistory` | **NO** | Entire category missing |

### 5.9 Transfers & Custody

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Transfers | Transfer (spot↔futures) | `POST /assets/transfer` | YES | `Transfer` |
| Transfers | Transfer history | `GET /assets/transfer` | YES | `TransferHistory` |
| Transfers | Universal transfer | `POST /assets/universal-transfer` | YES | `SubAccountTransfer` |
| Transfers | Spot balance summary | `GET /spot/wallets` | YES | `SpotWallets` |
| Custody | Deposit address | `GET /exchange/wallets/v2/depositAddress` | YES | `DepositAddress` |
| Custody | Withdraw | `POST /exchange/wallets/createWithdraw` | YES | `Withdraw` |
| Custody | Deposit list | `GET /exchange/wallets/depositList` | YES | `DepositList` |
| Custody | Withdraw list | `GET /exchange/wallets/withdrawList` | YES | `WithdrawList` |
| Custody | Confirm withdraw | `POST /exchange/wallets/confirmWithdraw` | **NO** | Two-factor withdraw confirmation |
| Custody | Cancel withdraw | `DELETE /exchange/wallets/cancelWithdraw` | **NO** | Cancel pending withdrawal |

### 5.10 Sub-Accounts

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Sub-Accounts | Create sub-account | `POST /phemex-user/users/children` | YES | `SubAccountCreate` |
| Sub-Accounts | List sub-accounts | `GET /phemex-user/users/children` | YES | `SubAccountList` |
| Sub-Accounts | Sub-account transfer | `POST /assets/universal-transfer` | YES | `SubAccountTransfer` |
| Sub-Accounts | Sub-account balance | `GET /phemex-user/wallets/v2/tradeAccountBalance` | **NO** | Query sub-account balances |

### 5.11 WebSocket Streams

| Category | Stream | We Have? | Notes |
|----------|--------|----------|-------|
| WS | Orderbook | **NO** | `orderbook.subscribe` |
| WS | Trades | **NO** | `trade.subscribe` |
| WS | Klines | **NO** | `kline.subscribe` |
| WS | 24hr ticker | **NO** | `spot_market24h.subscribe` / `market24h.subscribe` |
| WS | Mark price | **NO** | `perp_market24h_pack_p.subscribe` |
| WS (Auth) | Orders | **NO** | `aop.subscribe` |
| WS (Auth) | Positions | **NO** | `aop.subscribe` (position events) |
| WS (Auth) | Account | **NO** | `aop.subscribe` (account events) |

---

## Summary: Missing Endpoint Counts

| Exchange | Total Endpoints in File | Key Missing Categories |
|----------|------------------------|------------------------|
| **Bitget** | 48 endpoints | Margin trading (entire), Copy trading (entire), History candles, Plan order management, Mark/index price, Open interest, Account bills |
| **BingX** | 39 endpoints | Standard Futures / Coin-M (entire), Copy trading (entire), Order queries by ID, Batch spot orders, OCO orders, Open interest, Mark price klines |
| **Crypto.com** | 37 endpoints | Advanced order cancel/query methods (8 missing), Fiat wallet (entire), Staking (entire), Currency networks, Balance history, Account settings |
| **Gemini** | 34 endpoints | Staking (entire), Clearing (entire), Bank account management, Approved addresses, Account list/create/rename, Order history, Session heartbeat |
| **Phemex** | 24 endpoints | Margin trading (entire), Hedged contract full management (8 missing), Spot order history/trades, Positions w/ PnL, Withdraw confirm/cancel |

---

## Priority Gaps (High Impact)

### P1 — Core Trading Completeness

| Exchange | Gap | Why Important |
|----------|-----|---------------|
| Bitget | Futures plan order cancel + query | Can't manage conditional orders after placing |
| Bitget | `symbol-price` (mark/index) | Required for accurate P&L and liquidation price display |
| Bitget | `open-interest` | Standard market data metric |
| BingX | Standard Futures (Coin-M) entire API | Entire product category missing — `/openApi/cswap/v1/*` |
| BingX | Swap order query by ID | Can't look up a specific order |
| Crypto.com | Advanced order cancel/query (8 endpoints) | Can't manage OCO/OTO/OTOCO after placement |
| Phemex | Hedged contract open orders + history | Active USD-M order management broken |
| Phemex | Margin trading (entire category) | Separate margin product line not covered at all |

### P2 — Account Management

| Exchange | Gap | Why Important |
|----------|-----|---------------|
| Bitget | Account bills | Audit trail for trading fees |
| Gemini | Account list/create/rename | Multi-account group management |
| Gemini | Approved addresses | Withdrawal security management |
| Phemex | Spot trade history | Can't retrieve filled spot trades |
| Crypto.com | Balance history | Historical snapshot needed for reporting |

### P3 — Institutional / Specialized

| Exchange | Gap | Why Important |
|----------|-----|---------------|
| Bitget | Copy trading (all) | Large product line — 12+ endpoints |
| BingX | Copy trading (all) | Popular BingX feature — 4+ endpoints |
| Crypto.com | Staking (all) | 11 endpoints, separate product line |
| Gemini | Clearing (all) | Institutional OTC — 8 endpoints |
| Gemini | Staking (all) | 5 staking endpoints |
| Crypto.com | Fiat wallet (all) | Fiat on/off-ramp — 7 endpoints |

---

## Sources

- [Bitget API Documentation](https://www.bitget.com/api-doc/common/intro)
- [Bitget Futures API](https://www.bitget.com/api-doc/contract/intro)
- [Bitget Copy Trading API](https://www.bitget.com/api-doc/copytrading/intro)
- [Bitget Changelog](https://www.bitget.com/api-doc/common/changelog)
- [BingX API Documentation](https://bingx-api.github.io/docs/)
- [Crypto.com Exchange API v1](https://exchange-docs.crypto.com/exchange/v1/rest-ws/index.html)
- [Gemini REST API - Market Data](https://docs.gemini.com/rest/market-data)
- [Gemini REST API - Orders](https://docs.gemini.com/rest/orders)
- [Gemini REST API - Fund Management](https://docs.gemini.com/rest/fund-management)
- [Gemini REST API - Account Administration](https://docs.gemini.com/rest/account-administration)
- [Gemini REST API - Staking](https://docs.gemini.com/rest/staking)
- [Gemini REST API - Clearing](https://docs.gemini.com/rest/clearing)
- [Phemex API Reference](https://phemex-docs.github.io/)
- [Phemex Hedged Perpetual API (GitHub)](https://github.com/phemex/phemex-api-docs/blob/master/Public-Hedged-Perpetual-API.md)
- [Phemex Contract API (GitHub)](https://github.com/phemex/phemex-api-docs/blob/master/Public-Contract-API-en.md)
- [Phemex Spot API (GitHub)](https://github.com/phemex/phemex-api-docs/blob/master/Public-Spot-API-en.md)
