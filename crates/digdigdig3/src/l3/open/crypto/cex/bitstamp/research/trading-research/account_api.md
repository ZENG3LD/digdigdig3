# Bitstamp Account API — Full Specification

> **CRITICAL NOTE**: Bitstamp is SPOT ONLY. No positions, no margin, no leverage, no futures balances.
> All private endpoints use POST with `application/x-www-form-urlencoded`.

---

## 1. ACCOUNT TYPES

Bitstamp offers a single account type: **Spot Account**.

- Unified balance across all currency holdings
- No margin account
- No futures/derivatives account
- Sub-accounts exist (for institutional use) — limited API support

There is NO concept of:
- Position (long/short) — not applicable to spot
- Unrealized PnL — not applicable
- Margin ratio — not applicable
- Liquidation price — not applicable

---

## 2. BALANCE ENDPOINTS

### 2.1 Get All Account Balances
```
POST /api/v2/account_balances/
```
Returns balances for ALL currencies held.

**Parameters:** None (auth headers only)

**Response:** Array of balance objects:
```json
[
  {
    "currency": "btc",
    "total": "1.25000000",
    "available": "1.00000000",
    "reserved": "0.25000000"
  },
  {
    "currency": "usd",
    "total": "50000.00",
    "available": "45000.00",
    "reserved": "5000.00"
  },
  {
    "currency": "eth",
    "total": "10.00000000",
    "available": "10.00000000",
    "reserved": "0.00000000"
  }
]
```

**Field Reference:**
| Field | Type | Description |
|-------|------|-------------|
| `currency` | string | Lowercase currency code: `"btc"`, `"usd"`, `"eth"`, `"eur"` |
| `total` | string | Total balance (available + reserved) |
| `available` | string | Free to trade or withdraw |
| `reserved` | string | Locked in open orders |

### 2.2 Get Balance for Specific Currency
```
POST /api/v2/account_balances/{currency}/
```
**Parameters:** None (auth headers only)

**Response:** Single balance object:
```json
{
  "currency": "btc",
  "total": "1.25000000",
  "available": "1.00000000",
  "reserved": "0.25000000"
}
```

### 2.3 Legacy Balance Endpoint (still functional)
```
POST /api/v2/balance/
```
Returns a flat key-value structure for all currencies (older format):
```json
{
  "btc_available": "1.00000000",
  "btc_balance": "1.25000000",
  "btc_reserved": "0.25000000",
  "btcusd_fee": "0.16000",
  "usd_available": "45000.00",
  "usd_balance": "50000.00",
  "usd_reserved": "5000.00",
  "eth_available": "10.00000000",
  "eth_balance": "10.00000000",
  "eth_reserved": "0.00000000"
}
```
> Prefer the newer `/api/v2/account_balances/` endpoint — cleaner structure.

### 2.4 Balance for Specific Pair (legacy)
```
POST /api/v2/balance/{currency_pair}/
```
Returns balance info relevant to the given trading pair plus fee:
```json
{
  "btc_available": "1.00000000",
  "btc_balance": "1.25000000",
  "btc_reserved": "0.25000000",
  "usd_available": "45000.00",
  "usd_balance": "50000.00",
  "usd_reserved": "5000.00",
  "fee": "0.16000"
}
```

---

## 3. POSITIONS — NOT APPLICABLE

Bitstamp is spot-only. The following traits return `UnsupportedOperation`:
- `get_positions()`
- `get_position(symbol)`
- `close_position()`
- `set_leverage()`
- `set_margin_mode()`

---

## 4. LEVERAGE / MARGIN — NOT APPLICABLE

- No margin trading available
- No leverage settings
- No cross/isolated margin modes
- No liquidation mechanics

---

## 5. TRANSFERS AND SUB-ACCOUNTS

### 5.1 Transfer to Main Account
```
POST /api/v2/transfer-to-main/
```
Transfer funds from sub-account to main account.

**Parameters:**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `amount` | string/decimal | Yes | Amount to transfer |
| `currency` | string | Yes | Currency code (e.g., `"btc"`, `"usd"`) |
| `subAccount` | string | Yes | Sub-account ID or username |

**Response:**
```json
{
  "status": "ok"
}
```

### 5.2 Transfer from Main Account
```
POST /api/v2/transfer-from-main/
```
Transfer funds from main account to sub-account.

**Parameters:**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `amount` | string/decimal | Yes | Amount to transfer |
| `currency` | string | Yes | Currency code |
| `subAccount` | string | Yes | Sub-account ID or username |

**Response:**
```json
{
  "status": "ok"
}
```

> Sub-account transfers are institutional features; most retail users will not have sub-accounts.

---

## 6. FEES ENDPOINTS

### 6.1 Get All Trading Fees
```
POST /api/v2/fees/trading/
```
Returns maker/taker fees for ALL trading pairs.

**Parameters:** None (auth headers only)

**Response:** Array of fee objects:
```json
[
  {
    "currency_pair": "btcusd",
    "market": "btcusd",
    "fees": {
      "maker": "0.15000",
      "taker": "0.16000"
    }
  },
  {
    "currency_pair": "ethusd",
    "market": "ethusd",
    "fees": {
      "maker": "0.15000",
      "taker": "0.16000"
    }
  }
]
```

**Field Reference:**
| Field | Type | Description |
|-------|------|-------------|
| `currency_pair` | string | Trading pair (deprecated field name) |
| `market` | string | Trading pair (current field name) |
| `fees.maker` | string | Maker fee as percentage string (`"0.15000"` = 0.15%) |
| `fees.taker` | string | Taker fee as percentage string |

### 6.2 Get Trading Fee for Specific Pair
```
POST /api/v2/fees/trading/{market_symbol}/
```
**Parameters:** None (auth headers only)

**Response:** Single fee object:
```json
{
  "currency_pair": "btcusd",
  "market": "btcusd",
  "fees": {
    "maker": "0.15000",
    "taker": "0.16000"
  }
}
```

### 6.3 Withdrawal Fees
No dedicated API endpoint for withdrawal fees — check fee schedule at `https://www.bitstamp.net/fee-schedule/`.

### 6.4 Fee Schedule (Reference)
Bitstamp uses a tiered fee structure based on 30-day USD-equivalent trading volume:

| 30d Volume | Maker | Taker |
|------------|-------|-------|
| 0 – $10k | 0.30% | 0.40% |
| $10k – $20k | 0.24% | 0.30% |
| $20k – $100k | 0.22% | 0.25% |
| $100k – $500k | 0.14% | 0.20% |
| $500k – $1M | 0.12% | 0.16% |
| $1M – $2M | 0.10% | 0.14% |
| $2M – $10M | 0.06% | 0.10% |
| $10M – $20M | 0.04% | 0.06% |
| > $20M | 0.00% | 0.03% |

---

## 7. DEPOSIT AND WITHDRAWAL ENDPOINTS

### 7.1 Get Deposit Addresses
```
POST /api/v2/bitcoin_deposit_address/
POST /api/v2/ltc_address/
POST /api/v2/eth_address/
POST /api/v2/bch_address/
POST /api/v2/xrp_address/
```

### 7.2 Withdrawals
```
POST /api/v2/bitcoin_withdrawal/
POST /api/v2/ltc_withdrawal/
POST /api/v2/eth_withdrawal/
POST /api/v2/bch_withdrawal/
POST /api/v2/xrp_withdrawal/
```

**Common withdrawal parameters:**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `amount` | string/decimal | Yes | Amount to withdraw |
| `address` | string | Yes | Destination address |
| `instant` | integer | No | `1` = instant withdrawal (BTC only) |

**XRP additional parameter:**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `destination_tag` | integer | No | XRP destination tag |

### 7.3 Withdrawal Requests History
```
POST /api/v2/withdrawal-requests/
```
**Parameters:**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `timedelta` | integer | No | Seconds back to query (default 86400 = 24h) |

---

## 8. TRADING PAIRS INFO

### 8.1 Get All Trading Pairs (Public)
```
GET /api/v2/trading-pairs-info/
```
**Response:** Array of trading pair objects:
```json
[
  {
    "name": "BTC/USD",
    "url_symbol": "btcusd",
    "base_decimals": 8,
    "counter_decimals": 2,
    "minimum_order": "10.0 USD",
    "trading": "Enabled",
    "description": "Bitcoin/U.S. dollar"
  }
]
```

**Field Reference:**
| Field | Type | Description |
|-------|------|-------------|
| `url_symbol` | string | Use this in endpoint paths (e.g., `btcusd`) |
| `base_decimals` | integer | Decimal places for base currency |
| `counter_decimals` | integer | Decimal places for quote currency |
| `minimum_order` | string | Minimum order value in counter currency |
| `trading` | string | `"Enabled"` or `"Disabled"` |

---

## 9. ERROR CODES (Account Endpoints)

| Code | Meaning |
|------|---------|
| `400.001` | Invalid request parameters |
| `400.002` | Rate limit exceeded |
| `400.003` | Trading market currently disabled |
| `400.004` | Insufficient balance |
| `400.005` | Amount too small (below minimum order) |

---

## 10. V5 TRAIT MAPPING SUMMARY

| V5 Trait Method | Bitstamp Endpoint | Notes |
|-----------------|-------------------|-------|
| `get_balance()` | `POST /api/v2/account_balances/` | Returns array; parse to HashMap |
| `get_balance_currency(c)` | `POST /api/v2/account_balances/{currency}/` | Single currency |
| `get_trading_fee(pair)` | `POST /api/v2/fees/trading/{market}/` | Returns maker+taker |
| `get_all_trading_fees()` | `POST /api/v2/fees/trading/` | Returns array |
| `get_positions()` | — | `UnsupportedOperation` (spot only) |
| `get_leverage()` | — | `UnsupportedOperation` |
| `get_trading_pairs()` | `GET /api/v2/trading-pairs-info/` | Public, no auth needed |
| `transfer_to_sub()` | `POST /api/v2/transfer-from-main/` | Institutional only |
| `transfer_from_sub()` | `POST /api/v2/transfer-to-main/` | Institutional only |
