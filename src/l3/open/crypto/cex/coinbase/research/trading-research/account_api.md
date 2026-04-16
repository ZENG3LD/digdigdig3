# Coinbase Advanced Trade API — Account & Portfolio API Reference

Base URL: `https://api.coinbase.com/api/v3/brokerage/`

---

## 1. ACCOUNT TYPES

Coinbase Advanced Trade has **one account per currency per platform type**. There are no sub-accounts in the traditional sense; instead, multiple **portfolios** exist, and each portfolio has its own set of currency accounts.

### Account Type Enum (`type` field)
| Value | Description |
|---|---|
| `ACCOUNT_TYPE_CRYPTO` | Cryptocurrency holding account |
| `ACCOUNT_TYPE_FIAT` | Fiat currency account (USD, EUR, etc.) |
| `ACCOUNT_TYPE_VAULT` | Coinbase Vault account |
| `ACCOUNT_TYPE_PERP_FUTURES` | Perpetual futures margin account |

### Platform Enum (`platform` field)
| Value | Description |
|---|---|
| `ACCOUNT_PLATFORM_CONSUMER` | Spot trading (standard Coinbase Advanced) |
| `ACCOUNT_PLATFORM_CFM_CONSUMER` | US Derivatives (regulated futures) |
| `ACCOUNT_PLATFORM_INTX` | International Exchange (perpetuals) |

### Trading Scope
- **Spot**: Available to all users globally
- **Perpetuals (INTX)**: International Exchange perpetuals — available to eligible non-US users
- **US Derivatives (CFM)**: CFTC-regulated futures — US users only, requires separate enrollment
- No leverage for standard spot accounts
- Perpetuals use USDC as margin currency

---

## 2. ACCOUNTS ENDPOINTS

### 2.1 List Accounts

```
GET /api/v3/brokerage/accounts
```

**Auth Required**: Yes (Bearer JWT)
**Permission**: `view`

#### Query Parameters

| Parameter | Type | Default | Description |
|---|---|---|---|
| `limit` | integer | 49 | Results per page (max: 250) |
| `cursor` | string | — | Pagination cursor for next page |
| `retail_portfolio_id` | string | — | (Deprecated) Filter by portfolio UUID |

#### Response

```json
{
  "accounts": [
    {
      "uuid": "8bfc20d7-f7c6-4422-bf07-8243ca4169fe",
      "name": "BTC Wallet",
      "currency": "BTC",
      "available_balance": {
        "value": "1.23",
        "currency": "BTC"
      },
      "hold": {
        "value": "0.10",
        "currency": "BTC"
      },
      "default": false,
      "active": true,
      "created_at": "2021-05-31T09:59:59.000Z",
      "updated_at": "2021-05-31T09:59:59.000Z",
      "deleted_at": null,
      "type": "ACCOUNT_TYPE_CRYPTO",
      "ready": true,
      "retail_portfolio_id": "b87a2d3f-8a1e-49b3-a4ea-402d8c389aca",
      "platform": "ACCOUNT_PLATFORM_CONSUMER"
    }
  ],
  "has_next": false,
  "cursor": "",
  "size": 1
}
```

#### Key Fields

| Field | Type | Description |
|---|---|---|
| `uuid` | string | Unique account identifier (use for account-specific calls) |
| `name` | string | Display name (e.g. "BTC Wallet") |
| `currency` | string | ISO currency code (e.g. "BTC", "USD", "ETH") |
| `available_balance.value` | string | Spendable/tradeable balance as decimal string |
| `available_balance.currency` | string | Currency of the balance |
| `hold.value` | string | Amount locked in open orders or pending transfers |
| `hold.currency` | string | Currency of the hold |
| `default` | boolean | Whether this is the user's default account |
| `active` | boolean | Account is usable |
| `ready` | boolean | Account is ready for trading |
| `type` | AccountType enum | Account category (CRYPTO, FIAT, VAULT, PERP_FUTURES) |
| `platform` | AccountPlatform enum | CONSUMER / CFM_CONSUMER / INTX |
| `retail_portfolio_id` | string | Parent portfolio UUID |

---

### 2.2 Get Account

```
GET /api/v3/brokerage/accounts/{account_uuid}
```

Returns a single account object (same schema as list).

#### Response

```json
{
  "account": {
    "uuid": "8bfc20d7-f7c6-4422-bf07-8243ca4169fe",
    "name": "BTC Wallet",
    "currency": "BTC",
    "available_balance": {
      "value": "1.23",
      "currency": "BTC"
    },
    "hold": {
      "value": "0.10",
      "currency": "BTC"
    },
    "default": false,
    "active": true,
    "created_at": "2021-05-31T09:59:59.000Z",
    "updated_at": "2021-05-31T09:59:59.000Z",
    "deleted_at": null,
    "type": "ACCOUNT_TYPE_CRYPTO",
    "ready": true,
    "retail_portfolio_id": "b87a2d3f-8a1e-49b3-a4ea-402d8c389aca",
    "platform": "ACCOUNT_PLATFORM_CONSUMER"
  }
}
```

---

## 3. PORTFOLIO ENDPOINTS

Portfolios are the top-level containers. Each portfolio has multiple accounts (one per currency).

### 3.1 List Portfolios

```
GET /api/v3/brokerage/portfolios
```

#### Query Parameters

| Parameter | Type | Description |
|---|---|---|
| `portfolio_type` | enum | `UNDEFINED`, `DEFAULT`, `CONSUMER`, `INTX` |

#### Response

```json
{
  "portfolios": [
    {
      "name": "Default",
      "uuid": "b87a2d3f-8a1e-49b3-a4ea-402d8c389aca",
      "type": "DEFAULT",
      "deleted": false
    }
  ]
}
```

### 3.2 Get Portfolio Breakdown

```
GET /api/v3/brokerage/portfolios/{portfolio_uuid}
```

Returns summary of all holdings in a portfolio.

### 3.3 Move Portfolio Funds

```
POST /api/v3/brokerage/portfolios/move_funds
```

Transfer funds between portfolios (e.g. move USDC from spot portfolio to INTX perpetuals portfolio).

```json
{
  "funds": {
    "value": "100.00",
    "currency": "USDC"
  },
  "source_portfolio_uuid": "source-portfolio-uuid",
  "target_portfolio_uuid": "target-portfolio-uuid"
}
```

---

## 4. PERPETUALS POSITIONS (INTX)

Perpetuals are available on the International Exchange (INTX) for eligible non-US users.
Requires USDC margin deposited into the INTX portfolio.

### 4.1 Get Perpetuals Portfolio Summary

```
GET /api/v3/brokerage/intx/portfolio/{portfolio_uuid}
```

Returns aggregate portfolio-level P&L and margin metrics.

```json
{
  "portfolio_summary": {
    "unrealized_pnl": {
      "value": "25.50",
      "currency": "USDC"
    },
    "notional_value": {
      "value": "5000.00",
      "currency": "USDC"
    },
    "open_position_notional": {
      "value": "5000.00",
      "currency": "USDC"
    },
    "portfolio_im_notional": {
      "value": "250.00",
      "currency": "USDC"
    },
    "portfolio_mm_notional": {
      "value": "100.00",
      "currency": "USDC"
    },
    "liquidation_percentage": "0.02",
    "subscription_id": "...",
    "portfolio_uuid": "portfolio-uuid",
    "portfolio_name": "INTX Portfolio",
    "portfolio_type": "INTX",
    "total_balance": {
      "value": "1000.00",
      "currency": "USDC"
    },
    "available_margin": {
      "value": "750.00",
      "currency": "USDC"
    },
    "buying_power": {
      "value": "7500.00",
      "currency": "USDC"
    }
  }
}
```

### 4.2 List Perpetuals Positions

```
GET /api/v3/brokerage/intx/positions/{portfolio_uuid}
```

**Auth Required**: Yes (Bearer JWT)
**Permission**: `view`

`portfolio_uuid` must be an INTX-type portfolio UUID.

#### Response

```json
{
  "positions": [
    {
      "product_id": "BTC-PERP-INTX",
      "symbol": "BTC-PERP-INTX",
      "product_uuid": "product-uuid",
      "portfolio_uuid": "portfolio-uuid",
      "vwap": {
        "value": "50000.00",
        "currency": "USDC"
      },
      "entry_vwap": {
        "value": "49500.00",
        "currency": "USDC"
      },
      "net_size": "0.1",
      "buy_order_size": "0",
      "sell_order_size": "0",
      "im_contribution": {
        "value": "250.00",
        "currency": "USDC"
      },
      "unrealized_pnl": {
        "value": "50.00",
        "currency": "USDC"
      },
      "mark_price": {
        "value": "50500.00",
        "currency": "USDC"
      },
      "liquidation_price": {
        "value": "45000.00",
        "currency": "USDC"
      },
      "margin_type": "CROSS",
      "leverage": "10",
      "long_unrealized_pnl": {
        "value": "50.00",
        "currency": "USDC"
      },
      "short_unrealized_pnl": {
        "value": "0",
        "currency": "USDC"
      },
      "aggregated_pnl": {
        "value": "50.00",
        "currency": "USDC"
      },
      "position_side": "LONG",
      "im_notional": {
        "value": "250.00",
        "currency": "USDC"
      },
      "mm_notional": {
        "value": "100.00",
        "currency": "USDC"
      }
    }
  ],
  "summary": {
    "aggregated_pnl": {
      "value": "50.00",
      "currency": "USDC"
    }
  }
}
```

#### Position Fields

| Field | Type | Description |
|---|---|---|
| `product_id` | string | Symbol (e.g. "BTC-PERP-INTX") |
| `net_size` | string | Position size; positive = long, negative = short |
| `position_side` | enum | `LONG`, `SHORT`, `POSITION_SIDE_UNKNOWN` |
| `margin_type` | enum | `CROSS` or `ISOLATED` |
| `leverage` | string | Current leverage multiplier |
| `unrealized_pnl.value` | string | Unrealized P&L in USDC |
| `mark_price.value` | string | Current mark price |
| `liquidation_price.value` | string | Price at which position gets liquidated |
| `entry_vwap.value` | string | Average entry price (VWAP) |
| `im_notional.value` | string | Initial margin requirement |
| `mm_notional.value` | string | Maintenance margin requirement |

### 4.3 Get Perpetuals Position

```
GET /api/v3/brokerage/intx/positions/{portfolio_uuid}/{symbol}
```

Returns single position for a specific perpetual symbol.

### 4.4 Set Perpetuals Margin Type

```
POST /api/v3/brokerage/intx/order_book/set_margin_type
```

Switch between CROSS and ISOLATED margin for a specific product.

---

## 5. RISK & FEE METRICS

### 5.1 Get Transaction Summary (Fee Tier + Volume)

```
GET /api/v3/brokerage/transaction_summary
```

**Auth Required**: Yes (Bearer JWT)

Returns the user's current fee tier, maker/taker rates, and 30-day volume metrics.

#### Query Parameters

| Parameter | Type | Description |
|---|---|---|
| `product_type` | enum | `SPOT`, `FUTURE`, `UNKNOWN_PRODUCT_TYPE` |
| `contract_expiry_type` | enum | `EXPIRING` or `PERPETUAL` (futures only) |
| `product_venue` | enum | `CBE`, `FCM`, `INTX`, `UNKNOWN_VENUE_TYPE` |

#### Response

```json
{
  "total_fees": 25.0,
  "fee_tier": {
    "pricing_tier": "<$10k",
    "taker_fee_rate": "0.0060",
    "maker_fee_rate": "0.0040",
    "aop_from": "0",
    "aop_to": "10000",
    "volume_types_and_range": [
      {
        "volume_types": [
          "VOLUME_TYPE_SPOT",
          "VOLUME_TYPE_US_DERIVATIVES"
        ],
        "vol_from": "0",
        "vol_to": "10000"
      }
    ]
  },
  "margin_rate": 0.5,
  "goods_and_services_tax": {
    "rate": "0.10",
    "type": "INCLUSIVE"
  },
  "advanced_trade_only_volume": 1000.0,
  "advanced_trade_only_fees": 25.0,
  "coinbase_pro_volume": 0.0,
  "coinbase_pro_fees": 0.0,
  "total_balance": "1000.00",
  "volume_breakdown": [
    {
      "volume_type": "VOLUME_TYPE_SPOT",
      "volume": 1000.0
    }
  ],
  "has_cost_plus_commission": false
}
```

**Standard Coinbase Advanced fee tiers (2024):**
- Level 0 (< $1k): Taker 0.60%, Maker 0.40%
- Level 1 ($1k–$10k): Taker 0.40%, Maker 0.25%
- Level 2 ($10k–$50k): Taker 0.25%, Maker 0.15%
- Level 3 ($50k–$100k): Taker 0.20%, Maker 0.10%
- Level 4 ($100k–$1M): Taker 0.18%, Maker 0.08%
- (Higher tiers approach 0.05%/0.00%)

---

## 6. TRANSFERS & PAYMENT METHODS

### 6.1 List Payment Methods

```
GET /api/v3/brokerage/payment_methods
```

Returns linked bank accounts, cards, and other funding sources.

### 6.2 Internal Portfolio Transfer

Use the `move_funds` endpoint (section 3.3) to move assets between portfolios (spot ↔ INTX perpetuals).

### 6.3 Withdrawals / Deposits

The Advanced Trade API does not expose direct deposit/withdrawal endpoints for external transfers. These are handled via Coinbase's retail wallet API or the separate Exchange API. For programmatic transfers, the recommended approach is:
- Use Coinbase retail wallet API (`/v2/accounts/{id}/transactions`) for on-chain sends
- Use Coinbase Exchange API for institutional deposits/withdrawals

---

## 7. BALANCES — PRACTICAL NOTES FOR V5 TRAIT IMPLEMENTATION

For implementing `get_balance()`:

1. Call `GET /api/v3/brokerage/accounts` to get all currency wallets
2. Each account has `currency`, `available_balance.value`, and `hold.value`
3. To find spot BTC balance: filter for `currency == "BTC"` and `platform == "ACCOUNT_PLATFORM_CONSUMER"`
4. For perpetuals margin (USDC): filter for `currency == "USDC"` and `platform == "ACCOUNT_PLATFORM_INTX"`
5. Paginate if `has_next == true` (unlikely unless user has many obscure currency accounts)

For implementing `get_positions()`:
1. First get portfolios via `GET /api/v3/brokerage/portfolios?portfolio_type=INTX` to get `portfolio_uuid`
2. Then call `GET /api/v3/brokerage/intx/positions/{portfolio_uuid}`
3. `net_size > 0` = long position, `net_size < 0` = short position

---

## Sources

- [List Accounts Reference](https://docs.cdp.coinbase.com/api-reference/advanced-trade-api/rest-api/accounts/list-accounts)
- [Get Account Reference](https://docs.cdp.coinbase.com/api-reference/advanced-trade-api/rest-api/accounts/get-account)
- [List Perpetuals Positions](https://docs.cdp.coinbase.com/api-reference/advanced-trade-api/rest-api/perpetuals/list-perpetuals-positions)
- [Get Perpetuals Position](https://docs.cdp.coinbase.com/api-reference/advanced-trade-api/rest-api/perpetuals/get-perpetuals-position)
- [Get Transaction Summary](https://docs.cdp.coinbase.com/api-reference/advanced-trade-api/rest-api/fees/get-transaction-summary)
- [Advanced Trade Perpetual Futures](https://docs.cdp.coinbase.com/advanced-trade-api/docs/perpetuals)
