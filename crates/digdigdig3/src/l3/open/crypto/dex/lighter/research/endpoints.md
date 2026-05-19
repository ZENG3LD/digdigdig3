# Lighter Exchange API Endpoints

## Base URLs

- **Mainnet**: `https://mainnet.zklighter.elliot.ai`
- **Testnet**: `https://testnet.zklighter.elliot.ai`
- **Explorer**: `https://explorer.elliot.ai`

All REST API endpoints are versioned under `/api/v1/`.

---

## MarketData Trait Endpoints

### 1. Get Order Books Metadata
**Endpoint**: `GET /api/v1/orderBooks`

**Parameters**:
- `market_id` (optional, int16, default: 255) - Filter by specific market

**Response**:
```json
{
  "code": 200,
  "message": "string",
  "order_books": [
    {
      "symbol": "ETH",
      "market_id": 0,
      "market_type": "perp",
      "base_asset_id": 0,
      "quote_asset_id": 0,
      "status": "active",
      "taker_fee": "0.0001",
      "maker_fee": "0.0000",
      "liquidation_fee": "0.01",
      "min_base_amount": "0.01",
      "min_quote_amount": "0.1",
      "supported_size_decimals": 4,
      "supported_price_decimals": 4,
      "supported_quote_decimals": 4,
      "order_quote_limit": "281474976.710655"
    }
  ]
}
```

**Weight**: 300

---

### 2. Get Order Book Details
**Endpoint**: `GET /api/v1/orderBookDetails`

**Parameters**:
- `market_id` (optional, int16, default: 255) - Specific market identifier
- `filter` (optional, string, default: "all") - Filter by type: "all", "spot", "perp"

**Response**:
```json
{
  "code": 200,
  "message": "string",
  "order_book_details": [
    {
      "symbol": "ETH",
      "market_id": 0,
      "market_type": "perp",
      "base_asset_id": 0,
      "quote_asset_id": 0,
      "status": "active",
      "taker_fee": "0.0001",
      "maker_fee": "0.0000",
      "liquidation_fee": "0.01",
      "min_base_amount": "0.01",
      "min_quote_amount": "0.1",
      "supported_size_decimals": 4,
      "supported_price_decimals": 4,
      "supported_quote_decimals": 4,
      "order_quote_limit": "281474976.710655",
      "size_decimals": 4,
      "price_decimals": 4,
      "quote_multiplier": 10000,
      "default_initial_margin_fraction": 100,
      "min_initial_margin_fraction": 100,
      "maintenance_margin_fraction": 50,
      "closeout_margin_fraction": 100,
      "last_trade_price": 3024.66,
      "daily_trades_count": 68,
      "daily_base_token_volume": 235.25,
      "daily_quote_token_volume": 93566.25,
      "daily_price_low": 3014.66,
      "daily_price_high": 3024.66,
      "daily_price_change": 3.66,
      "open_interest": 93.0,
      "daily_chart": {"1640995200": 3024.66},
      "market_config": {
        "market_margin_mode": 0,
        "insurance_fund_account_index": 281474976710655,
        "liquidation_mode": 0,
        "force_reduce_only": false,
        "trading_hours": ""
      }
    }
  ],
  "spot_order_book_details": [
    {
      "symbol": "ETH/USDC",
      "market_id": 2048,
      "market_type": "spot",
      "base_asset_id": 1,
      "quote_asset_id": 3,
      "status": "active",
      "last_trade_price": 2731.79,
      "daily_price_change": -10.24
    }
  ]
}
```

**Weight**: 300

---

### 3. Get Order Book Orders
**Endpoint**: `GET /api/v1/orderBookOrders`

**Parameters**:
- `market_id` (required, int16) - Market identifier
- Additional parameters TBD from SDK

**Weight**: 300

---

### 4. Get Recent Trades
**Endpoint**: `GET /api/v1/recentTrades`

**Parameters**:
- `market_id` (required, int16) - Market identifier
- `limit` (optional, int) - Number of trades to return

**Response**:
```json
{
  "code": 200,
  "message": "string",
  "trades": [
    {
      "trade_id": 12345,
      "tx_hash": "0x...",
      "type": "string",
      "market_id": 0,
      "size": "1.5",
      "price": "3024.66",
      "usd_amount": "4536.99",
      "ask_id": 100,
      "bid_id": 101,
      "ask_account_id": 1000,
      "bid_account_id": 1001,
      "is_maker_ask": true,
      "block_height": 123456,
      "timestamp": 1640995200,
      "taker_fee": 1000,
      "taker_position_size_before": "10.0",
      "taker_entry_quote_before": "30000.0",
      "taker_initial_margin_fraction_before": 100,
      "taker_position_sign_changed": false,
      "maker_fee": 0,
      "maker_position_size_before": "5.0",
      "maker_entry_quote_before": "15000.0",
      "maker_initial_margin_fraction_before": 100,
      "maker_position_sign_changed": false,
      "transaction_time": 1640995200
    }
  ]
}
```

**Weight**: 600

---

### 5. Get Trades
**Endpoint**: `GET /api/v1/trades`

**Parameters**:
- `market_id` (optional, int16) - Filter by market
- `account_id` (optional, int) - Filter by account
- `start_timestamp` (optional, int) - Start time filter
- `end_timestamp` (optional, int) - End time filter

**Response**: Same structure as recentTrades

**Weight**: 600

---

### 6. Get Candlesticks (OHLCV)
**Endpoint**: `GET /api/v1/candlesticks`

**Parameters**:
- `market_id` (required, int) - Market identifier
- `resolution` (required, string) - Timeframe: "1m", "5m", "15m", "1h", "4h", "1d"
- `start_timestamp` (optional, int) - Start timestamp
- `end_timestamp` (optional, int) - End timestamp
- `count_back` (optional, int) - Number of candles to retrieve
- `set_timestamp_to_end` (optional, bool, default: false)

**Response**:
```json
{
  "code": 200,
  "message": "string",
  "candlesticks": [
    {
      "timestamp": 1640995200,
      "open": "3020.00",
      "high": "3030.00",
      "low": "3015.00",
      "close": "3024.66",
      "volume": "235.25",
      "quote_volume": "93566.25"
    }
  ]
}
```

**Supported Resolutions**: 1m, 5m, 15m, 1h, 4h, 1d

**Weight**: 300

---

### 7. Get Funding Rates
**Endpoint**: `GET /api/v1/fundings`

**Parameters**:
- `market_id` (optional, int16) - Filter by market

**Response**:
```json
{
  "code": 200,
  "message": "string",
  "fundings": [
    {
      "market_id": 0,
      "funding_rate": "0.0001",
      "timestamp": 1640995200
    }
  ]
}
```

**Weight**: 300

---

### 8. Get Exchange Statistics
**Endpoint**: `GET /api/v1/exchangeStats`

**Parameters**: None

**Response**:
```json
{
  "code": 200,
  "message": "string",
  "stats": {
    "total_volume_24h": "1000000.0",
    "total_trades_24h": 10000,
    "active_markets": 50
  }
}
```

**Weight**: 300

---

## Trading Trait Endpoints

### 1. Send Transaction
**Endpoint**: `POST /api/v1/sendTx`

**Description**: Submit a signed transaction (create order, cancel order, modify order, etc.)

**Request Body**:
```json
{
  "tx_type": 14,
  "tx_info": {
    "account_index": 1,
    "api_key_index": 3,
    "market_id": 0,
    "base_amount": "1000000",
    "price": "30246600",
    "side": "buy",
    "order_type": "limit",
    "client_order_index": 12345,
    "nonce": 1,
    "signature": "0x..."
  }
}
```

**Transaction Types**:
- `14` - L2CreateOrder
- `15` - L2CancelOrder
- `16` - L2CancelAllOrders
- `17` - L2ModifyOrder
- `28` - L2CreateGroupedOrders

**Response**:
```json
{
  "code": 200,
  "message": "Transaction accepted",
  "tx_hash": "0x..."
}
```

**Weight**: 6

---

### 2. Send Transaction Batch
**Endpoint**: `POST /api/v1/sendTxBatch`

**Description**: Submit multiple signed transactions simultaneously (up to 50)

**Request Body**:
```json
{
  "tx_types": [14, 15],
  "tx_infos": [
    {
      "account_index": 1,
      "api_key_index": 3,
      "market_id": 0,
      "base_amount": "1000000",
      "price": "30246600",
      "side": "buy",
      "order_type": "limit",
      "client_order_index": 12345,
      "nonce": 1,
      "signature": "0x..."
    },
    {
      "account_index": 1,
      "api_key_index": 3,
      "order_index": 12344,
      "nonce": 2,
      "signature": "0x..."
    }
  ]
}
```

**Response**:
```json
{
  "code": 200,
  "message": "Batch accepted",
  "tx_hashes": ["0x...", "0x..."]
}
```

**Weight**: 6

---

### 3. Get Next Nonce
**Endpoint**: `GET /api/v1/nextNonce`

**Parameters**:
- `account_index` (required, int) - Account identifier
- `api_key_index` (required, int) - API key index (3-254)

**Response**:
```json
{
  "code": 200,
  "message": "string",
  "nonce": 42
}
```

**Weight**: 6

---

## Account Trait Endpoints

### 1. Get Account Information
**Endpoint**: `GET /api/v1/account`

**Parameters**:
- `by` (required, string) - "index" or "l1_address"
- `value` (required, string) - Account index or L1 address

**Response**:
```json
{
  "code": 200,
  "message": "string",
  "total": 1,
  "accounts": [
    {
      "code": 200,
      "account_type": 1,
      "index": 1,
      "l1_address": "0x70997970C51812dc3A010C7d01b50e0d17dc79C8",
      "status": 1,
      "collateral": "46342",
      "available_balance": "19995",
      "total_order_count": 100,
      "pending_order_count": 100,
      "positions": [
        {
          "market_id": 1,
          "symbol": "ETH",
          "open_order_count": 3,
          "sign": 1,
          "position": "3.6956",
          "avg_entry_price": "3024.66",
          "unrealized_pnl": "17.521309",
          "realized_pnl": "2.000000"
        }
      ],
      "assets": [
        {
          "symbol": "USDC",
          "asset_id": 1,
          "balance": "1000",
          "locked_balance": "1000"
        }
      ]
    }
  ]
}
```

**Field Descriptions**:
- `status`: 1 = active, 0 = inactive
- `sign`: 1 = Long, -1 = Short
- `collateral`: Total account collateral in USDC
- `available_balance`: Available balance for trading
- `unrealized_pnl`: Unsettled profit/loss on current positions
- `realized_pnl`: Settled profit/loss

**Weight**: 3000

---

### 2. Get Accounts by L1 Address
**Endpoint**: `GET /api/v1/accountsByL1Address`

**Parameters**:
- `l1_address` (required, string) - Layer 1 wallet address

**Description**: Retrieves all accounts (master + subaccounts) associated with an L1 address

**Response**: Same structure as account endpoint but may contain multiple accounts

**Weight**: 3000

---

### 3. Get API Keys
**Endpoint**: `GET /api/v1/apikeys`

**Parameters**:
- `account_index` (required, int) - Account identifier
- `api_key_index` (optional, int, default: 255) - Specific key or 255 for all keys

**Response**:
```json
{
  "code": 200,
  "message": "string",
  "api_keys": [
    {
      "api_key_index": 3,
      "public_key": "0x...",
      "status": "active",
      "created_at": 1640995200
    }
  ]
}
```

**Weight**: 150

---

### 4. Get Account Inactive Orders
**Endpoint**: `GET /api/v1/accountInactiveOrders`

**Parameters**:
- `account_index` (required, int) - Account identifier
- `market_id` (optional, int16) - Filter by market
- `limit` (optional, int) - Number of orders to return

**Response**:
```json
{
  "code": 200,
  "message": "string",
  "orders": [
    {
      "order_index": 12345,
      "client_order_index": 100,
      "market_id": 0,
      "side": "buy",
      "order_type": "limit",
      "base_amount": "1.5",
      "price": "3024.66",
      "status": "filled",
      "created_at": 1640995200,
      "updated_at": 1640995300
    }
  ]
}
```

**Weight**: 100

---

### 5. Get Account Transactions
**Endpoint**: `GET /api/v1/accountTxs`

**Parameters**:
- `account_index` (required, int) - Account identifier
- `start_timestamp` (optional, int) - Start time filter
- `end_timestamp` (optional, int) - End time filter
- `limit` (optional, int) - Number of transactions to return

**Response**:
```json
{
  "code": 200,
  "message": "string",
  "transactions": [
    {
      "hash": "0x...",
      "type": 14,
      "info": "{}",
      "event_info": "{}",
      "status": 1,
      "transaction_index": 100,
      "l1_address": "0x...",
      "account_index": 1,
      "nonce": 1,
      "expire_at": 1640999999,
      "block_height": 123456,
      "queued_at": 1640995200,
      "executed_at": 1640995201,
      "sequence_index": 1,
      "parent_hash": "0x...",
      "transaction_time": 1640995201
    }
  ]
}
```

**Weight**: 3000

---

### 6. Get PnL
**Endpoint**: `GET /api/v1/pnl`

**Parameters**:
- `account_index` (required, int) - Account identifier
- `market_id` (optional, int16) - Filter by market

**Response**:
```json
{
  "code": 200,
  "message": "string",
  "pnl": [
    {
      "market_id": 0,
      "realized_pnl": "100.50",
      "unrealized_pnl": "50.25",
      "total_pnl": "150.75"
    }
  ]
}
```

**Weight**: 3000

---

## Positions Trait Endpoints

### 1. Get Positions
**Endpoint**: `GET /api/accounts/{param}/positions`

**Description**: Available via Explorer API at `https://explorer.elliot.ai`

**Parameters**:
- `param` - Account index or L1 address

**Response**:
```json
{
  "positions": [
    {
      "market_id": 1,
      "symbol": "ETH",
      "open_order_count": 3,
      "sign": 1,
      "position": "3.6956",
      "avg_entry_price": "3024.66",
      "unrealized_pnl": "17.521309",
      "realized_pnl": "2.000000"
    }
  ]
}
```

**Weight (Explorer)**: 2

---

## Deposit/Withdrawal Endpoints

### 1. Get Deposit History
**Endpoint**: `GET /api/v1/deposit/history`

**Parameters**:
- `account_index` (required, int) - Account identifier
- `limit` (optional, int) - Number of records to return

**Response**:
```json
{
  "code": 200,
  "message": "string",
  "deposits": [
    {
      "tx_hash": "0x...",
      "l1_tx_hash": "0x...",
      "amount": "1000.0",
      "asset_id": 1,
      "status": "completed",
      "timestamp": 1640995200
    }
  ]
}
```

**Weight**: 100

---

### 2. Get Latest Deposit
**Endpoint**: `GET /api/v1/deposit/latest`

**Parameters**:
- `account_index` (required, int) - Account identifier

**Response**: Same structure as deposit history but returns single latest deposit

**Weight**: 100

---

### 3. Get Withdrawal History
**Endpoint**: `GET /api/v1/withdraw/history`

**Parameters**:
- `account_index` (required, int) - Account identifier
- `limit` (optional, int) - Number of records to return

**Response**:
```json
{
  "code": 200,
  "message": "string",
  "withdrawals": [
    {
      "tx_hash": "0x...",
      "l1_tx_hash": "0x...",
      "amount": "1000.0",
      "asset_id": 1,
      "status": "completed",
      "timestamp": 1640995200
    }
  ]
}
```

**Weight**: 3000

---

## Block and Transaction Lookup Endpoints

### 1. Get Block
**Endpoint**: `GET /api/v1/block`

**Parameters**:
- `height` (required, int) - Block height

**Weight**: 300

---

### 2. Get Blocks
**Endpoint**: `GET /api/v1/blocks`

**Parameters**:
- `start_height` (optional, int) - Start block
- `end_height` (optional, int) - End block
- `limit` (optional, int) - Number of blocks

**Weight**: 300

---

### 3. Get Current Height
**Endpoint**: `GET /api/v1/currentHeight`

**Parameters**: None

**Response**:
```json
{
  "code": 200,
  "message": "string",
  "height": 123456
}
```

**Weight**: 300

---

### 4. Get Transaction
**Endpoint**: `GET /api/v1/tx`

**Parameters**:
- `tx_hash` (required, string) - Transaction hash

**Weight**: 300

---

### 5. Get Transactions
**Endpoint**: `GET /api/v1/txs`

**Parameters**:
- `start_timestamp` (optional, int)
- `end_timestamp` (optional, int)
- `limit` (optional, int)

**Weight**: 300

---

### 6. Get Block Transactions
**Endpoint**: `GET /api/v1/blockTxs`

**Parameters**:
- `block_height` (required, int) - Block height

**Weight**: 300

---

### 7. Get Transaction from L1 Hash
**Endpoint**: `GET /api/v1/txFromL1TxHash`

**Parameters**:
- `l1_tx_hash` (required, string) - Layer 1 transaction hash

**Weight**: 50

---

## Additional Endpoints

### 1. Get Public Pools
**Endpoint**: `GET /api/v1/publicPools`

**Parameters**: TBD

**Weight**: 50

---

### 2. Get Transfer Fee Info
**Endpoint**: `GET /api/v1/transferFeeInfo`

**Parameters**: TBD

**Weight**: 500

---

### 3. Status Endpoint
**Endpoint**: `GET /`

**Description**: Check service status

**Response**:
```json
{
  "status": "ok"
}
```

**Weight**: 300

---

### 4. Service Information
**Endpoint**: `GET /info`

**Description**: Get service information

**Weight**: 300

---

## Error Responses

All endpoints may return error responses with the following structure:

```json
{
  "code": 400,
  "message": "Error description"
}
```

**Common Error Codes**:
- `400` - Bad Request
- `429` - Too Many Requests (Rate limit exceeded)
- `500` - Internal Server Error

---

## Notes

1. All numeric values for amounts and prices are passed as **strings** to preserve precision
2. Amounts and prices in transaction requests must be passed as **integers** (multiply by precision)
3. The `market_id` value of 255 typically means "all markets"
4. Transaction signing must be performed client-side before submission
5. Nonce values must increment by 1 for each transaction per API key
6. Maximum batch size is 50 transactions
