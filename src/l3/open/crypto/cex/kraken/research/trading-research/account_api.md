# Kraken Account API — Balances, Positions, Margin, and Transfers

## Critical Architecture Note

Kraken has **two separate account systems** that do not share state:

| System | Description | Base URL |
|--------|-------------|----------|
| Kraken Spot | Traditional spot exchange + integrated margin trading | `https://api.kraken.com/0` |
| Kraken Futures | Derivatives/perpetuals platform | `https://futures.kraken.com/derivatives/api/v3` |

Transferring funds between them requires explicit wallet transfer endpoints.

---

## 1. ACCOUNT TYPES

### Spot Accounts
- **Single unified account** — all spot balances in one account
- No separate sub-accounts for spot (sub-account features exist but are institutional)
- Margin trading is **integrated into the spot account** — no separate margin account to create
- Special balance suffixes:
  - `.B` — yield-bearing product balances (e.g. `XXBT.B`)
  - `.F` — automatically earning Kraken Rewards balances
  - `.T` — tokenized asset balances

### Futures Accounts (multi-account structure)
Each futures symbol has its own **marginAccount** with its own balances and margin tracking:

| Account Type | Description |
|-------------|-------------|
| `cashAccount` | Multi-currency cash wallet for funding futures |
| `marginAccount` | Per-contract margin account (e.g. `fi_xbtusd`) |

---

## 2. BALANCE & ACCOUNT INFO

### Spot: Get Balance

**Endpoint:** `POST /0/private/Balance`

**Required Permission:** `Funds permissions — Query`

Returns all asset balances as string-formatted decimals.

#### Response

```json
{
  "error": [],
  "result": {
    "ZUSD": "16.4272",
    "ZEUR": "0.3880",
    "ZJPY": "0.45",
    "XXBT": "0.0000000072",
    "XETH": "2.5000000000",
    "XXRP": "0.00000000",
    "XLTC": "0.0000000100",
    "XXDG": "13997.00000000",
    "KFEE": "10368.39"
  }
}
```

**Currency Naming Convention:**
- Fiat currencies prefixed with `Z`: `ZUSD`, `ZEUR`, `ZGBP`, `ZJPY`, `ZCAD`
- Cryptocurrencies prefixed with `X`: `XXBT` (BTC), `XETH`, `XLTC`, `XXRP`
- Some newer assets use plain names: `SOL`, `DOT`, `ADA`

---

### Spot: Get Trade Balance (Margin Summary)

**Endpoint:** `POST /0/private/TradeBalance`

**Required Permission:** `Orders and trades — Query open orders & trades`

Optional `asset` parameter to denominate in a specific asset (default: ZUSD).

#### Response

```json
{
  "error": [],
  "result": {
    "eb": "2.8987347115",
    "tb": "1.1694303513",
    "m": "0.0000000000",
    "uv": "0",
    "n": "0.0000000000",
    "c": "0.0000000000",
    "v": "0.0000000000",
    "e": "1.1694303513",
    "mf": "1.1694303513",
    "ml": null
  }
}
```

#### Field Descriptions

| Field | Full Name | Description |
|-------|-----------|-------------|
| `eb` | Equivalent balance | Total balance of all assets converted to the base currency |
| `tb` | Trade balance | Balance of equity currencies only |
| `m` | Margin | Total margin amount of open positions |
| `n` | Net PnL | Unrealized net profit/loss of open margin positions |
| `c` | Cost | Cost basis of open margin positions |
| `v` | Value | Current floating valuation of open positions (midpoint bid/ask) |
| `e` | Equity | `tb + n` — effective equity |
| `mf` | Free margin | `e - m` — maximum margin available to open new positions |
| `ml` | Margin level | `(e / m) * 100` — null if no margin in use |
| `uv` | Unexecuted value | Value of pending orders |

---

### Spot: Extended Balance

**Endpoint:** `POST /0/private/ExtendedBalance`

Returns balances with hold amounts (funds reserved for open orders).

---

### Futures: Get Accounts / Get Wallets

**Endpoint:** `GET /derivatives/api/v3/accounts`

Returns all account structures: cash account + per-symbol margin accounts.

#### Response

```json
{
  "result": "success",
  "accounts": {
    "fi_xbtusd": {
      "auxiliary": {
        "usd": 0,
        "pv": 0.0,
        "pnl": 0.0,
        "af": 0.0,
        "funding": 0.0
      },
      "marginRequirements": {
        "im": 0.0,
        "mm": 0.0,
        "lt": 0.0,
        "tt": 0.0
      },
      "triggerEstimates": {
        "im": 0,
        "mm": 0,
        "lt": 0,
        "tt": 0
      },
      "balances": {
        "xbt": 0.0
      },
      "currency": "xbt",
      "type": "marginAccount"
    },
    "cash": {
      "balances": {
        "eur": 4567.7117591172,
        "gbp": 4002.4975584765,
        "bch": 39.3081761006,
        "usd": 5000.0,
        "xrp": 10055.1019587339,
        "eth": 2.6868286287,
        "usdt": 4999.3200924674,
        "usdc": 4999.8300057798,
        "ltc": 53.9199827456,
        "xbt": 0.1785169809
      },
      "type": "cashAccount"
    }
  }
}
```

#### Futures Account Field Descriptions

**marginAccount.auxiliary:**

| Field | Description |
|-------|-------------|
| `pv` | Portfolio value — total account value in account currency |
| `pnl` | Unrealized profit/loss of open positions |
| `af` | Available funds — funds available for new orders/positions |
| `funding` | Accumulated funding costs |
| `usd` | USD equivalent value |

**marginAccount.marginRequirements:**

| Field | Description |
|-------|-------------|
| `im` | Initial margin required for current positions |
| `mm` | Maintenance margin required |
| `lt` | Liquidation trigger — when equity hits this, liquidation begins |
| `tt` | Termination trigger — point at which position is force-closed |

**marginAccount.triggerEstimates:**
Same fields as `marginRequirements` but expressed as estimated price levels at which these thresholds would be triggered.

---

## 3. POSITIONS

### Spot: Open Margin Positions

**Endpoint:** `POST /0/private/OpenPositions`

**Required Permission:** `Orders and trades — Query open orders & trades`

Optional `txid` param (comma-delimited) to filter specific positions.
Optional `docalcs=true` to include current floating P&L calculations.

#### Response

```json
{
  "error": [],
  "result": {
    "TF5GVO-T7ZZ2-6NBKBI": {
      "ordertxid": "OLWNFG-LLH4R-D6SFFP",
      "posstatus": "open",
      "pair": "XETHZUSD",
      "time": 1616462409.7518,
      "type": "buy",
      "ordertype": "market",
      "cost": "1500.460",
      "fee": "3.601",
      "vol": "0.75000000",
      "vol_closed": "0.00000000",
      "margin": "300.091",
      "value": "1516.725",
      "net": "+16.265",
      "terms": "0.0200% per 4 hours",
      "rollovertm": 1616480409,
      "misc": "",
      "oflags": ""
    }
  }
}
```

#### Field Descriptions

| Field | Description |
|-------|-------------|
| `ordertxid` | Transaction ID of the order that opened this position |
| `posstatus` | `open` or `closing` |
| `pair` | Asset pair |
| `time` | Unix timestamp when position was opened |
| `type` | `buy` (long) or `sell` (short) |
| `ordertype` | Order type used to open |
| `cost` | Opening cost of position |
| `fee` | Opening fee paid |
| `vol` | Total position volume |
| `vol_closed` | Volume already closed |
| `margin` | Initial margin posted |
| `value` | Current value (if `docalcs=true`) |
| `net` | Unrealized PnL (if `docalcs=true`) |
| `terms` | Rollover fee terms |
| `rollovertm` | Unix timestamp of next rollover fee event |

### Closing Spot Margin Positions

- Use `ordertype=settle-position` in `AddOrder` to close a margin position
- Or place an opposite order and Kraken matches it against the open position

---

### Futures: Open Positions

**Endpoint:** `GET /derivatives/api/v3/openpositions`

#### Response

```json
{
  "result": "success",
  "openPositions": [
    {
      "side": "long",
      "symbol": "PI_XBTUSD",
      "price": 27500.0,
      "fillTime": "2024-01-15T09:00:00.000Z",
      "size": 1000,
      "unrealizedFunding": -0.0015
    }
  ],
  "serverTime": "2024-01-15T10:30:00.000Z"
}
```

| Field | Description |
|-------|-------------|
| `side` | `long` or `short` |
| `symbol` | Futures contract symbol |
| `price` | Average entry price |
| `fillTime` | Timestamp of position opening |
| `size` | Position size in contracts |
| `unrealizedFunding` | Accumulated unpaid funding cost/gain |

**WebSocket openPositions feed** provides richer data including:
- `pnl`, `entry_price`, `mark_price`, `index_price`
- `liquidation_threshold`, `effective_leverage`
- `return_on_equity`, `initial_margin`, `maintenance_margin`

---

## 4. LEVERAGE & MARGIN

### Spot Leverage

**No separate set_leverage endpoint exists for Spot.**

- Leverage is specified **per-order** via the `leverage` param in `AddOrder`
- Example: `leverage=5` means 5x leverage on that specific order
- Max leverage varies by asset pair (typically 2x-5x for major pairs)
- Borrow/repay is handled automatically by Kraken — no explicit borrow API

### Futures Leverage

**Separate leverage settings exist per symbol.**

**Get Leverage Setting:**

`GET /derivatives/api/v3/leveragepreferences`

**Set Leverage Setting:**

`PUT /derivatives/api/v3/leveragepreferences`

```json
{
  "symbol": "PI_XBTUSD",
  "maxLeverage": 25
}
```

Futures support up to 50x leverage depending on the contract. Setting leverage here affects the margin model for that symbol.

---

## 5. MARGIN TRADING (Spot)

Spot margin trading is **seamlessly integrated** — no separate account setup needed.

### How It Works
1. Add `leverage` param to any `AddOrder` call
2. Kraken automatically borrows the required funds
3. A margin position is created in `OpenPositions`
4. Rollover fees accrue every 4 hours (terms visible in position)
5. Close via `settle-position` order type or opposite order

### Margin Requirements
- `TradeBalance.mf` (free margin) shows available margin capacity
- `TradeBalance.ml` (margin level) shows current risk ratio
- `TradeBalance.m` (margin) shows total margin committed

---

## 6. TRANSFERS

### Spot to Futures Transfer

**Endpoint:** `POST /0/private/WalletTransfer`

Transfers from Kraken Spot wallet to Kraken Futures cash wallet.

**Required Permission:** `Funds permissions — Withdraw`

| Parameter | Type | Description |
|-----------|------|-------------|
| `nonce` | integer | Monotonically increasing nonce |
| `asset` | string | Asset to transfer (e.g. `XBT`, `ETH`, `USD`) |
| `from` | string | Source wallet (e.g. `Spot Wallet`) |
| `to` | string | Destination wallet (e.g. `Futures Wallet`) |
| `amount` | decimal | Amount to transfer |

### Futures to Spot Transfer

Must be initiated from the Futures side:

**Endpoint:** `POST /derivatives/api/v3/withdrawal`

Initiates withdrawal from Futures cash wallet back to Spot wallet.

### Staking-Related Endpoints

- `POST /0/private/Stake` — stake an asset
- `POST /0/private/Unstake` — unstake an asset
- `GET /0/private/Staking/Assets` — list stakeable assets

---

## 7. TRADING VOLUME & FEE TIER

**Endpoint:** `POST /0/private/TradeVolume`

**Required Permission:** `Orders and trades — Query closed orders & trades`

Returns 30-day trading volume and current fee schedule.

| Parameter | Type | Description |
|-----------|------|-------------|
| `pair` | string | Comma-delimited pairs to get fee info for |

#### Response

```json
{
  "error": [],
  "result": {
    "currency": "ZUSD",
    "volume": "2145.654",
    "fees": {
      "XXBTZUSD": {
        "fee": "0.2600",
        "minfee": "0.1000",
        "maxfee": "0.2600",
        "nextfee": "0.2400",
        "nextvolume": "50000.0000",
        "tiervolume": "0.0000"
      }
    },
    "fees_maker": {
      "XXBTZUSD": {
        "fee": "0.1600",
        "minfee": "0.0000",
        "maxfee": "0.1600",
        "nextfee": "0.1400",
        "nextvolume": "50000.0000",
        "tiervolume": "0.0000"
      }
    }
  }
}
```

---

## 8. LEDGER HISTORY

### Get Ledgers

**Endpoint:** `POST /0/private/Ledgers`

**Required Permission:** `Data - Query ledger entries`

Returns all account activity: trades, deposits, withdrawals, transfers, margin events.

| Parameter | Type | Description |
|-----------|------|-------------|
| `asset` | string | Filter by asset |
| `aclass` | string | Asset class filter |
| `type` | string | `trade`, `deposit`, `withdrawal`, `transfer`, `margin`, `rollover`, `credit`, `settled`, `staking`, `dividend`, `sale`, `nfts` |
| `start` | decimal | Start Unix timestamp |
| `end` | decimal | End Unix timestamp |
| `ofs` | integer | Offset for pagination |
| `without_count` | boolean | Skip total count for performance |

#### Ledger Entry Response

```json
{
  "ABCDE-FGHIJ-KLMNO": {
    "refid": "FGHIJ-KLMNO-PQRST",
    "time": 1616462409.7518,
    "type": "trade",
    "subtype": "",
    "aclass": "currency",
    "asset": "XXBT",
    "amount": "0.01500000",
    "fee": "0.00001500",
    "balance": "0.27500000"
  }
}
```

---

## 9. DEPOSITS & WITHDRAWALS

### Deposit Methods

`POST /0/private/DepositMethods` — list available deposit methods for an asset

`POST /0/private/DepositAddresses` — get deposit address

`POST /0/private/DepositStatus` — check deposit status

**Required Permission:** `Funds permissions — Deposit`

### Withdrawals

`POST /0/private/WithdrawInfo` — get withdrawal fee estimate

`POST /0/private/Withdraw` — initiate withdrawal

`POST /0/private/WithdrawStatus` — check withdrawal status

`POST /0/private/WithdrawCancel` — cancel pending withdrawal

**Required Permission:** `Funds permissions — Withdraw`

---

## 10. ENDPOINT REFERENCE SUMMARY

### Spot Private Endpoints (account-related)

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/0/private/Balance` | POST | All spot asset balances |
| `/0/private/ExtendedBalance` | POST | Balances including hold amounts |
| `/0/private/TradeBalance` | POST | Margin equity summary |
| `/0/private/OpenPositions` | POST | Open margin positions |
| `/0/private/TradesHistory` | POST | Executed trade history |
| `/0/private/Ledgers` | POST | Full account ledger |
| `/0/private/QueryLedgers` | POST | Query specific ledger entries |
| `/0/private/TradeVolume` | POST | 30-day volume and fee tier |
| `/0/private/WalletTransfer` | POST | Transfer Spot → Futures |

### Futures Private Endpoints (account-related)

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/derivatives/api/v3/accounts` | GET | All account balances and margin info |
| `/derivatives/api/v3/openpositions` | GET | Open futures positions |
| `/derivatives/api/v3/fills` | GET | Trade fill history |
| `/derivatives/api/v3/transfer` | POST | Internal Futures transfer |
| `/derivatives/api/v3/withdrawal` | POST | Transfer Futures → Spot |
| `/derivatives/api/v3/leveragepreferences` | GET/PUT | Get/set leverage per symbol |

---

## Sources

- [Get Account Balance | Kraken API Center](https://docs.kraken.com/api/docs/rest-api/get-account-balance/)
- [Get Trade Balance | Kraken API Center](https://docs.kraken.com/api/docs/rest-api/get-trade-balance/)
- [Get Open Positions | Kraken API Center](https://docs.kraken.com/api/docs/rest-api/get-open-positions/)
- [Get Wallets (Futures) | Kraken API Center](https://docs.kraken.com/api/docs/futures-api/trading/get-accounts/)
- [Account Information | Kraken API Center](https://docs.kraken.com/api/docs/futures-api/trading/account-information/)
- [Get Open Positions (Futures) | Kraken API Center](https://docs.kraken.com/api/docs/futures-api/trading/get-open-positions/)
- [Request Wallet Transfer | Kraken API Center](https://docs.kraken.com/api/docs/rest-api/wallet-transfer/)
- [Get Leverage Settings | Kraken API Center](https://docs.kraken.com/api/docs/futures-api/trading/get-leverage-setting/)
- [Margin Trading Terms | Kraken Support](https://support.kraken.com/hc/en-us/articles/205246667-Position)
- [Futures REST Python SDK Docs](https://python-kraken-sdk.readthedocs.io/en/v2.0.0/src/futures/rest.html)
