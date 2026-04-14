# dYdX V4 Account API — Subaccounts, Balances, Transfers, History

Sources: https://docs.dydx.xyz/ (primary)

---

## 1. Subaccount System

### Structure

Each main Cosmos address (wallet) on dYdX V4 has **128,001 subaccounts**, identified by a tuple `(address, subaccount_number)`.

| Subaccount Number | Type | Purpose |
|---|---|---|
| `0` | **Cross-margin** (parent) | Default trading account; positions cross-margined together |
| `1–127` | **Cross-margin** (additional) | Additional cross-margin subaccounts |
| `128–128,000` | **Isolated margin** (child) | One market per subaccount; isolated from cross positions |

### Key Properties

- Subaccounts are **automatically created** when funds are first deposited to a valid subaccount ID
- Only the **main account** can execute transactions on behalf of its subaccounts
- **No gas** is consumed for order placement on subaccounts
- Collateral token: currently **USDC** (Noble USDC)
- Each subaccount tracks `quoteBalance` (USDC balance) and open positions
- Balance changes occur on: transfer, deposit, withdrawal, position modify, funding payment, liquidation

### Cross vs Isolated Margin

| Feature | Cross Margin (0–127) | Isolated Margin (128–128,000) |
|---|---|---|
| Positions per subaccount | Multiple | One only |
| Margin sharing | Shared across positions | Isolated to single market |
| Funding collateral source | Shared USDC balance | Dedicated USDC per subaccount |
| Market type | `PERPETUAL_MARKET_TYPE_CROSS` | `PERPETUAL_MARKET_TYPE_ISOLATED` |
| Liquidation risk | Whole subaccount | Only this subaccount |

### UI Behavior for Isolated Margin

The UI automatically moves collateral from cross subaccount (subaccount 0) to an isolated subaccount (128+) when a user opens an isolated position.

---

## 2. Account Information Endpoints (Indexer — Read Only)

Base URL: `https://indexer.dydx.trade`

### Get All Subaccounts for Address

```
GET /v4/addresses/{address}
```

| Parameter | Type | Required | Description |
|---|---|---|---|
| `address` | string | path | Cosmos address |
| `limit` | integer | no | Max subaccounts returned |

**Response:**
```json
{
  "subaccounts": [
    {
      "address": "dydx1abc...",
      "subaccountNumber": 0,
      "equity": "10000.00",
      "freeCollateral": "8000.00",
      "marginEnabled": true,
      "updatedAtHeight": "1234567",
      "latestProcessedBlockHeight": "1234567",
      "openPerpetualPositions": { ... },
      "assetPositions": { ... }
    }
  ]
}
```

### Get Single Subaccount

```
GET /v4/addresses/{address}/subaccountNumber/{number}
```

| Parameter | Type | Required | Description |
|---|---|---|---|
| `address` | string | path | Cosmos address |
| `number` | integer | path | Subaccount index |

**Response fields:**

| Field | Type | Description |
|---|---|---|
| `address` | string | Cosmos address |
| `subaccountNumber` | integer | Subaccount index |
| `equity` | string | Total account value in USDC |
| `freeCollateral` | string | Available margin (equity - used margin) |
| `marginEnabled` | boolean | Whether margin trading is enabled |
| `openPerpetualPositions` | object | Map of market → position details |
| `assetPositions` | object | USDC balance and other asset positions |
| `updatedAtHeight` | string | Block height of last update |

### Get Parent Subaccount (aggregated view)

```
GET /v4/addresses/{address}/parentSubaccountNumber/{number}
```

Returns aggregate view across parent and all associated child subaccounts.

---

## 3. Asset Positions (USDC Balance)

```
GET /v4/assetPositions
```

| Parameter | Type | Required | Description |
|---|---|---|---|
| `address` | string | yes | Cosmos address |
| `subaccountNumber` | integer | yes | Subaccount index |

**Response (per asset position):**

```json
{
  "symbol": "USDC",
  "side": "LONG",
  "size": "5000.00",
  "assetId": "0",
  "subaccountNumber": 0
}
```

The `size` represents the USDC balance of the subaccount.

---

## 4. Deposits: Ethereum → dYdX

dYdX uses **Noble USDC** as collateral. Deposits from Ethereum flow via CCTP (Circle Cross-Chain Transfer Protocol):

### Deposit Flow

```
1. User initiates deposit from Ethereum wallet
        ↓
2. USDC burned on Ethereum source chain
        ↓
3. USDC minted on Noble blockchain
        ↓
4. IBC transfer: Noble → dYdX Chain (main account x/bank module)
        ↓
5. MsgDepositToSubaccount: x/bank → x/subaccounts (subaccount 0)
        ↓
6. Funds available for trading
```

### Timeline

- Noble: up to **30 minutes** for funds to appear
- dYdX frontend sweeps from Noble to dYdX automatically after detection

### MsgDepositToSubaccount (on-chain Cosmos message)

```protobuf
message MsgDepositToSubaccount {
    string sender = 1;          // Cosmos address (must equal recipient.owner for self-deposit)
    SubaccountId recipient = 2; // { owner: address, number: subaccount_number }
    uint32 asset_id = 3;        // Asset ID (0 = USDC)
    bytes quantums = 4;         // Amount in base units (quantums)
}
```

This moves funds from the Cosmos `x/bank` module to the `x/subaccounts` module.

---

## 5. Withdrawals: dYdX → Ethereum

### Withdrawal Flow

```
1. MsgWithdrawFromSubaccount: x/subaccounts → x/bank (main account)
        ↓
2. IBC transfer: dYdX Chain → Noble blockchain
        ↓
3. CCTP burn on Noble → mint on destination chain (Ethereum, etc.)
        ↓
4. USDC arrives on Ethereum
```

### MsgWithdrawFromSubaccount (on-chain Cosmos message)

```protobuf
message MsgWithdrawFromSubaccount {
    SubaccountId sender = 1;    // { owner: address, number: subaccount_number }
    string recipient = 2;       // Cosmos address (typically same as sender.owner)
    uint32 asset_id = 3;        // Asset ID (0 = USDC)
    bytes quantums = 4;         // Amount in base units
}
```

### Withdrawal Rate Limits

| Period | Limit |
|---|---|
| Hourly | `max(1% of TVL, $1,000,000)` |
| Daily | `max(10% of TVL, $10,000,000)` |

These apply to Noble USDC withdrawals and are adjustable via governance.

### Withdrawal Gating (Emergency)

Withdrawals are **temporarily blocked for 50 blocks** in these conditions:
1. Negative-collateralized subaccounts appear that cannot be liquidated/deleveraged
2. Chain outage lasting 5+ minutes occurred

---

## 6. Transfers Between Subaccounts

### MsgCreateTransfer (on-chain Cosmos message)

Transfer USDC between subaccounts within the same or different address.

```protobuf
message MsgCreateTransfer {
    Transfer transfer = 1;
}

message Transfer {
    SubaccountId sender    = 1;  // { owner, number }
    SubaccountId recipient = 2;  // { owner, number }
    uint32 asset_id        = 3;  // 0 = USDC
    uint64 quantums        = 4;  // Amount in base units
}
```

### Common Transfer Patterns

| Pattern | Sender | Recipient |
|---|---|---|
| Cross → Isolated | `{addr, 0}` | `{addr, 128}` |
| Isolated → Cross | `{addr, 128}` | `{addr, 0}` |
| Between accounts | `{addr1, N}` | `{addr2, M}` |

All transfers require signing from the `sender.owner` account.

---

## 7. Transfer History (Indexer — Read Only)

### Get Transfers for Subaccount

```
GET /v4/transfers
```

| Parameter | Type | Required | Description |
|---|---|---|---|
| `address` | string | yes | Cosmos address |
| `subaccountNumber` | integer | yes | Subaccount index |
| `limit` | integer | no | Max results |
| `createdBeforeOrAt` | ISO8601 | no | Pagination timestamp |
| `page` | integer | no | Page number |

### Get Transfers Between Two Subaccounts

```
GET /v4/transfers/between
```

| Parameter | Type | Required | Description |
|---|---|---|---|
| `sourceAddress` | string | yes | Source Cosmos address |
| `sourceSubaccountNumber` | integer | yes | Source subaccount |
| `recipientAddress` | string | yes | Recipient Cosmos address |
| `recipientSubaccountNumber` | integer | yes | Recipient subaccount |

### Get Transfers by Parent Subaccount

```
GET /v4/transfers/parentSubaccountNumber
```
Parameters: `address`, `parentSubaccountNumber`, `limit`

### Transfer Response Fields

```json
{
  "id": "transfer-uuid",
  "sender": {
    "address": "dydx1abc...",
    "subaccountNumber": 0
  },
  "recipient": {
    "address": "dydx1abc...",
    "subaccountNumber": 128
  },
  "size": "1000.00",
  "symbol": "USDC",
  "type": "TRANSFER_IN",
  "transactionHash": "0xabc...",
  "createdAt": "2024-01-01T00:00:00.000Z",
  "createdAtHeight": "1234567"
}
```

---

## 8. Trade Fills History

```
GET /v4/fills
```

| Parameter | Type | Required | Description |
|---|---|---|---|
| `address` | string | yes | Cosmos address |
| `subaccountNumber` | integer | yes | Subaccount index |
| `ticker` | string | no | Market filter |
| `limit` | integer | no | Max results |
| `createdBeforeOrAt` | ISO8601 | no | Pagination |

### Fill Response Fields

```json
{
  "id": "fill-uuid",
  "side": "BUY",
  "liquidity": "TAKER",
  "type": "LIMIT",
  "market": "BTC-USD",
  "orderId": "order-uuid",
  "price": "45000.00",
  "size": "0.01",
  "fee": "1.35",
  "createdAt": "2024-01-01T00:00:00.000Z",
  "createdAtHeight": "1234567",
  "subaccountNumber": 0
}
```

---

## 9. PnL History

```
GET /v4/historical-pnl
```
Parameters: `address`, `subaccountNumber`, `createdBeforeOrAt`, `page`

---

## 10. Rewards / Trading Rewards

### Block-Level Rewards

```
GET /v4/historicalBlockTradingRewards/{address}
```
Parameters: `address`, `limit`, `startingBeforeOrAt`

### Aggregated Rewards

```
GET /v4/historicalTradingRewardAggregations/{address}
```
Parameters: `address`, `period` (`DAILY`/`WEEKLY`/`MONTHLY`), `limit`, `startingBeforeOrAt`

---

## 11. Funding Payments

```
GET /v4/fundingPayments
```

| Parameter | Type | Required | Description |
|---|---|---|---|
| `address` | string | yes | Cosmos address |
| `subaccountNumber` | integer | yes | Subaccount index |
| `ticker` | string | no | Market filter |
| `limit` | integer | no | Max results |
| `afterOrAt` | ISO8601 | no | Start time filter |
| `page` | integer | no | Page number |

---

## 12. Compliance Screening

Before trading, addresses can be screened:

```
GET /v4/screen?address={address}
GET /v4/compliance/screen/{address}
```

Returns whether the address is restricted from using dYdX based on compliance rules.

---

## Sources

- [dYdX Documentation](https://docs.dydx.xyz/)
- [Indexer HTTP API](https://docs.dydx.xyz/indexer-client/http)
- [Accounts and Subaccounts](https://docs.dydx.xyz/concepts/trading/accounts)
- [Onboarding Guide](https://docs.dydx.xyz/interaction/integration/integration-onboarding)
- [Margin Documentation](https://docs.dydx.xyz/concepts/trading/margin)
- [Withdrawal Rate Limits](https://docs.dydx.exchange/api_integration-deposits_and_withdrawals/rate_limits_and_gating)
- [v4-chain sending/transfer.proto](https://github.com/dydxprotocol/v4-chain/blob/main/proto/dydxprotocol/sending/transfer.proto)
- [Isolated Markets and Isolated Margin Blog](https://www.dydx.xyz/blog/introducing-isolated-markets-and-isolated-margin)
