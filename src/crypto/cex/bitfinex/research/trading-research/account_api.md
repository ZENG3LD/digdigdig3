# Bitfinex Account API — Wallets, Positions, Margin

Source: https://docs.bitfinex.com/reference

---

## 1. ACCOUNT TYPES

Bitfinex has three distinct wallet/account types:

| Wallet Type | API String | Description |
|---|---|---|
| Exchange (Spot) | `"exchange"` | Standard spot trading wallet |
| Margin | `"margin"` | Margin trading wallet, backed by funding |
| Funding | `"funding"` | Lending/borrowing wallet (also called "deposit") |

**Derivatives** use a separate USDt0 balance (`USTF0` currency) held in the exchange wallet. Must convert USDt to USDt0 via Transfer endpoint. Derivative symbols use format `tBTCF0:USTF0`, `tETHF0:USTF0`.

**Order routing by wallet type:**
- `EXCHANGE *` order types → exchange wallet
- `LIMIT`/`MARKET`/etc. (no prefix) → margin wallet
- Derivatives → exchange wallet with `USTF0` currency

---

## 2. BALANCE — Wallets Endpoint

```
POST https://api.bitfinex.com/v2/auth/r/wallets
```

No request body parameters. Returns all wallets for the authenticated account.

### Wallet Response Array Format

Each wallet is an array of 7 fields:

| Index | Field | Type | Description |
|---|---|---|---|
| [0] | WALLET_TYPE | string | `"exchange"`, `"margin"`, or `"funding"` |
| [1] | CURRENCY | string | Currency code (e.g. `"USD"`, `"BTC"`, `"UST"`, `"USTF0"`) |
| [2] | BALANCE | float | Total wallet balance |
| [3] | UNSETTLED_INTEREST | float | Unsettled interest amount |
| [4] | BALANCE_AVAILABLE | float or null | Available balance (not in orders/positions); null if not yet calculated |
| [5] | DESCRIPTION | string or null | Ledger entry description (only in update messages) |
| [6] | META | JSON or null | Reason for wallet update (only in update messages) |

**Note**: Fields [5] and [6] appear only in WebSocket wallet update (`wu`) messages, not in REST snapshots. `BALANCE_AVAILABLE` may be `null` until computed — use a `calc` request to get current value.

**Example response:**
```json
[
  ["exchange", "BTC", 0.5, 0, 0.3, null, null],
  ["exchange", "USD", 10000.0, 0, 8500.0, null, null],
  ["margin", "USD", 5000.0, -12.5, 3200.0, null, null],
  ["exchange", "USTF0", 2000.0, 0, 2000.0, null, null]
]
```

---

## 3. POSITIONS

### Retrieve Active Positions

```
POST https://api.bitfinex.com/v2/auth/r/positions
```

No request body parameters. Returns all currently open positions.

### Position Array Format (20 fields)

| Index | Field | Type | Description |
|---|---|---|---|
| [0] | SYMBOL | string | Trading pair (e.g. `"tBTCUSD"`, `"tBTCF0:USTF0"`) |
| [1] | STATUS | string | `"ACTIVE"` or `"CLOSED"` |
| [2] | AMOUNT | float | Position size; positive = long, negative = short |
| [3] | BASE_PRICE | float | Average entry price |
| [4] | MARGIN_FUNDING | float | Funding amount applied to position |
| [5] | MARGIN_FUNDING_TYPE | int | Funding type: `0` = daily rate, `1` = fixed term |
| [6] | PL | float | Unrealized Profit & Loss |
| [7] | PL_PERC | float | Unrealized P&L as percentage |
| [8] | PRICE_LIQ | float or null | Liquidation price (null until calculated) |
| [9] | LEVERAGE | float | Current leverage (relevant for derivatives) |
| [10] | FLAGS | int | Special position flags |
| [11] | POSITION_ID | int | Unique position identifier |
| [12] | MTS_CREATE | int | Position creation timestamp (ms) |
| [13] | MTS_UPDATE | int | Last update timestamp (ms) |
| [14] | — | null | Reserved placeholder |
| [15] | TYPE | int | `0` = margin position, `1` = derivatives position |
| [16] | — | null | Reserved placeholder |
| [17] | COLLATERAL | float | Applied collateral amount (derivatives) |
| [18] | COLLATERAL_MIN | float | Minimum required collateral (derivatives) |
| [19] | META | JSON or null | Trade/order IDs and creation metadata |

**Note**: `PRICE_LIQ` may be `null` for newly opened positions. Use the `calc` endpoint to force calculation.

### Positions History

```
POST https://api.bitfinex.com/v2/auth/r/positions/hist
```

Returns historical position snapshots.

### Close / Claim Position

To close a position on Bitfinex, submit an opposing order with:
- Flag `512` (`CLOSE`) — closes position at market
- Or flag `1024` (`REDUCE_ONLY`) combined with a limit order to close partially

For margin positions, you can also use the **Claim Position** endpoint:
```
POST https://api.bitfinex.com/v2/auth/w/position/claim
```

---

## 4. LEVERAGE AND MARGIN

### Set Leverage (Derivatives)

Leverage is set per-order via the `lev` parameter (1–100, default 10). There is no separate "set account leverage" endpoint — leverage is specified at order submission or update time.

For derivative orders:
```json
{
  "type": "EXCHANGE LIMIT",
  "symbol": "tBTCF0:USTF0",
  "amount": "0.01",
  "price": "50000",
  "lev": 20
}
```

Leverage can also be updated on an existing order via the update endpoint:
```json
{
  "id": 123456789,
  "lev": 10
}
```

### Margin Info

```
POST https://api.bitfinex.com/v2/auth/r/info/margin/{key}
```

**Key options:**

| Key | Returns |
|---|---|
| `base` | Account-level margin metrics |
| `tBTCUSD` (any symbol) | Symbol-specific margin data |
| `sym_all` | Margin data for all symbols |

**Base response format:**
```
["base", [USER_PL, USER_SWAPS, MARGIN_BALANCE, MARGIN_BALANCE_NET, MARGIN_MIN]]
```

| Field | Description |
|---|---|
| USER_PL | Unrealized P&L across all positions |
| USER_SWAPS | Financing charges (funding costs) |
| MARGIN_BALANCE | Total margin balance |
| MARGIN_BALANCE_NET | Net margin balance (after deducting minimum) |
| MARGIN_MIN | Minimum margin required |

**Symbol response format:**
```
["sym", "SYMBOL", [TRADABLE_BALANCE, GROSS_BALANCE, BUY, SELL, null, null, null, null]]
```

| Field | Description |
|---|---|
| TRADABLE_BALANCE | Available for new positions |
| GROSS_BALANCE | Total gross balance |
| BUY | Current buy exposure |
| SELL | Current sell exposure |

---

## 5. TRANSFERS BETWEEN WALLETS

```
POST https://api.bitfinex.com/v2/auth/w/transfer
```

**Required parameters:**

| Field | Type | Description |
|---|---|---|
| `from` | string | Source wallet: `"exchange"`, `"margin"`, `"funding"` |
| `to` | string | Destination wallet: `"exchange"`, `"margin"`, `"funding"` |
| `currency` | string | Asset to transfer (e.g. `"USD"`, `"BTC"`, `"UST"`) |
| `amount` | string | Amount to transfer |

**Optional parameters:**

| Field | Type | Description |
|---|---|---|
| `currency_to` | string | Target currency for conversions (e.g. `"USTF0"` for USDt→USDt0) |
| `email_dst` | string | Transfer to sub/master account by email |
| `user_id_dst` | int | Transfer to sub/master account by user ID |
| `tfaToken` | object | 2FA token (required when transferring to other accounts) |

**Common use case** — convert USDt for derivatives trading:
```json
{
  "from": "exchange",
  "to": "exchange",
  "currency": "UST",
  "currency_to": "USTF0",
  "amount": "1000"
}
```

**Response array:**
```
[MTS, "acc_tf", null, null, [WALLET_FROM, WALLET_TO, CURRENCY, AMOUNT], null, "SUCCESS", "text"]
```

---

## 6. TRADE HISTORY AND LEDGER

### Trades History

```
POST https://api.bitfinex.com/v2/auth/r/trades/hist
POST https://api.bitfinex.com/v2/auth/r/trades/{symbol}/hist
```

| Parameter | Type | Description |
|---|---|---|
| `start` | int | Start timestamp (ms) |
| `end` | int | End timestamp (ms) |
| `limit` | int | Max records |
| `sort` | int | `1` = oldest first, `-1` = newest first |

**Trade array format:**

| Index | Field | Type | Description |
|---|---|---|---|
| [0] | ID | int | Trade ID |
| [1] | PAIR | string | Trading pair |
| [2] | MTS_CREATE | int | Execution timestamp (ms) |
| [3] | ORDER_ID | int | Parent order ID |
| [4] | EXEC_AMOUNT | float | Executed size (positive=buy, negative=sell) |
| [5] | EXEC_PRICE | float | Execution price |
| [6] | ORDER_TYPE | string | Type of the parent order |
| [7] | ORDER_PRICE | float | Original order price |
| [8] | MAKER | int | `1`=maker, `0`=taker |
| [9] | FEE | float | Fee charged (negative = paid) |
| [10] | FEE_CURRENCY | string | Currency fee was paid in |

### Ledger History

```
POST https://api.bitfinex.com/v2/auth/r/ledgers/{currency}/hist
```

| Parameter | Type | Description |
|---|---|---|
| `start` | int | Start timestamp (ms) |
| `end` | int | End timestamp (ms) |
| `limit` | int | Max records (cap: 2500) |
| `category` | int | Ledger category filter |

Ledger categories: deposits, withdrawals, trades, funding, etc.

---

## Sources

- https://docs.bitfinex.com/reference/ws-auth-wallets
- https://docs.bitfinex.com/reference/ws-auth-positions
- https://docs.bitfinex.com/reference/rest-auth-info-margin
- https://docs.bitfinex.com/reference/rest-auth-transfer
- https://docs.bitfinex.com/docs/derivatives
- https://bitfinex.readthedocs.io/en/latest/restv2.html
