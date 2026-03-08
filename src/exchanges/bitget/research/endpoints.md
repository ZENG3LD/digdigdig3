# Bitget API Endpoints

Base URL: `https://api.bitget.com`

## MarketData Trait Endpoints

### Get Server Time
```
GET /api/spot/v1/public/time
```
Returns server timestamp in milliseconds.

**Response:**
```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695806875837,
  "data": {
    "serverTime": "1695806875837"
  }
}
```

### Get Trading Symbols (Spot)
```
GET /api/spot/v1/public/products
```
Returns all trading symbols with configuration.

**Response Fields:**
- `symbol`: Trading pair (e.g., "BTCUSDT_SPBL")
- `baseCoin`: Base currency
- `quoteCoin`: Quote currency
- `minTradeAmount`: Minimum order amount
- `maxTradeAmount`: Maximum order amount
- `takerFeeRate`: Taker fee
- `makerFeeRate`: Maker fee
- `pricePrecision`: Price decimal places
- `quantityPrecision`: Quantity decimal places
- `status`: Trading status (online/offline)

### Get Single Symbol (Spot)
```
GET /api/spot/v1/public/product?symbol={symbol}
```
Returns specific trading pair details.

### Get Ticker (Spot)
```
GET /api/spot/v1/market/ticker?symbol={symbol}
```
Returns 24h ticker data for a symbol.

**Response Fields:**
- `symbol`: Trading pair
- `high24h`: 24h high
- `low24h`: 24h low
- `close`: Latest price
- `quoteVol`: Quote volume
- `baseVol`: Base volume
- `usdtVol`: USDT volume
- `ts`: Timestamp
- `bidPr`: Best bid price
- `askPr`: Best ask price
- `bidSz`: Best bid size
- `askSz`: Best ask size
- `openUtc`: Open price (UTC 00:00)
- `changeUtc24h`: 24h change percentage
- `change24h`: 24h change

### Get All Tickers (Spot)
```
GET /api/spot/v1/market/tickers
```
Returns ticker data for all trading pairs.

### Get Recent Trades (Spot)
```
GET /api/spot/v1/market/fills?symbol={symbol}&limit={1-500}
```
Returns recent transaction history.

**Response Fields:**
- `symbol`: Trading pair
- `tradeId`: Trade ID
- `side`: Buy/sell
- `fillPrice`: Fill price
- `fillQuantity`: Fill quantity
- `fillTime`: Fill timestamp

### Get Market Trades (Spot)
```
GET /api/spot/v1/market/fills-history?symbol={symbol}&limit={1-1000}&startTime={ts}&endTime={ts}
```
Returns historical trades from past 30 days.

### Get Candles/Klines (Spot)
```
GET /api/spot/v1/market/candles?symbol={symbol}&period={timeframe}&limit={1-1000}
```

**Parameters:**
- `symbol`: Trading pair
- `period`: Timeframe (1min, 5min, 15min, 30min, 1h, 4h, 12h, 1day, 1week, 1M)
- `limit`: Max 1000
- `after`: Start timestamp (optional)
- `before`: End timestamp (optional)

**Response:** Array of `[timestamp, open, high, low, close, volume, quoteVolume]`

### Get Historical Candles (Spot)
```
GET /api/spot/v1/market/history-candles?symbol={symbol}&period={timeframe}&endTime={ts}&limit={1-200}
```
Returns historical candlestick data.

### Get Order Book (Spot)
```
GET /api/spot/v1/market/depth?symbol={symbol}&type={step0-5}&limit={1-200}
```

**Parameters:**
- `symbol`: Trading pair
- `type`: Precision level (step0-step5)
- `limit`: Depth levels (default 100, max 200)

**Response:**
```json
{
  "asks": [["price", "quantity"], ...],
  "bids": [["price", "quantity"], ...],
  "ts": "timestamp"
}
```

### Get Merged Depth (Spot)
```
GET /api/spot/v1/market/merge-depth?symbol={symbol}&precision={scale}&limit={limit}
```
Returns aggregated order book at specified precision.

---

## Futures/Mix Market Data Endpoints

### Get Futures Symbols
```
GET /api/mix/v1/market/contracts?productType={type}
```

**Product Types:**
- `umcbl`: USDT-margined futures
- `dmcbl`: Coin-margined futures
- `cmcbl`: USDC-margined futures
- `sumcbl`: Simulated trading

**Response Fields:**
- `symbol`: Contract symbol
- `baseCoin`: Base currency
- `quoteCoin`: Quote currency
- `buyLimitPriceRatio`: Max buy price ratio
- `sellLimitPriceRatio`: Max sell price ratio
- `feeRateUpRatio`: Fee rate upper ratio
- `makerFeeRate`: Maker fee
- `takerFeeRate`: Taker fee
- `openCostUpRatio`: Opening cost ratio
- `supportMarginCoins`: Supported margin currencies
- `minTradeNum`: Minimum trade size
- `priceEndStep`: Price precision
- `volumePlace`: Volume precision
- `sizeMultiplier`: Contract size

### Get Futures Ticker
```
GET /api/mix/v1/market/ticker?symbol={symbol}&productType={type}
```

**Response Fields:**
- `symbol`: Contract symbol
- `last`: Latest price
- `bestAsk`: Best ask
- `bestBid`: Best bid
- `bidSz`: Bid size
- `askSz`: Ask size
- `high24h`: 24h high
- `low24h`: 24h low
- `timestamp`: Update time
- `priceChangePercent`: Price change percentage
- `baseVolume`: Base volume
- `quoteVolume`: Quote volume
- `usdtVolume`: USDT volume
- `openUtc`: UTC open price
- `chgUtc`: UTC change
- `indexPrice`: Index price
- `fundingRate`: Funding rate
- `holdingAmount`: Total holdings

### Get All Futures Tickers
```
GET /api/mix/v1/market/tickers?productType={type}
```
Returns all tickers for a product type.

### Get Futures Depth
```
GET /api/mix/v1/market/depth?symbol={symbol}&productType={type}&limit={5|15|50|100}
```

**Response:**
```json
{
  "asks": [["price", "size"], ...],
  "bids": [["price", "size"], ...],
  "timestamp": "ts"
}
```

### Get Futures Candles
```
GET /api/mix/v1/market/candles?symbol={symbol}&productType={type}&granularity={timeframe}&limit={max:1000}
```

**Granularities:** 1m, 3m, 5m, 15m, 30m, 1H, 2H, 4H, 6H, 12H, 1D, 3D, 1W, 1M

**Response:** `[timestamp, open, high, low, close, baseVolume, quoteVolume, usdtVolume]`

### Get Index Price
```
GET /api/mix/v1/market/index-price?symbol={symbol}&productType={type}
```
Returns index price information.

### Get Funding Rate
```
GET /api/mix/v1/market/funding-rate?symbol={symbol}&productType={type}
```
Returns current and historical funding rates.

**Response Fields:**
- `symbol`: Contract symbol
- `fundingRate`: Current funding rate
- `fundingTime`: Next funding time

### Get Mark Price
```
GET /api/mix/v1/market/mark-price?symbol={symbol}&productType={type}
```
Returns mark price data.

### Get Open Interest
```
GET /api/mix/v1/market/open-interest?symbol={symbol}&productType={type}
```
Returns open interest metrics.

---

## Trading Trait Endpoints

### Place Order (Spot)
```
POST /api/spot/v1/trade/orders
```

**Request:**
```json
{
  "symbol": "BTCUSDT_SPBL",
  "side": "buy",
  "orderType": "limit",
  "force": "normal",
  "price": "50000.00",
  "quantity": "0.01",
  "clientOrderId": "custom_id_123"
}
```

**Parameters:**
- `symbol`: Trading pair (required)
- `side`: "buy" or "sell" (required)
- `orderType`: "limit" or "market" (required)
- `force`: "normal", "post_only", "fok", "ioc" (required)
- `price`: Order price (required for limit orders)
- `quantity`: Order quantity (required)
- `clientOrderId`: Custom order ID (optional)

**Response:**
```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695806875837,
  "data": {
    "orderId": "1234567890",
    "clientOrderId": "custom_id_123"
  }
}
```

**Rate Limit:** 10 req/sec/UID

### Batch Place Orders (Spot)
```
POST /api/spot/v1/trade/batch-orders
```

**Request:**
```json
{
  "symbol": "BTCUSDT_SPBL",
  "orderList": [
    {
      "side": "buy",
      "orderType": "limit",
      "force": "normal",
      "price": "50000.00",
      "quantity": "0.01",
      "clientOrderId": "id1"
    }
  ]
}
```

### Cancel Order (Spot)
```
POST /api/spot/v1/trade/cancel-order
```

**Request:**
```json
{
  "symbol": "BTCUSDT_SPBL",
  "orderId": "1234567890"
}
```
OR
```json
{
  "symbol": "BTCUSDT_SPBL",
  "clientOrderId": "custom_id_123"
}
```

### Get Order Details (Spot)
```
GET /api/spot/v1/trade/orderInfo?symbol={symbol}&orderId={id}
```
OR
```
GET /api/spot/v1/trade/orderInfo?symbol={symbol}&clientOrderId={clientOid}
```

**Response Fields:**
- `orderId`: Order ID
- `clientOrderId`: Client order ID
- `symbol`: Trading pair
- `side`: Buy/sell
- `orderType`: Order type
- `price`: Order price
- `quantity`: Order quantity
- `fillPrice`: Average fill price
- `fillQuantity`: Filled quantity
- `fillTotalAmount`: Total filled amount
- `status`: Order status (init, new, partial_fill, full_fill, canceled, failed)
- `cTime`: Create time
- `uTime`: Update time

### Get Open Orders (Spot)
```
GET /api/spot/v1/trade/open-orders?symbol={symbol}
```
Returns all unfilled and partially filled orders.

### Get Order History (Spot)
```
GET /api/spot/v1/trade/history?symbol={symbol}&startTime={ts}&endTime={ts}&limit={limit}
```
Returns historical orders.

### Get Fills/Trades (Spot)
```
GET /api/spot/v1/trade/fills?symbol={symbol}&orderId={id}&limit={limit}
```
Returns trade execution details.

---

## Futures Trading Endpoints

### Place Order (Futures)
```
POST /api/mix/v1/order/placeOrder
```

**Request:**
```json
{
  "symbol": "BTCUSDT_UMCBL",
  "productType": "umcbl",
  "marginCoin": "USDT",
  "size": "0.01",
  "price": "50000.00",
  "side": "open_long",
  "orderType": "limit",
  "timeInForceValue": "normal",
  "clientOid": "custom_id_123"
}
```

**Parameters:**
- `symbol`: Contract symbol (required)
- `productType`: umcbl/dmcbl/cmcbl/sumcbl (required)
- `marginCoin`: Margin currency (required)
- `size`: Order size (required)
- `side`: open_long, open_short, close_long, close_short (required)
- `orderType`: limit, market (required)
- `price`: Order price (required for limit)
- `timeInForceValue`: normal, post_only, fok, ioc (optional)
- `clientOid`: Custom order ID (optional)
- `reduceOnly`: Reduce only flag (optional)

**Side values (based on position mode):**
- One-way mode: open_long, open_short, close_long, close_short
- Hedge mode: buy (long), sell (short) + tradeSide (open/close)

**Rate Limit:** 10 req/sec/UID

### Batch Place Orders (Futures)
```
POST /api/mix/v1/order/batch-orders
```
Max 50 orders per request.

### Cancel Order (Futures)
```
POST /api/mix/v1/order/cancel-order
```

**Request:**
```json
{
  "symbol": "BTCUSDT_UMCBL",
  "productType": "umcbl",
  "marginCoin": "USDT",
  "orderId": "1234567890"
}
```

### Modify Order (Futures)
```
POST /api/mix/v1/order/modify-order
```

**Request:**
```json
{
  "symbol": "BTCUSDT_UMCBL",
  "productType": "umcbl",
  "orderId": "1234567890",
  "newPrice": "51000.00",
  "newSize": "0.02",
  "newClientOid": "new_id_456"
}
```

**Note:** Modifying price/size cancels old order and creates new one asynchronously. Must provide `newClientOid` since new orderId cannot be returned synchronously.

### Cancel All Orders by Symbol (Futures)
```
POST /api/mix/v1/order/cancel-order-by-symbol
```

**Request:**
```json
{
  "symbol": "BTCUSDT_UMCBL",
  "productType": "umcbl",
  "marginCoin": "USDT"
}
```

### Get Current Orders (Futures)
```
GET /api/mix/v1/order/current?symbol={symbol}&productType={type}
```
Returns open/pending orders.

### Get Order History (Futures)
```
GET /api/mix/v1/order/history?symbol={symbol}&productType={type}&startTime={ts}&endTime={ts}
```

### Get Order Details (Futures)
```
GET /api/mix/v1/order/detail?symbol={symbol}&productType={type}&orderId={id}
```
OR
```
GET /api/mix/v1/order/detail?symbol={symbol}&productType={type}&clientOid={clientOid}
```

### Get Fills (Futures)
```
GET /api/mix/v1/order/fills?symbol={symbol}&productType={type}&orderId={id}
```
Returns order execution details.

---

## Plan Orders (Stop Loss / Take Profit)

### Place Plan Order (Spot)
```
POST /api/spot/v1/trade/place-plan-order
```

**Request:**
```json
{
  "symbol": "BTCUSDT_SPBL",
  "side": "buy",
  "orderType": "limit",
  "triggerPrice": "49000.00",
  "executePrice": "49100.00",
  "size": "0.01",
  "triggerType": "fill_price",
  "clientOid": "plan_123"
}
```

### Place TP/SL Order (Futures)
```
POST /api/mix/v1/plan/placeTPSL
```

**Request:**
```json
{
  "symbol": "BTCUSDT_UMCBL",
  "productType": "umcbl",
  "marginCoin": "USDT",
  "planType": "normal_plan",
  "triggerPrice": "52000.00",
  "triggerType": "fill_price",
  "side": "close_long",
  "size": "0.01",
  "executePrice": "52000.00"
}
```

### Place Position TP/SL (Futures)
```
POST /api/mix/v1/plan/placePositionsTPSL
```
Set take-profit and stop-loss for entire position.

### Place Trailing Stop (Futures)
```
POST /api/mix/v1/plan/placeTrailStop
```

**Request:**
```json
{
  "symbol": "BTCUSDT_UMCBL",
  "productType": "umcbl",
  "marginCoin": "USDT",
  "triggerPrice": "51000.00",
  "side": "close_long",
  "size": "0.01",
  "rangeRate": "0.01"
}
```

---

## Account Trait Endpoints

### Get Account Info (Spot)
```
GET /api/spot/v1/account/getInfo
```

**Response Fields:**
- `userId`: User ID
- `inviterId`: Inviter ID
- `ips`: IP whitelist
- `authorities`: Permissions (trader, spotTrader, etc.)
- `parentId`: Parent account ID
- `trader`: Trader status

### Get Account Assets (Spot)
```
GET /api/spot/v1/account/assets?coin={coin}
```

**Response Fields:**
- `coin`: Currency
- `available`: Available balance
- `frozen`: Frozen balance
- `locked`: Locked balance
- `uTime`: Update time

### Get Account Assets Lite (Spot)
```
GET /api/spot/v1/account/assets-lite?coin={coin}
```
Optimized endpoint returning only non-zero balances.

### Get Account Bills (Spot)
```
GET /api/spot/v1/account/bills?coinId={id}&startTime={ts}&endTime={ts}
```
Returns transaction history.

**Response Fields:**
- `billId`: Bill ID
- `coin`: Currency
- `businessType`: Transaction type
- `amount`: Amount
- `balance`: Balance after
- `fees`: Fee
- `cTime`: Create time

### Get User Fee Rate
```
GET /api/user/v1/fee/query?symbol={symbol}&business={spot|futures}
```

**Response:**
```json
{
  "symbol": "BTCUSDT",
  "makerFeeRate": "0.002",
  "takerFeeRate": "0.006"
}
```

### Transfer Between Accounts (Spot)
```
POST /api/spot/v1/wallet/transfer
```

**Request:**
```json
{
  "fromType": "spot",
  "toType": "mix_usdt",
  "amount": "100.00",
  "coin": "USDT"
}
```

**Account Types:**
- `spot`: Spot account
- `mix_usdt`: USDT futures
- `mix_usd`: Coin-M futures
- `mix_usdc`: USDC futures
- `margin`: Margin account
- `crossed_margin`: Cross margin
- `isolated_margin`: Isolated margin

---

## Futures Account Endpoints

### Get Single Account (Futures)
```
GET /api/mix/v1/account/account?symbol={symbol}&productType={type}&marginCoin={coin}
```

**Response Fields:**
- `marginCoin`: Margin currency
- `locked`: Locked margin
- `available`: Available balance
- `crossMaxAvailable`: Max available (cross mode)
- `fixedMaxAvailable`: Max available (isolated mode)
- `maxTransferOut`: Max transferable
- `equity`: Account equity
- `usdtEquity`: USDT equity
- `btcEquity`: BTC equity
- `crossRiskRate`: Cross margin risk rate
- `unrealizedPL`: Unrealized P&L
- `bonus`: Bonus amount

### Get All Accounts (Futures)
```
GET /api/mix/v1/account/accounts?productType={type}
```
Returns all accounts for a product type.

### Set Leverage
```
POST /api/mix/v1/account/setLeverage
```

**Request:**
```json
{
  "symbol": "BTCUSDT_UMCBL",
  "productType": "umcbl",
  "marginCoin": "USDT",
  "leverage": "10",
  "holdSide": "long"
}
```

**Parameters:**
- `leverage`: Leverage value (cross margin mode)
- `longLeverage`: Long leverage (isolated mode)
- `shortLeverage`: Short leverage (isolated mode)
- `holdSide`: Position side (long/short)

**Rate Limit:** 5 req/sec

### Set Margin
```
POST /api/mix/v1/account/setMargin
```

**Request:**
```json
{
  "symbol": "BTCUSDT_UMCBL",
  "productType": "umcbl",
  "marginCoin": "USDT",
  "amount": "100.00",
  "holdSide": "long"
}
```

**Rate Limit:** 5 req/sec

### Set Margin Mode
```
POST /api/mix/v1/account/setMarginMode
```

**Request:**
```json
{
  "symbol": "BTCUSDT_UMCBL",
  "productType": "umcbl",
  "marginCoin": "USDT",
  "marginMode": "crossed"
}
```

**Margin Modes:**
- `crossed`: Cross margin (all funds shared)
- `fixed`: Isolated margin (per-position margin)

### Set Position Mode
```
POST /api/mix/v1/account/setPositionMode
```

**Request:**
```json
{
  "symbol": "BTCUSDT_UMCBL",
  "productType": "umcbl",
  "posMode": "single_hold"
}
```

**Position Modes:**
- `single_hold`: One-way position
- `double_hold`: Hedge mode (long + short simultaneously)

---

## Positions Trait Endpoints

### Get Single Position (Futures)
```
GET /api/mix/v1/position/singlePosition?symbol={symbol}&productType={type}&marginCoin={coin}
```

**Response Fields:**
- `marginCoin`: Margin currency
- `symbol`: Contract symbol
- `holdSide`: Position side (long/short/net)
- `openDelegateCount`: Open orders count
- `margin`: Position margin
- `available`: Available to close
- `locked`: Locked amount
- `total`: Total position size
- `leverage`: Leverage
- `achievedProfits`: Realized profit
- `averageOpenPrice`: Average entry price
- `marginMode`: Margin mode (crossed/fixed)
- `holdMode`: Position mode (single_hold/double_hold)
- `unrealizedPL`: Unrealized P&L
- `liquidationPrice`: Liquidation price
- `keepMarginRate`: Maintenance margin rate
- `marketPrice`: Current market price
- `cTime`: Create time
- `uTime`: Update time

### Get Single Position V2 (Futures)
```
GET /api/mix/v1/position/singlePosition-v2?symbol={symbol}&productType={type}&marginCoin={coin}
```
Enhanced version with additional fields.

### Get All Positions (Futures)
```
GET /api/mix/v1/position/allPosition?productType={type}&marginCoin={coin}
```
Returns all open positions for a product type.

### Get All Positions V2 (Futures)
```
GET /api/mix/v1/position/allPosition-v2?productType={type}&marginCoin={coin}
```
Enhanced version with extended information.

### Get Historical Positions (Futures)
```
GET /api/mix/v1/position/history-position?productType={type}&startTime={ts}&endTime={ts}&symbol={symbol}
```

**Response Fields:**
- `symbol`: Contract symbol
- `marginCoin`: Margin currency
- `holdSide`: Position side
- `openAvgPrice`: Average open price
- `closeAvgPrice`: Average close price
- `marginMode`: Margin mode
- `openTotalPos`: Total opened
- `closeTotalPos`: Total closed
- `pnl`: Profit and loss
- `netProfit`: Net profit
- `totalFunding`: Total funding fee
- `openFee`: Open fee
- `closeFee`: Close fee
- `cTime`: Create time
- `uTime`: Update time

### Calculate Max Open Size
```
POST /api/mix/v1/account/open-count
```

**Request:**
```json
{
  "symbol": "BTCUSDT_UMCBL",
  "productType": "umcbl",
  "marginCoin": "USDT",
  "openPrice": "50000.00",
  "openAmount": "1000.00",
  "leverage": "10"
}
```

**Response:**
```json
{
  "maxOpenSize": "0.20"
}
```

---

## Wallet Endpoints

### Get Deposit Address
```
GET /api/spot/v1/wallet/deposit-address?coin={coin}&chain={chain}
```

**Response:**
```json
{
  "coin": "USDT",
  "chain": "TRC20",
  "address": "TXxx...xxx",
  "tag": "",
  "url": "https://..."
}
```

### Withdraw
```
POST /api/spot/v1/wallet/withdrawal
```

**Request:**
```json
{
  "coin": "USDT",
  "transferType": "on_chain",
  "address": "0x...",
  "chain": "ERC20",
  "amount": "100.00",
  "remark": "withdrawal",
  "clientOid": "withdraw_123"
}
```

### Get Withdrawal List
```
GET /api/spot/v1/wallet/withdrawal-list?coin={coin}&startTime={ts}&endTime={ts}
```

### Get Deposit List
```
GET /api/spot/v1/wallet/deposit-list?coin={coin}&startTime={ts}&endTime={ts}
```

---

## Assets Overview (All Accounts)

### Get All Account Balances
```
GET /api/v2/account/all-account-balance
```

**Response:**
```json
{
  "code": "00000",
  "msg": "success",
  "data": [
    {
      "accountType": "spot",
      "assets": [
        {
          "coin": "USDT",
          "available": "1000.00",
          "frozen": "50.00",
          "locked": "0"
        }
      ]
    },
    {
      "accountType": "futures",
      "assets": [...]
    }
  ]
}
```

**Account Types in Response:**
- `spot`: Spot account
- `futures`: Futures account
- `funding`: Funding account
- `earn`: Earn products
- `bots`: Trading bots
- `margin`: Margin trading

---

## Sources

- [Bitget API Documentation](https://www.bitget.com/api-doc/common/intro)
- [Bitget Spot API Docs](https://bitgetlimited.github.io/apidoc/en/spot/)
- [Bitget Futures API Docs](https://bitgetlimited.github.io/apidoc/en/mix/)
- [Bitget API Guide](https://wundertrading.com/journal/en/learn/article/bitget-api)
