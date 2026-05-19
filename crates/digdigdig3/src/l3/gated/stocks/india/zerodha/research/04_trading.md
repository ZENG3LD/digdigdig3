# Zerodha Kite Connect - Trading Endpoints

## Overview

Kite Connect provides comprehensive order management capabilities for all supported Indian exchanges, including regular orders, advanced order types (GTT, iceberg, basket), and full order lifecycle management.

---

## Core Trading Endpoints

### 1. Place Order

**Method**: `POST`

**Endpoint**: `/orders/{variety}`

**URL**: `https://api.kite.trade/orders/{variety}`

**Varieties**:

| Variety | Description |
|---------|-------------|
| `regular` | Standard orders |
| `amo` | After Market Orders (placed outside market hours) |
| `co` | Cover Orders (deprecated in many cases by exchanges) |
| `iceberg` | Iceberg orders (hidden quantity) |
| `auction` | Auction participation orders |

**Parameters**:

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| exchange | string | Yes | NSE, BSE, NFO, BFO, MCX, CDS, BCD |
| tradingsymbol | string | Yes | Trading symbol (e.g., "INFY", "NIFTY26FEB20000CE") |
| transaction_type | string | Yes | BUY or SELL |
| order_type | string | Yes | MARKET, LIMIT, SL, SL-M |
| quantity | int | Yes | Order quantity |
| product | string | Yes | CNC, NRML, MIS, MTF |
| price | float | Conditional | Required for LIMIT orders |
| trigger_price | float | Conditional | Required for SL and SL-M orders |
| validity | string | No | DAY (default), IOC, TTL |
| validity_ttl | int | Conditional | Minutes (required if validity=TTL) |
| disclosed_quantity | int | No | Disclosed quantity for icebergs |
| iceberg_legs | int | No | Number of iceberg legs |
| iceberg_quantity | int | No | Quantity per iceberg leg |
| auction_number | string | Conditional | Required for auction variety |
| tag | string | No | Custom order tag (max 20 chars, alphanumeric) |
| market_protection | int | No | 0 (none), 1-100 (%), -1 (automatic) |
| autoslice | bool | No | Auto slice large orders across exchanges |

**Order Types**:

| Order Type | Description | Requires Price | Requires Trigger |
|------------|-------------|----------------|------------------|
| MARKET | Immediate execution at best available price | No | No |
| LIMIT | Execution at specified price or better | Yes | No |
| SL | Stop-loss limit order | Yes | Yes |
| SL-M | Stop-loss market order | No | Yes |

**Products**:

| Product | Description | Use Case |
|---------|-------------|----------|
| CNC | Cash & Carry | Equity delivery (held overnight) |
| NRML | Normal | Futures & Options (held overnight) |
| MIS | Margin Intraday Squareoff | Intraday (auto-squared off) |
| MTF | Margin Trading Facility | Leveraged equity positions |

**Validity Options**:

| Validity | Description |
|----------|-------------|
| DAY | Valid for the trading session (default) |
| IOC | Immediate or Cancel |
| TTL | Time-to-live in minutes (requires validity_ttl) |

**Request Example** (LIMIT order):
```http
POST /orders/regular HTTP/1.1
Host: api.kite.trade
Authorization: token api_key:access_token
Content-Type: application/x-www-form-urlencoded

exchange=NSE
&tradingsymbol=INFY
&transaction_type=BUY
&order_type=LIMIT
&quantity=10
&price=1450.00
&product=CNC
&validity=DAY
&tag=myorder123
```

**Request Example** (MARKET order):
```http
POST /orders/regular HTTP/1.1
Host: api.kite.trade
Authorization: token api_key:access_token
Content-Type: application/x-www-form-urlencoded

exchange=NSE
&tradingsymbol=RELIANCE
&transaction_type=SELL
&order_type=MARKET
&quantity=5
&product=MIS
```

**Request Example** (Stop-Loss order):
```http
POST /orders/regular HTTP/1.1
Host: api.kite.trade
Authorization: token api_key:access_token
Content-Type: application/x-www-form-urlencoded

exchange=NSE
&tradingsymbol=INFY
&transaction_type=SELL
&order_type=SL
&quantity=10
&price=1440.00
&trigger_price=1445.00
&product=CNC
```

**Request Example** (Iceberg order):
```http
POST /orders/iceberg HTTP/1.1
Host: api.kite.trade
Authorization: token api_key:access_token
Content-Type: application/x-www-form-urlencoded

exchange=NSE
&tradingsymbol=INFY
&transaction_type=BUY
&order_type=LIMIT
&quantity=1000
&price=1450.00
&product=CNC
&iceberg_legs=10
&iceberg_quantity=100
```

**Response**:
```json
{
  "status": "success",
  "data": {
    "order_id": "240126000012345"
  }
}
```

**Rate Limit**: 10 requests/second per API key

**Order Limits**:
- Maximum 3,000 orders per day per user/API key across all segments
- Maximum 200 orders per minute
- Maximum 25 modifications per order

---

### 2. Modify Order

**Method**: `PUT`

**Endpoint**: `/orders/{variety}/{order_id}`

**URL**: `https://api.kite.trade/orders/{variety}/{order_id}`

**Purpose**: Modify pending or open orders

**Parameters**:

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| order_id | string | Yes | Order ID to modify (in URL) |
| quantity | int | No | New quantity |
| price | float | No | New price (LIMIT orders) |
| trigger_price | float | No | New trigger price (SL orders) |
| order_type | string | No | New order type |
| validity | string | No | New validity |
| disclosed_quantity | int | No | New disclosed quantity |
| market_protection | int | No | New market protection |
| autoslice | bool | No | Enable/disable autoslice |

**Request Example**:
```http
PUT /orders/regular/240126000012345 HTTP/1.1
Host: api.kite.trade
Authorization: token api_key:access_token
Content-Type: application/x-www-form-urlencoded

quantity=15
&price=1448.00
```

**Response**:
```json
{
  "status": "success",
  "data": {
    "order_id": "240126000012345"
  }
}
```

**Rate Limit**: 10 requests/second per API key

**Modification Limit**: Maximum 25 modifications per order

**Important Notes**:
- Can only modify pending or open orders
- Cannot modify executed orders
- Cannot change exchange or tradingsymbol

---

### 3. Cancel Order

**Method**: `DELETE`

**Endpoint**: `/orders/{variety}/{order_id}`

**URL**: `https://api.kite.trade/orders/{variety}/{order_id}`

**Purpose**: Cancel pending or open orders

**Parameters**:

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| order_id | string | Yes | Order ID to cancel (in URL) |

**Request Example**:
```http
DELETE /orders/regular/240126000012345 HTTP/1.1
Host: api.kite.trade
Authorization: token api_key:access_token
```

**Response**:
```json
{
  "status": "success",
  "data": {
    "order_id": "240126000012345"
  }
}
```

**Rate Limit**: 10 requests/second per API key

**Important Notes**:
- Can only cancel pending or open orders
- Cannot cancel executed orders
- Cancelled orders appear in order history with status "CANCELLED"

---

### 4. Get Orders

**Method**: `GET`

**Endpoint**: `/orders`

**URL**: `https://api.kite.trade/orders`

**Purpose**: Retrieve all orders for the day

**Parameters**: None

**Response**:
```json
{
  "status": "success",
  "data": [
    {
      "order_id": "240126000012345",
      "parent_order_id": null,
      "exchange_order_id": "1200000012345678",
      "placed_by": "XX0000",
      "variety": "regular",
      "status": "COMPLETE",
      "tradingsymbol": "INFY",
      "exchange": "NSE",
      "instrument_token": 408065,
      "transaction_type": "BUY",
      "order_type": "LIMIT",
      "product": "CNC",
      "quantity": 10,
      "price": 1450.00,
      "trigger_price": 0,
      "average_price": 1449.75,
      "filled_quantity": 10,
      "pending_quantity": 0,
      "cancelled_quantity": 0,
      "disclosed_quantity": 0,
      "validity": "DAY",
      "validity_ttl": 0,
      "order_timestamp": "2026-01-26 09:15:32",
      "exchange_timestamp": "2026-01-26 09:15:33",
      "exchange_update_timestamp": "2026-01-26 09:15:34",
      "status_message": null,
      "status_message_raw": null,
      "guid": "abc123xyz",
      "tag": "myorder123",
      "tags": ["myorder123"],
      "meta": {}
    }
  ]
}
```

**Order Status Values**:

| Status | Description |
|--------|-------------|
| OPEN | Order is pending/active |
| COMPLETE | Order fully executed |
| CANCELLED | Order cancelled by user |
| REJECTED | Order rejected by exchange/system |
| TRIGGER PENDING | SL order waiting for trigger |
| MODIFY PENDING | Modification request pending |
| CANCEL PENDING | Cancellation request pending |
| AMO REQ RECEIVED | AMO received, pending market open |
| VALIDATION PENDING | Order validation in progress |

**Response Fields**:

| Field | Type | Description |
|-------|------|-------------|
| order_id | string | Unique order ID from Zerodha |
| parent_order_id | string | Parent order ID (for CO/BO) |
| exchange_order_id | string | Exchange-assigned order ID |
| placed_by | string | User ID who placed the order |
| variety | string | Order variety (regular, amo, etc.) |
| status | string | Order status |
| tradingsymbol | string | Trading symbol |
| exchange | string | Exchange (NSE, BSE, NFO, etc.) |
| instrument_token | int | Instrument token |
| transaction_type | string | BUY or SELL |
| order_type | string | MARKET, LIMIT, SL, SL-M |
| product | string | CNC, NRML, MIS, MTF |
| quantity | int | Total order quantity |
| price | float | Order price |
| trigger_price | float | Stop-loss trigger price |
| average_price | float | Average execution price |
| filled_quantity | int | Quantity filled |
| pending_quantity | int | Quantity pending |
| cancelled_quantity | int | Quantity cancelled |
| disclosed_quantity | int | Disclosed quantity |
| validity | string | Order validity |
| validity_ttl | int | Time-to-live in minutes |
| order_timestamp | datetime | Order placement timestamp |
| exchange_timestamp | datetime | Exchange timestamp |
| exchange_update_timestamp | datetime | Last update from exchange |
| status_message | string | Status message (if any) |
| status_message_raw | string | Raw status message from exchange |
| guid | string | Unique GUID |
| tag | string | Custom order tag |
| tags | array | Array of tags |
| meta | object | Additional metadata |

**Rate Limit**: 10 requests/second per API key

---

### 5. Get Order History

**Method**: `GET`

**Endpoint**: `/orders/{order_id}`

**URL**: `https://api.kite.trade/orders/{order_id}`

**Purpose**: Get order history/timeline for a specific order

**Parameters**:

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| order_id | string | Yes | Order ID (in URL) |

**Response**: Array of order state changes (same structure as Get Orders, but shows all state transitions)

```json
{
  "status": "success",
  "data": [
    {
      "order_id": "240126000012345",
      "status": "OPEN",
      "filled_quantity": 0,
      "pending_quantity": 10,
      "order_timestamp": "2026-01-26 09:15:32",
      // ... other fields
    },
    {
      "order_id": "240126000012345",
      "status": "COMPLETE",
      "filled_quantity": 10,
      "pending_quantity": 0,
      "order_timestamp": "2026-01-26 09:15:34",
      // ... other fields
    }
  ]
}
```

**Rate Limit**: 10 requests/second per API key

---

### 6. Get Trades

**Method**: `GET`

**Endpoint**: `/trades`

**URL**: `https://api.kite.trade/trades`

**Purpose**: Retrieve all trades for the day

**Parameters**: None

**Response**:
```json
{
  "status": "success",
  "data": [
    {
      "trade_id": "12345678",
      "order_id": "240126000012345",
      "exchange_order_id": "1200000012345678",
      "tradingsymbol": "INFY",
      "exchange": "NSE",
      "instrument_token": 408065,
      "transaction_type": "BUY",
      "product": "CNC",
      "quantity": 5,
      "average_price": 1449.50,
      "fill_timestamp": "2026-01-26 09:15:33",
      "exchange_timestamp": "2026-01-26 09:15:33"
    },
    {
      "trade_id": "12345679",
      "order_id": "240126000012345",
      "exchange_order_id": "1200000012345678",
      "tradingsymbol": "INFY",
      "exchange": "NSE",
      "instrument_token": 408065,
      "transaction_type": "BUY",
      "product": "CNC",
      "quantity": 5,
      "average_price": 1450.00,
      "fill_timestamp": "2026-01-26 09:15:34",
      "exchange_timestamp": "2026-01-26 09:15:34"
    }
  ]
}
```

**Trade Fields**:

| Field | Type | Description |
|-------|------|-------------|
| trade_id | string | Unique trade ID |
| order_id | string | Parent order ID |
| exchange_order_id | string | Exchange order ID |
| tradingsymbol | string | Trading symbol |
| exchange | string | Exchange |
| instrument_token | int | Instrument token |
| transaction_type | string | BUY or SELL |
| product | string | CNC, NRML, MIS, MTF |
| quantity | int | Trade quantity |
| average_price | float | Execution price |
| fill_timestamp | datetime | Fill timestamp |
| exchange_timestamp | datetime | Exchange timestamp |

**Rate Limit**: 10 requests/second per API key

---

### 7. Get Trades for Order

**Method**: `GET`

**Endpoint**: `/orders/{order_id}/trades`

**URL**: `https://api.kite.trade/orders/{order_id}/trades`

**Purpose**: Get trades for a specific order

**Parameters**:

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| order_id | string | Yes | Order ID (in URL) |

**Response**: Array of trades (same structure as Get Trades)

**Rate Limit**: 10 requests/second per API key

---

## GTT (Good Till Triggered) Orders

### 1. Place GTT

**Method**: `POST`

**Endpoint**: `/gtt/triggers`

**URL**: `https://api.kite.trade/gtt/triggers`

**Purpose**: Create a GTT order (valid for 1 year)

**GTT Types**:
- **single**: Single trigger (simple stop-loss or target)
- **two-leg**: OCO (One Cancels Other) - stop-loss + target

**Parameters**:

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| trigger_type | string | Yes | "single" or "two-leg" |
| condition | object | Yes | Condition object |
| orders | array | Yes | Array of order objects |

**Condition Object**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| exchange | string | Yes | NSE, BSE, NFO, etc. |
| tradingsymbol | string | Yes | Trading symbol |
| trigger_values | array | Yes | Array of trigger prices |
| last_price | float | Yes | Current instrument price |

**Order Object**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| exchange | string | Yes | NSE, BSE, NFO, etc. |
| tradingsymbol | string | Yes | Trading symbol |
| transaction_type | string | Yes | BUY or SELL |
| order_type | string | Yes | LIMIT only |
| quantity | int | Yes | Order quantity |
| price | float | Yes | Limit price |
| product | string | Yes | CNC, NRML, MIS |

**Request Example** (Single Trigger):
```http
POST /gtt/triggers HTTP/1.1
Host: api.kite.trade
Authorization: token api_key:access_token
Content-Type: application/json

{
  "trigger_type": "single",
  "condition": {
    "exchange": "NSE",
    "tradingsymbol": "INFY",
    "trigger_values": [1500.00],
    "last_price": 1450.00
  },
  "orders": [
    {
      "exchange": "NSE",
      "tradingsymbol": "INFY",
      "transaction_type": "SELL",
      "order_type": "LIMIT",
      "quantity": 10,
      "price": 1500.00,
      "product": "CNC"
    }
  ]
}
```

**Request Example** (Two-Leg OCO):
```http
POST /gtt/triggers HTTP/1.1
Host: api.kite.trade
Authorization: token api_key:access_token
Content-Type: application/json

{
  "trigger_type": "two-leg",
  "condition": {
    "exchange": "NSE",
    "tradingsymbol": "INFY",
    "trigger_values": [1400.00, 1500.00],
    "last_price": 1450.00
  },
  "orders": [
    {
      "exchange": "NSE",
      "tradingsymbol": "INFY",
      "transaction_type": "SELL",
      "order_type": "LIMIT",
      "quantity": 10,
      "price": 1400.00,
      "product": "CNC"
    },
    {
      "exchange": "NSE",
      "tradingsymbol": "INFY",
      "transaction_type": "SELL",
      "order_type": "LIMIT",
      "quantity": 10,
      "price": 1500.00,
      "product": "CNC"
    }
  ]
}
```

**Response**:
```json
{
  "status": "success",
  "data": {
    "trigger_id": 123456
  }
}
```

**Rate Limit**: 10 requests/second per API key

---

### 2. Modify GTT

**Method**: `PUT`

**Endpoint**: `/gtt/triggers/{trigger_id}`

**URL**: `https://api.kite.trade/gtt/triggers/{trigger_id}`

**Parameters**: Same as Place GTT

**Rate Limit**: 10 requests/second per API key

---

### 3. Delete GTT

**Method**: `DELETE`

**Endpoint**: `/gtt/triggers/{trigger_id}`

**URL**: `https://api.kite.trade/gtt/triggers/{trigger_id}`

**Response**:
```json
{
  "status": "success",
  "data": {
    "trigger_id": 123456
  }
}
```

**Rate Limit**: 10 requests/second per API key

---

### 4. Get GTTs

**Method**: `GET`

**Endpoint**: `/gtt/triggers`

**URL**: `https://api.kite.trade/gtt/triggers`

**Purpose**: Retrieve all active and recent GTTs

**Response**: Array of GTT objects with status, condition, orders, etc.

**GTT Status Values**:
- `active` - GTT is active
- `triggered` - GTT triggered and order placed
- `disabled` - GTT disabled
- `expired` - GTT expired (1 year)
- `cancelled` - GTT cancelled by user
- `rejected` - GTT rejected
- `deleted` - GTT deleted

**Note**: GTT history retained for 7 days after triggering/expiry

**Rate Limit**: 10 requests/second per API key

---

### 5. Get GTT Details

**Method**: `GET`

**Endpoint**: `/gtt/triggers/{trigger_id}`

**URL**: `https://api.kite.trade/gtt/triggers/{trigger_id}`

**Purpose**: Get specific GTT details

**Rate Limit**: 10 requests/second per API key

---

## Basket Orders

Zerodha allows basket orders through the Kite web interface, but there's no dedicated basket order placement API endpoint. However, you can:

1. Place multiple orders programmatically (up to 200 per minute)
2. Use the margin calculator API to pre-calculate basket margins

**No additional fees** for basket orders.

**Limits**:
- Up to 20 orders per basket (UI limitation)
- Up to 50 baskets total (UI limitation)
- API can place orders individually without basket concept

---

## Order Limits & Restrictions

### Daily Limits
- **Maximum 3,000 orders per day** per user/API key across all segments and varieties
- Applies to all order types: regular, AMO, GTT, etc.

### Per-Minute Limits
- **Maximum 200 orders per minute**

### Modification Limits
- **Maximum 25 modifications per order**

### MIS Product Limits
- Maximum 2,000 MIS orders per day across all segments

### Cover Order Limits
- Maximum 2,000 CO per day across all segments

### Rate Limits
- **10 requests per second** per API key
- Applies to all trading endpoints

---

## Error Handling

### OrderException
- Order placement or retrieval failures
- Check status_message for details

### MarginException
- Insufficient funds for order
- Response includes required margin vs available

### HoldingException
- Insufficient holdings for sell order
- Check holdings before selling

### InputException
- Missing or invalid parameters
- Validation errors

**Example Error Response**:
```json
{
  "status": "error",
  "message": "Insufficient funds. Available: 10000.00, Required: 15000.00",
  "error_type": "MarginException",
  "data": null
}
```

---

## Best Practices

1. **Validate before placing**:
   - Check margins using margin calculator API
   - Verify holdings for sell orders
   - Validate instrument symbols

2. **Handle order states**:
   - Poll order status after placement
   - Use WebSocket for real-time postbacks
   - Handle PENDING states gracefully

3. **Respect limits**:
   - Track daily order count
   - Implement rate limiting client-side
   - Don't exceed modification limits

4. **Use tags**:
   - Tag orders for tracking
   - Group related orders
   - Useful for strategy identification

5. **Error recovery**:
   - Implement retry logic with backoff
   - Handle network exceptions
   - Log all order activities

6. **Order lifecycle**:
   - OPEN → (partial fills) → COMPLETE
   - OPEN → CANCELLED
   - OPEN → REJECTED
   - Monitor state transitions

7. **GTT vs Regular Orders**:
   - Use GTT for long-term triggers
   - Use regular orders for immediate execution
   - GTT valid for 1 year vs DAY orders

8. **Product selection**:
   - CNC for delivery
   - MIS for intraday (auto-squared off)
   - NRML for overnight derivatives
   - MTF for leveraged equity
