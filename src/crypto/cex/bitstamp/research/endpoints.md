# Bitstamp REST API Endpoints

**Base URL**: `https://www.bitstamp.net`

**API Version**: v2

## Endpoint Organization

Endpoints are organized by trait implementation requirements:

- **MarketData**: Public endpoints (no authentication)
- **Trading**: Order placement and management (authentication required)
- **Account**: Account information and balances (authentication required)

---

## MarketData Trait Endpoints

### Get Ticker

**Endpoint**: `GET /api/v2/ticker/{pair}/`

**Description**: Returns ticker information for a specific trading pair.

**Parameters**:
- `{pair}`: Trading pair (e.g., `btcusd`, `btceur`)

**Response**:
```json
{
  "last": "2211.00",
  "high": "2811.00",
  "low": "2188.97",
  "vwap": "2189.80",
  "volume": "213.26801100",
  "bid": "2188.97",
  "ask": "2211.00",
  "timestamp": "1643640186",
  "open": "2211.00",
  "open_24": "2211.00",
  "percent_change_24": "13.57",
  "pair": "BTC/USD"
}
```

**Fields**:
- `last`: Last trade price
- `high`: 24h high
- `low`: 24h low
- `vwap`: Volume weighted average price
- `volume`: 24h volume
- `bid`: Highest buy order
- `ask`: Lowest sell order
- `timestamp`: Unix timestamp
- `open`: Opening price
- `open_24`: Price 24h ago
- `percent_change_24`: Percentage change over 24h

---

### Get Hourly Ticker

**Endpoint**: `GET /api/v2/ticker_hour/{pair}/`

**Description**: Returns hourly ticker information.

**Parameters**:
- `{pair}`: Trading pair

---

### Get All Tickers

**Endpoint**: `GET /api/v2/ticker/`

**Description**: Returns ticker information for all trading pairs.

---

### Get Order Book

**Endpoint**: `GET /api/v2/order_book/{pair}/`

**Description**: Returns order book (bids and asks) for a trading pair.

**Parameters**:
- `{pair}`: Trading pair

**Response**:
```json
{
  "timestamp": "1643643584",
  "microtimestamp": "1643643584684047",
  "bids": [
    ["9484.34", "1.00000000"],
    ["9483.00", "0.50000000"]
  ],
  "asks": [
    ["9485.00", "1.00000000"],
    ["9486.50", "0.75000000"]
  ]
}
```

**Fields**:
- `timestamp`: Unix timestamp (seconds)
- `microtimestamp`: Microsecond precision timestamp
- `bids`: Array of [price, amount] arrays (buy orders)
- `asks`: Array of [price, amount] arrays (sell orders)

---

### Get Recent Transactions

**Endpoint**: `GET /api/v2/transactions/{pair}/`

**Description**: Returns list of recent transactions.

**Parameters**:
- `{pair}`: Trading pair
- `time`: (optional) Time interval: `minute`, `hour`, or `day`

**Response**: Array of trade objects

---

### Get OHLC Data

**Endpoint**: `GET /api/v2/ohlc/{pair}/`

**Description**: Returns OHLC (candlestick) data.

**Parameters**:
- `{pair}`: Trading pair
- `step`: Time step in seconds (60, 180, 300, 900, 1800, 3600, 7200, 14400, 21600, 43200, 86400, 259200)
- `limit`: Number of candles (default: 1000, max: 1000)
- `start`: Unix timestamp for start time
- `end`: Unix timestamp for end time
- `exclude_current_candle`: (optional) Exclude current incomplete candle

**Response**:
```json
{
  "data": {
    "ohlc": [
      {
        "timestamp": "1505558814",
        "open": "212.80",
        "high": "213.50",
        "low": "212.00",
        "close": "213.20",
        "volume": "1.23456789"
      }
    ],
    "pair": "BTC/USD"
  }
}
```

---

### Get Trading Pairs Info

**Endpoint**: `GET /api/v2/trading-pairs-info/`

**Description**: Returns information about all trading pairs. (Note: This endpoint is obsolete and replaced by `/api/v2/markets/`)

**Alternative**: `GET /api/v2/markets/`

**Response**: Array of market objects with trading pair details

---

### Get Currencies

**Endpoint**: `GET /api/v2/currencies/`

**Description**: Returns information about available currencies.

**Response**: Information about supported currencies, networks, minimum withdrawal amounts, decimal precision, and status

---

### Get EUR/USD Rate

**Endpoint**: `GET /api/v2/eur_usd/`

**Description**: Returns current EUR/USD exchange rate.

---

## Trading Trait Endpoints

All trading endpoints require authentication.

### Buy Limit Order

**Endpoint**: `POST /api/v2/buy/{pair}/`

**Description**: Opens a buy limit order.

**Parameters**:
- `amount`: Amount to buy
- `price`: Limit price
- `limit_price`: (optional) If the order gets executed, its price will not be higher than this limit
- `daily_order`: (optional) Opens a daily order (expires at 00:00:00 UTC)
- `client_order_id`: (optional) Client-specified order ID (max 255 characters)

**Response**:
```json
{
  "id": "2344851866",
  "datetime": "2018-11-05 16:16:39.532897",
  "type": "0",
  "price": "0.45701",
  "amount": "205.33880000"
}
```

**Fields**:
- `id`: Order ID
- `datetime`: Order creation timestamp
- `type`: Order type (0 = buy, 1 = sell)
- `price`: Order price
- `amount`: Order amount

---

### Sell Limit Order

**Endpoint**: `POST /api/v2/sell/{pair}/`

**Description**: Opens a sell limit order.

**Parameters**: Same as Buy Limit Order

---

### Buy Market Order

**Endpoint**: `POST /api/v2/buy/market/{pair}/`

**Description**: Opens a buy market order.

**Parameters**:
- `amount`: Amount to buy
- `client_order_id`: (optional)

---

### Sell Market Order

**Endpoint**: `POST /api/v2/sell/market/{pair}/`

**Description**: Opens a sell market order.

**Parameters**:
- `amount`: Amount to sell
- `client_order_id`: (optional)

---

### Buy Instant Order

**Endpoint**: `POST /api/v2/buy/instant/{pair}/`

**Description**: Opens a buy instant order.

**Parameters**:
- `amount`: Amount to buy
- `client_order_id`: (optional)

---

### Sell Instant Order

**Endpoint**: `POST /api/v2/sell/instant/{pair}/`

**Description**: Opens a sell instant order.

**Parameters**:
- `amount`: Amount to sell
- `client_order_id`: (optional)

---

### Get Order Status

**Endpoint**: `POST /api/v2/order_status/`

**Description**: Returns the status of an order.

**Parameters**:
- `id`: Order ID
- `client_order_id`: (optional) Client order ID (use instead of `id`)

**Response**:
```json
{
  "status": "Open",
  "id": "2344851866",
  "transactions": []
}
```

**Fields**:
- `status`: Order status (e.g., "Open", "Finished", "Canceled")
- `id`: Order ID
- `transactions`: Array of transaction objects (filled orders)

**Note**: Only returns information for orders from the last 30 days.

---

### Cancel Order

**Endpoint**: `POST /api/v2/cancel_order/`

**Description**: Cancels a specific order.

**Parameters**:
- `id`: Order ID
- `client_order_id`: (optional) Client order ID (use instead of `id`)

**Response**:
```json
{
  "id": "2344851866",
  "amount": "1.00000000",
  "price": "1000.00",
  "type": "0",
  "status": "Canceled"
}
```

---

### Cancel All Orders

**Endpoint**: `POST /api/v2/cancel_all_orders/`

**Description**: Cancels all open orders.

**Parameters**: None required

---

### Get Open Orders (All)

**Endpoint**: `POST /api/v2/open_orders/all/`

**Description**: Returns all open orders across all trading pairs.

**Response**: Array of order objects
```json
[
  {
    "id": "12345",
    "datetime": "2018-11-05 16:16:39",
    "type": "0",
    "price": "9484.34",
    "amount": "1.00000000"
  }
]
```

**Note**: This endpoint is cached for 10 seconds.

---

### Get Open Orders (Specific Pair)

**Endpoint**: `POST /api/v2/open_orders/{pair}/`

**Description**: Returns open orders for a specific trading pair.

**Parameters**:
- `{pair}`: Trading pair

**Note**: This endpoint is cached for 10 seconds.

---

### Get Specific Open Order

**Endpoint**: `POST /api/v2/open_order`

**Description**: Returns a specific open order.

**Parameters**:
- `id`: Order ID
- `client_order_id`: (optional)

---

## Account Trait Endpoints

All account endpoints require authentication.

### Get Account Balances

**Endpoint**: `POST /api/v2/account_balances/`

**Description**: Returns account balances for all currencies.

**Response**:
```json
[
  {
    "currency": "usd",
    "total": "100.00",
    "available": "90.00",
    "reserved": "10.00"
  },
  {
    "currency": "btc",
    "total": "0.50000000",
    "available": "0.45000000",
    "reserved": "0.05000000"
  }
]
```

**Fields**:
- `currency`: Currency code
- `total`: Total balance
- `available`: Available for trading/withdrawal
- `reserved`: Reserved in open orders or pending withdrawals

---

### Get Account Balance (Specific Currency)

**Endpoint**: `POST /api/v2/account_balances/{currency}/`

**Description**: Returns account balance for a specific currency.

**Parameters**:
- `{currency}`: Currency code (e.g., `usd`, `btc`, `eur`)

---

### Get Balance (Legacy)

**Endpoint**: `POST /api/v2/balance/`

**Description**: Returns account balances (legacy format).

**Response**: Object with currency-specific fields like `usd_balance`, `usd_available`, `usd_reserved`, `btc_balance`, etc.

---

### Get Balance (Pair-specific)

**Endpoint**: `POST /api/v2/balance/{pair}/`

**Description**: Returns balance for both currencies in a trading pair.

**Parameters**:
- `{pair}`: Trading pair

---

### Get User Transactions

**Endpoint**: `POST /api/v2/user_transactions/`

**Description**: Returns user transaction history.

**Parameters**:
- `offset`: (optional) Skip this many transactions
- `limit`: (optional) Limit number of results (default: 100, max: 1000)
- `sort`: (optional) Sorting order: `asc` or `desc` (default: `desc`)

**Response**: Array of transaction objects

---

### Get User Transactions (Specific Pair)

**Endpoint**: `POST /api/v2/user_transactions/{pair}/`

**Description**: Returns user transactions for a specific trading pair.

**Parameters**: Same as Get User Transactions

---

### Get Crypto Transactions

**Endpoint**: `POST /api/v2/crypto-transactions/`

**Description**: Returns cryptocurrency deposit/withdrawal transactions.

**Parameters**:
- `offset`: (optional)
- `limit`: (optional)

---

### Get Trading Fees

**Endpoint**: `POST /api/v2/fees/trading/`

**Description**: Returns trading fee information for all markets.

**Response**: Fee structure for different markets

---

### Get Withdrawal Fees

**Endpoint**: `POST /api/v2/fees/withdrawal/`

**Description**: Returns withdrawal fee information for all currencies.

**Response**: Withdrawal fees for different currencies

---

## Withdrawal & Deposit Endpoints

### Get Crypto Deposit Address

**Endpoint**: `POST /api/v2/{coin}_address/`

**Description**: Returns deposit address for a specific cryptocurrency.

**Examples**:
- `/api/v2/bitcoin_address/`
- `/api/v2/ethereum_address/`
- `/api/v2/litecoin_address/`
- `/api/v2/ripple_address/`
- `/api/v2/xrp_address/`
- `/api/v2/bch_address/`

---

### Crypto Withdrawal

**Endpoint**: `POST /api/v2/{coin}_withdrawal/`

**Description**: Initiates a cryptocurrency withdrawal.

**Examples**:
- `/api/v2/bitcoin_withdrawal/`
- `/api/v2/ethereum_withdrawal/`
- `/api/v2/litecoin_withdrawal/`
- `/api/v2/ripple_withdrawal/`
- `/api/v2/xrp_withdrawal/`
- `/api/v2/bch_withdrawal/`

**Parameters** (varies by coin):
- `amount`: Amount to withdraw
- `address`: Destination address
- `instant`: (optional) Instant withdrawal flag
- `destination_tag`: (for XRP/Ripple)

---

### Get Withdrawal Requests

**Endpoint**: `POST /api/v2/withdrawal-requests/`

**Description**: Returns list of withdrawal requests.

**Parameters**:
- `timedelta`: (optional) Time period in seconds

---

### Get Withdrawal Status

**Endpoint**: `POST /api/v2/withdrawal/status/`

**Description**: Returns status of a specific withdrawal.

**Parameters**:
- `id`: Withdrawal ID

---

### Get Unconfirmed Bitcoin Deposits

**Endpoint**: `POST /api/v2/unconfirmed_btc/`

**Description**: Returns unconfirmed Bitcoin deposits.

---

## Sub-Account Management

### Transfer to Sub-Account

**Endpoint**: `POST /api/v2/transfer-to-main/`

**Description**: Transfers funds from sub-account to main account.

**Parameters**:
- `amount`: Amount to transfer
- `currency`: Currency code
- `subAccount`: Sub-account identifier

---

### Transfer to Main Account

**Endpoint**: `POST /api/v2/transfer-from-main/`

**Description**: Transfers funds from main account to sub-account.

**Parameters**:
- `amount`: Amount to transfer
- `currency`: Currency code
- `subAccount`: Sub-account identifier

---

## Banking Endpoints

### Open Bank Withdrawal

**Endpoint**: `POST /api/v2/withdrawal/open/`

**Description**: Opens a bank withdrawal request.

---

### Get Bank Withdrawal Status

**Endpoint**: `POST /api/v2/withdrawal/bank_status/`

**Description**: Returns status of a bank withdrawal.

**Parameters**:
- `id`: Withdrawal ID

---

### Cancel Bank Withdrawal

**Endpoint**: `POST /api/v2/withdrawal/cancel/`

**Description**: Cancels a bank withdrawal.

**Parameters**:
- `id`: Withdrawal ID

---

## Liquidation Endpoints

### Create Liquidation Address

**Endpoint**: `POST /api/v2/liquidation_address/new/`

**Description**: Creates a new liquidation address.

**Parameters**:
- `currency`: Currency code

---

### Get Liquidation Address Info

**Endpoint**: `POST /api/v2/liquidation_address/info/`

**Description**: Returns information about a liquidation address.

**Parameters**:
- `address`: Liquidation address

---

## Additional Endpoints

### Get Travel Rule VASPs

**Endpoint**: `GET /api/v2/travel_rule/vasps/`

**Description**: Returns Travel Rule VASP directory.

---

## Endpoint Summary by Trait

### MarketData Trait
- `GET /api/v2/ticker/{pair}/`
- `GET /api/v2/ticker_hour/{pair}/`
- `GET /api/v2/ticker/`
- `GET /api/v2/order_book/{pair}/`
- `GET /api/v2/transactions/{pair}/`
- `GET /api/v2/ohlc/{pair}/`
- `GET /api/v2/markets/`
- `GET /api/v2/currencies/`
- `GET /api/v2/eur_usd/`

### Trading Trait
- `POST /api/v2/buy/{pair}/`
- `POST /api/v2/sell/{pair}/`
- `POST /api/v2/buy/market/{pair}/`
- `POST /api/v2/sell/market/{pair}/`
- `POST /api/v2/buy/instant/{pair}/`
- `POST /api/v2/sell/instant/{pair}/`
- `POST /api/v2/order_status/`
- `POST /api/v2/cancel_order/`
- `POST /api/v2/cancel_all_orders/`
- `POST /api/v2/open_orders/all/`
- `POST /api/v2/open_orders/{pair}/`
- `POST /api/v2/open_order`

### Account Trait
- `POST /api/v2/account_balances/`
- `POST /api/v2/account_balances/{currency}/`
- `POST /api/v2/balance/`
- `POST /api/v2/balance/{pair}/`
- `POST /api/v2/user_transactions/`
- `POST /api/v2/user_transactions/{pair}/`
- `POST /api/v2/crypto-transactions/`
- `POST /api/v2/fees/trading/`
- `POST /api/v2/fees/withdrawal/`
