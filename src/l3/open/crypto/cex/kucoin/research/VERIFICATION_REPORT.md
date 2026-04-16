# KuCoin Connector Verification Report

**Date**: 2026-01-20
**Status**: Research Complete - Issues Found

---

## Executive Summary

Research agents verified the KuCoin connector against official API documentation. The implementation is **mostly correct** but has **several issues** that need fixing before it can be a reliable reference implementation.

| Area | Status | Issues Found |
|------|--------|--------------|
| **endpoints.rs** | ✅ CORRECT | All endpoint paths verified |
| **auth.rs** | ✅ CORRECT | Signature algorithm matches docs |
| **parser.rs** | ⚠️ BUGS FOUND | 1 critical, 2 minor |
| **Symbol formatting** | ⚠️ BUG FOUND | Missing "T" in USDTM suffix |
| **Kline intervals** | ⚠️ INCOMPLETE | Futures uses different format |

---

## Critical Issues

### 1. ❌ CRITICAL: Bid/Ask Swap in parse_ticker()

**File**: [parser.rs:142-143](../parser.rs#L142-L143)

**Problem**: KuCoin uses counterintuitive naming:
- `"buy"` = bestAsk (price to buy at)
- `"sell"` = bestBid (price to sell at)

**Current (WRONG)**:
```rust
bid_price: Self::get_f64(data, "buy"),   // WRONG!
ask_price: Self::get_f64(data, "sell"),  // WRONG!
```

**Should be**:
```rust
bid_price: Self::get_f64(data, "sell"),  // bestBid
ask_price: Self::get_f64(data, "buy"),   // bestAsk
```

**Impact**: All bid/ask prices are swapped, causing wrong spread calculations.

---

### 2. ❌ CRITICAL: Wrong Futures Symbol Format

**File**: [endpoints.rs:216-226](../endpoints.rs#L216-L226)

**Problem**: Missing "T" in USDT-margined futures symbol.

**Current (WRONG)**:
```rust
format!("{}{}M", base, quote)  // Produces "XBTUSDM" for USDT quote
```

**Should be**:
```rust
match quote.to_uppercase().as_str() {
    "USDT" => format!("{}USDTM", base),  // XBTUSDTM
    "USD" => format!("{}USDM", base),    // XBTUSDM
    _ => format!("{}{}M", base, quote),
}
```

**Impact**: All USDT-margined futures requests will fail with "symbol not found".

---

## Medium Issues

### 3. ⚠️ Futures Kline Interval Format

**File**: [endpoints.rs:230-248](../endpoints.rs#L230-L248) + [connector.rs:349-350](../connector.rs#L349-L350)

**Problem**: Futures API uses numeric granularity (minutes as integer), not string intervals.

**Current**: Uses `map_kline_interval()` for both Spot and Futures.

**Spot format**: `"1min"`, `"1hour"`, `"1day"` (strings)
**Futures format**: `1`, `60`, `1440` (integers - minutes)

**Fix needed**: Separate function for futures granularity.

```rust
pub fn map_futures_granularity(interval: &str) -> u32 {
    match interval {
        "1m" => 1,
        "5m" => 5,
        "15m" => 15,
        "30m" => 30,
        "1h" => 60,
        "2h" => 120,
        "4h" => 240,
        "8h" => 480,
        "12h" => 720,
        "1d" => 1440,
        "1w" => 10080,
        _ => 60,
    }
}
```

---

### 4. ⚠️ Orderbook Timestamp Field

**File**: [parser.rs:128](../parser.rs#L128)

**Problem**: Futures orderbook uses `ts` field, Spot uses `time`.

**Current**:
```rust
timestamp: data.get("time").and_then(|t| t.as_i64()).unwrap_or(0),
```

**Should be**:
```rust
timestamp: data.get("ts")
    .or_else(|| data.get("time"))
    .and_then(|t| t.as_i64())
    .unwrap_or(0),
```

---

### 5. ⚠️ Orderbook Sequence Type

**File**: [parser.rs:131](../parser.rs#L131)

**Problem**: Sequence is returned as integer, not string.

**Current**:
```rust
sequence: Self::get_str(data, "sequence").map(String::from),
```

**Should handle both**:
```rust
sequence: data.get("sequence")
    .and_then(|s| {
        s.as_i64().map(|n| n.to_string())
            .or_else(|| s.as_str().map(String::from))
    }),
```

---

## Minor Issues / Improvements

### 6. Missing Predicted Funding Rate

**File**: [parser.rs:155-166](../parser.rs#L155-L166)

KuCoin returns `predictedValue` in funding rate response, but we don't extract it.

```rust
// Could add to FundingRate struct and parser
predicted_rate: Self::get_f64(data, "predictedValue"),
```

---

## Verified Correct

### ✅ All Endpoint Paths
Every endpoint path in `endpoints.rs` matches official documentation.

### ✅ Authentication Algorithm
- Signature: `HMAC-SHA256(timestamp + method + endpoint + body)` + Base64 ✓
- Passphrase: HMAC-SHA256 + Base64 ✓
- Headers: All correct (KC-API-KEY, KC-API-SIGN, etc.) ✓
- KC-API-KEY-VERSION: "2" ✓

### ✅ Base URLs
- Spot: `https://api.kucoin.com` ✓
- Futures: `https://api-futures.kucoin.com` ✓
- Testnet URLs: Correct ✓

### ✅ Order Parsing
- Order status logic using `isActive`, `cancelExist`, `dealSize` ✓
- Field mappings (`id`/`orderId`, `size`, `dealSize`, etc.) ✓
- Create order response parsing ✓

### ✅ Balance Parsing
- Spot: `currency`, `available`, `holds` ✓
- Futures: `currency`, `availableBalance`, `frozenFunds` ✓

### ✅ Position Parsing
- `currentQty` for quantity and side determination ✓
- All field mappings ✓

### ✅ Kline Parsing (Spot)
- Array structure: `[time, open, close, high, low, volume, turnover]` ✓
- Time conversion: seconds × 1000 ✓
- Reverse order: newest-first to oldest-first ✓

---

## Action Items

### Priority 1 (Must Fix)
- [ ] Fix bid/ask swap in `parse_ticker()` - **CRITICAL**
- [ ] Fix USDT futures symbol format (`XBTUSDTM` not `XBTUSDM`)

### Priority 2 (Should Fix)
- [ ] Add separate `map_futures_granularity()` function
- [ ] Fix orderbook timestamp field (support both `ts` and `time`)
- [ ] Fix orderbook sequence parsing (support integer)

### Priority 3 (Nice to Have)
- [ ] Extract `predictedValue` from funding rate
- [ ] Add rate limit tracking infrastructure
- [ ] Document FuturesSetLeverage behavior (risk limit, cancels orders)

---

## Research Documents

Detailed research is available in:

1. [endpoints.md](./endpoints.md) - Complete endpoint verification
2. [authentication.md](./authentication.md) - Auth algorithm details
3. [response_formats.md](./response_formats.md) - JSON response structures
4. [symbols.md](./symbols.md) - Symbol formatting rules
5. [rate_limits.md](./rate_limits.md) - Rate limit handling

---

## Conclusion

The KuCoin connector is **80% correct** but has critical bugs that will cause:
1. Wrong bid/ask prices in all ticker data
2. Failed requests for USDT-margined futures

These must be fixed before using as a reference implementation for other exchanges.
