# Coordinator Prompt: Exchange Connector Pipeline

## Overview

This prompt is for the Opus coordinator agent to run the full connector creation pipeline.

---

## Instructions for Coordinator

You are coordinating the creation of a new exchange connector. Follow this pipeline exactly.

### Step 1: Read the Exchange Registry

```
Read: ../CAROUSEL.md → Exchange Registry section
```

Identify the next exchange to implement from the "CEX - High Priority" or "CEX - Medium Priority" lists.

### Step 2: Run Phase 1 (Research)

```
Task: research-agent
Prompt: Read 01_research.md and use it with:
  {EXCHANGE} = target exchange name (lowercase)
  {DOCS_URL} = documentation URL from registry
```

**Wait for completion.**

Verify output:
```
ls src/exchanges/{exchange}/research/
```
Expected: endpoints.md, authentication.md, response_formats.md, symbols.md, rate_limits.md, websocket.md

### Step 3: Run Phase 2 (Implement)

```
Task: rust-implementer
Prompt: Read 02_implement.md and use it with:
  {EXCHANGE} = target exchange name (lowercase)
  {Exchange} = target exchange name (PascalCase)
```

**Wait for completion.**

Verify:
```
cargo check --package digdigdig3
```
Expected: no errors

### Step 4: Run Phase 3 (Test)

```
Task: rust-implementer
Prompt: Read 03_test.md and use it with:
  {EXCHANGE} = target exchange name (lowercase)
  {Exchange} = target exchange name (PascalCase)
```

**Wait for completion.**

Verify:
```
ls tests/{exchange}_integration.rs tests/{exchange}_websocket.rs
```
Expected: both files exist

### Step 5: Run Phase 4 (Debug Loop)

```
Task: rust-implementer
Prompt: Read 04_debug.md and use it with:
  {EXCHANGE} = target exchange name (lowercase)
```

**Repeat until all tests pass (max 10 iterations).**

Check:
```
cargo test --package digdigdig3 --test {exchange}_integration 2>&1 | tail -3
cargo test --package digdigdig3 --test {exchange}_websocket 2>&1 | tail -3
```

Expected: "test result: ok" for both

### Step 5.5: Run Phase 5 (Integration Tests)

Task: rust-implementer
Prompt: Read 05_integration_test.md and use it with:
  {EXCHANGE} = target exchange name (lowercase)
  {Exchange} = target exchange name (PascalCase)

**Wait for completion.**

Verify:
```
cargo test --package digdigdig3 --test {exchange}_live --no-run
```
Expected: compiles without errors

### Step 5.7: Run Phase 6 (Integration Debug Loop)

Task: rust-implementer
Prompt: Read 06_integration_debug.md and use it with:
  {EXCHANGE} = target exchange name (lowercase)

**Repeat until all integration tests pass (max 10 iterations).**

Check:
```
cargo test --package digdigdig3 --test {exchange}_live -- --ignored --nocapture 2>&1 | tail -3
```
Expected: "test result: ok" or all failures gracefully handled

### Step 6: Commit

```bash
git add src/exchanges/{exchange}/
git add tests/{exchange}_*.rs
git add tests/{exchange}_live.rs
git commit -m "feat(v5/{exchange}): implement connector with tests"
```

### Step 7: Update Registry

Update ../CAROUSEL.md:
- Move exchange from "Planned" to "Completed"
- Add test counts

### Step 8: Report

```
✓ {Exchange} connector completed
  - REST tests: X passed
  - WebSocket tests: Y passed
  - Commit: {hash}
```

---

## Parallel Execution (Optional)

For independent exchanges, run multiple pipelines in parallel:

```
[Exchange A Pipeline]     [Exchange B Pipeline]     [Exchange C Pipeline]
       ↓                         ↓                         ↓
   research                  research                  research
       ↓                         ↓                         ↓
   implement                 implement                 implement
       ↓                         ↓                         ↓
     test                       test                      test
       ↓                         ↓                         ↓
    debug                      debug                     debug
       ↓                         ↓                         ↓
   commit                     commit                    commit
```

---

## Quick Start Example

```
User: "Реализуй коннектор для Bybit"

Coordinator:
1. Task(research-agent): "Research Bybit API. Docs: https://bybit-exchange.github.io/docs/. Create src/exchanges/bybit/research/ with all 6 files."

2. [Wait] → Verify research files created

3. Task(rust-implementer): "Implement Bybit connector. Reference: kucoin/. Research: bybit/research/. Create endpoints.rs, auth.rs, parser.rs, connector.rs, websocket.rs, mod.rs. Run cargo check after each file."

4. [Wait] → Verify cargo check passes

5. Task(rust-implementer): "Write tests for Bybit. Reference: kucoin tests. Create tests/bybit_integration.rs and tests/bybit_websocket.rs. Include all required tests from 03_test.md."

6. [Wait] → Verify test files created

7. Loop: Task(rust-implementer): "Debug Bybit tests. Fix failures until all pass."

8. [Wait until all pass]

9. Commit and report
```

---

## Exchange-Specific Notes

### CEX
Standard REST + WebSocket pattern. Follow KuCoin reference.

### DEX (On-chain)
May require different approach:
- dYdX, Hyperliquid: REST-like API, standard pattern works
- GMX, Uniswap, Raydium: May need RPC/blockchain integration
- Consider creating separate DEX reference implementation first

---

## Troubleshooting

### Research agent returns incomplete data
- Check if docs URL is correct
- Some exchanges have separate Spot and Futures docs
- May need to search for specific endpoints manually

### Implementation fails cargo check
- Check trait implementations match exactly
- Verify all required methods are implemented
- Check imports are correct

### Tests fail consistently
- Check research docs for accuracy
- Run single test with --nocapture to see details
- Compare with working exchange (e.g., KuCoin)

### WebSocket connection drops
- Ping/pong not implemented correctly
- Check websocket.md for heartbeat format
- Some exchanges use gzip compression
