# dYdX V4 Trading API — Order & Position Specification

Sources: https://docs.dydx.xyz/ (primary, redirected from docs.dydx.exchange)

---

## 1. Order Placement Mechanism

dYdX V4 is a sovereign Cosmos SDK blockchain. **Orders are NOT placed via a REST API call** — they are Cosmos transactions broadcast directly to a validator node over **gRPC**.

### Flow

```
User wallet (mnemonic → private key)
        ↓
  Sign MsgPlaceOrder (Cosmos Tx)
        ↓
  Broadcast to validator node via gRPC
        ↓
  Validator gossips to peers, inserts into in-memory orderbook
        ↓
  Indexer reads fills → exposes via REST/WebSocket (read-only)
```

### Key Point

The **Indexer REST API** (`https://indexer.dydx.trade/v4`) is **read-only**. It cannot place, modify, or cancel orders. All mutating operations go through the **validator node gRPC endpoint**.

---

## 2. Order Types

dYdX V4 supports six primary order types plus TWAP (added in protocol v9.0):

| Order Type | Description | Execution |
|---|---|---|
| **Market** | Executes immediately at available liquidity | Submitted as IOC by default |
| **Limit** | Executes at specified price or better | Rests in orderbook if not immediately filled |
| **Stop Market** | Triggers when oracle/last-traded price crosses threshold | Executes as market order after trigger |
| **Stop Limit** | Triggers at threshold price | Rests as limit order after trigger |
| **Take Profit Market** | Closes position at target price | Executes as market order after trigger |
| **Take Profit Limit** | Closes position at target price | Executes as limit order after trigger |
| **TWAP** (v9.0+) | Time-Weighted Average Price execution | Uses `OrderFlags = 128` |

### Conditional Orders (Stop / Take Profit)

Conditional orders are long-term/stateful orders that:
- Are committed to the blockchain
- Trigger when oracle price reaches an LTE or GTE threshold
- Use `OrderFlags = 32`
- Are matched at end of block upon receipt

---

## 3. Order Duration: Short-Term vs Long-Term

This is a fundamental architectural distinction in dYdX V4:

### Short-Term Orders

- **Storage**: In-memory only (not committed to blockchain unless filled)
- **Max validity**: Current block height + `ShortBlockWindow` (default: **20 blocks ≈ 30 seconds**)
- **Time-in-force**: Good-Till-Block (GTB)
- **Supports**: IOC (Immediate-or-Cancel), FOK (Fill-or-Kill), Market orders
- **`OrderFlags` value**: `0`
- **Equity tier limits**: NOT enforced (short-term orders are exempt)
- **Replacement**: Can be replaced atomically by placing same order ID with larger `goodTilBlock`

### Long-Term / Stateful Orders

- **Storage**: Committed to blockchain state, persist across restarts
- **Max validity**: Current block time + **95 days**
- **Time-in-force**: Good-Till-Block-Time (GTBT)
- **`OrderFlags` value**: `64`
- **Equity tier limits**: ENFORCED (based on net collateral)
- **Includes**: Limit orders with long expiry, TP/SL orders

### TWAP Orders

- **`OrderFlags` value**: `128`
- Executes at current market prices over time intervals
- Added in protocol v9.0

---

## 4. Time-in-Force Options

| TIF Option | Applies To | Notes |
|---|---|---|
| **GTB** (Good-Till-Block) | Short-term orders only | Max current_block + 20 |
| **GTBT** (Good-Till-Block-Time) | Long-term / stateful orders | Max 95 days |
| **IOC** (Immediate-or-Cancel) | Short-term orders only | Cancel unfilled remainder immediately |
| **FOK** (Fill-or-Kill) | Short-term orders only | Cancel entirely if not fully filled |
| **Post-Only** | Not explicitly documented in official docs | — |

Market orders on the frontend are submitted as IOC short-term orders.

---

## 5. Order Placement — Proto Message: `MsgPlaceOrder`

Orders are broadcast as Cosmos transactions containing `MsgPlaceOrder`.

### Order ID Structure

Every order has a unique ID composed of:

```
OrderId {
    SubaccountId {
        owner:  string    // Cosmos address (e.g. "dydx1abc...")
        number: uint32    // Subaccount index (0 = cross-margin, 128+ = isolated)
    }
    client_id:   uint32   // 32-bit user-chosen identifier (must be unique per user)
    order_flags: uint32   // 0=SHORT_TERM, 32=CONDITIONAL, 64=LONG_TERM, 128=TWAP
    clob_pair_id: uint32  // Market identifier (e.g., 0=BTC-USD, 1=ETH-USD)
}
```

### `MsgPlaceOrder` Fields

```protobuf
message MsgPlaceOrder {
    Order order = 1;
}

message Order {
    OrderId    order_id          = 1;   // Unique order identifier
    Side       side              = 2;   // BUY or SELL
    uint64     quantums          = 3;   // Order size (base asset units, scaled)
    uint64     subticks          = 4;   // Order price (scaled, see market params)
    oneof good_til_oneof {
        uint32 good_til_block       = 5;   // Short-term: block height expiry
        uint32 good_til_block_time  = 6;   // Long-term: unix timestamp expiry
    }
    TimeInForce time_in_force    = 7;   // Execution condition
    uint64      reduce_only      = 8;   // 0=false, 1=true (only reduce position)
    uint32      client_metadata  = 9;   // Arbitrary metadata (ignored by protocol)
    ConditionType condition_type = 10;  // NONE, STOP_LOSS, TAKE_PROFIT
    uint64     conditional_order_trigger_subticks = 11; // Trigger price (conditional only)
}
```

### Side Enum
- `SIDE_BUY = 1`
- `SIDE_SELL = 2`

### TimeInForce Enum
- `TIME_IN_FORCE_UNSPECIFIED = 0` (standard limit / GTB/GTBT)
- `TIME_IN_FORCE_IOC = 1`
- `TIME_IN_FORCE_POST_ONLY = 2`
- `TIME_IN_FORCE_FILL_OR_KILL = 3`

### ConditionType Enum (for conditional orders)
- `CONDITION_TYPE_UNSPECIFIED = 0` (non-conditional)
- `CONDITION_TYPE_STOP_LOSS = 1` (trigger when price <= trigger_subticks for long, >= for short)
- `CONDITION_TYPE_TAKE_PROFIT = 2` (trigger when price >= trigger_subticks for long, <= for short)

---

## 6. Order Cancellation

### `MsgCancelOrder`

```protobuf
message MsgCancelOrder {
    OrderId order_id = 1;              // Same OrderId used when placing
    oneof good_til_oneof {
        uint32 good_til_block       = 2;   // Short-term cancel expiry
        uint32 good_til_block_time  = 3;   // Long-term cancel expiry
    }
}
```

- The `OrderId` must match exactly (same client_id, clob_pair_id, order_flags, subaccount)
- Cancel also expires: must be submitted before `good_til_block` / `good_til_block_time`

### Batch Cancel: `MsgBatchCancel`

The protocol Msg service includes `BatchCancel` — allows cancelling multiple orders in a single transaction. Documented in `proto/dydxprotocol/clob/tx.proto` but not fully detailed in public integration docs.

---

## 7. Querying Open Orders (Indexer — Read Only)

These are Indexer REST endpoints (no auth required):

### List Open Orders for a Subaccount

```
GET https://indexer.dydx.trade/v4/orders
```

| Parameter | Type | Required | Description |
|---|---|---|---|
| `address` | string | yes | Cosmos address |
| `subaccountNumber` | integer | yes | Subaccount index |
| `ticker` | string | no | Filter by market (e.g., "BTC-USD") |
| `side` | string | no | "BUY" or "SELL" |
| `status` | string | no | "OPEN", "FILLED", "CANCELED", "BEST_EFFORT_OPENED", "UNTRIGGERED" |
| `type` | string | no | "LIMIT", "MARKET", "STOP_LIMIT", "STOP_MARKET", "TRAILING_STOP", "TAKE_PROFIT", "TAKE_PROFIT_MARKET" |
| `limit` | integer | no | Max results (default varies) |

### Get Single Order

```
GET https://indexer.dydx.trade/v4/orders/{orderId}
```

### List Orders by Parent Subaccount

```
GET https://indexer.dydx.trade/v4/orders/parentSubaccountNumber
```
Parameters: `address`, `parentSubaccountNumber`, `ticker`, `side`, `status`

---

## 8. Position Management

### Get Open Perpetual Positions

```
GET https://indexer.dydx.trade/v4/perpetualPositions
```

| Parameter | Type | Required | Description |
|---|---|---|---|
| `address` | string | yes | Cosmos address |
| `subaccountNumber` | integer | yes | Subaccount index |
| `status` | string | no | "OPEN", "CLOSED", "LIQUIDATED" |
| `limit` | integer | no | Max results |
| `createdBeforeOrAt` | ISO8601 | no | Timestamp filter |

### Response Fields (per position)

```json
{
  "market": "BTC-USD",
  "status": "OPEN",
  "side": "LONG",
  "size": "0.01",
  "maxSize": "0.01",
  "entryPrice": "45000.00",
  "realizedPnl": "0",
  "unrealizedPnl": "150.00",
  "createdAt": "2024-01-01T00:00:00.000Z",
  "createdAtHeight": "1234567",
  "closedAt": null,
  "exitPrice": null,
  "sumOpen": "0.01",
  "sumClose": "0",
  "netFunding": "0",
  "subaccountNumber": 0
}
```

### Close a Position

dYdX has no dedicated "close position" API call. To close:
1. Place an opposing order (SELL if LONG, BUY if SHORT) with `reduce_only = true`
2. Set size equal to current position size

### Set Leverage

dYdX V4 does **not** have an explicit "set leverage" endpoint. Leverage is implicit:
- **Cross margin** (subaccount 0): leverage determined by position size vs account equity
- **Isolated margin** (subaccounts 128+): each subaccount has one market, max leverage set by market's Initial Margin Fraction (IMF)
- Maximum leverage = `1 / IMF` (varies by market)

### Funding Rate

```
GET https://indexer.dydx.trade/v4/historicalFunding/{market}
```

| Parameter | Type | Required | Description |
|---|---|---|---|
| `market` | string | yes | Market ticker e.g. "BTC-USD" |
| `limit` | integer | no | Max results |
| `effectiveBeforeOrAt` | ISO8601 | no | Timestamp filter |

Funding payments per subaccount:
```
GET https://indexer.dydx.trade/v4/fundingPayments
```
Parameters: `address`, `subaccountNumber`, `ticker`, `limit`, `afterOrAt`, `page`

### Liquidation Price

Not returned directly by API. Must be calculated client-side:
- Liquidation occurs when `Total Account Value < Total Maintenance Margin`
- `Liquidation Price ≈ EntryPrice × (1 - (EquityRatio - MMF))` for longs
- MMF per market queryable via `GET /v4/perpetualMarkets`

---

## 9. Fills / Trade History

```
GET https://indexer.dydx.trade/v4/fills
```

| Parameter | Type | Required | Description |
|---|---|---|---|
| `address` | string | yes | Cosmos address |
| `subaccountNumber` | integer | yes | Subaccount index |
| `ticker` | string | no | Market filter |
| `limit` | integer | no | Max results |
| `createdBeforeOrAt` | ISO8601 | no | Pagination timestamp |

### Response Fields (per fill)

```json
{
  "id": "fill-uuid",
  "side": "BUY",
  "liquidity": "TAKER",
  "type": "LIMIT",
  "market": "BTC-USD",
  "orderId": "order-uuid",
  "price": "45000.00",
  "size": "0.01",
  "fee": "1.35",
  "createdAt": "2024-01-01T00:00:00.000Z",
  "createdAtHeight": "1234567",
  "subaccountNumber": 0
}
```

---

## 10. Historical PnL

```
GET https://indexer.dydx.trade/v4/historical-pnl
```
Parameters: `address`, `subaccountNumber`, `createdBeforeOrAt`, `page`

---

## 11. Market Data Endpoints (Reference)

```
GET https://indexer.dydx.trade/v4/perpetualMarkets           # All markets + params
GET https://indexer.dydx.trade/v4/orderbooks/perpetualMarket/{market}
GET https://indexer.dydx.trade/v4/candles/perpetualMarkets/{market}
GET https://indexer.dydx.trade/v4/trades/perpetualMarket/{market}
GET https://indexer.dydx.trade/v4/sparklines
```

Candle resolutions: `1MIN`, `5MINS`, `15MINS`, `30MINS`, `1HOUR`, `4HOURS`, `1DAY`

---

## 12. Batch Orders

- **Batch placement**: Not supported as a single transaction — each order is a separate `MsgPlaceOrder` in the Cosmos tx, but multiple messages can be bundled in one Cosmos transaction
- **Batch cancel**: `MsgBatchCancel` exists in the protocol proto definitions; cancels multiple orders in one transaction

---

## 13. Unique dYdX V4 Features

| Feature | Description |
|---|---|
| **Fully on-chain orderbook** | Validators maintain orderbook in memory; no centralized matching |
| **No gas for trading** | Subaccounts do not consume gas for order placement |
| **Short-term order atomicity** | Replace short-term orders atomically by reusing same OrderId with higher goodTilBlock |
| **Isolated market subaccounts** | Each market can have isolated margin via dedicated subaccount (128+) |
| **Equity tier limits** | Open stateful order count is gated by account equity (TVL) |
| **TWAP orders** | Added in v9.0; time-interval execution at market prices |
| **Permissioned keys (Authenticators)** | Delegate signing to sub-keys without exposing main wallet |
| **Full-node gRPC streaming** | Real-time orderbook updates direct from full node |

---

## Sources

- [dYdX Documentation](https://docs.dydx.xyz/)
- [Orders Concept Page](https://docs.dydx.xyz/concepts/trading/orders)
- [Indexer HTTP API](https://docs.dydx.xyz/indexer-client/http)
- [Trading Interaction Guide](https://docs.dydx.xyz/interaction/trading)
- [Equity Tier Limits](https://docs.dydx.xyz/concepts/trading/limits/equity-tier-limits)
- [Rate Limits](https://docs.dydx.xyz/concepts/trading/limits/rate-limits)
- [Margin Documentation](https://docs.dydx.xyz/concepts/trading/margin)
- [v4-chain tx.proto on GitHub](https://github.com/dydxprotocol/v4-chain/blob/c092bf0166d1a111dcd9c2e4153334865c8fe553/proto/dydxprotocol/clob/tx.proto)
