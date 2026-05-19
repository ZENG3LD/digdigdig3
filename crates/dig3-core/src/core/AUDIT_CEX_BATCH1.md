# CEX Connector Audit — Batch 1 (9 Connectors)

**Audit date:** 2026-03-12
**Method:** Static analysis of `connector.rs` for each exchange
**Definition of REAL:** method body calls `self.get()`, `self.post()`, `self.delete()`, or `self.rpc_call()` and returns parsed API data
**Definition of STUB:** method arm returns `Err(ExchangeError::UnsupportedOperation(...))` or `Err(ExchangeError::NotSupported(...))` without any HTTP call

---

## Legend

| Symbol | Meaning |
|--------|---------|
| REAL   | Makes HTTP/RPC call, parses response |
| STUB   | Returns error without any API call |
| PARTIAL | Makes API call but returns hardcoded/incomplete data alongside it |

---

## 1. Binance

**File:** `digdigdig3/src/crypto/cex/binance/connector.rs`

### place_order — 12/14 REAL

| OrderType | Status | Note |
|-----------|--------|------|
| Market | REAL | SpotCreateOrder / FuturesCreateOrder |
| Limit | REAL | SpotCreateOrder / FuturesCreateOrder |
| StopMarket | REAL | FuturesCreateOrder only (Spot: STUB guard) |
| StopLimit | REAL | SpotCreateOrder / FuturesCreateOrder |
| TrailingStop | REAL | FuturesCreateOrder only (Spot: STUB guard) |
| Oco | REAL | SpotOcoOrder (Futures: STUB guard) |
| Bracket | STUB | UnsupportedOperation — no native endpoint |
| Iceberg | REAL | SpotCreateOrder (Futures: STUB guard) |
| Twap | STUB | UnsupportedOperation — no standard API |
| PostOnly | REAL | SpotCreateOrder / FuturesCreateOrder (GTX timeInForce) |
| Ioc | REAL | SpotCreateOrder / FuturesCreateOrder |
| Fok | REAL | SpotCreateOrder / FuturesCreateOrder |
| Gtd | STUB | UnsupportedOperation — not supported on Binance |
| ReduceOnly | REAL | FuturesCreateOrder only (Spot: STUB guard) |

**Score: 10 fully REAL, 2 conditionally REAL (Spot guard), 2 STUB = 12/14 REAL**

### cancel_order — 1/4 REAL

| CancelScope | Status | Note |
|-------------|--------|------|
| Single | REAL | SpotCancelOrder / FuturesCancelOrder via DELETE |
| Batch | STUB | "Use BatchOrders trait" |
| All | STUB | "Use CancelAll trait" |
| BySymbol | STUB | "Use CancelAll trait" |

**Score: 1/4 REAL** (All/BySymbol intentionally delegated to CancelAll trait)

### modify_position — 5/6 REAL

| PositionModification | Status | Note |
|----------------------|--------|------|
| SetLeverage | REAL | FuturesSetLeverage (Spot: STUB guard) |
| SetMarginMode | REAL | FuturesSetMarginType (Spot: STUB guard) |
| AddMargin | REAL | FuturesPositionMargin type=1 (Spot: STUB guard) |
| RemoveMargin | REAL | FuturesPositionMargin type=2 (Spot: STUB guard) |
| ClosePosition | REAL | FuturesCreateOrder reduceOnly (Spot: STUB guard) |
| SetTpSl | STUB | UnsupportedOperation — no single native endpoint |

**Score: 5/6 REAL**

### Other methods

| Method | Status | Note |
|--------|--------|------|
| get_order_history | REAL | SpotAllOrders / FuturesAllOrders |
| get_fees | REAL | SpotTradeFee (fallback to SpotAccount commissionRates) |
| get_balance | REAL | SpotAccount / FuturesAccount |
| get_account_info | REAL | SpotAccount / FuturesAccount + commissionRates |
| get_positions | REAL | FuturesPositions (Spot: STUB guard) |
| get_funding_rate | REAL | FundingRate endpoint (Spot: STUB guard) |

### Optional traits

| Trait | Implemented | Quality |
|-------|------------|---------|
| CancelAll | YES | REAL — SpotCancelAllOrders / FuturesCancelAllOrders via DELETE |
| AmendOrder | YES | REAL — FuturesAmendOrder (Spot: STUB guard) |
| BatchOrders | YES | REAL — FuturesBatchOrders (Spot: STUB) |

---

## 2. Bybit

**File:** `digdigdig3/src/crypto/cex/bybit/connector.rs`

### place_order — 10/14 REAL

| OrderType | Status | Note |
|-----------|--------|------|
| Market | REAL | PlaceOrder with orderType=Market |
| Limit | REAL | PlaceOrder with orderType=Limit |
| StopMarket | REAL | PlaceOrder with orderType=Market + stopPrice |
| StopLimit | REAL | PlaceOrder with orderType=Limit + stopPrice |
| TrailingStop | REAL* | Places order, but Spot guard returns STUB |
| PostOnly | REAL | PlaceOrder with timeInForce=PostOnly |
| Ioc | REAL | PlaceOrder with timeInForce=IOC |
| Fok | REAL | PlaceOrder with timeInForce=FOK |
| Gtd | REAL | PlaceOrder with timeInForce=GTD + orderLinkId |
| ReduceOnly | REAL* | Places order, but Spot guard returns STUB |
| Iceberg | STUB | UnsupportedOperation |
| Oco | STUB | UnsupportedOperation |
| Bracket | STUB | UnsupportedOperation |
| Twap | STUB | UnsupportedOperation |

**Score: 10/14 REAL**

### cancel_order — 3/4 REAL

| CancelScope | Status | Note |
|-------------|--------|------|
| Single | REAL | CancelOrder endpoint |
| All | REAL | CancelAll endpoint |
| BySymbol | REAL | CancelAll endpoint with symbol filter |
| Batch | STUB | UnsupportedOperation — no native batch |

**Score: 3/4 REAL**

### modify_position — 5/6 REAL (all 6 have API calls but some Spot guards)

| PositionModification | Status | Note |
|----------------------|--------|------|
| SetLeverage | REAL | SetLeverage endpoint (Spot: STUB guard) |
| SetMarginMode | REAL | SwitchMarginMode endpoint (Spot: STUB guard) |
| AddMargin | REAL | AddReduceMargin endpoint (Spot: STUB guard) |
| RemoveMargin | REAL | AddReduceMargin endpoint (Spot: STUB guard) |
| ClosePosition | REAL | PlaceOrder reduceOnly (Spot: STUB guard) |
| SetTpSl | REAL | PlaceOrder with tpPrice/slPrice (Spot: STUB guard) |

**Score: 6/6 REAL** (all make API calls; some only for futures)

### Other methods

| Method | Status | Note |
|--------|--------|------|
| get_order_history | REAL | OrderHistory endpoint |
| get_fees | REAL | FeeRate endpoint |
| get_balance | REAL | Balance endpoint with accountType param |
| get_account_info | REAL | AccountInfo endpoint + get_balance |
| get_positions | REAL | Positions endpoint (Spot: STUB guard) |
| get_funding_rate | REAL | FundingRate endpoint (Spot: STUB guard) |

### Optional traits

| Trait | Implemented | Quality |
|-------|------------|---------|
| CancelAll | NO | — |
| AmendOrder | NO | — |
| BatchOrders | NO | — |

---

## 3. OKX

**File:** `digdigdig3/src/crypto/cex/okx/connector.rs`

### place_order — 11/14 REAL

| OrderType | Status | Note |
|-----------|--------|------|
| Market | REAL | PlaceOrder ordType=market |
| Limit | REAL | PlaceOrder ordType=limit |
| PostOnly | REAL | PlaceOrder ordType=post_only |
| Ioc | REAL | PlaceOrder ordType=optimal_limit_ioc |
| Fok | REAL | PlaceOrder ordType=fok |
| StopMarket | REAL | PlaceOrder ordType=conditional with slTriggerPx/tpTriggerPx |
| StopLimit | REAL | PlaceOrder ordType=conditional with stop+limit prices |
| ReduceOnly | REAL | PlaceOrder with reduceOnly=true |
| Gtd | REAL | PlaceOrder ordType=limit (expire_time ignored with note) |
| TrailingStop | REAL | PlaceOrder ordType=move_order_stop |
| Twap | REAL | PlaceOrder ordType=twap |
| Oco | STUB | UnsupportedOperation |
| Bracket | STUB | UnsupportedOperation |
| Iceberg | STUB | UnsupportedOperation |

**Score: 11/14 REAL**

### cancel_order — 2/4 REAL

| CancelScope | Status | Note |
|-------------|--------|------|
| Single | REAL | CancelOrder endpoint |
| Batch | REAL | CancelBatchOrders endpoint |
| All | STUB | UnsupportedOperation — OKX no atomic cancel-all REST |
| BySymbol | STUB | UnsupportedOperation — same reason |

**Score: 2/4 REAL**

### modify_position — 6/6 REAL

| PositionModification | Status | Note |
|----------------------|--------|------|
| SetLeverage | REAL | SetLeverage endpoint |
| SetMarginMode | REAL | SetLeverage with mgnMode param (Spot: STUB guard) |
| AddMargin | REAL | Raw POST to /api/v5/account/position/margin-balance type=add |
| RemoveMargin | REAL | Raw POST to /api/v5/account/position/margin-balance type=reduce |
| ClosePosition | REAL | Raw POST to /api/v5/trade/close-position |
| SetTpSl | REAL | PlaceOrder with tpTriggerPx/slTriggerPx |

**Score: 6/6 REAL**

### Other methods

| Method | Status | Note |
|--------|--------|------|
| get_order_history | REAL | OrderHistory endpoint |
| get_fees | REAL | AccountConfig endpoint (makerFeeRate/takerFeeRate) |
| get_balance | REAL | Balance endpoint |
| get_account_info | PARTIAL | Calls get_balance; commission values are hardcoded defaults |
| get_positions | REAL | Positions endpoint |
| get_funding_rate | REAL | FundingRate endpoint |

### Optional traits

| Trait | Implemented | Quality |
|-------|------------|---------|
| CancelAll | NO | — |
| AmendOrder | NO | — |
| BatchOrders | NO | — |

---

## 4. KuCoin

**File:** `digdigdig3/src/crypto/cex/kucoin/connector.rs`

### place_order — 8/14 REAL

| OrderType | Status | Note |
|-----------|--------|------|
| Market | REAL | SpotCreateOrder / FuturesCreateOrder |
| Limit | REAL | SpotCreateOrder / FuturesCreateOrder |
| PostOnly | REAL | SpotCreateOrder with postOnly=true |
| Ioc | REAL | SpotCreateOrder with timeInForce=IOC |
| Fok | REAL | SpotCreateOrder with timeInForce=FOK |
| StopMarket | REAL | SpotStopOrder / FuturesCreateOrder |
| StopLimit | REAL | SpotStopOrder / FuturesCreateOrder |
| ReduceOnly | REAL | FuturesCreateOrder with closeOrder=true (Spot: STUB guard) |
| Gtd | REAL | SpotCreateOrder with remark timestamp |
| TrailingStop | STUB | UnsupportedOperation |
| Oco | STUB | UnsupportedOperation |
| Bracket | STUB | UnsupportedOperation |
| Iceberg | STUB | UnsupportedOperation |
| Twap | STUB | UnsupportedOperation |

**Score: 9/14 REAL**

### cancel_order — 3/4 REAL

| CancelScope | Status | Note |
|-------------|--------|------|
| Single | REAL | SpotCancelOrder / FuturesCancelOrder |
| All | REAL | SpotCancelAllOrders / FuturesCancelAllOrders |
| BySymbol | REAL | SpotCancelAllOrders with symbol filter |
| Batch | STUB | "KuCoin does not have native batch cancel" |

**Score: 3/4 REAL**

### modify_position — 5/6 REAL

| PositionModification | Status | Note |
|----------------------|--------|------|
| SetLeverage | REAL | FuturesSetLeverage (Spot: STUB guard) |
| SetMarginMode | REAL | FuturesChangeMarginMode (Spot: STUB guard) |
| AddMargin | REAL | FuturesAddMargin (Spot: STUB guard) |
| RemoveMargin | STUB | UnsupportedOperation — endpoint research inconclusive |
| ClosePosition | REAL | FuturesCreateOrder closeOrder=true (Spot: STUB guard) |
| SetTpSl | REAL | FuturesCreateOrder stop orders for TP+SL (Spot: STUB guard) |

**Score: 5/6 REAL**

### Other methods

| Method | Status | Note |
|--------|--------|------|
| get_order_history | REAL | SpotOrderHistory / FuturesOrderHistory |
| get_fees | REAL | Raw GET to /api/v1/base-fee |
| get_balance | REAL | SpotAccounts / FuturesAccount |
| get_account_info | PARTIAL | Calls get_balance; commission hardcoded to 0.1% defaults |
| get_positions | REAL | FuturesPosition(s) endpoint |
| get_funding_rate | REAL | FuturesFundingRate endpoint |

### Optional traits

| Trait | Implemented | Quality |
|-------|------------|---------|
| CancelAll | NO | — |
| AmendOrder | NO | — (weights module references AmendOrder but no impl) |
| BatchOrders | NO | — |

---

## 5. Kraken

**File:** `digdigdig3/src/crypto/cex/kraken/connector.rs`

### place_order — 9/14 REAL

| OrderType | Status | Note |
|-----------|--------|------|
| Market | REAL | SpotAddOrder / FuturesSendOrder |
| Limit | REAL | SpotAddOrder / FuturesSendOrder |
| PostOnly | REAL | SpotAddOrder with oflags=post |
| Ioc | REAL | SpotAddOrder with timeinforce=IOC |
| Fok | REAL | SpotAddOrder with timeinforce=IOC (no native FOK — uses IOC) |
| StopMarket | REAL | SpotAddOrder ordertype=stop-loss |
| StopLimit | REAL | SpotAddOrder ordertype=stop-loss-limit |
| Gtd | REAL | SpotAddOrder timeinforce=GTD + expiretm |
| ReduceOnly | REAL | FuturesSendOrder with reduceOnly=true (Spot: STUB guard) |
| TrailingStop | STUB | UnsupportedOperation |
| Oco | STUB | UnsupportedOperation |
| Bracket | STUB | UnsupportedOperation |
| Iceberg | STUB | UnsupportedOperation |
| Twap | STUB | UnsupportedOperation |

**Score: 9/14 REAL**

### cancel_order — 3/4 REAL

| CancelScope | Status | Note |
|-------------|--------|------|
| Single | REAL | SpotCancelOrder |
| All | REAL | SpotCancelOrder / FuturesCancelOrder (uses same endpoint, no symbol filtering for spot) |
| BySymbol | REAL | FuturesCancelOrder with symbol filter (spot uses no-filter endpoint) |
| Batch | STUB | UnsupportedOperation — Spot no batch, Futures per-item only |

**Score: 3/4 REAL**

### modify_position — 2/6 REAL

| PositionModification | Status | Note |
|----------------------|--------|------|
| SetLeverage | REAL | FuturesSetLeverage endpoint (Spot: STUB guard) |
| SetMarginMode | STUB | UnsupportedOperation |
| AddMargin | STUB | UnsupportedOperation |
| RemoveMargin | STUB | UnsupportedOperation |
| ClosePosition | REAL | FuturesSendOrder reduceOnly=true (Spot: STUB guard) |
| SetTpSl | STUB | UnsupportedOperation |

**Score: 2/6 REAL**

### Other methods

| Method | Status | Note |
|--------|--------|------|
| get_order_history | REAL | SpotClosedOrders / FuturesHistory |
| get_fees | REAL | SpotTradeBalance endpoint |
| get_balance | REAL | SpotBalance endpoint |
| get_account_info | PARTIAL | Calls get_balance; commission hardcoded to Starter tier defaults |
| get_positions | REAL | FuturesOpenPositions (Spot: STUB guard) |
| get_funding_rate | REAL | FuturesHistoricalFunding (Spot: STUB guard) |

### Optional traits

| Trait | Implemented | Quality |
|-------|------------|---------|
| CancelAll | NO | — |
| AmendOrder | NO | — |
| BatchOrders | NO | — |

---

## 6. Coinbase

**File:** `digdigdig3/src/crypto/cex/coinbase/connector.rs`

### place_order — 10/14 REAL

| OrderType | Status | Note |
|-----------|--------|------|
| Market | REAL | CreateOrder market_market_ioc |
| Limit | REAL | CreateOrder limit_limit_gtc/ioc/fok |
| PostOnly | REAL | CreateOrder limit_limit_gtc post_only=true |
| Ioc | REAL | CreateOrder limit_limit_ioc |
| Fok | REAL | CreateOrder limit_limit_fok |
| StopMarket | REAL | CreateOrder stop_limit_stop_limit_gtc |
| StopLimit | REAL | CreateOrder stop_limit_stop_limit_gtc |
| Gtd | REAL | CreateOrder limit_limit_gtd with end_time |
| Oco | REAL | CreateOrder trigger_bracket_gtc |
| Bracket | REAL | CreateOrder trigger_bracket_gtc |
| ReduceOnly | STUB | UnsupportedOperation |
| TrailingStop | STUB | UnsupportedOperation |
| Iceberg | STUB | UnsupportedOperation |
| Twap | STUB | UnsupportedOperation |

**Score: 10/14 REAL**

### cancel_order — 4/4 REAL

| CancelScope | Status | Note |
|-------------|--------|------|
| Single | REAL | CancelOrders endpoint with single order_id |
| All | REAL | Fetches open orders then calls CancelOrders with all IDs |
| BySymbol | REAL | Fetches open orders for symbol then calls CancelOrders |
| Batch | REAL | CancelOrders endpoint with order_ids array |

**Score: 4/4 REAL**

### modify_position — 0/6 REAL

| PositionModification | Status | Note |
|----------------------|--------|------|
| SetLeverage | STUB | NotSupported — Coinbase does not support leverage |
| SetMarginMode | STUB | UnsupportedOperation |
| AddMargin | STUB | UnsupportedOperation |
| RemoveMargin | STUB | UnsupportedOperation |
| ClosePosition | STUB | UnsupportedOperation |
| SetTpSl | STUB | UnsupportedOperation |

**Score: 0/6 REAL** — Coinbase perpetuals are limited; no position management via REST

### Other methods

| Method | Status | Note |
|--------|--------|------|
| get_order_history | REAL | ListOrders with historical filter |
| get_fees | REAL | TransactionSummary endpoint |
| get_balance | REAL | Accounts endpoint |
| get_account_info | REAL | TransactionSummary + get_balance |
| get_positions | STUB | NotSupported — Coinbase does not support futures/positions |
| get_funding_rate | STUB | NotSupported — Coinbase does not support funding rates |

### Optional traits

| Trait | Implemented | Quality |
|-------|------------|---------|
| CancelAll | NO | — |
| AmendOrder | NO | — |
| BatchOrders | NO | — |

---

## 7. Gate.io

**File:** `digdigdig3/src/crypto/cex/gateio/connector.rs`

### place_order — 7/14 REAL

| OrderType | Status | Note |
|-----------|--------|------|
| Market | REAL | SpotCreateOrder / FuturesCreateOrder |
| Limit | REAL | SpotCreateOrder / FuturesCreateOrder |
| PostOnly | REAL | SpotCreateOrder with text=post_only |
| Ioc | REAL | SpotCreateOrder with tif=IOC |
| Fok | REAL | SpotCreateOrder with tif=FOK |
| ReduceOnly | REAL* | FuturesCreateOrder reduce_only=true (Spot: STUB guard) |
| StopMarket | REAL* | Spot: STUB guard; Futures: FuturesCreateOrder |
| StopLimit | REAL* | Spot: STUB guard; Futures: FuturesCreateOrder |
| TrailingStop | STUB | UnsupportedOperation |
| Oco | STUB | UnsupportedOperation |
| Bracket | STUB | UnsupportedOperation |
| Iceberg | STUB | UnsupportedOperation |
| Twap | STUB | UnsupportedOperation |
| Gtd | STUB | UnsupportedOperation |

**Score: 7/14 REAL** (StopMarket/StopLimit partial — futures REAL, spot STUB)

### cancel_order — 3/4 REAL

| CancelScope | Status | Note |
|-------------|--------|------|
| Single | REAL | SpotCancelOrder / FuturesCancelOrder |
| All | REAL | SpotCancelAllOrders / FuturesCancelAllOrders |
| BySymbol | REAL | SpotCancelAllOrders / FuturesCancelAllOrders with symbol |
| Batch | STUB | UnsupportedOperation |

**Score: 3/4 REAL**

### modify_position — 5/6 REAL

| PositionModification | Status | Note |
|----------------------|--------|------|
| SetLeverage | REAL | FuturesUpdatePosition leverage param (Spot: STUB guard) |
| SetMarginMode | REAL | FuturesUpdatePosition cross_leverage_limit param (Spot: STUB guard) |
| AddMargin | REAL | FuturesUpdatePositionMargin with add amount (Spot: STUB guard) |
| RemoveMargin | REAL | FuturesUpdatePositionMargin with reduce amount (Spot: STUB guard) |
| ClosePosition | REAL | FuturesCreateOrder reduce_only + size=0 (Spot: STUB guard) |
| SetTpSl | STUB | UnsupportedOperation |

**Score: 5/6 REAL**

### Other methods

| Method | Status | Note |
|--------|--------|------|
| get_order_history | REAL | SpotOrderHistory / FuturesOrderHistory |
| get_fees | REAL | Raw GET to /spot/fee endpoint |
| get_balance | REAL | SpotAccounts / FuturesAccounts |
| get_account_info | PARTIAL | Calls get_balance; commission hardcoded to 0.2% defaults |
| get_positions | REAL | FuturesPosition / FuturesPositions (Spot: STUB guard) |
| get_funding_rate | REAL | FuturesFundingRate endpoint |

### Optional traits

| Trait | Implemented | Quality |
|-------|------------|---------|
| CancelAll | NO | — |
| AmendOrder | NO | — |
| BatchOrders | NO | — |

---

## 8. Bitfinex

**File:** `digdigdig3/src/crypto/cex/bitfinex/connector.rs`

### place_order — 9/14 REAL

| OrderType | Status | Note |
|-----------|--------|------|
| Market | REAL | SubmitOrder type=EXCHANGE MARKET |
| Limit | REAL | SubmitOrder type=EXCHANGE LIMIT |
| PostOnly | REAL | SubmitOrder type=EXCHANGE LIMIT flags=4096 |
| Ioc | REAL | SubmitOrder type=EXCHANGE IOC |
| Fok | REAL | SubmitOrder type=EXCHANGE FOK |
| StopMarket | REAL | SubmitOrder type=EXCHANGE STOP |
| StopLimit | REAL | SubmitOrder type=EXCHANGE STOP LIMIT |
| TrailingStop | REAL | SubmitOrder type=EXCHANGE TRAILING STOP |
| Iceberg | REAL | SubmitOrder type=EXCHANGE LIMIT flags=64 with max_show |
| ReduceOnly | REAL | SubmitOrder with flags=1024 (Spot: STUB guard) |
| Gtd | STUB | Falls through to catch-all UnsupportedOperation |
| Oco | STUB | Falls through to catch-all UnsupportedOperation |
| Bracket | STUB | Falls through to catch-all UnsupportedOperation |
| Twap | STUB | Falls through to catch-all UnsupportedOperation |

**Score: 10/14 REAL**

### cancel_order — 2/4 REAL

| CancelScope | Status | Note |
|-------------|--------|------|
| Single | REAL | CancelOrder endpoint |
| Batch | REAL | CancelMultipleOrders endpoint with id array |
| All | STUB | UnsupportedOperation — "use CancelAll trait" |
| BySymbol | STUB | UnsupportedOperation — "use CancelAll trait" |

**Score: 2/4 REAL** (All/BySymbol intentionally delegated to CancelAll trait)

### modify_position — 0/6 REAL

| PositionModification | Status | Note |
|----------------------|--------|------|
| SetLeverage | STUB | "use order flags instead" |
| SetMarginMode | STUB | UnsupportedOperation |
| AddMargin | STUB | UnsupportedOperation |
| RemoveMargin | STUB | UnsupportedOperation |
| ClosePosition | STUB | UnsupportedOperation |
| SetTpSl | STUB | UnsupportedOperation |

**Score: 0/6 REAL**

### Other methods

| Method | Status | Note |
|--------|--------|------|
| get_order_history | REAL | OrderHistory endpoint |
| get_fees | REAL | TradeHistory endpoint (derives rate from most recent trade) |
| get_balance | REAL | Wallets endpoint |
| get_account_info | PARTIAL | Calls get_balance; commission hardcoded to 0.1%/0.2% defaults |
| get_positions | REAL | Positions endpoint (Spot: STUB guard) |
| get_funding_rate | STUB | UnsupportedOperation — "endpoint not implemented" |

### Optional traits

| Trait | Implemented | Quality |
|-------|------------|---------|
| CancelAll | YES | REAL — CancelMultipleOrders with all=1 or symbol filter |
| AmendOrder | YES | REAL — UpdateOrder endpoint |
| BatchOrders | NO | — |

---

## 9. Deribit

**File:** `digdigdig3/src/crypto/cex/deribit/connector.rs`

### place_order — 11/14 REAL

| OrderType | Status | Note |
|-----------|--------|------|
| Market | REAL | Buy/Sell RPC type=market |
| Limit | REAL | Buy/Sell RPC type=limit |
| PostOnly | REAL | Buy/Sell RPC type=limit post_only=true |
| Ioc | REAL | Buy/Sell RPC type=limit tif=immediate_or_cancel |
| Fok | REAL | Buy/Sell RPC type=limit tif=fill_or_kill |
| StopMarket | REAL | Buy/Sell RPC type=stop_market |
| StopLimit | REAL | Buy/Sell RPC type=stop_limit |
| TrailingStop | REAL | Buy/Sell RPC type=trailing_stop |
| Gtd | REAL | Buy/Sell RPC type=limit tif=good_til_day |
| ReduceOnly | REAL | Buy/Sell RPC reduce_only=true |
| Iceberg | REAL | Buy/Sell RPC type=limit max_show param |
| Oco | STUB | Falls through to catch-all UnsupportedOperation |
| Bracket | STUB | Falls through to catch-all UnsupportedOperation |
| Twap | STUB | Falls through to catch-all UnsupportedOperation |

**Score: 11/14 REAL**

### cancel_order — 1/4 REAL

| CancelScope | Status | Note |
|-------------|--------|------|
| Single | REAL | Cancel RPC call |
| All | STUB | UnsupportedOperation — "use CancelAll trait" |
| BySymbol | STUB | UnsupportedOperation — "use CancelAll trait" |
| Batch | STUB | UnsupportedOperation — "use CancelAll trait" |

**Score: 1/4 REAL** (All/BySymbol/Batch intentionally delegated to CancelAll trait)

### modify_position — 1/6 REAL

| PositionModification | Status | Note |
|----------------------|--------|------|
| SetLeverage | STUB | UnsupportedOperation — "Deribit uses dynamic leverage" |
| SetMarginMode | STUB | UnsupportedOperation |
| AddMargin | STUB | UnsupportedOperation |
| RemoveMargin | STUB | UnsupportedOperation |
| ClosePosition | REAL | ClosePosition RPC call |
| SetTpSl | STUB | UnsupportedOperation |

**Score: 1/6 REAL** — Deribit's leverage model is inherently different; not missing work

### Other methods

| Method | Status | Note |
|--------|--------|------|
| get_order_history | REAL | GetUserTradesByInstrument / GetUserTradesByCurrency RPC |
| get_fees | REAL | GetAccountSummary extended=true (extracts maker/taker commission) |
| get_balance | REAL | GetAccountSummary RPC |
| get_account_info | PARTIAL | Calls get_balance; commission/permission fields are hardcoded |
| get_positions | REAL | GetPosition / GetPositions RPC |
| get_funding_rate | REAL | Ticker RPC (extracts current_funding / funding_8h) |

### Optional traits

| Trait | Implemented | Quality |
|-------|------------|---------|
| CancelAll | YES | REAL — CancelAll / CancelAllByInstrument RPC |
| AmendOrder | YES | REAL — Edit RPC |
| BatchOrders | NO | — |

---

## Summary Table

| Exchange | place_order | cancel_order | modify_position | get_balance | get_fees | get_account_info | get_positions | get_funding_rate | get_order_history | CancelAll | AmendOrder | BatchOrders |
|----------|------------|-------------|----------------|------------|---------|-----------------|--------------|----------------|------------------|-----------|-----------|------------|
| Binance  | 12/14 | 1/4* | 5/6 | REAL | REAL | REAL | REAL | REAL | REAL | YES | YES | YES |
| Bybit    | 10/14 | 3/4 | 6/6 | REAL | REAL | PARTIAL | REAL | REAL | REAL | NO | NO | NO |
| OKX      | 11/14 | 2/4 | 6/6 | REAL | REAL | PARTIAL | REAL | REAL | REAL | NO | NO | NO |
| KuCoin   | 9/14 | 3/4 | 5/6 | REAL | REAL | PARTIAL | REAL | REAL | REAL | NO | NO | NO |
| Kraken   | 9/14 | 3/4 | 2/6 | REAL | REAL | PARTIAL | REAL | REAL | REAL | NO | NO | NO |
| Coinbase | 10/14 | 4/4 | 0/6 | REAL | REAL | REAL | STUB | STUB | REAL | NO | NO | NO |
| Gate.io  | 7/14 | 3/4 | 5/6 | REAL | REAL | PARTIAL | REAL | REAL | REAL | NO | NO | NO |
| Bitfinex | 10/14 | 2/4* | 0/6 | REAL | REAL | PARTIAL | REAL | STUB | REAL | YES | YES | NO |
| Deribit  | 11/14 | 1/4* | 1/6 | REAL | REAL | PARTIAL | REAL | REAL | REAL | YES | YES | NO |

`*` = intentionally partial (remaining scopes delegated to optional trait)
`PARTIAL` = makes API call but returns hardcoded commission/permission values alongside real balance data

---

## Key Observations

### Recurring Stubs Across All Connectors
- `get_account_info`: All connectors except Binance and Coinbase return hardcoded commission rates (0.1% or similar) instead of fetching from the API. Balance data is real; commission data is fake.
- `modify_position::SetTpSl`: Almost universally stubbed except OKX and KuCoin.
- `cancel_order::Batch` in `Trading` trait: Usually stubbed or delegated to `BatchOrders` trait.

### Exchange-Specific Weaknesses
- **Kraken**: `modify_position` is 2/6 — only SetLeverage and ClosePosition work. No margin management.
- **Coinbase**: No position management at all (0/6). Makes sense given API limitations for perpetuals.
- **Bitfinex**: No position modification (0/6). Funding rate is stubbed.
- **Gate.io**: StopMarket/StopLimit are Futures-only (Spot returns STUB guard).

### Strongest Connectors (Trading completeness)
1. **Binance** — Most complete, all optional traits implemented
2. **Bybit** — Strong, 6/6 modify_position
3. **OKX** — Strong, 6/6 modify_position, 11/14 order types
4. **Deribit** — Excellent for its domain (options/futures), has CancelAll + AmendOrder

### Missing Optional Traits
- **Bybit**, **OKX**, **KuCoin**, **Kraken**, **Coinbase**, **Gate.io**: None of the 3 optional traits implemented.
- Only **Binance** implements all 3 (CancelAll + AmendOrder + BatchOrders).
