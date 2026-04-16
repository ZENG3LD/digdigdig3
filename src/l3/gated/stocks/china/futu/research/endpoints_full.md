# Futu OpenAPI - Complete Endpoint Reference

**Note**: Futu uses a custom TCP protocol (not REST). Below are SDK method names which correspond to protocol operations.

## Category: Subscription & Connection

| Method | Description | Auth? | Rate Limit | Notes |
|--------|-------------|-------|------------|-------|
| `connect()` | Connect to OpenD gateway | No | - | Persistent TCP connection |
| `close()` | Close connection | No | - | Graceful shutdown |
| `subscribe()` | Subscribe to real-time data | Yes | Subject to quota | See subscription quotas |
| `unsubscribe()` | Unsubscribe from data | Yes | No limit | Min 1-minute after subscribe |
| `unsubscribe_all()` | Unsubscribe all channels | Yes | No limit | - |
| `query_subscription()` | Get current subscription status | Yes | 60/30s | Check active subscriptions |
| `get_global_state()` | Get connection state | Yes | 60/30s | Login status, market state |

## Category: Market Data - Low Frequency

| Method | Description | Auth? | Rate Limit | Notes |
|--------|-------------|-------|------------|-------|
| `get_trading_days()` | Get trading calendar | Yes | 60/30s | Holidays, half-days |
| `get_stock_basicinfo()` | Get stock static info | Yes | 60/30s | Name, lot size, listing date |
| `get_multiple_history_kline()` | Get multiple securities' K-lines | Yes | 60/30s | Batch historical candlesticks |
| `request_history_kline()` | Request historical candlesticks | Yes | 60/30s | Uses historical quota |
| `request_history_kl_quota()` | Query historical K-line quota | Yes | 60/30s | Check remaining quota |
| `get_market_state()` | Get market trading status | Yes | 60/30s | Open, closed, pre-market |
| `get_global_state()` | Get server time and state | Yes | 60/30s | Synchronization |
| `get_static_info()` | Get stock static information | Yes | 60/30s | Similar to basicinfo |
| `get_security_snapshot()` | Get snapshot for securities | Yes | 60/30s | Latest quote snapshot |
| `get_plate_list()` | Get sector/plate lists | Yes | 60/30s | Industry classifications |
| `get_plate_stock()` | Get stocks in plate | Yes | 60/30s | Securities in sector |
| `get_plate_set()` | Get plate categories | Yes | 60/30s | Sector hierarchies |
| `get_capital_flow()` | Get capital flow data | Yes | 60/30s | Money flow analysis |
| `get_capital_distribution()` | Capital distribution | Yes | 60/30s | By investor type |
| `get_user_security()` | Get user's watchlist | Yes | 60/30s | Custom security groups |
| `modify_user_security()` | Modify watchlist | Yes | 60/30s | Add/remove securities |
| `get_market_snapshot()` | Get market snapshot batch | Yes | 60/30s | Multiple securities at once |
| `get_option_chain()` | Get options chain | Yes | 60/30s | Strikes and expirations |
| `get_warrant()` | Get warrant info | Yes | 60/30s | HK warrants |
| `get_reference_stock_list()` | Get reference stocks | Yes | 60/30s | Underlying securities |
| `get_owner_plate()` | Get plates containing stock | Yes | 60/30s | Reverse plate lookup |
| `get_holding_change_list()` | Get shareholding changes | Yes | 60/30s | Major holder changes |
| `get_ipo_list()` | Get IPO list | Yes | 60/30s | Upcoming/recent IPOs |
| `get_option_expiration_date()` | Get option expiry dates | Yes | 60/30s | Available expirations |
| `get_codelist_chan()` | Get codelist updates | Yes | - | Subscription for list changes |

## Category: Market Data - High Frequency (Real-time)

| Method | Description | Auth? | Rate Limit | Notes |
|--------|-------------|-------|------------|-------|
| `get_stock_quote()` | Get real-time quote | Yes | No limit | After subscription |
| `get_rt_ticker()` | Get real-time tick-by-tick | Yes | No limit | Trade flow |
| `get_cur_kline()` | Get current candlestick | Yes | No limit | Latest bar |
| `get_order_book()` | Get order book (L2 depth) | Yes | No limit | Bid/ask levels |
| `get_broker_queue()` | Get broker queue | Yes | No limit | HK broker IDs |
| `get_rt_data()` | Get real-time time frame | Yes | No limit | Intraday timeline |
| `get_multi_kline()` | Get multiple K-lines | Yes | No limit | Batch current bars |

### Real-time Push Callbacks (after subscription)

| Callback | Description | Trigger | Notes |
|----------|-------------|---------|-------|
| `on_recv_rsp()` (Quote) | Quote update pushed | Price change | Basic quote fields |
| `on_recv_rsp()` (Ticker) | Tick update pushed | Each trade | Trade price, volume, time |
| `on_recv_rsp()` (OrderBook) | Order book update | Depth change | Snapshot or delta |
| `on_recv_rsp()` (KL) | Candlestick update | Bar close/update | OHLC update |
| `on_recv_rsp()` (RT) | Time frame update | Minute tick | Intraday timeline |
| `on_recv_rsp()` (Broker) | Broker queue update | Queue change | HK only |

## Category: Corporate Actions & Adjustments

| Method | Description | Auth? | Rate Limit | Notes |
|--------|-------------|-------|------------|-------|
| `get_rehab()` | Get rehabilitation data | Yes | 60/30s | Splits, dividends |
| `get_suspension()` | Get suspension info | Yes | 60/30s | Trading halts |
| `request_rehab()` | Request rehab for security | Yes | 60/30s | Adjustment factors |

## Category: Trading - Account Management

| Method | Description | Auth? | Rate Limit | Notes |
|--------|-------------|-------|------------|-------|
| `get_acc_list()` | Get trading accounts | Yes | 60/30s | List all accounts |
| `unlock_trade()` | Unlock trading | Yes | 60/30s | Required before trading |
| `accinfo_query()` | Get account info (funds) | Yes | 60/30s | Net assets, cash, buying power |
| `get_funds()` | Get account funds | Yes | 60/30s | Detailed fund breakdown |
| `acctradinginfo_query()` | Get trading info | Yes | 60/30s | Max buy/sell quantities |
| `get_max_trd_qtys()` | Get max tradeable quantity | Yes | 60/30s | Based on buying power |
| `get_acc_info()` | Get account details | Yes | 60/30s | Account type, status |
| `get_margin_ratio()` | Get margin ratio | Yes | 60/30s | Margin requirements |
| `get_cash_info()` | Get cash details | Yes | 60/30s | Cash positions by currency |

## Category: Trading - Positions

| Method | Description | Auth? | Rate Limit | Notes |
|--------|-------------|-------|------------|-------|
| `position_list_query()` | Query positions | Yes | 60/30s | Holdings list |
| `get_position_list()` | Get position list | Yes | 60/30s | Current holdings |

## Category: Trading - Order Execution

| Method | Description | Auth? | Rate Limit | Notes |
|--------|-------------|-------|------------|-------|
| `place_order()` | Place order | Yes | **15/30s per account** | Min 0.02s between orders |
| `modify_order()` | Modify existing order | Yes | **15/30s per account** | Price/quantity changes |
| `cancel_order()` | Cancel order | Yes | **15/30s per account** | - |
| `cancel_all_order()` | Cancel all orders | Yes | **15/30s per account** | By account |

### Order Modification Operations
- `NORMAL` - Modify price/quantity
- `CANCEL` - Cancel order
- `DISABLE` - Disable (withdraw without delete)
- `ENABLE` - Re-enable disabled order
- `DELETE` - Delete order record

## Category: Trading - Order Status & History

| Method | Description | Auth? | Rate Limit | Notes |
|--------|-------------|-------|------------|-------|
| `order_list_query()` | Query open orders | Yes | 60/30s | Active orders only |
| `get_order_list()` | Get order list | Yes | 60/30s | Current day orders |
| `history_order_list_query()` | Query historical orders | Yes | 60/30s | Past orders |
| `get_history_order_list()` | Get historical orders | Yes | 60/30s | Completed/cancelled |
| `get_order_fee()` | Get order fee estimate | Yes | 60/30s | Commission calculation |

### Order Status Callback
| Callback | Description | Trigger | Notes |
|----------|-------------|---------|-------|
| `on_recv_rsp()` (Order) | Order status update | Status change | Real-time order updates |

## Category: Trading - Deal/Fill History

| Method | Description | Auth? | Rate Limit | Notes |
|--------|-------------|-------|------------|-------|
| `deal_list_query()` | Query today's deals | Yes | 60/30s | Current day fills |
| `get_order_fill_list()` | Get fill list | Yes | 60/30s | Executed trades |
| `history_deal_list_query()` | Query historical deals | Yes | 60/30s | Past fills |
| `get_history_order_fill_list()` | Get historical fills | Yes | 60/30s | Live trading only |

### Deal Status Callback
| Callback | Description | Trigger | Notes |
|----------|-------------|---------|-------|
| `on_recv_rsp()` (Deal) | Deal/fill update | Execution | Real-time fill updates |

## Category: Trading - Subscription & Notifications

| Method | Description | Auth? | Rate Limit | Notes |
|--------|-------------|-------|------------|-------|
| `sub_acc_push()` | Subscribe to account updates | Yes | - | Order/fill/position updates |

## Category: Advanced Market Data

| Method | Description | Auth? | Rate Limit | Notes |
|--------|-------------|-------|------------|-------|
| `get_price_reminder()` | Get price alerts | Yes | 60/30s | User-configured alerts |
| `set_price_reminder()` | Set price alert | Yes | 60/30s | Create alert |
| `modify_price_reminder()` | Modify alert | Yes | 60/30s | Update alert |

## Subscription Types (SubType Enum)

For `subscribe()` method:

| SubType | Description | Free? | Notes |
|---------|-------------|-------|-------|
| `QUOTE` | Basic quote | Depends on market | Price, bid, ask, volume |
| `ORDER_BOOK` | Order book depth | Depends on market | Bid/ask levels |
| `TICKER` | Trade tick-by-tick | Depends on market | Individual trades |
| `K_1M` | 1-minute candlestick | Depends on market | OHLC 1min |
| `K_3M` | 3-minute candlestick | Depends on market | OHLC 3min |
| `K_5M` | 5-minute candlestick | Depends on market | OHLC 5min |
| `K_15M` | 15-minute candlestick | Depends on market | OHLC 15min |
| `K_30M` | 30-minute candlestick | Depends on market | OHLC 30min |
| `K_60M` | 60-minute candlestick | Depends on market | OHLC 60min |
| `K_DAY` | Daily candlestick | Depends on market | OHLC daily |
| `K_WEEK` | Weekly candlestick | Depends on market | OHLC weekly |
| `K_MON` | Monthly candlestick | Depends on market | OHLC monthly |
| `RT_DATA` | Time frame data | Depends on market | Intraday timeline |
| `BROKER` | Broker queue | HK LV2 required | Broker IDs |

## Parameters Reference

### subscribe()
| Name | Type | Required | Description |
|------|------|----------|-------------|
| code_list | list[str] | Yes | Security codes (e.g., ["US.AAPL", "HK.00700"]) |
| subtype_list | list[SubType] | Yes | Data types to subscribe |
| is_first_push | bool | No (default: True) | Push cached data immediately |
| subscribe_push | bool | No (default: True) | Enable real-time push |
| is_detailed_orderbook | bool | No (default: False) | Detailed order book |
| extended_time | bool | No (default: False) | US pre/post market |
| session | Session | No (default: NONE) | US trading session |

### get_stock_quote()
| Name | Type | Required | Description |
|------|------|----------|-------------|
| code_list | list[str] | Yes | Security codes (must be subscribed first) |

### request_history_kline()
| Name | Type | Required | Description |
|------|------|----------|-------------|
| code | str | Yes | Security code |
| start | str | No | Start date (YYYY-MM-DD) or None |
| end | str | No | End date (YYYY-MM-DD) or None |
| ktype | KLType | No (default: K_DAY) | Candlestick interval |
| autype | AuType | No (default: QFQ) | Adjustment type (前复权/后复权/不复权) |
| fields | list[KL_FIELD] | No (default: ALL) | Fields to return |
| max_count | int | No (default: 1000) | Max bars to return |
| page_req_key | str | No (default: None) | Pagination key |
| extended_time | bool | No (default: False) | US extended hours |

### place_order()
| Name | Type | Required | Description |
|------|------|----------|-------------|
| price | float | Yes | Order price (precision varies by market) |
| qty | float | Yes | Quantity (shares/contracts) |
| code | str | Yes | Security code |
| trd_side | TrdSide | Yes | BUY or SELL |
| order_type | OrderType | No (default: NORMAL) | Order type |
| adjust_limit | float | No (default: 0) | Price adjustment % (e.g., 0.015 = 1.5%) |
| trd_env | TrdEnv | No (default: REAL) | REAL or SIMULATE |
| acc_id | int | No (default: 0) | Account ID (0 = first account) |
| acc_index | int | No (default: 0) | Account index |
| remark | str | No (default: None) | Order remark (max 64 bytes UTF-8) |
| time_in_force | TimeInForce | No (default: DAY) | DAY, GTC, etc. |
| fill_outside_rth | bool | No (default: False) | US pre/post market |
| aux_price | float | No (default: None) | Stop price (for stop orders) |
| trail_type | TrailType | No (default: None) | PRICE or PERCENTAGE |
| trail_value | float | No (default: None) | Trailing amount |
| trail_spread | float | No (default: None) | Trailing stop limit spread |
| session | Session | No (default: NONE) | US session (RTH, ETH, OVERNIGHT, ALL) |

### get_order_list()
| Name | Type | Required | Description |
|------|------|----------|-------------|
| order_id | str | No | Filter by order ID |
| status_filter_list | list[OrderStatus] | No | Filter by status |
| code | str | No | Filter by security |
| start | str | No | Start date |
| end | str | No | End date |
| trd_env | TrdEnv | No (default: REAL) | REAL or SIMULATE |
| acc_id | int | No (default: 0) | Account ID |
| acc_index | int | No (default: 0) | Account index |

## Protocol IDs (for reference)

Each endpoint has an internal protocol ID used in the TCP protocol:
- Subscribe: 3001
- Get Stock Quote: 3004
- Get Order Book: 3012
- Request History K-line: 3103
- Place Order: 2202
- Get Account List: 2001
- (Full list available in Protocol Buffer definitions)

## Notes on Rate Limits

1. **Standard limit**: 60 requests per 30 seconds for most endpoints
2. **Trading limit**: 15 requests per 30 seconds per account for order operations
3. **High-frequency**: Real-time data getters (after subscription) have no request limits
4. **Subscription quotas**: See tiers_and_limits.md for quota details
5. **Burst allowed**: Can send requests in bursts as long as total within window complies

## Error Codes (Common)

| Code | Description | Resolution |
|------|-------------|------------|
| -1 | Generic error | Check error message |
| 0 | Success | - |
| 1 | Already subscribed | Unsubscribe first |
| 2 | Server connection error | Check OpenD status |
| 3 | Server timeout | Retry request |
| 4 | Protocol error | Check parameters |
| 5 | Not logged in | Authenticate first |
| 400 | Parameter error | Check parameter format |
| 401 | Unauthorized | Check login status |
| 405 | Frequency limit | Wait and retry |
| 410 | Quota limit | Wait for quota reset |

(Detailed error codes available in SDK documentation)
