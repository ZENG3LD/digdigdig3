# CryptoCompare - Complete Endpoint Reference

Base URL: `https://min-api.cryptocompare.com`

## Category: Price Data (Current)

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /data/price | Single symbol current price | Yes | Optional | 50/sec, 1000/min, 150k/hr | Returns simple price object |
| GET | /data/pricemulti | Multiple symbols current price | Yes | Optional | 50/sec, 1000/min, 150k/hr | Matrix of prices |
| GET | /data/pricemultifull | Full price data (multiple symbols) | Yes | Optional | 50/sec, 1000/min, 150k/hr | Includes OHLCV, volume, market cap |
| GET | /data/generateAvg | Generate average price | Yes | Optional | 50/sec, 1000/min, 150k/hr | Volume-weighted avg across exchanges |
| GET | /data/dayAvg | Daily average price | Yes | Optional | 50/sec, 1000/min, 150k/hr | Based on hourly VWAP |

### Parameters Reference

#### GET /data/price
**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| fsym | string | Yes | - | From symbol (e.g., "BTC") |
| tsyms | string | Yes | - | Comma-separated to symbols (e.g., "USD,EUR") |
| e | string | No | "CCCAGG" | Exchange name or CCCAGG for aggregate |
| extraParams | string | No | - | Your app name for tracking |
| sign | boolean | No | false | Include signature if true |
| tryConversion | boolean | No | true | Try conversion if pair not found |
| api_key | string | No | - | API key (increases rate limits) |

#### GET /data/pricemulti
**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| fsyms | string | Yes | - | Comma-separated from symbols (e.g., "BTC,ETH") |
| tsyms | string | Yes | - | Comma-separated to symbols (e.g., "USD,EUR") |
| e | string | No | "CCCAGG" | Exchange name or CCCAGG |
| extraParams | string | No | - | Your app name |
| sign | boolean | No | false | Include signature |
| tryConversion | boolean | No | true | Try conversion if pair not found |
| api_key | string | No | - | API key |

#### GET /data/pricemultifull
**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| fsyms | string | Yes | - | Comma-separated from symbols |
| tsyms | string | Yes | - | Comma-separated to symbols |
| e | string | No | "CCCAGG" | Exchange name |
| extraParams | string | No | - | Your app name |
| sign | boolean | No | false | Include signature |
| tryConversion | boolean | No | true | Try conversion |
| api_key | string | No | - | API key |

## Category: Historical Data

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /data/pricehistorical | Historical price at timestamp | Yes | Optional | 50/sec, 1000/min, 150k/hr | End of day GMT price |
| GET | /data/histoday | Daily OHLCV bars | Yes | Optional | 50/sec, 1000/min, 150k/hr | Max 2000 days, full history available |
| GET | /data/histohour | Hourly OHLCV bars | Yes | Optional | 50/sec, 1000/min, 150k/hr | Max 2000 hours |
| GET | /data/histominute | Minute OHLCV bars | Yes | Optional | 50/sec, 1000/min, 150k/hr | 7 days free, 1 year enterprise |
| GET | /data/v2/histoday | Daily bars (v2) | Yes | Yes | 50/sec, 1000/min, 150k/hr | Newer version with more data |
| GET | /data/v2/histohour | Hourly bars (v2) | Yes | Yes | 50/sec, 1000/min, 150k/hr | Newer version |
| GET | /data/v2/histominute | Minute bars (v2) | Yes | Yes | 50/sec, 1000/min, 150k/hr | Newer version |

### Parameters Reference

#### GET /data/histoday
**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| fsym | string | Yes | - | From symbol |
| tsym | string | Yes | - | To symbol |
| e | string | No | "CCCAGG" | Exchange name |
| aggregate | int | No | 1 | Data aggregation (1, 3, 7, etc.) |
| limit | int | No | 30 | Number of bars (max 2000) |
| toTs | timestamp | No | now | End timestamp (Unix seconds) |
| allData | boolean | No | false | Return all available data (ignores limit) |
| extraParams | string | No | - | Your app name |
| sign | boolean | No | false | Include signature |
| tryConversion | boolean | No | true | Try conversion |
| api_key | string | No | - | API key |

#### GET /data/histohour
**Parameters:** Same as histoday

#### GET /data/histominute
**Parameters:** Same as histoday

#### GET /data/pricehistorical
**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| fsym | string | Yes | - | From symbol |
| tsyms | string | Yes | - | Comma-separated to symbols |
| ts | timestamp | Yes | - | Unix timestamp (seconds) |
| markets | string | No | - | Comma-separated exchanges |
| extraParams | string | No | - | Your app name |
| tryConversion | boolean | No | true | Try conversion |
| api_key | string | No | - | API key |

## Category: Exchange & Trading Pairs

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /data/top/exchanges | Top exchanges by volume | Yes | Optional | 50/sec, 1000/min, 150k/hr | For a given pair |
| GET | /data/top/exchanges/full | Top exchanges full data | Yes | Optional | 50/sec, 1000/min, 150k/hr | Includes all trading info |
| GET | /data/top/pairs | Top pairs by volume | Yes | Optional | 50/sec, 1000/min, 150k/hr | For a given symbol |
| GET | /data/top/volumes | Top coins by volume | Yes | Optional | 50/sec, 1000/min, 150k/hr | 24h total volume |
| GET | /data/top/mktcapfull | Top coins by market cap | Yes | Optional | 50/sec, 1000/min, 150k/hr | Full market data |
| GET | /data/top/totalvolfull | Top coins by total volume | Yes | Optional | 50/sec, 1000/min, 150k/hr | All markets aggregated |

### Parameters Reference

#### GET /data/top/exchanges
**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| fsym | string | Yes | - | From symbol |
| tsym | string | Yes | - | To symbol |
| limit | int | No | 5 | Number of exchanges (max 50) |
| extraParams | string | No | - | Your app name |
| sign | boolean | No | false | Include signature |
| api_key | string | No | - | API key |

#### GET /data/top/pairs
**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| fsym | string | Yes | - | From symbol |
| limit | int | No | 5 | Number of pairs (max 100) |
| extraParams | string | No | - | Your app name |
| sign | boolean | No | false | Include signature |
| api_key | string | No | - | API key |

#### GET /data/top/volumes
**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| tsym | string | Yes | - | To symbol (usually USD) |
| limit | int | No | 10 | Number of coins (max 100) |
| page | int | No | 0 | Page number |
| extraParams | string | No | - | Your app name |
| sign | boolean | No | false | Include signature |
| api_key | string | No | - | API key |

## Category: Metadata & Reference Data

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /data/all/coinlist | All coins list | Yes | No | 50/sec, 1000/min, 150k/hr | Full coin metadata |
| GET | /data/all/exchanges | All exchanges and pairs | Yes | No | 50/sec, 1000/min, 150k/hr | Complete exchange list |
| GET | /data/blockchain/list | Blockchain list | Yes | Optional | 50/sec, 1000/min, 150k/hr | Supported blockchains |
| GET | /data/blockchain/histo/day | Blockchain daily stats | Yes | Optional | 50/sec, 1000/min, 150k/hr | Historical blockchain data |

### Parameters Reference

#### GET /data/all/coinlist
**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| summary | boolean | No | false | Return summary only |
| api_key | string | No | - | API key |

#### GET /data/all/exchanges
**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| api_key | string | No | - | API key |

## Category: News & Social

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /data/v2/news/ | Latest news articles | Yes | Yes | 50/sec, 1000/min, 150k/hr | Crypto news aggregator |
| GET | /data/news/feeds | News feeds list | Yes | Optional | 50/sec, 1000/min, 150k/hr | Available news sources |
| GET | /data/news/categories | News categories | Yes | Optional | 50/sec, 1000/min, 150k/hr | Available categories |
| GET | /data/social/coin/latest | Latest social stats | Yes | Yes | 50/sec, 1000/min, 150k/hr | Reddit, Twitter, etc. |
| GET | /data/social/coin/histo/day | Historical social stats (daily) | Yes | Yes | 50/sec, 1000/min, 150k/hr | Social metrics over time |
| GET | /data/social/coin/histo/hour | Historical social stats (hourly) | Yes | Yes | 50/sec, 1000/min, 150k/hr | Social metrics over time |

### Parameters Reference

#### GET /data/v2/news/
**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| feeds | string | No | - | Comma-separated feed IDs |
| categories | string | No | - | Comma-separated categories |
| excludeCategories | string | No | - | Categories to exclude |
| lTs | timestamp | No | - | Latest timestamp (Unix seconds) |
| lang | string | No | "EN" | Language (EN, PT, etc.) |
| sortOrder | string | No | "latest" | latest or popular |
| api_key | string | Yes | - | API key (required) |

#### GET /data/social/coin/latest
**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| coinId | int | Yes | - | Coin ID from coinlist |
| api_key | string | Yes | - | API key (required) |

## Category: Blockchain Data (On-Chain)

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /data/blockchain/histo/day | Daily blockchain stats | Yes | Optional | 50/sec, 1000/min, 150k/hr | Transactions, hashrate, etc. |
| GET | /data/blockchain/list | Supported blockchains | Yes | Optional | 50/sec, 1000/min, 150k/hr | List of available chains |
| GET | /data/blockchain/mining/calculator | Mining calculator | Yes | Optional | 50/sec, 1000/min, 150k/hr | Mining profitability |
| GET | /data/blockchain/latest | Latest blockchain data | Yes | Optional | 50/sec, 1000/min, 150k/hr | Current blockchain stats |

### Parameters Reference

#### GET /data/blockchain/histo/day
**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| fsym | string | Yes | - | Symbol (e.g., BTC) |
| limit | int | No | 30 | Number of days (max 2000) |
| toTs | timestamp | No | now | End timestamp |
| aggregate | int | No | 1 | Data aggregation |
| api_key | string | No | - | API key |

## Category: Account & API Management

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /stats/rate/limit | Rate limit status | No | Yes | N/A | Check current usage |
| GET | /stats/rate/hour/limit | Hourly rate limit | No | Yes | N/A | Hourly usage stats |

### Parameters Reference

#### GET /stats/rate/limit
**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| api_key | string | Yes | - | API key (required) |

## WebSocket Control

WebSocket connections do not use REST endpoints for control. See `websocket_full.md` for details.

## Rate Limit Response Headers

All endpoints return rate limit information in response headers:

```
X-RateLimit-Limit: 50
X-RateLimit-Remaining: 45
X-RateLimit-Reset: 1706284800
```

## Error Responses

All endpoints follow consistent error format:

```json
{
  "Response": "Error",
  "Message": "Error description",
  "Type": 99,
  "Data": {}
}
```

**Error Types:**
- Type 1: General error
- Type 2: Invalid parameter
- Type 99: Rate limit exceeded

## Notes on API Versioning

- Most endpoints use implicit v1 (no version in path)
- Some endpoints have v2 versions with enhanced data
- WebSocket uses explicit v2 in path: `/v2`
- Older endpoints maintained for backward compatibility

## Cache Duration

Different endpoints have different cache durations:
- Current prices: 10 seconds
- Historical data: Longer (varies by endpoint)
- Metadata: Several hours
- News: Real-time to 1 minute

Check API response or documentation for specific endpoint cache info.

## Data Source

- `e=CCCAGG`: CryptoCompare Aggregate Index (volume-weighted across 170+ exchanges)
- `e=Binance`: Specific exchange data
- Exchange parameter available on most price endpoints
