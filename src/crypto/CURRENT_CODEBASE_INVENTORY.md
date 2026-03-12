# digdigdig3 Crate — Complete Codebase Inventory

Generated: 2026-03-12
Purpose: Pre-refactoring inventory for trait+enum architecture redesign.

---

## 1. Crate Metadata (`digdigdig3/Cargo.toml`)

```toml
name    = "digdigdig3"
version = "0.1.5"
edition = "2021"
```

**Description:** Multi-exchange connector library — unified async Rust API for 40+ crypto exchanges, stock brokers, forex providers, and 88 intelligence feeds.

### Features

| Feature | Default? | Description |
|---------|----------|-------------|
| `websocket` | No | WebSocket support |
| `onchain-ethereum` | Yes | Enables `alloy` for Uniswap WS |

### Key Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `serde` / `serde_json` | 1.0 | Serialization |
| `tokio` | 1.0 (full) | Async runtime |
| `async-trait` | 0.1 | Async trait support |
| `reqwest` | 0.12 (rustls) | HTTP client |
| `tokio-tungstenite` | 0.24 | WebSocket client |
| `tokio-stream` | 0.1 | Stream utilities |
| `thiserror` | 1.0 | Error types |
| `tracing` | 0.1 | Logging |
| `hmac` / `sha2` | 0.12 / 0.10 | HMAC signing |
| `hex` / `base64` | 0.4 / 0.22 | Encoding |
| `chrono` | 0.4 | ISO 8601 timestamps (OKX) |
| `byteorder` | 1.5 | Binary parsing (Dhan WS) |
| `uuid` | 1.0 (v4) | Client order IDs |
| `flate2` / `lzma-rs` | 1.0 / 0.3 | WS compression (BingX, HTX, Dukascopy) |
| `url` / `urlencoding` | 2.5 / 2.1 | URL encoding (HTX auth, Polygon) |
| `jsonwebtoken` | 9.3 | JWT auth (Coinbase) |
| `rand` | 0.8 | Random generation |
| `bs58` | 0.5 | Base58 (Raydium pubkeys) |
| `alloy` | 1 (optional) | Ethereum WS (Uniswap) |
| `totp-rs` | 5.6 | TOTP 2FA (Angel One) |
| `dashmap` | 5.5 | Lock-free pool (ConnectorPool) |

---

## 2. Crate Root (`digdigdig3/src/lib.rs`)

### Top-level Modules

| Module | Description |
|--------|-------------|
| `core` | Traits, types, utils, HTTP/WS transport |
| `crypto` | Crypto exchange connectors (CEX, DEX, swap) |
| `onchain` | On-chain integrations |
| `stocks` | Stock brokers and data providers (US, India, Japan, Korea, Russia) |
| `forex` | Forex brokers and data providers |
| `aggregators` | Multi-asset aggregators (IB, Yahoo, CryptoCompare, DefiLlama) |
| `intelligence_feeds` | 88 intelligence feeds across 20+ thematic categories |
| `prediction` | Prediction market connectors |
| `connector_manager` | Unified AnyConnector enum, pool, factory, aggregator |

### Re-exports from `core`

**Traits:** `ExchangeIdentity`, `MarketData`, `Trading`, `Positions`, `Account`, `CoreConnector`, `WebSocketConnector`, `WebSocketExt`, `Credentials`, `AuthRequest`, `SignatureLocation`, `ExchangeAuth`

**Types:** `ExchangeId`, `ExchangeType`, `AccountType`, `Symbol`, `ExchangeError`, `ExchangeResult`, `Price`, `Quantity`, `Asset`, `Timestamp`, `OrderSide`, `OrderType`, `OrderStatus`, `Order`, `Position`, `Balance`, `SymbolInfo`, `ConnectionStatus`, `StreamType`, `SubscriptionRequest`, `StreamEvent`, `OrderUpdateEvent`, `BalanceUpdateEvent`, `PositionUpdateEvent`

**Utils:** `hmac_sha256`, `hmac_sha512`, `sha256`, `sha512`, `encode_base64`, `encode_hex`, `encode_hex_lower`, `timestamp_millis`, `timestamp_seconds`, `timestamp_iso8601`

**Transport:** `HttpClient`

---

## 3. Core Module (`digdigdig3/src/core/`)

### 3.1 Types (`core/types/`)

All types are glob-re-exported via `types/mod.rs`.

#### `types/common.rs` — Exchange Identification & Errors

**`ExchangeId` (enum)** — `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]`

CEX variants: `Binance`, `Bybit`, `OKX`, `KuCoin`, `Kraken`, `Coinbase`, `GateIO`, `Bitfinex`, `Bitstamp`, `Gemini`, `MEXC`, `HTX`, `Bitget`, `BingX`, `Phemex`, `CryptoCom`, `Upbit`, `Deribit`, `HyperLiquid`

DEX variants: `Lighter`, `Uniswap`, `Jupiter`, `Raydium`, `Gmx`, `Paradex`, `Dydx`

Prediction: `Polymarket`

Data Providers: `Polygon`, `Finnhub`, `Tiingo`, `Twelvedata`, `Coinglass`, `CryptoCompare`, `WhaleAlert`, `Bitquery`

DeFi Aggregators: `DefiLlama`

Forex: `Oanda`, `AlphaVantage`, `Dukascopy`

Stock Brokers/Data: `AngelOne`, `Zerodha`, `Fyers`, `Dhan`, `Upstox`, `Alpaca`, `JQuants`, `Tinkoff`, `Moex`, `Krx`

Economic Data: `Fred`, `Bls`

Multi-Asset Aggregators: `YahooFinance`, `Ib`

Special: `Custom(u16)` — user-defined ID

Methods: `as_str() -> &'static str`, `from_str(s: &str) -> Option<Self>`, `exchange_type() -> ExchangeType`

**`ExchangeType` (enum)** — `#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]`

Variants: `Cex`, `Dex`, `Hybrid`, `Broker`, `DataProvider`

**`AccountType` (enum)** — `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]`

Variants: `Spot`, `Margin`, `FuturesCross`, `FuturesIsolated`

**`Symbol` (struct)** — `#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]`

Fields:
- `base: String` — base asset (e.g. "BTC")
- `quote: String` — quote asset (e.g. "USDT")
- `raw: Option<String>` — original exchange string (skip_serializing_if None)

Methods: `new(base, quote)`, `with_raw(base, quote, raw)`, `raw()`, `empty()`, `is_empty()`, `to_concat()`, `to_dash()`, `to_underscore()`, `parse(s)`

Display: `"BTC/USDT"`

**`ExchangeError` (enum)** — `#[derive(Debug, Error)]`

Variants:
- `Http(String)`
- `Network(String)`
- `Parse(String)`
- `ParseError(String)` — duplicate, exists alongside Parse
- `Api { code: i32, message: String }`
- `RateLimit`
- `RateLimitExceeded { retry_after: Option<u64>, message: String }`
- `Auth(String)`
- `InvalidCredentials(String)`
- `PermissionDenied(String)`
- `InvalidRequest(String)`
- `NotSupported(String)`
- `UnsupportedOperation(String)`
- `Timeout(String)`
- `NotFound(String)`

**`ExchangeResult<T>`** = `Result<T, ExchangeError>` (type alias)

**`WebSocketError` (enum)** — `#[derive(Debug, Clone, Error)]`

Variants: `ConnectionError(String)`, `NotConnected`, `ProtocolError(String)`, `Parse(String)`, `Subscription(String)`, `Auth(String)`, `SendError(String)`, `ReceiveError(String)`, `UnsupportedOperation(String)`, `Timeout`

**`WebSocketResult<T>`** = `Result<T, WebSocketError>` (type alias)

**`ConnectorStats` (struct)** — `#[derive(Debug, Clone, Default)]`

Fields:
- `http_requests: u64`
- `http_errors: u64`
- `last_latency_ms: u64`
- `rate_used: u32`
- `rate_max: u32`
- `rate_groups: Vec<(String, u32, u32)>`
- `ws_ping_rtt_ms: u64`

---

#### `types/market_data.rs` — Market Data Types

**`Kline` (struct)** — `#[derive(Debug, Clone, Serialize, Deserialize)]`

Fields:
- `open_time: i64` — Unix ms
- `open: f64`
- `high: f64`
- `low: f64`
- `close: f64`
- `volume: f64` — base asset volume
- `quote_volume: Option<f64>`
- `close_time: Option<i64>`
- `trades: Option<u64>`

**`Ticker` (struct)** — `#[derive(Debug, Clone, Serialize, Deserialize)]`

Fields:
- `symbol: String`
- `last_price: f64`
- `bid_price: Option<f64>`
- `ask_price: Option<f64>`
- `high_24h: Option<f64>`
- `low_24h: Option<f64>`
- `volume_24h: Option<f64>`
- `quote_volume_24h: Option<f64>`
- `price_change_24h: Option<f64>`
- `price_change_percent_24h: Option<f64>`
- `timestamp: i64`

**`OrderBook` (struct)** — `#[derive(Debug, Clone, Serialize, Deserialize)]`

Fields:
- `bids: Vec<(f64, f64)>` — sorted descending by price
- `asks: Vec<(f64, f64)>` — sorted ascending by price
- `timestamp: i64`
- `sequence: Option<String>` — for incremental updates

**`FundingRate` (struct)** — `#[derive(Debug, Clone, Serialize, Deserialize)]`

Fields:
- `symbol: String`
- `rate: f64`
- `next_funding_time: Option<i64>`
- `timestamp: i64`

**`MarkPrice` (struct)** — `#[derive(Debug, Clone, Serialize, Deserialize)]`

Fields:
- `symbol: String`
- `mark_price: f64`
- `index_price: Option<f64>`
- `timestamp: i64`

**`OpenInterest` (struct)** — `#[derive(Debug, Clone, Serialize, Deserialize)]`

Fields:
- `symbol: String`
- `open_interest: f64`
- `open_interest_value: Option<f64>`
- `timestamp: i64`

**`PublicTrade` (struct)** — `#[derive(Debug, Clone, Serialize, Deserialize)]`

Fields:
- `id: String`
- `symbol: String`
- `price: f64`
- `quantity: f64`
- `side: TradeSide`
- `timestamp: i64`

**`TradeSide` (enum)** — `#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]`

Variants: `Buy`, `Sell`

---

#### `types/trading.rs` — Trading Types

**Type Aliases:**
- `Price = f64`
- `Quantity = f64`
- `Asset = String`
- `Timestamp = i64`

**`OrderSide` (enum)** — `#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]`

Variants: `Buy`, `Sell`

Methods: `as_str() -> &'static str` ("BUY"/"SELL"), `opposite() -> Self`

**`OrderType` (enum)** — `#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]`

Variants: `Market`, `Limit`, `StopLoss`, `StopLossLimit`, `TakeProfit`, `TakeProfitLimit`

Methods: `as_str()`

**`OrderStatus` (enum)** — `#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]`

Variants: `New`, `Open`, `PartiallyFilled`, `Filled`, `Canceled`, `Rejected`, `Expired`

**`TimeInForce` (enum)** — `#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]`

Variants: `GTC` (default), `IOC`, `FOK`, `GTD`, `PostOnly`

Methods: `as_str()`

**`Order` (struct)** — `#[derive(Debug, Clone, Serialize, Deserialize)]`

Fields:
- `id: String` — exchange order ID
- `client_order_id: Option<String>`
- `symbol: String`
- `side: OrderSide`
- `order_type: OrderType`
- `status: OrderStatus`
- `price: Option<Price>`
- `stop_price: Option<Price>`
- `quantity: Quantity`
- `filled_quantity: Quantity`
- `average_price: Option<Price>`
- `commission: Option<Price>`
- `commission_asset: Option<String>`
- `created_at: Timestamp`
- `updated_at: Option<Timestamp>`
- `time_in_force: TimeInForce`

**`CreateOrderRequest` (struct)** — `#[derive(Debug, Clone)]`

Fields: `symbol: Symbol`, `side: OrderSide`, `order_type: OrderType`, `quantity: Quantity`, `price: Option<Price>`, `stop_price: Option<Price>`, `time_in_force: TimeInForce`, `account_type: AccountType`

**`PositionMode` (enum)** — `#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]`

Variants: `OneWay` (default), `Hedge`

**`PositionSide` (enum)** — `#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]`

Variants: `Long`, `Short`, `Both`

**`Position` (struct)** — `#[derive(Debug, Clone, Serialize, Deserialize)]`

Fields:
- `symbol: String`
- `side: PositionSide`
- `quantity: Quantity`
- `entry_price: Price`
- `mark_price: Option<Price>`
- `unrealized_pnl: Price`
- `realized_pnl: Option<Price>`
- `liquidation_price: Option<Price>`
- `leverage: u32`
- `margin_type: MarginType`
- `margin: Option<Price>`
- `take_profit: Option<Price>`
- `stop_loss: Option<Price>`

**`Balance` (struct)** — `#[derive(Debug, Clone, Serialize, Deserialize)]`

Fields:
- `asset: String`
- `free: f64`
- `locked: f64`
- `total: f64`

**`AccountInfo` (struct)** — `#[derive(Debug, Clone, Serialize, Deserialize)]`

Fields:
- `account_type: AccountType`
- `can_trade: bool`
- `can_withdraw: bool`
- `can_deposit: bool`
- `maker_commission: f64`
- `taker_commission: f64`
- `balances: Vec<Balance>`

**`UserTrade` (struct)** — `#[derive(Debug, Clone, Serialize, Deserialize)]`

Fields: `id`, `order_id`, `symbol`, `side: OrderSide`, `price: Price`, `quantity: Quantity`, `commission: Price`, `commission_asset: String`, `is_maker: bool`, `timestamp: Timestamp`

**`SymbolInfo` (struct)** — `#[derive(Debug, Clone, Serialize, Deserialize)]`

Fields:
- `symbol: String`
- `base_asset: String`
- `quote_asset: String`
- `status: String`
- `price_precision: u8`
- `quantity_precision: u8`
- `min_quantity: Option<f64>`
- `max_quantity: Option<f64>`
- `step_size: Option<f64>`
- `min_notional: Option<f64>`

**`ExchangeInfo` (struct)** — `#[derive(Debug, Clone, Serialize, Deserialize)]`

Fields: `server_time: Option<Timestamp>`, `symbols: Vec<SymbolInfo>`

**`MarginType` (enum)** — `#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]`

Variants: `Cross`, `Isolated`

**`MarginBorrowResult` (struct)** — Fields: `transaction_id: String`, `asset: String`, `amount: f64`

**`MarginRepayResult` (struct)** — Fields: `transaction_id: String`, `asset: String`, `amount: f64`

**`MarginLoan` (struct)** — Fields: `asset: String`, `borrowed: f64`, `interest: f64`, `total: f64`

**`TransferResult` (struct)** — Fields: `transaction_id: String`, `asset: String`, `amount: f64`

**`TransferHistory` (struct)** — Fields: `transaction_id`, `asset`, `amount`, `from_account: String`, `to_account: String`, `timestamp: Timestamp`

**`ListenKey` (struct)** — Fields: `key: String`, `expires_at: Option<Timestamp>`

---

#### `types/websocket.rs` — WebSocket Types

**`ConnectionStatus` (enum)** — `#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]`

Variants: `Disconnected`, `Connecting`, `Connected`, `Reconnecting`

**`StreamType` (enum)** — `#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]`

Public variants: `Ticker`, `Trade`, `Orderbook`, `OrderbookDelta`, `Kline { interval: String }`, `MarkPrice`, `FundingRate`

Private variants: `OrderUpdate`, `BalanceUpdate`, `PositionUpdate`

**`SubscriptionRequest` (struct)** — `#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]`

Fields: `symbol: Symbol`, `stream_type: StreamType`

Constructors: `new(symbol, stream_type)`, `ticker(symbol)`, `trade(symbol)`, `orderbook(symbol)`, `kline(symbol, interval)`

**`StreamEvent` (enum)** — `#[derive(Debug, Clone, Serialize, Deserialize)]`

Public variants:
- `Ticker(Ticker)`
- `Trade(PublicTrade)`
- `OrderbookSnapshot(OrderBook)`
- `OrderbookDelta { bids: Vec<(f64,f64)>, asks: Vec<(f64,f64)>, timestamp: i64 }`
- `Kline(Kline)`
- `MarkPrice { symbol, mark_price, index_price, timestamp }`
- `FundingRate { symbol, rate, next_funding_time, timestamp }`

Private variants:
- `OrderUpdate(OrderUpdateEvent)`
- `BalanceUpdate(BalanceUpdateEvent)`
- `PositionUpdate(PositionUpdateEvent)`

**`OrderUpdateEvent` (struct)** — `#[derive(Debug, Clone, Serialize, Deserialize)]`

Fields: `order_id`, `client_order_id`, `symbol`, `side: OrderSide`, `order_type: OrderType`, `status: OrderStatus`, `price`, `quantity`, `filled_quantity`, `average_price`, `last_fill_price`, `last_fill_quantity`, `last_fill_commission`, `commission_asset`, `trade_id`, `timestamp: Timestamp`

**`BalanceUpdateEvent` (struct)** — `#[derive(Debug, Clone, Serialize, Deserialize)]`

Fields: `asset: String`, `free: Price`, `locked: Price`, `total: Price`, `delta: Option<Price>`, `reason: Option<BalanceChangeReason>`, `timestamp: Timestamp`

**`BalanceChangeReason` (enum)** — `#[derive(Debug, Clone, Serialize, Deserialize)]`

Variants: `Deposit`, `Withdraw`, `Trade`, `Commission`, `Funding`, `RealizedPnl`, `Transfer`, `Other`

**`PositionUpdateEvent` (struct)** — `#[derive(Debug, Clone, Serialize, Deserialize)]`

Fields: `symbol`, `side: PositionSide`, `quantity`, `entry_price`, `mark_price`, `unrealized_pnl`, `realized_pnl`, `liquidation_price`, `leverage: Option<u32>`, `margin_type: Option<MarginType>`, `reason: Option<PositionChangeReason>`, `timestamp`

**`PositionChangeReason` (enum)** — `#[derive(Debug, Clone, Serialize, Deserialize)]`

Variants: `Trade`, `LeverageChange`, `MarginChange`, `Liquidation`, `Adl`, `Funding`, `Other`

---

### 3.2 Traits (`core/traits/`)

All traits require `ExchangeIdentity` as supertrait (except `ExchangeAuth`).

#### `traits/identity.rs` — `ExchangeIdentity`

Supertrait: `Send + Sync`

Methods:
- `fn exchange_id(&self) -> ExchangeId` — required
- `fn exchange_name(&self) -> &'static str` — default: `exchange_id().as_str()`
- `fn is_testnet(&self) -> bool` — required
- `fn supported_account_types(&self) -> Vec<AccountType>` — required
- `fn exchange_type(&self) -> ExchangeType` — default: `exchange_id().exchange_type()`
- `fn metrics(&self) -> ConnectorStats` — default: zeroed stats

#### `traits/market_data.rs` — `MarketData: ExchangeIdentity`

6 methods (5 required + 1 default):
- `async fn get_price(symbol, account_type) -> ExchangeResult<Price>` — required
- `async fn get_orderbook(symbol, depth: Option<u16>, account_type) -> ExchangeResult<OrderBook>` — required
- `async fn get_klines(symbol, interval: &str, limit: Option<u16>, account_type, end_time: Option<i64>) -> ExchangeResult<Vec<Kline>>` — required
- `async fn get_ticker(symbol, account_type) -> ExchangeResult<Ticker>` — required
- `async fn ping() -> ExchangeResult<()>` — required
- `async fn get_exchange_info(account_type) -> ExchangeResult<Vec<SymbolInfo>>` — default: `UnsupportedOperation`

#### `traits/trading.rs` — `Trading: ExchangeIdentity`

5 required methods:
- `async fn market_order(symbol, side, quantity, account_type) -> ExchangeResult<Order>`
- `async fn limit_order(symbol, side, quantity, price, account_type) -> ExchangeResult<Order>`
- `async fn cancel_order(symbol, order_id: &str, account_type) -> ExchangeResult<Order>`
- `async fn get_order(symbol, order_id: &str, account_type) -> ExchangeResult<Order>`
- `async fn get_open_orders(symbol: Option<Symbol>, account_type) -> ExchangeResult<Vec<Order>>`

#### `traits/account.rs` — `Account: ExchangeIdentity`

2 required methods:
- `async fn get_balance(asset: Option<Asset>, account_type) -> ExchangeResult<Vec<Balance>>`
- `async fn get_account_info(account_type) -> ExchangeResult<AccountInfo>`

#### `traits/positions.rs` — `Positions: ExchangeIdentity`

3 required methods:
- `async fn get_positions(symbol: Option<Symbol>, account_type) -> ExchangeResult<Vec<Position>>`
- `async fn get_funding_rate(symbol, account_type) -> ExchangeResult<FundingRate>`
- `async fn set_leverage(symbol, leverage: u32, account_type) -> ExchangeResult<()>`

#### `traits/auth.rs` — `ExchangeAuth: Send + Sync`

NOT in ExchangeIdentity hierarchy — standalone.

**`Credentials` (struct)** — `#[derive(Clone)]`

Fields: `api_key: String`, `api_secret: String`, `passphrase: Option<String>`

Constructors: `new(api_key, api_secret)`, `with_passphrase(passphrase)`

**`AuthRequest<'a>` (struct)**

Fields: `method: &'a str`, `path: &'a str`, `query: Option<&'a str>`, `body: Option<&'a str>`, `headers: HashMap<String, String>`, `query_params: HashMap<String, String>`

**`SignatureLocation` (enum)** — `#[derive(Debug, Clone, Copy, PartialEq, Eq)]`

Variants: `Headers`, `QueryParams`

**`ExchangeAuth` trait:**
- `fn sign_request(&self, credentials: &Credentials, req: &mut AuthRequest<'_>) -> ExchangeResult<()>` — required
- `fn signature_location(&self) -> SignatureLocation` — default: `Headers`

#### `traits/extensions.rs` — Extension Traits (optional)

These are **supertrait-constrained** and have full default implementations returning `UnsupportedOperation`.

**`BatchOperations: Trading`**
- `async fn create_orders_batch(requests: Vec<CreateOrderRequest>) -> ExchangeResult<Vec<Order>>` — default: sequential
- `async fn cancel_orders_batch(order_ids: Vec<String>, symbol, account_type) -> ExchangeResult<Vec<Order>>` — default: sequential

**`AdvancedOrders: Trading`**
- `async fn create_trailing_stop(symbol, side, quantity, callback_rate, activation_price, account_type) -> ExchangeResult<Order>` — default: UnsupportedOperation
- `async fn create_stop_limit_order(symbol, side, quantity, stop_price, limit_price, account_type) -> ExchangeResult<Order>` — default: UnsupportedOperation
- `async fn create_oco_order(symbol, side, quantity, price, stop_price, stop_limit_price, account_type) -> ExchangeResult<Order>` — default: UnsupportedOperation

**`MarginTrading: Account`**
- `async fn borrow_margin(asset, amount, symbol: Option<Symbol>) -> ExchangeResult<MarginBorrowResult>` — default: UnsupportedOperation
- `async fn repay_margin(asset, amount, symbol: Option<Symbol>) -> ExchangeResult<MarginRepayResult>` — default: UnsupportedOperation
- `async fn get_margin_info(asset: Option<Asset>) -> ExchangeResult<Vec<MarginLoan>>` — default: UnsupportedOperation
- `async fn set_margin_type(symbol, margin_type) -> ExchangeResult<()>` — default: UnsupportedOperation

**`Transfers: Account`**
- `async fn transfer(asset, amount, from_account, to_account) -> ExchangeResult<TransferResult>` — required
- `async fn get_transfer_history(start_time, end_time, limit) -> ExchangeResult<Vec<TransferHistory>>` — default: UnsupportedOperation

#### `traits/websocket.rs` — WebSocket Traits

**`WebSocketConnector: Send + Sync`**
- `async fn connect(&mut self, account_type) -> WebSocketResult<()>` — required
- `async fn disconnect(&mut self) -> WebSocketResult<()>` — required
- `fn connection_status(&self) -> ConnectionStatus` — required
- `async fn subscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()>` — required
- `async fn unsubscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()>` — required
- `fn event_stream(&self) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>>` — required
- `fn active_subscriptions(&self) -> Vec<SubscriptionRequest>` — required
- `fn has_subscription(&self, request: &SubscriptionRequest) -> bool` — default
- `fn ping_rtt_handle(&self) -> Option<Arc<TokioMutex<u64>>>` — default: `None`

**`WebSocketExt: WebSocketConnector`** — blanket impl for all `WebSocketConnector`

Convenience methods: `subscribe_ticker(symbol)`, `subscribe_trades(symbol)`, `subscribe_orderbook(symbol)`, `subscribe_klines(symbol, interval)`, `subscribe_orders()`, `subscribe_balance()`, `subscribe_positions()`

#### `traits/mod.rs` — Composite Trait

**`CoreConnector`** = `ExchangeIdentity + MarketData + Trading + Account + Positions + Send + Sync`

Blanket implementation: automatically implemented for all types satisfying the bound.

---

### 3.3 Utils (`core/utils/`)

- `crypto.rs`: `hmac_sha256`, `hmac_sha256_hex`, `hmac_sha384`, `hmac_sha512`, `sha256`, `sha512`
- `encoding.rs`: `encode_base64`, `encode_hex`, `encode_hex_lower`
- `time.rs`: `timestamp_millis`, `timestamp_seconds`, `timestamp_iso8601`
- `rate_limiter.rs`: `SimpleRateLimiter`, `WeightRateLimiter`

### 3.4 HTTP Transport (`core/http/`)

- `HttpClient` — wraps `reqwest::Client`, timeout-configurable, used by all connectors

### 3.5 WebSocket Base (`core/websocket/`)

- `base_websocket.rs` — base WebSocket implementation used by exchange connectors

---

## 4. Connector Manager (`digdigdig3/src/connector_manager/`)

### 4.1 `AnyConnector` Enum (`connector.rs`)

Unified enum wrapping all active connectors in `Arc<T>`.

**Currently implements:** `ExchangeIdentity` + `MarketData` (5+6 methods, delegated via macro/match)

**NOT YET implemented in AnyConnector:** `Trading`, `Account`, `Positions` (commented out, TODO)

#### CEX Variants (19)
`Binance(Arc<BinanceConnector>)`, `Bybit(Arc<BybitConnector>)`, `OKX(Arc<OkxConnector>)`, `KuCoin(Arc<KuCoinConnector>)`, `Kraken(Arc<KrakenConnector>)`, `Coinbase(Arc<CoinbaseConnector>)`, `GateIO(Arc<GateioConnector>)`, `Bitfinex(Arc<BitfinexConnector>)`, `Bitstamp(Arc<BitstampConnector>)`, `Gemini(Arc<GeminiConnector>)`, `MEXC(Arc<MexcConnector>)`, `HTX(Arc<HtxConnector>)`, `Bitget(Arc<BitgetConnector>)`, `BingX(Arc<BingxConnector>)`, `Phemex(Arc<PhemexConnector>)`, `CryptoCom(Arc<CryptoComConnector>)`, `Upbit(Arc<UpbitConnector>)`, `Deribit(Arc<DeribitConnector>)`, `HyperLiquid(Arc<HyperliquidConnector>)`

#### DEX Variants (7)
`Lighter(Arc<LighterConnector>)`, `Uniswap(Arc<UniswapConnector>)`, `Jupiter(Arc<JupiterConnector>)`, `Raydium(Arc<RaydiumConnector>)`, `Gmx(Arc<GmxConnector>)`, `Paradex(Arc<ParadexConnector>)`, `Dydx(Arc<DydxConnector>)`

#### Stocks US Variants (5)
`Polygon(Arc<PolygonConnector>)`, `Finnhub(Arc<FinnhubConnector>)`, `Tiingo(Arc<TiingoConnector>)`, `Twelvedata(Arc<TwelvedataConnector>)`, `Alpaca(Arc<AlpacaConnector>)`

#### Stocks India Variants (5)
`AngelOne(Arc<AngelOneConnector>)`, `Zerodha(Arc<ZerodhaConnector>)`, `Upstox(Arc<UpstoxConnector>)`, `Dhan(Arc<DhanConnector>)`, `Fyers(Arc<FyersConnector>)`

#### Stocks Other Variants (4)
`JQuants(Arc<JQuantsConnector>)`, `Krx(Arc<KrxConnector>)`, `Moex(Arc<MoexConnector>)`, `Tinkoff(Arc<TinkoffConnector>)`

#### Forex Variants (3)
`Oanda(Arc<OandaConnector>)`, `Dukascopy(Arc<DukascopyConnector>)`, `AlphaVantage(Arc<AlphaVantageConnector>)`

#### Prediction Variants (1)
`Polymarket(Arc<PolymarketConnector>)`

#### Aggregator Variants (4)
`Ib(Arc<IBConnector>)`, `YahooFinance(Arc<YahooFinanceConnector>)`, `CryptoCompare(Arc<CryptoCompareConnector>)`, `DefiLlama(Arc<DefiLlamaConnector>)`

**Total: ~48 variants** (doc says 51, some may be in additional categories not listed above)

### 4.2 `ConnectorPool` (`pool.rs`)

```rust
pub struct ConnectorPool {
    connectors: Arc<DashMap<ExchangeId, Arc<AnyConnector>>>,
}
```

Lock-free reads via `DashMap`. Clone shares the same underlying map.

Methods: `new()`, `insert(id, connector) -> Option<Arc<AnyConnector>>`, `get(id) -> Option<Arc<AnyConnector>>`, `remove(id) -> Option<Arc<AnyConnector>>`, `contains(id) -> bool`, `len() -> usize`, `is_empty() -> bool`, `clear()`, `iter()`, `ids() -> Vec<ExchangeId>`

**`ConnectorPoolBuilder`** — fluent API: `.with_connector(id, connector)`, `.build()`

### 4.3 `ConnectorAggregator` (`aggregator.rs`)

High-level API over `ConnectorPool`.

Struct: `ConnectorAggregator { pool: Arc<ConnectorPool> }`

Methods:
- `new(pool) -> Self`
- `pool() -> &ConnectorPool`
- `available_exchanges() -> Vec<ExchangeId>`
- `get_price(id, symbol, account_type) -> ExchangeResult<Price>`
- `get_ticker(id, symbol, account_type) -> ExchangeResult<Ticker>`
- `get_orderbook(id, symbol, account_type, depth) -> ExchangeResult<OrderBook>`
- `get_klines(id, symbol, interval, account_type, limit, end_time) -> ExchangeResult<Vec<Kline>>`
- `get_prices_multi(ids, symbol, account_type) -> ExchangeResult<HashMap<ExchangeId, Price>>`
- `get_best_bid_ask(ids, symbol, account_type) -> ExchangeResult<BestBidAsk>`

**Trading/Account methods are commented out** — pending `Trading`/`Account` delegation in `AnyConnector`.

**`BestBidAsk` (struct):** `bid: f64`, `bid_exchange: ExchangeId`, `ask: f64`, `ask_exchange: ExchangeId`, `spread: f64`, `spread_percent: f64`

### 4.4 `ConnectorFactory` (`factory.rs`)

Static factory for creating `Arc<AnyConnector>` by `ExchangeId`.

```rust
pub struct ConnectorFactory;
```

Methods: `create_public(id: ExchangeId) -> ExchangeResult<Arc<AnyConnector>>`, `create_authenticated(id, credentials) -> ExchangeResult<Arc<AnyConnector>>`

Constructor patterns across connectors:
- **Pattern A:** `::public(testnet: bool)` async — Binance, Bybit, OKX, BingX, Bitfinex, Deribit, Dydx
- **Pattern B:** `::public()` async — Bitget, Bitstamp, Coinbase, Gemini
- **Pattern C:** `::new()` sync — lightweight data feeds (AlphaVantage, Fred)
- **Pattern D:** `::new(api_key)` async — Jupiter (requires API key since Oct 2025)
- **Pattern E:** `::new(credentials, rate_limit)` async — Coinglass
- **Pattern F:** `::from_env()` sync — Fred, Alpaca (load from env vars)

### 4.5 `ConnectorRegistry` (`registry.rs`)

Metadata registry for all connectors.

Types exported: `AuthType`, `ConnectorCategory`, `ConnectorMetadata`, `ConnectorRegistry`, `Features`, `RateLimits`

### 4.6 `ConnectorConfig` (`config.rs`)

Types exported: `ConnectorConfig`, `ConnectorConfigManager`, `ExchangeCredentials`

---

## 5. Connector Implementations (`digdigdig3/src/crypto/`)

### 5.1 Module Structure

```
crypto/
├── mod.rs          — pub mod cex, dex, swap
├── cex/            — Centralized exchanges
│   ├── mod.rs
│   ├── binance/    — {auth, connector, endpoints, mod, parser, websocket}.rs
│   ├── bybit/
│   ├── okx/
│   ├── kucoin/
│   ├── kraken/
│   ├── coinbase/
│   ├── gateio/
│   ├── bitfinex/
│   ├── bitstamp/
│   ├── gemini/
│   ├── mexc/
│   ├── htx/
│   ├── bitget/
│   ├── bingx/
│   ├── phemex/
│   ├── crypto_com/
│   ├── upbit/
│   ├── deribit/
│   ├── hyperliquid/
│   ├── bithumb/     — DISABLED (infrastructure issues), files present with _tests_*.rs
│   └── vertex/      — DISABLED (exchange shut down Aug 14 2025), files present with _tests_*.rs
├── dex/            — Decentralized exchanges
│   ├── mod.rs
│   ├── lighter/
│   ├── jupiter/
│   ├── gmx/
│   ├── paradex/
│   └── dydx/
└── swap/           — On-chain swap protocols (require RPC)
    ├── mod.rs
    ├── uniswap/    — Ethereum, requires alloy feature
    └── raydium/    — Solana, raw WS to RPC
```

### 5.2 Standard Connector Module Layout

Every connector follows the same 5-file pattern:

| File | Contents |
|------|----------|
| `mod.rs` | `pub use connector::XxxConnector;` (and optionally re-exports parser/auth types) |
| `endpoints.rs` | URL constants, endpoint enum, symbol formatting functions, interval mapping |
| `auth.rs` | `XxxAuth` struct implementing `ExchangeAuth` trait |
| `parser.rs` | `XxxParser` struct with `parse_klines()`, `parse_orderbook()`, `parse_ticker()`, etc. |
| `connector.rs` | `XxxConnector` struct implementing `ExchangeIdentity`, `MarketData`, `Trading`, `Account`, `Positions` |
| `websocket.rs` | `XxxWebSocket` struct implementing `WebSocketConnector` |

### 5.3 Connector Structure (KuCoin as reference)

```rust
pub struct KuCoinConnector {
    http: HttpClient,
    auth: Option<KuCoinAuth>,
    urls: KuCoinUrls,
    testnet: bool,
    rate_limiter: Arc<Mutex<WeightRateLimiter>>,
}
```

Constructors: `new(credentials: Option<Credentials>, testnet: bool) -> ExchangeResult<Self>`, `public(testnet: bool) -> ExchangeResult<Self>`, `authenticated(credentials, testnet) -> ExchangeResult<Self>`

Implements:
- `ExchangeIdentity` — `exchange_id()`, `is_testnet()`, `supported_account_types()`, `metrics()`
- `MarketData` — all 6 methods including `get_exchange_info`
- `Trading` — all 5 methods
- `Account` — both methods
- `Positions` — all 3 methods

### 5.4 All Active CEX Connectors (19)

| Connector | Location | Traits Implemented |
|-----------|----------|-------------------|
| `BinanceConnector` | `crypto::cex::binance` | ExchangeIdentity, MarketData, Trading, Account, Positions |
| `BybitConnector` | `crypto::cex::bybit` | ExchangeIdentity, MarketData, Trading, Account, Positions |
| `OkxConnector` | `crypto::cex::okx` | ExchangeIdentity, MarketData, Trading, Account, Positions |
| `KuCoinConnector` | `crypto::cex::kucoin` | ExchangeIdentity, MarketData, Trading, Account, Positions |
| `KrakenConnector` | `crypto::cex::kraken` | ExchangeIdentity, MarketData, Trading, Account, Positions |
| `CoinbaseConnector` | `crypto::cex::coinbase` | ExchangeIdentity, MarketData, Trading, Account, Positions |
| `GateioConnector` | `crypto::cex::gateio` | ExchangeIdentity, MarketData, Trading, Account, Positions |
| `BitfinexConnector` | `crypto::cex::bitfinex` | ExchangeIdentity, MarketData, Trading, Account, Positions |
| `BitstampConnector` | `crypto::cex::bitstamp` | ExchangeIdentity, MarketData, Trading, Account, Positions |
| `GeminiConnector` | `crypto::cex::gemini` | ExchangeIdentity, MarketData, Trading, Account, Positions |
| `MexcConnector` | `crypto::cex::mexc` | ExchangeIdentity, MarketData, Trading, Account, Positions |
| `HtxConnector` | `crypto::cex::htx` | ExchangeIdentity, MarketData, Trading, Account, Positions |
| `BitgetConnector` | `crypto::cex::bitget` | ExchangeIdentity, MarketData, Trading, Account, Positions |
| `BingxConnector` | `crypto::cex::bingx` | ExchangeIdentity, MarketData, Trading, Account, Positions |
| `PhemexConnector` | `crypto::cex::phemex` | ExchangeIdentity, MarketData, Trading, Account, Positions |
| `CryptoComConnector` | `crypto::cex::crypto_com` | ExchangeIdentity, MarketData, Trading, Account, Positions |
| `UpbitConnector` | `crypto::cex::upbit` | ExchangeIdentity, MarketData, Trading, Account, Positions |
| `DeribitConnector` | `crypto::cex::deribit` | ExchangeIdentity, MarketData, Trading, Account, Positions |
| `HyperliquidConnector` | `crypto::cex::hyperliquid` | ExchangeIdentity, MarketData, Trading, Account, Positions |

### 5.5 All Active DEX Connectors (7)

| Connector | Location | Notes |
|-----------|----------|-------|
| `LighterConnector` | `crypto::dex::lighter` | Order book DEX (Starknet) |
| `UniswapConnector` | `crypto::swap::uniswap` | Ethereum AMM, requires `onchain-ethereum` feature |
| `JupiterConnector` | `crypto::dex::jupiter` | Solana aggregator, API key required since Oct 2025 |
| `RaydiumConnector` | `crypto::swap::raydium` | Solana AMM, raw WS to Solana RPC |
| `GmxConnector` | `crypto::dex::gmx` | Arbitrum/Avalanche perpetuals |
| `ParadexConnector` | `crypto::dex::paradex` | Starknet perpetuals |
| `DydxConnector` | `crypto::dex::dydx` | Cosmos-based perpetuals |

### 5.6 Disabled Connectors

| Connector | Reason | Files |
|-----------|--------|-------|
| `BithumbConnector` | Infrastructure issues (504 errors) | Present at `crypto::cex::bithumb`, includes `_tests_integration.rs`, `_tests_websocket.rs` |
| `VertexConnector` | Exchange shut down Aug 14, 2025 | Present at `crypto::cex::vertex`, includes `_tests_integration.rs`, `_tests_websocket.rs` |

---

## 6. Other Connector Categories

### 6.1 Stocks (`stocks/`)

**US (`stocks::us`):** `PolygonConnector`, `FinnhubConnector`, `TiingoConnector`, `TwelvedataConnector`, `AlpacaConnector`

**India (`stocks::india`):** `AngelOneConnector`, `ZerodhaConnector`, `UpstoxConnector`, `DhanConnector`, `FyersConnector`

**Japan (`stocks::japan`):** `JQuantsConnector`

**Korea (`stocks::korea`):** `KrxConnector`

**Russia (`stocks::russia`):** `MoexConnector`, `TinkoffConnector`

### 6.2 Forex (`forex/`)

`OandaConnector`, `DukascopyConnector`, `AlphaVantageConnector`

### 6.3 Aggregators (`aggregators/`)

`IBConnector` (Interactive Brokers), `YahooFinanceConnector`, `CryptoCompareConnector`, `DefiLlamaConnector`

### 6.4 Prediction (`prediction/`)

`PolymarketConnector`

### 6.5 Intelligence Feeds (`intelligence_feeds/`) — 88 feeds across 20 categories

Categories: `crypto` (Coinglass, CoinGecko), `economic` (FRED, World Bank, DBnomics, OECD, Eurostat, ECB, IMF, BIS, Bundesbank, ECOS, CBR, BoE), `us_gov` (EIA, BLS, BEA, Census, SEC EDGAR, Congress, FBI Crime, USASpending, SAM.gov), `financial` (AlphaVantage, Finnhub, NewsAPI, OpenFIGI), `trade` (UN COMTRADE, EU TED), `conflict` (ACLED, GDELT, UCDP, ReliefWeb, UNHCR), `maritime` (AISStream, IMF PortWatch, AIS), `aviation` (ADS-B Exchange, OpenSky, AviationStack, Wingbits, FAA), `space` (Launch Library, SpaceX, Space-Track, NASA, Sentinel Hub), `environment` (NOAA, OpenAQ, OpenWeatherMap, NASA FIRMS, NASA EONET, Global Forest Watch, USGS Earthquake, GDACS), `cyber` (Shodan, Censys, VirusTotal, NVD, AlienVault OTX, Cloudflare Radar, RIPE NCC), `c2intel_feeds`, `feodo_tracker`, `governance` (UK Parliament, EU Parliament), `sanctions` (OpenSanctions, OFAC, INTERPOL), `corporate` (GLEIF, OpenCorporates, UK Companies House), `demographics` (UN Population, WHO, Wikipedia, UN OCHA), `academic` (arXiv, Semantic Scholar), `hacker_news`, `rss_proxy`, `prediction` (PredictIt, Polymarket)

**Feed Manager** (`feed_manager`): registry, metadata, and factory for all 88 feeds.

---

## 7. Trait Hierarchy Diagram

```
Send + Sync
    └── ExchangeIdentity
            ├── MarketData           (public data: price, OB, klines, ticker, ping)
            ├── Trading              (private: market/limit order, cancel, get_order, open_orders)
            ├── Account              (private: balance, account_info)
            └── Positions            (futures: positions, funding_rate, set_leverage)

Extensions (supertrait-constrained, full default impls):
    Trading ─► BatchOperations      (create_orders_batch, cancel_orders_batch)
    Trading ─► AdvancedOrders       (trailing stop, stop limit, OCO)
    Account ─► MarginTrading        (borrow, repay, margin_info, set_margin_type)
    Account ─► Transfers            (transfer, get_transfer_history)

Composite:
    CoreConnector = ExchangeIdentity + MarketData + Trading + Account + Positions

WebSocket (parallel hierarchy):
    WebSocketConnector
        └── WebSocketExt (blanket impl)

Auth (standalone):
    ExchangeAuth (sign_request, signature_location)
```

---

## 8. AnyConnector Trait Coverage (Current State)

| Trait | Delegated in AnyConnector? | Status |
|-------|--------------------------|--------|
| `ExchangeIdentity` | Yes | Working |
| `MarketData` | Yes | Working (5/6 — get_exchange_info included) |
| `Trading` | **No** | Commented out, TODO |
| `Account` | **No** | Commented out, TODO |
| `Positions` | **No** | Commented out, TODO |

**Key gap:** `Trading`, `Account`, and `Positions` are NOT delegated through `AnyConnector`. Individual connector structs implement them, but the pool/aggregator layer cannot call them through `AnyConnector`.

---

## 9. Important Notes for Refactoring

1. **`ExchangeError` has duplicate variants:** Both `Parse(String)` and `ParseError(String)` exist — likely a historical artifact.

2. **`AnyConnector` is not `CoreConnector`:** Because `Trading`, `Account`, `Positions` are not delegated, `AnyConnector` cannot satisfy `CoreConnector`. Aggregator trading/account methods are commented out pending this delegation.

3. **`Trading::cancel_order` signature inconsistency:** In `extensions.rs` `BatchOperations::cancel_orders_batch` calls `self.cancel_order(&order_id, symbol.clone(), account_type)` with args `(order_id, symbol, account_type)`, but `Trading::cancel_order` signature in `trading.rs` is `cancel_order(symbol, order_id, account_type)` — different argument order. This is a bug.

4. **`BatchOperations::create_orders_batch` calls `create_limit_order`** — but the core `Trading` trait has `limit_order`, not `create_limit_order`. The extension has a naming inconsistency with the core trait.

5. **Disabled connectors' files were deleted from `exchanges/` (old v5 path) per git status** — they now live at `crypto/cex/bithumb/` and `crypto/cex/vertex/` in the new module layout.

6. **`ConnectorPool` clone semantics:** Clone shares the same `Arc<DashMap>` — both clones see each other's inserts.

7. **Intelligence feeds are NOT wrapped in `AnyConnector`** — they live entirely in `intelligence_feeds/` and are managed separately by `feed_manager`.

8. **`Coinglass` and related data-only providers** have intelligence feed counterparts AND `ExchangeId` variants (e.g. `ExchangeId::Coinglass`) — two separate implementations exist for some providers.
