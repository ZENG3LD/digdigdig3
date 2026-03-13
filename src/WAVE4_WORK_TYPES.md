# WAVE4 Gap Analysis — Regrouped by Work Type

**Source:** WAVE4_GAPS_BATCH1 through BATCH28_30 + WAVE4_SWAP_DEX_REPORT + WAVE4_CONSOLIDATED_SUMMARY
**Date:** 2026-03-13
**Connectors analyzed:** ~90 across CEX, DEX/Swap, Onchain, Stocks, Forex, Aggregators, Intel Feeds, Economic, Cyber, Humanitarian, Space, Aviation, Maritime, Sanctions, Governance, Demographics

---

## Type A: URL/Config Fixes

Simple bugs — wrong URL paths, wrong HTTP methods, stale base URLs, wrong API versions.
These require only editing `endpoints.rs` or base URL constants. No trait or type changes.

| Connector | What's Wrong | What It Should Be |
|-----------|-------------|-------------------|
| **Polygon** | `Dividends` enum path wrong | `/stocks/v1/dividends` → `/vX/reference/dividends` |
| **Polygon** | `Splits` enum path wrong | `/stocks/v1/splits` → `/vX/reference/splits` |
| **Polygon** | `FinancialRatios` enum path wrong | `/stocks/v1/financial-ratios` → `/vX/reference/financials` |
| **Binance Futures** | `FuturesAccount` uses stale API version | `/fapi/v2/account` → `/fapi/v3/account` |
| **UCDP** | `StateConflict` path wrong | `/stateconflict/{version}` → `/ucdpprioconflict/{version}` |
| **UCDP** | All version numbers stale | `24.1` hardcoded → current versions are `25.1` (yearly) / `26.0.1` (gedevents) |
| **AviationStack** | Base URL uses plain HTTP | `http://` → `https://` |
| **UN OCHA** | Stale API version in base URL | `/api/v1` → `/api/v2` |
| **ReliefWeb** | Stale API version in base URL | `/v1` → `/v2` |
| **Sentinel Hub** | Wrong catalog path | `/api/v1/catalog/search` → `/api/v1/catalog/1.0.0/` |
| **Launch Library 2** | Stale API version | v2.2.0 → v2.3.0 |
| **Launch Library 2** | Wrong path names | `/agency/` → `/agencies/`, `/space_station/` → `/spacestation/` |
| **Uniswap** | Trading API enum uses old/wrong paths | `/orders`, `/swaps`, `/check_approval`, `/swappable_tokens` do not match documented API; should be under `/swapping/...` and `/liquidity_provisioning/...` |
| **AbuseIPDB** | `Categories` enum is a false entry | AbuseIPDB does not expose `/categories` REST endpoint; category IDs are static constants — remove the enum variant |
| **IMF PortWatch** | Unverified paths | `/portwatch/v1/...` is inferred — actual ArcGIS REST paths may differ; needs verification before use |

---

## Type B: Parser/Auth Fixes

Auth mechanism broken, parser not handling response format, timestamp parsing wrong, or existing connector non-functional due to structural issue.

| Connector | What's Broken | What Needs to Change |
|-----------|--------------|---------------------|
| **OpenSky** | Auth migrated from Basic (username/password) to OAuth2 client credentials flow — existing connector is **non-functional** | Replace `Authorization: Basic base64(user:pass)` with OAuth2 `client_credentials` flow: `POST /oauth2/token` with client_id + client_secret, then use bearer token |
| **UN Population** | Core data endpoint entirely missing — connector can list indicators/locations but **cannot fetch any actual data** | Add core `GET /data/indicators/{indicatorId}/locations/{locId}` endpoint; also add auth token request flow for accessing restricted indicators |
| **Bitquery** | WS GraphQL subscription queries written but not wired to `WebSocketConnector` trait — partial blocker for real-time data | Wire existing `_build_blocks_subscription` and `_build_dex_trades_subscription` queries to the `WebSocketConnector` impl using `graphql-ws` subprotocol over tokio-tungstenite |
| **Space-Track** | Architectural bug — all query predicates hardcoded in enum variant URL paths | Refactor to a flexible query builder pattern so new object classes can be queried without adding new enum variants each time |
| **Paradex** | JWT auth flow accepts pre-obtained JWT via `api_key` field, but auto-refresh on expiry (every 5 min) is not implemented | JWT auto-refresh needs StarkNet ECDSA signing via `starknet-rs` (see Type F); current connector works manually but breaks on long-running sessions |

---

## Type C: New Endpoints (Same Traits)

Endpoints that fit within existing traits and existing type definitions. Adding these requires only new enum variants in `endpoints.rs` and wiring in `connector.rs`. No trait or type changes needed.

### C1 — CEX Market Data Extensions

| Connector | Missing Endpoint | Path | Maps To |
|-----------|-----------------|------|---------|
| Binance Spot | Recent public trades | `GET /api/v3/trades` | `MarketData::get_price` / new `SpotRecentTrades` |
| Binance Spot | Historical trades | `GET /api/v3/historicalTrades` | Same |
| Binance Spot | Average price | `GET /api/v3/avgPrice` | `MarketData::get_price` |
| Binance Spot | Book ticker | `GET /api/v3/ticker/bookTicker` | `MarketData::get_price` |
| Binance Futures | Recent trades | `GET /fapi/v1/trades` | Market data |
| Binance Futures | Open interest (current) | `GET /fapi/v1/openInterest` | `Positions::get_funding_rate` adjacent |
| Binance Futures | Open interest (historical) | `GET /futures/data/openInterestHist` | Same |
| Binance Futures | Premium index (mark + funding) | `GET /fapi/v1/premiumIndex` | `Positions::get_funding_rate` |
| Binance Futures | Long/short ratio (4 variants) | `GET /futures/data/topLongShortAccountRatio` etc. | New sentiment data |
| Binance Futures | Income history (closed PnL) | `GET /fapi/v1/income` | `Account` adjacent |
| Bybit | Open interest | `GET /v5/market/open-interest` | Derivatives market data |
| Bybit | Closed PnL | `GET /v5/position/closed-pnl` | Account / positions |
| Bybit | Long/short ratio | `GET /v5/market/account-ratio` | Sentiment data |
| Bybit | Mark/index/premium klines (3 variants) | `/v5/market/mark-price-kline` etc. | `MarketData::get_klines` adjacent |
| OKX | Open interest | `/api/v5/public/open-interest` | Derivatives |
| OKX | Long/short ratio | `/api/v5/rubik/stat/contracts/long-short-account-ratio` | Sentiment |
| OKX | Fill history (spot) | `GET /api/v5/trade/fills` | `Trading::get_order_history` |
| OKX | Fill history (futures) | `GET /api/v5/trade/fills-history` | Same |
| KuCoin Spot | Spot fills | `GET /api/v1/fills` | `Trading::get_order_history` |
| KuCoin Spot | Recent trades | `GET /api/v1/market/histories` | `MarketData` |
| KuCoin Spot | Full orderbook (L2) | `GET /api/v1/market/orderbook/level2` | `MarketData::get_orderbook` |
| KuCoin Futures | Fill history | `GET /api/v1/fills` | `Trading::get_order_history` |
| KuCoin Futures | Trade history (public) | `GET /api/v1/trade/history` | `MarketData` |
| KuCoin Futures | Open interest | via risk-limit endpoint | Derivatives |
| KuCoin Futures | Funding rate history | `GET /api/v1/contract/funding-rates` | `Positions::get_funding_rate` adjacent |
| KuCoin Futures | Mark price | `GET /api/v1/mark-price/{symbol}/current` | Derivatives |
| KuCoin Futures | Index price | `GET /api/v1/index-price/{symbol}/current` | Derivatives |
| KuCoin Futures | Premium index | `GET /api/v1/premium-index/{symbol}/current` | Derivatives |
| Kraken | Trade history | `POST /0/private/TradesHistory` | `Trading::get_order_history` |
| Coinbase | Fill history | `GET /api/v3/brokerage/orders/historical/fills` | `Trading::get_order_history` |
| Bitfinex | Stats1 endpoint (OI, longs/shorts) | `/v2/stats1/...` | Sentiment / derivatives |
| HTX | Match results (fills by order) | `GET /v1/order/orders/{id}/matchresults` | `Trading::get_order_history` |
| HTX | Open interest | `GET /linear-swap-api/v1/swap-open-interest` | Derivatives |
| HTX | Funding rate history | `GET /linear-swap-api/v3/swap-funding-rate-history` | Derivatives |
| HTX | Mark/index price | `/linear-swap-ex/market/index` + mark klines | Derivatives |
| Gate.io Spot | Spot trades (public) | `GET /spot/trades` | `MarketData` |
| Gate.io Spot | My trades | `GET /spot/my_trades` | `Trading::get_order_history` |
| Gate.io Futures | Trade fills | `GET /futures/usdt/my_trades` | Same |
| Gate.io Futures | Public trades | `GET /futures/usdt/trades` | `MarketData` |
| Gate.io Futures | Open interest | futures OI endpoint | Derivatives |
| Gate.io Futures | Funding rate history | funding history endpoint | Derivatives |
| MEXC | Recent trades | `GET /api/v3/trades` | `MarketData` |
| MEXC | My trades | `GET /api/v3/myTrades` | `Trading::get_order_history` |
| MEXC Futures | Mark price | mark price endpoint | Derivatives |
| Bitget Spot | Recent fills (public) | `GET /api/v2/spot/market/fills` | `MarketData` |
| Bitget Spot | Historical candles | `GET /api/v2/spot/market/history-candles` | `MarketData::get_klines` |
| Bitget Futures | Fill history | `GET /api/v2/mix/order/fill-history` | `Trading::get_order_history` |
| Bitget Futures | Open interest | `GET /api/v2/mix/market/open-interest` | Derivatives |
| Bitget Futures | Funding rate history | `GET /api/v2/mix/market/history-fund-rate` | Derivatives |
| Bitget Futures | Mark/symbol price | `GET /api/v2/mix/market/symbol-price` | Derivatives |
| BingX Spot | My trades | `GET /openApi/spot/v1/trade/myTrades` | `Trading::get_order_history` |
| BingX Swap | All fill orders | `GET /openApi/swap/v2/trade/allFillOrders` | Same |
| BingX Swap | Open interest | `GET /openApi/swap/v2/quote/openInterest` | Derivatives |
| BingX Swap | Funding rate history | `GET /openApi/swap/v2/quote/fundingRateHistory` | Derivatives |
| BingX Swap | Premium index | `GET /openApi/swap/v2/quote/premiumIndex` | Derivatives |
| Phemex | Trade history | `GET /exchange/order/v2/tradingList` | `Trading::get_order_history` |
| Phemex | Open interest | OI endpoint | Derivatives |
| Phemex | Funding rate history | funding history endpoint | Derivatives |
| Deribit | Funding rate history | `public/get_funding_rate_history` | `Positions::get_funding_rate` adjacent |
| Deribit | Funding rate value | `public/get_funding_rate_value` | Same |
| Deribit | Index price | `public/get_index_price` | Derivatives |
| Deribit | Historical volatility | `public/get_historical_volatility` | Analytics |
| Deribit | Mark price history | `public/get_mark_price_history` | Derivatives |
| Deribit | Order history by currency | `private/get_order_history_by_currency` | `Trading::get_order_history` |
| Deribit | Order history by instrument | `private/get_order_history_by_instrument` | Same |
| Deribit | User trades by currency + time | `private/get_user_trades_by_currency_and_time` | Same |
| Deribit | Trigger order history | `private/get_trigger_order_history` | Same |
| Upbit | Closed orders | `GET /v1/orders/closed` | `Trading::get_order_history` |
| Dhan | Trade history | `GET /v2/trades` | Same |

### C2 — CEX Batch Amend (New Endpoint, Extends BatchOrders Trait)

| Connector | Missing Endpoint | Path |
|-----------|-----------------|------|
| Binance Futures | Batch amend orders | `PATCH /fapi/v1/batchOrders` |
| Bybit | Batch amend orders | `POST /v5/order/amend-batch` |
| OKX | Batch amend orders | `POST /api/v5/trade/amend-batch-orders` |
| Gate.io Futures | Batch amend orders | `POST /futures/usdt/batch_amend_orders` |
| Hyperliquid | Batch modify | `batchModify` action on `/exchange` |

### C3 — CEX Account / Wallet Additions (fit Account/CustodialFunds traits)

| Connector | Missing Endpoints |
|-----------|-----------------|
| Binance | Listen key keepalive (`PUT /api/v3/userDataStream`) + close (`DELETE`) |
| Upbit | `GET /v1/orders/chance`, `GET /v1/orders/open`, wallet status `GET /v1/status/wallet` |
| Upbit | KRW withdrawal `POST /v1/withdraws/krw`, cancel withdrawal `DELETE /v1/withdraws/uuid` |
| Bitstamp | Order history per pair `POST /api/v2/order_history/{pair}/`, instant buy/sell |
| Bitstamp | Sub-account transfer endpoints (2), bank withdrawal endpoint |
| KuCoin | Transfer quotas, flex transfer, withdrawal cancellation, withdrawal quotas |
| Bybit | Exchange record, delivery record, coin balance variants |
| HTX | Inter-account transfer `POST /v1/account/transfer` |
| Gate.io | Sub-account balance, sub-account transfers |
| Bithumb | Single order lookup `POST /spot/singleOrder`, asset list `POST /spot/assetList` |

### C4 — DEX/Onchain Market Data Extensions (fit existing MarketData)

| Connector | Missing Endpoint | Notes |
|-----------|-----------------|-------|
| dYdX | Transfers between subaccounts `GET /v4/transfers/between` | Indexer REST |
| dYdX | Parent subaccount asset positions `GET /v4/assetPositions/parentSubaccountNumber` | Indexer REST |
| dYdX | Parent transfers `GET /v4/transfers/parentSubaccountNumber` | Indexer REST |
| dYdX | Vault endpoints (3) | MegaVault PnL, positions, all vaults PnL |
| dYdX | Affiliate metadata (2 endpoints) | Analytics |
| GMX | GLV APY `GET /glvs/apy` | Existing pattern |
| GMX | Stats endpoints (UI fees, position stats, fee metrics, volumes, account stats) | Analytics |
| GMX | Botanix chain URL | Third chain, same REST pattern |
| Lighter | Funding rates `GET /api/v1/funding-rates` | Verify vs `/fundings` |
| Lighter | Account limits `GET /api/v1/accountLimits` | Account metadata |
| Lighter | Account metadata `GET /api/v1/accountMetadata` | Same |
| Lighter | Position funding `GET /api/v1/positionFunding` | Per-position funding |
| Lighter | Liquidations `GET /api/v1/liquidations` | Account history |
| Lighter | Exchange metrics `GET /api/v1/exchangeMetrics` | Analytics |
| Lighter | Withdrawal delays `GET /api/v1/withdrawalDelays` | Custodial |
| Raydium | Chain time `GET /main/chain-time` | Utility |
| Raydium | Platform info `GET /main/info` | Analytics (TVL, 24h vol) |
| Raydium | Pool price history `GET /pools/line/price` | OHLCV for pools |
| Raydium | Pool liquidity history `GET /pools/line/liquidity` | Same |
| Raydium | Pool stats `GET /pools/info/stats` | Aggregate TVL/volume |
| Raydium | CLMM configs `GET /clmm/configs` | Pool metadata |
| Raydium | CPMM configs `GET /cpmm/configs` | Pool metadata |
| Raydium | Farm ownership `GET /farms/info/mine` | Account data |
| Raydium | Portfolio positions/farms (2) | Account aggregation |
| Jupiter | Ultra Swap API (5 endpoints) | Replaces Metis flow |
| Jupiter | Tokens metadata `GET /tokens/v2` | Token detail |

### C5 — Stock/Broker Connector Additions

| Connector | Missing Endpoints | Notes |
|-----------|-----------------|-------|
| Alpaca | Watchlist CRUD (6 endpoints) | Account management |
| Alpaca | Close all positions `DELETE /v2/positions` | Trading |
| Alpaca | Close position `DELETE /v2/positions/{symbol}` | Trading |
| Alpaca | Account configurations (2) | Account settings |
| Alpaca | Single-symbol bars/trades/quotes (3) | Market data variants |
| Alpaca | Options chain `GET /v1beta1/options/chain` | Options |
| Alpaca | Options contract detail | Options |
| Alpaca | Most-actives screener | Market data |
| Alpaca | Auctions data | Market data |
| Finnhub | ETF endpoints (4: holdings, profile, country, sector) | Fund data |
| Finnhub | Mutual fund endpoints (4) | Fund data |
| Finnhub | Bond price/profile/yield-curve (3) | Fixed income |
| Finnhub | IPO calendar, earnings surprise, social sentiment, transcripts, crypto profile | Analytics |
| Polygon | Options data surface (8 endpoints) | Options |
| Polygon | Indices snapshots/aggregates (2) | Market data |
| Polygon | Forex data (3) | Market data |
| Polygon | Crypto data (3) | Market data |
| Polygon | Reference conditions/exchanges/events | Reference |
| TwelveData | Real-time price endpoint | Market data |
| TwelveData | Complex data batch endpoint | Market data |
| TwelveData | Mutual funds list, bonds list | Reference |
| Tiingo | List daily tickers, list IEX all, list forex tickers, forex metadata (4) | Reference |
| Zerodha | GTT order list/delete/modify (3) | Trading |
| Zerodha | Instruments master file download | Reference |
| Zerodha | Portfolio margins, basket orders | Trading |
| Dhan | Options chain endpoint | Options |
| Dhan | Kill switch | Account |
| Fyers | Position conversion, basket orders, net position (3) | Trading |
| Upstox | Market quotes for multiple instruments | Market data |
| Upstox | Historical data v3 | Market data |

### C6 — Economic Feed Additions (fit MarketData / custom trait methods)

| Connector | Missing Endpoints | Notes |
|-----------|-----------------|-------|
| BIS | Category scheme (4 minor SDMX structural endpoints) | Structure |
| Bundesbank | Category/categorisation/delta-fetch (3 SDMX) | Structure |
| ECB | Category scheme, content constraint, agency scheme, etc. (4) | Structure |
| Eurostat | DCAT-AP, RSS, metabase bulk (4 catalogue endpoints) | Structure |
| CBR (Russia) | RUONIA rate, deposit rates, refinancing rate history (3) | Economic data |
| World Bank | Multi-country batch fetch, sub-national data | Economic data |
| IMF DataMapper | WEO forecast API (4 endpoints: indicators, countries, regions, data) | Macro forecasts |
| ReliefWeb | Blog, book, references (3) | Content |
| ReliefWeb | Base URL `/v1` → `/v2` (A-type fix also applicable) | Bug |

### C7 — Intel/Governance/Cyber Feed Additions

| Connector | Missing Endpoints | Notes |
|-----------|-----------------|-------|
| ACLED | CAST forecasts `GET /cast/`, deleted records `GET /deleted/` (2) | Conflict intel |
| GDELT | `DocMode` enum missing `TimelineVolInfo` + `ToneChart` variants | Query params |
| AbuseIPDB | Reports `GET /reports` (paginated IP reports) | Cyber intel |
| AlienVault OTX | Pulse CRUD (5: get by ID, create, user pulses, search, my pulses) | CTI |
| AlienVault OTX | IPv4/Domain/Hostname/File sections (per-section indicator detail) | CTI |
| AlienVault OTX | IPv6 indicators, CVE indicators, NIDS rule indicators | CTI |
| Censys | Host events, host names, view certificate, hosts by cert, aggregate certs, account info (6) | Cyber intel |
| INTERPOL | Yellow notice detail + images (2), UN persons/entities variants (4) | Sanctions/intel |
| OFAC | Bulk screening `POST /screen/bulk` | Sanctions |
| OpenSanctions | Entity adjacency `GET /entities/{id}/adjacent`, statements, reconcile API (3) | Sanctions |
| OpenCorporates | Control statements search, officer lookup, filing detail, statements subsystem, industry codes | Corporate |
| GLEIF | Relationship records (Level 2 ownership), BIC maps, reporting exceptions | Corporate |
| EU Parliament | Vote results (roll-call), parliamentary questions, activities, adopted texts (4) | Governance |
| UK Parliament | RegisteredInterests, amendment tracking, parties composition, posts (4) | Governance |
| UK Companies House | 6 search variants, disqualifications, charges/filings/PSC detail | Corporate |
| Space-Track | CDM public (collision data), gp_history (historical TLEs), boxscore, satcat_change, OMM (5) | Space |
| NASA | DONKI CMEAnalysis, notifications, Mars Rover Photos, Earth Imagery, NEO Browse (11+ endpoints) | Space |
| AIS (Datalastic) | Vessel inradius (geospatial), vessel bulk (batch), vessel pro estimated position (3) | Maritime |
| NGA Warnings | ASAM piracy reports, MODU positions, World Port Index (3) | Maritime |
| AviationStack | Timetable, flights future, airplanes endpoints (3, beyond the HTTP bug fix) | Aviation |
| Sentinel Hub | Batch processing, OGC WMS/WCS/WFS (4, beyond catalog path fix) | Earth obs |
| Launch Library 2 | 6 missing endpoints (launcher, pad, location, expedition, docking, payload) | Space |
| Wikipedia | 4 analytics subsystems (unique-devices, edits, editors, registered-users) | Analytics |
| UN OCHA | Conflict events (ACLED), national risk (INFORM), food prices (WFP) — v2 endpoints (3) | Humanitarian |
| UNHCR | Asylum applications endpoint | Humanitarian |

---

## Type D: Trait/Enum Extensions

Things that do NOT fit existing trait methods or enum variants. New enum variants, new trait methods, or new traits are needed.

### D1 — New Enum Variants Needed

#### D1.1 — New `OrderType` Variants

Current `OrderType` enum does not cover these order types present in the gap reports:

| New Variant | Exchanges | Description |
|-------------|----------|-------------|
| `Oto` (One-Triggers-Other) | Binance Spot | Entry order that triggers a secondary order on fill |
| `Opo` (One-Pending-Other) | Binance Spot | Order that holds another order pending |
| `Opoco` (One-Pending-OCO) | Binance Spot | Entry + pending OCO |
| `PlanOrder` / `ConditionalStop` | KuCoin, Bitget | Stop trigger/plan orders (spot + futures) — distinct from StopMarket in that they are managed server-side with conditional activation |
| `DcaOrder` | Jupiter Recurring API, BingX | Time-based or price-based recurring/DCA order |

```rust
// Proposed additions to OrderType enum:
Oto {
    entry_price: Option<Price>,
    secondary_order: Box<OrderType>,
},
ConditionalPlan {
    trigger_price: Price,
    trigger_direction: TriggerDirection,  // Above / Below
    order_after_trigger: Box<OrderType>,
},
DcaRecurring {
    interval_seconds: u64,
    total_cycles: Option<u32>,
    price_limit: Option<Price>,
},
```

#### D1.2 — New `AccountType` Variants

Current `AccountType`: `Spot`, `Margin`, `FuturesCross`, `FuturesIsolated`.

Missing variants needed by gap connectors:

| New Variant | Needed By | Description |
|-------------|----------|-------------|
| `Earn` / `Savings` | Binance, KuCoin, OKX, Bitget, Gate.io | Yield-bearing account type for Simple Earn / Staking / Flexible savings |
| `Loan` / `Lending` | Bitfinex, Binance, HTX, KuCoin | P2P lending / borrowing account |
| `Options` | Deribit, OKX, Binance | Options-specific account context |
| `Perpetual` | Deribit | Separate from `FuturesCross` for Deribit's model |
| `Convert` | Binance, OKX, KuCoin | Dust-conversion or instant-swap sub-account |

```rust
// Proposed additions to AccountType enum:
Earn,
Lending,
Options,
Convert,
```

#### D1.3 — New `CancelScope` Variants

Current scopes: `Single`, `Batch`, `All`, `BySymbol`.

| New Variant | Needed By | Description |
|-------------|----------|-------------|
| `ByLabel` | Deribit | Cancel all orders with a specific client label |
| `ByCurrencyKind` | Deribit | Cancel by currency pair + instrument kind (future/option) |
| `ScheduledAt` | Hyperliquid | Schedule cancel at specific future timestamp |

#### D1.4 — New `PositionModification` Variants

| New Variant | Needed By | Description |
|-------------|----------|-------------|
| `SwitchPositionMode` | Binance Futures, Bybit | Toggle OneWay / Hedge mode |
| `MovePositions` | Deribit | Move positions between sub-accounts |
| `SetPositionMode` | Multiple | Change position direction mode |

### D2 — New Methods on Existing Traits

#### D2.1 — New method: `Trading::get_trades` (fill/execution history)

Currently `get_order_history` returns `Vec<Order>` — but orders and fills are distinct. 30+ connectors have a separate fills endpoint that returns trade executions, not order states.

```rust
// Proposed addition to Trading trait:
async fn get_user_trades(
    &self,
    filter: UserTradeFilter,
    account_type: AccountType,
) -> ExchangeResult<Vec<UserTrade>>;
```

`UserTrade` already exists in `trading.rs`. `UserTradeFilter` struct needed:

```rust
pub struct UserTradeFilter {
    pub symbol: Option<Symbol>,
    pub order_id: Option<String>,
    pub start_time: Option<Timestamp>,
    pub end_time: Option<Timestamp>,
    pub limit: Option<u32>,
}
```

**Affected connectors (~25):** Binance, Bybit, OKX, KuCoin, Kraken, Coinbase, HTX, Gate.io, MEXC, Bitget, BingX, Phemex, Upbit, Dhan, AngelOne, Alpaca, Paradex, Deribit, Hyperliquid, Lighter, dYdX, KRX, Bithumb, Bitstamp, Gate.io

#### D2.2 — New method: `Positions::get_open_interest`

Required for 10 futures connectors. Currently the `Positions` trait has no OI method.

```rust
// Proposed addition to Positions trait:
async fn get_open_interest(
    &self,
    symbol: &str,
    account_type: AccountType,
) -> ExchangeResult<OpenInterest>;

pub struct OpenInterest {
    pub symbol: String,
    pub open_interest: f64,
    pub open_interest_value: Option<f64>,
    pub timestamp: Timestamp,
}
```

**Affected connectors (~10):** Binance, Bybit, OKX, KuCoin, HTX, Bitget, BingX, Gate.io, Deribit, Phemex

#### D2.3 — New method: `Positions::get_funding_rate_history`

Currently `get_funding_rate` returns only the current rate. History is a distinct endpoint on 8+ connectors.

```rust
// Proposed addition to Positions trait (or optional trait):
async fn get_funding_rate_history(
    &self,
    symbol: &str,
    start_time: Option<Timestamp>,
    end_time: Option<Timestamp>,
    limit: Option<u32>,
) -> ExchangeResult<Vec<FundingRate>>;
```

**Affected connectors (~8):** KuCoin Futures, HTX Futures, Bitget, BingX, Gate.io, Deribit, Phemex, Lighter

#### D2.4 — New method: `Positions::get_mark_price`

Required for accurate PnL and liquidation calculations.

```rust
async fn get_mark_price(
    &self,
    symbol: &str,
) -> ExchangeResult<MarkPrice>;

pub struct MarkPrice {
    pub symbol: String,
    pub mark_price: Price,
    pub index_price: Option<Price>,
    pub funding_rate: Option<f64>,
    pub timestamp: Timestamp,
}
```

**Affected connectors (~8):** KuCoin Futures, Binance Futures, Bybit, HTX, Bitget, BingX, Gate.io, MEXC

#### D2.5 — New method: `Positions::get_closed_pnl`

Required for performance reporting and tax accounting.

```rust
async fn get_closed_pnl(
    &self,
    symbol: Option<&str>,
    start_time: Option<Timestamp>,
    end_time: Option<Timestamp>,
    limit: Option<u32>,
) -> ExchangeResult<Vec<ClosedPnlRecord>>;

pub struct ClosedPnlRecord {
    pub symbol: String,
    pub side: PositionSide,
    pub closed_size: Quantity,
    pub avg_entry_price: Price,
    pub avg_exit_price: Price,
    pub closed_pnl: f64,
    pub timestamp: Timestamp,
}
```

**Affected connectors (~5):** Bybit, Binance Futures, KuCoin Futures, Gate.io Futures, OKX

#### D2.6 — New method: `MarketData::get_long_short_ratio`

Required for derivatives sentiment analysis.

```rust
async fn get_long_short_ratio(
    &self,
    symbol: &str,
    period: &str,  // "5m", "1h", "4h", "1d"
) -> ExchangeResult<LongShortRatio>;

pub struct LongShortRatio {
    pub symbol: String,
    pub long_ratio: f64,
    pub short_ratio: f64,
    pub timestamp: Timestamp,
}
```

**Affected connectors (~4):** Binance Futures, Bybit, Bitfinex, OKX

### D3 — New Optional Traits Entirely

These capability domains are entirely absent from the trait system. Each should be a new optional trait in `operations.rs` (or a new `specializations.rs` file).

#### D3.1 — `MarginTrading` trait

```rust
#[async_trait]
pub trait MarginTrading: Account {
    async fn borrow(&self, req: MarginBorrowRequest) -> ExchangeResult<MarginBorrowResponse>;
    async fn repay(&self, req: MarginRepayRequest) -> ExchangeResult<MarginRepayResponse>;
    async fn get_margin_interest(&self, filter: MarginInterestFilter) -> ExchangeResult<Vec<MarginInterestRecord>>;
    async fn get_margin_account(&self, account_type: AccountType) -> ExchangeResult<MarginAccountInfo>;
    async fn get_margin_orders(&self, filter: OrderHistoryFilter) -> ExchangeResult<Vec<Order>>;
}
```

**Coverage:** ~8 connectors: Binance, HTX, KuCoin, Bitget, Gate.io, Bybit, Bitfinex, Phemex
**Estimated missing endpoints total:** ~80

#### D3.2 — `EarnStaking` trait

```rust
#[async_trait]
pub trait EarnStaking: Account {
    async fn get_earn_products(&self, asset: Option<&str>) -> ExchangeResult<Vec<EarnProduct>>;
    async fn subscribe_earn(&self, req: EarnSubscribeRequest) -> ExchangeResult<EarnPosition>;
    async fn redeem_earn(&self, req: EarnRedeemRequest) -> ExchangeResult<EarnRedeemResponse>;
    async fn get_earn_positions(&self) -> ExchangeResult<Vec<EarnPosition>>;
    async fn get_earn_history(&self, filter: EarnHistoryFilter) -> ExchangeResult<Vec<EarnRecord>>;
}
```

**Coverage:** ~5 connectors: Binance, KuCoin, Bitget, OKX, Gate.io
**Estimated missing endpoints total:** ~35

#### D3.3 — `ConvertSwap` trait

```rust
#[async_trait]
pub trait ConvertSwap: Account {
    async fn get_convert_quote(&self, req: ConvertQuoteRequest) -> ExchangeResult<ConvertQuote>;
    async fn accept_convert_quote(&self, quote_id: &str) -> ExchangeResult<ConvertOrder>;
    async fn get_convert_history(&self, filter: ConvertHistoryFilter) -> ExchangeResult<Vec<ConvertRecord>>;
    async fn convert_dust(&self, assets: Vec<String>) -> ExchangeResult<ConvertDustResponse>;
}
```

**Coverage:** ~3 connectors: Binance (most complete), OKX, KuCoin
**Estimated missing endpoints total:** ~15

#### D3.4 — `CopyTrading` trait

```rust
#[async_trait]
pub trait CopyTrading: Trading {
    async fn get_lead_traders(&self, filter: LeadTraderFilter) -> ExchangeResult<Vec<LeadTrader>>;
    async fn follow_trader(&self, trader_id: &str) -> ExchangeResult<CopyFollowResponse>;
    async fn stop_following(&self, trader_id: &str) -> ExchangeResult<()>;
    async fn get_copy_positions(&self) -> ExchangeResult<Vec<CopyPosition>>;
    async fn get_copy_history(&self) -> ExchangeResult<Vec<CopyOrderRecord>>;
}
```

**Coverage:** ~3 connectors: Bitget (most complete, 15+ endpoints), BingX, OKX
**Estimated missing endpoints total:** ~25

#### D3.5 — `LiquidityProvider` trait (DEX/AMM-specific)

Required for Raydium and Uniswap LP position management. Entirely absent.

```rust
#[async_trait]
pub trait LiquidityProvider: Account {
    async fn create_position(&self, req: LpCreateRequest) -> ExchangeResult<LpPosition>;
    async fn add_liquidity(&self, req: LpAddRequest) -> ExchangeResult<LpPosition>;
    async fn remove_liquidity(&self, req: LpRemoveRequest) -> ExchangeResult<LpWithdrawResult>;
    async fn collect_fees(&self, position_id: &str) -> ExchangeResult<LpFeeCollection>;
    async fn get_lp_positions(&self, owner: &str) -> ExchangeResult<Vec<LpPosition>>;
}
```

**Coverage:** Raydium (CLMM + CPMM), Uniswap v3/v4

#### D3.6 — `VaultManager` trait (DEX vaults)

Required for HyperLiquid vaults, Paradex vaults, dYdX MegaVault, GMX GLV.

```rust
#[async_trait]
pub trait VaultManager: Account {
    async fn get_vaults(&self) -> ExchangeResult<Vec<VaultInfo>>;
    async fn get_vault_detail(&self, vault_id: &str) -> ExchangeResult<VaultDetail>;
    async fn deposit_vault(&self, req: VaultDepositRequest) -> ExchangeResult<VaultPosition>;
    async fn withdraw_vault(&self, req: VaultWithdrawRequest) -> ExchangeResult<VaultWithdrawResult>;
    async fn get_vault_history(&self) -> ExchangeResult<Vec<VaultHistoryRecord>>;
}
```

**Coverage:** HyperLiquid, Paradex, dYdX (MegaVault), GMX (GLV)

#### D3.7 — `StakingDelegation` trait

Required for HyperLiquid staking (HYPE delegation) and Vertex staking (VRTX).

```rust
#[async_trait]
pub trait StakingDelegation: Account {
    async fn delegate(&self, req: DelegateRequest) -> ExchangeResult<DelegationResult>;
    async fn undelegate(&self, req: DelegateRequest) -> ExchangeResult<DelegationResult>;
    async fn get_delegations(&self) -> ExchangeResult<Vec<Delegation>>;
    async fn get_staking_rewards(&self) -> ExchangeResult<Vec<StakingReward>>;
    async fn claim_rewards(&self) -> ExchangeResult<ClaimRewardResult>;
}
```

**Coverage:** HyperLiquid (HYPE), Vertex (VRTX), Jupiter (lending)

#### D3.8 — `BlockTradeOtc` trait

Required for Deribit and Paradex institutional/OTC flow.

```rust
#[async_trait]
pub trait BlockTradeOtc: Trading {
    async fn create_block_trade(&self, req: BlockTradeRequest) -> ExchangeResult<BlockTrade>;
    async fn verify_block_trade(&self, req: BlockTradeRequest) -> ExchangeResult<BlockTradeVerification>;
    async fn execute_block_trade(&self, trade_id: &str) -> ExchangeResult<BlockTradeResult>;
    async fn get_block_trades(&self) -> ExchangeResult<Vec<BlockTrade>>;
}
```

**Coverage:** Deribit, Paradex

#### D3.9 — `MarketMakerProtection` trait

Specific to Deribit's MMP (Market Maker Protection) system.

```rust
#[async_trait]
pub trait MarketMakerProtection: Trading {
    async fn get_mmp_config(&self) -> ExchangeResult<MmpConfig>;
    async fn set_mmp_config(&self, config: MmpConfig) -> ExchangeResult<()>;
    async fn get_mmp_status(&self) -> ExchangeResult<MmpStatus>;
    async fn reset_mmp(&self) -> ExchangeResult<()>;
    async fn mass_quote(&self, quotes: Vec<QuoteRequest>) -> ExchangeResult<Vec<QuoteResult>>;
}
```

**Coverage:** Deribit only

#### D3.10 — `TriggerOrders` trait (on-chain / DEX trigger orders)

Distinct from `StopMarket`/`StopLimit` in `OrderType` — these are server-managed conditional orders with their own lifecycle.

```rust
#[async_trait]
pub trait TriggerOrders: Trading {
    async fn place_trigger_order(&self, req: TriggerOrderRequest) -> ExchangeResult<TriggerOrder>;
    async fn cancel_trigger_order(&self, order_id: &str) -> ExchangeResult<()>;
    async fn get_trigger_orders(&self, symbol: Option<&str>) -> ExchangeResult<Vec<TriggerOrder>>;
    async fn get_trigger_order_history(&self, filter: OrderHistoryFilter) -> ExchangeResult<Vec<TriggerOrder>>;
}
```

**Coverage:** Jupiter (Trigger v2 API), Vertex (trigger_order execute action), Bithumb Futures

#### D3.11 — `PredictionMarket` trait

Required for Jupiter prediction markets and Polymarket.

```rust
#[async_trait]
pub trait PredictionMarket: MarketData {
    async fn get_prediction_events(&self) -> ExchangeResult<Vec<PredictionEvent>>;
    async fn get_event_orderbook(&self, event_id: &str) -> ExchangeResult<OrderBook>;
    async fn place_prediction_order(&self, req: PredictionOrderRequest) -> ExchangeResult<PredictionPosition>;
    async fn get_prediction_positions(&self) -> ExchangeResult<Vec<PredictionPosition>>;
    async fn close_prediction_position(&self, position_id: &str) -> ExchangeResult<()>;
    async fn claim_payout(&self, position_id: &str) -> ExchangeResult<ClaimResult>;
}
```

**Coverage:** Jupiter (prediction API), Polymarket

---

## Type E: Transport Layer

Connectors that need a different transport protocol than plain HTTP REST.

| Connector | Current Transport | Needed Transport | Operations Requiring It |
|-----------|------------------|-----------------|------------------------|
| **dYdX v4** | REST HTTPS (Indexer, read-only) | Cosmos gRPC (`tonic` + `prost`) over TLS | Order placement (`MsgPlaceOrder`), order cancellation (`MsgCancelOrder`), account nonces — via Node API `dydx-ops-rpc.kingnodes.com:443` |
| **Futu Securities** | TCP + Protocol Buffers (OpenD daemon) | Already using TCP+protobuf, but missing ~35 additional proto IDs | Push/streaming callbacks (proto push delivery), intraday tick data, broker queue Level 2, capital flow analytics, warrant filtering |
| **GMX** | REST HTTPS (read-only) | GraphQL over HTTPS (The Graph subgraph) | Historical positions, trades, liquidations, volume stats — only available via The Graph subgraph `gateway.thegraph.com` |
| **Raydium** | REST HTTPS | Solana gRPC (Yellowstone/Jito gRPC) | Real-time price feed — REST polling is 1-2s latency; true real-time requires Solana gRPC subscription (different from REST) |
| **Bitquery** | GraphQL over HTTPS (already) | GraphQL Subscriptions over WebSocket (`graphql-ws` subprotocol) | Real-time DEX trade streams, block streams — subscription queries written but not wired to WS layer |
| **Whale Alert** | REST HTTPS (already) | WebSocket (`tokio-tungstenite`) | Real-time whale alert stream at `wss://leviathan.whale-alert.io/ws` — URL stored, connector not implemented |
| **Uniswap** | Three transports (REST, GraphQL, ETH JSON-RPC) | Ethereum WebSocket (ETH WS JSON-RPC) | Real-time new blocks, event logs (`Swap` events), mempool monitoring — for low-latency DEX monitoring |
| **Jupiter (Perps)** | REST HTTPS | Solana RPC + Anchor IDL parsing | Jupiter perpetuals positions/pool state — no REST API exists, on-chain data via Solana account reads and Anchor IDL decoding |

### WebSocket Implementations Needed (full connectors, not just channels)

These connectors have a WS URL stored but no `WebSocketConnector` implementation at all:

| Connector | WS URL | Status |
|-----------|--------|--------|
| **Bitget** | `wss://ws.bitget.com/v2/ws/public` + private | No WS impl — entire streaming layer missing |
| **BingX** | Spot + swap WS URLs | No WS impl — entire streaming layer missing |
| **Bitstamp** | `wss://ws.bitstamp.net` | No WS impl |
| **Upbit** | `wss://api.upbit.com/websocket/v1` | No WS impl |
| **Angel One** | WebSocket market data stream | WS channels not typed |
| **Whale Alert** | `wss://leviathan.whale-alert.io/ws` | URL stored, not implemented |
| **Uniswap** | `wss://ethereum-rpc.publicnode.com` | URL stored, not implemented |

---

## Type F: External SDK/Crate Dependencies

Operations that need third-party signing or blockchain interaction crates. These are currently stubbed with `UnsupportedOperation`.

### F1 — `k256` (secp256k1 ECDSA)

**Version:** `0.13`

| Connector | Operations Enabled | Effort |
|-----------|-------------------|--------|
| **Lighter** | L2 order creation (`tx_type=14` L2CreateOrder) and cancellation (`tx_type=15` L2CancelOrder) via `POST /api/v1/sendTx`. REST API is already fully implemented for market data and read-only account ops. | LOW — one crate, REST complete |
| **HyperLiquid** | EIP-712 signing already partially present — the `ExchangeCredentials::EthereumWallet` type exists. If HyperLiquid uses ECDSA directly (not via ethers/alloy), `k256` handles signing. | LOW–MEDIUM |

### F2 — `starknet-rs` / `starknet-crypto`

**Version:** `0.7`

| Connector | Operations Enabled | Effort |
|-----------|-------------------|--------|
| **Paradex** | JWT auto-generation via `POST /v1/auth` with StarkNet ECDSA signature. Current connector works with pre-obtained JWT but breaks after 5-min expiry. Adding `starknet-rs` enables auto-refresh. | LOW — connector otherwise complete |
| **Lighter** | If Lighter's L2 signing is STARK-curve-based (not secp256k1), `starknet-crypto` needed instead of `k256`. Needs verification vs docs. | LOW |

### F3 — `solana-sdk`

**Version:** `1.18`

| Connector | Operations Enabled | Effort |
|-----------|-------------------|--------|
| **Jupiter** | Sign + submit swap transactions via Solana RPC `sendTransaction`. Quote API returns unsigned tx bytes; `solana-sdk` needed to sign and submit. Also enables Trigger v2 JWT auth flow. | MEDIUM |
| **Raydium** | Sign + submit swap transactions (same pattern as Jupiter). CLMM/CPMM LP operations (create position, add/remove liquidity, collect fees). Farm stake/unstake/harvest. | MEDIUM |

### F4 — `alloy` (preferred over legacy `ethers`)

**Version:** `0.1+` (current alloy, supersedes ethers-rs)

| Connector | Operations Enabled | Effort |
|-----------|-------------------|--------|
| **Uniswap** | Full swap execution (EIP-712 Permit2 signature + broadcast to Ethereum). LP provisioning (create/increase/decrease position, collect fees, claim rewards). UniswapX Dutch auction order flow. | MEDIUM |
| **GMX** | ExchangeRouter contract calls, ERC-20 approvals, order placement/cancellation. Requires keeper network interaction model (async execution, not instant REST response). | HIGH |
| **HyperLiquid** | If using EIP-712 typed data signing (as documented), `alloy`'s EIP-712 support is more complete than `k256` alone. | LOW–MEDIUM |

### F5 — `tonic` + `prost` (Cosmos gRPC)

**Version:** `tonic 0.12`, `prost 0.12`

| Connector | Operations Enabled | Effort |
|-----------|-------------------|--------|
| **dYdX v4** | Order placement (`MsgPlaceOrder`) and cancellation (`MsgCancelOrder`) via Cosmos gRPC Node API. This is a completely different transport from the read-only Indexer REST. Requires generating protobuf types from dYdX proto files and using `tonic` for gRPC connection. | HIGH — new transport layer entirely |
| **Futu** | Futu already uses TCP+protobuf. If migrating to a gRPC interface (newer OpenD versions may support it), `tonic`+`prost` would apply. | MEDIUM (if needed) |

### F6 — `graphql-client` or manual `graphql-ws` protocol (optional)

| Connector | Operations Enabled | Effort |
|-----------|-------------------|--------|
| **Bitquery** | Wire existing GraphQL subscription queries (`_build_blocks_subscription`, `_build_dex_trades_subscription`) to `WebSocketConnector` using `graphql-ws` subprotocol. No new crate strictly required — can use raw `tokio-tungstenite` with manual graphql-ws protocol implementation. | LOW — subscription queries already written |

---

## Type G: Missing WebSocket Channels

WebSocket implementations that exist (connector struct + `WebSocketConnector` trait) but are missing subscription channels or stream message types.

### CEX Connectors — Missing WS Channels

| Connector | Channels Implemented | Channels Missing |
|-----------|---------------------|-----------------|
| **Binance** | `@aggTrade`, `@kline`, `@ticker`, `@depth`, `@bookTicker`, User Data Stream | `PUT /api/v3/userDataStream` (keepalive), `DELETE /api/v3/userDataStream` (close) |
| **Bybit** | Trade, kline, orderbook, ticker | Multiple subscription variants missing per batch reports |
| **Deribit** | `public/subscribe`, `public/unsubscribe`, `private/subscribe`, `private/unsubscribe` | `public/unsubscribe_all`, `private/unsubscribe_all`; WS channel types for: `ticker.{instrument}`, `book.{instrument}`, `trades.{instrument}`, `user.orders.{instrument}`, `user.trades.{instrument}`, `user.portfolio.{currency}`, `user.positions`, funding chart, mark price, perpetual channel |
| **Gemini** | WS URL stored | Market data channel types not fully typed |
| **Phemex** | WS URL stored | Channel variants missing for ticker, orderbook, trades, account updates |

### DEX Connectors — Missing WS Channels (connector exists, channels missing)

| Connector | Channels Implemented | Channels Missing |
|-----------|---------------------|-----------------|
| **dYdX** | None (WS URL stored, no channel enum variants) | ALL 7 channels: `v4_markets`, `v4_orderbook`, `v4_trades`, `v4_candles`, `v4_subaccounts`, `v4_parent_subaccounts`, `v4_blockheight` |
| **Lighter** | None (WS URL stored, no channels typed) | ALL 20 channels: `order_book/{idx}`, `ticker/{idx}`, `market_stats/{idx}`, `market_stats/all`, `trade/{idx}`, `spot_market_stats/...`, `account_all/{id}`, `account_market/{mkt}/{id}`, `account_stats/{id}`, `account_tx/{id}`, `account_all_orders/{id}`, `account_orders/{mkt}/{id}`, `account_all_trades/{id}`, `account_all_assets/{id}`, `account_all_positions/{id}`, `account_spot_avg_entry_prices/{id}`, `pool_data/{id}`, `pool_info/{id}`, `notification/{id}`, `height` |
| **Paradex** | None (WS URL stored, no channels typed) | ALL 16 channels: `account`, `balance_events`, `transaction`, `transfers`, `bbo.{market}`, `markets_summary`, `markets_summary.{market}`, `trades.{market}`, `order_book.{market}.snapshot@...`, `order_book.{market}.delta@...`, `funding_data.{market}`, `orders.{market}`, `positions`, `fills.{market}`, `funding_payments.{market}`, `tradebusts` |

### Data Provider Connectors — Missing WS Channels

| Connector | WS URL | Missing Channels |
|-----------|--------|-----------------|
| **Alpaca (stocks)** | `wss://stream.data.alpaca.markets/v2/iex` | URL stored but stream enum variants empty — need: `bars`, `quotes`, `trades`, `statuses`, `lulds` channels |
| **Alpaca (trading)** | `wss://api.alpaca.markets/stream` | URL stored but no stream variants — need: `trade_updates` channel |
| **Alpaca (crypto)** | Missing — crypto WS URL not stored | Need URL `wss://stream.data.alpaca.markets/v1beta3/crypto/us` + channel variants |
| **Tiingo (IEX)** | WS URL stored | Stream enum variants empty — IEX channels: `subscribe`, `iex` |
| **Tiingo (Forex)** | WS URL stored | Stream enum variants empty — Forex channels: `subscribe`, `fx` |
| **Tiingo (Crypto)** | WS URL stored | Stream enum variants empty — Crypto channels: `subscribe`, `crypto` |
| **Finnhub** | WS URL stored | No stream enum variants — need: `subscribe`/`unsubscribe` messages + `trade` update type |
| **Polygon (stocks)** | WS URL stored | Channel variants missing for all 4 WS categories |
| **Polygon (options)** | Not stored | Options WS category entirely missing |
| **Polygon (forex)** | Not stored | Forex WS category entirely missing |
| **Polygon (crypto)** | Not stored | Crypto WS category entirely missing |
| **Polygon (indices)** | Not stored | Indices WS category entirely missing |

---

## Summary: Gap Count by Work Type

| Type | Category | Approximate Gap Count |
|------|----------|----------------------|
| A | URL/Config Fixes | 15 bugs across 12 connectors |
| B | Parser/Auth Fixes | 5 connectors (1 full blocker, 1 partial blocker, 3 structural) |
| C | New Endpoints (Same Traits) | 250+ missing endpoint variants across ~60 connectors |
| D1 | New Enum Variants | 10+ new variants across `OrderType`, `AccountType`, `CancelScope`, `PositionModification` |
| D2 | New Methods on Existing Traits | 6 new methods (`get_user_trades`, `get_open_interest`, `get_funding_rate_history`, `get_mark_price`, `get_closed_pnl`, `get_long_short_ratio`) |
| D3 | New Optional Traits | 11 new traits (`MarginTrading`, `EarnStaking`, `ConvertSwap`, `CopyTrading`, `LiquidityProvider`, `VaultManager`, `StakingDelegation`, `BlockTradeOtc`, `MarketMakerProtection`, `TriggerOrders`, `PredictionMarket`) |
| E | Transport Layer | 8 connectors needing new/different transports |
| F | External Crate Dependencies | 6 crate groups needed for 7 connectors |
| G | Missing WS Channels | 60+ channels across 14 connectors |

---

## Recommended Execution Order

### Sprint 1 — Bugs First (no new crates, no trait changes)
**Types A + B:** Fix all 15 URL bugs + auth/parser issues first. Unblocks connectors that are currently broken.

### Sprint 2 — Trade History (no new crates, no trait changes)
**Type D2.1 first:** Add `get_user_trades` method to `Trading` trait + implement across 10 top CEX connectors.
**Type C1:** Add fill history endpoint variants to Binance, Bybit, OKX, KuCoin, HTX, Gate.io, MEXC, Bitget, BingX.

### Sprint 3 — Derivatives Intelligence (no new crates, new enum variants)
**Types C1 + D1 + D2.2–D2.6:** Add OI, funding rate history, mark price, closed PnL, long/short ratio. Add `OpenInterest` + `MarkPrice` types. Add methods to traits.

### Sprint 4 — Batch Amend + Advanced Orders (no new crates)
**Type C2 + D1.1:** Add batch amend endpoint variants + new `OrderType` variants for conditional/plan orders.

### Sprint 5 — WebSocket Channels (no new crates, uses existing tokio-tungstenite)
**Type G:** Wire dYdX, Lighter, Paradex WS channels. Implement Bitget, BingX, Bitstamp, Upbit `WebSocketConnector`.

### Sprint 6 — DEX Signing (new crates: k256, starknet-rs)
**Type F1 + F2:** Lighter ECDSA trading. Paradex JWT auto-refresh. These are low-effort with existing REST infrastructure.

### Sprint 7 — Solana/EVM Signing (new crates: solana-sdk, alloy)
**Type F3 + F4:** Jupiter + Raydium swap execution. Uniswap swap + LP. GMX trading (hardest).

### Sprint 8 — dYdX gRPC (new transport: tonic + prost)
**Type E + F5:** Cosmos gRPC order placement for dYdX. Entirely new transport layer.

### Sprint 9 — Optional Trait Surface (new traits, no new crates)
**Type D3:** Implement `MarginTrading`, `EarnStaking`, `ConvertSwap`, `CopyTrading` traits across relevant connectors.

### Sprint 10 — Intel Feed Completeness
**Types C6 + C7:** Complete economic, governance, humanitarian, cyber, space, maritime feeds. Mostly REST endpoint additions, no trait changes.
