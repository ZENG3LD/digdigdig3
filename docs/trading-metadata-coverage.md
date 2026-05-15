---
title: digdigdig3 — Trading Metadata Coverage
status: shipped
crate_version: 0.1.32
related_plan: docs/plans/non_ohlcv_l2_completion_spec.md
e2e_stand: examples/e2e_metadata.rs
---

# Что и куда вписано

Документ для агентов, которым нужно использовать или продолжать развивать публичный market-data + trading-metadata surface dig3. Карта где что лежит, какие паттерны выбраны и где пробелы. Не о том что планировалось, а что реально есть в коде сейчас.

Базовый коммит до сессии: 27abd44 (Liquidation type + Binance forceOrders REST). 53 коммита далее закрывают полное покрытие публичных торговых данных для 18 CEX + 2 DEX.

## 1. Архитектура — три пласта

### 1.1 WebSocket — унифицировано через StreamEvent enum

Все биржи эмитят события в общий тип. Файл: src/core/types/websocket.rs:224-430.

Базовые (были до сессии): Ticker, Trade(PublicTrade), OrderbookSnapshot(OrderBook), OrderbookDelta, Kline.

Trading metadata (добавлены этой сессией):
- MarkPrice { symbol, mark_price, index_price, timestamp }
- FundingRate { symbol, rate, next_funding_time, timestamp }
- Liquidation { symbol, side, price, quantity, timestamp, value }
- OpenInterestUpdate { symbol, open_interest, open_interest_value, timestamp }
- LongShortRatio { symbol, ratio_type, long_ratio, short_ratio, timestamp }
- AggTrade { symbol, aggregate_id, price, quantity, first_trade_id, last_trade_id, side, timestamp }
- CompositeIndex { symbol, price, components, timestamp }

Kline variants:
- MarkPriceKline { symbol, interval, kline }
- IndexPriceKline { symbol, interval, kline }
- PremiumIndexKline { symbol, interval, kline }

Standalone derivatives:
- IndexPrice { symbol, price, timestamp }
- HistoricalVolatility { symbol, volatility, timestamp }
- InsuranceFund { symbol, balance, timestamp }
- Basis { symbol, basis, timestamp }

Options:
- OptionGreeks { symbol, delta, gamma, vega, theta, rho, mark_iv, bid_iv, ask_iv, timestamp }

Full-coverage push (последняя волна):
- VolatilityIndex { symbol, value, timestamp }
- BlockTrade { symbol, block_id, price, quantity, side, timestamp, is_iv }
- AuctionEvent { symbol, auction_id, indicative_price, indicative_qty, state, timestamp }
- MarketWarning { symbol, warning_kind, message, timestamp }
- OrderbookL3 { symbol, side, order_id, price, quantity, action, timestamp }
- SettlementEvent { symbol, settlement_price, settlement_time, timestamp }
- RiskLimit { symbol, tier, max_leverage, max_position_value, mmr, imr, timestamp }
- PredictedFunding { symbol, predicted_rate, next_funding_time, timestamp }
- FundingSettlement { symbol, settled_rate, settlement_time, timestamp }

Private (требуют ключей): OrderUpdate, BalanceUpdate, PositionUpdate.

Multi-emit паттерн: одно WS сообщение даёт несколько StreamEvent. Пример: tickers.BTCUSDT на Bybit для linear-perp выдаёт Ticker + FundingRate + MarkPrice + OpenInterestUpdate. Parser либо возвращает Vec<StreamEvent>, либо emit через event_tx.send в цикле.

Multi-emit подтверждён e2e: Bybit (tickers), OKX (tickers), Bitget (ticker), Hyperliquid (activeAssetCtx → 5 events), Deribit (ticker.*-PERPETUAL → ticker+funding+mark), Bitfinex (status:deriv:* → mark+index+funding+OI+insurance), Crypto.com (ticker с полем oi).

### 1.2 StreamType — для подписок

Зеркальный enum для subscribe. Файл: src/core/types/websocket.rs:50-141.

Связь StreamType → topic per exchange: ищи build_stream_name / build_topic / build_channel / to_okx_channel в каждом <exchange>/websocket.rs.

### 1.3 REST — inherent методы на конкретных connector

REST методы для trading metadata НЕ в трейтах, а как inherent impl блоки на конкретном типе (BinanceConnector, OkxConnector, BybitConnector). Сознательный паттерн: market-data специфика бирж не унифицируется через generic trait. Каждая биржа имеет свой набор endpoint с разными параметрами.

Чтобы вызвать get_top_long_short_account_ratio() нужен &BinanceConnector, не &dyn MarketData. Через MarketData trait остаются только get_price, get_orderbook, get_klines, get_ticker, ping.

Соглашение об именах:
- get_<thing>(...) — REST snapshot
- get_<thing>_history(...) — historical series
- get_<thing>_kline(...) — OHLCV
- Возврат типизирован (OpenInterest, LongShortRatio, Liquidation, FundingRate). Когда тип специфичен — serde_json::Value.


## 2. Где что искать

| Вопрос | Куда смотреть |
|--------|---------------|
| Какие StreamEvent варианты доступны? | src/core/types/websocket.rs |
| Какие inherent методы у конкретного коннектора? | src/l3/open/crypto/cex/<ex>/connector.rs (grep "pub async fn get_") |
| Какие WS каналы биржа реально парсит? | src/l3/open/crypto/cex/<ex>/websocket.rs (grep parse_data_message) |
| Как биржа подписывается на StreamType X? | в том же websocket.rs — build_stream_name / build_topic / build_channel |
| Все endpoint enum variants биржи? | src/l3/open/crypto/cex/<ex>/endpoints.rs |
| Что не работает / geo-blocked? | секция 4 |
| Как проверить живой работой? | cargo run --example e2e_metadata |
| Полный спек изначального плана | docs/plans/non_ohlcv_l2_completion_spec.md |

## 3. Capability матрица (post-сессии)

OK = WS+REST верифицированы. quiet = wired но не верифицирован живым. no = биржа не отдаёт публично.

| Биржа | Trade | Klines | Book | Ticker | Liq | Funding | MarkPx | OI | L/S | Agg | Index | Greeks | Vol Idx | Block | Auction | L3 | Settle | RiskLim | Pred Fund |
|-------|-------|--------|------|--------|-----|---------|--------|----|-----|-----|-------|--------|---------|-------|---------|----|----|---------|-----------|
| binance | OK | OK+M/I/P/comp | OK | OK | OK+REST+!arr | REST+CM | WS+arr | REST+hist+CM | OK x4 | OK | OK | — | — | — | — | — | — | — | — |
| bybit | OK | OK+lt | OK | OK multi | OK | REST | OK multi | OK WS+REST | OK | — | — | — | — | — | — | — | OK delivery | OK adl+REST | — |
| okx | OK | OK+M/I | OK | OK multi | OK+REST | OK multi+REST hist | OK WS+REST | OK WS+REST | OK | — | OK | OK opt-summary | — | OK block-trades | — | — | OK est-price | OK pos-tiers | — |
| hyperliquid | OK | OK | OK | OK allMids | OK | multi+user+pred REST | multi | multi | — | — | OK oracle | — | — | — | — | — | — | — | OK predFundings |
| bitget | OK | OK+mark | no | OK multi | OK liq-order | WS+REST | WS+multi | WS+REST | — | — | — | — | — | — | — | — | — | — | — |
| htx | — | — | OK | OK | OK public.* | WS+REST+hist | OK REST | REST | OK elite | — | OK kline | — | — | — | — | — | — | — | — |
| kucoin | OK | OK | — | OK | OK | WS+REST hist | OK via instrument | REST | — | — | OK via instrument | — | — | — | — | — | — | OK REST | — |
| gateio | OK | OK+mark/index | OK | OK | OK liq+ADL | OK REST+stats | OK via candles | REST+stats | OK stats | — | — | — | — | — | — | — | — | — | — |
| deribit | OK+quote | — | OK | OK multi | — | ticker+perp | OK ticker+hist+opts | — | — | — | OK | OK ticker+opt | OK DVOL | OK block | — | — | OK exp-price | — | — |
| kraken | OK+exec | OK | OK | OK | OK exec | REST | — | OK REST | — | — | — | — | — | — | — | no только L2 | — | — | — |
| coinbase | OK+batch | OK | OK | OK | — | — | — | — | — | — | — | — | — | OK rfq | — | — | — | — | — |
| gemini | — | OK | OK | — | — | — | — | — | — | — | — | — | — | — | OK auction | — | — | — | — |
| bitfinex | OK | OK | OK | OK+deriv multi | OK liq:global | ticker+stats+book | OK status:deriv | OK status:deriv | — | — | — | — | — | — | — | OK R0 | — | — | — |
| bitstamp | OK+L3 | OK | OK+L3 | OK | — | — | — | — | — | — | — | — | — | — | — | OK detail | — | — | — |
| mexc | no | — | — | OK | no | REST(geo) | REST(geo) | REST(geo) | — | — | — | — | — | — | — | — | — | — | — |
| upbit | OK | OK | OK+realtime-only | OK | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — |
| bingx | OK | OK | — | OK | OK forceOrder | WS+REST | OK | REST | — | — | — | — | — | — | — | — | — | — | — |
| crypto_com | OK | OK | OK | OK+OI multi | no | OK WS | OK WS | via ticker | — | — | OK WS | — | — | — | — | — | OK WS | — | OK estfunding |
| dydx | OK | OK | OK | OK | OK fill flag | OK markets+hist | — | — | — | — | OK oracle | — | — | — | — | — | — | — | — |
| lighter | OK | — | OK | OK stats | OK REST | REST | — | — | — | — | — | — | — | — | — | — | — | — | — |


## 4. Что НЕ работает (honest skips)

### Geo-blocked с текущей машины (RU IP)
- Binance fstream WS (wss://fstream.binance.com) — connect OK, 0 events за 10s. Лечится VPN.
- MEXC contract API (REST + WS) — timeout. Лечится VPN.
- Coinbase — частично, REST некоторые endpoint 401/403 без VPN.

### Биржа НЕ предоставляет публично
- MEXC public liquidation stream — нет в API.
- Crypto.com public liquidation — только в private user.order после факта.
- dYdX dedicated liquidation channel — нет; через fill.liquidity == LIQUIDATED.
- Kraken Spot funding — Kraken Spot != Futures (разные продукты).
- Coinbase / Bitstamp / Upbit / Gemini funding/mark/OI — spot биржи.
- Coinbase International / Gemini Derivatives perpetuals — отдельные API endpoints, отдельные коннекторы (вне scope).
- Bitget position-tier / insurance public REST — v2 API не предоставляет (404).
- HTX swap_basis / swap_insurance_fund REST — 404 / Function suspended.
- HTX <sym>.index.<period> WS — канала нет публично.
- Gate.io futures.premium_index WS — удалён сервером; premium через futures.candlesticks с mark_/index_ prefix.
- Kraken WS L3 — Kraken WS v2 только L2.
- Kraken /derivatives/api/v3/openinterests, /initialmargin — оба 404.
- Bybit /v5/market/lt/kline — 404.
- Bybit ins-loan/product-infos — требует auth (помечен).
- BingX premiumIndex WS, longShortRatio REST — нет в API.
- Lighter get_liquidations — требует account creds.

### Архитектурное
Расширенные методы (funding/OI/ratios/liq/block-trade/etc.) — inherent на конкретных коннекторах. Generic потребитель через &dyn MarketData к ним не доберётся. Нужен switch по ExchangeId к concrete типу.

## 5. E2E живая проверка

examples/e2e_metadata.rs — главный gate перед push. Покрывает 19 бирж, ~50 REST методов, ~38 WS каналов.

```
cd C:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3
cargo run --example e2e_metadata --quiet 2>&1 | tail -100
```

Baseline (с текущей машины, без VPN):
- REST 48/50 pass (1 fail = Bybit delivery_price тестим perp; 1 skip = Binance force_orders требует ключ).
- WS все 20 коннекторов отвечают, ~4000 events за 10s окно. Hyperliquid 2286, Deribit 1043 (vol_idx 6 + opt markprice 1114), Bybit 372, Bitfinex 95, OKX 71, Bitstamp 36, Coinbase 30, Gate.io 5, KuCoin 4, HTX 3, Kraken 1.
- Zero-event каналы (Binance fstream, OKX block-trades, Gemini auction, Crypto.com settlement) — geo либо нативно редкие.

## 6. Как добавлять новое (паттерн)

1. Новый тип события: вариант в StreamEvent и StreamType в src/core/types/websocket.rs. Все существующие match сейтхы либо exhaustive с `_ =>`, либо compile-error поймает.
2. Новый REST на бирже: EndpointVariant в <ex>/endpoints.rs (path + auth-list), pub async fn get_<thing>(...) в <ex>/connector.rs (helpers get / get_with_query / get_signed), парсер в <ex>/parser.rs.
3. Новый WS канал: arm в <ex>/websocket.rs::parse_data_message, маппинг StreamType → topic в subscribe builder, парсер. Multi-emit — возвращай Vec<StreamEvent> либо event_tx.send в цикле.
4. Перед коммитом: расширь examples/e2e_metadata.rs тест-кейсом — проверь живым.
5. CRITICAL: НЕ верь docs автоматически. Всегда curl-verify живое API:
   - REST: curl -s URL | jq .
   - WS: мини-probe в examples/ который шлёт subscribe и печатает ответ. Удалить после.

История этой сессии: первая волна агентов работала по внутренним знаниям, отправила 16 silently-broken WS подписок (HTX funding market. вместо public., Bybit mark_price_kline.* которого не существует, Hyperliquid assetCtxs фейковый канал, Binance !compositeIndex@arr глобального стрима нет, и др.). Все исправлены в 0511d2b...f1661ad + 7c18072 + f4fd300. Но проблемы нашлись только когда поставили docs-fetch gate + e2e.

## 7. Известные TODO (не блокеры)

- dYdX v4_subaccounts private parsing — basic schema есть, deep nested cases могут терять поля.
- HTX coin-margined (/swap-api/) vs USDT-margined (/linear-swap-api/) — некоторые REST только для USDT-M. Полной симметрии нет.
- MEXC start_futures_ws() — scaffold готов, маршрутизация symbol → futures-vs-spot не тестирована из-за geo-block.
- Bitfinex deriv status field индексы могут плыть между версиями API.
- Crypto.com / dYdX public liquidation streams — биржи не отдают, не наш баг.
- Binance fstream WS testing — нужен VPN.

## 8. Commit map (хронология)

53 коммита. Полный лог: git log --oneline 27abd44..HEAD. Ключевые точки:

- bfb3e9b — план
- 694167d — core +5 variants (этапы 1-3)
- c26f1cf — core +7 variants (kline variants + index + insurance + basis + vol)
- 370f904 — core +OptionGreeks
- 9422798 — core +9 variants (full-coverage)
- 15fe16c — e2e_metadata.rs создан
- 0511d2b...f1661ad — 16-fix wave после docs verification
- 7c18072 + f4fd300 — финальные 10 fixes после расширенного e2e

Per-exchange детали: git log --oneline --grep=<exchange>.
