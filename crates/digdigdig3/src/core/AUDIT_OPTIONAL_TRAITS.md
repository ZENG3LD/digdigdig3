# Audit: Optional Trait Implementations vs Capability Matrix

Generated: 2026-03-12
Sources: `TRADING_CAPABILITY_MATRIX.md`, `src/core/traits/operations.rs`, all `connector.rs` files under `src/crypto/cex/*/` and `src/crypto/dex/*/`

## Scope

This audit covers three optional traits defined in `src/core/traits/operations.rs`:
- `CancelAll` — native bulk-cancel endpoint
- `AmendOrder` — native in-place order modification endpoint
- `BatchOrders` — native multi-order placement/cancellation endpoint

The matrix references are from `TRADING_CAPABILITY_MATRIX.md` Table C (Order Management).

Exchanges not in the 24-exchange matrix (Bithumb, Vertex) are excluded from gap analysis — they are either disabled or reference-only.

**Gap definition:** Exchange has a native API endpoint per the matrix (Y or P) but no `impl <Trait> for <Connector>` in its `connector.rs`.

---

## CancelAll (matrix says 22/24)

Matrix: all exchanges EXCEPT dYdX (N — Cosmos tx-based, no bulk cancel in v4).
HyperLiquid and Coinbase are marked P (partial) in the matrix.

| Exchange | Has `impl CancelAll`? | Matrix says | Gap? |
|----------|----------------------|-------------|------|
| Binance | YES | Y | No |
| Bybit | NO | Y | **YES** |
| OKX | NO | Y | **YES** |
| KuCoin | NO | Y | **YES** |
| Kraken | NO | Y | **YES** |
| Coinbase | NO | P | Partial — low priority |
| Gate.io | NO | Y | **YES** |
| Bitfinex | YES | Y | No |
| Bitstamp | YES | Y | No |
| MEXC | YES | Y | No |
| HTX | YES | Y | No |
| Bitget | YES | Y | No |
| Gemini | YES | Y | No |
| BingX | YES | Y | No |
| Phemex | YES | Y | No |
| Crypto.com | YES | Y | No |
| Upbit | YES | Y | No |
| Deribit | YES | Y | No |
| HyperLiquid | NO | P | Partial — medium priority |
| Lighter | NO | Y | **YES** |
| Paradex | NO | Y | **YES** |
| dYdX | NO | N | Correct — Cosmos, no bulk cancel |

**Summary: 12 implemented, 7 confirmed gaps (Y in matrix but no impl), 2 partial gaps (Coinbase P, HyperLiquid P), 1 correctly absent.**

### CancelAll Gaps (Y in matrix, no impl)
1. `src/crypto/cex/bybit/connector.rs`
2. `src/crypto/cex/okx/connector.rs`
3. `src/crypto/cex/kucoin/connector.rs`
4. `src/crypto/cex/kraken/connector.rs`
5. `src/crypto/cex/gateio/connector.rs`
6. `src/crypto/dex/lighter/connector.rs`
7. `src/crypto/dex/paradex/connector.rs`

---

## AmendOrder (matrix says 18/24)

Matrix (Table C, "Amend" column): Y = Binance, Bybit, OKX, Gate.io, Bitfinex, Bitget, BingX, Phemex, Crypto.com, Deribit, HyperLiquid, Lighter, Paradex. P (partial) = KuCoin, Coinbase, Bitstamp, Bitget(F only), Upbit. N = MEXC, HTX, Gemini, dYdX.

Per `operations.rs` doc comment, the 18 non-N exchanges are:
Binance(F), Bybit, OKX, KuCoin(P), GateIO, Bitfinex, Bitget(F), BingX(F), Phemex, CryptoCom, Deribit, HyperLiquid, Lighter, Paradex, Upbit(P), Coinbase(P), Bitstamp(P), and dYdX is N.

| Exchange | Has `impl AmendOrder`? | Matrix says | Gap? |
|----------|----------------------|-------------|------|
| Binance | YES | Y (Futures only) | No |
| Bybit | NO | Y | **YES** |
| OKX | NO | Y | **YES** |
| KuCoin | NO | P | Partial — lower priority |
| Kraken | NO | P | Partial — lower priority |
| Coinbase | NO | P | Partial — lower priority |
| Gate.io | NO | Y | **YES** |
| Bitfinex | YES | Y | No |
| Bitstamp | NO | P | Partial — lower priority |
| MEXC | NO | N | Correct — no amend endpoint |
| HTX | NO | N | Correct — no amend endpoint |
| Bitget | YES | P (Futures only) | No |
| Gemini | NO | N | Correct — no amend endpoint |
| BingX | YES | Y (Futures only) | No |
| Phemex | YES | Y | No |
| Crypto.com | YES | Y | No |
| Upbit | NO | P | Partial — lower priority |
| Deribit | YES | Y | No |
| HyperLiquid | NO | Y | **YES** |
| Lighter | NO | Y | **YES** |
| Paradex | NO | Y | **YES** |
| dYdX | NO | N | Correct — no amend endpoint |

**Summary: 7 implemented, 6 confirmed gaps (Y in matrix but no impl), 5 partial gaps (P in matrix), 4 correctly absent.**

### AmendOrder Gaps (Y in matrix, no impl)
1. `src/crypto/cex/bybit/connector.rs`
2. `src/crypto/cex/okx/connector.rs`
3. `src/crypto/cex/gateio/connector.rs`
4. `src/crypto/cex/hyperliquid/connector.rs`
5. `src/crypto/dex/lighter/connector.rs`
6. `src/crypto/dex/paradex/connector.rs`

---

## BatchOrders (matrix says 17/24)

Matrix (Table C, "Batch" column — non-N exchanges):
Y = Binance(F:5), Bybit(20/10), OKX(20), KuCoin(S:5), Gate.io(S:10), Bitfinex(75), MEXC(S:20,F:50), HTX(10), Bitget(50), BingX(unspec.), Phemex, Crypto.com(10), Deribit, HyperLiquid(unspec.), Lighter(50), Paradex(10), dYdX(P).
N = Kraken (spot only, no batch), Coinbase (N), Gemini (N), Bitstamp (N), Upbit (N).

| Exchange | Has `impl BatchOrders`? | Matrix says | Gap? |
|----------|------------------------|-------------|------|
| Binance | YES | Y (Futures:5) | No |
| Bybit | NO | Y (20/10) | **YES** |
| OKX | NO | Y (20) | **YES** |
| KuCoin | NO | Y (S:5, F:open) | **YES** |
| Kraken | NO | N | Correct — no batch endpoint |
| Coinbase | NO | N | Correct — no batch endpoint |
| Gate.io | NO | Y (S:10) | **YES** |
| Bitfinex | NO | Y (75 mixed) | **YES** |
| Bitstamp | NO | N | Correct — no batch endpoint |
| MEXC | YES | Y (S:20, F:50) | No |
| HTX | YES | Y (10) | No |
| Bitget | YES | Y (50) | No |
| Gemini | NO | N | Correct — no batch endpoint |
| BingX | NO | Y (unspec.) | **YES** |
| Phemex | NO | N | Correct — Phemex has no batch create (matrix N for Batch) |
| Crypto.com | NO | Y (10) | **YES** |
| Upbit | NO | N | Correct — no batch endpoint |
| Deribit | NO | N | Correct — Deribit has no batch create (matrix N) |
| HyperLiquid | NO | Y (unspec.) | **YES** |
| Lighter | NO | Y (50) | **YES** |
| Paradex | NO | Y (10) | **YES** |
| dYdX | NO | P | Partial — lower priority |

**Summary: 4 implemented, 11 confirmed gaps (Y in matrix but no impl), 1 partial gap (dYdX P), 5 correctly absent.**

### BatchOrders Gaps (Y in matrix, no impl)
1. `src/crypto/cex/bybit/connector.rs`
2. `src/crypto/cex/okx/connector.rs`
3. `src/crypto/cex/kucoin/connector.rs`
4. `src/crypto/cex/gateio/connector.rs`
5. `src/crypto/cex/bitfinex/connector.rs`
6. `src/crypto/cex/bingx/connector.rs`
7. `src/crypto/cex/crypto_com/connector.rs`
8. `src/crypto/cex/hyperliquid/connector.rs`
9. `src/crypto/dex/lighter/connector.rs`
10. `src/crypto/dex/paradex/connector.rs`
11. `src/crypto/dex/dydx/connector.rs` (Partial — verify native batch endpoint)

> Note: For `Phemex` — matrix Table C shows "N" for Batch (no batch create), so Phemex is correctly absent. The `operations.rs` doc comment mentions Phemex in the BatchOrders list (line 88) but the matrix disagrees. Matrix is the authoritative source — Phemex should NOT implement BatchOrders.
> Note: For `Deribit` — matrix Table C shows "N" for Batch (no batch create). The `operations.rs` doc comment includes Deribit in the list, but the matrix says N. Verify before implementing.

---

## Priority Matrix

Sorting gaps by exchange importance (trading volume / rank) and how many traits are missing:

| Exchange | Missing CancelAll | Missing AmendOrder | Missing BatchOrders | Total Gaps | Priority |
|----------|:-----------------:|:------------------:|:-------------------:|:----------:|----------|
| Bybit | YES | YES | YES | 3 | P0 |
| OKX | YES | YES | YES | 3 | P0 |
| Gate.io | YES | YES | YES | 3 | P0 |
| HyperLiquid | P | YES | YES | 2 | P1 |
| Lighter | YES | YES | YES | 3 | P1 |
| Paradex | YES | YES | YES | 3 | P1 |
| KuCoin | YES | — | YES | 2 | P1 |
| Bitfinex | — | — | YES | 1 | P2 |
| BingX | — | — | YES | 1 | P2 |
| Crypto.com | — | — | YES | 1 | P2 |
| Kraken | YES | — | — | 1 | P2 |
| dYdX | — | — | P | 0-1 | P3 (partial only) |
| Coinbase | P | — | — | 0-1 | P3 (partial only) |

---

## Consistency Note: operations.rs vs Matrix

The doc comment in `operations.rs` lists exchanges for each trait. Cross-checking against the matrix reveals two discrepancies:

| Trait | operations.rs says | Matrix says | Verdict |
|-------|-------------------|-------------|---------|
| `BatchOrders` | includes Phemex | Matrix C: Phemex Batch = N | Matrix correct — Phemex has no batch place endpoint |
| `BatchOrders` | includes Deribit | Matrix C: Deribit Batch = N | Verify — Deribit has batch cancel but possibly not batch place |
| `AmendOrder` | includes dYdX | Matrix C: dYdX Amend = N | Verify — dYdX V4 has no native amend (Cosmos tx cancel+replace only) |

The `operations.rs` doc comment should be updated to reflect the matrix findings after verification.

---

## Files Referenced

- `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/crypto/TRADING_CAPABILITY_MATRIX.md`
- `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/core/traits/operations.rs`
- All `connector.rs` files under:
  - `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/crypto/cex/*/connector.rs`
  - `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/crypto/dex/*/connector.rs`
