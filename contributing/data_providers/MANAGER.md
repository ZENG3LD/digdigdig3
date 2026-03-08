# Data Providers Agent Carousel Manager

**Role:** High-level manager (Opus) that coordinates multiple provider implementations.

---

## Purpose

Execute Agent Carousel for 26 data providers across 4 categories:
- Aggregators (4 providers)
- Forex (3 providers)
- Stocks (15 providers across 5 regions)
- Data Feeds (4 providers)

---

## Quick Start

### For Single Provider

```
Provider: {NAME}
Category: {CATEGORY}
Docs: {URL}

Execute: Follow prompts/00_coordinator.md
```

### For Multiple Providers (Parallel)

```
Launch Phase 1 for all providers in parallel.
Wait for completion.
Launch Phase 2 for all providers in parallel.
...
```

---

## Provider Registry

### Aggregators (4)

| Provider | Docs URL | Priority | Status |
|----------|----------|----------|--------|
| cryptocompare | https://min-api.cryptocompare.com/documentation | HIGH | Not started |
| defillama | https://defillama.com/docs/api | HIGH | Not started |
| ib (Interactive Brokers) | https://interactivebrokers.github.io/cpwebapi/ | MEDIUM | Not started |
| yahoo | https://finance.yahoo.com/ | MEDIUM | Not started |

### Forex (3)

| Provider | Docs URL | Priority | Status |
|----------|----------|----------|--------|
| alphavantage | https://www.alphavantage.co/documentation/ | MEDIUM | Not started |
| dukascopy | https://www.dukascopy.com/trading-tools/widgets/api/ | LOW | Not started |
| oanda | https://developer.oanda.com/rest-live-v20/introduction/ | HIGH | Not started |

### Stocks - US (5)

| Provider | Docs URL | Priority | Status |
|----------|----------|----------|--------|
| polygon | https://polygon.io/docs/stocks | CRITICAL | Not started |
| alpaca | https://docs.alpaca.markets/docs | HIGH | Not started |
| finnhub | https://finnhub.io/docs/api | HIGH | Not started |
| tiingo | https://www.tiingo.com/documentation/general/overview | MEDIUM | Not started |
| twelvedata | https://twelvedata.com/docs | MEDIUM | Not started |

### Stocks - China (1)

| Provider | Docs URL | Priority | Status |
|----------|----------|----------|--------|
| futu | https://openapi.futunn.com/futu-api-doc/ | LOW | Not started |

### Stocks - India (5)

| Provider | Docs URL | Priority | Status |
|----------|----------|----------|--------|
| upstox | https://upstox.com/developer/api-documentation | MEDIUM | Not started |
| angel_one | https://smartapi.angelbroking.com/docs | MEDIUM | Not started |
| zerodha | https://kite.trade/docs/connect/v3/ | HIGH | Not started |
| dhan | https://dhanhq.co/docs/ | MEDIUM | Not started |
| fyers | https://myapi.fyers.in/docsv3 | MEDIUM | Not started |

### Stocks - Japan (1)

| Provider | Docs URL | Priority | Status |
|----------|----------|----------|--------|
| jquants | https://jpx.gitbook.io/j-quants-en/ | MEDIUM | Not started |

### Stocks - Korea (1)

| Provider | Docs URL | Priority | Status |
|----------|----------|----------|--------|
| krx | https://global.krx.co.kr/ | LOW | Not started |

### Stocks - Russia (2)

| Provider | Docs URL | Priority | Status |
|----------|----------|----------|--------|
| moex | https://www.moex.com/a2193 | LOW | Not started |
| tinkoff | https://tinkoff.github.io/investAPI/ | LOW | Not started |

### Data Feeds (4)

| Provider | Docs URL | Priority | Status |
|----------|----------|----------|--------|
| coinglass | https://coinglass-api.com/ | HIGH | Not started |
| fred | https://fred.stlouisfed.org/docs/api/ | MEDIUM | Not started |
| bitquery | https://docs.bitquery.io/ | MEDIUM | Not started |
| whale_alert | https://docs.whale-alert.io/ | LOW | Not started |

---

## Execution Strategies

### Strategy 1: Sequential (Safe)

Execute one provider at a time, all phases.

**Pros:** Simple, easy to debug
**Cons:** Slow (26 × 6 hours = 156 hours)

**Use when:** Testing new prompts, first provider

```
1. polygon (Phase 1-4)
2. oanda (Phase 1-4)
3. finnhub (Phase 1-4)
...
```

### Strategy 2: Batch by Phase (Recommended)

Execute same phase for multiple providers in parallel.

**Pros:** Faster (4 weeks vs 6 months), efficient
**Cons:** Need to track multiple agents

**Use when:** Production execution

```
Week 1:
  Phase 1 (Research) for all 26 providers in parallel

Week 2:
  Phase 2 (Implement) for all 26 providers in parallel

Week 3:
  Phase 3 (Tests) for all 26 providers in parallel

Week 4:
  Phase 4 (Debug) for all 26 providers (iterative)
```

### Strategy 3: Category Batches

Execute by category (all phases for category before next).

**Pros:** Focused, can learn patterns within category
**Cons:** Moderately slow

**Use when:** Want to focus on specific category first

```
Batch 1: Aggregators (4 providers)
  Week 1: All phases for cryptocompare, defillama, ib, yahoo

Batch 2: Forex (3 providers)
  Week 2: All phases for alphavantage, dukascopy, oanda

Batch 3: Stocks US (5 providers)
  Week 3-4: All phases for polygon, alpaca, finnhub, tiingo, twelvedata

...
```

### Strategy 4: Priority-Based

Execute high-priority providers first.

**Pros:** Get most valuable connectors quickly
**Cons:** May leave gaps

**Use when:** Need quick wins

```
Week 1: CRITICAL + HIGH priority (9 providers)
  - polygon, oanda, finnhub, alpaca, zerodha
  - cryptocompare, defillama, coinglass

Week 2: MEDIUM priority (11 providers)

Week 3: LOW priority (6 providers)
```

---

## Recommended Execution Plan

**4-Week Sprint:**

### Week 1: High-Value Providers (Phase 1-4)

Execute full pipeline for top 5 providers:

| Provider | Category | Why |
|----------|----------|-----|
| polygon | stocks/us | Best US stock data, REST+WS |
| oanda | forex | Only major forex broker with API |
| finnhub | stocks/us | Multi-asset, good free tier |
| coinglass | data_feeds | Derivatives analytics |
| defillama | aggregators | DeFi data |

**Commands:**
```bash
# Launch all Phase 1 in parallel
Task 1: Research polygon (stocks/us)
Task 2: Research oanda (forex)
Task 3: Research finnhub (stocks/us)
Task 4: Research coinglass (data_feeds)
Task 5: Research defillama (aggregators)

# After research complete, Phase 2-4 in parallel
...
```

### Week 2: Regional Stocks + Aggregators (10 providers)

| Provider | Category | Region |
|----------|----------|--------|
| alpaca | stocks/us | US broker |
| tiingo | stocks/us | US data |
| twelvedata | stocks/us | Multi-asset |
| zerodha | stocks/india | India #1 broker |
| upstox | stocks/india | India free |
| angel_one | stocks/india | India free |
| dhan | stocks/india | India free |
| fyers | stocks/india | India F&O |
| cryptocompare | aggregators | Crypto agg |
| alphavantage | forex | Multi-asset |

### Week 3: Remaining Regional + Forex (7 providers)

| Provider | Category | Region |
|----------|----------|--------|
| jquants | stocks/japan | Japan official |
| futu | stocks/china | China/HK |
| krx | stocks/korea | Korea |
| moex | stocks/russia | Russia |
| tinkoff | stocks/russia | Russia broker |
| dukascopy | forex | Forex specialist |
| ib | aggregators | Multi-asset broker |

### Week 4: Data Feeds + Cleanup (4 providers + fixes)

| Provider | Category |
|----------|----------|
| fred | data_feeds (macro) |
| bitquery | data_feeds (on-chain) |
| whale_alert | data_feeds (wallet) |
| yahoo | aggregators |

Plus: Fix any failing providers from previous weeks.

---

## Parallel Execution Template

### Phase 1: Research (Launch all in parallel)

```typescript
// Example: Launch 5 providers in parallel
const providers = [
  { name: 'polygon', category: 'stocks/us', docs: 'https://polygon.io/docs' },
  { name: 'oanda', category: 'forex', docs: 'https://developer.oanda.com' },
  { name: 'finnhub', category: 'stocks/us', docs: 'https://finnhub.io/docs' },
  { name: 'coinglass', category: 'data_feeds', docs: 'https://coinglass-api.com' },
  { name: 'defillama', category: 'aggregators', docs: 'https://defillama.com/docs/api' },
];

// Launch all research agents
for (const provider of providers) {
  launchAgent('research-agent', {
    prompt: 'prompts/01_research.md',
    variables: {
      PROVIDER: provider.name,
      CATEGORY: provider.category,
      DOCS_URL: provider.docs,
    },
  });
}

// Wait for all to complete
// Verify all 8 research files created for each
```

**In practice (Claude Code):**

Single message with multiple Task tool calls:
```
Launch 5 research agents in parallel:
1. Research polygon (stocks/us) - https://polygon.io/docs
2. Research oanda (forex) - https://developer.oanda.com
3. Research finnhub (stocks/us) - https://finnhub.io/docs
4. Research coinglass (data_feeds) - https://coinglass-api.com
5. Research defillama (aggregators) - https://defillama.com/docs/api

Follow: prompts/01_research.md for each
```

---

## Progress Tracking

Update registry table after each provider completes Phase 4:

```markdown
| Provider | Status | Tests Passing | Real Data | Notes |
|----------|--------|---------------|-----------|-------|
| polygon | ✅ Complete | 9/9 | ✅ | Stock prices $150+ |
| oanda | 🏗️ Phase 3 | - | - | WebSocket in progress |
| finnhub | ❌ Blocked | - | - | API key issue |
```

**Status codes:**
- ✅ Complete - All phases done, tests passing
- 🏗️ Phase X - Currently in phase X
- ⏸️ Paused - Blocked, waiting
- ❌ Failed - Cannot complete (API down, etc.)
- 📝 Not started

---

## Quality Checklist (Per Provider)

Before marking as ✅ Complete:

### Research (Phase 1)
- [ ] 8/8 research files created
- [ ] No TODO placeholders
- [ ] Exact JSON examples (not invented)
- [ ] All endpoints documented
- [ ] Tiers/pricing clear

### Implementation (Phase 2)
- [ ] 5-6 Rust files created
- [ ] Compiles with 0 errors
- [ ] ExchangeId added
- [ ] Module exported
- [ ] UnsupportedOperation for trading (unless broker)

### Tests (Phase 3)
- [ ] Integration test file
- [ ] WebSocket test file (if WS available)
- [ ] Tests compile
- [ ] Graceful error handling

### Debug (Phase 4)
- [ ] At least 1 test returns REAL data
- [ ] Trading returns UnsupportedOperation
- [ ] No panics
- [ ] Tests pass or fail gracefully

### Finalization
- [ ] Committed to repo
- [ ] Registry updated
- [ ] README.md created (if needed)

---

## Common Issues & Solutions

### Issue: Research agent invents data

**Solution:** Re-run with emphasis:
```
Use ONLY official docs. No invented examples.
If docs don't show example, say "Not documented".
```

### Issue: Too many providers fail Phase 4

**Solution:** Some APIs may be down/changed. Focus on working ones.
- Target: 80% success rate (21/26 providers)
- Acceptable: 15% deprecated/broken (4/26)
- Investigate: 5% fixable with more effort (1/26)

### Issue: Rate limits during testing

**Solution:**
- Add delays between tests: `tokio::sleep(Duration::from_secs(1))`
- Use paid tiers for high-priority providers
- Tests should handle 429 gracefully

### Issue: Parallel execution overwhelming

**Solution:** Reduce batch size
- Instead of 26 parallel, do batches of 5
- Week 1: 5 providers
- Week 2: 5 providers
- Week 3: 5 providers
- Week 4: 5 providers
- Week 5: 5 providers
- Week 6: 1 provider + cleanup

---

## Metrics Dashboard

Track progress across all providers:

```
Total Providers: 26
Completed: 0 (0%)
In Progress: 0 (0%)
Not Started: 26 (100%)

By Category:
- Aggregators: 0/4 (0%)
- Forex: 0/3 (0%)
- Stocks: 0/15 (0%)
- Data Feeds: 0/4 (0%)

By Priority:
- CRITICAL: 0/1 (0%)
- HIGH: 0/8 (0%)
- MEDIUM: 0/11 (0%)
- LOW: 0/6 (0%)

Estimated Completion: Week X of 4-week plan
```

Update after each provider completion.

---

## Next Steps

1. **Choose execution strategy** (Recommended: Week-by-week priority-based)

2. **Start with pilot** (1 provider to test prompts):
   - Polygon.io (well-documented, REST+WS, free tier)
   - Execute Phase 1-4
   - Validate prompts work
   - Adjust if needed

3. **Scale to batch** (5 providers):
   - Polygon, OANDA, Finnhub, Coinglass, DefiLlama
   - Execute Phase 1 in parallel
   - Review all research
   - Proceed to Phase 2-4

4. **Execute full plan** (26 providers):
   - Follow 4-week sprint plan
   - Track progress in registry
   - Update metrics dashboard
   - Handle failures gracefully

5. **Finalize**:
   - Commit all working connectors
   - Document learnings
   - Update ../exchanges/GUIDE.md
   - Create production deployment guide

---

## Success Definition

**Minimum Viable:**
- 15/26 providers working (60%)
- All CRITICAL + HIGH priority working
- Real data verified for each

**Target:**
- 21/26 providers working (80%)
- All categories represented
- Documentation complete

**Stretch Goal:**
- 26/26 providers working (100%)
- All with WebSocket support (where available)
- Full test coverage

---

## After Completion

**You will have:**
- 26 data provider connectors
- 4 categories covered (aggregators, forex, stocks, feeds)
- 130+ research documents (26 × 5)
- 130+ implementation files (26 × 5)
- 52+ test files (26 × 2)
- Production-ready data infrastructure

**Use cases:**
- Multi-asset trading systems
- Market data aggregation
- Research platforms
- Real-time monitoring
- Historical backtesting

---

## Manager Commands

### Launch Single Provider
```
Execute Agent Carousel for {PROVIDER}
Category: {CATEGORY}
Docs: {DOCS_URL}
Follow: prompts/00_coordinator.md
```

### Launch Batch (Phase 1)
```
Launch Research Phase for batch:
1. Provider A (category) - docs URL
2. Provider B (category) - docs URL
3. Provider C (category) - docs URL
...

Execute in parallel.
Follow: prompts/01_research.md for each
```

### Check Progress
```
List status of all 26 providers:
- Completed count
- In progress
- Blocked/Failed
- Update registry table
```

### Debug Failures
```
Identify failing providers:
- List errors
- Common patterns
- Recommendations
```

---

## End of Manager Guide

Use this guide to orchestrate the full execution of 26 data provider connectors.

**Ready to start?** Begin with pilot provider (Polygon.io).
