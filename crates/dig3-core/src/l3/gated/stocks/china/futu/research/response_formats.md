# Futu OpenAPI - Response Formats

**Note**: Futu SDKs return data in language-specific formats (Python DataFrame, C# DataTable, etc.). Below shows Python examples (most common), but structure is similar across languages.

All methods return a tuple: `(ret_code, data)`
- `ret_code`: RET_OK (0) on success, RET_ERROR (-1) on failure
- `data`: DataFrame/dict on success, error message string on failure

---

## Market Data Responses

### get_stock_quote()

**Returns**: DataFrame

**Columns**:
```python
{
    'code': str,                    # Security code ('US.AAPL')
    'data_date': str,               # Date ('2024-01-15')
    'data_time': str,               # Time ('16:00:00')
    'last_price': float,            # Latest price (150.25)
    'open_price': float,            # Open (149.80)
    'high_price': float,            # High (150.50)
    'low_price': float,             # Low (149.50)
    'prev_close_price': float,      # Previous close (148.00)
    'volume': int,                  # Volume (12345678)
    'turnover': float,              # Turnover amount (1850000000.0)
    'turnover_rate': float,         # Turnover rate % (0.75)
    'amplitude': float,             # Amplitude % (1.35)
    'suspension': bool,             # Suspended? (False)
    'price_spread': float,          # Bid-ask spread (0.02)
    'dark_status': str,             # Dark pool status ('NONE', HK only)
    'sec_status': str,              # Security status ('NORMAL')

    # Options/Futures specific (if applicable)
    'strike_price': float,          # Strike price (options)
    'contract_size': float,         # Contract size (100)
    'open_interest': int,           # Open interest
    'implied_volatility': float,    # IV (options)
    'premium': float,               # Premium (options)
    'delta': float,                 # Delta (options)
    'gamma': float,                 # Gamma (options)
    'vega': float,                  # Vega (options)
    'theta': float,                 # Theta (options)
    'rho': float,                   # Rho (options)
    'net_open_interest': int,       # Net OI change
    'expiry_date_distance': int,    # Days to expiry
    'contract_nominal_value': float,# Contract value
    'owner_lot_multiplier': float,  # Multiplier
    'option_area_type': str,        # 'AMERICAN'/'EUROPEAN'
    'contract_multiplier': float,   # Multiplier
    'index_option_type': str,       # Index option type
    'last_settle_price': float,     # Last settlement (futures)
    'position': float,              # Position quantity
    'position_change': float        # Position change
}
```

**Example**:
```python
ret, data = quote_ctx.get_stock_quote(['US.AAPL', 'HK.00700'])

# ret = RET_OK
# data (DataFrame):
#        code  data_date  data_time  last_price  open_price  high_price  ...
# 0  US.AAPL 2024-01-15   16:00:00      150.25      149.80      150.50  ...
# 1  HK.00700 2024-01-15   16:08:00      348.60      345.00      349.20  ...
```

---

### get_order_book()

**Returns**: Dictionary (not DataFrame)

**Structure**:
```python
{
    'code': str,                    # Security code ('US.AAPL')
    'name': str,                    # Security name ('APPLE')
    'svr_recv_time_bid': str,       # Server time bid ('2024-01-15 16:00:00.123')
    'svr_recv_time_ask': str,       # Server time ask ('2024-01-15 16:00:00.123')
    'Bid': [                        # Bid list (sorted descending)
        (price, volume, order_count, order_details),
        ...
    ],
    'Ask': [                        # Ask list (sorted ascending)
        (price, volume, order_count, order_details),
        ...
    ]
}
```

**Bid/Ask Tuple Format**:
- `price` (float): Price level
- `volume` (int): Total volume at price
- `order_count` (int): Number of orders
- `order_details` (dict): Order ID → volume mapping (HK SF only, otherwise {})

**Example**:
```python
ret, data = quote_ctx.get_order_book('US.AAPL')

# ret = RET_OK
# data:
{
    'code': 'US.AAPL',
    'name': 'APPLE',
    'svr_recv_time_bid': '2024-01-15 16:00:00.352',
    'svr_recv_time_ask': '2024-01-15 16:00:00.352',
    'Bid': [
        (150.25, 1000, 3, {}),   # $150.25: 1000 shares, 3 orders
        (150.24, 500, 2, {}),
        (150.23, 750, 1, {})
    ],
    'Ask': [
        (150.26, 800, 2, {}),    # $150.26: 800 shares, 2 orders
        (150.27, 1200, 4, {}),
        (150.28, 500, 1, {})
    ]
}
```

---

### get_cur_kline()

**Returns**: DataFrame

**Columns**:
```python
{
    'code': str,                    # Security code
    'time_key': str,                # Bar timestamp ('2024-01-15 16:00:00')
    'open': float,                  # Open price
    'close': float,                 # Close price
    'high': float,                  # High price
    'low': float,                   # Low price
    'volume': int,                  # Volume
    'turnover': float,              # Turnover amount
    'k_type': str,                  # K-line type ('K_1M', 'K_DAY', etc.)
    'last_close': float             # Previous bar close
}
```

**Example**:
```python
ret, data = quote_ctx.get_cur_kline('US.AAPL', 3, KLType.K_1M)

# ret = RET_OK
# data (DataFrame):
#        code            time_key    open   close    high     low   volume  ...
# 0  US.AAPL  2024-01-15 15:58:00  150.20  150.25  150.30  150.18  125000  ...
# 1  US.AAPL  2024-01-15 15:59:00  150.25  150.22  150.28  150.20  110000  ...
# 2  US.AAPL  2024-01-15 16:00:00  150.22  150.30  150.35  150.20  180000  ...
```

---

### request_history_kline()

**Returns**: DataFrame (same structure as get_cur_kline)

**Columns**: Same as `get_cur_kline()`

**Additional**: Supports pagination via `page_req_key`

**Example**:
```python
ret, data, page_req_key = quote_ctx.request_history_kline(
    'US.AAPL',
    start='2023-01-01',
    end='2023-12-31',
    ktype=KLType.K_DAY
)

# ret = RET_OK
# data (DataFrame): 252 rows (trading days)
# page_req_key: None (if all data fetched) or str (for next page)
```

---

### get_rt_ticker()

**Returns**: DataFrame

**Columns**:
```python
{
    'code': str,                    # Security code
    'sequence': int,                # Sequence number (increasing)
    'time': str,                    # Trade time ('16:00:01')
    'price': float,                 # Trade price
    'volume': int,                  # Trade volume
    'turnover': float,              # Trade amount
    'ticker_direction': str,        # 'BUY', 'SELL', 'NEUTRAL'
    'type': str,                    # 'AUTO_MATCH', 'LATE', etc.
    'push_data_type': str           # 'REALTIME' or 'CACHE'
}
```

**Example**:
```python
ret, data = quote_ctx.get_rt_ticker('US.AAPL')

# ret = RET_OK
# data (DataFrame):
#        code  sequence      time   price  volume  turnover ticker_direction  ...
# 0  US.AAPL  12345678  16:00:01  150.25     100   15025.0              BUY  ...
# 1  US.AAPL  12345679  16:00:01  150.24     200   30048.0             SELL  ...
```

---

### get_rt_data() (Time Frame)

**Returns**: DataFrame

**Columns**:
```python
{
    'code': str,                    # Security code
    'time': str,                    # Time point ('09:31')
    'is_blank': bool,               # No trades in this minute?
    'opened_mins': int,             # Minutes since market open
    'cur_price': float,             # Current price
    'last_close': float,            # Previous close
    'avg_price': float,             # Average price (VWAP)
    'volume': int,                  # Cumulative volume
    'turnover': float               # Cumulative turnover
}
```

**Example**:
```python
ret, data = quote_ctx.get_rt_data('US.AAPL')

# ret = RET_OK
# data (DataFrame):
#        code   time  is_blank  opened_mins  cur_price  last_close  ...
# 0  US.AAPL  09:30     False            0     149.80      148.00  ...
# 1  US.AAPL  09:31     False            1     149.85      148.00  ...
# 2  US.AAPL  09:32      True            2     149.85      148.00  ...  # No trades
```

---

### get_broker_queue() (HK only, LV2)

**Returns**: DataFrame

**Columns**:
```python
{
    'code': str,                    # Security code
    'bid_broker_id': int,           # Bid broker ID
    'bid_broker_name': str,         # Bid broker name
    'bid_broker_pos': int,          # Bid broker position in queue
    'ask_broker_id': int,           # Ask broker ID
    'ask_broker_name': str,         # Ask broker name
    'ask_broker_pos': int           # Ask broker position in queue
}
```

**Example**:
```python
ret, data = quote_ctx.get_broker_queue('HK.00700')

# ret = RET_OK
# data (DataFrame):
#        code  bid_broker_id bid_broker_name  bid_broker_pos  ...
# 0  HK.00700           5555      Goldman Sachs               1  ...
# 1  HK.00700           8888      Morgan Stanley              2  ...
```

---

### get_market_snapshot()

**Returns**: DataFrame (combines multiple data points)

**Columns**: Similar to `get_stock_quote()` but for multiple securities in batch

**Example**:
```python
ret, data = quote_ctx.get_market_snapshot(['US.AAPL', 'US.GOOGL', 'US.MSFT'])

# ret = RET_OK
# data (DataFrame): 3 rows with full quote data
```

---

### get_stock_basicinfo()

**Returns**: DataFrame

**Columns**:
```python
{
    'code': str,                    # Security code
    'name': str,                    # Security name
    'lot_size': int,                # Lot size (shares per lot, HK)
    'stock_type': str,              # 'STOCK', 'ETF', 'WARRANT', etc.
    'stock_child_type': str,        # More specific type
    'stock_owner': str,             # Parent stock (warrants/options)
    'option_type': str,             # 'CALL', 'PUT' (options only)
    'strike_time': str,             # Expiration date (options)
    'strike_price': float,          # Strike price (options)
    'suspension': bool,             # Is suspended?
    'listing_date': str,            # Listing date ('2010-05-01')
    'delisting': bool,              # Is delisted?
    'stock_id': int,                # Unique stock ID
    'main_contract': bool,          # Is main contract? (futures)
    'last_trade_time': str          # Last trading date
}
```

**Example**:
```python
ret, data = quote_ctx.get_stock_basicinfo(Market.HK, SecurityType.STOCK)

# ret = RET_OK
# data (DataFrame): 2,593+ HK stocks
#        code      name  lot_size stock_type  listing_date  ...
# 0  HK.00001      CKH       500      STOCK    1972-11-06  ...
# 1  HK.00002  CKHHOLDINGS 500      STOCK    2015-03-18  ...
```

---

## Trading Responses

### get_acc_list()

**Returns**: DataFrame

**Columns**:
```python
{
    'acc_id': int,                  # Account ID
    'trd_env': str,                 # 'REAL' or 'SIMULATE'
    'trd_market': str,              # 'HK', 'US', 'CN', 'HKCC', etc.
    'acc_type': str,                # 'CASH', 'MARGIN'
    'card_num': str,                # Account number
    'security_firm': str,           # 'FUTUSECURITIES', 'MOOMOO', etc.
    'sim_acc_type': str,            # 'STOCK', 'OPTION', 'FUTURES' (simulated)
    'trdmarket_auth': list          # List of authorized markets
}
```

**Example**:
```python
ret, data = trade_ctx.get_acc_list()

# ret = RET_OK
# data (DataFrame):
#   acc_id trd_env trd_market acc_type    card_num  security_firm  ...
# 0  12345    REAL         HK     CASH  1234567890  FUTUSECURITIES  ...
# 1  67890    REAL         US   MARGIN  9876543210         MOOMOO  ...
# 2  11111 SIMULATE         HK     CASH  SIM123456  FUTUSECURITIES  ...
```

---

### get_funds() (accinfo_query)

**Returns**: DataFrame

**Columns**:
```python
{
    'total_assets': float,          # Total assets
    'cash': float,                  # Cash
    'available_funds': float,       # Available for withdrawal
    'frozen_cash': float,           # Frozen in orders
    'market_val': float,            # Securities market value
    'net_cash_power': float,        # Net buying power (cash)
    'long_mv': float,               # Long position value
    'short_mv': float,              # Short position value (margin)
    'max_power_short': float,       # Max short buying power
    'max_withdraw': float,          # Max withdrawable
    'currency': str,                # Currency ('HKD', 'USD', etc.)
    'unrealized_pl': float,         # Unrealized P&L
    'realized_pl': float,           # Realized P&L
    'margin_call_margin': float,    # Margin call threshold
    'initial_margin': float,        # Initial margin required
    'maintenance_margin': float     # Maintenance margin required
}
```

**Example**:
```python
ret, data = trade_ctx.accinfo_query()

# ret = RET_OK
# data (DataFrame):
#   total_assets      cash  available_funds  market_val  ...
# 0     100000.00  30000.00         25000.00    70000.00  ...
```

---

### get_position_list() (position_list_query)

**Returns**: DataFrame

**Columns**:
```python
{
    'code': str,                    # Security code
    'stock_name': str,              # Security name
    'qty': float,                   # Total quantity held
    'can_sell_qty': float,          # Available to sell
    'cost_price': float,            # Average cost price
    'cost_price_valid': bool,       # Is cost price valid?
    'market_val': float,            # Market value
    'nominal_price': float,         # Current market price
    'pl_ratio': float,              # P&L ratio (%)
    'pl_ratio_valid': bool,         # Is P&L ratio valid?
    'pl_val': float,                # P&L value
    'pl_val_valid': bool,           # Is P&L value valid?
    'today_pl_val': float,          # Today's P&L
    'today_buy_qty': float,         # Today's buy quantity
    'today_buy_val': float,         # Today's buy value
    'today_sell_qty': float,        # Today's sell quantity
    'today_sell_val': float,        # Today's sell value
    'position_side': str,           # 'LONG', 'SHORT'
    'currency': str                 # Currency
}
```

**Example**:
```python
ret, data = trade_ctx.position_list_query()

# ret = RET_OK
# data (DataFrame):
#        code stock_name    qty  can_sell_qty  cost_price  market_val  pl_val  ...
# 0  US.AAPL      APPLE  100.0         100.0      145.50    15025.00  475.00  ...
# 1  HK.00700   TENCENT  200.0         200.0      320.00    69600.00 5720.00  ...
```

---

### place_order()

**Returns**: DataFrame

**Columns**:
```python
{
    'trd_env': str,                 # 'REAL' or 'SIMULATE'
    'order_id': str,                # Order ID (unique)
    'code': str,                    # Security code
    'stock_name': str,              # Security name
    'qty': float,                   # Order quantity
    'price': float,                 # Order price
    'trd_side': str,                # 'BUY' or 'SELL'
    'order_type': str,              # 'NORMAL', 'MARKET', 'STOP', etc.
    'order_status': str,            # 'SUBMITTED', 'WAITING_SUBMIT', etc.
    'create_time': str,             # Order creation time
    'updated_time': str,            # Last update time
    'dealt_qty': float,             # Filled quantity
    'dealt_avg_price': float,       # Average fill price
    'last_err_msg': str,            # Error message (if any)
    'remark': str,                  # User remark
    'time_in_force': str,           # 'DAY', 'GTC', etc.
    'fill_outside_rth': bool,       # Fill outside RTH?
    'session': str,                 # 'RTH', 'ETH', 'OVERNIGHT', 'ALL'
    'aux_price': float,             # Stop price (if stop order)
    'trail_type': str,              # 'PRICE', 'PERCENTAGE' (trailing)
    'trail_value': float,           # Trailing amount
    'trail_spread': float,          # Trailing stop limit spread
    'currency': str                 # Currency
}
```

**Example**:
```python
ret, data = trade_ctx.place_order(
    price=150.00, qty=100, code='US.AAPL',
    trd_side=TrdSide.BUY, order_type=OrderType.NORMAL
)

# ret = RET_OK
# data (DataFrame):
#   trd_env     order_id      code  qty   price trd_side order_type order_status  ...
# 0    REAL  123456789  US.AAPL  100  150.00      BUY     NORMAL    SUBMITTED  ...
```

---

### get_order_list() (order_list_query)

**Returns**: DataFrame

**Columns**: Same as `place_order()` response

**Example**:
```python
ret, data = trade_ctx.order_list_query(status_filter_list=[OrderStatus.SUBMITTED])

# ret = RET_OK
# data (DataFrame): All submitted orders
```

---

### get_history_order_fill_list() (deal_list_query)

**Returns**: DataFrame

**Columns**:
```python
{
    'trd_env': str,                 # 'REAL' or 'SIMULATE'
    'code': str,                    # Security code
    'stock_name': str,              # Security name
    'deal_id': str,                 # Deal ID (unique)
    'order_id': str,                # Associated order ID
    'qty': float,                   # Deal quantity
    'price': float,                 # Deal price
    'trd_side': str,                # 'BUY' or 'SELL'
    'create_time': str,             # Deal time
    'counter_broker_id': int,       # Counter broker ID (HK)
    'counter_broker_name': str,     # Counter broker name (HK)
    'status': str                   # Deal status
}
```

**Example**:
```python
ret, data = trade_ctx.deal_list_query()

# ret = RET_OK
# data (DataFrame):
#   deal_id    order_id      code   qty   price trd_side         create_time  ...
# 0  D12345  123456789  US.AAPL  100  150.25      BUY  2024-01-15 16:00:05  ...
```

---

## Metadata Responses

### get_trading_days()

**Returns**: DataFrame

**Columns**:
```python
{
    'time': str,                    # Date ('2024-01-15')
    'trade_date_type': str          # 'WHOLE', 'MORNING', 'AFTERNOON' (half-day)
}
```

**Example**:
```python
ret, data = quote_ctx.get_trading_days(Market.HK, start='2024-01-01', end='2024-12-31')

# ret = RET_OK
# data (DataFrame): All HK trading days in 2024
#          time trade_date_type
# 0  2024-01-02           WHOLE
# 1  2024-01-03           WHOLE
# ...
```

---

### get_plate_list()

**Returns**: DataFrame

**Columns**:
```python
{
    'code': str,                    # Plate code ('HK.BK1910')
    'plate_name': str,              # Plate name ('消費者服務')
    'plate_id': str                 # Plate ID
}
```

**Example**:
```python
ret, data = quote_ctx.get_plate_list(Market.HK, PlateSetType.INDUSTRY)

# ret = RET_OK
# data (DataFrame):
#          code      plate_name  plate_id
# 0  HK.BK1910      消費者服務   ...
# 1  HK.BK1911        非必需消費品   ...
```

---

### query_subscription()

**Returns**: DataFrame

**Columns**:
```python
{
    'code': str,                    # Security code
    'subtype_list': str,            # Subscription types (comma-separated)
    'total_used': int               # Total quota used for this security
}
```

**Example**:
```python
ret, data = quote_ctx.query_subscription()

# ret = RET_OK
# data (DataFrame):
#        code                  subtype_list  total_used
# 0  US.AAPL  QUOTE,TICKER,K_1M,ORDER_BOOK           4
# 1  HK.00700                         QUOTE           1
```

---

## Error Response Format

When `ret == RET_ERROR (-1)`:

```python
ret, data = quote_ctx.some_method()

# ret = RET_ERROR (-1)
# data = "error message string"  # Not a DataFrame, just string

# Common error messages:
# "not login"
# "freq limit"
# "not subscribed"
# "no authority"
# "unlock trade fail"
# "invalid parameter"
```

---

## Protocol Buffer Internal Format

While SDKs return language-friendly formats (DataFrame, DataTable, etc.), internally Futu uses Protocol Buffers.

**Structure**:
```protobuf
message Response {
    required int32 retType = 1;       // -1 = error, 0 = success
    optional string retMsg = 2;       // Message
    optional int32 errCode = 3;       // Error code
    optional S2C s2c = 4;             // Data payload
}

message S2C {
    // Varies by endpoint
    repeated SecurityData dataList = 1;
}
```

**SDK Conversion**:
- Python: Protocol Buffer → DataFrame
- C#: Protocol Buffer → DataTable
- Java: Protocol Buffer → List<Object>
- C++: Protocol Buffer → Native structures
- JavaScript: Protocol Buffer → Object/Array

---

## Summary

- **Success**: `ret = RET_OK (0)`, `data = DataFrame/dict`
- **Failure**: `ret = RET_ERROR (-1)`, `data = error string`
- **Most responses**: DataFrame format
- **Exception**: `get_order_book()` returns dict
- **Consistency**: Column names consistent across endpoints
- **Types**: Python types (str, float, int, bool)
- **Timestamps**: String format "YYYY-MM-DD HH:mm:ss"
- **Security Codes**: String format "MARKET.CODE"
