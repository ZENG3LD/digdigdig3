# Bitquery - WebSocket Documentation

## Availability: Yes

Bitquery provides WebSocket support via **GraphQL Subscriptions** for real-time blockchain data streaming.

---

## Connection

### URLs
- **Primary WebSocket**: `wss://streaming.bitquery.io/graphql`
- **With Token (recommended)**: `wss://streaming.bitquery.io/graphql?token=ory_at_YOUR_TOKEN`
- **EAP WebSocket**: `wss://streaming.bitquery.io/eap?token=YOUR_TOKEN`
- **No separate public/private** - Single endpoint with auth

### Regional Endpoints
- None - Single global WebSocket endpoint

---

## Connection Process

### 1. Establish WebSocket Connection
```javascript
const ws = new WebSocket('wss://streaming.bitquery.io/graphql?token=YOUR_TOKEN');
```

### 2. Protocol Negotiation
WebSocket must specify subprotocol during connection:
- **graphql-transport-ws** (recommended, modern)
- **graphql-ws** (legacy, still supported)

**Example with protocol**:
```javascript
const ws = new WebSocket(
  'wss://streaming.bitquery.io/graphql?token=YOUR_TOKEN',
  'graphql-transport-ws'
);
```

### 3. Send connection_init Message
After WebSocket opens, client sends:

**For graphql-transport-ws**:
```json
{
  "type": "connection_init",
  "payload": {}
}
```

**For graphql-ws**:
```json
{
  "type": "connection_init",
  "payload": {}
}
```

### 4. Await connection_ack
Server responds with:

**graphql-transport-ws**:
```json
{
  "type": "connection_ack"
}
```

**graphql-ws**:
```json
{
  "type": "connection_ack"
}
```

### 5. Subscribe to Data
Send GraphQL subscription message (see Subscription Format below)

---

## Authentication

### Method 1: URL Parameter (Recommended)
Include OAuth token in WebSocket URL:
```
wss://streaming.bitquery.io/graphql?token=ory_at_YOUR_ACCESS_TOKEN
```

### Method 2: Authorization Header (if supported by client)
Not typical for browser WebSocket, but some clients support:
```javascript
// Node.js example with ws library
const ws = new WebSocket('wss://streaming.bitquery.io/graphql', {
  headers: {
    'Authorization': 'Bearer ory_at_YOUR_ACCESS_TOKEN'
  }
});
```

### Token Generation
1. Sign up at https://account.bitquery.io/auth/signup
2. Generate OAuth token in IDE or via API
3. Token format: `ory_at_...` (OAuth 2.0 access token)
4. Separate auth docs: https://docs.bitquery.io/docs/authorisation/how-to-use/

### Auth Errors
- **No token**: Connection may succeed but subscriptions fail with 401/403
- **Invalid token**: Connection closes or subscription error
- **Expired token**: Re-authenticate and reconnect

---

## Protocols Supported

Bitquery supports **two GraphQL WebSocket standards**:

### 1. graphql-transport-ws (Recommended)
- **Modern standard**: Actively maintained
- **Keepalive**: Server sends `pong` messages
- **Library**: `graphql-ws` npm package

### 2. graphql-ws (Legacy)
- **Older standard**: Still supported
- **Keepalive**: Server sends `ka` (keepalive) messages
- **Library**: `subscriptions-transport-ws` (deprecated but works)

**Both protocols are functionally similar** with minor message format differences.

---

## ALL Available Channels/Topics

**CRITICAL**: Bitquery doesn't use "channels" or "topics" like traditional WebSocket APIs.

Instead, you create **GraphQL subscriptions** by converting any **query** to a **subscription**.

### How It Works
Any GraphQL query can become a subscription by:
1. Replace `query` keyword with `subscription`
2. Use `dataset: realtime` for live data
3. Send via WebSocket

### Example Conversion

**Query (HTTP)**:
```graphql
query {
  EVM(network: eth, dataset: archive) {
    Blocks(limit: {count: 10}) {
      Block {
        Number
        Time
      }
    }
  }
}
```

**Subscription (WebSocket)**:
```graphql
subscription {
  EVM(network: eth, dataset: realtime) {
    Blocks {
      Block {
        Number
        Time
      }
    }
  }
}
```

---

## Common Subscription Types

| Data Type | Description | Update Frequency | Example Use Case |
|-----------|-------------|------------------|------------------|
| Blocks | New blocks | Per block (~12s ETH) | Block explorer |
| Transactions | New transactions | Real-time | Transaction monitor |
| Transfers | Token transfers | Real-time | Wallet tracker |
| DEXTrades | DEX trades | Real-time | Trading bot, price feed |
| Events | Smart contract events | Real-time | Event listener |
| MempoolTransactions | Pending txs | Real-time | MEV bot, mempool monitor |
| BalanceUpdates | Balance changes | Real-time | Portfolio tracker |
| NFTTrades | NFT sales | Real-time | NFT marketplace |

All subscriptions support:
- **Auth**: Yes (OAuth token required)
- **Free**: Limited (2 streams on free tier)
- **Dataset**: `realtime` (for subscriptions)

---

## Subscription Format

### Subscribe Message

**graphql-transport-ws**:
```json
{
  "id": "1",
  "type": "subscribe",
  "payload": {
    "query": "subscription { EVM(network: eth, dataset: realtime) { Blocks { Block { Number Time } } } }"
  }
}
```

**graphql-ws** (legacy):
```json
{
  "id": "1",
  "type": "start",
  "payload": {
    "query": "subscription { EVM(network: eth, dataset: realtime) { Blocks { Block { Number Time } } } }"
  }
}
```

### With Variables
```json
{
  "id": "2",
  "type": "subscribe",
  "payload": {
    "query": "subscription($network: String!) { EVM(network: $network, dataset: realtime) { Blocks { Block { Number } } } }",
    "variables": {
      "network": "eth"
    }
  }
}
```

### Unsubscribe Message

**graphql-transport-ws**:
```json
{
  "id": "1",
  "type": "complete"
}
```

**graphql-ws**:
```json
{
  "id": "1",
  "type": "stop"
}
```

### Subscription Confirmation
No explicit confirmation - data starts flowing immediately after subscribe.

---

## Message Formats (for Common Subscriptions)

### 1. New Block Subscription

**Subscribe**:
```json
{
  "id": "blocks",
  "type": "subscribe",
  "payload": {
    "query": "subscription { EVM(network: eth, dataset: realtime) { Blocks { Block { Number Time Hash GasUsed } } } }"
  }
}
```

**Incoming Data** (graphql-transport-ws):
```json
{
  "id": "blocks",
  "type": "next",
  "payload": {
    "data": {
      "EVM": {
        "Blocks": [
          {
            "Block": {
              "Number": 18500000,
              "Time": "2024-01-15T10:30:45Z",
              "Hash": "0xabc123...",
              "GasUsed": 15000000
            }
          }
        ]
      }
    }
  }
}
```

**graphql-ws** uses `"type": "data"` instead of `"next"`.

---

### 2. DEX Trades Subscription

**Subscribe**:
```json
{
  "id": "trades",
  "type": "subscribe",
  "payload": {
    "query": "subscription { EVM(network: eth, dataset: realtime) { DEXTrades(where: {Trade: {Dex: {ProtocolName: {is: \"uniswap_v3\"}}}}) { Block { Time } Trade { Buy { Amount Price Currency { Symbol } } Sell { Amount Currency { Symbol } } Dex { ProtocolName } } Transaction { Hash } } } }"
  }
}
```

**Incoming Data**:
```json
{
  "id": "trades",
  "type": "next",
  "payload": {
    "data": {
      "EVM": {
        "DEXTrades": [
          {
            "Block": {
              "Time": "2024-01-15T10:31:00Z"
            },
            "Trade": {
              "Buy": {
                "Amount": 1.5,
                "Price": 2500.0,
                "Currency": {
                  "Symbol": "WETH"
                }
              },
              "Sell": {
                "Amount": 3750.0,
                "Currency": {
                  "Symbol": "USDC"
                }
              },
              "Dex": {
                "ProtocolName": "uniswap_v3"
              }
            },
            "Transaction": {
              "Hash": "0xdef456..."
            }
          }
        ]
      }
    }
  }
}
```

---

### 3. Token Transfers Subscription

**Subscribe**:
```json
{
  "id": "transfers",
  "type": "subscribe",
  "payload": {
    "query": "subscription { EVM(network: eth, dataset: realtime) { Transfers(where: {Transfer: {Currency: {SmartContract: {is: \"0xdac17f958d2ee523a2206206994597c13d831ec7\"}}}}) { Transfer { Amount Sender Receiver Currency { Symbol } } Block { Time } Transaction { Hash } } } }"
  }
}
```

**Incoming Data**:
```json
{
  "id": "transfers",
  "type": "next",
  "payload": {
    "data": {
      "EVM": {
        "Transfers": [
          {
            "Transfer": {
              "Amount": 1000.0,
              "Sender": "0x123...",
              "Receiver": "0x456...",
              "Currency": {
                "Symbol": "USDT"
              }
            },
            "Block": {
              "Time": "2024-01-15T10:31:12Z"
            },
            "Transaction": {
              "Hash": "0xghi789..."
            }
          }
        ]
      }
    }
  }
}
```

---

### 4. Mempool Transactions Subscription

**Subscribe**:
```json
{
  "id": "mempool",
  "type": "subscribe",
  "payload": {
    "query": "subscription { EVM(network: eth, dataset: realtime) { MempoolTransactions(limit: {count: 100}) { Transaction { Hash From To Value GasPrice } } } }"
  }
}
```

**Incoming Data**:
```json
{
  "id": "mempool",
  "type": "next",
  "payload": {
    "data": {
      "EVM": {
        "MempoolTransactions": [
          {
            "Transaction": {
              "Hash": "0xjkl012...",
              "From": "0xabc...",
              "To": "0xdef...",
              "Value": 0.5,
              "GasPrice": 30000000000
            }
          }
        ]
      }
    }
  }
}
```

---

### 5. NFT Trades Subscription

**Subscribe**:
```json
{
  "id": "nft-trades",
  "type": "subscribe",
  "payload": {
    "query": "subscription { EVM(network: eth, dataset: realtime) { DEXTrades(where: {Trade: {Buy: {Currency: {Fungible: false}}}}) { Trade { Buy { Amount Currency { Symbol SmartContract Fungible } } Sell { Amount Currency { Symbol } PriceInUSD } Dex { ProtocolName } } Transaction { Hash } Block { Time } } } }"
  }
}
```

**Incoming Data**:
```json
{
  "id": "nft-trades",
  "type": "next",
  "payload": {
    "data": {
      "EVM": {
        "DEXTrades": [
          {
            "Trade": {
              "Buy": {
                "Amount": 1,
                "Currency": {
                  "Symbol": "BAYC",
                  "SmartContract": "0xbc4ca0eda7647a8ab7c2061c2e118a18a936f13d",
                  "Fungible": false
                }
              },
              "Sell": {
                "Amount": 25.5,
                "Currency": {
                  "Symbol": "WETH"
                },
                "PriceInUSD": 63750.0
              },
              "Dex": {
                "ProtocolName": "seaport"
              }
            },
            "Transaction": {
              "Hash": "0xmno345..."
            },
            "Block": {
              "Time": "2024-01-15T10:32:00Z"
            }
          }
        ]
      }
    }
  }
}
```

---

## Heartbeat / Ping-Pong

**CRITICAL**: Bitquery uses protocol-specific keepalive mechanisms.

### graphql-transport-ws (Modern)

#### Server → Client
- **Message**: `{"type": "pong"}`
- **Frequency**: Periodic (every ~30 seconds typical)
- **Purpose**: Keep connection alive

#### Client → Server
- **Optional**: Client can send `{"type": "ping"}`
- **Not required**: Server keepalive is automatic

**Example**:
```
Server → Client: {"type": "pong"}
Client → Server: (no response needed, but can send {"type": "ping"})
```

---

### graphql-ws (Legacy)

#### Server → Client
- **Message**: `{"type": "ka"}` (keepalive)
- **Frequency**: Periodic (~30 seconds)
- **Purpose**: Connection health check

#### Client → Server
- **No response needed**: Just acknowledge receipt

**Example**:
```
Server → Client: {"type": "ka"}
Client: (no action required)
```

---

### Timeout / Disconnection
- **No keepalive received**: If no `pong`/`ka` for 10+ seconds, consider connection dead
- **Reconnect logic required**: Client should implement auto-reconnect
- **Important**: Bitquery docs emphasize implementing reconnect logic if no data/keepalive for 10 seconds

### Client Implementation Recommendation
```javascript
let lastMessageTime = Date.now();
let reconnectTimeout = null;

ws.onmessage = (event) => {
  lastMessageTime = Date.now();
  clearTimeout(reconnectTimeout);

  // Handle message...

  // Reset timeout
  reconnectTimeout = setTimeout(() => {
    if (Date.now() - lastMessageTime > 10000) {
      console.log('No keepalive, reconnecting...');
      ws.close();
      reconnect();
    }
  }, 10000);
};
```

---

## Connection Limits

### Max Connections
- **Per IP**: Not explicitly limited
- **Per API key**: Not explicitly limited
- **Recommendation**: Single WebSocket for multiple subscriptions

### Max Subscriptions per Connection
- **Free tier**: 2 simultaneous streams (subscriptions)
- **Commercial plan**: Unlimited simultaneous streams
- **Important**: Each **cube** in a subscription counts separately
  - Example: 1 subscription with 2 cubes = 2 streams

### Message Rate Limit
- **Not explicitly documented**
- **Server may throttle**: Yes, based on dataset and plan
- **Realtime dataset**: Sub-second updates possible

### Auto-disconnect
- **No time limit** mentioned in docs
- **Connection lifetime**: Can be indefinite if keepalives maintained
- **Idle timeout**: Not specified (assume none if keepalives work)
- **Important**: Must close WebSocket explicitly (no 'close' message support)

---

## Closing Connections

### Critical Limitation
**"Websockets using Bitquery GraphQL streams cannot send 'close' messages"**

### How to Close
- **Client-side**: Call `ws.close()` directly on WebSocket object
- **No graceful close message**: Cannot send `{"type": "complete"}` to end all subscriptions and close connection
- **Per-subscription close**: Send `{"type": "complete", "id": "..."}` to stop individual subscription
- **Full disconnect**: Close WebSocket socket entirely

**Example**:
```javascript
// Stop individual subscription
ws.send(JSON.stringify({
  "id": "trades",
  "type": "complete"
}));

// Close entire connection
ws.close();
```

---

## Billing Model

### Subscription Costs
**Internally**: 40 points per minute per subscription

**Examples**:
- 1 subscription for 10 minutes = 400 points (10 × 40)
- 2 subscriptions for 10 minutes = 800 points (2 × 10 × 40)
- 5 subscriptions for 1 hour = 12,000 points (5 × 60 × 40)

### Free Tier
- **2 simultaneous streams** allowed
- **10,000 points/month** total budget
- **Max runtime**: ~4 hours/month per stream (10,000 / (2 × 40) / 60)

### Commercial Plan
- **Unlimited streams**
- **Custom points allocation**
- **New pricing**: Per simultaneous streams (flat rate, not points)

### Important Notes
1. **Connection ≠ Stream**: Multiple subscriptions on 1 WebSocket = multiple streams
2. **Cube-based counting**: Each cube in subscription = separate stream
3. **Realtime dataset**: Subscriptions automatically use realtime

---

## Error Handling

### Connection Errors
```json
{
  "type": "error",
  "payload": {
    "message": "Authentication failed"
  }
}
```

### Subscription Errors
```json
{
  "id": "trades",
  "type": "error",
  "payload": {
    "errors": [
      {
        "message": "Invalid query syntax"
      }
    ]
  }
}
```

### Common Errors
- **401/403**: Invalid or missing token
- **429**: Rate limit exceeded (too many streams)
- **500**: Internal server error
- **WebSocket 1009**: Payload too big (increase `max_size` in client)

---

## Code Examples

### Python (graphql-ws library)
```python
from gql import Client, gql
from gql.transport.websockets import WebsocketsTransport

transport = WebsocketsTransport(
    url='wss://streaming.bitquery.io/graphql?token=YOUR_TOKEN',
    subprotocols=['graphql-transport-ws']
)

client = Client(transport=transport, fetch_schema_from_transport=False)

subscription = gql('''
subscription {
  EVM(network: eth, dataset: realtime) {
    Blocks {
      Block {
        Number
        Time
      }
    }
  }
}
''')

for result in client.subscribe(subscription):
    print(result)
```

### JavaScript (graphql-ws)
```javascript
import { createClient } from 'graphql-ws';

const client = createClient({
  url: 'wss://streaming.bitquery.io/graphql?token=YOUR_TOKEN',
});

const subscription = `
subscription {
  EVM(network: eth, dataset: realtime) {
    DEXTrades {
      Trade {
        Buy { Amount Price Currency { Symbol } }
        Sell { Amount Currency { Symbol } }
      }
    }
  }
}
`;

client.subscribe(
  { query: subscription },
  {
    next: (data) => console.log(data),
    error: (error) => console.error(error),
    complete: () => console.log('Done'),
  }
);
```

### Rust (tokio-tungstenite)
See: https://docs.bitquery.io/docs/subscriptions/example-rust/

---

## Best Practices

1. **Use graphql-transport-ws**: Modern, better maintained
2. **Implement reconnect logic**: Reconnect if no message for 10 seconds
3. **Token in URL**: Simplest authentication method
4. **Monitor keepalives**: Track `pong`/`ka` messages
5. **Single WebSocket**: Use one connection for multiple subscriptions (if within limits)
6. **Close explicitly**: Call `ws.close()` when done
7. **Handle errors**: Check for `error` type messages
8. **Test free tier limits**: 2 streams max, plan accordingly

---

## Postman Collection
Available for testing: Check Bitquery docs for Postman WebSocket examples.
