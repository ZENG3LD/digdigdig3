# WAVE4 Endpoint Gap Analysis — Batch 1

**Exchanges:** Binance, Bybit, OKX, Kraken, Coinbase
**Connector path:** `digdigdig3/src/crypto/cex/{name}/endpoints.rs`
**Date:** 2026-03-13

---

## How to Read This Report

- **YES** — variant exists as a named enum member in `endpoints.rs`
- **NO** — not present; would be a new enum variant
- **PARTIAL** — path exists but not all HTTP methods/variations are modeled

---

---

# 1. BINANCE

**Current enum variants (from `endpoints.rs`):**

Ping, ServerTime, SpotPrice, SpotOrderbook, SpotKlines, SpotTicker, SpotExchangeInfo,
SpotCreateOrder, SpotCancelOrder, SpotCancelAllOrders, SpotGetOrder, SpotOpenOrders,
SpotAllOrders, SpotOcoOrder, SpotOtocoOrder, SpotTradeFee, SpotAlgoTwap, SpotAccount,
FuturesPrice, FuturesOrderbook, FuturesKlines, FuturesTicker, FuturesExchangeInfo,
FundingRate, FuturesCreateOrder, FuturesCancelOrder, FuturesCancelAllOrders,
FuturesGetOrder, FuturesOpenOrders, FuturesAllOrders, FuturesAmendOrder,
FuturesBatchOrders, FuturesAlgoOrder, FuturesAlgoTwap, FuturesAccount,
FuturesPositions, FuturesSetLeverage, FuturesSetMarginType, FuturesPositionMargin,
FuturesCommissionRate, SpotListenKey, FuturesListenKey, AssetTransfer,
AssetTransferHistory, DepositAddress, Withdraw, DepositHistory, WithdrawHistory,
SubAccountCreate, SubAccountList, SubAccountTransfer, SubAccountAssets

---

## 1.1 Spot — Market Data

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Market Data | GET /api/v3/ping | YES | `Ping` |
| Market Data | GET /api/v3/time | YES | `ServerTime` |
| Market Data | GET /api/v3/exchangeInfo | YES | `SpotExchangeInfo` |
| Market Data | GET /api/v3/depth | YES | `SpotOrderbook` |
| Market Data | GET /api/v3/trades | NO | Recent public trades |
| Market Data | GET /api/v3/historicalTrades | NO | Older trades (requires API key) |
| Market Data | GET /api/v3/aggTrades | NO | Compressed/aggregate trades |
| Market Data | GET /api/v3/klines | YES | `SpotKlines` |
| Market Data | GET /api/v3/uiKlines | NO | UI-friendly klines (same data, different display) |
| Market Data | GET /api/v3/avgPrice | NO | Current average price |
| Market Data | GET /api/v3/ticker/24hr | YES | `SpotTicker` |
| Market Data | GET /api/v3/ticker/tradingDay | NO | Rolling window price stats |
| Market Data | GET /api/v3/ticker/price | YES | `SpotPrice` |
| Market Data | GET /api/v3/ticker/bookTicker | NO | Best bid/ask for symbol |
| Market Data | GET /api/v3/ticker | NO | Custom rolling window ticker |
| Market Data | GET /api/v3/referencePrice | NO | Reference price for STP |
| Market Data | GET /api/v3/referencePrice/calculation | NO | Reference price calculation method |

## 1.2 Spot — Trading

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Trading | POST /api/v3/order | YES | `SpotCreateOrder` |
| Trading | POST /api/v3/order/test | NO | Test new order (no execution) |
| Trading | DELETE /api/v3/order | YES | `SpotCancelOrder` |
| Trading | DELETE /api/v3/openOrders | YES | `SpotCancelAllOrders` |
| Trading | POST /api/v3/order/cancelReplace | NO | Cancel and replace in one call |
| Trading | PUT /api/v3/order/amend/keepPriority | NO | Amend order while keeping queue priority |
| Trading | POST /api/v3/orderList/oco | YES | `SpotOcoOrder` |
| Trading | POST /api/v3/orderList/oto | NO | One-Triggers-Other |
| Trading | POST /api/v3/orderList/otoco | YES | `SpotOtocoOrder` |
| Trading | POST /api/v3/orderList/opo | NO | One-Pending-Other (new 2024) |
| Trading | POST /api/v3/orderList/opoco | NO | One-Pending-OCO (new 2024) |
| Trading | DELETE /api/v3/orderList | NO | Cancel entire order list |
| Trading | POST /api/v3/sor/order | NO | Smart Order Routing (SOR) |
| Trading | POST /api/v3/sor/order/test | NO | Test SOR order |

## 1.3 Spot — Account

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Account | GET /api/v3/account | YES | `SpotAccount` |
| Account | GET /api/v3/order | YES | `SpotGetOrder` |
| Account | GET /api/v3/openOrders | YES | `SpotOpenOrders` |
| Account | GET /api/v3/allOrders | YES | `SpotAllOrders` |
| Account | GET /api/v3/orderList | NO | Query a specific order list |
| Account | GET /api/v3/allOrderList | NO | All order lists |
| Account | GET /api/v3/openOrderList | NO | Open order lists |
| Account | GET /api/v3/myTrades | NO | Account trade history (fills) |
| Account | GET /api/v3/rateLimit/order | NO | Query unfilled order count |
| Account | GET /api/v3/myPreventedMatches | NO | STP prevented matches |
| Account | GET /api/v3/myAllocations | NO | SOR allocations |
| Account | GET /api/v3/account/commission | NO | Commission rates |
| Account | GET /api/v3/order/amendments | NO | Order amendment history |
| Account | GET /api/v3/myFilters | NO | Relevant filters for account |
| Account | GET /sapi/v1/asset/tradeFee | YES | `SpotTradeFee` |
| Account | POST /api/v3/userDataStream | YES | `SpotListenKey` |

## 1.4 Futures (USDT-M) — Market Data

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Futures MD | GET /fapi/v1/exchangeInfo | YES | `FuturesExchangeInfo` |
| Futures MD | GET /fapi/v1/depth | YES | `FuturesOrderbook` |
| Futures MD | GET /fapi/v1/trades | NO | Recent trades |
| Futures MD | GET /fapi/v1/historicalTrades | NO | Historical trades |
| Futures MD | GET /fapi/v1/aggTrades | NO | Aggregate trades |
| Futures MD | GET /fapi/v1/klines | YES | `FuturesKlines` |
| Futures MD | GET /fapi/v1/continuousKlines | NO | Continuous contract klines |
| Futures MD | GET /fapi/v1/indexPriceKlines | NO | Index price klines |
| Futures MD | GET /fapi/v1/markPriceKlines | NO | Mark price klines |
| Futures MD | GET /fapi/v1/premiumIndexKlines | NO | Premium index klines |
| Futures MD | GET /fapi/v1/premiumIndex | NO | Mark price and funding rate |
| Futures MD | GET /fapi/v1/fundingRate | YES | `FundingRate` |
| Futures MD | GET /fapi/v1/fundingInfo | NO | Funding rate info per symbol |
| Futures MD | GET /fapi/v1/ticker/24hr | YES | `FuturesTicker` |
| Futures MD | GET /fapi/v1/ticker/price | YES | `FuturesPrice` |
| Futures MD | GET /fapi/v1/ticker/bookTicker | NO | Best bid/ask |
| Futures MD | GET /fapi/v1/openInterest | NO | Current open interest |
| Futures MD | GET /futures/data/openInterestHist | NO | Historical open interest |
| Futures MD | GET /futures/data/topLongShortAccountRatio | NO | Top trader long/short ratio (accounts) |
| Futures MD | GET /futures/data/topLongShortPositionRatio | NO | Top trader long/short ratio (positions) |
| Futures MD | GET /futures/data/globalLongShortAccountRatio | NO | Global long/short ratio |
| Futures MD | GET /futures/data/takerlongshortRatio | NO | Taker buy/sell volume |
| Futures MD | GET /fapi/v1/lvtKlines | NO | Leveraged Token klines |
| Futures MD | GET /fapi/v1/indexInfo | NO | Composite index info |
| Futures MD | GET /fapi/v1/assetIndex | NO | Asset index (multi-asset mode) |
| Futures MD | GET /fapi/v1/constituents | NO | Index constituents |

## 1.5 Futures (USDT-M) — Trading

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Futures Trade | POST /fapi/v1/order | YES | `FuturesCreateOrder` |
| Futures Trade | POST /fapi/v1/order/test | NO | Test new order |
| Futures Trade | DELETE /fapi/v1/order | YES | `FuturesCancelOrder` |
| Futures Trade | DELETE /fapi/v1/allOpenOrders | YES | `FuturesCancelAllOrders` |
| Futures Trade | DELETE /fapi/v1/batchOrders | PARTIAL | `FuturesBatchOrders` (place only; cancel uses same path) |
| Futures Trade | POST /fapi/v1/batchOrders | YES | `FuturesBatchOrders` |
| Futures Trade | PATCH /fapi/v1/order | YES | `FuturesAmendOrder` |
| Futures Trade | PATCH /fapi/v1/batchOrders | NO | Batch amend orders |
| Futures Trade | GET /fapi/v1/order | YES | `FuturesGetOrder` |
| Futures Trade | GET /fapi/v1/openOrders | YES | `FuturesOpenOrders` |
| Futures Trade | GET /fapi/v1/allOrders | YES | `FuturesAllOrders` |
| Futures Trade | GET /fapi/v1/userTrades | NO | Trade/fill history |
| Futures Trade | POST /fapi/v1/um/order/submit/algo | NO | New algo conditional order (post-migration) |
| Futures Trade | DELETE /fapi/v1/um/order/cancel/algo | NO | Cancel algo order |
| Futures Trade | GET /fapi/v1/um/order/get/algo | NO | Query algo order |
| Futures Trade | POST /sapi/v1/algo/futures/newOrderTwap | YES | `FuturesAlgoTwap` |

## 1.6 Futures (USDT-M) — Account

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Futures Acct | GET /fapi/v2/account | YES | `FuturesAccount` |
| Futures Acct | GET /fapi/v3/account | NO | Account info V3 (newer) |
| Futures Acct | GET /fapi/v2/balance | NO | Balance V2 |
| Futures Acct | GET /fapi/v3/balance | NO | Balance V3 (newest) |
| Futures Acct | GET /fapi/v2/positionRisk | YES | `FuturesPositions` |
| Futures Acct | POST /fapi/v1/leverage | YES | `FuturesSetLeverage` |
| Futures Acct | POST /fapi/v1/marginType | YES | `FuturesSetMarginType` |
| Futures Acct | POST /fapi/v1/positionMargin | YES | `FuturesPositionMargin` |
| Futures Acct | GET /fapi/v1/commissionRate | YES | `FuturesCommissionRate` |
| Futures Acct | GET /fapi/v1/income | NO | Income history |
| Futures Acct | GET /fapi/v1/notionalAndLeverageBrackets | NO | Notional brackets |
| Futures Acct | GET /fapi/v1/adlQuantile | NO | ADL quantile |
| Futures Acct | GET /fapi/v1/forceOrders | NO | Liquidation orders |
| Futures Acct | GET /fapi/v1/rateLimit/order | NO | Query order rate limit |
| Futures Acct | GET /fapi/v1/multiAssetsMargin | NO | Multi-assets margin mode |
| Futures Acct | GET /fapi/v1/positionMode | NO | Hedge vs one-way mode |
| Futures Acct | POST /fapi/v1/positionMode | NO | Change position mode |
| Futures Acct | GET /fapi/v1/symbolConfig | NO | Symbol configuration |
| Futures Acct | GET /fapi/v1/accountConfig | NO | Account configuration |
| Futures Acct | GET /fapi/v1/userTrades | NO | Account trade list |
| Futures Acct | POST /fapi/v1/listenKey | YES | `FuturesListenKey` |

## 1.7 Wallet / Asset Management

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Wallet | POST /sapi/v1/asset/transfer | YES | `AssetTransfer` |
| Wallet | GET /sapi/v1/asset/transfer | YES | `AssetTransferHistory` |
| Wallet | GET /sapi/v1/capital/deposit/address | YES | `DepositAddress` |
| Wallet | POST /sapi/v1/capital/withdraw/apply | YES | `Withdraw` |
| Wallet | GET /sapi/v1/capital/deposit/hisrec | YES | `DepositHistory` |
| Wallet | GET /sapi/v1/capital/withdraw/history | YES | `WithdrawHistory` |
| Wallet | GET /sapi/v1/capital/config/getall | NO | All coins info (networks, fees) |
| Wallet | GET /sapi/v1/asset/assetDividend | NO | Asset dividend record |
| Wallet | GET /sapi/v1/asset/dribblet | NO | Dust log |
| Wallet | POST /sapi/v1/asset/dust | NO | Convert dust to BNB |
| Wallet | POST /sapi/v1/asset/dust-btc | NO | Convert dust to BTC |
| Wallet | GET /sapi/v1/asset/get-funding-asset | NO | Funding wallet balance |
| Wallet | GET /sapi/v1/asset/wallet/balance | NO | All wallet balances |
| Wallet | GET /sapi/v1/asset/custody/transfer-history | NO | Custodial transfer history |
| Wallet | GET /sapi/v1/asset/ledger-transfer/cloud-mining/queryByPage | NO | Cloud mining payment history |

## 1.8 Sub-Accounts

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Sub-Acct | POST /sapi/v1/sub-account/virtualSubAccount | YES | `SubAccountCreate` |
| Sub-Acct | GET /sapi/v1/sub-account/list | YES | `SubAccountList` |
| Sub-Acct | POST /sapi/v1/sub-account/universalTransfer | YES | `SubAccountTransfer` |
| Sub-Acct | GET /sapi/v3/sub-account/assets | YES | `SubAccountAssets` |
| Sub-Acct | GET /sapi/v1/sub-account/transfer/subUserHistory | NO | Sub-account transfer history |
| Sub-Acct | GET /sapi/v1/sub-account/futures/account | NO | Sub-account futures account summary |
| Sub-Acct | GET /sapi/v1/sub-account/margin/account | NO | Sub-account margin account summary |
| Sub-Acct | GET /sapi/v2/sub-account/futures/accountSummary | NO | Futures account summary V2 |
| Sub-Acct | GET /sapi/v2/sub-account/margin/accountSummary | NO | Margin account summary V2 |
| Sub-Acct | POST /sapi/v1/sub-account/margin/enable | NO | Enable margin for sub-account |
| Sub-Acct | POST /sapi/v1/sub-account/futures/enable | NO | Enable futures for sub-account |
| Sub-Acct | GET /sapi/v1/sub-account/status | NO | Sub-account status list |
| Sub-Acct | POST /sapi/v1/sub-account/spotSummary | NO | Spot asset summary |

## 1.9 Margin Trading

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Margin | POST /sapi/v1/margin/borrow-repay | NO | Borrow or repay (unified endpoint) |
| Margin | GET /sapi/v1/margin/borrow-repay | NO | Borrow/repay history |
| Margin | GET /sapi/v1/margin/account | NO | Cross margin account details |
| Margin | GET /sapi/v1/margin/isolated/account | NO | Isolated margin account info |
| Margin | POST /sapi/v1/margin/isolated/account | NO | Enable/disable isolated margin |
| Margin | POST /sapi/v1/margin/max-leverage | NO | Adjust cross margin max leverage |
| Margin | GET /sapi/v1/margin/maxBorrowable | NO | Query max borrowable |
| Margin | GET /sapi/v1/margin/maxTransferable | NO | Query max transferable |
| Margin | GET /sapi/v1/margin/interestHistory | NO | Interest history |
| Margin | GET /sapi/v1/margin/myTrades | NO | Margin trade history |
| Margin | POST /sapi/v1/margin/order | NO | Place margin order |
| Margin | DELETE /sapi/v1/margin/order | NO | Cancel margin order |
| Margin | GET /sapi/v1/margin/order | NO | Query margin order |
| Margin | GET /sapi/v1/margin/openOrders | NO | Open margin orders |
| Margin | GET /sapi/v1/margin/allOrders | NO | All margin orders |
| Margin | POST /sapi/v1/margin/order/oco | NO | Margin OCO order |
| Margin | GET /sapi/v1/margin/orderList | NO | Margin OCO list |
| Margin | GET /sapi/v1/margin/cross/marginData | NO | Cross margin fee data |
| Margin | GET /sapi/v1/margin/isolated/marginData | NO | Isolated margin fee data |
| Margin | GET /sapi/v1/margin/allPairs | NO | All isolated margin pairs |
| Margin | GET /sapi/v1/margin/allAssets | NO | All margin assets |
| Margin | GET /sapi/v1/margin/pair | NO | Isolated margin pair info |
| Margin | GET /sapi/v1/margin/isolatedMarginData | NO | Isolated margin tier data |
| Margin | GET /sapi/v1/margin/capitalFlow | NO | Cross isolated margin capital flow |

## 1.10 Simple Earn / Staking

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Simple Earn | GET /sapi/v1/simple-earn/flexible/list | NO | Flexible product list |
| Simple Earn | GET /sapi/v1/simple-earn/locked/list | NO | Locked product list |
| Simple Earn | POST /sapi/v1/simple-earn/flexible/subscribe | NO | Subscribe flexible |
| Simple Earn | POST /sapi/v1/simple-earn/locked/subscribe | NO | Subscribe locked |
| Simple Earn | POST /sapi/v1/simple-earn/flexible/redeem | NO | Redeem flexible |
| Simple Earn | POST /sapi/v1/simple-earn/locked/redeem | NO | Redeem locked |
| Simple Earn | GET /sapi/v1/simple-earn/flexible/position | NO | Flexible position |
| Simple Earn | GET /sapi/v1/simple-earn/locked/position | NO | Locked position |
| Simple Earn | GET /sapi/v1/simple-earn/account | NO | Flexible + locked summary |
| Simple Earn | GET /sapi/v1/simple-earn/flexible/history/subscriptionRecord | NO | Flexible subscribe history |
| Simple Earn | GET /sapi/v1/simple-earn/locked/history/subscriptionRecord | NO | Locked subscribe history |
| Staking | POST /sapi/v1/staking/stake | NO | Stake asset (on-chain) |
| Staking | POST /sapi/v1/staking/unstake | NO | Unstake asset |
| Staking | GET /sapi/v1/staking/asset-info | NO | Staking asset info |
| Staking | GET /sapi/v1/staking/stakingRecord | NO | Staking history |
| ETH Staking | POST /sapi/v2/eth-staking/eth/stake | NO | Stake ETH |
| ETH Staking | POST /sapi/v2/eth-staking/eth/redeem | NO | Redeem WBETH |
| ETH Staking | GET /sapi/v2/eth-staking/account | NO | ETH staking account |
| ETH Staking | GET /sapi/v1/eth-staking/wbeth/history/unwrapHistory | NO | WBETH unwrap history |
| SOL Staking | POST /sapi/v1/sol-staking/sol/stake | NO | Stake SOL |
| SOL Staking | POST /sapi/v1/sol-staking/sol/redeem | NO | Redeem BNSOL |
| SOL Staking | GET /sapi/v1/sol-staking/sol/history/stakingHistory | NO | SOL staking history |
| SOL Staking | GET /sapi/v1/sol-staking/sol/history/redemptionHistory | NO | SOL redemption history |

## 1.11 Convert / Swap

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Convert | GET /sapi/v1/convert/exchangeInfo | NO | Convertible coin pairs |
| Convert | GET /sapi/v1/convert/assetInfo | NO | Asset details for convert |
| Convert | POST /sapi/v1/convert/getQuote | NO | Request quote |
| Convert | POST /sapi/v1/convert/acceptQuote | NO | Accept and execute quote |
| Convert | GET /sapi/v1/convert/tradeFlow | NO | Convert trade history |
| Convert | GET /sapi/v1/convert/orderStatus | NO | Query order status |
| Convert Dust | POST /sapi/v1/asset/convert-transfer | NO | Convert dust to BNB/BTC/ETH |
| Convert Dust | GET /sapi/v1/asset/convert-transfer/queryByPage | NO | Convert-transfer history |

## 1.12 Copy Trading / WebSocket Streams

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Copy Trading | (no dedicated REST endpoint in Spot API) | N/A | Handled via spot order endpoints with copy trade params |
| WS Streams | POST /api/v3/userDataStream | YES | `SpotListenKey` |
| WS Streams | PUT /api/v3/userDataStream | NO | Keep-alive listen key |
| WS Streams | DELETE /api/v3/userDataStream | NO | Close listen key |
| WS Streams | POST /fapi/v1/listenKey | YES | `FuturesListenKey` |
| WS Streams | PUT /fapi/v1/listenKey | NO | Keep-alive futures listen key |
| WS Streams | DELETE /fapi/v1/listenKey | NO | Close futures listen key |

---

---

# 2. BYBIT

**Current enum variants (from `endpoints.rs`):**

Ticker, Orderbook, Klines, Symbols, RecentTrades, ServerTime,
Balance, AccountInfo,
PlaceOrder, CancelOrder, CancelAllOrders, OrderStatus, OpenOrders, OrderHistory,
Positions, SetLeverage, SetMarginMode, AddMargin, TpSlMode, FundingRate,
AmendOrder, BatchPlaceOrders, BatchCancelOrders,
FeeRate,
InterTransfer, TransferHistory,
DepositAddress, Withdraw, DepositHistory, WithdrawHistory,
CreateSubMember, ListSubMembers, UniversalTransfer, SubAccountBalance

---

## 2.1 Market Data

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Market Data | GET /v5/market/time | YES | `ServerTime` |
| Market Data | GET /v5/market/kline | YES | `Klines` |
| Market Data | GET /v5/market/mark-price-kline | NO | Mark price klines |
| Market Data | GET /v5/market/index-price-kline | NO | Index price klines |
| Market Data | GET /v5/market/premium-index-price-kline | NO | Premium index klines |
| Market Data | GET /v5/market/instruments-info | YES | `Symbols` |
| Market Data | GET /v5/market/orderbook | YES | `Orderbook` |
| Market Data | GET /v5/market/orderbook (RPI) | NO | RPI orderbook (restricted price improvement) |
| Market Data | GET /v5/market/tickers | YES | `Ticker` |
| Market Data | GET /v5/market/funding/history | YES | `FundingRate` |
| Market Data | GET /v5/market/recent-trade | YES | `RecentTrades` |
| Market Data | GET /v5/market/open-interest | NO | Open interest |
| Market Data | GET /v5/market/historical-volatility | NO | Historical volatility (options) |
| Market Data | GET /v5/market/insurance | NO | Insurance pool info |
| Market Data | GET /v5/market/risk-limit | NO | Risk limit info |
| Market Data | GET /v5/market/delivery-price | NO | Delivery price |
| Market Data | GET /v5/market/delivery-price (new) | NO | New delivery price endpoint |
| Market Data | GET /v5/market/account-ratio | NO | Long/short ratio |
| Market Data | GET /v5/market/index-price-kline (components) | NO | Index price components |
| Market Data | GET /v5/market/order-price-limit | NO | Order price limit |
| Market Data | GET /v5/market/adl-quantile | NO | ADL alert |
| Market Data | GET /v5/market/fee-group | NO | Fee group structure |

## 2.2 Trade

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Trade | POST /v5/order/create | YES | `PlaceOrder` |
| Trade | POST /v5/order/amend | YES | `AmendOrder` |
| Trade | POST /v5/order/cancel | YES | `CancelOrder` |
| Trade | GET /v5/order/realtime | YES | `OpenOrders` / `OrderStatus` |
| Trade | POST /v5/order/cancel-all | YES | `CancelAllOrders` |
| Trade | GET /v5/order/history | YES | `OrderHistory` |
| Trade | GET /v5/order/execution | NO | Trade/fill history |
| Trade | POST /v5/order/create-batch | YES | `BatchPlaceOrders` |
| Trade | POST /v5/order/amend-batch | NO | Batch amend orders |
| Trade | POST /v5/order/cancel-batch | YES | `BatchCancelOrders` |
| Trade | GET /v5/order/spot-borrow-quota | NO | Spot borrow quota for margin |
| Trade | POST /v5/order/dcp | NO | Disconnect-cancel-protect (DCP) |
| Trade | POST /v5/order/pre-check-order | NO | Pre-check order feasibility |

## 2.3 Position

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Position | GET /v5/position/list | YES | `Positions` |
| Position | POST /v5/position/set-leverage | YES | `SetLeverage` |
| Position | POST /v5/position/switch-isolated | YES | `SetMarginMode` |
| Position | POST /v5/position/add-margin | YES | `AddMargin` |
| Position | POST /v5/position/trading-stop | YES | `TpSlMode` (TP/SL) |
| Position | POST /v5/position/set-tpsl-mode | NO | Set TP/SL mode (full vs partial) |
| Position | POST /v5/position/switch-mode | NO | Switch position mode (hedge vs one-way) |
| Position | GET /v5/position/closed-pnl | NO | Closed PnL history |
| Position | POST /v5/position/move-positions | NO | Move positions between UIDs (UTA) |
| Position | GET /v5/position/move-history | NO | Move positions history |
| Position | POST /v5/position/confirm-pending-mmr | NO | Confirm pending MMR |

## 2.4 Account

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Account | GET /v5/account/wallet-balance | YES | `Balance` |
| Account | POST /v5/account/upgrade-to-uta | NO | Upgrade to unified trading account |
| Account | GET /v5/account/borrow-history | NO | Borrow history |
| Account | POST /v5/account/repay-liability | NO | Repay liabilities |
| Account | GET /v5/account/collateral-info | NO | Collateral coin info |
| Account | GET /v5/account/greeks | NO | Portfolio Greeks (options) |
| Account | GET /v5/account/info | YES | `AccountInfo` |
| Account | GET /v5/account/transaction-log | NO | Transaction log |
| Account | POST /v5/account/set-margin-mode | NO | Set margin mode (isolated/cross/portfolio) |
| Account | GET /v5/account/smp-group | NO | SMP group info |
| Account | POST /v5/account/set-hedging-mode | NO | Set hedging mode |
| Account | GET /v5/account/fee-rate | YES | `FeeRate` |

## 2.5 Asset

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Asset | GET /v5/asset/exchange-record | NO | Exchange/conversion records |
| Asset | GET /v5/asset/delivery-record | NO | Delivery record (options/futures) |
| Asset | GET /v5/asset/settlement-record | NO | USDC session settlement |
| Asset | GET /v5/asset/coin-info | NO | All coin info (networks, fees) |
| Asset | GET /v5/asset/transfer/query-account-coins-balance | YES | `SubAccountBalance` |
| Asset | GET /v5/asset/transfer/query-asset-info | NO | Single account asset info |
| Asset | GET /v5/asset/transfer/query-transferable-coin-list | NO | Transferable coins list |
| Asset | POST /v5/asset/transfer/inter-transfer | YES | `InterTransfer` |
| Asset | GET /v5/asset/transfer/query-inter-transfer-list | YES | `TransferHistory` |
| Asset | GET /v5/asset/transfer/query-sub-member-list | NO | Query sub UIDs |
| Asset | POST /v5/asset/transfer/universal-transfer | YES | `UniversalTransfer` |
| Asset | GET /v5/asset/transfer/query-universal-transfer-list | NO | Universal transfer history |
| Asset | GET /v5/asset/deposit/query-address | YES | `DepositAddress` |
| Asset | GET /v5/asset/deposit/query-record | YES | `DepositHistory` |
| Asset | GET /v5/asset/deposit/query-sub-member-record | NO | Sub-account deposit records |
| Asset | GET /v5/asset/deposit/query-internal-record | NO | Internal deposit records |
| Asset | GET /v5/asset/coin/query-info | NO | Coin query info |
| Asset | POST /v5/asset/withdraw/create | YES | `Withdraw` |
| Asset | GET /v5/asset/withdraw/query-record | YES | `WithdrawHistory` |
| Asset | POST /v5/asset/withdraw/cancel | NO | Cancel withdrawal |
| Asset | GET /v5/asset/withdraw/vasp/list | NO | VASP list for withdraw |

## 2.6 User

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| User | POST /v5/user/create-sub-member | YES | `CreateSubMember` |
| User | GET /v5/user/query-sub-members | YES | `ListSubMembers` |
| User | POST /v5/user/sign-agreement | NO | Sign agreement |
| User | GET /v5/user/aff-customer-info | NO | Affiliate customer info |
| User | GET /v5/user/del-submember | NO | Delete sub-member |
| User | POST /v5/user/set-sub-member-quick-login | NO | Quick login for sub-member |
| User | POST /v5/user/frozen-sub-member | NO | Freeze/unfreeze sub-member |

## 2.7 Spot Margin Trade (UTA)

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Spot Margin | GET /v5/spot-margin-trade/data | NO | VIP margin data |
| Spot Margin | POST /v5/spot-margin-trade/switch-mode | NO | Enable/disable spot margin |
| Spot Margin | GET /v5/spot-margin-trade/state | NO | Spot margin state |
| Spot Margin | POST /v5/spot-margin-trade/set-leverage | NO | Set spot margin leverage |
| Spot Margin | GET /v5/spot-margin-trade/borrow-history | NO | Margin borrow history |
| Spot Margin | GET /v5/spot-margin-trade/repay-history | NO | Margin repay history |
| Spot Margin | GET /v5/spot-margin-trade/interest-rate | NO | Interest rate history |
| Spot Margin | GET /v5/spot-margin-trade/open-loan | NO | Open loans |
| Spot Margin | GET /v5/spot-margin-trade/obligation | NO | Liabilities |

## 2.8 Earn

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Earn | GET /v5/earn/product-info | NO | Earn product info |
| Earn | POST /v5/earn/create-order | NO | Stake/redeem |
| Earn | GET /v5/earn/order-history | NO | Stake/redeem history |
| Earn | GET /v5/earn/position | NO | Staked positions |
| Earn | GET /v5/earn/yield-history | NO | Yield history |
| Earn | GET /v5/earn/hourly-yield | NO | Hourly yield history |

## 2.9 Spread / RFQ / Broker

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Spread | (various /v5/spread endpoints) | NO | Spread trading |
| RFQ | (various /v5/rfq endpoints) | NO | RFQ block trading |
| Broker | GET /v5/broker/earnings-info | NO | Broker earnings |
| Broker | GET /v5/broker/account-info | NO | Broker account info |
| Broker | GET /v5/broker/sub-member | NO | Broker sub-members |
| Broker | POST /v5/broker/sub-member/create | NO | Create broker sub-member |

## 2.10 Crypto Loan

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Crypto Loan | GET /v5/crypto-loan/ongoing-orders | NO | Active loan orders |
| Crypto Loan | GET /v5/crypto-loan/borrow-history | NO | Borrow history |
| Crypto Loan | GET /v5/crypto-loan/repayment-history | NO | Repayment history |
| Crypto Loan | GET /v5/crypto-loan/ltv-adjustment-history | NO | LTV adjustment history |
| Crypto Loan | GET /v5/crypto-loan/collateral-data | NO | Collateral coin data |

---

---

# 3. OKX

**Current enum variants (from `endpoints.rs`):**

ServerTime, Ticker, AllTickers, Orderbook, OrderbookFull, Klines, HistoryKlines, Trades,
HistoryTrades, Instruments, PlaceOrder, PlaceBatchOrders, CancelOrder, CancelBatchOrders,
CancelAllAfter, AmendOrder, GetOrder, OpenOrders, OrderHistory, OrderHistoryArchive,
AlgoOrder, AlgoOrderCancel, AlgoOpenOrders, Balance, AssetBalances, AccountConfig,
Positions, PositionHistory, MaxOrderSize, SetLeverage, GetLeverage, SetPositionMode,
FundingRate, FundingRateHistory, AssetTransfer, TransferState, AssetBills, DepositAddress,
Withdrawal, DepositHistory, WithdrawalHistory, SubAccountCreate, SubAccountList,
SubAccountTransfer, SubAccountBalances

---

## 3.1 Market Data / Public Data

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Market Data | GET /api/v5/public/time | YES | `ServerTime` |
| Market Data | GET /api/v5/public/instruments | YES | `Instruments` |
| Market Data | GET /api/v5/market/ticker | YES | `Ticker` |
| Market Data | GET /api/v5/market/tickers | YES | `AllTickers` |
| Market Data | GET /api/v5/market/books | YES | `Orderbook` |
| Market Data | GET /api/v5/market/books-full | YES | `OrderbookFull` |
| Market Data | GET /api/v5/market/candles | YES | `Klines` |
| Market Data | GET /api/v5/market/history-candles | YES | `HistoryKlines` |
| Market Data | GET /api/v5/market/trades | YES | `Trades` |
| Market Data | GET /api/v5/market/history-trades | YES | `HistoryTrades` |
| Market Data | GET /api/v5/market/mark-price | NO | Mark price |
| Market Data | GET /api/v5/market/open-interest | NO | Open interest |
| Market Data | GET /api/v5/market/index-tickers | NO | Index tickers |
| Market Data | GET /api/v5/market/index-candles | NO | Index price candles |
| Market Data | GET /api/v5/market/index-components | NO | Index components |
| Market Data | GET /api/v5/market/exchange-rate | NO | Exchange rate USD/CNY |
| Market Data | GET /api/v5/market/block-tickers | NO | Block trading tickers |
| Market Data | GET /api/v5/market/block-trades | NO | Block trade history |
| Market Data | GET /api/v5/public/funding-rate | YES | `FundingRate` |
| Market Data | GET /api/v5/public/funding-rate-history | YES | `FundingRateHistory` |
| Market Data | GET /api/v5/public/estimated-price | NO | Estimated delivery/settlement price |
| Market Data | GET /api/v5/public/discount-rate-interest-free-quota | NO | Discount rate and interest-free quota |
| Market Data | GET /api/v5/public/tier | NO | Position tier data |
| Market Data | GET /api/v5/public/insurance-fund | NO | Insurance fund balance |
| Market Data | GET /api/v5/public/interest-loan-data | NO | Interest loan data |
| Market Data | GET /api/v5/public/underlying | NO | Underlying assets |
| Market Data | GET /api/v5/public/delivery-exercise-history | NO | Delivery/exercise history |
| Market Data | GET /api/v5/public/settlement-history | NO | Settlement history (new) |
| Market Data | GET /api/v5/public/liquidation-orders | NO | Liquidation order info |
| Market Data | GET /api/v5/public/mark-price-candles | NO | Mark price candles |
| Market Data | GET /api/v5/public/mark-price-candles-history | NO | Mark price candles history |
| Market Data | GET /api/v5/public/index-candles | NO | Index candles (public) |
| Market Data | GET /api/v5/public/index-candles-history | NO | Index candles history |
| Market Data | GET /api/v5/public/open-interest-volume | NO | Open interest volume |
| Market Data | GET /api/v5/public/long-short-account-ratio | NO | Long/short account ratio |
| Market Data | GET /api/v5/public/taker-volume | NO | Taker buy/sell volume |
| Market Data | GET /api/v5/public/price-limit | NO | Price limit |
| Market Data | GET /api/v5/public/opt-summary | NO | Option market data |

## 3.2 Trading (Order Book Trading — Trade)

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Trading | POST /api/v5/trade/order | YES | `PlaceOrder` |
| Trading | POST /api/v5/trade/batch-orders | YES | `PlaceBatchOrders` |
| Trading | POST /api/v5/trade/cancel-order | YES | `CancelOrder` |
| Trading | POST /api/v5/trade/cancel-batch-orders | YES | `CancelBatchOrders` |
| Trading | POST /api/v5/trade/amend-order | YES | `AmendOrder` |
| Trading | POST /api/v5/trade/amend-batch-orders | NO | Batch amend orders |
| Trading | POST /api/v5/trade/cancel-all-after | YES | `CancelAllAfter` |
| Trading | GET /api/v5/trade/order | YES | `GetOrder` |
| Trading | GET /api/v5/trade/orders-pending | YES | `OpenOrders` |
| Trading | GET /api/v5/trade/orders-history | YES | `OrderHistory` |
| Trading | GET /api/v5/trade/orders-history-archive | YES | `OrderHistoryArchive` |
| Trading | GET /api/v5/trade/fills | NO | Recent fills |
| Trading | GET /api/v5/trade/fills-history | NO | Fill history (3 months) |
| Trading | POST /api/v5/trade/easy-convert-currency-list | NO | Easy convert currency list |
| Trading | POST /api/v5/trade/easy-convert | NO | Easy convert (small amounts) |
| Trading | GET /api/v5/trade/easy-convert-history | NO | Easy convert history |
| Trading | POST /api/v5/trade/one-click-repay-currency-list | NO | One-click repay currency list |
| Trading | POST /api/v5/trade/one-click-repay | NO | One-click repay debt |
| Trading | GET /api/v5/trade/one-click-repay-history | NO | One-click repay history |
| Trading | POST /api/v5/trade/mass-cancel | NO | Mass cancel by instrument family |

## 3.3 Algo Orders

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Algo | POST /api/v5/trade/order-algo | YES | `AlgoOrder` (stop, trailing, twap, oco, iceberg) |
| Algo | POST /api/v5/trade/cancel-algos | YES | `AlgoOrderCancel` |
| Algo | POST /api/v5/trade/amend-algos | NO | Amend algo orders |
| Algo | POST /api/v5/trade/cancel-advance-algos | NO | Cancel advanced algo orders |
| Algo | GET /api/v5/trade/orders-algo-pending | YES | `AlgoOpenOrders` |
| Algo | GET /api/v5/trade/orders-algo-history | NO | Algo order history |
| Algo | GET /api/v5/trade/order-algo | NO | Get specific algo order details |

## 3.4 Account

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Account | GET /api/v5/account/instruments | NO | Available instruments for account |
| Account | GET /api/v5/account/balance | YES | `Balance` |
| Account | GET /api/v5/account/positions | YES | `Positions` |
| Account | GET /api/v5/account/positions-history | YES | `PositionHistory` |
| Account | GET /api/v5/account/account-risk | NO | Account and position risk |
| Account | GET /api/v5/account/bills | NO | Bills (last 7 days) |
| Account | GET /api/v5/account/bills-archive | NO | Bills (last 3 months) |
| Account | POST /api/v5/account/bills-archive-apply | NO | Apply for bills archive |
| Account | GET /api/v5/account/bills-archive-query | NO | Query archived bills |
| Account | GET /api/v5/account/config | YES | `AccountConfig` |
| Account | POST /api/v5/account/set-position-mode | YES | `SetPositionMode` |
| Account | POST /api/v5/account/set-leverage | YES | `SetLeverage` |
| Account | GET /api/v5/account/leverage-info | YES | `GetLeverage` |
| Account | POST /api/v5/account/set-margin | NO | Set margin (add/reduce) |
| Account | GET /api/v5/account/max-size | YES | `MaxOrderSize` |
| Account | GET /api/v5/account/max-avail-size | NO | Max available trade size |
| Account | GET /api/v5/account/interest-accrued | NO | Interest accrued |
| Account | GET /api/v5/account/interest-rate | NO | Interest rate |
| Account | POST /api/v5/account/set-greeks | NO | Set greeks display unit |
| Account | POST /api/v5/account/set-isolated-mode | NO | Set isolated margin mode |
| Account | GET /api/v5/account/max-loan | NO | Max loan amount |
| Account | GET /api/v5/account/fee-rates | NO | Fee rates |
| Account | GET /api/v5/account/max-withdrawal | NO | Max withdrawable amount |
| Account | GET /api/v5/account/risk-state | NO | Portfolio margin risk state |
| Account | POST /api/v5/account/borrow-repay | NO | Spot/futures cross borrow/repay |
| Account | GET /api/v5/account/borrow-repay-history | NO | Borrow/repay history |
| Account | GET /api/v5/account/vip-interest-accrued | NO | VIP interest accrued |
| Account | GET /api/v5/account/vip-interest-deducted | NO | VIP interest deducted |
| Account | GET /api/v5/account/vip-loan-order-list | NO | VIP loan orders |
| Account | GET /api/v5/account/vip-loan-order-detail | NO | VIP loan order detail |
| Account | POST /api/v5/account/set-auto-loan | NO | Enable auto-borrow |
| Account | GET /api/v5/account/quick-margin-borrow-repay | NO | Quick margin borrow/repay |
| Account | GET /api/v5/account/quick-margin-borrow-repay-history | NO | Quick margin history |

## 3.5 Asset / Funding

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Asset | GET /api/v5/asset/balances | YES | `AssetBalances` |
| Asset | GET /api/v5/asset/non-tradable-assets | NO | Non-tradable assets |
| Asset | GET /api/v5/asset/asset-valuation | NO | Asset valuation |
| Asset | POST /api/v5/asset/transfer | YES | `AssetTransfer` |
| Asset | GET /api/v5/asset/transfer-state | YES | `TransferState` |
| Asset | GET /api/v5/asset/bills | YES | `AssetBills` |
| Asset | GET /api/v5/asset/deposit-lightning | NO | Lightning deposit (BTC) |
| Asset | GET /api/v5/asset/deposit-address | YES | `DepositAddress` |
| Asset | GET /api/v5/asset/deposit-history | YES | `DepositHistory` |
| Asset | POST /api/v5/asset/withdrawal | YES | `Withdrawal` |
| Asset | POST /api/v5/asset/withdrawal-lightning | NO | Lightning withdrawal (BTC) |
| Asset | POST /api/v5/asset/cancel-withdrawal | NO | Cancel withdrawal |
| Asset | GET /api/v5/asset/withdrawal-history | YES | `WithdrawalHistory` |
| Asset | GET /api/v5/asset/deposit-withdraw-status | NO | Deposit/withdraw status |
| Asset | POST /api/v5/asset/convert-dust-assets | NO | Convert dust to OKB |
| Asset | GET /api/v5/asset/exchange-list | NO | Small asset exchange list |
| Asset | GET /api/v5/asset/monthly-statement | NO | Monthly statement download |

## 3.6 Sub-Accounts

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Sub-Acct | POST /api/v5/users/subaccount/create | YES | `SubAccountCreate` |
| Sub-Acct | GET /api/v5/users/subaccount/list | YES | `SubAccountList` |
| Sub-Acct | POST /api/v5/asset/subaccount/transfer | YES | `SubAccountTransfer` |
| Sub-Acct | GET /api/v5/account/subaccount/balances | YES | `SubAccountBalances` |
| Sub-Acct | GET /api/v5/users/subaccount/modify-apikey | NO | Modify sub-account API key |
| Sub-Acct | GET /api/v5/account/subaccount/interest-limits | NO | Sub-account interest limits |
| Sub-Acct | GET /api/v5/asset/subaccount/bills | NO | Sub-account transfer history |
| Sub-Acct | GET /api/v5/asset/subaccount/managed-subaccount-bills | NO | Managed sub-account bills |
| Sub-Acct | POST /api/v5/users/subaccount/set-transfer-out | NO | Enable/disable sub-acct transfers |
| Sub-Acct | GET /api/v5/users/entrust-subaccount-list | NO | Entrusted sub-accounts |

## 3.7 Earn / Financial Products

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Earn (On-chain) | GET /api/v5/finance/staking-defi/offers | NO | On-chain earn offers |
| Earn (On-chain) | POST /api/v5/finance/staking-defi/purchase | NO | Purchase on-chain earn |
| Earn (On-chain) | POST /api/v5/finance/staking-defi/redeem | NO | Redeem on-chain earn |
| Earn (On-chain) | POST /api/v5/finance/staking-defi/cancel | NO | Cancel purchase/redemption |
| Earn (On-chain) | GET /api/v5/finance/staking-defi/orders-active | NO | Active earn orders |
| Earn (On-chain) | GET /api/v5/finance/staking-defi/orders-history | NO | Earn order history |
| ETH Staking | GET /api/v5/finance/staking-defi/eth/purchase | NO | ETH staking product info |
| ETH Staking | POST /api/v5/finance/staking-defi/eth/purchase | NO | Stake ETH |
| ETH Staking | POST /api/v5/finance/staking-defi/eth/redeem | NO | Redeem staked ETH |
| ETH Staking | GET /api/v5/finance/staking-defi/eth/balance | NO | ETH staking balance |
| SOL Staking | GET /api/v5/finance/staking-defi/sol/purchase | NO | SOL staking product info |
| SOL Staking | POST /api/v5/finance/staking-defi/sol/purchase | NO | Stake SOL |
| SOL Staking | POST /api/v5/finance/staking-defi/sol/redeem | NO | Redeem staked SOL |
| SOL Staking | GET /api/v5/finance/staking-defi/sol/balance | NO | SOL staking balance |
| Simple Earn | GET /api/v5/finance/savings/balance | NO | Savings account balance |
| Simple Earn | POST /api/v5/finance/savings/purchase-redemption | NO | Purchase/redeem savings |
| Simple Earn | GET /api/v5/finance/savings/lending-history | NO | Lending history |
| Simple Earn | GET /api/v5/finance/savings/lending-rate-summary | NO | Lending rate summary |
| Flexible Loan | GET /api/v5/finance/flexible-loan/loan-info | NO | Loan info |
| Flexible Loan | POST /api/v5/finance/flexible-loan/borrow | NO | Borrow |
| Flexible Loan | POST /api/v5/finance/flexible-loan/repay | NO | Repay |

## 3.8 Copy Trading

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Copy Trade | GET /api/v5/copytrading/instruments | NO | Copy trading instruments |
| Copy Trade | GET /api/v5/copytrading/public-lead-traders | NO | Public lead traders |
| Copy Trade | GET /api/v5/copytrading/lead-portfolios | NO | Lead trader portfolios |
| Copy Trade | POST /api/v5/copytrading/batch-lead-trades | NO | Batch lead orders |
| Copy Trade | GET /api/v5/copytrading/copy-settings | NO | My copy settings |
| Copy Trade | GET /api/v5/copytrading/current-subpositions | NO | Current copy subpositions |

## 3.9 Convert

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Convert | GET /api/v5/asset/convert/currencies | NO | Convertible currencies |
| Convert | GET /api/v5/asset/convert/currency-pair | NO | Currency pair rate |
| Convert | POST /api/v5/asset/convert/estimate-quote | NO | Get quote |
| Convert | POST /api/v5/asset/convert/trade | NO | Execute conversion |
| Convert | GET /api/v5/asset/convert/history | NO | Conversion history |

## 3.10 WebSocket Streams

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| WS Public | wss://ws.okx.com:8443/ws/v5/public | PARTIAL | Configured in `OkxUrls` but no subscription helper |
| WS Private | wss://ws.okx.com:8443/ws/v5/private | PARTIAL | Configured in `OkxUrls` |
| WS Business | wss://ws.okx.com:8443/ws/v5/business | PARTIAL | Configured in `OkxUrls` but not modeled |

---

---

# 4. KRAKEN

**Current enum variants (from `endpoints.rs`):**

ServerTime,
SpotTicker, SpotOrderbook, SpotOHLC, SpotAssetPairs,
SpotAddOrder, SpotCancelOrder, SpotCancelAll, SpotEditOrder, SpotGetOrder,
SpotOpenOrders, SpotClosedOrders,
SpotBalance, SpotTradeBalance,
SpotWebSocketToken,
FuturesTickers, FuturesOrderbook, FuturesInstruments, FuturesHistory,
FuturesSendOrder, FuturesCancelOrder, FuturesBatchOrder, FuturesEditOrder,
FuturesAccounts, FuturesOpenPositions, FuturesHistoricalFunding,
FuturesSetLeverage,
SpotDepositAddresses, SpotWithdraw, SpotDepositStatus, SpotWithdrawStatus,
SpotListSubaccounts, SpotTransferToSubaccount, SpotTransferFromSubaccount

---

## 4.1 Spot — Market Data (Public)

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Market Data | GET /0/public/Time | YES | `ServerTime` |
| Market Data | GET /0/public/SystemStatus | NO | Exchange system status |
| Market Data | GET /0/public/Assets | NO | Asset info |
| Market Data | GET /0/public/AssetPairs | YES | `SpotAssetPairs` |
| Market Data | GET /0/public/Ticker | YES | `SpotTicker` |
| Market Data | GET /0/public/OHLC | YES | `SpotOHLC` |
| Market Data | GET /0/public/Depth | YES | `SpotOrderbook` |
| Market Data | GET /0/public/Depth (L3) | NO | L3 order book (per-order) |
| Market Data | GET /0/public/Depth (grouped) | NO | Grouped/aggregated order book |
| Market Data | GET /0/public/Trades | NO | Recent public trades |
| Market Data | GET /0/public/Spread | NO | Recent spread data |

## 4.2 Spot — Account Data (Private)

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Account | POST /0/private/Balance | YES | `SpotBalance` |
| Account | POST /0/private/BalanceEx | NO | Extended balance (includes holds) |
| Account | POST /0/private/GetCreditLines | NO | Credit lines |
| Account | POST /0/private/TradeBalance | YES | `SpotTradeBalance` |
| Account | POST /0/private/OpenOrders | YES | `SpotOpenOrders` |
| Account | POST /0/private/ClosedOrders | YES | `SpotClosedOrders` |
| Account | POST /0/private/QueryOrders | YES | `SpotGetOrder` |
| Account | POST /0/private/GetOrderAmends | NO | Order amendment history |
| Account | POST /0/private/TradesHistory | NO | Trade history |
| Account | POST /0/private/QueryTrades | NO | Query specific trades |
| Account | POST /0/private/OpenPositions | NO | Open margin positions |
| Account | POST /0/private/Ledgers | NO | Ledger entries |
| Account | POST /0/private/QueryLedgers | NO | Query specific ledger entries |
| Account | POST /0/private/TradeVolume | NO | Trade volume and fee info |
| Account | POST /0/private/RetrieveExport | NO | Export trade history |
| Account | POST /0/private/AddExport | NO | Request export |
| Account | POST /0/private/RemoveExport | NO | Remove export |
| Account | POST /0/private/ExportStatus | NO | Export status |
| Account | POST /0/private/GetAPIKeyInfo | NO | API key info |

## 4.3 Spot — Trading (Private)

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Trading | POST /0/private/AddOrder | YES | `SpotAddOrder` |
| Trading | POST /0/private/AddOrderBatch | NO | Batch add orders (up to 15) |
| Trading | POST /0/private/AmendOrder | NO | Amend order (atomic, keeps priority) |
| Trading | POST /0/private/EditOrder | YES | `SpotEditOrder` |
| Trading | POST /0/private/CancelOrder | YES | `SpotCancelOrder` |
| Trading | POST /0/private/CancelOrderBatch | NO | Batch cancel orders |
| Trading | POST /0/private/CancelAll | YES | `SpotCancelAll` |
| Trading | POST /0/private/CancelAllOrdersAfter | NO | Dead man's switch (cancel after X) |
| Trading | POST /0/private/GetWebSocketsToken | YES | `SpotWebSocketToken` |

## 4.4 Spot — Funding (Private)

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Funding | POST /0/private/DepositMethods | NO | Deposit methods |
| Funding | POST /0/private/DepositAddresses | YES | `SpotDepositAddresses` |
| Funding | POST /0/private/DepositStatus | YES | `SpotDepositStatus` |
| Funding | POST /0/private/WithdrawalMethods | NO | Withdrawal methods |
| Funding | POST /0/private/WithdrawalAddresses | NO | Withdrawal address book |
| Funding | POST /0/private/WithdrawFunds | NO | Note: separate from SpotWithdraw? |
| Funding | POST /0/private/Withdraw | YES | `SpotWithdraw` |
| Funding | POST /0/private/WithdrawStatus | YES | `SpotWithdrawStatus` |
| Funding | POST /0/private/CancelWithdrawal | NO | Cancel a pending withdrawal |
| Funding | POST /0/private/WithdrawInfo | NO | Withdrawal fee/limit info |
| Funding | POST /0/private/WalletTransfer | NO | Transfer between Kraken accounts (spot↔futures) |

## 4.5 Spot — Sub-Accounts

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Sub-Acct | POST /0/private/ListSubaccounts | YES | `SpotListSubaccounts` |
| Sub-Acct | POST /0/private/TransferToSubaccount | YES | `SpotTransferToSubaccount` |
| Sub-Acct | POST /0/private/TransferFromSubaccount | YES | `SpotTransferFromSubaccount` |
| Sub-Acct | POST /0/private/CreateSubaccount | NO | Create sub-account |
| Sub-Acct | POST /0/private/SetSubaccountOptions | NO | Configure sub-account |
| Sub-Acct | POST /0/private/GetSubaccounts | NO | Get sub-account list (alt?) |

## 4.6 Spot — Earn

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Earn | POST /0/private/Earn/Allocate | NO | Allocate funds to earn strategy |
| Earn | POST /0/private/Earn/Deallocate | NO | Deallocate from earn strategy |
| Earn | GET /0/private/Earn/AllocateStatus | NO | Allocation status |
| Earn | GET /0/private/Earn/DeallocateStatus | NO | Deallocation status |
| Earn | GET /0/private/Earn/Strategies | NO | List available earn strategies |
| Earn | GET /0/private/Earn/Allocations | NO | List current allocations |

## 4.7 Spot — Transparency (Private)

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Transparency | GET /0/private/PreTrade | NO | Pre-trade transparency data |
| Transparency | GET /0/private/PostTrade | NO | Post-trade transparency data |

## 4.8 Futures

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Fut MD | GET /derivatives/api/v3/tickers | YES | `FuturesTickers` |
| Fut MD | GET /derivatives/api/v3/orderbook | YES | `FuturesOrderbook` |
| Fut MD | GET /derivatives/api/v3/instruments | YES | `FuturesInstruments` |
| Fut MD | GET /derivatives/api/v3/history | YES | `FuturesHistory` |
| Fut MD | GET /derivatives/api/v3/candles | NO | OHLCV data |
| Fut MD | GET /derivatives/api/v3/tradeHistory | NO | Trade history |
| Fut MD | GET /derivatives/api/v3/analytics/volatility | NO | Volatility analytics |
| Fut MD | GET /derivatives/api/v3/initialMargin | NO | Initial margin requirements |
| Fut Trade | POST /derivatives/api/v3/sendorder | YES | `FuturesSendOrder` |
| Fut Trade | POST /derivatives/api/v3/cancelorder | YES | `FuturesCancelOrder` |
| Fut Trade | POST /derivatives/api/v3/batchorder | YES | `FuturesBatchOrder` |
| Fut Trade | POST /derivatives/api/v3/editorder | YES | `FuturesEditOrder` |
| Fut Trade | POST /derivatives/api/v3/cancelallorders | NO | Cancel all orders |
| Fut Trade | GET /derivatives/api/v3/openorders | NO | Open orders |
| Fut Trade | GET /derivatives/api/v3/fills | NO | Trade fills |
| Fut Trade | GET /derivatives/api/v3/orders | NO | Order history |
| Fut Acct | GET /derivatives/api/v3/accounts | YES | `FuturesAccounts` |
| Fut Acct | GET /derivatives/api/v3/openpositions | YES | `FuturesOpenPositions` |
| Fut Acct | GET /derivatives/api/v4/historicalfundingrates | YES | `FuturesHistoricalFunding` |
| Fut Acct | POST /derivatives/api/v3/leveragepreferences | YES | `FuturesSetLeverage` |
| Fut Acct | GET /derivatives/api/v3/leveragepreferences | NO | Get leverage preferences |
| Fut Acct | POST /derivatives/api/v3/initialmargin | NO | Max order size |
| Fut Acct | GET /derivatives/api/v3/portfolio | NO | Portfolio summary |
| Fut Acct | GET /derivatives/api/v3/pnlpreferences | NO | PnL currency preference |
| Fut Acct | POST /derivatives/api/v3/pnlpreferences | NO | Set PnL currency |
| Fut Acct | POST /derivatives/api/v3/transfer | NO | Transfer between spot and futures |
| Fut Acct | GET /derivatives/api/v3/transfer | NO | Transfer history |

---

---

# 5. COINBASE (Advanced Trade)

**Current enum variants (from `endpoints.rs`):**

ServerTime, Products, ProductDetails, BestBidAsk, ProductBook, Candles, MarketTrades,
Accounts, AccountDetails, TransactionSummary,
CreateOrder, CancelOrders, EditOrder, OrderDetails, ListOrders, ListFills, PreviewOrder,
V2AccountDeposits, V2AccountTransactions, V2CreateAddress, V2SendTransaction

---

## 5.1 Market Data

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Market Data | GET /time | YES | `ServerTime` |
| Market Data | GET /market/products | YES | `Products` (public alt) |
| Market Data | GET /market/products/{id} | YES | `ProductDetails` (public alt) |
| Market Data | GET /market/product_book | YES | `ProductBook` (public alt) |
| Market Data | GET /market/products/{id}/candles | YES | `Candles` (public alt) |
| Market Data | GET /market/products/{id}/ticker | YES | `MarketTrades` (public alt) |
| Market Data | GET /best_bid_ask | YES | `BestBidAsk` |

## 5.2 Orders / Trading

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Trading | POST /orders | YES | `CreateOrder` |
| Trading | POST /orders/batch_cancel | YES | `CancelOrders` |
| Trading | GET /orders/historical/batch | YES | `ListOrders` |
| Trading | GET /orders/historical/fills | YES | `ListFills` |
| Trading | GET /orders/historical/{order_id} | YES | `OrderDetails` |
| Trading | POST /orders/preview | YES | `PreviewOrder` |
| Trading | POST /orders/edit | YES | `EditOrder` |
| Trading | POST /orders/edit_preview | NO | Preview order edit |

## 5.3 Accounts

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Accounts | GET /accounts | YES | `Accounts` |
| Accounts | GET /accounts/{account_uuid} | YES | `AccountDetails` |

## 5.4 Portfolios

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Portfolios | GET /portfolios | NO | List portfolios |
| Portfolios | POST /portfolios | NO | Create portfolio |
| Portfolios | PUT /portfolios/{portfolio_uuid} | NO | Update portfolio |
| Portfolios | DELETE /portfolios/{portfolio_uuid} | NO | Delete portfolio |
| Portfolios | GET /portfolios/{portfolio_uuid} | NO | Portfolio breakdown |
| Portfolios | POST /portfolios/move_to_portfolio | NO | Move funds between portfolios |

## 5.5 Futures (CFM — Coinbase Financial Markets)

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Futures | GET /cfm/balance_summary | NO | Futures balance summary |
| Futures | GET /cfm/positions | NO | All futures positions |
| Futures | GET /cfm/positions/{product_id} | NO | Single futures position |
| Futures | POST /cfm/sweeps/schedule | NO | Schedule futures sweep |
| Futures | GET /cfm/sweeps | NO | List pending sweeps |
| Futures | DELETE /cfm/sweeps | NO | Cancel pending sweep |
| Futures | GET /cfm/intraday/margin_setting | NO | Intraday margin setting |
| Futures | POST /cfm/intraday/margin_setting | NO | Set intraday margin setting |
| Futures | GET /cfm/intraday/current_margin_window | NO | Current margin window |

## 5.6 Perpetuals (INTX)

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Perpetuals | GET /intx/portfolio | NO | INTX portfolio details |
| Perpetuals | GET /intx/positions | NO | All INTX positions |
| Perpetuals | GET /intx/positions/{symbol} | NO | Single INTX position |
| Perpetuals | GET /intx/balances | NO | INTX balances |
| Perpetuals | POST /intx/multi_asset_collateral | NO | Enable multi-asset collateral |
| Perpetuals | POST /intx/allocate | NO | Allocate portfolio |

## 5.7 Convert

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Convert | POST /convert/quote | NO | Get convert quote |
| Convert | POST /convert/{trade_id} | NO | Commit convert trade |
| Convert | GET /convert/{trade_id} | NO | Get convert trade |

## 5.8 Fees / Payments / Utility

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Fees | GET /transaction_summary | YES | `TransactionSummary` |
| Payments | GET /payment_methods | NO | List payment methods |
| Payments | GET /payment_methods/{id} | NO | Get payment method |
| Utility | GET /key_permissions | NO | API key permissions |

## 5.9 v2 API (Deposits / Withdrawals)

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| v2 | GET /v2/accounts/{id}/deposits | YES | `V2AccountDeposits` |
| v2 | GET /v2/accounts/{id}/transactions | YES | `V2AccountTransactions` |
| v2 | POST /v2/accounts/{id}/addresses | YES | `V2CreateAddress` |
| v2 | POST /v2/accounts/{id}/transactions | YES | `V2SendTransaction` |
| v2 | GET /v2/accounts/{id}/withdrawals | NO | Withdrawal history (v2 alt) |
| v2 | POST /v2/accounts/{id}/withdrawals | NO | Initiate withdrawal via v2 |

## 5.10 WebSocket Streams

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| WS | wss://advanced-trade-ws.coinbase.com | PARTIAL | Public URL in `CoinbaseUrls` |
| WS | wss://advanced-trade-ws-user.coinbase.com | PARTIAL | Private URL in `CoinbaseUrls` |
| WS Channels | subscriptions, heartbeats, candles, market_trades, level2, ticker, ticker_batch, status | NO | Channel subscription helpers not modeled |
| WS Channels | user (private) | NO | Order/fill updates |

---

---

# SUMMARY: Priority Missing Endpoints

## Tier 1 — High Business Value (add soon)

| Exchange | Missing Endpoints | Reason |
|----------|------------------|--------|
| Binance Spot | GET /api/v3/trades, GET /api/v3/aggTrades, GET /api/v3/myTrades | Trade data for fills and market history |
| Binance Spot | GET /api/v3/ticker/bookTicker | Fast best-bid-ask polling |
| Binance Spot | POST /api/v3/order/cancelReplace | Common pattern in algos |
| Binance Spot | GET /api/v3/orderList, GET /api/v3/openOrderList | OCO order management |
| Binance Futures | GET /fapi/v1/trades, GET /fapi/v1/aggTrades | Trade/fill history |
| Binance Futures | GET /fapi/v3/account, GET /fapi/v3/balance | Newer account endpoints |
| Binance Futures | GET /fapi/v1/userTrades | Fill history |
| Binance Futures | GET /fapi/v1/income | Income history |
| Binance Futures | GET /fapi/v1/openInterest | Market data |
| Binance Futures | PATCH /fapi/v1/batchOrders | Batch amend |
| Bybit | GET /v5/order/execution | Fill history |
| Bybit | GET /v5/position/closed-pnl | PnL history |
| Bybit | POST /v5/order/amend-batch | Batch amend |
| Bybit | GET /v5/account/transaction-log | Transaction history |
| Bybit | GET /v5/market/open-interest | Open interest |
| Bybit | POST /v5/asset/withdraw/cancel | Cancel withdrawal |
| OKX | GET /api/v5/trade/fills | Recent fills |
| OKX | GET /api/v5/trade/fills-history | Fill history |
| OKX | POST /api/v5/trade/amend-batch-orders | Batch amend |
| OKX | GET /api/v5/account/bills | Account bills |
| OKX | GET /api/v5/account/fee-rates | Fee rates |
| OKX | GET /api/v5/trade/orders-algo-history | Algo order history |
| OKX | POST /api/v5/asset/convert/* (5 endpoints) | Convert/swap |
| Kraken Spot | GET /0/public/Trades | Public trade stream |
| Kraken Spot | POST /0/private/TradesHistory | Fill history |
| Kraken Spot | POST /0/private/AddOrderBatch | Batch orders |
| Kraken Spot | POST /0/private/AmendOrder | Amend (keeps priority) |
| Kraken Spot | POST /0/private/CancelOrderBatch | Batch cancel |
| Kraken Spot | POST /0/private/WalletTransfer | Cross-product transfer |
| Kraken Futures | GET /derivatives/api/v3/openorders | Open orders |
| Kraken Futures | GET /derivatives/api/v3/fills | Fills |
| Coinbase | GET /portfolios (6 endpoints) | Portfolio management |
| Coinbase | GET /cfm/* (9 endpoints) | Futures management |
| Coinbase | GET /intx/* (6 endpoints) | Perpetuals |
| Coinbase | POST /convert/* (3 endpoints) | Convert/swap |
| Coinbase | POST /orders/edit_preview | Edit preview |

## Tier 2 — Medium Value (add when implementing full features)

| Exchange | Category | Count | Notes |
|----------|----------|-------|-------|
| Binance | Margin (cross + isolated) | ~25 | Full margin trading |
| Binance | Sub-account extensions | ~8 | Futures/margin sub-account ops |
| Binance | Wallet/dust/cloud mining | ~8 | Asset management |
| Binance | Listen key keep-alive/close | 4 | Required for proper WS lifecycle |
| Bybit | Spot margin (UTA) | ~9 | Margin borrowing |
| Bybit | Position mode/set-tpsl-mode | 2 | Futures position control |
| Bybit | Asset coin-info/exchange-record | ~6 | Asset management |
| Bybit | User management | ~5 | Sub-member control |
| OKX | Account bills/risk/limits | ~15 | Full account management |
| OKX | Sub-account extensions | ~5 | Sub-acct transfer history, API keys |
| OKX | Market data extended | ~15 | Mark price, index, liquidations, OI |
| Kraken Spot | Earn (6 endpoints) | 6 | Yield products |
| Kraken Spot | Full funding lifecycle | ~4 | Withdrawal methods, addresses, cancel |
| Kraken Futures | Analytics/candles | ~3 | OHLCV, trade history |
| Coinbase | Payment methods | 2 | Payment method listing |

## Tier 3 — Lower Priority (niche / institutional)

| Exchange | Category | Notes |
|----------|----------|-------|
| Binance | Simple Earn / Staking / ETH/SOL Staking | 20+ endpoints for yield products |
| Binance | Convert (8 endpoints) | Coin swap/conversion |
| Binance | Futures advanced market data | Open interest, long/short ratios, mark/index klines |
| Bybit | Earn | 6 endpoints for staking/yield |
| Bybit | Crypto Loan | 5 endpoints |
| Bybit | Broker | 4 endpoints |
| Bybit | Spread/RFQ Trading | Block trading |
| OKX | Earn (ETH/SOL staking, Simple Earn) | 15+ endpoints |
| OKX | Copy Trading | 6 endpoints |
| OKX | Broker | Institutional |
| Kraken | Spot Transparency | Pre/post-trade reports |
| Coinbase | Key permissions | Utility |

---

## Sources

- [Binance Spot REST API — Market Data](https://developers.binance.com/docs/binance-spot-api-docs/rest-api/market-data-endpoints)
- [Binance Spot REST API — Trading Endpoints](https://developers.binance.com/docs/binance-spot-api-docs/rest-api/trading-endpoints)
- [Binance Spot REST API — Account Endpoints](https://developers.binance.com/docs/binance-spot-api-docs/rest-api/account-endpoints)
- [Binance USDT-M Futures — Trade REST API](https://developers.binance.com/docs/derivatives/usds-margined-futures/trade/rest-api)
- [Binance USDT-M Futures — Account REST API](https://developers.binance.com/docs/derivatives/usds-margined-futures/account/rest-api)
- [Binance Margin SAPI Updates 2024](https://www.binance.com/en/support/announcement/updates-on-binance-margin-sapi-endpoints-2024-03-31-a1868c686ce7448da8c3061a82a87b0c)
- [Binance Simple Earn + ETH/SOL Staking API](https://www.binance.com/en/support/announcement/binance-earn-enables-api-functionality-for-simple-earn-eth-staking-c0250022fed440e0be7c3a388b08d9be)
- [Bybit V5 API Introduction](https://bybit-exchange.github.io/docs/v5/intro)
- [Bybit V5 Order — Place Order](https://bybit-exchange.github.io/docs/v5/order/create-order)
- [Bybit V5 Position](https://bybit-exchange.github.io/docs/v5/position)
- [Bybit V5 Earn](https://bybit-exchange.github.io/docs/v5/earn/product-info)
- [OKX API v5 Documentation](https://www.okx.com/docs-v5/en/)
- [OKX Trading Account REST API](https://www.okx.com/docs-v5/en/#trading-account-rest-api)
- [OKX Financial Product API](https://www.okx.com/docs-v5/en/#financial-product)
- [Kraken API Center](https://docs.kraken.com/api/)
- [Kraken Spot REST Introduction](https://docs.kraken.com/api/docs/guides/spot-rest-intro/)
- [Kraken Earn Strategies](https://docs.kraken.com/api/docs/rest-api/list-strategies/)
- [Kraken Add Order Batch](https://docs.kraken.com/api/docs/rest-api/add-order-batch/)
- [Coinbase Advanced Trade API Endpoints](https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/rest-api)
- [Binance Futures Market Data — Open Interest](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Open-Interest)
- [Binance Futures Market Data — Mark Price Kline](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Mark-Price-Kline-Candlestick-Data)
