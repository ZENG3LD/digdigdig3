# Disabled Exchanges

This document tracks exchanges that have been disabled in the V5 connector architecture while keeping their code for reference.

## Currently Disabled

### Vertex Protocol

**Status:** PERMANENTLY DISABLED
**Reason:** Exchange shut down on August 14, 2025 (acquired by Ink Foundation)
**Code Location:** `src/exchanges/vertex/`
**Tests Moved To:** `src/exchanges/vertex/_tests_integration.rs`, `src/exchanges/vertex/_tests_websocket.rs`

**Details:**
- Vertex Protocol was a decentralized perpetuals exchange on Arbitrum
- Acquired by Ink Foundation and permanently shut down
- All trading ceased on August 14, 2025
- See `research/vertex/` for shutdown announcement and details

**Re-enable:** Not applicable - exchange is permanently closed

---

### Bithumb

**Status:** TEMPORARILY DISABLED
**Reason:** Persistent infrastructure issues (SSL hangs, 403 geo-blocking, connection timeouts)
**Code Location:** `src/exchanges/bithumb/`
**Tests Moved To:** `src/exchanges/bithumb/_tests_integration.rs`, `src/exchanges/bithumb/_tests_websocket.rs`

**Details:**
- SSL/TLS connection hangs on initialization
- 403 Forbidden errors (likely geo-blocking)
- Connection timeouts on both REST and WebSocket
- See `src/exchanges/bithumb/research/504_investigation.md` for detailed analysis

**Re-enable Instructions:**
1. Verify Bithumb has fixed infrastructure issues
2. Test REST API connectivity from your region
3. Uncomment module declaration in `src/exchanges/mod.rs`:
   ```rust
   pub mod bithumb;
   ```
4. Uncomment ExchangeId variants in `src/core/types/common.rs`:
   ```rust
   Bithumb,  // In enum declaration
   Self::Bithumb => "bithumb",  // In as_str() method
   ```
5. Move tests back to `tests/` directory:
   ```bash
   mv src/exchanges/bithumb/_tests_integration.rs tests/bithumb_integration.rs
   mv src/exchanges/bithumb/_tests_websocket.rs tests/bithumb_websocket.rs
   ```
6. Run `cargo check` and `cargo test --package digdigdig3`

---

### Phemex

**Status:** REMOVED FROM LIVE WATCHLIST (code retained for reference)
**Reason:** HTTP 403 on WebSocket connections — IP/region block that persists even with Origin headers
**Code Location:** `src/exchanges/phemex/`
**Date Removed:** 2026-03-03

**Details:**
- WebSocket connection endpoint returns HTTP 403 Forbidden during the upgrade handshake
- Error is not authentication-related — it occurs before any credentials are sent
- Adding `Origin` headers matching Phemex web client does not resolve the block
- The block is IP-level or region-level, applied at the CDN/proxy layer before the WebSocket server
- REST API may still function for some regions, but real-time data via WebSocket is unavailable
- Without a live WebSocket feed, the exchange cannot provide ticker updates for the watchlist

**Re-enable Instructions:**
1. Verify WebSocket connectivity: `wscat -c wss://ws.phemex.com/ws` — must return HTTP 101
2. If connectivity is restored, uncomment in `src/exchanges/mod.rs`:
   ```rust
   pub mod phemex;
   ```
3. `ExchangeId::Phemex` is already present in `src/core/types/common.rs` — no changes needed there
4. Run `cargo check` and integration tests

---

### GMX

**Status:** REMOVED FROM LIVE WATCHLIST (code retained for reference)
**Reason:** No real WebSocket API — price data is sourced from on-chain queries only; REST polling by design
**Code Location:** `src/exchanges/gmx/`
**Date Removed:** 2026-03-03

**Details:**
- GMX is a decentralized perpetuals exchange on Arbitrum and Avalanche
- There is no official GMX WebSocket API for real-time price streams
- Ticker/price data requires querying on-chain contract state or GMX's own REST endpoint with polling
- The REST polling approach introduces latency that is not appropriate for a live watchlist alongside CEX feeds
- GMX prices are also derived from Chainlink oracle feeds (multi-source aggregated) rather than direct order-book matching, making tick-level streaming conceptually different from CEX data
- The GMX Stats API (`stats.gmx.io`) is read-only REST with no push capability

**Re-enable Instructions:**
1. Monitor GMX v2 API roadmap for any official WebSocket support
2. If a WebSocket feed becomes available, implement it in `src/exchanges/gmx/websocket.rs`
3. Uncomment in `src/exchanges/mod.rs`:
   ```rust
   pub mod gmx;
   ```
4. `ExchangeId::Gmx` is already present in `src/core/types/common.rs` — no changes needed there
5. Run `cargo check` and integration tests

---

### Paradex

**Status:** REMOVED FROM LIVE WATCHLIST (code retained for reference)
**Reason:** Unreliable data — global `markets_summary` channel makes per-symbol attribution impossible, showing random data from other markets.
**Code Location:** `src/exchanges/paradex/`
**Date Removed:** 2026-03-03

**Details:**
- Paradex WebSocket uses a single global `markets_summary` channel that broadcasts updates for all markets
- There is no per-symbol subscription mechanism; the channel cannot be filtered server-side
- This makes it impossible to attribute incoming price data to the correct symbol in a multi-symbol watchlist
- Data from unrelated markets bleeds into whichever symbol is being tracked, producing incorrect prices and volumes

**Re-enable Instructions:**
1. Monitor Paradex API for the introduction of per-symbol ticker subscriptions
2. If a symbol-scoped channel becomes available, update `src/exchanges/paradex/websocket.rs` to subscribe per-symbol
3. Uncomment in `src/exchanges/mod.rs`:
   ```rust
   pub mod paradex;
   ```
4. `ExchangeId::Paradex` is already present in `src/core/types/common.rs` — no changes needed there
5. Run `cargo check` and integration tests

---

## How to Disable an Exchange

If you need to disable another exchange:

1. **Move tests into exchange directory** (prevents them from running):
   ```bash
   mv tests/{exchange}_integration.rs src/exchanges/{exchange}/_tests_integration.rs
   mv tests/{exchange}_websocket.rs src/exchanges/{exchange}/_tests_websocket.rs
   ```

2. **Comment out module declaration** in `src/exchanges/mod.rs`:
   ```rust
   // DISABLED: Reason here
   // pub mod {exchange};
   ```

3. **Comment out ExchangeId variants** in `src/core/types/common.rs`:
   - In the `ExchangeId` enum declaration
   - In the `as_str()` method
   - In the `exchange_type()` method (if applicable)

4. **Document in this file** with reason, date, and re-enable instructions

5. **Verify compilation**:
   ```bash
   cargo check --package digdigdig3
   cargo test --package digdigdig3  # Should not run disabled exchange tests
   ```

---

## Notes

- Disabled exchange code is **kept for reference** - do not delete
- Tests are moved (not deleted) so they don't run but remain available
- Module declarations are commented out (not removed) for easy re-enabling
- Always document the reason for disabling and how to re-enable
