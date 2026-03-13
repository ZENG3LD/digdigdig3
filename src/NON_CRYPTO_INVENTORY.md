# Non-Crypto Connector Inventory

**digdigdig3/src — all non-crypto connectors**
Generated: 2026-03-13

---

## Classification Legend

| Class | Meaning |
|-------|---------|
| **FULL TRADING** | Trading + Account + Positions all have real implementations |
| **PARTIAL** | Some trading/account ops real, some UnsupportedOperation |
| **DATA ONLY** | Implements V5 ExchangeIdentity + MarketData; Trading/Account/Positions all return UnsupportedOperation |
| **RAW FEED** | Does NOT implement V5 exchange traits; exposes only domain-specific methods |
| **STUB** | Registered in V5 system but all methods return UnsupportedOperation (technical blocker) |

Trait columns use: `YES` = real implementation, `NO` = UnsupportedOperation, `—` = trait not implemented at all

---

## Stocks — US (`src/stocks/us/`)

| Provider | File | Class | Trading | Account | Positions | Notes |
|----------|------|-------|---------|---------|-----------|-------|
| **Alpaca** | `stocks/us/alpaca/connector.rs` | FULL TRADING | YES | YES | YES | US stocks + crypto. `get_order_history` → NO, `get_fees` → NO, `get_funding_rate` → NO. Paper trading testnet supported. Extended: `get_assets`, `get_clock`, `get_calendar`, `close_position`, `close_all_positions`, `cancel_all_orders`, `get_news` |
| **Polygon** | `stocks/us/polygon/connector.rs` | DATA ONLY | NO | NO | NO | 5 req/min free tier. Full MarketData. ExchangeType::Cex |
| **Finnhub** | `stocks/us/finnhub/connector.rs` | DATA ONLY | NO | NO | NO | 60 req/min. Full MarketData (Quote, BidAsk, StockCandles, MarketStatus, StockSymbols) |
| **Tiingo** | `stocks/us/tiingo/connector.rs` | DATA ONLY | NO | NO | NO | `get_orderbook` → NO. ExchangeType::DataProvider. Extended: `get_daily_prices`, `get_crypto_top`, `get_crypto_prices`, `get_forex_top`, `get_forex_prices` |
| **TwelveData** | `stocks/us/twelvedata/connector.rs` | DATA ONLY | NO | NO | NO | `get_orderbook` → NO. ExchangeType::DataProvider. Extended: `symbol_search`, `get_stocks`, `get_forex_pairs`, `get_cryptocurrencies`, `market_state`, `rsi`, `macd` |

---

## Stocks — India (`src/stocks/india/`)

All Indian brokers: NSE/BSE equity, F&O, commodities, currency segments. None support `get_order_history` or `get_fees`. All use JWT Bearer token auth.

| Provider | File | Class | Trading | Account | Positions | Notes |
|----------|------|-------|---------|---------|-----------|-------|
| **AngelOne** | `stocks/india/angel_one/connector.rs` | FULL TRADING | YES | YES | YES | TOTP 2FA + JWT; logs in during `new()`. `get_order_history` → NO, `get_fees` → NO, `get_funding_rate` → NO, `modify_position(SetLeverage)` → NO. Extended: `refresh_token`, `logout`, `get_profile`, `search_scrip`, `get_holdings`, `modify_order`, `get_order_book`, `get_trade_book`, `get_rms`, `convert_position`, `calculate_margin` |
| **Dhan** | `stocks/india/dhan/connector.rs` | FULL TRADING | YES | YES | YES | Testnet supported. 4 separate rate limiters (orders: 25/s, data: 5/s, quote: 1/s, general: 20/s). `get_order_history` → NO, `get_fees` → NO, `get_funding_rate` → NO, `modify_position` → NO |
| **Fyers** | `stocks/india/fyers/connector.rs` | FULL TRADING | YES | YES | YES | OAuth2 flow; separate data endpoint URL. `get_order_history` → NO, `get_fees` → NO, `get_funding_rate` → NO, `modify_position` → NO. Extended: `get_holdings`, `get_tradebook`, `convert_position`, `modify_order`, `exchange_auth_code`, `get_authorization_url` |
| **Upstox** | `stocks/india/upstox/connector.rs` | FULL TRADING | YES | YES | YES | OAuth2; HFT endpoint option; V3 API. `get_order_history` → NO, `get_fees` → NO, `get_funding_rate` → NO, `modify_position` → NO. Symbol master from gzip JSON CDN. Extended: `get_holdings`, `get_user_profile`, `cancel_all_orders` |
| **Zerodha** | `stocks/india/zerodha/connector.rs` | PARTIAL | YES | YES | YES | Kite Connect API. `get_klines` → NO (requires `instrument_token` lookup first). `get_order_history` → NO, `get_fees` → NO, `get_funding_rate` → NO, `modify_position` → NO. Symbol format: `"NSE:SBIN"`. Extended: `get_holdings`, `convert_position` |

---

## Stocks — Japan (`src/stocks/japan/`)

| Provider | File | Class | Trading | Account | Positions | Notes |
|----------|------|-------|---------|---------|-----------|-------|
| **JQuants** | `stocks/japan/jquants/connector.rs` | DATA ONLY | NO | NO | NO | JPX/TSE. Free tier: daily data only; intraday = premium. `get_orderbook` → NO. Dual token auth: refresh_token → id_token (cached with expiry). Extended: `get_symbols` |

---

## Stocks — Korea (`src/stocks/korea/`)

| Provider | File | Class | Trading | Account | Positions | Notes |
|----------|------|-------|---------|---------|-----------|-------|
| **KRX** | `stocks/korea/krx/connector.rs` | DATA ONLY | NO | NO | NO | KOSPI/KOSDAQ/KONEX. Only `1d` interval supported. Per-date API (must loop for date ranges). `get_orderbook` → NO. Requires `AUTH_KEY`. Extended: `get_stock_info`, `get_base_info`, `get_index_data` |

---

## Stocks — China (`src/stocks/china/`)

| Provider | File | Class | Trading | Account | Positions | Notes |
|----------|------|-------|---------|---------|-----------|-------|
| **Futu** | `stocks/china/futu/connector.rs` | STUB | NO | NO | NO | ALL methods return UnsupportedOperation. Futu OpenAPI uses TCP + Protocol Buffers, NOT HTTP REST. Struct is a placeholder. Note: "Run OpenD gateway; implement Protobuf client; or use Python SDK via PyO3/FFI" |

---

## Stocks — Russia (`src/stocks/russia/`)

| Provider | File | Class | Trading | Account | Positions | Notes |
|----------|------|-------|---------|---------|-----------|-------|
| **MOEX** | `stocks/russia/moex/connector.rs` | DATA ONLY | NO | NO | NO | MOEX ISS. Public access (15-min delay); authenticated = real-time. Full MarketData. Extended: `get_symbols`, `get_engines`, `get_markets`, `get_security_info`, `get_turnovers`, `has_realtime_access` |
| **Tinkoff** | `stocks/russia/tinkoff/connector.rs` | FULL TRADING | YES | YES | YES | Russian broker (MOEX stocks, bonds, ETFs, futures, options). FIGI identifiers (requires `get_figi_by_ticker()` lookup). Sandbox/testnet. All requests are POST. `get_order_history` → NO, `get_fees` → NO, `get_funding_rate` → NO, `modify_position` → NO. Extended: `get_accounts_list`, `initialize_account`, `get_figi_by_ticker`, `get_symbols` |

---

## Forex (`src/forex/`)

| Provider | File | Class | Trading | Account | Positions | Notes |
|----------|------|-------|---------|---------|-----------|-------|
| **OANDA** | `forex/oanda/connector.rs` | FULL TRADING | YES | YES | YES | Forex broker. Bearer token; practice/live modes; HTTP streaming (not WebSocket). Symbol format: `"EUR_USD"`. `get_order_history` → NO, `get_fees` → NO, `get_funding_rate` → NO, `modify_position(SetLeverage)` → NO ("set at account level"). Extended: `get_instruments`, `close_all_positions` |
| **AlphaVantage** | `forex/alphavantage/connector.rs` | DATA ONLY | NO | NO | NO | Forex/stock/crypto data. `get_ticker` → NO, `get_orderbook` → NO. `get_klines`: daily/weekly/monthly free; intraday = premium |
| **Dukascopy** | `forex/dukascopy/connector.rs` | DATA ONLY | NO | NO | NO | Downloads binary LZMA-compressed `.bi5` tick files. No API key. `get_orderbook` → NO. `get_price`/`get_ticker` from latest tick (backtracks 72h). `get_klines` constructed from ticks via aggregation |

---

## Aggregators (`src/aggregators/`)

| Provider | File | Class | Trading | Account | Positions | Notes |
|----------|------|-------|---------|---------|-----------|-------|
| **CryptoCompare** | `aggregators/cryptocompare/connector.rs` | DATA ONLY | NO | NO | NO | Multi-exchange crypto price aggregator. `get_orderbook` → NO (paid WebSocket only). Extended: `get_historical_price`, `get_top_exchanges`, `get_rate_limit` |
| **Yahoo Finance** | `aggregators/yahoo/connector.rs` | DATA ONLY | NO | NO | NO | `get_orderbook` → NO. `get_price`/`get_ticker` via Chart endpoint (Quote → 401 since Jan 2026). Extended: `get_market_summary`, `search_symbols`, `get_quote_summary`, `get_asset_profile`, `get_financial_data`, `get_earnings`, `get_options_chain`, `download_history_csv` |
| **DefiLlama** | `aggregators/defillama/connector.rs` | DATA ONLY | — | — | — | DeFi TVL aggregator. Implements ExchangeIdentity + MarketData only. `get_klines` → NO, `get_orderbook` → NO. `get_price` and `get_ticker` work via coingecko ID format. Extended: `get_protocols`, `get_protocol`, `get_protocol_tvl_history`, `get_token_prices`, `get_chains`, `get_stablecoins`, `get_yield_pools` |
| **Interactive Brokers** | `aggregators/ib/connector.rs` | PARTIAL | — | — | — | Implements ExchangeIdentity + MarketData only. `get_orderbook` → NO (TWS API required for L2). `get_price`, `get_ticker`, `get_klines` work via Client Portal Web API. Requires IB Gateway running locally + browser auth. Symbol cached via `conid` lookup. Extended: `get_positions`, `get_account_summary`. NOTE: Trading/Account/Positions traits NOT implemented (no V5 stubs) |

---

## Intelligence Feeds (`src/intelligence_feeds/`)

Intelligence feeds are divided into two sub-groups based on trait implementation:

### Group A: V5-Compliant (ExchangeIdentity + MarketData implemented)

These two connectors fully implement the V5 trait interface but return UnsupportedOperation for all trading operations.

| Provider | File | Class | Trading | Account | Positions | Notes |
|----------|------|-------|---------|---------|-----------|-------|
| **Coinglass** | `intelligence_feeds/crypto/coinglass/connector.rs` | DATA ONLY | NO | NO | NO | Derivatives analytics (liquidations, OI, funding rates, L/S ratios). ExchangeId::Coinglass. All MarketData methods → NO. Extended: `get_liquidations`, `get_open_interest_ohlc`, `get_funding_rate_data`, `get_long_short_ratio` |
| **FRED** | `intelligence_feeds/economic/fred/connector.rs` | DATA ONLY | NO | NO | NO | 840,000+ Federal Reserve economic series. ExchangeId::Fred. All MarketData methods → NO. Extended: `get_series_observations`, `search_series`, `get_series_metadata`, `get_categories`, `get_releases`, geography methods |

### Group B: Raw Feeds (Domain-specific only, no V5 exchange traits)

These connectors do NOT implement ExchangeIdentity, MarketData, Trading, Account, or Positions. They expose only domain-specific methods directly and use `ExchangeError`/`ExchangeResult` for error handling.

**Academic:**
| Provider | File | Data Domain |
|----------|------|-------------|
| **arXiv** | `intelligence_feeds/academic/arxiv/connector.rs` | Research papers (2M+). Methods: `search`, `search_by_title`, `get_quantitative_finance`. No auth required |
| **Semantic Scholar** | `intelligence_feeds/academic/semantic_scholar/connector.rs` | Academic paper database. Methods: paper search, author lookup, citation graph |

**Aviation:**
| Provider | File | Data Domain |
|----------|------|-------------|
| **ADS-B Exchange** | `intelligence_feeds/aviation/adsb_exchange/connector.rs` | Aircraft tracking (ADS-B). Methods: aircraft by registration, flight tracking |
| **AviationStack** | `intelligence_feeds/aviation/aviationstack/connector.rs` | Flight status, routes, airlines, airports |
| **OpenSky** | `intelligence_feeds/aviation/opensky/connector.rs` | Live aircraft positions, state vectors |
| **Wingbits** | `intelligence_feeds/aviation/wingbits/connector.rs` | ADS-B data with token rewards |

**C2/Threat Intel:**
| Provider | File | Data Domain |
|----------|------|-------------|
| **C2 Intel Feeds** | `intelligence_feeds/c2intel_feeds/connector.rs` | Command & control server threat feeds |

**Conflict/Geopolitics:**
| Provider | File | Data Domain |
|----------|------|-------------|
| **ACLED** | `intelligence_feeds/conflict/acled/connector.rs` | Armed Conflict Location & Event Data |
| **GDELT** | `intelligence_feeds/conflict/gdelt/connector.rs` | Global news event database (300B+ events) |
| **ReliefWeb** | `intelligence_feeds/conflict/reliefweb/connector.rs` | Humanitarian crisis reports (OCHA) |
| **UCDP** | `intelligence_feeds/conflict/ucdp/connector.rs` | Uppsala Conflict Data Program (organized violence) |
| **UNHCR** | `intelligence_feeds/conflict/unhcr/connector.rs` | Refugee and displacement data |

**Corporate:**
| Provider | File | Data Domain |
|----------|------|-------------|
| **GLEIF** | `intelligence_feeds/corporate/gleif/connector.rs` | Legal Entity Identifier (LEI) database |
| **OpenCorporates** | `intelligence_feeds/corporate/opencorporates/connector.rs` | Global company registry (200+ countries) |
| **UK Companies House** | `intelligence_feeds/corporate/uk_companies_house/connector.rs` | UK company filings and director data |

**Crypto (non-V5):**
| Provider | File | Data Domain |
|----------|------|-------------|
| **CoinGecko** | `intelligence_feeds/crypto/coingecko/connector.rs` | Crypto prices, market cap, DeFi data |

**Cybersecurity:**
| Provider | File | Data Domain |
|----------|------|-------------|
| **AbuseIPDB** | `intelligence_feeds/cyber/abuseipdb/connector.rs` | IP abuse reports and blacklists |
| **AlienVault OTX** | `intelligence_feeds/cyber/alienvault_otx/connector.rs` | Open Threat Exchange — IOCs, pulses |
| **Censys** | `intelligence_feeds/cyber/censys/connector.rs` | Internet-wide scanning (hosts, certs) |
| **Cloudflare Radar** | `intelligence_feeds/cyber/cloudflare_radar/connector.rs` | Internet traffic trends, DDoS data |
| **NVD** | `intelligence_feeds/cyber/nvd/connector.rs` | NIST CVE/vulnerability database |
| **RIPE NCC** | `intelligence_feeds/cyber/ripe_ncc/connector.rs` | IP routing, BGP, ASN data |
| **Shodan** | `intelligence_feeds/cyber/shodan/connector.rs` | Internet-connected device scanning |
| **URLhaus** | `intelligence_feeds/cyber/urlhaus/connector.rs` | Malicious URL tracking |
| **VirusTotal** | `intelligence_feeds/cyber/virustotal/connector.rs` | File/URL malware scanning |

**Demographics:**
| Provider | File | Data Domain |
|----------|------|-------------|
| **UN OCHA** | `intelligence_feeds/demographics/un_ocha/connector.rs` | Humanitarian data (HDX) |
| **UN Population** | `intelligence_feeds/demographics/un_population/connector.rs` | World population statistics |
| **WHO** | `intelligence_feeds/demographics/who/connector.rs` | World Health Organization data |
| **Wikipedia** | `intelligence_feeds/demographics/wikipedia/connector.rs` | Wikipedia search and article data |

**Economic (non-V5):**
| Provider | File | Data Domain |
|----------|------|-------------|
| **BIS** | `intelligence_feeds/economic/bis/connector.rs` | Bank for International Settlements statistics |
| **BoE** | `intelligence_feeds/economic/boe/connector.rs` | Bank of England interest rates, statistics |
| **Bundesbank** | `intelligence_feeds/economic/bundesbank/connector.rs` | Deutsche Bundesbank time series |
| **CBR** | `intelligence_feeds/economic/cbr/connector.rs` | Central Bank of Russia rates/statistics |
| **DB.nomics** | `intelligence_feeds/economic/dbnomics/connector.rs` | Aggregated economic databases (ECB, IMF, etc.) |
| **ECB** | `intelligence_feeds/economic/ecb/connector.rs` | European Central Bank statistical data |
| **ECOS** | `intelligence_feeds/economic/ecos/connector.rs` | Economic Statistics System (Bank of Korea) |
| **Eurostat** | `intelligence_feeds/economic/eurostat/connector.rs` | EU statistical data |
| **IMF** | `intelligence_feeds/economic/imf/connector.rs` | IMF financial data (IFS, WEO, DOTS) |
| **OECD** | `intelligence_feeds/economic/oecd/connector.rs` | OECD economic statistics |
| **World Bank** | `intelligence_feeds/economic/worldbank/connector.rs` | World development indicators |

**Environment:**
| Provider | File | Data Domain |
|----------|------|-------------|
| **GDACS** | `intelligence_feeds/environment/gdacs/connector.rs` | Global Disaster Alert and Coordination System |
| **Global Forest Watch** | `intelligence_feeds/environment/global_forest_watch/connector.rs` | Deforestation monitoring |
| **NASA EONET** | `intelligence_feeds/environment/nasa_eonet/connector.rs` | Natural events (wildfires, storms, floods) |
| **NASA FIRMS** | `intelligence_feeds/environment/nasa_firms/connector.rs` | Fire Information for Resource Management |
| **NOAA** | `intelligence_feeds/environment/noaa/connector.rs` | Weather, climate, ocean data |
| **NWS Alerts** | `intelligence_feeds/environment/nws_alerts/connector.rs` | National Weather Service active alerts |
| **OpenWeatherMap** | `intelligence_feeds/environment/open_weather_map/connector.rs` | Weather forecasts and conditions |
| **OpenAQ** | `intelligence_feeds/environment/openaq/connector.rs` | Air quality monitoring data |
| **USGS Earthquake** | `intelligence_feeds/environment/usgs_earthquake/connector.rs` | Earthquake data from USGS |

**FAA/Aviation Status:**
| Provider | File | Data Domain |
|----------|------|-------------|
| **FAA Status** | `intelligence_feeds/faa_status/connector.rs` | US FAA airport delays and ground stops |

**Threat Tracking:**
| Provider | File | Data Domain |
|----------|------|-------------|
| **Feodo Tracker** | `intelligence_feeds/feodo_tracker/connector.rs` | Botnet C&C tracker (Abuse.ch) |

**Financial (non-V5):**
| Provider | File | Data Domain |
|----------|------|-------------|
| **AlphaVantage** (intel) | `intelligence_feeds/financial/alpha_vantage/connector.rs` | Duplicate entry in intel_feeds namespace |
| **Finnhub** (intel) | `intelligence_feeds/financial/finnhub/connector.rs` | Duplicate entry in intel_feeds namespace |
| **NewsAPI** | `intelligence_feeds/financial/newsapi/connector.rs` | News headlines and articles |
| **OpenFIGI** | `intelligence_feeds/financial/openfigi/connector.rs` | Financial Instrument Global Identifier lookup |

**Governance:**
| Provider | File | Data Domain |
|----------|------|-------------|
| **EU Parliament** | `intelligence_feeds/governance/eu_parliament/connector.rs` | EU legislative data (MEPs, votes, documents) |
| **UK Parliament** | `intelligence_feeds/governance/uk_parliament/connector.rs` | UK parliamentary bills, debates, members |

**Hacker News:**
| Provider | File | Data Domain |
|----------|------|-------------|
| **Hacker News** | `intelligence_feeds/hacker_news/connector.rs` | HN stories, comments, user data |

**Maritime:**
| Provider | File | Data Domain |
|----------|------|-------------|
| **AIS** | `intelligence_feeds/maritime/ais/connector.rs` | Vessel tracking (generic AIS) |
| **AISstream** | `intelligence_feeds/maritime/aisstream/connector.rs` | WebSocket AIS ship tracking stream |
| **IMF PortWatch** | `intelligence_feeds/maritime/imf_portwatch/connector.rs` | Port cargo throughput and congestion |
| **NGA Warnings** | `intelligence_feeds/maritime/nga_warnings/connector.rs` | Maritime warnings (piracy, military) |

**Prediction Markets:**
| Provider | File | Data Domain |
|----------|------|-------------|
| **PredictIt** | `intelligence_feeds/prediction/predictit/connector.rs` | Political prediction market prices |

**RSS/News:**
| Provider | File | Data Domain |
|----------|------|-------------|
| **RSS Proxy** | `intelligence_feeds/rss_proxy/connector.rs` | Generic RSS/Atom feed reader |

**Sanctions:**
| Provider | File | Data Domain |
|----------|------|-------------|
| **Interpol** | `intelligence_feeds/sanctions/interpol/connector.rs` | Interpol red notices and wanted persons |
| **OFAC** | `intelligence_feeds/sanctions/ofac/connector.rs` | US Treasury sanctions list (SDN) |
| **OpenSanctions** | `intelligence_feeds/sanctions/opensanctions/connector.rs` | Aggregated global sanctions database |

**Space:**
| Provider | File | Data Domain |
|----------|------|-------------|
| **Launch Library** | `intelligence_feeds/space/launch_library/connector.rs` | Rocket launch schedules |
| **NASA** | `intelligence_feeds/space/nasa/connector.rs` | NASA APOD, NeoWs, Earth imagery |
| **Sentinel Hub** | `intelligence_feeds/space/sentinel_hub/connector.rs` | Satellite imagery (Copernicus) |
| **Space-Track** | `intelligence_feeds/space/space_track/connector.rs` | Satellite orbital elements (TLE) |
| **SpaceX** | `intelligence_feeds/space/spacex/connector.rs` | SpaceX launches, rockets, crew |

**Trade/Procurement:**
| Provider | File | Data Domain |
|----------|------|-------------|
| **UN Comtrade** | `intelligence_feeds/trade/comtrade/connector.rs` | International merchandise trade statistics |
| **EU TED** | `intelligence_feeds/trade/eu_ted/connector.rs` | EU public procurement tenders |

**US Government:**
| Provider | File | Data Domain |
|----------|------|-------------|
| **BEA** | `intelligence_feeds/us_gov/bea/connector.rs` | Bureau of Economic Analysis (GDP, national accounts) |
| **BLS** | `intelligence_feeds/us_gov/bls/connector.rs` | Bureau of Labor Statistics (CPI, employment) |
| **US Census** | `intelligence_feeds/us_gov/census/connector.rs` | US Census Bureau demographics and trade |
| **US Congress** | `intelligence_feeds/us_gov/congress/connector.rs` | Congressional bills, votes, members |
| **EIA** | `intelligence_feeds/us_gov/eia/connector.rs` | Energy Information Administration data |
| **FBI Crime** | `intelligence_feeds/us_gov/fbi_crime/connector.rs` | FBI Uniform Crime Reporting data |
| **SAM.gov** | `intelligence_feeds/us_gov/sam_gov/connector.rs` | Federal contractor and grants database |
| **SEC EDGAR** | `intelligence_feeds/us_gov/sec_edgar/connector.rs` | SEC filings (10-K, 10-Q, 8-K, insider trading) |
| **USASpending** | `intelligence_feeds/us_gov/usaspending/connector.rs` | Federal contract and grant spending |

---

## Onchain (`src/onchain/`)

| Provider | File | Class | Trading | Account | Positions | Notes |
|----------|------|-------|---------|---------|-----------|-------|
| **Bitquery** | `onchain/analytics/bitquery/connector.rs` | DATA ONLY | NO | NO | NO | GraphQL blockchain data. All MarketData → NO ("use get_dex_trades()"). Trading/Account/Positions all → NO. Rate limiter: 10 req/min (free). Extended: `get_dex_trades`, `get_realtime_dex_trades`, `get_token_transfers`, `get_balance_updates`, `get_blocks`, `get_transactions`, `get_smart_contract_events` |
| **WhaleAlert** | `onchain/analytics/whale_alert/connector.rs` | DATA ONLY | NO | NO | NO | Blockchain transaction tracking. All MarketData → NO. Trading/Account/Positions all → NO. Extended: `get_status`, `get_blockchain_status`, `get_transaction`, `get_transactions`, `get_block`, `get_address_transactions`, `get_address_attributions` |
| **Etherscan** | `onchain/ethereum/etherscan/connector.rs` | RAW FEED | — | — | — | Ethereum blockchain explorer. Does NOT implement V5 exchange traits. Domain-only: `get_balance`, `get_multi_balance`, `get_transactions`, `get_token_transfers`, `get_internal_transactions`, `get_eth_price`, `get_eth_supply`, `get_chain_size`, `get_token_supply`, `get_gas_oracle`, `get_latest_block_number`, `get_block_reward`, `get_contract_abi`. Testnet (Sepolia) supported |

---

## Summary

### Full Trading Brokers (9 connectors)

| Connector | Market |
|-----------|--------|
| `Alpaca` | US stocks + crypto |
| `AngelOne` | India NSE/BSE equity, F&O, commodity |
| `Dhan` | India NSE/BSE |
| `Fyers` | India NSE/BSE equity, F&O |
| `Upstox` | India NSE/BSE V3 API |
| `Zerodha` | India NSE/BSE (Kite Connect) |
| `Tinkoff` | Russia MOEX stocks, bonds, ETFs, futures |
| `OANDA` | Forex spot |

### Data Providers — V5 Trait Compliant (13 connectors)

All implement ExchangeIdentity + MarketData + Trading/Account/Positions (all returning UnsupportedOperation).

| Connector | Domain |
|-----------|--------|
| `Polygon` | US stocks/crypto |
| `Finnhub` | US stocks/forex/crypto news |
| `Tiingo` | US stocks/forex/crypto |
| `TwelveData` | Global stocks/forex/crypto |
| `JQuants` | Japan stocks (JPX/TSE) |
| `KRX` | Korea stocks (KOSPI/KOSDAQ) |
| `MOEX` | Russia stocks (MOEX ISS) |
| `AlphaVantage` (forex) | Forex/stocks/crypto |
| `Dukascopy` | Forex tick data (binary .bi5) |
| `Yahoo Finance` | Global stocks/ETFs |
| `CryptoCompare` | Crypto aggregator |
| `Coinglass` | Crypto derivatives analytics |
| `FRED` | US Federal Reserve economic data |

### Partial / Special Cases (2 connectors)

| Connector | Note |
|-----------|------|
| `DefiLlama` | Implements ExchangeIdentity + MarketData only; no Trading/Account/Positions trait stubs |
| `Interactive Brokers` | Implements ExchangeIdentity + MarketData only; has `get_positions` / `get_account_summary` as raw methods but Trading/Account/Positions traits NOT implemented |

### Stub — Not Functional (1 connector)

| Connector | Reason |
|-----------|--------|
| `Futu` | TCP + Protocol Buffers architecture; HTTP REST stub with all methods returning UnsupportedOperation |

### Onchain Data (3 connectors)

| Connector | Class | Note |
|-----------|-------|------|
| `Bitquery` | DATA ONLY (V5 compliant) | GraphQL blockchain analytics |
| `WhaleAlert` | DATA ONLY (V5 compliant) | Whale transaction tracking |
| `Etherscan` | RAW FEED | No V5 traits; Ethereum explorer only |

### Raw Intelligence Feeds (79 connectors)

No V5 exchange traits implemented. Domain-specific methods only. Grouped by category:

| Category | Count | Providers |
|----------|-------|-----------|
| Academic | 2 | arXiv, Semantic Scholar |
| Aviation | 4 | ADS-B Exchange, AviationStack, OpenSky, Wingbits |
| C2/Threat | 1 | C2 Intel Feeds |
| Conflict | 5 | ACLED, GDELT, ReliefWeb, UCDP, UNHCR |
| Corporate | 3 | GLEIF, OpenCorporates, UK Companies House |
| Crypto (raw) | 1 | CoinGecko |
| Cybersecurity | 9 | AbuseIPDB, AlienVault OTX, Censys, Cloudflare Radar, NVD, RIPE NCC, Shodan, URLhaus, VirusTotal |
| Demographics | 4 | UN OCHA, UN Population, WHO, Wikipedia |
| Economic | 11 | BIS, BoE, Bundesbank, CBR, DB.nomics, ECB, ECOS, Eurostat, IMF, OECD, World Bank |
| Environment | 9 | GDACS, Global Forest Watch, NASA EONET, NASA FIRMS, NOAA, NWS Alerts, OpenWeatherMap, OpenAQ, USGS Earthquake |
| FAA Status | 1 | FAA Status |
| Threat Tracking | 1 | Feodo Tracker |
| Financial (raw) | 4 | AlphaVantage (dup), Finnhub (dup), NewsAPI, OpenFIGI |
| Governance | 2 | EU Parliament, UK Parliament |
| Hacker News | 1 | Hacker News |
| Maritime | 4 | AIS, AISstream, IMF PortWatch, NGA Warnings |
| Prediction Markets | 1 | PredictIt |
| RSS/News | 1 | RSS Proxy |
| Sanctions | 3 | Interpol, OFAC, OpenSanctions |
| Space | 5 | Launch Library, NASA, Sentinel Hub, Space-Track, SpaceX |
| Trade/Procurement | 2 | UN Comtrade, EU TED |
| US Government | 9 | BEA, BLS, US Census, US Congress, EIA, FBI Crime, SAM.gov, SEC EDGAR, USASpending |

**Total raw intelligence feed connectors: 83**

---

## Capability Matrix — Trading Methods Detail

For FULL TRADING brokers, method-level breakdown:

| Method | Alpaca | AngelOne | Dhan | Fyers | Upstox | Zerodha | Tinkoff | OANDA |
|--------|--------|----------|------|-------|--------|---------|---------|-------|
| `place_order` (market) | YES | YES | YES | YES | YES | YES | YES | YES |
| `place_order` (limit) | YES | YES | YES | YES | YES | YES | YES | YES |
| `cancel_order` | YES | YES | YES | YES | YES | YES | YES | YES |
| `get_order` | YES | YES | YES | YES | YES | YES | YES | YES |
| `get_open_orders` | YES | YES | YES | YES | YES | YES | YES | YES |
| `get_order_history` | NO | NO | NO | NO | NO | NO | NO | NO |
| `get_balance` | YES | YES | YES | YES | YES | YES | YES | YES |
| `get_account_info` | YES | YES | YES | YES | YES | YES | YES | YES |
| `get_fees` | NO | NO | NO | NO | NO | NO | NO | NO |
| `get_positions` | YES | YES | YES | YES | YES | YES | YES | YES |
| `get_funding_rate` | NO | NO | NO | NO | NO | NO | NO | NO |
| `modify_position` | NO (leverage) | NO (leverage) | NO | NO | NO | NO | NO | NO (leverage = acct-level) |
| `get_klines` | YES | YES | YES | YES | YES | NO (needs instrument_token) | YES | YES |
| testnet support | YES (paper) | NO | YES | NO | NO | NO | YES (sandbox) | YES (practice) |

---

## Notes on Architecture

1. **V5 Trait Hierarchy**: All V5-compliant connectors implement `ExchangeIdentity` + `MarketData` + `Trading` + `Account` + `Positions`. Data-only connectors implement all traits but return `ExchangeError::UnsupportedOperation` for all trading/account/position methods.

2. **Raw Feed Pattern**: Many intelligence feeds (83 connectors) bypass the V5 trait system entirely. They only use `ExchangeError`/`ExchangeResult` for error handling but expose no standard interface.

3. **ExchangeId Coverage**: Only connectors with an `ExchangeId` variant are registered in the V5 exchange identity system: `ExchangeId::Coinglass`, `ExchangeId::Fred`, `ExchangeId::Bitquery`, `ExchangeId::WhaleAlert`, `ExchangeId::DefiLlama`, `ExchangeId::Ib`, plus all the stock/forex/aggregator providers.

4. **`get_order_history` and `get_fees` universal gap**: No non-crypto connector implements these two methods. All return UnsupportedOperation.

5. **`get_funding_rate` and `modify_position`**: Only relevant for futures/derivatives. All non-crypto connectors return UnsupportedOperation for these (even brokers that support futures like Tinkoff and AngelOne).

6. **Indian broker pattern**: All 5 Indian brokers use JWT Bearer tokens, implement FULL TRADING, but none support `get_order_history`, `get_fees`, `get_funding_rate`, or `modify_position`.

7. **Zerodha exception**: Unlike the other Indian brokers, `get_klines` returns UnsupportedOperation because it requires an `instrument_token` (not a symbol string) which must be resolved via a separate instrument master download.
