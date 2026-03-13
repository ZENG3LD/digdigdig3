# Wave 4 — Endpoint Gap Analysis: Batch 11 (Asia/Russia Stocks) & Batch 12 (Forex)

> Generated: 2026-03-13
> Base path: `digdigdig3/src/`

---

## Methodology

Each connector's `endpoints.rs` was read directly and compared against official API documentation for that provider. Gaps are endpoints or capabilities documented by the provider but absent from our implementation.

---

## Batch 11 — Asia / Russia Stocks

---

### 1. Futu (`stocks/china/futu/`)

**Protocol:** TCP to OpenD daemon — Protocol Buffer messages (not HTTP REST).
**Current file:** `endpoints.rs` defines proto IDs in `proto_id` module.

#### What We Have

| Proto ID | Name | Category |
|----------|------|----------|
| 1001 | InitConnect | Connection |
| 1002 | GetGlobalState | Connection |
| 1004 | KeepAlive | Connection |
| 2001 | Trd_GetAccList | Account |
| 2004 | Trd_UnlockTrade | Account |
| 2101 | Trd_GetFunds | Account |
| 2102 | Trd_GetPositionList | Positions |
| 2201 | Trd_GetOrderList | Trading |
| 2202 | Trd_PlaceOrder | Trading |
| 2205 | Trd_ModifyOrder | Trading |
| 2211 | Trd_GetOrderFillList | Trading |
| 2221 | Trd_GetHistOrderList | Trading |
| 2231 | Trd_GetHistOrderFillList | Trading |
| 3001 | Qot_Sub | Subscription |
| 3004 | Qot_GetStaticInfo | Market Data |
| 3005 | Qot_GetSecuritySnapshot | Market Data |
| 3006 | Qot_GetPlateSet | Market Data |
| 3012 | Qot_GetOrderBook | Market Data |
| 3100 | Qot_GetKL | Market Data |
| 3103 | Qot_RequestHistoryKL | Market Data |

#### Gap Analysis

| Category | Endpoint | Proto ID | We Have? | Notes |
|----------|----------|----------|----------|-------|
| Connection | Notify (push) | 1003 | NO | Server-initiated notify push |
| Market Data | Qot_RegQotPush | 3002 | NO | Register for quote push delivery |
| Market Data | Qot_GetSubInfo | 3003 | NO | Query current subscription state |
| Market Data | Qot_GetBasicQot | 3004 | PARTIAL | We have GetStaticInfo (also 3004); GetBasicQot = real-time quote snapshot |
| Market Data | Qot_UpdateBasicQot (push) | 3005 | NO | Push: live quote updates after subscribe |
| Market Data | Qot_UpdateKL (push) | 3007 | NO | Push: new candle bar on subscribe |
| Market Data | Qot_GetRT | 3008 | NO | Get intraday 1-min tick chart |
| Market Data | Qot_UpdateRT (push) | 3009 | NO | Push: RT tick chart updates |
| Market Data | Qot_GetTicker | 3010 | NO | Get tick-by-tick trade data |
| Market Data | Qot_UpdateTicker (push) | 3011 | NO | Push: live tick-by-tick trades |
| Market Data | Qot_UpdateOrderBook (push) | 3013 | NO | Push: order book changes after subscribe |
| Market Data | Qot_GetBroker | 3014 | NO | Get broker queue (Level 2 broker rows) |
| Market Data | Qot_UpdateBroker (push) | 3015 | NO | Push: broker queue changes |
| Market Data | Qot_GetHistoryKLPoints | 3101 | NO | Multi-security historical KL points |
| Market Data | Qot_GetHistoryKLQuota | 3102 | NO | Query remaining historical KL quota |
| Market Data | Qot_RequestHistoryKLByDate | 3104 | NO | Request history KL by date range (extended) |
| Market Data | Qot_GetSecurityList | 3201 | NO | Full security list for a market/plate |
| Market Data | Qot_GetPlateSecurity | 3202 | NO | List securities in a specific plate |
| Market Data | Qot_GetOwnerPlate | 3203 | NO | Get plates a security belongs to |
| Market Data | Qot_GetFutureInfo | 3204 | NO | Futures contract details |
| Market Data | Qot_GetCapitalFlow | 3211 | NO | Capital flow (net buy/sell by period) |
| Market Data | Qot_GetCapitalDistribution | 3212 | NO | Capital distribution (large/mid/small orders) |
| Market Data | Qot_GetWarrantFilter | 3215 | NO | Filter warrants by criteria |
| Market Data | Qot_GetMarketState | 3223 | NO | Market session state (pre-market, trading, etc.) |
| Market Data | Qot_GetAdjustFactor | 3224 | NO | Stock split/dividend adjustment factors |
| Trading | Trd_UpdateOrder (push) | 2208 | NO | Push: order status change notification |
| Trading | Trd_UpdateOrderFill (push) | 2218 | NO | Push: order fill notification |
| Account | Trd_GetMaxTrdQtys | 2111 | NO | Get max order quantity for a security |
| Account | Trd_GetOrderFeeInfo | 2227 | NO | Get commission info for an order |

**Summary:** 20 protocol IDs implemented out of ~55 documented. Primary gaps are: push/streaming callbacks, intraday/tick data, broker queue (Level 2), capital flow analytics, warrant filtering, and several trading push notifications.

---

### 2. JQuants (`stocks/japan/jquants/`)

**Protocol:** REST over HTTPS.
**Base URL:** `https://api.jquants.com/v1` (V1) / V2 migrating to `https://api.jquants.com/v2`

#### What We Have (V1 endpoints)

| Endpoint | Path | Category |
|----------|------|----------|
| AuthUser | POST /token/auth_user | Auth |
| AuthRefresh | POST /token/auth_refresh | Auth |
| DailyQuotes | GET /prices/daily_quotes | Market Data |
| ListedInfo | GET /listed/info | Symbols |
| Indices | GET /indices | Indices |
| IndicesTopix | GET /indices/topix | Indices |
| DerivativesFutures | GET /derivatives/futures | Derivatives |
| DerivativesOptions | GET /derivatives/options | Derivatives |
| FinStatements | GET /fins/statements | Fundamentals |
| FinDividend | GET /fins/dividend | Fundamentals |
| FinAnnouncement | GET /fins/announcement | Fundamentals |
| MarketsTradingByType | GET /markets/trading_by_type | Market Stats |
| MarketsShortSelling | GET /markets/short_selling | Market Stats |
| MarketsBreakdown | GET /markets/breakdown | Market Stats |
| MarketsMargin | GET /markets/margin | Market Stats |
| MarketsTradingCalendar | GET /markets/trading_calendar | Calendar |
| OptionIndexOption | GET /option/index_option | Options |

#### Gap Analysis (V2 API — New/Changed Endpoints)

| Category | Endpoint | V2 Path | We Have? | Notes |
|----------|----------|---------|----------|-------|
| Auth | API Key auth | Header-based | NO | V2 dropped refresh token; uses `Authorization: Bearer {apikey}` header directly |
| Market Data | Stock Prices (V2) | GET /equities/bars/daily | NO | V2 renamed from /prices/daily_quotes |
| Market Data | Morning Session Prices | GET /equities/bars/daily/am | NO | V2 only — morning session specific OHLC |
| Market Data | Minute Bar Prices (add-on) | GET /equities/bars/minute | NO | Intraday 1-min bars; add-on plan required |
| Market Data | Tick Data (add-on) | GET /equities/trades | NO | Full tick/trade-level data; premium |
| Symbols | Listed Issue Master (V2) | GET /equities/master | NO | V2 renamed from /listed/info |
| Fundamentals | Financial Summary | GET /fins/summary | NO | V2 new: summarized financials per period |
| Fundamentals | Financial Detail (BS/PL/CF) | GET /fins/details | NO | V2 new: full balance sheet / P&L / cash flow |
| Market Stats | Margin Trading Outstanding | GET /markets/margin-interest | NO | V2 renamed from /markets/margin |
| Market Stats | Short Sale by Sector | GET /markets/short-ratio | NO | V2 new: short ratio by industry sector |
| Market Stats | Outstanding Short Positions | GET /markets/short-sale-report | NO | V2 new: aggregated short position report |
| Market Stats | Margin Alert | GET /markets/margin-alert | NO | V2 new: daily margin trading alert data |
| Market Stats | Trading Calendar (V2) | GET /markets/calendar | NO | V2 renamed from /markets/trading_calendar |
| Indices | Indices OHLC (V2) | GET /indices/bars/daily | NO | V2 renamed from /indices |
| Indices | TOPIX OHLC (V2) | GET /indices/bars/daily/topix | NO | V2 renamed from /indices/topix |
| Derivatives | Futures Prices (V2) | GET /derivatives/bars/daily/futures | NO | V2 renamed from /derivatives/futures |
| Derivatives | Index Option Prices (V2) | GET /derivatives/bars/daily/options/225 | NO | V2 renamed |
| Derivatives | Options Prices (V2) | GET /derivatives/bars/daily/options | NO | V2 renamed |
| Fundamentals | Earnings Calendar (V2) | GET /equities/earnings-calendar | NO | V2 new path; replaces /fins/announcement |
| Bulk | Bulk File List | GET /bulk/list | NO | Enumerate downloadable bulk data files |
| Bulk | Bulk File Download | GET /bulk/get | NO | Get pre-signed URL for bulk file download |
| WebSocket | Real-time streaming | N/A | NO | JQuants is REST-only; no WebSocket |

**Summary:** All 17 V1 endpoints are implemented but the API is migrating to V2 with renamed paths, new data types (minute bars, ticks), and a new auth model. V2 adds ~8 new endpoints not present in V1. Critical: V1 auth (refresh token) is being deprecated.

---

### 3. KRX (`stocks/korea/krx/`)

**Protocol:** REST over HTTPS. KRX Open API requires auth key; Public Data Portal uses `serviceKey`.
**Base URL:** `https://data-dbg.krx.co.kr` (debug/dev) / `https://data.krx.co.kr` (production)

#### What We Have

| Endpoint | Path | Category |
|----------|------|----------|
| KospiDailyTrading | GET /svc/apis/sto/stk_bydd_trd.json | KOSPI Market Data |
| KospiBaseInfo | GET /svc/apis/sto/stk_isu_base_info.json | KOSPI Symbols |
| KosdaqDailyTrading | GET /svc/apis/sto/ksq_bydd_trd.json | KOSDAQ Market Data |
| KosdaqBaseInfo | GET /svc/apis/sto/ksq_isu_base_info.json | KOSDAQ Symbols |
| KonexDailyTrading | GET /svc/apis/sto/knx_bydd_trd.json | KONEX Market Data |
| KonexBaseInfo | GET /svc/apis/sto/knx_isu_base_info.json | KONEX Symbols |
| WarrantDailyTrading | GET /svc/apis/sto/sw_bydd_trd.json | Warrants |
| SubscriptionWarrantDailyTrading | GET /svc/apis/sto/sr_bydd_trd.json | Subscription Warrants |
| IndexDailyTrading | GET /svc/apis/idx/idx_bydd_trd.json | Index Data |

#### Gap Analysis

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| ETF | ETF Daily Trading | GET /svc/apis/etf/etf_bydd_trd.json | NO | ETF daily OHLCV |
| ETF | ETF Base Info | GET /svc/apis/etf/etf_isu_base_info.json | NO | ETF metadata, fund info |
| ETF | ETF Portfolio | GET /svc/apis/etf/etf_portfolio.json | NO | ETF constituent holdings |
| ETF | ETF Net Asset Value | GET /svc/apis/etf/etf_nav.json | NO | ETF NAV history |
| Bond | Bond Daily Trading | GET /svc/apis/bon/bon_bydd_trd.json | NO | Listed bond daily trading data |
| Bond | Bond Yield Info | GET /svc/apis/bon/bon_isu_base_info.json | NO | Bond characteristics (coupon, maturity) |
| Futures | KOSPI200 Futures Daily | GET /svc/apis/drv/fut_bydd_trd.json | NO | Equity futures OHLCV |
| Futures | KTB Futures Daily | GET /svc/apis/drv/ktbfu_bydd_trd.json | NO | Treasury bond futures |
| Options | KOSPI200 Options Daily | GET /svc/apis/drv/opt_bydd_trd.json | NO | Equity options daily data |
| Index | Index Base Info | GET /svc/apis/idx/idx_isu_base_info.json | NO | Index constituent info and metadata |
| Index | Index by Sector | GET /svc/apis/idx/sec_idx_bydd_trd.json | NO | Sector indices daily data |
| ELW | ELW Daily Trading | GET /svc/apis/elw/elw_bydd_trd.json | NO | Equity-linked warrant data |
| Market | Trading by Investor Type | GET /svc/apis/sto/stk_invst_ordr.json | NO | Institutional vs retail order flow |
| Market | Short Selling Data | GET /svc/apis/sto/stk_short.json | NO | Short selling volume and ratio |
| Market | Market Summary | GET /svc/apis/mktstat/mkt_sum.json | NO | Daily market summary (total volume etc.) |
| Listing | Listing/Delisting Info | GET /svc/apis/sto/stk_listing.json | NO | IPO dates, delisting events |
| WebSocket | Real-time streaming | N/A | NO | KRX is historical/EOD only; no real-time WS |

**Summary:** 9 endpoints implemented covering basic KOSPI/KOSDAQ/KONEX daily data. Missing: entire ETF category (4 endpoints), bond category (2), derivatives/futures/options (3), advanced market stats (short selling, investor type breakdown), and ELW data.

---

### 4. MOEX (`stocks/russia/moex/`)

**Protocol:** REST over HTTPS (ISS — Informational & Statistical Server).
**Base URL:** `https://iss.moex.com/iss`

#### What We Have

| Endpoint | Path | Category |
|----------|------|----------|
| Engines | GET /engines.json | Metadata |
| EngineMarkets | GET /engines/{engine}/markets.json | Metadata |
| MarketBoards | GET /engines/{engine}/markets/{market}/boards.json | Metadata |
| Securities | GET /securities.json | Securities |
| SecurityInfo | GET /securities/{security}.json | Securities |
| MarketSecurities | GET /engines/{engine}/markets/{market}/securities.json | Market Data |
| SecurityMarketData | GET /engines/{engine}/markets/{market}/securities/{security}.json | Market Data |
| BoardSecurityData | GET /engines/{engine}/markets/{market}/boards/{board}/securities/{security}.json | Market Data |
| SecurityTrades | GET /engines/{engine}/markets/{market}/securities/{security}/trades.json | Market Data |
| SecurityOrderbook | GET /engines/{engine}/markets/{market}/securities/{security}/orderbook.json | Market Data |
| Candles | GET /engines/{engine}/markets/{market}/securities/{security}/candles.json | Historical |
| BoardCandles | GET /engines/{engine}/markets/{market}/boards/{board}/securities/{security}/candles.json | Historical |
| CandleBorders | GET /engines/{engine}/markets/{market}/securities/{security}/candleborders.json | Historical |
| HistoricalData | GET /history/engines/{engine}/markets/{market}/securities/{security}.json | Historical |
| BoardHistory | GET /history/engines/{engine}/markets/{market}/boards/{board}/securities/{security}.json | Historical |
| StockIndices | GET /statistics/engines/stock/markets/index/analytics.json | Indices |
| IndexAnalytics | GET /statistics/engines/stock/markets/index/analytics/{indexid}.json | Indices |
| FuturesSeries | GET /statistics/engines/futures/markets/forts/series.json | Derivatives |
| OptionsSeries | GET /statistics/engines/futures/markets/options/assets.json | Derivatives |
| OpenInterest | GET /statistics/engines/futures/markets/{market}/openpositions/{asset}.json | Derivatives |
| Turnovers | GET /turnovers.json | Statistics |
| EngineTurnovers | GET /engines/{engine}/turnovers.json | Statistics |
| CompanyInfo | GET /cci/info/companies.json | Corporate |
| CorporateActions | GET /cci/corp-actions.json | Corporate |
| ConsensusForecasts | GET /cci/consensus/shares-price.json | Corporate |

#### Gap Analysis

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Corporate | Dividends | GET /cci/corp-actions/dividends.json | NO | Per-security dividend history |
| Corporate | Bond Coupons | GET /cci/corp-actions/coupons.json | NO | Bond coupon schedules |
| Corporate | Shareholder Meetings | GET /cci/corp-actions/meetings.json | NO | AGM/EGM dates and resolutions |
| Ratings | Company Ratings | GET /cci/rating/companies.json | NO | Credit/issuer ratings |
| Ratings | Security Ratings | GET /cci/rating/securities.json | NO | Per-issue credit ratings |
| Ratings | Rating History | GET /cci/rating/history/companies.json | NO | Historical rating changes |
| Ratings | Aggregated Ratings | GET /cci/rating/agg/companies.json | NO | Aggregated ratings across agencies |
| Fixed Income | Bond Aggregates | GET /statistics/engines/stock/markets/bonds/aggregates.json | NO | Bond market aggregate metrics |
| Fixed Income | Month-End Accrued Interest | GET /statistics/engines/stock/markets/bonds/monthendaccints.json | NO | Accrued interest data |
| Fixed Income | Yield Curve History | GET /history/engines/stock/zcyc.json | NO | Zero coupon yield curve |
| FX / Currency | FX Fixing Rates | GET /statistics/engines/currency/markets/fixing.json | NO | Daily FX fixing from MOEX |
| FX / Currency | CBR Exchange Rates | GET /statistics/engines/currency/markets/selt/rates.json | NO | Central Bank reference rates |
| FX / Currency | Indicative FX Rates | GET /statistics/engines/futures/markets/indicativerates/securities.json | NO | Indicative rates for FX futures |
| Repo / Money | Repo Dealer Info | GET /statistics/engines/state/markets/repo/dealers.json | NO | Repo market participants |
| Repo / Money | OTC Repo Daily | GET /history/otc/providers/nsd/markets/{market}/daily.json | NO | OTC repo trades |
| Repo / Money | Central Bank Operations | GET /statistics/engines/state/markets/repo/cboper.json | NO | CBR repo operations |
| Risk | Settlement Calendar | GET /rms/engines/{engine}/objects/settlementscalendar.json | NO | T+n settlement dates |
| Risk | Risk Indicators | GET /rms/engines/{engine}/objects/irr.json | NO | Margin/initial risk requirements |
| Listing | Historical Listing Status | GET /history/engines/{engine}/markets/{market}/listing.json | NO | IPO/delist history |
| Listing | Security Tradability Ref | GET /referencedata/engines/{engine}/markets/all/securitieslisting.json | NO | Current tradability flags |
| Listing | Security Type Classes | GET /cci/info-nsd/securitybooks.json | NO | NSD security type classification |
| Disclosure | Affiliated Persons | GET /cci/reporting/affiliates/reports.json | NO | Affiliated persons reporting |
| Disclosure | Related Party Groups | GET /cci/reporting/group-related.json | NO | Related party grouping data |
| WebSocket | STOMP streaming | wss://iss.moex.com/infocx/v3/websocket | NO | Real-time push via STOMP (ws_base defined but no implementation) |

**Summary:** 25 endpoints implemented covering core market data, history, derivatives, and basic corporate info. Major gaps: dividends/coupons/ratings (corporate actions), FX/currency markets (entirely missing), repo market, risk/settlement data, yield curve, and WebSocket STOMP streaming.

---

### 5. Tinkoff (`stocks/russia/tinkoff/`)

**Protocol:** gRPC (primary) with REST proxy at `https://invest-public-api.tbank.ru/rest`.
**Note:** The REST proxy wraps gRPC — each method maps to a POST path.

#### What We Have

All methods of: MarketDataService (7), InstrumentsService (22), OrdersService (5), StopOrdersService (3), OperationsService (7), UsersService (4), SandboxService (11) = **59 total methods**.

#### Gap Analysis

| Category | Service/Method | Path | We Have? | Notes |
|----------|---------------|------|----------|-------|
| Streaming | MarketDataStreamService/MarketDataStream | gRPC streaming | NO | Bidirectional gRPC stream: candles, orderbook, trades, last price — the primary real-time data channel |
| Streaming | OrdersStreamService/TradesStream | gRPC streaming | NO | Server-side stream for executed trades/fills in real time |
| Streaming | OperationsStreamService/PortfolioStream | gRPC streaming | NO | Real-time portfolio changes stream |
| Streaming | OperationsStreamService/PositionsStream | gRPC streaming | NO | Real-time position changes stream |
| Orders | OrdersService/GetOrderBook (stream) | gRPC streaming | NO | Streaming order book — part of MarketDataStream |
| Operations | OperationsService/GetOperationsBy Cursor (V2) | /OperationsService/GetOperationsByCursor | PARTIAL | Implemented but GetOperations (legacy) is deprecated |
| Signals | SignalService/GetSignals | /SignalService/GetSignals | NO | Trading signals from T-Bank analysts |
| Signals | SignalService/GetStrategies | /SignalService/GetStrategies | NO | List available signal strategies |
| Market Data | MarketDataService/GetTechAnalysis | /MarketDataService/GetTechAnalysis | NO | Server-side technical indicator values |
| Instruments | InstrumentsService/GetIndicativeOffers | /InstrumentsService/GetIndicativeOffers | NO | OTC indicative offers data |
| Sandbox | SandboxService/GetSandboxOperationsByCursor | /SandboxService/GetSandboxOperationsByCursor | NO | Cursor-based sandbox operations |
| Account | UsersService/GetAccountsStatus | N/A | NO | Real-time account status monitoring |

**Summary:** 59 REST proxy methods implemented (very comprehensive). Critical gap: **zero gRPC streaming implementations** — all of `MarketDataStreamService`, `OrdersStreamService`, and `OperationsStreamService` are missing. These are the primary real-time data channels for Tinkoff. Also missing: SignalService and GetTechAnalysis.

---

## Batch 12 — Forex

---

### 6. AlphaVantage (`forex/alphavantage/`)

**Protocol:** REST over HTTPS. Single base URL `https://www.alphavantage.co/query` with `function=` parameter.

#### What We Have

| Function | Category |
|----------|----------|
| CURRENCY_EXCHANGE_RATE | Forex |
| FX_INTRADAY | Forex |
| FX_DAILY | Forex |
| FX_WEEKLY | Forex |
| FX_MONTHLY | Forex |
| GLOBAL_QUOTE | Stocks |
| TIME_SERIES_INTRADAY | Stocks |
| TIME_SERIES_DAILY | Stocks |
| TIME_SERIES_WEEKLY | Stocks |
| TIME_SERIES_MONTHLY | Stocks |
| DIGITAL_CURRENCY_DAILY | Crypto |
| CRYPTO_RATING | Crypto |
| SYMBOL_SEARCH | Utility |
| MARKET_STATUS | Utility |

#### Gap Analysis

| Category | Function | We Have? | Notes |
|----------|----------|----------|-------|
| Stocks | TIME_SERIES_DAILY_ADJUSTED | NO | Dividend/split-adjusted daily prices |
| Stocks | TIME_SERIES_WEEKLY_ADJUSTED | NO | Adjusted weekly prices |
| Stocks | TIME_SERIES_MONTHLY_ADJUSTED | NO | Adjusted monthly prices |
| Stocks | REALTIME_BULK_QUOTES | NO | Up to 100 symbols in one call |
| Options | REALTIME_OPTIONS | NO | Current US options chain with Greeks |
| Options | HISTORICAL_OPTIONS | NO | Historical options data |
| Intelligence | NEWS_SENTIMENT | NO | Market news + AI sentiment scoring |
| Intelligence | EARNINGS_CALL_TRANSCRIPT | NO | Full earnings call transcripts |
| Intelligence | TOP_GAINERS_LOSERS | NO | Daily top movers in US market |
| Intelligence | INSIDER_TRANSACTIONS | NO | SEC insider transaction filings |
| Intelligence | INSTITUTIONAL_HOLDINGS | NO | 13F institutional holdings |
| Intelligence | ANALYTICS | NO | Fixed & sliding window return analytics |
| Fundamentals | COMPANY_OVERVIEW | NO | Full company profile, ratios, description |
| Fundamentals | ETF_PROFILE | NO | ETF holdings, expense ratio, NAV |
| Fundamentals | DIVIDEND | NO | Dividend history for a stock |
| Fundamentals | SPLITS | NO | Stock split history |
| Fundamentals | INCOME_STATEMENT | NO | Annual and quarterly income statement |
| Fundamentals | BALANCE_SHEET | NO | Annual and quarterly balance sheet |
| Fundamentals | CASH_FLOW | NO | Annual and quarterly cash flow statement |
| Fundamentals | SHARES_OUTSTANDING | NO | Historical shares outstanding |
| Fundamentals | EARNINGS | NO | EPS history vs estimates |
| Fundamentals | EARNINGS_ESTIMATES | NO | Analyst EPS estimates |
| Fundamentals | LISTING_STATUS | NO | Active/delisted status for all symbols |
| Fundamentals | EARNINGS_CALENDAR | NO | Upcoming earnings dates |
| Fundamentals | IPO_CALENDAR | NO | Upcoming IPO calendar |
| Crypto | CRYPTO_EXCHANGE_RATE | NO | Real-time crypto exchange rate |
| Crypto | DIGITAL_CURRENCY_INTRADAY | NO | Crypto intraday data |
| Crypto | DIGITAL_CURRENCY_WEEKLY | NO | Crypto weekly OHLCV |
| Crypto | DIGITAL_CURRENCY_MONTHLY | NO | Crypto monthly OHLCV |
| Commodities | WTI | NO | WTI crude oil price history |
| Commodities | BRENT | NO | Brent crude oil price history |
| Commodities | NATURAL_GAS | NO | Natural gas price history |
| Commodities | COPPER | NO | Copper price history |
| Commodities | ALUMINUM | NO | Aluminum price history |
| Commodities | WHEAT | NO | Wheat price history |
| Commodities | CORN | NO | Corn price history |
| Commodities | COTTON | NO | Cotton price history |
| Commodities | SUGAR | NO | Sugar price history |
| Commodities | COFFEE | NO | Coffee price history |
| Commodities | GOLD | NO | Gold price history |
| Commodities | SILVER | NO | Silver price history |
| Economic | REAL_GDP | NO | US real GDP (annual/quarterly) |
| Economic | REAL_GDP_PER_CAPITA | NO | US real GDP per capita |
| Economic | TREASURY_YIELD | NO | US treasury yield curve (3m, 2y, 5y, 7y, 10y, 30y) |
| Economic | FEDERAL_FUNDS_RATE | NO | Federal funds rate history |
| Economic | CPI | NO | Consumer Price Index |
| Economic | INFLATION | NO | Annual inflation rate |
| Economic | RETAIL_SALES | NO | Monthly retail sales |
| Economic | DURABLES | NO | Durable goods orders |
| Economic | UNEMPLOYMENT | NO | Unemployment rate |
| Economic | NONFARM_PAYROLL | NO | Monthly nonfarm payroll |
| Technical | SMA | NO | Simple Moving Average |
| Technical | EMA | NO | Exponential Moving Average |
| Technical | MACD | NO | MACD |
| Technical | RSI | NO | Relative Strength Index |
| Technical | BBANDS | NO | Bollinger Bands |
| Technical | STOCH | NO | Stochastic |
| Technical | ADX | NO | Average Directional Index |
| Technical | ATR | NO | Average True Range |
| Technical | OBV | NO | On-Balance Volume |
| Technical | 40+ more | NO | Full 50+ technical indicator suite |
| WebSocket | N/A | NO | AlphaVantage is REST-only; no WebSocket |

**Summary:** 14 functions implemented (forex + basic stocks + 2 crypto + 2 utility). AlphaVantage offers ~100+ functions in total. The entire Fundamentals category (15 functions), all Economic Indicators (10), all Commodities (12), all Technical Indicators (50+), all Options (2), and all Alpha Intelligence (6) categories are missing. This is the most under-implemented connector in this batch.

---

### 7. Dukascopy (`forex/dukascopy/`)

**Protocol:** Binary file download (`.bi5` format — LZMA-compressed binary tick data). No REST API or WebSocket.

#### What We Have

| Endpoint | Path Pattern | Category |
|----------|-------------|----------|
| HistoricalTicks | /{SYMBOL}/{YYYY}/{MM}/{DD}/{HH}h_ticks.bi5 | Historical Tick Data |

#### Gap Analysis

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Historical Candles | Daily OHLCV aggregates | NO | Dukascopy provides pre-aggregated daily candle files: `{SYMBOL}/YYYY/BID_candles_day_{YYYY}.bin` — not exposed |
| Instrument List | Symbol list JSON | NO | `https://www.dukascopy.com/datafeed/metadata/instruments.json` — lists all available symbols with metadata |
| Session hours | Instrument metadata | NO | Trading hours and session info per instrument not exposed |
| Streaming | Real-time feed | NO | Dukascopy does not offer public real-time streaming; binary files are historical only |
| Data Availability | Tick availability JSON | NO | Some instruments publish available date ranges; not exposed |

**Summary:** Only 1 of the available download patterns implemented (hourly tick files). Missing: pre-aggregated daily candle files, instrument metadata/symbol list endpoint. Note: Dukascopy's data offering is inherently limited to historical binary downloads; no REST API, no WebSocket.

---

### 8. OANDA (`forex/oanda/`)

**Protocol:** REST over HTTPS. Separate streaming URL for live price/transaction streams.
**Base URL (live):** `https://api-fxtrade.oanda.com`
**Stream URL (live):** `https://stream-fxtrade.oanda.com`

#### What We Have

| Endpoint | HTTP Method | Path | Category |
|----------|-------------|------|----------|
| ListAccounts | GET | /v3/accounts | Account |
| GetAccount | GET | /v3/accounts/{id} | Account |
| GetAccountSummary | GET | /v3/accounts/{id}/summary | Account |
| GetInstruments | GET | /v3/accounts/{id}/instruments | Account |
| PollAccountChanges | GET | /v3/accounts/{id}/changes | Account |
| GetPricing | GET | /v3/accounts/{id}/pricing | Pricing |
| StreamPricing | GET (stream) | /v3/accounts/{id}/pricing/stream | Pricing |
| GetLatestCandles | GET | /v3/accounts/{id}/candles/latest | Pricing |
| GetCandles | GET | /v3/instruments/{instrument}/candles | Instruments |
| CreateOrder | POST | /v3/accounts/{id}/orders | Orders |
| ListOrders | GET | /v3/accounts/{id}/orders | Orders |
| ListPendingOrders | GET | /v3/accounts/{id}/pendingOrders | Orders |
| GetOrder | GET | /v3/accounts/{id}/orders/{orderSpec} | Orders |
| CancelOrder | PUT | /v3/accounts/{id}/orders/{orderSpec}/cancel | Orders |
| AmendOrder | PUT | /v3/accounts/{id}/orders/{orderSpec} | Orders |
| ListTrades | GET | /v3/accounts/{id}/trades | Trades |
| ListOpenTrades | GET | /v3/accounts/{id}/openTrades | Trades |
| GetTrade | GET | /v3/accounts/{id}/trades/{tradeSpec} | Trades |
| CloseTrade | PUT | /v3/accounts/{id}/trades/{tradeSpec}/close | Trades |
| ListPositions | GET | /v3/accounts/{id}/positions | Positions |
| ListOpenPositions | GET | /v3/accounts/{id}/openPositions | Positions |
| GetPosition | GET | /v3/accounts/{id}/positions/{instrument} | Positions |
| ClosePosition | PUT | /v3/accounts/{id}/positions/{instrument}/close | Positions |
| StreamTransactions | GET (stream) | /v3/accounts/{id}/transactions/stream | Transactions |

#### Gap Analysis

| Category | Endpoint | HTTP Method | Path | We Have? | Notes |
|----------|----------|-------------|------|----------|-------|
| Account | AccountConfiguration | PATCH | /v3/accounts/{id}/configuration | NO | Update account margin rate or alias |
| Instruments | InstrumentsOrderBook | GET | /v3/instruments/{instrument}/orderBook | NO | Snapshots of order book histogram |
| Instruments | InstrumentsPositionBook | GET | /v3/instruments/{instrument}/positionBook | NO | Snapshots of position book histogram |
| Trades | TradeCRCDO | PUT | /v3/accounts/{id}/trades/{tradeSpec}/orders | NO | Create/Replace/Cancel Dependent Orders (TP/SL on trade) |
| Trades | TradeClientExtensions | PUT | /v3/accounts/{id}/trades/{tradeSpec}/clientExtensions | NO | Update client-side trade labels |
| Orders | OrderClientExtensions | PUT | /v3/accounts/{id}/orders/{orderSpec}/clientExtensions | NO | Update client-side order labels |
| Transactions | TransactionList | GET | /v3/accounts/{id}/transactions | NO | List all transactions with filters |
| Transactions | TransactionDetails | GET | /v3/accounts/{id}/transactions/{transactionID} | NO | Single transaction details |
| Transactions | TransactionIDRange | GET | /v3/accounts/{id}/transactions/idrange | NO | Transactions by ID range |
| Transactions | TransactionsSinceID | GET | /v3/accounts/{id}/transactions/sinceid | NO | Transactions since a given ID |
| ForexLabs | Autochartist Patterns | GET | /labs/v1/autochartist | NO | Chart pattern recognition signals |
| ForexLabs | Economic Calendar | GET | /labs/v1/calendar | NO | Upcoming economic events |
| ForexLabs | Commitments of Traders | GET | /labs/v1/commitments_of_traders | NO | COT data from CFTC |
| ForexLabs | Historical Position Ratios | GET | /labs/v1/historical_position_ratios | NO | OANDA client position ratio history |
| ForexLabs | Orderbook Data | GET | /labs/v1/orderbook_data | NO | Aggregated client orderbook |
| ForexLabs | Spreads | GET | /labs/v1/spreads | NO | Historical spread data |

**Summary:** 24 endpoints implemented — strong coverage of core trading (accounts, orders, trades, positions, pricing, streaming). Gaps: 4 transaction REST endpoints (non-streaming), `AccountConfiguration` PATCH, 2 instrument book endpoints, 2 trade dependent-order endpoints, and the entire ForexLabs suite (6 endpoints) with unique market intelligence data (COT, autochartist, position ratios, economic calendar).

---

## Cross-Connector Summary Table

| Connector | Implemented | Estimated Total | Gap % | Highest Priority Gaps |
|-----------|------------|-----------------|-------|----------------------|
| Futu | ~20 proto IDs | ~55 | ~64% | Push streaming callbacks, tick data, broker queue, capital flow |
| JQuants | 17 (V1) | ~21 (V2) | V2 migration needed | V2 auth, minute bars, tick data, bulk downloads |
| KRX | 9 | ~26 | ~65% | ETF data, bonds, derivatives/futures/options |
| MOEX | 25 | ~50 | ~50% | Dividends, coupons, ratings, FX market, STOMP WebSocket |
| Tinkoff | 59 (REST) | 59 REST + 4 gRPC streams | gRPC missing | MarketDataStream, OrdersStream, PortfolioStream, PositionsStream |
| AlphaVantage | 14 | ~115 | ~88% | Fundamentals, Economic Indicators, Commodities, Technical Indicators |
| Dukascopy | 1 | ~3 | ~67% | Daily candle files, instrument metadata list |
| OANDA | 24 | ~40 | ~40% | Transactions REST, ForexLabs suite, order/trade client extensions |

---

## Priority Recommendations

### Critical (Blocks Core Functionality)
1. **Tinkoff — gRPC Streaming:** `MarketDataStreamService` is the only real-time data source for Tinkoff. Without it, live price data is unavailable.
2. **Futu — Push Callbacks:** Without `Qot_RegQotPush` and push handlers (3007, 3009, 3011, 3013), there is no live streaming — only polling.
3. **MOEX — WebSocket STOMP:** The `ws_base` URL is defined but there is no implementation. Real-time data requires STOMP connection.

### High Value (Data Completeness)
4. **JQuants — V2 Migration:** V1 API is being deprecated. Auth model changes and path renames affect all 17 existing endpoints.
5. **AlphaVantage — Fundamentals:** 15 fundamental data functions (earnings, income statements, balance sheets) are entirely absent.
6. **KRX — ETF + Derivatives:** ETF data (4 endpoints) and derivatives (3 endpoints) are completely missing.

### Enhancement (Analytical Value)
7. **MOEX — Corporate Actions:** Dividends, coupons, ratings are useful for fundamental analysis of Russian equities.
8. **OANDA — ForexLabs:** Unique data: COT reports, position ratios, economic calendar, autochartist — not available elsewhere.
9. **AlphaVantage — Economic Indicators & Commodities:** 22 functions covering macro data and commodity prices.
10. **Futu — Capital Flow & Tick Data:** Capital flow (3211) and RT tick data (3010) are highly valued by active traders.

---

## Sources

- [Futu OpenAPI Protocol Introduction](https://openapi.futunn.com/futu-api-doc/en/ftapi/protocol.html)
- [Futu GetCapitalFlow Proto 3211](https://openapi.futunn.com/futu-api-doc/en/quote/get-capital-flow.html)
- [Futu GitHub py-futu-api utils](https://github.com/FutunnOpen/py-futu-api/blob/master/futu/common/utils.py)
- [JQuants V2 API Reference](https://jpx-jquants.com/en/spec)
- [JQuants V1→V2 Migration Guide](https://jpx-jquants.com/en/spec/migration-v1-v2)
- [KRX Open API Portal](https://openapi.krx.co.kr/)
- [KRX Data Marketplace](https://data.krx.co.kr/)
- [MOEX ISS API Reference](https://iss.moex.com/iss/reference/)
- [MOEX API Overview](https://www.moex.com/a2920)
- [Tinkoff InvestAPI Introduction](https://russianinvestments.github.io/investAPI/)
- [T-Bank Developer Portal](https://developer.tbank.ru/invest/intro/intro)
- [AlphaVantage Documentation](https://www.alphavantage.co/documentation/)
- [AlphaVantage Complete Guide 2026](https://alphalog.ai/blog/alphavantage-api-complete-guide)
- [OANDA v20 REST API Introduction](https://developer.oanda.com/rest-live-v20/introduction/)
- [OANDA Instrument Endpoints](https://developer.oanda.com/rest-live-v20/instrument-ep/)
- [oandapyV20 Endpoints Reference](https://oanda-api-v20.readthedocs.io/en/latest/oandapyV20.endpoints.html)
- [Dukascopy Historical Data](https://www.dukascopy.com/swiss/english/marketwatch/historical/)
