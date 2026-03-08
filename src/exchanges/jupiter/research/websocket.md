# Jupiter WebSocket API

## Overview

**Status:** Jupiter does NOT provide native WebSocket API for real-time price updates or swap notifications.

Jupiter's official API is **REST-only** (HTTP/HTTPS). However, real-time data can be obtained through alternative methods.

---

## Official Position

### REST API Only

Jupiter provides:
- RESTful endpoints for quotes, swaps, prices, tokens
- Polling-based price updates
- No native streaming or WebSocket support

### Why No WebSocket?

Jupiter focuses on:
1. **Aggregation**: Finding best routes across multiple DEXs
2. **Transaction Building**: Constructing optimal swap transactions
3. **Execution**: Submitting transactions (Ultra tier)

Real-time price streaming is better served by:
- Solana RPC WebSocket connections
- Specialized market data providers
- DEX-specific streaming APIs

---

## Alternative Methods for Real-Time Data

### 1. Solana RPC WebSocket

Monitor Solana accounts and programs directly via Solana RPC.

#### Account Subscription

Subscribe to token account changes:

```javascript
const { Connection, PublicKey } = require('@solana/web3.js');

const connection = new Connection('wss://api.mainnet-beta.solana.com', 'confirmed');

// Subscribe to account changes
const accountPubkey = new PublicKey('So11111111111111111111111111111111111111112');

const subscriptionId = connection.onAccountChange(
  accountPubkey,
  (accountInfo, context) => {
    console.log('Account changed:', accountInfo);
    console.log('Slot:', context.slot);
  },
  'confirmed'
);

// Unsubscribe
// connection.removeAccountChangeListener(subscriptionId);
```

#### Program Subscription

Monitor Jupiter program logs:

```javascript
// Subscribe to Jupiter program logs
const jupiterProgramId = new PublicKey('JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4');

connection.onLogs(
  jupiterProgramId,
  (logs, context) => {
    console.log('Jupiter program logs:', logs);
  },
  'confirmed'
);
```

**Rust Example:**

```rust
use solana_client::pubsub_client::PubsubClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "wss://api.mainnet-beta.solana.com";
    let pubsub_client = PubsubClient::new(url).await?;

    let jupiter_program = Pubkey::from_str("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4")?;

    let (mut notifications, unsubscribe) = pubsub_client
        .logs_subscribe(
            solana_client::rpc_config::RpcTransactionLogsFilter::Mentions(vec![
                jupiter_program.to_string()
            ]),
            None,
        )
        .await?;

    while let Some(notification) = notifications.next().await {
        println!("Jupiter transaction: {:?}", notification);
    }

    Ok(())
}
```

---

### 2. Third-Party WebSocket Providers

#### bloXroute Trader API

**Features:**
- WebSocket support for Jupiter quotes
- Low-latency price streams
- Transaction monitoring

**Endpoints:**
```
wss://solana.trader-api.bloxroute.com
```

**Example:**
```javascript
const WebSocket = require('ws');

const ws = new WebSocket('wss://solana.trader-api.bloxroute.com/ws');

ws.on('open', () => {
  // Subscribe to Jupiter price stream
  ws.send(JSON.stringify({
    method: 'subscribe',
    params: {
      channel: 'jupiter_prices',
      tokens: ['So11111111111111111111111111111111111111112']
    }
  }));
});

ws.on('message', (data) => {
  const update = JSON.parse(data);
  console.log('Price update:', update);
});
```

**Documentation:** https://docs.bloxroute.com/solana/trader-api

---

#### Bitquery Jupiter API

**Features:**
- GraphQL WebSocket subscriptions
- Jupiter swap monitoring
- Token launch tracking
- OHLC data

**Endpoint:**
```
wss://streaming.bitquery.io/graphql
```

**Example:**
```javascript
const WebSocket = require('ws');

const ws = new WebSocket('wss://streaming.bitquery.io/graphql', {
  headers: {
    'Authorization': 'Bearer YOUR_API_KEY'
  }
});

const subscription = {
  type: 'start',
  payload: {
    query: `
      subscription {
        Solana {
          DEXTrades(
            where: {
              Trade: {
                Dex: {
                  ProtocolName: { is: "jupiter" }
                }
              }
            }
          ) {
            Trade {
              Buy {
                Currency {
                  Symbol
                  MintAddress
                }
                Amount
              }
              Sell {
                Currency {
                  Symbol
                  MintAddress
                }
                Amount
              }
              Price
            }
            Block {
              Time
            }
          }
        }
      }
    `
  }
};

ws.on('open', () => {
  ws.send(JSON.stringify(subscription));
});

ws.on('message', (data) => {
  const trade = JSON.parse(data);
  console.log('Jupiter trade:', trade);
});
```

**Documentation:** https://docs.bitquery.io/docs/blockchain/Solana/solana-jupiter-api/

---

### 3. Polling with REST API

Implement efficient polling using Jupiter's REST endpoints.

#### Price Polling

```rust
use std::time::Duration;
use tokio::time::interval;

pub struct PricePoller {
    client: JupiterClient,
    poll_interval: Duration,
}

impl PricePoller {
    pub fn new(client: JupiterClient, interval_ms: u64) -> Self {
        Self {
            client,
            poll_interval: Duration::from_millis(interval_ms),
        }
    }

    pub async fn start_polling(
        &self,
        mints: Vec<String>,
        callback: impl Fn(PriceUpdate),
    ) {
        let mut interval = interval(self.poll_interval);

        loop {
            interval.tick().await;

            match self.client.get_prices(&mints).await {
                Ok(prices) => {
                    for (mint, price_data) in prices {
                        callback(PriceUpdate {
                            mint: mint.clone(),
                            price: price_data.usd_price,
                            timestamp: chrono::Utc::now(),
                        });
                    }
                }
                Err(e) => {
                    eprintln!("Error fetching prices: {}", e);
                }
            }
        }
    }
}

pub struct PriceUpdate {
    pub mint: String,
    pub price: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

// Usage
let poller = PricePoller::new(client, 1000); // Poll every 1 second

poller.start_polling(
    vec![
        "So11111111111111111111111111111111111111112".to_string(),
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
    ],
    |update| {
        println!("{}: ${}", update.mint, update.price);
    }
).await;
```

#### Optimized Polling

Reduce rate limit impact:

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct SmartPoller {
    cache: Arc<RwLock<HashMap<String, CachedPrice>>>,
    client: JupiterClient,
}

struct CachedPrice {
    price: f64,
    last_update: Instant,
}

impl SmartPoller {
    pub async fn get_price(&self, mint: &str) -> Result<f64, Error> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(mint) {
                // Use cache if less than 1 second old
                if cached.last_update.elapsed() < Duration::from_secs(1) {
                    return Ok(cached.price);
                }
            }
        }

        // Fetch fresh price
        let prices = self.client.get_prices(&[mint.to_string()]).await?;

        if let Some(price_data) = prices.get(mint) {
            let price = price_data.usd_price;

            // Update cache
            {
                let mut cache = self.cache.write().await;
                cache.insert(
                    mint.to_string(),
                    CachedPrice {
                        price,
                        last_update: Instant::now(),
                    },
                );
            }

            Ok(price)
        } else {
            Err(Error::PriceNotFound)
        }
    }
}
```

---

### 4. Event-Driven Architecture

Monitor transactions and trigger updates:

```rust
use solana_client::rpc_client::RpcClient;
use solana_sdk::signature::Signature;
use std::str::FromStr;

pub struct TransactionMonitor {
    rpc_client: RpcClient,
    jupiter_program: Pubkey,
}

impl TransactionMonitor {
    pub async fn monitor_swaps(
        &self,
        callback: impl Fn(SwapEvent),
    ) -> Result<(), Error> {
        let mut last_signature = None;

        loop {
            // Get recent transactions for Jupiter program
            let signatures = self.rpc_client.get_signatures_for_address_with_config(
                &self.jupiter_program,
                solana_client::rpc_client::GetConfirmedSignaturesForAddress2Config {
                    before: last_signature,
                    until: None,
                    limit: Some(10),
                    commitment: Some(CommitmentConfig::confirmed()),
                },
            )?;

            for sig_info in signatures {
                let signature = Signature::from_str(&sig_info.signature)?;

                // Fetch transaction details
                if let Ok(transaction) = self.rpc_client.get_transaction(
                    &signature,
                    solana_transaction_status::UiTransactionEncoding::JsonParsed,
                ) {
                    // Parse swap event
                    if let Some(swap_event) = parse_swap_transaction(&transaction) {
                        callback(swap_event);
                    }
                }

                last_signature = Some(signature);
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}

pub struct SwapEvent {
    pub signature: String,
    pub input_mint: String,
    pub output_mint: String,
    pub input_amount: u64,
    pub output_amount: u64,
    pub user: String,
    pub timestamp: i64,
}
```

---

## Recommended Approaches by Use Case

### High-Frequency Trading

**Method:** Third-party WebSocket (bloXroute)
- Lowest latency
- Real-time price streams
- Professional infrastructure

**Alternative:** Solana RPC WebSocket
- Direct on-chain monitoring
- No middleman
- Requires more setup

---

### Price Display (dApp)

**Method:** Polling (1-5 second intervals)
- Simple implementation
- Sufficient for UI updates
- Works with Free tier

**Implementation:**
```javascript
// React example
import { useEffect, useState } from 'react';

function useLivePrice(mint, intervalMs = 2000) {
  const [price, setPrice] = useState(null);

  useEffect(() => {
    const fetchPrice = async () => {
      const response = await fetch(
        `https://api.jup.ag/price/v3?ids=${mint}`,
        {
          headers: {
            'x-api-key': process.env.REACT_APP_JUPITER_API_KEY,
          },
        }
      );
      const data = await response.json();
      setPrice(data[mint]?.usdPrice);
    };

    fetchPrice(); // Initial fetch
    const interval = setInterval(fetchPrice, intervalMs);

    return () => clearInterval(interval);
  }, [mint, intervalMs]);

  return price;
}

// Usage
function TokenPrice({ mint }) {
  const price = useLivePrice(mint, 2000);
  return <div>${price?.toFixed(4)}</div>;
}
```

---

### Analytics/Monitoring

**Method:** Bitquery GraphQL WebSocket
- Historical data
- Aggregated stats
- Swap monitoring

---

### Portfolio Tracking

**Method:** Polling with smart caching
- Batch price requests
- Cache responses
- Update on-demand

```rust
pub struct PortfolioTracker {
    holdings: Vec<Holding>,
    price_cache: Arc<RwLock<HashMap<String, f64>>>,
}

struct Holding {
    mint: String,
    amount: f64,
}

impl PortfolioTracker {
    pub async fn update_prices(&self, client: &JupiterClient) -> Result<(), Error> {
        // Batch request for all holdings (max 50 at a time)
        let mints: Vec<String> = self.holdings
            .iter()
            .map(|h| h.mint.clone())
            .collect();

        for chunk in mints.chunks(50) {
            let prices = client.get_prices(chunk).await?;

            let mut cache = self.price_cache.write().await;
            for (mint, price_data) in prices {
                cache.insert(mint, price_data.usd_price);
            }
        }

        Ok(())
    }

    pub async fn total_value(&self) -> f64 {
        let cache = self.price_cache.read().await;

        self.holdings
            .iter()
            .filter_map(|holding| {
                cache.get(&holding.mint).map(|price| price * holding.amount)
            })
            .sum()
    }
}
```

---

## WebSocket Simulation

Create WebSocket-like interface over polling:

```rust
use tokio::sync::broadcast;

pub struct PriceStream {
    sender: broadcast::Sender<PriceUpdate>,
}

impl PriceStream {
    pub fn new(client: JupiterClient, mints: Vec<String>) -> Self {
        let (sender, _) = broadcast::channel(100);
        let sender_clone = sender.clone();

        // Background polling task
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(1));

            loop {
                interval.tick().await;

                if let Ok(prices) = client.get_prices(&mints).await {
                    for (mint, price_data) in prices {
                        let update = PriceUpdate {
                            mint: mint.clone(),
                            price: price_data.usd_price,
                            change_24h: price_data.price_change_24h,
                            timestamp: chrono::Utc::now(),
                        };

                        // Broadcast to all subscribers
                        let _ = sender_clone.send(update);
                    }
                }
            }
        });

        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<PriceUpdate> {
        self.sender.subscribe()
    }
}

// Usage
let stream = PriceStream::new(
    client,
    vec!["So11111111111111111111111111111111111111112".to_string()],
);

let mut receiver = stream.subscribe();

while let Ok(update) = receiver.recv().await {
    println!("{}: ${}", update.mint, update.price);
}
```

---

## Rate Limit Considerations

### Polling Frequency

**Free Tier (60 req/min):**
- Poll every 1-2 seconds for single token
- Poll every 5-10 seconds for multiple tokens (batched)

**Pro I (100 req/10s):**
- Poll every 100ms for single token
- Poll every 1 second for 10 tokens (batched)

**Pro II (500 req/10s):**
- Poll every 20ms for single token
- Poll every 200ms for 50 tokens (batched)

### Batch Optimization

Always batch price requests:

```rust
// Bad: 100 requests for 100 tokens
for mint in mints {
    get_price(mint).await;
}

// Good: 2 requests for 100 tokens
get_prices(&mints[0..50]).await;
get_prices(&mints[50..100]).await;
```

---

## Best Practices

### 1. Use Appropriate Method

- **Real-time trading**: Solana RPC WebSocket or bloXroute
- **Price display**: Polling (1-5s interval)
- **Analytics**: Bitquery or polling (longer interval)
- **Alerts**: Event monitoring with callbacks

### 2. Implement Caching

Always cache responses with TTL:
```rust
cache_duration = max(1_second, poll_interval / 2)
```

### 3. Handle Disconnections

When using Solana RPC WebSocket:
```rust
pub async fn reconnecting_websocket() {
    loop {
        match connect_websocket().await {
            Ok(mut ws) => {
                while let Some(msg) = ws.next().await {
                    handle_message(msg);
                }
                eprintln!("WebSocket disconnected, reconnecting...");
            }
            Err(e) => {
                eprintln!("Failed to connect: {}, retrying...", e);
            }
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
```

### 4. Batch and Deduplicate

Combine multiple subscribers:
```rust
pub struct PriceManager {
    subscriptions: HashMap<String, Vec<Callback>>,
}

impl PriceManager {
    pub fn subscribe(&mut self, mint: String, callback: Callback) {
        self.subscriptions
            .entry(mint)
            .or_insert_with(Vec::new)
            .push(callback);
    }

    pub async fn fetch_and_notify(&self) {
        let mints: Vec<String> = self.subscriptions.keys().cloned().collect();

        // Single batch request
        let prices = get_prices(&mints).await;

        // Notify all subscribers
        for (mint, price) in prices {
            if let Some(callbacks) = self.subscriptions.get(&mint) {
                for callback in callbacks {
                    callback(price);
                }
            }
        }
    }
}
```

---

## Notes

1. **No Native WebSocket**: Jupiter doesn't provide WebSocket API
2. **REST Polling**: Primary method for real-time updates
3. **Solana RPC**: Use for on-chain monitoring
4. **Third Parties**: bloXroute, Bitquery offer WebSocket
5. **Batch Requests**: Always batch to save rate limits
6. **Cache Aggressively**: Reduce redundant API calls
7. **Rate Limits Apply**: Polling counts toward rate limits
8. **Consider Alternatives**: WebSocket may not be necessary

---

## Resources

- **Solana RPC WebSocket**: https://docs.solana.com/api/websocket
- **bloXroute**: https://docs.bloxroute.com/solana/trader-api
- **Bitquery**: https://docs.bitquery.io/docs/blockchain/Solana/solana-jupiter-api/
- **Jupiter Discord**: https://discord.gg/jup
