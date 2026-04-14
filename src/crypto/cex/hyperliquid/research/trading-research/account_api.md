# HyperLiquid Account API Specification

Sources:
- https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint
- https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/exchange-endpoint

## Overview

Account information is queried via:
- **Mainnet:** `POST https://api.hyperliquid.xyz/info`
- **Testnet:** `POST https://api.hyperliquid-testnet.xyz/info`
- **Content-Type:** `application/json`

These are unauthenticated read-only queries (no signature required). The `user` field must be the actual account address — using an agent wallet address returns empty results.

---

## Account Information

### Get Account Type / Role

```json
{ "type": "userRole", "user": "0x..." }
```

Response:
```json
{ "role": "user" }
```

Possible roles: `"user"`, `"agent"`, `"vault"`, `"subAccount"`, `"missing"`

Rate limit weight: 60 per request (highest weight of any info call).

### Get Clearinghouse State (Positions + Balances)

Perpetuals account state:
```json
{ "type": "clearinghouseState", "user": "0x...", "dex": "" }
```

Response structure:
```json
{
  "assetPositions": [
    {
      "position": {
        "coin": "ETH",
        "entryPx": "1800.00",
        "leverage": { "type": "cross", "value": 5 },
        "liquidationPx": null,
        "marginUsed": "200.00",
        "positionValue": "1000.00",
        "unrealizedPnl": "50.00",
        "returnOnEquity": "0.25",
        "szi": "0.5",
        "cumFunding": { "allTime": "...", "sinceOpen": "...", "sinceChange": "..." }
      },
      "type": "oneWay"
    }
  ],
  "crossMarginSummary": {
    "accountValue": "10000.00",
    "totalMarginUsed": "200.00",
    "totalNtlPos": "1000.00"
  },
  "marginSummary": {
    "accountValue": "10000.00",
    "totalMarginUsed": "200.00",
    "totalNtlPos": "1000.00"
  },
  "withdrawable": "9800.00",
  "time": 1713148990947
}
```

Rate limit weight: 2.

### Get Spot Account State

```json
{ "type": "spotClearinghouseState", "user": "0x..." }
```

Returns spot token balances per asset, including available, total, and unrealized P&L for spot positions.

Rate limit weight: 2.

### Get Portfolio / P&L History

```json
{ "type": "portfolio", "user": "0x..." }
```

Response includes time-bucketed account value and P&L snapshots:
```json
[
  ["day",    { "pnl": "...", "vlm": "...", "accountValueHistory": [...] }],
  ["week",   { ... }],
  ["month",  { ... }],
  ["allTime",{ ... }],
  ["perpDay",{ ... }]
]
```

### Get User Rate Limit Status

```json
{ "type": "userRateLimit", "user": "0x..." }
```

Response:
```json
{
  "cumVlm": "2854574.123",
  "nRequestsUsed": 2890,
  "nRequestsCap": 2864574
}
```

- `cumVlm`: cumulative trading volume in USDC since address inception
- `nRequestsUsed`: requests consumed
- `nRequestsCap`: current cap (= `10000 + floor(cumVlm / 1)`)

---

## Fee Schedule

### Get User Fees

```json
{ "type": "userFees", "user": "0x..." }
```

Response:
```json
{
  "dailyUserVlm": [
    { "coin": "ETH", "crossVlm": "...", "isolatedVlm": "..." }
  ],
  "feeSchedule": {
    "taker": "0.00035",
    "maker": "-0.0001",
    "referralDiscount": "0.04"
  },
  "userCrossRate": "0.000315",
  "userAddRate": "-0.0001",
  "activeStakingDiscount": {
    "discount": "0.1",
    "stakedAmount": "100.0"
  }
}
```

### Get Max Builder Fee Approval

```json
{ "type": "maxBuilderFee", "user": "0x...", "builder": "0x..." }
```

Returns the maximum builder fee rate approved by the user for a specific builder address.

### Get Approved Builders

```json
{ "type": "approvedBuilders", "user": "0x..." }
```

Response:
```json
["0x476fa87b4d3818f437f38f1263bee508d7672d82"]
```

---

## Balance and Wallet

### Get USDC Balance

Balance is included in `clearinghouseState` response under `withdrawable` (available to withdraw) and `marginSummary.accountValue` (total).

### Get Spot Balances

```json
{ "type": "spotClearinghouseState", "user": "0x..." }
```

### Deposit

Deposits come from bridging USDC from Arbitrum via the HyperEVM bridge. No direct API action for deposits — they are initiated on-chain (L1). Once bridged, funds appear in the user's spot USDC balance.

### Withdrawal (Bridge Out)

**Action type:** `"withdraw3"` on `/exchange` — initiates a withdrawal to Arbitrum bridge.

- Approximate processing time: ~5 minutes
- Fee: $1 USDC
- This is a user-signed action (uses `sign_user_signed_action` scheme, not `sign_l1_action`)

```json
{
  "action": {
    "type": "withdraw3",
    "hyperliquidChain": "Mainnet",
    "signatureChainId": "0xa4b1",
    "destination": "0x...",
    "amount": "100.0",
    "time": 1713148990947
  },
  "nonce": 1713148990947,
  "signature": { ... }
}
```

Note: `signatureChainId` is the Arbitrum chain ID (0xa4b1 = 42161) in hex.

### Internal USD Transfer (No Bridge)

**Action type:** `"usdSend"` — send USDC to another address within the Hyperliquid L1 (does not touch EVM bridge).

```json
{
  "action": {
    "type": "usdSend",
    "hyperliquidChain": "Mainnet",
    "signatureChainId": "0xa4b1",
    "destination": "0x...",
    "amount": "100.0",
    "time": 1713148990947
  },
  "nonce": 1713148990947,
  "signature": { ... }
}
```

### Spot Asset Transfer (No Bridge)

**Action type:** `"spotSend"` — send spot tokens to another address on L1.

```json
{
  "action": {
    "type": "spotSend",
    "hyperliquidChain": "Mainnet",
    "signatureChainId": "0xa4b1",
    "destination": "0x...",
    "token": "USDC:0x...",
    "amount": "100.0",
    "time": 1713148990947
  },
  "nonce": 1713148990947,
  "signature": { ... }
}
```

### Transfer Between Spot and Perp Accounts

**Action type:** `"usdClassTransfer"` — move USDC between the user's spot and perp margin accounts.

```json
{
  "action": {
    "type": "usdClassTransfer",
    "hyperliquidChain": "Mainnet",
    "signatureChainId": "0xa4b1",
    "amount": "100.0",
    "toPerp": true
  },
  "nonce": 1713148990947,
  "signature": { ... }
}
```

`toPerp: true` = spot → perp; `toPerp: false` = perp → spot.

### Generalized Token Transfer

**Action type:** `"sendAsset"` — generalized transfers between perp DEXs, spot accounts, users, and sub-accounts.

---

## Sub-Accounts

### List Sub-Accounts

```json
{ "type": "subAccounts", "user": "0x..." }
```

Response:
```json
[
  {
    "name": "Test",
    "subAccountUser": "0x...",
    "master": "0x...",
    "clearinghouseState": { ... },
    "spotState": { ... }
  }
]
```

### Trading on Sub-Account

Sub-accounts do not have private keys. All actions on a sub-account are signed by the master account with `vaultAddress` set to the sub-account address:

```json
{
  "action": { ... },
  "nonce": ...,
  "signature": { ... },
  "vaultAddress": "0x<sub_account_address>"
}
```

---

## Vault Management

### Get Vault Details

```json
{ "type": "vaultDetails", "vaultAddress": "0x...", "user": "0x..." }
```

Response:
```json
{
  "name": "Test Vault",
  "leader": "0x...",
  "apr": 0.363,
  "followers": [
    { "user": "0x...", "vaultEquity": "742500.08" }
  ],
  "maxWithdrawable": 742557.680863,
  "pnlHistory": [...],
  "portfolio": [...]
}
```

### Get User Vault Positions

```json
{ "type": "userVaultEquities", "user": "0x..." }
```

Response:
```json
[{ "vaultAddress": "0x...", "equity": "742500.082809" }]
```

### Vault Deposit / Withdrawal

**Action type:** `"vaultTransfer"` on `/exchange`

```json
{
  "action": {
    "type": "vaultTransfer",
    "vaultAddress": "0x...",
    "isDeposit": true,
    "usd": 1000
  },
  "nonce": ...,
  "signature": { ... }
}
```

`isDeposit: true` = deposit; `isDeposit: false` = withdraw.

Withdrawal lock-up: 4 days from most recent deposit. Withdrawals that would breach the vault's max withdrawable limit are rejected.

---

## Staking / Delegation

### Get Delegations

```json
{ "type": "delegations", "user": "0x..." }
```

Response:
```json
[{ "validator": "0x...", "amount": "12060.165", "lockedUntilTimestamp": 1735466781353 }]
```

### Get Delegation Summary

```json
{ "type": "delegatorSummary", "user": "0x..." }
```

Response:
```json
{
  "delegated": "12060.165",
  "undelegated": "0.0",
  "totalPendingWithdrawal": "0.0"
}
```

### Stake / Unstake

**Action type:** `"tokenDelegate"` on `/exchange` — delegate or undelegate native token to a validator.

Lock-up: 1 day.

**Deposit to Staking:**

**Action type:** `"cDeposit"` — transfer native token to staking.

**Withdraw from Staking:**

**Action type:** `"cWithdraw"` — transfer native token from staking into the user's spot account. Processing queue: 7 days.

---

## Borrow / Lend

### Get User Borrow/Lend State

```json
{ "type": "borrowLendUserState", "user": "0x..." }
```

Response:
```json
{
  "tokenToState": [...],
  "health": "healthy",
  "healthFactor": null
}
```

### Get Reserve State (Single Token)

```json
{ "type": "borrowLendReserveState", "token": 0 }
```

Response:
```json
{
  "borrowYearlyRate": "0.05",
  "supplyYearlyRate": "0.00082",
  "utilization": "0.0183",
  "ltv": "0.0"
}
```

### Get All Reserve States

```json
{ "type": "allBorrowLendReserveStates" }
```

---

## Referral

### Get Referral Info

```json
{ "type": "referral", "user": "0x..." }
```

Response:
```json
{
  "referredBy": { "referrer": "0x...", "rewardUsdc": "5.00" },
  "cumVlm": "149428030.00",
  "unclaimedRewards": "11.047361",
  "referrerState": { ... }
}
```

### Claim Rewards

**Action type:** `"claimRewards"` on `/exchange`.

---

## Sources

- [Info Endpoint | Hyperliquid Docs](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint)
- [Exchange Endpoint | Hyperliquid Docs](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/exchange-endpoint)
- [Perpetuals Info | Hyperliquid Docs](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint/perpetuals)
- [Vaults | Hyperliquid Docs](https://hyperliquid.gitbook.io/hyperliquid-docs/hypercore/vaults)
