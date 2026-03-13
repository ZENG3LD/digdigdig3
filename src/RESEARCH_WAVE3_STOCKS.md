# Wave 3 Research: Stock Broker Trading APIs

> Research date: 2026-03-13
> Focus: REST endpoints for trading, account, positions, and optional operations

---

## OrderType Variant Reference

Our internal variants used throughout this document:
| Variant | Description |
|---------|-------------|
| `Market` | Immediate execution at market price |
| `Limit` | Execution at specified price or better |
| `StopMarket` | Market order triggered at stop price |
| `StopLimit` | Limit order triggered at stop price |
| `TrailingStop` | Stop that follows price by offset |
| `OCO` | One-Cancels-Other (two linked orders) |
| `Bracket` | Entry + take_profit + stop_loss combo |
| `Iceberg` | Large order split into disclosed legs |
| `PostOnly` | Maker-only limit order |
| `IOC` | Immediate-Or-Cancel |
| `FOK` | Fill-Or-Kill |
| `GTD` | Good-Till-Date |
| `ReduceOnly` | Futures: only reduce existing position |

---

## 1. Alpaca (US)

### Base URLs
- **Live trading**: `https://api.alpaca.markets`
- **Paper trading**: `https://paper-api.alpaca.markets`
- **Base path**: `/v2`

### Authentication
Headers required on every request:
```
APCA-API-KEY-ID: <api_key_id>
APCA-API-SECRET-KEY: <api_secret_key>
```

### Rate Limits
- **200 requests/minute** per account (default)
- Upgradeable to 1,000 requests/minute on request
- HTTP 429 returned when exceeded

---

### Trading Endpoints

#### POST /v2/orders — Place Order

| Parameter | Type | Required | Notes |
|-----------|------|----------|-------|
| `symbol` | string | Yes | Ticker symbol, e.g. `AAPL` |
| `qty` | string | Yes* | Number of shares (*or use `notional`) |
| `notional` | string | Yes* | Dollar amount (*or use `qty`) |
| `side` | string | Yes | `buy` or `sell` |
| `type` | string | Yes | `market`, `limit`, `stop`, `stop_limit`, `trailing_stop` |
| `time_in_force` | string | Yes | `day`, `gtc`, `opg`, `cls`, `ioc`, `fok` |
| `limit_price` | string | Cond. | Required for `limit`, `stop_limit` |
| `stop_price` | string | Cond. | Required for `stop`, `stop_limit` |
| `trail_price` | string | Cond. | Dollar offset for `trailing_stop` |
| `trail_percent` | string | Cond. | Percent offset for `trailing_stop` |
| `extended_hours` | bool | No | Enable pre/post-market execution |
| `client_order_id` | string | No | Custom idempotency key (max 48 chars) |
| `order_class` | string | No | `simple` (default), `bracket`, `oco`, `oto`, `mleg` |
| `take_profit` | object | Cond. | Required for `bracket`/`oco`: `{ "limit_price": "..." }` |
| `stop_loss` | object | Cond. | Required for `bracket`/`oco`: `{ "stop_price": "...", "limit_price": "..." }` |

**Bracket Order Example:**
```json
{
  "symbol": "AAPL",
  "qty": "10",
  "side": "buy",
  "type": "market",
  "time_in_force": "gtc",
  "order_class": "bracket",
  "take_profit": { "limit_price": "200.00" },
  "stop_loss": { "stop_price": "150.00", "limit_price": "148.00" }
}
```

**OCO Order:** Set `order_class: "oco"` with both `take_profit` and `stop_loss`.

**OTO Order:** Set `order_class: "oto"` — triggers a second order when the first fills.

#### GET /v2/orders — List Orders
- Query params: `status` (open/closed/all), `limit`, `after`, `until`, `direction`, `nested` (bool — includes legs)

#### GET /v2/orders/{order_id} — Get Order by ID

#### DELETE /v2/orders/{order_id} — Cancel Single Order

#### DELETE /v2/orders — Cancel All Open Orders
- Returns HTTP 207 Multi-Status with per-order sub-status array
- **Native CancelAll endpoint**: YES

#### PATCH /v2/orders/{order_id} — Replace/Amend Order
- **Native AmendOrder endpoint**: YES
- Replaces the identified order with new parameters; non-specified fields inherit original values

| Parameter | Type | Notes |
|-----------|------|-------|
| `qty` | string | New quantity |
| `time_in_force` | string | New TIF |
| `limit_price` | string | New limit price |
| `stop_price` | string | New stop price |
| `trail` | string | New trail offset |
| `client_order_id` | string | New client ID |

> Note: Replace creates a new order atomically. If the original fills before the replacement reaches the exchange, the replacement is rejected.

---

### Account Endpoints

#### GET /v2/account — Account Details
Returns: `id`, `cash`, `portfolio_value`, `buying_power`, `equity`, `long_market_value`, `short_market_value`, `initial_margin`, `maintenance_margin`, `pattern_day_trader`, `trading_blocked`, `account_blocked`, `status`

#### GET /v2/positions — All Open Positions

#### GET /v2/positions/{symbol_or_asset_id} — Single Position

#### DELETE /v2/positions — Close All Positions (optional `cancel_orders=true`)

#### DELETE /v2/positions/{symbol} — Close Single Position

---

### OrderType Support Matrix

| Our Variant | Alpaca Native | How |
|-------------|--------------|-----|
| `Market` | YES | `type: "market"` |
| `Limit` | YES | `type: "limit"` |
| `StopMarket` | YES | `type: "stop"` |
| `StopLimit` | YES | `type: "stop_limit"` |
| `TrailingStop` | YES | `type: "trailing_stop"` + `trail_price` or `trail_percent` |
| `OCO` | YES | `order_class: "oco"` |
| `Bracket` | YES | `order_class: "bracket"` + `take_profit` + `stop_loss` |
| `Iceberg` | NO | Not supported natively |
| `PostOnly` | NO | No explicit post-only flag |
| `IOC` | YES | `time_in_force: "ioc"` |
| `FOK` | YES | `time_in_force: "fok"` |
| `GTD` | NO | No GTD support (GTC only) |
| `ReduceOnly` | NO | Equity broker, not applicable |

### Optional Traits

| Trait | Native? | Endpoint |
|-------|---------|---------|
| `CancelAll` | YES | `DELETE /v2/orders` |
| `AmendOrder` | YES | `PATCH /v2/orders/{id}` |
| `BatchOrders` | NO | No batch placement endpoint |

### Special Notes
- Paper trading available at separate subdomain — use same API key but different base URL
- `time_in_force: "opg"` and `"cls"` are for market-on-open and market-on-close orders respectively
- Multi-leg options (mleg) available for options strategies
- Trailing stop requires either `trail_price` (dollar amount) or `trail_percent` (percentage) — not both

---

## 2. Zerodha Kite Connect (India)

### Base URL
`https://api.kite.trade`

### Authentication
Every request requires:
```
Authorization: token <api_key>:<access_token>
X-Kite-Version: 3
```
Access tokens expire daily at 6:00 AM IST and must be regenerated via OAuth2 login flow.

### Rate Limits
- **10 requests/second** per API key (across all users sharing the key)
- HTTP 429 on breach; 10-second sliding cooldown window

---

### Trading Endpoints

#### POST /orders/{variety} — Place Order

**Varieties:**
- `regular` — Normal orders during market hours
- `amo` — After Market Orders (executed on next market open)
- `co` — Cover Order (entry + mandatory stoploss leg)
- `iceberg` — Iceberg/disclosed-quantity orders
- `auction` — Auction session orders

| Parameter | Type | Required | Notes |
|-----------|------|----------|-------|
| `tradingsymbol` | string | Yes | Exchange-specific symbol, e.g. `INFY` |
| `exchange` | string | Yes | `NSE`, `BSE`, `NFO`, `CDS`, `MCX` |
| `transaction_type` | string | Yes | `BUY` or `SELL` |
| `order_type` | string | Yes | `MARKET`, `LIMIT`, `SL`, `SL-M` |
| `quantity` | int | Yes | Number of units |
| `product` | string | Yes | `CNC`, `NRML`, `MIS`, `MTF` |
| `price` | float | Cond. | Required for `LIMIT` and `SL` |
| `trigger_price` | float | Cond. | Required for `SL` and `SL-M` |
| `disclosed_quantity` | int | No | Partial disclosure |
| `validity` | string | No | `DAY` (default), `IOC`, `TTL` |
| `validity_ttl` | int | No | Minutes for TTL validity |
| `iceberg_legs` | int | Cond. | Required for `iceberg` variety; total number of legs |
| `iceberg_quantity` | int | Cond. | Required for `iceberg` variety; quantity per leg |
| `auction_number` | string | Cond. | Required for `auction` variety |
| `market_protection` | float | No | Market protection percentage |
| `autoslice` | bool | No | Auto-slice large orders |
| `tag` | string | No | Custom tag (max 20 chars) |

**Response:** `{ "order_id": "151220000000000" }`

#### PUT /orders/{variety}/{order_id} — Modify Order
- **Native AmendOrder**: YES

| Parameter | Notes |
|-----------|-------|
| `order_type` | New order type |
| `quantity` | New quantity |
| `price` | New price |
| `trigger_price` | New trigger price |
| `disclosed_quantity` | New disclosed quantity |
| `validity` | New validity |

**Cover Order modification** accepts only `order_id`, `price`, `trigger_price`.

#### DELETE /orders/{variety}/{order_id} — Cancel Order

#### GET /orders — All Orders for the Day

#### GET /orders/{order_id} — Order History

#### GET /trades — All Executed Trades

#### GET /orders/{order_id}/trades — Trades for Specific Order

---

### Account Endpoints

#### GET /user/margins — All Segment Margins
Returns equity and commodity segment margin data: `net`, `available` (live_balance, opening_balance, live_unrealised_profit, live_realised_profit, funds, collateral, intraday_payin), `utilised` (debits, span, option_premium, holding_sales, turnover, pnl, etc.)

#### GET /user/margins/{segment} — Single Segment
`segment` values: `equity`, `commodity`

#### GET /user/profile — User Profile

---

### Portfolio Endpoints

#### GET /portfolio/positions — Open Positions
Returns net and day positions with fields: `tradingsymbol`, `exchange`, `product`, `quantity`, `average_price`, `last_price`, `pnl`, `unrealised`, `realised`

#### PUT /portfolio/positions — Convert Position
| Parameter | Notes |
|-----------|-------|
| `exchange` | Exchange |
| `tradingsymbol` | Symbol |
| `transaction_type` | BUY/SELL |
| `position_type` | `day` or `overnight` |
| `quantity` | Units to convert |
| `old_product` | From product type |
| `new_product` | To product type |

#### GET /portfolio/holdings — Demat Holdings

---

### OrderType Support Matrix

| Our Variant | Kite Native | How |
|-------------|-------------|-----|
| `Market` | YES | `order_type: "MARKET"` |
| `Limit` | YES | `order_type: "LIMIT"` |
| `StopMarket` | YES | `order_type: "SL-M"` (stoploss-market) |
| `StopLimit` | YES | `order_type: "SL"` (stoploss-limit) |
| `TrailingStop` | NO | Not natively supported |
| `OCO` | NO | No OCO support |
| `Bracket` | PARTIAL | Cover Order (`co` variety) = entry + stoploss only (no take_profit leg) |
| `Iceberg` | YES | `variety: "iceberg"` + `iceberg_legs` + `iceberg_quantity` |
| `PostOnly` | NO | No post-only flag |
| `IOC` | YES | `validity: "IOC"` |
| `FOK` | NO | Not supported |
| `GTD` | PARTIAL | `validity: "TTL"` with `validity_ttl` in minutes |
| `ReduceOnly` | NO | Equity broker |

**Note on Cover Orders:** A Cover Order (`co` variety) is NOT a full bracket order. It requires an entry order + mandatory stop-loss leg, but has NO take-profit leg. The stop-loss must be set at order placement and cannot be converted to a target order.

### Optional Traits

| Trait | Native? | Endpoint |
|-------|---------|---------|
| `CancelAll` | NO | Must cancel individually |
| `AmendOrder` | YES | `PUT /orders/{variety}/{order_id}` |
| `BatchOrders` | NO | No batch placement |

### Special Notes
- Access token is per-user but rate limit is per-API-key (shared across all users)
- `product: "MTF"` is Margin Trading Facility (leveraged delivery)
- `product: "CNC"` = Cash and Carry (delivery equity only)
- `product: "MIS"` = Margin Intraday Squareoff (auto-square off before market close)
- `product: "NRML"` = Normal (for F&O positions, held overnight)
- AMO orders (`amo` variety) are accepted outside market hours and execute on next open
- Cover Order stoploss range is enforced by exchange; too wide/narrow will be rejected

---

## 3. Angel One SmartAPI (India)

### Base URL
`https://smartapi.angelone.in`

### Authentication
Every request requires these headers:
```
Authorization: Bearer <jwtToken>
X-PrivateKey: <api_key>
X-UserType: USER
X-SourceID: WEB
X-ClientLocalIP: <client_ip>
X-ClientPublicIP: <client_public_ip>
X-MACAddress: <mac_address>
X-ClientCode: <client_code>
Content-Type: application/json
```

Auth flow:
1. `POST /rest/auth/angelbroking/user/v1/loginByPassword` with `clientcode`, `password`, `totp`
2. Returns `jwtToken`, `refreshToken`, `feedToken`
3. Use `jwtToken` as Bearer token

---

### Trading Endpoints

#### POST /rest/secure/angelbroking/order/v1/placeOrder — Place Order

| Parameter | Type | Required | Notes |
|-----------|------|----------|-------|
| `variety` | string | Yes | `NORMAL`, `STOPLOSS`, `AMO`, `ROBO` |
| `tradingsymbol` | string | Yes | Symbol name, e.g. `SBIN-EQ` |
| `symboltoken` | string | Yes | Exchange token ID |
| `transactiontype` | string | Yes | `BUY` or `SELL` |
| `exchange` | string | Yes | `NSE`, `BSE`, `NFO`, `MCX`, `NCDEX` |
| `ordertype` | string | Yes | `MARKET`, `LIMIT`, `STOPLOSS_LIMIT`, `STOPLOSS_MARKET` |
| `producttype` | string | Yes | `INTRADAY`, `DELIVERY`, `CARRYFORWARD`, `MARGIN`, `BO`, `CO` |
| `duration` | string | Yes | `DAY`, `IOC` |
| `price` | string | Cond. | Required for `LIMIT` and `STOPLOSS_LIMIT` |
| `triggerprice` | string | Cond. | Required for `STOPLOSS_LIMIT` and `STOPLOSS_MARKET` |
| `quantity` | string | Yes | Order quantity |
| `squareoff` | string | Cond. | Required for BO (Bracket Order): take-profit offset |
| `stoploss` | string | Cond. | Required for BO (Bracket Order): stoploss offset |
| `trailingStopLoss` | string | No | Trailing stoploss for BO |
| `disclosedquantity` | string | No | Partial disclosure quantity |

**Variety values explained:**
- `NORMAL` — Regular market/limit/stop orders
- `STOPLOSS` — Stoploss variety (use with `STOPLOSS_LIMIT` or `STOPLOSS_MARKET` ordertype)
- `AMO` — After Market Order (executed next market open)
- `ROBO` — Bracket Order (takes `squareoff` and `stoploss` parameters)

**Response:** `{ "status": true, "message": "SUCCESS", "errorcode": "", "data": { "orderid": "...", "uniqueorderid": "..." } }`

#### POST /rest/secure/angelbroking/order/v1/modifyOrder — Modify Order
**Native AmendOrder**: YES

| Parameter | Notes |
|-----------|-------|
| `variety` | Order variety |
| `orderid` | Order ID to modify |
| `ordertype` | New order type |
| `producttype` | Product type |
| `duration` | New duration |
| `price` | New price |
| `quantity` | New quantity |
| `tradingsymbol` | Symbol |
| `symboltoken` | Token |
| `exchange` | Exchange |

#### POST /rest/secure/angelbroking/order/v1/cancelOrder — Cancel Order

| Parameter | Notes |
|-----------|-------|
| `variety` | Order variety |
| `orderid` | Order ID to cancel |

#### GET /rest/secure/angelbroking/order/v1/getOrderBook — All Orders

---

### Account Endpoints

#### GET /rest/secure/angelbroking/user/v1/getRMS — Risk Management System (Margins)
Returns margin and fund details: `net`, `availablecash`, `availableliquidityfunds`, `utiliseddebits`, `utilisedspan`, `utilisedoptionpremium`, `utilisedholdsales`, `utilisedturnover`, `utilisedpnl`, `utilisedintradaypaylater`

#### GET /rest/secure/angelbroking/order/v1/getPosition — Open Positions

#### GET /rest/secure/angelbroking/portfolio/v1/getHolding — Holdings (tradeable)

#### GET /rest/secure/angelbroking/portfolio/v1/getAllHolding — All Holdings (including non-tradeable)

---

### OrderType Support Matrix

| Our Variant | Angel One Native | How |
|-------------|-----------------|-----|
| `Market` | YES | `ordertype: "MARKET"` + `variety: "NORMAL"` |
| `Limit` | YES | `ordertype: "LIMIT"` + `variety: "NORMAL"` |
| `StopMarket` | YES | `ordertype: "STOPLOSS_MARKET"` + `variety: "STOPLOSS"` |
| `StopLimit` | YES | `ordertype: "STOPLOSS_LIMIT"` + `variety: "STOPLOSS"` |
| `TrailingStop` | PARTIAL | `trailingStopLoss` param in `ROBO` variety only |
| `OCO` | NO | Not supported |
| `Bracket` | YES | `variety: "ROBO"` + `producttype: "BO"` + `squareoff` + `stoploss` |
| `Iceberg` | NO | Not supported natively |
| `PostOnly` | NO | Not supported |
| `IOC` | YES | `duration: "IOC"` |
| `FOK` | NO | Not supported |
| `GTD` | NO | Only `DAY` and `IOC` |
| `ReduceOnly` | NO | Equity broker |

### Optional Traits

| Trait | Native? | Endpoint |
|-------|---------|---------|
| `CancelAll` | NO | Must cancel individually |
| `AmendOrder` | YES | `POST /rest/secure/angelbroking/order/v1/modifyOrder` |
| `BatchOrders` | NO | No batch placement endpoint |

### Special Notes
- TOTP (Time-based One-Time Password) mandatory for authentication — requires authenticator app setup
- `symboltoken` is a mandatory numeric token unique to each exchange+symbol — must be fetched from master symbol list
- `producttype: "BO"` (Bracket Order) uses `squareoff` and `stoploss` as point offsets from entry price, not absolute prices
- `producttype: "CO"` (Cover Order) requires a stop-loss trigger price; no take-profit leg
- Bracket/ROBO orders cannot be modified after placement (only cancellation)
- `uniqueorderid` in response is a UUID for idempotency checks

---

## 4. Fyers (India)

### Base URL
`https://api.fyers.in`

### Authentication
Header required on every request:
```
Authorization: <client_id>:<access_token>
```
OAuth2 flow: obtain `access_token` via authorization code grant. Token is valid for the trading day.

### Rate Limits
- **10 requests/second**
- **200 requests/minute**
- **100,000 requests/day**

---

### Trading Endpoints

#### POST /api/v3/orders — Place Single Order

| Parameter | Type | Required | Notes |
|-----------|------|----------|-------|
| `symbol` | string | Yes | Format: `NSE:SBIN-EQ` or `NSE:NIFTY2531523000CE` |
| `qty` | int | Yes | Quantity |
| `type` | int | Yes | `1`=Limit, `2`=Market, `3`=Stop (SL-M), `4`=Stop-Limit (SL-L) |
| `side` | int | Yes | `1`=Buy, `-1`=Sell |
| `productType` | string | Yes | `CNC`, `INTRADAY`, `MARGIN`, `CO`, `BO` |
| `limitPrice` | float | Cond. | Required for type `1` (Limit) and `4` (Stop-Limit) |
| `stopPrice` | float | Cond. | Required for type `3` (Stop) and `4` (Stop-Limit) |
| `validity` | string | Yes | `DAY`, `IOC` |
| `disclosedQty` | int | No | Disclosed quantity (iceberg-like disclosure) |
| `offlineOrder` | bool | No | `true` for AMO (After Market Order) |
| `stopLoss` | float | Cond. | Required for `productType: "BO"` — stoploss offset |
| `takeProfit` | float | Cond. | Required for `productType: "BO"` — take-profit offset |

**Order type integer mapping:**
- `1` → Limit
- `2` → Market
- `3` → Stop Order (SL-M, market triggered)
- `4` → Stop-Limit Order (SL-L, limit triggered)

**Bracket Order Example (productType: "BO"):**
```json
{
  "symbol": "NSE:NMDC-EQ",
  "qty": 1,
  "type": 2,
  "side": 1,
  "productType": "BO",
  "limitPrice": 0,
  "stopPrice": 0,
  "validity": "DAY",
  "disclosedQty": 0,
  "offlineOrder": false,
  "stopLoss": "0.50",
  "takeProfit": "1.00"
}
```

**Cover Order Example (productType: "CO"):**
```json
{
  "symbol": "NSE:SBIN-EQ",
  "qty": 1,
  "type": 2,
  "side": 1,
  "productType": "CO",
  "stopPrice": 599.50,
  "validity": "DAY"
}
```

#### DELETE /api/v3/orders — Cancel Order

| Parameter | Type | Required | Notes |
|-----------|------|----------|-------|
| `id` | string | Yes | Order ID to cancel |

#### PUT /api/v3/orders — Modify Order
**Native AmendOrder**: YES

| Parameter | Type | Required | Notes |
|-----------|------|----------|-------|
| `id` | string | Yes | Order ID |
| `type` | int | No | New order type |
| `qty` | int | No | New quantity |
| `limitPrice` | float | No | New limit price |
| `stopPrice` | float | No | New stop price |
| `validity` | string | No | New validity |

#### POST /api/v3/orders/multi — Place Multiple Orders (Batch)
**Native BatchOrders**: YES

Same parameters as single order, but request body is an **array** of order objects.
- **Maximum batch size**: 10 orders per request
- All orders are processed independently; partial success is possible

---

### Account Endpoints

#### GET /api/v3/profile — User Profile
Returns: `fy_id`, `email_id`, `name`, `display_name`, `pan`, `mobile_number`, `bank_account_number`, `demat_id`, `image`, `pin_change_date`, `client_type`, `enabled_exchanges`, `enabled_products`, `investment_status`

#### GET /api/v3/funds — Fund/Margin Details
Returns: `fund_limit` array with `title`, `equityAmount`, `commodityAmount` per row (e.g., "Total Balance", "Utilized Amount", "Available Balance")

#### GET /api/v3/positions — Open Positions
Returns list with: `id`, `symbol`, `qty`, `qty_multiplier`, `avg_price`, `unrealized_profit`, `realized_profit`, `product_type`, `side`

#### POST /api/v3/positions — Exit All Positions
Exits all open positions for the day.

---

### OrderType Support Matrix

| Our Variant | Fyers Native | How |
|-------------|-------------|-----|
| `Market` | YES | `type: 2` |
| `Limit` | YES | `type: 1` |
| `StopMarket` | YES | `type: 3` (SL-M) + `stopPrice` |
| `StopLimit` | YES | `type: 4` (SL-L) + `stopPrice` + `limitPrice` |
| `TrailingStop` | NO | Not supported natively |
| `OCO` | NO | Not supported |
| `Bracket` | YES | `productType: "BO"` + `stopLoss` + `takeProfit` |
| `Iceberg` | PARTIAL | `disclosedQty` provides partial disclosure but no auto-split |
| `PostOnly` | NO | Not supported |
| `IOC` | YES | `validity: "IOC"` |
| `FOK` | NO | Not supported |
| `GTD` | NO | Only `DAY` and `IOC` |
| `ReduceOnly` | NO | Equity broker |

### Optional Traits

| Trait | Native? | Endpoint |
|-------|---------|---------|
| `CancelAll` | NO | Must cancel individually |
| `AmendOrder` | YES | `PUT /api/v3/orders` |
| `BatchOrders` | YES | `POST /api/v3/orders/multi` (max 10 orders) |

### Special Notes
- Symbol format is mandatory: `EXCHANGE:SYMBOL-SUFFIX` (e.g., `NSE:SBIN-EQ`, `NFO:NIFTY25MAR25000CE`)
- `productType: "BO"` (Bracket Order) uses offset values for `stopLoss` and `takeProfit` relative to entry price
- AMO orders use `offlineOrder: true` flag — no separate endpoint needed
- Bracket orders cannot be modified after they are partially executed
- `type` parameter is an integer, not a string — important implementation detail
- `side` parameter is integer: `1` = Buy, `-1` = Sell (not string "BUY"/"SELL")

---

## 5. Dhan (India)

### Base URL
`https://api.dhan.co`

### Authentication
Header required on every request:
```
access-token: <JWT_access_token>
client-id: <dhan_client_id>
Content-Type: application/json
```
Access token is generated from the Dhan developer portal; no OAuth flow — static token per session.
**Requires Static IP whitelisting** for all API access.

---

### Trading Endpoints

#### POST /v2/orders — Place Regular Order

| Parameter | Type | Required | Notes |
|-----------|------|----------|-------|
| `dhanClientId` | string | Yes | Your Dhan client ID |
| `correlationId` | string | No | Custom idempotency ID |
| `transactionType` | string | Yes | `BUY` or `SELL` |
| `exchangeSegment` | string | Yes | `NSE_EQ`, `NSE_FNO`, `NSE_CURRENCY`, `BSE_EQ`, `MCX_COMM` |
| `productType` | string | Yes | `CNC`, `INTRADAY`, `MARGIN`, `MTF` |
| `orderType` | string | Yes | `LIMIT`, `MARKET`, `STOP_LOSS`, `STOP_LOSS_MARKET` |
| `validity` | string | Yes | `DAY`, `IOC` |
| `securityId` | string | Yes | Dhan security ID |
| `quantity` | int | Yes | Order quantity |
| `disclosedQuantity` | int | No | Disclosed quantity |
| `price` | float | Cond. | Required for `LIMIT` and `STOP_LOSS` |
| `triggerPrice` | float | Cond. | Required for `STOP_LOSS` and `STOP_LOSS_MARKET` |
| `afterMarketOrder` | bool | No | `true` to place as AMO |
| `amoTime` | string | Cond. | Required when `afterMarketOrder: true`; values: `OPEN`, `OPEN_30`, `OPEN_60` |
| `boProfitValue` | float | Cond. | Bracket Order take-profit offset (when placed via orders endpoint with BO product) |
| `boStopLossValue` | float | Cond. | Bracket Order stoploss offset |

#### PUT /v2/orders/{order-id} — Modify Order
**Native AmendOrder**: YES

| Parameter | Notes |
|-----------|-------|
| `dhanClientId` | Client ID |
| `orderType` | New order type |
| `legendQty` | New quantity |
| `price` | New price |
| `triggerPrice` | New trigger price |
| `disclosedQuantity` | New disclosed quantity |
| `validity` | New validity |

#### DELETE /v2/orders/{order-id} — Cancel Order

#### GET /v2/orders — Order Book (all orders for day)

#### GET /v2/orders/{order-id} — Single Order Status

#### GET /v2/orders/external/{correlation-id} — Order by Correlation ID

---

### After Market Order (AMO) Support
AMO is embedded in the regular `POST /v2/orders` endpoint using:
- `afterMarketOrder: true`
- `amoTime`: `OPEN` (market open), `OPEN_30` (30 min after open), `OPEN_60` (60 min after open)

**No separate AMO endpoint** — flag-based approach.

---

### Super Order (Bracket/Advanced Order)

#### POST /v2/super/orders — Place Super Order
**Native Bracket**: YES (as "Super Order")

| Parameter | Type | Required | Notes |
|-----------|------|----------|-------|
| `dhanClientId` | string | Yes | Client ID |
| `correlationId` | string | No | Custom ID |
| `transactionType` | string | Yes | `BUY` or `SELL` |
| `exchangeSegment` | string | Yes | Exchange segment |
| `productType` | string | Yes | `CNC` or `INTRADAY` |
| `orderType` | string | Yes | `LIMIT` or `MARKET` (entry) |
| `securityId` | string | Yes | Security ID |
| `quantity` | int | Yes | Quantity |
| `price` | float | Cond. | Entry price for LIMIT |
| `targetPrice` | float | Yes | Absolute target price |
| `stopLossPrice` | float | Yes | Absolute stop-loss price |
| `trailingJump` | float | No | Trailing step for stop-loss |

**Key difference from bracket orders:** Uses absolute prices for `targetPrice`/`stopLossPrice`, not offsets.

#### PUT /v2/super/orders/{order-id} — Modify Super Order
- Modify by leg: `ENTRY_LEG`, `TARGET_LEG`, `STOP_LOSS_LEG`
- `ENTRY_LEG` modifiable only while status is `PENDING` or `PART_TRADED`
- After entry fills, only `TARGET_LEG` and `STOP_LOSS_LEG` prices/trailing can be modified

#### DELETE /v2/super/orders/{order-id}/{order-leg} — Cancel Super Order Leg
Cancel individual legs: `ENTRY_LEG`, `TARGET_LEG`, `STOP_LOSS_LEG`

#### GET /v2/super/orders — List All Super Orders

---

### Order Slicing

#### POST /v2/orders/slicing — Slice Large Order
Splits a large order into multiple legs when it exceeds exchange freeze quantity limits.

| Parameter | Notes |
|-----------|-------|
| Same params as regular order | Dhan auto-determines slice count |

---

### Account Endpoints

#### GET /v2/fundlimit — Fund and Margin Details
Returns: `dhanClientId`, `availabelBalance`, `sodLimit`, `collateralAmount`, `receiveableAmount`, `utilizedAmount`, `blockedPayoutAmount`, `withdrawableBalance`

Note: `availabelBalance` is a known typo in the API — use as-is.

#### POST /v2/margincalculator — Margin Calculator
Returns required margin for hypothetical order(s).

---

### Portfolio Endpoints

#### GET /v2/positions — Open Positions

#### GET /v2/holdings — Demat Holdings

#### POST /v2/positions/convert — Convert Position
| Parameter | Notes |
|-----------|-------|
| `fromProductType` | `CNC`, `INTRADAY`, `MARGIN` |
| `toProductType` | Target product type |
| `positionType` | `LONG`, `SHORT`, `CLOSED` |
| `convertQty` | Units to convert |

#### DELETE /v2/positions — Exit All Positions

---

### OrderType Support Matrix

| Our Variant | Dhan Native | How |
|-------------|------------|-----|
| `Market` | YES | `orderType: "MARKET"` |
| `Limit` | YES | `orderType: "LIMIT"` |
| `StopMarket` | YES | `orderType: "STOP_LOSS_MARKET"` |
| `StopLimit` | YES | `orderType: "STOP_LOSS"` |
| `TrailingStop` | PARTIAL | `trailingJump` in Super Orders only |
| `OCO` | NO | Not supported |
| `Bracket` | YES | `POST /v2/super/orders` with `targetPrice` + `stopLossPrice` |
| `Iceberg` | PARTIAL | `disclosedQuantity` field; full iceberg via slicing endpoint |
| `PostOnly` | NO | Not supported |
| `IOC` | YES | `validity: "IOC"` |
| `FOK` | NO | Not supported |
| `GTD` | NO | Only `DAY` and `IOC` |
| `ReduceOnly` | NO | Equity broker |

### Optional Traits

| Trait | Native? | Endpoint |
|-------|---------|---------|
| `CancelAll` | NO | Must cancel individually |
| `AmendOrder` | YES | `PUT /v2/orders/{order-id}` |
| `BatchOrders` | NO | No batch placement endpoint |

### Special Notes
- **Static IP whitelisting required** — the API will reject requests from non-whitelisted IPs
- `securityId` is a numeric Dhan-specific identifier, different from exchange tokens — must be fetched from Dhan's security master
- AMO `amoTime` values: `OPEN` = at market open, `OPEN_30` = 30 min after open, `OPEN_60` = 60 min after open
- Super Order `trailingJump` is the price increment by which the stop-loss trails the market price
- Bracket orders via the regular orders endpoint (`boProfitValue`/`boStopLossValue`) use offsets; Super Orders use absolute prices
- `exchangeSegment` values: `NSE_EQ`, `NSE_FNO`, `NSE_CURRENCY`, `BSE_EQ`, `BSE_FNO`, `MCX_COMM`

---

## 6. Upstox (India)

### Base URLs
- **High-frequency trading**: `https://api-hft.upstox.com` (used for order placement/modification/cancellation)
- **Standard**: `https://api.upstox.com` (used for data endpoints)

### Authentication
Header required on every request:
```
Authorization: Bearer <access_token>
Content-Type: application/json
Accept: application/json
```
OAuth2 flow with authorization code grant; access token expires daily.

---

### Trading Endpoints

#### POST /v2/order/place — Place Order (Legacy v2)

| Parameter | Type | Required | Notes |
|-----------|------|----------|-------|
| `quantity` | int | Yes | Order quantity |
| `product` | string | Yes | `I` (Intraday), `D` (Delivery), `MTF` |
| `validity` | string | Yes | `DAY`, `IOC` |
| `price` | float | Yes | `0` for market orders |
| `instrument_token` | string | Yes | Format: `NSE_EQ\|INE669E01016` |
| `order_type` | string | Yes | `MARKET`, `LIMIT`, `SL`, `SL-M` |
| `transaction_type` | string | Yes | `BUY`, `SELL` |
| `disclosed_quantity` | int | Yes | Partial disclosure (0 for full) |
| `trigger_price` | float | No | For `SL` and `SL-M` orders |
| `is_amo` | bool | No | `true` for After Market Order |
| `tag` | string | No | Custom identifier (max 40 chars) |

**Response:**
```json
{ "status": "success", "data": { "order_id": "1644490272000" } }
```

#### POST /v3/order/place — Place Order V3 (Current Recommended)
Same parameters as v2, plus:

| Parameter | Type | Notes |
|-----------|------|-------|
| `slice` | bool | Auto-split order by freeze quantity |
| `market_protection` | int | Protection % for MARKET/SL-M orders (0–25 or -1 for auto) |

#### PUT /v2/order/modify — Modify Order
**Native AmendOrder**: YES

| Parameter | Type | Required | Notes |
|-----------|------|----------|-------|
| `order_id` | string | Yes | Order to modify |
| `quantity` | int | No | New quantity |
| `validity` | string | No | New validity |
| `price` | float | No | New price |
| `order_type` | string | No | New order type |
| `disclosed_quantity` | int | No | New disclosed quantity |
| `trigger_price` | float | No | New trigger price |

#### DELETE /v2/order/cancel — Cancel Single Order
- Query param: `order_id=<order_id>`

#### DELETE /v2/order/multi/cancel — Cancel Multiple Orders (CancelAll)
**Native CancelAll**: YES (filtered cancel-all)

| Parameter | Type | Required | Notes |
|-----------|------|----------|-------|
| `tag` | string | No | Cancel only orders with this tag |
| `segment` | string | No | Cancel only orders in this segment |

- Cancels all matching open orders (both regular and AMO)
- **Maximum 200 orders** per request
- No `order_id` needed — filters by tag/segment or cancels all if no filter

#### POST /v2/order/multi/place — Place Multiple Orders (Batch)
**Native BatchOrders**: YES

Same parameters as single order per element, wrapped in an array.
- **Maximum 25 orders** per batch request
- `correlation_id` required per order (max 20 chars)
- BUY orders execute before SELL within same batch

---

### Account Endpoints

#### GET /v2/user/profile — User Profile
Returns: `user_id`, `user_name`, `email`, `mobile_number`, `broker`, `exchanges`, `products`, `order_types`, `is_active`

#### GET /v2/user/fund-and-margin — Fund and Margin Details
Returns equity and commodity segment data: `used_margin`, `firm_payin_margin`, `collateral`, `adhoc_margin`, `notional_cash`, `available_margin`, `withdrawable_balance`

Note: As of July 19, 2025, combined fund data for both Equity and Commodity in the equity object.

---

### Portfolio Endpoints

#### GET /v2/portfolio/short-term-positions — Open Positions (Intraday/F&O)

#### GET /v2/portfolio/long-term-holdings — Demat Holdings

#### POST /v2/portfolio/convert-position — Convert Position
| Parameter | Notes |
|-----------|-------|
| `instrument_token` | Security |
| `transaction_type` | BUY/SELL |
| `transaction_type` | New product type |
| `quantity` | Units to convert |

---

### OrderType Support Matrix

| Our Variant | Upstox Native | How |
|-------------|--------------|-----|
| `Market` | YES | `order_type: "MARKET"` |
| `Limit` | YES | `order_type: "LIMIT"` |
| `StopMarket` | YES | `order_type: "SL-M"` |
| `StopLimit` | YES | `order_type: "SL"` |
| `TrailingStop` | NO | Not supported |
| `OCO` | NO | Not supported |
| `Bracket` | NO | No native bracket order; use GTT (Good Till Triggered) orders as workaround |
| `Iceberg` | PARTIAL | `disclosed_quantity` field; `slice: true` in v3 for auto-splitting |
| `PostOnly` | NO | Not supported |
| `IOC` | YES | `validity: "IOC"` |
| `FOK` | NO | Not supported |
| `GTD` | NO | Only `DAY` and `IOC` for regular orders |
| `ReduceOnly` | NO | Equity broker |

### Optional Traits

| Trait | Native? | Endpoint |
|-------|---------|---------|
| `CancelAll` | YES | `DELETE /v2/order/multi/cancel` (max 200) |
| `AmendOrder` | YES | `PUT /v2/order/modify` |
| `BatchOrders` | YES | `POST /v2/order/multi/place` (max 25) |

### Special Notes
- `instrument_token` format uses pipe separator: `NSE_EQ|INE669E01016` — not the slash format used elsewhere
- v2 APIs are being deprecated in favor of v3; migration recommended
- `market_protection` in v3 prevents extreme slippage on market orders (capped at 25%)
- AMO orders use `is_amo: true` flag — no separate endpoint
- `slice: true` in v3 automatically splits order into exchange-compliant freeze-quantity lots
- Two base URLs: order operations use `api-hft.upstox.com`, data queries use `api.upstox.com`
- GTT (Good Till Triggered) orders available as a separate feature for conditional/bracket-like strategies

---

## Cross-Broker Comparison Summary

### OrderType Support by Broker

| Variant | Alpaca | Zerodha | AngelOne | Fyers | Dhan | Upstox |
|---------|--------|---------|----------|-------|------|--------|
| `Market` | YES | YES | YES | YES | YES | YES |
| `Limit` | YES | YES | YES | YES | YES | YES |
| `StopMarket` | YES | YES (SL-M) | YES | YES (type=3) | YES | YES (SL-M) |
| `StopLimit` | YES | YES (SL) | YES | YES (type=4) | YES | YES (SL) |
| `TrailingStop` | YES | NO | PARTIAL (BO only) | NO | PARTIAL (Super Order) | NO |
| `OCO` | YES | NO | NO | NO | NO | NO |
| `Bracket` | YES | PARTIAL (CO) | YES (ROBO/BO) | YES (BO) | YES (Super Order) | NO |
| `Iceberg` | NO | YES (variety) | NO | PARTIAL | PARTIAL | PARTIAL |
| `PostOnly` | NO | NO | NO | NO | NO | NO |
| `IOC` | YES | YES | YES | YES | YES | YES |
| `FOK` | YES | NO | NO | NO | NO | NO |
| `GTD` | NO | PARTIAL (TTL) | NO | NO | NO | NO |
| `ReduceOnly` | NO | NO | NO | NO | NO | NO |

### Optional Traits Support

| Trait | Alpaca | Zerodha | AngelOne | Fyers | Dhan | Upstox |
|-------|--------|---------|----------|-------|------|--------|
| `CancelAll` | YES (`DELETE /v2/orders`) | NO | NO | NO | NO | YES (filtered, max 200) |
| `AmendOrder` | YES (`PATCH`) | YES (`PUT`) | YES (`POST`) | YES (`PUT`) | YES (`PUT`) | YES (`PUT`) |
| `BatchOrders` | NO | NO | NO | YES (max 10) | NO | YES (max 25) |

### Authentication Method Summary

| Broker | Method | Token Expiry |
|--------|--------|-------------|
| Alpaca | Static API Key + Secret in headers | No expiry |
| Zerodha | OAuth2; `api_key:access_token` in Authorization | Daily (6 AM IST) |
| Angel One | JWT Bearer + TOTP-based login | Session-based |
| Fyers | OAuth2 access token; `client_id:access_token` | Daily |
| Dhan | Static JWT from developer portal; IP-whitelisted | Long-lived; IP-locked |
| Upstox | OAuth2 Bearer token | Daily |

---

## Sources

- [Alpaca: POST /v2/orders Reference](https://docs.alpaca.markets/reference/postorder)
- [Alpaca: Working with Orders](https://docs.alpaca.markets/docs/working-with-orders)
- [Alpaca: Replace Order](https://docs.alpaca.markets/reference/patchorderbyorderid-1)
- [Alpaca: Delete All Orders](https://docs.alpaca.markets/reference/deleteallorders-1)
- [Zerodha Kite Connect v3: Orders](https://kite.trade/docs/connect/v3/orders/)
- [Zerodha Kite Connect v3: Portfolio](https://kite.trade/docs/connect/v3/portfolio/)
- [Zerodha Kite Connect v3: User](https://kite.trade/docs/connect/v3/user/)
- [Angel One SmartAPI: Python SDK](https://github.com/angel-one/smartapi-python/blob/main/SmartApi/smartConnect.py)
- [Angel One SmartAPI: Java SDK Examples](https://github.com/angel-one/smartapi-java/blob/main/src/main/java/com/angelbroking/smartapi/sample/Examples.java)
- [Angel One SmartAPI: getAllHolding Announcement](https://smartapi.angelone.in/smartapi/forum/topic/4006/new-fields-added-to-getholding-endpoint-and-introduction-of-getallholding-endpoint/3)
- [Fyers API v3: Order API Knowledge Base](https://support.fyers.in/portal/en/kb/fyers-api-integrations/fyers-api/api-v3/order-api)
- [Fyers API v3: Introduction Blog](https://fyers.in/community/blogs-gdppin8d/post/unveiling-fyers-api-version-3-v3-0-0-a-comprehensive-update-to-enhance-NUuYJmm6gt9toPm)
- [DhanHQ v2: Orders Documentation](https://dhanhq.co/docs/v2/orders/)
- [DhanHQ v2: Super Order Documentation](https://dhanhq.co/docs/v2/super-order/)
- [DhanHQ v2: Portfolio Documentation](https://dhanhq.co/docs/v2/portfolio/)
- [DhanHQ v2: Funds Documentation](https://dhanhq.co/docs/v2/funds/)
- [Upstox: Place Order v2](https://upstox.com/developer/api-documentation/place-order/)
- [Upstox: Place Order v3](https://upstox.com/developer/api-documentation/v3/place-order/)
- [Upstox: Place Multi Order](https://upstox.com/developer/api-documentation/place-multi-order/)
- [Upstox: Cancel Multi Order](https://upstox.com/developer/api-documentation/cancel-multi-order/)
- [Upstox: Orders Overview](https://upstox.com/developer/api-documentation/orders/)
