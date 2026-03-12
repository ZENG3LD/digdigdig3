# Wave 2 Batch 1: Deep Verification — Binance, Bybit, OKX Trading API Gaps

**Date:** 2026-03-12
**Scope:** Order type coverage, optional traits, and out-of-architecture APIs for three major CEXs.

---

## 1. Binance

### 1.1 Order Types — Gap Analysis

Our current enum has 11 arms: Market, Limit, StopMarket, StopLimit, TrailingStop, OCO, PostOnly, IOC, FOK, ReduceOnly, Iceberg.
Missing in our enum: Bracket, TWAP, GTD.

#### OTO / Bracket (One-Triggers-Other / One-Triggers-OCO)

**CONFIRMED: Binance has native OTO and OTOCO.**

Binance introduced OTO (One-Triggers-the-Other) and OTOCO (One-Triggers-a-One-Cancels-the-Other) orders on Spot.

- **OTO**: 2-leg order list. Working order must be `LIMIT` or `LIMIT_MAKER`. Pending order placed only after working order fills. Pending order can be any type except `MARKET` with `quoteOrderQty`. Endpoint: `POST /api/v3/orderList/oto`.
- **OTOCO**: 3-leg order list. Working order (LIMIT/LIMIT_MAKER) + 2 pending orders forming an OCO pair. Endpoint: `POST /api/v3/orderList/otoco`.
- **Futures**: Conditional orders on Futures (STOP, STOP_MARKET, TAKE_PROFIT, TAKE_PROFIT_MARKET) serve a similar bracket role but are being migrated to Algo Service as of 2025-12-09.
- **Verdict**: OTO is not a "Bracket" in the traditional sense (entry + TP + SL in one). OTOCO is the closest equivalent to Bracket. **Our `Bracket` arm needs to map to OTOCO on Spot, and to algo/conditional orders on Futures.**

#### TWAP

**CONFIRMED: Binance has native TWAP via a separate Algo API.**

- **Spot TWAP**: `POST /sapi/v1/algo/spot/newOrderTwap`
- **Futures TWAP**: `POST /sapi/v1/algo/futures/newOrderTwap`
- Constraints: Duration 5 min – 24 hours. Notional: 1,000–100,000 USDT (Spot), 1,000–1,000,000 USDT (Futures).
- Max concurrent: 20 (Spot), 10 (Futures).
- Query: `GET /sapi/v1/algo/spot/openOrders`, `GET /sapi/v1/algo/spot/historicalOrders`
- **Verdict**: TWAP lives in `/sapi/v1/algo/` namespace, not in `/api/v3/order`. **Our `TWAP` arm requires a separate Algo API call path — architecturally distinct from regular orders.**

#### GTD (Good Till Date)

**CONFIRMED: Binance supports GTD as a `timeInForce` option on Futures.**

- Available on: USDS-M Futures (`timeInForce=GTD`)
- Requires: `goodTillDate` parameter (Unix ms timestamp)
- Constraints: `goodTillDate` must be > current time + 600s, and < 253402300799000ms.
- Precision: Only second-level (millisecond part ignored).
- Compatible order types: `LIMIT`, `MARKET`, `STOP`, `TAKE_PROFIT`, `STOP_MARKET`, `TAKE_PROFIT_MARKET`, `TRAILING_STOP_MARKET`
- On Spot: GTD is **NOT** listed as a timeInForce option — only `GTC`, `IOC`, `FOK` on Spot.
- **Verdict**: GTD is supported on Futures. **Our `GTD` arm is valid for Futures but maps to nothing on Spot. Implementation needs to check product context.**

#### Complete Order Type Enums (for reference)

**Binance Spot** (`/api/v3/order` type parameter):
```
LIMIT, MARKET, STOP_LOSS, STOP_LOSS_LIMIT, TAKE_PROFIT,
TAKE_PROFIT_LIMIT, LIMIT_MAKER
```

**Binance Spot TimeInForce:**
```
GTC, IOC, FOK
```

**Binance USDS-M Futures** (`/fapi/v1/order` type parameter):
```
LIMIT, MARKET, STOP, STOP_MARKET, TAKE_PROFIT,
TAKE_PROFIT_MARKET, TRAILING_STOP_MARKET
```

**Binance Futures TimeInForce:**
```
GTC, IOC, FOK, GTX (Post-Only), GTD, RPI (Retail Price Improvement)
```

**Note — Futures Migration (effective 2025-12-09):** Conditional order types `STOP`, `STOP_MARKET`, `TAKE_PROFIT`, `TAKE_PROFIT_MARKET`, `TRAILING_STOP_MARKET` are being migrated to the Algo Service endpoint (`POST /fapi/v1/order/algo`). After migration, these types will be rejected at the regular `/fapi/v1/order` endpoint.

#### Order Types We Are Missing Entirely

| Order Type | API Location | Notes |
|---|---|---|
| `LIMIT_MAKER` | `/api/v3/order` Spot | Equivalent to our `PostOnly` arm — already covered semantically |
| OTO | `/api/v3/orderList/oto` | Closest to "Bracket" trigger chain |
| OTOCO | `/api/v3/orderList/otoco` | Actual bracket (entry + OCO) |
| Scaled Orders (Futures) | Futures UI only | Not available via public API |
| Conditional Orders (Futures Algo) | `/fapi/v1/order/algo` | Post-2025-12 migration path |
| GTX / PostOnly (Futures TIF) | Futures only | Covered by `PostOnly` arm semantically |
| RPI | Futures TIF | Retail Price Improvement — rare use case |

**Conclusion on Binance order type gaps:**
- `Bracket` arm → maps to `OTOCO` on Spot, algo conditional on Futures — **implementable**
- `TWAP` arm → maps to `/sapi/v1/algo/*/newOrderTwap` — **implementable via Algo API**
- `GTD` arm → maps to Futures `timeInForce=GTD` + `goodTillDate` — **implementable on Futures only**

---

### 1.2 Optional Traits

Currently implemented: `CancelAll`, `AmendOrder`, `BatchOrders`.

#### AccountTransfers

**CONFIRMED: Binance has a comprehensive internal transfer API.**

- Endpoint: `POST /sapi/v1/asset/transfer` (Universal Transfer)
- Requires "internal transfer" permission on API key.
- Supported transfer directions (30 types):
  - `MAIN_UMFUTURE` — Spot → USDS-M Futures
  - `MAIN_CMFUTURE` — Spot → COIN-M Futures
  - `UMFUTURE_MAIN` — USDS-M Futures → Spot
  - `CMFUTURE_MAIN` — COIN-M Futures → Spot
  - `MAIN_MARGIN` — Spot → Cross Margin
  - `MARGIN_MAIN` — Cross Margin → Spot
  - `UMFUTURE_MARGIN`, `CMFUTURE_MARGIN` — Futures → Cross Margin
  - `MARGIN_UMFUTURE`, `MARGIN_CMFUTURE` — Cross Margin → Futures
  - Isolated Margin ↔ Cross Margin
  - Funding ↔ Spot / Margin / Futures
  - Options ↔ Spot / Futures / Margin / Funding
  - Portfolio Margin ↔ Spot
- Query history: `GET /sapi/v1/asset/transfer`
- **Verdict: Binance DOES support AccountTransfers. Should be added as an optional trait.**

#### CustodialFunds (Deposit/Withdraw)

**CONFIRMED: Binance has full deposit/withdraw API.**

- Deposit address: `GET /sapi/v1/capital/deposit/address`
- Deposit address list with network: `GET /sapi/v1/capital/deposit/address/list`
- Withdrawal: `POST /sapi/v1/capital/withdraw/apply`
- Withdraw history: `GET /sapi/v1/capital/withdraw/history`
- Coin config (all available networks): `GET /sapi/v1/capital/config/getall`
- **Verdict: Binance DOES support CustodialFunds. Should be added as an optional trait.**

---

### 1.3 APIs Outside Our Architecture

| Product | Base URL / Namespace | Status |
|---|---|---|
| European Options (VOPTIONS) | `https://eapi.binance.com` | Completely separate API. Order types: LIMIT only. Strike/expiry selection. Greeks data via market endpoints. |
| Portfolio Margin (PAPI) | `https://papi.binance.com` | Separate API for unified margin across Spot + USDS-M + COIN-M. Has own trade, account, and UM conditional order endpoints. |
| Margin / Cross-Margin | `/sapi/v1/margin/*` | Same host as Spot but separate namespace. Borrow/Repay: `POST /sapi/v1/margin/borrow-repay`. Includes isolated margin order placement. |
| Copy Trading | `/sapi/v1/copyTrading/*` | Separate namespace. Lead trader status, follower management. e.g. `GET /sapi/v1/copyTrading/futures/userStatus`. Published as `@binance/copy-trading` npm package. |
| Algo Trading | `/sapi/v1/algo/*` | TWAP (Spot + Futures), VP (Volume Participation), Time Ordering. Separate algo order lifecycle. |
| Binance Link (Broker API) | Separate broker portal | Sub-account management, commissions. For institutional brokers. |
| Simple Earn / Flexible Products | `/sapi/v1/simple-earn/*` | Earn/savings products — subscribe, redeem, positions. |
| Loans | `/sapi/v1/loan/*` | Crypto loans, borrow, repay, LTV management. |
| Mining Pool API | `/sapi/v1/mining/*` | For miners — hashrate, workers, revenue. |

**None of these are currently in our trait architecture.** Most are niche (Mining, Loans). AccountTransfers and CustodialFunds (deposit/withdraw) are the highest priority gaps.

---

## 2. Bybit

### 2.1 Order Types — Gap Analysis

Our current enum vs Bybit v5 API actual support.

#### What Bybit v5 Natively Supports (via `/v5/order/create`)

**Order Types (orderType parameter):**
```
Limit, Market
```

**TimeInForce:**
```
GTC, IOC, FOK, PostOnly, RPI (Retail Price Improvement — post-only matched with app/web orders)
```

**Conditional / Stop Orders:**
- Created by setting `triggerPrice` on a regular Limit or Market order
- `triggerBy`: `LastPrice`, `MarkPrice`, `IndexPrice`
- These become "conditional orders" — do not occupy margin until triggered
- TP/SL can be set inline: `takeProfit`, `stopLoss`, `tpTriggerBy`, `slTriggerBy`, `tpOrderType`, `slOrderType`

#### OCO on Bybit

**CONFIRMED: OCO exists but API users cannot place them directly.**

- Bybit OCO (One-Cancels-the-Other) orders are available via UI only.
- "API users won't have access to OCO orders, as they can design strategies to replicate similar functionality."
- **Verdict: OCO arm → `UnsupportedOperation` on Bybit, or manual simulation.**

#### Bracket Orders on Bybit

**CONFIRMED: No native bracket order type.**

- Bracket/OCO orders UI only. API users implement manually (place entry, then attach TP/SL).
- **Verdict: Bracket arm → `UnsupportedOperation` on Bybit.**

#### TWAP on Bybit

**CONFIRMED: TWAP exists but it is a UI-only strategy feature.**

- Bybit TWAP strategy: breaks order into smaller parts over time intervals.
- Available through the trading interface.
- **No public API endpoint for programmatic TWAP order placement confirmed.**
- **Verdict: TWAP arm → `UnsupportedOperation` on Bybit API.**

#### Iceberg on Bybit

**CONFIRMED: Iceberg order type exists.**

- Bybit Iceberg Orders: "split significant orders into discreet sub-orders."
- Available via UI and API. Can be placed via `/v5/order/create` with specific iceberg parameters.
- **Verdict: Iceberg arm → supported on Bybit. Needs implementation.**

#### GTD on Bybit

**NOT SUPPORTED on Bybit v5.**

- TimeInForce options: `GTC`, `IOC`, `FOK`, `PostOnly`, `RPI`.
- GTD is absent from the Bybit enum.
- **Verdict: GTD arm → `UnsupportedOperation` on Bybit.**

#### Complete Order Capabilities Summary for Bybit

| Order Type | Supported | Notes |
|---|---|---|
| Market | Yes | `orderType=Market` |
| Limit | Yes | `orderType=Limit` |
| StopMarket | Yes | Limit/Market + `triggerPrice` (conditional) |
| StopLimit | Yes | Limit + `triggerPrice` |
| TrailingStop | Yes | `trailingStop` parameter |
| OCO | No | UI only |
| Bracket | No | Manual TP/SL attachment |
| PostOnly | Yes | `timeInForce=PostOnly` |
| IOC | Yes | `timeInForce=IOC` |
| FOK | Yes | `timeInForce=FOK` |
| ReduceOnly | Yes | `reduceOnly=true` |
| Iceberg | Yes | Iceberg sub-order splitting |
| TWAP | No | UI only |
| GTD | No | Not in TimeInForce enum |

---

### 2.2 Optional Traits

Currently implemented: `CancelAll`, `AmendOrder`, `BatchOrders`.

#### AccountTransfers

**CONFIRMED: Bybit has internal transfers via v5 API.**

- Endpoint: `POST /v5/asset/transfer/inter-transfer`
- Parameters: `transferId` (UUID), `coin`, `amount`, `fromAccountType`, `toAccountType`
- Account types: `UNIFIED` (Unified Trading Account), `FUND` (Funding Account)
- Query records: `GET /v5/asset/transfer/inter-transfer-list-query`
- **Verdict: Bybit DOES support AccountTransfers. Should be added.**

#### CustodialFunds

**CONFIRMED: Bybit has deposit/withdraw API.**

- Withdraw: `POST /v5/asset/withdraw/create` (params: `coin`, `chain`, `address`, `amount`, `accountType=FUND`)
- Rate limit: 5 req/s; 1 withdrawal per 10s per coin/chain
- Sub-account deposit address: `GET /v5/asset/deposit/query-sub-member-address`
- Coin info (networks): `GET /v5/asset/coin-info`
- **Note:** Custodial sub-account deposit addresses cannot be queried via the standard endpoint.
- **Verdict: Bybit DOES support CustodialFunds. Should be added.**

---

### 2.3 APIs Outside Our Architecture

| Product | API Location | Status |
|---|---|---|
| Options Trading | `/v5/order/create` with `category=option` | **Unified in v5** — same endpoint, different category. European and American options. |
| Copy Trading | Standard `/v5/order/create` | Integrated into v5. Requires master trader status. Uses `Contract - Orders & Positions` permission. Only USDT Perpetual supported. Check `copyTrading` field on instrument. |
| Earn (On-Chain + Flexible Savings) | `/v5/earn/*` | API access recently added (announced 2025). For automated yield strategy management. Expanding to more earn types. |
| Institutional Rate Limits | Same v5 endpoints | Enhanced rate limit framework starting 2025-08-13 for institutional clients. |
| Broker / Affiliate API | Separate portal | Sub-account management for brokers. |
| Fund Custodial Sub-Accounts | `/v5/user/create-sub-api` | Specialized custodial sub-accounts for institutional use. |

**Key takeaway**: Bybit's v5 is more unified than Binance — Spot, Derivatives, and Options all use the same endpoints with `category` parameter. Copy Trading is integrated, not separate. Earn is the main new addition outside our architecture.

---

## 3. OKX

### 3.1 Order Types — Gap Analysis

OKX is currently ranked ~45/55 in our matrix. Actual OKX support is much richer.

#### Standard Order Types (`POST /api/v5/trade/order` — `ordType` parameter)

```
limit         — Standard limit order (requires px and sz)
market        — Market order (spot: sweeps order book; futures: taker)
post_only     — Post-only (maker only; cancelled if would match immediately)
fok           — Fill or Kill
ioc           — Immediate or Cancel
optimal_limit_ioc — "Optimal limit IOC" — limit order with IOC behavior at best bid/ask
```

#### Algo Order Types (`POST /api/v5/trade/order-algo` — `ordType` parameter)

```
conditional   — Trigger + stop order (supports TP/SL with trigger price)
oco           — One-Cancels-Other (TP + SL pair)
trigger       — Plain trigger order (fires when price hits level)
move_order_stop — Trailing stop order
twap          — Time-Weighted Average Price algorithm
iceberg       — Iceberg (large order split into hidden sub-orders)
```

**Note on TWAP and Iceberg**: These are documented as supported on demo trading and for certain products. Production availability may vary per instrument type.

#### Bracket Orders on OKX

- OKX `oco` ordType is a TP+SL pair (closest to bracket legs).
- A full bracket (entry + TP + SL) can be constructed via: regular `limit` entry + attached `attachAlgoOrds` (TP/SL) parameter or separate `oco` algo order after fill.
- **No single "bracket" ordType** in their API.
- **Verdict: Bracket arm → constructable via OCO algo + entry order. Maps to 2-step process.**

#### GTD on OKX

**NOT natively supported by OKX API.**

- OKX does not have a `goodTillDate` or GTD timeInForce.
- The `expTime` parameter exists for request timeout (not order expiry) — it controls when the API request itself expires, not the order.
- NautilusTrader docs explicitly state: "For GTD functionality, you must use Nautilus's strategy-managed GTD feature" which cancels orders at specified time externally.
- **Verdict: GTD arm → `UnsupportedOperation` on OKX. Must be simulated by client-side cancellation.**

#### PostOnly / ReduceOnly on OKX

- `post_only` is a native `ordType` value.
- `reduceOnly` is a separate boolean parameter on the order.
- Both supported natively.

#### OKX Algo Order Rate Limits (pending order maximums)

```
TP/SL (conditional):  100 per instrument
Trigger orders:       500 total
Trailing stop:        50 total
Iceberg:              100 total
TWAP:                 20 total
```

#### Complete Order Capabilities Summary for OKX

| Order Type | Supported | Location |
|---|---|---|
| Market | Yes | `/api/v5/trade/order`, ordType=market |
| Limit | Yes | ordType=limit |
| StopMarket | Yes | algo: ordType=conditional |
| StopLimit | Yes | algo: ordType=conditional |
| TrailingStop | Yes | algo: ordType=move_order_stop |
| OCO | Yes | algo: ordType=oco |
| Bracket | Partial | Entry + OCO = 2 calls; no single bracket ordType |
| PostOnly | Yes | ordType=post_only |
| IOC | Yes | ordType=ioc |
| FOK | Yes | ordType=fok |
| ReduceOnly | Yes | `reduceOnly=true` parameter |
| Iceberg | Yes | algo: ordType=iceberg |
| TWAP | Yes | algo: ordType=twap |
| GTD | No | Not supported; simulated externally |

---

### 3.2 Optional Traits

Currently implemented: `CancelAll`, `AmendOrder`, `BatchOrders`.

#### AccountTransfers

**CONFIRMED: OKX has full internal transfer API.**

- Endpoint: `POST /api/v5/asset/transfer`
- Sub-account transfer: `POST /api/v5/asset/subaccount/transfer`
- Query transfer state: `GET /api/v5/asset/transfer-state`
- Transfer scope: Between Trading Account (`trading`), Funding Account (`6`), and subaccounts.
- Supports all 4 account modes: Spot mode, Futures mode, Multi-currency margin, Portfolio margin.
- **Verdict: OKX DOES support AccountTransfers. Should be added.**

#### CustodialFunds

- OKX has deposit/withdraw endpoints in the Asset API (same pattern as others).
- Endpoint namespace: `/api/v5/asset/deposit-address`, `/api/v5/asset/withdrawal`
- **Verdict: OKX DOES support CustodialFunds. Should be added.**

#### SubAccounts

**CONFIRMED: OKX has a separate SubAccount trait-worthy API.**

- Master account can create and manage subaccounts.
- Transfer between subaccounts: `POST /api/v5/asset/subaccount/transfer`
- View subaccount balances, positions, trading history.
- Group RFQ: master account allocates sub-accounts to RFQ quotes.
- **This is a potential new optional trait: `SubAccountManagement`.**

---

### 3.3 APIs Outside Our Architecture

| Product | API Namespace | Notes |
|---|---|---|
| Spread Trading | `/api/v5/sprd/*` | Entire separate namespace. Place/cancel spread orders, order book, tickers. WebSocket supported. Not standard perpetual/spot. |
| Block Trading (RFQ) | `/api/v5/rfq/*` | Request-for-Quote for large OTC-style trades. Counterparty discovery, create/cancel RFQ, create/execute quotes. MMP (Market Maker Protection) reset. |
| Copy Trading | `/api/v5/copytrading/*` | Lead trader subposition management, profit sharing, instrument config. Full API namespace. |
| Earn / Finance | `/api/v5/finance/*` | Staking: `/staking-defi/`, Savings: `/savings/`, Flexible Loans: `/flexible-loan/`, ETH staking, SOL staking. Comprehensive earn ecosystem. |
| Options | `/api/v5/trade/order` with `instType=OPTION` | Options are **unified** in the main trade API — same order endpoint with `instType=OPTION`. Greeks activation: `/api/v5/account/activate-option`. Greeks data: `/api/v5/account/greeks`. Options market data: `/api/v5/public/opt-summary`. |
| Grid Trading | Algo namespace | Grid trading bots via algo order types — spot grid, contract grid. |
| Lending | `/api/v5/finance/savings/*` | Peer-to-peer crypto lending, set lending rate, lending history. |

**Key takeaway**: OKX is the most feature-rich of the three. Block Trading (RFQ), Spread Trading, and full Earn ecosystem are entirely outside our current architecture and require dedicated trait design if ever needed.

---

## 4. Cross-Exchange Comparison Summary

### Order Type Coverage

| Order Type | Binance | Bybit | OKX |
|---|---|---|---|
| Market | Spot+Futures | Yes | Yes |
| Limit | Spot+Futures | Yes | Yes |
| StopMarket | Futures | Yes (conditional) | Yes (algo) |
| StopLimit | Spot+Futures | Yes (conditional) | Yes (algo) |
| TrailingStop | Spot+Futures | Yes | Yes (algo: move_order_stop) |
| OCO | Spot (orderList) | UI only | Yes (algo: oco) |
| OTO/Bracket | Spot (OTOCO) | No | Partial (entry + oco) |
| PostOnly | Spot (LIMIT_MAKER) + Futures (GTX) | Yes (TIF) | Yes (ordType) |
| IOC | Spot+Futures | Yes | Yes |
| FOK | Spot+Futures | Yes | Yes |
| ReduceOnly | Futures | Yes | Yes |
| Iceberg | Spot (Binance.US) | Yes | Yes (algo) |
| TWAP | Algo API (/sapi/v1/algo) | UI only | Yes (algo) |
| GTD | Futures only (timeInForce=GTD) | No | No (external sim) |
| RPI | Futures TIF | Yes (TIF) | N/A |

### Optional Traits Missing (all three)

| Trait | Binance | Bybit | OKX |
|---|---|---|---|
| AccountTransfers | YES — 30+ routes via `/sapi/v1/asset/transfer` | YES — UNIFIED ↔ FUND via `/v5/asset/transfer/inter-transfer` | YES — `/api/v5/asset/transfer` |
| CustodialFunds | YES — `/sapi/v1/capital/*` | YES — `/v5/asset/withdraw/*` + deposit | YES — `/api/v5/asset/*` |
| SubAccounts | YES — Sub-account transfer, deposit, balance | YES — Fund Custodial sub-accounts | YES — Full subaccount management + group RFQ |

### Out-of-Architecture Features (outside current trait design)

| Feature | Binance | Bybit | OKX |
|---|---|---|---|
| Options | Separate API (`eapi.binance.com`) — LIMIT only | Unified in v5 (`category=option`) | Unified in v5 (`instType=OPTION`) |
| Copy Trading | Separate namespace (`/sapi/v1/copyTrading/*`) | Integrated in v5 (same order endpoint) | Separate namespace (`/api/v5/copytrading/*`) |
| Earn / Savings | `/sapi/v1/simple-earn/*` + `/sapi/v1/loan/*` | `/v5/earn/*` (recently added) | `/api/v5/finance/*` (comprehensive) |
| Algo Orders | `/sapi/v1/algo/*` (TWAP, VP) | Not exposed | `/api/v5/trade/order-algo` (full) |
| Margin Trading | `/sapi/v1/margin/*` (separate namespace) | Integrated in UTA | Integrated in multi-margin modes |
| Portfolio Margin | `papi.binance.com` (completely separate API) | Built into UTA | Built into portfolio margin mode |
| Block / RFQ | Not available | Not available | `/api/v5/rfq/*` (OKX unique) |
| Spread Trading | Not available | Not available | `/api/v5/sprd/*` (OKX unique) |

---

## 5. Actionable Recommendations

### High Priority — Add to Architecture

1. **`AccountTransfers` optional trait** — All three exchanges support this. Endpoints differ but semantics are identical (move funds between internal wallet types). Required for any strategy that needs to pre-fund specific accounts.

2. **`CustodialFunds` optional trait** — All three exchanges support deposit address query + withdrawal submission. Essential for fund management automation.

3. **`GTD` base trait arm** — Map to Binance Futures `timeInForce=GTD`, return `UnsupportedOperation` for Bybit and OKX.

4. **`TWAP` base trait arm** — Map to Binance `/sapi/v1/algo/*/newOrderTwap`, OKX `/api/v5/trade/order-algo` with `ordType=twap`. Bybit returns `UnsupportedOperation`.

5. **`Bracket` base trait arm** — Map to Binance `POST /api/v3/orderList/otoco` (Spot), algo conditional on Futures. OKX: entry order + OCO. Bybit: `UnsupportedOperation`.

### Medium Priority

6. **`SubAccountManagement` optional trait** — OKX and Bybit have full sub-account APIs. Binance has sub-account management. Useful for institutional deployments.

7. **Update `OCO` arm for Bybit** — Currently mapped, but Bybit OCO is UI-only. Must return `UnsupportedOperation` for Bybit API.

8. **Binance Futures conditional order migration** — After 2025-12-09, `STOP`, `STOP_MARKET`, `TAKE_PROFIT`, `TAKE_PROFIT_MARKET`, `TRAILING_STOP_MARKET` move to Algo Service endpoint. Our Binance Futures connector will break without this update.

### Low Priority / Out of Scope

9. **Options API** — Different product class. Would require new instrument type handling (strikes, expiries, Greeks). Should be a separate connector entirely.
10. **Copy Trading** — Pure lead-trader management; not a trading strategy trait.
11. **Block Trading / RFQ (OKX)** — Institutional OTC feature; niche use case.
12. **Spread Trading (OKX)** — Multi-leg spread instruments; requires new instrument model.
13. **Earn / Lending** — Asset management, not trading.

---

## Sources

- [Binance OTO Order Explanation](https://www.binance.com/en/support/faq/binance-oto-one-triggers-the-other-otoco-one-triggers-a-one-cancels-the-other-order-5344bac15f224ad1a692adddd8ab1d1b)
- [Binance Trading Endpoints (Spot)](https://developers.binance.com/docs/binance-spot-api-docs/rest-api/trading-endpoints)
- [Binance TWAP — Futures Algo](https://developers.binance.com/docs/algo/future-algo/Time-Weighted-Average-Price-New-Order)
- [Binance TWAP on Spot Announcement](https://www.binance.com/en/support/announcement/binance-spot-launches-twap-time-weighted-average-price-on-api-6f450412c2d1472ba30c007dd4dd1a91)
- [Binance USDS-M Futures New Order](https://developers.binance.com/docs/derivatives/usds-margined-futures/trade/rest-api)
- [Binance USDS-M Futures Common Definition (Enums)](https://developers.binance.com/docs/derivatives/usds-margined-futures/common-definition)
- [Binance Universal Transfer](https://developers.binance.com/docs/wallet/asset/user-universal-transfer)
- [Binance Deposit Address](https://developers.binance.com/docs/wallet/capital/deposite-address)
- [Binance Withdraw](https://developers.binance.com/docs/wallet/capital/withdraw)
- [Binance Options API General Info](https://developers.binance.com/docs/derivatives/options-trading/general-info)
- [Binance Copy Trading npm package](https://www.npmjs.com/package/@binance/copy-trading)
- [Bybit Place Order v5](https://bybit-exchange.github.io/docs/v5/order/create-order)
- [Bybit Types of Orders](https://www.bybit.com/en/help-center/article/Types-of-Orders-Available-on-Bybit)
- [Bybit OCO Orders](https://www.bybit.com/en/help-center/article/One-Cancels-the-Other-OCO-Orders)
- [Bybit Iceberg Order](https://www.bybit.com/en/help-center/article/Iceberg-Order)
- [Bybit TWAP Strategy](https://www.bybit.com/en/help-center/article/Introduction-to-TWAP-Strategy)
- [Bybit Create Internal Transfer](https://bybit-exchange.github.io/docs/v5/asset/transfer/create-inter-transfer)
- [Bybit Withdraw](https://bybit-exchange.github.io/docs/v5/asset/withdraw)
- [Bybit Enum Definitions v5](https://bybit-exchange.github.io/docs/v5/enum)
- [Bybit Copy Trading API](https://bybit-exchange.github.io/docs/v5/copytrade)
- [Bybit Earn API Announcement](https://www.prnewswire.co.uk/news-releases/bybit-introduces-api-access-for-on-chain-earn-and-flexible-savings-302439795.html)
- [OKX API v5 Documentation](https://www.okx.com/docs-v5/en/)
- [OKX Place Algo Order](https://www.okx.com/docs-v5/en/#order-book-trading-algo-trading-post-place-algo-order)
- [OKX TWAP Bot Guide](https://www.okx.com/en-us/help/how-do-i-use-the-twap-trading-bot)
- [OKX Iceberg Strategy](https://www.okx.com/help/xii-iceberg-strategy)
- [OKX Asset Transfer](https://www.okx.com/docs-v5/en/#funding-account-rest-api-funds-transfer)
- [OKX Copy Trading API Zone](https://www.okx.com/campaigns/copytrading-apizone)
- [python-okx consts.py (endpoint paths)](https://github.com/okxapi/python-okx/blob/master/okx/consts.py)
- [NautilusTrader OKX Integration (GTD note)](https://nautilustrader.io/docs/latest/integrations/okx/)
