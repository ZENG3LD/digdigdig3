# Agent Carousel Coordinator - Data Providers

**Role:** Opus coordinator managing the full pipeline for data provider connectors.

---

## Mission

Execute the complete 4-phase pipeline to implement a data provider connector from scratch to production-ready.

**Input:** Provider name, category, documentation URL
**Output:** Fully tested connector with real data

---

## Pipeline Overview

```
┌─────────────────┐
│ Phase 1         │ → research/{provider}/
│ Research Agent  │    (8 markdown files)
└────────┬────────┘
         ▼
┌─────────────────┐
│ Phase 2         │ → src/{category}/{provider}/
│ Implement Agent │    (5-6 Rust files)
└────────┬────────┘
         ▼
┌─────────────────┐
│ Phase 3         │ → tests/{provider}_*.rs
│ Test Agent      │    (1-2 test files)
└────────┬────────┘
         ▼
┌─────────────────┐     ┌──────────────┐
│ Phase 4         │────►│ Tests Pass   │
│ Debug Agent     │     │ Real Data ✓  │
└────────┬────────┘     └──────────────┘
         │ failures
         ▼
    [loop back to fix]
```

---

## Execution Steps

### Step 0: Preparation

1. **Identify provider details:**
   - Provider name (lowercase, e.g., "polygon")
   - Category (aggregators / forex / stocks / data_feeds)
   - Documentation URL
   - Region (if stocks, e.g., "us", "india")

2. **Create research folder:**
   ```bash
   mkdir -p src/{CATEGORY}/{PROVIDER}/research
   # Example: src/stocks/us/polygon/research
   ```

3. **Set variables:**
   - `{PROVIDER}` = provider name (lowercase)
   - `{CATEGORY}` = category folder
   - `{DOCS_URL}` = official documentation URL

---

### Step 1: Launch Research Agent

**Agent:** `research-agent`
**Prompt:** `01_research.md`
**Duration:** ~1-2 hours

**Task:**
```
Research {PROVIDER} API for V5 connector implementation.

Category: {CATEGORY}
Documentation: {DOCS_URL}

Create folder: src/{CATEGORY}/{PROVIDER}/research/

Follow 01_research.md exactly.
Create all 8 research files with EXHAUSTIVE documentation.
```

**Variables to replace in prompt:**
- `{PROVIDER}` → actual provider name
- `{CATEGORY}` → aggregators/forex/stocks/data_feeds
- `{DOCS_URL}` → documentation URL

**Output verification:**
```bash
ls src/{CATEGORY}/{PROVIDER}/research/
# Should show:
# api_overview.md
# endpoints_full.md
# websocket_full.md
# authentication.md
# tiers_and_limits.md
# data_types.md
# response_formats.md
# coverage.md
```

**Review checklist:**
- [ ] All 8 files created
- [ ] No "TODO" or placeholder content
- [ ] Exact JSON examples from docs (not invented)
- [ ] All endpoints documented (including specialized ones)
- [ ] WebSocket documented (or noted as unavailable)
- [ ] Tier/pricing clear
- [ ] Data types cataloged

**If research incomplete:** Re-run research agent with specific gaps to fill.

---

### Step 2: Launch Implementation Agent

**Agent:** `rust-implementer`
**Prompt:** `02_implement.md`
**Duration:** ~2-4 hours

**Task:**
```
Implement {PROVIDER} connector based on research.

Category: {CATEGORY}
Research folder: src/{CATEGORY}/{PROVIDER}/research/

Follow 02_implement.md exactly.
Create 5-6 Rust files for the connector.

Reference implementation: src/exchanges/kucoin/
```

**Variables to replace in prompt:**
- `{PROVIDER}` → actual provider name
- `{CATEGORY}` → category folder

**Output verification:**
```bash
ls src/{CATEGORY}/{PROVIDER}/
# Should show:
# mod.rs
# endpoints.rs
# auth.rs
# parser.rs
# connector.rs
# websocket.rs (if WS available)
```

**Compilation check:**
```bash
cd zengeld-terminal/crates/connectors/crates/v5
cargo check --package digdigdig3
```

**Must compile with 0 errors** before proceeding.

**Review checklist:**
- [ ] All required files created
- [ ] ExchangeId variant added to core/types/common.rs
- [ ] Module exported in src/{CATEGORY}/mod.rs
- [ ] UnsupportedOperation for trading/account (unless broker)
- [ ] Compiles successfully (0 errors)

**If compilation fails:** Fix errors before Phase 3.

---

### Step 3: Launch Test Agent

**Agent:** `rust-implementer`
**Prompt:** `03_test.md`
**Duration:** ~1-2 hours

**Task:**
```
Create tests for {PROVIDER} connector.

Category: {CATEGORY}
Implementation: src/{CATEGORY}/{PROVIDER}/

Follow 03_test.md exactly.
Create integration and WebSocket test files.
```

**Variables to replace in prompt:**
- `{PROVIDER}` → actual provider name
- `{CATEGORY}` → category folder

**Output verification:**
```bash
ls tests/
# Should show:
# {provider}_integration.rs
# {provider}_websocket.rs (if WS available)
```

**Compilation check:**
```bash
cargo test --package digdigdig3 --test {provider}_integration --no-run
cargo test --package digdigdig3 --test {provider}_websocket --no-run
```

**Must compile** before proceeding to Phase 4.

**Review checklist:**
- [ ] Integration test file created
- [ ] WebSocket test file created (if applicable)
- [ ] Tests use graceful error handling (no panics)
- [ ] Tests compile successfully
- [ ] Clear println! output

---

### Step 4: Launch Debug Agent (Loop)

**Agent:** `rust-implementer`
**Prompt:** `04_debug.md`
**Duration:** ~1-4 hours (iterative)

**Task:**
```
Debug {PROVIDER} tests until they return REAL data.

Category: {CATEGORY}
Tests: tests/{provider}_*.rs

Follow 04_debug.md exactly.

Goal: At least one test returns real market data.
All tests handle errors gracefully.
```

**Variables to replace in prompt:**
- `{PROVIDER}` → actual provider name
- `{CATEGORY}` → category folder

**Run tests:**
```bash
# Integration tests
cargo test --package digdigdig3 --test {provider}_integration -- --nocapture

# WebSocket tests
cargo test --package digdigdig3 --test {provider}_websocket -- --nocapture
```

**Debug loop:**
1. Run tests
2. Identify failures
3. Fix implementation (endpoints.rs, parser.rs, connector.rs, websocket.rs)
4. Repeat until criteria met

**Exit criteria:**
- [ ] At least 1 data test returns REAL data (price, ticker, or WS events)
- [ ] Trading tests return UnsupportedOperation
- [ ] Account tests return UnsupportedOperation (unless broker)
- [ ] No panics in tests
- [ ] Graceful error handling for network/API issues

**Example successful output:**
```
✓ Price for AAPL/USD: $150.25
✓ Ticker: last=$150.25, bid=$150.24, ask=$150.26
✓ Retrieved 10 klines
✓ Trading correctly marked as unsupported
test result: ok. 9 passed; 0 failed
```

**If stuck:** Review common issues in 04_debug.md.

---

### Step 4.5: Launch Integration Test Agent

**Agent:** `rust-implementer`
**Prompt:** `05_integration_test.md`
**Duration:** ~1-2 hours

**Task:**
```
Write integration tests for {PROVIDER} with REAL API calls.

Category: {CATEGORY}
Implementation: src/{CATEGORY}/{PROVIDER}/

Follow 05_integration_test.md exactly.
Create live test file with real data validation.
```

**Variables to replace in prompt:**
- `{PROVIDER}` → actual provider name
- `{CATEGORY}` → category folder

**Output verification:**
```bash
ls tests/{provider}_live.rs
```

**Compilation check:**
```bash
cargo test --package digdigdig3 --test {provider}_live --no-run
```

**Must compile** before proceeding to Step 4.7.

**Review checklist:**
- [ ] Live test file created
- [ ] Tests marked with #[ignore]
- [ ] Credentials loaded from environment
- [ ] Data accuracy tests included
- [ ] WebSocket live stream tests included
- [ ] Tests compile successfully

---

### Step 4.7: Launch Integration Debug Agent (Loop)

**Agent:** `rust-implementer`
**Prompt:** `06_integration_debug.md`
**Duration:** ~1-4 hours (iterative)

**Task:**
```
Debug {PROVIDER} integration tests until they pass with REAL data.

Category: {CATEGORY}
Tests: tests/{provider}_live.rs

Follow 06_integration_debug.md exactly.

Goal: All integration tests pass or are gracefully skipped.
```

**Variables to replace in prompt:**
- `{PROVIDER}` → actual provider name
- `{CATEGORY}` → category folder

**Run tests:**
```bash
cargo test --package digdigdig3 --test {provider}_live -- --ignored --nocapture
```

**Debug loop:**
1. Run integration tests
2. Identify failures
3. Fix implementation or adapt test expectations
4. Repeat until criteria met

**Exit criteria:**
- [ ] All live tests pass with real data
- [ ] Or: remaining failures are gracefully skipped with documented reasons
- [ ] No panics or crashes
- [ ] Connector works with real provider data

**Example successful output:**
```
✓ test_live_price_matches_ticker ... ok
✓ test_live_orderbook_spread ... ok
✓ test_live_ws_ticker_accuracy ... ok
test result: ok. 8 passed; 0 failed
```

**If stuck:** Review common issues in 06_integration_debug.md.

---

### Step 5: Finalization

**After all tests passing:**

1. **Commit changes:**
   ```bash
   cd zengeld-terminal/crates/connectors/crates/v5
   git add src/{CATEGORY}/{PROVIDER}/ tests/{provider}_*.rs tests/{provider}_live.rs
   git commit -m "feat(v5): add {PROVIDER} connector ({CATEGORY})

   Phase 1-6 complete:
   - Research: 8 documentation files
   - Implementation: REST + WebSocket support
   - Unit tests: X/X passing with real data
   - Integration tests: Y/Y passing with live API
   - Data types: [list]

   Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
   ```

2. **Update registry** (if exists):
   - Add to ../../exchanges/CAROUSEL.md or similar

3. **Document learnings:**
   - Create README.md in provider folder if needed
   - Note any quirks or special handling

---

## Parallel Execution

**For multiple providers in same category:**

Launch Phase 1 for all in parallel:
```
Agent 1: Research polygon (stocks/us)
Agent 2: Research finnhub (stocks/us)
Agent 3: Research alpaca (stocks/us)
Agent 4: Research oanda (forex)
```

Wait for all research to complete, then:
```
Agent 1: Implement polygon
Agent 2: Implement finnhub
...
```

**Do NOT mix phases** - complete Phase 1 for all before starting Phase 2.

---

## Provider-Specific Notes

### Aggregators (cryptocompare, defillama, ib, yahoo)
- Focus on data coverage documentation
- May aggregate from multiple sources
- Symbol/ticker mapping critical

### Forex (alphavantage, dukascopy, oanda)
- Forex pair format varies: EUR_USD / EUR/USD / EURUSD
- Bid/Ask spreads important
- Some are brokers (OANDA) - may support trading

### Stocks (polygon, finnhub, alpaca, etc.)
- Region matters (US vs India vs Japan)
- Ticker format: "AAPL" (no quote asset)
- Real-time vs delayed data distinction
- Broker APIs (Alpaca, Zerodha) support trading

### Data Feeds (coinglass, fred, bitquery, whale_alert)
- Specialized data types (liquidations, macro, on-chain)
- May need custom parsers beyond standard traits
- Often read-only, no trading

---

## Quality Gates

**Before proceeding to next phase:**

| Phase | Gate | Command |
|-------|------|---------|
| 1 → 2 | All research files exist | `ls research/` |
| 2 → 3 | Code compiles | `cargo check` |
| 3 → 4 | Tests compile | `cargo test --no-run` |
| 4 → 4.5 | Unit tests pass with real data | `cargo test -- --nocapture` |
| 4.5 → 4.7 | Integration tests compile | `cargo test --test {provider}_live --no-run` |
| 4.7 → Done | Integration tests pass with real data | `cargo test --test {provider}_live -- --ignored --nocapture` |

**Do not skip gates** - fix issues before proceeding.

---

## Troubleshooting

### Research Agent Issues
- **Problem:** Agent invents data instead of using docs
- **Fix:** Re-run with emphasis on "EXACT examples from docs"

### Implementation Agent Issues
- **Problem:** Code doesn't compile
- **Fix:** Provide compilation errors and ask to fix

### Test Agent Issues
- **Problem:** Tests panic on errors
- **Fix:** Emphasize graceful error handling with `match`

### Debug Agent Issues
- **Problem:** Can't get real data
- **Fix:** Check API key, rate limits, endpoint URLs
- **Fix:** Make manual curl call to verify API works

---

## Success Metrics

**Per Provider:**
- Research: 8/8 files complete
- Implementation: Compiles with 0 errors
- Tests: At least 1 returning real data
- Time: 4-8 hours total (with agents)

**For 26 providers:**
- Total time: ~4 weeks (with parallel execution)
- Success rate target: 80%+ (some may be deprecated/broken)

---

## After Completion

**Provider is production-ready when:**
- ✅ Research documented
- ✅ Code compiles
- ✅ Tests pass
- ✅ Real data verified
- ✅ Committed to repo

**Next steps:**
- Use connector in trading systems
- Monitor for API changes
- Add more features as needed
- Document production issues

---

## Templates

### Launch Research (Phase 1)
```
Research {PROVIDER} API.
Category: {CATEGORY}
Docs: {DOCS_URL}
Follow: 01_research.md
Output: src/{CATEGORY}/{PROVIDER}/research/
```

### Launch Implementation (Phase 2)
```
Implement {PROVIDER} connector.
Category: {CATEGORY}
Research: src/{CATEGORY}/{PROVIDER}/research/
Follow: 02_implement.md
Reference: src/exchanges/kucoin/
```

### Launch Tests (Phase 3)
```
Create tests for {PROVIDER}.
Category: {CATEGORY}
Follow: 03_test.md
Output: tests/{provider}_*.rs
```

### Launch Debug (Phase 4)
```
Debug {PROVIDER} tests.
Category: {CATEGORY}
Follow: 04_debug.md
Goal: Real data from tests
```

---

## End of Coordinator Guide

Use this guide to execute the full pipeline for any data provider.
Follow phases sequentially, verify gates, and ensure quality at each step.
