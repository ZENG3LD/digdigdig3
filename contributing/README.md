# Contributing: Connector Development

This directory contains everything needed to add new connectors to digdigdig3 using the Agent Carousel pattern.

## What is the Agent Carousel?

The Agent Carousel is an automated pipeline for building production-ready connectors through a sequence of specialized agents. Each connector goes through 6 phases: Research, Implement, Test, Debug, Integration Test, Integration Debug. Each phase is handled by a dedicated agent using a structured prompt from the `prompts/` directory.

The pattern ensures every connector ships with:
- A `research/` folder containing API documentation pulled from official sources
- Full REST and WebSocket implementations
- Unit tests and integration tests with real data validation

## Two Pipelines

### Exchange Connectors (CEX and DEX)

For crypto exchanges: Binance, OKX, Bybit, Gate.io, and others.

```
contributing/exchanges/
├── CAROUSEL.md    — full pipeline with phase-by-phase instructions and exchange registry
├── GUIDE.md       — lessons learned and quick-reference checklist for agents
└── prompts/       — agent prompt files for each phase
    ├── 00_coordinator.md    — coordinator instructions (read this first)
    ├── 01_research.md       — Phase 1: Research agent
    ├── 02_implement.md      — Phase 2: Implementation agent
    ├── 03_test.md           — Phase 3: Test agent
    ├── 04_debug.md          — Phase 4: Debug agent (loop)
    ├── 05_integration_test.md   — Phase 5: Integration test agent
    └── 06_integration_debug.md  — Phase 6: Integration debug agent (loop)
```

Start here: `exchanges/CAROUSEL.md`

### Data Provider Connectors

For stocks, forex, aggregators, and specialized data feeds: Polygon.io, OANDA, Finnhub, CoinGlass, and others.

```
contributing/data_providers/
├── CAROUSEL.md    — full pipeline adapted for data providers (8 research files, no trading)
├── MANAGER.md     — coordination guide for running 26 providers in parallel
└── prompts/       — agent prompt files for each phase
    ├── 00_coordinator.md    — coordinator instructions
    ├── 01_research.md       — Phase 1: Research (8 files vs 6 for exchanges)
    ├── 02_implement.md      — Phase 2: Implementation
    ├── 03_test.md           — Phase 3: Tests
    ├── 04_debug.md          — Phase 4: Debug (loop)
    ├── 05_integration_test.md   — Phase 5: Integration tests
    └── 06_integration_debug.md  — Phase 6: Integration debug (loop)
```

Start here: `data_providers/CAROUSEL.md`

## Pipeline Phases

Both pipelines follow the same 6-phase structure:

| Phase | Agent | Output |
|-------|-------|--------|
| 1: Research | research-agent | `research/` folder with API docs |
| 2: Implement | rust-implementer | `endpoints.rs`, `auth.rs`, `parser.rs`, `connector.rs`, `websocket.rs` |
| 3: Test | rust-implementer | `tests/{name}_integration.rs`, `tests/{name}_websocket.rs` |
| 4: Debug | rust-implementer (loop) | All unit tests passing with real data |
| 5: Integration Test | rust-implementer | `tests/{name}_live.rs` |
| 6: Integration Debug | rust-implementer (loop) | All live tests passing |

## Submitting a New Connector

Every PR that adds a connector must include a `research/` folder alongside the implementation. This is a hard requirement — the research documents are how reviewers verify the implementation matches the real API.

Exchange connectors: `src/exchanges/{name}/research/` (6 files)
Data providers: `src/{category}/{name}/research/` (8 files)

Follow the prompts in the relevant `prompts/` directory. The reference implementation for all connectors is `src/exchanges/kucoin/`.
