# Jupiter Account & Balance API Specification

Source: https://dev.jup.ag/ (official Jupiter developer documentation)
Research date: 2026-03-11

---

## Overview

Jupiter is a non-custodial DEX aggregator on Solana. There are no "accounts" in the CEX sense — all funds remain in the user's own Solana wallet. Account information is either:

1. **Queried via Jupiter's Ultra API** — token holdings endpoint (convenience wrapper over on-chain data)
2. **Queried via Solana RPC** — direct `getTokenAccountsByOwner`, `getBalance`, etc.

There is no registration, no deposit/withdrawal to Jupiter. Jupiter never holds user funds.

---

## 1. Token Holdings (Ultra API)

### 1A. Get Token Holdings (Current — Recommended)

```
GET https://api.jup.ag/ultra/v1/holdings/{address}
```

**Required headers:**
- `x-api-key: <your-api-key>`

**Path parameters:**

| Parameter | Type | Description |
|---|---|---|
| `address` | string | Wallet public key (base58) |

**Response (200):**
```json
{
  "amount": "1000000000",
  "uiAmount": 1.0,
  "uiAmountString": "1.0",
  "tokens": {
    "<token-mint-address>": {
      "account": "<token-account-address>",
      "amount": "1000000",
      "uiAmount": 1.0,
      "uiAmountString": "1.0",
      "isFrozen": false,
      "isAssociatedTokenAccount": true,
      "decimals": 6,
      "programId": "<token-program-address>"
    }
  }
}
```

**Top-level fields (native SOL):**

| Field | Type | Description |
|---|---|---|
| `amount` | string | Raw lamport balance (1 SOL = 1,000,000,000 lamports) |
| `uiAmount` | number | Human-readable SOL balance |
| `uiAmountString` | string | Human-readable SOL balance as string |
| `tokens` | object | Map of mint address → token balance objects |

**Per-token fields (within `tokens`):**

| Field | Type | Description |
|---|---|---|
| `account` | string | Token account address (base58) |
| `amount` | string | Raw token balance (pre-decimals) |
| `uiAmount` | number | Human-readable token balance |
| `uiAmountString` | string | Human-readable token balance as string |
| `isFrozen` | boolean | Whether the token account is frozen |
| `isAssociatedTokenAccount` | boolean | Whether this is the canonical ATA for this mint |
| `decimals` | number | Token decimal places |
| `programId` | string | Associated token program address |

**Error response:**
```json
{
  "error": "Invalid address"
}
```

**Performance note:** For wallets with thousands of token accounts, response time may be slow due to the volume of on-chain data fetched.

---

### 1B. Get Native SOL Balance Only

```
GET https://api.jup.ag/ultra/v1/holdings/{address}/native
```

Returns only the native SOL balance for the wallet. Same `x-api-key` header required.

---

### 1C. Get Balances (DEPRECATED)

```
GET https://api.jup.ag/ultra/v1/balances/{address}
```

**Status:** DEPRECATED. Jupiter recommends using `/holdings/{address}` instead.

**Response (200) — for reference only:**
```json
{
  "SOL": {
    "amount": "0",
    "uiAmount": 0,
    "slot": 324307186,
    "isFrozen": false
  }
}
```

Fields: `amount` (string), `uiAmount` (number), `slot` (uint64), `isFrozen` (boolean).

---

## 2. Order Account Queries

Jupiter does not have a centralized order book. Open orders for limit/trigger and recurring strategies are stored as on-chain accounts, queried via Jupiter's own order query endpoints.

### 2A. Active and Historical Trigger (Limit) Orders

```
GET https://api.jup.ag/trigger/v1/getTriggerOrders
  ?user=<wallet-address>
  &orderStatus=active
```

See `trading_api.md` Section 2E for full parameter specification.

### 2B. Active and Historical Recurring (DCA) Orders

```
GET https://api.jup.ag/recurring/v1/getRecurringOrders
  ?user=<wallet-address>
  &orderStatus=active
  &recurringType=time
```

See `trading_api.md` Section 3C for full parameter specification.

---

## 3. On-Chain Balance Queries via Solana RPC

For production systems, token balances are also queryable directly from Solana chain data without using Jupiter's API. This is the standard approach used by all Solana wallets.

### 3A. Get SOL Balance

```
POST https://<rpc-endpoint>
Content-Type: application/json

{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "getBalance",
  "params": ["<wallet-public-key>"]
}
```

Response: `{ "result": { "value": <lamports> } }`

### 3B. Get All SPL Token Balances

```
POST https://<rpc-endpoint>
Content-Type: application/json

{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "getTokenAccountsByOwner",
  "params": [
    "<wallet-public-key>",
    { "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA" },
    { "encoding": "jsonParsed" }
  ]
}
```

### 3C. Get Specific Token Balance

```
POST https://<rpc-endpoint>
Content-Type: application/json

{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "getTokenAccountBalance",
  "params": ["<associated-token-account-address>"]
}
```

---

## 4. Perpetuals Position Data (On-Chain)

Jupiter Perps positions are stored as on-chain Solana accounts and are readable from the blockchain. No REST API exists for perps data as of early 2026.

**Position account contains:**
- Owner (trader wallet)
- Pool and custody references
- Position size in USD
- Collateral amount
- Side (Long/Short)
- Entry price
- PnL data
- TP/SL trigger parameters (if set)

Access via: Solana RPC `getAccountInfo` + Anchor IDL deserialization using the `julianfssen/jupiter-perps-anchor-idl-parsing` reference implementation.

---

## 5. What Does NOT Exist in Jupiter's API

The following concepts from CEX account APIs do NOT apply to Jupiter:

| CEX Concept | Jupiter Reality |
|---|---|
| Account registration / login | Not applicable; wallet-based |
| Deposit / withdrawal | Not applicable; user holds own keys |
| Account balance endpoint with all assets | Holdings endpoint covers SPL tokens + SOL |
| Trade history / fills | Not in public REST API; derivable from on-chain tx history |
| Account tier / KYC | Not applicable |
| API secret for order signing | Private key signs Solana transactions directly |

---

## Sources

- [Get Holdings Endpoint](https://dev.jup.ag/docs/ultra-api/get-holdings)
- [Get Balances Endpoint (deprecated)](https://dev.jup.ag/docs/ultra-api/get-balances)
- [Ultra Swap API Overview](https://dev.jup.ag/docs/ultra)
- [About Perps API](https://dev.jup.ag/docs/perps)
- [Solana RPC Documentation](https://docs.solana.com/api)
