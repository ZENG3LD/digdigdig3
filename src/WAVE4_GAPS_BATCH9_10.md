# Wave 4 Endpoint Gap Analysis — Batch 9 & 10

**Scope**: US Stocks (Alpaca, Finnhub, Polygon, Tiingo, TwelveData) + India Stocks (Angel One, Dhan, Fyers, Upstox, Zerodha)
**Base path**: `digdigdig3/src/`
**Date**: 2026-03-13

---

## Legend

| Symbol | Meaning |
|--------|---------|
| YES | Endpoint enum variant exists |
| NO | Missing from our endpoints.rs |
| PARTIAL | Exists but path/variant is incomplete or wrong |

---

## Batch 9 — Stocks US

---

### 1. Alpaca (`stocks/us/alpaca/endpoints.rs`)

**API Docs**: https://docs.alpaca.markets/reference/

**What we have**: Account, AccountPortfolioHistory, AccountActivities, Orders, OrderById, OrderByClientId, Positions, PositionBySymbol, Assets, AssetBySymbol, OptionContracts, Calendar, Clock, StockBars, StockBarsLatest, StockTrades, StockTradesLatest, StockQuotes, StockQuotesLatest, StockSnapshots, StockSnapshotBySymbol, OptionsSnapshots, OptionsBars, OptionsTrades, OptionsQuotes, CryptoBars, CryptoTrades, CryptoQuotes, CryptoOrderbooks, CryptoSnapshots, News, CorporateActions, Movers

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Trading — Account | GET /v2/account | YES | `Account` |
| Trading — Account | GET /v2/account/portfolio/history | YES | `AccountPortfolioHistory` |
| Trading — Account | GET /v2/account/activities/{activity_type} | YES | `AccountActivities` (path misses activity_type param) |
| Trading — Account | GET /v2/account/configurations | NO | Missing — returns risk controls, pdt_check, etc. |
| Trading — Account | PATCH /v2/account/configurations | NO | Missing — update account settings |
| Trading — Orders | GET/POST /v2/orders | YES | `Orders` |
| Trading — Orders | GET/DELETE /v2/orders/{order_id} | YES | `OrderById` |
| Trading — Orders | GET /v2/orders:by_client_order_id | YES | `OrderByClientId` |
| Trading — Positions | GET /v2/positions | YES | `Positions` |
| Trading — Positions | GET /v2/positions/{symbol} | YES | `PositionBySymbol` |
| Trading — Positions | DELETE /v2/positions | NO | Missing — close all positions |
| Trading — Positions | DELETE /v2/positions/{symbol} | NO | Missing — close a specific position |
| Trading — Watchlists | GET /v2/watchlists | NO | Missing — list all watchlists |
| Trading — Watchlists | POST /v2/watchlists | NO | Missing — create watchlist |
| Trading — Watchlists | GET /v2/watchlists/{watchlist_id} | NO | Missing — get specific watchlist |
| Trading — Watchlists | PUT /v2/watchlists/{watchlist_id} | NO | Missing — update watchlist |
| Trading — Watchlists | DELETE /v2/watchlists/{watchlist_id} | NO | Missing — delete watchlist |
| Trading — Watchlists | GET /v2/watchlists:by_name | NO | Missing — get watchlist by name |
| Trading — Assets | GET /v2/assets | YES | `Assets` |
| Trading — Assets | GET /v2/assets/{symbol_or_asset_id} | YES | `AssetBySymbol` |
| Trading — Options | GET /v2/options/contracts | YES | `OptionContracts` |
| Trading — Options | GET /v2/options/contracts/{symbol_or_contract_id} | NO | Missing — get single options contract details |
| Trading — Calendar | GET /v2/calendar | YES | `Calendar` |
| Trading — Clock | GET /v2/clock | YES | `Clock` |
| Market Data — Stock Bars | GET /v2/stocks/bars | YES | `StockBars` |
| Market Data — Stock Bars | GET /v2/stocks/{symbol}/bars | NO | Missing — single-symbol bars (vs multi) |
| Market Data — Stock Bars | GET /v2/stocks/bars/latest | YES | `StockBarsLatest` |
| Market Data — Stock Trades | GET /v2/stocks/trades | YES | `StockTrades` |
| Market Data — Stock Trades | GET /v2/stocks/{symbol}/trades | NO | Missing — single-symbol trades |
| Market Data — Stock Trades | GET /v2/stocks/trades/latest | YES | `StockTradesLatest` |
| Market Data — Stock Quotes | GET /v2/stocks/quotes | YES | `StockQuotes` |
| Market Data — Stock Quotes | GET /v2/stocks/{symbol}/quotes | NO | Missing — single-symbol quotes |
| Market Data — Stock Quotes | GET /v2/stocks/quotes/latest | YES | `StockQuotesLatest` |
| Market Data — Snapshots | GET /v2/stocks/snapshots | YES | `StockSnapshots` |
| Market Data — Snapshots | GET /v2/stocks/{symbol}/snapshot | YES | `StockSnapshotBySymbol` |
| Market Data — Auctions | GET /v2/stocks/{symbol}/auctions | NO | Missing — opening/closing auction data |
| Market Data — Options | GET /v1beta1/options/snapshots/{underlying} | YES | `OptionsSnapshots` |
| Market Data — Options | GET /v1beta1/options/bars | YES | `OptionsBars` |
| Market Data — Options | GET /v1beta1/options/trades | YES | `OptionsTrades` |
| Market Data — Options | GET /v1beta1/options/quotes | YES | `OptionsQuotes` |
| Market Data — Options | GET /v1beta1/options/chain | NO | Missing — option chain endpoint (full chain snapshot) |
| Market Data — Crypto | GET /v1beta3/crypto/us/bars | YES | `CryptoBars` |
| Market Data — Crypto | GET /v1beta3/crypto/us/trades | YES | `CryptoTrades` |
| Market Data — Crypto | GET /v1beta3/crypto/us/quotes | YES | `CryptoQuotes` |
| Market Data — Crypto | GET /v1beta3/crypto/us/latest/orderbooks | YES | `CryptoOrderbooks` |
| Market Data — Crypto | GET /v1beta3/crypto/us/snapshots | YES | `CryptoSnapshots` |
| Market Data — Screener | GET /v1beta1/screener/stocks/movers | YES | `Movers` (gainers/losers) |
| Market Data — Screener | GET /v1beta1/screener/stocks/most-actives | NO | Missing — most active stocks by volume/trade count |
| Market Data — Screener | GET /v1beta1/screener/conditions | NO | Missing — screener conditions list |
| Market Data — News | GET /v1beta1/news | YES | `News` |
| Market Data — Corporate Actions | GET /v1beta1/corporate-actions/announcements | YES | `CorporateActions` |
| WebSocket — Market Data | wss://stream.data.alpaca.markets/v2/iex | PARTIAL | URL stored but no stream enum variants |
| WebSocket — Trading | wss://api.alpaca.markets/stream | PARTIAL | URL stored but no stream enum variants |
| WebSocket — Crypto | wss://stream.data.alpaca.markets/v1beta3/crypto/us | NO | Missing — crypto WebSocket URL |

**Summary of Alpaca Gaps**: 17 missing endpoints — watchlist CRUD (6), position close (2), account config (2), single-symbol market data endpoints (3), options chain, options contract detail, most-actives screener, auctions.

---

### 2. Finnhub (`stocks/us/finnhub/endpoints.rs`)

**API Docs**: https://finnhub.io/docs/api/

**What we have**: Quote, StockCandles, TickData, BidAsk, StockSymbols, CompanyProfile, BasicFinancials, FinancialStatements, CompanyPeers, CompanyExecutives, EarningsCalendar, EpsEstimates, RevenueEstimates, PriceTarget, Recommendations, UpgradeDowngrade, MarketNews, CompanyNews, NewsSentiment, ForexExchanges, ForexSymbols, ForexCandles, ExchangeRates, CryptoExchanges, CryptoSymbols, CryptoCandles, InsiderTransactions, InsiderSentiment, CongressionalTrading, InstitutionalOwnership, PatentData, VisaApplications, UsaSpending, SupplyChain, SenateLobby, EsgScores, ExecutiveCompensation, RevenueBreakdown, TechnicalIndicators, PatternRecognition, SupportResistance, AggregateIndicators, MarketStatus, MarketHoliday, EconomicCalendar, SecFilings

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Stock Market Data | GET /quote | YES | `Quote` |
| Stock Market Data | GET /stock/candle | YES | `StockCandles` |
| Stock Market Data | GET /stock/tick | YES | `TickData` |
| Stock Market Data | GET /stock/bidask | YES | `BidAsk` |
| Stock Market Data | GET /stock/symbol | YES | `StockSymbols` |
| Company Fundamentals | GET /stock/profile2 | YES | `CompanyProfile` |
| Company Fundamentals | GET /stock/metric | YES | `BasicFinancials` |
| Company Fundamentals | GET /stock/financials | YES | `FinancialStatements` |
| Company Fundamentals | GET /stock/financials-reported | NO | Missing — as-reported financials (raw SEC data) |
| Company Fundamentals | GET /stock/peers | YES | `CompanyPeers` |
| Company Fundamentals | GET /stock/executive | YES | `CompanyExecutives` |
| Company Fundamentals | GET /stock/investment-theme | NO | Missing — investment theme data (premium) |
| Estimates | GET /calendar/earnings | YES | `EarningsCalendar` |
| Estimates | GET /stock/eps-estimate | YES | `EpsEstimates` |
| Estimates | GET /stock/revenue-estimate | YES | `RevenueEstimates` |
| Estimates | GET /stock/eps-surprise | NO | Missing — historical earnings surprises |
| Estimates | GET /stock/price-target | YES | `PriceTarget` |
| Estimates | GET /stock/recommendation | YES | `Recommendations` |
| Estimates | GET /stock/upgrade-downgrade | YES | `UpgradeDowngrade` |
| News | GET /news | YES | `MarketNews` |
| News | GET /company-news | YES | `CompanyNews` |
| News | GET /news-sentiment | YES | `NewsSentiment` |
| IPO | GET /calendar/ipo | NO | Missing — IPO calendar |
| ETF | GET /etf/holdings | NO | Missing — ETF holdings and constituents |
| ETF | GET /etf/profile | NO | Missing — ETF profile and info |
| ETF | GET /etf/country | NO | Missing — ETF country exposure |
| ETF | GET /etf/sector | NO | Missing — ETF sector allocation |
| Mutual Funds | GET /mutual-fund/holdings | NO | Missing — mutual fund holdings |
| Mutual Funds | GET /mutual-fund/profile | NO | Missing — mutual fund profile |
| Mutual Funds | GET /mutual-fund/sector | NO | Missing — mutual fund sector allocation |
| Mutual Funds | GET /mutual-fund/country | NO | Missing — mutual fund country exposure |
| Bonds | GET /bond/price | NO | Missing — bond price data |
| Bonds | GET /bond/profile | NO | Missing — bond profile |
| Bonds | GET /bond/yield-curve | NO | Missing — treasury yield curve |
| Forex | GET /forex/exchange | YES | `ForexExchanges` |
| Forex | GET /forex/symbol | YES | `ForexSymbols` |
| Forex | GET /forex/candle | YES | `ForexCandles` |
| Forex | GET /forex/rates | YES | `ExchangeRates` |
| Crypto | GET /crypto/exchange | YES | `CryptoExchanges` |
| Crypto | GET /crypto/symbol | YES | `CryptoSymbols` |
| Crypto | GET /crypto/candle | YES | `CryptoCandles` |
| Crypto | GET /crypto/profile | NO | Missing — crypto coin profile |
| Alternative Data | GET /stock/insider-transactions | YES | `InsiderTransactions` |
| Alternative Data | GET /stock/insider-sentiment | YES | `InsiderSentiment` |
| Alternative Data | GET /stock/congressional-trading | YES | `CongressionalTrading` |
| Alternative Data | GET /stock/ownership | YES | `InstitutionalOwnership` |
| Alternative Data | GET /stock/usa-patent | YES | `PatentData` |
| Alternative Data | GET /stock/visa-application | YES | `VisaApplications` |
| Alternative Data | GET /stock/usa-spending | YES | `UsaSpending` |
| Alternative Data | GET /stock/supply-chain | YES | `SupplyChain` |
| Alternative Data | GET /stock/lobbying | YES | `SenateLobby` |
| Alternative Data | GET /stock/social-sentiment | NO | Missing — social sentiment data (Reddit, Twitter) |
| Alternative Data | GET /stock/transcript | NO | Missing — earnings call transcripts |
| Alternative Data | GET /stock/transcript-list | NO | Missing — list of available transcripts |
| ESG | GET /stock/esg | YES | `EsgScores` |
| Technical Analysis | GET /indicator | YES | `TechnicalIndicators` |
| Technical Analysis | GET /scan/pattern | YES | `PatternRecognition` |
| Technical Analysis | GET /scan/support-resistance | YES | `SupportResistance` |
| Technical Analysis | GET /scan/technical-indicator | YES | `AggregateIndicators` |
| Market Info | GET /stock/market-status | YES | `MarketStatus` |
| Market Info | GET /stock/market-holiday | YES | `MarketHoliday` |
| Economic | GET /calendar/economic | YES | `EconomicCalendar` |
| Economic | GET /country | NO | Missing — country data (sector metrics, country code) |
| SEC Filings | GET /stock/filings | YES | `SecFilings` |
| SEC Filings | GET /stock/similarity-index | NO | Missing — similarity index between 10-K filings |
| WebSocket | wss://ws.finnhub.io | PARTIAL | URL stored but no stream enum variants |

**Summary of Finnhub Gaps**: 19 missing endpoints — ETF family (4), mutual fund family (4), bond family (3), IPO calendar, earnings surprise, financial-as-reported, social sentiment, transcripts (2), crypto profile, investment theme, country data, similarity index.

---

### 3. Polygon (`stocks/us/polygon/endpoints.rs`)

**API Docs**: https://polygon.io/docs/stocks

**What we have**: Tickers, TickerDetails, TickerTypes, Aggregates, PreviousClose, GroupedDaily, SingleSnapshot, AllSnapshot, UnifiedSnapshot, Trades, LastTrade, Quotes, LastQuote, SMA, EMA, MACD, RSI, Dividends, Splits, FinancialRatios, MarketStatus, MarketHolidays, News

**Note**: Polygon base URL in code is `api.massive.com` (rebranded), which is correct.

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Reference — Tickers | GET /v3/reference/tickers | YES | `Tickers` |
| Reference — Tickers | GET /v3/reference/tickers/{ticker} | YES | `TickerDetails` |
| Reference — Tickers | GET /v3/reference/tickers/types | YES | `TickerTypes` |
| Reference — Tickers | GET /v3/reference/tickers/events | NO | Missing — ticker events (name changes, delistings) |
| Reference — Conditions | GET /v3/reference/conditions | NO | Missing — trade condition codes |
| Reference — Exchanges | GET /v3/reference/exchanges | NO | Missing — all exchanges list |
| Reference — Options Contracts | GET /v3/reference/options/contracts | NO | Missing — option contracts reference data |
| Reference — Options Contracts | GET /v3/reference/options/contracts/{options_ticker} | NO | Missing — single option contract details |
| Market Data — Aggregates | GET /v2/aggs/ticker/{ticker}/range/{mult}/{span}/{from}/{to} | YES | `Aggregates` |
| Market Data — Aggregates | GET /v2/aggs/ticker/{ticker}/prev | YES | `PreviousClose` |
| Market Data — Aggregates | GET /v2/aggs/grouped/locale/us/market/stocks/{date} | YES | `GroupedDaily` |
| Market Data — Trades | GET /v3/trades/{ticker} | YES | `Trades` |
| Market Data — Trades | GET /v2/last/trade/{ticker} | YES | `LastTrade` |
| Market Data — Quotes | GET /v3/quotes/{ticker} | YES | `Quotes` |
| Market Data — Quotes | GET /v2/last/nbbo/{ticker} | YES | `LastQuote` |
| Snapshots | GET /v2/snapshot/locale/us/markets/stocks/tickers/{ticker} | YES | `SingleSnapshot` |
| Snapshots | GET /v2/snapshot/locale/us/markets/stocks/tickers | YES | `AllSnapshot` |
| Snapshots | GET /v3/snapshot | YES | `UnifiedSnapshot` |
| Snapshots | GET /v2/snapshot/locale/us/markets/stocks/gainers | NO | Missing — top gainers snapshot |
| Snapshots | GET /v2/snapshot/locale/us/markets/stocks/losers | NO | Missing — top losers snapshot |
| Technical Indicators | GET /v1/indicators/sma/{ticker} | YES | `SMA` |
| Technical Indicators | GET /v1/indicators/ema/{ticker} | YES | `EMA` |
| Technical Indicators | GET /v1/indicators/macd/{ticker} | YES | `MACD` |
| Technical Indicators | GET /v1/indicators/rsi/{ticker} | YES | `RSI` |
| Fundamentals | GET /vX/reference/dividends | PARTIAL | `Dividends` — path in code is wrong: `/stocks/v1/dividends` should be `/vX/reference/dividends` |
| Fundamentals | GET /vX/reference/splits | PARTIAL | `Splits` — path wrong: `/stocks/v1/splits` should be `/vX/reference/splits` |
| Fundamentals | GET /vX/reference/financials | PARTIAL | `FinancialRatios` — path wrong: `/stocks/financials/v1/ratios` should be `/vX/reference/financials` |
| Market Status | GET /v1/marketstatus/now | YES | `MarketStatus` |
| Market Status | GET /v1/marketstatus/upcoming | YES | `MarketHolidays` |
| News | GET /v2/reference/news | YES | `News` |
| Options | GET /v3/reference/options/contracts | NO | Missing — options reference data |
| Options | GET /v3/snapshot/options/{underlyingAsset} | NO | Missing — options chain snapshot |
| Options | GET /v3/snapshot/options/{underlyingAsset}/{optionContract} | NO | Missing — single option snapshot |
| Options | GET /v3/trades/{optionsTicker} | NO | Missing — options trades |
| Options | GET /v3/quotes/{optionsTicker} | NO | Missing — options quotes |
| Options | GET /v2/aggs/ticker/{optionsTicker}/range/... | NO | Missing — options aggregates |
| Options | GET /v2/last/trade/{optionsTicker} | NO | Missing — options last trade |
| Options | GET /v2/aggs/ticker/{optionsTicker}/prev | NO | Missing — options previous close |
| Indices | GET /v3/snapshot/indices | NO | Missing — indices snapshot |
| Indices | GET /v2/aggs/ticker/{indicesTicker}/range/... | NO | Missing — indices aggregates |
| Forex | GET /v2/aggs/ticker/{forexTicker}/range/... | NO | Missing — forex aggregates |
| Forex | GET /v2/last/nbbo/{forexTicker} | NO | Missing — forex last quote |
| Forex | GET /v1/conversion/{from}/{to} | NO | Missing — real-time currency conversion |
| Crypto | GET /v2/aggs/ticker/{cryptoTicker}/range/... | NO | Missing — crypto aggregates |
| Crypto | GET /v2/snapshot/locale/global/markets/crypto/tickers/{ticker} | NO | Missing — crypto snapshot |
| Crypto | GET /v1/open-close/crypto/{from}/{to}/{date} | NO | Missing — crypto daily open/close |
| WebSocket — Stocks | wss://socket.polygon.io/stocks | PARTIAL | URL stored but no stream enum variants |
| WebSocket — Options | wss://socket.polygon.io/options | NO | Missing — options WebSocket |
| WebSocket — Forex | wss://socket.polygon.io/forex | NO | Missing — forex WebSocket |
| WebSocket — Crypto | wss://socket.polygon.io/crypto | NO | Missing — crypto WebSocket |
| WebSocket — Indices | wss://socket.polygon.io/indices | NO | Missing — indices WebSocket |

**Summary of Polygon Gaps**: 25+ missing endpoints — 3 fundamental paths are wrong/outdated, full options section (8 endpoints), indices support (2), forex data (3), crypto (3), gainers/losers snapshots, reference conditions/exchanges/events, options contract reference, 4 WebSocket categories missing.

---

### 4. Tiingo (`stocks/us/tiingo/endpoints.rs`)

**API Docs**: https://www.tiingo.com/documentation/general/overview

**What we have**: DailyMeta, DailyPrices, IexMeta, IexPrices, CryptoMeta, CryptoTop, CryptoPrices, ForexTop, ForexPrices, FundamentalsDefinitions, FundamentalsDaily, FundamentalsStatements, News

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| EOD Stock Data | GET /tiingo/daily/{ticker} | YES | `DailyMeta` |
| EOD Stock Data | GET /tiingo/daily/{ticker}/prices | YES | `DailyPrices` |
| EOD Stock Data | GET /tiingo/daily | NO | Missing — list all supported tickers |
| IEX Intraday | GET /iex/{ticker} | YES | `IexMeta` |
| IEX Intraday | GET /iex/{ticker}/prices | YES | `IexPrices` |
| IEX Intraday | GET /iex | NO | Missing — top-of-book for all tickers |
| Crypto | GET /tiingo/crypto | YES | `CryptoMeta` |
| Crypto | GET /tiingo/crypto/top | YES | `CryptoTop` |
| Crypto | GET /tiingo/crypto/prices | YES | `CryptoPrices` |
| Forex | GET /tiingo/fx | NO | Missing — list all supported forex tickers |
| Forex | GET /tiingo/fx/{ticker} | NO | Missing — forex ticker metadata |
| Forex | GET /tiingo/fx/{ticker}/top | YES | `ForexTop` |
| Forex | GET /tiingo/fx/{ticker}/prices | YES | `ForexPrices` |
| Fundamentals | GET /tiingo/fundamentals/definitions | YES | `FundamentalsDefinitions` |
| Fundamentals | GET /tiingo/fundamentals/{ticker}/daily | YES | `FundamentalsDaily` |
| Fundamentals | GET /tiingo/fundamentals/{ticker}/statements | YES | `FundamentalsStatements` |
| News | GET /tiingo/news | YES | `News` |
| Power Data — Bulk | GET /tiingo/daily/{ticker}/prices (bulk) | NO | Missing — Power Data bulk EOD endpoint |
| WebSocket — IEX | wss://api.tiingo.com/iex | PARTIAL | URL stored but no stream enum variants |
| WebSocket — Forex | wss://api.tiingo.com/fx | PARTIAL | URL stored but no stream enum variants |
| WebSocket — Crypto | wss://api.tiingo.com/crypto | PARTIAL | URL stored but no stream enum variants |

**Summary of Tiingo Gaps**: 4 missing REST endpoints (list daily tickers, list IEX all, list forex tickers, forex metadata), 3 WebSocket URLs stored but no stream enum variants. Tiingo is mostly well-covered for its REST API. Power Data bulk endpoints require enterprise access and are lower priority.

---

### 5. TwelveData (`stocks/us/twelvedata/endpoints.rs`)

**API Docs**: https://twelvedata.com/docs

**What we have**: Price, Quote, TimeSeries, Eod, ExchangeRate, Stocks, ForexPairs, Cryptocurrencies, Etf, Commodities, Indices, SymbolSearch, EarliestTimestamp, Exchanges, MarketState, Rsi, Macd, BBands, Sma, Ema, Logo, Profile, Statistics

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Core Market Data | GET /price | YES | `Price` |
| Core Market Data | GET /quote | YES | `Quote` |
| Core Market Data | GET /time_series | YES | `TimeSeries` |
| Core Market Data | GET /eod | YES | `Eod` |
| Core Market Data | GET /exchange_rate | YES | `ExchangeRate` |
| Core Market Data | GET /real_time_price | NO | Missing — real-time price (vs /price which may be delayed) |
| Core Market Data | GET /complex_data | NO | Missing — batch complex requests combining multiple endpoints |
| Reference Data | GET /stocks | YES | `Stocks` |
| Reference Data | GET /forex_pairs | YES | `ForexPairs` |
| Reference Data | GET /cryptocurrencies | YES | `Cryptocurrencies` |
| Reference Data | GET /etf | YES | `Etf` |
| Reference Data | GET /commodities | YES | `Commodities` |
| Reference Data | GET /indices | YES | `Indices` |
| Reference Data | GET /funds | NO | Missing — mutual funds list |
| Reference Data | GET /bonds | NO | Missing — bonds list |
| Discovery | GET /symbol_search | YES | `SymbolSearch` |
| Discovery | GET /earliest_timestamp | YES | `EarliestTimestamp` |
| Markets | GET /exchanges | YES | `Exchanges` |
| Markets | GET /market_state | YES | `MarketState` |
| Technical Indicators | GET /rsi | YES | `Rsi` |
| Technical Indicators | GET /macd | YES | `Macd` |
| Technical Indicators | GET /bbands | YES | `BBands` |
| Technical Indicators | GET /sma | YES | `Sma` |
| Technical Indicators | GET /ema | YES | `Ema` |
| Technical Indicators | GET /adx | NO | Missing — Average Directional Index |
| Technical Indicators | GET /atr | NO | Missing — Average True Range |
| Technical Indicators | GET /stoch | NO | Missing — Stochastic Oscillator |
| Technical Indicators | GET /cci | NO | Missing — Commodity Channel Index |
| Technical Indicators | GET /williams_r | NO | Missing — Williams %R |
| Technical Indicators | GET /mom | NO | Missing — Momentum |
| Technical Indicators | GET /stddev | NO | Missing — Standard Deviation |
| Technical Indicators | GET /dema | NO | Missing — Double EMA (100+ indicators total in API) |
| Fundamentals | GET /logo | YES | `Logo` |
| Fundamentals | GET /profile | YES | `Profile` |
| Fundamentals | GET /statistics | YES | `Statistics` |
| Fundamentals | GET /dividends | NO | Missing — dividend calendar/history |
| Fundamentals | GET /splits | NO | Missing — stock splits history |
| Fundamentals | GET /earnings | NO | Missing — earnings data/calendar |
| Fundamentals | GET /earnings_calendar | NO | Missing — earnings calendar |
| Fundamentals | GET /ipo_calendar | NO | Missing — IPO calendar |
| Fundamentals | GET /income_statement | NO | Missing — income statement (Grow+ tier) |
| Fundamentals | GET /balance_sheet | NO | Missing — balance sheet (Grow+ tier) |
| Fundamentals | GET /cash_flow | NO | Missing — cash flow statement (Grow+ tier) |
| Fundamentals | GET /analyst_ratings | NO | Missing — analyst buy/sell ratings |
| Fundamentals | GET /insider_transactions | NO | Missing — insider trading data |
| Fundamentals | GET /options/expiration | NO | Missing — options expiration dates |
| Fundamentals | GET /options/chain | NO | Missing — options chain |
| WebSocket | wss://ws.twelvedata.com/v1/quotes/price | PARTIAL | URL stored; no stream enum variants |

**Summary of TwelveData Gaps**: 22 missing endpoints — fundamentals heavy (income statement, balance sheet, cash flow, earnings, dividends, splits, analyst ratings, insider transactions), options (2), mutual funds, bonds, 7+ additional technical indicators (ADX, ATR, STOCH, CCI, Williams%R, etc.), complex_data batch endpoint. TwelveData has 100+ technical indicators; we only model 5.

---

## Batch 10 — Stocks India

---

### 6. Angel One (`stocks/india/angel_one/endpoints.rs`)

**API Docs**: https://smartapi.angelbroking.com/docs

**What we have**: Login, TokenRefresh, GetProfile, Logout, GetFeedToken, Quote, HistoricalCandles, SearchScrip, PlaceOrder, PlaceOrderFullResponse, ModifyOrder, CancelOrder, GetOrderBook, GetOrderDetails, GetTradeBook, CreateGTT, ModifyGTT, CancelGTT, GetGTTDetails, ListGTT, GetHoldings, GetPositions, ConvertPosition, GetRMS, MarginCalculator

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Session — Login | POST /loginByPassword | YES | `Login` |
| Session — Token Refresh | POST /generateTokens | YES | `TokenRefresh` |
| Session — Profile | GET /getProfile | YES | `GetProfile` |
| Session — Logout | POST /logout | YES | `Logout` |
| Session — Feed Token | GET /getfeedToken | YES | `GetFeedToken` |
| Session — TOTP | POST /loginByTotp | NO | Missing — TOTP-based login (2FA) |
| Market Data — Quote | POST /quote/ | YES | `Quote` |
| Market Data — Historical | POST /getCandleData | YES | `HistoricalCandles` |
| Market Data — Search | POST /searchScrip | YES | `SearchScrip` |
| Market Data — Instrument List | GET (CSV download) | NO | Missing — downloadable instrument/scrip master CSV |
| Market Data — Market Depth | GET /marketDepth | NO | Missing — L2 order book / market depth |
| Trading — Place Order | POST /placeOrder | YES | `PlaceOrder` |
| Trading — Place Full Response | POST /placeOrderFullResponse | YES | `PlaceOrderFullResponse` |
| Trading — Modify Order | POST /modifyOrder | YES | `ModifyOrder` |
| Trading — Cancel Order | POST /cancelOrder | YES | `CancelOrder` |
| Trading — Order Book | GET /getOrderBook | YES | `GetOrderBook` |
| Trading — Order Details | GET /details/ | YES | `GetOrderDetails` |
| Trading — Trade Book | GET /getTradeBook | YES | `GetTradeBook` |
| GTT — Create | POST /createRule | YES | `CreateGTT` |
| GTT — Modify | POST /modifyRule | YES | `ModifyGTT` |
| GTT — Cancel | POST /cancelRule | YES | `CancelGTT` |
| GTT — Details | POST /ruleDetails | YES | `GetGTTDetails` |
| GTT — List | POST /ruleList | YES | `ListGTT` |
| Portfolio — Holdings | GET /getHolding | YES | `GetHoldings` |
| Portfolio — Positions | GET /getPosition | YES | `GetPositions` |
| Portfolio — Convert Position | POST /convertPosition | YES | `ConvertPosition` |
| Account — RMS | GET /getRMS | YES | `GetRMS` |
| Account — Brokerage Calculator | POST /v1/brokerage | NO | Missing — brokerage charge calculator |
| Margin | POST /margin/v1/batch | YES | `MarginCalculator` |
| WebSocket V2 | wss://smartapisocket.angelone.in/smart-stream | PARTIAL | URL stored; no stream message enum |

**Summary of Angel One Gaps**: 4 missing endpoints — TOTP login, instrument master CSV download, market depth (L2), brokerage calculator. Angel One is notably well-covered. Option chain data is not available via SmartAPI (confirmed limitation).

---

### 7. Dhan (`stocks/india/dhan/endpoints.rs`)

**API Docs**: https://dhanhq.co/docs/v2/

**What we have**: GenerateToken, RenewToken, LTP, OHLC, Quote, HistoricalDaily, HistoricalIntraday, OptionChain, InstrumentList, PlaceOrder, ModifyOrder, CancelOrder, GetOrderBook, GetOrder, PlaceSlicedOrder, PlaceSuperOrder, ModifySuperOrder, CancelSuperOrder, GetSuperOrders, GetSuperOrder, PlaceForeverOrder, ModifyForeverOrder, CancelForeverOrder, GetForeverOrders, GetTradesByOrder, GetTradeHistory, GetHoldings, GetPositions, ConvertPosition, GetFunds, GetLedger, GenerateTPIN, GetEDISForm, CheckEDISStatus

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Authentication | POST /v2/access_token | YES | `GenerateToken` |
| Authentication | POST /v2/access_token/renew | YES | `RenewToken` |
| Market Data — LTP | POST /v2/marketfeed/ltp | YES | `LTP` |
| Market Data — OHLC | POST /v2/marketfeed/ohlc | YES | `OHLC` |
| Market Data — Quote | POST /v2/marketfeed/quote | YES | `Quote` |
| Market Data — Historical Daily | POST /v2/charts/historical | YES | `HistoricalDaily` |
| Market Data — Historical Intraday | POST /v2/charts/intraday | YES | `HistoricalIntraday` |
| Market Data — Option Chain | POST /v2/optionchain | YES | `OptionChain` |
| Market Data — Expiry List | GET /v2/optionchain/expirylist | NO | Missing — list of expiry dates for a symbol |
| Market Data — Instruments | GET /v2/instrument/{exchangeSegment} | YES | `InstrumentList` |
| Market Data — Market Depth 20 | WebSocket only | PARTIAL | URL stored for ws_depth_20, no enum |
| Market Data — Market Depth 200 | WebSocket only | PARTIAL | URL stored for ws_depth_200, no enum |
| Trading — Orders | POST /v2/orders | YES | `PlaceOrder` |
| Trading — Orders | PUT /v2/orders/{orderId} | YES | `ModifyOrder` |
| Trading — Orders | DELETE /v2/orders/{orderId} | YES | `CancelOrder` |
| Trading — Orders | GET /v2/orders | YES | `GetOrderBook` |
| Trading — Orders | GET /v2/orders/{orderId} | YES | `GetOrder` |
| Trading — Orders | POST /v2/orders/slicing | YES | `PlaceSlicedOrder` |
| Trading — Super Orders | POST /v2/super/orders | YES | `PlaceSuperOrder` |
| Trading — Super Orders | PUT /v2/super/orders/{orderId} | YES | `ModifySuperOrder` |
| Trading — Super Orders | DELETE /v2/super/orders/{orderId}/{orderLeg} | YES | `CancelSuperOrder` |
| Trading — Super Orders | GET /v2/super/orders | YES | `GetSuperOrders` |
| Trading — Super Orders | GET /v2/super/orders/{orderId} | YES | `GetSuperOrder` |
| Trading — Forever Orders | POST /v2/forever/orders | YES | `PlaceForeverOrder` |
| Trading — Forever Orders | PUT /v2/forever/orders/{orderId} | YES | `ModifyForeverOrder` |
| Trading — Forever Orders | DELETE /v2/forever/orders/{orderId} | YES | `CancelForeverOrder` |
| Trading — Forever Orders | GET /v2/forever/orders | YES | `GetForeverOrders` |
| Trading — Trade History | GET /v2/trades/{orderId} | YES | `GetTradesByOrder` |
| Trading — Trade History | GET /v2/trades/{fromDate}/{toDate}/{page} | YES | `GetTradeHistory` |
| Portfolio — Holdings | GET /v2/holdings | YES | `GetHoldings` |
| Portfolio — Positions | GET /v2/positions | YES | `GetPositions` |
| Portfolio — Convert Position | POST /v2/positions/convert | YES | `ConvertPosition` |
| Funds | GET /v2/funds | YES | `GetFunds` |
| Funds | GET /v2/ledger | YES | `GetLedger` |
| Funds — Margin Calculator | POST /v2/margincalculator | NO | Missing — margin calculator for single order |
| Funds — Multi Margin Calculator | POST /v2/margincalculator/multi | NO | Missing — batch margin calculation |
| EDIS | POST /v2/edis/tpin | YES | `GenerateTPIN` |
| EDIS | POST /v2/edis/form | YES | `GetEDISForm` |
| EDIS | POST /v2/edis/inquiry | YES | `CheckEDISStatus` |
| Statements | GET /v2/statements/{fromDate}/{toDate} | NO | Missing — account statements/P&L |
| Kill Switch | POST /v2/killswitch | NO | Missing — kill switch to stop all trading |
| WebSocket — Live Feed | wss://api-feed.dhan.co | PARTIAL | URL stored; no stream message enum |
| WebSocket — Depth 20 | wss://depth-api-feed.dhan.co/twentydepth | PARTIAL | URL stored; no enum |
| WebSocket — Depth 200 | wss://full-depth-api.dhan.co/twohundreddepth | PARTIAL | URL stored; no enum |

**Summary of Dhan Gaps**: 5 missing REST endpoints — option chain expiry list, margin calculator (2 variants), account statements, kill switch. Dhan is very well-covered overall. Multiple WebSocket URLs present but no stream enum defined.

---

### 8. Fyers (`stocks/india/fyers/endpoints.rs`)

**API Docs**: https://myapi.fyers.in/docsv3

**What we have**: GenerateAuthCode, ValidateAuthCode, GenerateToken, Profile, Funds, Holdings, Quotes, Depth, History, MarketStatus, SymbolMaster, PlaceOrder, PlaceOrderMulti, ModifyOrder, CancelOrder, GetOrders, GetOrderById, Positions, ConvertPosition, Tradebook, GenerateTpin, EdisTransactions, SubmitHoldings, InquireTransaction

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Authentication — Auth Code | POST /api/v3/generate-authcode | YES | `GenerateAuthCode` |
| Authentication — Validate | POST /api/v3/validate-authcode | YES | `ValidateAuthCode` |
| Authentication — Token | POST /api/v3/token | YES | `GenerateToken` |
| Authentication — Revoke Token | DELETE /api/v3/token | NO | Missing — logout / revoke token |
| User Profile | GET /api/v3/profile | YES | `Profile` |
| User Profile — Logout | DELETE /api/v3/logout | NO | Missing — logout endpoint |
| Account — Funds | GET /api/v3/funds | YES | `Funds` |
| Account — Holdings | GET /api/v3/holdings | YES | `Holdings` |
| Account — Overall Positions | GET /api/v3/positions | YES | `Positions` (only positions; no separate overall positions endpoint) |
| Market Data — Quotes | GET /data/quotes | YES | `Quotes` |
| Market Data — Depth | GET /data/depth/ | YES | `Depth` |
| Market Data — History | GET /data/history | YES | `History` |
| Market Data — Market Status | GET /data/market-status | YES | `MarketStatus` |
| Market Data — Symbol Master | GET /data/symbol-master | YES | `SymbolMaster` |
| Market Data — Option Chain | GET /data/optionchain | NO | Missing — option chain for F&O instruments |
| Market Data — Market Expiry | GET /data/expiry | NO | Missing — option expiry dates list |
| Trading — Place Order | POST /api/v3/orders | YES | `PlaceOrder` |
| Trading — Place Multi Order | POST /api/v3/orders/multi | YES | `PlaceOrderMulti` |
| Trading — Modify Order | PUT /api/v3/orders | YES | `ModifyOrder` |
| Trading — Cancel Order | DELETE /api/v3/orders | YES | `CancelOrder` |
| Trading — Get Orders | GET /api/v3/orders | YES | `GetOrders` |
| Trading — Get Order By ID | GET /api/v3/orders/{id} | YES | `GetOrderById` |
| Trading — Cancel Multi Order | DELETE /api/v3/orders/multi | NO | Missing — cancel multiple orders at once |
| Trading — GTT Place | POST /api/v3/gtt | NO | Missing — GTT order placement |
| Trading — GTT Modify | PUT /api/v3/gtt | NO | Missing — GTT order modification |
| Trading — GTT Cancel | DELETE /api/v3/gtt | NO | Missing — GTT order cancellation |
| Trading — GTT List | GET /api/v3/gtt | NO | Missing — list GTT orders |
| Positions & Trades | GET /api/v3/positions | YES | `Positions` |
| Positions & Trades | PUT /api/v3/positions | YES | `ConvertPosition` |
| Positions & Trades | GET /api/v3/tradebook | YES | `Tradebook` |
| EDIS | POST /api/v3/edis/generate-tpin | YES | `GenerateTpin` |
| EDIS | GET /api/v3/edis/transactions | YES | `EdisTransactions` |
| EDIS | POST /api/v3/edis/submit-holdings | YES | `SubmitHoldings` |
| EDIS | GET /api/v3/edis/inquire-transaction | YES | `InquireTransaction` |
| Margin | POST /api/v3/margin | NO | Missing — basket margin calculator |
| WebSocket — Data | wss://api-t1.fyers.in/socket/v3/dataSock | PARTIAL | URL stored; no stream enum |
| WebSocket — Order | wss://api-t1.fyers.in/socket/v3/orderSock | PARTIAL | URL stored; no stream enum |
| WebSocket — TBT | wss://rtsocket-api.fyers.in/versova | PARTIAL | URL stored; no stream enum |

**Summary of Fyers Gaps**: 9 missing endpoints — GTT order CRUD (4 endpoints), option chain, expiry list, cancel multi order, margin calculator, revoke token/logout. Three WebSocket URLs present but no stream enum defined.

---

### 9. Upstox (`stocks/india/upstox/endpoints.rs`)

**API Docs**: https://upstox.com/developer/api-documentation/

**What we have**: LoginDialog, LoginToken, MarketQuoteLtp, MarketQuoteQuotes, MarketQuoteOhlc, HistoricalCandleV2, HistoricalCandleV3, IntradayCandleV2, IntradayCandleV3, OptionChain, OptionContract, OrderPlaceV2, OrderPlaceV3, OrderModify, OrderCancel, OrderDetails, OrderBook, OrderTrades, TradeHistory, MultiOrderPlace, MultiOrderCancel, GttPlace, GttModify, GttCancel, GttOrders, GttOrderDetails, PositionsShortTerm, HoldingsLongTerm, MtfPositions, ConvertPosition, ExitAllPositions, FundsAndMargin, MarginRequirement, TradeCharges, TradePnl, Brokerage, UserProfile, WsMarketDataAuthorize, WsPortfolioAuthorize

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Authentication | GET /login/authorization/dialog | YES | `LoginDialog` |
| Authentication | POST /login/authorization/token | YES | `LoginToken` |
| Authentication | DELETE /login/authorization/logout | NO | Missing — logout / revoke token |
| Market Data — Quotes | GET /market-quote/ltp | YES | `MarketQuoteLtp` |
| Market Data — Quotes | GET /market-quote/quotes | YES | `MarketQuoteQuotes` |
| Market Data — Quotes | GET /market-quote/ohlc | YES | `MarketQuoteOhlc` |
| Market Data — Historical | GET /historical-candle (v2) | YES | `HistoricalCandleV2` |
| Market Data — Historical | GET /historical-candle (v3) | YES | `HistoricalCandleV3` |
| Market Data — Intraday | GET /historical-candle/intraday (v2) | YES | `IntradayCandleV2` |
| Market Data — Intraday | GET /historical-candle/intraday (v3) | YES | `IntradayCandleV3` |
| Market Data — Options | GET /option/chain | YES | `OptionChain` |
| Market Data — Options | GET /option/contract | YES | `OptionContract` |
| Market Data — Instruments | GET /instruments | NO | Missing — complete instruments master list (JSON) |
| Market Data — Market Holidays | GET /market/holidays | NO | Missing — market holiday calendar |
| Market Data — Market Timing | GET /market/timings | NO | Missing — market segment timings |
| Market Data — Depth | GET /market-quote/depth | NO | Missing — market depth / order book |
| Trading — Orders | POST /order/place (v2/v3) | YES | `OrderPlaceV2`, `OrderPlaceV3` |
| Trading — Orders | PUT /order/modify | YES | `OrderModify` |
| Trading — Orders | DELETE /order/cancel | YES | `OrderCancel` |
| Trading — Orders | GET /order/details | YES | `OrderDetails` / `OrderBook` |
| Trading — Orders | GET /order/trades | YES | `OrderTrades` |
| Trading — Orders | GET /order/history | YES | `TradeHistory` |
| Trading — Multi Orders | POST /order/multi/place | YES | `MultiOrderPlace` |
| Trading — Multi Orders | DELETE /order/multi/cancel | YES | `MultiOrderCancel` |
| Trading — GTT | POST /order/gtt/place | YES | `GttPlace` |
| Trading — GTT | PUT /order/gtt/modify | YES | `GttModify` |
| Trading — GTT | DELETE /order/gtt/cancel | YES | `GttCancel` |
| Trading — GTT | GET /gtt/orders | YES | `GttOrders` |
| Trading — GTT | GET /gtt/order | YES | `GttOrderDetails` |
| Portfolio | GET /portfolio/short-term-positions | YES | `PositionsShortTerm` |
| Portfolio | GET /portfolio/long-term-holdings | YES | `HoldingsLongTerm` |
| Portfolio | GET /portfolio/mtf-positions | YES | `MtfPositions` |
| Portfolio | PUT /portfolio/convert-position | YES | `ConvertPosition` |
| Portfolio | DELETE /portfolio/positions | YES | `ExitAllPositions` |
| Account | GET /user/get-funds-and-margin | YES | `FundsAndMargin` |
| Account | GET /user/profile | YES | `UserProfile` |
| Account | GET /user/segment-funds | NO | Missing — segment-wise fund details (equity vs commodity) |
| Account | GET /user/bank-account | NO | Missing — linked bank account details |
| Charges | GET /charges/margin | YES | `MarginRequirement` |
| Charges | GET /charges/brokerage | YES | `Brokerage` |
| Charges | GET /trade/profit-loss/charges | YES | `TradeCharges` |
| Charges | GET /trade/profit-loss/data | YES | `TradePnl` |
| Charges | GET /trade/profit-loss/metadata | NO | Missing — P&L metadata endpoint |
| EDIS | POST /edis/tpin | NO | Missing — generate TPIN |
| EDIS | GET /edis/holdings-form | NO | Missing — EDIS holdings form |
| EDIS | GET /edis/inquiry | NO | Missing — EDIS status inquiry |
| WebSocket — Market Data | wss://api.upstox.com/v2/feed/market-data-feed/protobuf | PARTIAL | URL stored; no stream message enum |
| WebSocket — Portfolio | wss://api.upstox.com/v2/feed/portfolio-stream-feed | PARTIAL | URL stored; no stream message enum |
| WebSocket — Market Data V3 | wss://api.upstox.com/v3/feed/market-data-feed/protobuf | NO | Missing — V3 market data WebSocket URL |

**Summary of Upstox Gaps**: 9 missing REST endpoints — logout, instruments list, market holidays, market timing, market depth, segment funds, bank account, P&L metadata, EDIS (3 endpoints), V3 WebSocket. Upstox is well-covered for trading and GTT. Note: EDIS endpoints were completely missed despite being in the API.

---

### 10. Zerodha (`stocks/india/zerodha/endpoints.rs`)

**API Docs**: https://kite.trade/docs/connect/v3/

**What we have**: SessionToken, SessionLogout, UserProfile, Instruments, InstrumentsExchange, Quote, QuoteOhlc, QuoteLtp, HistoricalCandles, PlaceOrder, ModifyOrder, CancelOrder, GetOrders, GetOrder, GetTrades, GetOrderTrades, PlaceGtt, ModifyGtt, DeleteGtt, GetGtts, GetGtt, GetMargins, GetMarginsSegment, OrderMargins, BasketMargins, Holdings, HoldingsAuctions, AuthorizeHoldings, Positions, ConvertPosition

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Authentication | POST /session/token | YES | `SessionToken` |
| Authentication | DELETE /session/token | YES | `SessionLogout` |
| User | GET /user/profile | YES | `UserProfile` |
| Instruments | GET /instruments | YES | `Instruments` |
| Instruments | GET /instruments/{exchange} | YES | `InstrumentsExchange` |
| Market Data — Quote | GET /quote | YES | `Quote` |
| Market Data — Quote | GET /quote/ohlc | YES | `QuoteOhlc` |
| Market Data — Quote | GET /quote/ltp | YES | `QuoteLtp` |
| Market Data — Historical | GET /instruments/historical/{token}/{interval} | YES | `HistoricalCandles` |
| Trading — Orders | POST /orders/{variety} | YES | `PlaceOrder` |
| Trading — Orders | PUT /orders/{variety}/{order_id} | YES | `ModifyOrder` |
| Trading — Orders | DELETE /orders/{variety}/{order_id} | YES | `CancelOrder` |
| Trading — Orders | GET /orders | YES | `GetOrders` |
| Trading — Orders | GET /orders/{order_id} | YES | `GetOrder` |
| Trading — Orders | GET /trades | YES | `GetTrades` |
| Trading — Orders | GET /orders/{order_id}/trades | YES | `GetOrderTrades` |
| GTT | POST /gtt/triggers | YES | `PlaceGtt` |
| GTT | PUT /gtt/triggers/{trigger_id} | YES | `ModifyGtt` |
| GTT | DELETE /gtt/triggers/{trigger_id} | YES | `DeleteGtt` |
| GTT | GET /gtt/triggers | YES | `GetGtts` |
| GTT | GET /gtt/triggers/{trigger_id} | YES | `GetGtt` |
| Margins | GET /user/margins | YES | `GetMargins` |
| Margins | GET /user/margins/{segment} | YES | `GetMarginsSegment` |
| Margins | POST /margins/orders | YES | `OrderMargins` |
| Margins | POST /margins/basket | YES | `BasketMargins` |
| Portfolio — Holdings | GET /portfolio/holdings | YES | `Holdings` |
| Portfolio — Holdings | GET /portfolio/holdings/auctions | YES | `HoldingsAuctions` |
| Portfolio — Holdings | POST /portfolio/holdings/authorise | YES | `AuthorizeHoldings` |
| Portfolio — Positions | GET /portfolio/positions | YES | `Positions` |
| Portfolio — Positions | PUT /portfolio/positions | YES | `ConvertPosition` |
| Mutual Funds — Instruments | GET /mf/instruments | NO | Missing — list of MF instruments (Coin) |
| Mutual Funds — Orders | GET /mf/orders | NO | Missing — list all MF orders |
| Mutual Funds — Orders | POST /mf/orders | NO | Missing — place MF order |
| Mutual Funds — Orders | DELETE /mf/orders/{order_id} | NO | Missing — cancel MF order |
| Mutual Funds — Orders | GET /mf/orders/{order_id} | NO | Missing — get specific MF order |
| Mutual Funds — SIP | GET /mf/sips | NO | Missing — list all SIPs |
| Mutual Funds — SIP | POST /mf/sips | NO | Missing — create SIP |
| Mutual Funds — SIP | PUT /mf/sips/{sip_id} | NO | Missing — modify SIP |
| Mutual Funds — SIP | DELETE /mf/sips/{sip_id} | NO | Missing — cancel SIP |
| Mutual Funds — SIP | GET /mf/sips/{sip_id} | NO | Missing — get specific SIP |
| Mutual Funds — Holdings | GET /mf/holdings | NO | Missing — MF holdings |
| Mutual Funds — Allotments | GET /mf/allotments | NO | Missing — SIP installment allotments |
| WebSocket — Ticker | wss://ws.kite.trade | PARTIAL | URL stored (as `_ws_base`); no stream message enum |

**Summary of Zerodha Gaps**: 12 missing endpoints — the entire Mutual Fund / Coin API section (instruments, orders CRUD, SIP CRUD, holdings, allotments). The MF API is a significant and documented part of Kite Connect v3 that is fully absent. WebSocket URL is stored but prefixed with `_` (unused) and has no stream enum.

---

## Cross-Connector Summary

| Connector | Missing Endpoints | Top Priority Gaps |
|-----------|------------------|-------------------|
| Alpaca | 17 | Watchlists (6), close positions (2), account config, options chain/contract |
| Finnhub | 19 | ETF family (4), mutual fund family (4), bond family (3), IPO, earnings surprise, transcripts |
| Polygon | 25+ | Wrong fundamental paths (3), full options section (8), indices, forex, crypto, WebSockets (4) |
| Tiingo | 4 | List tickers endpoints (3), WebSocket enums |
| TwelveData | 22 | Fundamentals heavy (10+), options (2), 15+ more technical indicators, batch endpoint |
| Angel One | 4 | TOTP login, instrument CSV, market depth, brokerage calculator |
| Dhan | 5 | Expiry list, margin calculators (2), statements, kill switch |
| Fyers | 9 | GTT CRUD (4), option chain, expiry, cancel multi, margin, logout |
| Upstox | 9 | Logout, instruments, market holidays, market depth, EDIS (3), V3 WebSocket |
| Zerodha | 12 | Entire MF/Coin API missing (11 endpoints), WebSocket enum |

### Critical Fixes (Wrong Paths)

| Connector | Variant | Current (Wrong) Path | Correct Path |
|-----------|---------|---------------------|--------------|
| Polygon | `Dividends` | `/stocks/v1/dividends` | `/vX/reference/dividends` |
| Polygon | `Splits` | `/stocks/v1/splits` | `/vX/reference/splits` |
| Polygon | `FinancialRatios` | `/stocks/financials/v1/ratios` | `/vX/reference/financials` |

### Most Impactful Missing Sections

1. **Zerodha Mutual Funds** — 12 endpoints, entire API section absent
2. **Polygon Options** — 8+ endpoints, full options vertical missing
3. **Finnhub ETF + MF + Bonds** — 11 endpoints across 3 asset classes
4. **TwelveData Fundamentals** — 10 endpoints (financial statements, earnings, dividends, analyst ratings)
5. **Alpaca Watchlists** — 6 CRUD endpoints absent
6. **Fyers GTT** — 4 CRUD endpoints absent

---

## Sources

- [Alpaca Trading API Reference](https://docs.alpaca.markets/reference/)
- [Alpaca Screener API — Most Actives](https://docs.alpaca.markets/reference/mostactives-1)
- [Alpaca Watchlist Documentation](https://alpaca.markets/deprecated/docs/api-documentation/api-v2/watchlist/)
- [Alpaca MCP Server (full endpoint scope)](https://github.com/alpacahq/alpaca-mcp-server)
- [Finnhub API Documentation](https://finnhub.io/docs/api)
- [Finnhub Mutual Fund Holdings](https://finnhub.io/docs/api/mutual-fund-holdings)
- [Finnhub IPO Calendar](https://finnhub.io/docs/api/ipo-calendar)
- [Finnhub Bond Price](https://finnhub.io/docs/api/bond-price)
- [Polygon.io API Docs](https://polygon.io/docs/stocks)
- [Polygon Options Snapshot](https://massive.com/docs/rest/options/snapshots/option-chain-snapshot)
- [Polygon Options Contracts Reference](https://massive.com/docs/rest/options/contracts/all-contracts)
- [Tiingo Documentation](https://www.tiingo.com/documentation/general/overview)
- [Tiingo IEX Documentation](https://www.tiingo.com/documentation/iex)
- [Tiingo Fundamentals Documentation](https://www.tiingo.com/documentation/fundamentals)
- [Twelvedata API Documentation](https://twelvedata.com/docs)
- [Twelvedata Fundamentals](https://twelvedata.com/fundamentals)
- [Angel One SmartAPI Docs](https://smartapi.angelbroking.com/docs)
- [Angel One SmartAPI Python SDK](https://github.com/angel-one/smartapi-python)
- [Dhan HQ API v2 Documentation](https://dhanhq.co/docs/v2/)
- [Dhan Funds & Margin](https://dhanhq.co/docs/v2/funds/)
- [Dhan Option Chain](https://dhanhq.co/docs/v2/option-chain/)
- [Fyers API v3 Documentation](https://myapi.fyers.in/docsv3)
- [Fyers API v3 PyPI package](https://pypi.org/project/fyers-apiv3/)
- [Upstox API Documentation](https://upstox.com/developer/api-documentation/)
- [Upstox Market Holidays](https://upstox.com/developer/api-documentation/get-market-holidays/)
- [Upstox Brokerage Details](https://upstox.com/developer/api-documentation/get-brokerage/)
- [Upstox Instruments](https://upstox.com/developer/api-documentation/instruments/)
- [Zerodha Kite Connect v3 Documentation](https://kite.trade/docs/connect/v3/)
- [Zerodha Mutual Funds API](https://kite.trade/docs/connect/v3/mutual-funds/)
- [pykiteconnect GitHub](https://github.com/zerodha/pykiteconnect)
