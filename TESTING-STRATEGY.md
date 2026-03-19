# Testing Strategy: Consolidated Testability Reference

**Date:** 2026-03-17
**Sources consolidated from:**
- `cex-testnets-audit.md`
- `dex-testnets-audit.md`
- `broker-sandbox-audit.md`
- `intelligence-feeds-free-tiers.md`
- `testnet-support-codebase-audit.md`

---

## Section 1: Testability Matrix

All ~55 connectors across all categories.

Priority tiers:
- **Tier 1** — Free testnet with full trading simulation (test everything for free)
- **Tier 2** — Free market data only (no free trading test)
- **Tier 3** — Paid required or real account required
- **Tier 4** — Blocked, shut down, or structurally impossible to test

### CEX Connectors (19)

| Connector | Category | Testnet/Sandbox? | Free Data? | Free Trading Test? | Needs Real Money? | API Key Source | Priority |
|-----------|----------|-----------------|------------|-------------------|-----------------|----------------|----------|
| Binance | crypto/cex | YES — separate testnet | Yes (public) | Yes | No | testnet.binance.vision + GitHub OAuth | Tier 1 |
| Bybit | crypto/cex | YES — full testnet + demo | Yes (public) | Yes | No | testnet.bybit.com — email signup | Tier 1 |
| OKX | crypto/cex | YES — demo via header | Yes (public) | Yes (simulated) | No | OKX demo account — free signup | Tier 1 |
| KuCoin | crypto/cex | YES — sandbox env | Yes (public) | Yes | No | sandbox.kucoin.com — free signup | Tier 1 |
| Kraken | crypto/cex | PARTIAL — futures demo only | Yes (public, no auth) | Futures only | No for futures demo | demo-futures.kraken.com | Tier 1 (futures), Tier 2 (spot) |
| Gemini | crypto/cex | YES — full sandbox | Yes (public) | Yes | No | exchange.sandbox.gemini.com | Tier 1 |
| Bitstamp | crypto/cex | YES — sandbox | Yes (public) | Yes | No (but KYC on live acct) | sandbox.bitstamp.net | Tier 1 |
| Deribit | crypto/cex | YES — full testnet | Yes (public) | Yes | No | test.deribit.com — free signup | Tier 1 |
| Phemex | crypto/cex | YES — full testnet | Yes (public) | Yes | No | testnet.phemex.com — free signup | Tier 1 |
| HyperLiquid | crypto/cex | YES — testnet (faucet requires mainnet deposit) | Yes (public) | Yes* | $5+ USDC on Arbitrum mainnet for faucet | Wallet-based — EVM private key | Tier 1* |
| Bitget | crypto/cex | YES — demo via header (paptrading: 1) | Yes (public) | Yes (simulated) | No | Bitget demo account — free signup | Tier 1 |
| BingX | crypto/cex | PARTIAL — VST demo pairs on live endpoint | Yes (public) | Yes (VST, no real money) | No | BingX account — free signup | Tier 1 |
| Bitfinex | crypto/cex | PARTIAL — 2 paper trading pairs | Yes (public) | Spot only (TEST pairs) | No | Bitfinex paper sub-account | Tier 1 (limited) |
| Gate.io | crypto/cex | PARTIAL — futures testnet only | Yes (public) | Futures only | No | Gate.io account for testnet keys | Tier 1 (futures), Tier 2 (spot) |
| MEXC | crypto/cex | PARTIAL — futures demo UI only, no spot API sandbox | Yes (public) | No API sandbox | No | Live account for spot API | Tier 2 |
| Crypto.com | crypto/cex | PARTIAL — UAT institutional only | Yes (public) | No (retail blocked) | No | Institutional account required | Tier 3 |
| HTX | crypto/cex | NO — testnet shut down | Yes (public) | No | No | Live account (minimal balance) | Tier 2 |
| Upbit | crypto/cex | NO — no testnet | Yes (public quotation API) | No | KYC required | Static IP required | Tier 2 |
| Bithumb | crypto/cex | NO — no real testnet (param ignored) | Yes (public) | No | Korean KYC required | Live account | Tier 2 |

### DEX Connectors (5)

| Connector | Category | Testnet/Sandbox? | Free Data? | Free Trading Test? | Needs Real Money? | API Key Source | Priority |
|-----------|----------|-----------------|------------|-------------------|-----------------|----------------|----------|
| dYdX v4 | crypto/dex | YES — active testnet (dydx-testnet-4) | Yes | Yes (faucet drip 300 USDC) | No | Wallet + faucet.v4testnet.dydx.exchange | Tier 1 |
| Lighter | crypto/dex | YES — testnet.zklighter.elliot.ai | Yes | Yes (credited funds) | No | Testnet account — Discord for funds | Tier 1 |
| Paradex | crypto/dex | YES — new testnet (Mar 2025) | Yes | Yes (testnet USDC via Discord) | No | Wallet + Paradex Discord | Tier 1 |
| Jupiter | crypto/dex | NO — mainnet-only | Yes (quote = dry-run) | No (quote only) | SOL for real swaps | No API key needed (public) | Tier 2 |
| GMX v2 | crypto/dex | PARTIAL — Arbitrum Sepolia/Fuji (not maintained) | Partial (on-chain only) | Partial (on-chain) | No (test ETH from faucet) | No key (on-chain contracts) | Tier 2 |

### Swap Connectors (2)

| Connector | Category | Testnet/Sandbox? | Free Data? | Free Trading Test? | Needs Real Money? | API Key Source | Priority |
|-----------|----------|-----------------|------------|-------------------|-----------------|----------------|----------|
| Raydium | crypto/swap | PARTIAL — Solana devnet (no pre-seeded pools) | Partial (devnet only) | Yes (custom pools) | No (devnet SOL from faucet) | No key (public) | Tier 1 (limited) |
| Uniswap | crypto/swap | YES — Ethereum Sepolia + multi-chain testnets | Yes (on-chain) | Yes (QuoterV2 dry-run + real swaps) | No (Sepolia ETH from faucet) | No key (on-chain contracts) | Tier 1 |

### Stocks US (5)

| Connector | Category | Testnet/Sandbox? | Free Data? | Free Trading Test? | Needs Real Money? | API Key Source | Priority |
|-----------|----------|-----------------|------------|-------------------|-----------------|----------------|----------|
| Alpaca | stocks/us | YES — paper trading (default) | Yes (IEX feed) | Yes | No | app.alpaca.markets — email only | Tier 1 |
| Polygon | stocks/us | No sandbox — free tier is safe path | Yes (5 calls/min) | No | No | polygon.io — email signup | Tier 2 |
| Finnhub | stocks/us | No sandbox — free tier available | Yes (60 calls/min, WS) | No | No | finnhub.io — email signup | Tier 2 |
| Tiingo | stocks/us | No sandbox — free tier available | Yes (100 req/day, EOD) | No | No | tiingo.com — email signup | Tier 2 |
| Twelvedata | stocks/us | No sandbox — demo API key tier | Yes (8 calls/min) | No | No | twelvedata.com — email signup | Tier 2 |

### Stocks India (5)

| Connector | Category | Testnet/Sandbox? | Free Data? | Free Trading Test? | Needs Real Money? | API Key Source | Priority |
|-----------|----------|-----------------|------------|-------------------|-----------------|----------------|----------|
| Dhan | stocks/india | YES — sandbox (no brokerage acct needed) | Partial (sandbox fills at ₹100) | Yes (simulated orders) | No | developer.dhanhq.co — email signup | Tier 1 |
| Upstox | stocks/india | YES — sandbox (Jan 2025, orders only) | Partial (no market data) | Yes (order flow only) | Indian KYC required | Upstox account required | Tier 3 |
| Angel One | stocks/india | No sandbox (param stored but ignored) | Yes (live only) | No | Indian KYC required | Angel One account | Tier 3 |
| Fyers | stocks/india | No free sandbox — paid API Bridge | Live data (account req) | No (paid paper trade) | Indian KYC + paid | Fyers account | Tier 3 |
| Zerodha | stocks/india | No sandbox | Live data (account req) | No | Indian KYC required | Zerodha account + Kite subscription | Tier 3 |

### Stocks Russia (2)

| Connector | Category | Testnet/Sandbox? | Free Data? | Free Trading Test? | Needs Real Money? | API Key Source | Priority |
|-----------|----------|-----------------|------------|-------------------|-----------------|----------------|----------|
| Tinkoff | stocks/russia | YES — full sandbox (gRPC) | Yes (real market data) | Yes (virtual rubles) | Russian T-Bank account required | T-Bank account — sandbox token | Tier 3 |
| MOEX | stocks/russia | NO API sandbox (ISS public is free) | Yes (ISS — no auth, free) | No | Licensed trade participant for test env | No key for ISS (public) | Tier 2 |

### Stocks China (1)

| Connector | Category | Testnet/Sandbox? | Free Data? | Free Trading Test? | Needs Real Money? | API Key Source | Priority |
|-----------|----------|-----------------|------------|-------------------|-----------------|----------------|----------|
| Futu | stocks/china | YES — paper via TrdEnv::Simulate | Yes (real market data) | Yes (simulated) | Futu ID (ID verification may be required) | Futu account + OpenD daemon | Tier 3 |

### Stocks Japan (1)

| Connector | Category | Testnet/Sandbox? | Free Data? | Free Trading Test? | Needs Real Money? | API Key Source | Priority |
|-----------|----------|-----------------|------------|-------------------|-----------------|----------------|----------|
| JQuants | stocks/japan | No sandbox | Yes (12-week delay free) | No | No | jpx-jquants.com — email signup | Tier 2 |

### Stocks Korea (1)

| Connector | Category | Testnet/Sandbox? | Free Data? | Free Trading Test? | Needs Real Money? | API Key Source | Priority |
|-----------|----------|-----------------|------------|-------------------|-----------------|----------------|----------|
| KRX | stocks/korea | No sandbox | Yes (KRX Open API free) | No | No | openapi.krx.co.kr — Korean portal | Tier 2 |

### Forex (3)

| Connector | Category | Testnet/Sandbox? | Free Data? | Free Trading Test? | Needs Real Money? | API Key Source | Priority |
|-----------|----------|-----------------|------------|-------------------|-----------------|----------------|----------|
| OANDA | forex | YES — full practice account | Yes (streaming) | Yes | No | oanda.com — free registration | Tier 1 |
| AlphaVantage | forex | No sandbox — free tier | Yes (25 req/day) | No | No | alphavantage.co — email signup | Tier 2 |
| Dukascopy | forex | Demo expires in 14 days, Java SDK | Yes (historical tick data) | Yes (demo) | No | dukascopy.com — email (demo expires) | Tier 2 |

### Aggregators (4)

| Connector | Category | Testnet/Sandbox? | Free Data? | Free Trading Test? | Needs Real Money? | API Key Source | Priority |
|-----------|----------|-----------------|------------|-------------------|-----------------|----------------|----------|
| Interactive Brokers | aggregators | YES — paper trading | Yes (delayed or subscribed) | Yes | IBKR account (free, KYC) + local TWS/Gateway | IBKR account + TWS application | Tier 3 |
| Finnhub | aggregators | No sandbox | Yes (60 calls/min) | No | No | finnhub.io — email signup | Tier 2 |
| DefiLlama | aggregators | No sandbox (not needed) | Yes (fully free, no key) | No (data only) | No | No key required | Tier 2 |
| Yahoo Finance | aggregators | No sandbox | Unofficial (unreliable) | No | No | No key (unofficial) | Tier 4 |
| CryptoCompare | aggregators | No sandbox | Yes (100K calls/month free) | No | No | cryptocompare.com — email signup | Tier 2 |

### Intelligence Feeds (implementing trait — 2)

| Connector | Category | Testnet/Sandbox? | Free Data? | Free Trading Test? | Needs Real Money? | API Key Source | Priority |
|-----------|----------|-----------------|------------|-------------------|-----------------|----------------|----------|
| Coinglass | intelligence_feeds | No | No (API paid from $29/mo) | No | $29/mo minimum | coinglass.com subscription | Tier 3 |
| FRED | intelligence_feeds | No sandbox (not needed) | Yes (fully free) | No (data only) | No | fredaccount.stlouisfed.org — free | Tier 2 |

### On-Chain (3)

| Connector | Category | Testnet/Sandbox? | Free Data? | Free Trading Test? | Needs Real Money? | API Key Source | Priority |
|-----------|----------|-----------------|------------|-------------------|-----------------|----------------|----------|
| Etherscan | onchain/ethereum | YES — api-sepolia.etherscan.io | Yes (limited free tier) | No (data only) | No | etherscan.io — free account | Tier 2 |
| Bitquery | onchain/analytics | No testnet | Yes (1,000 trial points) | No (data only) | No | ide.bitquery.io — email signup | Tier 2 |
| Whale Alert | onchain/analytics | No testnet | 7-day trial only | No (data only) | Paid after trial | whale-alert.io | Tier 3 |

### Prediction (1)

| Connector | Category | Testnet/Sandbox? | Free Data? | Free Trading Test? | Needs Real Money? | API Key Source | Priority |
|-----------|----------|-----------------|------------|-------------------|-----------------|----------------|----------|
| Polymarket | prediction | No testnet | Yes (public CLOB API) | No | Real USDC on Polygon mainnet | No key for read; CLOB key for trading | Tier 2 |

---

## Section 2: Tier 1 — Free Full Testing (testnet + trading)

These connectors allow testing the complete trading flow — market data, orders, fills — at zero cost.

### CEX

**Binance (Spot + Futures)**
- Spot testnet: `https://testnet.binance.vision/api`
- WS streams: `wss://testnet.binance.vision/stream`
- Futures testnet: `https://testnet.binancefuture.com`
- Keys: Visit https://testnet.binance.vision, authorize with GitHub. Keys generated instantly. No Binance account needed.
- Codebase URL: `https://testapi.binance.vision` (different from real testnet URL — see Section 7)

**Bybit (Full — spot, perp, futures, options)**
- REST: `https://api-testnet.bybit.com`
- WS: `wss://stream-testnet.bybit.com`
- Keys: Register at https://testnet.bybit.com with email. Request test coins once per 24h (10,000 USDT + 1 BTC). Create API key from API Management page.

**OKX (Spot, margin, futures, options — simulated)**
- Base URL: `https://www.okx.com` (same as production)
- Required header: `x-simulated-trading: 1`
- Keys: Login to OKX → Demo Trading → Personal Center → Demo Trading API → Create Demo API Key. Free for all registered users.
- Note: Not isolated infrastructure — same domain, differentiated by header.

**KuCoin (Spot + Futures)**
- REST: `https://openapi-sandbox.kucoin.com`
- WS: `wss://ws-api-sandbox.kucoin.com`
- Keys: Register at https://sandbox.kucoin.com (separate account). Virtual BTC/ETH/KCS issued on registration.

**Kraken Futures (perpetuals and futures only)**
- REST: `https://demo-futures.kraken.com/derivatives/api/v3`
- WS: `wss://demo-futures.kraken.com/ws/v1`
- Keys: Visit https://demo-futures.kraken.com, use any email/password (email access not required). Free $50,000 virtual USD wallet.
- Note: Spot has no sandbox. Spot public market data is unauthenticated.

**Gemini (Full spot + margin)**
- REST: `https://api.sandbox.gemini.com/v1`
- WS: `wss://api.sandbox.gemini.com`
- Keys: Register at https://exchange.sandbox.gemini.com. Auto-verified with test funds (USD, BTC, ETH, etc.). 2FA bypass: use header `GEMINI-SANDBOX-2FA` with code `9999999`.

**Bitstamp (Spot)**
- REST: `https://sandbox.bitstamp.net/api/v2/`
- Keys: Existing Bitstamp account → Settings → API Access → New API Key. Separate sandbox credentials from production. Note: May require live account with KYC first.

**Bitget (Spot + futures — simulated)**
- REST: `https://api.bitget.com` + header `paptrading: 1`
- WS public demo: `wss://wspap.bitget.com/v3/ws/public`
- WS private demo: `wss://wspap.bitget.com/v3/ws/private`
- Keys: Login to Bitget → Demo mode → Personal Center → API Key Management → Create Demo API Key.

**BingX (Futures via VST pairs)**
- REST: Production endpoint with `BTC-VST` pairs (no real money risk)
- Keys: BingX account (free signup). New traders auto-receive 100,000 VST. Generate API keys from API Management.
- Note: Not an isolated sandbox — VST is virtual currency on the live endpoint.

**Bitfinex (Spot — 2 paper pairs only)**
- REST: `https://api.bitfinex.com` (same as production)
- Paper pairs: `tTESTBTC:TESTUSD`, `tTESTBTC:TESTUSDT`
- Keys: Bitfinex account → Sub-accounts → Create Paper sub-account. Very limited: only 2 trading pairs.

**Deribit (Options, futures, perpetuals, spot)**
- REST: `https://test.deribit.com/api/v2`
- WS: `wss://test.deribit.com/ws/api/v2`
- Keys: Register at https://test.deribit.com. No email verification, no KYC. Virtual funds auto-credited.

**Phemex (Spot + perpetuals up to 100x)**
- REST: `https://testnet-api.phemex.com`
- WS: `wss://testnet.phemex.com/ws`
- Keys: Register at https://testnet.phemex.com. API keys from testnet API management. 0.5 BTC virtual funds on registration.

**HyperLiquid (Perpetuals — with caveat)**
- REST: `https://api.hyperliquid-testnet.xyz`
- WS: `wss://api.hyperliquid-testnet.xyz/ws`
- Keys: Connect EVM wallet at https://app.hyperliquid-testnet.xyz. Generate API Wallet for delegation.
- CAVEAT: Faucet requires at least one mainnet deposit of $5+ USDC on Arbitrum. Third-party faucets (Chainstack, QuickNode) may bypass this.

### DEX

**dYdX v4 (Perpetuals — Cosmos chain)**
- REST indexer: `https://indexer.v4testnet.dydx.exchange/v4`
- WS: `wss://indexer.v4testnet.dydx.exchange/v4/ws`
- Faucet: https://faucet.v4testnet.dydx.exchange (300 USDC Dv4TNT drip)
- Testnet UI: https://v4.testnet.dydx.exchange/
- Keys: Cosmos wallet (no CEX-style API key). No mainnet requirement.

**Lighter (zkLighter perps)**
- REST: `https://testnet.zklighter.elliot.ai`
- WS: `wss://testnet.zklighter.elliot.ai/stream`
- Keys: Account via https://testnet.app.lighter.xyz. Test funds via Lighter Discord (no automated faucet as of 2026).

**Paradex (StarkNet perps)**
- REST: `https://api.testnet.paradex.trade/v1`
- WS: `wss://ws.api.testnet.paradex.trade/v1`
- Keys: StarkNet wallet + STARK signing. Test USDC via Paradex Discord #developers channel.
- Note: Testnet launched March 2025 — may still have limited markets.

### Swap

**Uniswap v3/v4 (Ethereum Sepolia)**
- No hosted API — interact directly with on-chain contracts
- QuoterV2 Sepolia: `0xEd1f6473345F45b75F8179591dd5bA1888cf2FB3`
- SwapRouter02 Sepolia: `0x3bFA4769FB09eefC5a80d6E87c3B9C650f7Ae48E`
- Test ETH: https://faucets.chain.link/sepolia, https://www.alchemy.com/faucets/ethereum-sepolia
- Test USDC: https://faucet.circle.com/
- No API key needed — on-chain contracts.

**Raydium (Solana devnet — limited)**
- REST: `https://api-v3-devnet.raydium.io`
- Solana devnet RPC: `https://api.devnet.solana.com`
- Devnet SOL: `https://faucet.solana.com` or `solana airdrop 2 <ADDRESS> --url devnet`
- Limitation: No pre-seeded liquidity pools. Must create custom token mints and pools.

### Stocks US

**Alpaca (US equities, crypto, options — full paper trading)**
- REST: `https://paper-api.alpaca.markets`
- WS data: `wss://stream.data.alpaca.markets`
- Keys: Signup at https://app.alpaca.markets with email only. Select Paper Trading → API Keys → Generate. No credit card. No real money.
- Note: The default constructor `new()` already points to paper trading (testnet=true).

### Stocks India

**Dhan (Orders + historical data)**
- REST: `https://api.dhan.co/v2/` (sandbox token used)
- DevPortal: https://developer.dhanhq.co
- Keys: Email + mobile signup at DevPortal. No Dhan brokerage account needed.
- Orders fill at static ₹100. Capital resets to ₹10,00,000 daily.
- Limitation: No streaming market data in sandbox.

### Forex

**OANDA (Forex, metals, CFDs — full parity)**
- REST: `https://api-fxpractice.oanda.com`
- Streaming: `https://stream-fxpractice.oanda.com`
- Keys: Register at https://www.oanda.com for free practice account → My Account → My Services → Manage API Access → Generate personal access token. No credit card, globally available.
- Rate limits: 120 REST req/second, 20 active streams.

---

## Section 3: Tier 2 — Free Data Testing Only

These connectors have free public market data (no auth or free tier API key) but trading requires real money or is not supported.

### CEX (data only with public endpoints)

| Connector | What's Free | Notes |
|-----------|------------|-------|
| **Kraken spot** | Public OHLC, orderbook, ticker — no auth required | No spot testnet; private (trading) API needs live account |
| **HTX** | Public market data (OHLC, orderbook, trades) — no auth | Testnet shut down |
| **MEXC** | Public REST + WS market data — no auth | Futures API is institutional-only |
| **Upbit** | Full public Quotation API — no auth | No testnet; private API needs KYC |
| **Bithumb** | Public market data | Korean KYC for trading |
| **Gate.io spot** | Public market data | Spot has no testnet; futures testnet available |

### DEX (quote/read only)

| Connector | What's Free | Notes |
|-----------|------------|-------|
| **Jupiter** | `/v6/quote` endpoint — truly non-executing dry-run | Mainnet-only, no devnet support |
| **GMX v2** | Reader contract view functions on testnet | Not actively maintained; sparse testnet docs |
| **Polymarket** | Public CLOB API — markets, orderbook, prices | Trading requires real USDC on Polygon mainnet |

### Stocks US (data only)

| Connector | What's Free | Rate Limit | Notes |
|-----------|------------|------------|-------|
| **Polygon** | Historical OHLCV, aggregates, tickers | 5 calls/min | Free Stocks Basic plan |
| **Finnhub** | US quotes, historical candles, fundamentals, news | 60 calls/min, 50 WS symbols | Best free tier for US stocks data |
| **Tiingo** | EOD OHLCV (30+ years), fundamentals | 100 req/day | Tight daily limit |
| **Twelvedata** | Real-time quotes, OHLCV, indicators | 8 calls/min, 8 WS symbols | Very restrictive |
| **Alpha Vantage** | Stock OHLCV, 50+ technical indicators | 25 req/day | Barely useful free tier |

### Stocks Russia

| Connector | What's Free | Notes |
|-----------|------------|-------|
| **MOEX ISS** | Full REST API — all MOEX market data, delayed + historical, no auth | No test trading env for retail; ISS is a hidden gem |

### Stocks Japan

| Connector | What's Free | Notes |
|-----------|------------|-------|
| **JQuants** | Historical stock prices (Japanese equities) — 12-week delay | Free plan registration only |

### Stocks Korea

| Connector | What's Free | Notes |
|-----------|------------|-------|
| **KRX** | KOSPI/KOSDAQ market data, historical prices | Korean-language portal; Twelvedata covers KRX as alternative |

### Forex (data only)

| Connector | What's Free | Notes |
|-----------|------------|-------|
| **Alpha Vantage** | Forex rates, basic indicators | 25 req/day on free tier |
| **Dukascopy** | Free historical tick/OHLCV download at dukascopy.com/datafeed | Demo account (14-day expiry) for trading |

### Aggregators (data only, no auth)

| Connector | What's Free | Notes |
|-----------|------------|-------|
| **DefiLlama** | DeFi TVL, token prices, yields, DEX volumes — no auth, no key | Completely open |
| **CryptoCompare** | Crypto OHLCV (minute/hour/daily), prices, on-chain metrics | ~100K calls/month on free tier; attribution required |
| **MOEX ISS** | See above | |

### Intelligence Feeds (data only)

| Connector | What's Free | Notes |
|-----------|------------|-------|
| **FRED** | 800K+ economic series, 120 req/min (free v2 key) | No payment ever; register at fredaccount.stlouisfed.org |
| **CoinMetrics Community** | On-chain + market data, 1.6 RPS | No signup needed: `community-api.coinmetrics.io/v4` |
| **Messari** | Crypto prices, market data, on-chain metrics | 20 req/min free tier |
| **Dune Analytics** | On-chain analytics queries, 2,500 credits/month | API included on free tier |
| **World Bank** | 29K+ economic indicators, 200+ countries, 1960-present | No auth, no key, no limits |
| **IMF** | WEO, BoP, Financial Soundness indicators | No auth needed |
| **GDELT** | Global news/events/sentiment (last 3 months searchable) | No auth, no rate limits |
| **Nasdaq Data Link** | Selected free datasets (FRED mirror, community data) | 50K calls/day with free key |
| **NewsAPI** | 100 req/day; articles 24h delayed | Dev plan; not suitable for live signals |
| **Nansen** | 100 one-time API credits | One-time allocation; then $49/mo |
| **Bitquery** | 1,000 trial points, 10 req/min | One-time trial |

### On-Chain (data only)

| Connector | What's Free | Notes |
|-----------|------------|-------|
| **Etherscan** | Free tier API (limited calls/day) on Sepolia testnet | Testnet constructor in codebase |

---

## Section 4: Tier 3 — Paid Required or Real Account Required

These connectors cannot be tested without a real brokerage account (KYC required) or without paying for API access.

| Connector | Why Tier 3 | Minimum Cost | Notes |
|-----------|-----------|-------------|-------|
| **Crypto.com** | UAT sandbox is institutional/invitation-only | Real account (retail allowed but sandbox blocked) | Public market data is free |
| **Interactive Brokers** | Account required (KYC/ID verification), local TWS/Gateway app must run | IBKR Lite (free, no minimum balance) | Comprehensive paper trading once account open |
| **Tinkoff/T-Bank** | Russian T-Bank brokerage account required | Russian bank customer only | Full sandbox with gRPC once account open |
| **Futu/moomoo** | Futu ID + possible ID verification by region | Free once registered | Paper trading via OpenD daemon |
| **Upstox** | Indian KYC brokerage account required | Indian resident only | Sandbox exists but incomplete (orders only) |
| **Angel One** | Indian KYC + Angel One account | Free API once account open | No testnet |
| **Fyers** | Indian KYC + paid API Bridge for paper trading | API Bridge subscription + account | Least accessible Indian broker |
| **Zerodha** | Indian KYC + Kite Connect subscription (₹500/month for data) | Real Indian account | Order placement API free since 2025 |
| **Coinglass** | API access starts at $29/month | $29/month minimum | Dashboard free; no free API tier |
| **Whale Alert** | 7-day trial only, then paid (pricing not public) | Contact sales | Trial gives REST API access |
| **Glassnode** | API requires $79/mo plan + API add-on (contact sales) | $79/month + addon | Dashboard only on free plan |
| **CryptoQuant** | API starts at $109/month | $109/month | Dashboard free; API is paid |

---

## Section 5: Tier 4 — Blocked/Impossible

| Connector | Why Blocked | Workaround |
|-----------|------------|------------|
| **Yahoo Finance** | No official API; scraping only; fragile and unreliable | Use Finnhub, Polygon, or Twelvedata instead. Do NOT build production connector on Yahoo. |
| **Aylien** | Acquired by Quantexa 2023; no new user signups | Use NewsAPI or GDELT as alternative |
| **MEXC spot API (automation)** | Futures API restricted to institutional accounts for automated trading | Manual futures demo at futures.testnet.mexc.com; spot public market data available |
| **IntoTheBlock** | Sunset/rebranded to Sentora; API access unclear | Check Sentora.io; consider Nansen or Glassnode instead |
| **Bithumb** (for non-Koreans) | Korean KYC required; no testnet; codebase URLs point to mainnet anyway | Public market data available without auth |

---

## Section 6: Recommended Test Order

Ordered by lowest friction to highest friction. Start here for CI/integration testing setup.

### Phase 1 — Zero-Friction Start (no account needed, minutes to running)

1. **DefiLlama** — No key, no signup. `GET https://api.llama.fi/protocols`. Hit it now.
2. **CoinMetrics Community** — No signup. `GET https://community-api.coinmetrics.io/v4/catalog-v2/assets`. Hit it now.
3. **World Bank API** — No key. `GET https://api.worldbank.org/v2/country/US/indicator/NY.GDP.MKTP.CD?format=json`. Hit it now.
4. **IMF Data API** — No key. `GET https://dataservices.imf.org/REST/SDMX_JSON.svc/CompactData/...`. Hit it now.
5. **GDELT** — No key. Hit endpoints directly.
6. **MOEX ISS** — No key. `GET http://iss.moex.com/iss/engines.json`. Hit it now.

### Phase 2 — Email Signup Only (< 10 minutes to running)

7. **FRED v2** — Register at fredaccount.stlouisfed.org. Free key, 120 req/min.
8. **Messari** — Register at messari.io. Free API key, 20 req/min.
9. **Dune Analytics** — Register at dune.com. 2,500 credits/month, API included.
10. **Finnhub** — Register at finnhub.io/register. 60 calls/min, WebSocket on free tier.
11. **NewsAPI** — Register at newsapi.org/register. Key shown instantly.
12. **Nasdaq Data Link** — Register at data.nasdaq.com/sign-up. 50K calls/day free key.

### Phase 3 — Full Trading Testnets, No Real Money (minutes to hours to set up)

13. **Binance testnet** — GitHub OAuth → instant keys. `https://testnet.binance.vision`
14. **OKX demo** — OKX account → Demo API key. Header-based.
15. **Alpaca paper** — Email signup only. Default constructor already uses paper URL.
16. **OANDA practice** — Free registration at oanda.com. Full practice API identical to production.
17. **Bybit testnet** — Email signup → test coin request (24h) → API key.
18. **KuCoin sandbox** — Separate sandbox.kucoin.com account → API key.
19. **Deribit testnet** — No email verification at test.deribit.com. Fastest CEX testnet.
20. **Phemex testnet** — Open registration at testnet.phemex.com. 0.5 BTC virtual.
21. **dYdX v4 testnet** — Wallet + faucet. Cosmos chain. 300 USDC drip.
22. **Dhan sandbox** — Email + mobile at developer.dhanhq.co. No brokerage account needed.
23. **Uniswap Sepolia** — Sepolia ETH from Chainlink faucet. On-chain contracts.

### Phase 4 — Requires Identity or Real Account

24. **Kraken futures demo** — Any email (no verification), instant $50K virtual USD.
25. **Gemini sandbox** — Email registration at exchange.sandbox.gemini.com. 2FA bypass available.
26. **Bitget demo** — Bitget account + demo mode switch.
27. **HyperLiquid testnet** — EVM wallet + $5 mainnet deposit for faucet.
28. **Lighter testnet** — Discord required for test funds.
29. **Paradex testnet** — Discord required for test USDC.
30. **Interactive Brokers paper** — IBKR Lite account (KYC, no minimum) + local TWS/Gateway.

### Phase 5 — Restricted Access (region, institution, or payment)

31. Indian brokers (Zerodha, Angel One, Upstox, Fyers) — Indian KYC
32. Tinkoff sandbox — Russian T-Bank account
33. Futu paper — Futu ID + OpenD daemon
34. Coinglass API — $29/month
35. CryptoQuant API — $109/month
36. Glassnode API — $79/month + addon

---

## Section 7: Codebase Gaps

> **Status update (commit 6c61d70):** Gaps 2-9, 10 have been fixed. Gaps 1, 12 remain open for URL verification. Gaps 11, 13, 14, 15 are documented but not critical.

From `testnet-support-codebase-audit.md` — cases where code claims testnet support but the implementation is incomplete, uses wrong URLs, or ignores the parameter entirely.

### Gap 1: Binance — Wrong Testnet URL

**File:** `src/crypto/cex/binance/endpoints.rs`

**Problem:** Codebase uses `https://testapi.binance.vision` as the spot testnet REST base URL.
**Correct URL:** `https://testnet.binance.vision/api`
**Impact:** Requests to `testapi.binance.vision` will fail or hit wrong environment. The real testnet is at `testnet.binance.vision`, not `testapi.binance.vision`.

---

### Gap 2: HTX — testnet param accepted but silently ignored

**File:** `src/crypto/cex/htx/connector.rs`

**Problem:** Constructor accepts `testnet: bool` and stores it, but all URL functions in `endpoints.rs` hardcode `https://api.huobi.pro`. The `_testnet` parameter is ignored in URL selection.
**Reality:** HTX testnet was shut down. This is correct behavior — but the parameter acceptance is misleading. The connector stores `testnet: bool` in the struct and returns it from `is_testnet()`, meaning callers will think they are on testnet when they are on mainnet.
**Fix needed:** Either remove the testnet param entirely (set `is_testnet()` to hardcoded false and remove the bool from constructor), or add a compile-time warning that testnet is unavailable.

**Status: FIXED** — documented no testnet, flag stored.

---

### Gap 3: Bitget — Incomplete testnet implementation

**File:** `src/crypto/cex/bitget/connector.rs`, `endpoints.rs`

**Problem:** `_testnet: bool` is in the constructor signature but unused (prefixed with `_`). A `TESTNET` const is defined in endpoints.rs but it contains the same URLs as mainnet (`https://api.bitget.com`). `is_testnet()` returns hardcoded `false`.
**Reality:** Bitget demo trading requires `paptrading: 1` header and separate demo API keys, plus different WebSocket domains (`wspap.bitget.com`). None of this is implemented.
**Fix needed:** Either implement real demo support (header injection, WS domain switch) or document the gap and remove the dead `testnet` parameter from the constructor signature.

**Status: FIXED** — header injection + WS domain switch implemented.

---

### Gap 4: BingX — testnet param silently ignored

**File:** `src/crypto/cex/bingx/connector.rs`

**Problem:** `_testnet: bool` in constructor, no field stored, `is_testnet()` hardcoded to `false`, comment says "BingX doesn't have public testnet."
**Reality:** BingX has VST (Virtual USDT) demo pairs on the production endpoint. This is testable. The comment is inaccurate — BingX does have a form of free demo trading.
**Fix needed:** If VST pairs are deemed sufficient for connector testing, implement VST pair support. At minimum, correct the comment.

**Status: FIXED** — testnet flag stored, comment corrected.

---

### Gap 5: Bitfinex — testnet param silently ignored

**File:** `src/crypto/cex/bitfinex/connector.rs`

**Problem:** `_testnet: bool` in constructor, not stored, `is_testnet()` hardcoded `false`.
**Reality:** Bitfinex has paper trading sub-accounts with 2 test pairs (TESTBTCTESTUSD, TESTBTCTESTUSDT) on the same endpoint. The comment says "doesn't have a public testnet" which is misleading.
**Fix needed:** Could implement paper trading pair support. At minimum, clarify the comment to say "paper trading pairs available but not implemented in this connector."

**Status: FIXED** — testnet flag stored, comment corrected.

---

### Gap 6: Bithumb — fake testnet URLs

**File:** `src/crypto/cex/bithumb/endpoints.rs`

**Problem:** `testnet: bool` stored in struct and returned by `is_testnet()`. The `TESTNET` const is defined but contains the same mainnet URLs (`https://api.bithumb.com`). No actual testnet environment exists.
**Reality:** Bithumb has no documented testnet. The parameter is accepted and stored but functionally a no-op — all requests go to mainnet regardless.
**Fix needed:** Same as HTX — either remove the testnet param entirely or document clearly that testnet=true has no effect.

**Status: FIXED** — returns UnsupportedOperation when testnet=true.

---

### Gap 7: Angel One — fake testnet URLs

**File:** `src/stocks/india/angel_one/endpoints.rs`

**Problem:** `testnet: bool` stored, `TESTNET` const defined but same URLs as mainnet.
**Reality:** Angel One has no testnet. The testnet param is a stub with no effect.
**Fix needed:** Remove testnet param or add explicit runtime error when `testnet=true` is passed.

**Status: FIXED** — returns UnsupportedOperation when testnet=true.

---

### Gap 8: Dhan — fake testnet URLs (Dhan sandbox exists but URLs not implemented)

**File:** `src/stocks/india/dhan/endpoints.rs`

**Problem:** `testnet: bool` stored but `TESTNET` const uses same mainnet URLs. However, Dhan's sandbox actually uses the same base URL (`api.dhan.co/v2/`) — the sandbox is differentiated by the API token, not URL.
**Reality:** The implementation is closer to correct than it appears — Dhan sandbox genuinely uses the same URL. But `is_testnet()` returning the stored bool and the constructor accepting the param without token-level validation means there is no enforcement that a sandbox token is used.
**Fix needed:** Document that testnet mode for Dhan means using a sandbox token from developer.dhanhq.co, not a URL change.

**Status: FIXED** — documented sandbox is token-based.

---

### Gap 9: IB (Interactive Brokers) — testnet field hardcoded false

**File:** `src/aggregators/interactive_brokers/connector.rs`

**Problem:** Constructor `from_gateway(base_url, account_id)` creates struct with `testnet: false` hardcoded. `is_testnet()` returns this field — always false.
**Reality:** IB paper trading uses port 7497 (TWS) or 4002 (Gateway) instead of live ports 7496/4001. The URL passed to `from_gateway` is the full base URL including port, so paper trading technically works by passing the paper trading URL. But the `testnet` bool is never set to true and cannot be through the public API.
**Fix needed:** Add a `from_paper_gateway(base_url, account_id)` constructor that sets `testnet=true`, or modify `from_gateway` to infer testnet from the port number.

**Status: FIXED** — added paper() constructor.

---

### Gap 10: FRED — testnet field exists but is always false

**File:** `src/intelligence_feeds/fred/connector.rs`

**Problem:** `testnet: bool` field in struct, hardcoded to `false` in constructor. No testnet constructor.
**Reality:** FRED has no testnet — this is correct behavior. But the field existing suggests incomplete implementation.
**Fix needed:** Remove the field or add a comment clarifying FRED has no testnet.

**Status: Cosmetic/documented** — FRED has no testnet by design; field is a harmless stub.

---

### Gap 11: Kraken spot testnet URL mismatch

**File:** `src/crypto/cex/kraken/endpoints.rs`

**Problem:** When `testnet=true`, the spot REST URL returns `https://api.kraken.com` (production URL). Only the futures URL correctly switches to `https://demo-futures.kraken.com`.
**Reality:** Kraken has no spot testnet — this is correct behavior. But returning the production URL when testnet=true is misleading.
**Fix needed:** When testnet=true is passed for Kraken spot usage, callers should get an error or explicit warning that spot testnet is unavailable, not silently use production.

---

### Gap 12: GateIO spot testnet URL

**File:** `src/crypto/cex/gateio/endpoints.rs`

**Note:** Codebase uses `https://api-testnet.gateapi.io/api/v4` as the spot testnet URL. The research audit found the futures testnet at `https://fx-api-testnet.gateio.ws/api/v4`. The `api-testnet.gateapi.io` domain should be verified — Gate.io has been migrating from `gateio.io` to `gateio.ws` and the testnet domain may have changed.
**Action needed:** Verify `api-testnet.gateapi.io` is still active and correct.

---

### Gap 13: Coinbase — no testnet at all

**File:** `src/crypto/cex/coinbase/connector.rs`

**Problem:** Constructor has no testnet param, `is_testnet()` hardcoded false. Comment says "Coinbase doesn't have testnet for Advanced Trade."
**Reality:** Coinbase Exchange Sandbox (`public-sandbox.exchange.coinbase.com`) and Advanced Trade Sandbox (`api-sandbox.coinbase.com`) both exist. The Advanced Trade sandbox has static responses only, but it does exist.
**Fix needed:** Consider adding sandbox support with `https://api-public.sandbox.exchange.coinbase.com` for the Exchange sandbox (which has real-ish order books). At minimum, update the comment to distinguish between "Advanced Trade" (static sandbox) and "Exchange" (fuller sandbox).

---

### Gap 14: Bitstamp — no testnet support in constructor

**File:** `src/crypto/cex/bitstamp/connector.rs`

**Problem:** Constructor `new(credentials)` has no testnet param. `is_testnet()` hardcoded false. Comment says "Bitstamp doesn't have testnet via this API."
**Reality:** Bitstamp has a sandbox at `sandbox.bitstamp.net` that does exist. The comment is incorrect.
**Fix needed:** Add testnet param and sandbox URL support.

---

### Gap 15: Crypto.com — UAT URL correct but access is gated

**File:** `src/crypto/cex/crypto_com/endpoints.rs`

**Note:** Codebase correctly implements UAT URLs (`uat-api.3ona.co`, `uat-stream.3ona.co`). However, the UAT environment requires institutional invitation. The constructor accepts the testnet flag but in practice, most users cannot authenticate against UAT.
**Not a code bug** — the implementation is correct. Just a documentation/access gap. Consider adding a runtime error or warning if testnet credentials fail UAT auth.

---

### Summary of Gap Severity

| Gap | Severity | Type |
|-----|----------|------|
| Binance wrong URL | High | Wrong URL — requests will fail (open) |
| HTX misleading param | **FIXED** | Documented no testnet, flag stored |
| Bitget incomplete | **FIXED** | Header injection + WS domain switch implemented |
| BingX inaccurate comment | **FIXED** | Testnet flag stored, comment corrected |
| Bitfinex inaccurate comment | **FIXED** | Testnet flag stored, comment corrected |
| Bithumb fake testnet | **FIXED** | Returns UnsupportedOperation when testnet=true |
| Angel One fake testnet | **FIXED** | Returns UnsupportedOperation when testnet=true |
| Dhan URL semantics | **FIXED** | Documented sandbox is token-based |
| IB no paper constructor | **FIXED** | Added paper() constructor (port 4004) |
| FRED dead field | Cosmetic | FRED has no testnet by design; documented |
| Kraken spot silent mainnet | Medium | Silent fallback to production when testnet=true |
| GateIO URL verification needed | Medium | Domain migration may have changed testnet URL (open) |
| Coinbase missing sandbox | Low | Sandbox exists; not implemented |
| Bitstamp missing sandbox | Low | Sandbox exists; not implemented |
| Crypto.com access gated | Info | Implementation correct; access requires institutional approval |
