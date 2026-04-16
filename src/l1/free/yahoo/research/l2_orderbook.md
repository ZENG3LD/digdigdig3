# Yahoo Finance — L2 Orderbook Capabilities

## Summary

Yahoo Finance does **NOT** provide L2 orderbook data. There is no bid/ask ladder, no depth-of-market, and no order book endpoint — neither in the (long-dead) official API nor in any of the current unofficial/reverse-engineered endpoints. Yahoo Finance is a financial news and data aggregator oriented toward retail investors; it was never designed as a trading infrastructure provider.

**Verdict: L2 orderbook is UNSUPPORTED. Return `UnsupportedOperation` for any orderbook trait.**

---

## Details

### What Yahoo Finance Actually Provides

Yahoo Finance exposes the following market data categories (all unofficial, reverse-engineered):

| Data Type | Available | Endpoint |
|-----------|-----------|----------|
| Current price (last trade) | Yes | `/v8/finance/chart/{symbol}` |
| OHLCV bars (intraday + daily) | Yes | `/v8/finance/chart/{symbol}` |
| Day high/low/volume | Yes | `/v8/finance/chart/{symbol}` |
| Pre/post market price | Yes | `/v8/finance/chart/{symbol}` |
| Options chain (strikes/expirations) | Yes | `/v7/finance/options/{symbol}` |
| Fundamental data (income, balance sheet) | Yes | `/v10/finance/quoteSummary/{symbol}` |
| Analyst ratings / institutional ownership | Yes | `/v10/finance/quoteSummary/{symbol}` |
| Symbol search | Yes | `/v1/finance/search` |
| **Bid price** | **No** | — |
| **Ask price** | **No** | — |
| **Bid/ask size** | **No** | — |
| **Order book (any depth)** | **No** | — |
| **Level 2 quotes** | **No** | — |

### Why L2 Is Not Available

1. **Not a trading venue.** Yahoo Finance aggregates and displays market data sourced from exchanges and data vendors. It does not itself provide exchange-level market microstructure data (order books, trade-by-trade feeds, etc.).

2. **Designed for retail consumers.** The product targets news readers and passive investors, not algorithmic traders or market makers who need depth-of-book.

3. **No official API since 2017.** The official Yahoo Finance API was shut down in 2017. All current access is via reverse-engineered browser endpoints. None of these endpoints expose any bid/ask or orderbook structure.

4. **Even the WebSocket stream has no orderbook.** Yahoo Finance does run a WebSocket feed (`wss://streamer.finance.yahoo.com/`) used by its website for live price updates, but this stream carries only last-trade price ticks and quote summaries — not bid/ask depth.

5. **Explicitly confirmed in existing research.** From `API_REALITY_2026.md`, section 7 "Current Limitations":
   > "No bid/ask prices — Yahoo Finance doesn't provide order book data"

### What L2 Sources Should Be Used Instead

For actual L2 orderbook data, use exchange-native connectors or dedicated market data providers:

| Provider | L2 Support | Notes |
|----------|-----------|-------|
| Binance (V4/V5 connector) | Full L2 | `@depth` WebSocket stream, 5/10/20 levels |
| KuCoin (V5 connector) | Full L2 | `/market/orderbook/level2` REST + WebSocket |
| Phemex | Full L2 | Order book WebSocket |
| Finnhub | Partial | Best bid/ask only (no full depth on free tier) |
| Polygon.io | Full L2 (stocks) | NBBO + full depth for US equities (paid) |

### Connector Implementation Note

The Yahoo Finance V5 connector must return `ExchangeError::UnsupportedOperation` for:
- `get_orderbook()`
- Any L2/depth subscription methods
- Bid/ask spread queries

This is already the expected behavior per the V5 connector architecture for data providers that do not support trading or market microstructure data.

---

## Sources

- Internal research: `api_overview.md` — confirms no official API since 2017
- Internal research: `endpoints_full.md` — no orderbook endpoint in any endpoint category
- Internal research: `API_REALITY_2026.md` (2026-01-26) — explicitly states "No bid/ask prices — Yahoo Finance doesn't provide order book data" (Section 7, Current Limitations, item 1)
- Yahoo Finance website: https://finance.yahoo.com/ — retail-oriented product, no depth-of-market UI
- Community library yfinance: https://github.com/ranaroussi/yfinance — no orderbook methods exist in any community wrapper
