# Futu OpenAPI - WebSocket Documentation

## Availability: Custom TCP Protocol (Not Standard WebSocket)

**Important**: Futu does NOT use standard WebSocket (wss://). Instead, it uses a **custom TCP protocol with Protocol Buffers**.

However, the functionality is similar to WebSocket - persistent connection with bidirectional communication.

## Connection

### URLs
- **Not applicable** - Connects to OpenD gateway via TCP
- Default host: `127.0.0.1` (local OpenD)
- Default port: `11111` (configurable)
- Can connect to remote OpenD: `remote_ip:port`

### Connection Configuration

```python
# Python SDK Example
from futu import *

# Quote Context (market data)
quote_ctx = OpenQuoteContext(host='127.0.0.1', port=11111)

# Trade Context (trading)
trade_ctx = OpenSecTradeContext(host='127.0.0.1', port=11111)
# For futures trading: OpenFutureTradeContext
# For HK options trading: OpenHKTradeContext
# For US options trading: OpenUSTradeContext
```

### Connection Process
1. Instantiate context (QuoteContext or TradeContext)
2. Connection automatically established on first API call
3. For trading: Must call `unlock_trade()` after connection
4. Optional: Call `start()` to enable asynchronous push callbacks
5. Must call `close()` when finished to release resources

### Connection Parameters
| Parameter | Default | Description |
|-----------|---------|-------------|
| host | '127.0.0.1' | OpenD gateway IP |
| port | 11111 | OpenD gateway port |
| is_encrypt | None | Enable encryption (optional) |
| security_firm | SecurityFirm.FUTUSECURITIES | Broker selection |
| is_async_connect | False | Async connection mode |

## Subscription Mechanism (Similar to WebSocket Channels)

Instead of WebSocket channels, Futu uses a **subscription system** via `subscribe()` method.

### Available Subscription Types

| SubType | Description | Push Frequency | Auth Required | Quota Cost |
|---------|-------------|----------------|---------------|------------|
| `QUOTE` | Basic quote data | On change | Yes (LV1+) | 1 per security |
| `ORDER_BOOK` | Order book depth | On change | Yes (LV1+) | 1 per security |
| `TICKER` | Trade tick-by-tick | Per trade | Yes (LV1+) | 1 per security |
| `K_1M` | 1-minute K-line | On bar close/update | Yes (LV1+) | 1 per security |
| `K_3M` | 3-minute K-line | On bar close/update | Yes (LV1+) | 1 per security |
| `K_5M` | 5-minute K-line | On bar close/update | Yes (LV1+) | 1 per security |
| `K_15M` | 15-minute K-line | On bar close/update | Yes (LV1+) | 1 per security |
| `K_30M` | 30-minute K-line | On bar close/update | Yes (LV1+) | 1 per security |
| `K_60M` | 60-minute K-line | On bar close/update | Yes (LV1+) | 1 per security |
| `K_DAY` | Daily K-line | On bar close/update | Yes (LV1+) | 1 per security |
| `K_WEEK` | Weekly K-line | On bar close/update | Yes (LV1+) | 1 per security |
| `K_MON` | Monthly K-line | On bar close/update | Yes (LV1+) | 1 per security |
| `RT_DATA` | Real-time time frame | Per minute tick | Yes (LV1+) | 1 per security |
| `BROKER` | Broker queue (HK) | On queue change | Yes (LV2) | 1 per security |

### Subscription Format

**Subscribe Message:**
```python
ret, err = quote_ctx.subscribe(
    code_list=['US.AAPL', 'HK.00700'],
    subtype_list=[SubType.QUOTE, SubType.TICKER, SubType.K_1M, SubType.ORDER_BOOK],
    is_first_push=True,      # Push cached data immediately
    subscribe_push=True,     # Enable real-time push
    is_detailed_orderbook=False,  # Simple order book
    extended_time=False,     # US extended hours (pre/post market)
    session=Session.NONE     # US trading session
)

if ret == RET_OK:
    print("Subscribed successfully")
else:
    print(f"Subscription failed: {err}")
```

**Unsubscribe Message:**
```python
ret, err = quote_ctx.unsubscribe(
    code_list=['US.AAPL'],
    subtype_list=[SubType.QUOTE, SubType.TICKER]
)
```

**Unsubscribe All:**
```python
ret, err = quote_ctx.unsubscribe_all()
```

**Check Subscription Status:**
```python
ret, data = quote_ctx.query_subscription()
# Returns DataFrame with current subscriptions
```

### Subscription Confirmation
The return tuple `(ret, err)` indicates success:
- `ret == RET_OK` (0): Success
- `ret == RET_ERROR` (-1): Failure, check `err` message

## Message Formats (Push Callbacks)

Futu uses **callback handlers** instead of WebSocket message listeners.

### Setting Up Callbacks

```python
from futu import *

class QuoteHandler(StockQuoteHandlerBase):
    def on_recv_rsp(self, rsp_pb):
        # Handle quote update
        ret_code, data = super(QuoteHandler, self).on_recv_rsp(rsp_pb)
        if ret_code != RET_OK:
            print(f"Error: {data}")
            return RET_ERROR, data

        # Process quote data (DataFrame)
        print(data)
        return RET_OK, data

class TickerHandler(TickerHandlerBase):
    def on_recv_rsp(self, rsp_pb):
        # Handle tick-by-tick update
        ret_code, data = super(TickerHandler, self).on_recv_rsp(rsp_pb)
        if ret_code != RET_OK:
            print(f"Error: {data}")
            return RET_ERROR, data

        print(data)  # Ticker DataFrame
        return RET_OK, data

class OrderBookHandler(OrderBookHandlerBase):
    def on_recv_rsp(self, rsp_pb):
        # Handle order book update
        ret_code, data = super(OrderBookHandler, self).on_recv_rsp(rsp_pb)
        if ret_code != RET_OK:
            print(f"Error: {data}")
            return RET_ERROR, data

        print(data)  # OrderBook DataFrame
        return RET_OK, data

# Register handlers
quote_ctx.set_handler(QuoteHandler())
quote_ctx.set_handler(TickerHandler())
quote_ctx.set_handler(OrderBookHandler())

# Start async receiving
quote_ctx.start()
```

### Quote Update (QUOTE SubType)

**Push Data (DataFrame format):**
```python
# Columns:
code               # str - Security code (e.g., "US.AAPL")
data_date          # str - Date (YYYY-MM-DD)
data_time          # str - Time (HH:mm:ss)
last_price         # float - Latest price
open_price         # float - Open price
high_price         # float - High price (today)
low_price          # float - Low price (today)
prev_close_price   # float - Previous close
volume             # int - Volume
turnover           # float - Turnover amount
turnover_rate      # float - Turnover rate (%)
amplitude          # float - Amplitude (%)
suspension         # bool - Suspended?
price_spread       # float - Bid-ask spread
dark_status        # DarkStatus - Dark pool status (HK)
sec_status         # SecurityStatus - Security trading status
strike_price       # float - Strike (options only)
contract_size      # float - Contract size (options only)
open_interest      # int - Open interest (options/futures)
implied_volatility # float - IV (options only)
premium            # float - Premium (options only)
delta              # float - Delta (options only)
gamma              # float - Gamma (options only)
vega               # float - Vega (options only)
theta              # float - Theta (options only)
rho                # float - Rho (options only)
index_option_type  # IndexOptionType - Index option type
net_open_interest  # int - Net open interest change
expiry_date_distance  # int - Days to expiry (options)
contract_nominal_value  # float - Contract value
owner_lot_multiplier    # float - Multiplier
option_area_type   # OptionAreaType - Option area
contract_multiplier  # float - Contract multiplier
last_settle_price  # float - Last settlement price (futures)
position           # float - Position quantity
position_change    # float - Position change
```

### Trade Update (TICKER SubType)

**Push Data (DataFrame format):**
```python
# Columns:
code               # str - Security code
sequence           # int - Sequence number
time               # str - Trade time (HH:mm:ss)
price              # float - Trade price
volume             # int - Trade volume
turnover           # float - Trade amount
ticker_direction   # TickerDirection - BUY, SELL, NEUTRAL
type               # TickerType - AUTO_MATCH, LATE, NON_AUTO_MATCH, etc.
push_data_type     # PushDataType - REALTIME or CACHE
```

**TickerDirection Enum:**
- `BUY`: Buyer initiated
- `SELL`: Seller initiated
- `NEUTRAL`: Unknown direction

### Order Book Update (ORDER_BOOK SubType)

**Push Data (DataFrame format):**
```python
# Columns:
code               # str - Security code
svr_recv_time_bid  # str - Server receive time (bid side)
svr_recv_time_ask  # str - Server receive time (ask side)

# For each level (1 to N, where N depends on quote level):
Bid[i]             # float - Bid price at level i
BidVolume[i]       # int - Bid volume at level i
BidOrderNum[i]     # int - Number of orders at level i (if detailed)
Ask[i]             # float - Ask price at level i
AskVolume[i]       # int - Ask volume at level i
AskOrderNum[i]     # int - Number of orders at level i (if detailed)
```

**Depth Levels:**
- **US Stocks**: Up to 60 levels with LV2 data
- **US Futures**: Up to 40 levels
- **HK Stocks**: 10 levels (LV1), 40 levels (LV2)
- **A-Shares**: 10 levels (LV1)

**Update Types:**
- Snapshot: Full order book
- Delta: Only changed levels (if supported)

### Candlestick Update (K_* SubTypes)

**Push Data (DataFrame format):**
```python
# Columns:
code               # str - Security code
time_key           # str - Bar timestamp (YYYY-MM-DD HH:mm:ss)
open               # float - Open price
close              # float - Close price
high               # float - High price
low                # float - Low price
volume             # int - Volume
turnover           # float - Turnover amount
k_type             # KLType - Candlestick type
last_close         # float - Previous bar close
```

**K-line Types:**
- `K_1M`, `K_3M`, `K_5M`, `K_15M`, `K_30M`, `K_60M`: Intraday bars
- `K_DAY`: Daily bars
- `K_WEEK`: Weekly bars
- `K_MON`: Monthly bars

### Time Frame Update (RT_DATA SubType)

**Push Data (DataFrame format):**
```python
# Columns:
code               # str - Security code
time               # str - Time point (HH:mm)
is_blank           # bool - Is blank point? (no trades)
opened_mins        # int - Minutes since market open
cur_price          # float - Current price
last_close         # float - Previous close
avg_price          # float - Average price
volume             # int - Cumulative volume
turnover           # float - Cumulative turnover
```

### Broker Queue Update (BROKER SubType, HK only)

**Push Data (DataFrame format):**
```python
# Columns:
code               # str - Security code
bid_broker_id      # int - Bid broker ID
bid_broker_name    # str - Bid broker name
bid_broker_pos     # int - Bid broker position in queue
ask_broker_id      # int - Ask broker ID
ask_broker_name    # str - Ask broker name
ask_broker_pos     # int - Ask broker position in queue
```

## Heartbeat / Ping-Pong

### Who initiates?
- **Server → Client ping**: No (not applicable with TCP protocol)
- **Client → Server ping**: Not required (TCP keepalive handles connection)

### Connection Management
- TCP connection is persistent
- OpenD gateway manages connection health
- No explicit ping/pong messages required
- Context has `get_global_state()` to check connection status

### Timeout Handling
- TCP socket timeout: Configurable at OS level
- Connection drops: Context will return error on next operation
- Reconnection: Manual - create new context

### Keep-Alive
```python
# Check connection status periodically
ret, data = quote_ctx.get_global_state()
if ret != RET_OK:
    print("Connection lost, reconnect required")
    # Re-create context
```

## Connection Limits

### Subscription Quota Limits (Most Important)
| User Tier | Max Subscriptions | Historical K-line Quota (30 days) |
|-----------|------------------|----------------------------------|
| Basic | 100 | 100 |
| Assets >10K HKD | 300 | 300 |
| High Volume | 1,000 | 1,000 |
| Premium | 2,000 | 2,000 |

**Subscription Cost:**
- Each security costs **1 subscription quota** per SubType
- Example: `US.AAPL` with `[QUOTE, TICKER, K_1M]` = 3 quotas

**Special Cases:**
- SF (Singapore/Japan futures) authorized users: Limited to **50 securities** regardless of quota

### Connection Limits
- **Max connections per account**: Not explicitly limited (reasonable use)
- **Concurrent contexts**: Multiple contexts allowed (Quote + Trade simultaneously)
- **Remote OpenD**: Single OpenD can serve multiple client connections

### Subscription Management
- **Unsubscribe wait time**: Minimum **1 minute** after subscription before unsubscribe
- **Re-subscription**: Immediate after unsubscription
- **Subscription persistence**: Survives client disconnect if OpenD stays running

### Message Rate Limits
- **Push messages**: No rate limit (server-controlled)
- **Pull requests**: See endpoints_full.md for per-method limits
- **High-frequency gets**: No limit after subscription (e.g., `get_stock_quote()`)

### Connection Duration
- **Max lifetime**: Unlimited (persistent connection)
- **Auto-reconnect**: Not built-in, must handle manually
- **Idle timeout**: None (connection persists)
- **OpenD restart**: Requires client reconnection

## Authentication (for private data)

### Connection Authentication
OpenD itself handles authentication to Futu servers. Client connects to OpenD without credentials.

### OpenD Authentication
1. OpenD logs in to Futu servers using:
   - Account credentials (Futu ID or moomoo ID)
   - Password
   - Two-factor authentication (if enabled)

2. OpenD maintains session

3. Client connects to OpenD via:
   - Plain TCP (local: 127.0.0.1)
   - Encrypted TCP (remote: requires RSA key)

### Trading Authentication
**Unlock Trade** required before placing orders:
```python
ret, err = trade_ctx.unlock_trade(password="trade_password")
if ret == RET_OK:
    print("Trade unlocked")
else:
    print(f"Unlock failed: {err}")
```

**Trade password**: Set in OpenD configuration or via API

### Auth Success/Failure
**Connection Success:**
- First API call returns `RET_OK`
- No explicit auth success message

**Connection Failure:**
```python
ret, data = quote_ctx.get_stock_quote(['US.AAPL'])
if ret == RET_ERROR:
    print(f"Error: {data}")  # Error message describes issue
```

**Common Auth Errors:**
- "not login": OpenD not authenticated to Futu servers
- "unlock trade fail": Wrong trade password
- "no trade permission": Account lacks trading permission for market

## Trading Push Notifications (Similar to Private WebSocket)

### Order Status Updates

```python
from futu import *

class OrderHandler(TradeOrderHandlerBase):
    def on_recv_rsp(self, rsp_pb):
        ret_code, data = super(OrderHandler, self).on_recv_rsp(rsp_pb)
        if ret_code != RET_OK:
            print(f"Error: {data}")
            return RET_ERROR, data

        # Order update DataFrame
        print(data)
        return RET_OK, data

# Register handler
trade_ctx.set_handler(OrderHandler())

# Must subscribe to account push
ret, err = trade_ctx.sub_acc_push()
if ret == RET_OK:
    print("Subscribed to account updates")
```

**Order Update Fields:**
```python
# Columns:
code               # str - Security code
stock_name         # str - Security name
order_id           # str - Order ID
order_type         # OrderType - Order type
order_status       # OrderStatus - Current status
price              # float - Order price
qty                # float - Order quantity
dealt_qty          # float - Filled quantity
dealt_avg_price    # float - Average fill price
create_time        # str - Order creation time
updated_time       # str - Last update time
trd_side           # TrdSide - BUY or SELL
trd_market         # TrdMarket - Market (HK, US, etc.)
currency           # str - Currency
filled_qty         # float - Filled quantity (alias)
last_err_msg       # str - Error message if any
remark             # str - User remark
time_in_force      # TimeInForce - DAY, GTC, etc.
fill_outside_rth   # bool - Fill outside regular hours?
```

### Deal/Fill Updates

```python
class DealHandler(TradeDealHandlerBase):
    def on_recv_rsp(self, rsp_pb):
        ret_code, data = super(DealHandler, self).on_recv_rsp(rsp_pb)
        if ret_code != RET_OK:
            print(f"Error: {data}")
            return RET_ERROR, data

        # Deal update DataFrame
        print(data)
        return RET_OK, data

# Register handler
trade_ctx.set_handler(DealHandler())
```

**Deal Update Fields:**
```python
# Columns:
code               # str - Security code
stock_name         # str - Security name
deal_id            # str - Deal ID
order_id           # str - Associated order ID
qty                # float - Deal quantity
price              # float - Deal price
trd_side           # TrdSide - BUY or SELL
create_time        # str - Deal time
counter_broker_id  # int - Counter broker ID (HK)
counter_broker_name  # str - Counter broker name (HK)
```

## Error Handling

### Common Errors

| Error Code | Message | Cause | Solution |
|------------|---------|-------|----------|
| RET_ERROR (-1) | "not login" | OpenD not logged in | Check OpenD status, login |
| RET_ERROR (-1) | "subscription limit" | Quota exceeded | Unsubscribe unused securities |
| RET_ERROR (-1) | "subscribe failed: no authority" | Insufficient quote level | Purchase quote card |
| RET_ERROR (-1) | "unlock trade fail" | Wrong trade password | Check password in OpenD config |
| RET_ERROR (-1) | "freq limit" | Rate limit exceeded | Wait and retry |
| RET_ERROR (-1) | "not subscribed" | Called getter without subscription | Subscribe first |

### Disconnection Handling

```python
try:
    ret, data = quote_ctx.get_stock_quote(['US.AAPL'])
    if ret != RET_OK:
        # Handle error
        print(f"Error: {data}")
        # Reconnect logic
        quote_ctx.close()
        quote_ctx = OpenQuoteContext(host='127.0.0.1', port=11111)
except Exception as e:
    print(f"Exception: {e}")
    # Reconnect
```

## Best Practices

1. **Subscription Management**:
   - Subscribe only to needed securities/subtypes
   - Monitor quota usage with `query_subscription()`
   - Unsubscribe unused data to free quotas

2. **Callback Handlers**:
   - Keep handlers lightweight (fast processing)
   - Avoid blocking operations in callbacks
   - Return `RET_OK` from handlers to continue receiving

3. **Connection Management**:
   - Always call `close()` when done
   - Handle reconnection gracefully
   - Check `get_global_state()` periodically

4. **Error Handling**:
   - Always check `ret` code before using `data`
   - Log errors for debugging
   - Implement retry logic with exponential backoff

5. **Resource Cleanup**:
   ```python
   try:
       # Use context
       pass
   finally:
       quote_ctx.close()
       trade_ctx.close()
   ```

## Notes

- **Not Standard WebSocket**: Custom TCP protocol, but similar concepts
- **Protocol Buffers**: Binary serialization for efficiency
- **Push + Pull**: Can push (callbacks) or pull (getters) after subscription
- **Multi-Context**: Separate contexts for Quote and Trade
- **OpenD Gateway**: Central component managing connections
- **Subscription-Based**: Must subscribe before receiving real-time data
