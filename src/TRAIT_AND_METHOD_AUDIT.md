# Trait and Method Audit — digdigdig3

Generated: 2026-03-14

---

## 1. ALL TRAITS — Complete Method Listing

### CORE TRAITS (required by all full connectors)

#### `ExchangeIdentity` (no auth) — `src/core/traits/identity.rs`
Supertraits: `Send + Sync`
| Method | Signature | Notes |
|--------|-----------|-------|
| `exchange_id` | `fn exchange_id(&self) -> ExchangeId` | required |
| `exchange_name` | `fn exchange_name(&self) -> &'static str` | default → delegates to `exchange_id().as_str()` |
| `is_testnet` | `fn is_testnet(&self) -> bool` | required |
| `supported_account_types` | `fn supported_account_types(&self) -> Vec<AccountType>` | required |
| `exchange_type` | `fn exchange_type(&self) -> ExchangeType` | default → delegates to `exchange_id().exchange_type()` |
| `metrics` | `fn metrics(&self) -> ConnectorStats` | default → returns zeroed metrics |

**Coverage: 56 connectors** (includes connector_manager, coinglass, fred, ib, polymarket, defillama)

---

#### `MarketData` (no auth) — `src/core/traits/market_data.rs`
Supertraits: `ExchangeIdentity`
| Method | Signature | Auth? | Notes |
|--------|-----------|-------|-------|
| `get_price` | `async fn get_price(&self, symbol: Symbol, account_type: AccountType) -> ExchangeResult<Price>` | No | required |
| `get_orderbook` | `async fn get_orderbook(&self, symbol: Symbol, depth: Option<u16>, account_type: AccountType) -> ExchangeResult<OrderBook>` | No | required |
| `get_klines` | `async fn get_klines(&self, symbol: Symbol, interval: &str, limit: Option<u16>, account_type: AccountType, end_time: Option<i64>) -> ExchangeResult<Vec<Kline>>` | No | required |
| `get_ticker` | `async fn get_ticker(&self, symbol: Symbol, account_type: AccountType) -> ExchangeResult<Ticker>` | No | required |
| `ping` | `async fn ping(&self) -> ExchangeResult<()>` | No | required |
| `get_exchange_info` | `async fn get_exchange_info(&self, account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>>` | No | default → `UnsupportedOperation` |

**Coverage: 56 connectors** (same as ExchangeIdentity set)

---

#### `Trading` (auth required) — `src/core/traits/trading.rs`
Supertraits: `ExchangeIdentity`
| Method | Signature | Notes |
|--------|-----------|-------|
| `place_order` | `async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse>` | required |
| `cancel_order` | `async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order>` | required |
| `get_order` | `async fn get_order(&self, symbol: &str, order_id: &str, account_type: AccountType) -> ExchangeResult<Order>` | required |
| `get_open_orders` | `async fn get_open_orders(&self, symbol: Option<&str>, account_type: AccountType) -> ExchangeResult<Vec<Order>>` | required |
| `get_order_history` | `async fn get_order_history(&self, filter: OrderHistoryFilter, account_type: AccountType) -> ExchangeResult<Vec<Order>>` | required |
| `get_user_trades` | `async fn get_user_trades(&self, filter: UserTradeFilter, account_type: AccountType) -> ExchangeResult<Vec<UserTrade>>` | default → `UnsupportedOperation`; ~20/24 CEX override |

**Coverage: 50 connectors**

---

#### `Account` (auth required) — `src/core/traits/account.rs`
Supertraits: `ExchangeIdentity`
| Method | Signature | Notes |
|--------|-----------|-------|
| `get_balance` | `async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>>` | required |
| `get_account_info` | `async fn get_account_info(&self, account_type: AccountType) -> ExchangeResult<AccountInfo>` | required |
| `get_fees` | `async fn get_fees(&self, symbol: Option<&str>) -> ExchangeResult<FeeInfo>` | required; 22/24 (GMX+AMM return UnsupportedOperation) |

**Coverage: 50 connectors**

---

#### `Positions` (auth required) — `src/core/traits/positions.rs`
Supertraits: `ExchangeIdentity`
| Method | Signature | Notes |
|--------|-----------|-------|
| `get_positions` | `async fn get_positions(&self, query: PositionQuery) -> ExchangeResult<Vec<Position>>` | required |
| `get_funding_rate` | `async fn get_funding_rate(&self, symbol: &str, account_type: AccountType) -> ExchangeResult<FundingRate>` | required |
| `modify_position` | `async fn modify_position(&self, req: PositionModification) -> ExchangeResult<()>` | required |
| `get_open_interest` | `async fn get_open_interest(&self, symbol: &str, account_type: AccountType) -> ExchangeResult<OpenInterest>` | default → `UnsupportedOperation`; ~18/24 override |
| `get_funding_rate_history` | `async fn get_funding_rate_history(&self, symbol: &str, start_time: Option<u64>, end_time: Option<u64>, limit: Option<u32>) -> ExchangeResult<Vec<FundingRate>>` | default → `UnsupportedOperation`; ~16/24 override |
| `get_mark_price` | `async fn get_mark_price(&self, symbol: &str) -> ExchangeResult<MarkPrice>` | default → `UnsupportedOperation`; ~18/24 override |
| `get_closed_pnl` | `async fn get_closed_pnl(&self, symbol: Option<&str>, start_time: Option<u64>, end_time: Option<u64>, limit: Option<u32>) -> ExchangeResult<Vec<ClosedPnlRecord>>` | default → `UnsupportedOperation`; ~12/24 override |
| `get_long_short_ratio` | `async fn get_long_short_ratio(&self, symbol: &str, account_type: AccountType) -> ExchangeResult<LongShortRatio>` | default → `UnsupportedOperation`; ~8/24 override |

**Coverage: 44 connectors** (excludes spot-only: Bitstamp, Gemini, Jupiter, Raydium, Uniswap, Mexc spot, etc.)

---

#### `CoreConnector` (composite) — `src/core/traits/mod.rs`
Auto-blanket impl for all `T: ExchangeIdentity + MarketData + Trading + Account + Positions + Send + Sync`.
No new methods. Used as bound for generic code working with any exchange.

---

### WEBSOCKET TRAITS — `src/core/traits/websocket.rs`

#### `WebSocketConnector` (no auth for public streams)
Supertraits: `Send + Sync`
| Method | Signature | Notes |
|--------|-----------|-------|
| `connect` | `async fn connect(&mut self, account_type: AccountType) -> WebSocketResult<()>` | required |
| `disconnect` | `async fn disconnect(&mut self) -> WebSocketResult<()>` | required |
| `connection_status` | `fn connection_status(&self) -> ConnectionStatus` | required |
| `subscribe` | `async fn subscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()>` | required |
| `unsubscribe` | `async fn unsubscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()>` | required |
| `event_stream` | `fn event_stream(&self) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>>` | required |
| `active_subscriptions` | `fn active_subscriptions(&self) -> Vec<SubscriptionRequest>` | required |
| `has_subscription` | `fn has_subscription(&self, request: &SubscriptionRequest) -> bool` | default → checks active_subscriptions |
| `ping_rtt_handle` | `fn ping_rtt_handle(&self) -> Option<Arc<TokioMutex<u64>>>` | default → `None` |

**Coverage: 34 websocket.rs files with `impl WebSocketConnector for`**

#### `WebSocketExt` (convenience, blanket impl over `WebSocketConnector`)
All default implementations delegating to `subscribe()`:
| Method | Public/Private stream |
|--------|-----------------------|
| `subscribe_ticker(symbol)` | Public |
| `subscribe_trades(symbol)` | Public |
| `subscribe_orderbook(symbol)` | Public |
| `subscribe_klines(symbol, interval)` | Public |
| `subscribe_orders()` | Private (auth required) |
| `subscribe_balance()` | Private (auth required) |
| `subscribe_positions()` | Private (auth required) |

---

### EVENT STREAM TRAITS — `src/core/traits/event_stream.rs`

#### `EventFilter` (struct, not a trait)
Fields: `event_types: Vec<String>`, `addresses: Vec<String>`, `token_addresses: Vec<String>`, `min_usd_value: Option<f64>`, `protocols: Vec<String>`
Helper constructors: `is_empty()`, `for_type()`, `for_address()`, `for_protocol()`

#### `EventProducer` (on-chain event production)
Supertraits: `Send + Sync`
| Method | Signature | Notes |
|--------|-----------|-------|
| `chain_id` | `fn chain_id(&self) -> ChainId` | required |
| `get_events` | `async fn get_events(&self, from_block: u64, to_block: u64, filter: &EventFilter) -> Result<Vec<OnChainEvent>, ExchangeError>` | required; historical range |
| `poll_events` | `async fn poll_events(&self, filter: &EventFilter) -> Result<Vec<OnChainEvent>, ExchangeError>` | required; incremental polling |

**Coverage: 0 explicit `impl EventProducer for` found** — trait defined, no connectors implement it yet.

---

### AUTH TRAITS — `src/core/traits/auth.rs`

#### `Authenticated`
Supertraits: `Send + Sync`
| Method | Signature | Notes |
|--------|-----------|-------|
| `set_credentials` | `fn set_credentials(&mut self, creds: ExchangeCredentials)` | required |
| `is_authenticated` | `fn is_authenticated(&self) -> bool` | required |
| `credential_type` | `fn credential_type(&self) -> Option<CredentialKind>` | required |

#### `ExchangeAuth` (internal signing, not public API)
| Method | Signature |
|--------|-----------|
| `sign_request` | `fn sign_request(&self, credentials: &Credentials, req: &mut AuthRequest<'_>) -> ExchangeResult<()>` |
| `signature_location` | `fn signature_location(&self) -> SignatureLocation` (default → `Headers`) |

---

### OPTIONAL OPERATION TRAITS — `src/core/traits/operations.rs`

#### Traits with Hard Supertrait Requirements (no default impls)

| Trait | Supertraits | Methods | Coverage |
|-------|-------------|---------|----------|
| `CancelAll` | `Trading` | `cancel_all_orders(scope, account_type) -> CancelAllResponse` | **22** connectors |
| `AmendOrder` | `Trading` | `amend_order(req: AmendRequest) -> Order` | **25** connectors |
| `BatchOrders` | `Trading` | `place_orders_batch(orders) -> Vec<OrderResult>`, `cancel_orders_batch(order_ids, symbol, account_type) -> Vec<OrderResult>`, `max_batch_place_size() -> usize`, `max_batch_cancel_size() -> usize` | **16** connectors |
| `AccountTransfers` | `Account` | `transfer(req) -> TransferResponse`, `get_transfer_history(filter) -> Vec<TransferResponse>` | **19** connectors |
| `CustodialFunds` | `Account` | `get_deposit_address(asset, network) -> DepositAddress`, `withdraw(req) -> WithdrawResponse`, `get_funds_history(filter) -> Vec<FundsRecord>` | **13** connectors |
| `SubAccounts` | `Account` | `sub_account_operation(op: SubAccountOperation) -> SubAccountResult` | **13** connectors (grep for SubAccounts) |

#### Traits with Default `UnsupportedOperation` Impls (opt-in overrides)

| Trait | Supertraits | Methods | Notes |
|-------|-------------|---------|-------|
| `MarginTrading` | `Send + Sync` | `margin_borrow(asset, amount, account_type)`, `margin_repay(asset, amount, account_type)`, `get_margin_interest(asset)`, `get_margin_account(account_type)` | CEX margin exchanges |
| `EarnStaking` | `Send + Sync` | `get_earn_products(asset)`, `subscribe_earn(product_id, amount)`, `redeem_earn(product_id, amount)`, `get_earn_positions()` | Binance/Bybit/OKX earn |
| `ConvertSwap` | `Send + Sync` | `get_convert_quote(from, to, amount)`, `accept_convert_quote(quote_id)`, `get_convert_history(start, end)`, `convert_dust(assets)` | Binance/Bybit/OKX convert |
| `CopyTrading` | `Send + Sync` | `get_lead_traders(limit)`, `follow_trader(trader_id)`, `stop_following(trader_id)`, `get_copy_positions()` | Bybit/Bitget/BingX |
| `LiquidityProvider` | `Send + Sync` | `create_lp_position(pool_id, amount_a, amount_b)`, `add_liquidity(pos, a, b)`, `remove_liquidity(pos, pct)`, `collect_fees(pos)`, `get_lp_positions()` | Uniswap/Raydium/Jupiter |
| `VaultManager` | `Send + Sync` | `get_vaults()`, `deposit_vault(id, amount)`, `withdraw_vault(id, amount)`, `get_vault_positions()` | GMX/Paradex/dYdX/HyperLiquid |
| `StakingDelegation` | `Send + Sync` | `delegate(validator, amount)`, `undelegate(validator, amount)`, `get_delegations()`, `claim_staking_rewards()` | dYdX/Paradex on-chain |
| `BlockTradeOtc` | `Send + Sync` | `create_block_trade(params)`, `verify_block_trade(params)`, `execute_block_trade(trade_id)`, `get_block_trades()` | Deribit/Bybit/OKX |
| `MarketMakerProtection` | `Send + Sync` | `get_mmp_config()`, `set_mmp_config(config)`, `reset_mmp()`, `mass_quote(quotes)` | Deribit/Bybit/OKX/Paradex/Lighter |
| `TriggerOrders` | `Send + Sync` | `place_trigger_order(params)`, `cancel_trigger_order(order_id)`, `get_trigger_orders(symbol)` | Bybit/OKX/Bitget/BingX/Phemex |
| `PredictionMarket` | `Send + Sync` | `get_prediction_events()`, `get_event_orderbook(event_id)`, `place_prediction_order(params)`, `get_prediction_positions()` | Polymarket/Kalshi |

**Note:** No `impl MarginTrading/EarnStaking/ConvertSwap/etc. for` found in connector.rs files — these optional traits have default impls but zero explicit overrides in current codebase state.

---

## 2. CONNECTOR COUNTS BY TRAIT

| Trait | Count | Category |
|-------|-------|----------|
| `ExchangeIdentity` | 56 | Core |
| `MarketData` | 56 | Core — no auth |
| `Trading` | 50 | Core — auth required |
| `Account` | 50 | Core — auth required |
| `Positions` | 44 | Core — auth required |
| `WebSocketConnector` | 34 | WebSocket |
| `CancelAll` | 22 | Optional execution |
| `AmendOrder` | 25 | Optional execution |
| `BatchOrders` | 16 | Optional execution |
| `AccountTransfers` | 19 | Optional account |
| `CustodialFunds` | 13 | Optional account |
| `SubAccounts` | 13 | Optional account |
| `EventProducer` | 0 | On-chain (defined, unimplemented) |

**Connectors that implement MarketData but NOT Trading (data-only / read-only feeds):**
56 (MarketData) - 50 (Trading) = **6 read-only connectors**:
- `connector_manager/connector.rs`
- `intelligence_feeds/crypto/coinglass/connector.rs`
- `intelligence_feeds/economic/fred/connector.rs`
- `aggregators/ib/connector.rs`
- `prediction/polymarket/connector.rs`
- `aggregators/defillama/connector.rs`

---

## 3. CHAIN PROVIDER USAGE

| Provider | Location | Used By |
|----------|----------|---------|
| `SolanaProvider` | `src/core/chain/solana.rs` | `crypto/swap/raydium/connector.rs`, `crypto/dex/jupiter/connector.rs` |
| `CosmosProvider` | `src/core/chain/cosmos.rs` | `crypto/dex/dydx/connector.rs` |
| `StarkNetProvider` | `src/core/chain/starknet_chain.rs` | `crypto/dex/paradex/connector.rs` |
| `EvmProvider` | `src/core/chain/evm.rs` | `crypto/swap/uniswap/connector.rs`, `crypto/dex/gmx/connector.rs` (via onchain.rs) |
| `BitcoinProvider` | `src/core/chain/bitcoin_chain.rs` | No connector references found |

Other chain modules defined but no connector uses yet:
- `src/core/chain/aptos_chain.rs`
- `src/core/chain/sui_chain.rs`
- `src/core/chain/ton_chain.rs`

---

## 4. TRANSPORT METHODS BY CONNECTOR CATEGORY

| Category | Transport | Notes |
|----------|-----------|-------|
| `crypto/cex/*` (Binance, Bybit, OKX, KuCoin, GateIO, Bitfinex, MEXC, HTX, Bitget, BingX, Phemex, CryptoCom, Kraken, Coinbase, Gemini, Bitstamp, Deribit, HyperLiquid, Upbit, Bithumb, Vertex) | REST + WebSocket | All 21 CEX have both connector.rs and websocket.rs |
| `crypto/dex/dydx` | REST + WebSocket + **gRPC** | Uses `tonic` for Cosmos gRPC; also has WS |
| `crypto/dex/lighter` | REST + WebSocket | StarkEx-based |
| `crypto/dex/paradex` | REST + WebSocket | StarkNet on-chain |
| `crypto/dex/gmx` | REST + WebSocket + **On-chain EVM** | Has `onchain.rs` using EvmProvider |
| `crypto/dex/jupiter` | REST + WebSocket + **On-chain Solana** | Uses SolanaProvider for tx submission |
| `crypto/swap/raydium` | REST + WebSocket + **On-chain Solana** | Uses SolanaProvider |
| `crypto/swap/uniswap` | REST + WebSocket + **On-chain EVM** | Uses EvmProvider |
| `stocks/us/*` (Alpaca, Polygon, Finnhub, Tiingo, Twelvedata) | REST + WebSocket | All have websocket.rs |
| `stocks/india/*` (Upstox, Dhan, Zerodha, Fyers, AngelOne) | REST + WebSocket (Dhan only confirmed) | REST-first; Dhan has websocket.rs |
| `stocks/russia/*` (Tinkoff, MOEX) | REST + WebSocket + **gRPC** (Tinkoff) | Tinkoff uses tonic; MOEX has websocket.rs |
| `stocks/china/futu` | REST | No websocket.rs found |
| `stocks/japan/jquants` | REST only | No websocket.rs |
| `stocks/korea/krx` | REST only | No websocket.rs |
| `forex/*` (OANDA, AlphaVantage, Dukascopy) | REST only | No websocket.rs in forex dir |
| `aggregators/*` (CryptoCompare, Yahoo, IB, DeFiLlama) | REST + WebSocket (CryptoCompare, Yahoo, IB) | IB uses gRPC-like TWS API |
| `onchain/analytics/*` (BitQuery, WhaleAlert) | REST + WebSocket + **GraphQL** (BitQuery) | BitQuery uses GraphQL transport |
| `onchain/ethereum/etherscan` | REST only | |
| `intelligence_feeds/*` | REST only | ~80+ feeds, all HTTP REST |
| `prediction/polymarket` | REST + WebSocket | |

**Transport summary:**
- **REST only**: ~80+ intelligence feeds + etherscan + stocks (china/japan/korea) + forex
- **REST + WebSocket**: All crypto CEX, major stocks brokers, aggregators, polymarket
- **REST + WebSocket + gRPC**: dYdX (Cosmos gRPC), Tinkoff (tonic)
- **REST + WebSocket + On-chain RPC**: GMX (EVM), Jupiter+Raydium (Solana), Uniswap (EVM), Paradex (StarkNet)
- **GraphQL**: BitQuery (alongside REST)

---

## 5. AUTH METHODS BY CONNECTOR

| Auth Method | Connectors |
|-------------|-----------|
| **HMAC-SHA256** (`hmac` crate) | Binance, BingX, Bitfinex, Bitget, Bitstamp, Bithumb, ByBit, CryptoCom, GateIO, Gemini, HTX, KuCoin, MEXC, Phemex, Upbit, Kraken (SHA512), MOEX, KRX, Alpaca, Twelvedata, OKX, CryptoCompare, OANDA, some intel feeds (EIA, SAM, Congress, BEA, FRED, NASA, Coinglass, AlphaVantage) |
| **HMAC-SHA512** | Kraken |
| **HMAC + Passphrase** | OKX, KuCoin, Bitget |
| **ECDSA / Ethereum wallet (EIP-712)** | HyperLiquid, GMX, Uniswap, Vertex, Bitfinex (also uses HMAC), Phemex (legacy), dYdX (also Cosmos) |
| **Ed25519 / Solana keypair** | Raydium, Lighter |
| **STARK key (StarkNet)** | Paradex |
| **Cosmos SDK wallet** | dYdX |
| **JWT ES256 (EC P-256)** | Coinbase, Upbit, Tinkoff, Dhan, Jquants, Polygon, Deribit, OpenSky, SentinelHub, CloudflareRadar |
| **OAuth2 / Bearer token** | Upstox, Fyers, Zerodha, AlphaVantage, OpenSky, Bitquery, WhaleAlert, IB, Deribit (also JWT), SentinelHub |
| **API key in header** (simple bearer / `X-API-Key`) | Most intel feeds, Finnhub, AngelOne, DeFiLlama |
| **No auth (public)** | Most `intelligence_feeds/` (GDELT, WHO, Wikipedia, UN, etc.), open APIs |

---

## 6. CAPABILITY CLASSIFICATION SUMMARY

### DataFeed (no auth) — works without API keys
- `ExchangeIdentity` (all 5 methods, 2 with defaults)
- `MarketData`: `get_price`, `get_orderbook`, `get_klines`, `get_ticker`, `ping`, `get_exchange_info`
- `WebSocketConnector` public streams: ticker, trades, orderbook, klines
- `EventProducer`: `chain_id`, `get_events`, `poll_events` (defined, 0 implementations)
- ~80+ intelligence feed connectors (REST only, no auth or simple API key)

### DataFeed (with auth) — needs API keys, read-only
- `Trading`: `get_order`, `get_open_orders`, `get_order_history`, `get_user_trades`
- `Account`: `get_balance`, `get_account_info`, `get_fees`
- `Positions`: `get_positions`, `get_funding_rate`, `get_open_interest`, `get_funding_rate_history`, `get_mark_price`, `get_closed_pnl`, `get_long_short_ratio`
- `AccountTransfers`: `get_transfer_history`
- `CustodialFunds`: `get_funds_history`, `get_deposit_address` (read-only part)
- `WebSocketConnector` private streams: `subscribe_orders`, `subscribe_balance`, `subscribe_positions`

### Execution — writes data (places orders, manages accounts)
- `Trading`: `place_order`, `cancel_order`
- `CancelAll`: `cancel_all_orders`
- `AmendOrder`: `amend_order`
- `BatchOrders`: `place_orders_batch`, `cancel_orders_batch`
- `AccountTransfers`: `transfer`
- `CustodialFunds`: `withdraw`
- `SubAccounts`: `sub_account_operation` (create/transfer)
- `MarginTrading`: `margin_borrow`, `margin_repay`
- `EarnStaking`: `subscribe_earn`, `redeem_earn`
- `ConvertSwap`: `accept_convert_quote`, `convert_dust`
- `CopyTrading`: `follow_trader`, `stop_following`
- `LiquidityProvider`: `create_lp_position`, `add_liquidity`, `remove_liquidity`, `collect_fees`
- `VaultManager`: `deposit_vault`, `withdraw_vault`
- `StakingDelegation`: `delegate`, `undelegate`, `claim_staking_rewards`
- `BlockTradeOtc`: `create_block_trade`, `verify_block_trade`, `execute_block_trade`
- `MarketMakerProtection`: `set_mmp_config`, `reset_mmp`, `mass_quote`
- `TriggerOrders`: `place_trigger_order`, `cancel_trigger_order`
- `PredictionMarket`: `place_prediction_order`
- `Positions`: `modify_position`

---

## 7. KEY OBSERVATIONS

1. **56 total connectors** implement `ExchangeIdentity`/`MarketData`; **50 implement Trading/Account**; 6 are data-only feeds.
2. **Optional operation traits** (MarginTrading, EarnStaking, ConvertSwap, CopyTrading, LiquidityProvider, VaultManager, StakingDelegation, BlockTradeOtc, MarketMakerProtection, TriggerOrders, PredictionMarket) **have zero explicit `impl X for` blocks** — they are defined with default `UnsupportedOperation` impls, ready for connector-level override.
3. **`EventProducer` is defined but has zero implementations** — the on-chain event layer architecture exists but is not yet wired to any connector.
4. **Chain providers** (SolanaProvider, CosmosProvider, StarkNetProvider, EvmProvider) are optional attachments via builder pattern (`with_solana_provider()`, etc.) — connectors work as REST-only without them.
5. **`Authenticated` trait implementations** are NOT in connector.rs — they would be in the struct definition files. Credential kinds are documented in `CredentialKind` enum: 11 distinct auth schemes across 24+ exchanges.
6. **WebSocket coverage**: 34 out of ~140 total connectors have WS — mainly all crypto CEX/DEX + major stock brokers + a few aggregators.
7. **gRPC transport**: dYdX (Cosmos gRPC via tonic), Tinkoff (TWS-like), BitQuery (GraphQL). All others are REST.
