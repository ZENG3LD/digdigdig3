# Contributing to digdigdig3

## Where Help Is Needed Most

### 1. Trading modules are incomplete

L3/open CEX connectors (Binance, Bybit, OKX, Kraken, KuCoin, etc.) in `src/l3/open/crypto/cex/` have verified `MarketData` — price, klines, orderbook, ticker all work. But `Trading`, `Account`, and `Positions` trait implementations are **untested stubs** in most of them. If you have exchange accounts with API keys, help us get trading working.

### 2. API-key-required connectors need data verification

Connectors in `src/l3/gated/` (stocks, forex, multi-asset brokers) and paid data providers in `src/l1/paid/` and `src/l2/paid/` have not been verified — neither market data nor trading. If you have keys for OANDA, Interactive Brokers, Alpaca, Tinkoff, Dhan, Zerodha, or similar, your help is especially valuable.

### Priority

| Priority | What | Where |
|----------|------|-------|
| High | Test trading on CEX connectors | `src/l3/open/crypto/cex/` |
| High | Verify gated broker connectors | `src/l3/gated/` |
| Medium | Verify paid data providers | `src/l1/paid/`, `src/l2/paid/` |
| Low | Add new connectors | see Adding a New Connector below |

## Adding a New Connector

Every new connector follows the **Agent Carousel** pipeline — a 6-phase process that ensures consistent quality across all connectors. See [`contributing/`](contributing/) for the full pipeline documentation and prompts.

### Requirements for a PR

1. **`research/` folder** — API documentation pulled from official sources (6 files for exchanges, 8 for data providers). This is mandatory — reviewers use it to verify the implementation.

2. **Standard module structure:**
   ```
   src/l3/open/crypto/cex/<name>/
   ├── mod.rs          # public exports
   ├── endpoints.rs    # URL constants, endpoint enum, symbol formatting
   ├── auth.rs         # request signing
   ├── parser.rs       # JSON → unified types
   ├── connector.rs    # trait implementations
   ├── websocket.rs    # WebSocket (if supported)
   └── research/       # API documentation (6-8 .md files)
   ```

   Place new connectors in the appropriate subtree:
   - `src/l1/free/` or `src/l1/paid/` — data-only feeds (no orderbook)
   - `src/l2/free/` or `src/l2/paid/` — orderbook data (no execution)
   - `src/l3/open/crypto/cex/` — crypto CEX (no registration needed for market data)
   - `src/l3/open/crypto/dex/` — decentralized exchanges
   - `src/l3/open/prediction/` — prediction markets
   - `src/l3/gated/stocks/`, `src/l3/gated/forex/`, `src/l3/gated/multi/` — brokers requiring account/KYC

3. **Trait implementations** — at minimum `ExchangeIdentity` + `MarketData`. Add `Trading`, `Account`, `Positions` if the exchange supports them.

4. **Registry entry** — add your connector to `ConnectorRegistry` in `src/connector_manager/registry.rs` and to `ConnectorFactory` in `src/connector_manager/factory.rs`.

5. **Tests pass** — `cargo check` and `cargo clippy -- -D warnings` must pass with zero errors.

### Reference Implementation

`src/l3/open/crypto/cex/kucoin/` — follow this as the canonical example.

## Code Style

- `cargo clippy -- -D warnings` must pass — no exceptions
- Use `ExchangeResult<T>` for all fallible operations
- Prefer `&str` over `String` in function parameters
- Follow existing patterns — consistency matters more than cleverness

## License

By contributing, you agree that your contributions will be dual-licensed under MIT and Apache 2.0.
