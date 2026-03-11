# CEX Trading API Capability Matrix — Batch 1 (8 Exchanges)

Generated from research files in `digdigdig3/src/crypto/cex/*/research/trading-research/`

Legend: **Y** = Yes / **N** = No / **P** = Partial / **S** = Spot only / **F** = Futures only / **n** = number

---

## 1. ORDER TYPES

| Feature | Binance | Bybit | OKX | KuCoin | Kraken | Coinbase | Gate.io | Bitfinex |
|---------|---------|-------|-----|--------|--------|----------|---------|----------|
| **Market** | Y | Y | Y | Y | Y | Y | Y | Y |
| **Limit** | Y | Y | Y | Y | Y | Y | Y | Y |
| **StopMarket** | Y (F: `STOP_MARKET`; S: `STOP_LOSS`) | Y (via `triggerPrice` + Market) | Y (algo: `trigger` w/ market) | Y (S: stop-order endpoint; F: `stop=up/down`) | Y (`stop-loss`, `stp`) | N (stop-limit only) | Y (price_orders) | Y (`STOP`, `EXCHANGE STOP`) |
| **StopLimit** | Y (F: `STOP`; S: `STOP_LOSS_LIMIT`) | Y (via `triggerPrice` + Limit) | Y (algo: `trigger` w/ limit) | Y (S: stop endpoint `type=limit`; F: embedded) | Y (`stop-loss-limit`, `stp` w/ `limitPrice`) | Y (`stop_limit_stop_limit_gtc/gtd`) | Y (price_orders `type=limit`) | Y (`STOP LIMIT`, `EXCHANGE STOP LIMIT`) |
| **TrailingStop** | Y (F: `TRAILING_STOP_MARKET`; S: `trailingDelta`) | Y (`trailingStop` via `trading-stop`) | Y (algo: `move_order_stop`) | N (no native trailing) | Y (`trailing-stop`, `trailing_stop`) | N | N | Y (`TRAILING STOP`, `EXCHANGE TRAILING STOP`) |
| **TP** | Y (separate order: `TAKE_PROFIT`/`TAKE_PROFIT_MARKET`) | Y (inline `takeProfit` param or `trading-stop`) | Y (inline `tpTriggerPx` or algo `conditional`/`oco`) | Y (S: stop-order `stop=entry`; F: `st-orders` `triggerStopUpPrice`) | Y (`take-profit`, `take_profit`) | P (via bracket order's `limit_price`) | Y (price_orders) | Y (inline OCO flag or separate STOP order) |
| **SL** | Y (separate order: `STOP_MARKET` reduceOnly) | Y (inline `stopLoss` param or `trading-stop`) | Y (inline `slTriggerPx` or algo `conditional`/`oco`) | Y (S: stop-order `stop=loss`; F: `st-orders` `triggerStopDownPrice`) | Y (`stop-loss`, `stp`) | P (via bracket's `stop_trigger_price`) | Y (price_orders) | Y (inline OCO flag or separate STOP order) |
| **OCO** | Y (S: `POST /api/v3/orderList/oco`; F: N) | Y (S: `OcoOrder` orderFilter) | Y (algo: `oco` type) | N (no native OCO) | N | N | N | Y (`flags=16384` + `price_oco_stop`) |
| **Bracket (Entry+TP+SL one call)** | N | N | N | N | N (but S has `close[]` OTO) | Y (`trigger_bracket_gtc/gtd` or `attached_order_configuration`) | N | N |
| **PostOnly** | Y (S: `LIMIT_MAKER`; F: `GTX` TIF) | Y (`PostOnly` TIF) | Y (`post_only` ordType) | Y (`postOnly=true` param) | Y (`post` oflag; F: `post` orderType) | Y (`post_only` in limit configs) | Y (`poc` TIF) | Y (`POST_ONLY` flag 4096) |
| **Iceberg** | Y (S: `icebergQty` param; F: N) | N | Y (algo: `iceberg` type) | Y (S+F: `iceberg=true` + `visibleSize`) | Y (S: `displayvol`; F: N) | N | Y (S+F: `iceberg` field) | Y (`HIDDEN` flag 64) |

---

## 2. TIME IN FORCE

| TIF | Binance | Bybit | OKX | KuCoin | Kraken | Coinbase | Gate.io | Bitfinex |
|-----|---------|-------|-----|--------|--------|----------|---------|----------|
| **GTC** | Y | Y | Y (implicit in `limit`) | Y | Y | Y (`limit_limit_gtc`) | Y (`gtc`) | Y (default for most types) |
| **IOC** | Y | Y | Y (`ioc` ordType) | Y | Y | Y (`market_market_ioc`, `sor_limit_ioc`) | Y (`ioc`) | Y (`IOC`, `EXCHANGE IOC`) |
| **FOK** | Y | Y | Y (`fok` ordType) | Y | N (Spot N; F: `ioc` only) | Y (`market_market_fok`, `limit_limit_fok`) | Y (`fok`) | Y (`FOK`, `EXCHANGE FOK`) |
| **PostOnly** | Y (see order types) | Y (`PostOnly` TIF value) | Y (`post_only` ordType) | Y (param, not TIF enum) | Y (`post` oflag) | Y (`post_only` field in limit config) | Y (`poc` = Post-Or-Cancel) | Y (flag 4096) |
| **GTD** | Y (F: `GTD` TIF; S: N) | N | N | Y (`GTT` = Good-Till-Time, via `cancelAfter` seconds) | Y (S: `GTD` + `expiretm`) | Y (`limit_limit_gtd`, `stop_limit_stop_limit_gtd`, `trigger_bracket_gtd`, `twap_limit_gtd`) | N | Y (`tif` = datetime string) |

---

## 3. ORDER MANAGEMENT

| Feature | Binance | Bybit | OKX | KuCoin | Kraken | Coinbase | Gate.io | Bitfinex |
|---------|---------|-------|-----|--------|--------|----------|---------|----------|
| **Single Create** | Y | Y | Y | Y | Y | Y | Y | Y |
| **Batch Create (max)** | F: 5; S: N | 20 (F/inverse/option), 10 (S) | 20 | S: 5 (HF/classic); F: no stated limit | S: 15 (same pair) | N | S: 10; F: no stated limit | 75 ops (mixed multi-op) |
| **Cancel Single** | Y | Y | Y | Y | Y | Y (batch_cancel w/ 1 id) | Y | Y |
| **Cancel All** | Y | Y | Y | Y | Y | P (batch_cancel, no "all" endpoint) | Y (by symbol) | Y (`all=1` in cancel/multi) |
| **Cancel By Symbol** | Y | Y | Y (`instId` param) | Y | N (cancel by txid/userref only) | N | Y | Y (optional symbol in active orders endpoint) |
| **Amend/Modify Order** | Y (F: PUT price+qty; S: qty-only keepPriority) | Y (`/v5/order/amend` — price+qty+TP/SL) | Y (`amend-order` — price+qty+TP/SL) | Y (HF: `alter` — cancel+recreate internally; F: N) | P (S: `EditOrder` deprecated, WS AmendOrder preferred; F: `editorder`) | P (`limit_limit_gtc` only, edit price+size) | Y (`PATCH /spot/orders/{id}`, `PATCH /futures/{settle}/orders/{id}`) | Y (`/v2/auth/w/order/update` — delta/price/amount) |
| **Batch Amend (max)** | F: 5; S: N | 20 (F), 10 (S) | 20 | N | N | N | Y (`POST /spot/amend_batch_orders`) | 75 ops (mixed) |
| **Get Single Order** | Y | Y (`/v5/order/realtime` + orderId) | Y (`GET /api/v5/trade/order`) | Y | Y (`QueryOrders`) | Y (`GET /orders/historical/{order_id}`) | Y | Y (`/v2/auth/r/orders/{symbol}`) |
| **Get Open Orders** | Y | Y (`/v5/order/realtime`) | Y (`orders-pending`) | Y (`/hf/orders/active`) | Y (`OpenOrders`) | Y (`list_orders` w/ `status=OPEN`) | Y (`GET /spot/orders?status=open`) | Y (`/v2/auth/r/orders`) |
| **Get History** | Y | Y (`/v5/order/history`) | Y (`orders-history` 7d; `orders-history-archive` 3mo) | Y | Y (`ClosedOrders`) | Y (`list_orders` with date filters) | Y (`GET /spot/orders?status=finished`) | Y (`/v2/auth/r/orders/hist`, 2 weeks, max 2500) |

---

## 4. POSITIONS (Futures/Derivatives)

| Feature | Binance | Bybit | OKX | KuCoin | Kraken | Coinbase | Gate.io | Bitfinex |
|---------|---------|-------|-----|--------|--------|----------|---------|----------|
| **GetPositions** | Y (`GET /fapi/v2/positionRisk`) | Y (`GET /v5/position/list`) | Y (`GET /api/v5/account/positions`) | Y (`GET /api/v1/positions` on futures host) | Y (S: `OpenPositions`; F: `GET openpositions`) | Y (INTX: `GET /intx/positions/{portfolio_uuid}`) | Y (`GET /futures/{settle}/positions`) | Y (`POST /v2/auth/r/positions`) |
| **ClosePosition** | P (no dedicated endpoint; use MARKET order or `closePosition=true`) | P (no dedicated endpoint; use `reduceOnly` order or `trading-stop`) | Y (`POST /api/v5/trade/close-position`) | P (`closeOrder=true` on futures order) | P (`settle-position` ordertype) | N (no direct; place SELL MARKET) | P (`size=0` or `close=true` on order) | P (flag 512 `CLOSE` on opposing order) |
| **SetLeverage** | Y (`POST /fapi/v1/leverage`) | Y (`POST /v5/position/set-leverage`) | Y (`POST /api/v5/account/set-leverage`) | P (leverage per order param; risk level via `risk-limit-level/change`) | Y (F: `PUT /leveragepreferences`; S: per-order param only) | P (via `leverage` field on order create for INTX) | Y (`POST /futures/{settle}/positions/{contract}/leverage`) | P (per-order `lev` param 1–100) |
| **MarginMode (Cross/Isolated)** | Y (`POST /fapi/v1/marginType` ISOLATED/CROSSED) | Y (`POST /v5/position/switch-isolated` 0=cross, 1=isolated) | Y (per order `tdMode=isolated/cross`; account `set-leverage` `mgnMode`) | Y (`marginMode=ISOLATED/CROSS` on futures order) | N (Spot margin integrated; Futures uses per-symbol margin account) | Y (`POST /intx/order_book/set_margin_type`) | P (`cross_leverage_limit` param; margin_mode in account) | N (margin vs exchange is by order type prefix, not a mode switch) |
| **AddRemoveMargin** | Y (`POST /fapi/v1/positionMargin` type=1 add, type=2 reduce) | Y (`POST /v5/position/add-margin` positive/negative) | Y (`POST /api/v5/account/position-margin` type=add/reduce) | Y (`POST /api/v1/position/margin/deposit-margin`; auto via `autoDeposit`) | N | N | Y (`POST /futures/{settle}/positions/{contract}/margin?change=±N`) | N |
| **FundingRate** | Y (public endpoint; F income history includes `FUNDING_FEE`) | Y (`GET /v5/market/funding/history`) | Y (public; account bills have type=8 funding fee) | Y (`GET /api/v1/funding-history` on futures host) | Y (F: unrealizedFunding in position; funding via WebSocket) | Y (INTX positions include funding data) | Y (`GET /futures/{settle}/account_book?type=fund`) | Y (`MARGIN_FUNDING` in position array) |
| **LiqPrice** | Y (`liquidationPrice` in positionRisk response) | Y (`liqPrice` in position/list response) | Y (`liqPx` in positions response) | Y (`liquidationPrice` in position response) | Y (F: via `triggerEstimates`; S: `ml` margin level) | Y (`liquidation_price` in INTX position) | Y (`liq_price` in position response) | Y (`PRICE_LIQ` in position array [8]) |

---

## 5. ACCOUNT

| Feature | Binance | Bybit | OKX | KuCoin | Kraken | Coinbase | Gate.io | Bitfinex |
|---------|---------|-------|-----|--------|--------|----------|---------|----------|
| **Balances** | Y (`GET /api/v3/account` spot; `GET /fapi/v2/balance` futures) | Y (`GET /v5/account/wallet-balance`) | Y (`GET /api/v5/account/balance`) | Y (`GET /api/v1/accounts`; separate futures host `GET /api/v1/account-overview`) | Y (S: `POST /0/private/Balance`; F: `GET /derivatives/api/v3/accounts`) | Y (`GET /api/v3/brokerage/accounts`) | Y (separate per type: `GET /spot/accounts`, `GET /futures/usdt/accounts`, `GET /unified/accounts`) | Y (`POST /v2/auth/r/wallets`) |
| **Fees** | Y (S: `GET /api/v3/account/commission`; F: `GET /fapi/v1/commissionRate`) | Y (`GET /v5/account/fee-rate`) | Y (`GET /api/v5/account/trade-fee`) | Y (`GET /api/v1/trade-fees`; `GET /api/v1/base-fee`) | Y (`POST /0/private/TradeVolume`) | Y (`GET /api/v3/brokerage/transaction_summary`) | Y (`GET /spot/fee`) | P (fee in individual trade responses; no dedicated fee-tier endpoint) |
| **InternalTransfer** | Y (`POST /sapi/v1/asset/transfer` with 30+ type enums) | Y (`POST /v5/asset/transfer/inter-transfer`) | Y (`POST /api/v5/asset/transfer`) | Y (`POST /api/v3/accounts/universal-transfer`) | Y (`POST /0/private/WalletTransfer` Spot↔Futures) | Y (`POST /api/v3/brokerage/portfolios/move_funds`) | Y (`POST /wallet/transfers`) | Y (`POST /v2/auth/w/transfer` exchange/margin/funding) |
| **DepositAddr** | Y (`GET /sapi/v1/capital/deposit/address`) | Y (`GET /v5/asset/deposit/query-address`) | Y (`GET /api/v5/asset/deposit-address`) | Y (`GET /api/v2/deposit-addresses`) | Y (`POST /0/private/DepositAddresses`) | N (not in Advanced Trade API; via retail wallet API) | N (not in researched endpoints) | N (not documented in researched files) |
| **Withdraw** | Y (`POST /sapi/v1/capital/withdraw/apply`) | Y (`POST /v5/asset/withdraw/create`) | Y (`POST /api/v5/asset/withdrawal`) | Y (`POST /api/v1/withdrawals`) | Y (`POST /0/private/Withdraw`) | N (Advanced Trade API only; use retail API) | Y (`POST /withdrawals`) | N (not documented in researched files) |
| **DepositWithdrawHistory** | Y (both: `GET /sapi/v1/capital/deposit/hisrec` + `withdraw/history`) | Y (both: `query-record` endpoints) | Y (both: `deposit-history` + `withdrawal-history`) | Y (both: `GET /api/v1/deposits` + `withdrawals`) | Y (both: `DepositStatus` + `WithdrawStatus`) | N | Y (both: `GET /wallet/deposits` + `withdrawals`) | N |

---

## 6. SUB-ACCOUNTS

| Feature | Binance | Bybit | OKX | KuCoin | Kraken | Coinbase | Gate.io | Bitfinex |
|---------|---------|-------|-----|--------|--------|----------|---------|----------|
| **Create Sub** | Y (via broker API) | N (not in researched docs) | Y (`type=1/2` in account config; managed via portal) | N (not in researched docs) | N | N | N | Y (via `email_dst` / `user_id_dst` in Transfer) |
| **List Sub-Accounts** | Y (broker API) | N | N | N | N | N | Y (`GET /sub_accounts`) | N |
| **Transfer to/from Sub** | Y (universal transfer `type=MAIN_SUB` etc.) | Y (inter-transfer with sub accountType) | Y (`POST /api/v5/asset/transfer` `type=1/2/3`) | Y (`POST /api/v3/accounts/universal-transfer` `type=PARENT_TO_SUB`) | N | N | Y (`POST /wallet/sub_account_transfers`) | Y (via `transfer` with `user_id_dst`) |

---

## 7. ADVANCED ORDER TYPES

| Feature | Binance | Bybit | OKX | KuCoin | Kraken | Coinbase | Gate.io | Bitfinex |
|---------|---------|-------|-----|--------|--------|----------|---------|----------|
| **TWAP** | P (broker-only algo API, not public) | N | Y (algo: `twap` type, max 20/account) | N | N | Y (`twap_limit_gtd` native order type) | N | N |
| **Iceberg** | Y (S: `icebergQty`; F: N) | N | Y (algo: `iceberg` type, max 100/account) | Y (S+F: `iceberg=true` + `visibleSize`) | Y (S: `displayvol`) | N | Y (S+F: `iceberg` field) | Y (`HIDDEN` flag; order is hidden not sliced) |
| **CopyTrading** | N (not via API) | P (`isMasterTrader` flag in account info) | N | N | N | N | N | N |
| **GridTrading** | N (UI only) | N | N | N | N | N | N | N |
| **ScaledOrders** | N | N | N | N | N | Y (`scaled_limit_gtc` — splits order into N limit orders across price range) | N | N |

---

## 8. AUTHENTICATION

| Exchange | Auth Method | Signature Algorithm | Notes |
|----------|-------------|---------------------|-------|
| **Binance** | API Key in header `X-MBX-APIKEY`; HMAC-SHA256 signature of query params | HMAC-SHA256 | `timestamp` + `recvWindow` required; `signature` as query param |
| **Bybit** | API Key in header; HMAC-SHA256 signature | HMAC-SHA256 | `X-BAPI-API-KEY`, `X-BAPI-SIGN`, `X-BAPI-TIMESTAMP`, `X-BAPI-RECV-WINDOW` headers |
| **OKX** | API Key + Passphrase + HMAC-SHA256 | HMAC-SHA256 | Headers: `OK-ACCESS-KEY`, `OK-ACCESS-SIGN`, `OK-ACCESS-TIMESTAMP`, `OK-ACCESS-PASSPHRASE`; demo trading via `x-simulated-trading: 1` |
| **KuCoin** | API Key + Passphrase + HMAC-SHA256; passphrase also signed | HMAC-SHA256 | `KC-API-KEY`, `KC-API-SIGN`, `KC-API-TIMESTAMP`, `KC-API-PASSPHRASE` (passphrase is HMAC-SHA256 signed) |
| **Kraken** | API Key + HMAC-SHA512 signature | HMAC-SHA512 | S: `API-Key` + `API-Sign` headers, nonce in body; F: separate auth (HMAC-SHA256 + SHA256 of path+nonce+body) |
| **Coinbase** | JWT (Bearer token) via CDP API keys | ES256 (ECDSA) | `Authorization: Bearer <JWT>`; JWT signed with private key from CDP portal |
| **Gate.io** | API Key + HMAC-SHA512 | HMAC-SHA512 | `KEY` and `SIGN` headers; payload = `method\npath\nquery\nbody_hash\ntimestamp` |
| **Bitfinex** | API Key + HMAC-SHA384 | HMAC-SHA384 | `bfx-apikey`, `bfx-signature`, `bfx-nonce` headers; payload = `/api/v2/path` + nonce + JSON body |

---

## 9. QUICK REFERENCE SUMMARY

### Order Type Coverage Score (out of 9 types)

| Exchange | Market | Limit | StopMkt | StopLmt | Trailing | TP | SL | OCO | Bracket | **Score** |
|----------|--------|-------|---------|---------|----------|----|----|-----|---------|-----------|
| Binance | Y | Y | Y | Y | Y | Y | Y | S | N | **8.5/9** |
| Bybit | Y | Y | Y | Y | Y | Y | Y | S | N | **8.5/9** |
| OKX | Y | Y | Y | Y | Y | Y | Y | Y | N | **9/9** |
| KuCoin | Y | Y | Y | Y | N | Y | Y | N | N | **7/9** |
| Kraken | Y | Y | Y | Y | Y | Y | Y | N | P(OTO) | **7.5/9** |
| Coinbase | Y | Y | N | Y | N | P | P | N | Y | **6/9** |
| Gate.io | Y | Y | Y | Y | N | Y | Y | N | N | **7/9** |
| Bitfinex | Y | Y | Y | Y | Y | Y | Y | Y | N | **9/9** |

### Batch Operations Summary

| Exchange | Batch Create | Batch Amend | Batch Cancel |
|----------|-------------|-------------|-------------|
| Binance | F:5 | F:5 | F:10 |
| Bybit | 20/10 | 20/10 | 20/10 |
| OKX | 20 | 20 | 20 |
| KuCoin | S:5, F:unlimited | S: via alter | S: batch cancel endpoint |
| Kraken | S:15 (same pair) | N | N (cancel all only) |
| Coinbase | N | N | 100 (cancel only) |
| Gate.io | S:10, F:Y | S: batch amend | S+F: Y |
| Bitfinex | 75 mixed ops | 75 mixed ops | 75 mixed ops |

### Notable Unique Features

| Exchange | Unique Capability |
|----------|------------------|
| **Binance** | Spot OTO/OTOCO order lists; Futures GTD + GTX TIF; per-request `priceMatch` queue priority |
| **Bybit** | `POST /v5/position/trading-stop` — set TP/SL/trailing on existing position without new order |
| **OKX** | Fully native algo system (TWAP, Iceberg, Trailing, OCO, Trigger) with independent lifecycle; Unified Account spanning all products |
| **KuCoin** | `POST /api/v1/st-orders` places entry+TP+SL in one call (Futures); HF (high-frequency) endpoint system |
| **Kraken** | Dead man's switch (`CancelAllOrdersAfter`); Spot margin seamlessly integrated (no separate account); `close[]` OTO bracket on order creation |
| **Coinbase** | `trigger_bracket_gtc` — native Bracket order type (entry+TP+SL in one call); `scaled_limit_gtc` — built-in price-range order splitting; native TWAP as order type |
| **Gate.io** | `action_mode=ACK` for async high-throughput spot; `poc` (Pending-Or-Cancel) TIF; price_orders system for both spot and futures TP/SL |
| **Bitfinex** | Array-based response format (not JSON objects); `order/multi` endpoint mixes create+update+cancel in 75-op batch; exchange/margin/funding wallet separation by order type prefix |
