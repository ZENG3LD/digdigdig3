# Alpaca - Complete Endpoint Reference

## Category: Trading API - Account Management

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v2/account | Retrieve account details | Yes | Yes | 200/min free, unlimited paid | Returns equity, cash, buying power, etc. |
| PATCH | /v2/account | Update account settings | Yes | Yes | 200/min free, unlimited paid | Modify account configuration |
| GET | /v2/account/portfolio/history | Portfolio performance history | Yes | Yes | 200/min free, unlimited paid | Historical equity curve |
| GET | /v2/account/activities | Account activity log | Yes | Yes | 200/min free, unlimited paid | All account activities |
| GET | /v2/account/activities/{activity_type} | Filtered activities | Yes | Yes | 200/min free, unlimited paid | Filter by type (FILL, TRANS, etc.) |

## Category: Trading API - Orders

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| POST | /v2/orders | Create new order | Yes | Yes | 200/min free, unlimited paid | Market, limit, stop, stop-limit, trailing stop |
| GET | /v2/orders | List all orders | Yes | Yes | 200/min free, unlimited paid | Filter by status, dates |
| GET | /v2/orders/{order_id} | Get specific order | Yes | Yes | 200/min free, unlimited paid | By order ID |
| GET | /v2/orders:by_client_order_id | Query by client ID | Yes | Yes | 200/min free, unlimited paid | Custom client order ID |
| PATCH | /v2/orders/{order_id} | Modify existing order | Yes | Yes | 200/min free, unlimited paid | Update qty, limit price, etc. |
| DELETE | /v2/orders | Cancel all orders | Yes | Yes | 200/min free, unlimited paid | Cancel all open orders |
| DELETE | /v2/orders/{order_id} | Cancel specific order | Yes | Yes | 200/min free, unlimited paid | By order ID |

## Category: Trading API - Positions

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v2/positions | List all positions | Yes | Yes | 200/min free, unlimited paid | All open positions |
| GET | /v2/positions/{symbol} | Get position by symbol | Yes | Yes | 200/min free, unlimited paid | Single symbol position |
| DELETE | /v2/positions | Close all positions | Yes | Yes | 200/min free, unlimited paid | Market orders to close all |
| DELETE | /v2/positions/{symbol} | Close specific position | Yes | Yes | 200/min free, unlimited paid | Market order to close |
| POST | /v2/positions/{symbol}/exercise | Exercise options | Yes | Yes | 200/min free, unlimited paid | Options contracts only |

## Category: Trading API - Assets

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v2/assets | List all assets | Yes | Yes | 200/min free, unlimited paid | Filter by status, class |
| GET | /v2/assets/{id_or_symbol} | Get asset details | Yes | Yes | 200/min free, unlimited paid | By symbol or asset ID |
| GET | /v2/option_contracts | List option contracts | Yes | Yes | 200/min free, unlimited paid | Filter by underlying, expiration |

## Category: Trading API - Watchlists

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v2/watchlists | Get all watchlists | Yes | Yes | 200/min free, unlimited paid | All user watchlists |
| POST | /v2/watchlists | Create watchlist | Yes | Yes | 200/min free, unlimited paid | Create new watchlist |
| GET | /v2/watchlists/{watchlist_id} | Get specific watchlist | Yes | Yes | 200/min free, unlimited paid | By watchlist ID |
| PUT | /v2/watchlists/{watchlist_id} | Update watchlist | Yes | Yes | 200/min free, unlimited paid | Modify watchlist |
| DELETE | /v2/watchlists/{watchlist_id} | Delete watchlist | Yes | Yes | 200/min free, unlimited paid | Remove watchlist |
| POST | /v2/watchlists/{watchlist_id} | Add asset to watchlist | Yes | Yes | 200/min free, unlimited paid | Add symbol |

## Category: Trading API - Market Info

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v2/calendar | Trading calendar | Yes | Yes | 200/min free, unlimited paid | Market days 1970-2029, early closures |
| GET | /v2/clock | Market clock status | Yes | Yes | 200/min free, unlimited paid | Open/closed, next open/close |

## Category: Market Data API - Stock Bars (OHLCV)

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v2/stocks/bars | Historical bars (multi-symbol) | Yes | Yes | 200/min free, unlimited paid | Timeframes: 1Min-1Week |
| GET | /v2/stocks/{symbol}/bars | Historical bars (single symbol) | Yes | Yes | 200/min free, unlimited paid | Same as multi-symbol |
| GET | /v2/stocks/bars/latest | Latest bars (multi-symbol) | Yes | Yes | 200/min free, unlimited paid | Most recent bar |
| GET | /v2/stocks/{symbol}/bars/latest | Latest bar (single symbol) | Yes | Yes | 200/min free, unlimited paid | Single symbol latest |

**Bars Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| symbols | string | Yes | - | Comma-separated symbols |
| timeframe | string | Yes | - | 1Min,5Min,15Min,30Min,1Hour,4Hour,1Day,1Week |
| start | datetime | No | - | Inclusive start (RFC-3339 or YYYY-MM-DD) |
| end | datetime | No | - | Inclusive end (RFC-3339 or YYYY-MM-DD) |
| limit | integer | No | 1000 | Max 10,000 per response |
| adjustment | string | No | raw | raw, split, dividend, all |
| feed | string | No | sip | iex (free), sip (paid), boats, overnight |
| sort | string | No | asc | asc, desc |
| page_token | string | No | - | Pagination token |

## Category: Market Data API - Stock Trades

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v2/stocks/trades | Historical trades (multi-symbol) | Yes | Yes | 200/min free, unlimited paid | All trade executions |
| GET | /v2/stocks/{symbol}/trades | Historical trades (single symbol) | Yes | Yes | 200/min free, unlimited paid | Single symbol trades |
| GET | /v2/stocks/trades/latest | Latest trades (multi-symbol) | Yes | Yes | 200/min free, unlimited paid | Most recent trade |
| GET | /v2/stocks/{symbol}/trades/latest | Latest trade (single symbol) | Yes | Yes | 200/min free, unlimited paid | Single symbol latest |

**Trades Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| symbols | string | Yes | - | Comma-separated symbols |
| start | datetime | No | - | Inclusive start (RFC-3339 or YYYY-MM-DD) |
| end | datetime | No | - | Inclusive end (RFC-3339 or YYYY-MM-DD) |
| limit | integer | No | 1000 | Max data points per response |
| feed | string | No | sip | iex (free), sip (paid) |
| sort | string | No | asc | asc, desc |
| page_token | string | No | - | Pagination token |

## Category: Market Data API - Stock Quotes

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v2/stocks/quotes | Historical quotes (multi-symbol) | Yes | Yes | 200/min free, unlimited paid | Bid/ask history |
| GET | /v2/stocks/{symbol}/quotes | Historical quotes (single symbol) | Yes | Yes | 200/min free, unlimited paid | Single symbol quotes |
| GET | /v2/stocks/quotes/latest | Latest quotes (multi-symbol) | Yes | Yes | 200/min free, unlimited paid | Most recent bid/ask |
| GET | /v2/stocks/{symbol}/quotes/latest | Latest quote (single symbol) | Yes | Yes | 200/min free, unlimited paid | Single symbol latest |

**Quotes Parameters:** Same as Trades (symbols, start, end, limit, feed, sort, page_token)

## Category: Market Data API - Stock Snapshots

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v2/stocks/snapshots | Snapshots (multi-symbol) | Yes | Yes | 200/min free, unlimited paid | Latest trade, quote, bars |
| GET | /v2/stocks/{symbol}/snapshot | Snapshot (single symbol) | Yes | Yes | 200/min free, unlimited paid | All current data for symbol |

**Snapshot includes:** Latest trade, latest quote, minute bar, daily bar, previous daily bar

## Category: Market Data API - Options Data

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v1beta1/options/snapshots/{underlying_symbol} | Option chain | Indicative free, real-time paid | Yes | 200/min free, unlimited paid | Latest trade, quote, greeks, IV |
| GET | /v1beta1/options/bars | Historical option bars | Indicative free, real-time paid | Yes | 200/min free, unlimited paid | OHLCV for options |
| GET | /v1beta1/options/trades | Historical option trades | Indicative free, real-time paid | Yes | 200/min free, unlimited paid | Trade executions |
| GET | /v1beta1/options/quotes | Historical option quotes | Indicative free, real-time paid | Yes | 200/min free, unlimited paid | Bid/ask history |

**Options Chain Response includes:**
- Latest trade data
- Latest quote data
- Greeks (delta, gamma, theta, vega, rho)
- Implied volatility

## Category: Market Data API - Crypto Data

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v1beta3/crypto/us/bars | Crypto bars | Yes | No (historical) | 200/min free, unlimited paid | OHLCV data |
| GET | /v1beta3/crypto/us/trades | Crypto trades | Yes | Yes | 200/min free, unlimited paid | Trade executions |
| GET | /v1beta3/crypto/us/quotes | Crypto quotes | Yes | Yes | 200/min free, unlimited paid | Bid/ask data |
| GET | /v1beta3/crypto/us/latest/orderbooks | Latest orderbooks | Yes | Yes | 200/min free, unlimited paid | Depth of market |
| GET | /v1beta3/crypto/us/snapshots | Crypto snapshots | Yes | Yes | 200/min free, unlimited paid | Latest trade, quote, bars |

**Crypto Feed:** Data from Alpaca and Kraken exchanges

## Category: Market Data API - News

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v1beta1/news | News articles | Yes | Yes | 200/min free, unlimited paid | Latest 10 by default |

**News Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| symbols | string | No | - | Comma-separated symbols filter |
| start | datetime | No | - | Inclusive start (RFC-3339 or YYYY-MM-DD) |
| end | datetime | No | - | Inclusive end (RFC-3339 or YYYY-MM-DD) |
| sort | string | No | desc | asc, desc |
| limit | integer | No | 10 | 1-50 articles per page |
| include_content | boolean | No | false | Include full article content |
| exclude_contentless | boolean | No | false | Exclude articles without content |
| page_token | string | No | - | Pagination token |

**News Response Fields:**
- id, headline, author, created_at, updated_at
- summary, content (may contain HTML), url
- images (array with thumb, small, large sizes)
- symbols (associated tickers)
- source (e.g., "benzinga")

## Category: Market Data API - Corporate Actions

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v1beta1/corporate-actions/announcements | Corporate actions | Yes | Yes | 200/min free, unlimited paid | Dividends, splits, mergers, spinoffs |

**Corporate Actions Parameters:**
- Filter by type (dividend, split, merger, spinoff)
- Filter by symbol, CUSIP
- Filter by date type (declaration, ex-dividend, record, payable)

## Category: Market Data API - Screener

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v1beta1/screener/stocks/movers | Top movers | Yes | Yes | 200/min free, unlimited paid | Gainers, losers, most active |

## Category: Market Data API - Forex & Fixed Income

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v1beta1/forex/rates | Forex rates | Yes | Yes | 200/min free, unlimited paid | Currency pair rates |
| GET | /v1beta1/fixed-income | Fixed income prices | Yes | Yes | 200/min free, unlimited paid | Bond pricing |

## Category: Market Data API - Logos

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v1beta1/logos/{symbol} | Company logo | Yes | Yes | 200/min free, unlimited paid | Logo image URL |

## Category: Broker API (for businesses)

The Broker API includes extensive endpoints for:

### Account Management
- GET/POST /v1/accounts - List/create customer accounts
- GET/PATCH /v1/accounts/{account_id} - Retrieve/update account
- POST /v1/accounts/{account_id}/actions/close - Close account

### Funding
- Bank relationships, ACH transfers, instant funding
- Crypto wallets and crypto transfers
- JIT (Just-In-Time) settlements

### Compliance & Reporting
- KYC/CIP verification, document management
- Pattern Day Trader (PDT) status tracking
- Corporate actions processing
- EOD positions, cash interest, aggregate reporting

### Trading (mirrors Trading API for customer accounts)
- Orders, positions, account data per customer account

## Category: OAuth / Connect API

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| POST | /v1/oauth2/token | Exchange credentials for token | Yes | No | - | OAuth2 token endpoint |

**Grant Types:** Authorization Code, Client Credentials, Private Key JWT

## WebSocket Control (Market Data)

WebSocket connections do NOT use REST endpoints for control. Authentication and subscription are done via WebSocket messages.

## Rate Limit Headers (all endpoints)

All REST responses include:
- X-RateLimit-Limit: Maximum requests per window
- X-RateLimit-Remaining: Remaining requests
- X-RateLimit-Reset: Timestamp when limit resets

## Error Responses

| Code | Description | Resolution |
|------|-------------|------------|
| 401 | Unauthorized | Invalid/missing API credentials |
| 402 | Authentication failed | Check API key and secret |
| 403 | Forbidden | Insufficient subscription or buying power |
| 405 | Symbol limit exceeded | Reduce symbols in request |
| 406 | Connection limit exceeded | Too many concurrent connections |
| 409 | Insufficient subscription | Upgrade tier for access |
| 422 | Unprocessable Entity | Invalid request parameters |
| 429 | Rate limit exceeded | Wait for rate limit reset |

## Data Feeds

**Stock Market Data:**
- **iex**: IEX exchange only (~2.5% market volume) - FREE tier
- **sip**: All US exchanges (100% market volume) - PAID tier (SIP = Securities Information Processor)
- **boats**: Blue Ocean ATS for extended evening hours
- **overnight**: Alpaca-derived feed from BOATS with 15-min delay

**Options Data:**
- **Indicative**: Free tier (delayed, not real-time)
- **Real-time (OPRA)**: Paid tier (Algo Trader Plus)

**Crypto Data:**
- **us**: Alpaca + Kraken exchanges (v1beta3)
