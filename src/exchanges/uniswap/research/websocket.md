# Uniswap WebSocket Events and Real-Time Data

## Overview

Uniswap doesn't have a dedicated WebSocket API. Real-time data is obtained through:

1. **Ethereum WebSocket Subscriptions** - Monitor on-chain events
2. **The Graph Subscriptions** - Real-time subgraph updates (limited)
3. **Third-Party Streams** - Services like Finazon

---

## 1. Ethereum WebSocket Subscriptions

### Connection Setup

**WebSocket URL Format:**
```
wss://<provider>/<api_key>
```

**Providers:**
- Infura: `wss://mainnet.infura.io/ws/v3/YOUR_PROJECT_ID`
- Alchemy: `wss://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY`
- Chainstack: `wss://nd-123-456-789.p2pify.com/YOUR_API_KEY`

### Available Subscriptions

#### 1.1 New Block Headers

**Subscribe:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "eth_subscribe",
  "params": ["newHeads"]
}
```

**Subscription Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": "0x1234567890abcdef"
}
```

**Block Notification:**
```json
{
  "jsonrpc": "2.0",
  "method": "eth_subscription",
  "params": {
    "subscription": "0x1234567890abcdef",
    "result": {
      "number": "0x121a7b0",
      "hash": "0x123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
      "parentHash": "0xabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
      "timestamp": "0x679e3e40",
      "gasLimit": "0x1c9c380",
      "gasUsed": "0xf4240",
      "baseFeePerGas": "0xba43b7400",
      "difficulty": "0x0",
      "extraData": "0x",
      "logsBloom": "0x00000000...",
      "miner": "0x1234567890123456789012345678901234567890",
      "mixHash": "0x...",
      "nonce": "0x0000000000000000",
      "receiptsRoot": "0x...",
      "sha3Uncles": "0x...",
      "stateRoot": "0x...",
      "transactionsRoot": "0x..."
    }
  }
}
```

**Use Case:** Trigger actions on new blocks (price updates, arbitrage checks)

---

#### 1.2 Event Logs

**Subscribe to Swap Events:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "eth_subscribe",
  "params": [
    "logs",
    {
      "address": "0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640",
      "topics": [
        "0xc42079f94a6350d7e6235f29174924f928cc2ac818eb64fed8004e115fbcca67"
      ]
    }
  ]
}
```

**Parameters:**
- `address`: Pool contract address
- `topics[0]`: Event signature hash (Swap event)

**Event Signature Hash:**
```solidity
// Swap(address,address,int256,int256,uint160,uint128,int24)
keccak256("Swap(address,address,int256,int256,uint160,uint128,int24)")
= 0xc42079f94a6350d7e6235f29174924f928cc2ac818eb64fed8004e115fbcca67
```

**Swap Event Notification:**
```json
{
  "jsonrpc": "2.0",
  "method": "eth_subscription",
  "params": {
    "subscription": "0xabcdef123456",
    "result": {
      "address": "0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640",
      "topics": [
        "0xc42079f94a6350d7e6235f29174924f928cc2ac818eb64fed8004e115fbcca67",
        "0x000000000000000000000000e592427a0aece92de3edee1f18e0157c05861564",
        "0x0000000000000000000000001234567890123456789012345678901234567890"
      ],
      "data": "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffc465e60000000000000000000000000000000000000000000000000000016345785d8a000000000000000000000000000000000000000000004e5f42b4b91a2bfffba60000000000000000000000000000000000000000000000000000ab54a98ceb1f0eafffffffffffffffffffffffffffffffffffffffffffffffffffffffffffcfd88",
      "blockNumber": "0x121a7b0",
      "transactionHash": "0x123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
      "transactionIndex": "0x5a",
      "blockHash": "0xabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
      "logIndex": "0x12",
      "removed": false
    }
  }
}
```

**Decoded Event:**
```rust
struct SwapEvent {
    sender: Address,          // topics[1]
    recipient: Address,       // topics[2]
    amount0: I256,            // data[0:32]
    amount1: I256,            // data[32:64]
    sqrt_price_x96: U256,     // data[64:96]
    liquidity: u128,          // data[96:112]
    tick: i32,                // data[112:116]
}
```

---

#### 1.3 Pending Transactions (Mempool)

**Subscribe:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "eth_subscribe",
  "params": ["newPendingTransactions"]
}
```

**Transaction Hash Notification:**
```json
{
  "jsonrpc": "2.0",
  "method": "eth_subscription",
  "params": {
    "subscription": "0x9876543210fedcba",
    "result": "0x123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
  }
}
```

**Use Case:** Front-running detection, MEV strategies (requires additional `eth_getTransactionByHash` call)

---

### Unsubscribe

**Stop Subscription:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "eth_unsubscribe",
  "params": ["0x1234567890abcdef"]
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": true
}
```

---

## 2. Uniswap Event Types

### 2.1 Swap Event

**Solidity Signature:**
```solidity
event Swap(
    address indexed sender,
    address indexed recipient,
    int256 amount0,
    int256 amount1,
    uint160 sqrtPriceX96,
    uint128 liquidity,
    int24 tick
);
```

**Topic Hash:** `0xc42079f94a6350d7e6235f29174924f928cc2ac818eb64fed8004e115fbcca67`

**Fields:**
- `sender`: Address initiating swap (usually router)
- `recipient`: Address receiving tokens
- `amount0`: Token0 delta (negative = sold, positive = bought)
- `amount1`: Token1 delta (negative = sold, positive = bought)
- `sqrtPriceX96`: New pool price after swap
- `liquidity`: Pool liquidity after swap
- `tick`: Current tick after swap

---

### 2.2 Mint Event (Add Liquidity)

**Solidity Signature:**
```solidity
event Mint(
    address sender,
    address indexed owner,
    int24 indexed tickLower,
    int24 indexed tickUpper,
    uint128 amount,
    uint256 amount0,
    uint256 amount1
);
```

**Topic Hash:** `0x7a53080ba414158be7ec69b987b5fb7d07dee101fe85488f0853ae16239d0bde`

---

### 2.3 Burn Event (Remove Liquidity)

**Solidity Signature:**
```solidity
event Burn(
    address indexed owner,
    int24 indexed tickLower,
    int24 indexed tickUpper,
    uint128 amount,
    uint256 amount0,
    uint256 amount1
);
```

**Topic Hash:** `0x0c396cd989a39f4459b5fa1aed6a9a8dcdbc45908acfd67e028cd568da98982c`

---

### 2.4 Collect Event (Claim Fees)

**Solidity Signature:**
```solidity
event Collect(
    address indexed owner,
    address recipient,
    int24 indexed tickLower,
    int24 indexed tickUpper,
    uint128 amount0,
    uint128 amount1
);
```

**Topic Hash:** `0x70935338e69775456a85ddef226c395fb668b63fa0115f5f20610b388e6ca9c0`

---

## 3. Rust Implementation

### 3.1 WebSocket Connection

**Using tokio-tungstenite:**

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;

async fn connect_to_ethereum_ws(url: &str) -> Result<()> {
    let (ws_stream, _) = connect_async(url).await?;
    let (mut write, mut read) = ws_stream.split();

    // Subscribe to new block headers
    let subscribe_msg = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "eth_subscribe",
        "params": ["newHeads"]
    });

    write.send(Message::Text(subscribe_msg.to_string())).await?;

    // Process messages
    while let Some(msg) = read.next().await {
        match msg? {
            Message::Text(text) => {
                let data: serde_json::Value = serde_json::from_str(&text)?;
                println!("Received: {}", data);
            }
            Message::Close(_) => {
                println!("Connection closed");
                break;
            }
            _ => {}
        }
    }

    Ok(())
}
```

---

### 3.2 Subscribe to Swap Events

```rust
use alloy::primitives::{Address, keccak256};

async fn subscribe_to_swap_events(
    ws_stream: &mut WebSocketStream,
    pool_address: Address,
) -> Result<String> {
    // Swap event signature
    let swap_signature = keccak256("Swap(address,address,int256,int256,uint160,uint128,int24)");

    let subscribe_msg = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "eth_subscribe",
        "params": [
            "logs",
            {
                "address": format!("{:?}", pool_address),
                "topics": [format!("0x{}", hex::encode(swap_signature))]
            }
        ]
    });

    ws_stream.send(Message::Text(subscribe_msg.to_string())).await?;

    // Wait for subscription ID
    if let Some(Message::Text(response)) = ws_stream.next().await {
        let data: serde_json::Value = serde_json::from_str(&response)?;
        if let Some(sub_id) = data["result"].as_str() {
            return Ok(sub_id.to_string());
        }
    }

    Err(Error::SubscriptionFailed)
}
```

---

### 3.3 Decode Swap Event

```rust
use alloy::primitives::{I256, U256};
use alloy::sol_types::SolEvent;

// Define the event using alloy macros
alloy::sol! {
    event Swap(
        address indexed sender,
        address indexed recipient,
        int256 amount0,
        int256 amount1,
        uint160 sqrtPriceX96,
        uint128 liquidity,
        int24 tick
    );
}

fn decode_swap_event(log: &Log) -> Result<Swap> {
    let event = Swap::decode_log(log, true)?;
    Ok(event)
}

// Usage
let swap = decode_swap_event(&log)?;
println!("Swap: {} token0 for {} token1", swap.amount0, swap.amount1);
println!("New price: {}", swap.sqrtPriceX96);
println!("New tick: {}", swap.tick);
```

---

### 3.4 Monitor Multiple Pools

```rust
use std::collections::HashMap;

async fn monitor_pools(
    ws_url: &str,
    pool_addresses: Vec<Address>,
) -> Result<()> {
    let (ws_stream, _) = connect_async(ws_url).await?;
    let (mut write, mut read) = ws_stream.split();

    let swap_signature = keccak256("Swap(address,address,int256,int256,uint160,uint128,int24)");

    // Subscribe to all pools
    let subscribe_msg = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "eth_subscribe",
        "params": [
            "logs",
            {
                "address": pool_addresses.iter()
                    .map(|a| format!("{:?}", a))
                    .collect::<Vec<_>>(),
                "topics": [format!("0x{}", hex::encode(swap_signature))]
            }
        ]
    });

    write.send(Message::Text(subscribe_msg.to_string())).await?;

    // Process events
    while let Some(msg) = read.next().await {
        if let Message::Text(text) = msg? {
            let data: serde_json::Value = serde_json::from_str(&text)?;

            if let Some(result) = data["params"]["result"].as_object() {
                let pool_address = result["address"].as_str().unwrap();
                let log_data = result["data"].as_str().unwrap();

                println!("Swap on pool {}: {}", pool_address, log_data);
                // Decode and process event
            }
        }
    }

    Ok(())
}
```

---

## 4. Block-Based Monitoring

### Approach: Subscribe to New Blocks + Filter Transactions

**Algorithm:**
1. Subscribe to `newHeads`
2. On each new block, fetch full block data
3. Filter transactions by destination address (Uniswap router)
4. Decode transaction input to identify swaps

**Implementation:**

```rust
use alloy::rpc::types::Block;

async fn monitor_swaps_via_blocks(
    provider: &Provider,
    router_address: Address,
) -> Result<()> {
    let subscription = provider.subscribe_blocks().await?;
    let mut stream = subscription.into_stream();

    while let Some(block) = stream.next().await {
        println!("New block: {}", block.header.number);

        // Fetch full block with transactions
        let full_block = provider
            .get_block_by_number(block.header.number.into(), true)
            .await?;

        if let Some(txs) = full_block.transactions {
            for tx in txs {
                // Check if transaction calls Uniswap router
                if tx.to == Some(router_address) {
                    // Decode transaction input
                    if let Some(method) = decode_swap_method(&tx.input) {
                        println!("Swap detected: {:?}", method);
                    }
                }
            }
        }
    }

    Ok(())
}
```

**Swap Method Signatures:**
```rust
const SWAP_EXACT_ETH_FOR_TOKENS: [u8; 4] = [0x7f, 0xf3, 0x6a, 0xb5];
const SWAP_EXACT_TOKENS_FOR_ETH: [u8; 4] = [0x18, 0xcb, 0xaf, 0xe5];
const EXACT_INPUT_SINGLE: [u8; 4] = [0x41, 0x4b, 0xf3, 0x89];
const EXACT_INPUT: [u8; 4] = [0xc0, 0x4b, 0x8d, 0x59];
```

---

## 5. The Graph WebSocket Subscriptions

### Limited Support

The Graph supports GraphQL subscriptions with **limited functionality**:

**Subscription Example:**
```graphql
subscription {
  swaps(
    first: 10,
    orderBy: timestamp,
    orderDirection: desc
  ) {
    id
    amount0
    amount1
    amountUSD
  }
}
```

**Limitations:**
- Not all subgraphs support subscriptions
- Updates are batched (not instant)
- May have additional costs
- Polling often more reliable

**Recommendation:** Use Ethereum WebSocket subscriptions for real-time data.

---

## 6. Third-Party WebSocket Services

### Finazon WebSocket API

**Endpoint:**
```
wss://api.finazon.io/v1/ws
```

**Features:**
- Pre-aggregated OHLCV data
- 1s, 10s, 1m frequency
- Simplified interface

**Subscribe to Uniswap Data:**
```json
{
  "action": "subscribe",
  "dataset": "uniswap",
  "ticker": "WETH_USDC",
  "interval": "1s"
}
```

**Data Stream:**
```json
{
  "ticker": "WETH_USDC",
  "timestamp": 1735680000,
  "open": 2000.50,
  "high": 2001.00,
  "low": 2000.00,
  "close": 2000.75,
  "volume": 123456.78
}
```

**Cost:** Paid service (pricing not specified in research)

---

## 7. Reconnection and Error Handling

### Automatic Reconnection

```rust
async fn maintain_websocket_connection(
    url: String,
    on_message: impl Fn(Message) + Send + 'static,
) {
    loop {
        match connect_and_listen(&url, &on_message).await {
            Ok(_) => {
                println!("WebSocket connection closed normally");
            }
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
            }
        }

        println!("Reconnecting in 5 seconds...");
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

async fn connect_and_listen(
    url: &str,
    on_message: &impl Fn(Message),
) -> Result<()> {
    let (ws_stream, _) = connect_async(url).await?;
    let (mut write, mut read) = ws_stream.split();

    // Re-subscribe after reconnect
    let subscribe_msg = create_subscription_message();
    write.send(Message::Text(subscribe_msg)).await?;

    // Process messages
    while let Some(msg) = read.next().await {
        match msg? {
            Message::Text(_) | Message::Binary(_) => {
                on_message(msg?);
            }
            Message::Close(_) => {
                break;
            }
            Message::Ping(data) => {
                write.send(Message::Pong(data)).await?;
            }
            _ => {}
        }
    }

    Ok(())
}
```

### Heartbeat/Ping-Pong

```rust
use tokio::time::{interval, Duration};

async fn send_heartbeat(write: &mut SplitSink<WebSocketStream, Message>) {
    let mut interval = interval(Duration::from_secs(30));

    loop {
        interval.tick().await;
        if let Err(e) = write.send(Message::Ping(vec![])).await {
            eprintln!("Failed to send ping: {}", e);
            break;
        }
    }
}

// Run in separate task
tokio::spawn(send_heartbeat(write));
```

---

## 8. Performance Considerations

### Connection Limits

**Provider Limits:**
- Infura: 100 concurrent subscriptions
- Alchemy: 500 concurrent subscriptions
- Chainstack: Unlimited (dedicated nodes)

**Optimization:**
- Monitor multiple pools with single subscription (filter multiple addresses)
- Share WebSocket connection across application
- Unsubscribe from inactive pools

### Message Volume

**Mainnet Swap Volume:**
- ~10-50 swaps per second during normal activity
- ~100+ swaps per second during high volatility
- Each event ~300-500 bytes

**Bandwidth Estimate:**
```
50 swaps/sec × 400 bytes = 20 KB/sec = 70 MB/hour
```

**Filtering:**
```rust
// Only subscribe to high-liquidity pools
let top_pools = [
    "0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640",  // USDC/WETH 0.05%
    "0x8ad599c3A0ff1De082011EFDDc58f1908eb6e6D8",  // USDC/WETH 0.30%
    // ... top 10-20 pools only
];
```

---

## 9. Use Cases

### 9.1 Price Tracking

**Monitor pool prices in real-time:**
```rust
async fn track_pool_price(pool_address: Address) {
    subscribe_to_swap_events(pool_address).await;

    while let Some(swap_event) = event_stream.next().await {
        let price = calculate_price_from_sqrt(swap_event.sqrtPriceX96);
        println!("New price: {}", price);

        // Update chart, trigger alerts, etc.
    }
}
```

### 9.2 Arbitrage Detection

**Find price discrepancies across pools:**
```rust
let mut pool_prices = HashMap::new();

// Subscribe to multiple pools
for pool in pools {
    let events = subscribe_to_swap_events(pool).await;

    tokio::spawn(async move {
        while let Some(swap) = events.next().await {
            let price = calculate_price(swap);
            pool_prices.insert(pool, price);

            check_arbitrage_opportunities(&pool_prices);
        }
    });
}
```

### 9.3 Transaction Monitoring

**Track specific wallet activity:**
```rust
subscribe_to_logs(
    pool_address,
    vec![swap_signature],
    vec![
        None,  // sender (any)
        Some(wallet_address),  // recipient filter
    ],
).await;
```

### 9.4 Volume Analysis

**Aggregate swap volume per pool:**
```rust
let mut volume_tracker = HashMap::new();

while let Some(swap) = events.next().await {
    let volume_usd = swap.amount0.abs() * token0_price;
    *volume_tracker.entry(pool_address).or_insert(0.0) += volume_usd;
}
```

---

## 10. WebSocket vs Polling Comparison

| Aspect | WebSocket | HTTP Polling |
|--------|-----------|--------------|
| **Latency** | <100ms | 1-5 seconds |
| **Overhead** | Low (push-based) | High (repeated requests) |
| **Rate Limits** | Connection-based | Request-based (strict) |
| **Reliability** | Requires reconnection logic | Simpler error handling |
| **Use Case** | Real-time trading, monitoring | Periodic updates, historical data |

**Recommendation:**
- Use **WebSocket** for: Live price feeds, swap monitoring, arbitrage
- Use **HTTP** for: Historical data, analytics, infrequent queries

---

## Summary

**Key Points:**

1. **No Native Uniswap WebSocket** - Use Ethereum node WebSocket subscriptions
2. **Event Types**: Swap, Mint, Burn, Collect
3. **Subscription Methods**: `newHeads`, `logs`, `newPendingTransactions`
4. **Decoding**: Use alloy/ethers to decode event data
5. **Reconnection**: Implement automatic reconnection with exponential backoff
6. **Filtering**: Subscribe to specific pools/events to reduce noise
7. **Performance**: Monitor 10-50 swaps/sec, ~20 KB/sec bandwidth
8. **Providers**: Infura, Alchemy, Chainstack (paid tiers recommended)

**Implementation Checklist:**
- [ ] Choose WebSocket provider (Infura/Alchemy/Chainstack)
- [ ] Implement connection with auto-reconnect
- [ ] Subscribe to relevant event types (Swap, Mint, Burn)
- [ ] Decode events using alloy/ethers
- [ ] Handle errors and connection drops gracefully
- [ ] Filter by pool addresses to reduce noise
- [ ] Implement heartbeat/ping-pong for connection health
- [ ] Monitor connection metrics (message rate, latency)
