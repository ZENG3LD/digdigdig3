# CEX Trading API Capability Matrix — Batch 2

Exchanges: Bitstamp, MEXC, HTX, Bitget, Gemini, BingX, Phemex, Crypto.com
Generated: 2026-03-11

Legend: Y = Supported | N = Not supported | P = Partial/limited | ? = Unclear from docs

---

## 1. ORDER TYPES

| Feature | Bitstamp | MEXC | HTX | Bitget | Gemini | BingX | Phemex | Crypto.com |
|---------|----------|------|-----|--------|--------|-------|--------|------------|
| Market | Y | Y (spot+futures) | Y (spot+futures) | Y (spot+futures) | Y (via IOC limit) | Y (spot+futures) | Y (spot+futures) | Y |
| Limit | Y | Y | Y | Y | Y | Y | Y | Y |
| StopMarket | N (discontinued May 2025) | N (spot) / Y (futures) | N (spot) / Y (futures via algo) | N (spot) / Y (futures) | N | Y (futures: STOP_MARKET) | Y (futures: Stop type) | N (migrated to Advanced API) |
| StopLimit | Y (stop_price param) | N (spot) / Y (futures) | Y (spot+futures) | N (spot) / Y (futures via plan) | Y (exchange stop limit) | Y (futures: STOP) | Y (spot+futures: StopLimit) | N (migrated to Advanced API) |
| TrailingStop | N (discontinued May 2025) | N (spot) / N (futures) | Y (spot via algo-orders trailingRate) | Y (futures: track_plan) | N | Y (futures: TRAILING_STOP_MARKET) | Y (futures: TrailingStopPeg) | N |
| TP (Take Profit) | N | N (spot) / Y (futures: inline) | Y (futures inline: tp_trigger_price) | Y (spot+futures: preset/plan) | N | Y (futures: TAKE_PROFIT/TAKE_PROFIT_MARKET) | Y (futures: takeProfitEp param) | Y (via Advanced API: OTOCO) |
| SL (Stop Loss) | N | N (spot) / Y (futures: inline) | Y (futures inline: sl_trigger_price) | Y (spot+futures: preset/plan) | N | Y (futures: STOP_MARKET embedded) | Y (futures: stopLossEp param) | Y (via Advanced API: OTOCO) |
| OCO | N | N | N | N | N | N (TP+SL bracket-style only) | N | Y (private/advanced/create-oco) |
| Bracket | N | N | N | Y (futures: OTOCO-style via plan) | N | P (TP+SL attached to order) | Y (futures: up to 5 related orders) | Y (private/advanced/create-otoco) |

---

## 2. TIME-IN-FORCE

| Feature | Bitstamp | MEXC | HTX | Bitget | Gemini | BingX | Phemex | Crypto.com |
|---------|----------|------|-----|--------|--------|-------|--------|------------|
| GTC | Y (default) | Y (default) | Y (default `gtc`) | Y (`gtc`) | Y (default, no option specified) | Y | Y (`GoodTillCancel`) | Y (`GOOD_TILL_CANCEL`) |
| IOC | Y (`ioc_order` flag) | Y (via type=IOC spot; type=3 futures) | Y (`buy-ioc`/`sell-ioc` spot; `ioc` in algo-orders) | Y (`ioc` force param) | Y (`immediate-or-cancel` option) | Y | Y (`ImmediateOrCancel`) | Y (`IMMEDIATE_OR_CANCEL`) |
| FOK | Y (`fok_order` flag) | Y (via type=FOK spot; type=4 futures) | Y (`buy-limit-fok`/`sell-limit-fok` spot; `fok` in algo-orders) | Y (`fok` force param) | Y (`fill-or-kill` option) | Y | Y (`FillOrKill`) | Y (`FILL_OR_KILL`) |
| PostOnly | N | Y (type=LIMIT_MAKER spot; type=2 futures) | Y (`buy-limit-maker`/`sell-limit-maker`; `boc` in algo-orders) | Y (`post_only` force param) | Y (`maker-or-cancel` option) | Y | Y (`PostOnly`) | Y (`POST_ONLY` exec_inst) |
| GTD | Y (`gtd_order` flag + `expire_time`) | N | N | N | N | N | N | N |

---

## 3. ORDER MANAGEMENT

| Feature | Bitstamp | MEXC | HTX | Bitget | Gemini | BingX | Phemex | Crypto.com |
|---------|----------|------|-----|--------|--------|-------|--------|------------|
| Single Place | Y | Y | Y | Y | Y | Y | Y | Y |
| Batch Place (max size) | N | Y (spot: 20 same-symbol; futures: 50) | Y (10) | Y (50) | N | Y (size undocumented) | N | Y (10) |
| Cancel Single | Y | Y | Y | Y | Y | Y | Y | Y |
| Cancel All | Y | Y | Y (batchCancelOpenOrders, max 100) | Y | Y (all sessions) | Y (futures) | Y | Y |
| Cancel By Symbol | Y | Y (up to 5 symbols) | Y (symbol param on batchCancel) | Y (cancel-symbol-orders) | N | Y | Y | Y (instrument_name param) |
| Amend / Modify | P (replace_order = cancel+replace atomic) | N (cancel+resubmit only) | N (cancel+resubmit only) | Y (futures: modify-order; spot: N) | N | Y (futures: amend endpoint; spot: N) | Y (spot+futures: replace endpoint) | Y (private/amend-order, added 2025-06-10) |
| Get Single Order | Y | Y | Y | Y | Y | Y | Y | Y |
| Get Open Orders | Y | Y | Y | Y | Y | Y | Y | Y |
| Get Order History | Y (user_transactions) | Y (7-day max range) | Y (48h max, use /v1/order/history) | Y (90-day history) | Y (limit 500) | Y | Y | Y (6-month retention) |

---

## 4. POSITIONS (FUTURES)

| Feature | Bitstamp | MEXC | HTX | Bitget | Gemini | BingX | Phemex | Crypto.com |
|---------|----------|------|-----|--------|--------|-------|--------|------------|
| GetPositions | N (spot only) | Y | Y (Coin-M + USDT-M) | Y | P (PERP exists, endpoint undocumented) | Y | Y (COIN-M + USDM) | Y |
| ClosePosition | N | P (place close order) | P (place close order) | Y (flash close endpoint) | N | Y (closeAllPositions + single) | P (place close order or reduceOnly) | Y (private/close-position) |
| SetLeverage | N | Y | Y (Coin-M + USDT-M) | Y | N | Y | Y (Coin-M + USDM) | Y (account-level + isolated) |
| MarginMode (cross/isolated) | N | Y (openType param) | Y (isolated + cross-margin accounts) | Y (set-margin-mode) | N | Y (marginType endpoint) | P (leverage sign = 0 → cross, >0 → isolated) | P (isolated via isolation_id, no explicit switch) |
| AddRemoveMargin | N | N (not documented) | N | Y (set-margin endpoint) | N | Y | Y (assign endpoint) | Y (create-isolated-margin-transfer) |
| FundingRate | N | N (not private endpoint) | N (public only) | N | Y (public: /v1/fundingamount) | Y (public endpoint) | Y (public history endpoint) | Y (public/get-valuations) |
| LiqPrice | N | Y (in position response: liquidatePrice) | Y (in position response: liquidation_price) | Y (in position response: liquidationPrice) | N | N (client-side calc only) | Y (in position response: liquidationPriceEp) | N (derived from risk params, no dedicated field) |

---

## 5. ACCOUNT

| Feature | Bitstamp | MEXC | HTX | Bitget | Gemini | BingX | Phemex | Crypto.com |
|---------|----------|------|-----|--------|--------|-------|--------|------------|
| Balances | Y | Y (spot + futures separate) | Y (must query account-id first) | Y (spot + futures + margin) | Y | Y (spot + futures + unified view) | Y (wallets endpoint) | Y (unified balance) |
| Fees | Y (fees/trading endpoint) | Y (tradeFee per symbol) | Y (v2/reference/transact-fee-rate) | Y (trade-rate + VIP table) | Y (notionalvolume: bps) | Y (commissionRate endpoints) | Y (fee-rate public endpoint) | Y (get-fee-rate + get-instrument-fee-rate) |
| InternalTransfer | P (spot↔sub only) | Y (spot↔futures: capital/transfer) | Y (spot↔margin↔futures: multiple endpoints) | Y (wallet/transfer: 6 account types) | Y (account/transfer/{currency}) | Y (asset/transfer) | Y (assets/transfer) | Y (create-subaccount-transfer) |
| DepositAddr | P (per-coin endpoints: bitcoin_deposit_address etc.) | N (not documented) | N (not in retrieved docs) | N (not in retrieved docs) | Y (addresses/{network}) | Y (wallets/v1/capital/deposit/address) | Y (deposit-address endpoint) | Y (get-deposit-address) |
| Withdraw | Y (per-coin endpoints) | N (not documented) | N (not in retrieved docs) | N (not in retrieved docs) | Y (withdraw/{currency}) | Y (wallets/v1/capital/withdraw/apply) | Y (POST /withdraw) | Y (create-withdrawal) |
| Deposit+Withdraw History | P (withdrawal-requests endpoint; no deposit history) | N | N | N | Y (transfers endpoint for both) | P (deposit: uncertain; withdraw: uncertain) | Y (deposit-history + withdraw-history) | Y (get-deposit-history + get-withdrawal-history) |

---

## 6. SUB-ACCOUNTS

| Feature | Bitstamp | MEXC | HTX | Bitget | Gemini | BingX | Phemex | Crypto.com |
|---------|----------|------|-----|--------|--------|-------|--------|------------|
| Create | P (institutional only, API-limited) | P (max 30, docs limited) | N (not documented) | N (not documented) | Y (account/create endpoint) | Y (subAccount/v1/account/create) | N (not documented) | N (UI only) |
| List | N | N | N | N | Y (account/list, up to 500) | Y (subAccount/v1/account/list) | N | Y (via get-accounts) |
| Transfer | Y (transfer-to-main / transfer-from-main) | N | N | N | Y (account/transfer/{currency}) | Y (account/transfer/v1/subAccount/transferAsset) | Y (subUserTransfer + universalTransfer) | Y (create-subaccount-transfer) |

---

## 7. ADVANCED FEATURES

| Feature | Bitstamp | MEXC | HTX | Bitget | Gemini | BingX | Phemex | Crypto.com |
|---------|----------|------|-----|--------|--------|-------|--------|------------|
| TWAP | N | N | N | N | N | Y (swap/v1/twap/order) | N | N |
| Iceberg | N | N (field in response, creation undocumented) | N | N | N | N | Y (spot: displayQty param) | N |
| CopyTrading | N | N | N | N | N | Y (exists, endpoints undocumented) | N (read-only query only) | N |
| GridTrading | N | N | N | N | N | Y (exists, endpoints undocumented) | N | N |

---

## 8. AUTHENTICATION METHOD

| Exchange | Method | Notes |
|----------|--------|-------|
| Bitstamp | HMAC-SHA256 | API key + secret; all private calls are POST with `application/x-www-form-urlencoded`; auth via headers |
| MEXC | HMAC-SHA256 | API key in `X-MEXC-APIKEY` header; signature in query string (`signature` param); timestamp required |
| HTX | HMAC-SHA256 | API key, secret key, timestamp, access key in query params; signature covers canonical request |
| Bitget | HMAC-SHA256 | `ACCESS-KEY`, `ACCESS-SIGN`, `ACCESS-TIMESTAMP`, `ACCESS-PASSPHRASE` headers; passphrase set at key creation |
| Gemini | HMAC-SHA256 | Payload (JSON) base64-encoded as `X-GEMINI-PAYLOAD` header; signature in `X-GEMINI-SIGNATURE`; nonce required |
| BingX | HMAC-SHA256 | `X-BX-APIKEY` header; `signature` param in query/body; timestamp required (`recvWindow` optional) |
| Phemex | HMAC-SHA256 | `x-phemex-access-token`, `x-phemex-request-expiry`, `x-phemex-request-signature` headers; scaled integer prices |
| Crypto.com | HMAC-SHA256 | `api_key` + `sig` (HMAC of sorted params) in JSON request body; `nonce` (millisecond timestamp) required |

---

## 9. EXCHANGE OVERVIEW NOTES

### Bitstamp
- **Spot only** — no futures, no perpetuals, no margin
- Separate buy/sell endpoints (not a unified `side` param) — unusual design
- Trailing stop and stop-market DISCONTINUED May 2025
- `replace_order` = atomic cancel+replace (not true amend)
- Deposit/withdraw per-coin endpoints (not unified)
- Sub-accounts: institutional only

### MEXC
- Spot V3 (Binance-compatible) + Futures Contract V1 are **completely separate APIs** with separate auth
- Spot: no stop orders, no OCO, no amend; IOC/FOK via `type` field (not `timeInForce`)
- Futures: TP/SL inline on order; batch up to 50; hedge/one-way mode
- Position leverage requires positionId when position exists
- Sub-accounts: max 30, limited API support

### HTX (Huobi)
- Side baked into order type string (`buy-limit`, `sell-market`, etc.) — no separate `side` field on spot
- Separate algo-orders API for TP/SL on spot (`/v2/algo-orders`)
- Futures TP/SL are inline parameters on order placement
- Must query account-id list first — routing key for all orders
- Coin-M futures (`api.hbdm.com`) and USDT-M swaps (`linear-swap-api/v1`) are separate namespaces
- Dead man's switch: `cancel-all-after` endpoint

### Bitget
- V2 API — spot namespace `/api/v2/spot/` and futures namespace `/api/v2/mix/`
- Futures supports true amend (`modify-order`)
- Plan orders (trigger/TP/SL) are separate from regular orders
- Trailing stop via `planType: track_plan`
- `presetTakeProfitPrice` / `presetStopLossPrice` inline on regular orders
- Simultaneous TP+SL on position: `place-pos-tpsl-order`
- 6 account transfer types including isolated margin

### Gemini
- **Primarily spot** — perpetuals exist (PERP symbols) but derivatives endpoints poorly documented
- No dedicated market order type — use limit + `immediate-or-cancel`
- No batch orders, no amend, no cancel-by-symbol
- Unique auth: payload base64-encoded in header, not query string
- Sub-accounts: full support (create/list/transfer)
- Withdraw requires FundManager role (separate from Trader role)
- IP affirmation required for trading keys as of June 2025

### BingX
- Three market types: Spot, USDT-M Perpetual (swap/v2), Coin-M Inverse (cswap/v1)
- TWAP algo orders: confirmed (`swap/v1/twap/order`)
- Trailing stop: `TRAILING_STOP_MARKET` type
- Amend: swap v1 endpoint (`/openApi/swap/v1/trade/amend`)
- Sub-accounts: comprehensive API (create, list, transfer, API key management)
- TP+SL as embedded objects on single order call — bracket-like behavior
- All WebSocket messages are GZIP compressed

### Phemex
- Scaled integer prices — `priceEp` = price × scale_factor (varies by symbol)
- Iceberg on spot via `displayQty` parameter
- Bracket orders on futures: up to 5 related orders
- Trailing stop via `pegPriceType: TrailingStopPeg` + `pegOffsetValueEp`
- Leverage sign determines margin mode: `leverage=0` → cross; `>0` → isolated
- No batch order placement; bulk cancellation by comma-separated IDs
- Sub-account transfer endpoints exist; create/list not documented publicly
- Unified Trading Account (UTA) available with risk mode switching

### Crypto.com
- **Unified balance model** — no separate spot/futures wallet; single account namespace
- Stop/TP/SL orders migrated to `private/advanced/` API (as of Jan 2026)
- Full OCO, OTO, OTOCO (bracket) via Advanced Order Management API
- `private/amend-order` added June 2025 (reduces queue priority except size-down)
- Sub-account creation via UI only (not API)
- All numeric values in requests must be strings (not numbers)
- Response to order placement is async — only returns order_id, not fill info
- Max 200 open orders per pair; 1000 total per account

---

## 10. QUICK REFERENCE MATRIX

### Spot Capability Summary

| Capability | Bitstamp | MEXC Spot | HTX Spot | Bitget Spot | Gemini | BingX Spot | Phemex Spot | Crypto.com Spot |
|------------|----------|-----------|----------|-------------|--------|------------|-------------|-----------------|
| Market | Y | Y | Y | Y | Y* | Y | Y | Y |
| Limit | Y | Y | Y | Y | Y | Y | Y | Y |
| StopLimit | Y | N | Y | N | Y | N | Y | N** |
| IOC | Y | Y | Y | Y | Y | Y | Y | Y |
| FOK | Y | Y | Y | Y | Y | Y | Y | Y |
| PostOnly | N | Y | Y | Y | Y | Y | Y | Y |
| Batch | N | Y (20) | Y (10) | Y (50) | N | Y | N | Y (10) |
| Amend | P | N | N | N | N | N | Y | Y |
| TP/SL | N | N | N | Y (preset) | N | N | Y | N** |

*Market via IOC+limit combo
**Moved to Advanced API

### Futures Capability Summary

| Capability | Bitstamp | MEXC Futures | HTX Futures | Bitget Futures | Gemini PERP | BingX Futures | Phemex Futures | Crypto.com PERP |
|------------|----------|--------------|-------------|----------------|-------------|---------------|----------------|-----------------|
| Exists | N | Y | Y (Coin-M + USDT-M) | Y | P | Y | Y (Coin-M + USDM) | Y |
| StopMarket | - | Y | Y | Y | - | Y | Y | N** |
| StopLimit | - | Y | Y | Y | - | Y | Y | N** |
| TrailingStop | - | N | Y | Y | - | Y | Y | N |
| TP/SL inline | - | Y | Y | Y | - | Y | Y | N** |
| OCO | - | N | N | N | - | N | N | Y |
| Bracket | - | N | N | Y | - | P | Y | Y |
| Batch | - | Y (50) | Y (10) | Y (50) | - | Y | N | Y (10) |
| Amend | - | N | N | Y | - | Y | Y | Y |
| SetLeverage | - | Y | Y | Y | - | Y | Y | Y |
| MarginMode | - | Y | Y | Y | - | Y | P | P |
| AddMargin | - | N | N | Y | - | Y | Y | Y |
| FundingRate | - | N | N | N | Y | Y | Y | Y |
| LiqPrice | - | Y | Y | Y | N | N | Y | N |

**Moved to Advanced API (`private/advanced/create-order`)
