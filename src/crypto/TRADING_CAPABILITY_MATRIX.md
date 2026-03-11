# Trading Capability Matrix — All 24 Exchanges

Generated: 2026-03-11
Sources: matrix_batch1.md (Binance, Bybit, OKX, KuCoin, Kraken, Coinbase, Gate.io, Bitfinex), matrix_batch2.md (Bitstamp, MEXC, HTX, Bitget, Gemini, BingX, Phemex, Crypto.com), matrix_batch3.md (Upbit, Deribit, HyperLiquid, Lighter, Jupiter, GMX, Paradex, dYdX V4)

Legend: **Y** = Yes | **N** = No | **P** = Partial / limited | **N/A** = Not applicable (structural reason — e.g. non-custodial DEX, spot-only)

---

## 1. Master Tables

### Table A — Order Types

| Exchange | Market | Limit | StopMarket | StopLimit | TrailingStop | TP | SL | OCO | Bracket |
|----------|--------|-------|------------|-----------|--------------|----|----|-----|---------|
| Binance | Y | Y | Y | Y | Y | Y | Y | P (spot) | N |
| Bybit | Y | Y | Y | Y | Y | Y | Y | P (spot) | N |
| OKX | Y | Y | Y | Y | Y | Y | Y | Y | N |
| KuCoin | Y | Y | Y | Y | N | Y | Y | N | N |
| Kraken | Y | Y | Y | Y | Y | Y | Y | N | P (OTO) |
| Coinbase | Y | Y | N | Y | N | P | P | N | Y |
| Gate.io | Y | Y | Y | Y | N | Y | Y | N | N |
| Bitfinex | Y | Y | Y | Y | Y | Y | Y | Y | N |
| Bitstamp | Y | Y | N | Y | N | N | N | N | N |
| MEXC | Y | Y | P (F only) | P (F only) | N | P (F only) | P (F only) | N | N |
| HTX | Y | Y | P (F only) | Y | Y (spot algo) | Y (F inline) | Y (F inline) | N | N |
| Bitget | Y | Y | P (F only) | P (F only) | Y (F only) | Y | Y | N | Y (F OTOCO) |
| Gemini | Y | Y | N | Y | N | N | N | N | N |
| BingX | Y | Y | Y (F only) | Y (F only) | Y (F only) | Y (F only) | Y (F only) | N | P (TP+SL) |
| Phemex | Y | Y | Y (F only) | Y | Y (F only) | Y (F only) | Y (F only) | N | Y (F, 5 orders) |
| Crypto.com | Y | Y | N | N | N | Y (Advanced API) | Y (Advanced API) | Y (Advanced API) | Y (OTOCO) |
| Upbit | Y | Y | N | N | N | N | N | N | N |
| Deribit | Y | Y | Y | Y | Y | Y | Y | Y | Y (OTOCO) |
| HyperLiquid | Y | Y | Y | Y | N | Y | Y | N | P |
| Lighter | Y | Y | Y | Y | N | Y | Y | P | P |
| Jupiter | Y | Y | N | N | N | P (on-chain) | P (on-chain) | N | N |
| GMX | Y | Y | P | N | N | Y | Y | N | P (multicall) |
| Paradex | Y | Y | Y | Y | N | Y | Y | N | N |
| dYdX V4 | Y | Y | Y | Y | P | Y | Y | N | N |

**Column totals (Y only):**
- Market: 24/24
- Limit: 24/24
- StopMarket: 14 Y + 5 P/partial = 19 non-N
- StopLimit: 16 Y + 3 P = 19 non-N
- TrailingStop: 9 Y + 1 P = 10 non-N
- TP: 16 Y + 3 P = 19 non-N
- SL: 16 Y + 3 P = 19 non-N
- OCO: 5 Y + 2 P = 7 non-N
- Bracket: 4 Y + 5 P = 9 non-N

---

### Table B — Time In Force

| Exchange | GTC | IOC | FOK | PostOnly | GTD |
|----------|-----|-----|-----|----------|-----|
| Binance | Y | Y | Y | Y | P (F only) |
| Bybit | Y | Y | Y | Y | N |
| OKX | Y | Y | Y | Y | N |
| KuCoin | Y | Y | Y | Y | Y (GTT) |
| Kraken | Y | Y | N (S) / P (F) | Y | Y |
| Coinbase | Y | Y | Y | Y | Y |
| Gate.io | Y | Y | Y | Y | N |
| Bitfinex | Y | Y | Y | Y | Y |
| Bitstamp | Y | Y | Y | N | Y |
| MEXC | Y | Y | Y | Y | N |
| HTX | Y | Y | Y | Y | N |
| Bitget | Y | Y | Y | Y | N |
| Gemini | Y | Y | Y | Y | N |
| BingX | Y | Y | Y | Y | N |
| Phemex | Y | Y | Y | Y | N |
| Crypto.com | Y | Y | Y | Y | N |
| Upbit | P | Y | Y | Y | N |
| Deribit | Y | Y | Y | Y | Y |
| HyperLiquid | Y | Y | N | Y (ALO) | N |
| Lighter | Y | Y | N | Y | P (GTT) |
| Jupiter | P | N | N | N | Y (expiry) |
| GMX | Y | N | N | N | P |
| Paradex | Y | Y | N | Y | N |
| dYdX V4 | Y | Y | Y | Y | P (GTBT) |

**Column totals (Y only):**
- GTC: 21 Y + 2 P = 23 non-N
- IOC: 21 Y + 0 = 21 non-N
- FOK: 18 Y + 1 P = 19 non-N
- PostOnly: 21 Y + 1 ALO = 22 non-N
- GTD: 7 Y + 4 P = 11 non-N

---

### Table C — Order Management

| Exchange | Single | Batch (max) | Cancel1 | CancelAll | CancelBySymbol | Amend | GetOrder | GetOpen | GetHistory |
|----------|--------|-------------|---------|-----------|----------------|-------|----------|---------|------------|
| Binance | Y | F:5, S:N | Y | Y | Y | Y | Y | Y | Y |
| Bybit | Y | 20/10 | Y | Y | Y | Y | Y | Y | Y |
| OKX | Y | 20 | Y | Y | Y | Y | Y | Y | Y |
| KuCoin | Y | S:5, F:open | Y | Y | Y | P | Y | Y | Y |
| Kraken | Y | S:15 | Y | Y | N | P | Y | Y | Y |
| Coinbase | Y | N | Y | P | N | P | Y | Y | Y |
| Gate.io | Y | S:10 | Y | Y | Y | Y | Y | Y | Y |
| Bitfinex | Y | 75 (mixed) | Y | Y | P | Y | Y | Y | Y |
| Bitstamp | Y | N | Y | Y | Y | P | Y | Y | Y |
| MEXC | Y | S:20, F:50 | Y | Y | Y | N | Y | Y | Y |
| HTX | Y | 10 | Y | Y | Y | N | Y | Y | Y |
| Bitget | Y | 50 | Y | Y | Y | P (F only) | Y | Y | Y |
| Gemini | Y | N | Y | Y | N | N | Y | Y | Y |
| BingX | Y | Y (unspec.) | Y | Y (F) | Y | Y (F only) | Y | Y | Y |
| Phemex | Y | N | Y | Y | Y | Y | Y | Y | Y |
| Crypto.com | Y | 10 | Y | Y | Y | Y | Y | Y | Y |
| Upbit | Y | N | Y | Y | Y | P | Y | Y | Y |
| Deribit | Y | N | Y | Y | Y | Y | Y | Y | Y |
| HyperLiquid | Y | Y (unspec.) | Y | P | N | Y | Y | Y | Y |
| Lighter | Y | 50 | Y | Y | N | Y | P | Y | Y |
| Jupiter | Y | N | Y | Y | Y | N | Y | Y | Y |
| GMX | Y | Y (multicall) | Y | N | N | Y | Y | Y | Y |
| Paradex | Y | 10 | Y | Y | Y | Y | Y | Y | Y |
| dYdX V4 | Y | P | Y | N | N | N | Y | Y | Y |

**Column totals (Y only, not counting P):**
- Single: 24/24
- Batch: 14 Y + 3 P = 17 non-N
- Cancel1: 24/24
- CancelAll: 20 Y + 2 P = 22 non-N
- CancelBySymbol: 16 Y + 1 P = 17 non-N
- Amend: 13 Y + 5 P = 18 non-N
- GetOrder: 23 Y + 1 P = 24 non-N
- GetOpen: 24/24
- GetHistory: 24/24

---

### Table D — Positions (Futures/Derivatives)

N/A entries apply to: Bitstamp (spot only), Upbit (spot only), Jupiter (non-custodial), GMX (non-custodial collateral model).

| Exchange | GetPos | ClosePos | SetLev | MarginMode | AddRemMargin | FundingRate | LiqPrice |
|----------|--------|----------|--------|------------|--------------|-------------|----------|
| Binance | Y | P | Y | Y | Y | Y | Y |
| Bybit | Y | P | Y | Y | Y | Y | Y |
| OKX | Y | Y | Y | Y | Y | Y | Y |
| KuCoin | Y | P | P | Y | Y | Y | Y |
| Kraken | Y | P | Y | N | N | Y | Y |
| Coinbase | Y | N | P | Y | N | Y | Y |
| Gate.io | Y | P | Y | P | Y | Y | Y |
| Bitfinex | Y | P | P | N | N | Y | Y |
| Bitstamp | N/A | N/A | N/A | N/A | N/A | N/A | N/A |
| MEXC | Y | P | Y | Y | N | N | Y |
| HTX | Y | P | Y | Y | N | N | Y |
| Bitget | Y | Y | Y | Y | Y | N | Y |
| Gemini | P | N | N | N | N | Y | N |
| BingX | Y | Y | Y | Y | Y | Y | N |
| Phemex | Y | P | Y | P | Y | Y | Y |
| Crypto.com | Y | Y | Y | P | Y | Y | N |
| Upbit | N/A | N/A | N/A | N/A | N/A | N/A | N/A |
| Deribit | Y | Y | N | P | N | Y | Y |
| HyperLiquid | Y | P | Y | Y | Y | Y | Y |
| Lighter | Y | P | Y | P | N | Y | N |
| Jupiter | P | P | N/A | N/A | N/A | N/A | N/A |
| GMX | Y | Y | N | N | Y | Y | P |
| Paradex | Y | P | P | Y | P | Y | Y |
| dYdX V4 | Y | P | N | Y | Y | Y | P |

**Column totals (Y only, excluding N/A rows):**
Denominator is 22 (excluding Bitstamp, Upbit which are spot-only; Jupiter/GMX are N/A for most).

- GetPos: 18 Y + 2 P (of 22 futures-capable) = strong majority
- ClosePos: 5 Y + 14 P = nearly universal but rarely a dedicated endpoint
- SetLev: 15 Y + 2 P
- MarginMode: 13 Y + 4 P
- AddRemMargin: 11 Y + 1 P
- FundingRate: 17 Y
- LiqPrice: 14 Y + 3 P

---

### Table E — Account

| Exchange | Balances | Fees | InternalTransfer | DepositAddr | Withdraw | Dep/Wdw History |
|----------|----------|------|------------------|-------------|----------|-----------------|
| Binance | Y | Y | Y | Y | Y | Y |
| Bybit | Y | Y | Y | Y | Y | Y |
| OKX | Y | Y | Y | Y | Y | Y |
| KuCoin | Y | Y | Y | Y | Y | Y |
| Kraken | Y | Y | Y | Y | Y | Y |
| Coinbase | Y | Y | Y | N | N | N |
| Gate.io | Y | Y | Y | N | Y | Y |
| Bitfinex | Y | P | Y | N | N | N |
| Bitstamp | Y | Y | P | P | Y | P |
| MEXC | Y | Y | Y | N | N | N |
| HTX | Y | Y | Y | N | N | N |
| Bitget | Y | Y | Y | N | N | N |
| Gemini | Y | Y | Y | Y | Y | Y |
| BingX | Y | Y | Y | Y | Y | P |
| Phemex | Y | Y | Y | Y | Y | Y |
| Crypto.com | Y | Y | Y | Y | Y | Y |
| Upbit | Y | P | N | Y | Y | Y |
| Deribit | Y | Y | Y | Y | Y | Y |
| HyperLiquid | Y | Y | Y | N/A | Y | P |
| Lighter | Y | Y | Y | Y | Y | Y |
| Jupiter | Y | P | N/A | N/A | N/A | N/A |
| GMX | P | Y | N/A | N/A | N/A | P |
| Paradex | Y | P | N | N/A | N/A | Y |
| dYdX V4 | Y | P | Y | N/A | Y | Y |

**Column totals (Y only, all 24):**
- Balances: 23 Y + 1 P = 24/24 non-N
- Fees: 17 Y + 6 P = 23/24 non-N
- InternalTransfer: 16 Y + 1 P (of those where applicable)
- DepositAddr: 11 Y + 1 P (many DEXes are N/A)
- Withdraw: 14 Y (many DEXes are N/A)
- Dep/Wdw History: 14 Y + 3 P

---

### Table F — Sub-accounts

| Exchange | Create | List | Transfer |
|----------|--------|------|----------|
| Binance | Y (broker) | Y (broker) | Y |
| Bybit | N | N | Y |
| OKX | Y | N | Y |
| KuCoin | N | N | Y |
| Kraken | N | N | N |
| Coinbase | N | N | N |
| Gate.io | N | Y | Y |
| Bitfinex | Y | N | Y |
| Bitstamp | P | N | Y |
| MEXC | P | N | N |
| HTX | N | N | N |
| Bitget | N | N | N |
| Gemini | Y | Y | Y |
| BingX | Y | Y | Y |
| Phemex | N | N | Y |
| Crypto.com | N | Y | Y |
| Upbit | N | N | N |
| Deribit | Y | Y | Y |
| HyperLiquid | P | Y | Y |
| Lighter | Y | Y | Y |
| Jupiter | N/A | N/A | N/A |
| GMX | N/A | N/A | N/A |
| Paradex | P | N | N |
| dYdX V4 | Y (auto) | Y | Y |

**Column totals (Y only, excluding N/A):**
- Create: 8 Y + 4 P = 12/22
- List: 9 Y = 9/22
- Transfer: 15 Y = 15/22

---

### Table G — Advanced Features

| Exchange | TWAP | Iceberg | CopyTrading | GridTrading |
|----------|------|---------|-------------|-------------|
| Binance | P (broker) | Y (spot) | N | N |
| Bybit | N | N | P | N |
| OKX | Y | Y | N | N |
| KuCoin | N | Y | N | N |
| Kraken | N | Y (spot) | N | N |
| Coinbase | Y (native) | N | N | Y (scaled) |
| Gate.io | N | Y | N | N |
| Bitfinex | N | Y (hidden) | N | N |
| Bitstamp | N | N | N | N |
| MEXC | N | N | N | N |
| HTX | N | N | N | N |
| Bitget | N | N | N | N |
| Gemini | N | N | N | N |
| BingX | Y | N | P | P |
| Phemex | N | Y (spot) | N | N |
| Crypto.com | N | N | N | N |
| Upbit | N | N | N | N |
| Deribit | N | Y | N | N |
| HyperLiquid | Y | N | P (vaults) | N |
| Lighter | Y | N | N | N |
| Jupiter | N | N | N | N |
| GMX | N | N | N | N |
| Paradex | Y | N | N | N |
| dYdX V4 | Y | N | N | N |

**Column totals:**
- TWAP: 6 Y + 1 P = 7/24
- Iceberg: 8 Y = 8/24
- CopyTrading: 0 Y + 3 P = 3/24
- GridTrading: 0 Y + 1 P + 1 Y (scaled/Coinbase) = 2/24

---

### Table H — Authentication Method

| Exchange | Auth Type | Algorithm | Mechanism |
|----------|-----------|-----------|-----------|
| Binance | API Key + Signature | HMAC-SHA256 | `X-MBX-APIKEY` header + `signature` query param; `timestamp` + `recvWindow` |
| Bybit | API Key + Signature | HMAC-SHA256 | `X-BAPI-API-KEY`, `X-BAPI-SIGN`, `X-BAPI-TIMESTAMP`, `X-BAPI-RECV-WINDOW` headers |
| OKX | API Key + Passphrase + Signature | HMAC-SHA256 | `OK-ACCESS-KEY`, `OK-ACCESS-SIGN`, `OK-ACCESS-TIMESTAMP`, `OK-ACCESS-PASSPHRASE` headers |
| KuCoin | API Key + Passphrase + Signature | HMAC-SHA256 | `KC-API-KEY`, `KC-API-SIGN`, `KC-API-TIMESTAMP`, `KC-API-PASSPHRASE` (passphrase is itself HMAC-SHA256 signed) |
| Kraken | API Key + Signature | HMAC-SHA512 (Spot) / HMAC-SHA256 (Futures) | `API-Key` + `API-Sign` headers; nonce in body |
| Coinbase | JWT Bearer | ES256 (ECDSA P-256) | `Authorization: Bearer <JWT>`; JWT signed with CDP private key |
| Gate.io | API Key + Signature | HMAC-SHA512 | `KEY` and `SIGN` headers; payload = `method\npath\nquery\nbody_hash\ntimestamp` |
| Bitfinex | API Key + Signature | HMAC-SHA384 | `bfx-apikey`, `bfx-signature`, `bfx-nonce` headers; payload = `/api/v2/path` + nonce + body |
| Bitstamp | API Key + Signature | HMAC-SHA256 | Headers; all private calls `application/x-www-form-urlencoded` POST |
| MEXC | API Key + Signature | HMAC-SHA256 | `X-MEXC-APIKEY` header + `signature` query param |
| HTX | API Key + Signature | HMAC-SHA256 | All params in query string including `AccessKeyId`, `SignatureMethod`, `SignatureVersion`, `Timestamp` |
| Bitget | API Key + Passphrase + Signature | HMAC-SHA256 | `ACCESS-KEY`, `ACCESS-SIGN`, `ACCESS-TIMESTAMP`, `ACCESS-PASSPHRASE` headers |
| Gemini | API Key + Payload Signature | HMAC-SHA256 | Payload base64-encoded in `X-GEMINI-PAYLOAD` header; nonce required |
| BingX | API Key + Signature | HMAC-SHA256 | `X-BX-APIKEY` header + `signature` param; timestamp required |
| Phemex | API Key + Signature | HMAC-SHA256 | `x-phemex-access-token`, `x-phemex-request-expiry`, `x-phemex-request-signature` headers |
| Crypto.com | API Key + Signature | HMAC-SHA256 | `api_key` + `sig` in JSON request body; `nonce` (ms timestamp) required |
| Upbit | Bearer JWT | HMAC-SHA256 (JWT payload) | `Authorization: Bearer {jwt_token}`; JWT payload signed with secret |
| Deribit | OAuth client_credentials | — | `public/auth` → `access_token` → Bearer; scopes per operation |
| HyperLiquid | ECDSA per-action | Ethereum ECDSA (secp256k1) | Signs action hash + nonce with Ethereum private key; two schemes: L1 action vs user-signed action |
| Lighter | ZK-signed transactions | STARK-like ZK proof | API private key; `SignerClient` SDK produces ZK signatures; REST reads use `auth` token |
| Jupiter | Solana wallet signing | Ed25519 (Solana) | Private key signs transactions; `x-api-key` header for REST reads |
| GMX | On-chain EVM only | ECDSA (secp256k1) | No REST auth; wallet signs `ExchangeRouter` contract calls; REST API is read-only, no auth |
| Paradex | STARK elliptic curve | STARK EC | Signature `[r,s]` + `signature_timestamp` per order; JWT Bearer for REST reads (obtained via STARK key sign) |
| dYdX V4 | Cosmos wallet | Cosmos secp256k1 | `MsgPlaceOrder` / `MsgCancelOrder` broadcast via gRPC; Indexer REST is fully public, no auth |

---

## 2. Capability Tiers

Counts below include Y only (not P or N/A) unless noted.

### UNIVERSAL (22–24 / 24)

These should be base trait methods — every connector must implement them.

| Capability | Count | Notes |
|-----------|-------|-------|
| Market order | 24/24 | |
| Limit order | 24/24 | |
| Single order placement | 24/24 | |
| Cancel single order | 24/24 | |
| Get open orders | 24/24 | |
| Get order history | 24/24 | |
| Account balances | 23 Y + 1 P / 24 | GMX is partial (ERC-20 balanceOf) |
| GTC time-in-force | 21 Y + 2 P / 24 | |
| IOC time-in-force | 21/24 | |
| PostOnly | 21 Y + 1 ALO / 24 | |
| Get single order | 23 Y + 1 P / 24 | |
| Fees query | 17 Y + 6 P / 24 | 23/24 non-N |

### COMMON (15–21 / 24)

These should be extended trait methods — most connectors implement them.

| Capability | Count | Notes |
|-----------|-------|-------|
| StopMarket order | 14 Y + 5 P = 19 non-N / 24 | |
| StopLimit order | 16 Y + 3 P = 19 non-N / 24 | |
| TP (Take Profit) | 16 Y + 3 P = 19 non-N / 24 | |
| SL (Stop Loss) | 16 Y + 3 P = 19 non-N / 24 | |
| Cancel all orders | 20 Y + 2 P / 24 | |
| Cancel by symbol | 16 Y + 1 P / 24 | |
| Amend/modify order | 13 Y + 5 P = 18 non-N / 24 | |
| FOK time-in-force | 18 Y + 1 P / 24 | |
| Batch order placement | 14 Y + 3 P / 24 | |
| Internal transfer | 16 Y + 1 P / ~20 applicable | |
| Funding rate | 17 Y / 22 futures-capable | |
| Get positions | 18 Y + 2 P / 22 futures-capable | |

### SPECIALIZED (8–14 / 24)

These should be optional trait extensions.

| Capability | Count | Notes |
|-----------|-------|-------|
| TrailingStop | 9 Y + 1 P = 10 / 24 | |
| OCO order | 5 Y + 2 P = 7 / 24 | (see RARE; borderline) |
| GTD time-in-force | 7 Y + 4 P = 11 / 24 | |
| Set leverage | 15 Y + 2 P / 22 futures-capable | |
| Margin mode switch | 13 Y + 4 P / 22 futures-capable | |
| Add/remove margin | 11 Y + 1 P / 22 futures-capable | |
| Liquidation price | 14 Y + 3 P / 22 futures-capable | |
| Close position (dedicated) | 5 Y + 14 P / 22 futures-capable | |
| Deposit address | 11 Y + 1 P / ~18 custodial | |
| Withdraw | 14 Y / ~18 custodial | |
| Deposit/withdraw history | 14 Y + 3 P / ~18 custodial | |
| Iceberg order | 8 Y / 24 | |
| Sub-account transfer | 15 Y / 22 non-DEX | |

### RARE (3–7 / 24)

Exchange-specific optional features.

| Capability | Count | Notes |
|-----------|-------|-------|
| Bracket order | 4 Y + 5 P = 9 non-N / 24 | but most are partial |
| TWAP | 6 Y + 1 P = 7 / 24 | |
| Sub-account create | 8 Y + 4 P / 22 non-DEX | |
| Sub-account list | 9 Y / 22 non-DEX | |
| Copy trading | 0 Y + 3 P / 24 | |
| Grid trading | 1 Y (Coinbase scaled) + 1 P / 24 | |

### ULTRA-RARE (1–2 / 24) — Do not include in traits

| Capability | Count | Notes |
|-----------|-------|-------|
| Block trades / RFQ | 2 (Deribit, Paradex) | |
| Mass quote (market maker) | 1 (Deribit) | |
| Scaled order type | 1 (Coinbase) | |
| Dead man's switch | 2–3 (Kraken, HyperLiquid, HTX) | |
| ZK privacy (Paradex) | 1 | |
| Vault / LP system | 2 (GMX, Lighter) | |
| DCA / Recurring orders | 1 (Jupiter) | |
| Permissioned sub-keys | 2 (dYdX Authenticators, Paradex Subkeys) | |

---

## 3. Exchange Rankings

### Maximum Exchange (Full Implementation Reference)

**Deribit** is the most capable exchange across all categories:
- Full order type suite (Market, Limit, StopMarket, StopLimit, TrailingStop, TP Market, TP Limit, SL, OCO, Bracket/OTOCO, OTO, Market-Limit)
- All TIF (GTC, IOC, FOK, PostOnly, GTD)
- Full order management (amend, batch, cancel all/by-symbol/by-label)
- Full positions (GetPos, Close, FundingRate, LiqPrice)
- Full account (balances, fees, transfer, deposit, withdraw, history)
- Full sub-accounts (create, list, transfer)
- Advanced: Iceberg, BlockTrades, BlockRFQ, MassQuote

**OKX** is the top CEX runner-up:
- Full order types including native OCO algo
- Full TIF except GTD
- All management endpoints including batch amend (20)
- Full positions including dedicated ClosePosition
- Full account
- TWAP + Iceberg advanced features
- Unified Account spanning all products

### Minimum Exchange (Absolute Floor Definition)

**Upbit** has the fewest capabilities:
- Market + Limit orders only (no stop, no trailing, no TP/SL, no OCO, no bracket)
- TIF: IOC, FOK, PostOnly (no GTD, GTC only implicit)
- No batch, no true amend, no sub-accounts
- No futures, no positions
- No internal transfer
- Defines absolute minimum: single create, cancel, get-open, get-history, balances

**GMX** and **Jupiter** are also minimal but for structural reasons (non-custodial DEX).

### All 24 Exchanges Ranked (Most → Fewest Y Capabilities)

Rank is based on approximate total Y count across all tables (excluding N/A which would inflate DEX scores unfairly).

| Rank | Exchange | Type | Approx Y Score | Notable Gaps |
|------|----------|------|----------------|--------------|
| 1 | Deribit | CEX | 48/55 | No SetLev, no true AddMargin |
| 2 | OKX | CEX | 45/55 | No GTD, no sub-account list |
| 3 | Binance | CEX | 44/55 | No bracket, batch spot limited |
| 4 | Bybit | CEX | 43/55 | No OCO (spot only P), no GTD |
| 5 | Bitfinex | CEX | 43/55 | No sub-account list, array API |
| 6 | Gate.io | CEX | 42/55 | No trailing spot, no sub-create |
| 7 | KuCoin | CEX | 41/55 | No trailing, no OCO |
| 8 | Kraken | CEX | 41/55 | No cancel-by-symbol, no margin add |
| 9 | BingX | CEX | 40/55 | Futures-heavy, spot limited |
| 10 | Phemex | CEX | 40/55 | No batch create, no sub-create |
| 11 | Crypto.com | CEX | 39/55 | Stop/SL moved to Advanced API |
| 12 | HyperLiquid | CEX/L1 | 38/55 | No trailing, no FOK, no deposit addr |
| 13 | Lighter | DEX/ZK | 37/55 | No trailing, no FOK, no liq price |
| 14 | Bitget | CEX | 37/55 | Futures-heavy, spot stop N |
| 15 | HTX | CEX | 36/55 | No batch amend, no deposit addr |
| 16 | Paradex | DEX/L2 | 36/55 | No OCO, no bracket, no FOK |
| 17 | dYdX V4 | DEX/Cosmos | 35/55 | No cancel-all, no amend, no deposit addr |
| 18 | Coinbase | CEX | 34/55 | No StopMarket, no cancel-all endpoint |
| 19 | MEXC | CEX | 33/55 | Spot very limited, stop/trailing N |
| 20 | Gemini | CEX | 31/55 | No trailing, no TP/SL, no batch |
| 21 | Bitstamp | CEX | 29/55 | Spot only, many discontinued features |
| 22 | GMX | DEX/EVM | 25/55 | Non-custodial — many N/A by design |
| 23 | Jupiter | DEX/Solana | 22/55 | Non-custodial — many N/A by design |
| 24 | Upbit | CEX | 20/55 | Spot only, minimal feature set |

---

## 4. Proposed Trait Hierarchy

Based on the tier analysis above.

```rust
// ============================================================
// UNIVERSAL — every connector MUST implement
// ============================================================
pub trait BaseTrade {
    /// Place a single order (market or limit)
    async fn place_order(&self, req: OrderRequest) -> Result<OrderResponse>;

    /// Cancel a single order by exchange order ID
    async fn cancel_order(&self, symbol: &str, order_id: &str) -> Result<CancelResponse>;

    /// Get a single order by exchange order ID
    async fn get_order(&self, symbol: &str, order_id: &str) -> Result<Order>;

    /// Get all currently open orders (optionally filtered by symbol)
    async fn get_open_orders(&self, symbol: Option<&str>) -> Result<Vec<Order>>;

    /// Get historical/closed orders with time range filter
    async fn get_order_history(&self, filter: OrderHistoryFilter) -> Result<Vec<Order>>;

    /// Get all account balances
    async fn get_balances(&self) -> Result<Vec<Balance>>;

    /// Get trading fees (maker/taker rates)
    async fn get_fees(&self, symbol: Option<&str>) -> Result<FeeInfo>;
}

// All 24 exchanges implement BaseTrade.


// ============================================================
// COMMON — most connectors implement (15–21/24)
// ============================================================
pub trait ExtendedOrders: BaseTrade {
    /// Cancel all open orders, optionally filtered by symbol
    async fn cancel_all_orders(&self, symbol: Option<&str>) -> Result<CancelAllResponse>;

    /// Amend an existing order (price and/or quantity)
    async fn amend_order(&self, req: AmendOrderRequest) -> Result<OrderResponse>;

    /// Place multiple orders in a single request
    async fn batch_place_orders(&self, orders: Vec<OrderRequest>) -> Result<Vec<OrderResult>>;

    /// Place a stop order (StopMarket or StopLimit)
    async fn place_stop_order(&self, req: StopOrderRequest) -> Result<OrderResponse>;

    /// Place a take-profit or stop-loss order attached to a position
    async fn place_tpsl_order(&self, req: TpSlRequest) -> Result<OrderResponse>;
}

// Exchanges implementing ExtendedOrders (19+ non-N for stop/TP/SL):
// Binance, Bybit, OKX, KuCoin, Kraken, Gate.io, Bitfinex, HTX, Bitget,
// BingX, Phemex, Deribit, HyperLiquid, Lighter, Paradex, dYdX V4,
// MEXC (futures), Crypto.com (Advanced API)


// ============================================================
// FUTURES — connectors with perpetuals/futures positions
// ============================================================
pub trait FuturesTrading: BaseTrade {
    /// Get current open positions
    async fn get_positions(&self, symbol: Option<&str>) -> Result<Vec<Position>>;

    /// Set account or position leverage
    async fn set_leverage(&self, symbol: &str, leverage: u32) -> Result<()>;

    /// Switch between cross and isolated margin mode
    async fn set_margin_mode(&self, symbol: &str, mode: MarginMode) -> Result<()>;

    /// Add or remove isolated margin from a position
    async fn adjust_margin(&self, symbol: &str, amount: Decimal, direction: MarginDirection) -> Result<()>;

    /// Get current and historical funding rates
    async fn get_funding_rate(&self, symbol: &str) -> Result<FundingRate>;

    /// Get funding rate payment history for the account
    async fn get_funding_history(&self, symbol: Option<&str>) -> Result<Vec<FundingPayment>>;
}

// Exchanges implementing FuturesTrading (all 22 futures-capable):
// Binance, Bybit, OKX, KuCoin, Kraken, Coinbase (INTX), Gate.io, Bitfinex,
// MEXC, HTX, Bitget, BingX, Phemex, Crypto.com (PERP),
// Deribit, HyperLiquid, Lighter, Paradex, dYdX V4, GMX
// NOT: Bitstamp, Upbit (spot-only)


// ============================================================
// ACCOUNT_MGMT — custodial exchanges with transfer/withdraw
// ============================================================
pub trait AccountManagement: BaseTrade {
    /// Transfer funds between internal account types (spot/futures/margin)
    async fn internal_transfer(&self, req: TransferRequest) -> Result<TransferResponse>;

    /// Get deposit address for a given asset and network
    async fn get_deposit_address(&self, asset: &str, network: Option<&str>) -> Result<DepositAddress>;

    /// Request a withdrawal to an external address
    async fn withdraw(&self, req: WithdrawRequest) -> Result<WithdrawResponse>;

    /// Get deposit and withdrawal history
    async fn get_deposit_withdraw_history(&self, filter: HistoryFilter) -> Result<Vec<Transfer>>;
}

// Exchanges implementing AccountManagement (~18 custodial CEXes):
// Binance, Bybit, OKX, KuCoin, Kraken, Gate.io, Bitfinex, Bitstamp,
// MEXC, HTX, Bitget, Gemini, BingX, Phemex, Crypto.com, Upbit, Deribit, Lighter
// NOT: Coinbase (no deposit API), HyperLiquid (bridge-based), Jupiter, GMX, Paradex, dYdX (bridge)


// ============================================================
// SUBACCOUNTS — exchanges with programmatic sub-account support
// ============================================================
pub trait SubAccounts: BaseTrade {
    /// Create a sub-account
    async fn create_subaccount(&self, params: SubAccountParams) -> Result<SubAccount>;

    /// List all sub-accounts
    async fn list_subaccounts(&self) -> Result<Vec<SubAccount>>;

    /// Transfer funds to or from a sub-account
    async fn transfer_to_subaccount(&self, req: SubAccountTransfer) -> Result<TransferResponse>;
}

// Exchanges implementing SubAccounts (~10 with full support):
// Binance (broker), OKX, Gemini, BingX, Deribit, Lighter, dYdX V4 (auto-create)
// Partial: Bitfinex, Gate.io, Crypto.com, HyperLiquid, Phemex


// ============================================================
// ADVANCED_ORDERS — specialized order algorithms
// ============================================================
pub trait AdvancedOrders: BaseTrade {
    /// Place a trailing stop order
    async fn place_trailing_stop(&self, req: TrailingStopRequest) -> Result<OrderResponse>;

    /// Place an OCO (One-Cancels-Other) order pair
    async fn place_oco(&self, req: OcoRequest) -> Result<OcoResponse>;

    /// Place a bracket order (entry + TP + SL in one call)
    async fn place_bracket_order(&self, req: BracketRequest) -> Result<BracketResponse>;

    /// Place an iceberg order with visible quantity
    async fn place_iceberg_order(&self, req: IcebergRequest) -> Result<OrderResponse>;

    /// Place a TWAP algorithmic order
    async fn place_twap_order(&self, req: TwapRequest) -> Result<AlgoOrderResponse>;
}

// Exchanges implementing AdvancedOrders (selectively per method):
// TrailingStop: Binance, Bybit, OKX, Kraken, HTX, Bitget, BingX, Phemex, Deribit
// OCO: OKX, Bitfinex, Deribit, Crypto.com (Advanced)
// Bracket: Coinbase, Phemex, Deribit, Crypto.com (OTOCO), Bitget (F)
// Iceberg: OKX, KuCoin, Kraken, Gate.io, Bitfinex, Phemex, Deribit
// TWAP: OKX, Coinbase, BingX, HyperLiquid, Lighter, Paradex, dYdX V4
```

---

## 5. Auth Classification

### HMAC-SHA256 — Standard CEX (16/24)

The dominant pattern. API key + secret, signature over request payload.

| Exchange | Variant Notes |
|----------|---------------|
| Binance | Key in header, sig in query param, `timestamp`+`recvWindow` |
| Bybit | 4 custom headers including recv-window |
| OKX | Key + Passphrase + Signature (3-factor) |
| KuCoin | Key + Passphrase + Signature; passphrase itself is HMAC-SHA256 signed |
| Bitget | Key + Passphrase + Signature (mirrors OKX style) |
| MEXC | Key in header, sig in query param (Binance-compatible) |
| HTX | All params in query string including access key and sig |
| BingX | Key in header, sig as param (Binance-compatible) |
| Bitstamp | POST only, form-encoded body |
| Phemex | 3 custom headers; scaled integer prices |
| Crypto.com | Key + sig in JSON request body (not headers) |
| Gemini | Payload base64-encoded in header (unique variant) |

### HMAC-SHA512 (2/24)

| Exchange | Notes |
|----------|-------|
| Kraken (Spot) | `API-Key` + `API-Sign`; nonce in body; Futures uses SHA256 |
| Gate.io | `KEY` + `SIGN` headers; payload includes body hash |
| Bitfinex | HMAC-SHA384 (distinct — not SHA512); `bfx-apikey`, `bfx-nonce`, `bfx-signature` |

### JWT / OAuth (2/24)

| Exchange | Notes |
|----------|-------|
| Coinbase | ES256 (ECDSA P-256) JWT signed with CDP private key; `Authorization: Bearer <JWT>` |
| Upbit | HMAC-SHA256 signed JWT payload; `Authorization: Bearer {jwt_token}` |
| Deribit | OAuth 2.0 `client_credentials` flow; `public/auth` → `access_token` → Bearer |

### Ethereum ECDSA / EVM Wallet (2/24)

| Exchange | Notes |
|----------|-------|
| HyperLiquid | ECDSA per-action (secp256k1); two signing schemes (L1 action vs user-signed) |
| GMX | On-chain EVM wallet signing only; REST API is read-only, no auth needed |

### Solana / Ed25519 (1/24)

| Exchange | Notes |
|----------|-------|
| Jupiter | Ed25519 Solana wallet signs transactions; `x-api-key` header for REST reads |

### ZK / STARK Signatures (2/24)

| Exchange | Notes |
|----------|-------|
| Lighter | STARK-like ZK proof signatures via `SignerClient` SDK; reads use `auth` token header |
| Paradex | STARK EC curve `[r,s]` signature per order; JWT Bearer for REST reads obtained via STARK key signing |

### Cosmos Wallet (1/24)

| Exchange | Notes |
|----------|-------|
| dYdX V4 | Cosmos secp256k1; `MsgPlaceOrder` broadcast via gRPC; Indexer REST is fully public |

### Auth Classification Summary for ExchangeAuth Trait Design

```
AuthType::HmacSha256 { key, secret, passphrase: Option<String> }
    → Binance, Bybit, OKX, KuCoin, Bitget, MEXC, HTX, BingX, Bitstamp,
      Phemex, Crypto.com, Gemini (+ passphrase = None for most)

AuthType::HmacSha512 { key, secret }
    → Gate.io

AuthType::HmacSha384 { key, secret }
    → Bitfinex

AuthType::HmacSha256Futures { key, secret }
    → Kraken Futures (Spot uses SHA512)

AuthType::JwtCdp { private_key_pem }
    → Coinbase

AuthType::JwtHmac { key, secret }
    → Upbit

AuthType::Oauth2ClientCredentials { client_id, client_secret }
    → Deribit

AuthType::EthereumEcdsa { private_key }
    → HyperLiquid, GMX (on-chain only)

AuthType::SolanaKeypair { private_key }
    → Jupiter

AuthType::StarkKey { private_key }
    → Lighter, Paradex

AuthType::CosmosWallet { mnemonic }
    → dYdX V4
```

---

## Notes on DEX vs CEX Trait Design

For **non-custodial DEXes** (Jupiter, GMX, HyperLiquid, Lighter, Paradex, dYdX V4), many `AccountManagement` methods are structurally inapplicable:
- No deposit address (bridge-based deposits)
- No internal transfer (or it maps to subaccount move)
- No withdrawal API (position close releases collateral, or bridge tx)

These should return `ExchangeError::UnsupportedOperation` rather than panic.

For **on-chain DEXes** (GMX), order placement is a smart contract call — the connector wraps `ethers-rs` / `alloy` and broadcasts transactions. The REST `BaseTrade` trait methods map to on-chain calls instead of HTTP requests.

For **Cosmos-based DEXes** (dYdX V4), mutations are Cosmos SDK transactions broadcast via gRPC. The connector wraps the Cosmos client. Reads hit the Indexer REST API.
