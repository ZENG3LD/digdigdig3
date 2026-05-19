# MOEX — L2 Orderbook Capabilities

## Summary

MOEX (Moscow Exchange) provides L2 orderbook data through multiple tiers:

1. **ISS REST API** — Public REST polling interface. Provides snapshot orderbook with **10x10 depth** for Equities/Bonds/FX and **5x5 depth** for Futures/Options. Free with 15-minute delay; real-time requires authentication (MOEX Passport account) and a paid subscription.
2. **ALGOPACK (apim.moex.com)** — MOEX's enhanced API gateway. Same ISS endpoints but also includes `obstats` (aggregated orderbook statistics). Bearer-token auth.
3. **FAST/SIMBA Protocol** — Professional-grade binary multicast feeds over UDP. Full-depth incremental L2 + snapshots. Requires co-location or dedicated connectivity agreement with MOEX.
4. **No public WebSocket for standard L2** — WebSocket endpoint exists (`wss-api.moex.com`) but is limited to OTC Bonds and select marketplace products, not standard equities/futures L2 depth.

**Bottom line for digdigdig3**: ISS REST is viable for non-HFT use. Real-time data needs paid auth; free tier gives 15-minute delayed data. For true streaming L2, no free public option exists — must poll ISS or subscribe to FAST feeds (professional/institutional access only).

---

## ISS REST API

### Base URLs

| Gateway | URL | Notes |
|---------|-----|-------|
| Standard ISS | `https://iss.moex.com/iss/` | Legacy, always-on |
| ALGOPACK gateway | `https://apim.moex.com/iss/` | Newer, supports Bearer token |

### Orderbook Endpoints

All endpoints return JSON (append `.json`) or XML (append `.xml`).

#### 1. Orderbook for a specific instrument (default board group)
```
GET /iss/engines/{engine}/markets/{market}/securities/{secid}/orderbook.json
```
Example — Sberbank equities:
```
https://iss.moex.com/iss/engines/stock/markets/shares/securities/SBER/orderbook.json
```

#### 2. Orderbook for a specific instrument on a specific board
```
GET /iss/engines/{engine}/markets/{market}/boards/{board}/securities/{secid}/orderbook.json
```
Example — SBER on TQBR board:
```
https://apim.moex.com/iss/engines/stock/markets/shares/boards/tqbr/securities/SBER/orderbook.json
```

#### 3. All best quotes for a trading board (all instruments)
```
GET /iss/engines/{engine}/markets/{market}/boards/{board}/orderbook.json
```

#### 4. All orderbooks for a market (all instruments)
```
GET /iss/engines/{engine}/markets/{market}/orderbook.json
```
Example:
```
https://iss.moex.com/iss/engines/stock/markets/shares/orderbook.json
```

#### 5. Orderbook for instrument in a board group
```
GET /iss/engines/{engine}/markets/{market}/boardgroups/{boardgroup}/securities/{secid}/orderbook.json
```

#### 6. Column metadata (field descriptions for a market's orderbook)
```
GET /iss/engines/{engine}/markets/{market}/orderbook/columns.json
```
Example:
```
https://iss.moex.com/iss/engines/stock/markets/shares/orderbook/columns.json
```

### Engine / Market / Board Combinations

| Asset Class | engine | market | board | Notes |
|-------------|--------|--------|-------|-------|
| Equities | `stock` | `shares` | `TQBR` | Main equity board |
| Bonds | `stock` | `bonds` | `TQCB` | Corp bonds |
| FX / Currency | `currency` | `selt` | `CETS` | USD/RUB etc. |
| Futures | `futures` | `forts` | `RFUD` | FORTS derivatives |
| Options | `futures` | `options` | `ROPD` | FORTS options |

### Response Fields (orderbook block)

The `orderbook` block in the ISS response contains rows with the following columns:

| Field | Type | Description |
|-------|------|-------------|
| `SECID` | string | Instrument ticker (e.g. `"SBER"`) |
| `BOARDID` | string | Trading board code (e.g. `"TQBR"`) |
| `BUYSELL` | string | Side: `"B"` = Bid (buy), `"S"` = Ask (sell) |
| `PRICE` | float64 | Price level |
| `QUANTITY` | int32 | Volume in lots at this price level |
| `NUMCONTRACTS` | int32 | Number of contracts (futures/options only) |
| `SEQNUM` | int64 | Sequence number of the data packet |
| `UPDATETIME` | string | Timestamp of last update (HH:MM:SS) |
| `DECIMALS` | int32 | Decimal precision for price |

### JSON Response Structure Example

```json
{
  "orderbook": {
    "columns": ["SECID", "BOARDID", "BUYSELL", "PRICE", "QUANTITY", "SEQNUM", "UPDATETIME", "DECIMALS"],
    "data": [
      ["SBER", "TQBR", "B", 285.50, 1200, 12345678, "10:35:42", 2],
      ["SBER", "TQBR", "B", 285.40, 3400, 12345678, "10:35:42", 2],
      ["SBER", "TQBR", "B", 285.30, 800,  12345678, "10:35:42", 2],
      ...
      ["SBER", "TQBR", "S", 285.60, 950,  12345678, "10:35:42", 2],
      ["SBER", "TQBR", "S", 285.70, 2100, 12345678, "10:35:42", 2],
      ...
    ]
  }
}
```

Rows are sorted: Bids descending by price, Asks ascending by price.

---

## WebSocket / Streaming

### Public WebSocket Endpoint

MOEX does operate a WebSocket server at:
```
wss://wss-api.moex.com
```

However, this endpoint is **limited in scope**:
- Primarily used for the **OTC Bonds market**, **Corporate marketplace**, and **Investment marketplace** products
- Does **not** provide standard equities or FORTS L2 orderbook streaming via this WebSocket
- MOEX has noted IP address changes for this service — firewall whitelisting required

### ISS — No Native WebSocket

The standard ISS API is **poll-based REST only**. There is no official WebSocket stream for L2 orderbook data from the equities or derivatives markets through a public API.

### ALGOPACK Real-Time (REST Polling)

ALGOPACK provides "real-time" data but still via REST polling (not push streaming):
- Data freshness: **~10-second updates** for candles, trades, orderbook snapshots
- Endpoint: `https://apim.moex.com/iss/datashop/algopack/{segment}/obstats.json`
- `obstats` = aggregated orderbook metrics (price level count, spread, liquidity, bid/ask imbalance) — not raw L2 rows

**Practical polling latency**: ISS is not suitable for HFT. Typical use case is algorithmic strategies with second-scale latency tolerance.

---

## FAST/FIX Protocol (Professional Access)

### Overview

MOEX operates two professional-grade binary multicast market data platforms:

| Platform | Protocol | Markets | Latency |
|----------|----------|---------|---------|
| **FAST** | FIX/FAST v1.1 over UDP multicast | Equities (ASTS), FX, Bonds | Very low |
| **SIMBA** | FIX Simple Binary Encoding (SBE) | Equities (ASTS), FX | Ultra-low (HFT) |
| **FAST/Spectra** | FIX/FAST for derivatives | Futures, Options (SPECTRA) | Very low |

### FAST (ASTS Equities/FX/Bonds)

**Architecture**: Dual redundant UDP channels (A/B feeds). Combines incremental refresh + snapshot recovery + TCP historical replay.

**Channel types for ASTS:**
| Channel ID | Description |
|-----------|-------------|
| `OLR` | Order List feed — individual order events |
| `OBR` | OrderBook feed — aggregated book updates |
| `TLR` | Trade List feed — executed trades |
| `MSR` | Market Statistics feed |
| `ISF` | Instrument Status feed |

**Key SIMBA messages for L2 orderbook:**
| Msg ID | Name | Description |
|--------|------|-------------|
| 5 | `OrderUpdate` | Add order, change visible quantity, delete order |
| 6 | `OrderExecution` | Passive order execution (trade), contains trade data |
| 7 | `OrderBookSnapshot` | Full snapshot of active orders list |
| 8 | `SecurityDefinition` | Instrument definition/metadata |
| 1002 | `MarketDataRequest` | TCP Replay — request missed incremental messages |

**Documentation**: `ftp.moex.com/pub/SIMBA/ASTS/doc/sbe_asts_marketdata_user_guide_eng_v_1.13.pdf`

### FAST/Spectra (Derivatives)

**L2 orderbook channels by depth for futures:**
| Channel | Description |
|---------|-------------|
| `FUT-BOOK-1` | Futures orderbook, 1 level |
| `FUT-BOOK-5` | Futures orderbook, 5 levels |
| `FUT-BOOK-20` | Futures orderbook, 20 levels |
| `FUT-BOOK-50` | Futures orderbook, 50 levels |
| `OPT-BOOK-1` | Options orderbook, 1 level |
| `OPT-BOOK-5` | Options orderbook, 5 levels |
| `OPT-BOOK-20` | Options orderbook, 20 levels |
| `OPT-BOOK-50` | Options orderbook, 50 levels |

Each channel sends `J` (Empty Book) message when no orders exist for an instrument.

**Documentation**: `ftp.moex.com/pub/FAST/Spectra/prod/docs/spectra_fastgate_en.pdf`

### Incremental Update Model

FAST/SIMBA uses delta-based incremental updates:
- Operations: **Add new record**, **Change record**, **Delete record**
- Significantly reduces bandwidth vs full snapshot on every tick
- Snapshot channel available for initial state recovery
- TCP Historical Replay for gap filling (limited throughput — use only for small recovery)

**Recovery workflow:**
1. Queue incremental messages from Incremental feed
2. Receive full snapshot from Snapshot/Recovery feed
3. Apply queued incrementals on top of snapshot
4. Continue processing incremental stream

---

## Depth Levels

| Market | Protocol | Depth |
|--------|----------|-------|
| Equities (TQBR) | ISS REST | **10 bid + 10 ask** (10x10) |
| Bonds | ISS REST | **10 bid + 10 ask** (10x10) |
| Currencies/FX | ISS REST | **10 bid + 10 ask** (10x10) |
| Futures/Options | ISS REST | **5 bid + 5 ask** (5x5) |
| Futures | FAST/Spectra | Up to **50 levels** per side (configurable channel) |
| Options | FAST/Spectra | Up to **50 levels** per side (configurable channel) |
| Equities/FX | SIMBA/FAST-ASTS | Full order-by-order book (no artificial limit) |

**Full Order Book product**: MOEX offers a historical "Full Order Book" data product (all order events reconstructable) via separate purchase. See `fs.moex.com/f/3430/full-orderbook-product-description.pdf`.

---

## Update Speed

| Interface | Update Model | Latency / Frequency |
|-----------|-------------|---------------------|
| ISS REST | Snapshot on request (polling) | ~1 second minimum poll interval (practical) |
| ISS REST (real-time tier) | Snapshot on request | Data fresh within ~1s of trade event |
| ISS REST (free tier) | Snapshot on request | **15-minute delay** |
| ALGOPACK obstats | REST polling | ~10-second update cycle |
| FAST/SIMBA | Incremental push (UDP multicast) | Microsecond-to-millisecond latency |
| FAST/SIMBA snapshot | Full snapshot on demand | Used for reconnect only |

**ISS rate limits**: Not officially published, but community reports suggest ~1 request/second per IP before throttling. Heavy polling of all-market endpoints may trigger rate limits sooner.

---

## Access Tiers

### Tier 1: Free / Public (ISS REST)

- **Data**: Delayed by **15 minutes**
- **Access**: No authentication required
- **Endpoint**: `https://iss.moex.com/iss/`
- **Orderbook depth**: 10x10 (equities/FX), 5x5 (futures)
- **Use case**: Research, backtesting with delayed data, non-time-sensitive applications

### Tier 2: Real-Time ISS (Authenticated)

- **Data**: Real-time (no delay)
- **Access**: Requires **MOEX Passport account** (free registration at moex.com)
- **Authentication**: HTTP Basic auth via `https://passport.moex.com/authenticate`, session cookie, or Bearer token via `apim.moex.com`
- **Orderbook depth**: Same 10x10 / 5x5
- **Use case**: Algorithmic trading with second-scale latency tolerance

**Authentication flow (cookie-based ISS):**
```
POST https://passport.moex.com/authenticate
Headers: Authorization: Basic base64(login:password)
Response: Set-Cookie: MicexPassportCookie=...
Then include cookie on all ISS requests.
```

**Authentication flow (Bearer token via ALGOPACK):**
```
GET https://apim.moex.com/iss/...
Headers: Authorization: Bearer <APIKEY>
API key obtained from data.moex.com after subscription.
```

### Tier 3: ALGOPACK Subscription (data.moex.com)

- **Data**: Real-time, plus advanced derived metrics
- **Access**: Paid subscription, API key from data.moex.com
- **Additional data**: `obstats` (orderbook statistics: spread, imbalance, liquidity metrics), Super Candles (50+ derived features)
- **Markets**: Equities (`eq`), Futures & Options (`fo`), FX (`fx`)
- **Use case**: Quantitative strategies needing aggregated order flow signals

**ALGOPACK obstats endpoint:**
```
GET https://apim.moex.com/iss/datashop/algopack/eq/obstats.json?date=2024-10-15
Headers: Authorization: Bearer <APIKEY>
```

### Tier 4: FAST/SIMBA Professional Feed

- **Data**: Real-time, full-depth, ultra-low latency
- **Access**: Contract with MOEX, co-location or dedicated VPN/leased line
- **Depth**: Unlimited (order-by-order) on SIMBA; up to 50 levels per side on FAST/Spectra
- **Protocol**: UDP multicast binary (FAST v1.1 or SBE)
- **Pricing**: Institutional pricing (contact MOEX market data team)
- **Use case**: HFT, market making, direct arbitrage

### Commercial Data Products (Order Book)

From `moex.com/en/orders`:

| Product | Contents | Price (1 market) | Price (1 instrument) |
|---------|----------|-----------------|---------------------|
| **Type A** (Full OB) | All trades + all orders | $500/month or $5,000/year | $150/month or $1,500/year |
| **Type B** (Top of Book) | All trades + best orders only | $150/month or $1,500/year | $50/month or $500/year |

---

## Raw Examples

### ISS REST — Orderbook Request (curl)

**Free (delayed) — no auth needed:**
```bash
curl "https://iss.moex.com/iss/engines/stock/markets/shares/securities/SBER/orderbook.json"
```

**Real-time — cookie auth:**
```bash
# Step 1: authenticate
curl -c cookies.txt -X POST \
  -H "Authorization: Basic $(echo -n 'user@email.com:password' | base64)" \
  "https://passport.moex.com/authenticate"

# Step 2: fetch orderbook with cookie
curl -b cookies.txt \
  "https://iss.moex.com/iss/engines/stock/markets/shares/securities/SBER/orderbook.json"
```

**Real-time — Bearer token via ALGOPACK:**
```bash
curl -H "Authorization: Bearer YOUR_API_KEY" \
  "https://apim.moex.com/iss/engines/stock/markets/shares/boards/tqbr/securities/SBER/orderbook.json"
```

### ISS REST — Column Metadata Request
```bash
curl "https://iss.moex.com/iss/engines/stock/markets/shares/orderbook/columns.json"
curl "https://iss.moex.com/iss/engines/futures/markets/forts/orderbook/columns.json"
```

### Futures Orderbook (FORTS)
```bash
curl -H "Authorization: Bearer YOUR_API_KEY" \
  "https://apim.moex.com/iss/engines/futures/markets/forts/boards/rfud/securities/SRH4/orderbook.json"
```

### ALGOPACK — OBStats (aggregated orderbook metrics)
```bash
curl -H "Authorization: Bearer YOUR_API_KEY" \
  "https://apim.moex.com/iss/datashop/algopack/eq/obstats.json?date=2024-10-15"
```

### Typical ISS Response Shape

```json
{
  "orderbook": {
    "metadata": {
      "SECID":      {"type": "string", "bytes": 36},
      "BOARDID":    {"type": "string", "bytes": 12},
      "BUYSELL":    {"type": "string", "bytes": 1},
      "PRICE":      {"type": "double"},
      "QUANTITY":   {"type": "int32"},
      "SEQNUM":     {"type": "int64"},
      "UPDATETIME": {"type": "time"},
      "DECIMALS":   {"type": "int32"}
    },
    "columns": ["SECID","BOARDID","BUYSELL","PRICE","QUANTITY","SEQNUM","UPDATETIME","DECIMALS"],
    "data": [
      ["SBER","TQBR","B",285.50,1200,123456,"10:35:42",2],
      ["SBER","TQBR","B",285.40,3400,123456,"10:35:42",2],
      ["SBER","TQBR","B",285.30, 800,123456,"10:35:42",2],
      ["SBER","TQBR","B",285.20,1100,123456,"10:35:42",2],
      ["SBER","TQBR","B",285.10, 500,123456,"10:35:42",2],
      ["SBER","TQBR","B",285.00, 700,123456,"10:35:42",2],
      ["SBER","TQBR","B",284.90,2200,123456,"10:35:42",2],
      ["SBER","TQBR","B",284.80, 350,123456,"10:35:42",2],
      ["SBER","TQBR","B",284.70, 600,123456,"10:35:42",2],
      ["SBER","TQBR","B",284.60, 900,123456,"10:35:42",2],
      ["SBER","TQBR","S",285.60, 950,123456,"10:35:42",2],
      ["SBER","TQBR","S",285.70,2100,123456,"10:35:42",2],
      ["SBER","TQBR","S",285.80,1300,123456,"10:35:42",2],
      ["SBER","TQBR","S",285.90, 400,123456,"10:35:42",2],
      ["SBER","TQBR","S",286.00,1750,123456,"10:35:42",2],
      ["SBER","TQBR","S",286.10, 620,123456,"10:35:42",2],
      ["SBER","TQBR","S",286.20, 800,123456,"10:35:42",2],
      ["SBER","TQBR","S",286.30, 200,123456,"10:35:42",2],
      ["SBER","TQBR","S",286.40,1100,123456,"10:35:42",2],
      ["SBER","TQBR","S",286.50, 500,123456,"10:35:42",2]
    ]
  }
}
```

---

## Sources

- [ISS API Reference — iss.moex.com/iss/reference/](https://iss.moex.com/iss/reference/)
- [MOEX Programming Interface for ISS](https://www.moex.com/a2920)
- [MOEX Interfaces Overview (ISS, FAST, SIMBA, FIX)](https://www.moex.com/a7939)
- [MOEX Order Book Data Products and Pricing](https://www.moex.com/en/orders)
- [MOEX FAST Service Description](https://www.moex.com/a1527)
- [MOEX FAST Protocol Spec v1.29.3 (Spectra, 2026)](https://ftp.moex.com/pub/FAST/Spectra/prod/docs/spectra_fastgate_en.pdf)
- [MOEX SIMBA SBE ASTS Market Data User Guide v1.13](https://ftp.moex.com/pub/SIMBA/ASTS/doc/sbe_asts_marketdata_user_guide_eng_v_1.13.pdf)
- [MOEX Market Data Multicast FIX/FAST User Guide v4.0.1](https://ftp.moex.com/pub/FAST/ASTS/docs/Archive/ENG_Market_Data_Multicast_User_Guide_Ver_4_0_1.pdf)
- [MOEX Full Order Book Product Description](https://fs.moex.com/f/3430/full-orderbook-product-description.pdf)
- [ALGOPACK Documentation](https://moexalgo.github.io/)
- [ALGOPACK Python Library Docs (moexalgo)](https://moexalgo.readthedocs.io/ru/latest/candels.html)
- [ALGOPACK Datashop](https://data.moex.com/products/algopack)
- [go-moex-iss Library (Go)](https://pkg.go.dev/github.com/Ruvad39/go-moex-iss)
- [B2BITS MOEX FIX/FAST Market Data Adaptor](https://www.b2bits.com/trading_solutions/market-data-solutions/moex-fixfast)
