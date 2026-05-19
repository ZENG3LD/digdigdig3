# Gemini Exchange API Response Formats

Comprehensive response format documentation for implementing V5 connector parser.rs module.

---

## General Response Structure

### Success Responses

Most endpoints return JSON objects or arrays directly:

```json
{
  "field1": "value1",
  "field2": 123
}
```

Or array of objects:

```json
[
  {"field1": "value1"},
  {"field2": "value2"}
]
```

### Error Responses

All errors follow this structure:

```json
{
  "result": "error",
  "reason": "InvalidNonce",
  "message": "Nonce must be increasing with each request"
}
```

**Error Fields**:
- `result`: Always "error" for failures
- `reason`: Error code (e.g., "InvalidSignature", "InsufficientFunds")
- `message`: Human-readable description

---

## MarketData Responses

### List Symbols

**Endpoint**: `GET /v1/symbols`

```json
["btcusd", "ethusd", "ethbtc", "bchusd", "ltcusd", "zecusd", "btcgusdperp"]
```

**Format**: Array of symbol strings
- All lowercase
- No separator between base and quote
- Perpetuals end with "perp" or "gusdperp"

### Symbol Details

**Endpoint**: `GET /v1/symbols/details/{symbol}`

```json
{
  "symbol": "BTCUSD",
  "base_currency": "BTC",
  "quote_currency": "USD",
  "tick_size": 1e-8,
  "quote_increment": 0.01,
  "min_order_size": "0.00001",
  "status": "open",
  "wrap_enabled": false,
  "product_type": "cfd"
}
```

**Fields**:
- `symbol` (string): Trading pair in uppercase
- `base_currency` (string): Base currency code
- `quote_currency` (string): Quote currency code
- `tick_size` (number): Minimum price increment for base
- `quote_increment` (number): Minimum price increment for quote
- `min_order_size` (string): Minimum order size in base currency
- `status` (string): "open", "closed", "cancel_only", "post_only", "limit_only"
- `wrap_enabled` (boolean): Whether wrapped trading is enabled
- `product_type` (string, optional): "cfd", "future", etc.

### Ticker V1

**Endpoint**: `GET /v1/pubticker/{symbol}`

```json
{
  "bid": "50000.00",
  "ask": "50001.00",
  "last": "50000.50",
  "volume": {
    "BTC": "1234.56789",
    "USD": "61728000.00",
    "timestamp": 1640000000000
  }
}
```

**Fields**:
- `bid` (string): Best bid price
- `ask` (string): Best ask price
- `last` (string): Last trade price
- `volume` (object):
  - `{BASE}` (string): 24h volume in base currency
  - `{QUOTE}` (string): 24h volume in quote currency
  - `timestamp` (number): Volume calculation timestamp (ms)

### Ticker V2

**Endpoint**: `GET /v2/ticker/{symbol}`

```json
{
  "symbol": "BTCUSD",
  "open": "49500.00",
  "high": "51000.00",
  "low": "49000.00",
  "close": "50000.00",
  "changes": ["50100.00", "49900.00", "50050.00"],
  "bid": "50000.00",
  "ask": "50001.00"
}
```

**Fields**:
- `symbol` (string): Trading pair
- `open` (string): 24h opening price
- `high` (string): 24h high
- `low` (string): 24h low
- `close` (string): Current/closing price
- `changes` (array): Array of recent price changes
- `bid` (string): Best bid
- `ask` (string): Best ask

### Order Book

**Endpoint**: `GET /v1/book/{symbol}`

```json
{
  "bids": [
    {
      "price": "50000.00",
      "amount": "1.5",
      "timestamp": "1640000000"
    },
    {
      "price": "49999.00",
      "amount": "2.3",
      "timestamp": "1640000001"
    }
  ],
  "asks": [
    {
      "price": "50001.00",
      "amount": "0.8",
      "timestamp": "1640000002"
    },
    {
      "price": "50002.00",
      "amount": "1.2",
      "timestamp": "1640000003"
    }
  ]
}
```

**Fields**:
- `bids` (array): Buy orders, sorted by price descending
  - `price` (string): Bid price
  - `amount` (string): Quantity available at this price
  - `timestamp` (string): Unix timestamp (seconds)
- `asks` (array): Sell orders, sorted by price ascending
  - Same structure as bids

### Recent Trades

**Endpoint**: `GET /v1/trades/{symbol}`

```json
[
  {
    "timestamp": 1640000000,
    "timestampms": 1640000000000,
    "tid": 123456789,
    "price": "50000.00",
    "amount": "0.5",
    "exchange": "gemini",
    "type": "buy",
    "broken": false
  },
  {
    "timestamp": 1640000001,
    "timestampms": 1640000001000,
    "tid": 123456790,
    "price": "50001.00",
    "amount": "0.3",
    "exchange": "gemini",
    "type": "sell"
  }
]
```

**Fields**:
- `timestamp` (number): Unix timestamp (seconds)
- `timestampms` (number): Unix timestamp (milliseconds)
- `tid` (number): Trade ID
- `price` (string): Trade price
- `amount` (string): Trade quantity
- `exchange` (string): Always "gemini"
- `type` (string): "buy" or "sell" (taker side)
- `broken` (boolean, optional): True if auction break occurred

### Candles / OHLCV

**Endpoint**: `GET /v2/candles/{symbol}/{time_frame}`

```json
[
  [1640000000000, 49500, 51000, 49000, 50000, 1234.56789],
  [1640003600000, 50000, 50500, 49800, 50200, 987.65432],
  [1640007200000, 50200, 50800, 50100, 50500, 1100.12345]
]
```

**Format**: Array of arrays `[timestamp, open, high, low, close, volume]`
- `[0]` (number): Timestamp (milliseconds)
- `[1]` (number): Open price
- `[2]` (number): High price
- `[3]` (number): Low price
- `[4]` (number): Close price
- `[5]` (number): Volume in base currency

**Note**: Numbers are not strings in candle responses

### Funding Amount

**Endpoint**: `GET /v1/fundingamount/{symbol}`

```json
{
  "symbol": "BTCGUSDPERP",
  "funding_time": 1640003600,
  "next_funding_time": 1640007200,
  "funding_amount": "0.00123456",
  "timestamp": 1640000000
}
```

**Fields**:
- `symbol` (string): Perpetual symbol
- `funding_time` (number): Current funding epoch (Unix seconds)
- `next_funding_time` (number): Next funding epoch
- `funding_amount` (string): Estimated funding for 1 unit long position (USD)
- `timestamp` (number): Response timestamp

---

## Trading Responses

### New Order

**Endpoint**: `POST /v1/order/new`

```json
{
  "order_id": "987654321",
  "id": "987654321",
  "symbol": "btcusd",
  "exchange": "gemini",
  "avg_execution_price": "50000.25",
  "side": "buy",
  "type": "exchange limit",
  "timestamp": "1640000000",
  "timestampms": 1640000000000,
  "is_live": true,
  "is_cancelled": false,
  "is_hidden": false,
  "was_forced": false,
  "executed_amount": "0.3",
  "remaining_amount": "0.2",
  "original_amount": "0.5",
  "price": "50000.00",
  "options": ["maker-or-cancel"],
  "client_order_id": "my-order-123",
  "stop_price": null
}
```

**Fields**:
- `order_id` (string): Gemini order ID
- `id` (string): Same as order_id
- `symbol` (string): Trading pair (lowercase)
- `exchange` (string): Always "gemini"
- `avg_execution_price` (string): Average fill price (0.00 if not filled)
- `side` (string): "buy" or "sell"
- `type` (string): "exchange limit", "exchange market", "exchange stop limit"
- `timestamp` (string): Order creation time (Unix seconds)
- `timestampms` (number): Order creation time (Unix milliseconds)
- `is_live` (boolean): True if order is active
- `is_cancelled` (boolean): True if order was cancelled
- `is_hidden` (boolean): True for hidden orders
- `was_forced` (boolean): True if order was forced
- `executed_amount` (string): Filled quantity
- `remaining_amount` (string): Unfilled quantity
- `original_amount` (string): Original order quantity
- `price` (string): Limit price
- `options` (array): Order options/flags
- `client_order_id` (string): Custom order ID
- `stop_price` (string|null): Stop trigger price (for stop-limit orders)

### Order Status

**Endpoint**: `POST /v1/order/status`

Same format as New Order, with optional `trades` array if `include_trades: true`:

```json
{
  "order_id": "987654321",
  "symbol": "btcusd",
  "side": "buy",
  "type": "exchange limit",
  "is_live": false,
  "is_cancelled": false,
  "executed_amount": "0.5",
  "remaining_amount": "0",
  "original_amount": "0.5",
  "avg_execution_price": "50000.25",
  "trades": [
    {
      "price": "50000.00",
      "amount": "0.3",
      "timestamp": 1640000000,
      "timestampms": 1640000000000,
      "type": "Buy",
      "aggressor": true,
      "fee_currency": "USD",
      "fee_amount": "15.00",
      "tid": 123456789,
      "order_id": "987654321",
      "exchange": "gemini",
      "is_auction_fill": false
    },
    {
      "price": "50001.00",
      "amount": "0.2",
      "timestamp": 1640000001,
      "timestampms": 1640000001000,
      "type": "Buy",
      "aggressor": true,
      "fee_currency": "USD",
      "fee_amount": "10.00",
      "tid": 123456790,
      "order_id": "987654321",
      "exchange": "gemini",
      "is_auction_fill": false
    }
  ]
}
```

### Active Orders

**Endpoint**: `POST /v1/orders`

```json
[
  {
    "order_id": "987654321",
    "symbol": "btcusd",
    "side": "buy",
    "type": "exchange limit",
    "is_live": true,
    "is_cancelled": false,
    "price": "50000.00",
    "remaining_amount": "0.5",
    "executed_amount": "0",
    "original_amount": "0.5"
  },
  {
    "order_id": "987654322",
    "symbol": "ethusd",
    "side": "sell",
    "type": "exchange limit",
    "is_live": true,
    "is_cancelled": false,
    "price": "3000.00",
    "remaining_amount": "2.0",
    "executed_amount": "0",
    "original_amount": "2.0"
  }
]
```

**Format**: Array of order objects (same structure as New Order)

### Cancel Order / Cancel All

**Endpoint**: `POST /v1/order/cancel` or `/v1/order/cancel/all`

Single cancel returns the cancelled order:
```json
{
  "order_id": "987654321",
  "is_cancelled": true,
  "is_live": false,
  "remaining_amount": "0",
  "executed_amount": "0.2",
  "original_amount": "0.5"
}
```

Cancel all returns summary:
```json
{
  "result": "ok",
  "details": {
    "cancelledOrders": [987654321, 987654322, 987654323],
    "cancelRejects": []
  }
}
```

### Past Trades

**Endpoint**: `POST /v1/mytrades`

```json
[
  {
    "price": "50000.00",
    "amount": "0.5",
    "timestamp": 1640000000,
    "timestampms": 1640000000000,
    "type": "Buy",
    "aggressor": true,
    "fee_currency": "USD",
    "fee_amount": "25.00",
    "tid": 123456789,
    "order_id": "987654321",
    "exchange": "gemini",
    "is_auction_fill": false,
    "client_order_id": "my-order-123",
    "symbol": "btcusd",
    "break": null
  }
]
```

**Fields**:
- `price` (string): Trade execution price
- `amount` (string): Trade quantity
- `timestamp` (number): Trade time (Unix seconds)
- `timestampms` (number): Trade time (Unix milliseconds)
- `type` (string): "Buy" or "Sell" (capitalized)
- `aggressor` (boolean): True if you were the taker
- `fee_currency` (string): Fee currency
- `fee_amount` (string): Fee paid
- `tid` (number): Trade ID
- `order_id` (string): Associated order ID
- `exchange` (string): Always "gemini"
- `is_auction_fill` (boolean): True if filled in auction
- `client_order_id` (string): Custom order ID
- `symbol` (string): Trading pair
- `break` (string|null): Auction break info

### Notional Volume

**Endpoint**: `POST /v1/notionalvolume`

```json
{
  "web_maker_fee_bps": 25,
  "web_taker_fee_bps": 35,
  "web_auction_fee_bps": 25,
  "api_maker_fee_bps": 10,
  "api_taker_fee_bps": 35,
  "api_auction_fee_bps": 20,
  "block_maker_fee_bps": 5,
  "block_taker_fee_bps": 30,
  "notional_30d_volume": 1000000.00,
  "last_updated_ms": 1640000000000,
  "date": "2023-12-20",
  "notional_1d_volume": [
    {
      "date": "2023-12-20",
      "notional_volume": 50000.00
    }
  ]
}
```

**Fields**:
- `*_maker_fee_bps` (number): Maker fee in basis points
- `*_taker_fee_bps` (number): Taker fee in basis points
- `*_auction_fee_bps` (number): Auction fee in basis points
- `notional_30d_volume` (number): 30-day trading volume (USD)
- `last_updated_ms` (number): Last update timestamp
- `date` (string): Date of volume calculation
- `notional_1d_volume` (array): Daily volume breakdown

---

## Account Responses

### Balances

**Endpoint**: `POST /v1/balances`

```json
[
  {
    "type": "exchange",
    "currency": "BTC",
    "amount": "1.50000000",
    "available": "1.00000000",
    "availableForWithdrawal": "0.90000000",
    "pendingWithdrawal": "0.10000000",
    "pendingDeposit": "0.50000000"
  },
  {
    "type": "exchange",
    "currency": "USD",
    "amount": "10000.00",
    "available": "9500.00",
    "availableForWithdrawal": "9500.00",
    "pendingWithdrawal": "0.00",
    "pendingDeposit": "500.00"
  }
]
```

**Fields**:
- `type` (string): Account type ("exchange", "custody")
- `currency` (string): Currency code
- `amount` (string): Total balance
- `available` (string): Available for trading
- `availableForWithdrawal` (string): Available for withdrawal
- `pendingWithdrawal` (string): Pending withdrawal amount
- `pendingDeposit` (string): Pending deposit amount (if `showPendingBalances: true`)

### Notional Balances

**Endpoint**: `POST /v1/notionalbalances/{currency}`

```json
[
  {
    "currency": "BTC",
    "amount": "1.50000000",
    "amountNotional": "75000.00",
    "available": "1.00000000",
    "availableNotional": "50000.00",
    "availableForWithdrawal": "0.90000000",
    "availableForWithdrawalNotional": "45000.00"
  },
  {
    "currency": "ETH",
    "amount": "10.00000000",
    "amountNotional": "30000.00",
    "available": "8.00000000",
    "availableNotional": "24000.00",
    "availableForWithdrawal": "7.50000000",
    "availableForWithdrawalNotional": "22500.00"
  }
]
```

**Fields**: Same as regular balances, plus:
- `amountNotional` (string): Total balance in quote currency
- `availableNotional` (string): Available balance in quote currency
- `availableForWithdrawalNotional` (string): Withdrawable in quote currency

### Deposit Addresses

**Endpoint**: `POST /v1/addresses/{network}`

```json
[
  {
    "address": "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh",
    "timestamp": 1640000000000,
    "label": "Primary deposit address",
    "memo": null,
    "network": "bitcoin"
  },
  {
    "address": "0x1234567890abcdef1234567890abcdef12345678",
    "timestamp": 1640001000000,
    "label": "Trading deposits",
    "memo": null,
    "network": "ethereum"
  }
]
```

**Fields**:
- `address` (string): Deposit address
- `timestamp` (number): Creation timestamp (ms)
- `label` (string): User-defined label
- `memo` (string|null): Memo/tag for currencies requiring it
- `network` (string): Network name

### New Deposit Address

**Endpoint**: `POST /v1/deposit/{network}/newAddress`

```json
{
  "address": "bc1q...",
  "timestamp": 1640000000000,
  "label": "New trading address",
  "memo": null,
  "network": "bitcoin"
}
```

Same structure as single address from list.

### Withdraw

**Endpoint**: `POST /v1/withdraw/{currency}`

```json
{
  "address": "bc1q...",
  "amount": "0.50000000",
  "fee": "0.0001",
  "withdrawalId": "abc123def456",
  "message": "Withdrawal request submitted"
}
```

**Fields**:
- `address` (string): Destination address
- `amount` (string): Withdrawal amount
- `fee` (string): Network fee
- `withdrawalId` (string): Unique withdrawal ID
- `message` (string): Status message

### Transfers

**Endpoint**: `POST /v1/transfers`

```json
[
  {
    "type": "Deposit",
    "status": "Complete",
    "timestampms": 1640000000000,
    "eid": 123456789,
    "currency": "BTC",
    "amount": "1.00000000",
    "txHash": "0xabc123...",
    "destination": "bc1q...",
    "purpose": "Transfer"
  },
  {
    "type": "Withdrawal",
    "status": "Complete",
    "timestampms": 1640001000000,
    "eid": 123456790,
    "currency": "BTC",
    "amount": "0.50000000",
    "txHash": "0xdef456...",
    "destination": "bc1q...",
    "purpose": "Withdrawal"
  }
]
```

**Fields**:
- `type` (string): "Deposit" or "Withdrawal"
- `status` (string): "Complete", "Pending", "Failed"
- `timestampms` (number): Transaction timestamp (ms)
- `eid` (number): Event ID
- `currency` (string): Currency code
- `amount` (string): Transfer amount
- `txHash` (string): Blockchain transaction hash
- `destination` (string): Destination address
- `purpose` (string): Transfer purpose

### Account Transfer

**Endpoint**: `POST /v1/account/transfer/{currency}`

```json
{
  "fromAccount": "primary",
  "toAccount": "trading",
  "amount": "0.50000000",
  "fee": "0.00000000",
  "currency": "BTC",
  "withdrawalId": "abc123",
  "message": "Transfer successful",
  "txHash": null
}
```

**Fields**:
- `fromAccount` (string): Source account name
- `toAccount` (string): Destination account name
- `amount` (string): Transfer amount
- `fee` (string): Transfer fee (usually 0 for internal)
- `currency` (string): Currency code
- `withdrawalId` (string): Transfer ID
- `message` (string): Status message
- `txHash` (string|null): Transaction hash (null for internal)

---

## Positions Responses

### Open Positions

**Endpoint**: `POST /v1/positions`

```json
[
  {
    "symbol": "BTCGUSDPERP",
    "instrument_type": "perp",
    "quantity": "2.50000000",
    "notional_value": "125000.00",
    "realised_pnl": "1250.00",
    "unrealised_pnl": "500.00",
    "average_cost": "49000.00",
    "mark_price": "50200.00"
  },
  {
    "symbol": "ETHGUSDPERP",
    "instrument_type": "perp",
    "quantity": "-5.00000000",
    "notional_value": "-15000.00",
    "realised_pnl": "-200.00",
    "unrealised_pnl": "150.00",
    "average_cost": "3100.00",
    "mark_price": "2970.00"
  }
]
```

**Fields**:
- `symbol` (string): Perpetual symbol
- `instrument_type` (string): "perp" for perpetuals, "spot" for spot
- `quantity` (string): Position size (negative for shorts)
- `notional_value` (string): USD value of position (quantity * mark_price)
- `realised_pnl` (string): Realized profit/loss
- `unrealised_pnl` (string): Unrealized profit/loss
- `average_cost` (string): Average entry price
- `mark_price` (string): Current mark price

### Account Margin

**Endpoint**: `POST /v1/margin`

```json
{
  "margin_assets_value": "10000.00",
  "initial_margin": "2500.00",
  "available_margin": "7500.00",
  "margin_maintenance_limit": "1250.00",
  "leverage": "4.0",
  "estimated_liquidation_price": "37500.00",
  "initial_margin_positions": "2000.00",
  "reserved_margin_buys": "250.00",
  "reserved_margin_sells": "250.00",
  "buying_power": "30000.00",
  "selling_power": "30000.00"
}
```

**Fields**:
- `margin_assets_value` (string): Total margin collateral value
- `initial_margin` (string): Initial margin requirement
- `available_margin` (string): Available margin (assets - requirement)
- `margin_maintenance_limit` (string): Maintenance margin (liquidation trigger)
- `leverage` (string): Current leverage ratio
- `estimated_liquidation_price` (string): Approximate liquidation price
- `initial_margin_positions` (string): Margin used by positions
- `reserved_margin_buys` (string): Margin reserved for buy orders
- `reserved_margin_sells` (string): Margin reserved for sell orders
- `buying_power` (string): Max buying power at current leverage
- `selling_power` (string): Max selling power at current leverage

### Margin Interest Rates

**Endpoint**: `POST /v1/margin/rates`

```json
[
  {
    "currency": "BTC",
    "borrowRate": "0.000004167",
    "borrowRateDaily": "0.0001",
    "borrowRateAnnual": "0.0365",
    "lastUpdated": 1640000000000
  },
  {
    "currency": "USD",
    "borrowRate": "0.000002778",
    "borrowRateDaily": "0.00006667",
    "borrowRateAnnual": "0.0244",
    "lastUpdated": 1640000000000
  }
]
```

**Fields**:
- `currency` (string): Currency code
- `borrowRate` (string): Hourly interest rate
- `borrowRateDaily` (string): Daily interest rate
- `borrowRateAnnual` (string): Annual interest rate
- `lastUpdated` (number): Last rate update (ms)

### Funding Payments

**Endpoint**: `POST /v1/perpetuals/fundingPayment`

```json
[
  {
    "eventType": "Hourly Funding Transfer",
    "timestamp": 1640003600,
    "assetCode": "USD",
    "action": "Credit",
    "quantity": {
      "currency": "USD",
      "value": "1.23456789"
    },
    "instrumentSymbol": "BTCGUSDPERP"
  },
  {
    "eventType": "Hourly Funding Transfer",
    "timestamp": 1640007200,
    "assetCode": "USD",
    "action": "Debit",
    "quantity": {
      "currency": "USD",
      "value": "0.98765432"
    },
    "instrumentSymbol": "ETHGUSDPERP"
  }
]
```

**Fields**:
- `eventType` (string): Always "Hourly Funding Transfer"
- `timestamp` (number): Funding time (Unix seconds)
- `assetCode` (string): Settlement currency
- `action` (string): "Credit" (received) or "Debit" (paid)
- `quantity` (object):
  - `currency` (string): Currency code
  - `value` (string): Funding amount
- `instrumentSymbol` (string): Perpetual symbol (since April 16, 2024)

### Risk Stats

**Endpoint**: `GET /v1/riskstats/{symbol}`

```json
{
  "product_type": "PerpetualSwapContract",
  "mark_price": "50200.00",
  "index_price": "50205.00",
  "open_interest": "12345.67890000",
  "open_interest_notional": "620000000.00",
  "timestamp": 1640000000000
}
```

**Fields**:
- `product_type` (string): "PerpetualSwapContract"
- `mark_price` (string): Current mark price
- `index_price` (string): Current index price
- `open_interest` (string): Total open interest (quantity)
- `open_interest_notional` (string): Total open interest (USD value)
- `timestamp` (number, optional): Data timestamp

---

## WebSocket Responses

### Market Data V2 WebSocket

**Connection**: `wss://api.gemini.com/v2/marketdata`

#### Subscription Acknowledgment

```json
{
  "type": "subscribed",
  "subscriptions": [
    {
      "name": "l2",
      "symbols": ["BTCUSD", "ETHUSD"]
    }
  ]
}
```

#### L2 Order Book Updates

```json
{
  "type": "l2_updates",
  "symbol": "BTCUSD",
  "changes": [
    ["buy", "50000.00", "1.5"],
    ["sell", "50001.00", "0.0"],
    ["buy", "49999.00", "2.3"]
  ],
  "trades": [],
  "auction_events": []
}
```

**Changes Format**: `[side, price, quantity]`
- side: "buy" or "sell"
- price: Price level (string)
- quantity: New quantity at this level (string) - "0.0" means removed

#### Trade Updates

```json
{
  "type": "trade",
  "symbol": "BTCUSD",
  "event_id": 123456789,
  "timestamp": 1640000000000,
  "price": "50000.00",
  "quantity": "0.5",
  "side": "buy"
}
```

#### Candle Updates

```json
{
  "type": "candles_1m_updates",
  "symbol": "BTCUSD",
  "changes": [
    [1640000000000, 49500, 50100, 49400, 50000, 123.45]
  ]
}
```

### Order Events WebSocket

**Connection**: `wss://api.gemini.com/v1/order/events`

#### Subscription Acknowledgment

```json
{
  "type": "subscription_ack",
  "accountId": 123456,
  "subscriptionId": "abc-def-ghi-jkl",
  "symbolFilter": [],
  "apiSessionFilter": [],
  "eventTypeFilter": []
}
```

#### Heartbeat

```json
{
  "type": "heartbeat",
  "timestampms": 1640000000000,
  "sequence": 12345,
  "socket_sequence": 67890,
  "trace_id": "xyz123"
}
```

#### Initial Orders

```json
{
  "type": "initial",
  "order_id": "987654321",
  "account_name": "primary",
  "api_session": "session-abc",
  "symbol": "btcusd",
  "side": "buy",
  "order_type": "exchange limit",
  "timestamp": "1640000000",
  "timestampms": 1640000000000,
  "is_live": true,
  "is_cancelled": false,
  "is_hidden": false,
  "executed_amount": "0",
  "remaining_amount": "0.5",
  "original_amount": "0.5",
  "price": "50000.00",
  "socket_sequence": 1
}
```

#### Order Accepted

```json
{
  "type": "accepted",
  "order_id": "987654321",
  "account_name": "primary",
  "symbol": "btcusd",
  "side": "buy",
  "order_type": "exchange limit",
  "timestampms": 1640000000000,
  "is_live": true,
  "is_cancelled": false,
  "price": "50000.00",
  "original_amount": "0.5",
  "remaining_amount": "0.5",
  "executed_amount": "0",
  "socket_sequence": 2
}
```

#### Order Fill

```json
{
  "type": "fill",
  "order_id": "987654321",
  "account_name": "primary",
  "symbol": "btcusd",
  "side": "buy",
  "order_type": "exchange limit",
  "timestampms": 1640000001000,
  "is_live": false,
  "is_cancelled": false,
  "executed_amount": "0.5",
  "remaining_amount": "0",
  "original_amount": "0.5",
  "price": "50000.00",
  "fill": {
    "trade_id": "123456789",
    "liquidity": "Maker",
    "price": "50000.00",
    "amount": "0.5",
    "fee": "25.00",
    "fee_currency": "USD"
  },
  "socket_sequence": 3
}
```

**Fill Object**:
- `trade_id` (string): Trade ID
- `liquidity` (string): "Maker" or "Taker"
- `price` (string): Fill price
- `amount` (string): Fill quantity
- `fee` (string): Fee paid
- `fee_currency` (string): Fee currency

#### Order Cancelled

```json
{
  "type": "cancelled",
  "order_id": "987654321",
  "account_name": "primary",
  "symbol": "btcusd",
  "side": "buy",
  "order_type": "exchange limit",
  "timestampms": 1640000002000,
  "is_live": false,
  "is_cancelled": true,
  "executed_amount": "0.3",
  "remaining_amount": "0",
  "original_amount": "0.5",
  "price": "50000.00",
  "reason": "Requested",
  "socket_sequence": 4
}
```

#### Order Rejected

```json
{
  "type": "rejected",
  "order_id": "987654321",
  "account_name": "primary",
  "symbol": "btcusd",
  "side": "buy",
  "order_type": "exchange limit",
  "timestampms": 1640000000000,
  "is_live": false,
  "is_cancelled": false,
  "original_amount": "0.5",
  "price": "50000.00",
  "reason": "InsufficientFunds",
  "socket_sequence": 5
}
```

---

## Data Type Summary

| Field Pattern | Type | Notes |
|---------------|------|-------|
| `*_amount` | string | Preserve precision |
| `price`, `*_price` | string | Preserve precision |
| `timestamp` | number | Unix seconds |
| `timestampms`, `*_ms` | number | Unix milliseconds |
| `*_id` | string/number | Can be either |
| `is_*` | boolean | Status flags |
| `type` | string | Discriminator |
| `side` | string | "buy"/"sell" lowercase (except trades: "Buy"/"Sell") |
| Candle values | number | Not strings |
| `volume` | string/number | Context-dependent |

---

## Parser Implementation Notes

### Key Considerations

1. **Precision**: Use `String` for all price/amount fields, parse to `Decimal` type
2. **Timestamps**: Handle both seconds and milliseconds
3. **Optional Fields**: Many fields can be null or missing
4. **Type Variations**: Some IDs are strings, some are numbers
5. **Capitalization**: "Buy"/"Sell" in trades vs "buy"/"sell" in orders
6. **Arrays vs Objects**: Some endpoints return arrays, others objects
7. **Error Detection**: Check for `result: "error"` field
8. **WebSocket Events**: Use `type` field to discriminate event types

### Suggested Rust Structures

```rust
#[derive(Deserialize)]
#[serde(untagged)]
enum ApiResponse<T> {
    Success(T),
    Error(ApiError),
}

#[derive(Deserialize)]
struct ApiError {
    result: String, // "error"
    reason: String,
    message: String,
}
```

---

## References

- Market Data: https://docs.gemini.com/rest/market-data
- Orders: https://docs.gemini.com/rest/orders
- Fund Management: https://docs.gemini.com/rest/fund-management
- Derivatives: https://docs.gemini.com/rest/derivatives
- WebSocket Market Data: https://docs.gemini.com/websocket/market-data/v2/about
- WebSocket Order Events: https://docs.gemini.com/websocket/order-events/event-types
