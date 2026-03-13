# WAVE4 Endpoint Gap Analysis — Batch 4

**Exchanges:** Bithumb, Bitstamp, Upbit, Deribit, Vertex Protocol, Hyperliquid
**Date:** 2026-03-13
**Method:** Current `endpoints.rs` compared against official API documentation

---

## 1. Bithumb (Bithumb Pro Global)

**Base URL:** `https://global-openapi.bithumb.pro/openapi/v1`
**Futures URL:** `https://bithumbfutures.com/api/pro/v1`
**WS URL:** `wss://global-api.bithumb.pro/message/realtime`

**Source:** [bithumb-pro official REST docs](https://github.com/bithumb-pro/bithumb.pro-official-api-docs/blob/master/rest-api.md)

### Endpoint Gap Table

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| **General** | Server Time | GET `/serverTime` | YES | `ServerTime` |
| **Spot Market Data** | Ticker | GET `/spot/ticker` | YES | `SpotTicker` |
| **Spot Market Data** | Order Book | GET `/spot/orderBook` | YES | `SpotOrderbook` |
| **Spot Market Data** | Klines | GET `/spot/kline` | YES | `SpotKlines` |
| **Spot Market Data** | Recent Trades | GET `/spot/trades` | YES | `SpotTrades` |
| **Spot Market Data** | Config (instruments) | GET `/spot/config` | YES | `SpotConfig` |
| **Spot Trading** | Place Order | POST `/spot/placeOrder` | YES | `SpotCreateOrder` |
| **Spot Trading** | Cancel Order | POST `/spot/cancelOrder` | YES | `SpotCancelOrder` |
| **Spot Trading** | Order Detail | POST `/spot/orderDetail` | YES | `SpotOrderDetail` |
| **Spot Trading** | Open Orders | POST `/spot/openOrders` | YES | `SpotOpenOrders` |
| **Spot Trading** | History Orders | POST `/spot/historyOrders` | YES | `SpotHistoryOrders` |
| **Spot Trading** | **Batch Cancel Orders** | POST `/spot/cancelOrder/batch` | **NO** | Cancel multiple orders at once |
| **Spot Trading** | **Batch Place Orders** | POST `/spot/placeOrders` | **NO** | Create multiple orders in batch |
| **Spot Trading** | **Asset List** | POST `/spot/assetList` | **NO** | Retrieve account asset balances (different from `/spot/account`) |
| **Spot Trading** | **Single Order** | POST `/spot/singleOrder` | **NO** | Query a specific order by ID |
| **Spot Trading** | **My Trades** | POST `/spot/myTrades` | **NO** | Personal trade history (fills) |
| **Spot Trading** | **Order List** | POST `/spot/orderList` | **NO** | Historical order list (different from open/history) |
| **Spot Account** | Account | POST `/spot/account` | YES | `SpotAccount` |
| **Spot Account** | Deposit Address | POST `/wallet/depositAddress` | YES | `SpotDepositAddress` |
| **Spot Account** | Withdraw | POST `/withdraw` | YES | `SpotWithdraw` |
| **Spot Account** | Deposit History | POST `/wallet/depositHistory` | YES | `SpotDepositHistory` |
| **Spot Account** | Withdrawal History | POST `/wallet/withdrawHistory` | YES | `SpotWithdrawHistory` |
| **Futures Market Data** | Ticker | GET `/ticker` | YES | `FuturesTicker` |
| **Futures Market Data** | Order Book | GET `/depth` | YES | `FuturesOrderbook` |
| **Futures Market Data** | Klines | GET `/barhist` | YES | `FuturesKlines` |
| **Futures Market Data** | Recent Trades | GET `/trades` | YES | `FuturesTrades` |
| **Futures Market Data** | Contracts | GET `/futures/contracts` | YES | `FuturesContracts` |
| **Futures Market Data** | Market Data | GET `/futures/market-data` | YES | `FuturesMarketData` |
| **Futures Market Data** | Funding Rates | GET `/futures/funding-rates` | YES | `FuturesFundingRates` |
| **Futures Trading** | **Place Order** | POST `/futures/order` | **NO** | Create a futures order |
| **Futures Trading** | **Cancel Order** | POST `/futures/order/cancel` | **NO** | Cancel futures order |
| **Futures Trading** | **Batch Place Orders** | POST `/futures/order/batch` | **NO** | Multiple futures orders |
| **Futures Account** | **Futures Position** | POST `/futures/position` | **NO** | Query open positions |
| **Futures Account** | **Futures Account Info** | POST `/futures/account` | **NO** | Futures account balance/margin |
| **Futures Account** | **Adjust Leverage** | POST `/futures/leverageEdit` (or `/contract/leverageEdit`) | **NO** | Set leverage for contract |
| **Futures Account** | **Adjust Margin** | POST `/futures/margin/update` | **NO** | Add/remove margin from position |
| **WebSocket** | TICKER stream | Topic `TICKER:{symbol}` | **NO** | Spot ticker updates |
| **WebSocket** | ORDERBOOK stream | Topic `ORDERBOOK:{symbol}` | **NO** | Spot order book updates |
| **WebSocket** | TRADE stream | Topic `TRADE:{symbol}` | **NO** | Spot trade stream |
| **WebSocket** | CONTRACT_TICKER stream | Topic `CONTRACT_TICKER:{symbol}` | **NO** | Futures ticker (funding rate, OI) |
| **WebSocket** | CONTRACT_ORDERBOOK stream | Topic `CONTRACT_ORDERBOOK:{symbol}` | **NO** | Futures order book |
| **WebSocket** | ORDER stream (private) | Topic `ORDER` | **NO** | Spot order updates |
| **WebSocket** | CONTRACT_ORDER stream (private) | Topic `CONTRACT_ORDER` | **NO** | Futures order updates |
| **WebSocket** | CONTRACT_ASSET (private) | Topic `CONTRACT_ASSET` | **NO** | Futures balance updates |
| **WebSocket** | CONTRACT_POSITION (private) | Topic `CONTRACT_POSITION` | **NO** | Position changes |

**Summary:** We have all core spot endpoints (5 market data, 5 trading, 5 account). Missing: 7 additional spot trading endpoints (batch orders, single order lookup, trades history, asset list), all futures trading/account endpoints (~5), and all WebSocket streams (9 topics).

---

## 2. Bitstamp

**Base URL:** `https://www.bitstamp.net`
**WS URL:** `wss://ws.bitstamp.net`

**Source:** [Bitstamp API](https://www.bitstamp.net/api/), [node-bitstamp](https://github.com/krystianity/node-bitstamp)

### Endpoint Gap Table

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| **Market Data** | Ticker | GET `/api/v2/ticker/{pair}/` | YES | `Ticker` |
| **Market Data** | Order Book | GET `/api/v2/order_book/{pair}/` | YES | `Orderbook` |
| **Market Data** | Transactions | GET `/api/v2/transactions/{pair}/` | YES | `Transactions` |
| **Market Data** | OHLC | GET `/api/v2/ohlc/{pair}/` | YES | `Ohlc` |
| **Market Data** | Markets | GET `/api/v2/markets/` | YES | `Markets` |
| **Market Data** | Currencies | GET `/api/v2/currencies/` | YES | `Currencies` |
| **Market Data** | **Hourly Ticker** | GET `/api/v2/ticker_hour/{pair}/` | **NO** | Hourly aggregated ticker |
| **Market Data** | **Order Data** | POST `/api/v2/order_data/` | **NO** | Historical public order events for a market |
| **Market Data** | **Account Order Data** | POST `/api/v2/account_order_data/` | **NO** | Historical order events for authenticated user |
| **Account** | Balance | POST `/api/v2/account_balances/` | YES | `Balance` |
| **Account** | Account Info | POST `/api/v2/balance/` | YES | `AccountInfo` (legacy) |
| **Account** | Trading Fees | POST `/api/v2/fees/trading/` | YES | `TradingFees` |
| **Account** | User Transactions | POST `/api/v2/user_transactions/` | YES | `UserTransactions` |
| **Account** | **Withdrawal Fees** | POST `/api/v2/fees/withdrawal/` | **NO** | Withdrawal fee schedule |
| **Account** | **API Keys Info** | POST `/api/v2/api-key/` | **NO** | Active API key info (read-only) |
| **Trading** | Buy Limit | POST `/api/v2/buy/{pair}/` | YES | `BuyLimit` |
| **Trading** | Sell Limit | POST `/api/v2/sell/{pair}/` | YES | `SellLimit` |
| **Trading** | Buy Market | POST `/api/v2/buy/market/{pair}/` | YES | `BuyMarket` |
| **Trading** | Sell Market | POST `/api/v2/sell/market/{pair}/` | YES | `SellMarket` |
| **Trading** | Cancel Order | POST `/api/v2/cancel_order/` | YES | `CancelOrder` |
| **Trading** | Cancel All Orders | POST `/api/v2/cancel_all_orders/` | YES | `CancelAllOrders` |
| **Trading** | Replace Order | POST `/api/v2/replace_order/` | YES | `ReplaceOrder` |
| **Trading** | Order Status | POST `/api/v2/order_status/` | YES | `OrderStatus` |
| **Trading** | Open Orders | POST `/api/v2/open_orders/all/` | YES | `OpenOrders` |
| **Trading** | Open Orders (pair) | POST `/api/v2/open_orders/{pair}/` | **PARTIAL** | `OpenOrders` only covers `/all/` variant |
| **Trading** | **Order History** | POST `/api/v2/order_history/{pair}/` | **NO** | Filled/cancelled order history per pair |
| **Trading** | **Instant Buy** | POST `/api/v2/buy/instant/{pair}/` | **NO** | Instant buy (market-like execution) |
| **Trading** | **Instant Sell** | POST `/api/v2/sell/instant/{pair}/` | **NO** | Instant sell |
| **Stop Limit** | Buy Stop Limit | POST `/api/v2/buy/stop_limit/{pair}/` | YES | `BuyStopLimit` |
| **Stop Limit** | Sell Stop Limit | POST `/api/v2/sell/stop_limit/{pair}/` | YES | `SellStopLimit` |
| **Stop Limit** | **Buy Daily Order** | POST `/api/v2/buy/{pair}/` with `daily_order=true` | **PARTIAL** | Parameter on BuyLimit, not a separate endpoint |
| **Perpetuals** | Open Positions | POST `/api/v2/open_positions/` | YES | `OpenPositions` |
| **Custodial** | Deposit Address | POST `/api/v2/deposit-address/{currency}/` | YES | `DepositAddress` |
| **Custodial** | Withdrawal | POST `/api/v2/{currency}_withdrawal/` | YES | `Withdrawal` |
| **Custodial** | Withdrawal Requests | POST `/api/v2/withdrawal-requests/` | YES | `WithdrawalRequests` |
| **Custodial** | **Unconfirmed BTC Deposits** | GET `/api/v2/btc_unconfirmed_transactions/` | **NO** | Unconfirmed Bitcoin deposit transactions |
| **Custodial** | **Transfer to Sub** | POST `/api/v2/transfer-to-subaccount/` | **NO** | Transfer funds to sub-account |
| **Custodial** | **Transfer from Sub** | POST `/api/v2/transfer-from-subaccount/` | **NO** | Transfer funds from sub-account |
| **Custodial** | **Bank Withdrawal** | POST `/api/v2/withdrawal/open/` | **NO** | Initiate bank (SEPA/SWIFT) withdrawal |
| **WebSocket** | Live Trades | Channel `live_trades_{pair}` | **NO** | Real-time trade stream |
| **WebSocket** | Live Orders | Channel `live_orders_{pair}` | **NO** | Real-time order book stream |
| **WebSocket** | Order Book | Channel `order_book_{pair}` | **NO** | Full order book snapshot + updates |
| **WebSocket** | Order Book Diff | Channel `diff_order_book_{pair}` | **NO** | Order book diff updates |
| **WebSocket** | Detail Order Book | Channel `detail_order_book_{pair}` | **NO** | Detailed order book (with order IDs) |
| **WebSocket** | Private Orders (auth) | Channel `private-my_orders_{pair}_{user_id}` | **NO** | User's own order updates |
| **WebSocket** | Private Trades (auth) | Channel `private-my_trades_{pair}_{user_id}` | **NO** | User's own trade updates |

**Summary:** We have all primary trading endpoints (buy/sell limit, market, cancel, status) and account/custodial basics. Missing: hourly ticker, order data endpoints, instant buy/sell, order history per pair, withdrawal fees, sub-account transfers, bank withdrawals, and all 7 WebSocket channels.

---

## 3. Upbit

**Base URL:** `https://api.upbit.com` (Korea) / regional variants
**WS URL:** `wss://api.upbit.com/websocket/v1`

**Source:** [Upbit Global Docs](https://global-docs.upbit.com/reference/available-order-information), [Upbit Client Reference](https://ujhin.github.io/upbit-client-docs/)

### Endpoint Gap Table

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| **Market Data** | Trading Pairs | GET `/v1/market/all` | YES | `TradingPairs` |
| **Market Data** | Second Candles | GET `/v1/candles/seconds` | YES | `CandlesSeconds` |
| **Market Data** | Minute Candles | GET `/v1/candles/minutes/{unit}` | YES | `CandlesMinutes` |
| **Market Data** | Day Candles | GET `/v1/candles/days` | YES | `CandlesDays` |
| **Market Data** | Week Candles | GET `/v1/candles/weeks` | YES | `CandlesWeeks` |
| **Market Data** | Month Candles | GET `/v1/candles/months` | YES | `CandlesMonths` |
| **Market Data** | Year Candles | GET `/v1/candles/years` | YES | `CandlesYears` |
| **Market Data** | Recent Trades | GET `/v1/trades/ticks` | YES | `RecentTrades` |
| **Market Data** | Tickers | GET `/v1/ticker` | YES | `Tickers` |
| **Market Data** | Tickers (quote) | GET `/v1/ticker` (variant) | YES | `TickersQuote` |
| **Market Data** | Order Book | GET `/v1/orderbook` | YES | `Orderbook` |
| **Market Data** | Orderbook Instruments | GET `/v1/orderbook` (instruments) | YES | `OrderbookInstruments` |
| **Account** | Balances | GET `/v1/accounts` | YES | `Balances` |
| **Account** | **API Keys** | GET `/v1/api_keys` | **NO** | List authenticated user's API keys |
| **Account** | **Wallet Status** | GET `/v1/status/wallet` | **NO** | Deposit/withdrawal service status per currency |
| **Trading** | Order Info | GET `/v1/order` | YES | `OrderInfo` |
| **Trading** | Create Order | POST `/v1/orders` | YES | `CreateOrder` |
| **Trading** | Test Order | POST `/v1/orders` (test) | YES | `TestOrder` |
| **Trading** | Get Order | GET `/v1/order` | YES | `GetOrder` |
| **Trading** | List Orders | GET `/v1/orders` | YES | `ListOrders` |
| **Trading** | Cancel Order | DELETE `/v1/order` | YES | `CancelOrder` |
| **Trading** | Batch Cancel Orders | DELETE `/v1/orders` (batch) | YES | `BatchCancelOrders` |
| **Trading** | Replace Order | POST `/v1/orders/cancel_and_new` | YES | `ReplaceOrder` |
| **Trading** | **Available Order Info** | GET `/v1/orders/chance` | **NO** | Per-market order availability, min/max sizes, fees |
| **Trading** | **List Orders by IDs** | GET `/v1/orders/uuids` | **NO** | Bulk order lookup by UUID list |
| **Trading** | **List Open Orders** | GET `/v1/orders/open` | **NO** | Explicit open orders endpoint |
| **Trading** | **List Closed Orders** | GET `/v1/orders/closed` | **NO** | Closed (filled/cancelled) orders with time filter |
| **Deposits** | Deposit Info | GET `/v1/deposit` | YES | `DepositInfo` |
| **Deposits** | List Deposit Addresses | GET `/v1/deposits/coin_addresses` | YES | `ListDepositAddresses` |
| **Deposits** | Create Deposit Address | POST `/v1/deposits/generate_coin_address` | YES | `CreateDepositAddress` |
| **Deposits** | List Deposits | GET `/v1/deposits` | YES | `ListDeposits` |
| **Deposits** | **Get Deposit Address** | GET `/v1/deposits/coin_address` | **NO** | Single deposit address by currency |
| **Deposits** | **Available Deposit Info** | GET `/v1/deposits/chance` | **NO** | Per-currency deposit availability and limits |
| **Deposits** | **Travel Rule Verification** | POST `/v1/deposits/travel_rule/...` | **NO** | VASP travel rule compliance endpoints |
| **Withdrawals** | Withdrawal Info | GET `/v1/withdrawal` | YES | `WithdrawalInfo` |
| **Withdrawals** | List Withdrawal Addresses | GET `/v1/withdraws/coin_addresses` | YES | `ListWithdrawalAddresses` |
| **Withdrawals** | Initiate Withdrawal | POST `/v1/withdraws/coin` | YES | `InitiateWithdrawal` |
| **Withdrawals** | List Withdrawals | GET `/v1/withdraws` | YES | `ListWithdrawals` |
| **Withdrawals** | **Available Withdrawal Info** | GET `/v1/withdraws/chance` | **NO** | Per-currency withdrawal availability and fee |
| **Withdrawals** | **KRW Withdrawal** | POST `/v1/withdraws/krw` | **NO** | Withdraw Korean Won to bank |
| **Withdrawals** | **Cancel Withdrawal** | DELETE `/v1/withdraws/uuid` | **NO** | Cancel pending withdrawal |
| **WebSocket** | Ticker stream | Type `ticker` | **NO** | Real-time ticker |
| **WebSocket** | Trade stream | Type `trade` | **NO** | Real-time trades |
| **WebSocket** | Order Book stream | Type `orderbook` | **NO** | Real-time order book |
| **WebSocket** | Candles stream | Type `candles` | **NO** | Real-time candle updates |
| **WebSocket** | My Orders (private) | Type `myOrder` | **NO** | User's order updates |
| **WebSocket** | My Trades (private) | Type `myTrade` | **NO** | User's trade fills |
| **WebSocket** | My Assets (private) | Type `myAsset` | **NO** | User's balance updates |

**Summary:** We have comprehensive market data and most trading order CRUD. Missing: 4 key trading queries (`chance`, closed/open orders, orders by IDs), wallet status, API keys list, 3 deposit/withdrawal utility endpoints, KRW withdrawal, cancel withdrawal, travel rule, and all 7 WebSocket streams.

---

## 4. Deribit

**Base URL:** `https://www.deribit.com/api/v2`
**WS URL:** `wss://www.deribit.com/ws/api/v2`
**Protocol:** JSON-RPC 2.0

**Source:** [Deribit API docs](https://docs.deribit.com/)

### Endpoint Gap Table

| Category | Method | We Have? | Notes |
|----------|--------|----------|-------|
| **Authentication** | `public/auth` | YES | `Auth` |
| **Authentication** | `public/exchange_token` | **NO** | Exchange access token between accounts |
| **Authentication** | `public/fork_token` | **NO** | Fork token for subaccount |
| **Authentication** | `private/logout` | **NO** | Invalidate auth token |
| **Market Data** | `public/get_instruments` | YES | `GetInstruments` |
| **Market Data** | `public/get_order_book` | YES | `GetOrderBook` |
| **Market Data** | `public/ticker` | YES | `Ticker` |
| **Market Data** | `public/get_book_summary_by_currency` | YES | `GetBookSummaryByCurrency` |
| **Market Data** | `public/get_last_trades_by_instrument` | YES | `GetLastTradesByInstrument` |
| **Market Data** | `public/get_last_trades_by_instrument_and_time` | YES | `GetLastTradesByInstrumentAndTime` |
| **Market Data** | `public/get_tradingview_chart_data` | YES | `GetTradingviewChartData` |
| **Market Data** | `public/get_book_summary_by_instrument` | **NO** | Book summary for specific instrument |
| **Market Data** | `public/get_contract_size` | **NO** | Contract size for an instrument |
| **Market Data** | `public/get_currencies` | **NO** | All supported currencies |
| **Market Data** | `public/get_delivery_prices` | **NO** | Delivery prices at expiry |
| **Market Data** | `public/get_expirations` | **NO** | Available expiration dates |
| **Market Data** | `public/get_funding_chart_data` | **NO** | Funding rate chart data |
| **Market Data** | `public/get_funding_rate_history` | **NO** | Historical funding rates |
| **Market Data** | `public/get_funding_rate_value` | **NO** | Current funding rate |
| **Market Data** | `public/get_historical_volatility` | **NO** | Historical volatility data |
| **Market Data** | `public/get_index_chart_data` | **NO** | Index price chart |
| **Market Data** | `public/get_index_price` | **NO** | Current index price |
| **Market Data** | `public/get_index_price_names` | **NO** | Available index names |
| **Market Data** | `public/get_instrument` | **NO** | Single instrument details |
| **Market Data** | `public/get_last_settlements_by_currency` | **NO** | Settlement history by currency |
| **Market Data** | `public/get_last_settlements_by_instrument` | **NO** | Settlement history by instrument |
| **Market Data** | `public/get_last_trades_by_currency` | **NO** | Recent trades by currency |
| **Market Data** | `public/get_last_trades_by_currency_and_time` | **NO** | Trades by currency in time range |
| **Market Data** | `public/get_mark_price_history` | **NO** | Mark price historical data |
| **Market Data** | `public/get_order_book_by_instrument_id` | **NO** | Order book by numeric instrument ID |
| **Market Data** | `public/get_supported_index_names` | **NO** | All supported index names |
| **Market Data** | `public/get_trade_volumes` | **NO** | Exchange-wide trade volumes |
| **Market Data** | `public/get_volatility_index_data` | **NO** | Volatility index (DVOL) data |
| **Market Data** | `public/get_apr_history` | **NO** | APR history |
| **Market Data** | `public/get_announcements` | **NO** | Platform announcements |
| **Options** | `public/get_block_rfq_trades` | **NO** | Block RFQ trade history |
| **Options** | `public/get_combo_details` | **NO** | Combo instrument details |
| **Options** | `public/get_combo_ids` | **NO** | Available combo IDs |
| **Options** | `public/get_combos` | **NO** | All combo instruments |
| **Trading** | `private/buy` | YES | `Buy` |
| **Trading** | `private/sell` | YES | `Sell` |
| **Trading** | `private/edit` | YES | `Edit` |
| **Trading** | `private/cancel` | YES | `Cancel` |
| **Trading** | `private/cancel_by_label` | YES | `CancelByLabel` |
| **Trading** | `private/cancel_all` | YES | `CancelAll` |
| **Trading** | `private/cancel_all_by_currency` | YES | `CancelAllByCurrency` |
| **Trading** | `private/cancel_all_by_instrument` | YES | `CancelAllByInstrument` |
| **Trading** | `private/get_open_orders` | YES | `GetOpenOrders` |
| **Trading** | `private/get_open_orders_by_currency` | YES | `GetOpenOrdersByCurrency` |
| **Trading** | `private/get_open_orders_by_instrument` | YES | `GetOpenOrdersByInstrument` |
| **Trading** | `private/get_order_state` | YES | `GetOrderState` |
| **Trading** | `private/close_position` | YES | `ClosePosition` |
| **Trading** | `private/cancel_all_by_currency_pair` | **NO** | Cancel all orders by currency pair |
| **Trading** | `private/cancel_all_by_kind_or_type` | **NO** | Cancel orders filtered by kind (future/option) or type |
| **Trading** | `private/cancel_quotes` | **NO** | Cancel market maker quotes |
| **Trading** | `private/edit_by_label` | **NO** | Edit order by label (vs by ID) |
| **Trading** | `private/get_margins` | **NO** | Required margin for hypothetical order |
| **Trading** | `private/get_mmp_config` | **NO** | Market Maker Protection config |
| **Trading** | `private/get_mmp_status` | **NO** | Current MMP status |
| **Trading** | `private/get_open_orders_by_label` | **NO** | Open orders filtered by label |
| **Trading** | `private/get_order_history_by_currency` | **NO** | Full order history by currency |
| **Trading** | `private/get_order_history_by_instrument` | **NO** | Full order history by instrument |
| **Trading** | `private/get_order_margin_by_ids` | **NO** | Order margin for specific order IDs |
| **Trading** | `private/get_order_state_by_label` | **NO** | Order state by client label |
| **Trading** | `private/get_trigger_order_history` | **NO** | History of trigger orders |
| **Trading** | `private/get_user_trades_by_currency_and_time` | **NO** | Trades by currency in time range |
| **Trading** | `private/get_user_trades_by_order` | **NO** | Trades for specific order |
| **Trading** | `private/mass_quote` | **NO** | Market maker: submit multiple quotes |
| **Trading** | `private/move_positions` | **NO** | Move positions between sub-accounts |
| **Trading** | `private/reset_mmp` | **NO** | Reset MMP to enabled state |
| **Trading** | `private/set_mmp_config` | **NO** | Configure MMP parameters |
| **Trading** | `private/get_settlement_history_by_currency` | **NO** | Settlement history by currency |
| **Account** | `private/get_account_summary` | YES | `GetAccountSummary` |
| **Account** | `private/get_user_trades_by_instrument` | YES | `GetUserTradesByInstrument` |
| **Account** | `private/get_user_trades_by_currency` | YES | `GetUserTradesByCurrency` |
| **Account** | `private/get_settlement_history_by_instrument` | YES | `GetSettlementHistoryByInstrument` |
| **Account** | `private/get_account_summaries` | **NO** | Multiple account summaries (including subaccounts) |
| **Account** | `private/get_subaccounts` | **NO** | List all sub-accounts |
| **Account** | `private/get_subaccounts_details` | **NO** | Detailed sub-account info |
| **Account** | `private/get_transaction_log` | **NO** | Full transaction/ledger log |
| **Account** | `private/get_user_locks` | **NO** | Active account locks/restrictions |
| **Account** | `private/change_margin_model` | **NO** | Switch between cross/isolated margin |
| **Account** | `private/get_positions` | YES | `GetPositions` |
| **Account** | `private/get_position` | YES | `GetPosition` |
| **Account** | `private/simulate_portfolio` | **NO** | Simulate portfolio margin impact |
| **Account** | `private/pme/simulate` | **NO** | Portfolio Margin Engine simulation |
| **Wallet** | `private/get_current_deposit_address` | YES | `GetCurrentDepositAddress` |
| **Wallet** | `private/withdraw` | YES | `Withdraw` |
| **Wallet** | `private/get_deposits` | YES | `GetDeposits` |
| **Wallet** | `private/get_withdrawals` | YES | `GetWithdrawals` |
| **Wallet** | `private/create_deposit_address` | **NO** | Generate a new deposit address |
| **Wallet** | `private/get_transfers` | **NO** | Transfer history between accounts |
| **Wallet** | `private/submit_transfer_between_subaccounts` | **NO** | Move funds between sub-accounts |
| **Wallet** | `private/submit_transfer_to_subaccount` | **NO** | Transfer to specific sub-account |
| **Wallet** | `private/submit_transfer_to_user` | **NO** | Transfer to another Deribit user |
| **Wallet** | `private/cancel_withdrawal` | **NO** | Cancel pending withdrawal |
| **Wallet** | `private/get_address_book` | **NO** | Withdrawal address book |
| **Wallet** | `private/add_to_address_book` | **NO** | Add withdrawal address |
| **Block Trade** | `private/execute_block_trade` | **NO** | Execute block trade |
| **Block Trade** | `private/get_block_trades` | **NO** | Block trade history |
| **Block Trade** | `private/verify_block_trade` | **NO** | Verify block trade parameters |
| **Combo** | `private/create_combo` | **NO** | Create combo instrument |
| **Block RFQ** | `private/create_block_rfq` | **NO** | Create block RFQ |
| **Block RFQ** | `private/accept_block_rfq` | **NO** | Accept a block RFQ |
| **Session** | `private/enable_cancel_on_disconnect` | **NO** | Enable cancel-on-disconnect |
| **Session** | `private/get_cancel_on_disconnect` | **NO** | Check cancel-on-disconnect status |
| **Session** | `public/set_heartbeat` | **NO** | Set heartbeat interval |
| **Session** | `public/disable_heartbeat` | **NO** | Disable heartbeat |
| **WebSocket** | `public/subscribe` | YES | `Subscribe` |
| **WebSocket** | `public/unsubscribe` | YES | `Unsubscribe` |
| **WebSocket** | `private/subscribe` | YES | `SubscribePrivate` |
| **WebSocket** | `private/unsubscribe` | YES | `UnsubscribePrivate` |
| **WebSocket** | `public/unsubscribe_all` | **NO** | Unsubscribe all public channels |
| **WebSocket** | `private/unsubscribe_all` | **NO** | Unsubscribe all private channels |
| **Support** | `public/test` | YES | `Test` |
| **Support** | `public/get_time` | **NO** | Server time |
| **Support** | `public/hello` | **NO** | Client identification handshake |
| **Support** | `public/status` | **NO** | Platform status |

**Summary:** We have 28 of ~197 total methods. Well covered: core trading (buy/sell/edit/cancel), open order queries, positions, account summary, wallet basics, WebSocket sub/unsub. Major gaps: 20+ market data methods (funding rates, volatility, index data, settlements), 14 advanced trading methods (MMP, edit by label, mass quote, trigger orders), sub-account management, block trades, combo instruments, cancel-on-disconnect session control.

---

## 5. Vertex Protocol

**Gateway URL:** `https://gateway.prod.vertexprotocol.com/v1`
**Indexer URL:** `https://archive.prod.vertexprotocol.com/v1`
**WS URL:** `wss://gateway.prod.vertexprotocol.com/v1/ws`
**Subscribe URL:** `wss://gateway.prod.vertexprotocol.com/v1/subscribe`

**Source:** [Vertex Docs](https://docs.vertexprotocol.com/developer-resources/api), [Python SDK](https://vertex-protocol.github.io/vertex-python-sdk/api-reference.html)

### Endpoint Gap Table

**Gateway Query Types (`/query`):**

| Category | Query Type | We Have? | Notes |
|----------|-----------|----------|-------|
| **Products/Symbols** | `all_products` | YES | `AllProducts` |
| **Products/Symbols** | `symbols` (GET `/symbols`) | YES | `Symbols` |
| **Products/Symbols** | `contracts` | YES | `Contracts` |
| **Products/Symbols** | `status` | YES | `Status` |
| **Market Data** | `market_liquidity` | YES | `MarketLiquidity` |
| **Market Data** | `market_price` | YES | `MarketPrice` |
| **Account** | `subaccount_info` | YES | `SubaccountInfo` |
| **Account** | `fee_rates` | YES | `FeeRates` |
| **Account** | `max_withdrawable` | YES | `MaxWithdrawable` |
| **Account** | `subaccount_orders` (open orders) | YES | `SubaccountOrders` |
| **Account** | `order` (single order) | YES | `Order` |
| **Account** | `max_order_size` | YES | `MaxOrderSize` |
| **Account** | **`nonces`** | **NO** | Get transaction nonces for signing |
| **Account** | **`health_groups`** | **NO** | Risk/health group info for margin |
| **Account** | **`linked_signer`** | **NO** | Get linked signer address for subaccount |
| **Account** | **`isolated_positions`** | **NO** | Isolated margin positions |
| **Account** | **`insurance`** | **NO** | Insurance fund state |
| **Account** | **`max_lp_mintable`** | **NO** | Maximum LP tokens mintable |
| **Account** | **`subaccounts`** | **NO** | List subaccounts for a wallet |
| **Account** | **`min_deposit_rates`** | **NO** | Minimum deposit rates |
| **Account** | **`assets`** | **NO** | All spot assets info |
| **Account** | **`pairs`** | **NO** | Trading pairs info |
| **Account** | **`spots_apr`** | **NO** | Spot lending APR rates |
| **Account** | **`orderbook`** (depth) | **NO** | Orderbook snapshot at depth |

**Gateway Execute Actions (`/execute`):**

| Category | Action Type | We Have? | Notes |
|----------|------------|----------|-------|
| **Trading** | `place_order` | YES | via `Execute` generic |
| **Trading** | `place_isolated_order` | **PARTIAL** | Via `Execute` — no dedicated enum variant |
| **Trading** | `cancel_orders` | **PARTIAL** | Via `Execute` — no dedicated enum variant |
| **Trading** | `cancel_product_orders` | **PARTIAL** | Via `Execute` — no dedicated enum variant |
| **Trading** | `cancel_and_place` | **PARTIAL** | Via `Execute` — no dedicated enum variant |
| **LP** | `mint_lp` | **PARTIAL** | Via `Execute` — not exposed as separate variant |
| **LP** | `burn_lp` | **PARTIAL** | Via `Execute` — not exposed as separate variant |
| **Collateral** | `withdraw_collateral` | **PARTIAL** | Via `Execute` — no dedicated enum variant |
| **Liquidation** | `liquidate_subaccount` | **PARTIAL** | Via `Execute` — no dedicated enum variant |
| **Signer** | `link_signer` | **PARTIAL** | Via `Execute` — no dedicated enum variant |
| **Trigger Orders** | **`place_trigger_order`** | **NO** | Conditional trigger orders |
| **Trigger Orders** | **`cancel_trigger_orders`** | **NO** | Cancel trigger orders |
| **Trigger Orders** | **`cancel_trigger_product_orders`** | **NO** | Cancel all trigger orders for product |
| **Position** | **`close_position`** | **NO** | Close a position |
| **Rewards** | **`claim_vrtx`** | **NO** | Claim VRTX rewards |
| **Rewards** | **`stake_vrtx`** | **NO** | Stake VRTX tokens |
| **Rewards** | **`unstake_vrtx`** | **NO** | Unstake VRTX tokens |
| **Rewards** | **`claim_usdc_rewards`** | **NO** | Claim USDC rewards |
| **Rewards** | **`claim_foundation_rewards`** | **NO** | Claim foundation rewards |

**Indexer Archive Queries (POST to `/v1`):**

| Category | Query Type | We Have? | Notes |
|----------|-----------|----------|-------|
| **Price Data** | candlesticks | YES | `Candlesticks` |
| **Price Data** | funding_rate | YES | `FundingRate` |
| **Snapshots** | product_snapshots | YES | `ProductSnapshots` |
| **Order History** | **orders** (historical) | **NO** | Historical orders for subaccount |
| **Order History** | **orders_by_digest** | **NO** | Orders by digest list |
| **Trades** | **matches** | **NO** | Matched trade events |
| **Account** | **events** | **NO** | Subaccount event history |
| **Account** | **summary** | **NO** | Subaccount summary |
| **Snapshots** | **market_snapshots** | **NO** | Historical market snapshots |
| **Funding** | **perp_funding_rates** | **NO** | All perp funding rates |
| **Price Data** | **perp_prices** | **NO** | Historical perp prices |
| **Price Data** | **oracle_prices** | **NO** | Oracle price data |
| **Rewards** | **token_rewards** | **NO** | Token reward history |
| **Statistics** | **maker_statistics** | **NO** | Market maker statistics |
| **Liquidations** | **liquidation_feed** | **NO** | Liquidation event feed |
| **Signer** | **linked_signer_rate_limits** | **NO** | Rate limits for linked signer |
| **Referral** | **referral_code** | **NO** | Referral code info |
| **USDC** | **usdc_price** | **NO** | USDC price feed |
| **Rewards** | **vrtx_merkle_proofs** | **NO** | Merkle proofs for VRTX claims |
| **Trades** | **historical_trades** | **NO** | Historical trade data |
| **Tickers** | **tickers** | **NO** | 24h ticker data |
| **Perp Info** | **perp_contracts_info** | **NO** | Perp contract specifications |
| **Interest** | **interest_and_funding_payments** | **NO** | Interest/funding payment history |
| **WebSocket** | Subscribe stream | **NO** | No WebSocket variants defined |

**Summary:** We have 12 gateway queries and the single `Execute` catch-all. Major gaps: 12 additional gateway queries (nonces, health groups, isolated positions, linked signer, etc.), trigger order execute actions, rewards/staking actions, close position, and the entire indexer API (23 query types covering historical trades, matches, events, account summary, market snapshots, oracle prices, liquidations, etc.).

---

## 6. Hyperliquid

**REST URL:** `https://api.hyperliquid.xyz`
**WS URL:** `wss://api.hyperliquid.xyz/ws`
**Protocol:** POST to `/info` or `/exchange`

**Source:** [Hyperliquid Docs](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api), [Elixir SDK](https://hexdocs.pm/hyperliquid/Hyperliquid.Api.Info.html)

### Info Endpoint Types (`/info`)

| Category | Info Type (`type` field) | We Have? | Notes |
|----------|--------------------------|----------|-------|
| **Perp Meta** | `meta` | YES | `Meta` |
| **Perp Meta** | `metaAndAssetCtxs` | YES | `MetaAndAssetCtxs` |
| **Spot Meta** | `spotMeta` | YES | `SpotMeta` |
| **Spot Meta** | **`spotMetaAndAssetCtxs`** | **NO** | Spot metadata + asset contexts combined |
| **Market Data** | `allMids` | YES | `AllMids` |
| **Market Data** | `l2Book` | YES | `L2Book` |
| **Market Data** | `recentTrades` | YES | `RecentTrades` |
| **Market Data** | `candleSnapshot` | YES | `CandleSnapshot` |
| **Market Data** | `fundingHistory` | YES | `FundingHistory` |
| **Market Data** | **`predictedFundings`** | **NO** | Predicted funding rates across venues |
| **Perp Extra** | **`allPerpMetas`** | **NO** | All perpetual asset metadata |
| **Perp Extra** | **`marginTable`** | **NO** | Margin requirements and leverage tiers |
| **Perp Extra** | **`perpDexs`** | **NO** | All deployed perpetual DEXs |
| **Perp Extra** | **`perpDexLimits`** | **NO** | Caps and limits for specific DEX |
| **Perp Extra** | **`maxMarketOrderNtls`** | **NO** | Maximum market order notional values |
| **Perp Extra** | **`perpsAtOpenInterestCap`** | **NO** | Assets at max open interest cap |
| **Perp Extra** | **`liquidatable`** | **NO** | Positions eligible for liquidation |
| **Perp Extra** | **`perpDeployAuctionStatus`** | **NO** | Gas auction for new perpetuals |
| **Account** | `clearinghouseState` | YES | `ClearinghouseState` |
| **Account** | `spotClearinghouseState` | YES | `SpotClearinghouseState` |
| **Account** | `openOrders` | YES | `OpenOrders` |
| **Account** | `orderStatus` | YES | `OrderStatus` |
| **Account** | `userFills` | YES | `UserFills` |
| **Account** | `userFillsByTime` | YES | `UserFillsByTime` |
| **Account** | `userFees` | YES | `UserFees` |
| **Account** | `userRateLimit` | YES | `UserRateLimit` |
| **Account** | `historicalOrders` | YES | `HistoricalOrders` |
| **Account** | **`activeAssetData`** | **NO** | Leverage, max sizes, available balance for asset |
| **Account** | **`userTwapSliceFills`** | **NO** | TWAP order slice fill history |
| **Account** | **`userTwapSliceFillsByTime`** | **NO** | TWAP fills filtered by time range |
| **Account** | **`portfolio`** | **NO** | Historical account value, PnL, and volume |
| **Account** | **`userRole`** | **NO** | User role information |
| **Account** | **`extraAgents`** | **NO** | Authorized additional agents |
| **Account** | **`userToMultiSigSigners`** | **NO** | Multi-signature signer information |
| **Account** | **`userDexAbstraction`** | **NO** | DEX abstraction settings |
| **Account** | **`userNonFundingLedgerUpdates`** | **NO** | Non-funding ledger entries (deposits/withdrawals) |
| **Account** | **`subAccounts`** | **NO** | User's sub-accounts list |
| **Account** | **`subAccounts2`** | **NO** | Extended sub-accounts information |
| **Account** | **`frontendOpenOrders`** | **NO** | Orders with frontend display fields |
| **Account** | **`twapHistory`** | **NO** | TWAP order history |
| **Vaults** | **`vaultSummaries`** | **NO** | Vault summary information |
| **Vaults** | **`vaultDetails`** | **NO** | Detailed vault metrics |
| **Vaults** | **`userVaultEquities`** | **NO** | User's equity in vaults |
| **Vaults** | **`leadingVaults`** | **NO** | Top performing vaults |
| **Vaults** | **`referral`** | **NO** | Referral statistics and rewards |
| **Staking** | **`delegations`** | **NO** | Validators user has delegated to |
| **Staking** | **`delegatorSummary`** | **NO** | Total delegation status summary |
| **Staking** | **`delegatorHistory`** | **NO** | Delegation action history |
| **Staking** | **`delegatorRewards`** | **NO** | Staking reward history |
| **Validators** | **`validatorSummaries`** | **NO** | Validator information |
| **Validators** | **`validatorL1Votes`** | **NO** | Validator L1 voting data |
| **Validators** | **`gossipRootIps`** | **NO** | Gossip network root IPs |
| **Compliance** | **`legalCheck`** | **NO** | Jurisdiction-based platform eligibility |
| **Compliance** | **`isVip`** | **NO** | VIP tier status and fee rates |
| **Compliance** | **`preTransferCheck`** | **NO** | User existence and sanction check |
| **Token Info** | **`tokenDetails`** | **NO** | Specific token/asset details |
| **Token Info** | **`alignedQuoteTokenInfo`** | **NO** | Aligned quote token data |
| **Exchange** | **`exchangeStatus`** | **NO** | Exchange operational status |
| **Exchange** | **`maxBuilderFee`** | **NO** | Maximum builder fee for user |
| **Spot Deploy** | **`spotDeployState`** | **NO** | Spot token deployment status |
| **Spot Deploy** | **`spotPairDeployAuctionStatus`** | **NO** | Spot pair deployment auction |

### Exchange Endpoint Actions (`/exchange`)

| Category | Action Type | We Have? | Notes |
|----------|------------|----------|-------|
| **Trading** | `order` | YES | `Order` |
| **Trading** | `cancel` | YES | `Cancel` |
| **Trading** | `cancelByCloid` | YES | `CancelByCloid` |
| **Trading** | `modify` | YES | `Modify` |
| **Position Mgmt** | `updateLeverage` | YES | `UpdateLeverage` |
| **Position Mgmt** | `updateIsolatedMargin` | YES | `UpdateIsolatedMargin` |
| **Transfers** | `usdClassTransfer` | YES | `UsdClassTransfer` |
| **Transfers** | `usdSend` | YES | `UsdSend` |
| **Transfers** | `spotSend` | YES | `SpotSend` |
| **Transfers** | `withdraw3` | YES | `Withdraw3` |
| **Trading** | **`batchModify`** | **NO** | Modify multiple orders in batch |
| **Trading** | **`scheduleCancel`** | **NO** | Schedule cancel at specific time |
| **Vaults** | **`vaultTransfer`** | **NO** | Deposit/withdraw from vault |
| **Vaults** | **`createSubAccount`** | **NO** | Create a sub-account |
| **Vaults** | **`subAccountTransfer`** | **NO** | Transfer between sub-accounts |
| **Agents** | **`approveAgent`** | **NO** | Approve an API agent wallet |
| **Agents** | **`approveBuilderFee`** | **NO** | Approve builder fee for front-end |
| **Staking** | **`tokenDelegate`** | **NO** | Delegate/undelegate native tokens to validator |
| **Display** | **`setDisplayName`** | **NO** | Set account display name |
| **Display** | **`setReferrer`** | **NO** | Set referral code |
| **Spot Deploy** | **`spotDeploy`** | **NO** | Deploy HIP-1/HIP-2 spot token |
| **Spot Deploy** | **`deploySpotPair`** | **NO** | Deploy spot trading pair |
| **Perp Deploy** | **`perpDeploy`** | **NO** | Deploy new perpetual market |

**Summary:** We have all 10 core exchange actions (order, cancel, modify, leverage, margin, transfers, withdraw) and 9 info query types covering basic market data and primary account endpoints. Missing: ~50 info types covering vault ecosystem, staking/delegation, compliance, token deploy, detailed perp metadata, TWAP fills, portfolio history, sub-accounts, and multi-sig. Missing: ~13 exchange actions covering batch operations, vault transfers, agent approval, staking delegation, spot/perp deployment.

---

## Overall Priority Summary

| Exchange | Total Endpoints (approx) | We Have | Key Gaps |
|----------|--------------------------|---------|----------|
| Bithumb | ~40 REST + 11 WS | ~25 REST | Batch orders, futures trading/account, all WS |
| Bitstamp | ~35 REST + 7 WS | ~26 REST | Hourly ticker, order data, instant orders, bank transfers, all WS |
| Upbit | ~35 REST + 7 WS | ~25 REST | `chance` endpoint, open/closed order lists, wallet status, KRW withdrawal, all WS |
| Deribit | ~197 JSON-RPC | ~28 | Funding/volatility/index data, MMP, order history, sub-accounts, block trades |
| Vertex | ~15 gateway + ~25 indexer | ~12 gateway, 3 indexer | Nonces, health groups, trigger orders, full indexer API, rewards/staking |
| Hyperliquid | ~60 info + ~23 exchange | ~9 info, ~10 exchange | Vaults, staking, compliance, portfolio, TWAP, spot deploy, batch ops |

### Highest Priority Missing Endpoints

1. **Deribit** — `public/get_funding_rate_history`, `public/get_funding_rate_value`, `public/get_index_price`, `public/get_historical_volatility`, `private/get_order_history_by_currency` — critical for perpetuals and options trading analytics
2. **Vertex** — Full indexer API (historical orders, matches, events, market snapshots, oracle prices) — needed for backtesting and analytics
3. **Hyperliquid** — `portfolio`, `activeAssetData`, `userTwapSliceFills`, `vaultDetails`, `delegations` — needed for full account management
4. **Upbit** — `GET /v1/orders/chance`, `GET /v1/orders/open`, `GET /v1/orders/closed`, `GET /v1/status/wallet` — core exchange info endpoints
5. **Bithumb** — Futures trading endpoints (place/cancel/position/account) — needed to actually trade futures
6. **Bitstamp** — Instant buy/sell, hourly ticker, order history per pair, WebSocket channels

---

## Sources

- [Bithumb Pro REST API (GitHub)](https://github.com/bithumb-pro/bithumb.pro-official-api-docs/blob/master/rest-api.md)
- [Bithumb Pro WS API (GitHub)](https://github.com/bithumb-pro/bithumb.pro-official-api-docs/blob/master/ws-api.md)
- [Bitstamp API](https://www.bitstamp.net/api/)
- [Bitstamp WS API v2](https://www.bitstamp.net/websocket/v2/)
- [node-bitstamp (endpoint reference)](https://github.com/krystianity/node-bitstamp)
- [Upbit Global Developer Center](https://global-docs.upbit.com/reference/available-order-information)
- [Upbit Client Reference](https://ujhin.github.io/upbit-client-docs/)
- [Deribit API Documentation](https://docs.deribit.com/)
- [Deribit API llms.txt (method index)](https://docs.deribit.com/llms.txt)
- [Vertex Protocol API Docs](https://docs.vertexprotocol.com/developer-resources/api)
- [Vertex Python SDK API Reference](https://vertex-protocol.github.io/vertex-python-sdk/api-reference.html)
- [Hyperliquid API Docs](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api)
- [Hyperliquid Info Endpoint](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint)
- [Hyperliquid Exchange Endpoint](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/exchange-endpoint)
- [Hyperliquid Elixir SDK Info Reference](https://hexdocs.pm/hyperliquid/Hyperliquid.Api.Info.html)
