# CEX Connector Audit — Batch 2 (9 Connectors)

**Date**: 2026-03-12
**Files audited**: connector.rs for htx, mexc, bitget, bingx, phemex, crypto_com, upbit, bitstamp, gemini

**Legend**:
- REAL = method makes an actual API call (self.get/post/put/delete)
- STUB = returns UnsupportedOperation / NotSupported without any API call
- PARTIAL = makes an API call for some account_type branches, stubs others
- HARDCODED = returns hardcoded values (no API call, no error — just defaults)

---

## 1. HTX (`src/crypto/cex/htx/connector.rs`)

**place_order** — 6/14 REAL, 8 STUB
| OrderType | Status |
|-----------|--------|
| Market | REAL (POST PlaceOrder) |
| Limit | REAL (POST PlaceOrder) |
| StopLimit | REAL (POST PlaceOrder, buy/sell-stop-limit) |
| PostOnly | REAL (POST PlaceOrder, buy/sell-limit-maker) |
| Ioc | REAL (POST PlaceOrder, buy/sell-ioc) |
| Fok | REAL (POST PlaceOrder, buy/sell-limit-fok) |
| StopMarket | STUB (`_` wildcard) |
| ReduceOnly | STUB (`_` wildcard) |
| TrailingStop | STUB (`_` wildcard) |
| Bracket | STUB (`_` wildcard) |
| Oco | STUB (`_` wildcard) |
| Iceberg | STUB (`_` wildcard) |
| Twap | STUB (`_` wildcard) |
| Gtd | STUB (`_` wildcard) |

**cancel_order** — 2/4 REAL
| CancelScope | Status |
|-------------|--------|
| Single | REAL (POST CancelOrder via path variable) |
| Batch | REAL (POST CancelAllOrders with `order-ids` array) |
| All | STUB (UnsupportedOperation) |
| BySymbol | STUB (UnsupportedOperation) |

**modify_position** — 0/6 STUB (HTX is Spot-only connector)
| PositionModification | Status |
|----------------------|--------|
| SetLeverage | STUB (NotSupported — spot trading) |
| SetMarginMode | STUB (UnsupportedOperation) |
| AddMargin | STUB (UnsupportedOperation) |
| RemoveMargin | STUB (UnsupportedOperation) |
| ClosePosition | STUB (UnsupportedOperation) |
| SetTpSl | STUB (UnsupportedOperation) |

**get_order_history**: REAL (GET OrderHistory with states filter)
**get_fees**: REAL (GET /v2/reference/transact-fee-rate)
**get_balance**: REAL (GET Balance with account-id path var)
**get_account_info**: REAL (delegates to get_balance, constructs AccountInfo)
**get_positions**: REAL (returns empty vec — spot has no positions, not an error)
**get_funding_rate**: STUB (NotSupported — spot trading)

**Optional traits**:
- `impl CancelAll for HtxConnector` — YES (REAL: POST CancelAllOrders for All/BySymbol; STUB for other scopes)
- `impl BatchOrders for HtxConnector` — YES (place_orders_batch = STUB UnsupportedOperation; cancel_orders_batch = STUB)
- `impl AmendOrder for HtxConnector` — NO

---

## 2. MEXC (`src/crypto/cex/mexc/connector.rs`)

**place_order** — 5/14 REAL, 9 STUB
| OrderType | Status |
|-----------|--------|
| Market | REAL (POST PlaceOrder) |
| Limit | REAL (POST PlaceOrder) |
| PostOnly | REAL (POST PlaceOrder, LIMIT_MAKER) |
| Ioc | REAL (POST PlaceOrder, LIMIT+IOC) |
| Fok | REAL (POST PlaceOrder, LIMIT+FOK) |
| StopMarket | STUB (`_` wildcard) |
| StopLimit | STUB (`_` wildcard) |
| ReduceOnly | STUB (`_` wildcard) |
| TrailingStop | STUB (`_` wildcard) |
| Bracket | STUB (`_` wildcard) |
| Oco | STUB (`_` wildcard) |
| Iceberg | STUB (`_` wildcard) |
| Twap | STUB (`_` wildcard) |
| Gtd | STUB (`_` wildcard) |

**cancel_order** — 1/4 REAL
| CancelScope | Status |
|-------------|--------|
| Single | REAL (DELETE CancelOrder) |
| All | STUB (UnsupportedOperation — use CancelAll trait) |
| BySymbol | STUB (UnsupportedOperation) |
| Batch | STUB (UnsupportedOperation) |

**modify_position** — NOT IMPLEMENTED (no `impl Positions for MexcConnector`)

**get_order_history**: REAL (GET AllOrders)
**get_fees**: REAL (GET TradeFee)
**get_balance**: REAL (GET Account)
**get_account_info**: REAL (GET Account, extracts canTrade/Withdraw/Deposit and commissions)
**get_positions**: N/A — Positions trait not implemented
**get_funding_rate**: N/A — Positions trait not implemented

**Optional traits**:
- `impl CancelAll for MexcConnector` — YES (REAL for All+symbol/BySymbol; returns error for All without symbol; STUB for other scopes)
- `impl BatchOrders for MexcConnector` — YES (place_orders_batch = REAL via POST BatchOrders; cancel_orders_batch = STUB)
- `impl AmendOrder for MexcConnector` — NO

---

## 3. Bitget (`src/crypto/cex/bitget/connector.rs`)

**place_order** — 11/14 REAL, 3 STUB
| OrderType | Status |
|-----------|--------|
| Market | REAL (POST SpotCreateOrder / FuturesCreateOrder) |
| Limit | REAL (POST, spot+futures) |
| PostOnly | REAL (POST, force=post_only) |
| Ioc | REAL (POST, force=ioc) |
| Fok | REAL (POST, force=fok) |
| Gtd | REAL (POST, mapped to GTC limit — expire_time noted but not natively supported) |
| ReduceOnly | REAL (futures only; spot returns UnsupportedOperation inline, not a wildcard stub) |
| StopMarket | REAL (futures only; POST FuturesPlanOrder, planType=normal_plan, orderType=market) |
| StopLimit | REAL (futures only; POST FuturesPlanOrder, planType=normal_plan, orderType=limit) |
| TrailingStop | REAL (futures only; POST FuturesPlanOrder, planType=track_plan) |
| Bracket | REAL (futures only; POST FuturesCreateOrder with presetStopSurplusPrice) |
| Oco | STUB (`_` wildcard) |
| Iceberg | STUB (`_` wildcard) |
| Twap | STUB (`_` wildcard) |

**cancel_order** — 2/4 REAL
| CancelScope | Status |
|-------------|--------|
| Single | REAL (POST SpotCancelOrder / FuturesCancelOrder) |
| Batch | REAL (POST SpotBatchCancelOrders / FuturesBatchCancelOrders) |
| All | STUB (UnsupportedOperation — use CancelAll trait) |
| BySymbol | STUB (UnsupportedOperation) |

**modify_position** — 6/6 REAL
| PositionModification | Status |
|----------------------|--------|
| SetLeverage | REAL (POST FuturesSetLeverage; spots return inline UnsupportedOperation) |
| SetMarginMode | REAL (POST FuturesSetMarginMode) |
| AddMargin | REAL (POST FuturesSetMargin, operationType=add) |
| RemoveMargin | REAL (POST FuturesSetMargin, operationType=reduce) |
| ClosePosition | REAL (POST FuturesClosePositions) |
| SetTpSl | REAL (POST tpsl-order endpoint) |

**get_order_history**: REAL (GET SpotAllOrders / FuturesAllOrders)
**get_fees**: REAL (GET TradeRate for symbol; GET VipFeeRate for no-symbol)
**get_balance**: REAL (GET SpotAccounts / FuturesAccount)
**get_account_info**: REAL (delegates to get_balance, returns hardcoded commission)
**get_positions**: REAL (GET FuturesPosition / FuturesPositions; Spot returns UnsupportedOperation)
**get_funding_rate**: REAL (GET FundingRate; Spot returns UnsupportedOperation)

**Optional traits**:
- `impl CancelAll for BitgetConnector` — YES (REAL for All/BySymbol; STUB for other scopes)
- `impl AmendOrder for BitgetConnector` — YES (REAL: POST SpotAmendOrder / FuturesAmendOrder)
- `impl BatchOrders for BitgetConnector` — YES (place_orders_batch = REAL; cancel_orders_batch = REAL)

---

## 4. BingX (`src/crypto/cex/bingx/connector.rs`)

**place_order** — 9/14 REAL, 5 STUB
| OrderType | Status |
|-----------|--------|
| Market | REAL (POST SpotOrder / SwapOrder) |
| Limit | REAL (POST, GTC) |
| PostOnly | REAL (futures only; spot returns inline UnsupportedOperation) |
| Ioc | REAL (LIMIT+IOC) |
| Fok | REAL (LIMIT+FOK) |
| StopMarket | REAL (futures only; STOP_MARKET) |
| StopLimit | REAL (futures only; STOP) |
| TrailingStop | REAL (futures only; TRAILING_STOP_MARKET) |
| ReduceOnly | REAL (futures only; LIMIT/MARKET+reduceOnly=true) |
| Bracket | REAL (futures only; LIMIT/MARKET with embedded takeProfit/stopLoss JSON) |
| Oco | STUB (`_` wildcard) |
| Iceberg | STUB (`_` wildcard) |
| Twap | STUB (`_` wildcard) |
| Gtd | STUB (`_` wildcard) |

**cancel_order** — 1/4 REAL
| CancelScope | Status |
|-------------|--------|
| Single | REAL (DELETE SpotOrder / SwapOrder) |
| All | STUB (UnsupportedOperation — use CancelAll trait) |
| BySymbol | STUB (UnsupportedOperation) |
| Batch | STUB (UnsupportedOperation) |

**modify_position** — 3/6 REAL
| PositionModification | Status |
|----------------------|--------|
| SetLeverage | REAL (POST SwapLeverage; Spot returns inline UnsupportedOperation) |
| SetMarginMode | REAL (POST SwapMarginType) |
| ClosePosition | REAL (POST SwapOrder with closePosition=true) |
| AddMargin | STUB (`_` wildcard) |
| RemoveMargin | STUB (`_` wildcard) |
| SetTpSl | STUB (`_` wildcard) |

**get_order_history**: REAL (GET SwapAllOrders or SpotHistoryOrders)
**get_fees**: REAL (GET SpotCommissionRate; falls back to hardcoded 0.001 if call fails)
**get_balance**: REAL (GET SpotBalance / SwapBalance)
**get_account_info**: REAL (delegates to get_balance; uses hardcoded 0.1 fee)
**get_positions**: REAL (GET SwapPositions; Spot returns UnsupportedOperation)
**get_funding_rate**: REAL (GET SwapFundingRate; Spot returns UnsupportedOperation)

**Optional traits**:
- `impl CancelAll for BingxConnector` — YES (REAL: DELETE SpotCancelAllOrders / SwapCancelAllOrders for All/BySymbol)
- `impl AmendOrder for BingxConnector` — YES (REAL: POST SwapAmend; Spot returns UnsupportedOperation)
- `impl BatchOrders for BingxConnector` — NO

---

## 5. Phemex (`src/crypto/cex/phemex/connector.rs`)

**place_order** — 8/14 REAL, 6 STUB
| OrderType | Status |
|-----------|--------|
| Market | REAL (POST SpotCreateOrder / ContractCreateOrder) |
| Limit | REAL (POST, GoodTillCancel) |
| PostOnly | REAL (Limit+PostOnly timeInForce, both Spot and Contract) |
| Ioc | REAL (ImmediateOrCancel, both Spot/Contract, with/without price) |
| Fok | REAL (FillOrKill, both Spot/Contract) |
| StopMarket | REAL (Contract only; ordType=Stop; Spot returns inline UnsupportedOperation) |
| StopLimit | REAL (Contract only; ordType=StopLimit) |
| ReduceOnly | REAL (Contract only; reduceOnly=true) |
| TrailingStop | STUB (grouped wildcard) |
| Oco | STUB (grouped wildcard) |
| Bracket | STUB (grouped wildcard) |
| Iceberg | STUB (grouped wildcard) |
| Twap | STUB (grouped wildcard) |
| Gtd | STUB (grouped wildcard) |

**cancel_order** — 3/4 REAL
| CancelScope | Status |
|-------------|--------|
| Single | REAL (DELETE SpotCancelOrder / ContractCancelOrder) |
| All | REAL (DELETE SpotCancelAllOrders / ContractCancelAllOrders) |
| BySymbol | REAL (DELETE same endpoints with symbol param) |
| Batch | STUB (UnsupportedOperation — use CancelAll trait) |

**modify_position** — 5/6 REAL
| PositionModification | Status |
|----------------------|--------|
| SetLeverage | REAL (PUT SetLeverage) |
| SetMarginMode | REAL (PUT SetLeverage with 0 for cross / positive for isolated) |
| AddMargin | REAL (POST AssignBalance, add=true) |
| RemoveMargin | REAL (POST AssignBalance, add=false) |
| ClosePosition | REAL (POST ContractCreateOrder with reduceOnly=true, qty=999999999) |
| SetTpSl | STUB (UnsupportedOperation — "place separate TP/SL orders") |

**get_order_history**: REAL (GET ContractClosedOrders)
**get_fees**: HARDCODED (returns static 0.01%/0.06% — "Phemex doesn't expose a public fee endpoint")
**get_balance**: REAL (GET SpotWallets or ContractAccount)
**get_account_info**: REAL (delegates to get_balance)
**get_positions**: REAL (GET Positions with currency=BTC)
**get_funding_rate**: REAL (GET FundingRateHistory)

**Optional traits**:
- `impl CancelAll for PhemexConnector` — YES (REAL: DELETE SpotCancelAllOrders / ContractCancelAllOrders for All/BySymbol)
- `impl AmendOrder for PhemexConnector` — YES (REAL: PUT SpotAmendOrder / ContractAmendOrder)
- `impl BatchOrders for PhemexConnector` — NO

---

## 6. Crypto.com (`src/crypto/cex/crypto_com/connector.rs`)

**place_order** — 7/14 REAL, 7 STUB
| OrderType | Status |
|-----------|--------|
| Market | REAL (POST CreateOrder) |
| Limit | REAL (POST CreateOrder, GOOD_TILL_CANCEL) |
| StopMarket | REAL (POST CreateOrder, type=STOP_LOSS) |
| StopLimit | REAL (POST CreateOrder, type=STOP_LIMIT) |
| PostOnly | REAL (POST CreateOrder, exec_inst=POST_ONLY) |
| Ioc | REAL (POST CreateOrder, IMMEDIATE_OR_CANCEL) |
| Fok | REAL (POST CreateOrder, FILL_OR_KILL) |
| TrailingStop | STUB (grouped wildcard) |
| Oco | STUB (grouped wildcard) |
| Bracket | STUB (grouped wildcard) |
| Iceberg | STUB (grouped wildcard) |
| Twap | STUB (grouped wildcard) |
| Gtd | STUB (grouped wildcard) |
| ReduceOnly | STUB (grouped wildcard) |

**cancel_order** — 3/4 REAL (cancel_order handles All/BySymbol inline — no separate CancelAll needed in Trading trait itself)
| CancelScope | Status |
|-------------|--------|
| Single | REAL (POST CancelOrder) |
| All | REAL (POST CancelAllOrders, optional instrument_name) |
| BySymbol | REAL (POST CancelAllOrders, with instrument_name) |
| Batch | STUB (UnsupportedOperation) |

**modify_position** — 4/6 REAL
| PositionModification | Status |
|----------------------|--------|
| SetLeverage | REAL (POST ChangeAccountLeverage) |
| SetMarginMode | REAL (POST ChangeIsolatedMarginLeverage, leverage=0 for cross) |
| ClosePosition | REAL (POST ClosePosition, type=MARKET) |
| AddMargin | STUB (UnsupportedOperation — "endpoint not yet mapped") |
| RemoveMargin | STUB (UnsupportedOperation) |
| SetTpSl | STUB (UnsupportedOperation — "place separate TP/SL orders") |

**get_order_history**: REAL (POST GetOrderHistory)
**get_fees**: REAL (POST GetFeeRate or GetInstrumentFeeRate depending on symbol)
**get_balance**: REAL (POST UserBalance)
**get_account_info**: REAL (delegates to get_balance, uses hardcoded 0.075% fee)
**get_positions**: REAL (POST GetPositions; Spot returns UnsupportedOperation)
**get_funding_rate**: REAL (POST GetValuations)

**Optional traits**:
- `impl CancelAll for CryptoComConnector` — YES (REAL: POST CancelAllOrders for All/BySymbol)
- `impl AmendOrder for CryptoComConnector` — YES (REAL: POST AmendOrder)
- `impl BatchOrders for CryptoComConnector` — NO

---

## 7. Upbit (`src/crypto/cex/upbit/connector.rs`)

**place_order** — 2/14 REAL, 12 STUB
| OrderType | Status |
|-----------|--------|
| Market | REAL (POST CreateOrder, ord_type=price/market by side) |
| Limit | REAL (POST CreateOrder, ord_type=limit) |
| PostOnly | STUB (`_` wildcard) |
| Ioc | STUB (`_` wildcard) |
| Fok | STUB (`_` wildcard) |
| StopMarket | STUB (`_` wildcard) |
| StopLimit | STUB (`_` wildcard) |
| ReduceOnly | STUB (`_` wildcard) |
| TrailingStop | STUB (`_` wildcard) |
| Bracket | STUB (`_` wildcard) |
| Oco | STUB (`_` wildcard) |
| Iceberg | STUB (`_` wildcard) |
| Twap | STUB (`_` wildcard) |
| Gtd | STUB (`_` wildcard) |

Note: Upbit is Spot-only — no futures order types are expected.

**cancel_order** — 1/4 REAL
| CancelScope | Status |
|-------------|--------|
| Single | REAL (DELETE CancelOrder with uuid) |
| All | STUB (UnsupportedOperation) |
| BySymbol | STUB (UnsupportedOperation) |
| Batch | STUB (UnsupportedOperation) |

**modify_position** — NOT IMPLEMENTED (no Positions trait on Upbit)

**get_order_history**: REAL (GET ListOrders with state=done)
**get_fees**: STUB (UnsupportedOperation — "Upbit does not provide a fee query API endpoint")
**get_balance**: REAL (GET Balances with optional asset filter)
**get_account_info**: REAL (delegates to get_balance, uses hardcoded 0.05% fee)
**get_positions**: N/A — Positions trait not implemented
**get_funding_rate**: N/A — Positions trait not implemented

**Optional traits**:
- `impl CancelAll for UpbitConnector` — YES (REAL: DELETE BatchCancelOrders for All/BySymbol)
- `impl AmendOrder for UpbitConnector` — NO
- `impl BatchOrders for UpbitConnector` — NO

---

## 8. Bitstamp (`src/crypto/cex/bitstamp/connector.rs`)

**place_order** — 2/14 REAL, 12 STUB
| OrderType | Status |
|-----------|--------|
| Market | REAL (POST BuyMarket / SellMarket) |
| Limit | REAL (POST BuyLimit / SellLimit) |
| PostOnly | STUB (`_` wildcard) |
| Ioc | STUB (`_` wildcard) |
| Fok | STUB (`_` wildcard) |
| StopMarket | STUB (`_` wildcard) |
| StopLimit | STUB (`_` wildcard) |
| ReduceOnly | STUB (`_` wildcard) |
| TrailingStop | STUB (`_` wildcard) |
| Bracket | STUB (`_` wildcard) |
| Oco | STUB (`_` wildcard) |
| Iceberg | STUB (`_` wildcard) |
| Twap | STUB (`_` wildcard) |
| Gtd | STUB (`_` wildcard) |

Note: Bitstamp is primarily Spot — no futures order types expected.

**cancel_order** — 1/4 REAL
| CancelScope | Status |
|-------------|--------|
| Single | REAL (POST CancelOrder with id) |
| All | STUB (UnsupportedOperation) |
| BySymbol | STUB (UnsupportedOperation) |
| Batch | STUB (UnsupportedOperation) |

**modify_position** — NOT IMPLEMENTED (no Positions trait on Bitstamp)

**get_order_history**: REAL (POST UserTransactions)
**get_fees**: REAL (POST TradingFees)
**get_balance**: REAL (POST Balance)
**get_account_info**: REAL (POST Balance, constructs from balances, hardcoded 0.5% fee)
**get_positions**: N/A — Positions trait not implemented
**get_funding_rate**: N/A — Positions trait not implemented

**Optional traits**:
- `impl CancelAll for BitstampConnector` — YES (REAL: POST CancelAllOrders globally — no per-symbol scope, all scopes go to same endpoint)
- `impl AmendOrder for BitstampConnector` — NO
- `impl BatchOrders for BitstampConnector` — NO

---

## 9. Gemini (`src/crypto/cex/gemini/connector.rs`)

**place_order** — 6/14 REAL, 8 STUB
| OrderType | Status |
|-----------|--------|
| Market | REAL (POST NewOrder, type="exchange market") |
| Limit | REAL (POST NewOrder, type="exchange limit") |
| StopLimit | REAL (POST NewOrder, type="exchange stop limit") |
| PostOnly | REAL (POST NewOrder, options=["maker-or-cancel"]) |
| Ioc | REAL (POST NewOrder, options=["immediate-or-cancel"]) |
| Fok | REAL (POST NewOrder, options=["fill-or-kill"]) |
| StopMarket | STUB (`_` wildcard) |
| ReduceOnly | STUB (`_` wildcard) |
| TrailingStop | STUB (`_` wildcard) |
| Bracket | STUB (`_` wildcard) |
| Oco | STUB (`_` wildcard) |
| Iceberg | STUB (`_` wildcard) |
| Twap | STUB (`_` wildcard) |
| Gtd | STUB (`_` wildcard) |

**cancel_order** — 1/4 REAL
| CancelScope | Status |
|-------------|--------|
| Single | REAL (POST CancelOrder with order_id) |
| All | STUB (UnsupportedOperation) |
| BySymbol | STUB (UnsupportedOperation) |
| Batch | STUB (UnsupportedOperation) |

**modify_position** — 0/6 STUB
| PositionModification | Status |
|----------------------|--------|
| SetLeverage | STUB (NotSupported — "Gemini doesn't have a set leverage endpoint") |
| SetMarginMode | STUB (`_` wildcard → UnsupportedOperation) |
| AddMargin | STUB (`_` wildcard) |
| RemoveMargin | STUB (`_` wildcard) |
| ClosePosition | STUB (`_` wildcard) |
| SetTpSl | STUB (`_` wildcard) |

**get_order_history**: REAL (POST PastTrades)
**get_fees**: REAL (POST NotionalVolume)
**get_balance**: REAL (POST Balances)
**get_account_info**: HARDCODED (returns static `can_trade=true`, `maker_commission=0.0`, no API call)
**get_positions**: REAL (POST Positions)
**get_funding_rate**: REAL (GET FundingAmount)

**Optional traits**:
- `impl CancelAll for GeminiConnector` — YES (REAL: POST CancelAllOrders globally — ignores scope)
- `impl AmendOrder for GeminiConnector` — NO
- `impl BatchOrders for GeminiConnector` — NO

---

## Summary Table

| Connector | place_order (REAL/14) | cancel_order (REAL/4) | modify_position (REAL/6) | order_history | fees | balance | account_info | positions | funding_rate | CancelAll | AmendOrder | BatchOrders |
|-----------|----------------------|-----------------------|--------------------------|---------------|------|---------|--------------|-----------|--------------|-----------|------------|-------------|
| HTX | 6/14 | 2/4 | 0/6 (spot) | REAL | REAL | REAL | REAL | REAL (empty) | STUB | YES | NO | YES (stub impl) |
| MEXC | 5/14 | 1/4 | N/A | REAL | REAL | REAL | REAL | N/A | N/A | YES | NO | YES |
| Bitget | 11/14 | 2/4 | 6/6 | REAL | REAL | REAL | REAL | REAL | REAL | YES | YES | YES |
| BingX | 9/14 | 1/4 | 3/6 | REAL | REAL | REAL | REAL | REAL | REAL | YES | YES | NO |
| Phemex | 8/14 | 3/4 | 5/6 | REAL | HARDCODED | REAL | REAL | REAL | REAL | YES | YES | NO |
| Crypto.com | 7/14 | 3/4 | 4/6 | REAL | REAL | REAL | REAL | REAL | REAL | YES | YES | NO |
| Upbit | 2/14 | 1/4 | N/A | REAL | STUB | REAL | REAL | N/A | N/A | YES | NO | NO |
| Bitstamp | 2/14 | 1/4 | N/A | REAL | REAL | REAL | REAL | N/A | N/A | YES | NO | NO |
| Gemini | 6/14 | 1/4 | 0/6 | REAL | REAL | REAL | HARDCODED | REAL | REAL | YES | NO | NO |

---

## Key Findings

1. **Bitget** is the most complete connector: 11/14 order types, full modify_position (6/6), all optional traits present.
2. **BingX** is second: 9/14 order types, 3/6 modify_position (missing AddMargin, RemoveMargin, SetTpSl).
3. **Phemex** and **Crypto.com** are competitive at 7-8/14 and 4-5/6 modify_position.
4. **Upbit** and **Bitstamp** are Spot-only minimal connectors: 2/14 order types, no futures features.
5. **HTX** is Spot-only, 6/14 order types — has stop/ioc/fok, but no futures.
6. **MEXC** is 5/14 — missing all advanced order types (stop, trailing, reduce-only).
7. **Gemini** is 6/14 — has stop-limit and execution options (IOC/FOK/PostOnly) but no futures mechanics.

### Common gaps across all connectors:
- `Oco`, `Iceberg`, `Twap` — STUB everywhere
- `Gtd` — STUB in most (only Bitget has it as a semi-real mapping to GTC)
- `cancel_order(Batch)` — mostly STUB in cancel_order directly; handled differently in CancelAll/BatchOrders traits

### Fees edge cases:
- **Phemex** `get_fees` returns hardcoded 0.01%/0.06% — no API call
- **Gemini** `get_account_info` returns hardcoded 0% fees + empty balances — should call Balances endpoint
- **Upbit** `get_fees` always returns UnsupportedOperation (no fee API exists on Upbit)
