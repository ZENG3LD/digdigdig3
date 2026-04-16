# Gemini Exchange — Trading API Specification

Source: https://docs.gemini.com/rest/orders
Retrieved: 2026-03-11

---

## Order Types Supported

### Supported Order Types

| Type String (API value) | Description |
|-------------------------|-------------|
| `"exchange limit"` | Standard limit order — rests on book at specified price |
| `"exchange stop limit"` | Two-part order: triggered at `stop_price`, placed at `price` |
| `"exchange market"` | NOT a true market order type string; achieved via `"exchange limit"` + `"immediate-or-cancel"` execution option |

**NOTE:** Gemini does NOT have a dedicated `"exchange market"` type in the `type` field. Market-like behavior is achieved by combining `"exchange limit"` with the `"immediate-or-cancel"` execution option (fills immediately or cancels).

### NOT Supported
- Trailing Stop: NOT AVAILABLE
- Stop-Market: NOT AVAILABLE (stop-limit only, no stop-market variant)
- Bracket / OCO orders: NOT AVAILABLE
- TWAP / Iceberg / Algo orders: NOT AVAILABLE
- Grid trading: NOT AVAILABLE
- Copy trading API: NOT AVAILABLE

---

## Time-in-Force / Execution Options

Gemini does NOT use the traditional TIF field names (GTC/GTD/IOC/FOK). Instead, it uses an `options` array with mutually exclusive execution option strings.

| Execution Option String | Equivalent TIF | Behavior |
|-------------------------|---------------|----------|
| `"maker-or-cancel"` | Post-Only | Order adds liquidity only; cancels immediately if it would take liquidity |
| `"immediate-or-cancel"` | IOC | Fills as much as possible immediately; cancels unfilled remainder |
| `"fill-or-kill"` | FOK | Entire order must fill immediately or the entire order is cancelled |
| *(no option specified)* | GTC | Default behavior — order rests on book until filled or cancelled |

**Constraints:**
- Only ONE execution option may be specified per order (mutually exclusive)
- Execution options apply to `"exchange limit"` orders ONLY
- `"exchange stop limit"` orders do NOT support execution options
- GTD (Good-Till-Date): NOT AVAILABLE

---

## Order Placement

### Single Order Endpoint

**Method:** POST
**Path:** `/v1/order/new`
**Auth Required:** Yes — Trader role

#### Request Payload (JSON, base64-encoded as X-GEMINI-PAYLOAD header)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `request` | string | Yes | Must be `"/v1/order/new"` |
| `nonce` | integer | Yes | Monotonically increasing integer (millisecond timestamp recommended) |
| `symbol` | string | Yes | Trading pair, e.g. `"btcusd"`, `"ethusd"` |
| `amount` | string | Yes | Quantity in base currency (as string to preserve precision) |
| `price` | string | Yes | Limit price per unit (as string) |
| `side` | string | Yes | `"buy"` or `"sell"` |
| `type` | string | Yes | `"exchange limit"` or `"exchange stop limit"` |
| `stop_price` | string | Conditional | Required for `"exchange stop limit"` only — trigger price |
| `options` | array | No | Execution options — max one item: `["maker-or-cancel"]`, `["immediate-or-cancel"]`, or `["fill-or-kill"]` |
| `client_order_id` | string | No | User-defined order identifier (max 100 chars) |
| `margin_order` | boolean | No | Use borrowed funds (margin-enabled accounts only); defaults to `false` |
| `account` | string | No | Sub-account name; required when using Master API key to act on a specific account |

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `order_id` | string | Exchange-assigned order ID |
| `client_order_id` | string | User-provided ID (if supplied) |
| `symbol` | string | Trading pair |
| `exchange` | string | Always `"gemini"` |
| `price` | string | Order price |
| `avg_execution_price` | string | Average fill price |
| `side` | string | `"buy"` or `"sell"` |
| `type` | string | Order type |
| `timestamp` | string | Unix timestamp (seconds) |
| `timestampms` | integer | Unix timestamp (milliseconds) |
| `is_live` | boolean | `true` if order is resting on book |
| `is_cancelled` | boolean | `true` if order was cancelled |
| `is_hidden` | boolean | `true` if order is hidden (reserve order) |
| `was_forced` | boolean | Whether order was placed by system |
| `executed_amount` | string | Amount filled so far |
| `remaining_amount` | string | Amount not yet filled |
| `options` | array | Execution options applied |
| `stop_price` | string | Stop trigger price (stop-limit only) |

---

### Batch/Bulk Order Placement

NOT AVAILABLE. Gemini does not provide a batch order endpoint. Orders must be placed individually.

---

### Conditional Orders (TP/SL, OCO, Bracket)

NOT AVAILABLE. No OCO, bracket, or linked conditional orders exist in the REST API.

---

## Order Management

### Cancel Single Order

**Method:** POST
**Path:** `/v1/order/cancel`
**Auth Required:** Yes — Trader role

#### Request Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `request` | string | Yes | `"/v1/order/cancel"` |
| `nonce` | integer | Yes | Monotonically increasing |
| `order_id` | integer | Yes | Exchange order ID to cancel |
| `account` | string | No | Sub-account name (Master key only) |

**Cancellation Reasons (returned in response):**
- `MakerOrCancelWouldTake`
- `ExceedsPriceLimits`
- `SelfCrossPrevented`
- `ImmediateOrCancelWouldPost`
- `FillOrKillWouldNotFill`
- `Requested` (manual cancel)
- `MarketClosed`
- `TradingClosed`

---

### Cancel All Active Orders

**Method:** POST
**Path:** `/v1/order/cancel/all`
**Auth Required:** Yes — Trader role
**Scope:** Cancels ALL active orders across ALL sessions for the account.

**Response:** Returns `cancelledOrders` (array of order IDs) and `cancelRejects` (array of failed cancels with reasons).

---

### Cancel All Session Orders

**Method:** POST
**Path:** `/v1/order/cancel/session`
**Auth Required:** Yes — Trader role
**Scope:** Cancels only orders placed in the CURRENT session (same API key session). Equivalent to what happens when heartbeat expires.

---

### Cancel by Symbol

NOT AVAILABLE as a dedicated endpoint. Use Cancel All and re-place desired orders, or cancel individually by order_id.

---

### Amend/Modify Order

NOT AVAILABLE. Gemini does not support modifying an existing order's price or quantity. Must cancel and re-place.

---

### Get Single Order Status

**Method:** POST
**Path:** `/v1/order/status`
**Auth Required:** Yes — Trader or Auditor role

#### Request Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `request` | string | Yes | `"/v1/order/status"` |
| `nonce` | integer | Yes | Monotonically increasing |
| `order_id` | integer | Conditional | Exchange order ID (cannot use with `client_order_id`) |
| `client_order_id` | string | Conditional | User order ID (cannot use with `order_id`) |
| `include_trades` | boolean | No | If `true`, response includes individual fill events |
| `account` | string | No | Sub-account name (Master key only) |

**Response:** Same fields as order creation response, plus `trades` array if `include_trades` is `true`.

---

### Get Open (Active) Orders

**Method:** POST
**Path:** `/v1/orders`
**Auth Required:** Yes — Trader or Auditor role

Returns an array of all currently active (resting/unfilled) orders.

#### Request Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `request` | string | Yes | `"/v1/orders"` |
| `nonce` | integer | Yes | Monotonically increasing |
| `account` | string | No | Sub-account name (Master key only) |

**Filtering:** No symbol filter available — returns ALL active orders.

---

### Get Order History

**Method:** POST
**Path:** `/v1/orders/history`
**Auth Required:** Yes — Trader or Auditor role

#### Request Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `request` | string | Yes | `"/v1/orders/history"` |
| `nonce` | integer | Yes | Monotonically increasing |
| `limit_orders` | integer | No | Default: 50, Maximum: 500 |
| `timestamp` | integer | No | Only return orders after this Unix timestamp |
| `symbol` | string | No | Filter by trading pair |
| `account` | string | No | Sub-account name (Master key only) |

**Pagination:** Walk backward through history by using the highest returned timestamp + 1 on each subsequent call, until an empty list is returned.

---

### List Past Trades

**Method:** POST
**Path:** `/v1/mytrades`
**Auth Required:** Yes — Trader or Auditor role

#### Request Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `request` | string | Yes | `"/v1/mytrades"` |
| `nonce` | integer | Yes | Monotonically increasing |
| `symbol` | string | Yes | Trading pair |
| `limit_trades` | integer | No | Default: 50, Maximum: 500 |
| `timestamp` | integer | No | Only return trades after this Unix timestamp |
| `account` | string | No | Sub-account name (Master key only) |

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `price` | string | Fill price |
| `amount` | string | Fill quantity |
| `timestamp` | integer | Unix timestamp (seconds) |
| `timestampms` | integer | Unix timestamp (milliseconds) |
| `type` | string | `"Buy"` or `"Sell"` |
| `aggressor` | boolean | `true` if this side was the taker |
| `fee_currency` | string | Currency of fee |
| `fee_amount` | string | Fee paid |
| `tid` | integer | Trade ID |
| `order_id` | string | Parent order ID |
| `client_order_id` | string | User-defined order ID (if set) |
| `exchange` | string | Always `"gemini"` |
| `is_auction_fill` | boolean | Whether fill was from auction |
| `break` | string | Present if trade was broken (settlement issue) |

---

### Get Trading Volume

**Method:** POST
**Path:** `/v1/tradevolume`
**Auth Required:** Yes — Trader or Auditor role

Returns per-symbol volume data including total base volume, buy/sell counts, and notional amounts.

---

### Get Notional Volume (Fee Tier)

**Method:** POST
**Path:** `/v1/notionalvolume`
**Auth Required:** Yes — Trader or Auditor role

Returns 30-day and 1-day notional volume and the current fee tier including:
- `maker_fee_bps` — maker fee in basis points
- `taker_fee_bps` — taker fee in basis points
- `auction_fee_bps` — auction fee in basis points
- Separate rates for `web`, `api`, and `fix` channels
- `fee_tier` object with tier name and volume thresholds

---

## Position Management (Futures / Perpetuals)

Gemini supports perpetual swap instruments (symbols ending in `PERP`, e.g. `BTCGUSDPERP`).

### Get Open Positions

**Method:** POST
**Path:** NOT fully documented (listed as "Get Open Positions" under Derivatives section)
**Auth Required:** Yes

**NOTE:** Specific path and full response schema NOT DOCUMENTED in available API reference.

### Get Account Margin (Derivatives)

**Method:** POST
**Path:** NOT fully documented
**Auth Required:** Yes
Returns margin information for derivative account positions.

### List Funding Payments

**Method:** POST
**Path:** NOT fully documented
Returns historical funding payment records for perpetual positions.

### Get Funding Amount (Public)

**Method:** GET
**Path:** `/v1/fundingamount/{symbol}`
**Auth Required:** No
Returns: dollar amount for a Long 1 position held in the symbol for a 1-hour funding period (current and estimated).

### Get Risk Stats

**Method:** GET
**Path:** NOT fully documented
Returns risk-related statistics for derivative accounts.

### Set Leverage

NOT AVAILABLE — not documented in REST API.

### Change Margin Mode (Cross/Isolated)

NOT AVAILABLE — not documented in REST API.

### Add/Remove Margin

NOT AVAILABLE — not documented in REST API.

### Get Liquidation Price

NOT AVAILABLE — not documented in REST API.

---

## Margin Trading (Spot Margin)

Margin orders on spot pairs are placed via the standard `/v1/order/new` endpoint with `margin_order: true`.

### Get Margin Account Summary

**Method:** POST
**Path:** `/v1/margin/account/summary` (inferred from docs structure)
**Auth Required:** Yes — Trader or Auditor role
**Response fields:** NOT FULLY DOCUMENTED in available reference.

### Get Margin Interest Rates

**Method:** POST
**Path:** `/v1/margin/interest/rates` (inferred)
**Auth Required:** Yes
**Response fields:** NOT FULLY DOCUMENTED.

### Preview Margin Order Impact

**Method:** POST
**Path:** `/v1/margin/order/impact/preview` (inferred)
**Auth Required:** Yes
**Purpose:** Estimate the effect of a proposed trade on your margin account.
**Response fields:** NOT FULLY DOCUMENTED.

### Borrow / Repay

NOT AVAILABLE as separate endpoints — margin is managed implicitly through the `margin_order` flag on order placement.

### Get Max Borrowable

NOT AVAILABLE — not documented.

---

## Advanced Features

### Wrap Order

**Method:** POST
**Path:** `/v1/wrap/{symbol}`
**Auth Required:** Yes — Trader role
**Purpose:** Execute wrap/unwrap operations for wrapped tokens (e.g., WBTC).

#### Request Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `amount` | string | Yes | Amount to wrap/unwrap |
| `side` | string | Yes | `"buy"` (wrap) or `"sell"` (unwrap) |
| `client_order_id` | string | No | User-defined ID |
| `account` | string | No | Sub-account (Master key only) |

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `orderId` | string | Order ID |
| `pair` | string | Trading pair |
| `price` | string | Execution price |
| `side` | string | Buy or sell |
| `quantity` | string | Amount |
| `quantityCurrency` | string | Base currency |
| `totalSpend` | string | Total cost |
| `totalSpendCurrency` | string | Quote currency |
| `fee` | string | Fee charged |
| `feeCurrency` | string | Fee currency |
| `depositFee` | string | Network deposit fee |
| `depositFeeCurrency` | string | Deposit fee currency |

---

### TWAP Orders

NOT AVAILABLE.

### Iceberg Orders

NOT AVAILABLE.

### Copy Trading API

NOT AVAILABLE.

### Grid Trading API

NOT AVAILABLE.

---

## Base URLs

| Environment | URL |
|-------------|-----|
| Production | `https://api.gemini.com` |
| Sandbox | `https://api.sandbox.gemini.com` |

---

## Sources

- [Orders — REST API — Gemini Crypto Exchange](https://docs.gemini.com/rest/orders)
- [Market Data — REST API — Gemini Crypto Exchange](https://docs.gemini.com/rest-api/)
- [Gemini Crypto Exchange: Build on Gemini](https://docs.gemini.com/)
