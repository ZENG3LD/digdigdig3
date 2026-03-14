# digdigdig3 Roadmap

Checkbox status tracks real validation against live data or real accounts — not just compile-time success.

---

## Phase 0: Precision Guard (DONE)

f64 → Decimal conversion at Trading trait boundary. Prevents IEEE-754 drift from corrupting order prices.

- [x] Research: f64 accumulation errors, CCXT approach, sub-tick drift analysis
- [x] Core functions: `safe_price()` (round), `safe_qty()` (floor) in `core/utils/precision.rs`
- [x] `PrecisionCache`: thread-safe per-symbol tick/step cache with RwLock
- [x] `SymbolInfo.tick_size` field added
- [x] 23 parsers extract real tick_size from exchange APIs
- [x] PrecisionCache wired into all 19 CEX connectors (place_order, amend_order, batch_orders)
- [x] 17 unit tests for precision functions + cache
- [x] Broker/DEX connectors: Paradex, OANDA, Alpaca, Zerodha wired to PrecisionCache
- [x] DEX with own precision systems: dYdX (MarketConfigCache), GMX (U256 on-chain), Lighter ({:.8} fixed)
- [x] Stubs skipped: Jupiter, Raydium (place_order → UnsupportedOperation)

## Phase 0.5: DEX/Swap Connector Gaps (Priority: HIGH)

Complete unfinished DEX/swap connectors — wire existing endpoints, fix bugs, implement missing swap APIs.
Trading trait is mandatory (CoreConnector supertrait) — AMMs return UnsupportedOperation by design.
Real swap execution lives in struct methods + optional on-chain features.

### GMX — stubs that should be real (data exists in API/Subsquid)
- [ ] Fix `get_funding_rate` — wrongly stubbed, `/markets/info` returns `fundingRateLong/Short`
- [ ] Implement `get_positions` via Subsquid GraphQL (`positions` entity)
- [ ] Implement `get_open_orders` via Subsquid or `Reader.getAccountOrders()` on-chain
- [ ] Implement `get_order_history` via Subsquid `TradeAction` entity
- [ ] Implement `get_order` via Subsquid single-item lookup
- [ ] Wire ERC-20 `balanceOf` into `get_balance` (token addresses already in connector)
- [ ] Verify ExchangeRouter address (discrepancy: `onchain.rs` vs research doc)

### Jupiter — swap APIs not wired
- [ ] Fix Ultra Swap bug: wrong HTTP method (POST→GET) + missing `requestId` in execute
- [ ] Update `get_ultra_balances` endpoint (deprecated `/balances` → `/holdings/{address}`)
- [ ] Wire `get_ultra_balances` into Account trait `get_balance`
- [ ] Implement Trigger API (limit orders): create, cancel, execute, query orders
- [ ] Implement Recurring API (DCA orders): create, cancel, query
- [ ] Add token search/trending/category wrapper methods

### Raydium — swap pipeline missing wrappers
- [ ] Implement `get_swap_quote()` wrapper over `SwapQuoteBaseIn/Out` endpoints
- [ ] Implement `build_swap_transaction()` wrapper over `SwapTransactionBaseIn/Out` endpoints
- [ ] Fix WebSocket: dynamic pool lookup (currently only SOL/USDC hardcoded)
- [ ] Fix WebSocket race condition: broadcast channel init before `event_stream()`
- [ ] Add missing wrappers: token list, farm list, pool list, recommended RPCs, priority fees

### Uniswap — swap flow 80% done, needs final wiring
- [ ] Wire `POST /swap` → `parse_swap_transaction()` (both exist, not connected)
- [ ] Fix WebSocket prices: add decimal scaling (raw wei → human-readable)
- [ ] Fix `get_token_balance()`: dynamic decimals query instead of hardcoded 18
- [ ] Fix `volume_24h` in ticker: currently lifetime volume, not 24h
- [ ] Surface TVL from pool queries (fetched but silently dropped)
- [ ] Wire `POST /approval` endpoint (defined, not called)

### dYdX — small gaps in order types
- [ ] Wire TakeProfit conditional orders (`OrderConditionType::TakeProfit`)
- [ ] Wire long-term orders via `TimeInForce::GoodTilTime` → `build_place_long_term_order_tx`
- [ ] Implement cancel-all helper (loop `get_open_orders` + serial cancel with sequence tracking)

### Lighter — blocked on cryptography
- [ ] Port ECgFp5+Poseidon2 signing from TypeScript SDK to Rust (unblocks all write operations)
- [ ] Fix `get_order` for active orders (currently only scans inactive history)
- [ ] Wire auth token into authenticated read endpoints

## Phase 1: Crypto CEX/DEX Execution Testing (Priority: HIGH)

Testing order placement, cancellation, account queries on real exchanges.
Requires API keys with trading permissions (testnet where available).

### Binance
- [ ] REST: place_order (LIMIT, MARKET)
- [ ] REST: cancel_order
- [ ] REST: get_open_orders
- [ ] REST: get_order_history
- [ ] REST: get_balance
- [ ] REST: get_positions (futures)
- [ ] REST: amend_order
- [ ] REST: cancel_all_orders
- [ ] REST: batch_orders
- [ ] WS: private order updates
- [ ] WS: private balance updates

### Bybit
- [ ] REST: place_order (LIMIT, MARKET)
- [ ] REST: cancel_order
- [ ] REST: get_open_orders
- [ ] REST: get_order_history
- [ ] REST: get_balance
- [ ] REST: get_positions (futures)
- [ ] REST: amend_order
- [ ] REST: cancel_all_orders
- [ ] REST: batch_orders
- [ ] WS: private order updates
- [ ] WS: private balance updates

### OKX
- [ ] REST: place_order (LIMIT, MARKET)
- [ ] REST: cancel_order
- [ ] REST: get_open_orders
- [ ] REST: get_order_history
- [ ] REST: get_balance
- [ ] REST: get_positions (futures)
- [ ] REST: amend_order
- [ ] REST: cancel_all_orders
- [ ] REST: batch_orders
- [ ] WS: private order updates
- [ ] WS: private balance updates

### KuCoin
- [ ] REST: place_order (LIMIT, MARKET)
- [ ] REST: cancel_order
- [ ] REST: get_open_orders
- [ ] REST: get_order_history
- [ ] REST: get_balance
- [ ] REST: get_positions (futures)
- [ ] REST: amend_order
- [ ] REST: cancel_all_orders
- [ ] REST: batch_orders
- [ ] WS: private order updates
- [ ] WS: private balance updates

### Kraken
- [ ] REST: place_order (LIMIT, MARKET)
- [ ] REST: cancel_order
- [ ] REST: get_open_orders
- [ ] REST: get_order_history
- [ ] REST: get_balance
- [ ] REST: get_positions (futures)
- [ ] REST: amend_order
- [ ] REST: cancel_all_orders
- [ ] WS: private order updates
- [ ] WS: private balance updates

### Coinbase
- [ ] REST: place_order (LIMIT, MARKET)
- [ ] REST: cancel_order
- [ ] REST: get_open_orders
- [ ] REST: get_order_history
- [ ] REST: get_balance
- [ ] REST: amend_order (not supported — returns UnsupportedOperation)
- [ ] REST: cancel_all_orders
- [ ] WS: private order updates
- [ ] WS: private balance updates

### Gate.io
- [ ] REST: place_order (LIMIT, MARKET)
- [ ] REST: cancel_order
- [ ] REST: get_open_orders
- [ ] REST: get_order_history
- [ ] REST: get_balance
- [ ] REST: get_positions (futures)
- [ ] REST: amend_order
- [ ] REST: cancel_all_orders
- [ ] REST: batch_orders
- [ ] WS: private order updates
- [ ] WS: private balance updates

### Bitfinex
- [ ] REST: place_order (LIMIT, MARKET)
- [ ] REST: cancel_order
- [ ] REST: get_open_orders
- [ ] REST: get_order_history
- [ ] REST: get_balance
- [ ] REST: get_positions (margin)
- [ ] REST: amend_order
- [ ] REST: cancel_all_orders
- [ ] REST: batch_orders
- [ ] WS: private order updates
- [ ] WS: private balance updates

### Bitstamp
- [ ] REST: place_order (LIMIT, MARKET)
- [ ] REST: cancel_order
- [ ] REST: get_open_orders
- [ ] REST: get_order_history
- [ ] REST: get_balance
- [ ] REST: amend_order
- [ ] WS: private order updates
- [ ] WS: private balance updates

### Gemini
- [ ] REST: place_order (LIMIT, MARKET)
- [ ] REST: cancel_order
- [ ] REST: get_open_orders
- [ ] REST: get_order_history
- [ ] REST: get_balance
- [ ] REST: cancel_all_orders
- [ ] WS: private order updates
- [ ] WS: private balance updates

### MEXC
- [ ] REST: place_order (LIMIT, MARKET)
- [ ] REST: cancel_order
- [ ] REST: get_open_orders
- [ ] REST: get_order_history
- [ ] REST: get_balance
- [ ] REST: get_positions (futures)
- [ ] REST: cancel_all_orders
- [ ] REST: batch_orders
- [ ] WS: private order updates
- [ ] WS: private balance updates

### HTX
- [ ] REST: place_order (LIMIT, MARKET)
- [ ] REST: cancel_order
- [ ] REST: get_open_orders
- [ ] REST: get_order_history
- [ ] REST: get_balance
- [ ] REST: get_positions (futures)
- [ ] REST: cancel_all_orders
- [ ] REST: batch_orders
- [ ] WS: private order updates
- [ ] WS: private balance updates

### Bitget
- [ ] REST: place_order (LIMIT, MARKET)
- [ ] REST: cancel_order
- [ ] REST: get_open_orders
- [ ] REST: get_order_history
- [ ] REST: get_balance
- [ ] REST: get_positions (futures)
- [ ] REST: amend_order
- [ ] REST: cancel_all_orders
- [ ] REST: batch_orders
- [ ] WS: private order updates
- [ ] WS: private balance updates

### BingX
- [ ] REST: place_order (LIMIT, MARKET)
- [ ] REST: cancel_order
- [ ] REST: get_open_orders
- [ ] REST: get_order_history
- [ ] REST: get_balance
- [ ] REST: get_positions (futures)
- [ ] REST: amend_order
- [ ] REST: cancel_all_orders
- [ ] REST: batch_orders
- [ ] WS: private order updates
- [ ] WS: private balance updates

### Crypto.com
- [ ] REST: place_order (LIMIT, MARKET)
- [ ] REST: cancel_order
- [ ] REST: get_open_orders
- [ ] REST: get_order_history
- [ ] REST: get_balance
- [ ] REST: get_positions (futures)
- [ ] REST: amend_order
- [ ] REST: cancel_all_orders
- [ ] REST: batch_orders
- [ ] WS: private order updates
- [ ] WS: private balance updates

### Upbit
- [ ] REST: place_order (LIMIT, MARKET)
- [ ] REST: cancel_order
- [ ] REST: get_open_orders
- [ ] REST: get_order_history
- [ ] REST: get_balance
- [ ] REST: amend_order
- [ ] WS: private order updates
- [ ] WS: private balance updates

### Deribit
- [ ] REST: place_order (LIMIT, MARKET)
- [ ] REST: cancel_order
- [ ] REST: get_open_orders
- [ ] REST: get_order_history
- [ ] REST: get_balance
- [ ] REST: get_positions (options/futures)
- [ ] REST: amend_order
- [ ] REST: cancel_all_orders
- [ ] WS: private order updates
- [ ] WS: private balance updates

### HyperLiquid
- [ ] REST: place_order (LIMIT, MARKET)
- [ ] REST: cancel_order
- [ ] REST: get_open_orders
- [ ] REST: get_order_history
- [ ] REST: get_balance
- [ ] REST: get_positions (perps)
- [ ] REST: amend_order
- [ ] REST: cancel_all_orders
- [ ] REST: batch_orders
- [ ] WS: private order updates
- [ ] WS: private balance updates

### dYdX v4
- [ ] REST/gRPC: place_order (LIMIT, MARKET)
- [ ] REST/gRPC: cancel_order
- [ ] REST: get_open_orders
- [ ] REST: get_order_history
- [ ] REST: get_balance
- [ ] REST: get_positions (perps)
- [ ] REST: cancel_all_orders
- [ ] WS: private order updates

### Lighter
- [ ] place_order_signed() — ZK-native signed order (internal method, not via Trading trait)
- [ ] REST: cancel_order
- [ ] REST: get_open_orders
- [ ] REST: get_order_history
- [ ] REST: get_balance
- [ ] REST: get_positions
- [ ] Expose place_order_signed() through standard Trading trait

### GMX
- [ ] Wire EvmProvider to connector
- [ ] REST: place_order (requires EVM wallet)
- [ ] REST: cancel_order
- [ ] REST: get_positions
- [ ] REST: get_balance

### Paradex
- [ ] REST: place_order (StarkNet signing)
- [ ] REST: cancel_order
- [ ] REST: get_open_orders
- [ ] REST: get_order_history
- [ ] REST: get_balance
- [ ] REST: get_positions

---

## Phase 2: Crypto DataFeed Gaps

Methods that exist but were never validated on real data.

### All CEX (19 active connectors)
- [ ] REST: get_orderbook — validate for all 19 connectors
- [ ] REST: get_recent_trades — implement for the 18 missing, then validate all
- [ ] WS: subscribe_orderbook — validate for all 19 connectors
- [ ] WS: subscribe_klines — validate where supported

### REST ticker (all CEX except Bitstamp and Gemini)
- [ ] Binance: REST get_ticker
- [ ] Bybit: REST get_ticker
- [ ] OKX: REST get_ticker
- [ ] KuCoin: REST get_ticker
- [ ] Kraken: REST get_ticker
- [ ] Coinbase: REST get_ticker
- [ ] Gate.io: REST get_ticker
- [ ] Bitfinex: REST get_ticker
- [ ] MEXC: REST get_ticker
- [ ] HTX: REST get_ticker
- [ ] Bitget: REST get_ticker
- [ ] BingX: REST get_ticker
- [ ] Crypto.com: REST get_ticker
- [ ] Upbit: REST get_ticker
- [ ] Deribit: REST get_ticker
- [ ] HyperLiquid: REST get_ticker
- [ ] dYdX v4: REST get_ticker
- [ ] Lighter: REST get_ticker
- [ ] MOEX ISS: REST get_ticker

### DEX data gaps
- [ ] dYdX: REST get_orderbook — validate on real data
- [ ] dYdX: WS subscribe_orderbook (v4_orderbook channel) — validate
- [ ] Lighter: REST get_orderbook — validate on real data
- [ ] GMX: get_orderbook — impossible (oracle pricing, no orderbook by design)
- [ ] Jupiter: get_klines — impossible (no historical OHLCV API, need Birdeye/CoinGecko)
- [ ] Jupiter: get_orderbook — impossible (aggregator, no native book)

---

## Phase 3: On-Chain Provider Testing

Test chain providers against live nodes. None of this has been exercised.

### EVM (Ethereum and compatible chains)
- [ ] Connect to Ethereum mainnet RPC (QuickNode, Infura, or public)
- [ ] get_height — latest block number
- [ ] get_native_balance — ETH balance for address
- [ ] erc20_balance() — ERC-20 token balance via eth_call
- [ ] get_logs — Transfer event filter over a block range
- [ ] EvmDecoder: decode_block() on a real block (ERC-20 transfers, Uniswap swaps)
- [ ] Log subscription via WebSocket provider

### Bitcoin
- [ ] Connect to Bitcoin RPC (QuickNode or public Electrum-compatible)
- [ ] get_height — current chain tip
- [ ] get_raw_mempool — unconfirmed tx list
- [ ] BitcoinDecoder: decode_block() on a real block (UTXO analysis, coinbase detection)

### Solana
- [ ] Connect to mainnet-beta RPC
- [ ] get_height — current slot
- [ ] get_native_balance — SOL balance for address
- [ ] SolanaDecoder: decode_transaction() on a real tx (SPL transfers, Raydium swap)
- [ ] Account subscription via WebSocket

### Cosmos (Osmosis as test target)
- [ ] Connect to Osmosis LCD endpoint
- [ ] get_all_balances — token balances for address
- [ ] get_pools — liquidity pool list
- [ ] CosmosDecoder: decode_tx_events() on a real tx (IBC packet, governance vote)
- [ ] Tendermint WebSocket subscription

### Aptos
- [ ] Connect to Aptos mainnet REST API
- [ ] get_height — current ledger version
- [ ] get_native_balance — APT balance
- [ ] Module event subscription
- [ ] Coin transfer decode

### StarkNet
- [ ] Connect to StarkNet mainnet RPC
- [ ] get_height — latest block
- [ ] Contract call via starknet_call
- [ ] Event monitoring for a known contract

### Sui
- [ ] Connect to Sui mainnet RPC
- [ ] get_height — latest checkpoint
- [ ] get_native_balance — SUI balance
- [ ] Move event subscription
- [ ] SuiDecoder: DeepBook event decode

### TON
- [ ] Connect to TON mainnet (toncenter or lite-server)
- [ ] get_height — masterchain seqno
- [ ] Jetton transfer detection
- [ ] DEX op-code decode from a real transaction

---

## Phase 4: Authenticated DataFeed Testing (requires paid or sandbox API keys)

Lower priority — most providers require payment or specific account setup.

### Stock Brokers
- [ ] Alpaca: connect with paper trading key, place a paper order
- [ ] Alpaca: get_balance, get_order_history on paper account
- [ ] Tinkoff: connect with sandbox token, place sandbox order
- [ ] Tinkoff: get_balance, get_positions on sandbox account
- [ ] Zerodha (Kite): connect with OAuth token, validate get_open_orders
- [ ] Upstox: connect with OAuth token, validate get_balance
- [ ] Angel One: connect with JWT + TOTP, validate get_balance
- [ ] Dhan: connect with access token, validate get_balance
- [ ] Fyers: connect with JWT, validate get_order_history
- [ ] Futu: connect with OpenD daemon running locally, validate get_balance

### Intelligence Feeds (free tier testing)
- [ ] Coinglass: test liquidations endpoint with free API key
- [ ] Coinglass: test open interest endpoint with free API key
- [ ] Coinglass: test funding rates endpoint with free API key
- [ ] Etherscan: test get_transactions with free API key
- [ ] Etherscan: test get_token_transfers with free API key
- [ ] Whale Alert: test large transaction alerts with free tier
- [ ] CoinGecko: test get_coin_details with no auth (public)
- [ ] CoinGecko: test get_market_chart with no auth
- [ ] DeFiLlama: test get_protocols (no auth, public)
- [ ] DeFiLlama: test get_pools (no auth, public)

### Data Aggregators
- [ ] Yahoo Finance: test get_klines (no auth, scraping-based)
- [ ] Yahoo Finance: test get_ticker (no auth)
- [ ] CryptoCompare: test multi-exchange price aggregation with free API key

---

## Phase 5: Extended Connectors

New connectors and trait completions.

### Missing CEX
- [ ] AscendEX
- [ ] BitMart
- [ ] CoinEx
- [ ] WOO X
- [ ] XT.com
- [ ] LBank
- [ ] HashKey
- [ ] WhiteBIT
- [ ] BTSE
- [ ] BigONE
- [ ] ProBit

### Jupiter (complete implementation)
- [ ] Implement get_klines (currently stub returning UnsupportedOperation)
- [ ] Implement get_orderbook (currently stub)
- [ ] Implement place_order through standard Trading trait
- [ ] Wire SolanaProvider for transaction submission

### EventProducer trait implementations
- [ ] EVM: implement EventProducer — emit OnChainEvent from log subscriptions
- [ ] Solana: implement EventProducer — emit OnChainEvent from account/tx subscriptions
- [ ] Bitcoin: implement EventProducer — emit OnChainEvent from block scanning
- [ ] Cosmos: implement EventProducer — emit OnChainEvent from Tendermint events

### Optional Trait Overrides (per-connector explicit impl blocks)
- [ ] MarginTrading: Binance, Bybit, OKX (all support margin borrow/repay)
- [ ] EarnStaking: Binance, Bybit, OKX (earn/flexible savings products)
- [ ] ConvertSwap: Binance, Bybit (dust conversion, instant swap)
- [ ] VaultManager: GMX, HyperLiquid, Paradex (vault deposit/withdraw)
- [ ] LiquidityProvider: Jupiter, Raydium (LP position management)
- [ ] StakingDelegation: CosmosProvider-backed connectors (dYdX, Osmosis)
- [ ] TriggerOrders: Binance, Bybit, OKX (TP/SL conditional orders)
- [ ] MarketMakerProtection: Binance, Bybit, Deribit (MMP config, mass quoting)
- [ ] BlockTradeOtc: Deribit (OTC block trade creation)
- [ ] CopyTrading: Bybit, Bitget, OKX (follow/unfollow traders)

### Infrastructure
- [ ] Interactive Brokers: wire proper brokerage execution (currently aggregator/Web API mode only)
- [ ] MOEX ISS: verify WebSocket stream works (currently listed as untested on WS)
- [ ] India broker WebSocket: Zerodha, Upstox, Angel One, Fyers — add WS implementations
