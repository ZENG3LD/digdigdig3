---
title: digdigdig3 non-OHLCV/L2 completion spec
status: pending
owner: digdigdig3
consumer: mylittleindicators
---

# Цель

Полностью закрыть пробелы в `digdigdig3` для non-OHLCV/L2 публичных
маркет-данных: liquidations, funding, mark price, open interest,
long/short ratios, taker stats. Это нужно для `mylittleindicators` который
уже имеет consumer traits и хочет реальные стримы/REST в качестве источника.

Уже сделано (не трогать):
- `Liquidation` тип в `core/types/market_data.rs`
- Binance REST `get_force_orders()` + endpoint `FuturesForceOrders` + parser

# Этап 1 — Core type расширение (фундамент)

Файл: `src/core/types/websocket.rs`

## 1.1 Добавить варианты `StreamType`

После существующего `MarkPrice`:

```rust
Liquidation,
OpenInterest,
LongShortRatio,
AggTrade,
CompositeIndex,
```

## 1.2 Добавить варианты `StreamEvent`

После `StreamEvent::MarkPrice`:

```rust
Liquidation {
    symbol: String,
    side: TradeSide,    // Buy = long liquidated, Sell = short liquidated
    price: f64,
    quantity: f64,
    timestamp: i64,
    /// Optional quote value (price * qty)
    value: Option<f64>,
},
OpenInterestUpdate {
    symbol: String,
    open_interest: f64,
    /// Quote value if available
    open_interest_value: Option<f64>,
    timestamp: i64,
},
LongShortRatio {
    symbol: String,
    /// Ratio type identifier: "top_account", "top_position", "global_account", "taker"
    ratio_type: String,
    long_ratio: f64,
    short_ratio: f64,
    timestamp: i64,
},
AggTrade {
    symbol: String,
    aggregate_id: i64,
    price: f64,
    quantity: f64,
    first_trade_id: i64,
    last_trade_id: i64,
    side: TradeSide,
    timestamp: i64,
},
CompositeIndex {
    symbol: String,
    price: f64,
    components: Vec<(String, f64)>,  // (component_symbol, weight)
    timestamp: i64,
},
```

Re-exports в `src/core/types/mod.rs` уже работают через `pub use`. Проверить что
новые варианты доступны через `digdigdig3::core::types::StreamEvent::*`.

# Этап 2 — Binance дозаполнение

## 2.1 WS `!forceOrder@arr` → `StreamEvent::Liquidation`

Файл: `src/l3/open/crypto/cex/binance/websocket.rs`

Binance посылает на `!forceOrder@arr` сообщения вида:
```json
{
  "e": "forceOrder",
  "E": 1568014460893,
  "o": {
    "s": "BTCUSDT",
    "S": "SELL",         // side of the order — opposite to liquidated position
    "o": "LIMIT",
    "f": "IOC",
    "q": "0.014",
    "p": "9910",
    "ap": "9910",
    "X": "FILLED",
    "l": "0.014",
    "z": "0.014",
    "T": 1568014460893
  }
}
```

Важно: `S` — сторона **forced order** (контр-ордер ликвидации).
Если `S == "SELL"` — long position liquidated (forced sell to close long).
Если `S == "BUY"` — short position liquidated.

Маппинг:
- `S == "SELL"` → `TradeSide::Buy` (long was liquidated — в нашей семантике Buy=long)
- `S == "BUY"` → `TradeSide::Sell` (short liquidated)

Подписаться можно через market-wide `!forceOrder@arr` или per-symbol `<symbol>@forceOrder`.

В parser добавить case для `event_type == "forceOrder"` или для general `!forceOrder@arr`,
эмитить `StreamEvent::Liquidation { symbol, side, price = ap (average price), quantity = q, timestamp = T, value = Some(ap*q) }`.

Add subscription support: в connector `subscribe_*` или подобной функции
добавить возможность подписки на liquidation stream.

## 2.2 REST: top-trader + taker ratios

Файл: `src/l3/open/crypto/cex/binance/endpoints.rs`

Добавить варианты enum:

```rust
FuturesTopLongShortPositionRatio,   // /futures/data/topLongShortPositionRatio
FuturesGlobalLongShortAccountRatio, // /futures/data/globalLongShortAccountRatio
FuturesTakerLongShortRatio,         // /futures/data/takerlongshortRatio
```

(`FuturesTopLongShortAccountRatio` уже есть в enum по аудиту, проверить — если есть, просто использовать; если нет — добавить.)

Файл: `src/l3/open/crypto/cex/binance/connector.rs`

Добавить методы (паттерн как `get_open_interest`):

```rust
pub async fn get_top_long_short_account_ratio(
    &self,
    symbol: &str,
    period: &str,  // "5m" | "15m" | "30m" | "1h" | "2h" | "4h" | "6h" | "12h" | "1d"
    limit: Option<u32>,
    start_time: Option<i64>,
    end_time: Option<i64>,
) -> ExchangeResult<Value> { ... }

pub async fn get_top_long_short_position_ratio(...) -> ExchangeResult<Value> { ... }
pub async fn get_global_long_short_account_ratio(...) -> ExchangeResult<Value> { ... }
pub async fn get_taker_long_short_ratio(...) -> ExchangeResult<Value> { ... }
```

Все 4 публичные (без подписи), AccountType::Futures.

Файл: `src/l3/open/crypto/cex/binance/parser.rs`

Добавить parser для каждого ответа. Binance возвращает array объектов:
```json
[
  {
    "symbol": "BTCUSDT",
    "longShortRatio": "1.0500",
    "longAccount": "0.5121",   // или longPosition или buySellRatio
    "shortAccount": "0.4879",
    "timestamp": 1583139600000
  }
]
```

Парсер должен возвращать `Vec<LongShortRatio>` где `LongShortRatio` — новый типизированный struct (см. 2.2.1 ниже) или продолжить с `Value` raw — на выбор. Рекомендую типизировать.

### 2.2.1 Тип LongShortRatio в core/types/market_data.rs

```rust
#[derive(Debug, Clone)]
pub struct LongShortRatio {
    pub symbol: String,
    /// "top_account" | "top_position" | "global_account" | "taker"
    pub ratio_type: String,
    pub long_ratio: f64,
    pub short_ratio: f64,
    /// long_ratio / short_ratio (если применимо)
    pub ratio: Option<f64>,
    pub timestamp: i64,
}
```

## 2.3 REST: OI history

Endpoint `FuturesOpenInterestHist` уже в enum (`/futures/data/openInterestHist`). Connector method отсутствует.

Файл: `src/l3/open/crypto/cex/binance/connector.rs`

```rust
pub async fn get_open_interest_history(
    &self,
    symbol: &str,
    period: &str,  // "5m" | "15m" | "30m" | "1h" | "2h" | "4h" | "6h" | "12h" | "1d"
    limit: Option<u32>,
    start_time: Option<i64>,
    end_time: Option<i64>,
) -> ExchangeResult<Value> { ... }
```

Парсер для array `OpenInterest { symbol, openInterest, openInterestValue, timestamp }`.

# Этап 3 — Liquidations на других биржах

## 3.1 Bybit WS `liquidation.<symbol>`

Файл: `src/l3/open/crypto/cex/bybit/websocket.rs`

Bybit V5 формат:
```json
{
  "topic": "liquidation.BTCUSDT",
  "data": {
    "symbol": "BTCUSDT",
    "side": "Buy",      // side of order, opposite to liquidated position
    "size": "0.500",
    "price": "29000.5",
    "updatedTime": 1672304801000
  }
}
```

Маппинг side: аналогично Binance — Buy order = short liquidated, Sell order = long liquidated.

Добавить case в parser → emit `StreamEvent::Liquidation`.

## 3.2 OKX REST liquidation orders

Файл: `src/l3/open/crypto/cex/okx/endpoints.rs`

Добавить variant `PublicLiquidationOrders` → `/api/v5/public/liquidation-orders`.

Файл: `src/l3/open/crypto/cex/okx/connector.rs`

```rust
pub async fn get_liquidation_orders(
    &self,
    inst_type: &str,    // "SWAP" | "FUTURES" | "MARGIN" | "OPTION"
    inst_family: Option<&str>,
    inst_id: Option<&str>,
    state: Option<&str>, // "unfilled" | "filled"
    before: Option<&str>,
    after: Option<&str>,
    limit: Option<u32>,
) -> ExchangeResult<Value> { ... }
```

OKX формат ответа: `data[].details[].{side, posSide, sz, fillPx, ts}`. Парсер `parse_liquidations` для конверсии в `Vec<Liquidation>`.

## 3.3 OKX WS liquidation-orders channel

Файл: `src/l3/open/crypto/cex/okx/websocket.rs`

OKX V5 имеет `liquidation-orders` channel для public stream. Добавить:
- Subscribe support
- Parser → `StreamEvent::Liquidation`

Формат: `{"arg": {"channel": "liquidation-orders", "instType": "SWAP"}, "data": [{"details": [{...}]}]}`.

## 3.4 Hyperliquid WS liquidation channel

Hyperliquid имеет публичный `liquidations` channel. Файл: `src/l3/open/crypto/cex/hyperliquid/websocket.rs`.

Добавить subscribe + parser → `StreamEvent::Liquidation`.

# Этап 4 — Hyperliquid event separation

Файл: `src/l3/open/crypto/cex/hyperliquid/websocket.rs::parse_active_asset_ctx`

Сейчас всё пакуется в `StreamEvent::Ticker`. Нужно эмитить **дополнительно**:
- `StreamEvent::MarkPrice { symbol, mark_price, index_price, timestamp }` — если `markPx` поле есть
- `StreamEvent::FundingRate { symbol, rate, next_funding_time, timestamp }` — если `funding` поле есть
- `StreamEvent::OpenInterestUpdate { symbol, open_interest, open_interest_value, timestamp }` — если `openInterest` поле есть

Ticker эмитится продолжать (для backwards-compat). Это **multi-emit** — один WS message может производить 2-4 StreamEvent одновременно.

Сигнатура parse функций может потребовать изменения с `Result<Option<StreamEvent>, _>` на `Result<Vec<StreamEvent>, _>`. Проверь общую сигнатуру и обнови dispatch если нужно.

# Этап 5 — OKX/Bybit/HTX/Bitget ticker → отдельные event'ы

Аналогично Hyperliquid: если WS ticker содержит OI/funding/markPrice, эмитить отдельные `StreamEvent::OpenInterestUpdate`, `StreamEvent::FundingRate`, `StreamEvent::MarkPrice` в дополнение к `Ticker`.

Файлы:
- `src/l3/open/crypto/cex/okx/websocket.rs`
- `src/l3/open/crypto/cex/bybit/websocket.rs`
- `src/l3/open/crypto/cex/htx/websocket.rs` (если WS подключён)
- `src/l3/open/crypto/cex/bitget/websocket.rs`

# Этап 6 — Прочие connectors (без срочности)

## 6.1 Bitget WS FundingRate/MarkPrice parsing
Сейчас подписано но не парсится. Прочитать сообщения, добавить parser.

## 6.2 HTX WS wire
HTX REST для funding/OI/mark есть. WS не подключён для этих типов. Добавить subscribe + parser.

## 6.3 OKX REST mark_price method
Не существует connector method, хотя endpoint доступен. Добавить `get_mark_price()`.

# Тестирование

Для каждого добавленного REST метода — синтетический тест с mock-ответом (raw JSON → parser → typed struct).

WS parsers — тесты на семплах сообщений из каждой биржи (формат документации).

`cargo check` + `cargo test` в digdigdig3 — pass, 0 warnings.

# Что НЕ делать

- НЕ трогать L1/L2 stocks/forex connectors (Yahoo, Polygon, Finnhub) — отдельный этап
- НЕ менять `MarketData` core trait — все эти методы остаются как extension methods на конкретных connectors
- НЕ запускать workspace check вне digdigdig3

# Финальный отчёт

Для каждого этапа:
- Что сделано
- Какие файлы созданы/изменены
- Тестов добавлено
- cargo check + test result

Особенно важно для mli-integration:
- Полный список новых вариантов `StreamEvent` (для добавления consumer dispatch'а в mli)
- Полный список новых connector REST методов (для документации)
- Изменения сигнатуры parse функций (если перешли на `Vec<StreamEvent>`)

# После завершения

Юзер передаст этот отчёт обратно в mli-side агенту для:
1. Добавления `Liquidation`, `LongShortRatio`, `AggTrade`, `CompositeIndex` типов и consumers в mli
2. Реализации `LongShortRatioConsumer` trait + индикаторов (ratio momentum, divergence vs price)
3. Расширения существующих consumers с новыми вариантами событий
