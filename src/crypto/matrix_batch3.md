# Trading API Capability Matrix — Batch 3 (8 Exchanges)

Generated: 2026-03-11
Sources: trading_api.md + account_api.md per exchange

Legend: Y = supported, N = not supported, Partial = partially supported / limited, N/A = not applicable

---

## Exchange Overview

| Exchange | Type | Markets | Auth Method |
|----------|------|---------|-------------|
| Upbit | CEX | Spot only | Bearer JWT (HMAC-SHA256 signed payload) |
| Deribit | CEX | Spot, Futures, Options, Perpetuals | JSON-RPC 2.0; OAuth client_credentials (client_id + client_secret → access_token) |
| HyperLiquid | CEX (L1 chain) | Perps + Spot | ECDSA signature (Ethereum private key) per action |
| Lighter | DEX (ZK-rollup, Ethereum L1) | Perps + Spot | API private key; signed ZK transactions via `sendTx` |
| Jupiter | DEX (Solana) | Spot swaps + Trigger/DCA; Perps on-chain only | Solana wallet private key signs transactions; x-api-key header for REST |
| GMX | DEX (Arbitrum/Avalanche) | Perps + Spot swaps | On-chain EVM signing (no REST auth); REST API read-only, no auth required |
| Paradex | DEX (Starknet L2) | Perps only | STARK elliptic curve signature per order; Bearer JWT for REST read endpoints |
| dYdX V4 | DEX (Cosmos chain) | Perps only | Cosmos wallet signing (MsgPlaceOrder via gRPC); Indexer REST read-only, no auth |

---

## 1. Upbit (CEX — Spot Only)

**Auth:** Bearer JWT. Header: `Authorization: Bearer {jwt_token}`. JWT payload is HMAC-SHA256 signed with API secret. Permissions are scoped per key: View Account, View Orders, Make Orders, Withdraw.

### Order Types

| Feature | Supported | Notes |
|---------|-----------|-------|
| Market | Y | Asymmetric: `price` = market buy (quote amount), `market` = market sell (base qty) |
| Limit | Y | Standard limit order |
| StopMarket | N | Not available |
| StopLimit | N | Not available |
| TrailingStop | N | Not available |
| TP | N | Not available |
| SL | N | Not available |
| OCO | N | Not available |
| Bracket | N | Not available |
| Best-price order | Y | `ord_type=best`; executes at best available price; requires IOC or FOK |

### Time-in-Force

| TIF | Supported | Notes |
|-----|-----------|-------|
| GTC | Partial | Implicit default for limit orders (no explicit GTC parameter) |
| IOC | Y | `time_in_force=ioc` |
| FOK | Y | `time_in_force=fok` |
| PostOnly | Y | `time_in_force=post_only` |
| GTD | N | Not available |

### Order Management

| Feature | Supported | Notes |
|---------|-----------|-------|
| Single order | Y | `POST /orders` |
| Batch placement | N | Not supported |
| Cancel single | Y | `DELETE /order` by uuid or identifier |
| Cancel all | Y | `DELETE /orders/open`; up to 300 orders; 1 req/2sec rate limit |
| Cancel by symbol | Y | Cancel all with `market` filter via `DELETE /orders/open?market=...` |
| Amend/Edit | Partial | No true amend; `POST /orders/cancel_and_new` atomically cancels + replaces |
| Get single order | Y | `GET /order` by uuid or identifier |
| Get open orders | Y | `GET /orders/open` with pagination |
| Get history | Y | `GET /orders/closed` (done/cancelled) |
| Get by multiple IDs | Y | `GET /orders/uuids` batch lookup |

### Positions

| Feature | Supported | Notes |
|---------|-----------|-------|
| GetPositions | N/A | Spot only — no positions concept |
| Close position | N/A | Spot only |
| SetLeverage | N/A | Spot only |
| MarginMode | N/A | Spot only |
| AddRemoveMargin | N/A | Spot only |
| FundingRate | N/A | Spot only |
| LiqPrice | N/A | Spot only |

### Account

| Feature | Supported | Notes |
|---------|-----------|-------|
| Balances | Y | `GET /accounts` — all assets, free + locked |
| Fees | Partial | Embedded in order responses and `GET /orders/info`; no standalone fee schedule endpoint |
| InternalTransfer | N | Single account type; no spot/futures split |
| DepositAddr | Y | `GET /deposits/coin_address` + `POST /deposits/generate_coin_address` |
| Withdraw | Y | `POST /withdraws/coin` (address must be pre-whitelisted) |
| Deposit/Withdraw History | Y | `GET /deposits`, `GET /withdraws` with filters |

### Sub-accounts

| Feature | Supported |
|---------|-----------|
| Create | N |
| List | N |
| Transfer | N |

### Advanced

| Feature | Supported |
|---------|-----------|
| TWAP | N |
| Iceberg | N |
| CopyTrading | N |
| GridTrading | N |
| SMP (Self-Match Prevention) | Y |

---

## 2. Deribit (CEX — Options + Futures + Perpetuals + Spot)

**Auth:** JSON-RPC 2.0 over WebSocket or HTTP. Uses OAuth: `public/auth` with `client_credentials` grant returns `access_token`. All private methods require `access_token` as Bearer or in JSON-RPC auth field. Scopes: `trade:read`, `trade:read_write`, `wallet:read`, `wallet:read_write`, `account:read`, `account:read_write`, `block_trade:read_write`, etc.

### Order Types

| Feature | Supported | Notes |
|---------|-----------|-------|
| Market | Y | `type=market` |
| Limit | Y | `type=limit` (default) |
| StopMarket | Y | `type=stop_market` |
| StopLimit | Y | `type=stop_limit` |
| TrailingStop | Y | `type=trailing_stop`; `trigger_offset` for max deviation |
| TP (Take Profit Market) | Y | `type=take_market` |
| TP (Take Profit Limit) | Y | `type=take_limit` |
| SL | Y | Via `stop_market` / `stop_limit` types |
| OCO | Y | `linked_order_type=one_cancels_other` with `otoco_config` |
| Bracket (OTOCO) | Y | `linked_order_type=one_triggers_one_cancels_other` |
| OTO | Y | `linked_order_type=one_triggers_other` |
| Market-Limit | Y | `type=market_limit`; converts to limit at best bid/ask if not immediately filled |

### Time-in-Force

| TIF | Supported | Notes |
|-----|-----------|-------|
| GTC | Y | `time_in_force=good_til_cancelled` (default) |
| IOC | Y | `time_in_force=immediate_or_cancel` |
| FOK | Y | `time_in_force=fill_or_kill` |
| PostOnly | Y | `post_only=true` boolean param (GTC only) |
| GTD | Y | `time_in_force=good_til_day` |

### Order Management

| Feature | Supported | Notes |
|---------|-----------|-------|
| Single order | Y | `private/buy` or `private/sell` |
| Batch placement | N | No batch endpoint for standard orders; `private/mass_quote` for market maker options quoting only |
| Cancel single | Y | `private/cancel` by order_id |
| Cancel all | Y | `private/cancel_all` (all instruments); `private/cancel_all_by_currency`; `private/cancel_all_by_instrument` |
| Cancel by symbol | Y | `private/cancel_all_by_instrument` |
| Cancel by label | Y | `private/cancel_by_label` |
| Amend/Edit | Y | `private/edit` — modify price, size, trigger, display_amount |
| Get single order | Y | `private/get_order_state` |
| Get open orders | Y | `private/get_open_orders` (by kind, type); `private/get_open_orders_by_instrument` |
| Get history | Y | `private/get_order_history_by_instrument`; `private/get_order_history_by_currency` |

### Positions

| Feature | Supported | Notes |
|---------|-----------|-------|
| GetPositions | Y | `private/get_positions` (by currency + kind); `private/get_position` (single) |
| Close | Y | `private/close_position` (limit or market) |
| SetLeverage | N | No explicit endpoint; leverage is implicit via margin allocation |
| MarginMode | Partial | Portfolio margin is account-level (not per-instrument API call) |
| AddRemoveMargin | N | No dedicated per-position margin add/remove endpoint |
| FundingRate | Y | `public/get_funding_rate_value`; `public/get_funding_rate_history` |
| LiqPrice | Y | Returned in position response as `estimated_liquidation_price` |

### Account

| Feature | Supported | Notes |
|---------|-----------|-------|
| Balances | Y | `private/get_account_summary` (single currency); `private/get_account_summaries` (all) |
| Fees | Y | `fee_group` and `fees` in account summary (extended mode) |
| InternalTransfer | Y | `private/submit_transfer_to_subaccount`; `private/submit_transfer_between_subaccounts`; `private/submit_transfer_to_user` |
| DepositAddr | Y | `private/get_current_deposit_address`; `private/create_deposit_address` |
| Withdraw | Y | `private/withdraw` (requires 2FA) |
| Deposit/Withdraw History | Y | `private/get_deposits`; `private/get_withdrawals`; `private/get_transfers` |

### Sub-accounts

| Feature | Supported | Notes |
|---------|-----------|-------|
| Create | Y | `private/create_subaccount` |
| List | Y | `private/get_subaccounts`; `private/get_subaccounts_details` |
| Transfer | Y | `private/submit_transfer_to_subaccount`; `private/submit_transfer_between_subaccounts` |

### Advanced

| Feature | Supported | Notes |
|---------|-----------|-------|
| TWAP | N | No public TWAP algorithm |
| Iceberg | Y | `display_amount` param on buy/sell/edit; must be >= 100x instrument minimum |
| CopyTrading | N | Not documented |
| GridTrading | N | Not documented |
| BlockTrades | Y | `private/verify_block_trade`, `private/execute_block_trade` |
| BlockRFQ | Y | `private/create_block_rfq`, `private/accept_block_rfq` |
| MassQuote | Y | `private/mass_quote` for market makers |

---

## 3. HyperLiquid (CEX/L1 — Perpetuals + Spot)

**Auth:** ECDSA signature (Ethereum private key) on every action. Signature covers action hash + nonce. Two signing schemes: `sign_l1_action` (trading) and `sign_user_signed_action` (withdrawals/transfers). Info endpoint (`/info`) is fully public — no auth required for reads.

### Order Types

| Feature | Supported | Notes |
|---------|-----------|-------|
| Market | Y | `type: {limit: {tif: "Ioc"}}` at aggressive price, or trigger with `isMarket: true` |
| Limit | Y | `type: {limit: {tif: "Gtc|Ioc|Alo"}}` |
| StopMarket | Y | Trigger order with `isMarket: true`, `tpsl: "sl"` |
| StopLimit | Y | Trigger order with `isMarket: false`, `tpsl: "sl"` |
| TrailingStop | N | Not documented |
| TP | Y | Trigger with `tpsl: "tp"`, `isMarket: true/false` |
| SL | Y | Trigger with `tpsl: "sl"`, `isMarket: true/false` |
| OCO | N | Not explicitly documented; TP+SL grouping via `grouping: "normalTpsl"/"positionTpsl"` |
| Bracket | Partial | TP/SL grouping supports bracket-style via `grouping: "positionTpsl"` |

### Time-in-Force

| TIF | Supported | Notes |
|-----|-----------|-------|
| GTC | Y | `tif: "Gtc"` |
| IOC | Y | `tif: "Ioc"` |
| FOK | N | Not documented |
| PostOnly (ALO) | Y | `tif: "Alo"` (Add Liquidity Only) |
| GTD | N | Not documented |

### Order Management

| Feature | Supported | Notes |
|---------|-----------|-------|
| Single order | Y | `action: {type: "order", orders: [single_order]}` |
| Batch placement | Y | `orders` array; no documented max; weight scales as `1 + floor(n/40)` |
| Cancel single | Y | `action: {type: "cancel", cancels: [{a, o}]}` |
| Cancel by cloid | Y | `action: {type: "cancelByCloid"}` |
| Cancel all | Partial | `scheduleCancel` (dead man's switch) cancels all at a future timestamp; no instant cancel-all documented |
| Cancel by symbol | N | No symbol-level cancel-all; batch cancel array required |
| Amend single | Y | `action: {type: "modify"}` |
| Batch amend | Y | `action: {type: "batchModify"}` |
| Get single order | Y | `{type: "orderStatus", oid: ...}` via `/info` |
| Get open orders | Y | `{type: "openOrders"}` or `{type: "frontendOpenOrders"}` |
| Get history | Y | `{type: "historicalOrders"}` (up to 2000); `{type: "userFills"}` |

### Positions

| Feature | Supported | Notes |
|---------|-----------|-------|
| GetPositions | Y | `{type: "clearinghouseState"}` via `/info` |
| Close | Partial | No dedicated close endpoint; place opposing reduce-only order |
| SetLeverage | Y | `action: {type: "updateLeverage", isCross, leverage}` |
| MarginMode | Y | `isCross` in `updateLeverage` switches cross/isolated |
| AddRemoveMargin | Y | `action: {type: "updateIsolatedMargin", ntli}` or `topUpIsolatedOnlyMargin` |
| FundingRate | Y | `{type: "fundingHistory"}`, `{type: "predictedFundings"}`, `{type: "userFunding"}` |
| LiqPrice | Y | In `clearinghouseState` response as `liquidationPx` (null for cross) |

### Account

| Feature | Supported | Notes |
|---------|-----------|-------|
| Balances | Y | Perp: `clearinghouseState.withdrawable` + `marginSummary`; Spot: `spotClearinghouseState` |
| Fees | Y | `{type: "userFees"}` — taker/maker rates, daily volume, staking discount |
| InternalTransfer | Y | `usdSend` (L1 internal), `usdClassTransfer` (spot↔perp), `spotSend`, `sendAsset` |
| DepositAddr | N/A | Deposits via Arbitrum bridge on-chain; no API deposit address |
| Withdraw | Y | `action: {type: "withdraw3"}` to Arbitrum; ~5 min, $1 fee |
| Deposit/Withdraw History | Partial | No dedicated REST history endpoint documented; observable via on-chain |

### Sub-accounts

| Feature | Supported | Notes |
|---------|-----------|-------|
| Create | Partial | Sub-accounts created via on-chain interaction; `vaultAddress` param for trading on behalf of sub-account |
| List | Y | `{type: "subAccounts", user: "0x..."}` |
| Transfer | Y | Via `vaultAddress` in action envelope + `usdSend`/`sendAsset` |

### Advanced

| Feature | Supported | Notes |
|---------|-----------|-------|
| TWAP | Y | `action: {type: "twapOrder"}`; duration in minutes; optional randomization |
| Iceberg | N | Not documented |
| CopyTrading | Partial | Vault system allows following vault leaders |
| GridTrading | N | Not documented |
| DeadMansSwitch | Y | `action: {type: "scheduleCancel"}` |

---

## 4. Lighter (DEX — ZK-rollup, Ethereum L1 — Perpetuals + Spot)

**Auth:** API private key stored off-chain. All write operations are ZK-signed transactions submitted via `POST /api/v1/sendTx` or batch via `POST /api/v1/sendTxBatch`. Each transaction type has a numeric `tx_type` constant. Read endpoints (GET) use `auth` token (API key). The signing is done by the `SignerClient` SDK — produces STARK-like ZK proof signatures.

### Order Types

| Feature | Supported | Notes |
|---------|-----------|-------|
| Market | Y | `ORDER_TYPE_MARKET` with `ORDER_TIME_IN_FORCE_IMMEDIATE_OR_CANCEL` |
| Limit | Y | `ORDER_TYPE_LIMIT` |
| StopMarket | Y | `ORDER_TYPE_STOP_LOSS` (triggered when markPrice <= triggerPrice) |
| StopLimit | Y | `ORDER_TYPE_STOP_LOSS_LIMIT` |
| TrailingStop | N | Not documented |
| TP Market | Y | `ORDER_TYPE_TAKE_PROFIT` (triggered when markPrice >= triggerPrice) |
| TP Limit | Y | `ORDER_TYPE_TAKE_PROFIT_LIMIT` |
| SL | Y | `ORDER_TYPE_STOP_LOSS` / `ORDER_TYPE_STOP_LOSS_LIMIT` |
| OCO | Partial | `to_cancel_order_id_0` field in order allows linking; `L2CreateGroupedOrders` tx_type=28 |
| Bracket | Partial | Grouped orders via `tx_type=28`; `to_trigger_order_id_0/1` links TP+SL to parent |
| TWAP | Y | `ORDER_TYPE_TWAP` — internal sub-orders generated automatically |

### Time-in-Force

| TIF | Supported | Notes |
|-----|-----------|-------|
| GTC (GTT) | Y | `ORDER_TIME_IN_FORCE_GOOD_TILL_TIME` with `order_expiry` unix timestamp |
| IOC | Y | `ORDER_TIME_IN_FORCE_IMMEDIATE_OR_CANCEL` |
| FOK | N | Not documented |
| PostOnly | Y | `ORDER_TIME_IN_FORCE_POST_ONLY` |
| GTD | Partial | GTT is effectively GTD via `order_expiry` timestamp |

### Order Management

| Feature | Supported | Notes |
|---------|-----------|-------|
| Single order | Y | `POST /api/v1/sendTx` with `L2CreateOrder` tx_type=14 |
| Batch placement | Y | `POST /api/v1/sendTxBatch`; up to 50 transactions per batch |
| Cancel single | Y | `create_cancel_order(order_index=client_order_index)` via sendTx |
| Cancel all | Y | `L2CancelAllOrders` tx_type=5; does not consume Volume Quota |
| Cancel by symbol | N | No per-market cancel-all; use cancel-all (all markets) |
| Amend/Modify | Y | `L2ModifyOrder` tx_type=17 (consumes 1 Volume Quota) |
| Get single order | Partial | Via active/inactive order lists filtered by client_order_index |
| Get open orders | Y | `GET /api/v1/accountActiveOrders` (by account_index + market_id) |
| Get history | Y | `GET /api/v1/accountInactiveOrders` with cursor pagination (limit 1-100) |

### Positions

| Feature | Supported | Notes |
|---------|-----------|-------|
| GetPositions | Y | Embedded in `GET /api/v1/account` response as `positions` array |
| Close | Partial | No dedicated close; place opposing market/limit with `reduce_only=true` |
| SetLeverage | Y | `L2UpdateLeverage` signed tx; rate limit 40/min |
| MarginMode | Partial | `margin_mode` param in leverage tx (standard vs premium tier) |
| AddRemoveMargin | N | No dedicated margin add/remove endpoint |
| FundingRate | Y | `GET /api/v1/funding-rates`; `GET /api/v1/fundings`; `GET /api/v1/positionFunding` |
| LiqPrice | N | Not directly returned; liquidation events in `GET /api/v1/liquidations` |

### Account

| Feature | Supported | Notes |
|---------|-----------|-------|
| Balances | Y | `GET /api/v1/account` — `available_balance`, `collateral`, `total_asset_value`, `assets` array |
| Fees | Y | `accountLimits` returns `current_maker_fee_tick`, `current_taker_fee_tick`, `user_tier` |
| InternalTransfer | Y | `L2Transfer` tx_type; rate limit 120/min |
| DepositAddr | Y | `GET /api/v1/deposit/networks`; `POST /api/v1/createIntentAddress` (fast bridge) |
| Withdraw | Y | Secure withdrawal via sendTx (withdraw tx_type); fast via `POST /api/v1/fastwithdraw`; rate limit 2/min |
| Deposit/Withdraw History | Y | `GET /api/v1/deposit/history`; `GET /api/v1/withdraw/history`; `GET /api/v1/transfer/history` |

### Sub-accounts

| Feature | Supported | Notes |
|---------|-----------|-------|
| Create | Y | Ethereum wallet registers main account; sub-accounts registered via `L2ChangePubKey` (rate 300/min) |
| List | Y | `GET /api/v1/accountsByL1Address` returns all sub-accounts under an L1 address |
| Transfer | Y | `L2Transfer` tx between account indices |

### Advanced

| Feature | Supported | Notes |
|---------|-----------|-------|
| TWAP | Y | `ORDER_TYPE_TWAP`; automatically generates internal sub-orders |
| Iceberg | N | Not documented |
| CopyTrading | N | Not documented |
| GridTrading | N | Not documented |
| LiquidityPools | Y | `can_create_public_pool` flag in accountLimits; pool management endpoints |

---

## 5. Jupiter (DEX — Solana — Spot Swaps + Trigger Orders + DCA + Perps on-chain)

**Auth:** Solana wallet private key signs all transactions. REST endpoints require `x-api-key` header for `api.jup.ag`. Perps are purely on-chain Anchor program interactions — no REST API for perps (work in progress as of early 2026). Account data read via Jupiter REST holdings endpoint or directly via Solana RPC.

### Order Types

| Feature | Supported | Notes |
|---------|-----------|-------|
| Market (swap) | Y | Ultra Swap API or Metis Swap API; instant execution via Solana DEX routing |
| Limit (Trigger) | Y | Trigger API: `POST /trigger/v1/createOrder`; executes when `takingAmount/makingAmount` ratio is met |
| StopMarket | N | Not available via REST |
| StopLimit | N | Not available via REST |
| TrailingStop | N | Not available |
| TP | Partial | Perps on-chain: `requestType=Trigger, triggerAboveThreshold` |
| SL | Partial | Perps on-chain: `requestType=Trigger, triggerAboveThreshold=false` |
| OCO | N | Not available |
| Bracket | N | Not available via REST |
| DCA / Recurring | Y | Recurring API: fixed interval swaps (time-based) |
| Perp Market/TP/SL | Partial | On-chain Anchor program only; REST API not yet available |

### Time-in-Force

| TIF | Supported | Notes |
|-----|-----------|-------|
| GTC | Partial | Trigger orders remain on-chain until filled or explicitly cancelled |
| IOC | N | Swap orders are instant (no resting); no IOC flag |
| FOK | N | Not documented |
| PostOnly | N | Not applicable for DEX swaps |
| GTD | Y | Trigger API: `expiredAt` unix timestamp for order expiration |

### Order Management

| Feature | Supported | Notes |
|---------|-----------|-------|
| Single order | Y | Ultra API: `GET /order` + `POST /execute`; Trigger API: `POST /createOrder` + `POST /execute` |
| Batch placement | N | No batch API |
| Cancel single | Y | `POST /trigger/v1/cancelOrder` |
| Cancel all | Y | `POST /trigger/v1/cancelOrders` with `orders` array omitted = cancel all for maker |
| Cancel by symbol | Y | Filter by `inputMint` + `outputMint` in cancel endpoint |
| Amend | N | No order modification; cancel and recreate |
| Get open orders | Y | `GET /trigger/v1/getTriggerOrders?orderStatus=active` |
| Get history | Y | `GET /trigger/v1/getTriggerOrders?orderStatus=history`; recurring via `getRecurringOrders` |

### Positions

| Feature | Supported | Notes |
|---------|-----------|-------|
| GetPositions | Partial | Perps only; on-chain Anchor deserialization required; no REST endpoint |
| Close | Partial | On-chain: `DecreasePosition` instruction |
| SetLeverage | N/A | Perps leverage = sizeUsdDelta / collateralValue (implicit, up to 100x) |
| MarginMode | N/A | No explicit margin mode control |
| AddRemoveMargin | N/A | Collateral managed via position increase/decrease |
| FundingRate | N/A | Perps use JLP pool-based funding; no REST funding rate endpoint |
| LiqPrice | N/A | Calculable on-chain from position data |

### Account

| Feature | Supported | Notes |
|---------|-----------|-------|
| Balances | Y | `GET /ultra/v1/holdings/{address}` — SOL + all SPL tokens; or Solana RPC |
| Fees | Partial | Swap fees visible in quote response (`routePlan`); DCA fee 0.1%/execution |
| InternalTransfer | N/A | Self-custodial; transfers are standard Solana token transfers |
| DepositAddr | N/A | Non-custodial; wallet address is deposit address |
| Withdraw | N/A | Non-custodial; direct on-chain transfer |
| Deposit/Withdraw History | N/A | On-chain transaction history only |

### Sub-accounts

| Feature | Supported | Notes |
|---------|-----------|-------|
| Create | N/A | Non-custodial; no sub-accounts concept |
| List | N/A | |
| Transfer | N/A | |

### Advanced

| Feature | Supported | Notes |
|---------|-----------|-------|
| TWAP | N | No TWAP API |
| Iceberg | N | Not available |
| DCA/Recurring | Y | Recurring API with interval, count, min/max price filters |
| CopyTrading | N | Not documented |
| GridTrading | N | Not documented |

---

## 6. GMX V2 (DEX — Arbitrum/Avalanche — Perpetuals + Spot Swaps)

**Auth:** On-chain EVM signing only. All trading actions are direct smart contract calls (`ExchangeRouter`). The REST API (`gmxinfra.io`) is read-only and requires NO authentication. No API keys exist for trading — wallets sign transactions directly.

### Order Types

| Feature | Supported | Notes |
|---------|-----------|-------|
| Market (swap) | Y | `orderType=0 MarketSwap` |
| Market (position) | Y | `orderType=2 MarketIncrease`; `orderType=4 MarketDecrease` |
| Limit (swap) | Y | `orderType=1 LimitSwap` |
| Limit (position) | Y | `orderType=3 LimitIncrease` |
| StopMarket | Partial | `orderType=6 StopLossDecrease`; v2.1+: `StopIncrease` |
| StopLimit | N | No separate stop-limit; limit decrease is effectively a limit TP |
| TrailingStop | N | Not available |
| TP | Y | `orderType=5 LimitDecrease` (take profit for position) |
| SL | Y | `orderType=6 StopLossDecrease` |
| OCO | N | Not supported as atomic type; multiple orders can coexist |
| Bracket | Partial | Place position + SL + TP in multicall; not a single bracket order type |

### Time-in-Force

| TIF | Supported | Notes |
|-----|-----------|-------|
| GTC | Y | All on-chain orders rest until filled or cancelled (implicit GTC) |
| IOC | N | No IOC; market orders execute via keeper two-step |
| FOK | N | Not available |
| PostOnly | N/A | On-chain; no maker/taker distinction in this sense |
| GTD | Partial | `validFromTime` for earliest execution; no expiry timestamp |
| Auto-cancel | Y | `autoCancel=true` on LimitDecrease/StopLossDecrease auto-cancels when position closed |

### Order Management

| Feature | Supported | Notes |
|---------|-----------|-------|
| Single order | Y | `ExchangeRouter.createOrder(params)` on-chain |
| Batch placement | Y | `ExchangeRouter.multicall([...])` bundles multiple createOrder calls |
| Cancel single | Y | `ExchangeRouter.cancelOrder(key)` |
| Cancel all | N | No cancel-all function; must cancel each order key individually |
| Cancel by symbol | N | No symbol-level batch cancel |
| Amend/Update | Y | `ExchangeRouter.updateOrder(key, ...)` — change size, price, triggerPrice |
| Get single order | Y | `Reader.getOrder(dataStore, key)` on-chain view |
| Get open orders | Y | `Reader.getAccountOrders(dataStore, account, start, end)` |
| Get history | Y | Subsquid GraphQL: `tradeActions` entity |

### Positions

| Feature | Supported | Notes |
|---------|-----------|-------|
| GetPositions | Y | `Reader.getAccountPositions(dataStore, account, start, end)` on-chain |
| Close | Y | `orderType=4 MarketDecrease` with full position `sizeDeltaUsd` |
| SetLeverage | N | No explicit leverage endpoint; leverage = sizeDeltaUsd / collateralValue (implicit, up to 100x) |
| MarginMode | N | No isolated/cross toggle; collateral committed per-position |
| AddRemoveMargin | Y | Add: `MarketIncrease` with `sizeDeltaUsd=0`; Remove: `MarketDecrease` with `sizeDeltaUsd=0` |
| FundingRate | Y | `GET /markets/info` returns `fundingRateLong/Short`; `Reader.getMarketInfo()` on-chain |
| LiqPrice | Partial | Calculable from position data + maintenance margin fraction; not returned directly |

### Account

| Feature | Supported | Notes |
|---------|-----------|-------|
| Balances | Partial | ERC-20 `balanceOf()` on-chain; collateral is inside positions as `collateralAmount` |
| Fees | Y | `GET /markets/info` REST: fee structures; on-chain fee calculation |
| InternalTransfer | N/A | No custodial wallet; collateral goes directly into positions |
| DepositAddr | N/A | Non-custodial; own wallet address |
| Withdraw | N/A | Collateral released on position close/reduce |
| Deposit/Withdraw History | Partial | Subsquid GraphQL for trade/position history |

### Sub-accounts

| Feature | Supported | Notes |
|---------|-----------|-------|
| Create | N/A | Non-custodial; no sub-accounts |
| List | N/A | |
| Transfer | N/A | |

### Advanced

| Feature | Supported | Notes |
|---------|-----------|-------|
| TWAP | N | Not available |
| Iceberg | N | Not available |
| CopyTrading | N | Not documented |
| GridTrading | N | Not documented |
| GMPool (LP) | Y | `createDeposit()` / `createWithdrawal()` for liquidity provision |

---

## 7. Paradex (DEX — Starknet L2 — Perpetuals Only)

**Auth:** STARK elliptic curve signature on every order (fields: `signature = [r,s]`, `signature_timestamp`). JWT Bearer token required for all REST read endpoints (obtained via onboarding flow that signs a message with STARK private key). Subkeys (derived keypairs) supported for safer API access without withdrawal rights.

### Order Types

| Feature | Supported | Notes |
|---------|-----------|-------|
| Market | Y | `type=MARKET`; use `price="0"` |
| Limit | Y | `type=LIMIT` |
| StopMarket | Y | `type=STOP_MARKET` with `trigger_price` |
| StopLimit | Y | `type=STOP_LIMIT` with `trigger_price` |
| TrailingStop | N | Not documented |
| TP Market | Y | `type=TAKE_PROFIT_MARKET` |
| TP Limit | Y | `type=TAKE_PROFIT_LIMIT` |
| SL Market | Y | `type=STOP_LOSS_MARKET` |
| SL Limit | Y | `type=STOP_LOSS_LIMIT` |
| OCO | N | Not documented as standalone type |
| Bracket | N | Not documented as standalone type |
| TWAP | Y | Via `/v1/algo/orders` with `algo_type="TWAP"`; sub-orders every 30s |
| Scaled Order | Partial | UI feature; not confirmed as REST API type |

### Time-in-Force

| TIF | Supported | Notes |
|-----|-----------|-------|
| GTC | Y | `instruction=GTC` (default) |
| IOC | Y | `instruction=IOC` |
| FOK | N | Not documented |
| PostOnly | Y | `instruction=POST_ONLY` |
| GTD | N | No GTD; but `TWAP` orders have `duration_seconds` |
| RPI | Y | `instruction=RPI` (Retail Price Improvement — special fill type) |

### Order Management

| Feature | Supported | Notes |
|---------|-----------|-------|
| Single order | Y | `POST /v1/orders` |
| Batch placement | Y | `POST /v1/orders/batch`; 1–10 orders per request; 1 rate limit unit per batch |
| Cancel single | Y | `DELETE /v1/orders/{order_id}` |
| Cancel batch | Y | `DELETE /v1/orders/batch` by order_ids or client_order_ids |
| Cancel all | Y | `DELETE /v1/orders`; optional `market` filter for faster targeted cancel |
| Cancel by symbol | Y | `DELETE /v1/orders?market={market}` |
| Amend/Modify | Y | `PUT /v1/orders/{order_id}` (confirmed via Python SDK; requires re-signing) |
| Get single order | Y | `GET /v1/orders/{order_id}` or `GET /v1/orders/by_client_id/{client_id}` |
| Get open orders | Y | `GET /v1/orders-history?status=OPEN` |
| Get history | Y | `GET /v1/orders-history` with time filters + pagination cursor |
| Get algo history | Y | `GET /v1/algo/orders-history` |

### Positions

| Feature | Supported | Notes |
|---------|-----------|-------|
| GetPositions | Y | `GET /v1/positions` — full position data including LiqPrice, leverage, PnL |
| Close | Partial | No dedicated endpoint; place opposing MARKET/LIMIT with `REDUCE_ONLY` flag |
| SetLeverage | Partial | `GET /v1/account/margin` returns leverage config; no confirmed PUT endpoint for changing |
| MarginMode | Y | Response shows `margin_type: CROSS | ISOLATED`; `margin_methodology: cross_margin | portfolio_margin` |
| AddRemoveMargin | Partial | `on_behalf_of_account` for isolated margin sub-accounts |
| FundingRate | Y | `GET /v1/funding/payments` — account funding history with timestamps |
| LiqPrice | Y | Returned in position response as `liquidation_price` |

### Account

| Feature | Supported | Notes |
|---------|-----------|-------|
| Balances | Y | `GET /v1/account` — `account_value`, `free_collateral`, `total_collateral`, margin requirements; `GET /v1/balances` (confirmed via SDK) |
| Fees | Partial | `GET /v1/account/history?type=fee_savings`; fee tier not directly queryable |
| InternalTransfer | N | No direct transfer endpoint; isolated margin via `on_behalf_of_account` |
| DepositAddr | N/A | Starknet bridge-based; no deposit address API |
| Withdraw | N/A | Withdrawal via Starknet bridge contracts or UI; no direct REST withdraw endpoint |
| Deposit/Withdraw History | Y | `GET /v1/transfers` — full deposit/withdrawal/vault history with bridge info |

### Sub-accounts

| Feature | Supported | Notes |
|---------|-----------|-------|
| Create | Partial | Isolated margin accounts via `on_behalf_of_account` parameter; no explicit create endpoint documented |
| List | N | Not documented |
| Transfer | N | Not documented |

### Advanced

| Feature | Supported | Notes |
|---------|-----------|-------|
| TWAP | Y | Native TWAP via `/v1/algo/orders`; sub-orders every 30s; duration 30–86400s |
| Iceberg | N | Not documented |
| CopyTrading | N | Not documented |
| GridTrading | N | Not documented |
| BlockTrades | Y | `POST /v1/block-trades` |
| ZK Privacy | Y | Position data private via ZK proofs on Starknet |
| VWAP protection | Y | `vwap_price` param on market orders |

---

## 8. dYdX V4 (DEX — Cosmos chain — Perpetuals Only)

**Auth:** Cosmos wallet (mnemonic → private key). All mutations are `MsgPlaceOrder` / `MsgCancelOrder` / `MsgBatchCancel` Cosmos transactions broadcast via gRPC to validator nodes. Indexer REST API (`indexer.dydx.trade`) is fully public/read-only — no authentication required. Permissioned keys (Authenticators) allow delegating signing to sub-keys.

### Order Types

| Feature | Supported | Notes |
|---------|-----------|-------|
| Market | Y | Short-term IOC order; submitted as `TIME_IN_FORCE_IOC` |
| Limit | Y | Short-term (GTB) or long-term (GTBT) limit order |
| StopMarket | Y | `condition_type=CONDITION_TYPE_STOP_LOSS` + `TIME_IN_FORCE_IOC` |
| StopLimit | Y | `condition_type=CONDITION_TYPE_STOP_LOSS` + `TIME_IN_FORCE_UNSPECIFIED` |
| TrailingStop | Partial | Listed in Indexer type enum as `TRAILING_STOP`; not detailed in protocol docs |
| TP Market | Y | `condition_type=CONDITION_TYPE_TAKE_PROFIT` + IOC |
| TP Limit | Y | `condition_type=CONDITION_TYPE_TAKE_PROFIT` + limit |
| OCO | N | Not supported |
| Bracket | N | Not supported |
| TWAP | Y | `OrderFlags=128`; added in protocol v9.0 |

### Time-in-Force

| TIF | Supported | Notes |
|-----|-----------|-------|
| GTC (GTBT) | Y | Long-term stateful orders; up to 95 days via `good_til_block_time` |
| GTB | Y | Short-term; max current_block + 20 (~30 seconds) |
| IOC | Y | `TIME_IN_FORCE_IOC = 1` (short-term only) |
| FOK | Y | `TIME_IN_FORCE_FILL_OR_KILL = 3` (short-term only) |
| PostOnly | Y | `TIME_IN_FORCE_POST_ONLY = 2` |
| GTD | Partial | GTBT (Good-Till-Block-Time) is effectively GTD with block timestamp |

### Order Management

| Feature | Supported | Notes |
|---------|-----------|-------|
| Single order | Y | `MsgPlaceOrder` broadcast via gRPC |
| Batch placement | Partial | Multiple `MsgPlaceOrder` in single Cosmos tx; no dedicated batch-place proto msg |
| Cancel single | Y | `MsgCancelOrder` with matching OrderId |
| Cancel batch | Y | `MsgBatchCancel` (exists in proto; cancels multiple in single tx) |
| Cancel all | N | No cancel-all message; `MsgBatchCancel` requires explicit order IDs |
| Cancel by symbol | N | No per-market cancel-all |
| Amend | N | No amend; replace short-term order by reusing same OrderId with higher goodTilBlock |
| Get single order | Y | `GET /v4/orders/{orderId}` (Indexer) |
| Get open orders | Y | `GET /v4/orders` with `status=OPEN` (Indexer) |
| Get history | Y | `GET /v4/orders` with status filters; `GET /v4/fills` for fill history (Indexer) |

### Positions

| Feature | Supported | Notes |
|---------|-----------|-------|
| GetPositions | Y | `GET /v4/perpetualPositions` (Indexer) with status filter |
| Close | Partial | No dedicated endpoint; place opposing reduce-only order matching position size |
| SetLeverage | N | No explicit leverage endpoint; leverage implicit via position size / account equity |
| MarginMode | Y | Cross (subaccount 0–127) vs Isolated (subaccount 128+); mode determined by subaccount number |
| AddRemoveMargin | Y | `MsgCreateTransfer` to move USDC between subaccounts (effectively changes leverage) |
| FundingRate | Y | `GET /v4/historicalFunding/{market}`; `GET /v4/fundingPayments` per subaccount (Indexer) |
| LiqPrice | Partial | Not returned by API; calculated client-side from equity + maintenance margin fraction |

### Account

| Feature | Supported | Notes |
|---------|-----------|-------|
| Balances | Y | `GET /v4/addresses/{address}/subaccountNumber/{n}` — equity, freeCollateral, USDC balance (Indexer) |
| Fees | Partial | Fee rates in `GET /v4/perpetualMarkets`; actual fees in fill history |
| InternalTransfer | Y | `MsgCreateTransfer` between subaccounts; `GET /v4/transfers` for history |
| DepositAddr | N/A | CCTP/Noble USDC bridge; no deposit address API |
| Withdraw | Y | `MsgWithdrawFromSubaccount` Cosmos tx; IBC → Noble → Ethereum via CCTP |
| Deposit/Withdraw History | Y | `GET /v4/transfers` (Indexer) covers deposit/withdraw/transfer events |

### Sub-accounts

| Feature | Supported | Notes |
|---------|-----------|-------|
| Create | Y | Auto-created on first deposit to valid subaccount number (0–128,000) |
| List | Y | `GET /v4/addresses/{address}` returns all subaccounts (Indexer) |
| Transfer | Y | `MsgCreateTransfer`; `GET /v4/transfers` + `GET /v4/transfers/between` for history |

### Advanced

| Feature | Supported | Notes |
|---------|-----------|-------|
| TWAP | Y | `OrderFlags=128`; v9.0+ feature |
| Iceberg | N | Not documented |
| CopyTrading | N | Not documented |
| GridTrading | N | Not documented |
| PermissionedKeys | Y | Authenticators allow sub-keys with scoped signing authority |
| EquityTierLimits | Y | Open stateful order count gated by account equity |

---

## Summary Comparison Matrix

### Order Types

| Feature | Upbit | Deribit | HyperLiquid | Lighter | Jupiter | GMX | Paradex | dYdX |
|---------|-------|---------|-------------|---------|---------|-----|---------|------|
| Market | Y | Y | Y | Y | Y | Y | Y | Y |
| Limit | Y | Y | Y | Y | Y | Y | Y | Y |
| StopMarket | N | Y | Y | Y | N | Partial | Y | Y |
| StopLimit | N | Y | Y | Y | N | N | Y | Y |
| TrailingStop | N | Y | N | N | N | N | N | Partial |
| TP | N | Y | Y | Y | Partial | Y | Y | Y |
| SL | N | Y | Y | Y | Partial | Y | Y | Y |
| OCO | N | Y | N | Partial | N | N | N | N |
| Bracket | N | Y | Partial | Partial | N | Partial | N | N |
| TWAP | N | N | Y | Y | N | N | Y | Y |
| Iceberg | N | Y | N | N | N | N | N | N |

### Time-in-Force

| TIF | Upbit | Deribit | HyperLiquid | Lighter | Jupiter | GMX | Paradex | dYdX |
|-----|-------|---------|-------------|---------|---------|-----|---------|------|
| GTC | Partial | Y | Y | Y (GTT) | Partial | Y | Y | Y (GTBT) |
| IOC | Y | Y | Y | Y | N | N | Y | Y |
| FOK | Y | Y | N | N | N | N | N | Y |
| PostOnly | Y | Y | Y (ALO) | Y | N | N | Y | Y |
| GTD | N | Y | N | Partial | Y (expiry) | Partial | N | Partial (GTBT) |

### Order Management

| Feature | Upbit | Deribit | HyperLiquid | Lighter | Jupiter | GMX | Paradex | dYdX |
|---------|-------|---------|-------------|---------|---------|-----|---------|------|
| Batch Place | N | N | Y | Y (50) | N | Y (multicall) | Y (10) | Partial |
| Cancel1 | Y | Y | Y | Y | Y | Y | Y | Y |
| CancelAll | Y | Y | Partial | Y | Y | N | Y | N |
| CancelBySymbol | Y | Y | N | N | Y | N | Y | N |
| Amend | Partial | Y | Y | Y | N | Y | Y | N |
| GetOrder | Y | Y | Y | Partial | Y | Y | Y | Y |
| GetOpen | Y | Y | Y | Y | Y | Y | Y | Y |
| GetHistory | Y | Y | Y | Y | Y | Y (GraphQL) | Y | Y |

### Positions

| Feature | Upbit | Deribit | HyperLiquid | Lighter | Jupiter | GMX | Paradex | dYdX |
|---------|-------|---------|-------------|---------|---------|-----|---------|------|
| GetPositions | N/A | Y | Y | Y | Partial | Y | Y | Y |
| Close | N/A | Y | Partial | Partial | Partial | Y | Partial | Partial |
| SetLeverage | N/A | N | Y | Y | N/A | N | Partial | N |
| MarginMode | N/A | Partial | Y | Partial | N/A | N | Y | Y |
| AddRemoveMargin | N/A | N | Y | N | N/A | Y | Partial | Y |
| FundingRate | N/A | Y | Y | Y | N/A | Y | Y | Y |
| LiqPrice | N/A | Y | Y | N | N/A | Partial | Y | Partial |

### Account

| Feature | Upbit | Deribit | HyperLiquid | Lighter | Jupiter | GMX | Paradex | dYdX |
|---------|-------|---------|-------------|---------|---------|-----|---------|------|
| Balances | Y | Y | Y | Y | Y | Partial | Y | Y |
| Fees | Partial | Y | Y | Y | Partial | Y | Partial | Partial |
| InternalTransfer | N | Y | Y | Y | N/A | N/A | N | Y |
| DepositAddr | Y | Y | N/A | Y | N/A | N/A | N/A | N/A |
| Withdraw | Y | Y | Y | Y | N/A | N/A | N/A | Y |
| Dep/Wdw History | Y | Y | Partial | Y | N/A | Partial | Y | Y |

### Sub-accounts

| Feature | Upbit | Deribit | HyperLiquid | Lighter | Jupiter | GMX | Paradex | dYdX |
|---------|-------|---------|-------------|---------|---------|-----|---------|------|
| Create | N | Y | Partial | Y | N/A | N/A | Partial | Y (auto) |
| List | N | Y | Y | Y | N/A | N/A | N | Y |
| Transfer | N | Y | Y | Y | N/A | N/A | N | Y |
