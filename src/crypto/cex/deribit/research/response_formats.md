# Deribit Response Formats

Complete specification of JSON-RPC response structures for Deribit API.

## JSON-RPC 2.0 Protocol

All Deribit API responses follow JSON-RPC 2.0 specification with Deribit-specific extensions.

## Base Response Structure

### Successful Response

```json
{
  "jsonrpc": "2.0",
  "id": 42,
  "result": { /* Method-specific result data */ },
  "testnet": false,
  "usIn": 1535043730126248,
  "usOut": 1535043730126250,
  "usDiff": 2
}
```

**Standard Fields**:
- `jsonrpc`: string - Always `"2.0"` (required by spec)
- `id`: integer/string - Matches request ID (enables response correlation)
- `result`: object/array - Method-specific result data (present on success)

**Deribit Extensions** (not part of JSON-RPC standard):
- `testnet`: boolean - `true` if testnet environment, `false` if production
- `usIn`: integer - Server received request timestamp (microseconds)
- `usOut`: integer - Server sent response timestamp (microseconds)
- `usDiff`: integer - Server processing time (microseconds) = `usOut - usIn`

**Performance Monitoring**: Use `usDiff` to track server-side latency and optimize requests.

---

### Error Response

```json
{
  "jsonrpc": "2.0",
  "id": 8163,
  "error": {
    "code": 11050,
    "message": "bad_request",
    "data": {
      "reason": "additional context",
      "param": "invalid_parameter_name"
    }
  },
  "testnet": false,
  "usIn": 1535037392434763,
  "usOut": 1535037392448119,
  "usDiff": 13356
}
```

**Error Fields**:
- `error`: object - Present instead of `result` on error
  - `code`: integer - Error code (see Error Codes section)
  - `message`: string - Human-readable error message
  - `data`: object (optional) - Additional error context

**Note**: `result` and `error` are mutually exclusive (one is always present, never both).

---

## Common Error Codes

| Code  | Message | Description | Action |
|-------|---------|-------------|--------|
| 10000 | `authorization_required` | Missing authentication | Authenticate before calling private methods |
| 10001 | `error` | Generic error | Check `data` field for details |
| 10002 | `qty_too_low` | Order quantity too small | Increase order quantity |
| 10003 | `order_overlap` | Order would match own order | Adjust price or use post-only |
| 10004 | `order_not_found` | Order ID not found | Verify order ID |
| 10005 | `price_too_low` | Price below minimum | Increase price |
| 10006 | `price_too_low_for_this_option` | Option price too low | Increase price |
| 10007 | `price_too_high` | Price above maximum | Decrease price |
| 10008 | `locked_by_admin` | Account locked | Contact support |
| 10009 | `reg_temporary_banned` | Temporarily banned | Wait for ban to expire |
| 10010 | `trading_banned` | Trading not allowed | Check account status |
| 10011 | `not_enough_funds` | Insufficient balance | Add funds or reduce order size |
| 10012 | `already_closed` | Position already closed | Refresh position state |
| 10013 | `price_not_allowed` | Price not on tick size grid | Adjust price to tick size |
| 10014 | `book_closed` | Orderbook closed | Wait for market to open |
| 10015 | `pme_max_total_cost_exceeded` | Portfolio margin limit exceeded | Reduce position size |
| 10016 | `pme_max_open_orders_exceeded` | Too many open orders | Cancel some orders |
| 10017 | `pme_prohibited_instrument_pairing` | Invalid instrument combo | Check instrument compatibility |
| 10018 | `market_price_not_available` | No market price | Use limit order or wait |
| 10019 | `already_filled` | Order already executed | Refresh order state |
| 10020 | `withdrawal_not_allowed` | Withdrawal restricted | Check withdrawal settings |
| 10021 | `max_position_size_exceeded` | Position limit exceeded | Reduce order size |
| 10028 | `too_many_requests` | Rate limit exceeded | Implement backoff, wait for credits to refill |
| 10029 | `system_maintenance` | Maintenance mode | Retry after maintenance |
| 10030 | `subscription_not_found` | WebSocket subscription invalid | Re-subscribe |
| 10031 | `transfer_not_found` | Transfer ID not found | Verify transfer ID |
| 11029 | `invalid_arguments` | Invalid method parameters | Check parameter format/values |
| 11030 | `other_reject` | Reject (other reason) | Check `data` field |
| 11035 | `not_open_yet` | Market not open | Wait for market opening |
| 11036 | `already_open` | Market already open | N/A |
| 11040 | `price_missmatch` | Price mismatch | Adjust price |
| 11041 | `not_owner_of_order` | Order belongs to different user | Check order ownership |
| 11042 | `must_be_websocket_request` | WebSocket required | Use WebSocket connection |
| 11044 | `invalid_order_label` | Invalid order label | Check label format |
| 11050 | `bad_request` | Malformed request | Validate JSON-RPC format |
| 11051 | `system_is_busy` | System busy | Retry with backoff |
| 11052 | `invalid_addr` | Invalid address | Verify address format |
| 11053 | `restricted_country` | Geographic restriction | N/A |
| 11055 | `invalid_currency` | Unsupported currency | Use supported currency |
| 11056 | `invalid_currency_pair` | Invalid currency pair | Check available pairs |
| 11057 | `invalid_instrument_name` | Instrument not found | Verify instrument name |
| 11058 | `invalid_max_show_amount` | Invalid max_show value | Check min requirements |
| 11059 | `invalid_order_type` | Invalid order type | Use valid type (limit/market) |
| 11060 | `invalid_price` | Invalid price value | Check price constraints |
| 11061 | `invalid_quantity` | Invalid quantity value | Check quantity constraints |
| 11062 | `invalid_trigger_type` | Invalid trigger type | Use valid trigger (index/mark/last) |
| 13004 | `invalid_credentials` | Wrong API key/secret | Verify credentials |
| 13005 | `not_on_trading_facility` | Account not approved for trading | Contact support |
| 13006 | `already_logged_in` | Already authenticated | N/A |
| 13009 | `IP_address_not_whitelisted` | IP not allowed | Whitelist IP in settings |
| 13668 | `security_key_authorization_error` | 2FA/Security key error | Check `data.reason` field |

---

## Market Data Response Formats

### Instruments List (`public/get_instruments`)

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": [
    {
      "tick_size": 0.5,
      "tick_size_steps": [],
      "taker_commission": 0.0005,
      "settlement_period": "perpetual",
      "settlement_currency": "BTC",
      "quote_currency": "USD",
      "price_index": "btc_usd",
      "min_trade_amount": 10,
      "max_liquidation_commission": 0.0075,
      "max_leverage": 50,
      "maker_commission": 0.0,
      "kind": "future",
      "is_active": true,
      "instrument_name": "BTC-PERPETUAL",
      "instrument_id": 139,
      "instrument_type": "reversed",
      "expiration_timestamp": 32503708800000,
      "creation_timestamp": 1535107200000,
      "counter_currency": "USD",
      "contract_size": 10,
      "block_trade_tick_size": 0.5,
      "block_trade_min_trade_amount": 25000,
      "block_trade_commission": 0.00025,
      "base_currency": "BTC",
      "state": "open"
    }
  ]
}
```

**Key Fields**:
- `instrument_name`: string - Full instrument name (e.g., "BTC-PERPETUAL", "ETH-29DEC23-2000-C")
- `instrument_id`: integer - Unique instrument ID
- `kind`: string - `"future"`, `"option"`, `"spot"`, `"future_combo"`, `"option_combo"`
- `settlement_period`: string - `"perpetual"`, `"month"`, `"week"`, etc.
- `tick_size`: number - Minimum price increment
- `min_trade_amount`: number - Minimum order size (in USD for futures/perpetuals)
- `contract_size`: number - Contract multiplier
- `is_active`: boolean - Trading enabled
- `state`: string - `"open"`, `"closed"`, `"pre_open"`
- `expiration_timestamp`: integer - Expiry time in milliseconds (far future for perpetuals)

---

### Order Book (`public/get_order_book`)

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "timestamp": 1550757569415,
    "stats": {
      "volume_usd": 60234917.27,
      "volume": 15894.89,
      "price_change": -1.48,
      "low": 3744.5,
      "high": 3861.5
    },
    "state": "open",
    "settlement_price": 3803.23,
    "open_interest": 88234.5,
    "min_price": 3676.82,
    "max_price": 3932.44,
    "mark_price": 3804.63,
    "mark_iv": 72.34,
    "last_price": 3804.5,
    "interest_rate": 0.00,
    "instrument_name": "BTC-PERPETUAL",
    "index_price": 3802.76,
    "funding_8h": 0.00003852,
    "current_funding": 0.00001284,
    "change_id": 37580728,
    "bids": [
      ["new", 3804.5, 51960],
      ["new", 3804.0, 46850],
      ["new", 3803.5, 60140]
    ],
    "best_bid_price": 3804.5,
    "best_bid_amount": 51960,
    "best_ask_price": 3805.0,
    "best_ask_amount": 68750,
    "asks": [
      ["new", 3805.0, 68750],
      ["new", 3805.5, 41390],
      ["new", 3806.0, 25740]
    ]
  }
}
```

**Book Entry Format**: `[action, price, amount]`
- `action`: string - `"new"`, `"change"`, `"delete"`
- `price`: number - Price level
- `amount`: number - Total quantity at this price

**Key Fields**:
- `bids`: array - Buy side of orderbook (descending price)
- `asks`: array - Sell side of orderbook (ascending price)
- `best_bid_price` / `best_ask_price`: number - Top of book
- `mark_price`: number - Mark price for liquidations
- `index_price`: number - Underlying index price
- `open_interest`: number - Total open contracts
- `funding_8h`: number - 8-hour funding rate (for perpetuals)
- `change_id`: integer - Orderbook version (for delta updates)

---

### Ticker (`public/ticker`)

```json
{
  "jsonrpc": "2.0",
  "id": 8106,
  "result": {
    "timestamp": 1550757569415,
    "stats": {
      "volume_usd": 60234917.27,
      "volume": 15894.89,
      "price_change": -1.48,
      "low": 3744.5,
      "high": 3861.5
    },
    "state": "open",
    "settlement_price": 3803.23,
    "open_interest": 88234.5,
    "min_price": 3676.82,
    "max_price": 3932.44,
    "mark_price": 3804.63,
    "mark_iv": 72.34,
    "last_price": 3804.5,
    "interest_rate": 0.00,
    "instrument_name": "BTC-PERPETUAL",
    "index_price": 3802.76,
    "funding_8h": 0.00003852,
    "current_funding": 0.00001284,
    "best_bid_price": 3804.5,
    "best_bid_amount": 51960,
    "best_ask_price": 3805.0,
    "best_ask_amount": 68750
  }
}
```

**Futures/Perpetuals Fields**:
- `last_price`: number - Last traded price
- `mark_price`: number - Mark price (fair value)
- `index_price`: number - Spot index price
- `funding_8h`: number - 8h funding rate (annualized)
- `current_funding`: number - Current funding rate
- `volume`: number - 24h volume (contracts)
- `volume_usd`: number - 24h volume (USD)
- `open_interest`: number - Total open interest

**Options Fields** (additional):
- `mark_iv`: number - Mark implied volatility (%)
- `underlying_price`: number - Underlying asset price
- `greeks`: object - Delta, gamma, theta, vega, rho

---

### Recent Trades (`public/get_last_trades_by_instrument`)

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "trades": [
      {
        "trade_seq": 35684420,
        "trade_id": "48079254",
        "timestamp": 1590484512188,
        "tick_direction": 0,
        "price": 8950.5,
        "mark_price": 8948.03,
        "instrument_name": "BTC-PERPETUAL",
        "index_price": 8947.52,
        "direction": "sell",
        "amount": 40
      }
    ],
    "has_more": true
  }
}
```

**Trade Fields**:
- `trade_id`: string - Unique trade identifier
- `trade_seq`: integer - Sequence number (incremental)
- `timestamp`: integer - Trade time (milliseconds)
- `direction`: string - `"buy"` or `"sell"` (taker side)
- `price`: number - Trade price
- `amount`: number - Trade quantity
- `tick_direction`: integer - Price movement: `0` (zero plus tick), `1` (plus tick), `2` (minus tick), `3` (zero minus tick)
- `index_price`: number - Index at trade time
- `mark_price`: number - Mark price at trade time

---

## Trading Response Formats

### Place Order (`private/buy`, `private/sell`)

```json
{
  "jsonrpc": "2.0",
  "id": 5275,
  "result": {
    "trades": [
      {
        "trade_seq": 35684502,
        "trade_id": "48079338",
        "timestamp": 1590484645991,
        "tick_direction": 1,
        "state": "filled",
        "self_trade": false,
        "reduce_only": false,
        "price": 8952.5,
        "post_only": false,
        "order_type": "market",
        "order_id": "4008314325",
        "matching_id": null,
        "mark_price": 8948.79,
        "liquidity": "T",
        "iv": 0,
        "instrument_name": "BTC-PERPETUAL",
        "index_price": 8948.33,
        "fee_currency": "BTC",
        "fee": 0.00002244,
        "direction": "buy",
        "amount": 40
      }
    ],
    "order": {
      "web": true,
      "time_in_force": "good_til_cancelled",
      "replaced": false,
      "reduce_only": false,
      "profit_loss": 0.0,
      "price": 8952.5,
      "post_only": false,
      "order_type": "market",
      "order_state": "filled",
      "order_id": "4008314325",
      "max_show": 40,
      "last_update_timestamp": 1590484645991,
      "label": "",
      "is_liquidation": false,
      "instrument_name": "BTC-PERPETUAL",
      "filled_amount": 40,
      "direction": "buy",
      "creation_timestamp": 1590484645991,
      "commission": 0.00002244,
      "average_price": 8952.5,
      "api": true,
      "amount": 40
    }
  }
}
```

**Order Object Fields**:
- `order_id`: string - Unique order identifier
- `order_state`: string - `"open"`, `"filled"`, `"rejected"`, `"cancelled"`, `"untriggered"`
- `order_type`: string - `"limit"`, `"market"`, `"stop_limit"`, `"stop_market"`
- `instrument_name`: string - Instrument traded
- `direction`: string - `"buy"` or `"sell"`
- `amount`: number - Order size
- `filled_amount`: number - Executed quantity
- `price`: number - Order price (0 for market orders)
- `average_price`: number - Average fill price
- `time_in_force`: string - `"good_til_cancelled"`, `"good_til_day"`, `"immediate_or_cancel"`, `"fill_or_kill"`
- `label`: string - Custom order label
- `creation_timestamp`: integer - Order created time
- `last_update_timestamp`: integer - Last modification time
- `reduce_only`: boolean - Position-reducing only
- `post_only`: boolean - Maker-only order
- `commission`: number - Total fees paid

**Trades Array**: Immediate fills (if any) - same structure as user trades

---

### Cancel Order (`private/cancel`, `private/cancel_all_by_currency`)

**Single Cancel**:
```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "result": {
    "order_id": "4008314325",
    "order_state": "cancelled"
  }
}
```

**Mass Cancel (detailed=false)**:
```json
{
  "jsonrpc": "2.0",
  "id": 7,
  "result": 5
}
```
Returns the number of cancelled orders.

**Mass Cancel (detailed=true)**:
```json
{
  "jsonrpc": "2.0",
  "id": 7,
  "result": [
    {
      "order_id": "4008314325",
      "order_state": "cancelled",
      "instrument_name": "BTC-PERPETUAL"
    }
  ]
}
```
Returns execution reports for each cancelled order.

---

### Edit Order (`private/edit`)

```json
{
  "jsonrpc": "2.0",
  "id": 8,
  "result": {
    "order": {
      "order_id": "4008314325",
      "order_state": "open",
      "price": 9000.0,
      "amount": 50,
      "filled_amount": 0,
      "instrument_name": "BTC-PERPETUAL",
      "direction": "buy"
    },
    "trades": []
  }
}
```

---

## Account Response Formats

### Account Summary (`private/get_account_summary`)

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "result": {
    "total_pl": 0.0234,
    "session_upl": 0.0012,
    "session_rpl": 0.0,
    "projected_maintenance_margin": 0.1234,
    "projected_initial_margin": 0.2345,
    "projected_delta_total": 1.5,
    "portfolio_margining_enabled": false,
    "options_vega": 0.0,
    "options_theta": 0.0,
    "options_session_upl": 0.0,
    "options_session_rpl": 0.0,
    "options_pl": 0.0,
    "options_gamma": 0.0,
    "options_delta": 0.0,
    "margin_balance": 1.2345,
    "maintenance_margin": 0.1234,
    "limits": {
      "matching_engine": {
        "used": 0,
        "burst": 20,
        "rate": 20
      },
      "non_matching_engine": {
        "used": 2,
        "burst": 200,
        "rate": 200
      }
    },
    "initial_margin": 0.2345,
    "futures_session_upl": 0.0012,
    "futures_session_rpl": 0.0,
    "futures_pl": 0.0234,
    "fee_balance": 0.0,
    "equity": 1.2579,
    "delta_total": 1.5,
    "currency": "BTC",
    "balance": 1.2345,
    "available_withdrawal_funds": 1.0,
    "available_funds": 1.0111
  }
}
```

**Key Fields**:
- `currency`: string - Account currency
- `balance`: number - Total balance
- `equity`: number - Balance + unrealized P&L
- `available_funds`: number - Available for trading
- `available_withdrawal_funds`: number - Available for withdrawal
- `margin_balance`: number - Balance used for margin
- `initial_margin`: number - Initial margin requirement
- `maintenance_margin`: number - Maintenance margin requirement
- `total_pl`: number - Total profit/loss
- `session_upl`: number - Unrealized P&L this session
- `session_rpl`: number - Realized P&L this session
- `delta_total`: number - Total portfolio delta
- `limits`: object - Rate limit information
  - `matching_engine`: Order placement limits
  - `non_matching_engine`: Data query limits

---

### Positions (`private/get_positions`)

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "result": [
    {
      "total_profit_loss": 0.0012,
      "size_currency": 0.4472,
      "size": 40,
      "settlement_price": 8952.37,
      "realized_profit_loss": 0.0,
      "realized_funding": -0.00001234,
      "open_orders_margin": 0.0234,
      "mark_price": 8950.23,
      "maintenance_margin": 0.0112,
      "kind": "future",
      "instrument_name": "BTC-PERPETUAL",
      "initial_margin": 0.0223,
      "index_price": 8949.78,
      "floating_profit_loss": 0.0012,
      "estimated_liquidation_price": 7890.5,
      "direction": "buy",
      "delta": 0.4472,
      "average_price": 8945.0,
      "average_price_usd": 8945.0
    }
  ]
}
```

**Position Fields**:
- `instrument_name`: string - Position instrument
- `size`: number - Position size (contracts, positive for long, negative for short)
- `size_currency`: number - Size in base currency
- `direction`: string - `"buy"` (long), `"sell"` (short), `"zero"` (flat)
- `average_price`: number - Average entry price
- `mark_price`: number - Current mark price
- `index_price`: number - Current index price
- `settlement_price`: number - Daily settlement price
- `floating_profit_loss`: number - Unrealized P&L
- `realized_profit_loss`: number - Realized P&L
- `total_profit_loss`: number - Total P&L
- `initial_margin`: number - Initial margin used
- `maintenance_margin`: number - Maintenance margin required
- `estimated_liquidation_price`: number - Estimated liquidation price
- `delta`: number - Position delta (for options)

---

## WebSocket Subscription Notifications

### Notification Format

```json
{
  "jsonrpc": "2.0",
  "method": "subscription",
  "params": {
    "channel": "book.BTC-PERPETUAL.100ms",
    "data": { /* channel-specific data */ }
  }
}
```

**Note**: Notifications do NOT have an `id` field (they're not responses to requests).

### Book Channel (`book.{instrument}.{interval}`)

```json
{
  "jsonrpc": "2.0",
  "method": "subscription",
  "params": {
    "channel": "book.BTC-PERPETUAL.100ms",
    "data": {
      "type": "change",
      "timestamp": 1590484645991,
      "instrument_name": "BTC-PERPETUAL",
      "change_id": 37580800,
      "bids": [
        ["change", 8950.5, 12340],
        ["delete", 8950.0, 0]
      ],
      "asks": [
        ["new", 8951.0, 5670]
      ]
    }
  }
}
```

**Data Fields**:
- `type`: string - `"snapshot"` (full book) or `"change"` (delta update)
- `change_id`: integer - Orderbook version (incremental)
- `bids` / `asks`: array of `[action, price, amount]`

---

### Ticker Channel (`ticker.{instrument}.{interval}`)

```json
{
  "jsonrpc": "2.0",
  "method": "subscription",
  "params": {
    "channel": "ticker.BTC-PERPETUAL.100ms",
    "data": {
      "timestamp": 1590484645991,
      "stats": {
        "volume": 15894.89,
        "low": 3744.5,
        "high": 3861.5
      },
      "state": "open",
      "settlement_price": 3803.23,
      "open_interest": 88234.5,
      "min_price": 3676.82,
      "max_price": 3932.44,
      "mark_price": 3804.63,
      "last_price": 3804.5,
      "instrument_name": "BTC-PERPETUAL",
      "index_price": 3802.76,
      "funding_8h": 0.00003852,
      "current_funding": 0.00001284,
      "best_bid_price": 3804.5,
      "best_bid_amount": 51960,
      "best_ask_price": 3805.0,
      "best_ask_amount": 68750
    }
  }
}
```

---

### Trades Channel (`trades.{instrument}.{interval}`)

```json
{
  "jsonrpc": "2.0",
  "method": "subscription",
  "params": {
    "channel": "trades.BTC-PERPETUAL.100ms",
    "data": [
      {
        "trade_seq": 35684502,
        "trade_id": "48079338",
        "timestamp": 1590484645991,
        "tick_direction": 1,
        "price": 8952.5,
        "instrument_name": "BTC-PERPETUAL",
        "index_price": 8948.33,
        "direction": "buy",
        "amount": 40
      }
    ]
  }
}
```

---

### User Orders Channel (`user.orders.{kind}.{currency}.{interval}`)

```json
{
  "jsonrpc": "2.0",
  "method": "subscription",
  "params": {
    "channel": "user.orders.BTC-PERPETUAL.raw",
    "data": {
      "order_id": "4008314325",
      "order_state": "open",
      "instrument_name": "BTC-PERPETUAL",
      "direction": "buy",
      "amount": 40,
      "filled_amount": 0,
      "price": 8950.0
    }
  }
}
```

---

### User Trades Channel (`user.trades.{kind}.{currency}.{interval}`)

```json
{
  "jsonrpc": "2.0",
  "method": "subscription",
  "params": {
    "channel": "user.trades.BTC-PERPETUAL.raw",
    "data": [
      {
        "trade_id": "48079338",
        "order_id": "4008314325",
        "timestamp": 1590484645991,
        "instrument_name": "BTC-PERPETUAL",
        "direction": "buy",
        "price": 8952.5,
        "amount": 40,
        "fee": 0.00002244,
        "fee_currency": "BTC"
      }
    ]
  }
}
```

---

## Response Validation Checklist

When parsing responses:

- [ ] Verify `jsonrpc` field is `"2.0"`
- [ ] Match response `id` to request `id`
- [ ] Check for `error` field presence (error response)
- [ ] Parse `result` field (success response)
- [ ] Handle missing optional fields gracefully
- [ ] Validate numeric ranges (e.g., prices, amounts)
- [ ] Parse timestamps as milliseconds (not seconds)
- [ ] Handle `testnet` flag for environment awareness
- [ ] Monitor `usDiff` for performance issues
- [ ] Implement proper enum parsing for strings (order_state, direction, etc.)

---

## References

- Deribit API Documentation: https://docs.deribit.com/
- JSON-RPC 2.0 Specification: https://www.jsonrpc.org/specification
- JSON-RPC 2.0 Protocol Overview: https://docs.deribit.com/articles/json-rpc-overview
