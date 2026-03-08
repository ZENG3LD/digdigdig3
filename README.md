# digdigdig3

> **Multi-exchange connector library for Rust** — unified async trait API covering crypto exchanges, stock brokers, forex providers, aggregators, and 88 intelligence feeds.

[![Crates.io](https://img.shields.io/crates/v/digdigdig3.svg)](https://crates.io/crates/digdigdig3)
[![docs.rs](https://docs.rs/digdigdig3/badge.svg)](https://docs.rs/digdigdig3)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/ZENG3LD/digdigdig3#license)

## Features

- **26 crypto venues** — 17 CEX + 2 derivatives platforms + 7 DEX
- **14 stock market connectors** — US, India, Japan, Korea, Russia
- **3 forex providers** — broker and historical data
- **4 multi-asset aggregators** — DeFi, crypto, global equities
- **88 intelligence feeds** — economic, geopolitical, aviation, maritime, cyber, space, and more
- **Unified async trait API** — `MarketData`, `Trading`, `Account`, `Positions` across all venues
- **`AnyConnector` enum** — store heterogeneous connectors in collections without `dyn Trait`
- **`ConnectorFactory`** — create any connector by `ExchangeId` with one call
- **WebSocket streaming** — real-time klines, trades, orderbook, ticker, private events
- **Built-in rate limiting** — per-exchange weight limiters and simple request-count limiters
- **Testnet support** — Binance, Bybit, OKX, Kraken, Phemex, Deribit, and others

## Quick Start

```toml
[dependencies]
digdigdig3 = "0.1"
tokio = { version = "1", features = ["full"] }
```

```rust
use digdigdig3::{
    connector_manager::{ConnectorFactory, AnyConnector},
    ExchangeId, AccountType, Symbol, MarketData,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a public Binance connector — no API key required
    let connector = ConnectorFactory::create_public(ExchangeId::Binance).await?;

    let symbol = Symbol::new("BTC", "USDT");
    let price = connector.get_price(symbol.clone(), AccountType::Spot).await?;
    println!("BTC/USDT: {price}");

    let klines = connector
        .get_klines(symbol, "1h", Some(10), AccountType::Spot, None)
        .await?;
    println!("Got {} candles", klines.len());

    Ok(())
}
```

## Supported Venues

### Crypto — CEX

| Exchange | Spot | Futures | WebSocket | Testnet |
|----------|------|---------|-----------|---------|
| Binance | yes | yes | yes | yes |
| Bybit | yes | yes | yes | yes |
| OKX | yes | yes | yes | yes |
| KuCoin | yes | yes | yes | no |
| Kraken | yes | yes | yes | yes |
| Coinbase | yes | no | yes | no |
| Gate.io | yes | yes | yes | no |
| Bitfinex | yes | yes | yes | yes |
| Bitstamp | yes | no | yes | no |
| Gemini | yes | no | yes | no |
| MEXC | yes | yes | yes | no |
| HTX | yes | yes | yes | yes |
| Bitget | yes | yes | yes | no |
| BingX | yes | yes | yes | yes |
| Phemex | yes | yes | yes | yes |
| Crypto.com | yes | yes | yes | yes |
| Upbit | yes | no | yes | no |

### Crypto — Derivatives

| Exchange | Type | WebSocket | Testnet |
|----------|------|-----------|---------|
| Deribit | Options / Perpetuals | yes | yes |
| HyperLiquid | Perpetuals (hybrid CEX/DEX) | yes | yes |

### Crypto — DEX

| Exchange | Chain | WebSocket |
|----------|-------|-----------|
| Uniswap | Ethereum | no |
| Lighter | Ethereum L2 | yes |
| dYdX v4 | Cosmos | yes |
| Paradex | Starknet | yes |
| Jupiter | Solana | no |
| Raydium | Solana | yes |
| GMX | Arbitrum | no |

### Prediction Markets

| Provider | Description |
|----------|-------------|
| Polymarket | Probability-based trading on real-world events |

### Stocks — US

| Provider | Type | Trading |
|----------|------|---------|
| Polygon | Data (NYSE, NASDAQ) | no |
| Finnhub | Data (global coverage) | no |
| Tiingo | Data (stocks, forex, crypto) | no |
| Twelvedata | Data (100+ indicators, ETFs) | no |
| Alpaca | Broker (commission-free) | yes |

### Stocks — India

| Provider | Exchanges | Trading |
|----------|-----------|---------|
| AngelOne | NSE, BSE, MCX, CDS | yes |
| Zerodha | NSE, BSE, NFO, BFO, MCX | yes |
| Fyers | NSE, BSE, MCX, NCDEX | yes |
| Dhan | NSE, BSE, MCX (200-level depth) | yes |
| Upstox | NSE, BSE, MCX | yes |

### Stocks — Other Regions

| Provider | Market | Type |
|----------|--------|------|
| JQuants | Japan (TSE / JPX) | Data |
| Tinkoff | Russia (MOEX) | Broker |
| MOEX ISS | Russia | Data |
| KRX | Korea (KOSPI, KOSDAQ, KONEX) | Data |

### Forex

| Provider | Type | Trading |
|----------|------|---------|
| OANDA | Broker (REST + streaming) | yes |
| Alpha Vantage | Data (forex, stocks, crypto) | no |
| Dukascopy | Historical tick data (2003+) | no |

### Multi-Asset Aggregators

| Provider | Coverage |
|----------|----------|
| Yahoo Finance | Stocks, crypto, forex, options, fundamentals |
| CryptoCompare | 5,700+ coins, 170+ exchanges, CCCAGG index |
| DefiLlama | DeFi TVL, protocols, yields |
| Interactive Brokers | Stocks, forex, futures, options (multi-market) |

## Architecture

### Trait System

All connectors implement a minimal core trait stack:

```
ExchangeIdentity   — id(), exchange_name(), is_testnet(), supported_account_types()
     │
     ├── MarketData — get_price(), get_orderbook(), get_klines(), get_ticker(), ping()
     ├── Trading    — market_order(), limit_order(), cancel_order(), get_order(), open_orders()
     ├── Account    — get_balance(), get_account_info()
     └── Positions  — get_positions(), get_funding_rate(), set_leverage()
```

The `CoreConnector` supertrait combines all five:

```rust
pub trait CoreConnector:
    ExchangeIdentity + MarketData + Trading + Account + Positions + Send + Sync {}
```

Exchange-specific extensions (modify_order, cancel_all, TP/SL orders, sub-accounts, etc.) live directly on each connector struct — not in core traits — keeping the shared interface lean and guaranteed across all venues.

### `AnyConnector` and `ConnectorFactory`

`AnyConnector` is an enum with one variant per connector. It implements all core traits via delegation and is cheaply clonable via `Arc` wrapping:

```rust
// All of these have the same type: Arc<AnyConnector>
let binance = ConnectorFactory::create_public(ExchangeId::Binance).await?;
let bybit   = ConnectorFactory::create_public(ExchangeId::Bybit).await?;

// Store in a Vec, pass to generic functions — same type, no dyn
let connectors: Vec<Arc<AnyConnector>> = vec![binance, bybit];
```

`ConnectorRegistry` provides O(1) metadata lookup by `ExchangeId`: name, category, feature flags (market_data, trading, websocket, etc.).

### Intelligence Feed Registry

`FeedId` (88 variants) and `FeedRegistry` follow the same pattern for intelligence feeds. `FeedMetadata` carries category, auth type, endpoint base URL, and a human-readable description.

## Usage Examples

### Public Market Data (no API key)

```rust
use digdigdig3::{
    connector_manager::ConnectorFactory,
    ExchangeId, AccountType, Symbol, MarketData,
};

let conn = ConnectorFactory::create_public(ExchangeId::Bybit).await?;
let symbol = Symbol::new("ETH", "USDT");

// Current price
let price = conn.get_price(symbol.clone(), AccountType::Spot).await?;

// Order book (20 levels)
let book = conn.get_orderbook(symbol.clone(), Some(20), AccountType::Spot).await?;

// Last 100 hourly candles
let candles = conn
    .get_klines(symbol.clone(), "1h", Some(100), AccountType::Spot, None)
    .await?;

// 24-hour ticker
let ticker = conn.get_ticker(symbol, AccountType::Spot).await?;
println!("24h volume: {}", ticker.volume);
```

### Authenticated Trading

```rust
use digdigdig3::{
    connector_manager::ConnectorFactory,
    Credentials, ExchangeId, AccountType, Symbol, OrderSide, MarketData, Trading, Account,
};

let creds = Credentials::new("YOUR_API_KEY", "YOUR_API_SECRET");
let conn = ConnectorFactory::create_authenticated(ExchangeId::Binance, creds).await?;

// Account balance
let balance = conn.get_balance(None, AccountType::Spot).await?;

// Place a limit order
let symbol = Symbol::new("BTC", "USDT");
let order = conn
    .limit_order(symbol.clone(), OrderSide::Buy, 0.001, 60_000.0, AccountType::Spot)
    .await?;
println!("Order id: {}", order.id);

// Cancel it
conn.cancel_order(symbol, &order.id, AccountType::Spot).await?;
```

### WebSocket Streaming

```rust
use digdigdig3::{
    ExchangeId, AccountType, Symbol, StreamType, SubscriptionRequest,
};
use digdigdig3::exchanges::binance::BinanceConnector;
use digdigdig3::WebSocketConnector;
use futures_util::StreamExt;

let mut ws = BinanceConnector::public(false).await?;
ws.connect(AccountType::Spot).await?;

let req = SubscriptionRequest {
    stream_type: StreamType::Kline { interval: "1m".to_string() },
    symbol: Symbol::new("BTC", "USDT"),
    account_type: AccountType::Spot,
};
ws.subscribe(req).await?;

let mut stream = ws.event_stream();
while let Some(event) = stream.next().await {
    match event? {
        digdigdig3::StreamEvent::Kline(k) => {
            println!("open={} close={} volume={}", k.open, k.close, k.volume);
        }
        _ => {}
    }
}
```

### Intelligence Feeds

Intelligence feeds are separate from exchange connectors and live under `digdigdig3::intelligence_feeds`. Each feed has its own typed connector in `src/intelligence_feeds/<category>/<feed>/`.

```rust
// Example: fetch earthquake data from USGS
use digdigdig3::intelligence_feeds::feed_manager::{FeedId, FeedRegistry};

let meta = FeedRegistry::get(FeedId::UsgsEarthquake);
println!("Feed: {} — auth required: {:?}", meta.name, meta.auth_type);
```

### Query Available Connectors

```rust
use digdigdig3::connector_manager::{ConnectorRegistry, ConnectorCategory};

// All crypto CEX connectors
let cex = ConnectorRegistry::by_category(ConnectorCategory::CryptoExchangeCex);
for meta in cex {
    println!("{} — ws={} trading={}", meta.name, meta.features.websocket, meta.features.trading);
}

// All connectors that support WebSocket
let ws_capable = ConnectorRegistry::all()
    .filter(|m| m.features.websocket)
    .collect::<Vec<_>>();
println!("{} connectors support WebSocket", ws_capable.len());
```

## Intelligence Feed Categories

88 feeds across 22 categories:

| Category | Count | Examples |
|----------|-------|---------|
| Economic | 12 | FRED, ECB, IMF, World Bank, OECD, BIS, Eurostat |
| Cyber | 9 | Shodan, VirusTotal, NVD, AbuseIPDB, Censys, RIPE NCC |
| US Government | 9 | SEC EDGAR, BLS, EIA, Census, Congress.gov, BEA |
| Environment | 9 | USGS Earthquake, NOAA, NASA EONET/FIRMS, OpenWeatherMap |
| Conflict | 5 | ACLED, GDELT, UCDP, ReliefWeb, UNHCR |
| Space | 5 | NASA, SpaceX, Space-Track, Launch Library 2, Sentinel Hub |
| Aviation | 4 | ADS-B Exchange, OpenSky, AviationStack, Wingbits |
| Maritime | 4 | AISStream, Datalastic AIS, IMF PortWatch, NGA Warnings |
| Demographics | 4 | WHO, World Bank Population, UN OCHA, Wikipedia |
| Financial | 4 | Alpha Vantage, Finnhub, NewsAPI, OpenFIGI |
| Crypto | 5 | CoinGecko, Coinglass, Whale Alert, Etherscan, Bitquery |
| Sanctions | 3 | OFAC, OpenSanctions, INTERPOL |
| Corporate | 3 | GLEIF, OpenCorporates, UK Companies House |
| Trade | 2 | UN COMTRADE, EU TED |
| Governance | 2 | EU Parliament, UK Parliament |
| Academic | 2 | arXiv, Semantic Scholar |
| C2 Intel | 1 | C2 Intel Feeds |
| FAA | 1 | FAA Status |
| Feodo | 1 | Feodo Tracker (botnet C2 blocklist) |
| Hacker News | 1 | Hacker News |
| Prediction | 1 | PredictIt |
| RSS | 1 | RSS Proxy |

## Error Handling

All fallible operations return `ExchangeResult<T>` = `Result<T, ExchangeError>`:

```rust
use digdigdig3::ExchangeError;

match connector.get_price(symbol, AccountType::Spot).await {
    Ok(price) => println!("Price: {price}"),
    Err(ExchangeError::RateLimit) => eprintln!("Rate limited — back off"),
    Err(ExchangeError::Auth(msg)) => eprintln!("Auth failed: {msg}"),
    Err(ExchangeError::Api { code, message }) => eprintln!("API {code}: {message}"),
    Err(e) => eprintln!("Error: {e}"),
}
```

## Crate Features

| Feature | Default | Description |
|---------|---------|-------------|
| `websocket` | no | Enable WebSocket streaming support |

## Contributing

New exchange connectors follow the Agent Carousel pipeline documented in `contributing/`. Each connector lives in:

```
src/exchanges/<name>/
├── mod.rs         # public exports
├── endpoints.rs   # URL constants, symbol formatting
├── auth.rs        # request signing
├── parser.rs      # JSON deserialization
├── connector.rs   # trait implementations
└── websocket.rs   # WebSocket (optional)
```

Reference implementation: `src/exchanges/kucoin/`

## Support the Project

If you find this library useful, consider supporting development:

| Currency | Network | Address |
|----------|---------|---------|
| USDT | TRC20 | `TNxMKsvVLYViQ5X5sgCYmkzH4qjhhh5U7X` |
| USDC | Arbitrum | `0xEF3B94Fe845E21371b4C4C5F2032E1f23A13Aa6e` |
| ETH | Ethereum | `0xEF3B94Fe845E21371b4C4C5F2032E1f23A13Aa6e` |
| BTC | Bitcoin | `bc1qjgzthxja8umt5tvrp5tfcf9zeepmhn0f6mnt40` |
| SOL | Solana | `DZJjmH8Cs5wEafz5Ua86wBBkurSA4xdWXa3LWnBUR94c` |

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

---

<p align="center">
  <img src="assets/author.svg" alt="zengeld" />
</p>
