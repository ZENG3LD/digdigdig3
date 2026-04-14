# Crypto.com Exchange WebSocket API - Confirmation Report

Researched: 2026-03-02
Source: https://exchange-docs.crypto.com/exchange/v1/rest-ws/index.html

---

## Q1: Does Crypto.com Have a Public WebSocket API for Real-Time Market Data?

**Yes.** Crypto.com Exchange provides a fully documented, production-grade WebSocket API with
separate endpoints for public market data and private user data.

---

## Q2: WebSocket Endpoints

### Spot + Derivatives (Exchange API v1)

| Purpose | Production URL |
|---------|---------------|
| Market Data (public) | `wss://stream.crypto.com/exchange/v1/market` |
| User API (private) | `wss://stream.crypto.com/exchange/v1/user` |

### Derivatives-Only (Derivatives API)

| Purpose | Production URL |
|---------|---------------|
| Market Data (public) | `wss://deriv-stream.crypto.com/v1/market` |
| User API (private) | `wss://deriv-stream.crypto.com/v1/user` |

### UAT Sandbox (Exchange API v1)

| Purpose | Sandbox URL |
|---------|------------|
| Market Data | `wss://uat-stream.3ona.co/exchange/v1/market` |
| User API | `wss://uat-stream.3ona.co/exchange/v1/user` |

### UAT Sandbox (Derivatives API)

| Purpose | Sandbox URL |
|---------|------------|
| Market Data | `wss://uat-deriv-stream.3ona.co/v1/market` |
| User API | `wss://uat-deriv-stream.3ona.co/v1/user` |

---

## Q3: Channels and Streams Available

### Public Market Data Channels

All available on the market data endpoint without authentication.

#### Ticker — `ticker.{instrument_name}`

Example: `ticker.BTCUSD-PERP`, `ticker.BTC_USDT`

Response fields:
- `i` — instrument name
- `b` — best bid price
- `k` — best ask price
- `a` — last trade price
- `c` — 24h price change (percentage)
- `h` — 24h high
- `l` — 24h low
- `v` — 24h volume (base currency)
- `vv` — 24h volume (quote currency / USD)
- `oi` — open interest (derivatives)
- `t` — timestamp (ms)

Example response:
```json
{
  "method": "subscribe",
  "result": {
    "channel": "ticker",
    "subscription": "ticker.BTCUSD-PERP",
    "instrument_name": "BTCUSD-PERP",
    "data": [{
      "i": "BTCUSD-PERP",
      "b": "51170.000000",
      "k": "51180.000000",
      "a": "51174.500000",
      "c": "0.03955106",
      "h": "51790.00",
      "l": "47895.50",
      "v": "879.5024",
      "vv": "26370000.12",
      "oi": "12345.12",
      "t": 1613580710768
    }]
  }
}
```

#### Order Book — `book.{instrument_name}.{depth}`

Examples: `book.BTCUSD-PERP.10`, `book.BTC_USDT.50`

Supported depths: `10`, `50`

Two update types exist in the data stream:
- `tt: "SNAPSHOT"` — Full order book, sent every 500ms
- `tt: "DELTA"` — Incremental changes, sent every 100ms (active subscription type as of 2025-02-27; the pure SNAPSHOT 100ms mode was removed)

Entry format: `[price, quantity, order_count]`
- A quantity of `"0.000000"` means remove that price level.

Snapshot example:
```json
{
  "result": {
    "channel": "book",
    "subscription": "book.BTCUSD-PERP.10",
    "instrument_name": "BTCUSD-PERP",
    "depth": 10,
    "data": [{
      "bids": [
        ["50113.500000", "0.400000", "0"],
        ["50113.000000", "0.051800", "0"]
      ],
      "asks": [
        ["50126.000000", "0.400000", "0"],
        ["50130.000000", "1.279000", "0"]
      ],
      "t": 1613580710768,
      "tt": "SNAPSHOT"
    }]
  }
}
```

Delta example:
```json
{
  "result": {
    "channel": "book",
    "data": [{
      "bids": [["50115.000000", "0.500000", "0"]],
      "asks": [],
      "t": 1613580710868,
      "tt": "DELTA"
    }]
  }
}
```

#### Trades — `trade.{instrument_name}`

Example: `trade.BTCUSD-PERP`, `trade.BTC_USDT`

Response fields:
- `d` — trade ID
- `t` — timestamp (ms)
- `tn` — timestamp (ns)
- `p` — price
- `q` — quantity
- `s` — side: `"BUY"` or `"SELL"` (taker direction)
- `i` — instrument name
- `m` — match ID

Example response:
```json
{
  "result": {
    "channel": "trade",
    "subscription": "trade.BTCUSD-PERP",
    "instrument_name": "BTCUSD-PERP",
    "data": [{
      "d": "15281981878",
      "t": 1613547060925,
      "tn": "1613547060925523623",
      "q": "0.181900",
      "p": "50772.000000",
      "s": "SELL",
      "i": "BTCUSD-PERP",
      "m": "76423"
    }]
  }
}
```

#### Candlestick — `candlestick.{timeframe}.{instrument_name}`

Examples: `candlestick.1h.BTCUSD-PERP`, `candlestick.5m.BTC_USDT`

Supported timeframes: `1m`, `5m`, `15m`, `30m`, `1h`, `2h`, `4h`, `12h`, `1D`, `7D`, `14D`, `1M`

Response fields:
- `t` — candle open timestamp (ms)
- `o` — open price
- `h` — high price
- `l` — low price
- `c` — close price
- `v` — volume

Example response:
```json
{
  "result": {
    "channel": "candlestick",
    "subscription": "candlestick.1h.BTCUSD-PERP",
    "instrument_name": "BTCUSD-PERP",
    "interval": "1h",
    "data": [{
      "t": 1613577600000,
      "o": "50100.00",
      "h": "51500.00",
      "l": "49800.00",
      "c": "51100.00",
      "v": "123.4567"
    }]
  }
}
```

#### Derivatives-Specific Public Channels

| Channel | Format | Purpose |
|---------|--------|---------|
| Index price | `index.{instrument_name}` | Reference index price |
| Mark price | `mark.{instrument_name}` | Mark price for liquidations |
| Settlement price | `settlement.{instrument_name}` | Settlement price at expiry |
| Funding rate | `funding.{instrument_name}` | Fixed hourly funding rate |
| Estimated funding | `estimatedfunding.{instrument_name}` | Next-interval funding estimate |

### Private User Channels (require authentication)

Available only on the user endpoint after `public/auth`.

| Channel | Format | Purpose |
|---------|--------|---------|
| Orders | `user.order.{instrument_name}` | Order lifecycle events |
| Advanced orders | `user.advance.order.{instrument_name}` | OTO/OTOCO order updates |
| User trades | `user.trade.{instrument_name}` | Fill notifications with fee |
| Balance | `user.balance` | Account balance changes |
| Positions | `user.positions` | Position updates |
| Account risk | `user.account_risk` | Margin and risk metrics |
| Position balance | `user.position_balance` | Isolated margin balances |

---

## Q4: Is the WS API Available Without Authentication for Public Market Data?

**Yes.** The market data WebSocket endpoint (`wss://stream.crypto.com/exchange/v1/market`)
requires no authentication. Connect and subscribe to any public channel immediately.

Only the user endpoint (`wss://stream.crypto.com/exchange/v1/user`) requires authentication,
and only for private channels (orders, trades, balance, positions).

---

## Subscribe / Unsubscribe Message Format

Subscribe (public, no auth needed):
```json
{
  "id": 1,
  "method": "subscribe",
  "params": {
    "channels": ["ticker.BTCUSD-PERP", "trade.BTCUSD-PERP", "book.BTCUSD-PERP.10"]
  },
  "nonce": 1611022832613
}
```

Unsubscribe:
```json
{
  "id": 2,
  "method": "unsubscribe",
  "params": {
    "channels": ["ticker.BTCUSD-PERP"]
  },
  "nonce": 1611022832614
}
```

Subscription confirmation response:
```json
{
  "id": 1,
  "method": "subscribe",
  "code": 0,
  "result": {
    "subscription": "ticker.BTCUSD-PERP",
    "channel": "ticker"
  }
}
```

---

## Heartbeat Protocol

The server sends a heartbeat every 30 seconds. The client must respond within 5 seconds
or the connection is dropped.

Server sends:
```json
{
  "id": 1587523073344,
  "method": "public/heartbeat",
  "code": 0
}
```

Client must respond with the same `id`:
```json
{
  "id": 1587523073344,
  "method": "public/respond-heartbeat"
}
```

---

## Connection Rules

1. After connecting, **wait 1 second before sending any messages**. Rate limits are
   pro-rated from the calendar-second of connection opening; sending too early triggers
   `TOO_MANY_REQUESTS` (code 42901).
2. Respond to all `public/heartbeat` messages within 5 seconds.
3. Use separate WebSocket connections for market data and user data.
4. On reconnect: re-authenticate (user endpoint), then re-subscribe to all channels.

---

## Rate Limits

| Endpoint | Limit |
|----------|-------|
| Market Data WebSocket | 100 requests/second |
| User API WebSocket | 150 requests/second |
| `private/get-trades`, `private/get-order-history` | 5 requests/second |

---

## Key Error Codes

| Code | Name | Meaning |
|------|------|---------|
| 0 | OK | Success |
| 40101 | UNAUTHORIZED | Auth failed or invalid signature |
| 40102 | INVALID_NONCE | Nonce differs by more than 60 seconds |
| 40107 | EXCEED_MAX_SUBSCRIPTIONS | Session subscription limit exceeded |
| 42901 | TOO_MANY_REQUESTS | Rate limit exceeded |
| 30002 | INSTRUMENT_NOT_FOUND | Invalid symbol |

WebSocket close codes:
- `1000` — Normal closure
- `1006` — Abnormal closure (reconnect)
- `1013` — Server restart (reconnect)

---

## Sources

- [Crypto.com Exchange API v1 (REST + WS)](https://exchange-docs.crypto.com/exchange/v1/rest-ws/index.html)
- [Crypto.com Exchange Derivatives API](https://exchange-docs.crypto.com/derivatives/index.html)
- [Crypto.com Institutional API v1](https://exchange-docs.crypto.com/exchange/v1/rest-ws/index-insto-8556ea5c-4dbb-44d4-beb0-20a4d31f63a7.html)
