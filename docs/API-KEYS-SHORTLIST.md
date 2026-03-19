# API Keys Shortlist — digdigdig3

Чеклист для получения тестнет-ключей и бесплатных API-ключей.
Работай сверху вниз, ставь галочки по мере получения ключей.

---

## Раздел 1: Тестнет-ключи (Phase 1A)

Отсортировано по простоте получения — начинай сверху.

- [ ] **Binance** — Testnet
  - Тип: Testnet (отдельный домен)
  - Signup: https://testnet.binance.vision
  - Вход через GitHub OAuth, ключи выдаются мгновенно после авторизации
  - ```
    BINANCE_API_KEY=""
    BINANCE_API_SECRET=""
    BINANCE_TESTNET=true
    ```

- [ ] **Bybit** — Testnet
  - Тип: Testnet (отдельный домен)
  - Signup: https://testnet.bybit.com
  - Регистрация по email, на счёт зачисляется 10K USDT + 1 BTC / 24ч автоматически
  - ```
    BYBIT_API_KEY=""
    BYBIT_API_SECRET=""
    BYBIT_TESTNET=true
    ```

- [ ] **Deribit** — Testnet
  - Тип: Testnet (отдельный домен)
  - Signup: https://test.deribit.com
  - Без верификации email и без KYC — регистрируйся и сразу создавай ключи
  - ```
    DERIBIT_API_KEY=""
    DERIBIT_API_SECRET=""
    DERIBIT_TESTNET=true
    ```

- [ ] **Phemex** — Testnet
  - Тип: Testnet (отдельный домен)
  - Signup: https://testnet.phemex.com
  - При регистрации на счёт падает 0.5 BTC автоматически
  - ```
    PHEMEX_API_KEY=""
    PHEMEX_API_SECRET=""
    PHEMEX_TESTNET=true
    ```

- [ ] **Kraken** — Demo Futures
  - Тип: Demo (только фьючерсы, спот не поддерживается)
  - Signup: https://demo-futures.kraken.com
  - Подойдёт любой email, $50K виртуальных средств, никакого KYC
  - ВНИМАНИЕ: только фьючерсы — спот-торговля в demo-режиме недоступна
  - ```
    KRAKEN_API_KEY=""
    KRAKEN_API_SECRET=""
    KRAKEN_TESTNET=true
    ```

- [ ] **Gemini** — Sandbox
  - Тип: Sandbox (отдельный домен)
  - Signup: https://exchange.sandbox.gemini.com
  - Аккаунт верифицируется автоматически, без KYC
  - ```
    GEMINI_API_KEY=""
    GEMINI_API_SECRET=""
    GEMINI_TESTNET=true
    ```

- [ ] **KuCoin** — Sandbox
  - Тип: Sandbox (отдельный домен)
  - Signup: https://sandbox.kucoin.com
  - Отдельный аккаунт, не связан с основным KuCoin
  - ```
    KUCOIN_API_KEY=""
    KUCOIN_API_SECRET=""
    KUCOIN_PASSPHRASE=""
    KUCOIN_TESTNET=true
    ```

- [ ] **OKX** — Demo Trading
  - Тип: Demo (через заголовок `x-simulated-trading: 1`, основной домен)
  - Signup: https://okx.com → войди в аккаунт → Demo Trading → Create Demo API Key
  - Нужен основной OKX-аккаунт (без KYC для создания demo-ключа)
  - ```
    OKX_API_KEY=""
    OKX_API_SECRET=""
    OKX_PASSPHRASE=""
    OKX_TESTNET=true
    ```

- [ ] **Bitget** — Demo Mode
  - Тип: Demo (через заголовок, основной домен)
  - Signup: https://bitget.com → войди в аккаунт → Demo mode → Create Demo Key
  - Нужен основной Bitget-аккаунт
  - ```
    BITGET_API_KEY=""
    BITGET_API_SECRET=""
    BITGET_PASSPHRASE=""
    BITGET_TESTNET=true
    ```

- [ ] **BingX** — VST Paper Trading
  - Тип: Paper (виртуальные VST-пары на живом endpoint)
  - Signup: https://bingx.com
  - Живой аккаунт BingX, фьючерсы VST (100K VST начисляется автоматически)
  - ВНИМАНИЕ: торгуется только ~2 виртуальные VST-пары, не обычные рыночные символы
  - ```
    BINGX_API_KEY=""
    BINGX_API_SECRET=""
    BINGX_TESTNET=true
    ```

- [ ] **Alpaca** — Paper Trading
  - Тип: Paper (отдельный endpoint в том же аккаунте)
  - Signup: https://app.alpaca.markets
  - Только email, кредитная карта не нужна; Paper Trading включён по умолчанию
  - ```
    ALPACA_API_KEY=""
    ALPACA_API_SECRET=""
    ALPACA_TESTNET=true
    ```

- [ ] **OANDA** — Practice Account
  - Тип: Practice (отдельный endpoint, тот же аккаунт)
  - Signup: https://www.oanda.com → открой Practice аккаунт при регистрации
  - Бесплатно, без верификации, виртуальные средства
  - ```
    OANDA_API_KEY=""
    OANDA_API_SECRET=""
    OANDA_TESTNET=true
    ```

- [ ] **Dhan** — Sandbox
  - Тип: Sandbox (token-based, отдельный endpoint)
  - Signup: https://developer.dhanhq.co
  - Нужен email + мобильный номер (Индия)
  - ```
    DHAN_API_KEY=""
    DHAN_API_SECRET=""
    DHAN_TESTNET=true
    ```

- [ ] **Gate.io** — Futures Testnet
  - Тип: Testnet (фьючерсы, основной аккаунт Gate.io)
  - Войди в аккаунт Gate.io → Futures → Testnet → Create API Key
  - ВНИМАНИЕ: спот-тестнет домен требует отдельной проверки — использовать только futures testnet
  - ```
    GATEIO_API_KEY=""
    GATEIO_API_SECRET=""
    GATEIO_TESTNET=true
    ```

- [ ] **dYdX v4** — Testnet Chain
  - Тип: Testnet (отдельная Cosmos-цепочка)
  - Signup: https://v4.testnet.dydx.exchange
  - Нужен Cosmos-кошелёк (Keplr/MetaMask); тестовые USDC через фaucet (300 USDC)
  - ```
    DYDX_API_KEY=""
    DYDX_API_SECRET=""
    DYDX_TESTNET=true
    ```

- [ ] **Lighter** — Testnet
  - Тип: Testnet (отдельный домен)
  - Signup: https://testnet.app.lighter.xyz
  - Тестовые средства запрашиваются в Discord сервере Lighter
  - ```
    LIGHTER_API_KEY=""
    LIGHTER_API_SECRET=""
    LIGHTER_TESTNET=true
    ```

- [ ] **Paradex** — Testnet
  - Тип: Testnet (StarkNet)
  - Signup: подключи StarkNet-кошелёк на тестнете; тестовый USDC через Discord #developers
  - ```
    PARADEX_API_KEY=""
    PARADEX_API_SECRET=""
    PARADEX_TESTNET=true
    ```

- [ ] **Bitfinex** — Paper Sub-Account
  - Тип: Paper (суб-аккаунт в основном Bitfinex аккаунте)
  - Signup: https://bitfinex.com → Sub-accounts → Paper
  - ВНИМАНИЕ: только 2 пары — TESTBTC:TESTUSD и TESTBTC:TESTUSDT
  - ```
    BITFINEX_API_KEY=""
    BITFINEX_API_SECRET=""
    BITFINEX_TESTNET=true
    ```

- [ ] **HyperLiquid** — Testnet
  - Тип: Testnet (EVM-кошелёк)
  - Signup: https://app.hyperliquid-testnet.xyz
  - ВНИМАНИЕ: для получения тестовых средств через faucet нужен реальный депозит $5 на mainnet
  - ```
    HYPERLIQUID_API_KEY=""
    HYPERLIQUID_API_SECRET=""
    HYPERLIQUID_TESTNET=true
    ```

- [ ] **Bitstamp** — Sandbox
  - Тип: Sandbox (отдельный домен)
  - Signup: https://sandbox.bitstamp.net
  - ВНИМАНИЕ: возможно потребуется KYC-верифицированный основной аккаунт Bitstamp
  - ```
    BITSTAMP_API_KEY=""
    BITSTAMP_API_SECRET=""
    BITSTAMP_TESTNET=true
    ```

---

## Раздел 2: API-ключи бесплатные (data feeds)

Провайдеры данных — бесплатный tier, ключи выдаются сразу после регистрации.

- [ ] **Polygon** — акции США, OHLCV
  - Бесплатный tier: 5 req/min, исторические данные
  - Signup: https://polygon.io
  - ```
    POLYGON_API_KEY=""
    ```

- [ ] **Finnhub** — акции, форекс, крипто
  - Бесплатный tier: 60 req/min, WebSocket до 50 символов
  - Signup: https://finnhub.io/register
  - ```
    FINNHUB_API_KEY=""
    ```

- [ ] **Tiingo** — исторические данные акций
  - Бесплатный tier: 100 req/day, EOD данные 30+ лет
  - Signup: https://tiingo.com
  - ```
    TIINGO_API_KEY=""
    ```

- [ ] **Twelvedata** — акции, форекс, крипто
  - Бесплатный tier: 8 req/min, WebSocket до 8 символов
  - Signup: https://twelvedata.com
  - ```
    TWELVEDATA_API_KEY=""
    ```

- [ ] **Alpha Vantage** — акции, форекс, крипто
  - Бесплатный tier: 25 req/day (очень мало — только для тестов)
  - Signup: https://alphavantage.co
  - ВНИМАНИЕ: переменная называется `ALPHAVANTAGE_API_KEY` (без подчёркивания между ALPHA и VANTAGE)
  - ```
    ALPHAVANTAGE_API_KEY=""
    ```

- [ ] **FRED** — макроэкономические данные (ФРС США)
  - Бесплатный tier: 120 req/min, 800K+ серий, навсегда бесплатно
  - Signup: https://fredaccount.stlouisfed.org
  - ```
    FRED_API_KEY=""
    ```

- [ ] **CryptoCompare** — крипто данные
  - Бесплатный tier: ~100K вызовов/месяц
  - Signup: https://cryptocompare.com
  - ```
    CRYPTOCOMPARE_API_KEY=""
    ```

- [ ] **Coinglass** — метрики деривативов (open interest, funding, liquidations)
  - Бесплатный tier: базовые метрики без подписки
  - Signup: https://coinglass.com
  - ```
    COINGLASS_API_KEY=""
    ```

- [ ] **Bitquery** — on-chain данные (GraphQL)
  - Бесплатный tier: 1000 trial points, 10 req/min
  - Signup: https://ide.bitquery.io
  - ```
    BITQUERY_API_KEY=""
    ```

- [ ] **Etherscan** — Ethereum on-chain данные
  - Бесплатный tier: 5 req/sec; работает для Mainnet и Sepolia testnet
  - Signup: https://etherscan.io
  - ВНИМАНИЕ: уточни префикс переменной в коннекторе `onchain/ethereum/etherscan`
  - ```
    ETHERSCAN_API_KEY=""
    ```

- [ ] **JQuants** — японские акции (JPX)
  - Бесплатный tier: исторические данные с задержкой 12 недель
  - Signup: https://jpx-jquants.com
  - ```
    JQUANTS_API_KEY=""
    ```

---

## Раздел 3: Не нужен ключ (уже работает)

Справочный список — ключи не нужны, подключай напрямую.

| Провайдер | Примечание |
|-----------|------------|
| **DefiLlama** | Открытый API, без ключа, без rate limit |
| **MOEX ISS** | Все данные MOEX бесплатны, регистрация не нужна |
| **WhaleAlert** | Базовый tier бесплатен (10 req/min) |
| **YahooFinance** | Неофициальный API, без ключа (есть rate limit) |
| **Uniswap** | On-chain, ключ не нужен (нужен Sepolia ETH с фaucet) |
| **Raydium** | Solana devnet, ключ не нужен (нужен devnet SOL с фaucet) |

---

## Раздел 4: .env шаблон

Скопируй в `.env` файл проекта, заполни по мере получения ключей.

```env
# =============================================================================
# TESTNET KEYS (Phase 1A)
# =============================================================================

# Binance Testnet — https://testnet.binance.vision (GitHub OAuth)
BINANCE_API_KEY=""
BINANCE_API_SECRET=""
BINANCE_TESTNET=true

# Bybit Testnet — https://testnet.bybit.com (email signup, 10K USDT/24h)
BYBIT_API_KEY=""
BYBIT_API_SECRET=""
BYBIT_TESTNET=true

# Deribit Testnet — https://test.deribit.com (no email verify, no KYC)
DERIBIT_API_KEY=""
DERIBIT_API_SECRET=""
DERIBIT_TESTNET=true

# Phemex Testnet — https://testnet.phemex.com (0.5 BTC on signup)
PHEMEX_API_KEY=""
PHEMEX_API_SECRET=""
PHEMEX_TESTNET=true

# Kraken Demo Futures — https://demo-futures.kraken.com (futures only, $50K virtual)
KRAKEN_API_KEY=""
KRAKEN_API_SECRET=""
KRAKEN_TESTNET=true

# Gemini Sandbox — https://exchange.sandbox.gemini.com (auto-verified)
GEMINI_API_KEY=""
GEMINI_API_SECRET=""
GEMINI_TESTNET=true

# KuCoin Sandbox — https://sandbox.kucoin.com (separate account)
KUCOIN_API_KEY=""
KUCOIN_API_SECRET=""
KUCOIN_PASSPHRASE=""
KUCOIN_TESTNET=true

# OKX Demo Trading — okx.com → Demo Trading → Create Demo API Key
OKX_API_KEY=""
OKX_API_SECRET=""
OKX_PASSPHRASE=""
OKX_TESTNET=true

# Bitget Demo Mode — bitget.com → Demo mode → Create Demo Key
BITGET_API_KEY=""
BITGET_API_SECRET=""
BITGET_PASSPHRASE=""
BITGET_TESTNET=true

# BingX VST Paper — https://bingx.com (VST pairs only, 100K VST)
BINGX_API_KEY=""
BINGX_API_SECRET=""
BINGX_TESTNET=true

# Alpaca Paper — https://app.alpaca.markets (email only, no credit card)
ALPACA_API_KEY=""
ALPACA_API_SECRET=""
ALPACA_TESTNET=true

# OANDA Practice — https://www.oanda.com (free practice account)
OANDA_API_KEY=""
OANDA_API_SECRET=""
OANDA_TESTNET=true

# Dhan Sandbox — https://developer.dhanhq.co (email + mobile)
DHAN_API_KEY=""
DHAN_API_SECRET=""
DHAN_TESTNET=true

# Gate.io Futures Testnet — gateio.com → Futures → Testnet API Key
GATEIO_API_KEY=""
GATEIO_API_SECRET=""
GATEIO_TESTNET=true

# dYdX v4 Testnet — https://v4.testnet.dydx.exchange (Cosmos wallet + faucet 300 USDC)
DYDX_API_KEY=""
DYDX_API_SECRET=""
DYDX_TESTNET=true

# Lighter Testnet — https://testnet.app.lighter.xyz (Discord for test funds)
LIGHTER_API_KEY=""
LIGHTER_API_SECRET=""
LIGHTER_TESTNET=true

# Paradex Testnet — StarkNet wallet + USDC via Discord #developers
PARADEX_API_KEY=""
PARADEX_API_SECRET=""
PARADEX_TESTNET=true

# Bitfinex Paper — bitfinex.com → Sub-accounts → Paper (only 2 pairs!)
BITFINEX_API_KEY=""
BITFINEX_API_SECRET=""
BITFINEX_TESTNET=true

# HyperLiquid Testnet — https://app.hyperliquid-testnet.xyz (EVM wallet; $5 mainnet deposit for faucet)
HYPERLIQUID_API_KEY=""
HYPERLIQUID_API_SECRET=""
HYPERLIQUID_TESTNET=true

# Bitstamp Sandbox — https://sandbox.bitstamp.net (may need live KYC account)
BITSTAMP_API_KEY=""
BITSTAMP_API_SECRET=""
BITSTAMP_TESTNET=true

# =============================================================================
# FREE DATA FEED KEYS
# =============================================================================

# Polygon — 5 req/min, historical OHLCV — https://polygon.io
POLYGON_API_KEY=""

# Finnhub — 60 req/min, WS 50 symbols — https://finnhub.io/register
FINNHUB_API_KEY=""

# Tiingo — 100 req/day, 30+ years EOD — https://tiingo.com
TIINGO_API_KEY=""

# Twelvedata — 8 req/min, WS 8 symbols — https://twelvedata.com
TWELVEDATA_API_KEY=""

# Alpha Vantage — 25 req/day — https://alphavantage.co
# NOTE: ALPHAVANTAGE (no underscore between ALPHA and VANTAGE)
ALPHAVANTAGE_API_KEY=""

# FRED (St. Louis Fed) — 120 req/min, 800K+ series, forever free — https://fredaccount.stlouisfed.org
FRED_API_KEY=""

# CryptoCompare — ~100K calls/month — https://cryptocompare.com
CRYPTOCOMPARE_API_KEY=""

# Coinglass — free tier basic metrics — https://coinglass.com
COINGLASS_API_KEY=""

# Bitquery — 1000 trial points, 10 req/min — https://ide.bitquery.io
BITQUERY_API_KEY=""

# Etherscan — free tier + Sepolia testnet — https://etherscan.io
ETHERSCAN_API_KEY=""

# JQuants — JP equities, 12-week delay on free tier — https://jpx-jquants.com
JQUANTS_API_KEY=""
```
