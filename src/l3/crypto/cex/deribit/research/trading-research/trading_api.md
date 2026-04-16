# Deribit Trading API Specification

Source: https://docs.deribit.com/
API Version: v2
Protocol: JSON-RPC 2.0 over WebSocket (recommended) or HTTP REST

Production WebSocket: `wss://www.deribit.com/ws/api/v2`
Production HTTP: `https://www.deribit.com/api/v2/{method}`
Test WebSocket: `wss://test.deribit.com/ws/api/v2`
Test HTTP: `https://test.deribit.com/api/v2/{method}`

All methods use the format: `{scope}/{method_name}` (e.g., `private/buy`, `private/cancel`)

---

## Order Types Supported

From `private/buy` and `private/sell` documentation, the `type` parameter accepts:

| Order Type | Description |
|---|---|
| `limit` | Standard limit order (default) |
| `market` | Market order, executes at best available price |
| `stop_limit` | Stop-limit trigger order |
| `stop_market` | Stop-market trigger order |
| `take_limit` | Take-profit limit order |
| `take_market` | Take-profit market order |
| `market_limit` | Converts to limit at best bid/ask if not immediately filled |
| `trailing_stop` | Trailing stop order |

### Time-in-Force Options

The `time_in_force` parameter accepts:

| Value | Meaning |
|---|---|
| `good_til_cancelled` | GTC — default; stays open until filled or cancelled |
| `good_til_day` | GTD — expires at end of trading day |
| `fill_or_kill` | FOK — must fill entirely immediately or cancel |
| `immediate_or_cancel` | IOC — fills what it can immediately, cancels remainder |

### Post-Only Behaviour

- `post_only` (boolean, default: `true`) — Prevents immediate fill; order price adjusts to just below/above spread if needed
- `reject_post_only` (boolean, default: `false`) — Rejects order outright if post-only conditions cannot be met
- `post_only` only works with `time_in_force="good_til_cancelled"`

### Iceberg Orders

- `display_amount` (number, default: `1`) — Initial display quantity for iceberg orders; must be at least 100x the instrument minimum

---

## Order Placement

### Place Buy Order

**Method:** `private/buy`
**Scope required:** `trade:read_write`

#### Parameters

| Parameter | Type | Required | Default | Description |
|---|---|---|---|---|
| `instrument_name` | string | Yes | — | e.g., `"BTC-PERPETUAL"`, `"ETH-29MAR24-3000-C"` |
| `amount` | number | Cond. | — | Order size; USD for inverse perps/futures, base currency for linear/options. Either `amount` or `contracts` required |
| `contracts` | number | Cond. | — | Order size in contract units; must match `amount` if both provided |
| `type` | string | No | `"limit"` | Order type: limit, stop_limit, take_limit, market, stop_market, take_market, market_limit, trailing_stop |
| `label` | string | No | — | Custom label (max 64 characters) |
| `price` | number | No | — | Limit price in base currency. Required for limit/stop_limit |
| `time_in_force` | string | No | `"good_til_cancelled"` | good_til_cancelled, good_til_day, fill_or_kill, immediate_or_cancel |
| `display_amount` | number | No | `1` | Iceberg display quantity |
| `post_only` | boolean | No | `true` | Post-only flag |
| `reject_post_only` | boolean | No | `false` | Reject if post-only conditions not met |
| `reduce_only` | boolean | No | `false` | Only reduces existing position |
| `trigger_price` | number | No | — | Trigger price for stop/take orders |
| `trigger_offset` | number | No | — | Max deviation from price peak for trailing stop |
| `trigger` | string | No | — | Trigger source: `index_price`, `mark_price`, `last_price` |
| `advanced` | string | No | — | Options only: `"usd"` (price in USD) or `"implv"` (implied volatility %) |
| `mmp` | boolean | No | `false` | Market maker protection flag (limit orders only) |
| `valid_until` | integer | No | — | Server timestamp in ms; request rejected if processing exceeds this |
| `linked_order_type` | string | No | — | `one_triggers_other`, `one_cancels_other`, `one_triggers_one_cancels_other` |
| `trigger_fill_condition` | string | No | `"first_hit"` | `first_hit`, `complete_fill`, `incremental` |
| `otoco_config` | array | No | — | Array of secondary order config objects for OCO/OTOCO |

#### Response

```json
{
  "jsonrpc": "2.0",
  "id": 6130,
  "result": {
    "order": {
      "order_id": "ETH-584864807",
      "order_state": "filled",
      "order_type": "limit",
      "time_in_force": "good_til_cancelled",
      "instrument_name": "ETH-PERPETUAL",
      "creation_timestamp": 1590486335742,
      "last_update_timestamp": 1590486335742,
      "direction": "sell",
      "price": 198.75,
      "label": "",
      "post_only": false,
      "reject_post_only": false,
      "reduce_only": true,
      "api": true,
      "amount": 21,
      "filled_amount": 21,
      "average_price": 202.8
    },
    "trades": [
      {
        "trade_id": "ETH-2696097",
        "trade_seq": 1966068,
        "instrument_name": "ETH-PERPETUAL",
        "timestamp": 1590486335742,
        "order_type": "limit",
        "order_id": "ETH-584864807",
        "direction": "sell",
        "price": 202.8,
        "amount": 21,
        "fee": 0.00007766,
        "fee_currency": "ETH",
        "state": "filled",
        "liquidity": "T",
        "mark_price": 202.79,
        "index_price": 202.86,
        "tick_direction": 0
      }
    ]
  }
}
```

---

### Place Sell Order

**Method:** `private/sell`
**Scope required:** `trade:read_write`

Parameters are identical to `private/buy`:

| Parameter | Type | Required | Default | Description |
|---|---|---|---|---|
| `instrument_name` | string | Yes | — | Unique instrument identifier |
| `amount` | number | Cond. | — | Order size (either `amount` or `contracts` required) |
| `contracts` | number | Cond. | — | Order size in contract units |
| `type` | string | No | `"limit"` | limit, stop_limit, take_limit, market, stop_market, take_market, market_limit, trailing_stop |
| `price` | number | No | — | Limit price |
| `time_in_force` | string | No | `"good_til_cancelled"` | good_til_cancelled, good_til_day, fill_or_kill, immediate_or_cancel |
| `label` | string | No | — | Max 64 chars |
| `display_amount` | number | No | `1` | Iceberg quantity |
| `post_only` | boolean | No | `true` | Post-only |
| `reject_post_only` | boolean | No | `false` | Reject if post-only fails |
| `reduce_only` | boolean | No | `false` | Reduce-only flag |
| `trigger_price` | number | No | — | Trigger price for stop/take orders |
| `trigger_offset` | number | No | — | Max deviation for trailing stop |
| `trigger` | string | No | — | `index_price`, `mark_price`, `last_price` |
| `advanced` | string | No | — | Options only: `"usd"` or `"implv"` |
| `mmp` | boolean | No | `false` | Market maker protection |
| `valid_until` | integer | No | — | Expiry timestamp in ms |
| `linked_order_type` | string | No | — | OCO/OTOCO type |
| `trigger_fill_condition` | string | No | `"first_hit"` | `first_hit`, `complete_fill`, `incremental` |
| `otoco_config` | array | No | — | Secondary order objects |

---

### Batch/Bulk Order Placement

Deribit does NOT have a dedicated batch order endpoint for standard orders. Each order must be placed individually via `private/buy` or `private/sell`.

**Exception:** `private/mass_quote` exists for market makers to submit multiple option quotes simultaneously. This is a specialized endpoint for quoting, not general batch order placement.

---

### Conditional / Linked Orders (OCO, OTO, OTOCO)

Built into `private/buy` and `private/sell` via parameters:

| Parameter | Values | Description |
|---|---|---|
| `linked_order_type` | `one_triggers_other` | OTO: primary fill triggers secondary |
| `linked_order_type` | `one_cancels_other` | OCO: filling one cancels the other |
| `linked_order_type` | `one_triggers_one_cancels_other` | OTOCO: OTO + OCO bracket |
| `trigger_fill_condition` | `first_hit` | Trigger on first partial fill |
| `trigger_fill_condition` | `complete_fill` | Trigger only on complete fill |
| `trigger_fill_condition` | `incremental` | Trigger incrementally as filled |
| `otoco_config` | array of order objects | Secondary order configuration |

---

## Order Management

### Amend / Modify Order

**Method:** `private/edit`
**Scope required:** `trade:read_write`
**Note:** Only open orders can be edited.

#### Parameters

| Parameter | Type | Required | Description |
|---|---|---|---|
| `order_id` | string | Yes | Unique order identifier |
| `amount` | number | No | New order size |
| `contracts` | number | No | New size in contracts (must match `amount` if both provided) |
| `price` | number | No | New price |
| `post_only` | boolean | No | Post-only flag (default: true) |
| `reduce_only` | boolean | No | Reduce-only flag (default: false) |
| `reject_post_only` | boolean | No | Reject if post-only fails (default: false) |
| `advanced` | string | No | Options: `"usd"` or `"implv"` |
| `trigger_price` | number | No | New trigger price for stop/take orders |
| `trigger_offset` | number | No | New max deviation for trailing stop |
| `mmp` | boolean | No | Market maker protection (default: false) |
| `valid_until` | integer | No | Expiry server timestamp in ms |
| `display_amount` | number | No | Iceberg display quantity (default: 1) |

Response includes `order` object and `trades` array. The order object contains `replaced: true` boolean flag.

---

### Cancel Single Order

**Method:** `private/cancel`
**Scope required:** `trade:read_write`

#### Parameters

| Parameter | Type | Required | Description |
|---|---|---|---|
| `order_id` | string | Yes | Unique order identifier (e.g., `"ETH-100234"`) |

#### Response

Returns full order object with `order_state: "cancelled"` and `cancel_reason: "user_request"`.

```json
{
  "jsonrpc": "2.0",
  "id": 4214,
  "result": {
    "order_id": "ETH-SLIS-12",
    "order_state": "cancelled",
    "order_type": "stop_market",
    "direction": "sell",
    "amount": 5,
    "cancel_reason": "user_request"
  }
}
```

---

### Cancel All Orders

**Method:** `private/cancel_all`
**Scope required:** `trade:read_write`

Cancels ALL open orders across all currencies and instrument types.

#### Parameters

| Parameter | Type | Required | Default | Description |
|---|---|---|---|---|
| `detailed` | boolean | No | `false` | If true, returns list of all cancelled orders instead of count |
| `freeze_quotes` | boolean | No | `false` | Reject incoming quotes for 1 second after cancel (for market makers) |

#### Response

```json
{
  "jsonrpc": "2.0",
  "id": 47,
  "result": 4
}
```
`result` is the total number of successfully cancelled orders (integer).

---

### Cancel All by Currency

**Method:** `private/cancel_all_by_currency`
**Scope required:** `trade:read_write`

#### Parameters

| Parameter | Type | Required | Description |
|---|---|---|---|
| `currency` | string | Yes | Currency: BTC, ETH, USDC, USDT, EURR |
| `kind` | string | No | Instrument kind: future, option, spot, future_combo, option_combo, combo, any |
| `type` | string | No | Order type: all, limit, stop, take, trigger_all, trailing_stop |
| `detailed` | boolean | No | If true, returns list of cancelled orders |
| `freeze_quotes` | boolean | No | Reject incoming quotes for 1 sec after (default: false) |

Response: `result` integer (count of cancelled orders).

---

### Cancel All by Instrument

**Method:** `private/cancel_all_by_instrument`
(Inferred from API structure; cancels all orders for a specific instrument)

---

### Cancel by Label

**Method:** `private/cancel_by_label`
Cancels all orders with a specific label string.

---

### Get Single Order State

**Method:** `private/get_order_state`
**Scope required:** `trade:read`

#### Parameters

| Parameter | Type | Required | Description |
|---|---|---|---|
| `order_id` | string | Yes | Order identifier (e.g., `"ETH-100234"`) |

#### Response Fields

| Field | Type | Description |
|---|---|---|
| `order_id` | string | Unique order identifier |
| `order_state` | enum | open, filled, rejected, cancelled, untriggered, triggered |
| `order_type` | enum | limit, market, stop_market, stop_limit, take_market, take_limit, trailing_stop |
| `time_in_force` | enum | good_til_cancelled, good_til_day, fill_or_kill, immediate_or_cancel |
| `instrument_name` | string | Instrument identifier |
| `creation_timestamp` | integer | Unix ms |
| `last_update_timestamp` | integer | Unix ms |
| `direction` | enum | buy or sell |
| `price` | number/string | Price or `"market_price"` |
| `amount` | number | Requested order size |
| `filled_amount` | number | Executed portion |
| `average_price` | number | Mean fill price |
| `api` | boolean | true if placed via API |
| `label` | string | Custom label (if set) |
| `post_only` | boolean | Post-only flag |
| `reduce_only` | boolean | Reduce-only flag |
| `trigger_price` | number | Trigger price (for stop/take orders) |
| `contracts` | number | Size in contract units |

---

### Get Open Orders

**Method:** `private/get_open_orders`
**Scope required:** `trade:read`

#### Parameters

| Parameter | Type | Required | Allowed Values | Description |
|---|---|---|---|---|
| `kind` | string | No | future, option, spot, future_combo, option_combo | Filter by instrument kind |
| `type` | string | No | all, limit, trigger_all, stop_all, stop_limit, stop_market, take_all, take_limit, take_market, trailing_all, trailing_stop | Filter by order type (default: all) |

Returns array of order objects.

---

### Get Open Orders by Instrument

**Method:** `private/get_open_orders_by_instrument`
Filter open orders by specific instrument name.

---

### Get Order History

**Method:** `private/get_order_history_by_instrument`
**Scope required:** `trade:read`

#### Parameters

| Parameter | Type | Required | Default | Description |
|---|---|---|---|---|
| `instrument_name` | string | Yes | — | Instrument identifier |
| `count` | integer | No | 20 | Max items per page (max: 1000) |
| `offset` | integer | No | 0 | Pagination offset |
| `include_old` | boolean | No | `false` | Include orders older than 2 days |
| `include_unfilled` | boolean | No | `false` | Include fully unfilled closed orders |
| `with_continuation` | boolean | No | — | Returns object with continuation token for paging |
| `continuation` | string | No | — | Continuation token from previous response |
| `historical` | boolean | No | `false` | false = recent (30min/24h), true = full history (with indexing delay) |

Response: array of order objects with same fields as `get_order_state`.

Also available: `private/get_order_history_by_currency` (same params but `currency` instead of `instrument_name`).

---

### Get Trade History

**Method:** `private/get_user_trades_by_instrument`
**Scope required:** `trade:read`

#### Parameters

| Parameter | Type | Required | Default | Description |
|---|---|---|---|---|
| `instrument_name` | string | Yes | — | Instrument identifier |
| `start_seq` | integer | No | — | First trade sequence number |
| `end_seq` | integer | No | — | Last trade sequence number |
| `count` | integer | No | 10 | Items per page (max: 1000) |
| `start_timestamp` | integer | No | — | Earliest timestamp in ms |
| `end_timestamp` | integer | No | — | Latest timestamp in ms |
| `historical` | boolean | No | `false` | false = recent 24h; true = full history |
| `sorting` | string | No | — | `asc`, `desc`, or `default` |

#### Trade Response Fields

| Field | Type | Description |
|---|---|---|
| `trade_id` | string | Unique trade identifier |
| `trade_seq` | integer | Trade sequence number |
| `instrument_name` | string | Instrument |
| `timestamp` | integer | Unix ms |
| `order_id` | string | Associated order |
| `order_type` | enum | limit, market, liquidation |
| `direction` | enum | buy or sell |
| `price` | number | Fill price |
| `amount` | number | Fill quantity |
| `fee` | number | Fee amount |
| `fee_currency` | string | Currency of fee |
| `state` | enum | filled |
| `liquidity` | string | `"M"` = maker, `"T"` = taker |
| `mark_price` | number | Mark price at fill time |
| `index_price` | number | Index price at fill time |
| `iv` | number | Implied volatility (options only) |
| `underlying_price` | number | Underlying price (options only) |
| `profit_loss` | number | P&L from this trade |
| `block_trade_id` | string | Block trade ID (if block trade) |
| `has_more` | boolean | More records exist |

Also available:
- `private/get_user_trades_by_currency` — by currency
- `private/get_user_trades_by_order` — by order ID
- `private/get_user_trades_by_instrument_and_time` — time-range query

---

## Position Management

### Get Single Position

**Method:** `private/get_position`
**Scope required:** `trade:read`

#### Parameters

| Parameter | Type | Required | Description |
|---|---|---|---|
| `instrument_name` | string | Yes | Instrument identifier |

#### Response Fields

| Field | Type | Description |
|---|---|---|
| `instrument_name` | string | Instrument identifier |
| `kind` | enum | future, option, spot, future_combo, option_combo |
| `direction` | enum | buy, sell, or zero |
| `size` | number | Position size in quote currency (USD for inverse futures) |
| `size_currency` | number | Position size in base currency (futures only) |
| `average_price` | number | Average price of trades building this position |
| `mark_price` | number | Current mark price |
| `index_price` | number | Current index price |
| `initial_margin` | number | Initial margin required |
| `maintenance_margin` | number | Maintenance margin |
| `estimated_liquidation_price` | number | Estimated liquidation price (futures only) |
| `open_orders_margin` | number | Margin reserved for open orders |
| `leverage` | integer | Current available leverage (futures only) |
| `total_profit_loss` | number | Total P&L from position |
| `floating_profit_loss` | number | Unrealized P&L |
| `realized_profit_loss` | number | Realized P&L |
| `delta` | number | Delta |
| `gamma` | number | Gamma (options only) |
| `vega` | number | Vega (options only) |
| `theta` | number | Theta (options only) |
| `settlement_price` | number | Last settlement price |
| `realized_funding` | number | Realized funding (perpetuals only) |
| `interest_value` | number | Value for funding calculation |
| `average_price_usd` | number | Average price in USD (options only) |
| `floating_profit_loss_usd` | number | Floating P&L in USD (options only) |

---

### Get All Positions

**Method:** `private/get_positions`
**Scope required:** `trade:read`

Parameters include `currency` (required: BTC, ETH, USDC, etc.) and optional `kind` filter (future, option, spot, future_combo, option_combo).
Returns array of position objects with same fields as `private/get_position`.

---

### Close Position

**Method:** `private/close_position`
**Scope required:** `trade:read_write`

#### Parameters

| Parameter | Type | Required | Description |
|---|---|---|---|
| `instrument_name` | string | Yes | Instrument to close |
| `type` | string | Yes | Order type: `"limit"` or `"market"` |
| `price` | number | Cond. | Required if `type="limit"` |

Response: `PrivateBuyAndSellResponse` with `order` and `trades` array. The resulting order has `reduce_only: true`.

---

### Set Leverage

Leverage is set implicitly through margin allocation. Deribit does not expose a dedicated set-leverage endpoint in the v2 API — leverage is controlled through position margin management.

---

### Change Margin Mode

**Method:** Cross-collateral is account-level and configured in account settings, not via a per-instrument API call. Deribit uses a portfolio margin system.

---

### Add / Remove Margin

For futures positions, margin adjustment is managed through the overall account balance. No dedicated per-position margin add/remove endpoint exists in v2 API (unlike some other exchanges).

---

### Get Funding Rate

**Method:** `public/get_funding_rate_value`
Parameters: `instrument_name`, `start_timestamp`, `end_timestamp`.

**Method:** `public/get_funding_rate_history`
Parameters: `instrument_name`, `start_timestamp`, `end_timestamp`.

---

## Advanced Features

### Algo Orders (Iceberg)

Supported via `display_amount` parameter in `private/buy` / `private/sell`:
- `display_amount` sets the visible quantity
- Must be at least 100x the instrument minimum
- Only the `display_amount` is shown in the order book at any time

No dedicated TWAP algorithm is documented in the public API.

---

### Options-Specific Trading

Options use `advanced` parameter for price specification:

| `advanced` value | Meaning |
|---|---|
| `"usd"` | Price specified in USD |
| `"implv"` | Price specified as implied volatility percentage |

Greeks are available in position responses: `delta`, `gamma`, `vega`, `theta`.

Instruments format: `{CURRENCY}-{EXPIRY}-{STRIKE}-{C/P}` e.g., `BTC-29MAR24-50000-C`

---

### Block Trades

Block trades are executed outside the order book with a counterparty.

**Endpoints:**
- `private/verify_block_trade` — Step 1: verify proposed block trade
- `private/execute_block_trade` — Step 2: execute agreed block trade
- `private/get_block_trade` — Get specific block trade details
- `private/get_last_block_trades_by_currency` — Recent block trades

**Required scope:** `block_trade:read_write`

---

### Block RFQ (Request for Quote)

- `private/create_block_rfq` — Initiate an RFQ
- `private/accept_block_rfq` — Accept a quote
- `private/reject_block_rfq` — Reject a quote
- `private/cancel_block_rfq` — Cancel an RFQ

**Required scope:** `block_rfq:read_write`

---

### Market Maker Mass Quoting

- `private/mass_quote` — Submit multiple option quotes simultaneously
- `private/cancel_quotes` — Cancel quotes

`freeze_quotes` parameter in cancel endpoints blocks new incoming quotes for 1 second, useful for market maker quote management.

---

## Order States

| State | Description |
|---|---|
| `open` | Active in the order book |
| `filled` | Completely executed |
| `rejected` | Rejected by exchange |
| `cancelled` | Cancelled by user or system |
| `untriggered` | Stop/take order waiting for trigger |
| `triggered` | Trigger condition met, order submitted to book |

---

## Sources

- [Deribit API Documentation](https://docs.deribit.com/)
- [private/buy reference](https://docs.deribit.com/api-reference/trading/private-buy)
- [private/sell reference](https://docs.deribit.com/api-reference/trading/private-sell)
- [private/edit reference](https://docs.deribit.com/api-reference/trading/private-edit)
- [private/cancel reference](https://docs.deribit.com/api-reference/trading/private-cancel)
- [private/cancel_all reference](https://docs.deribit.com/api-reference/trading/private-cancel_all)
- [private/cancel_all_by_currency reference](https://docs.deribit.com/api-reference/trading/private-cancel_all_by_currency)
- [private/get_open_orders reference](https://docs.deribit.com/api-reference/trading/private-get_open_orders)
- [private/get_order_state reference](https://docs.deribit.com/api-reference/trading/private-get_order_state)
- [private/get_order_history_by_instrument reference](https://docs.deribit.com/api-reference/trading/private-get_order_history_by_instrument)
- [private/get_user_trades_by_instrument reference](https://docs.deribit.com/api-reference/trading/private-get_user_trades_by_instrument)
- [private/close_position reference](https://docs.deribit.com/api-reference/trading/private-close_position)
- [private/get_position reference](https://docs.deribit.com/api-reference/account-management/private-get_position)
- [JSON-RPC Overview](https://docs.deribit.com/articles/json-rpc-overview)
