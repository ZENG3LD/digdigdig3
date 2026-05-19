# dYdX V4 Authentication & Rate Limits

Sources: https://docs.dydx.xyz/ (primary)

---

## 1. Authentication Architecture Overview

dYdX V4 is a **Cosmos SDK sovereign blockchain**. There is no traditional API key system like centralized exchanges. Authentication works at the **cryptographic transaction signing** level.

### Core Principle

Every state-mutating action (place order, cancel order, transfer) is a **signed Cosmos transaction** broadcast to a validator node. The signature is verified on-chain by the protocol.

---

## 2. Key Derivation: Mnemonic → dYdX Address

### Standard Flow

```
1. User's Ethereum wallet signs a fixed message:
   { "action": "dYdX Chain Onboarding" }
        ↓
2. Signature is used as entropy by v4-client library
        ↓
3. Library deterministically derives:
   - mnemonic (24-word BIP39)
   - private key (SECP256K1)
   - public key
   - dYdX Chain address (dydx1...)
        ↓
4. Private key signs all Cosmos transactions
```

This means the dYdX Chain address is **derived from** the user's Ethereum wallet signature, creating a deterministic mapping between Ethereum identity and dYdX Chain identity.

### Key Format

- Curve: **SECP256K1** (same as Ethereum, Bitcoin)
- Address format: Bech32 with prefix `dydx` (e.g., `dydx1abc...def`)
- Standard Cosmos HD derivation path: `m/44'/118'/0'/0/0`

### API Key Convention (per Gunbot docs)

Some third-party integrations treat dYdX credentials as:
- **API Key**: dYdX Chain address
- **API Secret**: 24-word mnemonic phrase

This is a convention for API key fields in third-party tools; the protocol itself only uses the signed transaction.

---

## 3. Permissioned Keys (Authenticators)

dYdX V4 implements an **authenticator system** — a Cosmos SDK extension that allows an account to add **custom logic** for verifying transactions. This enables delegating trading without exposing the main wallet key.

### Authenticator Types

| Authenticator | Function |
|---|---|
| `SignatureVerification` | Validates transactions via a designated sub-key |
| `MessageFilter` | Restricts which Cosmos message types the key can sign |
| `SubaccountFilter` | Limits which subaccounts the key can operate on |
| `ClobPairIdFilter` | Limits which trading pairs the key can trade |
| `AnyOf` | Composite: succeeds if ANY sub-authenticator validates |
| `AllOf` | Composite: requires ALL sub-authenticators to validate |

### Capabilities Enabled by Permissioned Keys

| Use Case | Implementation |
|---|---|
| Multiple trading keys per account | Multiple `SignatureVerification` authenticators |
| Separate trading key from withdrawal key | `MessageFilter` blocking transfer messages on trading key |
| Whitelist specific trading pairs | `ClobPairIdFilter` on trading key |
| Per-subaccount access control | `SubaccountFilter` on specific sub-keys |
| Read-only API agent | `MessageFilter` allowing only read operations (none needed — reads are via Indexer with no auth) |

### How to Use Permissioned Keys

1. Generate a sub-key (SECP256K1 keypair)
2. Add `SignatureVerification` authenticator to your account with the sub-key's public key
3. Optionally compose with `MessageFilter`, `SubaccountFilter`, `ClobPairIdFilter` using `AllOf`
4. Sign transactions with the sub-key
5. The protocol verifies against the registered authenticator

This allows **hot wallet / bot key** to trade without access to the main wallet that holds withdrawal rights.

---

## 4. What Requires Authentication vs What Does Not

| Operation | Auth Required? | Method |
|---|---|---|
| Read market data (orderbook, candles, trades) | No | Indexer REST/WS — public |
| Read account data (positions, orders, fills) | No | Indexer REST — public (any address can be queried) |
| Place order | **Yes** | Signed `MsgPlaceOrder` Cosmos tx → gRPC |
| Cancel order | **Yes** | Signed `MsgCancelOrder` Cosmos tx → gRPC |
| Batch cancel | **Yes** | Signed `MsgBatchCancel` Cosmos tx → gRPC |
| Transfer between subaccounts | **Yes** | Signed `MsgCreateTransfer` Cosmos tx → gRPC |
| Deposit to subaccount | **Yes** | Signed `MsgDepositToSubaccount` Cosmos tx → gRPC |
| Withdraw from subaccount | **Yes** | Signed `MsgWithdrawFromSubaccount` Cosmos tx → gRPC |
| Compliance screen address | No | Indexer REST — public |

---

## 5. Network Endpoints

### Mainnet

| Service | Protocol | URL |
|---|---|---|
| Indexer REST | HTTPS | `https://indexer.dydx.trade/v4` |
| Indexer WebSocket | WSS | `wss://indexer.dydx.trade/v4/ws` |
| Node gRPC (OEGS) | gRPC+TLS | `grpc://oegs.dydx.trade:443` |
| Node gRPC (Polkachu) | gRPC+TLS | `https://dydx-dao-grpc-1.polkachu.com:443` |
| Node gRPC (KingNodes) | gRPC+TLS | `https://dydx-ops-grpc.kingnodes.com:443` |
| Node REST (Polkachu) | HTTPS | `https://dydx-dao-api.polkachu.com:443` |
| Node REST (KingNodes) | HTTPS | `https://dydx-ops-rest.kingnodes.com:443` |

### Testnet

| Service | Protocol | URL |
|---|---|---|
| Indexer REST | HTTPS | `https://indexer.v4testnet.dydx.exchange` |
| Indexer WebSocket | WSS | `wss://indexer.v4testnet.dydx.exchange/v4/ws` |
| Node gRPC (OEGS) | gRPC+TLS | `oegs-testnet.dydx.exchange:443` |
| Node gRPC (Lavender Five) | gRPC+TLS | `testnet-dydx.lavenderfive.com:443` |
| Node REST (Enigma) | HTTPS | `https://dydx-lcd-testnet.enigma-validator.com` |
| Node REST (Lavender Five) | HTTPS | `https://testnet-dydx-api.lavenderfive.com` |

---

## 6. Rate Limits

### 6.1 Indexer HTTP Rate Limits

| Resource | Limit | Measured By |
|---|---|---|
| HTTP Requests | **100 requests per 10 seconds** | Per IP address |

### 6.2 Indexer WebSocket Channel Limits (per connection)

| Channel Type | Max Subscriptions per Connection |
|---|---|
| `v4_accounts` | 256 |
| `v4_parent_accounts` | 256 |
| `v4_candles` | 32 |
| `v4_markets` | 32 |
| `v4_orderbook` | 32 |
| `v4_trades` | 32 |

### 6.3 Validator / On-Chain Rate Limits (Block-Level)

These limits apply per account address:

| Order Type | Limit |
|---|---|
| Stateful (long-term) orders | 2 per 1 block |
| Stateful (long-term) orders | 20 per 100 blocks |
| Short-term orders + cancellations (combined) | 4,000 per 5 blocks |

These limits are:
- Applied in **AND** fashion (all must be satisfied simultaneously)
- Applied per account (subaccounts share limits with their parent address)
- Both successful and **failed** attempts count toward limits
- Subject to change via governance; queryable at `GET /dydxprotocol/clob/block_rate`

### 6.4 Equity Tier Limits (Open Stateful Orders)

Maximum number of **simultaneously open** stateful (long-term + conditional) orders per subaccount, based on total net collateral (TNC):

| Net Collateral | Max Open Stateful Orders |
|---|---|
| < $20 | 0 |
| $20 – $100 | 4 |
| $100 – $1,000 | 8 |
| $1,000 – $10,000 | 10 |
| $10,000 – $100,000 | 100 |
| >= $100,000 | 200 |

**Important notes:**
- **Short-term orders (IOC, FOK, Market) are EXEMPT** from equity tier limits
- Only `stateful_order_equity_tiers` is currently in effect; `short_term_order_equity_tiers` is no longer enforced
- Limits are per subaccount, subject to governance
- Queryable via: `GET /dydxprotocol/clob/equity_tier`

---

## 7. Transaction Signing Summary

### Signing a Cosmos Transaction (Rust implementation notes)

```
1. Build the Cosmos Tx:
   - MsgPlaceOrder (or other message)
   - AuthInfo: fee, gas limit (0 for trading — no gas)
   - SignerInfo: public key + sequence number

2. Sign with SECP256K1 private key:
   - Hash: SHA-256(SignDoc bytes)
   - Sign: SECP256K1 deterministic signature

3. Encode as protobuf TxRaw

4. Broadcast via gRPC: cosmos.tx.v1beta1.Service/BroadcastTx
```

### Account Sequence Management

Each Cosmos account has a **sequence number** that increments with every transaction. Must be managed carefully:
- Fetch current sequence: `GET /cosmos/auth/v1beta1/accounts/{address}`
- Include correct sequence in each transaction's `AuthInfo`
- Stale sequence → `SEQUENCE_MISMATCH` error

### Account Number

- Static per account (assigned at account creation)
- Fetch once and cache: `GET /cosmos/auth/v1beta1/accounts/{address}`

---

## 8. No API Key Whitelist System

Unlike CEXes (Binance, Bybit, etc.), dYdX V4 has **no server-side API key registration**. Security is entirely cryptographic:

| CEX Pattern | dYdX V4 Equivalent |
|---|---|
| Create API key in UI | Generate a SECP256K1 keypair |
| API key whitelist IPs | No IP restriction at protocol level |
| API key permissions (read/write/withdraw) | Permissioned Keys (Authenticators) with MessageFilter |
| Revoke API key | Remove authenticator from account (on-chain tx) |
| API key + secret header in requests | Signed Cosmos transaction |

---

## 9. Full-Node gRPC Streaming (Advanced)

For lower-latency order updates, a full node can stream real-time data:

- **Endpoint**: `grpc://{full_node_url}:9090` (default gRPC port)
- **Service**: `dydxprotocol.clob.QueryService/StreamOrderbookUpdates`
- Provides: active orderbook updates, place order updates, cancel order updates, optimistic fills
- Alternative to Indexer WebSocket for tighter integration

---

## Sources

- [dYdX Documentation Home](https://docs.dydx.xyz/)
- [Connecting to dYdX — Endpoints](https://docs.dydx.xyz/interaction/endpoints)
- [Authenticators / Permissioned Keys](https://docs.dydx.xyz/concepts/trading/authenticators)
- [Equity Tier Limits](https://docs.dydx.xyz/concepts/trading/limits/equity-tier-limits)
- [Rate Limits](https://docs.dydx.xyz/concepts/trading/limits/rate-limits)
- [Onboarding Guide](https://docs.dydx.xyz/interaction/integration/integration-onboarding)
- [Permissioned Keys (old docs)](https://docs.dydx.exchange/api_integration-guides/how_to_permissioned_keys)
- [Gunbot dYdX V4 API Configuration](https://www.gunbot.com/support/guides/exchange-configuration/creating-api-keys/dydx-v4-api-configuration/)
- [Full Node gRPC Streaming](https://docs.dydx.exchange/infrastructure_providers-validators/full_node_streaming)
