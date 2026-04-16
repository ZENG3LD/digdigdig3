# Gemini Exchange API Endpoints

Complete endpoint reference for V5 connector implementation covering all traits: MarketData, Trading, Account, and Positions.

## Base URLs

- **Production**: `https://api.gemini.com`
- **Sandbox**: `https://api.sandbox.gemini.com`

## Authentication

All private endpoints require three headers:
- `X-GEMINI-APIKEY`: Your API key
- `X-GEMINI-PAYLOAD`: Base64-encoded JSON payload
- `X-GEMINI-SIGNATURE`: HMAC-SHA384 hex signature

Additional headers:
- `Content-Type: text/plain`
- `Content-Length: 0`
- `Cache-Control: no-cache`

---

## MarketData Trait Endpoints

### Get Symbols / List Symbols
```
GET /v1/symbols
```
**Description**: Retrieve all available trading pairs
**Authentication**: None (public)
**Parameters**: None
**Response**: Array of symbol strings
```json
["btcusd", "ethusd", "ethbtc", ...]
```

### Get Symbol Details
```
GET /v1/symbols/details/{symbol}
```
**Description**: Detailed information about a specific trading pair
**Authentication**: None (public)
**Path Parameters**:
- `symbol` (string, required): Trading pair (e.g., "btcusd")

**Response**:
```json
{
  "symbol": "BTCUSD",
  "base_currency": "BTC",
  "quote_currency": "USD",
  "tick_size": 1e-8,
  "quote_increment": 0.01,
  "min_order_size": "0.00001",
  "status": "open",
  "wrap_enabled": false
}
```

### Get Ticker (V1)
```
GET /v1/pubticker/{symbol}
```
**Description**: 24-hour ticker data
**Authentication**: None (public)
**Path Parameters**:
- `symbol` (string, required): Trading pair

**Response**:
```json
{
  "bid": "50000.00",
  "ask": "50001.00",
  "last": "50000.50",
  "volume": {
    "BTC": "1234.56",
    "USD": "61728000.00",
    "timestamp": 1640000000000
  }
}
```

### Get Ticker V2
```
GET /v2/ticker/{symbol}
```
**Description**: Enhanced ticker with more fields
**Authentication**: None (public)
**Path Parameters**:
- `symbol` (string, required): Trading pair

**Response**:
```json
{
  "symbol": "BTCUSD",
  "open": "49500.00",
  "high": "51000.00",
  "low": "49000.00",
  "close": "50000.00",
  "changes": ["50100.00", "49900.00", ...],
  "bid": "50000.00",
  "ask": "50001.00"
}
```

### Get Order Book
```
GET /v1/book/{symbol}
```
**Description**: Current order book state
**Authentication**: None (public)
**Path Parameters**:
- `symbol` (string, required): Trading pair

**Query Parameters**:
- `limit_bids` (integer, optional): Limit number of bids (default: 50)
- `limit_asks` (integer, optional): Limit number of asks (default: 50)

**Response**:
```json
{
  "bids": [
    {
      "price": "50000.00",
      "amount": "1.5",
      "timestamp": "1640000000"
    }
  ],
  "asks": [
    {
      "price": "50001.00",
      "amount": "2.0",
      "timestamp": "1640000001"
    }
  ]
}
```

### Get Recent Trades
```
GET /v1/trades/{symbol}
```
**Description**: Recent trade history (last 50 trades by default)
**Authentication**: None (public)
**Path Parameters**:
- `symbol` (string, required): Trading pair

**Query Parameters**:
- `timestamp` (integer, optional): Only trades after this timestamp
- `since_tid` (integer, optional): Only trades after this trade ID
- `limit_trades` (integer, optional): Maximum number of trades (default: 50)
- `include_breaks` (boolean, optional): Include auction break info

**Response**:
```json
[
  {
    "timestamp": 1640000000,
    "timestampms": 1640000000000,
    "tid": 123456789,
    "price": "50000.00",
    "amount": "0.5",
    "exchange": "gemini",
    "type": "buy"
  }
]
```

### Get Candles / OHLCV
```
GET /v2/candles/{symbol}/{time_frame}
```
**Description**: Candlestick / OHLCV data
**Authentication**: None (public)
**Path Parameters**:
- `symbol` (string, required): Trading pair
- `time_frame` (string, required): One of: `1m`, `5m`, `15m`, `30m`, `1hr`, `6hr`, `1day`

**Response**: Array of candles [timestamp, open, high, low, close, volume]
```json
[
  [1640000000000, 49500, 51000, 49000, 50000, 1234.56],
  [1640003600000, 50000, 50500, 49800, 50200, 987.65]
]
```

### Get Derivative Candles
```
GET /v2/derivatives/candles/{symbol}/{time_frame}
```
**Description**: Candles for perpetual futures
**Authentication**: None (public)
**Path Parameters**: Same as regular candles
**Response**: Same format as regular candles

### Get Price Feed
```
GET /v1/pricefeed
```
**Description**: Price feed for all symbols
**Authentication**: None (public)
**Response**: Array of price objects for all trading pairs

### Get Network Info
```
GET /v1/network/{token}
```
**Description**: Network information for a token
**Authentication**: None (public)
**Path Parameters**:
- `token` (string, required): Token symbol (e.g., "btc")

### Get Funding Amount
```
GET /v1/fundingamount/{symbol}
```
**Description**: Current and next funding rates for perpetuals
**Authentication**: None (public)
**Path Parameters**:
- `symbol` (string, required): Perpetual symbol (e.g., "btcgusdperp")

**Response**:
```json
{
  "symbol": "BTCGUSDPERP",
  "funding_time": 1640003600,
  "next_funding_time": 1640007200,
  "funding_amount": "0.00123"
}
```

### Get Fee Promos
```
GET /v1/feepromos
```
**Description**: Available fee promotions
**Authentication**: None (public)

---

## Trading Trait Endpoints

### Create New Order
```
POST /v1/order/new
```
**Description**: Place a new order (limit, market, or stop-limit)
**Authentication**: Required (Trader role)
**OAuth Scope**: `orders:create`

**Request Body** (JSON in payload):
```json
{
  "request": "/v1/order/new",
  "nonce": 1640000000000,
  "symbol": "btcusd",
  "amount": "0.5",
  "price": "50000.00",
  "side": "buy",
  "type": "exchange limit",
  "options": ["maker-or-cancel"],
  "stop_price": "49000.00",
  "client_order_id": "my-order-123",
  "margin_order": false,
  "account": "primary"
}
```

**Parameters**:
- `symbol` (string, required): Trading pair
- `amount` (string, required): Order quantity in base currency
- `price` (string, required for limit orders): Limit price
- `side` (string, required): "buy" or "sell"
- `type` (string, required): "exchange limit", "exchange market", "exchange stop limit"
- `options` (array, optional): ["maker-or-cancel", "immediate-or-cancel", "fill-or-kill", "auction-only"]
- `stop_price` (string, optional): Trigger price for stop-limit orders
- `client_order_id` (string, optional): Custom order ID
- `margin_order` (boolean, optional): Use margin/leverage (default: false)
- `account` (string, optional): Subaccount name

**Response**:
```json
{
  "order_id": "987654321",
  "id": "987654321",
  "symbol": "btcusd",
  "exchange": "gemini",
  "avg_execution_price": "0.00",
  "side": "buy",
  "type": "exchange limit",
  "timestamp": "1640000000",
  "timestampms": 1640000000000,
  "is_live": true,
  "is_cancelled": false,
  "is_hidden": false,
  "was_forced": false,
  "executed_amount": "0",
  "remaining_amount": "0.5",
  "original_amount": "0.5",
  "price": "50000.00",
  "options": ["maker-or-cancel"],
  "client_order_id": "my-order-123"
}
```

### Cancel Order
```
POST /v1/order/cancel
```
**Description**: Cancel a specific order
**Authentication**: Required (Trader role)
**OAuth Scope**: `orders:create`

**Request Body**:
```json
{
  "request": "/v1/order/cancel",
  "nonce": 1640000001000,
  "order_id": 987654321,
  "account": "primary"
}
```

**Parameters**:
- `order_id` (integer, required): Order ID to cancel
- `account` (string, optional): Subaccount name

**Response**: Same format as order status

### Cancel All Active Orders
```
POST /v1/order/cancel/all
```
**Description**: Cancel all outstanding orders across all sessions
**Authentication**: Required (Trader role)

**Request Body**:
```json
{
  "request": "/v1/order/cancel/all",
  "nonce": 1640000002000,
  "account": "primary"
}
```

**Response**:
```json
{
  "result": "ok",
  "details": {
    "cancelledOrders": [987654321, 987654322],
    "cancelRejects": []
  }
}
```

### Cancel All Session Orders
```
POST /v1/order/cancel/session
```
**Description**: Cancel orders from current API session only
**Authentication**: Required (Trader role)

**Request Body**: Same as cancel all
**Response**: Same as cancel all

### Get Order Status
```
POST /v1/order/status
```
**Description**: Get status of a specific order
**Authentication**: Required (Trader, Auditor, or Fund Manager role)
**OAuth Scope**: `orders:read`

**Request Body**:
```json
{
  "request": "/v1/order/status",
  "nonce": 1640000003000,
  "order_id": 987654321,
  "include_trades": true,
  "account": "primary"
}
```

**Parameters**:
- `order_id` (integer, required): Order ID
- `client_order_id` (string, alternative): Use custom order ID instead
- `include_trades` (boolean, optional): Include trade executions
- `account` (string, optional): Subaccount name

**Response**: Same format as new order response, with optional trades array

### Get Active Orders
```
POST /v1/orders
```
**Description**: List all currently active orders
**Authentication**: Required (Trader, Auditor, or Fund Manager role)
**OAuth Scope**: `orders:read`

**Request Body**:
```json
{
  "request": "/v1/orders",
  "nonce": 1640000004000,
  "account": "primary"
}
```

**Response**: Array of order objects

### Get Past Trades
```
POST /v1/mytrades
```
**Description**: Retrieve your past trade executions
**Authentication**: Required (Trader, Auditor, or Fund Manager role)
**OAuth Scope**: `orders:read`

**Request Body**:
```json
{
  "request": "/v1/mytrades",
  "nonce": 1640000005000,
  "symbol": "btcusd",
  "limit_trades": 500,
  "timestamp": 1640000000,
  "account": "primary"
}
```

**Parameters**:
- `symbol` (string, required): Trading pair
- `limit_trades` (integer, optional): Max trades to return (default: 500)
- `timestamp` (integer, optional): Only trades after this timestamp
- `account` (string, optional): Subaccount name

**Response**:
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
    "client_order_id": "my-order-123"
  }
]
```

### Get Trading Volume
```
POST /v1/tradevolume
```
**Description**: Get your 30-day trading volume
**Authentication**: Required
**OAuth Scope**: `orders:read`

**Request Body**:
```json
{
  "request": "/v1/tradevolume",
  "nonce": 1640000006000,
  "account": "primary"
}
```

**Response**: Array of volume by symbol

### Get Notional Trading Volume
```
POST /v1/notionalvolume
```
**Description**: Get notional volume and fee information
**Authentication**: Required
**OAuth Scope**: `balances:read`

**Request Body**:
```json
{
  "request": "/v1/notionalvolume",
  "nonce": 1640000007000,
  "account": "primary"
}
```

**Response**:
```json
{
  "web_maker_fee_bps": 25,
  "web_taker_fee_bps": 35,
  "web_auction_fee_bps": 25,
  "api_maker_fee_bps": 10,
  "api_taker_fee_bps": 35,
  "api_auction_fee_bps": 20,
  "notional_30d_volume": 1000000.00,
  "last_updated_ms": 1640000000000
}
```

### Wrap Order
```
POST /v1/wrap/{symbol}
```
**Description**: Execute wrapped token orders
**Authentication**: Required (Trader role)

**Path Parameters**:
- `symbol` (string, required): Wrapped token pair

---

## Account Trait Endpoints

### Get Available Balances
```
POST /v1/balances
```
**Description**: Get all account balances
**Authentication**: Required (Trader, Fund Manager, or Auditor role)
**OAuth Scope**: `balances:read`

**Request Body**:
```json
{
  "request": "/v1/balances",
  "nonce": 1640000008000,
  "account": "primary",
  "showPendingBalances": true
}
```

**Parameters**:
- `account` (string, optional): Subaccount name
- `showPendingBalances` (boolean, optional): Include pending deposits/withdrawals

**Response**:
```json
[
  {
    "type": "exchange",
    "currency": "BTC",
    "amount": "1.5",
    "available": "1.0",
    "availableForWithdrawal": "0.9",
    "pendingWithdrawal": "0.1",
    "pendingDeposit": "0.5"
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

### Get Notional Balances
```
POST /v1/notionalbalances/{currency}
```
**Description**: Get balances with notional value in specified currency
**Authentication**: Required (Trader, Fund Manager, or Auditor role)
**OAuth Scope**: `balances:read`

**Path Parameters**:
- `currency` (string, required): Quote currency (e.g., "usd")

**Request Body**:
```json
{
  "request": "/v1/notionalbalances/usd",
  "nonce": 1640000009000,
  "account": "primary"
}
```

**Response**:
```json
[
  {
    "currency": "BTC",
    "amount": "1.5",
    "amountNotional": "75000.00",
    "available": "1.0",
    "availableNotional": "50000.00",
    "availableForWithdrawal": "0.9",
    "availableForWithdrawalNotional": "45000.00"
  }
]
```

### Get Staking Balances
```
POST /v1/balances/staking
```
**Description**: Get staking balances
**Authentication**: Required
**OAuth Scope**: `balances:read`

**Request Body**:
```json
{
  "request": "/v1/balances/staking",
  "nonce": 1640000010000,
  "account": "primary"
}
```

### List Deposit Addresses
```
POST /v1/addresses/{network}
```
**Description**: Get all deposit addresses for a network
**Authentication**: Required
**OAuth Scope**: `addresses:read`

**Path Parameters**:
- `network` (string, required): Network name (e.g., "bitcoin", "ethereum")

**Request Body**:
```json
{
  "request": "/v1/addresses/bitcoin",
  "nonce": 1640000011000,
  "account": "primary",
  "timestamp": 1640000000
}
```

**Response**:
```json
[
  {
    "address": "bc1q...",
    "timestamp": 1640000000000,
    "label": "My deposit address",
    "memo": null,
    "network": "bitcoin"
  }
]
```

### Create New Deposit Address
```
POST /v1/deposit/{network}/newAddress
```
**Description**: Generate a new deposit address
**Authentication**: Required
**OAuth Scope**: `addresses:create`

**Path Parameters**:
- `network` (string, required): Network name

**Request Body**:
```json
{
  "request": "/v1/deposit/bitcoin/newAddress",
  "nonce": 1640000012000,
  "label": "Trading bot deposits",
  "legacy": false,
  "account": "primary"
}
```

**Parameters**:
- `label` (string, optional): Address label
- `legacy` (boolean, optional): Use legacy address format
- `account` (string, optional): Subaccount name

**Response**:
```json
{
  "address": "bc1q...",
  "timestamp": 1640000012000,
  "label": "Trading bot deposits",
  "memo": null,
  "network": "bitcoin"
}
```

### Withdraw Crypto Funds
```
POST /v1/withdraw/{currency}
```
**Description**: Withdraw cryptocurrency
**Authentication**: Required (Fund Manager role)
**OAuth Scope**: `payments:send_crypto`

**Path Parameters**:
- `currency` (string, required): Currency code (lowercase)

**Request Body**:
```json
{
  "request": "/v1/withdraw/btc",
  "nonce": 1640000013000,
  "address": "bc1q...",
  "amount": "0.5",
  "client_transfer_id": "withdrawal-123",
  "account": "primary",
  "memo": "optional-memo"
}
```

**Parameters**:
- `address` (string, required): Destination address
- `amount` (string, required): Withdrawal amount
- `client_transfer_id` (string, optional): Custom withdrawal ID
- `account` (string, optional): Subaccount name
- `memo` (string, optional): Memo/tag for currencies that require it

**Response**:
```json
{
  "address": "bc1q...",
  "amount": "0.5",
  "fee": "0.0001",
  "withdrawalId": "abc123",
  "message": "Withdrawal request submitted"
}
```

### Get Gas Fee Estimation
```
POST /v1/withdraw/{currencyCodeLowerCase}/feeEstimate
```
**Description**: Estimate withdrawal fee
**Authentication**: Required
**OAuth Scope**: `payments:send_crypto`

**Request Body**:
```json
{
  "request": "/v1/withdraw/btc/feeEstimate",
  "nonce": 1640000014000,
  "address": "bc1q...",
  "amount": "0.5",
  "account": "primary"
}
```

**Response**:
```json
{
  "currency": "BTC",
  "fee": "0.0001",
  "isOverride": false,
  "monthlyLimit": "100.0",
  "monthlyRemaining": "99.5"
}
```

### List Past Transfers
```
POST /v1/transfers
```
**Description**: Get deposit and withdrawal history
**Authentication**: Required
**OAuth Scope**: `history:read`

**Request Body**:
```json
{
  "request": "/v1/transfers",
  "nonce": 1640000015000,
  "currency": "btc",
  "timestamp": 1640000000,
  "limit_transfers": 50,
  "account": "primary",
  "show_completed_deposit_advances": false
}
```

**Parameters**:
- `currency` (string, optional): Filter by currency
- `timestamp` (integer, optional): Only transfers after this time
- `limit_transfers` (integer, optional): Max results
- `account` (string, optional): Subaccount name
- `show_completed_deposit_advances` (boolean, optional)

**Response**:
```json
[
  {
    "type": "Deposit",
    "status": "Complete",
    "timestampms": 1640000000000,
    "eid": 123456789,
    "currency": "BTC",
    "amount": "1.0",
    "txHash": "0x..."
  }
]
```

### Transfer Between Accounts
```
POST /v1/account/transfer/{currency}
```
**Description**: Internal transfer between subaccounts
**Authentication**: Required (Fund Manager role)
**OAuth Scope**: `payments:create`

**Path Parameters**:
- `currency` (string, required): Currency code

**Request Body**:
```json
{
  "request": "/v1/account/transfer/btc",
  "nonce": 1640000016000,
  "sourceAccount": "primary",
  "targetAccount": "trading",
  "amount": "0.5",
  "clientTransferId": "transfer-123",
  "withdrawalId": "optional-id"
}
```

**Response**:
```json
{
  "fromAccount": "primary",
  "toAccount": "trading",
  "amount": "0.5",
  "fee": "0.0",
  "currency": "BTC",
  "withdrawalId": "abc123",
  "message": "Transfer successful",
  "txHash": null
}
```

### Get Transaction History
```
POST /v1/transactions
```
**Description**: Complete transaction history (trades and transfers)
**Authentication**: Required
**OAuth Scope**: `history:read`

**Request Body**:
```json
{
  "request": "/v1/transactions",
  "nonce": 1640000017000,
  "timestamp_nanos": 1640000000000000000,
  "limit": 100,
  "continuation_token": "optional-pagination-token"
}
```

**Response**:
```json
{
  "results": [
    {
      "type": "trade",
      "symbol": "btcusd",
      "timestamp": 1640000000000,
      "amount": "0.5",
      "price": "50000.00",
      "total": "25000.00",
      "fee": "25.00",
      "fee_currency": "USD"
    }
  ],
  "continuationToken": "next-page-token"
}
```

### List Payment Methods
```
POST /v1/payments/methods
```
**Description**: Get linked bank accounts and payment methods
**Authentication**: Required
**OAuth Scope**: `payments:read`

**Request Body**:
```json
{
  "request": "/v1/payments/methods",
  "nonce": 1640000018000,
  "account": "primary"
}
```

**Response**: Array of payment method objects

### Account Detail
```
POST /v1/account
```
**Description**: Get account information
**Authentication**: Required

**Request Body**:
```json
{
  "request": "/v1/account",
  "nonce": 1640000019000
}
```

**Response**: Account details including user information and country code

---

## Positions Trait Endpoints

### Get Open Positions
```
POST /v1/positions
```
**Description**: Get all open derivative positions
**Authentication**: Required (Trader or Auditor role)
**OAuth Scope**: `orders:read`

**Request Body**:
```json
{
  "request": "/v1/positions",
  "nonce": 1640000020000,
  "account": "primary"
}
```

**Response**:
```json
[
  {
    "symbol": "BTCGUSDPERP",
    "instrument_type": "perp",
    "quantity": "2.5",
    "notional_value": "125000.00",
    "realised_pnl": "1250.00",
    "unrealised_pnl": "500.00",
    "average_cost": "49000.00",
    "mark_price": "50200.00"
  }
]
```

**Fields**:
- `symbol`: Trading pair identifier
- `instrument_type`: "perp" for perpetual futures, "spot" for spot
- `quantity`: Position size (negative for shorts)
- `notional_value`: Position value in USD (quantity * mark_price)
- `realised_pnl`: Realized profit/loss
- `unrealised_pnl`: Unrealized profit/loss
- `average_cost`: Average entry price
- `mark_price`: Current mark price

### Get Account Margin
```
POST /v1/margin
```
**Description**: Get margin account summary for derivatives
**Authentication**: Required (Trader or Auditor role)
**OAuth Scope**: `balances:read`

**Request Body**:
```json
{
  "request": "/v1/margin",
  "nonce": 1640000021000,
  "symbol": "BTCGUSDPERP",
  "account": "primary"
}
```

**Parameters**:
- `symbol` (string, required): Perpetual symbol
- `account` (string, optional): Subaccount name

**Response**:
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

### Get Margin Account Summary (Legacy)
```
POST /v1/margin/account
```
**Description**: Get overall margin account summary
**Authentication**: Required (Trader, Fund Manager, or Auditor role)
**OAuth Scope**: `balances:read`

**Request Body**:
```json
{
  "request": "/v1/margin/account",
  "nonce": 1640000022000,
  "account": "primary"
}
```

**Response**:
```json
{
  "marginAssetValue": "10000.00",
  "availableCollateral": "7500.00",
  "notionalValue": "40000.00",
  "totalBorrowed": "0.00",
  "leverage": "4.0",
  "buyingPower": "30000.00",
  "sellingPower": "30000.00",
  "liquidationRisk": "0.15",
  "interestRate": "0.0001",
  "reservedBuyOrders": "250.00",
  "reservedSellOrders": "250.00"
}
```

### Get Margin Interest Rates
```
POST /v1/margin/rates
```
**Description**: Get current margin interest rates
**Authentication**: Required (Trader, Fund Manager, or Auditor role)
**OAuth Scope**: `balances:read`

**Request Body**:
```json
{
  "request": "/v1/margin/rates",
  "nonce": 1640000023000,
  "account": "primary"
}
```

**Response**:
```json
[
  {
    "currency": "BTC",
    "borrowRate": "0.000004167",
    "borrowRateDaily": "0.0001",
    "borrowRateAnnual": "0.0365",
    "lastUpdated": 1640000000000
  }
]
```

### Preview Margin Order Impact
```
POST /v1/margin/order/preview
```
**Description**: Preview how an order would affect margin account
**Authentication**: Required (Trader or Auditor role)
**OAuth Scope**: `orders:read`

**Request Body**:
```json
{
  "request": "/v1/margin/order/preview",
  "nonce": 1640000024000,
  "symbol": "BTCGUSDPERP",
  "side": "buy",
  "type": "market",
  "amount": "1.0",
  "price": null,
  "totalSpend": null,
  "account": "primary"
}
```

**Parameters**:
- `symbol` (string, required): Trading pair
- `side` (string, required): "buy" or "sell"
- `type` (string, required): "market" or "limit"
- `amount` (string, optional): Order size
- `price` (string, optional): Limit price
- `totalSpend` (string, optional): Total spend amount for market buys

**Response**: Pre-order and post-order margin statistics

### List Funding Payments
```
POST /v1/perpetuals/fundingPayment
```
**Description**: Get funding payment history for perpetuals
**Authentication**: Required
**OAuth Scope**: `history:read`

**Request Body**:
```json
{
  "request": "/v1/perpetuals/fundingPayment",
  "nonce": 1640000025000,
  "since": 1640000000,
  "to": 1640086400
}
```

**Query Parameters**:
- `since` (integer, optional): Start timestamp
- `to` (integer, optional): End timestamp

**Response**:
```json
[
  {
    "eventType": "Hourly Funding Transfer",
    "timestamp": 1640003600,
    "assetCode": "USD",
    "action": "Credit",
    "quantity": {
      "currency": "USD",
      "value": "1.23"
    },
    "instrumentSymbol": "BTCGUSDPERP"
  }
]
```

### Get Funding Payment Report (XLSX)
```
GET /v1/perpetuals/fundingpaymentreport/records.xlsx
```
**Description**: Download funding payment report as Excel file
**Authentication**: Required

**Query Parameters**:
- `fromDate` (string, optional): Start date (YYYY-MM-DD)
- `toDate` (string, optional): End date (YYYY-MM-DD)
- `numRows` (integer, optional): Max rows (default: 8760)

**Response**: XLSX file download

### Get Funding Payment Report (JSON)
```
POST /v1/perpetuals/fundingpaymentreport/records.json
```
**Description**: Get funding payment report as JSON
**Authentication**: Required

**Request Body**: Same query parameters as XLSX version

**Response**: JSON array of funding payment records

### Get Risk Stats
```
GET /v1/riskstats/{symbol}
```
**Description**: Get risk statistics for a perpetual
**Authentication**: None (public)

**Path Parameters**:
- `symbol` (string, required): Perpetual symbol (e.g., "BTCGUSDPERP")

**Response**:
```json
{
  "product_type": "PerpetualSwapContract",
  "mark_price": "50200.00",
  "index_price": "50205.00",
  "open_interest": "12345.67",
  "open_interest_notional": "620000000.00"
}
```

---

## Error Responses

All endpoints may return error responses:

```json
{
  "result": "error",
  "reason": "InvalidNonce",
  "message": "Nonce must be increasing"
}
```

Common error reasons:
- `InvalidSignature`: Authentication failed
- `InvalidNonce`: Nonce not increasing or out of time window
- `InvalidQuantity`: Order amount too small or invalid
- `InvalidPrice`: Price precision or value invalid
- `InsufficientFunds`: Not enough balance
- `RateLimit`: Rate limit exceeded (429 status)
- `MaintenanceMode`: Exchange in maintenance
- `InvalidSymbol`: Trading pair not found
- `OrderNotFound`: Order ID doesn't exist
- `Unauthorized`: Insufficient permissions

---

## Notes

1. **Symbol Format**: All lowercase, no separator (e.g., "btcusd", "ethusd", "btcgusdperp")
2. **Timestamps**: Unix timestamps in seconds; timestampms in milliseconds
3. **Amounts/Prices**: Always strings to preserve precision
4. **Nonce**: Must be strictly increasing per API key session, recommend using millisecond timestamp
5. **Roles**: API keys must have appropriate roles (Trader, Fund Manager, Auditor)
6. **OAuth Scopes**: Required for OAuth-based authentication
7. **Pagination**: Use continuation_token for large result sets
8. **Subaccounts**: Use `account` parameter to specify subaccount
9. **Derivatives**: Perpetual symbols end with "PERP" (e.g., "BTCGUSDPERP")
10. **Public Data Limit**: Public API limited to 7 calendar days of historical data
