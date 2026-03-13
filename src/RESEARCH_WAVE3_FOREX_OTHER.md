# Research Wave 3: Forex, Remaining Stocks, Aggregators, Intelligence Feeds

**Date:** 2026-03-13
**Scope:** Trading/account API capabilities, exact REST endpoints, protocol details

---

## Table of Contents

1. [OANDA (Forex — Full Trading)](#1-oanda-forex--full-trading)
2. [Dukascopy (Forex — FIX + JForex, No Public REST)](#2-dukascopy-forex--fix--jforex-no-public-rest)
3. [AlphaVantage (Data Only)](#3-alphavantage-data-only)
4. [Futu (China Stocks — TCP Protobuf Trading)](#4-futu-china-stocks--tcp-protobuf-trading)
5. [Tinkoff (Russia Stocks — gRPC Trading)](#5-tinkoff-russia-stocks--grpc-trading)
6. [MOEX ISS (Russia — Data Only)](#6-moex-iss-russia--data-only)
7. [J-Quants (Japan — Data Only)](#7-j-quants-japan--data-only)
8. [KRX (Korea — Official Open API + Scraper)](#8-krx-korea--official-open-api--scraper)
9. [CryptoCompare (Data Only)](#9-cryptocompare-data-only)
10. [Yahoo Finance (Unofficial — Data Only)](#10-yahoo-finance-unofficial--data-only)
11. [Coinglass (Intelligence Feed — Data Only)](#11-coinglass-intelligence-feed--data-only)
12. [FRED (Federal Reserve — Data Only)](#12-fred-federal-reserve--data-only)

---

## 1. OANDA (Forex — Full Trading)

**Status:** FULL TRADING SUPPORTED
**Base URL:** `https://api-fxtrade.oanda.com` (live), `https://api-fxpractice.oanda.com` (practice)
**Authentication:** Bearer token in `Authorization` header
**Protocol:** REST/HTTP + JSON

### Order Endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST | `/v3/accounts/{accountID}/orders` | Place an order |
| GET | `/v3/accounts/{accountID}/orders` | List orders (filterable) |
| GET | `/v3/accounts/{accountID}/pendingOrders` | List pending orders only |
| GET | `/v3/accounts/{accountID}/orders/{orderSpecifier}` | Get single order |
| PUT | `/v3/accounts/{accountID}/orders/{orderSpecifier}` | Replace/amend order |
| PUT | `/v3/accounts/{accountID}/orders/{orderSpecifier}/cancel` | Cancel order |
| PUT | `/v3/accounts/{accountID}/orders/{orderSpecifier}/clientExtensions` | Update order metadata |

### Supported Order Types

- `MARKET` — immediate fill at market price
- `LIMIT` — fill at specified price or better
- `STOP` — stop order
- `MARKET_IF_TOUCHED` — MIT order
- `TAKE_PROFIT` — standalone TP order
- `STOP_LOSS` — standalone SL order

### Bracket / Conditional Order Fields (on fill)

These fields are attached to the parent order and trigger when the parent fills:

```json
{
  "order": {
    "type": "LIMIT",
    "instrument": "EUR_USD",
    "units": "10000",
    "price": "1.09000",
    "timeInForce": "GTC",
    "takeProfitOnFill": {
      "price": "1.10000",
      "timeInForce": "GTC"
    },
    "stopLossOnFill": {
      "price": "1.08000",
      "timeInForce": "GTC"
    },
    "trailingStopLossOnFill": {
      "distance": "0.00500",
      "timeInForce": "GTC"
    }
  }
}
```

### Account Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/v3/accounts` | List all authorized accounts |
| GET | `/v3/accounts/{accountID}` | Full account details (orders, trades, positions) |
| GET | `/v3/accounts/{accountID}/summary` | Account summary only |
| PATCH | `/v3/accounts/{accountID}/configuration` | Modify alias and margin rate |
| GET | `/v3/accounts/{accountID}/instruments` | Tradeable instruments for account |
| GET | `/v3/accounts/{accountID}/changes` | Poll account state changes since transaction ID |

### Position Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/v3/accounts/{accountID}/positions` | All positions |
| GET | `/v3/accounts/{accountID}/openPositions` | Open positions only |
| GET | `/v3/accounts/{accountID}/positions/{instrument}` | Specific instrument position |
| PUT | `/v3/accounts/{accountID}/positions/{instrument}/close` | Close position |

**Close Position Body:**
```json
{
  "longUnits": "ALL",    // "ALL", "NONE", or decimal
  "shortUnits": "NONE"
}
```

### Transaction Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/v3/accounts/{accountID}/transactions` | List transactions (time-based query) |
| GET | `/v3/accounts/{accountID}/transactions/{transactionID}` | Single transaction |
| GET | `/v3/accounts/{accountID}/transactions/idrange` | Range by ID |
| GET | `/v3/accounts/{accountID}/transactions/sinceid` | Since specific ID |
| GET | `/v3/accounts/{accountID}/transactions/stream` | Real-time streaming |

**Note:** No explicit `TRANSFER` type documented. Transaction types include `MARKET_ORDER`, `ORDER_FILL`, and more. Use `type=` query param with `TransactionFilter` values for filtering.

### Trade Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/v3/accounts/{accountID}/trades` | List trades |
| GET | `/v3/accounts/{accountID}/openTrades` | Open trades only |
| GET | `/v3/accounts/{accountID}/trades/{tradeSpecifier}` | Single trade |
| PUT | `/v3/accounts/{accountID}/trades/{tradeSpecifier}/close` | Close trade |
| PUT | `/v3/accounts/{accountID}/trades/{tradeSpecifier}/orders` | Attach TP/SL to trade |
| PUT | `/v3/accounts/{accountID}/trades/{tradeSpecifier}/clientExtensions` | Update metadata |

### Pricing (Market Data)

| Method | Path | Description |
|--------|------|-------------|
| GET | `/v3/accounts/{accountID}/pricing` | Current bid/ask prices |
| GET | `/v3/accounts/{accountID}/pricing/stream` | Streaming prices |
| GET | `/v3/instruments/{instrument}/candles` | OHLCV candles |
| GET | `/v3/instruments/{instrument}/orderBook` | Order book |
| GET | `/v3/instruments/{instrument}/positionBook` | Position book |

### Optional Traits Applicability

- `TradingApi`: YES — full order lifecycle
- `AccountApi`: YES — balance, margin, config
- `PositionsApi`: YES — open/close positions
- `TransactionsApi`: YES — history (no explicit transfer type)
- Bracket orders: YES (via `takeProfitOnFill`, `stopLossOnFill`, `trailingStopLossOnFill`)
- Trailing stop: YES (distance-based via `trailingStopLossOnFill`)
- Cancel-all: NO (no single endpoint; must cancel individually)
- Batch orders: NO

---

## 2. Dukascopy (Forex — FIX + JForex, No Public REST)

**Status:** TRADING SUPPORTED — but NOT via public REST API
**Protocols available:** FIX 4.4, JForex (Java SDK), community wrappers

### What Dukascopy Actually Provides

1. **FIX 4.4 API** — Binary FIX protocol for institutional clients
   - Real-time data feed
   - Submit, modify, cancel orders
   - Automated trading notifications
   - Requires approved FIX connection credentials
   - Not suitable for casual REST integration

2. **JForex Java SDK** — Java-based trading platform SDK
   - Full trading functionality via Java interfaces
   - Direct connection to Dukascopy trade servers over TLS
   - Can develop custom strategies and applications

3. **No Official Public REST API** — Dukascopy does NOT offer a public REST/HTTP trading API.

### Community Workarounds

- **dukas-proxy** (GitHub: `after-the-sunrise/dukas-proxy`): Standalone server that wraps JForex SDK and exposes REST + WebSocket endpoints. Unofficial, requires Java + JForex SDK license.
- **dukascopy-api-websocket** (GitHub: `ismailfer/dukascopy-api-websocket`): Spring Boot wrapper exposing market data, account data, and order placement via REST/WebSocket.

### Conclusion for Rust Implementation

Dukascopy **cannot be implemented** as a standard REST connector. Options:
- Use Dukascopy's **historical tick data** HTTP endpoint (`https://datafeed.dukascopy.com/datafeed/`) for data-only access (unofficial, not documented, used by community tools)
- Skip trading integration entirely — return `UnsupportedOperation` for all trading traits

### Historical Data Endpoint (Unofficial)

```
https://datafeed.dukascopy.com/datafeed/{INSTRUMENT}/{YEAR}/{MONTH}/{DAY}/{HOUR}_ticks.bi5
```
Binary BI5 format (LZMA-compressed), tick data only.

### Optional Traits Applicability

- `TradingApi`: NO (not via REST)
- `AccountApi`: NO
- `PositionsApi`: NO
- Data streaming: Only via FIX or JForex SDK

---

## 3. AlphaVantage (Data Only)

**Status:** DATA ONLY — no trading endpoints
**Base URL:** `https://www.alphavantage.co/query`
**Authentication:** `apikey` query parameter
**Protocol:** REST/HTTP + JSON or CSV

### Data Categories

All requests use `GET https://www.alphavantage.co/query?function=...&apikey=...`

#### Time Series (Equities)
- `TIME_SERIES_INTRADAY` — intraday OHLCV (1min, 5min, 15min, 30min, 60min)
- `TIME_SERIES_DAILY` — daily OHLCV
- `TIME_SERIES_DAILY_ADJUSTED` — daily adjusted (split/dividend)
- `TIME_SERIES_WEEKLY` — weekly OHLCV
- `TIME_SERIES_WEEKLY_ADJUSTED`
- `TIME_SERIES_MONTHLY`
- `TIME_SERIES_MONTHLY_ADJUSTED`

#### Fundamental Data
- `OVERVIEW` — company overview, ratios, fundamentals
- `INCOME_STATEMENT`
- `BALANCE_SHEET`
- `CASH_FLOW`
- `EARNINGS`
- `LISTING_STATUS` — active/delisted tickers
- `EARNINGS_CALENDAR`
- `IPO_CALENDAR`

#### Forex (FX)
- `FX_INTRADAY`
- `FX_DAILY`
- `FX_WEEKLY`
- `FX_MONTHLY`
- `CURRENCY_EXCHANGE_RATE` — real-time rate

#### Crypto
- `CRYPTO_INTRADAY`
- `DIGITAL_CURRENCY_DAILY`
- `DIGITAL_CURRENCY_WEEKLY`
- `DIGITAL_CURRENCY_MONTHLY`

#### Economic Indicators
- `REAL_GDP`, `REAL_GDP_PER_CAPITA`
- `FEDERAL_FUNDS_RATE`
- `CPI`
- `INFLATION`
- `RETAIL_SALES`
- `DURABLES`
- `UNEMPLOYMENT`
- `NONFARM_PAYROLL`

#### Technical Indicators
- 50+ functions: `SMA`, `EMA`, `WMA`, `DEMA`, `TEMA`, `TRIMA`, `KAMA`, `MAMA`
- `MACD`, `MACDEXT`, `STOCH`, `STOCHF`, `RSI`, `STOCHRSI`
- `WILLR`, `ADX`, `ADXR`, `APO`, `PPO`, `MOM`, `BOP`
- `CCI`, `CMO`, `ROC`, `ROCR`, `AROON`, `AROONOSC`, `MFI`
- `TRIX`, `ULTOSC`, `DX`, `MINUS_DI`, `PLUS_DI`, `MINUS_DM`, `PLUS_DM`
- `BBANDS`, `MIDPOINT`, `MIDPRICE`, `SAR`, `TRANGE`, `ATR`, `NATR`
- `AD`, `ADOSC`, `OBV`, `HT_TRENDLINE`, `HT_SINE`, `HT_TRENDMODE`
- `HT_DCPERIOD`, `HT_DCPHASE`, `HT_PHASOR`

#### News & Sentiment
- `NEWS_SENTIMENT`
- `TOP_GAINERS_LOSERS`

### Optional Traits Applicability

- `TradingApi`: NO
- `AccountApi`: NO
- `MarketDataApi`: YES (OHLCV, fundamentals, fx, crypto, economic)

---

## 4. Futu (China Stocks — TCP Protobuf Trading)

**Status:** FULL TRADING SUPPORTED — via Futu OpenD gateway
**Protocol:** TCP sockets + Protocol Buffers (NOT REST/HTTP)
**Architecture:** Requires local/cloud Futu OpenD daemon process running

### Architecture Note

Futu OpenAPI does NOT use REST. It uses a **TCP + Protobuf** protocol:
1. Run `futu-openD` locally (or on cloud server)
2. Client connects to OpenD via TCP (default port: 11111)
3. Messages exchanged as Protocol Buffer payloads with packet IDs

**Rate Limit:** Max 15 requests per 30 seconds per account; minimum 0.02s between consecutive requests.

### Trading Functions

| Function | Protocol ID | Description |
|----------|-------------|-------------|
| `place_order` / `PlaceOrder` | 2202 | Place a new order |
| `modify_order` / `ModifyOrder` | 2205 | Modify or cancel order |
| `cancel_order` (via ModifyOrder) | 2205 | Cancel with `ModifyOrderOp.CANCEL` |
| `get_order_list` / `GetOrderList` | 2201 | List open/historical orders |
| `get_order_fee` | — | Get order fee estimate |

#### PlaceOrder Request Fields

```
packetID       - unique request ID
TrdHeader      - account + environment header
  accID        - account ID
  trdEnv       - TrdEnv.REAL or TrdEnv.SIMULATE
trdSide        - TrdSide.BUY or TrdSide.SELL
orderType      - OrderType enum
code           - security code (e.g., "HK.00700")
qty            - quantity (float64)
price          - price (float64)
auxPrice       - aux/trigger price (optional)
timeInForce    - TimeInForce enum (optional)
fillOutsideRTH - bool (optional)
remark         - custom note (optional)
```

#### Supported Order Types

- `Normal` (Limit)
- `Market`
- `Stop`
- `Stop Limit`
- `Trailing Stop`
- `Trailing Stop Limit`
- `Market If Touched`
- `Limit If Touched`

### Account Functions

| Function | Protocol ID | Description |
|----------|-------------|-------------|
| `get_acc_list` | 2001 | List all trading accounts |
| `get_funds` / `accinfo_query` | 2101 | Get account balance/funds |
| `get_position_list` | 2102 | Get positions |
| `get_acc_cash_flow` | — | Cash flow history |
| `sub_acc_push` | — | Subscribe to account push updates |
| `unlock_trade` | — | Unlock trading (PIN required) |

#### get_funds Response Fields

- Total net assets
- Securities/fund/bond asset values
- Cash in multiple currencies (HKD, USD, CNH, JPY, SGD, AUD, CAD, MYR)
- Buying power (long + short)
- Market values
- Initial/maintenance/margin-call margins
- Risk status (LEVEL1–LEVEL9)
- Withdrawable cash
- Unrealized/realized P&L

#### get_position_list Response Fields

```
position_side, code, stock_name, position_market
qty, can_sell_qty, currency, nominal_price
cost_price, average_cost, diluted_cost
market_val, pl_ratio, pl_val
today_pl_val, today_trd_val
today_buy_qty, today_buy_val, today_sell_qty, today_sell_val
unrealized_pl, realized_pl, position_id
```

### Markets Supported

Hong Kong (HK), US, China A-shares (CN), Singapore (SG)

### Optional Traits Applicability

- `TradingApi`: YES (via TCP Protobuf, not REST)
- `AccountApi`: YES
- `PositionsApi`: YES
- Trailing stop: YES
- Bracket orders: NOT native (separate stop/TP orders required)
- Cancel-all: NO (must cancel individually)

**Implementation Note:** Rust implementation must use TCP + protobuf (tonic or manual proto encoding), NOT reqwest/HTTP. This is a non-standard connector.

---

## 5. Tinkoff (Russia Stocks — gRPC Trading)

**Status:** FULL TRADING SUPPORTED — via gRPC
**Protocol:** gRPC (primary), gRPC-Web (browser), Swagger/REST proxy (available)
**Production endpoint:** `invest-public-api.tinkoff.ru:443` (TLS)
**Sandbox endpoint:** `sandbox-invest-public-api.tinkoff.ru:443` (TLS)
**Authentication:** `Authorization: Bearer <access_token>` header

### REST Gateway

A Swagger/REST proxy is available at: `https://invest-public-api.tinkoff.ru/rest/`
Full Swagger UI at: `https://tinkoff.github.io/investAPI/swagger-ui/`

REST paths follow pattern: `/tinkoff.public.invest.api.contract.v1.{ServiceName}/{MethodName}`

### OrdersService

| gRPC Method | Description |
|-------------|-------------|
| `PostOrder` | Place exchange order |
| `CancelOrder` | Cancel exchange order |
| `GetOrderState` | Get order status |
| `GetOrders` | List active orders for account |
| `ReplaceOrder` | Modify existing order |

#### PostOrder Request Fields

```
figi             - instrument FIGI identifier
quantity         - lot count (int64)
price            - limit price as Quotation (optional for market)
direction        - OrderDirection enum (BUY/SELL)
account_id       - account ID string
order_type       - OrderType enum (LIMIT/MARKET/BESTPRICE)
order_id         - idempotency key (string)
instrument_id    - alternative to figi
```

### StopOrdersService

| gRPC Method | Description |
|-------------|-------------|
| `PostStopOrder` | Place stop order |
| `GetStopOrders` | List active stop orders |
| `CancelStopOrder` | Cancel stop order |

#### PostStopOrder Request Fields

```
figi/instrument_id - security identifier
quantity           - lot count
price              - order price (Quotation)
stop_price         - trigger price (Quotation)
direction          - OrderDirection
account_id         - account ID
expiration_type    - StopOrderExpirationType (GOOD_TILL_DATE/GOOD_TILL_CANCEL)
stop_order_type    - StopOrderType enum (STOP_LOSS/TAKE_PROFIT/STOP_LIMIT)
expire_date        - expiry timestamp (optional)
```

### OperationsService

| gRPC Method | Description |
|-------------|-------------|
| `GetOperations` | List operations for account |
| `GetPortfolio` | Get portfolio by account |
| `GetPositions` | List positions for account |
| `GetWithdrawLimits` | Available withdrawal balance |
| `GetBrokerReport` | Broker report generation |
| `GetDividendsForeignIssuer` | Foreign income report |
| `GetOperationsByCursor` | Paginated operations list |

### OperationsStreamService

| gRPC Method | Description |
|-------------|-------------|
| `PortfolioStream` | Server-side stream of portfolio updates |
| `PositionsStream` | Server-side stream of position changes |

### UsersService

| gRPC Method | Description |
|-------------|-------------|
| `GetAccounts` | List user accounts |
| `GetMarginAttributes` | Margin indicators for account |
| `GetUserTariff` | Rate limits and tariff info |
| `GetInfo` | User profile information |

### InstrumentsService (partial)

- `GetInstrumentBy` — search by FIGI/ticker/ISIN
- `Shares`, `Bonds`, `Etfs`, `Futures`, `Options`, `Currencies` — catalog endpoints
- `GetTradingSchedules` — trading calendar

### MarketDataService (partial)

- `GetCandles` — historical OHLCV
- `GetOrderBook` — order book
- `GetTradingStatus` — instrument status
- `GetLastPrices` — latest prices
- `GetLastTrades` — recent trades

### Optional Traits Applicability

- `TradingApi`: YES (via gRPC/REST proxy)
- `AccountApi`: YES (GetAccounts, GetPortfolio, GetMarginAttributes)
- `PositionsApi`: YES (GetPositions, GetWithdrawLimits)
- `TransactionsApi`: YES (GetOperations, GetBrokerReport)
- Stop orders: YES (separate StopOrdersService)
- Streaming: YES (gRPC server-side streams)

**Implementation Note:** Use `tonic` crate for gRPC in Rust. The REST proxy can be used with `reqwest` but the Swagger proxy does NOT support all methods equally.

---

## 6. MOEX ISS (Russia — Data Only)

**Status:** DATA ONLY — no trading endpoints
**Base URL:** `https://iss.moex.com`
**Authentication:** None required for public data; login cookie for some endpoints
**Protocol:** REST/HTTP, returns JSON or XML
**Format:** `{endpoint}.json` or `{endpoint}.xml` suffix

### Architecture Note

ISS (Information Statistical Server) is RESTful — endpoint parameters form the URL path, not just query strings. Candle example: `/iss/engines/stock/markets/shares/securities/SBER/candles?from=2024-01-01&till=2024-12-31&interval=24`

### Key Endpoint Categories

#### Market Data
```
GET /iss/engines                                           — list all engines
GET /iss/engines/{engine}/markets                          — markets in engine
GET /iss/engines/{engine}/markets/{market}/securities      — list securities
GET /iss/engines/{engine}/markets/{market}/securities/{security}/candles
                                                           — OHLCV candles
GET /iss/engines/{engine}/markets/{market}/boards/{board}/securities/{security}/candles
                                                           — candles for board
GET /iss/engines/{engine}/markets/{market}/orderbook       — order book
GET /iss/engines/{engine}/markets/{market}/trades          — recent trades
```

#### Instruments / Reference
```
GET /iss/securities                   — search all securities
GET /iss/securities/{security}        — security details + markets
GET /iss/securities/{security}/indices — indices containing security
```

#### Historical Data
```
GET /iss/history/engines/{engine}/markets/{market}/securities/{security}
                                       — historical trade data
GET /iss/history/engines/{engine}/markets/{market}/boards/{board}/securities/{security}
```

#### Indices
```
GET /iss/statistics/engines/stock/markets/index/analytics
GET /iss/indices                       — index list
```

#### FX Rates / Fixing
```
GET /iss/statistics/engines/currency/markets/selt/rates   — FX fixing rates
```

#### Corporate Actions (CCI)
```
GET /iss/securities/{security}/dividends
GET /iss/statistics/engines/stock/markets/shares/dividends
```

### candles Interval Codes

| Code | Interval |
|------|----------|
| 1 | 1 minute |
| 10 | 10 minutes |
| 60 | 1 hour |
| 24 | 1 day |
| 7 | 1 week |
| 31 | 1 month |
| 4 | 1 quarter |

### Optional Traits Applicability

- `TradingApi`: NO
- `AccountApi`: NO
- `MarketDataApi`: YES (real-time quotes, order book, candles)
- `HistoricalDataApi`: YES

---

## 7. J-Quants (Japan — Data Only)

**Status:** DATA ONLY — no trading endpoints
**Base URL:** `https://api.jquants.com/v1` (V1, legacy) / V2 now available
**Authentication:** JWT — POST `/token/auth_user` with email+password → `refreshToken`, then POST `/token/auth_refresh` → `idToken` (24h validity)
**Protocol:** REST/HTTP + JSON

### Authentication Flow

```
POST https://api.jquants.com/v1/token/auth_user
Body: { "mailaddress": "...", "password": "..." }
→ Response: { "refreshToken": "..." }

POST https://api.jquants.com/v1/token/auth_refresh?refreshtoken={token}
→ Response: { "idToken": "..." }  // valid 24 hours

All subsequent requests: Authorization: Bearer {idToken}
```

### All Endpoints

#### Authentication
```
POST /token/auth_user             — get refresh token
POST /token/auth_refresh          — get ID token (24h)
```

#### Listed Info
```
GET /listed/info                  — listed company information
  params: code (ticker), date
```

#### Prices
```
GET /prices/daily_quotes          — daily OHLCV + turnover
  params: code, date, from, to, pagingKey
GET /prices/prices_am             — morning session prices
  params: date
```

#### Financial Data
```
GET /fins/statements              — financial statements (BS/PL/CF)
  params: code, date, pagingKey
GET /fins/fs_details              — detailed financial statements
GET /fins/dividend                — dividend information
  params: code, date, from, to
GET /fins/announcement            — earnings announcement schedule
  params: today (YYYY-MM-DD)
```

#### Market Data
```
GET /markets/trades_spec          — trading specifications by investor type
GET /markets/weekly_margin_interest  — weekly margin trading interest
GET /markets/short_selling        — short selling by sector
GET /markets/short_selling_positions — reported short positions (since Aug 2025)
GET /markets/daily_margin_interest   — daily margin trading interest
GET /markets/breakdown            — trading breakdown (J-Quants Pro only)
GET /markets/trading_calendar     — exchange trading calendar
  params: holidayDivision, from, to
```

#### Index Data
```
GET /indices/topix                — TOPIX index daily prices
  params: from, to
```

#### Derivatives (J-Quants Pro)
```
GET /option/index_option          — index option daily prices
GET /derivatives/futures          — futures daily prices
GET /derivatives/options          — options daily prices
```

### Free vs Paid Tiers

| Feature | Free | Light | Standard | Premium |
|---------|------|-------|----------|---------|
| Price data history | 12 weeks | 2 years | 10 years | Full |
| Financial statements | Yes | Yes | Yes | Yes |
| Intraday data | No | No | No | Pro |
| Options/Futures | No | No | No | Yes |

### Optional Traits Applicability

- `TradingApi`: NO
- `HistoricalDataApi`: YES
- `FundamentalsApi`: YES (financial statements)
- `MarketCalendarApi`: YES

---

## 8. KRX (Korea — Official Open API + Scraper)

**Status:** DATA ONLY — no trading endpoints
**Official API:** `https://openapi.krx.co.kr/` — requires registration and API key approval
**Scraper approach:** Also widely scraped via `https://data.krx.co.kr/` (hidden form POST endpoints)

### Official KRX Open API

- Registration required at `https://openapi.krx.co.kr/`
- API key approval takes up to 1 business day
- Data availability: T-1 (previous day), historical from 2010 onwards
- Returns JSON

**Known endpoint pattern:**
```
POST https://openapi.krx.co.kr/contents/COM/GenerateOTP.jspx
   (generate OTP token)
POST https://openapi.krx.co.kr/contents/COM/CheckOTP.jspx
   (verify OTP)
GET https://openapi.krx.co.kr/contents/MKD/04/0404/04040200/mkd04040200.jspx
   (market data, format varies by module)
```

The endpoint structure is not publicly documented in a clean REST format — it uses internal JSP endpoint names.

### Community Scraper Approach (pykrx)

The most common approach uses hidden POST endpoints on `data.krx.co.kr`:

```
POST https://data.krx.co.kr/comm/bldAttendant/getJsonData.cmd
Body: bld=dbms/MDC/STAT/standard/MDCSTAT01501  (for stock price data)
      search_bas_ymd=20241231
      mktId=STK  (KOSPI) or KSQ (KOSDAQ) or KNX (KONEX)
```

**Important:** These are unofficial scraper endpoints. KRX adds delays/captchas for heavy use.

### Available Data Types

- KOSPI, KOSDAQ, KONEX stock prices (OHLCV)
- Index prices (KOSPI 200, KOSDAQ 150, etc.)
- Bond market data
- Derivatives (futures, options)
- Foreign investor trading data
- Block trade data
- Market capitalization
- Trading calendar

### Rust Library

`krx-rs` crate provides a typed Rust client for the KRX Open API.

### Optional Traits Applicability

- `TradingApi`: NO
- `MarketDataApi`: YES (with scraper or official API key)
- Data freshness: T-1 (no real-time via official API)

---

## 9. CryptoCompare (Data Only)

**Status:** DATA ONLY — no trading endpoints
**Note:** CryptoCompare was acquired by CoinDesk/CCData; legacy API redirects to `developers.coindesk.com`
**Base URL (legacy):** `https://min-api.cryptocompare.com`
**Base URL (new):** `https://data-api.cryptocompare.com` / `https://data-api.ccdata.io`
**Authentication:** API key in `Authorization: Apikey {key}` header or `api_key` query param

### Legacy API Endpoints (min-api.cryptocompare.com)

#### Price / Aggregates
```
GET /data/price              — single symbol price
GET /data/pricemulti         — multiple symbol prices
GET /data/pricemultifull     — full price data with metadata
GET /data/generateAvg        — market average
GET /data/dayAvg             — daily average
```

#### Historical OHLCV
```
GET /data/v2/histoday        — daily OHLCV
  params: fsym, tsym, limit(max 2000), toTs, aggregate, e, allData
GET /data/v2/histohour       — hourly OHLCV
  params: fsym, tsym, limit(max 2000), toTs, aggregate, e
GET /data/v2/histominute     — minute OHLCV (7 day window)
  params: fsym, tsym, limit(max 2000), toTs, aggregate, e
GET /data/histo/minute/daily — minute CSV (enterprise only)
```

#### Exchange Data
```
GET /data/exchanges/general  — exchange metadata
GET /data/v2/cccagg/pairs    — CCCAGG index pairs
GET /data/top/exchanges/full — top exchanges for pair
GET /data/top/mktcapfull     — top by market cap
GET /data/top/volumes        — top by volume
GET /data/top/pairs          — top trading pairs for coin
GET /data/top/totalvolfull   — top by total volume
```

#### Coin Data
```
GET /data/all/coinlist       — full coin list with metadata
GET /data/blockchain/list    — blockchain coin list
GET /data/blockchain/histo/day — blockchain metrics (transactions, active addresses)
```

#### Social Stats
```
GET /data/social/coin/latest          — latest social stats
GET /data/social/coin/histo/day       — daily historical social
GET /data/social/coin/histo/hour      — hourly historical social
```

**Social fields:** Reddit (subscribers, active_users, posts, comments), Twitter (followers, following, statuses), GitHub (stars, forks, subscribers)

#### News
```
GET /data/v2/news/            — latest news
GET /data/news/categories     — news categories
GET /data/news/feeds          — news sources
```

#### Mining
```
GET /data/mining/equipment/used — mining hardware list
GET /data/mining/pools/general  — mining pool list
```

#### Streaming (WebSocket)
```
wss://streamer.cryptocompare.com  — real-time price/trade/order book streaming
```

### Optional Traits Applicability

- `TradingApi`: NO
- `MarketDataApi`: YES (real-time + historical OHLCV, social, news)
- Social data: YES (unique feature)
- Mining data: YES

---

## 10. Yahoo Finance (Unofficial — Data Only)

**Status:** DATA ONLY — no trading endpoints
**Note:** UNOFFICIAL API. Yahoo shut down official API in 2017. Endpoints are undocumented and subject to change. No SLA.
**Base URLs:** `https://query1.finance.yahoo.com` or `https://query2.finance.yahoo.com` (load balanced)
**Authentication:** None required (cookie-based crumb system for some endpoints)
**Rate limits:** Not documented; aggressive use triggers blocks

### Core Endpoints

#### Historical OHLCV (Primary)
```
GET /v8/finance/chart/{ticker}
  params:
    period1       - unix timestamp (start)
    period2       - unix timestamp (end)
    interval      - 1m, 2m, 5m, 15m, 30m, 60m, 90m, 1h, 1d, 5d, 1wk, 1mo, 3mo
    range         - 1d, 5d, 1mo, 3mo, 6mo, 1y, 2y, 5y, 10y, ytd, max
    includePrePost - bool (pre/after market data)
    events        - dividends, splits, capitalGains
```

**Response:** JSON with timestamps array + OHLCV arrays (open, high, low, close, volume)

#### Quote Summary (Fundamentals)
```
GET /v10/finance/quoteSummary/{ticker}?modules={module1,module2}
```

**Available modules:**
- `assetProfile` — company description, sector, industry
- `summaryProfile`
- `summaryDetail` — market cap, P/E, beta, 52w range
- `financialData` — revenue, margins, cash, debt, analyst targets
- `defaultKeyStatistics` — EPS, book value, short interest
- `incomeStatementHistory` — P&L statements
- `incomeStatementHistoryQuarterly`
- `balanceSheetHistory`
- `balanceSheetHistoryQuarterly`
- `cashflowStatementHistory`
- `cashflowStatementHistoryQuarterly`
- `earnings` — quarterly/annual EPS
- `earningsHistory`
- `earningsTrend` — analyst estimates
- `institutionOwnership`
- `fundOwnership`
- `insiderTransactions`
- `insiderHolders`
- `majorHoldersBreakdown`
- `recommendationTrend` — analyst recommendations
- `upgradeDowngradeHistory`
- `calendarEvents` — earnings date, ex-dividend date

#### Real-time Quotes
```
GET /v7/finance/quote?symbols={ticker1,ticker2}
  returns: bid, ask, price, volume, market state, etc.
```

#### Options Chain
```
GET /v7/finance/options/{ticker}
GET /v7/finance/options/{ticker}?date={unix_timestamp}
```

#### Market Summary
```
GET /v6/finance/quote/marketSummary
```

#### Trending
```
GET /v1/finance/trending/US
```

#### Crumb (for some authenticated requests)
```
GET https://fc.yahoo.com/          — get cookies
GET /v1/test/getcrumb              — get crumb value
```

### Rate Limiting & Reliability

- No official rate limits documented
- Community experience: ~2,000 requests/hour before soft-blocking
- User-Agent header required to avoid 429 errors
- Some endpoints require valid session cookies + crumb token
- IP blocks possible; use rotating proxies for production

### Optional Traits Applicability

- `TradingApi`: NO
- `MarketDataApi`: YES (OHLCV, quotes, options)
- `FundamentalsApi`: YES (via quoteSummary modules)
- Reliability: LOW — unofficial, subject to breaking changes

---

## 11. Coinglass (Intelligence Feed — Data Only)

**Status:** DATA ONLY — no trading endpoints
**Base URL:** `https://open-api-v4.coinglass.com`
**Authentication:** `coinglassSecret: {api_key}` header
**Protocol:** REST/HTTP + JSON
**Versioning:** API V4 (current)

### Futures Market

```
GET /api/futures/supported-coins                    — supported coins
GET /api/futures/supported-exchange-pairs           — exchange/pair list
GET /api/futures/coins-markets                      — coin market overview
GET /api/futures/pairs-markets                      — pair market overview
GET /api/futures/coins-price-change                 — price changes
GET /api/futures/price/history                      — OHLC history
GET /api/futures/exchange-rank                      — exchange rankings
GET /api/futures/supported-exchanges                — exchange list
```

### Open Interest

```
GET /api/futures/open-interest/history                        — pair OI OHLC
GET /api/futures/open-interest/aggregated-history             — coin aggregated OI
GET /api/futures/open-interest/aggregated-stablecoin-history  — stablecoin margin
GET /api/futures/open-interest/aggregated-coin-margin-history — coin margin
GET /api/futures/open-interest/exchange-list                  — OI by exchange
GET /api/futures/open-interest/exchange-history-chart         — chart data
```

### Funding Rate

```
GET /api/futures/funding-rate/history              — pair FR OHLC history
GET /api/futures/funding-rate/oi-weight-history    — OI-weighted FR
GET /api/futures/funding-rate/vol-weight-history   — volume-weighted FR
GET /api/futures/funding-rate/exchange-list        — FR by exchange
GET /api/futures/funding-rate/accumulated-exchange-list — cumulative FR
GET /api/futures/funding-rate/arbitrage            — FR arbitrage opportunities
```

### Long/Short Ratio

```
GET /api/futures/global-long-short-account-ratio/history    — global account ratio
GET /api/futures/top-long-short-account-ratio/history       — top trader accounts
GET /api/futures/top-long-short-position-ratio/history      — top trader positions
GET /api/futures/taker-buy-sell-volume/exchange-list        — taker buy/sell
GET /api/futures/net-position/history                       — net position
GET /api/futures/v2/net-position/history                    — net position v2
```

### Liquidation

```
GET /api/futures/liquidation/history              — pair liquidation history
GET /api/futures/liquidation/aggregated-history   — coin liquidation history
GET /api/futures/liquidation/coin-list            — liquidation coins
GET /api/futures/liquidation/exchange-list        — liquidations by exchange
GET /api/futures/liquidation/order                — individual orders
GET /api/futures/liquidation/heatmap/model1       — liquidation heatmap model 1
GET /api/futures/liquidation/heatmap/model2       — liquidation heatmap model 2
GET /api/futures/liquidation/heatmap/model3       — liquidation heatmap model 3
GET /api/futures/liquidation/aggregated-heatmap/model1-3
GET /api/futures/liquidation/map                  — pair map
GET /api/futures/liquidation/aggregated-map       — coin map
GET /api/futures/liquidation/max-pain             — max pain levels
```

### Order Book (Futures)

```
GET /api/futures/orderbook/ask-bids-history       — bid/ask range history
GET /api/futures/orderbook/aggregated-ask-bids-history
GET /api/futures/orderbook/history                — order book heatmap
GET /api/futures/orderbook/large-limit-order      — large orders
GET /api/futures/orderbook/large-limit-order-history
```

### Taker Buy/Sell Volume

```
GET /api/futures/v2/taker-buy-sell-volume/history          — pair history
GET /api/futures/aggregated-taker-buy-sell-volume/history  — coin history
GET /api/futures/volume/footprint-history                  — 90-day footprint
GET /api/futures/cvd/history                               — cumulative volume delta
GET /api/futures/aggregated-cvd/history
GET /api/futures/netflow-list                              — net flow
```

### Spot Market

```
GET /api/spot/supported-coins
GET /api/spot/supported-exchange-pairs
GET /api/spot/coins-markets
GET /api/spot/pairs-markets
GET /api/spot/price/history
(+ spot order book and taker buy/sell endpoints mirroring futures)
```

### Options

```
GET /api/option/max-pain
GET /api/option/info
GET /api/option/exchange-oi-history
GET /api/option/exchange-vol-history
```

### On-Chain / Exchange Flows

```
GET /api/exchange/assets
GET /api/exchange/balance/list
GET /api/exchange/balance/chart
GET /api/exchange/chain/tx/list       — ERC-20 transfers
GET /api/chain/v2/whale-transfer      — whale transfers
GET /api/coin/unlock-list             — token unlocks
GET /api/coin/vesting                 — vesting schedules
```

### Hyperliquid Positions

```
GET /api/hyperliquid/whale-alert
GET /api/hyperliquid/whale-position
GET /api/hyperliquid/position
GET /api/hyperliquid/user-position
GET /api/hyperliquid/wallet/position-distribution
GET /api/hyperliquid/wallet/pnl-distribution
GET /api/hyperliquid/global-long-short-account-ratio/history
```

### ETF Data

```
GET /api/etf/bitcoin/flows-history
GET /api/etf/ethereum/flows-history
GET /api/etf/bitcoin/holdings
GET /api/etf/grayscale/premium-discount
GET /api/etf/grayscale/holdings
```

### Indicators & Indices

```
GET /api/index/fear-greed             — Fear & Greed index
GET /api/index/bitcoin-dominance
GET /api/index/altcoin-season
GET /api/indicator/on-chain           — on-chain metrics
GET /api/macro/m2                     — M2 money supply
GET /api/calendar/economic-data       — economic calendar
GET /api/article/list                 — news
GET /api/futures_spot_volume_ratio    — futures/spot volume ratio
```

### Optional Traits Applicability

- `TradingApi`: NO
- `IntelligenceFeedApi`: YES
- Unique data: liquidation heatmaps, funding rate arbitrage, whale tracking

---

## 12. FRED (Federal Reserve — Data Only)

**Status:** DATA ONLY — no trading endpoints
**Base URL:** `https://api.stlouisfed.org`
**Authentication:** `api_key={32-char-key}` query parameter (free registration at fred.stlouisfed.org)
**Protocol:** REST/HTTP, returns JSON, XML, XLSX, or CSV
**Output format:** Set via `file_type=json` query param (default: xml)

### Series Endpoints

```
GET /fred/series
  params: series_id, api_key, file_type, realtime_start, realtime_end

GET /fred/series/observations          ← PRIMARY endpoint
  params: series_id (required), api_key (required)
          realtime_start, realtime_end (YYYY-MM-DD, default: today)
          observation_start, observation_end
          limit (1–100000, default: 100000)
          offset (pagination)
          sort_order (asc/desc)
          units (lin/chg/ch1/pch/pc1/pca/cch/cca/log)
          frequency (d/w/bw/m/q/sa/a/wef/weth/wew/wetu/wem/wesu/wesa/bwew/bwem)
          aggregation_method (avg/sum/eop)
          output_type (1/2/3/4)
          vintage_dates (comma-separated YYYY-MM-DD)

GET /fred/series/categories            — categories for a series
GET /fred/series/release               — release for a series
GET /fred/series/search                — full-text search
  params: search_text, search_type
GET /fred/series/search/tags
GET /fred/series/search/related_tags
GET /fred/series/tags
GET /fred/series/updates               — recently updated series
GET /fred/series/vintagedates          — historical revision dates
```

### Category Endpoints

```
GET /fred/category              — category info
  params: category_id (default: 0 = root)
GET /fred/category/children
GET /fred/category/related
GET /fred/category/series       — series within category
GET /fred/category/tags
GET /fred/category/related_tags
```

### Release Endpoints

```
GET /fred/releases              — all releases
GET /fred/releases/dates        — all release dates
GET /fred/release               — specific release
  params: release_id
GET /fred/release/dates
GET /fred/release/series        — series on a release
GET /fred/release/sources
GET /fred/release/tags
GET /fred/release/related_tags
GET /fred/release/tables
```

### Source Endpoints

```
GET /fred/sources               — all data sources
GET /fred/source                — specific source
  params: source_id
GET /fred/source/releases       — releases for a source
```

### Tags Endpoints

```
GET /fred/tags                  — all tags or search tags
GET /fred/related_tags          — related tags
GET /fred/tags/series           — series matching tags
```

### Maps API

```
GET /fred/maps/shapefiles       — geographic shape files
GET /fred/maps/series/group     — series group metadata
GET /fred/maps/series/regional_data — regional data values
```

### Key Series Examples

| Series ID | Description |
|-----------|-------------|
| `FEDFUNDS` | Federal Funds Effective Rate |
| `DGS10` | 10-Year Treasury Constant Maturity Rate |
| `CPIAUCSL` | Consumer Price Index for All Urban Consumers |
| `UNRATE` | Unemployment Rate |
| `GDP` | Gross Domestic Product |
| `M2SL` | M2 Money Stock |
| `DEXUSEU` | US/Euro Exchange Rate |
| `BAMLH0A0HYM2` | ICE BofA US High Yield Index OAS |
| `T10Y2Y` | 10-Year minus 2-Year Treasury Yield Spread |

### Optional Traits Applicability

- `TradingApi`: NO
- `EconomicDataApi`: YES — comprehensive macroeconomic data
- Historical series: YES (some going back to 1913)
- Real-time data: NO (typical lag: 1 day to 1 month depending on series)

---

## Summary Matrix

| Provider | Trading | Account | Positions | Data | Protocol | Special Notes |
|----------|---------|---------|-----------|------|----------|---------------|
| OANDA | YES | YES | YES | YES | REST | Full bracket orders, trailing stop |
| Dukascopy | NO* | NO* | NO* | YES* | FIX/Java | *No public REST; FIX 4.4 or JForex only |
| AlphaVantage | NO | NO | NO | YES | REST | 50+ technical indicators, fundamentals |
| Futu | YES | YES | YES | YES | TCP+Protobuf | Requires OpenD gateway process |
| Tinkoff | YES | YES | YES | YES | gRPC | REST proxy available; use `tonic` crate |
| MOEX ISS | NO | NO | NO | YES | REST | Russian market data, real-time + historical |
| J-Quants | NO | NO | NO | YES | REST | JWT auth; financial statements; T+0 delayed |
| KRX | NO | NO | NO | YES | REST/Scraper | T-1 data; registration required for API key |
| CryptoCompare | NO | NO | NO | YES | REST/WS | Social + mining data unique feature |
| Yahoo Finance | NO | NO | NO | YES | REST | UNOFFICIAL; no SLA; crumb auth for some |
| Coinglass | NO | NO | NO | YES | REST | Liquidations, OI, funding, whale tracking |
| FRED | NO | NO | NO | YES | REST | Macroeconomic indicators, free API key |

---

## Sources

- [OANDA REST v3 Order Endpoints](https://developer.oanda.com/rest-live-v20/order-ep/)
- [OANDA Position Endpoints](https://developer.oanda.com/rest-live-v20/position-ep/)
- [OANDA Transaction Endpoints](https://developer.oanda.com/rest-live-v20/transaction-ep/)
- [Dukascopy FIX API](https://www.dukascopy.com/swiss/english/forex/api/fix-api/)
- [Dukascopy JForex API](https://www.dukascopy.com/europe/english/forex/api/jforex-api/)
- [dukas-proxy (community wrapper)](https://github.com/after-the-sunrise/dukas-proxy)
- [AlphaVantage Documentation](https://www.alphavantage.co/documentation/)
- [Futu OpenAPI — Place Orders](https://openapi.futunn.com/futu-api-doc/en/trade/place-order.html)
- [Futu OpenAPI — Get Account List](https://openapi.futunn.com/futu-api-doc/en/trade/get-acc-list.html)
- [Futu OpenAPI — Get Positions](https://openapi.futunn.com/futu-api-doc/en/trade/get-position-list.html)
- [Tinkoff Invest API — Orders](https://tinkoff.github.io/investAPI/orders/)
- [Tinkoff Invest API — GitHub](https://github.com/Tinkoff/investAPI)
- [MOEX ISS API Reference](https://iss.moex.com/iss/reference/)
- [J-Quants API Documentation](https://jpx.gitbook.io/j-quants-en/)
- [KRX Open API](https://openapi.krx.co.kr/)
- [pykrx (KRX scraper)](https://github.com/sharebook-kr/pykrx)
- [CryptoCompare/CoinDesk API](https://developers.coindesk.com/)
- [Yahoo Finance API Guide](https://scrapfly.io/blog/posts/guide-to-yahoo-finance-api)
- [Coinglass API Endpoint Overview](https://docs.coinglass.com/reference/endpoint-overview)
- [FRED API Documentation](https://fred.stlouisfed.org/docs/api/fred/)
- [FRED Series Observations](https://fred.stlouisfed.org/docs/api/fred/series_observations.html)
