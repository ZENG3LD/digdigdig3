# Interactive Brokers Client Portal Web API - Trading Specific Details

## Supported Order Types

### Basic Order Types (Confirmed Available)

#### 1. Market Order (MKT)

**Description:** Executes immediately at best available price

**Parameters:**
```json
{
  "conid": 265598,
  "orderType": "MKT",
  "side": "BUY",
  "tif": "DAY",
  "quantity": 100
}
```

**Use Cases:**
- Immediate execution required
- High liquidity instruments
- Small orders relative to volume

**Risks:**
- Price uncertainty
- Slippage in volatile markets
- Gap risk

#### 2. Limit Order (LMT)

**Description:** Executes only at specified price or better

**Parameters:**
```json
{
  "conid": 265598,
  "orderType": "LMT",
  "price": 185.00,
  "side": "BUY",
  "tif": "GTC",
  "quantity": 100
}
```

**Use Cases:**
- Price control required
- Patient entry/exit
- Low liquidity instruments

**Risks:**
- May not fill
- Partial fills possible
- Market may move away

#### 3. Stop Order (STP)

**Description:** Market order triggered when stop price reached

**Parameters:**
```json
{
  "conid": 265598,
  "orderType": "STP",
  "price": 180.00,
  "side": "SELL",
  "tif": "DAY",
  "quantity": 100
}
```

**Notes:**
- `price` field specifies stop trigger price
- Converts to market order when triggered
- No price protection after trigger

**Use Cases:**
- Stop loss protection
- Breakout entries
- Momentum following

**Risks:**
- Slippage after trigger
- Gap risk
- Whipsaw in volatile markets

#### 4. Stop Limit Order (STP_LMT)

**Description:** Limit order triggered when stop price reached

**Parameters:**
```json
{
  "conid": 265598,
  "orderType": "STP_LMT",
  "price": 179.50,
  "auxPrice": 180.00,
  "side": "SELL",
  "tif": "GTC",
  "quantity": 100
}
```

**Field Mapping:**
- `price` - Limit price (execution price)
- `auxPrice` - Stop price (trigger price)

**Use Cases:**
- Stop loss with price protection
- Controlled exits
- Gap protection

**Risks:**
- May not fill after trigger
- Market may move through both prices
- More complex than simple stop

#### 5. Trailing Stop Order (TRAIL)

**Description:** Stop that trails price by specified amount/percentage

**Parameters (Fixed Amount):**
```json
{
  "conid": 265598,
  "orderType": "TRAIL",
  "price": 185.00,
  "auxPrice": 2.00,
  "trailingAmt": 2.00,
  "trailingType": "amt",
  "side": "SELL",
  "tif": "GTC",
  "quantity": 100
}
```

**Parameters (Percentage):**
```json
{
  "conid": 265598,
  "orderType": "TRAIL",
  "price": 185.00,
  "trailingAmt": 2,
  "trailingType": "%",
  "side": "SELL",
  "tif": "GTC",
  "quantity": 100
}
```

**Trailing Types:**
- `amt` - Fixed dollar/point amount
- `%` - Percentage of price

**Use Cases:**
- Riding trends
- Protecting profits
- Dynamic stop management

**Behavior:**
- For SELL orders: Stop trails price upward, triggers on decline
- For BUY orders: Stop trails price downward, triggers on rise
- Once triggered, converts to market order

#### 6. Trailing Stop Limit Order (TRAILLMT)

**Description:** Trailing stop that converts to limit order when triggered

**Parameters:**
```json
{
  "conid": 265598,
  "orderType": "TRAILLMT",
  "price": 185.00,
  "auxPrice": 2.00,
  "trailingAmt": 2.00,
  "trailingType": "amt",
  "side": "SELL",
  "tif": "GTC",
  "quantity": 100
}
```

**Use Cases:**
- Trailing with price protection
- Avoiding gap slippage
- Profit protection with control

#### 7. Limit if Touched (LIT)

**Description:** Limit order triggered when price touches specified level

**Parameters:**
```json
{
  "conid": 265598,
  "orderType": "LIT",
  "price": 185.00,
  "side": "BUY",
  "tif": "DAY",
  "quantity": 100
}
```

**Use Cases:**
- Entry on pullback
- Limit order at support/resistance
- Better than limit for momentum entries

#### 8. Market if Touched (MIT)

**Description:** Market order triggered when price touches specified level

**Parameters:**
```json
{
  "conid": 265598,
  "orderType": "MIT",
  "price": 185.00,
  "side": "BUY",
  "tif": "DAY",
  "quantity": 100
}
```

**Use Cases:**
- Breakout entries
- Quick execution at level
- Alternative to stop orders

#### 9. Limit on Close (LOC)

**Description:** Limit order executed at market close

**Parameters:**
```json
{
  "conid": 265598,
  "orderType": "LOC",
  "price": 185.00,
  "side": "BUY",
  "tif": "DAY",
  "quantity": 100
}
```

**Use Cases:**
- Closing auction participation
- EOD rebalancing
- Index tracking

**Notes:**
- Executes in closing auction
- Must be submitted before market close cutoff
- TIF must be DAY

#### 10. Market on Close (MOC)

**Description:** Market order executed at market close

**Parameters:**
```json
{
  "conid": 265598,
  "orderType": "MOC",
  "side": "BUY",
  "tif": "DAY",
  "quantity": 100
}
```

**Use Cases:**
- Guaranteed EOD execution
- Closing auction participation
- Benchmark tracking

**Notes:**
- Executes at closing price
- TIF must be DAY
- Cutoff times vary by exchange

#### 11. Market on Open (MOO)

**Description:** Market order executed at market open

**Parameters:**
```json
{
  "conid": 265598,
  "orderType": "MKT",
  "side": "BUY",
  "tif": "OPG",
  "quantity": 100
}
```

**Notes:**
- Use MKT order type with TIF=OPG
- Executes in opening auction
- Must be submitted before market open

#### 12. Limit on Open (LOO)

**Description:** Limit order executed at market open

**Parameters:**
```json
{
  "conid": 265598,
  "orderType": "LMT",
  "price": 185.00,
  "side": "BUY",
  "tif": "OPG",
  "quantity": 100
}
```

**Notes:**
- Use LMT order type with TIF=OPG
- Executes in opening auction if price available
- Price protection at open

### Forex-Specific Order Types

#### Forex Cash Quantity Order

**Description:** Order specified in cash amount rather than units

**Parameters:**
```json
{
  "conid": 12087792,
  "orderType": "LMT",
  "price": 1.0950,
  "side": "BUY",
  "tif": "GTC",
  "fxQty": 10000,
  "isCcyConv": true
}
```

**Fields:**
- `fxQty` - Cash quantity in base currency
- `isCcyConv` - Currency conversion flag

### Combo/Spread Orders

#### Combo Market Order

**Description:** Market order for multi-leg spreads

**Parameters:**
```json
{
  "conidex": "265598;1:8314;-1",
  "orderType": "MKT",
  "side": "BUY",
  "tif": "DAY",
  "quantity": 10
}
```

**Notes:**
- `conidex` format: `{conid};{ratio}:{conid};{ratio}`
- Positive ratio = long leg
- Negative ratio = short leg
- Used for options spreads, stock pairs, etc.

## Order Types NOT Available in Client Portal API

The following order types are **TWS API-exclusive** and cannot be used via Client Portal API:

- Market to Limit (MTL)
- Discretionary orders
- Auction orders (relative/pegged)
- Block orders
- Box Top orders
- Limit on Open IOC
- Limit on Close IOC
- Market with Protection
- Pegged to Market
- Pegged to Stock
- Relative orders
- Sweep to Fill
- Most algorithmic order types (except limited Adaptive support)

## Time in Force (TIF) Options

### DAY

**Description:** Valid for current trading day only

**Behavior:**
- Expires at market close
- Cancelled automatically if not filled
- Default for most order types

**Use Cases:**
- Day trading
- Intraday strategies
- When overnight risk not desired

### GTC (Good Til Cancelled)

**Description:** Remains active until filled or manually cancelled

**Behavior:**
- Persists across trading days
- May have exchange-specific limits (typically 90 days)
- Automatically cancelled at expiration limit

**Use Cases:**
- Long-term targets
- Patient entries
- Set-and-forget strategies

**Notes:**
- Monitor periodically for stale orders
- May need renewal after 90 days
- Consider corporate actions impact

### IOC (Immediate or Cancel)

**Description:** Fill immediately (full or partial), cancel remainder

**Behavior:**
- Attempts immediate execution
- Partial fills accepted
- Unfilled portion cancelled immediately
- No order remains on book

**Use Cases:**
- Testing liquidity
- Avoiding market impact
- Quick fills without leaving orders

**Notes:**
- Typically used with limit orders
- Good for large orders
- May result in partial fills

### OPG (At the Open)

**Description:** Participate in opening auction

**Behavior:**
- Queued for opening auction
- Executes at opening price (if applicable)
- Cancelled if not executed at open

**Use Cases:**
- Opening auction participation
- Gap trading
- Index tracking

**Order Types:**
- Market on Open: MKT + TIF=OPG
- Limit on Open: LMT + TIF=OPG

### GTD (Good Til Date)

**Description:** Valid until specified date

**Parameters:**
```json
{
  "conid": 265598,
  "orderType": "LMT",
  "price": 185.00,
  "side": "BUY",
  "tif": "GTD",
  "quantity": 100,
  "goodTillDate": "20240131"
}
```

**Date Format:** YYYYMMDD

**Use Cases:**
- Event-based trading (earnings, dividends)
- Specific time horizon strategies
- Automatic order management

## Outside Regular Trading Hours (RTH)

### Configuration

**Parameter:** `outsideRth`
**Type:** Boolean
**Default:** false

```json
{
  "conid": 265598,
  "orderType": "LMT",
  "price": 185.00,
  "side": "BUY",
  "tif": "GTC",
  "quantity": 100,
  "outsideRth": true
}
```

### Behavior

**When true:**
- Order active during pre-market (4:00 AM - 9:30 AM ET)
- Order active during after-hours (4:00 PM - 8:00 PM ET)
- Order active during regular hours (9:30 AM - 4:00 PM ET)

**When false:**
- Order only active during regular trading hours
- Automatically suspended outside RTH
- Reactivated at market open

### Restrictions

**Allowed Order Types Outside RTH:**
- Limit orders (LMT) - PRIMARY
- Some exchanges restrict to limit orders only

**Not Allowed:**
- Market orders (typically)
- Stop orders (typically)
- Complex order types

**Exchange Variations:**
- Check `orderTypesOutside` in contract rules
- Varies by exchange and instrument
- US stocks: Generally limit orders only

### Use Cases

- Pre-market news trading
- After-hours earnings reactions
- International time zone trading
- Extended liquidity access

### Risks

- Lower liquidity
- Wider spreads
- Higher volatility
- Limited order types

## Bracket Orders

### Overview

**Description:** Parent order with attached profit target and stop loss

**Components:**
1. **Parent Order:** Initial entry order
2. **Profit Target:** Limit order to take profit
3. **Stop Loss:** Stop order to limit loss

### Structure

```json
{
  "orders": [
    {
      "conid": 265598,
      "orderType": "MKT",
      "side": "BUY",
      "tif": "DAY",
      "quantity": 100
    },
    {
      "conid": 265598,
      "orderType": "LMT",
      "price": 190.00,
      "side": "SELL",
      "tif": "GTC",
      "quantity": 100,
      "parentId": 1
    },
    {
      "conid": 265598,
      "orderType": "STP",
      "price": 180.00,
      "side": "SELL",
      "tif": "GTC",
      "quantity": 100,
      "parentId": 1
    }
  ]
}
```

### Order Relationships

**Parent ID:**
- Child orders reference parent with `parentId`
- Parent ID = position in orders array (1-indexed)
- Example: First order = parentId: 1

**Execution Logic:**
1. Parent order submitted and (if applicable) executed
2. Child orders automatically submitted after parent fills
3. When one child fills, other child automatically cancelled (OCO behavior)

### One-Cancels-Other (OCO)

**Behavior:**
- Profit target and stop loss are OCO
- When one fills, the other is automatically cancelled
- Ensures only one exit per entry

### Use Cases

- Risk management
- Automated exits
- Trading plans with defined risk/reward
- Eliminating emotional decision-making

### Limitations

- Parent must fill before children are activated
- Children must reference same conid as parent
- All children cancelled if parent cancelled/rejected

## What-If Orders (Preview)

### Purpose

Preview order impact on account **without actual execution**

### Endpoint

```http
POST /iserver/account/{accountId}/whatiforder
```

### Request

```json
{
  "conid": 265598,
  "orderType": "MKT",
  "side": "BUY",
  "quantity": 100,
  "tif": "DAY"
}
```

### Response

```json
{
  "amount": {
    "amount": "-18550.00",
    "commission": "1.00",
    "total": "-18551.00"
  },
  "equity": {
    "current": "125000.50",
    "change": "-18551.00",
    "after": "106449.50"
  },
  "initial": {
    "current": "31250.13",
    "change": "4637.75",
    "after": "35887.88"
  },
  "maintenance": {
    "current": "25000.10",
    "change": "3710.20",
    "after": "28710.30"
  },
  "position": {
    "current": 0,
    "change": 100,
    "after": 100
  },
  "warn": ""
}
```

### Information Provided

**Financial Impact:**
- `amount` - Trade cost/proceeds
  - `amount` - Order value
  - `commission` - Estimated commission
  - `total` - Total impact

**Account Metrics:**
- `equity` - Equity with loan value impact
- `initial` - Initial margin requirement impact
- `maintenance` - Maintenance margin impact
- `position` - Position size impact

**Values:**
- `current` - Current value
- `change` - Expected change
- `after` - Value after order execution

### Use Cases

- Pre-trade risk assessment
- Margin requirement verification
- Order validation
- Impact analysis for large orders
- Educational/testing purposes

### Limitations

- Estimates only (actual execution may differ)
- Does not guarantee order acceptance
- Commission estimates may vary
- Market prices change

## Order Modification

### Modifiable Fields

**Can Modify:**
- Quantity (increase or decrease)
- Price (for limit/stop orders)
- Auxiliary price (for stop-limit orders)
- Time in force
- Outside RTH flag
- Trailing amount (for trailing stops)

**Cannot Modify:**
- Contract ID (conid)
- Order type (must cancel and replace)
- Side (must cancel and replace)

### Endpoint

```http
POST /iserver/account/{accountId}/order/{orderId}
```

### Request

```json
{
  "conid": 265598,
  "orderType": "LMT",
  "price": 186.00,
  "quantity": 150,
  "tif": "GTC"
}
```

### Behavior

- Order remains in queue (order ID unchanged)
- Modification may affect queue priority
- Partial fills retained
- Remaining quantity subject to new parameters

### Risks

- Loss of queue priority (exchange-dependent)
- May miss execution opportunity during modification
- Modification may be rejected

### Best Practices

- Verify order status before modification
- Consider cancel/replace for significant changes
- Monitor execution after modification

## Order Cancellation

### Simple Cancellation

**Endpoint:**
```http
DELETE /iserver/account/{accountId}/order/{orderId}
```

**Response:**
```json
{
  "msg": "Request was submitted",
  "conid": 265598,
  "order_id": 987654321
}
```

### Behavior

- Cancellation request sent to exchange
- Not guaranteed (order may fill first)
- Order status changes to "PendingCancel"
- Final status: "Cancelled" or "Filled"

### Partial Fills

- If partially filled, filled portion retained
- Only unfilled portion cancelled
- Position reflects fills prior to cancellation

### Cancel All

Programmatically cancel all orders:
1. Get all live orders: `GET /iserver/account/orders`
2. Iterate and cancel each: `DELETE /iserver/account/{accountId}/order/{orderId}`

**No batch cancel endpoint** - must cancel individually

## Order Confirmation Flow

### Confirmation Messages

Some orders require explicit confirmation before submission:

**Initial Response:**
```json
{
  "id": "reply-123abc",
  "message": [
    "This order will be placed on the next trading day.",
    "Are you sure you want to submit this order?"
  ]
}
```

**Common Confirmation Scenarios:**
- Orders outside regular trading hours
- Large order sizes
- Orders with high market impact
- First order of session
- Orders in volatile markets

### Confirmation Endpoint

```http
POST /iserver/reply/{replyId}
```

**Request Body:**
```json
{
  "confirmed": true
}
```

**Response:**
```json
{
  "order_id": "987654321",
  "order_status": "Submitted",
  "encrypt_message": "1"
}
```

### Implementation

```python
def place_order_with_confirmation(order):
    # Initial submission
    response = place_order(order)

    # Check if confirmation needed
    if 'id' in response and 'message' in response:
        reply_id = response['id']
        messages = response['message']

        # Log confirmation messages
        for msg in messages:
            print(f"Confirmation required: {msg}")

        # Send confirmation
        confirm_response = confirm_order(reply_id, confirmed=True)
        return confirm_response

    # No confirmation needed
    return response
```

## Order Status Lifecycle

### Status Flow

```
PendingSubmit -> PreSubmitted -> Submitted -> Filled
                                           -> PartiallyFilled -> Filled
                                           -> Cancelled
                                           -> Rejected
                                           -> Inactive
```

### Status Descriptions

**PendingSubmit:**
- Order created, awaiting submission
- Validation in progress
- May require confirmation

**PreSubmitted:**
- Order received by IBKR
- Awaiting transmission to exchange
- Not yet on exchange book

**Submitted:**
- Order on exchange book
- Awaiting execution
- Cancellable

**PartiallyFilled:**
- Partial execution occurred
- Remaining quantity still active
- May fully fill or cancel

**Filled:**
- Order completely executed
- Terminal state
- Check trades endpoint for execution details

**Cancelled:**
- Order cancelled (user request or system)
- Terminal state
- Partial fills may have occurred

**Rejected:**
- Order rejected by exchange or IBKR
- Terminal state
- Check error message for reason

**Inactive:**
- Order inactive (e.g., outside RTH with outsideRth=false)
- Will reactivate when conditions met
- Not a terminal state

**PendingCancel:**
- Cancellation requested
- Awaiting exchange confirmation
- May still fill

**ApiCancelled:**
- Cancelled via API request
- Terminal state
- Specific reason: API cancellation

### Monitoring Order Status

**REST Polling:**
```http
GET /iserver/account/orders
```

**WebSocket Streaming:**
```
sor+{}
```

**Best Practice:**
- Use WebSocket for real-time updates
- Avoid excessive REST polling (rate limits)
- Handle all status transitions gracefully

## Order Rejection Reasons

### Common Rejections

**Insufficient Funds:**
- Not enough buying power
- Margin requirement exceeded
- Solution: Reduce order size or add funds

**Market Closed:**
- Order submitted when market closed
- `outsideRth=false` and market outside RTH
- Solution: Wait for market open or set `outsideRth=true`

**Invalid Price:**
- Limit price outside exchange limits
- Price increment violation
- Solution: Check contract rules, adjust price

**Position Limits:**
- Exchange position limits exceeded
- Regulatory limits
- Solution: Reduce position or close other positions

**Contract Not Found:**
- Invalid conid
- Contract expired (derivatives)
- Solution: Verify conid, check expiration

**Order Size Violation:**
- Below minimum size
- Above maximum size
- Not a multiple of size increment
- Solution: Check contract rules (`sizeIncrement`)

**Time in Force Not Allowed:**
- TIF not supported for order type/exchange
- Solution: Check `tifTypes` in contract rules

**Order Type Not Allowed:**
- Order type not supported for contract
- Order type restricted outside RTH
- Solution: Check `orderTypes` and `orderTypesOutside`

### Handling Rejections

```python
def handle_order_response(response):
    if 'error' in response:
        error = response['error']
        print(f"Order rejected: {error}")

        # Parse error and take action
        if 'insufficient funds' in error.lower():
            # Handle insufficient funds
            pass
        elif 'market closed' in error.lower():
            # Handle market closed
            pass
        # ... etc

        return False

    # Order accepted
    order_id = response.get('order_id')
    print(f"Order accepted: {order_id}")
    return True
```

## Commissions and Fees

### Commission Structure

IBKR uses **tiered pricing** based on monthly volume:

**US Stocks (Tiered, USD):**
- 0 - 300,000 shares: $0.0035 per share
- 300,001 - 3,000,000: $0.0020 per share
- 3,000,001 - 20,000,000: $0.0015 per share
- 20,000,001 - 100,000,000: $0.0010 per share
- 100,000,001+: $0.0005 per share

**Minimum:** $0.35 per order
**Maximum:** 1% of trade value

**Fixed Pricing (Alternative):**
- $0.005 per share
- Minimum: $1.00 per order
- Maximum: 1% of trade value

**Note:** Actual commissions vary by account type, region, and instrument. Check IBKR pricing page for current rates.

### Fee Types

**Exchange Fees:**
- Pass-through fees from exchanges
- Vary by exchange and liquidity (adding vs removing)
- SEC fees, TAF fees (US stocks)

**Market Data Fees:**
- Real-time data subscription fees
- Vary by exchange and data type
- Required for live trading

**Regulatory Fees:**
- SEC fees (US stocks): $0.00278 per $1,000 of sale proceeds (2024)
- FINRA TAF: $0.000166 per share sold (max $7.27 per trade)
- Other regional regulatory fees

### Commission in Orders

**What-If Preview:**
Provides estimated commission before order submission

**Execution Report:**
Actual commission included in trade confirmation:
```json
{
  "commission": "1.00",
  "net_amount": 18553.00
}
```

**Calculation:**
- `net_amount` = (price × quantity × multiplier) + commission (for buys)
- `net_amount` = (price × quantity × multiplier) - commission (for sells)

## Position Management

### Position Sizing

**Whole Shares:**
- Standard for most stocks
- Fractional shares if account supports

**Contract Multiplier:**
- Options: Typically 100 (1 contract = 100 shares)
- Futures: Varies by contract
- Consider multiplier in position calculations

**Position Limits:**
- Check contract rules
- Exchange position limits
- Regulatory limits
- Account-specific limits

### Position Tracking

**REST API:**
```http
GET /portfolio/{accountId}/positions/{page}
```

**WebSocket:**
Subscribe to account updates: `acc+{}`

### Position Reconciliation

**Best Practices:**
- Periodically fetch full position snapshot
- Compare with internal position tracking
- Handle corporate actions (splits, dividends)
- Account for execution fills

**Discrepancies:**
- Check execution reports for missing fills
- Verify order cancellations were successful
- Consider pending orders (not yet filled)

### Closing Positions

**Market Order:**
- Fastest execution
- Side opposite to position (long -> SELL, short -> BUY)

**Limit Order:**
- Price control
- May not fill immediately

**Partial Close:**
- Specify quantity less than position size
- Remaining position stays open

## Risk Management Features

### Stop Loss Orders

**Simple Stop:**
```json
{
  "orderType": "STP",
  "price": 180.00,
  "side": "SELL"
}
```

**Stop Limit:**
```json
{
  "orderType": "STP_LMT",
  "price": 179.50,
  "auxPrice": 180.00,
  "side": "SELL"
}
```

**Trailing Stop:**
```json
{
  "orderType": "TRAIL",
  "trailingAmt": 2.00,
  "trailingType": "amt",
  "side": "SELL"
}
```

### Take Profit Orders

**Limit Order:**
```json
{
  "orderType": "LMT",
  "price": 190.00,
  "side": "SELL",
  "tif": "GTC"
}
```

**Bracket Order:**
Combines entry, stop loss, and take profit

### Position Limits

**Monitor:**
- Check position size vs account equity
- Diversification across instruments
- Sector/industry concentration

**Implement:**
- Pre-trade checks in application logic
- What-if orders for margin verification
- Position size calculators

### Margin Requirements

**Check Before Trading:**
```http
POST /iserver/account/{accountId}/whatiforder
```

**Monitor:**
```http
GET /portfolio/{accountId}/summary
```

**Key Metrics:**
- Initial margin requirement
- Maintenance margin requirement
- Available funds
- Excess liquidity
- Cushion (%)

### Maximum Loss Calculation

**Per Trade:**
- Entry price - Stop price = max loss per share
- Max loss per share × quantity = total max loss
- Verify against account risk tolerance

**Portfolio Level:**
- Sum of all position risks
- Account for correlation
- Stress test scenarios

## Best Practices

### Order Placement

1. **Validate Contract:** Verify conid before ordering
2. **Check Rules:** Get trading rules via contract info endpoint
3. **Preview Order:** Use what-if order for large trades
4. **Set Stops:** Always define risk before entering
5. **Monitor Execution:** Track order status via WebSocket
6. **Confirm Fills:** Verify execution via trades endpoint

### Error Handling

1. **Retry Logic:** Implement for transient errors
2. **Exponential Backoff:** For rate limit errors
3. **Order Validation:** Pre-validate before submission
4. **Status Monitoring:** Track order lifecycle completely
5. **Logging:** Log all orders and responses

### Performance Optimization

1. **WebSocket for Updates:** Don't poll order status
2. **Batch Validations:** Check multiple contracts together
3. **Cache Contract Info:** Reuse trading rules
4. **Connection Pooling:** Reuse HTTP connections
5. **Async Requests:** Use async I/O for scalability

---

**Research Date:** 2026-01-26
**API Version:** v1.0
**Trading Features:** Comprehensive, Multi-Asset Support
