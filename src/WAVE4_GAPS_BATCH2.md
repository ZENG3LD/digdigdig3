# Wave 4 Endpoint Gap Analysis — Batch 2

> Generated: 2026-03-13
> Connectors analyzed: KuCoin, Bitfinex, HTX (Huobi), Gate.io, MEXC
> Source path: `digdigdig3/src/crypto/cex/{name}/endpoints.rs`

---

## Summary

| Exchange | Endpoints We Have | Estimated API Total | Coverage |
|----------|------------------|---------------------|----------|
| KuCoin | 44 | ~90+ | ~49% |
| Bitfinex | 34 | ~70+ | ~49% |
| HTX | 35 | ~80+ | ~44% |
| Gate.io | 44 | ~85+ | ~52% |
| MEXC | 38 | ~75+ | ~51% |

---

## 1. KuCoin

**Current implementation**: `crypto/cex/kucoin/endpoints.rs`
**Docs**: https://www.kucoin.com/docs/rest/spot-trading/market-data/introduction

### 1.1 Endpoint Gap Table

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| **Market Data** | Server Timestamp | `GET /api/v1/timestamp` | YES | `Timestamp` |
| **Market Data** | Spot Symbols | `GET /api/v2/symbols` | YES | `SpotSymbols` |
| **Market Data** | Spot Ticker (single) | `GET /api/v1/market/stats` | YES | `SpotTicker` |
| **Market Data** | Spot All Tickers | `GET /api/v1/market/allTickers` | YES | `SpotAllTickers` |
| **Market Data** | Spot Orderbook | `GET /api/v1/market/orderbook/level2_100` | YES | `SpotOrderbook` |
| **Market Data** | Spot Klines | `GET /api/v1/market/candles` | YES | `SpotKlines` |
| **Market Data** | Spot Best Bid/Ask (Level 1) | `GET /api/v1/market/orderbook/level1` | YES | `SpotPrice` |
| **Market Data** | Spot Full Orderbook (Level 2) | `GET /api/v1/market/orderbook/level2` | NO | Authenticated, full 200-level book |
| **Market Data** | Spot Recent Trades | `GET /api/v1/market/histories` | NO | Recent public trades list |
| **Market Data** | Spot 24hr Stats | `GET /api/v1/market/stats` | YES | Same as `SpotTicker` |
| **Market Data** | Currencies List | `GET /api/v3/currencies` | NO | All currencies metadata |
| **Market Data** | Currency Detail | `GET /api/v3/currencies/{currency}` | NO | Single currency info |
| **Market Data** | Service Status | `GET /api/v1/status` | NO | Platform health check |
| **Market Data** | Market Announcements | `GET /api/v3/announcements` | NO | Exchange announcements |
| **Futures Market** | Futures Contracts (active) | `GET /api/v1/contracts/active` | YES | `FuturesContracts` |
| **Futures Market** | Futures Ticker | `GET /api/v1/ticker` | YES | `FuturesTicker` |
| **Futures Market** | Futures All Tickers | `GET /api/v1/ticker?symbols=` | YES | `FuturesAllTickers` |
| **Futures Market** | Futures Orderbook (depth) | `GET /api/v1/level2/depth100` | YES | `FuturesOrderbook` |
| **Futures Market** | Futures Full Orderbook | `GET /api/v1/level2/snapshot` | NO | Full snapshot, authenticated |
| **Futures Market** | Futures Klines | `GET /api/v1/kline/query` | YES | `FuturesKlines` |
| **Futures Market** | Futures Trade History | `GET /api/v1/trade/history` | NO | Public trade history |
| **Futures Market** | Futures Mark Price | `GET /api/v1/mark-price/{symbol}/current` | NO | Mark price for liquidations |
| **Futures Market** | Futures Index Price | `GET /api/v1/index-price/{symbol}/current` | NO | Spot index reference |
| **Futures Market** | Futures Premium Index | `GET /api/v1/premium-index/{symbol}/current` | NO | Premium index |
| **Futures Market** | Funding Rate (current) | `GET /api/v1/funding-rate/{symbol}/current` | YES | `FundingRate` |
| **Futures Market** | Funding Rate History | `GET /api/v1/contract/funding-rates` | NO | Historical funding rates |
| **Futures Market** | Interest Rate Index | `GET /api/v1/interest-rate/{currency}` | NO | Funding component |
| **Futures Market** | Open Interest | `GET /api/v1/contracts/risk-limit/{symbol}` | NO | Open interest / risk limits |
| **Futures Market** | 24hr Futures Stats | `GET /api/v1/stats` | NO | Futures market stats |
| **Spot Trading** | Place Order | `POST /api/v1/orders` | YES | `SpotCreateOrder` |
| **Spot Trading** | Cancel Order | `DELETE /api/v1/orders/{orderId}` | YES | `SpotCancelOrder` |
| **Spot Trading** | Get Order | `GET /api/v1/orders/{orderId}` | YES | `SpotGetOrder` |
| **Spot Trading** | Open Orders | `GET /api/v1/orders` | YES | `SpotOpenOrders` |
| **Spot Trading** | Order History | `GET /api/v1/orders` | YES | `SpotAllOrders` |
| **Spot Trading** | Cancel All Orders | `DELETE /api/v1/orders` | YES | `SpotCancelAllOrders` |
| **Spot Trading** | Get Order by ClientOid | `GET /api/v1/orders/client-order/{clientOid}` | NO | Client-assigned order ID lookup |
| **Spot Trading** | Cancel by ClientOid | `DELETE /api/v1/orders/client-order/{clientOid}` | NO | Cancel using custom ID |
| **Spot Trading** | Trade History (fills) | `GET /api/v1/fills` | NO | Executed trades / fill history |
| **Spot Trading** | Recent Fill History | `GET /api/v1/limit/fills` | NO | 24hr fills (no pagination) |
| **Spot Trading** | OCO Order | `POST /api/v3/oco/order` | YES | `SpotOcoOrder` |
| **Spot Trading** | HF Batch Orders | `POST /api/v1/hf/orders/multi` | YES | `SpotBatchOrders` |
| **Spot Trading** | HF Cancel Order | `DELETE /api/v1/hf/orders/{orderId}` | NO | HF account cancel |
| **Spot Trading** | HF Get Order | `GET /api/v1/hf/orders/{orderId}` | NO | HF account order detail |
| **Spot Trading** | Stop Orders (place) | `POST /api/v1/stop-order` | NO | TP/SL trigger orders |
| **Spot Trading** | Stop Orders (cancel) | `DELETE /api/v1/stop-order/{orderId}` | NO | Cancel stop order |
| **Spot Trading** | Stop Orders (list) | `GET /api/v1/stop-order` | NO | List stop orders |
| **Futures Trading** | Place Futures Order | `POST /api/v1/orders` | YES | `FuturesCreateOrder` |
| **Futures Trading** | Cancel Futures Order | `DELETE /api/v1/orders/{orderId}` | YES | `FuturesCancelOrder` |
| **Futures Trading** | Get Futures Order | `GET /api/v1/orders/{orderId}` | YES | `FuturesGetOrder` |
| **Futures Trading** | Open Futures Orders | `GET /api/v1/orders` | YES | `FuturesOpenOrders` |
| **Futures Trading** | All Futures Orders | `GET /api/v1/orders` | YES | `FuturesAllOrders` |
| **Futures Trading** | Cancel All Futures Orders | `DELETE /api/v1/orders` | YES | `FuturesCancelAllOrders` |
| **Futures Trading** | Batch Futures Orders | `POST /api/v1/orders/multi` | YES | `FuturesBatchOrders` |
| **Futures Trading** | Amend Futures Order | `PUT /api/v1/orders/{orderId}` | YES | `FuturesAmendOrder` |
| **Futures Trading** | Futures Order by ClientOid | `GET /api/v1/orders/client-order/{clientOid}` | NO | Client ID order lookup |
| **Futures Trading** | Cancel by ClientOid | `DELETE /api/v1/orders/client-order/{clientOid}` | NO | Cancel via custom ID |
| **Futures Trading** | Stop-Loss/TP Order | `POST /api/v1/stop-order` | NO | Futures stop orders |
| **Futures Trading** | Cancel All Stop Orders | `DELETE /api/v1/stop-order/all` | NO | Bulk stop cancel |
| **Futures Trading** | Futures Trade History | `GET /api/v1/fills` | NO | Futures executed fills |
| **Futures Trading** | Test Order | `POST /api/v1/orders/test` | NO | Dry-run order placement |
| **Futures Trading** | Open Order Value | `GET /api/v1/orders/open-value` | NO | Notional value of open orders |
| **Spot Account** | Accounts List | `GET /api/v1/accounts` | YES | `SpotAccounts` |
| **Spot Account** | Account Detail | `GET /api/v1/accounts/{accountId}` | YES | `SpotAccountDetail` |
| **Spot Account** | Account Summary | `GET /api/v2/overview` | NO | Unified asset overview |
| **Spot Account** | Account Ledger | `GET /api/v1/accounts/{accountId}/ledgers` | NO | Account history/ledger |
| **Spot Account** | Trade Fee | `GET /api/v1/trade-fees` | NO | Per-symbol fee rates |
| **Spot Account** | Basic Trade Fee | `GET /api/v1/base-fee` | NO | Account tier fee rate |
| **Futures Account** | Account Overview | `GET /api/v1/account-overview` | YES | `FuturesAccount` |
| **Futures Account** | Positions | `GET /api/v1/positions` | YES | `FuturesPositions` |
| **Futures Account** | Position Detail | `GET /api/v1/position` | YES | `FuturesPosition` |
| **Futures Account** | Set Leverage | `POST /api/v1/position/risk-limit-level/change` | YES | `FuturesSetLeverage` |
| **Futures Account** | Get Margin Mode | `GET /api/v1/position/marginMode` | NO | Cross vs isolated margin |
| **Futures Account** | Switch Margin Mode | `POST /api/v1/position/marginMode` | NO | Toggle margin mode |
| **Futures Account** | Max Withdrawable Margin | `GET /api/v1/margin/maxWithdrawMargin` | NO | Risk calc endpoint |
| **Futures Account** | Risk Limit | `GET /api/v1/position/riskLimit/{symbolId}` | NO | Per-symbol risk limits |
| **Futures Account** | Modify Risk Limit | `POST /api/v1/position/riskLimit` | NO | Change risk level |
| **Futures Account** | Funding Fee History | `GET /api/v1/funding-history` | NO | Historical funding payments |
| **Transfers** | Inner Transfer | `POST /api/v3/accounts/inner-transfer` | YES | `InnerTransfer` |
| **Transfers** | Transfer History | `GET /api/v1/accounts/inner-transfer` | YES | `TransferHistory` |
| **Transfers** | Transfer Quotas | `GET /api/v3/accounts/transferable` | NO | Check transferable amounts |
| **Transfers** | Flex Transfer | `POST /api/v3/accounts/universal-transfer` | NO | Unified account transfer |
| **Custodial** | Deposit Address | `GET /api/v3/deposit-addresses` | YES | `DepositAddress` |
| **Custodial** | Withdraw | `POST /api/v1/withdrawals` | YES | `Withdraw` |
| **Custodial** | Deposit History | `GET /api/v1/deposits` | YES | `DepositHistory` |
| **Custodial** | Withdrawal History | `GET /api/v1/withdrawals` | YES | `WithdrawalHistory` |
| **Custodial** | Cancel Withdrawal | `DELETE /api/v1/withdrawals/{withdrawalId}` | NO | Cancel pending withdrawal |
| **Custodial** | Withdrawal Quotas | `GET /api/v1/withdrawals/quotas` | NO | Per-currency withdrawal limits |
| **Sub-Accounts** | Create Sub | `POST /api/v2/sub/user/created` | YES | `SubAccountCreate` |
| **Sub-Accounts** | List Subs | `GET /api/v2/sub/user` | YES | `SubAccountList` |
| **Sub-Accounts** | Sub Transfer | `POST /api/v2/accounts/sub-transfer` | YES | `SubAccountTransfer` |
| **Sub-Accounts** | Sub Balance | `GET /api/v1/sub-accounts/{subUserId}` | YES | `SubAccountBalance` |
| **Sub-Accounts** | Sub API Key Create | `POST /api/v1/sub/api-key` | NO | Generate sub-account API keys |
| **Sub-Accounts** | Sub API Key List | `GET /api/v1/sub/api-key` | NO | List sub-account API keys |
| **Sub-Accounts** | Sub Account Permissions | `POST /api/v2/sub-user/management` | NO | Set trading permissions |
| **Margin Trading** | Cross Margin Symbols | `GET /v3/margin/symbols` | NO | Cross margin trading pairs |
| **Margin Trading** | Isolated Margin Symbols | `GET /v1/isolated/symbols` | NO | Isolated margin pairs |
| **Margin Trading** | ETF Info | `GET /api/v1/etf/info` | NO | Leveraged ETF details |
| **Margin Trading** | Mark Price | `GET /api/v1/mark-price/{symbol}/current` | NO | Margin liquidation price |
| **Margin Trading** | Margin Config | `GET /api/v3/margin/config` | NO | Margin account configuration |
| **Margin Trading** | Borrow | `POST /api/v3/margin/borrow` | NO | Borrow funds on margin |
| **Margin Trading** | Repay | `POST /api/v3/margin/repay` | NO | Repay borrowed funds |
| **Margin Trading** | Borrow History | `GET /api/v3/margin/borrow` | NO | Margin borrow records |
| **Margin Trading** | Repay History | `GET /api/v3/margin/repay` | NO | Repayment records |
| **Margin Trading** | Interest History | `GET /api/v3/margin/interest` | NO | Margin interest payments |
| **Margin Trading** | Margin Risk Limit | `GET /api/v3/margin/risk-limit` | NO | Margin risk parameters |
| **Margin Trading** | Loan Market | `GET /api/v3/project/list` | NO | Available lending products |
| **Margin Trading** | Purchase (Lend) | `POST /api/v3/earn/fixed-income/redeem` | NO | Place lending offer |
| **Margin Trading** | Redeem | `POST /api/v3/earn/fixed-income/redeem` | NO | Redeem lending position |
| **Earn** | Simple Earn Products | `GET /api/v3/earn/savings/products` | NO | Earn product listings |
| **Earn** | Purchase Earn | `POST /api/v3/earn/savings/subscribe` | NO | Subscribe to earn product |
| **Earn** | Redeem Earn | `POST /api/v3/earn/savings/redeem` | NO | Exit earn position |
| **WebSocket** | WS Public Token | `POST /api/v1/bullet-public` | YES | `WsPublicToken` |
| **WebSocket** | WS Private Token | `POST /api/v1/bullet-private` | YES | `WsPrivateToken` |

### 1.2 Key Missing Groups

- **Spot/Futures by ClientOid**: Cancel/Get by `clientOid` — useful for idempotent order management
- **Stop Orders (Spot + Futures)**: TP/SL trigger orders entirely absent
- **Futures Mark/Index/Premium Prices**: Required for position risk calculation
- **Futures Funding Fee History**: `/api/v1/funding-history`
- **Futures Margin Mode**: Switch between cross and isolated margin
- **Margin/Lending**: The entire margin borrow/repay/interest sub-system is missing
- **Trade Fees**: No fee endpoints for spot or futures
- **Account Ledger**: No transaction history endpoint
- **Withdrawal Cancellation / Quotas**: Operational gaps
- **Sub-Account API Key Management**: Cannot programmatically create sub-keys

---

## 2. Bitfinex

**Current implementation**: `crypto/cex/bitfinex/endpoints.rs`
**Docs**: https://docs.bitfinex.com/reference

### 2.1 Endpoint Gap Table

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| **Public** | Platform Status | `GET /platform/status` | YES | `PlatformStatus` |
| **Market Data** | Ticker | `GET /ticker/{symbol}` | YES | `Ticker` |
| **Market Data** | Tickers | `GET /tickers` | YES | `Tickers` |
| **Market Data** | Tickers History | `GET /tickers/hist` | NO | Historical ticker snapshots |
| **Market Data** | Orderbook | `GET /book/{symbol}/{precision}` | YES | `Orderbook` |
| **Market Data** | Trades | `GET /trades/{symbol}/hist` | YES | `Trades` |
| **Market Data** | Candles | `GET /candles/{candle}/hist` | YES | `Candles` |
| **Market Data** | Symbols List | `GET /conf/pub:list:pair:exchange` | YES | `Symbols` |
| **Market Data** | Statistics | `GET /stats1/{key}:{size}:{sym}/{section}` | NO | Open interest, longs/shorts, etc. |
| **Market Data** | Derivatives Status | `GET /status/deriv` | NO | Perpetual futures status |
| **Market Data** | Derivatives Status Hist | `GET /status/deriv/{key}/hist` | NO | Historical deriv status |
| **Market Data** | Liquidations | `GET /liquidations/hist` | NO | Liquidation events |
| **Market Data** | Leaderboards | `GET /rankings/{key}/{section}` | NO | Volume/PnL leaderboards |
| **Market Data** | Funding Statistics | `GET /funding/stats/{symbol}/hist` | NO | Lending market stats |
| **Market Data** | Config Data | `GET /conf/pub:{Action}:{Object}:{Detail}` | NO | Currency configs, precision info |
| **Market Data** | Market Average Price | `POST /calc/trade/avg` | NO | VWAP calculation |
| **Market Data** | FX Rate | `POST /calc/fx` | NO | Foreign exchange rates |
| **Trading** | Submit Order | `POST /auth/w/order/submit` | YES | `SubmitOrder` |
| **Trading** | Update Order | `POST /auth/w/order/update` | YES | `UpdateOrder` |
| **Trading** | Cancel Order | `POST /auth/w/order/cancel` | YES | `CancelOrder` |
| **Trading** | Cancel Multiple Orders | `POST /auth/w/order/cancel/multi` | YES | `CancelMultipleOrders` |
| **Trading** | Order Multi-OP | `POST /auth/w/order/multi` | YES | `OrderMulti` |
| **Trading** | Active Orders | `POST /auth/r/orders` | YES | `ActiveOrders` |
| **Trading** | Active Orders by Symbol | `POST /auth/r/orders/{symbol}` | YES | `ActiveOrdersBySymbol` |
| **Trading** | Order History | `POST /auth/r/orders/hist` | YES | `OrderHistory` |
| **Trading** | Order Trades | `POST /auth/r/order/{symbol}:{id}/trades` | YES | `OrderTrades` |
| **Trading** | Trade History | `POST /auth/r/trades/hist` | YES | `TradeHistory` |
| **Trading** | Trade History by Symbol | `POST /auth/r/trades/{symbol}/hist` | YES | `TradeHistoryBySymbol` |
| **Trading** | OTC Orders History | `POST /auth/r/orders/otc/{Symbol}/hist` | NO | OTC trades history |
| **Positions** | Positions | `POST /auth/r/positions` | YES | `Positions` |
| **Positions** | Positions History | `POST /auth/r/positions/hist` | YES | `PositionHistory` |
| **Positions** | Positions Snapshot | `POST /auth/r/positions/snap` | YES | `PositionSnapshot` |
| **Positions** | Positions Audit | `POST /auth/r/positions/audit` | NO | Detailed position audit log |
| **Positions** | Claim Position | `POST /auth/w/position/claim` | NO | Claim a margin position |
| **Positions** | Increase Position | `POST /auth/w/position/increase` | NO | Add to existing position |
| **Positions** | Increase Position Info | `POST /auth/r/position/increase/info` | NO | Cost preview for increase |
| **Positions** | Margin Info | `POST /auth/r/info/margin/{key}` | NO | Cross/isolated margin summary |
| **Derivatives** | Set Derivative Collateral | `POST /auth/w/deriv/collateral/set` | NO | Adjust perp collateral |
| **Derivatives** | Collateral Limits | `POST /auth/calc/deriv/collateral/limits` | NO | Max collateral check |
| **Funding/Lending** | Funding Offers | `POST /auth/r/funding/offers/{symbol}` | NO | Active lending offers |
| **Funding/Lending** | Submit Funding Offer | `POST /auth/w/funding/offer/submit` | NO | Place a lending offer |
| **Funding/Lending** | Cancel Funding Offer | `POST /auth/w/funding/offer/cancel` | NO | Cancel single lending offer |
| **Funding/Lending** | Cancel All Funding Offers | `POST /auth/w/funding/offer/cancel/all` | NO | Cancel all lending offers |
| **Funding/Lending** | Funding Offer History | `POST /auth/r/funding/offers/{symbol}/hist` | NO | Historical lending offers |
| **Funding/Lending** | Funding Loans | `POST /auth/r/funding/loans/{symbol}` | NO | Active loans taken |
| **Funding/Lending** | Funding Loans History | `POST /auth/r/funding/loans/{symbol}/hist` | NO | Loan history |
| **Funding/Lending** | Funding Credits | `POST /auth/r/funding/credits/{symbol}` | NO | Active credit positions |
| **Funding/Lending** | Funding Credits History | `POST /auth/r/funding/credits/{symbol}/hist` | NO | Credit history |
| **Funding/Lending** | Funding Trades | `POST /auth/r/funding/trades/{symbol}/hist` | NO | Lending trade history |
| **Funding/Lending** | Funding Close | `POST /auth/w/funding/close` | NO | Close a funding loan |
| **Funding/Lending** | Funding Auto-Renew | `POST /auth/w/funding/auto` | NO | Enable auto-renewal |
| **Funding/Lending** | Funding Info | `POST /auth/r/info/funding/{key}` | NO | Account funding summary |
| **Account** | Wallets | `POST /auth/r/wallets` | YES | `Wallets` |
| **Account** | Transfer | `POST /auth/w/transfer` | YES | `Transfer` |
| **Account** | User Info | `POST /auth/r/info/user` | NO | KYC level, UID, etc. |
| **Account** | Summary | `POST /auth/r/summary` | NO | Account summary/stats |
| **Account** | Ledgers | `POST /auth/r/ledgers/{Currency}/hist` | NO | Detailed ledger entries |
| **Account** | Key Permissions | `POST /auth/r/permissions` | NO | Check API key scopes |
| **Account** | Login History | `POST /auth/r/logins/hist` | NO | Account login audit |
| **Account** | Changelog | `POST /auth/r/audit/hist` | NO | Account change audit |
| **Account** | Generate Token | `POST /auth/w/token` | NO | OAuth-style token generation |
| **Account** | Alerts List | `POST /auth/r/alerts` | NO | Price/margin alert list |
| **Account** | Set Alert | `POST /auth/w/alert/set` | NO | Create price alert |
| **Account** | Delete Alert | `POST /auth/w/alert/del` | NO | Remove price alert |
| **Account** | User Settings Read | `POST /auth/r/settings` | NO | Read user settings |
| **Account** | User Settings Write | `POST /auth/w/settings/set` | NO | Update user settings |
| **Account** | Balance Available | `POST /auth/calc/order/avail` | NO | Available balance for orders |
| **Custodial** | Deposit Address | `POST /auth/w/deposit/address` | YES | `DepositAddress` |
| **Custodial** | Deposit Address List | `POST /auth/r/deposit/address/all` | NO | All deposit addresses |
| **Custodial** | Generate Invoice (LN) | `POST /auth/w/deposit/invoice` | NO | Lightning Network invoice |
| **Custodial** | LN Invoice Payments | `POST /auth/r/ext/invoice/payments` | NO | Lightning payment history |
| **Custodial** | Withdraw | `POST /auth/w/withdraw` | YES | `Withdraw` |
| **Custodial** | Movements (history) | `POST /auth/r/movements/{Currency}/hist` | YES | `Movements` |
| **Custodial** | Movement Info | `POST /auth/r/movements/info` | NO | Details on specific movement |
| **Custodial** | Movement Fee Calc | `POST /auth/r/movements/fee/calc` | NO | Estimate withdrawal fee |
| **Sub-Accounts** | Sub Account List | `POST /auth/r/sub_accounts/list` | YES | `SubAccountList` |
| **Sub-Accounts** | Sub Account Transfer | `POST /auth/w/sub_account/transfer` | YES | `SubAccountTransfer` |

### 2.2 Key Missing Groups

- **Funding/Lending** (12 endpoints): Bitfinex is a major P2P lending exchange — the entire lending sub-system is missing
- **Derivatives**: No perpetual futures collateral management
- **Market Statistics**: `stats1` endpoint for open interest, long/short ratios
- **Margin Info**: No way to query margin requirements
- **Account Audit/Settings**: Login history, changelog, alerts, user settings
- **Position Management**: Claim, increase, audit endpoints
- **Lightning Network**: Invoice deposit/payment endpoints

---

## 3. HTX (Huobi)

**Current implementation**: `crypto/cex/htx/endpoints.rs`
**Docs**: https://huobiapi.github.io/docs/spot/v1/en/

### 3.1 Endpoint Gap Table

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| **Reference Data** | Server Timestamp | `GET /v1/common/timestamp` | YES | `ServerTime` |
| **Reference Data** | Symbols V2 | `GET /v2/settings/common/symbols` | YES | `Symbols` |
| **Reference Data** | Symbols V1 | `GET /v1/common/symbols` | YES | `SymbolsV1` |
| **Reference Data** | Currencies | `GET /v2/settings/common/currencies` | NO | All supported currencies |
| **Reference Data** | Chains Info | `GET /v1/settings/common/chains` | NO | Blockchain chain metadata |
| **Reference Data** | Market Symbols | `GET /v1/settings/common/market-symbols` | NO | Market-specific symbol info |
| **Reference Data** | Market Status | `GET /v2/market-status` | NO | Exchange operational status |
| **Market Data** | Ticker (merged) | `GET /market/detail/merged` | YES | `Ticker` |
| **Market Data** | All Tickers | `GET /market/tickers` | YES | `Tickers` |
| **Market Data** | Orderbook | `GET /market/depth` | YES | `Orderbook` |
| **Market Data** | Klines | `GET /market/history/kline` | YES | `Klines` |
| **Market Data** | Recent Trades | `GET /market/trade` | YES | `RecentTrades` |
| **Market Data** | Trade History | `GET /market/history/trade` | YES | `HistoryTrades` |
| **Market Data** | Market Detail (24hr) | `GET /market/detail` | NO | 24hr OHLCV summary |
| **Market Data** | Best BBO | `GET /market/etp/detail` | NO | ETP/leveraged token info |
| **Futures Market** | Futures Ticker | `GET /linear-swap-ex/market/detail/merged` | YES | `FuturesTicker` |
| **Futures Market** | Futures Orderbook | `GET /linear-swap-ex/market/depth` | YES | `FuturesOrderbook` |
| **Futures Market** | Futures Klines | `GET /linear-swap-ex/market/history/kline` | YES | `FuturesKlines` |
| **Futures Market** | Futures Trades | `GET /linear-swap-ex/market/trade` | YES | `FuturesTrades` |
| **Futures Market** | Futures All Tickers | `GET /linear-swap-ex/market/tickers` | NO | All USDT-margined tickers |
| **Futures Market** | Futures Contract Info | `GET /linear-swap-api/v3/swap-contract-info` | NO | Contract specifications |
| **Futures Market** | Futures Index Price | `GET /linear-swap-ex/market/index` | NO | Index price data |
| **Futures Market** | Futures Mark Price | `GET /linear-swap-api/v1/swap-mark-price-kline` | NO | Mark price klines |
| **Futures Market** | Funding Rate | `GET /linear-swap-api/v1/swap-funding-rate` | NO | Current funding rate |
| **Futures Market** | Funding Rate History | `GET /linear-swap-api/v3/swap-funding-rate-history` | NO | Historical funding rates |
| **Futures Market** | Open Interest | `GET /linear-swap-api/v1/swap-open-interest` | NO | Open interest data |
| **Futures Market** | Insurance Fund | `GET /linear-swap-api/v1/swap-insurance-fund` | NO | Insurance fund history |
| **Account** | Account List | `GET /v1/account/accounts` | YES | `AccountList` |
| **Account** | Balance | `GET /v1/account/accounts/{account-id}/balance` | YES | `Balance` |
| **Account** | Asset Valuation | `GET /v2/account/asset-valuation` | YES | `AccountInfo` |
| **Account** | Ledger | `GET /v2/account/ledger` | YES | `Ledger` |
| **Account** | Account History | `GET /v1/account/history` | NO | Transaction history |
| **Account** | Account Valuation V2 | `GET /v2/account/valuation` | NO | Platform asset valuation |
| **Account** | UID | `GET /v2/user/uid` | NO | Unique user identifier |
| **Account** | Point Account | `GET /v2/point/account` | NO | Loyalty point balance |
| **Trading** | Place Order | `POST /v1/order/orders/place` | YES | `PlaceOrder` |
| **Trading** | Batch Orders | `POST /v1/order/batch-orders` | NO | Place multiple orders at once |
| **Trading** | Cancel Order | `POST /v1/order/orders/{id}/submitcancel` | YES | `CancelOrder` |
| **Trading** | Cancel by ClientOid | `POST /v1/order/orders/submitCancelClientOrder` | NO | Cancel via client order ID |
| **Trading** | Batch Cancel | `POST /v1/order/orders/batchcancel` | YES | `CancelAllOrders` |
| **Trading** | Cancel Open Orders | `POST /v1/order/orders/batchCancelOpenOrders` | YES | `CancelOpenOrders` |
| **Trading** | Get Order | `GET /v1/order/orders/{order-id}` | YES | `OrderStatus` |
| **Trading** | Get Order by ClientOid | `GET /v1/order/orders/getClientOrder` | NO | Lookup by custom order ID |
| **Trading** | Open Orders | `GET /v1/order/openOrders` | YES | `OpenOrders` |
| **Trading** | Order History | `GET /v1/order/orders` | YES | `OrderHistory` |
| **Trading** | Order History V2 | `GET /v1/order/history` | NO | 48-hour window history |
| **Trading** | Match Results | `GET /v1/order/matchresults` | YES | `MatchResults` |
| **Trading** | Match by Order ID | `GET /v1/order/orders/{id}/matchresults` | NO | Fills for specific order |
| **Trading** | Transaction Fee Rate | `GET /v2/reference/transact-fee-rate` | YES | `TransactFee` |
| **Algo Orders** | Place Algo Order | `POST /v2/algo-orders` | YES | `AlgoOrders` |
| **Algo Orders** | Cancel Algo Order | `POST /v2/algo-orders/cancel` | NO | Cancel conditional order |
| **Algo Orders** | Cancel All After | `POST /v2/algo-orders/cancel-all-after` | NO | Dead man's switch |
| **Algo Orders** | Open Algo Orders | `GET /v2/algo-orders/opening` | NO | Active conditional orders |
| **Algo Orders** | Algo Order History | `GET /v2/algo-orders/history` | NO | Historical conditional orders |
| **Algo Orders** | Specific Algo Order | `GET /v2/algo-orders/specific` | NO | Single conditional order |
| **Wallet** | Deposit Address | `GET /v2/account/deposit/address` | YES | `DepositAddress` |
| **Wallet** | Withdraw Quota | `GET /v2/account/withdraw/quota` | YES | `WithdrawQuota` |
| **Wallet** | Withdraw Address | `GET /v2/account/withdraw/address` | YES | `WithdrawAddress` |
| **Wallet** | Withdraw | `POST /v1/dw/withdraw/api/create` | YES | `Withdraw` |
| **Wallet** | Cancel Withdraw | `POST /v1/dw/withdraw/api/cancel` | YES | `WithdrawCancel` |
| **Wallet** | Deposit History | `GET /v1/query/deposit-withdraw` | YES | `DepositHistory` |
| **Wallet** | Withdraw by ClientOrderId | `GET /v1/query/withdrawal/client-order-id` | NO | Lookup withdrawal by client ID |
| **Transfers** | Transfer (inter-account) | `POST /v1/account/transfer` | NO | Transfer between own accounts |
| **Transfers** | Transfer to Futures | `POST /v1/futures/transfer` | YES | `Transfer` |
| **Transfers** | Transfer History | `GET /v2/account/transfer` | YES | `TransferHistory` |
| **Margin (Isolated)** | Transfer In | `POST /v1/dw/transfer-in/margin` | NO | Fund isolated margin account |
| **Margin (Isolated)** | Transfer Out | `POST /v1/dw/transfer-out/margin` | NO | Withdraw from isolated margin |
| **Margin (Isolated)** | Loan Info | `GET /v1/margin/loan-info` | NO | Borrow rates and limits |
| **Margin (Isolated)** | Borrow | `POST /v1/margin/orders` | NO | Take isolated margin loan |
| **Margin (Isolated)** | Repay | `POST /v1/margin/orders/{id}/repay` | NO | Repay isolated loan |
| **Margin (Isolated)** | Loan History | `GET /v1/margin/loan-orders` | NO | Isolated borrow history |
| **Margin (Isolated)** | Margin Balance | `GET /v1/margin/accounts/balance` | NO | Isolated margin balances |
| **Margin (Cross)** | Transfer In | `POST /v1/cross-margin/transfer-in` | NO | Fund cross margin account |
| **Margin (Cross)** | Transfer Out | `POST /v1/cross-margin/transfer-out` | NO | Withdraw from cross margin |
| **Margin (Cross)** | Loan Info | `GET /v1/cross-margin/loan-info` | NO | Cross margin borrow rates |
| **Margin (Cross)** | Borrow | `POST /v1/cross-margin/orders` | NO | Take cross margin loan |
| **Margin (Cross)** | Repay | `POST /v1/cross-margin/orders/{id}/repay` | NO | Repay cross margin debt |
| **Margin (Cross)** | Loan History | `GET /v1/cross-margin/loan-orders` | NO | Cross margin borrow history |
| **Margin (Cross)** | Cross Margin Balance | `GET /v1/cross-margin/accounts/balance` | NO | Cross margin balances |
| **Margin (Cross)** | General Repayment | `POST /v2/account/repayment` | NO | Unified repay endpoint |
| **Sub-Accounts** | Create Sub | `POST /v2/sub-user/creation` | YES | `SubAccountCreate` |
| **Sub-Accounts** | List Subs | `GET /v2/sub-user/user-list` | YES | `SubAccountList` |
| **Sub-Accounts** | Sub Transfer | `POST /v1/subuser/transfer` | YES | `SubAccountTransfer` |
| **Sub-Accounts** | Sub Balance | `GET /v1/account/accounts/{sub-uid}` | YES | `SubAccountBalance` |
| **Sub-Accounts** | Sub State | `GET /v2/sub-user/user-state` | NO | Active/frozen status |
| **Sub-Accounts** | Manage Sub | `POST /v2/sub-user/management` | NO | Lock/unlock sub-account |
| **Sub-Accounts** | Sub Tradable Market | `POST /v2/sub-user/tradable-market` | NO | Set trading permissions |
| **Sub-Accounts** | Sub Transferability | `POST /v2/sub-user/transferability` | NO | Configure transfer rights |
| **Sub-Accounts** | Sub Account List (v2) | `GET /v2/sub-user/account-list` | NO | Sub-account balance summary |
| **Sub-Accounts** | Sub Deposit Address | `GET /v2/sub-user/deposit-address` | NO | Sub-account deposit addr |
| **Sub-Accounts** | Sub Deposit History | `GET /v2/sub-user/query-deposit` | NO | Sub-account deposit history |
| **Sub-Accounts** | Aggregate Balance | `GET /v1/subuser/aggregate-balance` | NO | Total across all sub-accounts |
| **Sub-Accounts** | Sub API Key Create | `POST /v2/sub-user/api-key-generation` | NO | Generate sub API key |
| **Sub-Accounts** | Sub API Key Modify | `POST /v2/sub-user/api-key-modification` | NO | Update sub API key |
| **Sub-Accounts** | Sub API Key Delete | `POST /v2/sub-user/api-key-deletion` | NO | Delete sub API key |
| **Sub-Accounts** | Query API Key | `GET /v2/user/api-key` | NO | Query API key info |
| **Sub-Accounts** | Sub Deduct Mode | `POST /v2/sub-user/deduct-mode` | NO | Fee deduction settings |
| **WebSocket** | Market WS | `wss://api.huobi.pro/ws` | YES | `ws_market_url()` |
| **WebSocket** | MBP Feed WS | `wss://api.huobi.pro/feed` | YES | `ws_mbp_url()` |
| **WebSocket** | Account WS | `wss://api.huobi.pro/ws/v2` | YES | `ws_account_url()` |
| **WebSocket** | Linear Swap WS | `wss://api.hbdm.com/linear-swap-ws` | YES | `ws_linear_swap_url()` |

### 3.2 Key Missing Groups

- **Margin (Isolated + Cross)**: Entire borrow/repay/balance sub-system for both isolated and cross margin (14 endpoints)
- **Algo Orders Queries**: Place is covered but cancel, list, history all missing (5 endpoints)
- **Batch Orders**: No `POST /v1/order/batch-orders` for spot
- **Cancel by ClientOid**: `submitCancelClientOrder` missing
- **Sub-Account Management**: State, lock/unlock, permissions, API key management, deposit addresses (13 endpoints)
- **Reference Data**: Currencies, chains, market status endpoints
- **Account History / Valuation V2**: `/v1/account/history`, `/v2/account/valuation`
- **Futures Market Data**: Contract info, funding rates, open interest (6 endpoints)
- **Transfer (inter-account)**: Generic `/v1/account/transfer`

---

## 4. Gate.io

**Current implementation**: `crypto/cex/gateio/endpoints.rs`
**Docs**: https://www.gate.com/docs/developers/apiv4/en/

### 4.1 Endpoint Gap Table

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| **General** | Server Time | `GET /spot/time` | YES | `ServerTime` |
| **Spot Market** | All Currency Pairs | `GET /spot/currency_pairs` | YES | `SpotSymbols` |
| **Spot Market** | Single Currency Pair | `GET /spot/currency_pairs/{pair}` | NO | Pair detail/specs |
| **Spot Market** | Tickers | `GET /spot/tickers` | YES | `SpotTickers` |
| **Spot Market** | Orderbook | `GET /spot/order_book` | YES | `SpotOrderbook` |
| **Spot Market** | Klines | `GET /spot/candlesticks` | YES | `SpotKlines` |
| **Spot Market** | Currencies | `GET /spot/currencies` | NO | All currency metadata |
| **Spot Market** | Single Currency | `GET /spot/currencies/{currency}` | NO | Single currency info |
| **Spot Market** | Recent Trades | `GET /spot/trades` | NO | Public trade history |
| **Spot Trading** | Place Order | `POST /spot/orders` | YES | `SpotCreateOrder` |
| **Spot Trading** | Cancel Order | `DELETE /spot/orders/{order_id}` | YES | `SpotCancelOrder` |
| **Spot Trading** | Get Order | `GET /spot/orders/{order_id}` | YES | `SpotGetOrder` |
| **Spot Trading** | Open Orders | `GET /spot/orders` | YES | `SpotOpenOrders` |
| **Spot Trading** | Cancel All Orders | `DELETE /spot/orders` | YES | `SpotCancelAllOrders` |
| **Spot Trading** | Amend Order | `PATCH /spot/orders/{order_id}` | YES | `SpotAmendOrder` |
| **Spot Trading** | Batch Orders | `POST /spot/batch_orders` | YES | `SpotBatchOrders` |
| **Spot Trading** | Cancel Batch | `DELETE /spot/cancel_batch_orders` | NO | Batch cancel by ID list |
| **Spot Trading** | My Trades | `GET /spot/my_trades` | NO | Personal trade history |
| **Spot Trading** | Account Book | `GET /spot/account_book` | NO | Account transaction history |
| **Spot Trading** | Fee Rates | `GET /spot/fee` | NO | Account fee schedule |
| **Spot Trading** | Price Orders (create) | `POST /spot/price_orders` | YES | `SpotPriceOrders` |
| **Spot Trading** | Price Orders (list) | `GET /spot/price_orders` | NO | List active stop orders |
| **Spot Trading** | Price Orders (cancel all) | `DELETE /spot/price_orders` | NO | Cancel all stop orders |
| **Spot Account** | Spot Accounts | `GET /spot/accounts` | YES | `SpotAccounts` |
| **Futures Market** | Contracts | `GET /futures/{settle}/contracts` | YES | `FuturesContracts` |
| **Futures Market** | Contract Detail | `GET /futures/{settle}/contracts/{contract}` | NO | Single contract spec |
| **Futures Market** | Tickers | `GET /futures/{settle}/tickers` | YES | `FuturesTickers` |
| **Futures Market** | Orderbook | `GET /futures/{settle}/order_book` | YES | `FuturesOrderbook` |
| **Futures Market** | Klines | `GET /futures/{settle}/candlesticks` | YES | `FuturesKlines` |
| **Futures Market** | Funding Rate | `GET /futures/{settle}/funding_rate` | YES | `FundingRate` |
| **Futures Market** | Recent Trades | `GET /futures/{settle}/trades` | NO | Public futures trade history |
| **Futures Market** | Index Constituents | `GET /futures/{settle}/index_constituents/{index}` | NO | Index composition |
| **Futures Market** | Liquidations | `GET /futures/{settle}/liq_orders` | NO | Recent liquidation orders |
| **Futures Market** | Premium Index | `GET /futures/{settle}/premium_index` | NO | Funding premium |
| **Futures Market** | Stats | `GET /futures/{settle}/contract_stats` | NO | Long/short ratio, position stats |
| **Futures Market** | Insurance | `GET /futures/{settle}/insurance` | NO | Insurance fund history |
| **Futures Trading** | Place Order | `POST /futures/{settle}/orders` | YES | `FuturesCreateOrder` |
| **Futures Trading** | Cancel Order | `DELETE /futures/{settle}/orders/{order_id}` | YES | `FuturesCancelOrder` |
| **Futures Trading** | Get Order | `GET /futures/{settle}/orders/{order_id}` | YES | `FuturesGetOrder` |
| **Futures Trading** | Open Orders | `GET /futures/{settle}/orders` | YES | `FuturesOpenOrders` |
| **Futures Trading** | Cancel All Orders | `DELETE /futures/{settle}/orders` | YES | `FuturesCancelAllOrders` |
| **Futures Trading** | Amend Order | `PATCH /futures/{settle}/orders/{order_id}` | YES | `FuturesAmendOrder` |
| **Futures Trading** | Batch Orders | `POST /futures/{settle}/batch_orders` | YES | `FuturesBatchOrders` |
| **Futures Trading** | My Trades | `GET /futures/{settle}/my_trades` | NO | Personal futures trade history |
| **Futures Trading** | Price-Triggered Orders | `POST /futures/{settle}/price_orders` | NO | Futures stop/trigger orders |
| **Futures Trading** | List Price Orders | `GET /futures/{settle}/price_orders` | NO | Active futures price orders |
| **Futures Account** | Futures Account | `GET /futures/{settle}/accounts` | YES | `FuturesAccounts` |
| **Futures Account** | Positions | `GET /futures/{settle}/positions` | YES | `FuturesPositions` |
| **Futures Account** | Position Detail | `GET /futures/{settle}/positions/{contract}` | YES | `FuturesPosition` |
| **Futures Account** | Set Leverage | `POST /futures/{settle}/positions/{contract}/leverage` | YES | `FuturesSetLeverage` |
| **Futures Account** | Set Risk Limit | `POST /futures/{settle}/positions/{contract}/risk_limit` | NO | Adjust position risk limit |
| **Futures Account** | Set Margin | `POST /futures/{settle}/positions/{contract}/margin` | NO | Add/reduce isolated margin |
| **Futures Account** | Dual Mode | `POST /futures/{settle}/dual_mode` | NO | Toggle hedge/one-way mode |
| **Futures Account** | Account Book | `GET /futures/{settle}/account_book` | NO | Futures account history |
| **Wallet** | Transfer | `POST /wallet/transfers` | YES | `WalletTransfer` |
| **Wallet** | Transfer History | `GET /wallet/transfers` | YES | `WalletTransferHistory` |
| **Wallet** | Deposit Address | `GET /wallet/deposit_address` | YES | `DepositAddress` |
| **Wallet** | Withdraw | `POST /withdrawals` | YES | `Withdraw` |
| **Wallet** | Deposit History | `GET /wallet/deposits` | YES | `DepositHistory` |
| **Wallet** | Withdrawal History | `GET /wallet/withdrawals` | YES | `WithdrawalHistory` |
| **Wallet** | Cancel Withdrawal | `DELETE /withdrawals/{withdrawal_id}` | NO | Cancel pending withdrawal |
| **Wallet** | Saved Addresses | `GET /wallet/saved_address` | NO | Saved withdrawal addresses |
| **Wallet** | Total Balances | `GET /wallet/total_balance` | NO | Total balance across accounts |
| **Wallet** | Withdraw Status | `GET /wallet/withdraw_status` | NO | Per-currency withdraw status |
| **Wallet** | Small Balance Currencies | `GET /wallet/small_balance` | NO | Dust balance list |
| **Wallet** | Convert Small Balance | `POST /wallet/small_balance` | NO | Convert dust to GT/USDT |
| **Sub-Accounts** | Create Sub | `POST /sub_accounts` | YES | `SubAccountCreate` |
| **Sub-Accounts** | List Subs | `GET /sub_accounts` | YES | `SubAccountList` |
| **Sub-Accounts** | Sub Transfer | `POST /sub_accounts/transfers` | YES | `SubAccountTransfer` |
| **Sub-Accounts** | Sub Balances | `GET /sub_accounts/{user_id}/balances` | YES | `SubAccountBalance` |
| **Sub-Accounts** | Sub Transfer History | `GET /sub_accounts/transfers` | NO | History of sub transfers |
| **Sub-Accounts** | Sub API Keys Create | `POST /sub_accounts/{user_id}/keys` | NO | Create sub-account API key |
| **Sub-Accounts** | Sub API Keys List | `GET /sub_accounts/{user_id}/keys` | NO | List sub-account API keys |
| **Sub-Accounts** | Sub API Keys Update | `PUT /sub_accounts/{user_id}/keys/{key}` | NO | Update sub-account API key |
| **Sub-Accounts** | Sub API Keys Delete | `DELETE /sub_accounts/{user_id}/keys/{key}` | NO | Delete sub-account API key |
| **Margin** | Margin Accounts | `GET /margin/accounts` | NO | Margin account balances |
| **Margin** | Margin Balance History | `GET /margin/account_book` | NO | Margin account transactions |
| **Margin** | Cross Margin Funding | `GET /margin/cross/currencies` | NO | Cross margin loanable coins |
| **Margin** | Cross Margin Balance | `GET /margin/cross/accounts` | NO | Cross margin account balance |
| **Margin** | Borrow (cross) | `POST /margin/cross/loans` | NO | Borrow in cross margin |
| **Margin** | Repay (cross) | `POST /margin/cross/repayments` | NO | Repay cross margin loan |
| **Margin** | Loan History | `GET /margin/cross/loans` | NO | Cross margin loan history |
| **Margin** | Interest History | `GET /margin/cross/interest_records` | NO | Margin interest records |
| **Delivery Futures** | Contracts | `GET /delivery/{settle}/contracts` | NO | Delivery futures contracts |
| **Delivery Futures** | Tickers | `GET /delivery/{settle}/tickers` | NO | Delivery futures tickers |
| **Delivery Futures** | Orderbook | `GET /delivery/{settle}/order_book` | NO | Delivery futures orderbook |
| **Delivery Futures** | Orders | `POST /delivery/{settle}/orders` | NO | Delivery futures trading |
| **Options** | Underlyings | `GET /options/underlyings` | NO | Options underlying assets |
| **Options** | Expirations | `GET /options/expirations` | NO | Available expiry dates |
| **Options** | Contracts | `GET /options/contracts` | NO | Options contracts list |
| **Options** | Tickers | `GET /options/tickers` | NO | Options market data |
| **Options** | Orders | `POST /options/orders` | NO | Options trading |

### 4.2 Key Missing Groups

- **Spot Public Trades**: No `GET /spot/trades` endpoint for trade history
- **Spot My Trades + Account Book**: Execution history and ledger both missing
- **Spot Stop Order List/Cancel**: Price order management incomplete
- **Futures Market Stats**: Long/short ratios, insurance fund, liquidations (5 endpoints)
- **Futures Price-Triggered Orders**: Stop/trigger orders for futures
- **Futures Position Management**: Risk limit, isolated margin adjustment, dual mode toggle
- **Margin Trading**: Entire cross-margin borrowing system absent (8 endpoints)
- **Delivery Futures**: Not represented at all
- **Options**: Not represented at all
- **Wallet Utilities**: Cancel withdrawal, saved addresses, small balance conversion
- **Sub-Account API Keys**: API key management for sub-accounts

---

## 5. MEXC

**Current implementation**: `crypto/cex/mexc/endpoints.rs`
**Docs**: https://mexcdevelop.github.io/apidocs/spot_v3_en/ and https://mexcdevelop.github.io/apidocs/contract_v1_en/

### 5.1 Endpoint Gap Table

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| **Spot Market Data** | Ping | `GET /api/v3/ping` | YES | `Ping` |
| **Spot Market Data** | Server Time | `GET /api/v3/time` | YES | `ServerTime` |
| **Spot Market Data** | Default Symbols | `GET /api/v3/defaultSymbols` | NO | Default trading pairs list |
| **Spot Market Data** | Exchange Info | `GET /api/v3/exchangeInfo` | YES | `ExchangeInfo` |
| **Spot Market Data** | Orderbook | `GET /api/v3/depth` | YES | `Orderbook` |
| **Spot Market Data** | Recent Trades | `GET /api/v3/trades` | YES | `RecentTrades` |
| **Spot Market Data** | Agg Trades | `GET /api/v3/aggTrades` | NO | Compressed/aggregated trade history |
| **Spot Market Data** | Klines | `GET /api/v3/klines` | YES | `Klines` |
| **Spot Market Data** | Avg Price | `GET /api/v3/avgPrice` | YES | `AvgPrice` |
| **Spot Market Data** | 24hr Ticker | `GET /api/v3/ticker/24hr` | YES | `Ticker24hr` |
| **Spot Market Data** | Price Ticker | `GET /api/v3/ticker/price` | YES | `TickerPrice` |
| **Spot Market Data** | Book Ticker | `GET /api/v3/ticker/bookTicker` | YES | `BookTicker` |
| **Spot Account** | Account Info | `GET /api/v3/account` | YES | `Account` |
| **Spot Account** | My Trades | `GET /api/v3/myTrades` | YES | `MyTrades` |
| **Spot Account** | Trade Fee | `GET /api/v3/tradeFee` | YES | `TradeFee` |
| **Spot Account** | KYC Status | `GET /api/v3/kyc/status` | NO | Know-Your-Customer verification level |
| **Spot Account** | Self Symbols | `GET /api/v3/selfSymbols` | NO | User's available trading pairs |
| **Spot Account** | MX Deduct Enable | `POST /api/v3/mxDeduct/enable` | NO | Enable MX token fee discount |
| **Spot Account** | MX Deduct Query | `GET /api/v3/mxDeduct/enable` | NO | Query MX deduction status |
| **Spot Trading** | Place Order | `POST /api/v3/order` | YES | `PlaceOrder` |
| **Spot Trading** | Test Order | `POST /api/v3/order/test` | YES | `TestOrder` |
| **Spot Trading** | Cancel Order | `DELETE /api/v3/order` | YES | `CancelOrder` |
| **Spot Trading** | Cancel All Orders | `DELETE /api/v3/openOrders` | YES | `CancelAllOrders` |
| **Spot Trading** | Query Order | `GET /api/v3/order` | YES | `QueryOrder` |
| **Spot Trading** | Open Orders | `GET /api/v3/openOrders` | YES | `OpenOrders` |
| **Spot Trading** | All Orders | `GET /api/v3/allOrders` | YES | `AllOrders` |
| **Spot Trading** | Batch Orders | `POST /api/v3/batchOrders` | YES | `BatchOrders` |
| **Spot Trading** | Cancel Batch | `DELETE /api/v3/batchOrders` | YES | `BatchOrdersCancel` |
| **Wallet** | Currency Config | `GET /api/v3/capital/config/getall` | NO | Currency chain/contract details |
| **Wallet** | Withdraw | `POST /api/v3/capital/withdraw` | YES | `Withdraw` |
| **Wallet** | Cancel Withdraw | `DELETE /api/v3/capital/withdraw` | NO | Cancel pending withdrawal |
| **Wallet** | Deposit History | `GET /api/v3/capital/deposit/hisrec` | YES | `DepositHistory` |
| **Wallet** | Withdraw History | `GET /api/v3/capital/withdraw/history` | YES | `WithdrawHistory` |
| **Wallet** | Generate Deposit Address | `POST /api/v3/capital/deposit/address` | NO | Create deposit address |
| **Wallet** | Get Deposit Address | `GET /api/v3/capital/deposit/address` | YES | `DepositAddress` |
| **Wallet** | Withdrawal Addresses | `GET /api/v3/capital/withdraw/address` | NO | Saved withdrawal addresses |
| **Wallet** | Transfer | `POST /api/v3/capital/transfer` | YES | `Transfer` |
| **Wallet** | Transfer History | `GET /api/v3/capital/transfer` | YES | `TransferHistory` |
| **Wallet** | Transfer by TranId | `GET /api/v3/capital/transfer/tranId` | NO | Lookup transfer by ID |
| **Wallet** | Convert to MX (list) | `GET /api/v3/capital/convert/list` | NO | Dust assets convertible to MX |
| **Wallet** | Convert Dust to MX | `POST /api/v3/capital/convert` | NO | Convert small balances |
| **Wallet** | Convert History | `GET /api/v3/capital/convert` | NO | Dust conversion history |
| **Wallet** | Internal Transfer | `POST /api/v3/capital/transfer/internal` | NO | Transfer within MEXC accounts |
| **Wallet** | Internal Transfer History | `GET /api/v3/capital/transfer/internal` | NO | Internal transfer records |
| **Sub-Accounts** | Create Sub | `POST /api/v3/sub-account/virtualSubAccount` | YES | `SubAccountCreate` |
| **Sub-Accounts** | List Subs | `GET /api/v3/sub-account/list` | YES | `SubAccountList` |
| **Sub-Accounts** | Sub Transfer | `POST /api/v3/capital/sub-account/universalTransfer` | YES | `SubAccountTransfer` |
| **Sub-Accounts** | Sub Assets | `GET /api/v3/sub-account/assets` | YES | `SubAccountAssets` |
| **Sub-Accounts** | Sub API Key Create | `POST /api/v3/sub-account/apiKey` | NO | Generate API key for sub |
| **Sub-Accounts** | Sub API Key List | `GET /api/v3/sub-account/apiKey` | NO | Retrieve sub API keys |
| **Sub-Accounts** | Sub API Key Delete | `DELETE /api/v3/sub-account/apiKey` | NO | Remove sub API key |
| **Sub-Accounts** | Sub Asset Detail | `GET /api/v3/sub-account/asset` | NO | Sub-account balance detail |
| **Sub-Accounts** | Sub Transfer History | `GET /api/v3/capital/sub-account/universalTransfer` | NO | Sub transfer history query |
| **Futures Market** | Futures Ping | `GET /api/v1/contract/ping` | YES | `FuturesPing` |
| **Futures Market** | Contract Info | `GET /api/v1/contract/detail` | YES | `FuturesContractInfo` |
| **Futures Market** | Support Currencies | `GET /api/v1/contract/support_currencies` | NO | Transferable currency list |
| **Futures Market** | Orderbook | `GET /api/v1/contract/depth/{symbol}` | YES | `FuturesOrderbook` |
| **Futures Market** | Depth Snapshots | `GET /api/v1/contract/depth_commits/{symbol}/{limit}` | NO | Order book snapshots |
| **Futures Market** | Index Price | `GET /api/v1/contract/index_price/{symbol}` | NO | Index price data |
| **Futures Market** | Fair Price | `GET /api/v1/contract/fair_price/{symbol}` | NO | Mark/fair price |
| **Futures Market** | Funding Rate | `GET /api/v1/contract/funding_rate/{symbol}` | NO | Current funding rate |
| **Futures Market** | Klines | `GET /api/v1/contract/kline/{symbol}` | YES | `FuturesKlines` |
| **Futures Market** | Index Price Klines | `GET /api/v1/contract/kline/index_price/{symbol}` | NO | Index price candles |
| **Futures Market** | Fair Price Klines | `GET /api/v1/contract/kline/fair_price/{symbol}` | NO | Fair price candles |
| **Futures Market** | Recent Trades | `GET /api/v1/contract/deals/{symbol}` | YES | `FuturesRecentTrades` |
| **Futures Market** | Ticker | `GET /api/v1/contract/ticker` | YES | `FuturesTicker` |
| **Futures Market** | Risk Reverse | `GET /api/v1/contract/risk_reverse` | NO | Risk fund balance |
| **Futures Market** | Risk Reverse History | `GET /api/v1/contract/risk_reverse/history` | NO | Insurance fund history |
| **Futures Market** | Funding Rate History | `GET /api/v1/contract/funding_rate/history` | NO | Historical funding rates |
| **Futures Account** | All Assets | `GET /api/v1/private/account/assets` | NO | Futures account balances |
| **Futures Account** | Single Asset | `GET /api/v1/private/account/asset/{currency}` | NO | Single currency balance |
| **Futures Account** | Transfer Records | `GET /api/v1/private/account/transfer_record` | NO | Futures transfer history |
| **Futures Account** | Fee Rate | `GET /api/v1/private/account/tiered_fee_rate` | NO | Futures fee tiers |
| **Futures Account** | Risk Limit | `GET /api/v1/private/account/risk_limit` | NO | Account risk limits |
| **Futures Positions** | Open Positions | `GET /api/v1/private/position/open_positions` | NO | Active futures positions |
| **Futures Positions** | Position History | `GET /api/v1/private/position/list/history_positions` | NO | Historical positions |
| **Futures Positions** | Funding Records | `GET /api/v1/private/position/funding_records` | NO | Funding payment history |
| **Futures Positions** | Change Margin | `POST /api/v1/private/position/change_margin` | NO | Adjust position margin |
| **Futures Positions** | Get Leverage | `GET /api/v1/private/position/leverage` | NO | Current leverage setting |
| **Futures Positions** | Change Leverage | `POST /api/v1/private/position/change_leverage` | NO | Modify leverage |
| **Futures Positions** | Position Mode | `GET /api/v1/private/position/position_mode` | NO | Hedge vs one-way mode |
| **Futures Positions** | Change Position Mode | `POST /api/v1/private/position/change_position_mode` | NO | Toggle position mode |
| **Futures Orders** | Place Order | `POST /api/v1/private/order/submit` | NO | Place futures order |
| **Futures Orders** | Batch Orders | `POST /api/v1/private/order/submit_batch` | NO | Place multiple futures orders |
| **Futures Orders** | Cancel Orders | `POST /api/v1/private/order/cancel` | NO | Cancel futures order(s) |
| **Futures Orders** | Cancel All | `POST /api/v1/private/order/cancel_all` | NO | Cancel all futures orders |
| **Futures Orders** | Open Orders | `GET /api/v1/private/order/list/open_orders/{symbol}` | NO | Active futures orders |
| **Futures Orders** | Order History | `GET /api/v1/private/order/list/history_orders` | NO | Historical futures orders |
| **Futures Orders** | Get Order by ID | `GET /api/v1/private/order/get/{order_id}` | NO | Futures order detail |
| **Futures Orders** | Trade Details | `GET /api/v1/private/order/deal_details/{order_id}` | NO | Fills for futures order |
| **Futures Orders** | All Trade Details | `GET /api/v1/private/order/list/order_deals` | NO | All futures executions |
| **Futures Stop Orders** | Trigger Orders List | `GET /api/v1/private/planorder/list/orders` | NO | Trigger order list |
| **Futures Stop Orders** | Place Trigger Order | `POST /api/v1/private/planorder/place` | NO | Conditional/trigger order |
| **Futures Stop Orders** | Cancel Trigger Orders | `POST /api/v1/private/planorder/cancel` | NO | Cancel trigger orders |
| **Futures Stop Orders** | Cancel All Triggers | `POST /api/v1/private/planorder/cancel_all` | NO | Cancel all trigger orders |
| **Futures Stop Orders** | Stop-Loss List | `GET /api/v1/private/stoporder/list/orders` | NO | TP/SL order list |
| **Futures Stop Orders** | Cancel Stop Orders | `POST /api/v1/private/stoporder/cancel` | NO | Cancel stop-limit orders |
| **Futures Stop Orders** | Cancel All Stops | `POST /api/v1/private/stoporder/cancel_all` | NO | Cancel all stop orders |
| **Futures Stop Orders** | Change Stop Price | `POST /api/v1/private/stoporder/change_price` | NO | Modify TP/SL prices |
| **Futures Stop Orders** | Change Trigger Price | `POST /api/v1/private/stoporder/change_plan_price` | NO | Modify trigger price |

### 5.2 Key Missing Groups

- **Futures Trading (all private)**: The entire authenticated futures trading system is absent — 10 order endpoints, 8 position endpoints, 9 stop-order endpoints (27 total)
- **Futures Account**: Balances, fees, risk limits all missing (5 endpoints)
- **Futures Market Data**: Funding rate, index/fair prices, insurance fund (9 endpoints)
- **Wallet**: Cancel withdrawal, currency configs, dust conversion, internal transfers (10 endpoints)
- **Sub-Account API Keys**: API key CRUD for sub-accounts (3 endpoints)
- **KYC/Account Settings**: KYC status, self symbols, MX deduction toggle

---

## Cross-Exchange Summary: Top Missing Categories

| Category | KuCoin | Bitfinex | HTX | Gate.io | MEXC |
|----------|--------|----------|-----|---------|------|
| Margin/Lending | MISSING | MISSING | MISSING | MISSING | N/A |
| Futures Private Trading | partial | N/A | NO futures trading | partial | ENTIRELY MISSING |
| Futures Mark/Index Price | NO | N/A | NO | NO | NO |
| Funding Rate History | NO | N/A | NO | NO | NO |
| Stop/Trigger Orders | NO | N/A | NO | NO | NO |
| Trade History (fills) | NO | YES | partial | NO | YES |
| Cancel by ClientOid | NO | N/A | NO | N/A | N/A |
| Sub-Account API Keys | NO | N/A | NO | NO | NO |
| Withdrawal Cancel/Quotas | NO | NO | partial | NO | NO |
| Account Ledger/History | NO | NO | partial | NO | N/A |
| Batch Cancel Orders | N/A | YES | NO | NO | YES |
| Currencies/Chains Info | NO | partial | NO | NO | NO |

---

## Sources

- KuCoin REST API: https://www.kucoin.com/docs/rest/spot-trading/market-data/introduction
- KuCoin Futures: https://www.kucoin.com/docs/rest/futures-trading/market-data/introduction
- KuCoin Margin: https://www.kucoin.com/docs/rest/margin-trading/introduction
- Bitfinex v2 REST: https://docs.bitfinex.com/reference/rest-public-tickers
- Bitfinex Auth: https://docs.bitfinex.com/reference/rest-auth-funding-offers
- HTX (Huobi) Spot: https://huobiapi.github.io/docs/spot/v1/en/
- Gate.io API v4: https://www.gate.com/docs/developers/apiv4/en/
- MEXC Spot v3: https://mexcdevelop.github.io/apidocs/spot_v3_en/
- MEXC Futures v1: https://mexcdevelop.github.io/apidocs/contract_v1_en/
