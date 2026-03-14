# Lighter DEX — Open Orders Research

**Date**: 2026-03-14
**Status**: Active endpoint confirmed — `GET /api/v1/accountActiveOrders` EXISTS
**Bottom line**: The connector's `get_open_orders` comment "Lighter has no REST endpoint for listing open orders" is WRONG. The endpoint exists and was always there.

---

## 1. The Endpoint EXISTS

### `GET /api/v1/accountActiveOrders`

**Base URL**: `https://mainnet.zklighter.elliot.ai`
**Full URL**: `https://mainnet.zklighter.elliot.ai/api/v1/accountActiveOrders`

This endpoint retrieves all currently open/active orders for an account. It is explicitly listed in the official Lighter API docs at `https://apidocs.lighter.xyz/reference/accountactiveorders`.

#### Query Parameters

| Parameter      | Type   | Required | Description |
|----------------|--------|----------|-------------|
| `account_index` | int64 | YES      | The account identifier (numeric index, not L1 address) |
| `market_id`    | int16  | NO       | Filter by market. If omitted, returns active orders for ALL markets |
| `auth`         | string | NO       | Auth token (currently optional per docs, but will become required) |
| `authorization`| string | NO       | Alternative: pass auth token as a header |

**Note on auth**: Both `auth` query param and `Authorization` header are currently marked `required: false` in the API spec with a comment "make required after integ is done". This means the endpoint MAY work without auth right now, but this will change. Always pass auth for production use.

#### Response Structure

```json
{
  "code": 200,
  "message": "string",
  "next_cursor": "string",
  "orders": [
    {
      "order_index": 1,
      "client_order_index": 234,
      "order_id": "1",
      "client_order_id": "234",
      "market_index": 1,
      "owner_account_index": 1,
      "initial_base_amount": "0.1",
      "price": "3024.66",
      "nonce": 722,
      "remaining_base_amount": "0.1",
      "is_ask": true,
      "base_size": 12354,
      "base_price": 3024,
      "filled_base_amount": "0.1",
      "filled_quote_amount": "0.1",
      "side": "buy",
      "type": "limit",
      "time_in_force": "good-till-time",
      "reduce_only": true,
      "trigger_price": "3024.66",
      "order_expiry": 1640995200,
      "status": "open",
      "trigger_status": "na",
      "trigger_time": 1640995200,
      "parent_order_index": 1,
      "parent_order_id": "1",
      "to_trigger_order_id_0": "1",
      "to_trigger_order_id_1": "1",
      "to_cancel_order_id_0": "1",
      "block_height": 45434,
      "timestamp": 1640995200,
      "created_at": 1640995200,
      "updated_at": 1640995200,
      "transaction_time": 1640995200,
      "integrator_fee_collector_index": "string",
      "integrator_maker_fee": "string",
      "integrator_taker_fee": "string"
    }
  ]
}
```

#### Field Reference

| Field | Type | Description |
|-------|------|-------------|
| `order_index` | int64 | Lighter's internal order ID (use this for cancellation) |
| `client_order_index` | int64 | Client-provided order index (from order creation) |
| `order_id` | string | String version of order_index |
| `client_order_id` | string | String version of client_order_index |
| `market_index` | uint8 | Market ID (0=BTC-USDC, 1=ETH-USDC, etc.) |
| `owner_account_index` | int64 | Account that placed the order |
| `initial_base_amount` | string | Original quantity |
| `remaining_base_amount` | string | Unfilled quantity remaining |
| `price` | string | Limit price (decimal string) |
| `filled_base_amount` | string | Quantity already filled |
| `filled_quote_amount` | string | Quote currency filled |
| `side` | string | "buy" or "sell" |
| `type` | string | "limit", "market", "stop-loss", "stop-loss-limit", "take-profit", "take-profit-limit", "twap", "twap-sub", "liquidation" |
| `time_in_force` | string | "good-till-time", "immediate-or-cancel", "post-only", "Unknown" |
| `reduce_only` | bool | Whether order is reduce-only |
| `trigger_price` | string | For conditional orders |
| `order_expiry` | int64 | Unix timestamp when order expires |
| `status` | string | "in-progress", "pending", "open", "filled", "canceled" |
| `trigger_status` | string | "na", "ready", "mark-price", "twap", "parent-order" |
| `is_ask` | bool | true = sell/ask, false = buy/bid |
| `block_height` | int64 | L2 block when order was created |
| `timestamp` | int64 | Unix timestamp |
| `created_at` | int64 | Creation time |
| `updated_at` | int64 | Last update time |
| `transaction_time` | int64 | Time of on-chain transaction |
| `nonce` | int64 | Transaction nonce used |

#### Pagination

The response includes `next_cursor`. To page through results, pass the returned `next_cursor` value as a `cursor` query parameter in the next request.

---

## 2. Related Endpoints

### `GET /api/v1/accountInactiveOrders` (already in connector)

Returns filled, cancelled, and expired orders (trade history).

**Additional parameters vs accountActiveOrders**:

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `account_index` | int64 | YES | Account identifier |
| `market_id` | int16 | NO | Filter by market (default: 255 = all) |
| `ask_filter` | int8 | NO | Filter by side (-1 = all) |
| `between_timestamps` | string | NO | Timestamp range filter |
| `cursor` | string | NO | Pagination cursor |
| `limit` | int64 | YES | Results per page (1-100) |

**Rate limit weight**: 100 (standard account gets 60 req/min, so this costs 100/60 = 1.67 min equivalent — very heavy, use pagination wisely)

### `GET /api/v1/accountOrders` (NOT in connector yet)

Returns ALL orders (both active and inactive combined). Confirmed via Python SDK method `account_orders` mapping to `GET /api/v1/accountOrders`.

### `GET /api/v1/orderBookOrders` (already in connector)

Public orderbook bids and asks (all accounts, anonymous). Not account-specific.

**Parameters**:
- `market_id` (int16, required)
- `limit` (int64, required, max 250)

---

## 3. WebSocket: Real-Time Order Updates

Two WebSocket channels provide real-time order state — useful as a complement to REST polling.

### Channel: `account_all_orders/{ACCOUNT_ID}`

Subscribes to order updates across ALL markets for an account.

**Subscription message**:
```json
{
  "type": "subscribe",
  "channel": "account_all_orders/{ACCOUNT_ID}",
  "auth": "{AUTH_TOKEN}"
}
```

**Update message**:
```json
{
  "channel": "account_all_orders:{ACCOUNT_ID}",
  "type": "update/account_all_orders",
  "orders": {
    "0": [<Order>, <Order>],
    "1": [<Order>]
  }
}
```

The `orders` object is a map of `market_index` (string) → array of `Order` objects.

### Channel: `account_orders/{MARKET_INDEX}/{ACCOUNT_ID}`

Subscribes to order updates for a specific market only.

**Subscription message**:
```json
{
  "type": "subscribe",
  "channel": "account_orders/{MARKET_INDEX}/{ACCOUNT_ID}",
  "auth": "{AUTH_TOKEN}"
}
```

**Update message**:
```json
{
  "channel": "account_orders:{MARKET_INDEX}",
  "type": "update/account_orders",
  "account": 12345,
  "nonce": 42,
  "orders": {
    "1": [<Order>]
  }
}
```

Both channels deliver the full Order object (same schema as REST response).

**Note**: Auth token is required for all account WebSocket channels. The same auth token format applies (ECgFp5+Poseidon2 Schnorr signature — not implemented in current connector).

---

## 4. On-Chain / GraphQL

No GraphQL API was found. Lighter is a ZK-rollup L2 — order state is maintained off-chain in the rollup's state, not directly queryable via standard EVM contract calls. The REST API (`mainnet.zklighter.elliot.ai`) is the canonical read interface. The explorer at `explorer.elliot.ai` offers an additional query surface but with lower rate limits.

---

## 5. Auth Token Requirement for `accountActiveOrders`

The endpoint requires an auth token for production use. Auth token format (already documented in `auth.rs`):

- **Standard token**: `{expiry_unix}:{account_index}:{api_key_index}:{schnorr_signature}`
- **Read-only token**: `ro:{account_index}:{single|all}:{expiry_unix}:{schnorr_signature}`

The signing uses **ECgFp5 + Poseidon2** over the Goldilocks field — NOT standard ECDSA. The connector currently returns `ExchangeError::Auth` for all token generation.

**Workaround strategy for the connector**:

Since the endpoint currently accepts requests without auth (both params are `required: false`), the connector can attempt the call without auth token and fall back to an empty list if auth is required. This is a temporary measure until proper ECgFp5 signing is integrated.

```
GET https://mainnet.zklighter.elliot.ai/api/v1/accountActiveOrders?account_index=12345
```

This should work NOW. Whether it continues to work without auth is uncertain.

---

## 6. What the Connector Is Missing

### Critical Missing: `AccountActiveOrders` endpoint variant

The `LighterEndpoint` enum in `endpoints.rs` has `AccountInactiveOrders` but is **missing** `AccountActiveOrders`. This is the direct fix needed.

**Fix required in `endpoints.rs`**:
```rust
// In LighterEndpoint enum, ACCOUNT section — add:
AccountActiveOrders,  // GET /api/v1/accountActiveOrders

// In path() impl — add:
Self::AccountActiveOrders => "/api/v1/accountActiveOrders",
```

### Critical Missing: `get_open_orders` implementation

The current `get_open_orders` in `connector.rs` returns `UnsupportedOperation` with the comment "Lighter has no REST endpoint for listing open orders." This is incorrect. The fix:

```rust
async fn get_open_orders(
    &self,
    symbol: Option<&str>,
    account_type: AccountType,
) -> ExchangeResult<Vec<Order>> {
    let account_index = self._auth.as_ref()
        .and_then(|a| a.account_index())
        .ok_or_else(|| ExchangeError::Auth(
            "Lighter get_open_orders requires account_index in credentials passphrase JSON.".to_string()
        ))?;

    let mut params = HashMap::new();
    params.insert("account_index".to_string(), account_index.to_string());

    // Optional market filter
    if let Some(sym) = symbol {
        if let Ok(market_id) = self.get_market_id(sym, account_type).await {
            params.insert("market_id".to_string(), market_id.to_string());
        }
    }

    // Auth token — currently optional on server side, but pass if available
    // TODO: generate auth token once ECgFp5 signing is implemented
    // if let Some(token) = self._auth.as_ref().and_then(|a| a.generate_auth_token(3600).ok()) {
    //     params.insert("auth".to_string(), token);
    // }

    let url = self._urls.rest_url().to_string() + LighterEndpoint::AccountActiveOrders.path();
    let response = self._client.get(&url, &params, &HashMap::new()).await?;
    let orders = LighterParser::parse_open_orders(&response)?;
    Ok(orders)
}
```

### Missing: `AccountOrders` endpoint (all orders combined)

Not in the enum. Add as `AccountOrders` → `/api/v1/accountOrders`.

### Missing: Parser method `parse_open_orders`

The `LighterParser` needs a `parse_open_orders(v: &Value) -> ExchangeResult<Vec<Order>>` method that reads the `orders` array from the `accountActiveOrders` response.

The response schema is identical to `accountInactiveOrders`, so the inactive orders parser (if it exists) can be reused or shared.

---

## 7. Other Potentially Missing Operations

Based on reviewing the connector and API docs:

| Operation | Status | API Availability | Notes |
|-----------|--------|-----------------|-------|
| `get_open_orders` | WRONG (returns UnsupportedOperation) | YES — `GET /api/v1/accountActiveOrders` | Fix: implement with this endpoint |
| `get_order_history` | Partial | YES — `GET /api/v1/accountInactiveOrders` | Already has the endpoint in enum |
| `get_order_by_id` | Partial | Via `accountInactiveOrders` + filter | Uses current pattern of checking active then inactive |
| `get_positions` | Implemented | YES — embedded in `GET /api/v1/account` response | Already works |
| `get_account_info` | Implemented | YES — `GET /api/v1/account` | Already works |
| `get_balances` | Implemented | YES — from `account.assets` | Already works |
| `get_trades` | Check needed | YES — `GET /api/v1/trades` with `account_index` param | May be partially implemented |
| `get_funding_rates` | Check needed | YES — `GET /api/v1/fundings` | |
| `get_pnl` | Missing | YES — `GET /api/v1/pnl` | Endpoint in enum but no trait impl |
| `cancel_all_orders` | Not in Trading trait | YES — cancel_all via `tx_type=L2CancelAll` | Would need ZK signing |
| WebSocket orders | Stub | YES — `account_all_orders/{id}` channel | Needs auth token |

---

## 8. Rate Limits Summary

Relevant to order querying:

| Endpoint | Weight | Standard account limit |
|----------|--------|----------------------|
| `accountActiveOrders` | 300 (default) | 60 req/min total → ~12 calls/min for this endpoint |
| `accountInactiveOrders` | 100 | 60 req/min total → ~36 calls/min (but 100 weight each = only 0.6/min!) |
| `trades` | 600 | 60 req/min total → very expensive, 1 call costs 10x budget |
| `account` | 300 | ~12 calls/min |

**Standard account budget**: 60 weighted requests per rolling minute
**Premium account budget**: 24,000 weighted requests per rolling minute

The `accountInactiveOrders` weight of 100 means a standard account can only call it 0.6 times per minute before exhausting the entire rate limit budget. Use pagination + caching aggressively.

---

## 9. Implementation Plan

### Minimal fix (get_open_orders works):

1. **`endpoints.rs`**: Add `AccountActiveOrders` variant to `LighterEndpoint` enum, add its path `/api/v1/accountActiveOrders`.

2. **`parser.rs`**: Add `parse_open_orders(v: &Value) -> ExchangeResult<Vec<Order>>` — reuse `parse_inactive_orders` logic if it exists, both responses share the same Order schema.

3. **`connector.rs`**: Replace the `UnsupportedOperation` return in `get_open_orders` with an actual HTTP call to `AccountActiveOrders`.

### Full fix (production-ready):

4. **`auth.rs`**: Implement ECgFp5+Poseidon2 auth token generation (requires the `lighter-sdk` crate or a port of the TypeScript signing logic). Until then, attempt unauthenticated call and document the limitation.

5. **WebSocket `connector.rs`**: Subscribe to `account_all_orders/{id}` for real-time order tracking (eliminates polling).

---

## 10. Sources

- [accountActiveOrders — Lighter API Reference](https://apidocs.lighter.xyz/reference/accountactiveorders)
- [accountInactiveOrders — Lighter API Reference](https://apidocs.lighter.xyz/reference/accountinactiveorders)
- [WebSocket Reference — Lighter API Docs](https://apidocs.lighter.xyz/docs/websocket-reference)
- [Rate Limits — Lighter API Docs](https://apidocs.lighter.xyz/docs/rate-limits)
- [Get Started for Programmers — Lighter API Docs](https://apidocs.lighter.xyz/docs/get-started-for-programmers-1)
- [Order API — elliottech/lighter-python DeepWiki](https://deepwiki.com/elliottech/lighter-python/4.2-retrieving-account-and-market-information)
- [lighter-go client package — Go Packages](https://pkg.go.dev/github.com/drinkthere/lighter-go/client)
- [Issue #31: Critical: Need fetch_order routine — elliottech/lighter-python](https://github.com/elliottech/lighter-python/issues/31)
- [elliottech/lighter-python — GitHub](https://github.com/elliottech/lighter-python)
- [elliottech/lighter-go — GitHub](https://github.com/elliottech/lighter-go)
