# Vertex Protocol WebSocket API

Vertex Protocol provides WebSocket endpoints for real-time market data, order updates, and account notifications.

## WebSocket URLs

### Production (Mainnet)
- **Gateway WebSocket**: `wss://gateway.prod.vertexprotocol.com/v1/ws`
- **Subscribe WebSocket**: `wss://gateway.prod.vertexprotocol.com/v1/subscribe`

### Testnet (Sepolia)
- **Gateway WebSocket**: `wss://gateway.sepolia-test.vertexprotocol.com/v1/ws`

## Connection Management

### Connection Requirements

1. **Heartbeat**: Send ping frames every 30 seconds
2. **Max Connections**: 5 WebSocket connections per wallet address
3. **Protocol**: WSS (WebSocket Secure)
4. **Timeout**: Auto-disconnect after 60 seconds without ping

### Connection Limits

- **Per Wallet**: Maximum 5 simultaneous connections
- **Enforcement**: Regardless of originating IP address
- **Overflow**: Connections exceeding limit are automatically disconnected

### Heartbeat Implementation

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream};
use futures::{SinkExt, StreamExt};

async fn maintain_heartbeat(ws: &mut WebSocketStream<impl AsyncRead + AsyncWrite>) {
    let mut interval = tokio::time::interval(Duration::from_secs(30));

    loop {
        interval.tick().await;

        if let Err(e) = ws.send(Message::Ping(vec![])).await {
            log::error!("Failed to send ping: {}", e);
            break;
        }

        log::debug!("Sent WebSocket ping");
    }
}
```

## Authentication

### Public Streams (No Auth Required)

- `Trade`
- `BestBidOffer`
- `BookDepth`

### Private Streams (Auth Required)

- `OrderUpdate`
- `Fill`
- `PositionChange`

### Authentication Message

```json
{
  "method": "authenticate",
  "id": 0,
  "tx": {
    "sender": "0x7a5ec2748e9065794491a8d29dcf3f9edb8d7c43000000000000000000000000",
    "expiration": 1234567890000
  },
  "signature": "0x..."
}
```

**Fields**:
- `method`: "authenticate" (constant)
- `id`: Request ID (integer, incremental)
- `tx.sender`: bytes32 subaccount identifier
- `tx.expiration`: Expiration timestamp in milliseconds since Unix epoch
- `signature`: EIP-712 signature of the tx object

**Response**:
```json
{
  "id": 0,
  "status": "authenticated"
}
```

### Authentication Implementation

```rust
use vertex_auth::sign_auth_message;

async fn authenticate_websocket(
    ws: &mut WebSocketStream<impl AsyncRead + AsyncWrite>,
    sender: &str,
    private_key: &str,
    chain_id: u64,
    verifying_contract: &str,
) -> Result<(), Error> {
    let expiration = (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() + 3600000) as u64; // 1 hour validity

    let tx = json!({
        "sender": sender,
        "expiration": expiration,
    });

    let signature = sign_auth_message(&tx, private_key, chain_id, verifying_contract)?;

    let auth_message = json!({
        "method": "authenticate",
        "id": 0,
        "tx": tx,
        "signature": signature,
    });

    ws.send(Message::Text(auth_message.to_string())).await?;

    // Wait for confirmation
    if let Some(Ok(Message::Text(response))) = ws.next().await {
        let resp: serde_json::Value = serde_json::from_str(&response)?;
        if resp["status"] == "authenticated" {
            log::info!("WebSocket authenticated successfully");
            Ok(())
        } else {
            Err(Error::AuthenticationFailed)
        }
    } else {
        Err(Error::NoAuthResponse)
    }
}
```

## Subscription Management

### Subscribe Message

```json
{
  "method": "subscribe",
  "stream": {
    "type": "trade",
    "product_id": 2
  },
  "id": 1
}
```

**Fields**:
- `method`: "subscribe"
- `stream`: Stream configuration object
- `id`: Request ID (unique for each subscription)

### Unsubscribe Message

```json
{
  "method": "unsubscribe",
  "stream": {
    "type": "trade",
    "product_id": 2
  },
  "id": 2
}
```

### Subscription Confirmation

```json
{
  "id": 1,
  "status": "subscribed",
  "stream": {
    "type": "trade",
    "product_id": 2
  }
}
```

## Available Streams

### 1. Trade Stream

**Type**: `trade`
**Authentication**: Not required
**Parameters**:
- `product_id`: uint32 (required)

**Subscribe**:
```json
{
  "method": "subscribe",
  "stream": {
    "type": "trade",
    "product_id": 2
  },
  "id": 1
}
```

**Message Format**:
```json
{
  "stream": "trade",
  "data": {
    "product_id": 2,
    "price_x18": "30500000000000000000000",
    "size": "1000000000000000000",
    "side": "buy",
    "timestamp": 1234567890,
    "digest": "0x123abc..."
  }
}
```

**Fields**:
- `price_x18`: Trade price (X18 format)
- `size`: Trade size (absolute value)
- `side`: "buy" or "sell"
- `timestamp`: Unix timestamp (seconds)
- `digest`: Trade identifier

### 2. BestBidOffer (BBO) Stream

**Type**: `best_bid_offer`
**Authentication**: Not required
**Parameters**:
- `product_id`: uint32 (required)

**Subscribe**:
```json
{
  "method": "subscribe",
  "stream": {
    "type": "best_bid_offer",
    "product_id": 2
  },
  "id": 2
}
```

**Message Format**:
```json
{
  "stream": "best_bid_offer",
  "data": {
    "product_id": 2,
    "bid_x18": "29950000000000000000000",
    "ask_x18": "30050000000000000000000",
    "bid_size": "5000000000000000000",
    "ask_size": "3000000000000000000",
    "timestamp": 1234567890
  }
}
```

**Fields**:
- `bid_x18`: Best bid price
- `ask_x18`: Best ask price
- `bid_size`: Size at best bid
- `ask_size`: Size at best ask
- `timestamp`: Update timestamp

### 3. BookDepth Stream

**Type**: `book_depth`
**Authentication**: Not required
**Parameters**:
- `product_id`: uint32 (required)

**Subscribe**:
```json
{
  "method": "subscribe",
  "stream": {
    "type": "book_depth",
    "product_id": 2
  },
  "id": 3
}
```

**Message Format**:
```json
{
  "stream": "book_depth",
  "data": {
    "product_id": 2,
    "bids": [
      ["29950000000000000000000", "5000000000000000000"],
      ["29900000000000000000000", "10000000000000000000"],
      ["29850000000000000000000", "15000000000000000000"]
    ],
    "asks": [
      ["30050000000000000000000", "3000000000000000000"],
      ["30100000000000000000000", "8000000000000000000"],
      ["30150000000000000000000", "12000000000000000000"]
    ],
    "timestamp": 1234567890
  }
}
```

**Fields**:
- `bids`: Array of [price_x18, size] (sorted descending)
- `asks`: Array of [price_x18, size] (sorted ascending)
- `timestamp`: Snapshot timestamp

### 4. OrderUpdate Stream

**Type**: `order_update`
**Authentication**: Required
**Parameters**:
- `product_id`: uint32 (required)
- `subaccount`: bytes32 (required)

**Subscribe**:
```json
{
  "method": "subscribe",
  "stream": {
    "type": "order_update",
    "subaccount": "0x7a5ec2748e9065794491a8d29dcf3f9edb8d7c43000000000000000000000000",
    "product_id": 2
  },
  "id": 4
}
```

**Message Format**:
```json
{
  "stream": "order_update",
  "data": {
    "product_id": 2,
    "subaccount": "0x7a5ec2748e9065794491a8d29dcf3f9edb8d7c43000000000000000000000000",
    "order": {
      "digest": "0x123abc...",
      "priceX18": "30000000000000000000000",
      "amount": "1000000000000000000",
      "unfilled_amount": "200000000000000000",
      "status": "partially_filled",
      "nonce": 1234567890123,
      "expiration": 4611686018427387904
    },
    "timestamp": 1234567890
  }
}
```

**Order Status Values**:
- `open`: Order placed and active
- `partially_filled`: Order partially executed
- `filled`: Order fully executed
- `cancelled`: Order cancelled
- `rejected`: Order rejected by engine

### 5. Fill Stream

**Type**: `fill`
**Authentication**: Required
**Parameters**:
- `product_id`: uint32 (required)
- `subaccount`: bytes32 (required)

**Subscribe**:
```json
{
  "method": "subscribe",
  "stream": {
    "type": "fill",
    "subaccount": "0x7a5ec2748e9065794491a8d29dcf3f9edb8d7c43000000000000000000000000",
    "product_id": 2
  },
  "id": 5
}
```

**Message Format**:
```json
{
  "stream": "fill",
  "data": {
    "product_id": 2,
    "subaccount": "0x7a5ec2748e9065794491a8d29dcf3f9edb8d7c43000000000000000000000000",
    "digest": "0x123abc...",
    "price_x18": "30500000000000000000000",
    "size": "800000000000000000",
    "side": "buy",
    "fee": "12200000000000000",
    "is_maker": false,
    "timestamp": 1234567890
  }
}
```

**Fields**:
- `digest`: Order digest that was filled
- `price_x18`: Execution price
- `size`: Filled size
- `side`: "buy" or "sell"
- `fee`: Trading fee charged (in quote currency)
- `is_maker`: true if maker order, false if taker
- `timestamp`: Fill timestamp

### 6. PositionChange Stream

**Type**: `position_change`
**Authentication**: Required
**Parameters**:
- `product_id`: uint32 (required)
- `subaccount`: bytes32 (required)

**Subscribe**:
```json
{
  "method": "subscribe",
  "stream": {
    "type": "position_change",
    "subaccount": "0x7a5ec2748e9065794491a8d29dcf3f9edb8d7c43000000000000000000000000",
    "product_id": 2
  },
  "id": 6
}
```

**Message Format**:
```json
{
  "stream": "position_change",
  "data": {
    "product_id": 2,
    "subaccount": "0x7a5ec2748e9065794491a8d29dcf3f9edb8d7c43000000000000000000000000",
    "balance": {
      "amount": "5800000000000000000",
      "v_quote_balance": "174000000000000000000",
      "last_cumulative_funding_x18": "1000123456789012345"
    },
    "timestamp": 1234567890
  }
}
```

**Fields**:
- `balance.amount`: Position size (positive = long, negative = short)
- `balance.v_quote_balance`: Virtual quote balance
- `balance.last_cumulative_funding_x18`: Funding checkpoint
- `timestamp`: Update timestamp

## Implementation Example

### Complete WebSocket Client

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures::{SinkExt, StreamExt};
use serde_json::json;

pub struct VertexWebSocket {
    ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
    next_id: u64,
}

impl VertexWebSocket {
    pub async fn connect(url: &str) -> Result<Self, Error> {
        let (ws, _) = connect_async(url).await?;
        log::info!("WebSocket connected to {}", url);

        Ok(Self { ws, next_id: 0 })
    }

    pub async fn authenticate(
        &mut self,
        sender: &str,
        private_key: &str,
        chain_id: u64,
        verifying_contract: &str,
    ) -> Result<(), Error> {
        let expiration = (SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_millis() + 3600000) as u64;

        let tx = json!({
            "sender": sender,
            "expiration": expiration,
        });

        let signature = sign_auth_message(&tx, private_key, chain_id, verifying_contract)?;

        let auth_msg = json!({
            "method": "authenticate",
            "id": self.next_id,
            "tx": tx,
            "signature": signature,
        });

        self.next_id += 1;

        self.ws.send(Message::Text(auth_msg.to_string())).await?;
        Ok(())
    }

    pub async fn subscribe_trade(&mut self, product_id: u32) -> Result<(), Error> {
        let msg = json!({
            "method": "subscribe",
            "stream": {
                "type": "trade",
                "product_id": product_id,
            },
            "id": self.next_id,
        });

        self.next_id += 1;

        self.ws.send(Message::Text(msg.to_string())).await?;
        Ok(())
    }

    pub async fn subscribe_orderbook(&mut self, product_id: u32) -> Result<(), Error> {
        let msg = json!({
            "method": "subscribe",
            "stream": {
                "type": "book_depth",
                "product_id": product_id,
            },
            "id": self.next_id,
        });

        self.next_id += 1;

        self.ws.send(Message::Text(msg.to_string())).await?;
        Ok(())
    }

    pub async fn subscribe_order_updates(
        &mut self,
        product_id: u32,
        subaccount: &str,
    ) -> Result<(), Error> {
        let msg = json!({
            "method": "subscribe",
            "stream": {
                "type": "order_update",
                "product_id": product_id,
                "subaccount": subaccount,
            },
            "id": self.next_id,
        });

        self.next_id += 1;

        self.ws.send(Message::Text(msg.to_string())).await?;
        Ok(())
    }

    pub async fn next_message(&mut self) -> Result<Option<serde_json::Value>, Error> {
        while let Some(msg) = self.ws.next().await {
            match msg? {
                Message::Text(text) => {
                    let data: serde_json::Value = serde_json::from_str(&text)?;
                    return Ok(Some(data));
                }
                Message::Ping(payload) => {
                    self.ws.send(Message::Pong(payload)).await?;
                }
                Message::Pong(_) => {}
                Message::Close(_) => {
                    log::warn!("WebSocket closed by server");
                    return Ok(None);
                }
                _ => {}
            }
        }

        Ok(None)
    }

    pub async fn start_heartbeat(&mut self) {
        let mut interval = tokio::time::interval(Duration::from_secs(30));

        tokio::spawn(async move {
            loop {
                interval.tick().await;
                // Send ping via channel to main loop
            }
        });
    }
}
```

### Message Handler

```rust
pub async fn handle_websocket_messages(
    mut ws: VertexWebSocket,
    message_tx: mpsc::Sender<MarketEvent>,
) -> Result<(), Error> {
    loop {
        match ws.next_message().await? {
            Some(msg) => {
                if let Some(stream_type) = msg.get("stream").and_then(|v| v.as_str()) {
                    match stream_type {
                        "trade" => {
                            let trade = parse_trade(&msg)?;
                            message_tx.send(MarketEvent::Trade(trade)).await?;
                        }
                        "best_bid_offer" => {
                            let bbo = parse_bbo(&msg)?;
                            message_tx.send(MarketEvent::BBO(bbo)).await?;
                        }
                        "book_depth" => {
                            let book = parse_orderbook(&msg)?;
                            message_tx.send(MarketEvent::Orderbook(book)).await?;
                        }
                        "order_update" => {
                            let update = parse_order_update(&msg)?;
                            message_tx.send(MarketEvent::OrderUpdate(update)).await?;
                        }
                        "fill" => {
                            let fill = parse_fill(&msg)?;
                            message_tx.send(MarketEvent::Fill(fill)).await?;
                        }
                        "position_change" => {
                            let pos = parse_position_change(&msg)?;
                            message_tx.send(MarketEvent::PositionChange(pos)).await?;
                        }
                        _ => {
                            log::warn!("Unknown stream type: {}", stream_type);
                        }
                    }
                } else {
                    log::debug!("Non-stream message: {:?}", msg);
                }
            }
            None => {
                log::warn!("WebSocket connection closed");
                break;
            }
        }
    }

    Ok(())
}
```

## Reconnection Strategy

### Exponential Backoff

```rust
async fn connect_with_retry(url: &str, max_retries: u32) -> Result<VertexWebSocket, Error> {
    let mut retries = 0;
    let mut delay = Duration::from_secs(1);

    loop {
        match VertexWebSocket::connect(url).await {
            Ok(ws) => return Ok(ws),
            Err(e) if retries < max_retries => {
                log::warn!(
                    "WebSocket connection failed. Retry {}/{}. Waiting {:?}. Error: {}",
                    retries + 1,
                    max_retries,
                    delay,
                    e
                );

                tokio::time::sleep(delay).await;

                retries += 1;
                delay = std::cmp::min(delay * 2, Duration::from_secs(60));
            }
            Err(e) => return Err(e),
        }
    }
}
```

### Automatic Reconnection Loop

```rust
async fn maintain_websocket_connection(
    url: &str,
    message_tx: mpsc::Sender<MarketEvent>,
) -> Result<(), Error> {
    loop {
        match connect_with_retry(url, 10).await {
            Ok(mut ws) => {
                log::info!("WebSocket connected successfully");

                // Re-authenticate
                ws.authenticate(...).await?;

                // Re-subscribe to streams
                ws.subscribe_trade(2).await?;
                ws.subscribe_orderbook(2).await?;

                // Handle messages until disconnect
                if let Err(e) = handle_websocket_messages(ws, message_tx.clone()).await {
                    log::error!("WebSocket error: {}", e);
                }
            }
            Err(e) => {
                log::error!("Failed to connect after retries: {}", e);
                tokio::time::sleep(Duration::from_secs(60)).await;
            }
        }
    }
}
```

## Best Practices

1. **Single Connection**: Use one WebSocket for multiple subscriptions
2. **Heartbeat**: Send ping every 25-30 seconds
3. **Reconnect**: Implement exponential backoff reconnection
4. **Re-subscribe**: Re-subscribe to all streams after reconnect
5. **Message Queue**: Use channels to distribute messages
6. **Error Handling**: Log errors but don't crash on parse failures
7. **Connection Pool**: Limit to 3-4 connections per wallet
8. **Authentication**: Re-authenticate after reconnection
9. **Monitoring**: Track connection uptime and message rates
10. **Graceful Shutdown**: Properly close connections on exit

## Error Handling

```rust
#[derive(Debug)]
enum WebSocketError {
    ConnectionFailed(String),
    AuthenticationFailed,
    SubscriptionFailed(String),
    MessageParseFailed(String),
    SendFailed(String),
}

impl std::fmt::Display for WebSocketError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::ConnectionFailed(e) => write!(f, "Connection failed: {}", e),
            Self::AuthenticationFailed => write!(f, "Authentication failed"),
            Self::SubscriptionFailed(s) => write!(f, "Subscription failed: {}", s),
            Self::MessageParseFailed(e) => write!(f, "Message parse failed: {}", e),
            Self::SendFailed(e) => write!(f, "Send failed: {}", e),
        }
    }
}
```

## Summary

| Stream | Auth | Parameters | Use Case |
|--------|------|------------|----------|
| **trade** | No | product_id | Recent trades |
| **best_bid_offer** | No | product_id | Top of book |
| **book_depth** | No | product_id | Full orderbook |
| **order_update** | Yes | product_id, subaccount | Order status |
| **fill** | Yes | product_id, subaccount | Trade executions |
| **position_change** | Yes | product_id, subaccount | Position updates |

**Connection Requirements**:
- Ping every 30 seconds
- Max 5 connections per wallet
- Re-authenticate after reconnect
- Use WSS (secure WebSocket)
