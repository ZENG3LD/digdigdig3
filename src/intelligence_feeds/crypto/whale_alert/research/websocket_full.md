# Whale Alert - WebSocket Documentation

## Availability: Yes

Whale Alert provides two WebSocket APIs:
1. **Custom Alerts API** - Standard tier ($29.95/month)
2. **Priority Alerts API** - Professional tier ($1,299/month)

Both use the same connection endpoint but have different rate limits and latency.

---

## Connection

### URLs
- Custom Alerts: `wss://leviathan.whale-alert.io/ws?api_key=YOUR_API_KEY`
- Priority Alerts: `wss://leviathan.whale-alert.io/ws?api_key=YOUR_API_KEY` (same endpoint)
- Regional: None (single global endpoint)

### Connection Process
1. Connect to WebSocket URL with API key in query parameter
2. Connection established immediately
3. No separate welcome/handshake message required
4. Send subscription message to start receiving alerts
5. Authentication is via API key in connection URL

### Authentication Method
- API key passed as URL query parameter: `?api_key=YOUR_API_KEY`
- No separate authentication message required
- Invalid API key will result in connection rejection

---

## ALL Available Channels/Topics

**CRITICAL:** Whale Alert WebSocket has two subscription types, not traditional "channels"

| Channel/Topic | Type | Description | Auth? | Free? | Update Frequency | Example Subscription |
|---------------|------|-------------|-------|-------|------------------|---------------------|
| subscribe_alerts | Subscription | Transaction alerts based on filters | Yes | No ($29.95/mo) | Real-time | See below |
| subscribe_socials | Subscription | Social media posts from Whale Alert | Yes | No ($29.95/mo) | Real-time | See below |

---

## Subscription Format

### Subscribe to Alerts

```json
{
  "type": "subscribe_alerts",
  "blockchains": ["ethereum", "bitcoin"],
  "symbols": ["eth", "weth", "btc"],
  "tx_types": ["transfer", "mint", "burn"],
  "min_value_usd": 1000000,
  "channel_id": "optional_custom_id"
}
```

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| type | string | Yes | Must be "subscribe_alerts" |
| blockchains | array[string] | No | Filter by blockchain names (omit for all) |
| symbols | array[string] | No | Filter by currency symbols (omit for all) |
| tx_types | array[string] | No | Filter by transaction type (omit for all) |
| min_value_usd | float | Yes | Minimum transaction value in USD (minimum: $100,000) |
| channel_id | string | No | Custom identifier for this subscription (random ID assigned if omitted) |

**Transaction Types:**
- `transfer` - Standard transfers
- `mint` - Token/coin minting
- `burn` - Token/coin burning
- `freeze` - Asset freezing
- `unfreeze` - Asset unfreezing
- `lock` - Asset locking
- `unlock` - Asset unlocking

### Subscribe to Social Media Posts

```json
{
  "type": "subscribe_socials",
  "channel_id": "optional_custom_id"
}
```

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| type | string | Yes | Must be "subscribe_socials" |
| channel_id | string | No | Custom identifier for this subscription |

### Subscription Confirmation

After subscribing to alerts:

```json
{
  "type": "subscribed_alerts",
  "channel_id": "8QFdN74g",
  "blockchains": ["ethereum"],
  "symbols": ["eth", "weth"],
  "tx_types": ["transfer"],
  "min_value_usd": 1000000
}
```

After subscribing to socials:

```json
{
  "type": "subscribed_socials",
  "channel_id": "xlLZ7tJq"
}
```

### Unsubscribe (if supported)

**Note:** Documentation does not explicitly mention unsubscribe functionality. Likely handled by closing connection or sending new subscription.

---

## Message Formats (for EVERY channel)

### Alert Message (Transaction Alert)

Complete alert format with all transaction details:

```json
{
  "channel_id": "8QFdN74g",
  "timestamp": 1692724660,
  "blockchain": "ethereum",
  "transaction_type": "transfer",
  "from": "Binance",
  "to": "unknown",
  "amounts": [
    {
      "symbol": "USDC",
      "amount": 50000000.0,
      "value_usd": 50000000.0
    }
  ],
  "text": "🔥 50,000,000 #USDC (50,000,000 USD) transferred from Binance to unknown wallet\n\nhttps://whale-alert.io/transaction/ethereum/0x...",
  "transaction": {
    "height": 17887234,
    "index_in_block": 145,
    "timestamp": 1692724660,
    "hash": "0x1234567890abcdef...",
    "fee": "0.003456",
    "fee_symbol": "ETH",
    "fee_symbol_price": "1650.50",
    "sub_transactions": [
      {
        "symbol": "USDC",
        "unit_price_usd": "1.00",
        "transaction_type": "transfer",
        "inputs": [
          {
            "amount": "50000000.0",
            "address": "0xabc123...",
            "balance": "250000000.0",
            "locked": "0",
            "is_frozen": false,
            "owner": "Binance",
            "owner_type": "exchange",
            "address_type": "hot_wallet"
          }
        ],
        "outputs": [
          {
            "amount": "50000000.0",
            "address": "0xdef456...",
            "balance": "50000000.0",
            "locked": "0",
            "is_frozen": false,
            "owner": "",
            "owner_type": "unknown",
            "address_type": "unknown"
          }
        ]
      }
    ]
  }
}
```

**Root Alert Object Fields:**

| Field | Type | Description |
|-------|------|-------------|
| channel_id | string | Subscription channel identifier |
| timestamp | int | Unix timestamp of transaction |
| blockchain | string | Blockchain name (ethereum, bitcoin, etc.) |
| transaction_type | string | Type of transaction (transfer, mint, burn, etc.) |
| from | string | Owner name of sender address(es) |
| to | string | Owner name of recipient address(es) |
| amounts | array[Amount] | Array of currency amounts in transaction |
| text | string | Human-readable description (similar to Twitter posts) |
| transaction | Transaction | Complete transaction object |

**Amount Object:**

| Field | Type | Description |
|-------|------|-------------|
| symbol | string | Currency symbol (USDC, ETH, BTC, etc.) |
| amount | float | Amount of tokens/coins |
| value_usd | float | USD value of amount |

**Transaction Object:**

| Field | Type | Description |
|-------|------|-------------|
| height | int | Block height containing transaction |
| index_in_block | int | Position within the block |
| timestamp | int | Unix timestamp of block |
| hash | string | Transaction hash/identifier |
| fee | string | Transaction fee amount (string for precision) |
| fee_symbol | string | Currency of the fee |
| fee_symbol_price | string | USD price per unit at block time |
| sub_transactions | array[SubTransaction] | Individual value transfers |

**SubTransaction Object:**

| Field | Type | Description |
|-------|------|-------------|
| symbol | string | Currency symbol (single per sub-transaction) |
| unit_price_usd | string | USD conversion rate at block time |
| transaction_type | string | Type: transfer, mint, burn, freeze, unfreeze, lock, unlock |
| inputs | array[Address] | Source addresses (FROM) |
| outputs | array[Address] | Destination addresses (TO) |

**Address Object:**

| Field | Type | Description |
|-------|------|-------------|
| amount | string | Balance change quantity |
| address | string | Wallet identifier hash |
| balance | string | Post-transaction balance |
| locked | string | Non-transferable balance amount |
| is_frozen | bool | Frozen status indicator |
| owner | string | Attributed entity name (empty if unknown) |
| owner_type | string | Entity classification (see below) |
| address_type | string | Wallet category (see below) |

**Owner Types:**
- `exchange` - Cryptocurrency exchange
- `unknown` - Unidentified owner
- (Additional types exist but not fully documented)

**Address Types:**
- `hot_wallet` - Exchange hot wallet
- `cold_wallet` - Cold storage wallet
- `deposit_wallet` - Deposit wallet
- `exchange_wallet` - General exchange wallet
- `burn_address` - Burn address (unrecoverable)
- `mixer_wallet` - Mixer/tumbler wallet
- `coinjoin` - CoinJoin address
- `fraud_wallet` - Known fraud wallet
- `unknown` - Unclassified

### Social Media Alert

Social media posts from Whale Alert's official channels:

```json
{
  "channel_id": "xlLZ7tJq",
  "timestamp": 1692724660,
  "blockchain": "tron",
  "text": "🔥 🔥 🔥 🔥 🔥 🔥 🔥 🔥 🔥 🔥 1,200,000,000 #USDT (1,200,398,999 USD) burned at Tether Treasury\n\nhttps://whale-alert.io/transaction/tron/cf5b1ae18be3d3596a9920c0dffce82c5247e9672b4ff7b1194d0355e5bec470",
  "urls": [
    "https://twitter.com/whale_alert/status/1694036126422450598",
    "https://t.me/whale_alert_io/72364"
  ]
}
```

**Social Alert Fields:**

| Field | Type | Description |
|-------|------|-------------|
| channel_id | string | Subscription channel identifier |
| timestamp | int | Unix timestamp when posted |
| blockchain | string | Related blockchain |
| text | string | Social media post text |
| urls | array[string] | Links to posts on Twitter, Telegram, etc. |

---

## Heartbeat / Ping-Pong

**CRITICAL:** Documentation does not explicitly mention ping/pong mechanism

### Who initiates?
- Server → Client ping: Not documented
- Client → Server ping: Not documented (likely standard WebSocket ping frames)

### Message Format
- Binary ping/pong frames: Likely (standard WebSocket protocol)
- Text messages: Not documented
- JSON messages: Not documented

### Timing
- Ping interval: Not specified
- Timeout: Not specified
- Client must send ping: Not specified

### Recommendation
Use standard WebSocket ping/pong frames. Implementation should:
- Respond to WebSocket PING frames with PONG frames
- Send periodic PING frames (every 30-60 seconds recommended)
- Implement reconnection logic if no messages received for extended period

---

## Connection Limits

### Custom Alerts API
- Max connections per API key: 2 concurrent connections
- Max alerts per hour: 100 alerts
- Message rate limit: Not explicitly specified
- Auto-disconnect after: Not specified

### Priority Alerts API
- Max connections per API key: 2 concurrent connections
- Max alerts per hour: 10,000 alerts (technically unlimited)
- Message rate limit: Not explicitly specified
- Auto-disconnect after: Not specified
- **Latency advantage:** Up to 1 minute faster than Custom Alerts

### Subscription Limits
- Max subscriptions per connection: Not explicitly specified (likely multiple subscriptions possible)
- Filters per subscription: Multiple blockchains, symbols, and tx_types supported

---

## Error Handling

**Note:** Error format not fully documented. Based on standard WebSocket practices:

### Connection Errors
- Invalid API key: Connection rejected (401/403 equivalent)
- Rate limit exceeded: Connection may be throttled or closed
- Invalid subscription: Likely error message or no confirmation

### Expected Error Format (inferred)
```json
{
  "type": "error",
  "code": "ERROR_CODE",
  "message": "Error description"
}
```

---

## Reconnection Strategy

**Recommended Implementation:**

1. **Automatic Reconnection:** Implement exponential backoff on disconnect
2. **Subscription Recovery:** Re-subscribe to alerts after reconnection
3. **State Management:** Track channel_id and subscription parameters
4. **Error Handling:** Retry on network errors, abort on authentication errors

**Reconnection Example (from GitHub Go implementation):**
- Retry up to 3 times with pauses between attempts
- Use context/signal handling for graceful shutdown
- Implement read loops that trigger reconnection on error

---

## Code Example (Connection & Subscription)

### JavaScript/Node.js Example

```javascript
const WebSocket = require('ws');

const API_KEY = 'YOUR_API_KEY';
const ws = new WebSocket(`wss://leviathan.whale-alert.io/ws?api_key=${API_KEY}`);

ws.on('open', () => {
  console.log('Connected to Whale Alert');

  // Subscribe to alerts
  ws.send(JSON.stringify({
    type: 'subscribe_alerts',
    blockchains: ['ethereum', 'bitcoin'],
    symbols: ['eth', 'btc', 'usdt', 'usdc'],
    tx_types: ['transfer'],
    min_value_usd: 1000000,
    channel_id: 'my_channel_1'
  }));
});

ws.on('message', (data) => {
  const message = JSON.parse(data.toString());

  switch(message.type) {
    case 'subscribed_alerts':
      console.log('Subscription confirmed:', message);
      break;
    case 'alert':
      console.log('Transaction alert:', message.text);
      console.log('Value:', message.amounts);
      break;
    case 'subscribed_socials':
      console.log('Social subscription confirmed');
      break;
    default:
      console.log('Unknown message type:', message);
  }
});

ws.on('error', (error) => {
  console.error('WebSocket error:', error);
});

ws.on('close', () => {
  console.log('Connection closed');
  // Implement reconnection logic here
});
```

### Go Example (from official GitHub)

```go
import (
    "context"
    "fmt"
    "nhooyr.io/websocket"
    "time"
)

type AlertSubscription struct {
    Type        string   `json:"type"`
    Blockchains []string `json:"blockchains,omitempty"`
    Symbols     []string `json:"symbols,omitempty"`
    Types       []string `json:"tx_types,omitempty"`
    MinValueUSD float64  `json:"min_value_usd"`
}

func connectAndSubscribe(apiKey string) {
    ctx := context.Background()
    url := fmt.Sprintf("wss://leviathan.whale-alert.io/ws?api_key=%s", apiKey)

    conn, _, err := websocket.Dial(ctx, url, nil)
    if err != nil {
        panic(err)
    }
    defer conn.Close(websocket.StatusNormalClosure, "")

    // Subscribe
    sub := AlertSubscription{
        Type:        "subscribe_alerts",
        Blockchains: []string{"ethereum"},
        Symbols:     []string{"eth", "usdc"},
        Types:       []string{"transfer"},
        MinValueUSD: 1000000,
    }

    err = conn.Write(ctx, websocket.MessageText, marshalJSON(sub))
    if err != nil {
        panic(err)
    }

    // Read loop
    for {
        _, message, err := conn.Read(ctx)
        if err != nil {
            // Implement reconnection logic
            break
        }
        handleMessage(message)
    }
}
```

---

## Notes

1. **Minimum Value:** All alert subscriptions require `min_value_usd` >= $100,000
2. **Multiple Subscriptions:** You can send multiple subscription messages on the same connection
3. **Channel ID:** Custom channel_id helps track which subscription triggered each alert
4. **Priority Alerts:** Same API, just faster delivery (up to 1 minute advantage) and higher rate limits
5. **Social Alerts:** Receive Whale Alert's official Twitter/Telegram posts in real-time
6. **Attribution Data:** Alerts include rich metadata about addresses (owner, type, balance)
7. **Multi-Currency Transactions:** Single transaction can contain multiple currencies (see sub_transactions array)
