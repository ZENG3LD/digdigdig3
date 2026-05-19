# Bitget V1 to V2 API Migration Guide

This document provides a comprehensive migration guide from Bitget's deprecated V1 API to the new V2 API.

## Executive Summary

**Status**: V1 API was officially decommissioned on **November 28, 2025**.

All V1 REST endpoints now return error: **"The V1 API has been decommissioned. Please migrate to a newer version."**

**Impact**: Only REST API affected. WebSocket API works fine (12/12 tests passing).

**Good News**:
- Authentication mechanism unchanged (HMAC SHA256)
- Response structure very similar
- Most changes are endpoint path updates and symbol format simplification

## Quick Migration Checklist

- [ ] Update all endpoint paths from `/api/spot/v1/` to `/api/v2/spot/`
- [ ] Update futures paths from `/api/mix/v1/` to `/api/v2/mix/`
- [ ] Remove symbol suffixes (`_SPBL`, `_UMCBL`) - use plain format (`BTCUSDT`)
- [ ] Update pagination from `pageSize`/`pageNo` to `limit`/`idLessThan`
- [ ] Update some endpoint names (e.g., `open-orders` → `unfilled-orders`)
- [ ] Update response field parsing (minor field name changes)
- [ ] Test all endpoints thoroughly

## Major Changes

### 1. Endpoint Path Structure

**V1 Format**:
```
/api/spot/v1/...
/api/mix/v1/...
```

**V2 Format**:
```
/api/v2/spot/...
/api/v2/mix/...
```

**Pattern**: Version moved from second position to first position in path.

### 2. Symbol Format

**CRITICAL CHANGE**: Symbol format simplified in V2.

| Context | V1 Format | V2 Format |
|---------|-----------|-----------|
| Spot Trading | `BTCUSDT_SPBL` | `BTCUSDT` |
| USDT Futures | `BTCUSDT_UMCBL` | `BTCUSDT` |
| Coin Futures | `BTCUSD_DMCBL` | `BTCUSD` |
| USDC Futures | `BTCUSDC_CMCBL` | `BTCUSDC` |

**Migration**:
- **Request parameters**: Use plain symbol without suffix
- **Response parsing**: Expect plain symbol without suffix
- **Symbol formatting functions**: Remove suffix logic for V2

**Example**:
```rust
// V1
let symbol = format!("{}{}_{}", base, quote, "SPBL");  // BTCUSDT_SPBL

// V2
let symbol = format!("{}{}", base, quote);  // BTCUSDT
```

### 3. Pagination

**V1 Method**:
```json
{
  "pageSize": 100,
  "pageNo": 1
}
```

**V2 Method** (Cursor-based):
```json
{
  "limit": 100,
  "idLessThan": "1234567890"  // Start from this ID
}
```

**Benefits**: More efficient for large datasets, better performance.

**Migration**:
- Replace `pageSize` with `limit`
- Replace `pageNo` with cursor tracking using `idLessThan` (from `maxId`/`minId` in response)

### 4. Endpoint Name Changes

Some endpoints have been renamed for consistency:

| V1 Endpoint | V2 Endpoint |
|-------------|-------------|
| `open-orders` | `unfilled-orders` |
| `history` | `history-orders` |
| `allPosition` | `all-position` |
| `singlePosition` | `single-position` |
| `placeOrder` | `place-order` |
| `cancel-batch-orders` | `batch-cancel-order` |

**Pattern**: More consistent kebab-case naming.

## Detailed Migration Map

### Spot Market Data Endpoints

| Function | V1 Endpoint | V2 Endpoint | Notes |
|----------|-------------|-------------|-------|
| Server Time | `/api/spot/v1/public/time` | `/api/v2/public/time` | Path change only |
| Single Ticker | `/api/spot/v1/market/ticker` | `/api/v2/spot/market/tickers` | Same endpoint for single/all |
| All Tickers | `/api/spot/v1/market/tickers` | `/api/v2/spot/market/tickers` | Use `symbol` param for single |
| Orderbook | `/api/spot/v1/market/depth` | `/api/v2/spot/market/orderbook` | Name change + path |
| Candles | `/api/spot/v1/market/candles` | `/api/v2/spot/market/candles` | Path change only |
| History Candles | `/api/spot/v1/market/history-candles` | `/api/v2/spot/market/history-candles` | Path change only |
| Recent Fills | `/api/spot/v1/market/fills` | `/api/v2/spot/market/fills` | Path change only |
| Fills History | `/api/spot/v1/market/fills-history` | `/api/v2/spot/market/fills-history` | Path change only |
| Symbols/Products | `/api/spot/v1/public/products` | `/api/v2/spot/public/symbols` | Name change |
| Merged Depth | `/api/spot/v1/market/merge-depth` | `/api/v2/spot/market/merge-depth` | Path change only |

### Spot Trading Endpoints

| Function | V1 Endpoint | V2 Endpoint | Notes |
|----------|-------------|-------------|-------|
| Place Order | `/api/spot/v1/trade/orders` | `/api/v2/spot/trade/place-order` | Name change |
| Cancel Order | `/api/spot/v1/trade/cancel-order` | `/api/v2/spot/trade/cancel-order` | Path change only |
| Batch Orders | `/api/spot/v1/trade/batch-orders` | `/api/v2/spot/trade/batch-orders` | Path change only |
| Batch Cancel | `/api/spot/v1/trade/cancel-batch-orders` | `/api/v2/spot/trade/batch-cancel-order` | Name change |
| Order Info | `/api/spot/v1/trade/orderInfo` | `/api/v2/spot/trade/orderInfo` | Path change only |
| Open Orders | `/api/spot/v1/trade/open-orders` | `/api/v2/spot/trade/unfilled-orders` | Name change |
| Order History | `/api/spot/v1/trade/history` | `/api/v2/spot/trade/history-orders` | Name change |
| Fills | `/api/spot/v1/trade/fills` | `/api/v2/spot/trade/fills` | Path change only |

### Spot Account Endpoints

| Function | V1 Endpoint | V2 Endpoint | Notes |
|----------|-------------|-------------|-------|
| Account Assets | `/api/spot/v1/account/assets` | `/api/v2/spot/account/assets` | Path change only |
| Account Info | `/api/spot/v1/account/getInfo` | `/api/v2/spot/account/info` | Name change |
| Subaccount Assets | `/api/spot/v1/account/sub-account-spot-assets` | `/api/v2/spot/account/subaccount-assets` | Name change |
| Bills | `/api/spot/v1/account/bills` | `/api/v2/spot/account/bills` | Path change only |
| Transfer | `/api/spot/v1/wallet/transfer` | `/api/v2/spot/wallet/transfer` | Path change only |
| Withdrawal | `/api/spot/v1/wallet/withdrawal` | `/api/v2/spot/wallet/withdrawal` | Path change only |

### Futures Market Data Endpoints

| Function | V1 Endpoint | V2 Endpoint | Notes |
|----------|-------------|-------------|-------|
| Ticker | `/api/mix/v1/market/ticker` | `/api/v2/mix/market/ticker` | Path change only |
| All Tickers | `/api/mix/v1/market/tickers` | `/api/v2/mix/market/tickers` | Path change only |
| Orderbook | `/api/mix/v1/market/depth` | `/api/v2/mix/market/merge-depth` | Name change |
| Candles | `/api/mix/v1/market/candles` | `/api/v2/mix/market/candles` | Path change only |
| Fills | `/api/mix/v1/market/fills` | `/api/v2/mix/market/fills` | Path change only |
| Contracts | `/api/mix/v1/market/contracts` | `/api/v2/mix/market/contracts` | Path change only |
| Funding Rate | `/api/mix/v1/market/funding-rate` | `/api/v2/mix/market/current-fund-rate` | Name change |

### Futures Trading Endpoints

| Function | V1 Endpoint | V2 Endpoint | Notes |
|----------|-------------|-------------|-------|
| Place Order | `/api/mix/v1/order/placeOrder` | `/api/v2/mix/order/place-order` | Name change |
| Cancel Order | `/api/mix/v1/order/cancel-order` | `/api/v2/mix/order/cancel-order` | Path change only |
| Batch Place | `/api/mix/v1/order/batch-orders` | `/api/v2/mix/order/batch-place-order` | Name change |
| Batch Cancel | `/api/mix/v1/order/cancel-batch-orders` | `/api/v2/mix/order/batch-cancel-orders` | Path change only |
| Order Detail | `/api/mix/v1/order/detail` | `/api/v2/mix/order/detail` | Path change only |
| Pending Orders | `/api/mix/v1/order/current` | `/api/v2/mix/order/orders-pending` | Name change |
| Order History | `/api/mix/v1/order/history` | `/api/v2/mix/order/orders-history` | Name change |
| Fills | `/api/mix/v1/order/fills` | `/api/v2/mix/order/fills` | Path change only |

### Futures Account/Position Endpoints

| Function | V1 Endpoint | V2 Endpoint | Notes |
|----------|-------------|-------------|-------|
| Single Account | `/api/mix/v1/account/account` | `/api/v2/mix/account/account` | Path change only |
| All Accounts | `/api/mix/v1/account/accounts` | `/api/v2/mix/account/accounts` | Path change only |
| All Positions | `/api/mix/v1/position/allPosition` | `/api/v2/mix/position/all-position` | Name change |
| Single Position | `/api/mix/v1/position/singlePosition` | `/api/v2/mix/position/single-position` | Name change |
| Set Leverage | `/api/mix/v1/account/setLeverage` | `/api/v2/mix/account/set-leverage` | Name change |
| Set Margin | `/api/mix/v1/account/setMargin` | `/api/v2/mix/account/set-margin` | Name change |
| Set Margin Mode | `/api/mix/v1/account/setMarginMode` | `/api/v2/mix/account/set-margin-mode` | Name change |
| Account Bills | `/api/mix/v1/account/accountBill` | `/api/v2/mix/account/bill` | Name change |

## Response Format Changes

### Overall Structure

**Unchanged**: V2 maintains the same wrapper structure as V1.

```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695865615662,
  "data": { ... }
}
```

### Field Name Changes

Some response fields have been renamed for consistency:

| V1 Field | V2 Field | Context |
|----------|----------|---------|
| `last` | `lastPr` | Ticker - last price |
| `bestAsk` | `askPr` | Ticker - ask price |
| `bestBid` | `bidPr` | Ticker - bid price |
| `askSz` | `askSz` | No change |
| `bidSz` | `bidSz` | No change |

**Pattern**: More consistent use of `Pr` suffix for prices.

### Symbol Format in Responses

**CRITICAL**: All responses return plain symbols without suffixes.

**V1 Response**:
```json
{
  "symbol": "BTCUSDT_SPBL"
}
```

**V2 Response**:
```json
{
  "symbol": "BTCUSDT"
}
```

**Migration**: Update symbol parsing to expect plain format.

## Code Migration Examples

### Example 1: Update Endpoint Enum

**Before (V1)**:
```rust
impl BitgetEndpoint {
    pub fn path(&self) -> &'static str {
        match self {
            Self::SpotTicker => "/api/spot/v1/market/ticker",
            Self::SpotOrderbook => "/api/spot/v1/market/depth",
            Self::SpotCreateOrder => "/api/spot/v1/trade/orders",
            Self::SpotOpenOrders => "/api/spot/v1/trade/open-orders",
            // ...
        }
    }
}
```

**After (V2)**:
```rust
impl BitgetEndpoint {
    pub fn path(&self) -> &'static str {
        match self {
            Self::SpotTicker => "/api/v2/spot/market/tickers",
            Self::SpotOrderbook => "/api/v2/spot/market/orderbook",
            Self::SpotCreateOrder => "/api/v2/spot/trade/place-order",
            Self::SpotOpenOrders => "/api/v2/spot/trade/unfilled-orders",
            // ...
        }
    }
}
```

### Example 2: Update Symbol Formatting

**Before (V1)**:
```rust
pub fn format_symbol(base: &str, quote: &str, account_type: AccountType) -> String {
    let base = base.to_uppercase();
    let quote = quote.to_uppercase();

    match account_type {
        AccountType::Spot => format!("{}{}_{}", base, quote, "SPBL"),
        AccountType::FuturesCross | AccountType::FuturesIsolated => {
            match quote.as_str() {
                "USDT" => format!("{}{}_{}", base, quote, "UMCBL"),
                "USD" => format!("{}{}_{}", base, quote, "DMCBL"),
                _ => format!("{}{}_{}", base, quote, "UMCBL"),
            }
        }
    }
}
```

**After (V2)**:
```rust
pub fn format_symbol(base: &str, quote: &str, _account_type: AccountType) -> String {
    let base = base.to_uppercase();
    let quote = quote.to_uppercase();

    // V2 uses plain format for all account types
    format!("{}{}", base, quote)
}
```

### Example 3: Update Response Parsing

**Before (V1)**:
```rust
#[derive(Debug, Deserialize)]
pub struct TickerResponse {
    pub symbol: String,           // "BTCUSDT_SPBL"
    pub last: String,              // V1 field name
    #[serde(rename = "bestAsk")]
    pub best_ask: String,
    #[serde(rename = "bestBid")]
    pub best_bid: String,
    // ...
}

// After parsing, strip suffix
let plain_symbol = response.symbol.replace("_SPBL", "")
    .replace("_UMCBL", "")
    .replace("_DMCBL", "");
```

**After (V2)**:
```rust
#[derive(Debug, Deserialize)]
pub struct TickerResponse {
    pub symbol: String,           // "BTCUSDT" (already plain)
    #[serde(rename = "lastPr")]
    pub last_pr: String,          // V2 field name
    #[serde(rename = "askPr")]
    pub ask_pr: String,
    #[serde(rename = "bidPr")]
    pub bid_pr: String,
    // ...
}

// No suffix stripping needed
let plain_symbol = response.symbol;  // Already plain
```

### Example 4: Update Pagination

**Before (V1)**:
```rust
let mut params = HashMap::new();
params.insert("symbol", symbol);
params.insert("pageSize", "100");
params.insert("pageNo", "1");

// Next page
params.insert("pageNo", "2");
```

**After (V2)**:
```rust
let mut params = HashMap::new();
params.insert("symbol", symbol);
params.insert("limit", "100");
// First request: no idLessThan (starts from latest)

// Parse response
let response: OrdersResponse = /* ... */;
let min_id = response.data.min_id;

// Next page: use minId from previous response
params.insert("idLessThan", &min_id);
```

## New Features in V2

V2 adds several new endpoints not available in V1:

### 1. Cancel-Replace (Modify Order)
- Spot: `POST /api/v2/spot/trade/cancel-replace-order`
- Futures: `POST /api/v2/mix/order/modify-order`

**Benefit**: Atomic operation - cancel and place new order in single request.

### 2. Enhanced Plan/Trigger Orders
- Place: `/api/v2/spot/trade/place-plan-order`
- Modify: `/api/v2/spot/trade/modify-plan-order`
- Cancel: `/api/v2/spot/trade/cancel-plan-order`

**Benefit**: Better support for conditional orders.

### 3. Historical Candles (Index/Mark Price)
- Index: `/api/v2/mix/market/history-index-candles`
- Mark: `/api/v2/mix/market/history-mark-candles`

**Benefit**: Access historical index and mark price data.

### 4. Position Management
- Close positions: `/api/v2/mix/order/close-positions`
- Flash close: `/api/v2/mix/order/click-backhand`
- Set position mode: `/api/v2/mix/account/set-position-mode`

**Benefit**: More granular position control.

### 5. Coins Info
- Spot: `/api/v2/spot/public/coins`

**Benefit**: Get detailed coin/currency information.

## What Stays the Same

### ✅ Unchanged Components

1. **Authentication**: HMAC SHA256 signature algorithm (see `authentication_v2.md`)
2. **Headers**: Same header names and format
3. **Response Structure**: Same wrapper with `code`, `msg`, `requestTime`, `data`
4. **Data Types**: All numeric values still returned as strings
5. **Timestamps**: Still in milliseconds
6. **Rate Limits**: Similar structure (per-endpoint limits)
7. **WebSocket API**: Separate from REST, already using V2

## Migration Strategy

### Phase 1: Preparation (1-2 hours)

1. **Read Documentation**:
   - Review `endpoints_v2.md`
   - Review `response_formats_v2.md`
   - Review `authentication_v2.md` (confirm no auth changes)

2. **Identify Affected Code**:
   - List all V1 endpoints used
   - Identify symbol formatting logic
   - Identify response parsing code
   - Identify pagination code

3. **Plan Changes**:
   - Map V1 endpoints to V2 equivalents
   - Identify field name changes needed
   - Plan testing approach

### Phase 2: Code Updates (2-4 hours)

1. **Update Endpoints Module** (`endpoints.rs`):
   - Change all endpoint paths
   - Update endpoint names where changed
   - Update symbol formatting function

2. **Update Parser Module** (`parser.rs`):
   - Update field names in structs (e.g., `last` → `lastPr`)
   - Remove symbol suffix stripping logic
   - Update pagination response parsing

3. **Update Connector Module** (`connector.rs`):
   - Update request parameter building (pagination)
   - Test each trait method

### Phase 3: Testing (2-4 hours)

1. **Unit Tests**:
   - Test endpoint path generation
   - Test symbol formatting
   - Test response parsing

2. **Integration Tests**:
   - Test market data endpoints (public)
   - Test trading endpoints (private, on testnet)
   - Test pagination with multiple pages

3. **Manual Testing**:
   - Verify ticker data
   - Verify orderbook
   - Place and cancel test orders
   - Check account balances

### Phase 4: Deployment

1. **Update Documentation**
2. **Deploy to testnet first**
3. **Monitor for errors**
4. **Deploy to production**

## Current Connector Status

**Location**: `src/exchanges/bitget/`

**Current State**:
- WebSocket: ✅ Working (V2, 12/12 tests pass)
- REST: ❌ Broken (V1, returns decommission error)

**Files to Update**:
1. `endpoints.rs` - Update all endpoint paths and symbol formatting
2. `parser.rs` - Update response field names
3. `connector.rs` - Update pagination logic
4. `auth.rs` - No changes needed (authentication unchanged)

## Testing Checklist

After migration, verify:

### Market Data (Public Endpoints)
- [ ] Server time
- [ ] Single ticker
- [ ] All tickers
- [ ] Orderbook
- [ ] Klines/candles
- [ ] Recent trades
- [ ] Symbol info

### Trading (Private Endpoints - Testnet)
- [ ] Place limit order
- [ ] Place market order
- [ ] Cancel order
- [ ] Query order
- [ ] Get open orders
- [ ] Get order history

### Account (Private Endpoints)
- [ ] Get account balance
- [ ] Get account info

### Symbol Format
- [ ] Request uses plain symbols (no suffix)
- [ ] Response parses plain symbols correctly
- [ ] Symbol formatting function returns plain format

### Pagination
- [ ] First page loads
- [ ] Next page with cursor works
- [ ] Handles empty results

## Rollback Plan

If V2 migration fails:

1. **Not possible** - V1 API is permanently decommissioned
2. **Only option**: Fix V2 issues or use alternative exchange
3. **WebSocket fallback**: Use WebSocket for market data if REST fails

## Support Resources

- **Official Docs**: https://www.bitget.com/api-doc/
- **V2 Update Guide**: https://www.bitget.com/api-doc/common/release-note
- **Changelog**: https://www.bitget.com/api-doc/common/changelog
- **Support**: https://www.bitget.com/support/

## Timeline

| Event | Date | Status |
|-------|------|--------|
| V2 API Released | October 13, 2023 | ✅ |
| V1 Deprecation Announced | Early 2024 | ✅ |
| V1 API Decommissioned | November 28, 2025 | ✅ |
| **Current Status** | **January 20, 2026** | **Must migrate** |

## Priority for Current Project

Based on current connector usage, prioritize these endpoints:

### High Priority (Market Data)
1. ✅ `/api/v2/spot/market/tickers` - Already working via WebSocket
2. 🔴 `/api/v2/spot/market/orderbook` - **CRITICAL** for REST
3. 🔴 `/api/v2/spot/market/candles` - **CRITICAL** for REST
4. 🔴 `/api/v2/spot/public/symbols` - **CRITICAL** for symbol info

### Medium Priority (Trading)
5. 🟡 `/api/v2/spot/trade/place-order` - If trading enabled
6. 🟡 `/api/v2/spot/trade/cancel-order` - If trading enabled
7. 🟡 `/api/v2/spot/trade/unfilled-orders` - If trading enabled

### Low Priority (Account)
8. 🟢 `/api/v2/spot/account/assets` - If account info needed

Legend:
- ✅ Working
- 🔴 Critical, broken
- 🟡 Important, needed for trading
- 🟢 Nice to have

## Summary

**Bottom Line**: V2 migration is **straightforward** but **mandatory**.

**Main Changes**:
1. Update endpoint paths
2. Remove symbol suffixes
3. Update some field names
4. Change pagination method

**No Changes**:
1. Authentication (HMAC SHA256)
2. Response structure
3. WebSocket API

**Estimated Effort**: 4-8 hours for complete migration and testing.

## Sources

- [Bitget V2 API Update Guide](https://www.bitget.com/api-doc/common/release-note)
- [Bitget V1 API Deprecation Notice](https://www.bitget.com/support/articles/12560603838361)
- [Bitget API V2 Release Announcement](https://www.bitget.com/support/articles/12560603798900)
- [Bitget API Introduction](https://www.bitget.com/api-doc/common/intro)
- [Bitget API Changelog](https://www.bitget.com/api-doc/common/changelog)
- [tiagosiebler/bitget-api GitHub Reference](https://github.com/tiagosiebler/bitget-api)
