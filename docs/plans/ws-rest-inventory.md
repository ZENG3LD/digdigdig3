# WS + REST Inventory — digdigdig3

Generated: 2026-05-17. Based on grep `impl WebSocketConnector for` + `impl WsProtocol for` across `src/`
and `ConnectorFactory::create_public` match arms in `src/connector_manager/factory.rs`.

---

## WS Universe (35 impls total)

| ExchangeId | File | WS Class | Arch |
|---|---|---|---|
| Binance | l3/open/crypto/cex/binance/websocket.rs | BinanceWebSocket | UniversalWsTransport (WsProtocol) |
| Binance | l3/open/crypto/cex/binance/protocol.rs | BinanceProtocol | WsProtocol |
| Bybit | l3/open/crypto/cex/bybit/websocket.rs | BybitWebSocket | UniversalWsTransport (WsProtocol) |
| Bybit | l3/open/crypto/cex/bybit/protocol.rs | BybitProtocol | WsProtocol |
| OKX | l3/open/crypto/cex/okx/websocket.rs | OkxWebSocket | UniversalWsTransport (WsProtocol) |
| OKX | l3/open/crypto/cex/okx/protocol.rs | OkxProtocol | WsProtocol |
| KuCoin | l3/open/crypto/cex/kucoin/websocket.rs | KuCoinWebSocket | UniversalWsTransport (WsProtocol) |
| KuCoin | l3/open/crypto/cex/kucoin/protocol.rs | KuCoinProtocol | WsProtocol |
| GateIO | l3/open/crypto/cex/gateio/websocket.rs | GateioWebSocket | UniversalWsTransport (WsProtocol) |
| GateIO | l3/open/crypto/cex/gateio/protocol.rs | GateIoProtocol | WsProtocol |
| HTX | l3/open/crypto/cex/htx/websocket.rs | HtxWebSocket | UniversalWsTransport (WsProtocol) |
| HTX | l3/open/crypto/cex/htx/protocol.rs | HtxProtocol | WsProtocol |
| Bitget | l3/open/crypto/cex/bitget/websocket.rs | BitgetWebSocket | UniversalWsTransport (WsProtocol) |
| Bitget | l3/open/crypto/cex/bitget/protocol.rs | BitgetProtocol | WsProtocol |
| Deribit | l3/open/crypto/cex/deribit/websocket.rs | DeribitWebSocket | UniversalWsTransport (WsProtocol) |
| Deribit | l3/open/crypto/cex/deribit/protocol.rs | DeribitProtocol | WsProtocol |
| MEXC | l3/open/crypto/cex/mexc/websocket.rs | MexcWebSocket | UniversalWsTransport (WsProtocol) |
| MEXC | l3/open/crypto/cex/mexc/protocol.rs | MexcProtocol | WsProtocol |
| HyperLiquid | l3/open/crypto/cex/hyperliquid/websocket.rs | HyperliquidWebSocket | UniversalWsTransport (WsProtocol) |
| HyperLiquid | l3/open/crypto/cex/hyperliquid/protocol.rs | HyperliquidProtocol | WsProtocol |
| BingX | l3/open/crypto/cex/bingx/websocket.rs | BingxWebSocket | legacy bespoke |
| Bitfinex | l3/open/crypto/cex/bitfinex/websocket.rs | BitfinexWebSocket | legacy bespoke |
| Bitstamp | l3/open/crypto/cex/bitstamp/websocket.rs | BitstampWebSocket | legacy bespoke |
| Coinbase | l3/open/crypto/cex/coinbase/websocket.rs | CoinbaseWebSocket | legacy bespoke |
| CryptoCom | l3/open/crypto/cex/crypto_com/websocket.rs | CryptoComWebSocket | legacy bespoke |
| Gemini | l3/open/crypto/cex/gemini/websocket.rs | GeminiWebSocket | legacy bespoke |
| Kraken | l3/open/crypto/cex/kraken/websocket.rs | KrakenWebSocket | legacy bespoke |
| Upbit | l3/open/crypto/cex/upbit/websocket.rs | UpbitWebSocket | legacy bespoke |
| Dydx | l3/open/crypto/dex/dydx/websocket.rs | DydxWebSocket | legacy bespoke |
| Lighter | l3/open/crypto/dex/lighter/websocket.rs | LighterWebSocket | legacy bespoke |
| YahooFinance | l1/free/yahoo/websocket.rs | YahooFinanceWebSocket | legacy bespoke |
| Tiingo | l1/paid/tiingo/websocket.rs | TiingoWebSocket | legacy bespoke |
| CryptoCompare | l2/paid/cryptocompare/websocket.rs | CryptoCompareWebSocket | legacy bespoke |
| MOEX | l2/free/moex/websocket.rs | MoexWebSocket | legacy bespoke |
| Alpaca | l3/gated/stocks/us/alpaca/websocket.rs | AlpacaWebSocket | legacy bespoke |

**Total WebSocketConnector impls: 35** (20 unique exchange connectors + protocol helpers counted separately above)

**Distinct WS-capable exchanges: 25** (via factory: 22 CEX/DEX + YahooFinance + CryptoCompare; MOEX/Tiingo/Alpaca require direct construction)

---

## REST Universe (ExchangeId variants handled by create_public)

| ExchangeId | Path | create_public result | Traits |
|---|---|---|---|
| Binance | l3/open/crypto/cex/binance/ | OK | CoreConnector, MarketData |
| Bybit | l3/open/crypto/cex/bybit/ | OK | CoreConnector, MarketData |
| OKX | l3/open/crypto/cex/okx/ | OK | CoreConnector, MarketData |
| KuCoin | l3/open/crypto/cex/kucoin/ | OK | CoreConnector, MarketData |
| Kraken | l3/open/crypto/cex/kraken/ | OK | CoreConnector, MarketData |
| GateIO | l3/open/crypto/cex/gateio/ | OK | CoreConnector, MarketData |
| Bitfinex | l3/open/crypto/cex/bitfinex/ | OK | CoreConnector, MarketData |
| MEXC | l3/open/crypto/cex/mexc/ | OK | CoreConnector, MarketData |
| HTX | l3/open/crypto/cex/htx/ | OK | CoreConnector, MarketData |
| BingX | l3/open/crypto/cex/bingx/ | OK | CoreConnector, MarketData |
| CryptoCom | l3/open/crypto/cex/crypto_com/ | OK | CoreConnector, MarketData |
| Upbit | l3/open/crypto/cex/upbit/ | OK | CoreConnector, MarketData |
| Deribit | l3/open/crypto/cex/deribit/ | OK | CoreConnector, MarketData |
| HyperLiquid | l3/open/crypto/cex/hyperliquid/ | OK | CoreConnector, MarketData |
| Dydx | l3/open/crypto/dex/dydx/ | OK | CoreConnector, MarketData |
| Bitget | l3/open/crypto/cex/bitget/ | OK | CoreConnector, MarketData |
| Bitstamp | l3/open/crypto/cex/bitstamp/ | OK | CoreConnector, MarketData |
| Coinbase | l3/open/crypto/cex/coinbase/ | OK | CoreConnector, MarketData |
| Gemini | l3/open/crypto/cex/gemini/ | OK | CoreConnector, MarketData |
| Lighter | l3/open/crypto/dex/lighter/ | OK | CoreConnector, MarketData |
| Alpaca | l3/gated/stocks/us/alpaca/ | OK (crypto_only) | CoreConnector, MarketData |
| YahooFinance | l1/free/yahoo/ | OK | CoreConnector, MarketData |
| Polymarket | l3/open/prediction/polymarket/ | OK | CoreConnector |
| Twelvedata | l1/paid/twelvedata/ | OK (demo mode) | CoreConnector, MarketData |
| CryptoCompare | l2/paid/cryptocompare/ | OK | CoreConnector, MarketData |
| Dukascopy | l3/gated/forex/dukascopy/ | OK | CoreConnector, MarketData |
| Krx | l1/free/krx/ | OK | CoreConnector, MarketData |
| Moex | l2/free/moex/ | OK | CoreConnector, MarketData |
| Polygon | l2/paid/polygon/ | FAIL Auth (needs key) | — |
| Finnhub | l1/free/finnhub/ | FAIL Auth (needs key) | — |
| Tiingo | l1/paid/tiingo/ | FAIL Auth (needs key) | — |
| AlphaVantage | l1/paid/alphavantage/ | FAIL Auth (needs key) | — |
| AngelOne | l3/gated/stocks/india/angelone/ | FAIL Auth | — |
| Zerodha | l3/gated/stocks/india/zerodha/ | FAIL Auth | — |
| Upstox | l3/gated/stocks/india/upstox/ | FAIL Auth | — |
| Dhan | l3/gated/stocks/india/dhan/ | FAIL Auth | — |
| Fyers | l3/gated/stocks/india/fyers/ | FAIL Auth | — |
| Oanda | l3/gated/forex/oanda/ | FAIL Auth | — |
| JQuants | l3/gated/stocks/japan/jquants/ | FAIL Auth | — |
| Tinkoff | — | FAIL Auth | — |
| Ib | — | FAIL Auth | — |
| Futu | — | FAIL Auth | — |
| Bls | — | FAIL Auth | — |
| Coinglass | — | FAIL Auth | — |
| DefiLlama | — | FAIL Unsupported (moved to dig2feed) | — |
| WhaleAlert | — | FAIL Unsupported (moved to dig2onchain-data) | — |
| Fred | — | FAIL Auth (removed) | — |
| Bitquery | — | FAIL Unsupported (moved to dig2onchain-data) | — |

**REST total variants: 49** (28 return Ok from create_public; 21 return Err)

---

## MOEX Status

- **REST**: YES — `MoexConnector::new_public()` — `create_public(Moex)` returns `Ok`
- **WS**: YES impl exists (`MoexWebSocket` in `l2/free/moex/websocket.rs`, line 1020)
- **WS via factory**: NO — `create_websocket(Moex)` returns `Err(UnsupportedOperation)` with message "requires MoexAuth — construct directly with MoexWebSocket::new_public()"
- **Fix**: MOEX WS CAN be tested directly via `MoexWebSocket::new_public()` — the factory restriction is wrong/overly cautious since `new_public()` exists
