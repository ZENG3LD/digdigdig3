# GMX WebSocket & Real-Time Data

## Overview

GMX **does not provide a native WebSocket API** for real-time market data streaming. As a decentralized exchange, GMX operates entirely on-chain, and real-time data must be obtained through alternative methods.

## Alternative Real-Time Data Sources

### 1. Polling REST API

The most straightforward approach is polling GMX's REST endpoints.

#### Recommended Polling Intervals

**High-Frequency Data (1-5 seconds):**
- Price tickers
- Signed prices (for trading)
- Open interest

**Medium-Frequency Data (30-60 seconds):**
- Market info (liquidity, funding rates)
- Position updates (if using Reader contract)

**Low-Frequency Data (5-15 minutes):**
- Markets list
- Token metadata
- APY/performance metrics

#### Implementation Example

```rust
use tokio::time::{interval, Duration};

async fn poll_tickers(client: &GmxClient) {
    let mut ticker_interval = interval(Duration::from_secs(2));

    loop {
        ticker_interval.tick().await;

        match client.get_tickers().await {
            Ok(tickers) => {
                for (symbol, ticker) in tickers {
                    println!("{}: {} - {}", symbol, ticker.min_price, ticker.max_price);
                }
            }
            Err(e) => eprintln!("Failed to fetch tickers: {}", e),
        }
    }
}
```

#### Pros and Cons

**Pros:**
- Simple implementation
- No WebSocket connection management
- Works with existing REST client
- Predictable resource usage

**Cons:**
- Higher latency (1-5 seconds delay)
- More bandwidth usage than WebSocket
- Polling overhead during idle periods
- May miss rapid price movements

---

### 2. Blockchain Event Subscriptions

Subscribe to smart contract events in real-time via WebSocket RPC.

#### Event Types

**Order Events:**
- `OrderCreated` - New order submitted
- `OrderExecuted` - Order filled
- `OrderCancelled` - Order cancelled
- `OrderFrozen` - Order frozen (insufficient liquidity)
- `OrderUpdated` - Order parameters changed

**Position Events:**
- `PositionIncrease` - Position size increased
- `PositionDecrease` - Position size decreased
- `InsolventClose` - Position liquidated

**Market Events:**
- `SwapExecuted` - Token swap completed
- `DepositExecuted` - Liquidity deposited
- `WithdrawalExecuted` - Liquidity withdrawn

#### WebSocket RPC Connection

**Arbitrum WebSocket:**
```
wss://arb1.arbitrum.io/ws
```

**Avalanche WebSocket:**
```
wss://api.avax.network/ext/bc/C/ws
```

**Private RPC Providers (Recommended):**
- Alchemy: `wss://arb-mainnet.g.alchemy.com/v2/{API_KEY}`
- Infura: `wss://arbitrum-mainnet.infura.io/ws/v3/{API_KEY}`
- QuickNode: `wss://{ENDPOINT}.arbitrum-mainnet.quiknode.pro/{TOKEN}/`

#### Implementation Example

```rust
use ethers::prelude::*;

async fn subscribe_to_order_events() -> Result<()> {
    // Connect to WebSocket RPC
    let ws = Provider::<Ws>::connect("wss://arb1.arbitrum.io/ws").await?;

    // ExchangeRouter address on Arbitrum
    let exchange_router: Address = "0x602b805EedddBbD9ddff44A7dcBD46cb07849685".parse()?;

    // Filter for OrderExecuted events
    let event_filter = Filter::new()
        .address(exchange_router)
        .event("OrderExecuted(bytes32,uint256,uint256)")
        .from_block(BlockNumber::Latest);

    // Subscribe to logs
    let mut stream = ws.subscribe_logs(&event_filter).await?;

    // Process events
    while let Some(log) = stream.next().await {
        println!("Order executed!");
        println!("  Block: {}", log.block_number.unwrap());
        println!("  Tx: {}", log.transaction_hash.unwrap());

        // Decode event data
        let order_key = H256::from_slice(&log.topics[1]);
        println!("  Order key: {:#x}", order_key);

        // Fetch order details from DataStore if needed
        let order = fetch_order_details(order_key).await?;
        println!("  Market: {}", order.market);
        println!("  Size: {}", order.size_delta_usd);
    }

    Ok(())
}
```

#### Event Signatures

**Common GMX Events:**

```solidity
// Order events
event OrderCreated(
    bytes32 indexed key,
    Order.Props order
);

event OrderExecuted(
    bytes32 indexed key,
    uint256 executionPrice,
    uint256 indexTokenPrice
);

event OrderCancelled(
    bytes32 indexed key,
    string reason
);

// Position events
event PositionIncrease(
    bytes32 indexed key,
    address account,
    address market,
    address collateralToken,
    bool isLong,
    uint256 executionPrice,
    uint256 sizeDeltaUsd,
    uint256 sizeDeltaInTokens,
    uint256 collateralDeltaAmount
);

event PositionDecrease(
    bytes32 indexed key,
    address account,
    address market,
    address collateralToken,
    bool isLong,
    uint256 executionPrice,
    uint256 sizeDeltaUsd,
    uint256 sizeDeltaInTokens,
    uint256 collateralDeltaAmount
);

// Swap events
event SwapExecuted(
    address indexed tokenIn,
    address indexed tokenOut,
    uint256 amountIn,
    uint256 amountOut,
    uint256 price
);
```

#### Filtering Events by Account

Subscribe only to events for your account:

```rust
let account: Address = "0x...".parse()?;

let position_filter = Filter::new()
    .address(exchange_router)
    .event("PositionIncrease(bytes32,address,address,address,bool,uint256,uint256,uint256,uint256)")
    .topic1(account) // Filter by account
    .from_block(BlockNumber::Latest);

let mut stream = ws.subscribe_logs(&position_filter).await?;
```

#### Pros and Cons

**Pros:**
- True real-time updates (sub-second latency)
- Low bandwidth (only events, not full state)
- Decentralized (uses blockchain directly)
- No polling overhead

**Cons:**
- Requires WebSocket RPC provider
- Complex event decoding
- Need to reconstruct state from events
- Potential missed events on disconnection

---

### 3. GraphQL Subscriptions (Subsquid)

Subsquid **may** support GraphQL subscriptions for real-time queries.

#### GraphQL Subscription Endpoint

**Arbitrum:**
```
wss://gmx.squids.live/gmx-synthetics-arbitrum:prod/api/graphql
```

**Avalanche:**
```
wss://gmx.squids.live/gmx-synthetics-avalanche:prod/api/graphql
```

**Note:** Verify subscription support by checking Subsquid's documentation or testing the endpoint.

#### Potential Subscription Schema

```graphql
subscription OnPositionUpdate($account: String!) {
  positions(where: { account: $account }) {
    id
    account
    market
    isLong
    sizeInUsd
    collateralAmount
    realizedPnl
    unrealizedPnl
    updatedAt
  }
}
```

#### Implementation Considerations

**If subscriptions are supported:**
- Use a GraphQL client library with subscription support (e.g., `graphql-ws`, `subscriptions-transport-ws`)
- Handle reconnection logic
- Manage subscription lifecycle

**If subscriptions are NOT supported:**
- Fall back to polling GraphQL queries
- Poll every 5-30 seconds for position/order updates

#### Checking Subscription Support

```rust
// Test WebSocket connection
use tungstenite::{connect, Message};

let (mut socket, _) = connect(
    "wss://gmx.squids.live/gmx-synthetics-arbitrum:prod/api/graphql"
)?;

// Send subscription initialization
let init_msg = r#"{"type":"connection_init"}"#;
socket.write_message(Message::Text(init_msg.into()))?;

// Read response
let msg = socket.read_message()?;
println!("Response: {}", msg);

// If successful, subscriptions are supported
```

#### Pros and Cons

**Pros (if supported):**
- High-level abstraction (positions, orders)
- Filtered by account
- Lower latency than polling
- Easier state management than raw events

**Cons:**
- May not be supported by all Subsquid deployments
- Dependent on third-party infrastructure
- Potential subscription limits

---

### 4. Oracle Price Feeds (Chainlink)

GMX uses Chainlink and other oracles for price data. You can subscribe to oracle updates.

#### Chainlink Price Feed Events

```rust
let chainlink_feed: Address = "0x...".parse()?; // ETH/USD feed

let price_update_filter = Filter::new()
    .address(chainlink_feed)
    .event("AnswerUpdated(int256,uint256,uint256)")
    .from_block(BlockNumber::Latest);

let mut stream = ws.subscribe_logs(&price_update_filter).await?;

while let Some(log) = stream.next().await {
    let price = i256::from_raw(U256::from(log.topics[1]));
    let updated_at = U256::from(log.topics[2]);
    println!("Price updated: {} at {}", price, updated_at);
}
```

#### Pros and Cons

**Pros:**
- Real-time price updates
- Direct from oracle source
- Independent of GMX API

**Cons:**
- Not GMX-specific prices (no min/max spread)
- Need to track multiple feeds
- Oracle update frequency varies

---

## Recommended Hybrid Approach

Combine multiple methods for optimal real-time coverage:

### Tier 1: Critical Real-Time Data

**Blockchain Events (WebSocket RPC):**
- Order executions (your account)
- Position updates (your account)
- Liquidation alerts

### Tier 2: Price Updates

**Polling REST API (2-5 seconds):**
- GMX tickers (min/max prices)
- Signed prices (for trading)

**Alternative: Oracle Events**
- Chainlink price feeds
- Lower latency but no GMX spread

### Tier 3: Market State

**Polling REST API (30-60 seconds):**
- Market info (liquidity, OI)
- Funding rates
- Pool composition

### Tier 4: Historical/Aggregated Data

**GraphQL Polling (5-15 minutes):**
- Position history
- Trade history
- Account statistics

### Implementation Architecture

```rust
struct RealtimeDataManager {
    // WebSocket for blockchain events
    event_subscriber: EventSubscriber,

    // REST API poller for prices
    price_poller: PricePoller,

    // REST API poller for market data
    market_poller: MarketPoller,

    // GraphQL poller for historical data
    history_poller: HistoryPoller,

    // Unified data channel
    data_tx: Sender<RealtimeUpdate>,
}

enum RealtimeUpdate {
    OrderExecuted(OrderExecution),
    PositionChanged(PositionUpdate),
    PriceUpdate(PriceTick),
    MarketInfoUpdate(MarketInfo),
    HistoricalData(HistoricalRecord),
}

impl RealtimeDataManager {
    async fn start(&mut self) {
        // Spawn event subscriber
        tokio::spawn(async move {
            self.event_subscriber.subscribe_orders().await;
        });

        // Spawn price poller
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(2));
            loop {
                interval.tick().await;
                let prices = self.price_poller.fetch_prices().await;
                self.data_tx.send(RealtimeUpdate::PriceUpdate(prices));
            }
        });

        // Spawn market data poller
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                let market_info = self.market_poller.fetch_markets().await;
                self.data_tx.send(RealtimeUpdate::MarketInfoUpdate(market_info));
            }
        });

        // Spawn history poller
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(300));
            loop {
                interval.tick().await;
                let history = self.history_poller.fetch_history().await;
                self.data_tx.send(RealtimeUpdate::HistoricalData(history));
            }
        });
    }
}
```

---

## Connection Management

### WebSocket Reconnection

WebSocket connections can drop. Implement automatic reconnection:

```rust
async fn maintain_websocket_connection<F>(
    url: &str,
    handler: F,
) -> Result<()>
where
    F: Fn(Log) -> BoxFuture<'static, Result<()>>,
{
    let mut backoff = Duration::from_secs(1);

    loop {
        match connect_and_subscribe(url, &handler).await {
            Ok(_) => {
                // Connection closed normally, reset backoff
                backoff = Duration::from_secs(1);
                log::warn!("WebSocket disconnected, reconnecting...");
            }
            Err(e) => {
                log::error!("WebSocket error: {}, retrying in {:?}", e, backoff);
                tokio::time::sleep(backoff).await;

                // Exponential backoff
                backoff = (backoff * 2).min(Duration::from_secs(60));
            }
        }
    }
}

async fn connect_and_subscribe<F>(
    url: &str,
    handler: &F,
) -> Result<()>
where
    F: Fn(Log) -> BoxFuture<'static, Result<()>>,
{
    let ws = Provider::<Ws>::connect(url).await?;

    let filter = Filter::new()
        .address(exchange_router)
        .event("OrderExecuted(bytes32,uint256,uint256)")
        .from_block(BlockNumber::Latest);

    let mut stream = ws.subscribe_logs(&filter).await?;

    while let Some(log) = stream.next().await {
        handler(log).await?;
    }

    Ok(())
}
```

### Heartbeat/Ping

Keep WebSocket connections alive:

```rust
async fn send_heartbeat(ws: &mut WebSocket) {
    let mut interval = interval(Duration::from_secs(30));

    loop {
        interval.tick().await;
        if let Err(e) = ws.send(Message::Ping(vec![])).await {
            log::error!("Failed to send ping: {}", e);
            break;
        }
    }
}
```

### Missed Event Handling

If disconnected, sync missed events:

```rust
async fn sync_missed_events(
    last_seen_block: u64,
    current_block: u64,
) -> Result<Vec<Log>> {
    let provider = Provider::<Http>::try_from("https://arb1.arbitrum.io/rpc")?;

    let filter = Filter::new()
        .address(exchange_router)
        .event("OrderExecuted(bytes32,uint256,uint256)")
        .from_block(last_seen_block + 1)
        .to_block(current_block);

    let logs = provider.get_logs(&filter).await?;
    Ok(logs)
}
```

---

## Candlestick Streaming

GMX provides historical candlesticks via REST, but not real-time streaming.

### Approach 1: Poll Candlestick Endpoint

```rust
async fn stream_candlesticks(symbol: &str, period: &str) {
    let mut interval = interval(Duration::from_secs(60)); // 1 minute
    let mut last_candle: Option<Candle> = None;

    loop {
        interval.tick().await;

        let candles = fetch_candles(symbol, period, 1).await?;
        let latest = candles.first();

        if let Some(new_candle) = latest {
            if last_candle.as_ref() != Some(new_candle) {
                println!("New candle: {:?}", new_candle);
                last_candle = Some(new_candle.clone());
            }
        }
    }
}
```

### Approach 2: Build Candles from Ticks

```rust
struct CandleBuilder {
    period: Duration,
    current: Option<Candle>,
    last_close_time: Instant,
}

impl CandleBuilder {
    fn add_tick(&mut self, price: f64, timestamp: Instant) -> Option<Candle> {
        let elapsed = timestamp.duration_since(self.last_close_time);

        if elapsed >= self.period {
            // Close current candle and start new one
            let completed = self.current.take();
            self.current = Some(Candle {
                open: price,
                high: price,
                low: price,
                close: price,
                timestamp,
            });
            self.last_close_time = timestamp;
            return completed;
        }

        // Update current candle
        if let Some(ref mut candle) = self.current {
            candle.high = candle.high.max(price);
            candle.low = candle.low.min(price);
            candle.close = price;
        } else {
            self.current = Some(Candle {
                open: price,
                high: price,
                low: price,
                close: price,
                timestamp,
            });
        }

        None
    }
}
```

---

## Implementation Checklist

### WebSocket Module (`websocket.rs`)

- [ ] Blockchain event subscription (via WebSocket RPC)
- [ ] Event filtering by contract address
- [ ] Event filtering by account
- [ ] Event decoding (OrderExecuted, PositionIncrease, etc.)
- [ ] Auto-reconnection logic
- [ ] Exponential backoff on errors
- [ ] Heartbeat/ping messages
- [ ] Missed event synchronization
- [ ] Connection health monitoring

### Real-Time Data Manager

- [ ] Multi-source data aggregation
- [ ] Unified event channel
- [ ] Price polling task
- [ ] Market data polling task
- [ ] Event subscription task
- [ ] Data deduplication
- [ ] Timestamp synchronization

### Candlestick Streaming

- [ ] Candlestick polling (REST API)
- [ ] Tick-based candle building
- [ ] Multi-timeframe support
- [ ] Candle completion detection
- [ ] Historical backfill on startup

---

## Performance Considerations

### Latency Comparison

| Method | Typical Latency | Best For |
|--------|----------------|----------|
| WebSocket RPC Events | 100-500ms | Order executions, positions |
| REST Polling (2s) | 1-3 seconds | Price updates |
| REST Polling (30s) | 15-45 seconds | Market info |
| GraphQL Polling | 5-60 seconds | Historical data |

### Bandwidth Usage

| Method | Bandwidth | Notes |
|--------|-----------|-------|
| WebSocket Events | ~1-10 KB/s | Depends on market activity |
| REST Polling (2s) | ~5-50 KB/s | Depends on endpoint |
| GraphQL Polling | ~10-100 KB/s | Depends on query complexity |

### Resource Usage

**WebSocket Connections:**
- 1 connection per RPC provider
- Low CPU usage
- Minimal memory footprint

**REST Polling:**
- Higher CPU (HTTP overhead)
- Moderate memory (response caching)

**Recommendation:**
- Use WebSocket events for critical data
- Poll REST API for price/market data
- Limit concurrent polling tasks (3-5 max)

---

## Sources

Since GMX lacks native WebSocket API documentation, this document synthesizes:
- Blockchain RPC capabilities
- GraphQL subscription standards
- Event-driven architecture patterns
- Best practices for decentralized data sources

**References:**
- [GMX REST API](https://docs.gmx.io/docs/api/rest/)
- [GMX Synthetics Contracts](https://github.com/gmx-io/gmx-synthetics)
- [Ethers.rs WebSocket Provider](https://docs.rs/ethers/latest/ethers/providers/struct.Provider.html)
- [GraphQL Subscriptions Spec](https://graphql.org/learn/subscriptions/)
- [Subsquid Documentation](https://docs.sqd.dev/)
