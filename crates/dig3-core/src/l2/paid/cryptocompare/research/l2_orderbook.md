# CryptoCompare — L2 Orderbook Capabilities

## Summary

CryptoCompare (now operating as **CCData / CoinDesk Data** after acquisition) **does provide L2 orderbook data**, but it is gated behind paid tiers and, for the enterprise real-time streaming product, requires IP whitelisting plus a commercial agreement. The data is available via three surfaces:

1. **Legacy WebSocket** — Channel 16 (`ORDERBOOK_L2`), real-time snapshot + delta updates, paid tier required
2. **Legacy REST** — `/data/ob/l2/snapshot` style endpoint, per-request L2 snapshot, paid tier
3. **New Data API (CCData)** — `data-api.cryptocompare.com` REST endpoints for historical per-minute L2 snapshots and metrics; a separate Data Streamer for real-time replay/live L2 updates (enterprise only)

For most practical purposes (non-enterprise), the legacy WebSocket Channel 16 is the primary L2 access mechanism.

---

## WebSocket Channels

### Connection

```
wss://streamer.cryptocompare.com/v2?api_key=YOUR_API_KEY
```

Authentication is via API key in the URL query parameter. No separate auth message needed.

### Channel 16 — ORDERBOOK_L2 (Legacy WebSocket)

This is the primary real-time L2 orderbook channel on the legacy streamer.

**Subscription format:**
```json
{
  "action": "SubAdd",
  "subs": ["16~{EXCHANGE}~{FROM}~{TO}"]
}
```

**Example:**
```
16~Kraken~BTC~USD
16~Binance~BTC~USDT
```

**Availability:** Paid tier only. Not available on the free plan.

**Depth:** Full depth (all levels collected from the exchange). No configurable depth parameter — the feed reflects what the exchange provides.

**Update model:** Snapshot + delta. On first subscription you receive a full orderbook snapshot. Subsequent messages are delta (changed levels only).

#### Snapshot Message (TYPE "16")

```json
{
  "TYPE": "16",
  "M": "Kraken",
  "FSYM": "BTC",
  "TSYM": "USD",
  "BIDS": [
    {"P": 45000.00, "Q": 1.5},
    {"P": 44999.50, "Q": 2.0},
    {"P": 44999.00, "Q": 0.75}
  ],
  "ASKS": [
    {"P": 45001.00, "Q": 1.2},
    {"P": 45001.50, "Q": 1.8},
    {"P": 45002.00, "Q": 0.5}
  ],
  "TS": 1706280000
}
```

**Fields:**
| Field | Type | Description |
|-------|------|-------------|
| `TYPE` | string | `"16"` — channel identifier |
| `M` | string | Market/exchange name (e.g. `"Kraken"`) |
| `FSYM` | string | From symbol (base, e.g. `"BTC"`) |
| `TSYM` | string | To symbol (quote, e.g. `"USD"`) |
| `BIDS` | array | Bid levels — each `{"P": price, "Q": quantity}` |
| `ASKS` | array | Ask levels — each `{"P": price, "Q": quantity}` |
| `TS` | int | Unix timestamp (seconds) |

#### Delta/Update Message (TYPE "16~UPDATE")

After the initial snapshot, incremental updates are sent:

```json
{
  "TYPE": "16~UPDATE",
  "M": "Kraken",
  "FSYM": "BTC",
  "TSYM": "USD",
  "BID_CHANGES": [
    {"P": 45000.00, "Q": 2.0}
  ],
  "ASK_CHANGES": [
    {"P": 45001.00, "Q": 0}
  ],
  "TS": 1706280001
}
```

**Delta semantics:**
- Only changed levels are included
- `Q: 0` means the price level was removed (deleted from the book)
- Non-zero `Q` means updated quantity at that price level

**Note:** No sequence numbers or checksums are documented for the legacy Channel 16. Gap detection must be handled by timestamp comparison or periodic re-snapshot.

### New Data Streamer (Enterprise)

CoinDesk Data also provides a separate enterprise-grade streaming product:

- **Endpoint:** `https://developers.coindesk.com/documentation/data-streamer/spot_v1_orderbook_replay_l2_updates`
- **Description:** Real-time and replay L2 order book update streaming for spot markets
- **Access:** Requires IP whitelisting and commercial agreement (contact `data@cryptocompare.com`)
- **Coverage:** 3,000+ pairs across top-tier spot exchanges
- **Depth:** Full depth of book, tick-level granularity
- **Message format:** Normalized, standardized across exchanges with CryptoCompare internal sequence numbers

---

## REST Endpoints

### Legacy REST — L2 Snapshot

**Base URL:** `https://min-api.cryptocompare.com`

The legacy API has an L2 snapshot endpoint. The legacy endpoint path follows the pattern:

```
GET /data/ob/l2/snapshot?fsym=BTC&tsym=USD&e=Kraken&api_key=YOUR_API_KEY
```

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `fsym` | string | Yes | From symbol (e.g. `BTC`) |
| `tsym` | string | Yes | To symbol (e.g. `USD`) |
| `e` | string | Yes | Exchange name |
| `api_key` | string | Yes | API key (paid tier required) |

**Response structure (example):**
```json
{
  "Response": "Success",
  "Data": {
    "M": "Kraken",
    "FSYM": "BTC",
    "TSYM": "USD",
    "BidId": 12345678,
    "AskId": 12345679,
    "Bids": [
      {"P": 45000.00, "Q": 1.5},
      {"P": 44999.50, "Q": 2.0}
    ],
    "Asks": [
      {"P": 45001.00, "Q": 1.2},
      {"P": 45001.50, "Q": 1.8}
    ],
    "TS": 1706280000
  }
}
```

**Depth:** Returns the full current orderbook as known by CryptoCompare (all levels). No configurable depth parameter on the legacy endpoint.

**Note:** The exact legacy endpoint path is not publicly indexed in detail — documentation portal at `developers.coindesk.com` is a JavaScript SPA that does not render without a browser. The path above is reconstructed from available evidence. Actual path confirmed in legacy docs as `obL2SnapshotEndpoint`.

### New Data API (CCData) — Historical L2 Snapshots

**Base URL:** `https://data-api.cryptocompare.com` (also: `https://data-api.ccdata.io`)

These endpoints serve **historical** per-minute L2 snapshots, not real-time:

| Endpoint | Description |
|----------|-------------|
| `GET /spot/v2/historical/orderbook/l2/snapshots/minute` | Historical per-exchange per-minute L2 snapshots (v2) |
| `GET /spot/v1/historical/orderbook/l2/metrics/minute` | Historical per-minute L2 derived metrics (spread, depth, slippage) |
| `GET /spot/v1/historical/orderbook/l2/consolidated/snapshots/minute` | Consolidated (cross-exchange) per-minute L2 snapshots |
| `GET /spot/v1/historical/orderbook/l2/consolidated/metrics/minute` | Consolidated per-minute L2 metrics |

**Example request (pattern):**
```bash
curl -X GET "https://data-api.cryptocompare.com/spot/v2/historical/orderbook/l2/snapshots/minute?market=kraken&instrument=BTC-USD&limit=10&api_key=YOUR_API_KEY"
```

**Common parameters (inferred from spot API patterns):**
| Name | Type | Description |
|------|------|-------------|
| `market` | string | Exchange name (e.g. `kraken`, `coinbase`) |
| `instrument` | string | Trading pair in exchange format (e.g. `BTC-USD`) |
| `limit` | int | Number of records (max varies by plan) |
| `toTs` | timestamp | End timestamp (Unix seconds) |
| `api_key` | string | API key |

**Historical coverage:** Data available since September 2020 (exchange dependent).

**Snapshot frequency:** Once per minute per exchange.

**Futures equivalents also exist:**
- `GET /futures/v2/historical/orderbook/l2/snapshots/minute`
- `GET /futures/v2/historical/orderbook/l2/metrics/minute`

**Options equivalent:**
- `GET /options/v1/historical/orderbook/l2/snapshots/minute`

---

## Aggregation, Checksums, Sequence Numbers

### Legacy WebSocket (Channel 16)
- **No sequence numbers** documented in the legacy Channel 16 protocol
- **No checksums** documented
- Delta updates (BID_CHANGES / ASK_CHANGES) must be applied to the in-memory snapshot
- Gap detection: use `TS` field; periodic full re-snapshot recommended after network issues

### New Data Streamer (Enterprise)
- Messages include a **CryptoCompare sequence number** (`CCSEQ`) for ordering and gap detection
- This is the normalized, enterprise-grade product with proper sequencing
- Confirmed field: `"CCSEQ"` — CryptoCompare internal sequence number on L2 update messages

### Historical REST
- Snapshots are deterministic (one per minute per exchange)
- No checksums needed — data is at-rest

---

## Tier Requirements

| Feature | Free Tier | Starter (~$80/mo) | Professional (~$200/mo) | Enterprise (custom) |
|---------|-----------|-------------------|-------------------------|---------------------|
| WebSocket trades (Ch. 0) | Yes | Yes | Yes | Yes |
| WebSocket ticker (Ch. 2, 5) | Yes | Yes | Yes | Yes |
| WebSocket L2 orderbook (Ch. 16) | **No** | **Yes** | Yes | Yes |
| Legacy REST L2 snapshot | **No** | Likely yes | Yes | Yes |
| Historical L2 snapshots (CCData) | **No** | Limited | Yes | Full history |
| Real-time Data Streamer (L2 updates) | **No** | **No** | **No** | **Yes (IP whitelist)** |
| Historical L2 metrics | **No** | Limited | Yes | Full history |

**Free tier:** No L2 orderbook access at all. Channel 16 subscriptions will be rejected.

**Paid tiers:** WebSocket Channel 16 is available on Starter and above.

**Enterprise real-time streamer:** Requires a commercial agreement plus IP whitelisting; contact `data@cryptocompare.com`.

---

## Exchange Coverage for Orderbook

Coverage is exchange-dependent. CryptoCompare aggregates from 170+ exchanges total, but L2 orderbook data via Channel 16 is confirmed available for:
- Kraken
- Binance
- Coinbase
- Plus other major spot exchanges integrated in the feed

Real-time orderbook feed covers **3,000+ cryptocurrency pairs** on top-tier spot exchanges per the CCData press release.

---

## Key Limitations

1. **No L3 data** — No individual order-level (L3) data. Only aggregated price levels.
2. **No configurable depth on legacy** — Legacy Channel 16 returns all levels; no depth parameter.
3. **No sequence numbers on legacy WebSocket** — Channel 16 does not document sequence numbers or checksums, making gap detection harder than exchange-native feeds.
4. **Enterprise streaming requires IP whitelist** — The production-grade enterprise Data Streamer is not self-serve.
5. **Real-time is aggregator quality, not exchange native** — CryptoCompare normalizes and re-sequences data, introducing small latency versus direct exchange WebSocket connections.
6. **Historical snapshots are once-per-minute** — Not tick-by-tick for historical data.
7. **Instrument naming varies** — New CCData API uses `BTC-USD` format vs legacy `BTC/USD` or `BTC~USD` format.

---

## Comparison: Legacy vs New API

| Aspect | Legacy (min-api / streamer) | New CCData (data-api) |
|--------|----------------------------|------------------------|
| WebSocket base | `wss://streamer.cryptocompare.com/v2` | Separate enterprise endpoint |
| Channel format | `16~EXCHANGE~FROM~TO` | Via Data Streamer subscription |
| Instrument format | `BTC`, `USD` separate fields | `BTC-USD` combined |
| Sequence numbers | None | Yes (`CCSEQ`) |
| Real-time | Yes (Channel 16) | Enterprise only |
| Historical | No | Yes (per-minute snapshots) |
| Access | Paid API key | API key + potential IP whitelist |

---

## Sources

- [CoinDesk Data — Order Book Data product page](https://data.coindesk.com/order-book)
- [developers.coindesk.com — Spot L2 Snapshots Minute endpoint](https://developers.coindesk.com/documentation/data-api/spot_v2_historical_orderbook_l2_snapshots_minute)
- [developers.coindesk.com — Spot L2 Metrics Minute endpoint](https://developers.coindesk.com/documentation/data-api/spot_v1_historical_orderbook_l2_metrics_minute)
- [developers.coindesk.com — Legacy OrderbookL2 WebSocket channel](https://developers.coindesk.com/documentation/legacy-websockets/OrderbookL2)
- [developers.coindesk.com — Legacy L2 Snapshot REST endpoint](https://developers.coindesk.com/documentation/legacy/Orderbook/obL2SnapshotEndpoint)
- [developers.coindesk.com — Data Streamer Spot L2 Updates](https://developers.coindesk.com/documentation/data-streamer/spot_v1_orderbook_replay_l2_updates)
- [CCData Press Release — CryptoCompare Launches Real-time Order Book Feed](https://data.coindesk.com/press-releases/cryptocompare-launches-real-time-order-book-feed)
- [CoinAPI.io — CoinAPI vs CoinDesk comparison](https://www.coinapi.io/blog/best-crypto-api-alternative-to-cryptocompare)
- Existing internal research: `websocket_full.md`, `endpoints_full.md`, `api_overview.md`
