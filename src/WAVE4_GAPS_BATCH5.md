# Wave 4 Endpoint Gap Analysis — Batch 5: DEX + Swap Connectors

Analysis of 5 connectors: dYdX v4, GMX, Lighter, Paradex, Jupiter.
Each connector's `endpoints.rs` is compared against official API documentation to identify missing endpoints.

**Files analyzed:**
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\crypto\dex\dydx\endpoints.rs`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\crypto\dex\gmx\endpoints.rs`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\crypto\dex\lighter\endpoints.rs`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\crypto\dex\paradex\endpoints.rs`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\crypto\dex\jupiter\endpoints.rs`

---

## 1. dYdX v4

**Base REST URL:** `https://indexer.dydx.trade/v4`
**WebSocket URL:** `wss://indexer.dydx.trade/v4/ws`
**Transport:** REST (Indexer API) + WebSocket. Note: Order placement/cancellation requires the Cosmos gRPC chain node, NOT the Indexer.
**Source:** https://docs.dydx.xyz/indexer-client/http | https://docs.dydx.xyz/indexer-client/websockets

### REST Endpoint Gap Table

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| General | Server Time | `GET /v4/time` | YES | `DydxEndpoint::ServerTime` |
| General | Block Height | `GET /v4/height` | YES | `DydxEndpoint::BlockHeight` |
| Market Data | Perpetual Markets | `GET /v4/perpetualMarkets` | YES | |
| Market Data | Orderbook | `GET /v4/orderbooks/perpetualMarket/{market}` | YES | |
| Market Data | Trades | `GET /v4/trades/perpetualMarket/{market}` | YES | |
| Market Data | Candles | `GET /v4/candles/perpetualMarkets/{market}` | YES | |
| Market Data | Historical Funding | `GET /v4/historicalFunding/{market}` | YES | |
| Market Data | Sparklines | `GET /v4/sparklines` | YES | |
| Account | Subaccounts for Address | `GET /v4/addresses/{address}` | YES | `DydxEndpoint::Addresses` |
| Account | Specific Subaccount | `GET /v4/addresses/{address}/subaccountNumber/{n}` | YES | |
| Account | Parent Subaccount | `GET /v4/addresses/{address}/parentSubaccountNumber/{n}` | YES | |
| Account | Asset Positions | `GET /v4/assetPositions` | YES | |
| Account | Transfers | `GET /v4/transfers` | YES | |
| Account | Transfers Between | `GET /v4/transfers/between` | **NO** | Get transfers between two subaccounts |
| Account | Trading Rewards | `GET /v4/historicalBlockTradingRewards/{address}` | YES | |
| Account | Aggregated Rewards | `GET /v4/historicalTradingRewardAggregations/{address}` | YES | |
| Positions | Perpetual Positions | `GET /v4/perpetualPositions` | YES | |
| Positions | Parent Positions | `GET /v4/perpetualPositions/parentSubaccountNumber` | YES | |
| Positions | Historical PnL | `GET /v4/historical-pnl` | YES | |
| Positions | Parent Historical PnL | `GET /v4/historical-pnl/parentSubaccountNumber` | YES | |
| Positions | Parent Asset Positions | `GET /v4/assetPositions/parentSubaccountNumber` | **NO** | Asset positions for parent subaccount |
| Funding | Funding Payments | `GET /v4/fundingPayments` | YES | |
| Funding | Parent Funding Payments | `GET /v4/fundingPayments/parentSubaccount` | YES | |
| Trading | Orders List | `GET /v4/orders` | YES | |
| Trading | Specific Order | `GET /v4/orders/{orderId}` | YES | |
| Trading | Fills | `GET /v4/fills` | YES | |
| Trading | Parent Orders | `GET /v4/orders/parentSubaccountNumber` | YES | |
| Trading | Parent Fills | `GET /v4/fills/parentSubaccountNumber` | YES | |
| Trading | Parent Transfers | `GET /v4/transfers/parentSubaccountNumber` | **NO** | Transfers for parent subaccount |
| Compliance | Screen Address | `GET /v4/compliance/screen/{address}` | YES | |
| Vault | MegaVault Historical PnL | `GET /v4/vault/v1/megavault/historicalPnl` | **NO** | Vault product — separate sub-path |
| Vault | MegaVault Positions | `GET /v4/vault/v1/megavault/positions` | **NO** | Active positions held by megavault |
| Vault | All Vaults Historical PnL | `GET /v4/vault/v1/vaults/historicalPnl` | **NO** | Aggregated PnL across all vaults |
| Affiliates | Affiliate Metadata | `GET /v4/affiliates/metadata` | **NO** | Referral tier, earnings metadata |
| Affiliates | Affiliate Total Volume | `GET /v4/affiliates/total_volume` | **NO** | Total referred trading volume |

### WebSocket Channel Gap Table

| Channel | Topic | We Have? | Notes |
|---------|-------|----------|-------|
| Markets | `v4_markets` | **NO** | All perpetual market state updates |
| Orderbook | `v4_orderbook` | **NO** | Per-market bid/ask stream |
| Trades | `v4_trades` | **NO** | Per-market trade tape |
| Candles | `v4_candles` | **NO** | Per-market OHLCV at interval |
| Subaccount | `v4_subaccounts` | **NO** | Orders, fills, position updates for one subaccount |
| Parent Subaccount | `v4_parent_subaccounts` | **NO** | Same as above but for parent subaccounts 0–127 |
| Block Height | `v4_blockheight` | **NO** | Latest block height ticker |

### Transport Notes

- **Order Placement** (place/cancel/modify orders) requires direct gRPC or REST calls to the dYdX chain node (`https://dydx-ops-rpc.kingnodes.com` or similar), NOT the Indexer.
- The Indexer API is **read-only**. All 7 WebSocket channels are currently missing from our implementation.
- Vault endpoints use a nested path prefix `/vault/v1/` under the same Indexer base URL.

---

## 2. GMX

**Base REST URL (Arbitrum):** `https://arbitrum-api.gmxinfra.io`
**Base REST URL (Avalanche):** `https://avalanche-api.gmxinfra.io`
**Fallbacks:** `https://arbitrum-api-fallback.gmxinfra.io`, `https://arbitrum-api-fallback2.gmxinfra.io`
**Botanix chain also supported:** `https://botanix-api.gmxinfra.io`
**Transport:** REST only (no native WebSocket for Indexer; subgraph via The Graph for historical stats)
**Sources:** https://docs.gmx.io/docs/api/rest-v2/ | https://github.com/gmx-io/gmx-stats

### REST Endpoint Gap Table

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| General | Health Check | `GET /ping` | YES | `GmxEndpoint::Ping` |
| Oracle Prices | Latest Tickers | `GET /prices/tickers` | YES | `GmxEndpoint::Tickers` |
| Oracle Prices | Signed Prices | `GET /signed_prices/latest` | YES | `GmxEndpoint::SignedPrices` |
| Oracle Prices | Candles | `GET /prices/candles` | YES | `GmxEndpoint::Candles` — params: `tokenSymbol`, `period` |
| Tokens | Token List | `GET /tokens` | YES | `GmxEndpoint::Tokens` |
| Markets | Markets (basic) | `GET /markets` | YES | `GmxEndpoint::Markets` |
| Markets | Markets Info (detailed) | `GET /markets/info` | YES | `GmxEndpoint::MarketInfo` — liquidity, OI, rates |
| Liquidity | Fee APY | `GET /apy` | YES | `GmxEndpoint::FeeAPY` |
| Liquidity | Annualized Performance | `GET /performance/annualized` | YES | `GmxEndpoint::Performance` |
| GLV Vaults | GLV Token List | `GET /glvs` | YES | `GmxEndpoint::GlvTokens` |
| GLV Vaults | GLV Detailed Info | `GET /glvs/info` | YES | `GmxEndpoint::GlvInfo` |
| GLV Vaults | GLV APY | `GET /glvs/apy` | **NO** | APY specific to GLV vaults |
| Stats | UI Fees | `GET /ui_fees` | **NO** | Fees collected via UI integrations |
| Stats | Position Stats | `GET /positions/stats` | **NO** | Aggregate position stats |
| Stats | Fee Metrics | `GET /fees` | **NO** | Historical fee data by period |
| Stats | Trading Volume | `GET /volumes` | **NO** | Volume by period/token |
| Stats | Account Stats | `GET /accounts` | **NO** | Per-account trading statistics |
| Leaderboard | Leaderboard | `GET /leaderboard` | **NO** | Top traders by PnL (available in gmx-stats backend) |
| Subgraph | Subgraph (The Graph) | GraphQL via The Graph | **NO** | Historical positions, trades, liquidations — separate transport |
| Botanix | Botanix chain support | `https://botanix-api.gmxinfra.io` | **NO** | Third chain URL not in our `GmxUrls` struct |

### WebSocket / Streaming Notes

GMX does not provide a first-party WebSocket. Real-time data is obtained by polling REST endpoints.
- **Polling interval for prices:** ~1–2s against `/prices/tickers`
- **Subgraph (The Graph / Subsquid):** Historical positions, trades, liquidations, volume stats are available via The Graph GraphQL API. Not in our connector.
- The `gmx-stats` backend (`github.com/gmx-io/gmx-stats`) exposes additional analytics but is a community/internal tool, not official API.

### Transport Notes

- Our connector currently handles Arbitrum and Avalanche but misses **Botanix** (Bitcoin sidechain).
- Fallback chain needs to be implemented as retry logic, not just stored URLs.
- Subgraph integration (The Graph) is a separate transport entirely — GraphQL queries.

---

## 3. Lighter

**Base REST URL (Mainnet):** `https://mainnet.zklighter.elliot.ai`
**Base REST URL (Testnet):** `https://testnet.zklighter.elliot.ai`
**WebSocket URL:** `wss://mainnet.zklighter.elliot.ai/stream`
**Transport:** REST + WebSocket
**Sources:** https://apidocs.lighter.xyz | https://apidocs.lighter.xyz/docs/websocket-reference

### REST Endpoint Gap Table

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| General | Status / Root | `GET /` | YES | `LighterEndpoint::Status` |
| General | Exchange Info | `GET /info` | YES | `LighterEndpoint::Info` |
| General | Current Block Height | `GET /api/v1/currentHeight` | YES | |
| Market Data | Order Books (all) | `GET /api/v1/orderBooks` | YES | |
| Market Data | Order Book Details | `GET /api/v1/orderBookDetails` | YES | |
| Market Data | Order Book Orders | `GET /api/v1/orderBookOrders` | YES | |
| Market Data | Recent Trades | `GET /api/v1/recentTrades` | YES | |
| Market Data | Trades History | `GET /api/v1/trades` | YES | |
| Market Data | Candlesticks | `GET /api/v1/candles` | YES | |
| Market Data | Exchange Stats | `GET /api/v1/exchangeStats` | YES | |
| Funding | Funding Rates | `GET /api/v1/funding-rates` | **NO** | Funding rate history per market; path differs from `/api/v1/fundings` |
| Funding | Funding Data | `GET /api/v1/fundings` | YES | `LighterEndpoint::Fundings` — verify exact path matches |
| Account | Account Details | `GET /api/v1/account` | YES | |
| Account | Accounts by L1 Address | `GET /api/v1/accountsByL1Address` | YES | |
| Account | Account Limits | `GET /api/v1/accountLimits` | **NO** | Account tier limits and trading restrictions |
| Account | Account Metadata | `GET /api/v1/accountMetadata` | **NO** | Additional account metadata |
| Account | L1 Metadata | `GET /api/v1/l1Metadata` | **NO** | L1 chain metadata for account |
| Account | PnL Chart | `GET /api/v1/pnl` | YES | |
| Account | Inactive Orders | `GET /api/v1/accountInactiveOrders` | YES | |
| Account | Account Transactions | `GET /api/v1/accountTxs` | YES | |
| Account | Position Funding | `GET /api/v1/positionFunding` | **NO** | Per-position funding payment records |
| Account | Liquidations | `GET /api/v1/liquidations` | **NO** | Account liquidation history |
| Account | Public Pools Metadata | `GET /api/v1/publicPoolsMetadata` | **NO** | Additional metadata for public pools |
| Account | Change Account Tier | `POST /api/v1/changeAccountTier` | **NO** | Modify account tier (authenticated POST) |
| API Keys | API Keys List | `GET /api/v1/apikeys` | YES | |
| API Keys | Next Nonce | `GET /api/v1/nextNonce` | YES | |
| Trading | Send Transaction | `POST /api/v1/sendTx` | YES | |
| Trading | Send Transaction Batch | `POST /api/v1/sendTxBatch` | YES | |
| Deposit | Deposit History | `GET /api/v1/deposit/history` | YES | |
| Deposit | Latest Deposit | `GET /api/v1/deposit/latest` | YES | |
| Withdrawal | Withdrawal History | `GET /api/v1/withdraw/history` | YES | |
| Withdrawal | Withdrawal Delays | `GET /api/v1/withdrawalDelays` | **NO** | Current withdrawal delay info |
| Blockchain | Block | `GET /api/v1/block` | YES | |
| Blockchain | Blocks (list) | `GET /api/v1/blocks` | YES | |
| Blockchain | Transaction | `GET /api/v1/tx` | YES | |
| Blockchain | Transactions | `GET /api/v1/txs` | YES | |
| Blockchain | Block Transactions | `GET /api/v1/blockTxs` | YES | |
| Blockchain | Tx from L1 Hash | `GET /api/v1/txFromL1TxHash` | YES | |
| Misc | Public Pools | `GET /api/v1/publicPools` | YES | |
| Misc | Transfer Fee Info | `GET /api/v1/transferFeeInfo` | YES | |
| Info | Exchange Metrics | `GET /api/v1/exchangeMetrics` | **NO** | Aggregate exchange-level metrics |
| Bridge | Bridge Intent Address | `GET /api/v1/bridge/intentAddress` | **NO** | For cross-chain bridging |
| Bridge | Bridge Info | `GET /api/v1/bridge/info` | **NO** | Bridge configuration details |
| Announcements | System Announcements | `GET /api/v1/announcements` | **NO** | Platform status announcements |
| Notifications | Notifications | `GET /api/v1/notifications` | **NO** | Account notification list |
| Notifications | Acknowledge | `POST /api/v1/notifications/acknowledge` | **NO** | Mark notifications as read |
| Referral | Referral Info | `GET /api/v1/referral` | **NO** | Referral program details for account |
| Fee Credits | Fee Credit Options | `GET /api/v1/feeCredits` | **NO** | LIT lease options for fee reduction |

### WebSocket Channel Gap Table

| Channel | Topic | We Have? | Notes |
|---------|-------|----------|-------|
| Order Book | `order_book/{MARKET_INDEX}` | **NO** | Full order book snapshot + incremental updates every 50ms |
| Best Bid/Offer | `ticker/{MARKET_INDEX}` | **NO** | BBO triggered on every nonce change |
| Market Stats | `market_stats/{MARKET_INDEX}` | **NO** | Funding rate + price statistics per market |
| Market Stats (all) | `market_stats/all` | **NO** | All markets stats in one stream |
| Trades | `trade/{MARKET_INDEX}` | **NO** | Per-market trade stream |
| Spot Market Stats | `spot_market_stats/{MARKET_INDEX}` | **NO** | Spot-specific stats |
| Spot Market Stats (all) | `spot_market_stats/all` | **NO** | All spot market stats |
| Account All | `account_all/{ACCOUNT_ID}` | **NO** | All account market data (requires auth) |
| Account Market | `account_market/{MARKET_ID}/{ACCOUNT_ID}` | **NO** | Account data for specific market (requires auth) |
| Account Stats | `account_stats/{ACCOUNT_ID}` | **NO** | Collateral and leverage stats (requires auth) |
| Account Transactions | `account_tx/{ACCOUNT_ID}` | **NO** | Transaction stream (requires auth) |
| Account All Orders | `account_all_orders/{ACCOUNT_ID}` | **NO** | All orders across markets (requires auth) |
| Account Orders | `account_orders/{MARKET_INDEX}/{ACCOUNT_ID}` | **NO** | Orders per market (requires auth) |
| Account All Trades | `account_all_trades/{ACCOUNT_ID}` | **NO** | Trade history stream (requires auth) |
| Account Assets | `account_all_assets/{ACCOUNT_ID}` | **NO** | Spot asset balances (requires auth) |
| Account Positions | `account_all_positions/{ACCOUNT_ID}` | **NO** | All position data (requires auth) |
| Spot Avg Entry | `account_spot_avg_entry_prices/{ACCOUNT_ID}` | **NO** | Average entry prices for spot assets |
| Pool Data | `pool_data/{ACCOUNT_ID}` | **NO** | Liquidity pool activity (requires auth) |
| Pool Info | `pool_info/{ACCOUNT_ID}` | **NO** | Pool performance metrics (requires auth) |
| Notifications | `notification/{ACCOUNT_ID}` | **NO** | Liquidation/deleverage alerts (requires auth) |
| Block Height | `height` | **NO** | Latest block height stream |

### Transport Notes

- All 20 WebSocket channels are missing from our implementation.
- The `funding-rates` endpoint path (`/api/v1/funding-rates`) may differ from the stored `Fundings` path (`/api/v1/fundings`) — needs verification.
- Market identification uses **numeric indices** (`market_id: 0` = ETH, `1` = BTC, etc.) for WebSocket but symbol names for REST. Mapping already exists in `symbol_to_market_id()`.

---

## 4. Paradex

**Base REST URL:** `https://api.prod.paradex.trade/v1`
**WebSocket URL:** `wss://ws.api.prod.paradex.trade/v1`
**Transport:** REST + WebSocket
**Sources:** https://docs.paradex.trade | Paradex OpenAPI 3.1 spec

### REST Endpoint Gap Table

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Auth | JWT Token | `POST /auth` | YES | `ParadexEndpoint::Auth` |
| Auth | Onboarding | `POST /authentication/onboarding` | **NO** | First-time account setup |
| Auth | Register Sub-key | `POST /subkey/register` | **NO** | Register API sub-key |
| Auth | List Sub-keys | `GET /subkey/list` | **NO** | List registered sub-keys |
| Auth | Revoke Sub-key | `POST /subkey/revoke` | **NO** | Revoke a sub-key |
| Auth | Create Token | `POST /token/create` | **NO** | Create API token |
| Auth | List Tokens | `GET /token/list` | **NO** | List API tokens |
| Auth | Revoke Token | `POST /token/revoke` | **NO** | Revoke API token |
| System | System Config | `GET /system/config` | YES | |
| System | System State | `GET /system/state` | YES | |
| System | Server Time | `GET /system/time` | YES | |
| System | Announcements | `GET /system/announcements` | **NO** | Platform-level announcements |
| Market Data | Markets List | `GET /markets` | YES | |
| Market Data | Markets Summary | `GET /markets/summary` | YES | |
| Market Data | BBO (via path) | `GET /bbo/{market}` | **NO** | Dedicated best-bid-offer endpoint (we only have interactive variant) |
| Market Data | BBO Interactive | `GET /bbo/{market}/interactive` | YES | `ParadexEndpoint::BboInteractive` |
| Market Data | Orderbook | `GET /orderbook/{market}` | YES | |
| Market Data | Orderbook Interactive | `GET /orderbook/{market}/interactive` | YES | |
| Market Data | Markets Funding Data | `GET /markets/funding-data` | **NO** | Historical funding rates for a market |
| Market Data | Market Impact Price | `GET /markets/impact-price` | **NO** | Estimated execution price for a given size |
| Market Data | Trades | `GET /trades` | YES | |
| Market Data | Klines | `GET /klines` | YES | |
| Account | Account Info | `GET /account` | YES | `ParadexEndpoint::Account` |
| Account | Account Details | `GET /account/info` | YES | `ParadexEndpoint::AccountInfo` |
| Account | Account History | `GET /account/history` | YES | |
| Account | Margin Config | `GET /account/margin` | **NO** | Margin mode / leverage configuration |
| Account | Update Margin | `POST /account/margin` | **NO** | Change margin settings |
| Account | Profile | `GET /account/profile` | **NO** | Public profile information |
| Account | Update Profile | `POST /account/profile` | **NO** | Update profile |
| Account | Account Settings | `GET /account/settings` | **NO** | Notification and UI settings |
| Account | Sub-accounts List | `GET /account/sub-accounts` | YES | `ParadexEndpoint::Subaccounts` |
| Account | Account Summaries | `GET /account/accounts` | **NO** | Summarized data for multiple accounts |
| Balances | Balances | `GET /balances` | YES | |
| Positions | Positions | `GET /positions` | YES | |
| Trading | Create Order | `POST /orders` | YES | |
| Trading | Batch Orders | `POST /orders/batch` | YES | |
| Trading | Get Order | `GET /orders/{order_id}` | YES | |
| Trading | Get by Client ID | `GET /orders/by-client-id/{client_id}` | YES | |
| Trading | Open Orders | `GET /orders` | YES | |
| Trading | Orders History | `GET /orders/history` | YES | |
| Trading | Cancel Order | `DELETE /orders/{order_id}` | YES | |
| Trading | Cancel Order by Client ID | `POST /orders/cancel-client/{id}` | **NO** | Cancel using client-assigned order ID |
| Trading | Cancel Batch | `DELETE /orders/batch` | YES | |
| Trading | Cancel All | `DELETE /orders` | YES | |
| Trading | Modify Order | `PUT /orders/{order_id}` | YES | |
| Algo Orders | Create Algo Order | `POST /algo/orders` | YES | |
| Algo Orders | Cancel Algo Order | `DELETE /algo/orders/{algo_id}` | YES | |
| Algo Orders | Open Algo Orders | `GET /algos/open` | **NO** | List active algorithmic orders |
| Algo Orders | Algo Orders History | `GET /algos/history` | **NO** | Historical algo order records |
| Algo Orders | Get Algo Order | `GET /algos/{id}` | **NO** | Fetch a specific algo order |
| Trade History | Fills | `GET /fills` | YES | |
| Trade History | Funding Payments | `GET /funding/payments` | YES | |
| Trade History | Transactions | `GET /transactions` | YES | |
| Trade History | Transfers | `GET /transfers` | YES | |
| Trade History | Liquidations | `GET /liquidations` | YES | |
| Trade History | Tradebusts | `GET /tradebusts` | YES | |
| Block Trades | List Block Trades | `GET /block-trades` | **NO** | OTC/block trade records |
| Block Trades | Create Block Trade | `POST /block-trades/create` | **NO** | Initiate a block trade |
| Block Trades | Get Block Trade | `GET /block-trades/{id}` | **NO** | |
| Block Trades | Cancel Block Trade | `POST /block-trades/{id}/cancel` | **NO** | |
| Block Trades | Execute Block Trade | `POST /block-trades/{id}/execute` | **NO** | |
| Block Offers | List Block Offers | `GET /block-offers` | **NO** | |
| Block Offers | Create Block Offer | `POST /block-offers/create` | **NO** | |
| Block Offers | Get Block Offer | `GET /block-offers/{id}` | **NO** | |
| Block Offers | Cancel Block Offer | `POST /block-offers/{id}/cancel` | **NO** | |
| Block Offers | Execute Block Offer | `POST /block-offers/{id}/execute` | **NO** | |
| Insurance | Insurance Fund | `GET /insurance` | **NO** | Insurance fund balance/state |
| Vaults | List Vaults | `GET /vaults` | **NO** | Available managed vaults |
| Vaults | Create Vault | `POST /vaults/create` | **NO** | |
| Vaults | Vault Summary | `GET /vaults/summary` | **NO** | Aggregated vault stats |
| Vaults | Vault Account Summary | `GET /vaults/account-summary` | **NO** | Per-user vault position |
| Vaults | Vault Balance | `GET /vaults/balance` | **NO** | Vault token balance |
| Vaults | Vault Positions | `GET /vaults/positions` | **NO** | Positions held by vault |
| Vaults | Vault Transfers | `GET /vaults/transfers` | **NO** | Vault deposit/withdraw history |
| Vaults | Vault Historical Data | `GET /vaults/historical-data` | **NO** | Time-series PnL data |
| Vaults | Vault Config | `GET /vaults/config` | **NO** | Vault configuration params |
| XP Transfers | XP Balance | `GET /xp-transfers-v2/balance` | **NO** | User XP/points balance |
| XP Transfers | XP History | `GET /xp-transfers-v2/history` | **NO** | |
| XP Transfers | XP Transfer Details | `GET /xp-transfers-v2/{id}` | **NO** | |
| XP Transfers | Create XP Transfer | `POST /xp-transfers-v2/create` | **NO** | |
| XP Transfers | XP Config | `GET /xp-transfers-v2/config` | **NO** | Fee and config |
| XP Transfers | Public XP History | `GET /xp-transfers-v2/public-history` | **NO** | Public leaderboard of transfers |
| Referrals | Referral Summary | `GET /referrals/summary` | **NO** | Referral stats for account |
| Referrals | Referral QR Code | `GET /referrals/qr-code` | **NO** | QR code for referral link |
| Referrals | Referral Config | `GET /referrals/config` | **NO** | Referral program parameters |

### WebSocket Channel Gap Table

| Channel | Topic | We Have? | Notes |
|---------|-------|----------|-------|
| Account | `account` | **NO** | Account state changes |
| Balance Events | `balance_events` | **NO** | Real-time balance updates |
| Transactions | `transaction` | **NO** | Transaction notifications |
| Transfers | `transfers` | **NO** | Deposit/withdrawal events |
| BBO | `bbo.{market_symbol}` | **NO** | Real-time best bid/offer |
| Markets Summary | `markets_summary` | **NO** | All markets summary stream |
| Markets Summary (single) | `markets_summary.{market_symbol}` | **NO** | Single market summary |
| Trades | `trades.{market_symbol}` | **NO** | Per-market trade stream |
| Order Book (snapshot) | `order_book.{market}.snapshot@15@100ms` | **NO** | Full order book depth feed (format: `order_book.{market}.{feed_type}@{levels}@{refresh_rate}[@{price_tick}]`) |
| Order Book (delta) | `order_book.{market}.delta@15@100ms` | **NO** | Incremental order book updates |
| Funding Data | `funding_data.{market_symbol}` | **NO** | Funding rate stream |
| Orders | `orders.{market_symbol}` | **NO** | Per-market order updates (auth) |
| Positions | `positions` | **NO** | All positions updates (auth) |
| Fills | `fills.{market_symbol}` | **NO** | Fill notifications per market (auth) |
| Funding Payments | `funding_payments.{market_symbol}` | **NO** | Funding payment stream (auth) |
| Tradebusts | `tradebusts` | **NO** | Trade reversal alerts (auth) |

### Transport Notes

- All 16 WebSocket channels are missing from our implementation.
- Paradex uses **StarkNet-based authentication** (Stark ECDSA signatures for JWT). The `POST /auth` endpoint exists but the signing flow for StarkNet keys is not in scope for our Indexer-only work.
- The Vaults and XP Transfer sub-systems are entirely unimplemented (~14 endpoints).
- Block Trades is a major missing category (10 endpoints) representing institutional/OTC flow.

---

## 5. Jupiter

**Location:** `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\crypto\dex\jupiter\endpoints.rs`
**Note:** File is in `crypto/dex/jupiter/` but should be `crypto/swap/jupiter/` per intended module structure.

**Base REST URL:** `https://api.jup.ag`
**Transport:** REST (Solana on-chain for execution)
**Sources:** https://dev.jup.ag/llms.txt | https://dev.jup.ag/api-reference

### REST Endpoint Gap Table

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| **Ultra Swap API (Recommended)** | | | | New unified swap interface — all 5 endpoints missing |
| Ultra | Get Order (Quote + Tx) | `GET /ultra/v1/order` | **NO** | Aggregated quote + unsigned tx in one call |
| Ultra | Execute Order | `POST /ultra/v1/execute` | **NO** | Broadcast signed tx, poll status |
| Ultra | Token Search | `GET /ultra/v1/search` | **NO** | Search by symbol/name/mint |
| Ultra | Holdings / Balances | `GET /ultra/v1/holdings/{address}` | **NO** | SPL token balances for wallet |
| Ultra | Token Shield | `GET /ultra/v1/shield` | **NO** | Security warnings for tokens |
| **Metis Swap API (Advanced / Legacy)** | | | | 2 of 3 endpoints covered |
| Metis | Get Quote | `GET /swap/v1/quote` | YES | `JupiterEndpoint::Quote` |
| Metis | Build Swap Tx | `POST /swap/v1/swap` | YES | `JupiterEndpoint::Swap` |
| Metis | Swap Instructions | `POST /swap/v1/swap-instructions` | YES | `JupiterEndpoint::SwapInstructions` |
| **Price API** | | | | |
| Price | Token Prices | `GET /price/v3` | YES | `JupiterEndpoint::Price` — up to 50 tokens |
| **Tokens API** | | | | Partial — 2 of 4 endpoints present |
| Tokens | Token Search | `GET /tokens/v2/search` | YES | `JupiterEndpoint::TokenSearch` |
| Tokens | Token by Tag | `GET /tokens/v2/tag` | YES | `JupiterEndpoint::TokenTag` |
| Tokens | Token Category / Top Tokens | `GET /tokens/v2/{category}/{interval}` | YES | `JupiterEndpoint::TokenCategory` |
| Tokens | Recently Created | `GET /tokens/v2/recent` | YES | `JupiterEndpoint::TokenRecent` |
| Tokens | Token Metadata | `GET /tokens/v2` | **NO** | Get metadata for specific mint addresses |
| Tokens | Curated Content | `GET /tokens/v2/content` | **NO** | Pro tier — curated token content |
| **Trigger Orders API (Limit Orders v2 — Current)** | | | | Entirely missing |
| Trigger | Auth Challenge | `POST /trigger/v2/auth/challenge` | **NO** | Step 1 of JWT auth |
| Trigger | Auth Verify | `POST /trigger/v2/auth/verify` | **NO** | Step 2 — get JWT from signed challenge |
| Trigger | Get Vault | `GET /trigger/v2/vault` | **NO** | Retrieve existing order vault |
| Trigger | Register Vault | `GET /trigger/v2/vault/register` | **NO** | Register new vault for first-time users |
| Trigger | Craft Deposit | `POST /trigger/v2/deposit/craft` | **NO** | Build unsigned deposit transaction |
| Trigger | Create Order | `POST /trigger/v2/orders/price` | **NO** | Place limit / OCO / OTOCO orders |
| Trigger | Update Order | `PATCH /trigger/v2/orders/price/{orderId}` | **NO** | Modify existing trigger order |
| Trigger | Cancel Order | `POST /trigger/v2/orders/price/cancel/{orderId}` | **NO** | Initiate cancellation |
| Trigger | Confirm Cancel | `POST /trigger/v2/orders/price/confirm-cancel/{orderId}` | **NO** | Confirm order cancellation |
| Trigger | Order History | `GET /trigger/v2/orders/history` | **NO** | Get historical trigger orders |
| **Recurring Orders API (DCA)** | | | | Entirely missing |
| Recurring | Create DCA Order | `POST /recurring/v1/createOrder` | **NO** | Set up time-based or price-based DCA |
| Recurring | Execute Cycle | `POST /recurring/v1/execute` | **NO** | Trigger next DCA execution cycle |
| Recurring | Cancel Order | `POST /recurring/v1/cancelOrder` | **NO** | Stop recurring order |
| Recurring | Get DCA Orders | `GET /recurring/v1/getRecurringOrders` | **NO** | Wallet's active/historical DCA orders |
| **Lending API** | | | | Missing |
| Lend | Deposit to Earn | `POST /lend/v1/earn/deposit` | **NO** | Jupiter Lend vault deposit |
| **Portfolio API** | | | | Missing |
| Portfolio | All DeFi Positions | `GET /portfolio/v1/positions` | **NO** | Aggregate DeFi positions for wallet |
| **Prediction Markets API** | | | | Missing |
| Prediction | List Events | `GET /prediction/events` | **NO** | Active prediction markets |
| Prediction | Search Events | `GET /prediction/events/search` | **NO** | Search by keyword |
| Prediction | Market Details | `GET /prediction/markets/{marketId}` | **NO** | Single market info |
| Prediction | Orderbook | `GET /prediction/orderbook/{marketId}` | **NO** | Bid/ask depth for prediction market |
| Prediction | Create Order | `POST /prediction/orders` | **NO** | Buy prediction market outcome |
| Prediction | Positions | `GET /prediction/positions` | **NO** | Current holdings |
| Prediction | Close Position | `DELETE /prediction/positions/{positionPubkey}` | **NO** | Sell/close position |
| Prediction | Claim Payout | `POST /prediction/positions/{positionPubkey}/claim` | **NO** | Claim winning payout |
| **Send API** | | | | Missing |
| Send | Craft Send | `POST /send/v1/craft-send` | **NO** | Build Send transaction (social token sending) |
| Send | Craft Clawback | `POST /send/v1/craft-clawback` | **NO** | Build clawback transaction |
| Send | Pending Invites | `GET /send/v1/pending-invites` | **NO** | Query pending token invites |
| Send | Invite History | `GET /send/v1/invite-history` | **NO** | Historical send invites |
| **Perps API** | | | | On-chain only (WIP) |
| Perps | Positions / Pool State | On-chain via Solana RPC | **NO** | WIP — no REST API yet; uses Anchor IDL |
| **Trigger V1 (Legacy — being deprecated)** | | | | |
| Trigger v1 | Create Order (Legacy) | `POST /trigger/v1/createOrder` | **NO** | v1 deprecated Nov 2024 |
| Trigger v1 | Execute (Legacy) | `POST /trigger/v1/execute` | **NO** | |
| Trigger v1 | Cancel Order (Legacy) | `POST /trigger/v1/cancelOrder` | **NO** | |
| Trigger v1 | Get Orders (Legacy) | `GET /trigger/v1/getTriggerOrders` | **NO** | |

### Transport Notes

- **Ultra API is the recommended swap path** as of 2025. Metis/Quote-Swap API is still functional but Ultra handles aggregation + execution more robustly.
- **Limit Orders (Trigger API v2)** requires a two-step JWT authentication (challenge + verify) before placing orders. Our connector has no auth mechanism.
- **DCA (Recurring API)** is entirely absent — significant functional gap for systematic strategies.
- **Perps** are on-chain Solana accounts, not REST. Accessing positions requires Anchor IDL parsing of the `JupiterPerpetuals` program — a different transport layer entirely.
- All endpoints require `x-api-key` header (generated at `portal.jup.ag`).
- File is located at `crypto/dex/jupiter/` but logically belongs in `crypto/swap/jupiter/`.

---

## Summary Table — Gaps by Connector

| Connector | Endpoints We Have | Missing REST | Missing WS Channels | Priority Missing |
|-----------|-------------------|-------------|---------------------|-----------------|
| dYdX v4 | 24 | 7 | 7 | Vault endpoints, Affiliates, Transfers Between |
| GMX | 11 | 8 | 0 (no WS) | Stats endpoints, Subgraph, Botanix URL |
| Lighter | 31 | 18 | 20 | Funding rates, Account limits, WS entire layer |
| Paradex | 30 | 55 | 16 | Vaults (9), Block Trades (10), Auth sub-keys (6) |
| Jupiter | 8 | 34 | 0 (no WS) | Ultra API (5), Trigger v2 (9), Recurring/DCA (4) |

### Priority Recommendations

**High Priority (core product functionality):**
1. **Jupiter** — Ultra Swap API (`/ultra/v1/order` + `/ultra/v1/execute`) replaces the existing Metis flow for swap execution
2. **Jupiter** — Trigger v2 API (Limit Orders) for strategy execution
3. **Lighter** — All 20 WebSocket channels (entire streaming layer missing)
4. **Paradex** — All 16 WebSocket channels (entire streaming layer missing)
5. **dYdX** — All 7 WebSocket channels (`v4_markets`, `v4_orderbook`, `v4_trades`, `v4_candles`, `v4_subaccounts`, `v4_parent_subaccounts`, `v4_blockheight`)

**Medium Priority (analytics / account management):**
6. **dYdX** — Vault endpoints (3 missing: MegaVault PnL, positions, vaults PnL)
7. **Paradex** — Vaults sub-system (9 endpoints)
8. **Paradex** — Block Trades (10 endpoints for institutional/OTC flow)
9. **Jupiter** — Recurring/DCA API (4 endpoints)
10. **GMX** — Botanix chain URL + `/glvs/apy`

**Low Priority (non-critical extras):**
11. **dYdX** — Affiliates endpoints
12. **Lighter** — Bridge, Notifications, Referral, Fee Credits
13. **Paradex** — XP Transfers, Referrals
14. **Jupiter** — Prediction Markets, Send, Lend, Portfolio APIs
15. **GMX** — Subgraph (separate GraphQL transport entirely)

---

## Sources

- [dYdX Indexer HTTP API](https://docs.dydx.xyz/indexer-client/http)
- [dYdX Indexer WebSocket API](https://docs.dydx.xyz/indexer-client/websockets)
- [dYdX v4 GitHub API docs](https://github.com/dydxprotocol/v4-chain/blob/main/indexer/services/comlink/public/api-documentation.md)
- [GMX REST API docs](https://docs.gmx.io/docs/api/rest/)
- [GMX REST V2 docs](https://docs.gmx.io/docs/api/rest-v2/)
- [GMX Subgraph](https://github.com/gmx-io/gmx-subgraph)
- [Lighter API docs](https://apidocs.lighter.xyz)
- [Lighter WebSocket reference](https://apidocs.lighter.xyz/docs/websocket-reference)
- [Paradex API docs](https://docs.paradex.trade)
- [Paradex REST endpoints (WebFetch)](https://docs.paradex.trade/)
- [Jupiter Developer docs](https://dev.jup.ag)
- [Jupiter API Reference index](https://dev.jup.ag/llms.txt)
- [Jupiter Ultra Swap overview](https://dev.jup.ag/docs/ultra)
- [Jupiter Recurring API overview](https://dev.jup.ag/docs/recurring-api/)
- [Jupiter Trigger API create-order](https://dev.jup.ag/docs/trigger-api/create-order)
- [Jupiter Perps API overview](https://dev.jup.ag/docs/perps)
