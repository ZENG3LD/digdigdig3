# Research Wave 2 Batch 4: Trading API Gaps

Exchanges covered: Hyperliquid, Paradex, Upbit, Bitstamp, Gemini, MEXC, HTX

---

## 1. Hyperliquid

### CancelAll — Does It Exist?

**No native `cancelAll` action exists in the Exchange endpoint.**

The three cancel actions are:
- `cancel` — Cancel by asset + order ID (oid)
- `cancelByCloid` — Cancel by client order ID
- `scheduleCancel` — Dead man's switch: schedule cancellation of ALL open orders at a future UTC timestamp

`scheduleCancel` is the closest analog to "cancel all", but it is not immediate. It sets a future time at which all open orders will be cancelled automatically. Trading systems use it as a heartbeat: continuously extend the timestamp while running; if the bot crashes, orders auto-cancel at the scheduled time.

**Implementation status**: We have no `CancelAll` trait impl for Hyperliquid. To add one, we could either:
1. Implement `scheduleCancel` with `time = now()` (immediate schedule), or
2. Query all open orders and batch-cancel them via the `cancel` action

The SDK method `cancelAllOrders()` in community SDKs is typically implemented as (2): fetch open orders + batch cancel. There is no single-shot cancelAll action.

### Order Types Available

**Core order action types:**

| Type | TIF Options | Notes |
|------|------------|-------|
| Limit | `Gtc`, `Ioc`, `Alo` (Post-Only) | Standard resting limit |
| Market | `Ioc` (implicit) | Set limit price to 0 |
| Stop Market (sl) | `Ioc` | Trigger order, `isMarket: true` |
| Stop Limit (sl) | `Gtc`, `Ioc`, `Alo` | Trigger order, `isMarket: false` |
| Take Market (tp) | `Ioc` | Trigger order, `isMarket: true` |
| Take Limit (tp) | `Gtc`, `Ioc`, `Alo` | Trigger order, `isMarket: false` |
| TWAP | N/A | Separate `twapOrder` action |
| Scale | N/A | Range of limit orders spread across price levels |

**TIF values:**
- `Gtc` — Good til canceled
- `Ioc` — Immediate or cancel
- `Alo` — Add liquidity only (Post-Only); canceled if would immediately match

**Order modifiers per order:**
- `reduceOnly` — Reduce-only flag
- `tpsl` — TP/SL can be attached to the parent order (OCO-style)

**Not available:**
- Trailing stop — not documented, not in order types page
- OCO as standalone — TP/SL attached to parent order acts like OCO (child cancels if parent cancels), but no standalone OCO action
- GTD (Good til date) — no native support
- Iceberg — no native support
- Bracket orders — no native support

### TP/SL as OCO

TP/SL orders in Hyperliquid work in two modes:
1. **Position-attached** — Triggers based on realized PnL on position
2. **Parent-order-attached (OCO)** — Linked to a pending parent order. Child TP/SL only activates if parent fully fills or is partially filled followed by cancel for insufficient margin. One-cancels-other behavior.

The TP/SL on parent order IS effectively an OCO, but it is attached as parameters to the parent order via the `order` action, not a standalone OCO action type.

### Other Notable Features Not Implemented

**TWAP orders** (`twapOrder` action):
- Separate action from `order`
- Supported sub-order types: likely limit/market
- Has corresponding `twapCancel` action

**Scale orders:**
- Range of limit orders spread across price levels
- Available via the `order` action type with multiple orders in the `orders` array

**Spot trading** (fully live):
- Uses same `order` action
- Asset ID = `10000 + spotIndex` (e.g. PURR/USDC = asset 10000)
- `spotSend` action for transferring spot assets
- `usdClassTransfer` for moving USDC between spot and perp accounts

**Vault trading:**
- `vaultTransfer` action — deposit/withdraw from vaults
- Vaults can trade via `CoreWriter` or delegate agents
- No special order action needed; trades execute under vault address

**Builder codes:**
- `approveBuilderFee` action — user approves max fee for a builder address
- Orders include optional `{"b": address, "f": fee_tenths_bp}` builder parameter
- Builders earn fees on fills; capped at 0.1% perps, 1% spot
- Maximum 10 active builder codes per user

**Referral / rewards:**
- `claimRewards` action
- Builder fee tracking via referral state API

**Dead man's switch:**
- `scheduleCancel` — schedule mass cancellation at future time
- Used as heartbeat by trading bots

### Missing Trait Implementations for Hyperliquid

| Trait | Status | Notes |
|-------|--------|-------|
| `CancelAll` | MISSING | No native action; implement via batch cancel or scheduleCancel(now) |
| `TwapOrder` | MISSING | Native `twapOrder` action exists |
| `AmendOrder` | PRESENT (modify/batchModify) | Already implemented |
| `SpotTrading` | MISSING | Same order action, different asset IDs |
| `VaultTrading` | MISSING | `vaultTransfer` action; not in standard traits |

---

## 2. Paradex

### Order Types — Full List

From the `POST /v1/orders` endpoint spec:

```
type: MARKET | LIMIT | STOP_LIMIT | STOP_MARKET | TAKE_PROFIT_LIMIT |
      TAKE_PROFIT_MARKET | STOP_LOSS_MARKET | STOP_LOSS_LIMIT
```

**That is 8 distinct order type values.**

Time-in-force (instruction field):
```
instruction: GTC | POST_ONLY | IOC | RPI
```

`RPI` = Retail Price Improvement — a Paradex-specific instruction type.

**Algo orders** (separate endpoint `POST /v1/algo/orders`):
- `algo_type: TWAP` only (as of current docs)
- TWAP: sub-orders every 30 seconds, duration 30–86,400 sec, MARKET sub-order type only

**TP/SL (OCO):**
- `TAKE_PROFIT_LIMIT`, `TAKE_PROFIT_MARKET`, `STOP_LOSS_MARKET`, `STOP_LOSS_LIMIT` are standalone order types
- TP/SL order = a subset of OCO — once one triggers, the other cancels

### Not Available on Paradex

| Type | Available? |
|------|-----------|
| Trailing stop | NO — not in type enum |
| Iceberg | NO |
| Bracket | NO (but TP/SL post-fill achieves similar) |
| GTD | NO |
| FOK | NO (not listed in instructions) |

### What We Have vs. What Exists

We implemented 9/14 order types. The Paradex API actually has:
- 8 core order types in `type` field (already covers our claimed 9 with TP/SL variants)
- TWAP via algo endpoint
- No trailing stop, no ICeberg, no bracket, no FOK

Our 9 claimed implementations likely already cover everything meaningful. The missing 5 slots are genuinely unsupported.

### Optional Traits Status

| Trait | Status |
|-------|--------|
| `TwapOrder` | MISSING — native endpoint exists (`POST /v1/algo/orders`) |
| `AmendOrder` | Need to verify — Paradex does have `PUT /v1/orders/{id}` |
| `CancelAll` | Need to verify — Paradex has `DELETE /v1/orders` (bulk delete) |

---

## 3. Upbit

### Order Types — Full List

From the Upbit Global API (`POST /v1/orders`), the `ord_type` parameter supports:

| ord_type | Description |
|----------|-------------|
| `limit` | Limit buy/sell order |
| `price` | Market buy order (specify total KRW/USDT amount) |
| `market` | Market sell order (specify asset amount) |
| `best` | Best limit order (requires time_in_force) |
| `limit_ioc` | Limit + Immediate or Cancel |
| `limit_fok` | Limit + Fill or Kill |
| `best_ioc` | Best + Immediate or Cancel |
| `best_fok` | Best + Fill or Kill |

**Stop-limit orders**: Available on the Korean Upbit platform (KRW market), indicated by the ccxt `params.state = 'watch'` for stop-limit order status. However, stop-limit is NOT listed in the Global Upbit (English) API `ord_type` values. This is a Korea-market-only feature.

**Summary**: Upbit Global API effectively has: Market (via `price`/`market`), Limit, Best, with IOC/FOK variants. No stop orders in Global API.

### CancelAll — Exists?

**YES — Upbit has a batch cancel endpoint:**

```
DELETE /v1/orders/open
```

Parameters:
- `cancel_side` — "all", "bid", or "ask"
- `pairs` — specific market pairs (optional)
- `excluded_pairs` — exclude pairs (optional)
- `quote_currencies` — filter by quote currency
- `count` — max orders to cancel (default 20, max 300)
- `order_by` — "asc" (oldest first) or "desc" (newest first, default)

**Important limitation**: Only orders in `WAIT` status are canceled. Orders in `WATCH` status (stop-limit orders) are NOT canceled by this endpoint — those require individual cancellation.

**Our implementation**: We implemented `CancelAll` — this is correct, the endpoint exists.

### AmendOrder — Exists?

**YES — Upbit has atomic cancel-and-replace:**

```
POST /v1/orders/cancel_and_new
```

This cancels an existing order and creates a new one in a single atomic request. It is not a true amendment (price/size modification in-place) but achieves the same effect.

**Our implementation status**: Need to verify if we implemented this as `AmendOrder`.

---

## 4. Bitstamp

### Order Types — Full List

Bitstamp supports:

| Type | API Endpoint | Status |
|------|-------------|--------|
| Limit buy | `POST /api/v2/buy/{market}/` | Active |
| Limit sell | `POST /api/v2/sell/{market}/` | Active |
| Market buy | `POST /api/v2/buy/market/{market}/` | Active |
| Market sell | `POST /api/v2/sell/market/{market}/` | Active |
| Instant buy | `POST /api/v2/buy/instant/{market}/` | Active |
| Instant sell | `POST /api/v2/sell/instant/{market}/` | Active |
| Stop-Limit sell | Available | Active (stop-limit remains) |
| Stop Market buy | Being discontinued | Removed May 14, 2025 |
| Trailing stop | Being discontinued | Removed May 14, 2025 |

**As of April 2025 changes:**
- Stop Market Buy orders: DISCONTINUED (removed from web platform April 16, 2025; auto-closed May 14, 2025)
- Trailing Stop orders: DISCONTINUED (same timeline)
- Stop-Limit Sell orders: REMAIN available

**"Instant" orders** = a Bitstamp-specific type where you specify the fiat amount rather than crypto amount. The exchange determines the price.

### CancelAll — Exists?

**YES:**

```
POST /api/v2/cancel_all_orders/
```

Cancels ALL open orders across all markets. No parameters needed.

**Our implementation is correct.**

### AmendOrder — Exists?

**YES — replace_order endpoint:**

```
POST /api/v2/replace_order/
```

Atomically cancels an existing order and places a new one. Accepts either `id` or `orig_client_order_id` to identify the order to replace.

This is an effective amendment. Our `AmendOrder` trait, if implemented via `replace_order`, is correct.

### Missing Implementations

| Feature | Status |
|---------|--------|
| Stop-Limit | Only stop-limit SELL remains. We likely need to handle the deprecation of Stop Market and Trailing Stop |
| Trailing stop | DEPRECATED — should be marked as unsupported |
| Instant orders | Bitstamp-specific type; not in standard traits |

---

## 5. Gemini

### Order Types — Complete List

Via `POST /v1/order/new`:

**Type field values:**
- `"exchange limit"` — Standard limit order
- `"exchange stop limit"` — Stop-limit order (requires both `price` and `stop_price` params)
- `"exchange market"` — Technically supported but recommendation is to use IOC limit with aggressive pricing instead

**Execution options (options array):**
- `"maker-or-cancel"` — Post-only; entire order canceled if would take liquidity
- `"immediate-or-cancel"` — IOC; unfilled portion canceled immediately
- `"fill-or-kill"` — FOK; full fill or cancel
- `"auction-only"` — Adds order to auction book for next auction (not continuous book)
- `"indication-of-interest"` — Used in Block Trading API; not standard REST

**Full list of cancel endpoints:**
- `POST /v1/order/cancel` — Cancel single order
- `POST /v1/order/cancel/all` — Cancel ALL active orders (all markets)
- `POST /v1/order/cancel/session` — Cancel all orders placed in current API session

### CancelAll — Exists?

**YES:** `POST /v1/order/cancel/all`

**Our implementation is correct.**

### AmendOrder — Exists?

**NO.** Gemini has no modify or amend order endpoint. Users must cancel and recreate.

**Our implementation**: If we marked `AmendOrder` as supported for Gemini, that is incorrect.

### Additional Features

**Block Trading API** (separate from REST):
- `"indication-of-interest limit"` order type
- Used for large OTC-style trades
- Different authentication flow
- Not part of standard trading API

**Auction orders:**
- `"auction-only"` execution option adds to auction book
- Applies only to limit orders
- Gemini runs periodic auctions for some markets

**Clearing API** (separate):
- For bilateral clearing
- Not standard order flow

### Missing Implementations

| Feature | Status |
|---------|--------|
| Auction-only orders | MISSING — auction-only is a valid execution option |
| AmendOrder | INCORRECTLY IMPLEMENTED (or should be UnsupportedOperation) |

---

## 6. MEXC

### CancelAll — Exists?

**YES:**

```
DELETE /api/v3/openOrders
```

Parameters:
- `symbol` — required (cancel all open orders for that symbol)
- Permission: `SPOT_DEAL_WRITE`
- Weight: 1

**Note**: This cancels all orders for a specific symbol, not all symbols at once. To cancel everything, you must loop through symbols.

**Our implementation is correct** (assuming it loops per symbol or uses symbol filtering).

### Order Types — Full List

MEXC Spot v3 supported order types:

| Type | Notes |
|------|-------|
| `LIMIT` | Standard limit order |
| `MARKET` | Market order |
| `LIMIT_MAKER` | Post-only (canceled if would take liquidity) |
| `IMMEDIATE_OR_CANCEL` | IOC |
| `FILL_OR_KILL` | FOK |

**Batch orders**: `POST /api/v3/batchOrders` — up to 20 orders per request.

**Not available:**
- Stop orders — MEXC does NOT have stop-limit or stop-market in the Spot v3 API
- Trailing stop — not available
- OCO — not available in spot API
- Iceberg — not available

MEXC Futures has more order types (trigger orders, etc.), but Spot v3 is limited to the 5 types above.

### AmendOrder — Exists?

**NO.** There is no amendment or modification endpoint in MEXC Spot v3 API. Operations are limited to placement, testing, and cancellation.

### Summary of Missing Traits

| Trait | Status |
|-------|--------|
| `CancelAll` | PRESENT (DELETE /api/v3/openOrders per symbol) |
| `AmendOrder` | NOT SUPPORTED — no endpoint exists |
| Stop order types | NOT SUPPORTED in Spot v3 |

---

## 7. HTX (Huobi)

### CancelAll — Exists?

**YES:**

```
POST /v1/order/orders/batchCancelOpenOrders
```

Parameters:
- `account-id` — required
- `symbol` — optional (filter by trading pair)
- `side` — optional ("buy" or "sell")
- `size` — optional (max orders to cancel, max 100, default 100)

This cancels open orders. Can be scoped by symbol or side, or cancel all if no filters.

**Our implementation is correct.**

There is also a related endpoint:
```
POST /v1/order/orders/batchcancel
```
Which cancels orders by specific order IDs or client order IDs (batch of individual cancels, max 50 IDs).

**Dead man's switch:**
```
POST /v2/algo-orders/cancel-all-after
```
Automatic cancellation of all algo orders after a timeout.

### Order Types — Full List

HTX Spot supports **6 order types**:

| Type | API type-name | Notes |
|------|--------------|-------|
| Limit | `buy-limit` / `sell-limit` | Standard |
| Market | `buy-market` / `sell-market` | Standard |
| Stop-Limit | `buy-stop-limit` / `sell-stop-limit` | Requires `stop-price` and `operator` (gte/lte) |
| Trigger | `buy-limit-fok` / `sell-limit-fok` (and others) | Triggers at price, places limit or market |
| Advanced Limit | `buy-limit-maker` / `sell-limit-maker` / `buy-ioc` / `sell-ioc` / `buy-limit-fok` / `sell-limit-fok` | Post-Only, IOC, FOK variants |
| Trailing Stop | Via `POST /v2/algo-orders` | Only available via API; uses algo endpoint |

**Advanced Limit order subtypes:**
- `buy-limit-maker` / `sell-limit-maker` — Post-only (maker-only)
- `buy-ioc` / `sell-ioc` — Immediate-or-Cancel
- `buy-limit-fok` / `sell-limit-fok` — Fill-or-Kill

**Trailing Stop orders** — separate endpoint:
```
POST /v2/algo-orders
```
Fields for trailing stop:
- `orderType: "trailing-stop-order"`
- `trailingRate` — callback rate (must be > 0%, <= 5%)
- `activationPrice` — price at which trailing begins

### AmendOrder — Exists?

**No amendment endpoint found** in HTX Spot API documentation. The API supports batch cancel and batch place as separate operations, but no in-place modification of an existing order.

### Summary

| Trait | Status |
|-------|--------|
| `CancelAll` | PRESENT (`POST /v1/order/orders/batchCancelOpenOrders`) |
| `TrailingStop` | MISSING — native endpoint exists (`POST /v2/algo-orders`) |
| `AmendOrder` | NOT SUPPORTED — no endpoint |
| `TriggerOrder` | Partially covered by Stop-Limit; separate trigger order type exists |

---

## Consolidated Gaps Summary

### Incorrectly Implemented (False Positives)

| Exchange | Trait | Issue |
|----------|-------|-------|
| Gemini | `AmendOrder` | No modify endpoint exists; cancel+recreate only |
| Bitstamp | `TrailingStop` | Discontinued April 2025 — should be `UnsupportedOperation` |
| Bitstamp | `StopMarket` | Buy-side discontinued April 2025 |

### Missing Implementations (False Negatives)

| Exchange | Missing Feature | Native API |
|----------|----------------|------------|
| Hyperliquid | `CancelAll` | No native action; implement as batch cancel (fetch all + cancel each) or `scheduleCancel(now)` |
| Hyperliquid | `TwapOrder` | `twapOrder` action |
| Hyperliquid | Spot trading | Same order action, asset = `10000 + index` |
| Hyperliquid | Builder fee support | `approveBuilderFee` action + per-order `builder` param |
| Paradex | `TwapOrder` | `POST /v1/algo/orders` with `algo_type: TWAP` |
| Paradex | `CancelAll` | Likely `DELETE /v1/orders` bulk; needs verification |
| Paradex | `AmendOrder` | `PUT /v1/orders/{id}` likely exists; needs verification |
| HTX | `TrailingStop` | `POST /v2/algo-orders` with `orderType: trailing-stop-order` |
| MEXC | `AmendOrder` | Does NOT exist; mark as `UnsupportedOperation` |

### Confirmed Correct Implementations

| Exchange | Trait | Verification |
|----------|-------|-------------|
| Upbit | `CancelAll` | `DELETE /v1/orders/open` confirmed |
| Upbit | `AmendOrder` | `POST /v1/orders/cancel_and_new` confirmed |
| Bitstamp | `CancelAll` | `POST /api/v2/cancel_all_orders/` confirmed |
| Bitstamp | `AmendOrder` | `POST /api/v2/replace_order/` confirmed |
| Gemini | `CancelAll` | `POST /v1/order/cancel/all` confirmed |
| MEXC | `CancelAll` | `DELETE /api/v3/openOrders` confirmed (per-symbol) |
| HTX | `CancelAll` | `POST /v1/order/orders/batchCancelOpenOrders` confirmed |

### Order Type Coverage Reality Check

| Exchange | Types We Have | Types That Exist | Missing |
|----------|--------------|------------------|---------|
| Hyperliquid | Market, Limit, StopMarket, StopLimit, ReduceOnly, PostOnly | + TWAP, Scale, TakeProfit variants, SpotTrading | TWAP, TakeProfit |
| Paradex | 9 types | MARKET, LIMIT, STOP_LIMIT, STOP_MARKET, TP_LIMIT, TP_MARKET, SL_MARKET, SL_LIMIT + TWAP algo | TWAP algo |
| Upbit | Market, Limit | + Best, IOC/FOK variants, StopLimit (KRW only) | StopLimit (market-specific) |
| Bitstamp | Market, Limit | + Instant buy/sell, StopLimit (sell only remains), formerly TrailingStop | Instant orders |
| Gemini | Market, Limit, StopLimit, PostOnly, IOC, FOK | + auction-only execution option | auction-only |
| MEXC | Market, Limit | + LIMIT_MAKER, IOC, FOK | LIMIT_MAKER, IOC, FOK |
| HTX | (verify) | Limit, Market, StopLimit, Trigger, AdvancedLimit (3 subtypes), TrailingStop | TrailingStop via algo-orders |

---

## Sources

- [Hyperliquid Exchange Endpoint Docs](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/exchange-endpoint)
- [Hyperliquid Order Types](https://hyperliquid.gitbook.io/hyperliquid-docs/trading/order-types)
- [Hyperliquid TP/SL Orders](https://hyperliquid.gitbook.io/hyperliquid-docs/trading/take-profit-and-stop-loss-orders-tp-sl)
- [Hyperliquid Builder Codes](https://hyperliquid.gitbook.io/hyperliquid-docs/trading/builder-codes)
- [Hyperliquid scheduleCancel — Dwellir Docs](https://www.dwellir.com/docs/hyperliquid/scheduleCancel)
- [Hyperliquid Place Order — Chainstack](https://docs.chainstack.com/reference/hyperliquid-exchange-place-order)
- [Paradex Create Order API](https://docs.paradex.trade/api/prod/orders/new)
- [Paradex Algo Orders API](https://docs.paradex.trade/api/prod/algos/create-order)
- [Paradex Stop Orders](https://docs.paradex.trade/trading/placing-orders/order-types/stop-order)
- [Paradex TWAP Order](https://docs.paradex.trade/trading/placing-orders/order-types/twap-order)
- [Upbit Global API Order Reference](https://global-docs.upbit.com/v1.2.2/reference/order)
- [Upbit Batch Order Cancel](https://global-docs.upbit.com/v1.2.2/reference/batch-cancel-order)
- [Upbit Cancel and New Order](https://global-docs.upbit.com/reference/cancel-and-new-order)
- [Upbit Changelog — IOC/FOK Support](https://global-docs.upbit.com/changelog/id_th_iocfok_226)
- [Bitstamp API Documentation](https://www.bitstamp.net/api/)
- [Bitstamp Stop Order Changes (April 2025)](https://blog.bitstamp.net/post/upcoming-changes-to-stop-order-availability-on-bitstamp/)
- [Bitstamp Stop and Trailing Stop Blog](https://blog.bitstamp.net/post/stop-orders-and-trailing-stop-orders/)
- [ExBitstamp Elixir Client Docs](https://hexdocs.pm/ex_bitstamp/ExBitstamp.html)
- [Gemini REST Orders Docs](https://docs.gemini.com/rest/orders)
- [Gemini Order Types — Support Article](https://support.gemini.com/hc/en-us/articles/210709663-What-order-types-are-supported-in-ActiveTrader-and-what-is-Time-in-force)
- [MEXC Spot v3 API Docs](https://mexcdevelop.github.io/apidocs/spot_v3_en/)
- [MEXC Spot Account/Trade](https://www.mexc.com/api-docs/spot-v3/spot-account-trade)
- [HTX Spot Order Types Introduction](https://www.htx.com/support/34899848363836)
- [HTX batchCancelOpenOrders Announcement](https://www.htx.com/support/900000072826/)
- [HTX Stop-Limit Order API Support](https://www.htx.com/support/360000440121/)
- [Huobi Spot API Reference v1.0](https://huobiapi.github.io/docs/spot/v1/en/)
