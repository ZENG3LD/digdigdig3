# Paradex API Endpoints

## Base URLs

### Production
- **REST API**: `https://api.prod.paradex.trade/v1`
- **WebSocket**: `wss://ws.api.prod.paradex.trade/v1`

### Testnet (Sepolia)
- **REST API**: `https://api.testnet.paradex.trade/v1`
- **WebSocket**: `wss://ws.api.testnet.paradex.trade/v1`

---

## Authentication

### POST /auth
**Endpoint**: `POST /v1/auth`

**Description**: Obtain JWT token for authenticating subsequent API requests using StarkNet cryptographic signing.

**Required Headers**:
- `PARADEX-STARKNET-ACCOUNT`: StarkNet wallet address
- `PARADEX-STARKNET-SIGNATURE`: Cryptographic signature (array format: [r, s])
- `PARADEX-TIMESTAMP`: Unix timestamp of signature creation

**Optional Headers**:
- `PARADEX-SIGNATURE-EXPIRATION`: Token expiry timestamp (default 30 min, max 1 week)
- `PARADEX-AUTHORIZE-ISOLATED-MARKETS`: Boolean flag for isolated trading accounts

**Response** (200):
```json
{
  "jwt_token": "string"
}
```

**Error Responses**:
- 400: Bad Request
- 401: Unauthorized

---

## Market Data (Public)

### GET /markets
**Endpoint**: `GET /v1/markets`

**Description**: Get markets static data component.

**Query Parameters**:
- `market` (optional): Market identifier (e.g., "BTC-USD-PERP")

**Response** (200):
```json
{
  "results": [
    {
      "symbol": "string",
      "base_currency": "string",
      "quote_currency": "string",
      "settlement_currency": "string",
      "price_tick_size": "string",
      "price_feed_id": "string",
      "clamp_rate": "string",
      "asset_kind": "PERP | PERP_OPTION",
      "market_kind": "cross | isolated | isolated_margin",
      "open_at": "integer (ms timestamp)",
      "expiry_at": "integer (ms timestamp)",
      "max_order_size": "string",
      "max_open_orders": "integer",
      "order_size_increment": "string",
      "min_notional": "string",
      "position_limit": "string",
      "max_slippage": "string",
      "fee_config": {
        "api_fees": { "maker": "string", "taker": "string" },
        "interactive_fees": { "maker": "string", "taker": "string" },
        "rpi_fees": { "maker": "string", "taker": "string" }
      },
      "funding_period_hours": "integer",
      "funding_multiplier": "string",
      "max_funding_rate": "string",
      "interest_rate": "string",
      "max_funding_rate_change": "string",
      "option_type": "PUT | CALL",
      "strike_price": "string",
      "iv_bands_width": "string",
      "contract_address": "string",
      "collateral_address": "string",
      "oracle_address": "string",
      "fee_account_address": "string"
    }
  ]
}
```

### GET /markets/summary
**Endpoint**: `GET /v1/markets/summary`

**Description**: Get dynamic market data for trading instruments.

**Query Parameters**:
- `market` (required): Market name or "ALL" for all markets
- `start` (optional): Start time in Unix milliseconds
- `end` (optional): End time in Unix milliseconds

**Response** (200):
```json
{
  "results": [
    {
      "market": "string",
      "best_bid": "string",
      "best_ask": "string",
      "best_bid_iv": "string",
      "best_ask_iv": "string",
      "last_traded_price": "string",
      "mark_price": "string",
      "spot_price": "string",
      "volume_24h": "string",
      "total_volume": "string",
      "open_interest": "string",
      "price_change_rate_24h": "string",
      "funding_rate": "string",
      "predicted_funding_rate": "string",
      "greeks": {
        "delta": "string",
        "gamma": "string",
        "vega": "string",
        "rho": "string",
        "vanna": "string",
        "volga": "string"
      },
      "created_at": "integer"
    }
  ]
}
```

### GET /orderbook/:market
**Endpoint**: `GET /v1/orderbook/{market}`

**Description**: Get snapshot of the orderbook for the given market.

**Path Parameters**:
- `market` (required): Market symbol (e.g., "BTC-USD-PERP")

**Query Parameters**:
- `depth` (optional, default: 20): Orderbook depth
- `price_tick` (optional): Price tick for aggregation

**Response** (200):
```json
{
  "market": "string",
  "asks": [["price", "size"], ...],
  "bids": [["price", "size"], ...],
  "best_ask_api": ["price", "size"],
  "best_ask_interactive": ["price", "size"],
  "best_bid_api": ["price", "size"],
  "best_bid_interactive": ["price", "size"],
  "last_updated_at": "integer (ms)",
  "seq_no": "integer"
}
```

### GET /orderbook/:market/interactive
**Endpoint**: `GET /v1/orderbook/{market}/interactive`

**Description**: Get interactive orderbook with retail price improvement.

**Path Parameters**:
- `market` (required): Market symbol

**Query Parameters**: Similar to `/orderbook/:market`

### GET /bbo/:market/interactive
**Endpoint**: `GET /v1/bbo/{market}/interactive`

**Description**: Get interactive best bid/offer quotes.

**Path Parameters**:
- `market` (required): Market symbol

### GET /trades
**Endpoint**: `GET /v1/trades`

**Description**: Fetch exchange trades for specific markets (public endpoint).

**Query Parameters**:
- `market` (optional): Market symbol filter
- Additional filters available (see SDK)

**Note**: Exact response format available via Python SDK `fetch_trades(params)` method.

### GET /klines
**Endpoint**: `GET /v1/klines` (inferred from SDK)

**Description**: Get OHLCV candlestick data.

**Query Parameters** (from SDK):
- `symbol`: Market symbol
- `resolution`: Time resolution
- `start_at`: Start timestamp
- `end_at`: End timestamp
- `price_kind`: Price type (mark, last, index, etc.)

**Note**: Python SDK method: `fetch_klines(symbol, resolution, start_at, end_at, price_kind)`

---

## Account Endpoints (Private)

### GET /account
**Endpoint**: `GET /v1/account`

**Description**: Respond with requester's account information.

**Required Headers**:
- `Authorization`: Bearer {jwt_token}

**Query Parameters**:
- `subaccount_address` (optional): Subaccount filter

**Response** (200):
```json
{
  "account": "string",
  "account_value": "string",
  "free_collateral": "string",
  "initial_margin_requirement": "string",
  "maintenance_margin_requirement": "string",
  "margin_cushion": "string",
  "seq_no": "integer",
  "settlement_asset": "string",
  "status": "string",
  "total_collateral": "string",
  "updated_at": "integer"
}
```

**Error Responses**:
- 400: Bad Request
- 401: Unauthorized

### GET /account/info
**Endpoint**: `GET /v1/account/info`

**Description**: Return account info of current account.

**Required Headers**:
- `Authorization`: Bearer {jwt_token}

### GET /account/history
**Endpoint**: `GET /v1/account/history`

**Description**: Get account history data (PnL, value, volume, fee savings).

**Required Headers**:
- `Authorization`: Bearer {jwt_token}

**Query Parameters**:
- `type` (required): "pnl" | "value" | "volume" | "fee_savings"

**Response** (200):
```json
{
  "data": ["number", ...],
  "timestamps": ["integer", ...]
}
```

### GET /balances
**Endpoint**: `GET /v1/balances` (inferred from SDK)

**Description**: Fetch all coin balances for this account.

**Required Headers**:
- `Authorization`: Bearer {jwt_token}

### GET /positions
**Endpoint**: `GET /v1/positions`

**Description**: Get all positions owned by current user.

**Required Headers**:
- `Authorization`: Bearer {jwt_token}

**Response** (200):
```json
{
  "results": [
    {
      "id": "string",
      "account": "string",
      "market": "string",
      "side": "LONG | SHORT",
      "status": "OPEN | CLOSED",
      "size": "string",
      "leverage": "string",
      "average_entry_price": "string",
      "average_entry_price_usd": "string",
      "average_exit_price": "string",
      "liquidation_price": "string",
      "cost": "string",
      "cost_usd": "string",
      "unrealized_pnl": "string",
      "unrealized_funding_pnl": "string",
      "realized_positional_pnl": "string",
      "realized_positional_funding_pnl": "string",
      "cached_funding_index": "string",
      "created_at": "integer",
      "closed_at": "integer",
      "last_updated_at": "integer",
      "seq_no": "integer",
      "last_fill_id": "string"
    }
  ]
}
```

### GET /subaccounts
**Endpoint**: `GET /v1/subaccounts` (inferred from SDK)

**Description**: List sub-accounts of current account.

**Required Headers**:
- `Authorization`: Bearer {jwt_token}

---

## Order Management (Private)

### POST /orders
**Endpoint**: `POST /v1/orders`

**Description**: Create a new order. The API performs basic validation and queues the order for risk checks (status=NEW), then sends it to the matching engine which updates status to OPEN if resting.

**Required Headers**:
- `Authorization`: Bearer {jwt_token}
- `Content-Type`: application/json

**Request Body**:
```json
{
  "instruction": "GTC | IOC | POST_ONLY | RPI",
  "market": "string",
  "price": "string",
  "side": "BUY | SELL",
  "signature": "string",
  "signature_timestamp": "integer",
  "size": "string",
  "type": "MARKET | LIMIT | STOP_LIMIT | STOP_MARKET | TAKE_PROFIT_LIMIT | TAKE_PROFIT_MARKET | STOP_LOSS_MARKET | STOP_LOSS_LIMIT",
  "client_id": "string (optional)",
  "flags": ["REDUCE_ONLY", "STOP_CONDITION_BELOW_TRIGGER", ...],
  "recv_window": "integer (optional, min 10ms)",
  "stp": "EXPIRE_MAKER | EXPIRE_TAKER | EXPIRE_BOTH (optional)",
  "trigger_price": "string (optional, for stop orders)",
  "vwap_price": "string (optional)",
  "on_behalf_of_account": "string (optional)"
}
```

**Response** (201):
```json
{
  "id": "string",
  "status": "NEW | UNTRIGGERED | OPEN | CLOSED",
  "account": "string",
  "created_at": "integer",
  "avg_fill_price": "string",
  "remaining_size": "string"
}
```

**Rate Limit**: 800 req/s OR 17,250 req/min per account

### POST /orders/batch
**Endpoint**: `POST /v1/orders/batch`

**Description**: Create batch of orders (50x efficiency improvement vs individual operations).

**Required Headers**:
- `Authorization`: Bearer {jwt_token}
- `Content-Type`: application/json

**Request Body**: Array of order objects (same as POST /orders)

**Rate Limit**: 800 req/s OR 17,250 req/min per account (1 unit per batch regardless of count)

### GET /orders/:order_id
**Endpoint**: `GET /v1/orders/{order_id}`

**Description**: Get an order by ID. Only returns orders in OPEN or NEW status.

**Path Parameters**:
- `order_id` (required): Order identifier

**Required Headers**:
- `Authorization`: Bearer {jwt_token}

**Response** (200):
```json
{
  "id": "string",
  "account": "string",
  "market": "string",
  "side": "BUY | SELL",
  "type": "MARKET | LIMIT | ...",
  "size": "string",
  "price": "string",
  "status": "NEW | UNTRIGGERED | OPEN | CLOSED",
  "remaining_size": "string",
  "avg_fill_price": "string",
  "instruction": "GTC | POST_ONLY | IOC | RPI",
  "flags": ["string", ...],
  "stp": "string",
  "created_at": "integer",
  "last_updated_at": "integer",
  "seq_no": "integer"
}
```

### GET /orders/by-client-id/:client_id
**Endpoint**: `GET /v1/orders/by-client-id/{client_id}`

**Description**: Get an order by client ID. Only returns orders in OPEN status.

**Path Parameters**:
- `client_id` (required): Client order identifier

**Required Headers**:
- `Authorization`: Bearer {jwt_token}

### GET /orders
**Endpoint**: `GET /v1/orders` (inferred from SDK)

**Description**: List open orders with optional filters.

**Required Headers**:
- `Authorization`: Bearer {jwt_token}

**Query Parameters**: Various filters (market, side, etc.)

### GET /orders/history
**Endpoint**: `GET /v1/orders/history` (inferred from SDK)

**Description**: Fetch historical orders.

**Required Headers**:
- `Authorization`: Bearer {jwt_token}

### DELETE /orders/:order_id
**Endpoint**: `DELETE /v1/orders/{order_id}`

**Description**: Cancel an order. Returns 204 No Content which means the order has been queued for cancellation.

**Path Parameters**:
- `order_id` (required): Order identifier

**Required Headers**:
- `Authorization`: Bearer {jwt_token}

**Response** (204): Empty (order queued for cancellation)

**Error Responses**:
- 400: Bad Request (e.g., order already CLOSED)

**Rate Limit**: 800 req/s OR 17,250 req/min per account

### DELETE /orders/batch
**Endpoint**: `DELETE /v1/orders/batch`

**Description**: Cancel batch of orders by order IDs or client order IDs.

**Required Headers**:
- `Authorization`: Bearer {jwt_token}
- `Content-Type`: application/json

**Request Body**:
```json
{
  "order_ids": ["string", ...],
  "client_order_ids": ["string", ...]
}
```

**Rate Limit**: 800 req/s OR 17,250 req/min per account (1 unit per batch)

### DELETE /orders (cancel all)
**Endpoint**: `DELETE /v1/orders` (inferred from SDK)

**Description**: Cancel all orders with optional market filter.

**Required Headers**:
- `Authorization`: Bearer {jwt_token}

**Query Parameters**:
- `market` (optional): Market filter

### PUT /orders/:order_id
**Endpoint**: `PUT /v1/orders/{order_id}` (inferred from SDK)

**Description**: Modify an open order.

**Required Headers**:
- `Authorization`: Bearer {jwt_token}

**Rate Limit**: 800 req/s OR 17,250 req/min per account

---

## Trade History (Private)

### GET /fills
**Endpoint**: `GET /v1/fills`

**Description**: Fetch history of fills for this account. Includes positions changes due to liquidation alongside regular fills. Flags indicate RPI (Retail Price Improvement) fills.

**Required Headers**:
- `Authorization`: Bearer {jwt_token}

**Query Parameters**: Various filters (market, timestamp range, etc.)

**Note**: Exact response format available via Python SDK `fetch_fills(params)` method.

### GET /funding/payments
**Endpoint**: `GET /v1/funding/payments` (inferred from SDK)

**Description**: Fetch funding payment history.

**Required Headers**:
- `Authorization`: Bearer {jwt_token}

### GET /transactions
**Endpoint**: `GET /v1/transactions` (inferred from SDK)

**Description**: Fetch transaction history.

**Required Headers**:
- `Authorization`: Bearer {jwt_token}

### GET /transfers
**Endpoint**: `GET /v1/transfers` (inferred from SDK)

**Description**: List account's transfers (deposits and withdrawals).

**Required Headers**:
- `Authorization`: Bearer {jwt_token}

### GET /liquidations
**Endpoint**: `GET /v1/liquidations` (inferred from SDK)

**Description**: Fetch historical liquidation data.

**Required Headers**:
- `Authorization`: Bearer {jwt_token}

### GET /tradebusts
**Endpoint**: `GET /v1/tradebusts` (inferred from SDK)

**Description**: Fetch busted trade records.

**Required Headers**:
- `Authorization`: Bearer {jwt_token}

---

## System Endpoints (Public)

### GET /system/config
**Endpoint**: `GET /v1/system/config`

**Description**: Get platform configuration.

### GET /system/state
**Endpoint**: `GET /v1/system/state`

**Description**: Get system status.

### GET /system/time
**Endpoint**: `GET /v1/system/time`

**Description**: Get server timestamp.

### GET /insurance
**Endpoint**: `GET /v1/insurance` (inferred from SDK)

**Description**: Get insurance fund data.

---

## Additional Endpoints

### Algo Orders
- `POST /algos/orders` - Create algo order
- `GET /algos/orders` - List algo orders
- `GET /algos/orders/history` - Algo order history
- `DELETE /algos/orders/:id` - Cancel algo order

### Block Trades
- Block trade endpoints available (see SDK)

### Vaults
- `GET /vaults` - Get vaults information

### Points/XP
- Points program data endpoints (see SDK)

### Funding Data
- `GET /funding/data` - Historical funding data

---

## Notes

- All timestamps are in Unix milliseconds
- All prices and sizes are strings to preserve precision
- Private endpoints require valid JWT token in Authorization header
- Market symbols follow format: `{BASE}-{QUOTE}-{TYPE}` (e.g., "BTC-USD-PERP")
- Paradex supports both perpetual futures (PERP) and perpetual options (PERP_OPTION)
- Some endpoints inferred from Python SDK may have slightly different paths or parameters
