# Wave 4 — Endpoint Gap Analysis: Batch 13 (Aggregators)

Generated: 2026-03-13

Base path: `digdigdig3/src/`

Sources consulted:
- CryptoCompare/CoinDesk: https://developers.coindesk.com/documentation (formerly developers.cryptocompare.com → 301 redirect)
- DefiLlama: https://api-docs.defillama.com/llms.txt (full machine-readable endpoint list)
- IB Client Portal: https://interactivebrokers.github.io/cpwebapi/ + https://raw.githubusercontent.com/rcontesti/IB_MCP/main/ENDPOINTS.md (79 endpoints catalogued)
- Yahoo Finance: https://github.com/gadicc/yahoo-finance2 + community documentation

---

## 1. CryptoCompare (`aggregators/cryptocompare/`)

### What We Have
The connector covers the **legacy min-api** (https://min-api.cryptocompare.com):
- Price: `Price`, `PriceMulti`, `PriceMultiFull`, `DayAvg`, `PriceHistorical`
- Historical OHLCV: `HistoDay/Hour/Minute` (v1 + v2)
- Top Lists: `TopExchanges`, `TopExchangesFull`, `TopPairs`, `TopVolumes`, `TopMktCapFull`, `TopTotalVolFull`
- Metadata: `CoinList`, `ExchangeList`, `BlockchainList`, `BlockchainHistoDay`, `BlockchainLatest`
- News/Social: `News`, `NewsFeeds`, `NewsCategories`, `SocialLatest`, `SocialHistoDay`, `SocialHistoHour`
- Rate limits: `RateLimit`, `RateLimitHour`

### Gap Table — Legacy Min-API (https://min-api.cryptocompare.com)

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Price | `GET /data/price` | YES | Single symbol |
| Price | `GET /data/pricemulti` | YES | Multi-symbol |
| Price | `GET /data/pricemultifull` | YES | Full OHLCV data |
| Price | `GET /data/dayAvg` | YES | Daily average |
| Price | `GET /data/pricehistorical` | YES | Historical timestamp |
| Historical | `GET /data/histoday` | YES | Daily bars |
| Historical | `GET /data/histohour` | YES | Hourly bars |
| Historical | `GET /data/histominute` | YES | Minute bars |
| Historical | `GET /data/v2/histoday` | YES | Daily bars v2 |
| Historical | `GET /data/v2/histohour` | YES | Hourly bars v2 |
| Historical | `GET /data/v2/histominute` | YES | Minute bars v2 |
| **Orderbook** | `GET /data/ob/l1/top` | **NO** | Top of orderbook (L1) for one market |
| **Orderbook** | `GET /data/ob/l2/snapshot` | **NO** | Full L2 orderbook snapshot |
| **Trade Book** | `GET /data/v4/all/exchanges/trades` | **NO** | Latest trades per exchange |
| **Trade Book** | `GET /data/subs/top` | **NO** | Subscription tickers (streaming setup) |
| Top Lists | `GET /data/top/exchanges` | YES | |
| Top Lists | `GET /data/top/exchanges/full` | YES | |
| Top Lists | `GET /data/top/pairs` | YES | |
| Top Lists | `GET /data/top/volumes` | YES | |
| Top Lists | `GET /data/top/mktcapfull` | YES | |
| Top Lists | `GET /data/top/totalvolfull` | YES | |
| **Top Lists** | `GET /data/top/list` | **NO** | General top coins list |
| **Volume** | `GET /data/exchange/histoday` | **NO** | Historical exchange volume by day |
| **Volume** | `GET /data/exchange/histohour` | **NO** | Historical exchange volume by hour |
| Metadata | `GET /data/all/coinlist` | YES | |
| Metadata | `GET /data/all/exchanges` | YES | |
| **Metadata** | `GET /data/all/coinlist?summary=true` | **NO** | Lightweight coin summary list |
| **Metadata** | `GET /data/v4/all/exchanges` | **NO** | Exchanges v4 with richer metadata |
| Blockchain | `GET /data/blockchain/list` | YES | |
| Blockchain | `GET /data/blockchain/histo/day` | YES | |
| Blockchain | `GET /data/blockchain/latest` | YES | |
| **Mining** | `GET /data/mining/contracts/general` | **NO** | Cloud mining contracts |
| **Mining** | `GET /data/mining/equipment/general` | **NO** | Mining hardware specs |
| **Mining** | `GET /data/mining/pools/general` | **NO** | Mining pool list |
| **Mining** | `GET /data/mining/calculator/single` | **NO** | Profitability calculator |
| **Mining** | `GET /data/mining/difficulty/histo/day` | **NO** | Difficulty history |
| News | `GET /data/v2/news/` | YES | |
| News | `GET /data/news/feeds` | YES | |
| News | `GET /data/news/categories` | YES | |
| **News** | `GET /data/v2/news/?feedName=...` | **NO** | Filter news by feed |
| **News** | `GET /data/news/feeds/andcategories` | **NO** | Combined feed+category filter |
| Social | `GET /data/social/coin/latest` | YES | |
| Social | `GET /data/social/coin/histo/day` | YES | |
| Social | `GET /data/social/coin/histo/hour` | YES | |
| **Portfolio** | `GET /data/portfolio/coin/overview` | **NO** | Portfolio overview (legacy endpoint) |
| Rate Limits | `GET /stats/rate/limit` | YES | |
| Rate Limits | `GET /stats/rate/hour/limit` | YES | |

### Gap Table — New Data API (https://data-api.cryptocompare.com / https://data-api.coindesk.com)

CryptoCompare was acquired by CoinDesk and now also exposes a modern versioned API. Our connector only covers the legacy min-api and is entirely missing the new data API.

| Category | Endpoint Pattern | We Have? | Notes |
|----------|-----------------|----------|-------|
| **Spot Markets** | `GET /data/spot/v1/markets` | **NO** | Markets list |
| **Spot Markets** | `GET /data/spot/v1/markets/instruments` | **NO** | Trading pairs per market |
| **Spot Markets** | `GET /data/spot/v1/latest/tick` | **NO** | Real-time tick per instrument |
| **Spot Markets** | `GET /data/spot/v1/latest/tick/asset` | **NO** | Latest tick grouped by asset |
| **Spot Historical** | `GET /data/spot/v1/historical/hours` | **NO** | Hourly OHLCV (new API) |
| **Spot Historical** | `GET /data/spot/v1/historical/days` | **NO** | Daily OHLCV (new API) |
| **Spot Historical** | `GET /data/spot/v1/historical/minutes` | **NO** | Minute OHLCV (new API) |
| **Spot Orderbook** | `GET /data/spot/v1/orderbook/l2/metrics/minute` | **NO** | L2 orderbook metrics |
| **Spot Orderbook** | `GET /data/spot/v1/orderbook/l2/metrics/hour` | **NO** | Hourly orderbook metrics |
| **Indices** | `GET /data/index/cc/v1/markets` | **NO** | Index markets (CCCAGG, MVIS) |
| **Indices** | `GET /data/index/cc/v1/markets/instruments` | **NO** | Index instruments |
| **Indices** | `GET /data/index/cc/v1/latest/instrument/metadata` | **NO** | Index instrument metadata |
| **Indices** | `GET /data/index/v1/markets/instruments` | **NO** | Third-party index instruments |
| **Asset Data** | `GET /data/asset/v1/metadata` | **NO** | Full asset metadata (new API) |
| **Asset Data** | `GET /data/asset/v1/search` | **NO** | Asset search |
| **News** | `GET /data/news/v1/source/list` | **NO** | News sources (new API) |
| **News** | `GET /data/news/v1/article/list` | **NO** | News articles (new API) |
| **On-Chain DEX** | `GET /data/onchain/v1/amm/latest/instrument/metadata` | **NO** | DEX AMM instrument metadata |

### WebSocket Streams — Legacy (wss://streamer.cryptocompare.com/v2)

| Stream | Subscription Key | We Have? | Notes |
|--------|-----------------|----------|-------|
| **Real-time Tick** | `5~{exchange}~{base}~{quote}` | **NO** | Trade tick stream |
| **Current OHLCV** | `11~{exchange}~{base}~{quote}` | **NO** | Current OHLCV candle updates |
| **Full Orderbook L2** | `8~{exchange}~{base}~{quote}` | **NO** | Full order book depth |
| **Top of Book L1** | `30~{exchange}~{base}~{quote}` | **NO** | Best bid/ask stream |
| **Aggregate Index** | `2~CCCAGG~{base}~{quote}` | **NO** | CCCAGG aggregate price |
| **News** | `NEWS~{source}` | **NO** | Live news stream |

### WebSocket Streams — New Data Streamer (wss://data-streamer.coindesk.com)

| Stream | Topic | We Have? | Notes |
|--------|-------|----------|-------|
| **Order Book Realtime L1** | `spot/v1/orderbook/realtime/l1/top-of-book` | **NO** | Best bid/ask per exchange |

---

## 2. DefiLlama (`aggregators/defillama/`)

### What We Have
- Protocols: `Protocols`, `Protocol`, `ProtocolTvl`
- TVL: `TvlAll`, `ChainTvl`
- Prices: `PricesCurrent`, `PricesHistorical`, `PricesFirst`
- Stablecoins: `Stablecoins`, `Stablecoin`, `StablecoinCharts`, `StablecoinChain`
- Yields: `YieldPools`, `YieldPoolChart`
- Fees: `ProtocolFees`
- Volumes: `DexVolumes`
- Pro: `ProAnalytics` (single catch-all placeholder)

### Gap Table

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Protocols | `GET /protocols` | YES | |
| Protocols | `GET /protocol/{protocol}` | YES | |
| Protocols | `GET /tvl/{protocol}` | YES | Simple TVL |
| **Protocols** | `GET /tokenProtocols/{symbol}` | **NO** | Token holders across protocols — Pro |
| **Protocols** | `GET /inflows/{protocol}/{timestamp}` | **NO** | Capital flows into protocol — Pro |
| TVL | `GET /v2/chains` | YES | All chains current TVL |
| TVL | `GET /v2/historicalChainTvl` | **NO** | Historical TVL all chains — missing |
| TVL | `GET /v2/historicalChainTvl/{chain}` | YES | But path in code uses `ChainTvl` |
| **TVL** | `GET /chainAssets` | **NO** | Chain asset breakdown — Pro |
| Prices | `GET /prices/current/{coins}` | YES | |
| Prices | `GET /prices/historical/{timestamp}/{coins}` | YES | |
| **Prices** | `POST /batchHistorical` | **NO** | Batch historical prices — Pro (coins subdomain) |
| **Prices** | `GET /chart/{coins}` | **NO** | Price chart time-series — Pro (coins subdomain) |
| **Prices** | `GET /percentage/{coins}` | **NO** | Price change % — Pro (coins subdomain) |
| Prices | `GET /prices/first/{coins}` | YES | |
| **Prices** | `GET /block/{chain}/{timestamp}` | **NO** | Block number at timestamp (coins subdomain) |
| Stablecoins | `GET /stablecoins` | YES | |
| Stablecoins | `GET /stablecoin/{id}` | YES | |
| Stablecoins | `GET /stablecoincharts/all` | YES | |
| Stablecoins | `GET /stablecoinchains` | YES | |
| **Stablecoins** | `GET /stablecoindominance/{chain}` | **NO** | Dominance per chain |
| **Stablecoins** | `GET /stablecoincharts/{chain}` | **NO** | Chain-specific history |
| **Stablecoins** | `GET /stablecoinprices` | **NO** | Historical stablecoin prices |
| Yields | `GET /pools` | YES | |
| Yields | `GET /chart/{pool}` | YES | |
| **Yields** | `GET /poolsOld` | **NO** | Legacy pools — Pro |
| **Yields** | `GET /poolsBorrow` | **NO** | Borrow rates — Pro |
| **Yields** | `GET /chartLendBorrow/{pool}` | **NO** | Lend/borrow history — Pro |
| **Yields** | `GET /perps` | **NO** | Perpetuals rates — Pro |
| **Yields** | `GET /lsdRates` | **NO** | Liquid staking rates — Pro |
| Fees | `GET /summary/fees/{protocol}` | YES | Protocol fees |
| **Fees** | `GET /overview/fees` | **NO** | All protocols fees overview |
| **Fees** | `GET /overview/fees/{chain}` | **NO** | Fees by chain |
| Volumes | `GET /overview/dexs` | YES | DEX overview |
| **Volumes** | `GET /overview/dexs/{chain}` | **NO** | DEX volumes by chain |
| **Volumes** | `GET /summary/dexs/{protocol}` | **NO** | DEX volume for single protocol |
| **Options** | `GET /overview/options` | **NO** | Options volume overview |
| **Options** | `GET /overview/options/{chain}` | **NO** | Options by chain |
| **Options** | `GET /summary/options/{protocol}` | **NO** | Options for single protocol |
| **Derivatives** | `GET /overview/derivatives` | **NO** | Derivatives volume — Pro |
| **Derivatives** | `GET /summary/derivatives/{protocol}` | **NO** | Derivatives per protocol — Pro |
| **Emissions** | `GET /emissions` | **NO** | All token emissions — Pro |
| **Emissions** | `GET /emission/{protocol}` | **NO** | Protocol vesting schedule — Pro |
| **Ecosystem** | `GET /categories` | **NO** | Protocol categories — Pro |
| **Ecosystem** | `GET /forks` | **NO** | Protocol fork relationships — Pro |
| **Ecosystem** | `GET /oracles` | **NO** | Oracle usage data — Pro |
| **Ecosystem** | `GET /entities` | **NO** | Entity data (companies/funds) — Pro |
| **Ecosystem** | `GET /treasuries` | **NO** | Protocol treasury data — Pro |
| **Ecosystem** | `GET /hacks` | **NO** | DeFi exploits/hacks database — Pro |
| **Ecosystem** | `GET /raises` | **NO** | Funding rounds data — Pro |
| **Ecosystem** | `GET /historicalLiquidity/{token}` | **NO** | Token liquidity history — Pro |
| **ETFs** | `GET /etfs/overview` | **NO** | Bitcoin ETF overview — Pro |
| **ETFs** | `GET /etfs/overviewEth` | **NO** | Ethereum ETF overview — Pro |
| **ETFs** | `GET /etfs/history` | **NO** | Bitcoin ETF history — Pro |
| **ETFs** | `GET /etfs/historyEth` | **NO** | Ethereum ETF history — Pro |
| **ETFs** | `GET /fdv/performance/{period}` | **NO** | FDV performance by period — Pro |
| **Bridges** | `GET /bridges/bridges` | **NO** | All cross-chain bridges — Pro |
| **Bridges** | `GET /bridges/bridge/{id}` | **NO** | Bridge details — Pro |
| **Bridges** | `GET /bridges/bridgevolume/{chain}` | **NO** | Bridge volume per chain — Pro |
| **Bridges** | `GET /bridges/bridgedaystats/{timestamp}/{chain}` | **NO** | Bridge daily stats — Pro |
| **Bridges** | `GET /bridges/transactions/{id}` | **NO** | Bridge transactions — Pro |
| **DAT** | `GET /dat/institutions` | **NO** | Digital asset treasury institutions — Pro |
| **DAT** | `GET /dat/institutions/{symbol}` | **NO** | Institution details — Pro |
| **Account** | `GET /usage/{APIKEY}` | **NO** | API usage stats — Pro |

### Summary: DefiLlama Gaps

- **Free tier gaps** (7 missing endpoints): `historicalChainTvl` (all), `stablecoindominance`, `stablecoincharts/{chain}`, `stablecoinprices`, `overview/fees`, `overview/fees/{chain}`, `overview/dexs/{chain}`, `summary/dexs/{protocol}`, `overview/options`, `overview/options/{chain}`, `summary/options/{protocol}`
- **Pro tier gaps** (30+ endpoints): bridges, ETFs, emissions, ecosystem (hacks/raises/entities/treasuries), advanced yields, derivatives, DAT
- **Coins subdomain gaps** (5): batch historical, price chart, percentage change, block lookup, not all Pro-gated

---

## 3. Interactive Brokers (`aggregators/ib/`)

### What We Have
The connector covers a solid base of the Client Portal Web API:
- Auth: `AuthStatus`, `AuthInit`, `SsoValidate`, `Tickle`, `Logout`
- Portfolio: `PortfolioAccounts`, `PortfolioSubAccounts`, `PortfolioPositions`, `PortfolioPosition`, `PortfolioSummary`, `PortfolioLedger`, `PortfolioAllocation`, `PnlPartitioned`
- Contracts: `ContractSearch`, `ContractInfo`, `ContractInfoAndRules`, `ContractRules`, `ContractAlgos`, `SecdefInfo`
- Market Data: `MarketDataSnapshot`, `MarketDataHistory`, `MarketDataUnsubscribe`, `MarketDataUnsubscribeAll`
- Orders: `PlaceOrder`, `ConfirmOrder`, `LiveOrders`, `Trades`, `ModifyOrder`, `CancelOrder`, `WhatIfOrder`
- Scanner: `ScannerParams`, `ScannerRun`
- Alerts: `CreateAlert`, `GetAlerts`, `DeleteAlert`, `NotificationsUnreadCount`, `GetNotifications`, `MarkNotificationRead`
- Watchlists: `CreateWatchlist`, `GetWatchlists`, `GetWatchlist`, `DeleteWatchlist`
- Portfolio Analytics: `PerformanceMetrics`, `PerformanceSummary`, `TransactionHistory`
- Flex: `FlexGenerate`, `FlexStatus`

### Gap Table — Full Client Portal Web API (79 endpoints)

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| **Authentication** | `GET /iserver/auth/status` | YES | |
| **Authentication** | `POST /iserver/reauthenticate` | **NO** | Refresh expired sessions |
| Authentication | `POST /logout` | YES | |
| Authentication | `POST /sso/validate` | YES | |
| Authentication | `GET /tickle` | YES | |
| Authentication | `POST /iserver/auth/ssodh/init` | YES (as `AuthInit`) | |
| **Alerts** | `POST /iserver/account/alert/activate` | **NO** | Toggle alert on/off by ID |
| **Alerts** | `GET /iserver/account/mta` | **NO** | Mobile Trading Assistant alert |
| Alerts | `POST /iserver/account/{accountId}/alert` | YES (as `CreateAlert`) | |
| Alerts | `DELETE /iserver/account/{accountId}/alert/{alertId}` | YES (as `DeleteAlert`) | Path differs: code uses `order_id` not `alertId` |
| Alerts | `GET /iserver/account/{accountId}/alerts` | YES (as `GetAlerts`) | |
| **Contracts** | `GET /iserver/contract/{conid}/algos` | YES | |
| **Contracts** | `GET /iserver/contract/{conid}/info` | YES | |
| **Contracts** | `GET /iserver/contract/{conid}/info-and-rules` | YES | |
| **Contracts** | `POST /iserver/contract/rules` | YES (as `ContractRules`) | Note: actual method is POST |
| **Contracts** | `GET /iserver/secdef/bond-filters` | **NO** | Bond filter options |
| **Contracts** | `GET /iserver/secdef/currency` | **NO** | Currency pair search |
| Contracts | `GET /iserver/secdef/info` | YES | |
| Contracts | `GET /iserver/secdef/search` | YES | |
| **Contracts** | `GET /iserver/secdef/strikes` | **NO** | Option strike availability |
| **Contracts** | `GET /trsrv/futures` | **NO** | Futures contract list |
| **Contracts** | `GET /trsrv/secdef` | **NO** | Security definitions by conid |
| **Contracts** | `GET /trsrv/secdef/schedule` | **NO** | Trading schedules |
| **Contracts** | `GET /trsrv/stocks` | **NO** | Stock contracts by symbol |
| **Market Data** | `GET /hmds/history` | **NO** | HMDS historical data (separate from iserver) |
| **Market Data** | `GET /iserver/marketdata/availability` | **NO** | Data availability codes |
| **Market Data** | `GET /iserver/marketdata/bars` | **NO** | Valid bar units reference |
| **Market Data** | `GET /iserver/marketdata/fields` | **NO** | Snapshot field definitions |
| Market Data | `GET /iserver/marketdata/history` | YES (as `MarketDataHistory`) | |
| **Market Data** | `GET /iserver/marketdata/periods` | **NO** | Valid period units reference |
| Market Data | `GET /iserver/marketdata/snapshot` | YES | |
| Market Data | `POST /iserver/marketdata/unsubscribe` | YES (as `MarketDataUnsubscribe`) | Path: code uses conid param not POST |
| **Market Data** | `POST /iserver/marketdata/unsubscribeall` | **NO** | Different from current `MarketDataUnsubscribeAll` |
| **Market Data** | `GET /md/snapshot` | **NO** | Non-streaming snapshots (different path prefix) |
| Orders | `DELETE /iserver/account/{accountId}/order/{orderId}` | YES (as `CancelOrder`) | |
| Orders | `POST /iserver/account/{accountId}/order/{orderId}` | YES (as `ModifyOrder`) | |
| Orders | `POST /iserver/account/{accountId}/orders` | YES (as `PlaceOrder`) | |
| Orders | `POST /iserver/account/{accountId}/orders/whatif` | YES (as `WhatIfOrder`) | Path: code uses `whatiforder` not `orders/whatif` |
| Orders | `POST /iserver/reply/{replyId}` | YES (as `ConfirmOrder`) | |
| **Order Monitoring** | `GET /iserver/account/order/status/{orderId}` | **NO** | Single order status by ID |
| Order Monitoring | `GET /iserver/account/orders` | YES (as `LiveOrders`) | |
| Order Monitoring | `GET /iserver/account/trades` | YES (as `Trades`) | |
| Portfolio | `GET /portfolio/accounts` | YES | |
| **Portfolio** | `POST /portfolio/allocation` | **NO** | Multi-account allocation view |
| **Portfolio** | `GET /portfolio/positions/{conid}` | **NO** | Cross-account position by conid |
| Portfolio | `GET /portfolio/subaccounts` | YES | |
| **Portfolio** | `GET /portfolio/subaccounts2` | **NO** | Large tiered account list (>100 subaccounts) |
| Portfolio | `GET /portfolio/{accountId}/allocation` | YES | |
| **Portfolio** | `GET /portfolio/{accountId}/combo/positions` | **NO** | Complex/spread positions |
| Portfolio | `GET /portfolio/{accountId}/ledger` | YES | |
| **Portfolio** | `GET /portfolio/{accountId}/meta` | **NO** | Account metadata |
| **Portfolio** | `POST /portfolio/{accountId}/positions/invalidate` | **NO** | Invalidate position cache |
| Portfolio | `GET /portfolio/{accountId}/positions/{pageId}` | YES (as `PortfolioPositions`) | |
| Portfolio | `GET /portfolio/{accountId}/summary` | YES | |
| **Portfolio** | `GET /portfolio/{acctId}/position/{conid}` | YES (as `PortfolioPosition`) | |
| **Watchlists** | `POST /iserver/account/watchlist/{watchlistId}/contract` | **NO** | Add contract to watchlist |
| **Watchlists** | `DELETE /iserver/account/watchlist/{watchlistId}/contract/{conid}` | **NO** | Remove contract from watchlist |
| **Watchlists** | `GET /iserver/account/watchlist/{watchlistId}` | YES (as `GetWatchlist`) | |
| **Watchlists** | `DELETE /iserver/account/watchlist/{watchlistId}` | YES (as `DeleteWatchlist`) | |
| **Watchlists** | `GET /iserver/account/watchlists` | YES (as `GetWatchlists`) | |
| **Watchlists** | `POST /iserver/account/{accountId}/watchlist` | YES (as `CreateWatchlist`) | |
| **Financial Advisor** | `POST /fa/groups` | **NO** | Create FA allocation group |
| **Financial Advisor** | `GET /fa/groups` | **NO** | List FA allocation groups |
| **Financial Advisor** | `POST /iserver/account/{faGroup}/orders` | **NO** | Place FA group order |
| **Financial Advisor** | `GET /iserver/account/allocation` | **NO** | FA allocation profiles |
| **Financial Advisor** | `POST /iserver/account/allocation` | **NO** | Create/modify allocation preset |
| FYI Notifications | `POST /fyi/deliveryoptions` | **NO** | Toggle delivery method |
| **FYI Notifications** | `GET /fyi/deliveryoptions` | **NO** | Available delivery options |
| **FYI Notifications** | `PUT /fyi/deliveryoptions/device` | **NO** | Device notifications |
| **FYI Notifications** | `DELETE /fyi/notifications` | **NO** | Mark all notifications read |
| FYI Notifications | `GET /fyi/notifications` | YES (as `GetNotifications`) | |
| **FYI Notifications** | `POST /fyi/settings` | **NO** | Toggle disclaimer notifications |
| **FYI Notifications** | `PUT /fyi/settings/{typecode}` | **NO** | Toggle specific disclaimer type |
| FYI Notifications | `GET /fyi/unreadnumber` | YES (as `NotificationsUnreadCount`) | |
| Scanner | `POST /hmds/scanner` | **NO** | HMDS market scanner (separate from iserver) |
| Scanner | `GET /iserver/scanner/params` | YES | |
| Scanner | `POST /iserver/scanner/run` | YES | |
| **Options** | `GET /trsrv/secdef/chains` | **NO** | Full option chain data |
| **Events** | `GET /events/contracts` | **NO** | Event-driven contracts |
| **Events** | `GET /events/show` | **NO** | Specific event contracts |
| Portfolio Analyst | `POST /pa/allperiods` | **NO** | Available PA reporting periods |
| Portfolio Analyst | `POST /pa/performance` | YES (as `PerformanceMetrics`) | But method: GET in code vs POST in spec |
| Portfolio Analyst | `POST /pa/transactions` | YES (as `TransactionHistory`) | But method: GET in code vs POST in spec |

### Notable IB Bugs in Current Implementation

| Issue | Details |
|-------|---------|
| `WhatIfOrder` path | Code uses `/iserver/account/{id}/whatiforder` but spec requires `/iserver/account/{id}/orders/whatif` |
| `MarketDataUnsubscribeAll` | Code uses GET path `marketdata/unsubscribe` but spec requires POST `/iserver/marketdata/unsubscribeall` |
| `PerformanceMetrics` | Code uses `GET /pa/performance` but spec says POST |
| `TransactionHistory` | Code uses `GET /pa/transactions` but spec says POST |
| `MarkNotificationRead` | Code maps to `/fyi/notification/{id}` — confirm vs spec `/fyi/notifications` DELETE |
| FA Groups | Missing completely — no financial advisor group management |

---

## 4. Yahoo Finance (`aggregators/yahoo/`)

### What We Have
- Market Data: `Quote`, `Chart`, `QuoteSummary` (v10), `MarketSummary`, `Spark`
- Historical: `DownloadHistory` (CSV with crumb)
- Options: `Options`
- Search: `Search`, `Lookup`, `ScreenerPredefined`, `ScreenerCustom`, `Trending`, `RecommendationsBySymbol`
- Fundamentals TS: `FundamentalsTimeSeries`
- Auth: `GetCrumb`
- QuoteSummary modules: 30+ modules documented in `quote_summary_modules` mod

### Gap Table

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Market Data | `GET /v7/finance/quote` | YES | |
| Market Data | `GET /v8/finance/chart/{symbol}` | YES | |
| Market Data | `GET /v10/finance/quoteSummary/{symbol}` | YES | Note: code path is `/v10/` but enum maps to `/v10/` ✓ |
| Market Data | `GET /v6/finance/quote/marketSummary` | YES | |
| Market Data | `GET /v1/finance/spark` | YES | |
| **Market Data** | `GET /v7/finance/quote?symbols=&fields=` | **NO** | Extended field selection — same endpoint, needs field param support |
| **Market Data** | `GET /v8/finance/chart/{symbol}?includePrePost=true` | **NO** | Pre/post market data parameter |
| **Real-time** | `WSS wss://streamer.finance.yahoo.com/` | **NO** | WebSocket streaming (base URL exists but not implemented) |
| Historical | `GET /v7/finance/download/{symbol}` | YES (as `DownloadHistory`) | Requires crumb |
| **Historical** | `GET /v8/finance/chart/{symbol}?period1=&period2=` | **NO** | Date-range chart data — distinct from interval chart |
| Options | `GET /v7/finance/options/{symbol}` | YES | |
| **Options** | `GET /v7/finance/options/{symbol}?date={timestamp}` | **NO** | Options for specific expiry date |
| **Earnings** | `GET /v1/finance/calendar/earnings` | **NO** | Earnings calendar (all upcoming) |
| **Earnings** | `GET /v10/finance/quoteSummary/{symbol}?modules=calendarEvents` | Partial | Covered via quoteSummary but no dedicated endpoint |
| **News** | `GET /v1/finance/search?q={symbol}&newsCount=10` | Partial | Embedded in Search response |
| **News** | `GET /v2/finance/news` | **NO** | Dedicated news endpoint (v2) |
| **Analyst** | `GET /v1/finance/recommendationsBySymbol/{symbol}` | YES | |
| **Analyst** | `GET /v10/finance/quoteSummary?modules=earningsTrend` | Partial | Via quoteSummary |
| Search | `GET /v1/finance/search` | YES | |
| Search | `GET /v1/finance/lookup` | YES | |
| Search | `GET /v1/finance/screener/predefined` | YES | |
| Search | `POST /v1/finance/screener` | YES (as `ScreenerCustom`) | |
| Search | `GET /v1/finance/trending/{region}` | YES | |
| Search | `GET /v1/finance/recommendationsBySymbol/{symbol}` | YES | |
| **Insights** | `GET /v1/finance/insights` | **NO** | Yahoo Finance Insights (Smart Holdings, etc.) |
| **Insights** | `GET /v1/finance/insights?symbol=` | **NO** | Per-symbol insights |
| Fundamentals TS | `GET /ws/fundamentals-timeseries/v1/finance/timeseries/{symbol}` | YES | |
| **Quote Summary** | `assetProfile` module | YES (documented) | |
| **Quote Summary** | `balanceSheetHistory` module | YES (documented) | |
| **Quote Summary** | `calendarEvents` module | YES (documented) | |
| **Quote Summary** | `cashflowStatementHistory` module | YES (documented) | |
| **Quote Summary** | `defaultKeyStatistics` module | YES (documented) | |
| **Quote Summary** | `earnings` module | YES (documented) | |
| **Quote Summary** | `earningsHistory` module | YES (documented) | |
| **Quote Summary** | `earningsTrend` module | YES (documented) | |
| **Quote Summary** | `esgScores` module | YES (documented) | |
| **Quote Summary** | `financialData` module | YES (documented) | |
| **Quote Summary** | `fundOwnership` module | YES (documented) | |
| **Quote Summary** | `incomeStatementHistory` module | YES (documented) | |
| **Quote Summary** | `insiderHolders` module | YES (documented) | |
| **Quote Summary** | `insiderTransactions` module | YES (documented) | |
| **Quote Summary** | `institutionOwnership` module | YES (documented) | |
| **Quote Summary** | `majorHoldersBreakdown` module | YES (documented) | |
| **Quote Summary** | `recommendationTrend` module | YES (documented) | |
| **Quote Summary** | `secFilings` module | YES (documented) | |
| **Quote Summary** | `summaryDetail` module | YES (documented) | |
| **Quote Summary** | `upgradeDowngradeHistory` module | YES (documented) | |
| Auth | `GET /v1/test/getcrumb` | YES | |

### Notes on Yahoo Finance

Yahoo Finance provides no official API. All endpoints are reverse-engineered and subject to change without notice. Key practical gaps:

1. **WebSocket not implemented**: The `ws_base` field is set to `wss://streamer.finance.yahoo.com/` but no WebSocket code exists.
2. **Earnings calendar**: `GET /v1/finance/calendar/earnings` is frequently used by the community to get upcoming earnings — not implemented.
3. **v11 quoteSummary**: Some community projects reference `/v11/finance/quoteSummary` as an updated path — our code uses `/v10/`.
4. **Insights endpoint**: `GET /v1/finance/insights` provides "Smart Holdings" and technical signals — not implemented.
5. **Pre/post market data**: The chart endpoint supports `includePrePost=true` parameter — not wired up in the interval mapper.

---

## Summary Matrix

| Connector | Endpoints We Have | Key Missing (Free/Standard) | Key Missing (Paid/Advanced) |
|-----------|-------------------|----------------------------|------------------------------|
| CryptoCompare | ~30 (legacy min-api only) | Orderbook L1/L2, tradebook, mining, exchange-level volume history, new data-api entirely | New data-api: spot_v1, indices, asset_v1, on-chain DEX, data streamer WebSocket |
| DefiLlama | ~16 | `stablecoindominance`, `stablecoincharts/{chain}`, `stablecoinprices`, `overview/fees`, `overview/dexs/{chain}`, `overview/options`, `summary/options`, `historicalChainTvl` (all) | Bridges (5), ETFs (5), emissions (2), ecosystem/hacks/raises/entities (8), advanced yields (5), derivatives (2), DAT (2) |
| IB Client Portal | ~42 of 79 | `reauthenticate`, `trsrv/*` (4 endpoints), `hmds/history`, `hmds/scanner`, `md/snapshot`, FA groups (5), `portfolio/subaccounts2`, `portfolio/allocation`, order status by ID, watchlist add/remove contract | Events (2), PA periods |
| Yahoo Finance | ~14 | Earnings calendar, WebSocket streaming, insights endpoint, `news/v2`, pre/post market param | N/A (all unofficial) |

---

## Priority Recommendations

### High Priority (High utility, free/accessible)
1. **DefiLlama**: Add `overview/fees`, `overview/fees/{chain}`, `overview/dexs/{chain}`, `summary/dexs/{protocol}`, `overview/options` — all free tier
2. **DefiLlama**: Add `stablecoindominance/{chain}`, `stablecoincharts/{chain}`, `stablecoinprices` — all free tier
3. **DefiLlama**: Fix missing `historicalChainTvl` (all-chains version, no `{chain}` param) — currently only per-chain version exists
4. **IB**: Add `trsrv/stocks`, `trsrv/futures`, `trsrv/secdef/chains` (options chains) — core trading utility
5. **IB**: Add `iserver/reauthenticate` — needed for session recovery
6. **IB**: Fix HTTP method bugs: `pa/performance` and `pa/transactions` should be POST not GET
7. **Yahoo**: Add earnings calendar endpoint `GET /v1/finance/calendar/earnings`
8. **Yahoo**: Add `insights` endpoint `GET /v1/finance/insights`
9. **CryptoCompare**: Add missing free legacy endpoints: exchange volume history, mining data

### Medium Priority (Useful, Pro-gated or complex)
1. **DefiLlama Pro**: Bridges, ETFs, emissions, raises, hacks — highly valuable for DeFi intelligence
2. **IB**: Financial advisor endpoints (`/fa/groups`, `/iserver/account/allocation/*`) — needed for institutional use
3. **IB**: `hmds/history` and `hmds/scanner` (HMDS = Historical Market Data Service, separate from iserver)
4. **CryptoCompare New Data API**: `spot_v1` historical and real-time endpoints — official successor to min-api
5. **Yahoo**: WebSocket streaming implementation

### Low Priority (Niche or duplicative)
1. **CryptoCompare**: Mining equipment/pool data — niche use case
2. **IB**: Events contracts — specialized product
3. **DefiLlama**: DAT (Digital Asset Treasury) — institutional research only
4. **Yahoo**: Sparkline differences — already covered by `Chart`

---

## Sources

- [CoinDesk Data API Documentation (formerly CryptoCompare)](https://developers.coindesk.com/documentation)
- [CryptoCompare Legacy Min-API](https://min-api.cryptocompare.com/)
- [CoinDesk Data API - Spot v1 Markets](https://developers.coindesk.com/documentation/data-api/spot_v1_markets)
- [CoinDesk Data API - Indices](https://developers.coindesk.com/documentation/data-api/index_cc)
- [CoinDesk Data API - News v1](https://developers.coindesk.com/documentation/data-api/news_v1_source_list)
- [CoinDesk Data Streamer - Order Book](https://developers.coindesk.com/documentation/data-streamer/spot_v1_orderbook_realtime_l1_top_of_book)
- [DefiLlama API Docs (llms.txt)](https://api-docs.defillama.com/llms.txt)
- [DefiLlama API Docs](https://api-docs.defillama.com/)
- [IB Client Portal API Documentation](https://interactivebrokers.github.io/cpwebapi/)
- [IB Web API v1.0 Documentation](https://www.interactivebrokers.com/campus/ibkr-api-page/cpapi-v1/)
- [IB_MCP ENDPOINTS.md](https://raw.githubusercontent.com/rcontesti/IB_MCP/main/ENDPOINTS.md)
- [yahoo-finance2 (unofficial Yahoo Finance library)](https://github.com/gadicc/yahoo-finance2)
- [Yahoo Finance API Collection](https://github.com/Scarvy/yahoo-finance-api-collection)
