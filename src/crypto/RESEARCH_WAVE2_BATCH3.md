# Research Wave 2 Batch 3: Deep API Gap Verification
## Deribit, Bitfinex, Bitget, BingX, Phemex, Crypto.com, MEXC, HTX

**Date**: 2026-03-12
**Scope**: Optional trait gaps, base trait order type coverage, features outside our current architecture

---

## Section 1: Optional Trait Gaps — BatchOrders

### 1.1 Bitfinex — BatchOrders

**Verdict: YES, fully supported.**

Endpoint: `POST https://api.bitfinex.com/v2/auth/w/order/multi`

This endpoint is explicitly designed for multi-operation batching. It accepts an `ops` array where each element is an operation:

| Operation Code | Description |
|----------------|-------------|
| `"on"` | Submit new order |
| `"oc"` | Cancel single order by ID |
| `"oc_multi"` | Cancel multiple orders at once |
| `"ou"` | Update/amend existing order |

- Max 75 operations per request
- Response includes per-operation SUCCESS/FAILURE status
- Can mix creates, cancels, and amends in a single call

**Our connector is missing this.** We should implement `BatchOrders` for Bitfinex.

---

### 1.2 BingX — BatchOrders

**Verdict: YES, supported (Perpetual Futures).**

Endpoint: `POST /openApi/swap/v2/trade/batchOrders`
(also surfaced as "Place multiple orders" in their API docs)

- Supports placing multiple orders in a single request
- The `cancel_all_orders` endpoint also exists: `DELETE /openApi/swap/v2/trade/allOpenOrders`

Order types supported in perpetual futures:
- `LIMIT`
- `MARKET`
- `TRIGGER_MARKET` (stop)
- `TRAILING_STOP_MARKET`
- `POST_ONLY`

**Our connector is missing BatchOrders.** We should implement it for BingX perpetuals.

---

### 1.3 Crypto.com — BatchOrders

**Verdict: YES, supported via separate endpoint.**

Endpoint: `private/create-order-list`
Batch cancel: `private/cancel-order-list`

- Added 2023-08-11
- Part of the institutional API
- Works with LIST type orders
- Note: as of 2024, STOP_LOSS, STOP_LIMIT, TAKE_PROFIT, TAKE_PROFIT_LIMIT were removed from the standard `type` field in both `private/create-order` and `private/create-order-list`; conditional orders migrated to `private/advanced/create-order`

**Our connector is missing BatchOrders.** Implementable.

---

### 1.4 Deribit — BatchOrders

**Verdict: NO dedicated batch-place endpoint.**

Deribit does NOT have a `private/batch_order` or equivalent. Individual orders must be submitted one by one via `private/buy` and `private/sell`.

However, Deribit has specialized mass operations:

| Endpoint | Purpose |
|----------|---------|
| `private/mass_quote` | Submit bid/ask pairs on multiple instruments simultaneously (up to 100 quotes per call) — market maker oriented |
| `private/cancel_all` | Cancel all open orders across all currencies |
| `private/cancel_all_by_currency` | Cancel by currency (BTC, ETH, etc.) |
| `private/cancel_all_by_instrument` | Cancel for specific instrument |
| `private/cancel_all_by_kind_or_type` | Cancel by instrument kind (option/future/spot) |
| `private/cancel_all_by_currency_pair` | Cancel by currency pair |
| `private/cancel_by_label` | Cancel all orders sharing a label |

`private/mass_quote` is NOT a general batch order placer — it is specifically for market makers submitting simultaneous bid/ask quotes on multiple option instruments. All quotes in one call must share the same underlying index.

**Conclusion**: Deribit intentionally has no general BatchOrders. The `mass_quote` endpoint is a specialized market-making tool, not a retail batch placement API. Our current `CancelAll` coverage is accurate. BatchOrders: genuinely unsupported for standard trading.

---

### 1.5 Phemex — BatchOrders

**Verdict: NO batch order PLACEMENT, but has bulk cancel.**

Phemex does NOT have a batch place-orders endpoint for regular limit/market orders.

Available bulk operations:
- `DELETE /orders?symbol=<s>&orderID=<id1>,<id2>,...` — bulk cancel by ID list
- `DELETE /orders/all?symbol=<s>` — cancel all orders for symbol
- `PUT /orders/replace` — amend a single order (but not batch amend)

Special cases (not standard batch):
- **Scaled orders** are internally batched (system splits them), but the API call is a single order
- **Basket orders** — multi-pair orders, available but restricted to approved partners
- **RPI orders** — batch placement supported but only for approved market-making partners

**Conclusion**: Standard BatchOrders genuinely unsupported for Phemex retail API. Our current implementation is correct in not having it.

---

### 1.6 MEXC — CancelAll and AmendOrder

We have BatchOrders but no CancelAll or AmendOrder.

**CancelAll:**
- `POST /api/v1/private/order/cancel_all` — explicitly documented
- Optional `symbol` parameter to target specific contract
- **YES, CancelAll exists. We should implement it.**

**AmendOrder:**
- **NO amend/modify order endpoint exists in MEXC contract API.**
- Users must cancel and resubmit.
- Confirmed: no modify endpoint documented.

---

### 1.7 HTX — CancelAll and AmendOrder

We have BatchOrders but no CancelAll or AmendOrder.

**CancelAll:**
- `POST /v1/order/orders/batchCancelOpenOrders` — cancel open orders by criteria (symbol, account, side filters)
- `POST /v2/order/cancel-all-after` — "Dead Man's Switch" (timed auto-cancel, max 500 orders)
- **YES, CancelAll effectively exists via batchCancelOpenOrders. We should implement it.**

**AmendOrder:**
- **NO amend/modify order endpoint in HTX spot API.**
- HTX Futures (coin-margined swaps) added modify interface in version 1.2.3, but HTX spot has no equivalent.
- Confirmed: cancel + resubmit is the only way for spot orders.

---

## Section 2: Base Trait Gaps — Order Type Coverage

### 2.1 Deribit — 11/14 Order Types

**What Deribit supports (confirmed from `private/buy` docs):**

| Order Type | Deribit Name | Supported |
|------------|-------------|-----------|
| Limit | `limit` | YES |
| Market | `market` | YES |
| StopLimit | `stop_limit` | YES |
| StopMarket | `stop_market` | YES |
| TakeProfit (Limit) | `take_limit` | YES |
| TakeProfit (Market) | `take_market` | YES |
| MarketLimit | `market_limit` | YES (market order that becomes limit if no immediate fill) |
| TrailingStop | `trailing_stop` | YES |
| OTO (One Triggers Other) | `linked_order_type: one_triggers_other` | YES (via linked_order_type param) |
| OCO (One Cancels Other) | `linked_order_type: one_cancels_other` | YES (via linked_order_type param) |
| OTOCO | `linked_order_type: one_triggers_one_cancels_other` | YES (via otoco_config array param) |

**What is missing from our 3 gap slots:**

- **Iceberg**: YES, Deribit DOES support iceberg. The `display_amount` and `refresh_amount` parameters on `private/buy` enable iceberg behavior. This is a real order feature we are missing.
- **TWAP**: NO, Deribit does not have native TWAP orders.
- **Bracket**: Effectively covered by OTOCO (`linked_order_type: one_triggers_one_cancels_other`).

**Time-in-Force options:**
- `good_til_cancelled` (GTC) — default
- `good_til_day` (GTD) — auto-cancels at end of current day (8 AM UTC). YES, GTD is supported.
- `fill_or_kill` (FOK)
- `immediate_or_cancel` (IOC)

**Options-specific advanced parameter:**
- `advanced: "usd"` — price the option in USD (engine continuously updates to maintain USD value)
- `advanced: "implv"` — price the option by implied volatility percentage (e.g. price=100 means 100% IV)

**Corrections:**
- Our 11/14 needs rechecking. Deribit actually has: Limit, Market, StopLimit, StopMarket, TakeLimit, TakeMarket, MarketLimit, TrailingStop, OCO, OTO, OTOCO = 11 named types, plus Iceberg (as param flag).
- GTD is supported.
- TWAP is not natively supported.

---

### 2.2 Bitfinex — 10/14 Order Types

**Confirmed order types from REST API:**

| Type String | Description |
|-------------|-------------|
| `LIMIT` / `EXCHANGE LIMIT` | Standard limit |
| `MARKET` / `EXCHANGE MARKET` | Standard market |
| `STOP` / `EXCHANGE STOP` | Stop-market |
| `STOP LIMIT` / `EXCHANGE STOP LIMIT` | Stop-limit |
| `TRAILING STOP` / `EXCHANGE TRAILING STOP` | Trailing stop |
| `FOK` / `EXCHANGE FOK` | Fill or Kill |
| `IOC` / `EXCHANGE IOC` | Immediate or Cancel |

Note: `EXCHANGE` prefix = margin/exchange account, no prefix = margin trading (funding-based).

**Order flags (bitmask):**
- Flag `64` = Hidden order (order not visible in order book)
- Flag `4096` = Post Only
- Flag combination `4160` = Hidden + Post Only

**Algo orders via Honey Framework:**
- **TWAP** — available, but requires the bfx-hf-algo JavaScript framework, NOT native REST API
- **Iceberg** — available via Honey Framework with "excess as hidden" option, NOT native `max_show` param in REST
- **Accumulate/Distribute** — only via Honey Framework
- **Ping/Pong** — only via Honey Framework

**Important**: The Honey Framework repo (`bfx-hf-ui`) was marked "no longer maintained as of March 25, 2025". TWAP and Iceberg are effectively deprecated/framework-only.

**OCO**: Available via the `flags` mechanism in `POST /v2/auth/w/order/multi` (oc_multi operation links orders). Also via UI but no clean standalone OCO endpoint.

**What the 4 missing types are:**
1. **TWAP** — framework only, deprecated
2. **Iceberg** — framework only (hidden flag + manual slicing), not native
3. **OCO** — technically doable via order/multi but not a clean first-class type
4. **Bracket/OTOCO** — not supported

---

### 2.3 Bitget — 11/14 Order Types

**Confirmed supported:**

| Order Type | API Parameter | Notes |
|------------|--------------|-------|
| Limit | `orderType: limit` | Standard |
| Market | `orderType: market` | Standard |
| Stop Market | via `place-plan-order` | Trigger order |
| Stop Limit | via `place-plan-order` | Trigger order |
| Take Profit (Market) | via `place-tpsl-order` | TPSL endpoint |
| Take Profit (Limit) | via `place-tpsl-order` | TPSL endpoint |
| Trailing Stop | `orderType: trailing_stop` | Available in perpetuals |
| OCO | Spot OCO available | `POST /api/v2/spot/trade/place-oco-order` |
| TWAP | Available | Up to 30 active TWAP orders per account simultaneously |
| Iceberg | Available | Listed in futures order types |
| Batch Order | `POST /api/v2/mix/order/batch-place-order` | Up to 20 orders per batch |

**Also confirmed:**
- `POST /api/v2/mix/order/cancel-all-orders` — CancelAll for futures (confirmed)
- `POST /api/v2/mix/order/modify-order` — AmendOrder for futures (confirmed)

**What 3 are missing:**
1. **Bracket/OTOCO** — not confirmed as native endpoint type
2. **OCO for futures** — OCO is spot-only currently
3. **Some combination** — TBD

Note: TWAP and Iceberg are available but as separate algorithmic order flows, not simple `orderType` values in the standard endpoint.

---

### 2.4 BingX — 9/14 Order Types

**Confirmed supported (Perpetual Futures Swap V2):**

| Order Type | API Value | Notes |
|------------|-----------|-------|
| Limit | `LIMIT` | Standard |
| Market | `MARKET` | Standard |
| Stop Market | `STOP_MARKET` | Trigger at price |
| Stop Limit | `STOP` / `STOP_LIMIT` | Trigger with limit |
| Take Profit Market | `TAKE_PROFIT_MARKET` | TP trigger |
| Take Profit Limit | `TAKE_PROFIT` | TP with limit |
| Trailing Stop | `TRAILING_STOP_MARKET` | With `activationPrice`, `priceRate`, `price` fields |
| Post Only | `POST_ONLY` | Maker-only |

**BatchOrders**: Exists — `POST /openApi/swap/v2/trade/batchOrders`
**CancelAll**: Exists — `DELETE /openApi/swap/v2/trade/allOpenOrders`

**What is missing (5 gaps):**
1. **OCO** — not confirmed as available in BingX perpetuals
2. **Bracket/OTOCO** — not available
3. **TWAP** — not a native order type
4. **Iceberg** — not a native order type
5. **MarketIfTouched / LimitIfTouched** — not confirmed

---

## Section 3: Features Outside Our Current Architecture

### 3.1 Deribit — Full Feature Surface

#### Options Trading (PRIMARY product)

Deribit is THE crypto options exchange. ~80% of volume is options.

**How options work via API:**
- Options use the SAME `private/buy` and `private/sell` endpoints as futures
- The `instrument_name` field determines what you're trading
- Options instrument naming format: `{CURRENCY}-{EXPIRY}-{STRIKE}-{CALL/PUT}`
  - Example: `BTC-26JAN24-45000-C` (BTC Call, expires 26 Jan 2024, $45,000 strike)
  - Example: `ETH-22FEB24-2800-P` (ETH Put, expires 22 Feb 2024, $2,800 strike)
- Available currencies: BTC, ETH, SOL, and others
- Expiry types: weekly, monthly, quarterly

**Options-specific API features:**
- `advanced: "implv"` — price by implied volatility (%)
- `advanced: "usd"` — price by USD value (engine auto-updates to maintain peg)
- `implv` and `usd` fields in order state
- IV orders appear in order book but price updates continuously
- "Inverse Options" also available (settled in base currency, not USD)

**Options market data:**
- `public/get_instruments?currency=BTC&kind=option` — list all options
- `public/get_book_summary_by_currency?currency=BTC&kind=option` — options book summary
- Dedicated options Greeks (delta, gamma, vega, theta) in market data

**Our architecture has ZERO options support.** To properly support Deribit, we would need:
1. An `OptionsTrading` trait or similar (place options order, get options chain, get Greeks)
2. OR simply ensure our `place_order` handles the `instrument_name` format for options and passes `advanced` params

Minimum viable: options can be traded via `private/buy` with the right `instrument_name` — so if our connector passes through arbitrary instrument names, options "just work" at the API level.

#### Block Trading (Institutional)

A separate fully-negotiated trade flow:
1. `private/create_block_rfq` — taker requests quotes
2. `private/add_block_rfq_quote` — maker provides quotes
3. `private/accept_block_rfq` — taker accepts
4. `private/verify_block_trade` → `private/execute_block_trade` — two-step execution

Minimum block trade sizes apply (larger than exchange minimums).

#### Combo / Spread Orders

- `private/create_combo` — define a multi-leg combo instrument
- Trade on combo books with the same `private/buy`/`private/sell`
- Combo instruments: `future_combo`, `option_combo` kinds
- Examples: call spreads, put spreads, straddles, strangles — all as single tradeable instruments

#### Mass Quoting (Market Makers)

- `private/mass_quote` — up to 100 bid/ask pairs simultaneously
- Must be linked to an MMP group
- Requires manual activation by Deribit staff

#### Market Maker Protection (MMP)

- `private/set_mmp_config` — configure MMP limits
- `private/get_mmp_status` — check frozen state
- `private/reset_mmp` — resume after trigger

#### Portfolio Margin

- Deribit has portfolio margin (PM) mode
- PM considers entire portfolio holistically for margin calculation
- PM is better for hedged portfolios (spreads, futures+options combos)
- No separate API endpoints needed — it's an account mode, same trading API works

#### NOT supported by Deribit:
- Copy trading — not a feature
- Earn/Lending — not a feature (derivatives exchange only)
- Staking — not available

---

### 3.2 Bitfinex — Full Feature Surface

#### Algorithmic Orders (Honey Framework)
As noted above: TWAP, Accumulate/Distribute, Iceberg, Ping/Pong — available via bfx-hf-algo JavaScript framework only. The framework repo was deprecated March 2025.

#### Lending / Funding
Bitfinex has a peer-to-peer lending market (unique feature):
- Users can offer/request margin funding
- Endpoints: `POST /v2/auth/w/funding/offer/submit`, `GET /v2/auth/r/funding/offers/{symbol}`, etc.
- This is a significant unique feature of Bitfinex
- Our architecture has NO funding/lending trait

#### Derivatives (Bitfinex Derivatives)
- Bitfinex has perpetual swap contracts with different symbol prefix (`tBTCF0:USTF0`)
- Same order API, different instrument namespace

#### Paper Trading
- Bitfinex has a dedicated paper trading environment (separate credentials)

---

### 3.3 Bitget — Full Feature Surface

#### Copy Trading
- Bitget is one of the leading copy trading platforms
- There is a Copy Trading API for social traders
- NOT relevant to our current architecture

#### TWAP Orders (Native)
- `POST /api/v2/mix/order/place-twap-order` — dedicated TWAP endpoint
- Parameters: `symbol`, `productType`, `side`, `tradeSide`, `size`, `priceType`, `executePrice`, `executeQuantity`, `timeInterval`, `totalQuantity`
- Up to 30 active TWAP orders per account
- This is a genuine architecture gap — we have no TWAP trait

#### Iceberg Orders
- Available in Bitget UI and presumably API
- Splits large orders into visible + hidden portions

#### Earn/Savings Products
- Separate API for earn products — not exchange trading

#### TP/SL as Order Parameters
- `presetStopSurplusPrice` and `presetStopLossPrice` as fields on `place-order`
- These are "preset" TP/SL attached to the entry order
- Useful for bracket-like behavior without a separate OCO endpoint

---

### 3.4 BingX — Full Feature Surface

#### Copy Trading
- BingX has copy trading support
- Separate API not relevant to our architecture

#### TP/SL as Parameters
- TP/SL can be set directly on order placement via `stopPrice`, `profitPrice`, `profitStopPrice`
- "Trailing TP/SL" also available as a separate mode

#### No Options, No Block Trades
- BingX is a straightforward perpetuals/spot exchange
- No options, no block RFQ, no mass quoting, no combo instruments

---

### 3.5 Phemex — Full Feature Surface

#### Order Types Summary (Full)

| Order Type | API Name | Notes |
|------------|----------|-------|
| Limit | `Limit` | Standard |
| Market | `Market` | Standard |
| Stop Market | `Stop` | Triggered |
| Stop Limit | `StopLimit` | Triggered |
| Take Profit Market | `MarketIfTouched` | MIT |
| Take Profit Limit | `LimitIfTouched` | MIT limit |
| Trailing Stop | Available | With offset |
| Bracket | `Bracket` (type 11) | Entry + TP + SL in one |
| Bracket TP Limit | `BoTpLimit` (type 12) | TP leg of bracket |
| Bracket SL Limit | `BoSlLimit` (type 13) | SL limit leg |
| Bracket SL Market | `BoSlMarket` (type 14) | SL market leg |
| Scaled Order | UI/system-level | Splits into multiple limits |
| Post Only | `ProtectedMarket`, etc. | |

**Important**: Phemex's Bracket order type is a first-class API type! This is a genuine gap — we likely have this as `UnsupportedOperation` when we should be implementing it.

Also confirmed:
- `DELETE /orders/all` — CancelAll by symbol (we have this)
- `PUT /orders/replace` — AmendOrder (we have this)
- **No batch place endpoint** — confirmed, not available for retail API

#### Scaled Orders
- Available via UI
- API behavior: user places one "scaled order" and system splits it into N limit orders internally
- Not directly exposed as a batch placement via API

#### Earn/Lending/Copy Trading
- Phemex has an earn product and copy trading
- Separate API sections, not relevant to trading traits

---

### 3.6 Crypto.com — Full Feature Surface

#### Current Order Types (post-2026 migration)

Standard `private/create-order`:
- `LIMIT`
- `MARKET`

Advanced Order Management (migrated as of 2026-01-28):
- `private/advanced/create-order` — handles stop-loss, take-profit, stop-limit
- `private/advanced/create-oco` — One-Cancels-Other (currently Spot only)
- `private/advanced/create-oto` — One-Triggers-Other
- `private/advanced/create-otoco` — One-Triggers-One-Cancels-Other

**Trailing Stop**: Confirmed NOT available as native API order type. Available in UI only.
**TWAP**: Not available.
**Iceberg**: Not available.

**BatchOrders**: `private/create-order-list` — available (confirmed added 2023-08-11)

#### Earn / DeFi Products
- Crypto.com has extensive earn/savings products
- Completely separate API, not trading-relevant

#### Copy Trading
- Not a notable feature on Crypto.com Exchange (vs Crypto.com App)

---

## Section 4: Deribit Options Deep Dive

### Why Our Architecture Has a Fundamental Gap

Deribit is not just "another futures exchange" — it is the world's largest crypto options exchange. Options are its primary product. A Deribit connector that only handles futures/perpetuals covers maybe 20% of the platform's real usage.

### Options Are "Just Instruments" — But Not Really

At the API level, yes: options use the same `private/buy` and `private/sell` as everything else. You can trade a BTC call option with:

```json
{
  "method": "private/buy",
  "params": {
    "instrument_name": "BTC-28MAR25-100000-C",
    "amount": 1.0,
    "type": "limit",
    "price": 0.05
  }
}
```

This works. BUT to properly support options trading, you need:

1. **Options chain enumeration** — `public/get_instruments?currency=BTC&kind=option` returns hundreds of instruments (all strikes × all expiries × call/put)
2. **Options pricing modes** — `advanced: "implv"` and `advanced: "usd"` are options-only parameters. Pricing by IV is the standard way professional options traders work.
3. **Greeks data** — Delta, gamma, vega, theta are available in market data for options. Completely different from futures.
4. **Settlement understanding** — options expire and settle, futures rollover. Different lifecycle.
5. **Inverse options** — settled in BTC rather than USD, a different financial product entirely.

### Options-Specific Endpoints We Lack

| Need | Endpoint |
|------|---------|
| List all option instruments | `public/get_instruments?kind=option` |
| Options book summary | `public/get_book_summary_by_currency?kind=option` |
| Get ticker with Greeks | `public/get_ticker?instrument_name=BTC-28MAR25-100000-C` |
| Trade options (with IV pricing) | `private/buy` with `advanced=implv` |
| Options settlement history | `private/get_settlement_history_by_currency?type=delivery` |
| User's options portfolio | `private/get_positions?kind=option` |

### Recommended Architecture Decision

**Option A (Minimal)**: Options trading via `place_order` with opaque `instrument_name` and extra `params: HashMap<String,Value>`. Works but no type safety.

**Option B (Proper)**: Add `OptionsTrading` optional trait with:
- `get_options_chain(currency, expiry)` → `Vec<OptionInstrument>`
- `place_option_order(instrument, amount, advanced_price)` (IV or USD pricing)
- `get_options_greeks(instrument)` → Greeks struct
- `get_options_settlement_history(currency)` → settlements

**Option C (Interim)**: Document that Deribit options work via `place_order` if you pass the correct `instrument_name`, but add a `DeribitExtensions` trait for IV pricing, mass quoting, and block RFQ.

---

## Section 5: Consolidated Gap Table

### Optional Traits

| Exchange | BatchOrders | CancelAll | AmendOrder | Notes |
|----------|-------------|-----------|------------|-------|
| Bitfinex | MISSING — should add | Has via `order/multi` | Has via `order/multi (ou)` | All in one endpoint |
| BingX | MISSING — should add | Has | No amend | `/openApi/swap/v2/trade/batchOrders` |
| Crypto.com | MISSING — should add | Partial (no clean all) | No amend | `private/create-order-list` |
| Deribit | N/A (no batch) | Has (extensive) | Has (`private/edit`) | mass_quote ≠ batch orders |
| Phemex | N/A (confirmed no batch) | Has (`DELETE /orders/all`) | Has (`PUT /orders/replace`) | Current impl correct |
| MEXC | Has (batch submit) | MISSING — should add | N/A (no modify) | `cancel_all` endpoint confirmed |
| HTX | Has (batch orders) | MISSING — should add | N/A (no modify spot) | `batchCancelOpenOrders` |
| Bitget | Has (batch contract) | Has (confirmed) | Has (confirmed) | Full suite available |

### Base Trait Order Types

| Order Type | Deribit | Bitfinex | Bitget | BingX | Phemex | Crypto.com |
|------------|---------|----------|--------|-------|--------|------------|
| Limit | YES | YES | YES | YES | YES | YES |
| Market | YES | YES | YES | YES | YES | YES |
| StopMarket | YES | YES | YES | YES | YES | via Advanced |
| StopLimit | YES | YES | YES | YES | YES | via Advanced |
| TakeProfit (Market) | YES (take_market) | partial | YES | YES | YES (MIT) | via Advanced |
| TakeProfit (Limit) | YES (take_limit) | partial | YES | YES | YES (LIT) | via Advanced |
| TrailingStop | YES | YES | YES | YES | YES | NO (UI only) |
| Iceberg | YES (display_amount param) | Framework-only (deprecated) | YES | NO | NO (UI only) | NO |
| OCO | YES (linked_order_type) | Partial (order/multi) | Spot only | NO | NO | YES (Spot, via Advanced) |
| OTO | YES | NO | NO | NO | NO | YES (via Advanced) |
| OTOCO/Bracket | YES (otoco_config) | NO | NO | NO | YES (native type) | YES (via Advanced) |
| GTD | YES (good_til_day) | YES (time_in_force param) | Partial | NO | NO | Partial |
| TWAP | NO | Framework-only (deprecated) | YES (separate endpoint) | NO | NO | NO |
| MarketLimit | YES (market_limit) | NO | NO | NO | NO | NO |

### Features Outside Current Architecture

| Feature | Exchange | Available via API? | Priority |
|---------|---------|-------------------|----------|
| Options Trading | Deribit | YES (same buy/sell, options instrument_name) | HIGH |
| Options IV Pricing | Deribit | YES (advanced=implv param) | HIGH |
| Block RFQ | Deribit | YES (full endpoint suite) | MEDIUM |
| Combo/Spread Instruments | Deribit | YES (create_combo + buy/sell) | MEDIUM |
| Mass Quoting | Deribit | YES (mass_quote endpoint) | LOW (market makers only) |
| Market Maker Protection | Deribit | YES (MMP config endpoints) | LOW |
| Peer Lending | Bitfinex | YES (funding endpoints) | LOW |
| TWAP Orders | Bitget | YES (dedicated endpoint) | MEDIUM |
| Bracket Orders | Phemex | YES (native ordType 11-14) | MEDIUM |
| OCO Advanced | Crypto.com | YES (private/advanced/create-oco) | MEDIUM |
| OTO Advanced | Crypto.com | YES (private/advanced/create-oto) | MEDIUM |
| Copy Trading | Bitget, BingX | YES (separate API) | LOW (out of scope) |
| Earn/Lending | Multiple | YES (separate APIs) | LOW (out of scope) |

---

## Section 6: Key Findings Summary

1. **Bitfinex BatchOrders**: Fully supported via `POST /v2/auth/w/order/multi`. Max 75 ops per call. Supports mixed creates/cancels/amends. **Should implement.**

2. **BingX BatchOrders**: Supported via `/openApi/swap/v2/trade/batchOrders`. Also has CancelAll. **Should implement both.**

3. **Crypto.com BatchOrders**: Supported via `private/create-order-list`. **Should implement.**

4. **Deribit BatchOrders**: Intentionally absent. `mass_quote` is not a substitute — it's market maker only. CancelAll is very comprehensive (5+ variants). Our current CancelAll+Amend coverage is correct.

5. **Phemex BatchOrders**: Confirmed absent for retail API. Correct to skip.

6. **MEXC CancelAll**: Confirmed exists (`POST /api/v1/private/order/cancel_all`). **Should add.**

7. **HTX CancelAll**: Confirmed exists (`POST /v1/order/orders/batchCancelOpenOrders`). **Should add.**

8. **Deribit Iceberg**: Actually supported via `display_amount` and `refresh_amount` parameters on `private/buy`. This is a real gap in our order type coverage.

9. **Deribit GTD**: `good_til_day` is a valid `time_in_force` value. Supported.

10. **Deribit Options**: Primary product. Uses same API as futures but with options-format `instrument_name`. Options-specific: IV pricing (`advanced=implv`), USD pricing (`advanced=usd`). Greeks in market data. Our architecture has zero options support — this is the biggest gap.

11. **Deribit Block RFQ**: Full multi-step API for institutional block trades. 8+ dedicated endpoints.

12. **Deribit Combos**: First-class support for multi-leg strategy instruments traded on combo books.

13. **Bitfinex TWAP/Iceberg**: These exist but ONLY via the bfx-hf-algo JavaScript framework, which was deprecated March 2025. Effectively unsupported via direct REST API.

14. **Phemex Bracket Orders**: Native first-class `ordType` values (11=Bracket, 12=BoTpLimit, 13=BoSlLimit, 14=BoSlMarket). We likely have this as `UnsupportedOperation` when we should implement it.

15. **Bitget TWAP**: Available as a native dedicated endpoint (`place-twap-order`). Not covered by standard order traits — needs separate handling.

16. **Crypto.com trailing stop**: Not available via API. UI-only feature. Our `UnsupportedOperation` is correct.

---

## Sources

- [Bitfinex Order Multi-OP](https://docs.bitfinex.com/reference/rest-auth-order-multi)
- [Bitfinex Submit Order](https://docs.bitfinex.com/reference/rest-auth-submit-order)
- [Bitfinex Flag Values](https://docs.bitfinex.com/docs/flag-values)
- [Bitfinex Honey Framework](https://docs.bitfinex.com/reference/honey-framework)
- [BingX API Docs](https://bingx-api.github.io/docs/)
- [Crypto.com Exchange API v1](https://exchange-docs.crypto.com/exchange/v1/rest-ws/index.html)
- [Deribit API Documentation](https://docs.deribit.com/)
- [Deribit private/buy](https://docs.deribit.com/api-reference/trading/private-buy)
- [Deribit private/get_order_state](https://docs.deribit.com/api-reference/trading/private-get_order_state)
- [Deribit Block RFQ](https://support.deribit.com/hc/en-us/articles/25951371614621-Deribit-Block-RFQ)
- [Deribit Mass Quotes Specs](https://support.deribit.com/hc/en-us/articles/26302715576989-Mass-Quotes-Specifications)
- [Deribit Option Combo Order](https://support.deribit.com/hc/en-us/articles/25944794271261-Option-Combo-Order)
- [Deribit Portfolio Margin](https://support.deribit.com/hc/en-us/articles/25944756247837-Portfolio-Margin)
- [Deribit IV Order](https://support.deribit.com/hc/en-us/articles/25944809696029-IV-Order-Options)
- [Bitget Batch Order Endpoint](https://www.bitget.com/api-doc/contract/trade/Batch-Order)
- [Bitget Modify Order](https://www.bitget.com/api-doc/contract/trade/Modify-Order)
- [Bitget Cancel All Orders](https://www.bitget.com/api-doc/contract/trade/Cancel-All-Orders)
- [Bitget TWAP Guide](https://www.bitgetapp.com/support/articles/12560603819691)
- [Phemex API Reference](https://phemex-docs.github.io/)
- [Phemex Order Types](https://phemex.com/help-center/type-of-orders-on-phemex)
- [Phemex Bracket Order](https://phemex.com/help-center/what-is-a-bracket-order)
- [MEXC Contract API](https://mexcdevelop.github.io/apidocs/contract_v1_en/)
- [HTX Spot API](https://huobiapi.github.io/docs/spot/v1/en/)
