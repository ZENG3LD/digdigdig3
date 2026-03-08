# Contributing to digdigdig3

## Where Help Is Needed Most

### 1. Intelligence feeds are unverified

All 88 connectors in `src/intelligence_feeds/` were generated but **not tested against real APIs**. If you have API keys for any of these services, help us verify and fix the implementations.

### 2. Trading modules are incomplete

Connectors that work without API keys (Binance, Bybit, OKX, Kraken, etc.) have verified `MarketData` — price, klines, orderbook, ticker all work. But `Trading`, `Account`, and `Positions` trait implementations are **untested stubs** in most of them. If you have exchange accounts with API keys, help us get trading working.

### 3. API-key-required connectors need data verification

Connectors marked `requires_api_key_for_data: true` in the registry have not been verified at all — neither market data nor trading. If you have keys for Coinbase, Alpaca, OANDA, Interactive Brokers, or any Indian/Korean/Japanese brokers, your help is especially valuable.

### Priority

| Priority | What | Where |
|----------|------|-------|
| High | Verify & fix intelligence feed connectors | `src/intelligence_feeds/` |
| High | Test trading on no-API-key exchanges | `src/exchanges/` |
| Medium | Verify API-key-required data connectors | `src/stocks/`, `src/forex/`, `src/aggregators/` |
| Low | Add new exchanges or feeds | `src/exchanges/`, `src/intelligence_feeds/` |

## Adding a New Connector

Every new connector follows the **Agent Carousel** pipeline — a 6-phase process that ensures consistent quality across all connectors. See [`contributing/`](contributing/) for the full pipeline documentation and prompts.

### Requirements for a PR

1. **`research/` folder** — API documentation pulled from official sources (6 files for exchanges, 8 for data providers). This is mandatory — reviewers use it to verify the implementation.

2. **Standard module structure:**
   ```
   src/exchanges/<name>/
   ├── mod.rs          # public exports
   ├── endpoints.rs    # URL constants, endpoint enum, symbol formatting
   ├── auth.rs         # request signing
   ├── parser.rs       # JSON → unified types
   ├── connector.rs    # trait implementations
   ├── websocket.rs    # WebSocket (if supported)
   └── research/       # API documentation (6-8 .md files)
   ```

3. **Trait implementations** — at minimum `ExchangeIdentity` + `MarketData`. Add `Trading`, `Account`, `Positions` if the exchange supports them.

4. **Registry entry** — add your connector to `ConnectorRegistry` in `src/connector_manager/registry.rs` and to `ConnectorFactory` in `src/connector_manager/factory.rs`.

5. **Tests pass** — `cargo check` and `cargo clippy -- -D warnings` must pass with zero errors.

### Reference Implementation

`src/exchanges/kucoin/` — follow this as the canonical example.

### Intelligence Feeds

Intelligence feeds follow a similar pattern but live in `src/intelligence_feeds/<category>/<name>/`. Add entries to `FeedRegistry` and `FeedFactory` in `src/intelligence_feeds/feed_manager/`.

## Code Style

- `cargo clippy -- -D warnings` must pass — no exceptions
- Use `ExchangeResult<T>` for all fallible operations
- Prefer `&str` over `String` in function parameters
- Follow existing patterns — consistency matters more than cleverness

## License

By contributing, you agree that your contributions will be dual-licensed under MIT and Apache 2.0.
