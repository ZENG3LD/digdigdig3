# Trading API Gap Verification — Wave 2 Batch 2
## Exchanges: Gate.io, Kraken, KuCoin, Coinbase + MEXC/HTX CancelAll

**Research date:** 2026-03-12
**Sources:** Official exchange API documentation (2024-2025)

---

## 1. Optional Traits — Missing Implementations

### 1.1 Coinbase — CancelAll

**VERDICT: YES, a cancel-all mechanism exists. Implement as CancelAll.**

Coinbase Advanced Trade has:

```
POST /api/v3/brokerage/orders/batch_cancel
```

Request body:
```json
{ "order_ids": ["id1", "id2", ...] }
```

- Cancels "one or more orders" in a single request
- Maximum per call: **100 order IDs** (confirmed from changelog)
- No "cancel all without IDs" endpoint exists — you must supply order IDs
- Therefore true "cancel all" requires: fetch open orders → extract IDs → call batch_cancel

**Implementation strategy for `CancelAll`:**
```
GET /api/v3/brokerage/orders/historical/batch (with status=OPEN)
  → collect all order_ids
  → POST /api/v3/brokerage/orders/batch_cancel with chunks of 100
```

Source: https://docs.cdp.coinbase.com/api-reference/advanced-trade-api/rest-api/orders/cancel-order

---

### 1.2 Coinbase — BatchOrders

**VERDICT: NOT SUPPORTED (single order creation only).**

The `POST /api/v3/brokerage/orders` endpoint creates exactly one order per call. No batch create endpoint exists in the Advanced Trade API. The Python SDK methods (`market_order_buy()`, `limit_order_gtc()`, etc.) all call the same single-order endpoint.

Source: https://coinbase.github.io/coinbase-advanced-py/

---

### 1.3 KuCoin — BatchOrders

**VERDICT: SUPPORTED on both Spot and Futures. Implement BatchOrders.**

**Spot (HF/Pro account):**
```
POST /api/v1/hf/orders/multi
```
- Up to **5 limit orders** per batch, same trading pair
- Only limit orders (not market orders)

**Old Classic Spot (legacy, still works):**
```
POST /api/v1/orders/multi
```
- Listed under "abandoned endpoints" in new docs — prefer HF endpoint

**Futures:**
```
POST /api/v1/orders/multi   (base: api-futures.kucoin.com)
```
- Up to **20 orders** per batch
- Supports limit, market, and stop orders
- Parameters: `clientOid`, `side`, `symbol`, `leverage`, `type`, `price`, `size`, `timeInForce`, `reduceOnly`, `marginMode`, `positionSide`

Source:
- https://www.kucoin.com/docs-new/rest/futures-trading/orders/batch-add-orders
- https://www.kucoin.com/docs/rest/spot-trading/spot-hf-trade-pro-account/place-multiple-orders

---

### 1.4 Gate.io — AmendOrder on Spot

**VERDICT: YES, spot amend exists. Already should be implemented for Spot.**

```
PATCH /spot/orders/{order_id}
```
- Introduced in API v4.35.0
- Supports `amend_text` field
- `x_gate_exptime` header for expiration

**Batch spot amend:**
```
POST /spot/amend_batch_orders
```
- Added in API v4.57.0
- Up to 5 orders per batch
- Works for spot, unified account, and isolated margin accounts

Source: https://www.gate.com/docs/developers/apiv4/en/

---

## 2. Base Trait Match Arm Gaps — Order Type Support

### 2.1 Kraken — Supported Order Types

**Current implementation: 9/14 arms**

Kraken REST API and WebSocket v2 `ordertype` parameter values (complete official list):

| Order Type | Supported | Notes |
|------------|-----------|-------|
| `market` | YES | Immediate full fill at best price |
| `limit` | YES | Standard limit order |
| `stop-loss` | YES | Stop → market order (unfavorable direction) |
| `stop-loss-limit` | YES | Stop → limit order (unfavorable direction) |
| `take-profit` | YES | Stop → market order (favorable direction) |
| `take-profit-limit` | YES | Stop → limit order (favorable direction) |
| `trailing-stop` | YES | Market order triggered on price reversal from peak |
| `trailing-stop-limit` | YES | Limit order triggered on price reversal from peak |
| `iceberg` | YES | Partially hidden order |
| `settle-position` | YES | Close leveraged position |
| `post_only` | YES (flag) | Via `post_only: true` parameter, not separate type |
| OCO | NO | Not supported as a discrete order type |
| Bracket | PARTIAL | OTO (One-Triggers-Other) via `conditional` block |
| TWAP | NO | Not supported |

**Batch Orders:** `POST /private/AddOrderBatch` — 2 to 15 orders, single pair per batch.

**CancelAll:** Two mechanisms:
1. `POST /private/CancelAll` — immediate cancel all open orders (REST)
2. WebSocket v2 `cancel_all` method — immediate cancel with token
3. `POST /private/CancelAllOrdersAfter` — dead man's switch (set countdown timer)

**Gap analysis for our 9/14 implementation:**
- We likely have: market, limit, stop-loss (as StopMarket), stop-loss-limit (StopLimit), trailing-stop (TrailingStop), post-only (PostOnly), take-profit, take-profit-limit
- We are missing: **Iceberg** (native support via `iceberg` type), **settle-position** (margin close)
- Not implementable: OCO, Bracket (not truly OTO), TWAP

Source: https://docs.kraken.com/api/docs/websocket-v2/add_order/

---

### 2.2 KuCoin — Supported Order Types

**Current implementation: 9/14 arms**

KuCoin Spot supports **6 order types** (per official documentation):

| Order Type | Supported | Endpoint | Notes |
|------------|-----------|----------|-------|
| `market` | YES | `POST /api/v3/hf/orders` | Standard market order |
| `limit` | YES | `POST /api/v3/hf/orders` | Standard limit order |
| `stop-market` | YES | `POST /api/v3/orders/stop` | Stop → market |
| `stop-limit` | YES | `POST /api/v3/orders/stop` | Stop → limit |
| `OCO` | YES | `POST /api/v3/oco/order` | Native OCO endpoint |
| `trailing-stop` | YES | (UI-available, API docs confirm) | Trailing by % or amount |
| PostOnly | YES (flag) | `POST /api/v3/hf/orders` | `postOnly: true` |
| Iceberg | YES (flag) | `POST /api/v3/hf/orders` | `iceberg: true` + `visibleSize` |

**KuCoin Futures additional types:**
- `POST /api/v1/orders` with TP/SL parameters (`stopLoss`, `takeProfit`)
- Native TP/SL on futures orders (not position modification)

**OCO endpoint details:**
```
POST https://api.kucoin.com/api/v3/oco/order
```
Parameters: `symbol`, `side`, `price`, `size`, `stopPrice`, `limitPrice`, `clientOid`, `tradeType`

**CancelAll:**
- `DELETE /api/v1/hf/orders/cancelAll` — cancels all spot orders for all symbols
- `DELETE /api/v1/hf/orders/cancelAll/{symbol}` — by specific symbol

**Gap analysis for our 9/14 implementation:**
- We may be missing: **OCO** (native, separate endpoint), **TrailingStop** (confirmed API support)
- StopMarket and StopLimit are handled via stop order endpoint, not main order endpoint

Source:
- https://www.kucoin.com/docs-new/rest/spot-trading/orders/add-oco-order
- https://www.kucoin.com/docs-new/rest/spot-trading/orders/cancel-all-orders

---

### 2.3 Gate.io — Supported Order Types

**Current implementation: 7/14 arms**

Gate.io Spot supported types:

| Order Type | Supported | Endpoint | Notes |
|------------|-----------|----------|-------|
| `market` | YES | `POST /spot/orders` | Added v4.34.0 |
| `limit` | YES | `POST /spot/orders` | Standard |
| `iceberg` | YES (flag) | `POST /spot/orders` | `iceberg` field (partial hide only, taker fee on hidden portion) |
| `post_only` | YES (tif) | `POST /spot/orders` | `time_in_force: "poc"` (PendingOrCancelled) |
| `stop-limit` | YES (via price_orders) | `POST /spot/price_orders` | Price-triggered conditional order |
| `stop-market` | YES (via price_orders) | `POST /spot/price_orders` | Price-triggered → market |
| OCO | NO | — | Officially not supported on spot |
| TrailingStop | NO (spot) | Futures only | `POST /futures/{settle}/autoorder/v1/trail/create` |
| Bracket/TWAP | NO | — | Not supported |

**Gate.io Spot Price-Triggered Orders:**
```
POST /spot/price_orders       — create conditional (stop) order
GET  /spot/price_orders       — list pending conditional orders
DELETE /spot/price_orders     — cancel all conditional orders
DELETE /spot/price_orders/{id} — cancel single conditional order
```

**Gate.io Futures Trailing Stop (separate feature):**
```
POST /futures/{settle}/autoorder/v1/trail/create
POST /futures/{settle}/autoorder/v1/trail/stop
POST /futures/{settle}/autoorder/v1/trail/stop_all
GET  /futures/{settle}/autoorder/v1/trail/list
POST /futures/{settle}/autoorder/v1/trail/update
```

**CancelAll on Spot:**
```
DELETE /spot/orders
```
- Optional `currency_pair` parameter (if omitted: cancels all pairs)
- Also: `DELETE /spot/price_orders` cancels all conditional orders

**Gap analysis for our 7/14 implementation:**
- We are missing: **StopMarket** and **StopLimit** (via `/spot/price_orders`), **Iceberg** (flag on regular order), **PostOnly** (via `poc` tif)
- We correctly skip: OCO, TrailingStop (spot), Bracket, TWAP

Source: https://www.gate.com/docs/developers/apiv4/en/

---

### 2.4 Coinbase — Supported Order Types

**Current implementation: 10/14 arms**

Coinbase Advanced Trade API `order_configuration` types (complete official list from API reference):

| Configuration Key | Type | Notes |
|-------------------|------|-------|
| `market_market_ioc` | Market | Immediate-or-cancel market |
| `market_market_fok` | Market FOK | Perpetuals only |
| `sor_limit_ioc` | Smart Order Routing | IOC with routing |
| `limit_limit_gtc` | Limit GTC | Standard limit, optional `post_only` |
| `limit_limit_gtd` | Limit GTD | Limit with expiry, optional `post_only` |
| `limit_limit_fok` | Limit FOK | Fill-or-kill |
| `twap_limit_gtd` | TWAP | Buckets: `number_buckets`, `bucket_size`, `start_time`, `end_time` |
| `stop_limit_stop_limit_gtc` | Stop-Limit GTC | `stop_price`, `limit_price`, `stop_direction` |
| `stop_limit_stop_limit_gtd` | Stop-Limit GTD | + `end_time` |
| `trigger_bracket_gtc` | Bracket GTC | `stop_trigger_price` for exit |
| `trigger_bracket_gtd` | Bracket GTD | + `end_time` |
| `scaled_limit_gtc` | Scaled/Grid | `num_orders`, `min_price`, `max_price`, `price_distribution`, `size_distribution` |

**Notable findings:**
- **TWAP**: YES, natively supported (`twap_limit_gtd`)
- **Bracket**: YES, natively supported (`trigger_bracket_gtc/gtd`)
- **PostOnly**: YES, flag on `limit_limit_gtc` and `limit_limit_gtd`
- **Iceberg**: NOT in REST API (only in Exchange FIX API via separate fix-msg-oe-iceberg)
- **TrailingStop**: NOT supported
- **OCO**: NOT supported
- **StopMarket**: NOT supported (stop orders always produce limit orders, i.e., stop-limit only)
- **Scaled/Grid**: YES, via `scaled_limit_gtc` — not in our trait mapping

**CancelAll:**
```
POST /api/v3/brokerage/orders/batch_cancel
Body: { "order_ids": ["id1", "id2"] }
```
- Maximum 100 order IDs per call
- No direct "cancel all without IDs" — must fetch open orders first
- CancelAll implementation: paginate list → batch cancel in chunks of 100

**Gap analysis for our 10/14 implementation:**
- We likely have: Market, Limit, StopLimit, Bracket, TWAP, PostOnly — 10 arms
- We correctly return `UnsupportedOperation` for: TrailingStop, OCO, Iceberg (REST), StopMarket
- Potential missing arm: **ScaledLimit** (grid/scaled orders — unique to Coinbase)

Source:
- https://docs.cdp.coinbase.com/api-reference/advanced-trade-api/rest-api/orders/create-order
- https://docs.cdp.coinbase.com/api-reference/advanced-trade-api/rest-api/orders/cancel-order

---

## 3. Features We Don't Have Implemented

### 3.1 TP/SL as Order-Level Parameters

**KuCoin Futures:**
- `POST /api/v2/orders` (futures) supports `takeProfit` and `stopLoss` as sub-objects in the order body
- Endpoint: `POST /api/v3/order/tp-sl` for separate TP/SL placement
- This is native TP/SL at order placement time, not position modification

**Gate.io Futures:**
- Orders support `tpsl_r` (take profit / stop loss ratio) parameter on futures orders
- `Introduction to TP/SL` is a documented feature

**Kraken:**
- The `conditional` object on orders allows OTO (One-Triggers-Other) which effectively creates TP/SL on order fill
- Fields: `order_type`, `limit_price`, `trigger_price`

### 3.2 Earn/Lending/Staking

**KuCoin:**
- Earn API: `GET/POST /api/v1/earn/` endpoints for staking, savings products
- Lending market: `POST /api/v1/margin/order` for lend/borrow

**Gate.io:**
- Earn endpoints in separate `earn/` category
- Crypto lending: `POST /earn/uni/lends`

**Coinbase:**
- No earn/staking endpoints in Advanced Trade API (separate Coinbase product)

**Kraken:**
- Staking: `POST /private/Stake`, `POST /private/Unstake`, `GET /private/Staking/Assets`

### 3.3 Portfolio Margin

**Gate.io:**
- Portfolio margin account type supported via `account=portfolio` parameter on orders

**Coinbase:**
- Separate portfolios via `portfolio_uuid` parameter
- `GET/POST /api/v3/brokerage/portfolios`

### 3.4 Conditional/Trigger Orders Beyond Standard Stops

**Gate.io:**
- Price-triggered orders: `POST /spot/price_orders` — trigger based on last, bid, or ask price
- Trigger rules: `>=`, `<=`
- Trigger order types: `limit` or `market` on trigger

**KuCoin:**
- Stop orders API: `POST /api/v3/orders/stop` — stop-market or stop-limit
- Trigger condition: `stopTriggerType` (market, last, bid, ask, index)

### 3.5 Copy Trading

**Gate.io:** Has copy trading API (`/api/v4/copy-trading/`)
**KuCoin:** Has copy trading product but no documented public API endpoints
**Coinbase, Kraken:** No copy trading

### 3.6 Auto-invest / DCA

**Coinbase:**
- Recurring buys via `POST /api/v3/brokerage/orders` with `twap_limit_gtd` is the programmatic equivalent
- No dedicated "recurring investment" API endpoint in Advanced Trade

**KuCoin:**
- DCA bots documented separately as "Trading Bot API" — not in main order API

### 3.7 Block Trading

**Kraken:**
- Institutional block trading via separate OTC desk, not API
- No programmatic block trade API

**Coinbase:**
- Coinbase Prime (institutional) has block trading, not Advanced Trade API

---

## 4. CancelAll — Detailed Specifics

### 4.1 Coinbase — batch_cancel

```
POST /api/v3/brokerage/orders/batch_cancel
Authorization: Bearer {api_key}
Content-Type: application/json

{ "order_ids": ["0000-00000", "1111-11111"] }
```

**Response:**
```json
{
  "results": [
    { "success": true, "failure_reason": "", "order_id": "0000-00000" },
    { "success": false, "failure_reason": "UNKNOWN_CANCEL_ORDER", "order_id": "1111-11111" }
  ]
}
```

- This IS the cancel-all mechanism when combined with a list-orders call
- No single-call "nuke all" endpoint without order IDs
- Max 100 IDs per request

### 4.2 MEXC — CancelAll

**VERDICT: YES, MEXC has a cancel-all endpoint. Not implemented.**

```
DELETE /api/v3/openOrders
```

Parameters:
- `symbol` (string, REQUIRED): Up to 5 symbols comma-separated (e.g. `"BTCUSDT,MXUSDT"`)
- `recvWindow` (optional)
- `timestamp` (required, HMAC)

Permission: `SPOT_DEAL_WRITE`
Weight: 1

**Note:** Unlike most exchanges, MEXC requires `symbol` — you cannot cancel all orders across ALL pairs in one call. You must provide at least one symbol, up to 5 at a time.

Source: https://mexcdevelop.github.io/apidocs/spot_v3_en/

### 4.3 HTX — CancelAll

**VERDICT: YES, HTX has cancel-all-open-orders. Not implemented.**

Two relevant endpoints:

**1. Cancel open orders by criteria (cancel-all equivalent):**
```
POST /v1/order/orders/batchCancelOpenOrders
```
- Optional `symbol` parameter (filter by trading pair)
- Without symbol: cancels all open orders
- Returns list of cancelled order IDs

**2. Cancel multiple orders by IDs:**
```
POST /v1/order/orders/batchcancel
```
- Requires `order-ids` array (specific IDs)
- Up to 50 orders per request

**3. Dead man's switch:**
```
POST /v2/order/cancel-all-after
```
- Timeout-based auto-cancel (not immediate cancel-all)

For `CancelAll` trait: use `POST /v1/order/orders/batchCancelOpenOrders` — this is the true cancel-all.

Source: https://huobiapi.github.io/docs/spot/v1/en/

---

## 5. Summary Matrix — What to Implement

### Optional Traits to Add:

| Exchange | Trait | Action |
|----------|-------|--------|
| Coinbase | `CancelAll` | Implement: list open orders → batch_cancel in chunks of 100 |
| Coinbase | `BatchOrders` | Skip — no native batch create endpoint |
| KuCoin | `BatchOrders` | Implement: spot uses `/api/v1/hf/orders/multi` (5 max), futures `/api/v1/orders/multi` (20 max) |
| Gate.io | `AmendOrder` (Spot) | Implement: `PATCH /spot/orders/{order_id}` |
| MEXC | `CancelAll` | Implement: `DELETE /api/v3/openOrders` (symbol required, up to 5 per call) |
| HTX | `CancelAll` | Implement: `POST /v1/order/orders/batchCancelOpenOrders` |

### Match Arms to Add:

| Exchange | Missing Arms | Action |
|----------|-------------|--------|
| Kraken | `Iceberg` | Add: `ordertype: "iceberg"` with `displayvol` param |
| Kraken | `TrailingStop` + `TrailingStopLimit` | Already in? Verify arm names map to kraken `trailing-stop` / `trailing-stop-limit` |
| KuCoin | `Oco` | Add: separate endpoint `POST /api/v3/oco/order` |
| KuCoin | `TrailingStop` | Add: separate stop order via stop API |
| Gate.io | `StopMarket` | Add: via `POST /spot/price_orders` with `order_type: "market"` |
| Gate.io | `StopLimit` | Add: via `POST /spot/price_orders` with `order_type: "limit"` |
| Gate.io | `Iceberg` | Add: `iceberg` field flag on regular order |
| Gate.io | `PostOnly` | Add: `time_in_force: "poc"` |
| Coinbase | `ScaledLimit` | Consider adding as new enum variant — unique to Coinbase |

---

## 6. Key Corrections to Prior Assumptions

1. **KuCoin BatchOrders**: We said "no native batch" — WRONG. Both spot HF (`/api/v1/hf/orders/multi`, 5 max) and futures (`/api/v1/orders/multi`, 20 max) have native batch endpoints.

2. **Gate.io Spot AmendOrder**: EXISTS. `PATCH /spot/orders/{order_id}` introduced v4.35.0. Batch amend via `POST /spot/amend_batch_orders` added v4.57.0.

3. **Coinbase CancelAll**: Not a single-call cancel-all, but `batch_cancel` with up to 100 IDs effectively implements it in 1-2 requests for typical use.

4. **MEXC CancelAll**: Exists as `DELETE /api/v3/openOrders` but requires symbol (up to 5 symbols per call).

5. **HTX CancelAll**: Exists as `POST /v1/order/orders/batchCancelOpenOrders` — true cancel-all without needing order IDs.

6. **Kraken CancelAll**: Two forms — REST `POST /private/CancelAll` (immediate) and `POST /private/CancelAllOrdersAfter` (dead man's switch). The immediate form should be used for our `CancelAll` trait.

7. **Coinbase Iceberg**: Available only in the Exchange FIX API, NOT in the Advanced Trade REST API. REST API does not support iceberg orders.

8. **Coinbase TWAP**: Fully supported via `twap_limit_gtd` order configuration — confirmed as native API feature.

---

## Sources

- [Coinbase Advanced Trade Create Order](https://docs.cdp.coinbase.com/api-reference/advanced-trade-api/rest-api/orders/create-order)
- [Coinbase Advanced Trade Cancel Orders](https://docs.cdp.coinbase.com/api-reference/advanced-trade-api/rest-api/orders/cancel-order)
- [Coinbase Advanced Trade API Overview](https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/guides/orders)
- [KuCoin Futures Batch Add Orders](https://www.kucoin.com/docs-new/rest/futures-trading/orders/batch-add-orders)
- [KuCoin Spot Place Multiple Orders](https://www.kucoin.com/docs/rest/spot-trading/spot-hf-trade-pro-account/place-multiple-orders)
- [KuCoin Spot Add OCO Order](https://www.kucoin.com/docs-new/rest/spot-trading/orders/add-oco-order)
- [KuCoin Spot Cancel All Orders](https://www.kucoin.com/docs-new/rest/spot-trading/orders/cancel-all-orders)
- [KuCoin Stop Order Introduction](https://www.kucoin.com/docs/rest/spot-trading/stop-order/introduction)
- [Gate.io API v4 Documentation](https://www.gate.com/docs/developers/apiv4/en/)
- [Gate.io Spot Iceberg Orders Adjustment](https://www.gate.com/announcements/article/29076)
- [Kraken WebSocket v2 Add Order](https://docs.kraken.com/api/docs/websocket-v2/add_order/)
- [Kraken REST Add Order Batch](https://docs.kraken.com/api/docs/rest-api/add-order-batch/)
- [Kraken Cancel All Orders After](https://docs.kraken.com/api/docs/rest-api/cancel-all-orders-after/)
- [Kraken WebSocket Cancel All](https://docs.kraken.com/api/docs/websocket-v2/cancel_all/)
- [MEXC Spot API v3 Documentation](https://mexcdevelop.github.io/apidocs/spot_v3_en/)
- [HTX Spot API v1 Documentation](https://huobiapi.github.io/docs/spot/v1/en/)
- [HTX Batch Cancel Announcement](https://www.htx.com/support/900000072826/)
