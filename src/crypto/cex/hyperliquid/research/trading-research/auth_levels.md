# HyperLiquid Authentication and Rate Limits Specification

Sources:
- https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/signing
- https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/nonces-and-api-wallets
- https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/rate-limits-and-user-limits
- https://docs.chainstack.com/docs/hyperliquid-authentication-guide

---

## Authentication Method

HyperLiquid does NOT use traditional API key + secret pairs. Authentication is performed exclusively via **Ethereum-compatible wallet signatures** (ECDSA over secp256k1). The private key of either the master wallet or an approved agent wallet is used to sign each request.

There are no static API keys or bearer tokens. Every exchange action requires a fresh cryptographic signature.

---

## Two Signing Schemes

HyperLiquid uses two distinct signing approaches depending on the action type:

| Scheme | Python SDK Method | Chain ID | Domain | Operations |
|--------|-------------------|----------|--------|------------|
| L1 Action Signing | `sign_l1_action` | 1337 | "Exchange" | All trading: order, cancel, modify, updateLeverage, updateIsolatedMargin, vaultTransfer, twapOrder, etc. |
| User-Signed Action | `sign_user_signed_action` | 0x66eee (421614) | "HyperliquidSignTransaction" | Admin: approveAgent, usdSend, spotSend, withdraw3, usdClassTransfer |

The documentation strongly recommends using an existing SDK rather than implementing signing manually, due to debugging difficulty (an incorrect signature returns a wrong recovered address with no diagnostic detail).

---

## L1 Action Signing (sign_l1_action)

Used for all trading actions submitted to `/exchange` (order, cancel, modify, leverage, margin, etc.).

### Process

1. Serialize the action payload using **MessagePack (msgpack)** binary format.
   - Field order matters for correct hash computation.
   - Address fields must be lowercased before signing.
   - No trailing zeros on numeric values.
2. Append the nonce (uint64, big-endian) and vault address bytes (32 bytes, zeroed if no vault) to the serialized action.
3. Hash the concatenated bytes with **keccak256**.
4. Construct a "phantom agent" object:
   ```json
   {
     "source": "a",
     "connectionId": "<keccak256_hash_bytes>"
   }
   ```
5. Sign the phantom agent object using **EIP-712** with:
   - Chain ID: `1337`
   - Domain name: `"Exchange"`
   - Type: `{ "Agent": [{ "name": "source", "type": "string" }, { "name": "connectionId", "type": "bytes32" }] }`
6. The resulting signature `(r, s, v)` is placed in the `signature` field of the request.

### Signature Format

```json
{
  "r": "0x<32_bytes_hex>",
  "s": "0x<32_bytes_hex>",
  "v": 27
}
```

`v` is either 27 or 28 (recovery ID).

---

## User-Signed Action Signing (sign_user_signed_action)

Used for administrative/financial actions: `approveAgent`, `usdSend`, `spotSend`, `withdraw3`, `usdClassTransfer`.

### Process

1. Construct the action payload as a **JSON object** (direct structure, no msgpack).
2. Sign using **EIP-712** directly (no phantom agent wrapper):
   - Chain ID: `0x66eee` (421614 decimal) — NOT Arbitrum's 42161
   - Domain name: `"HyperliquidSignTransaction"`
3. The payload includes:
   ```json
   {
     "hyperliquidChain": "Mainnet",
     "signatureChainId": "0xa4b1",
     ...action_fields...
   }
   ```
   `signatureChainId` is the chain used for signing in hex (Arbitrum = `"0xa4b1"` for mainnet).
   On testnet: `"hyperliquidChain": "Testnet"`.

---

## Nonce Management

Nonces prevent replay attacks. Key rules:

- **Format:** Current Unix timestamp in **milliseconds** (not seconds).
- **Storage:** The 100 highest nonces are tracked per signer address.
- **Validity:** Every new transaction nonce must be larger than the smallest nonce in the stored set.
- **Time window:** Nonce must fall within `(T - 2 days, T + 1 day)` where T is the block timestamp.
- **Per-signer, not per-account:** Nonces are tracked by the signing key, not the master account. Each trading process or session should use a separate signing key to avoid nonce contention.

---

## API Wallets (Agent Wallets)

Agent wallets allow delegation of signing authority without exposing the master private key.

### Registration

**Action type:** `"approveAgent"` on `/exchange` (signed by master account using `sign_user_signed_action`).

```json
{
  "action": {
    "type": "approveAgent",
    "hyperliquidChain": "Mainnet",
    "signatureChainId": "0xa4b1",
    "agentAddress": "0x<agent_wallet_address>",
    "agentName": "MyTradingBot",
    "nonce": 1713148990947
  },
  "nonce": 1713148990947,
  "signature": { ... }
}
```

### Agent Wallet Capabilities

- Agent wallets can sign all L1 trading actions (order, cancel, modify, leverage, etc.) on behalf of the master account.
- Agent wallets can act on behalf of any sub-account of the master account (using `vaultAddress`).
- Agent wallets CANNOT initiate withdrawals (withdraw3) — that requires the master account.
- Agent wallets are signing-only: querying account data requires the actual account address, not the agent address.

### Agent Wallet Scope / Permissions

There is no documented granular permission scoping (no read-only, no specific-asset restrictions). An approved agent wallet has full trading authority over the master account. The only restriction is that agent wallets cannot sign user-signed actions (withdrawals, transfers).

### Agent Wallet Risks

- Nonce state for agent wallets is pruned if: the wallet is deregistered, expires, or the registering account loses funding.
- After pruning, previously signed actions with those nonces can theoretically be replayed.
- Recommendation: generate fresh agent wallet keys for each deployment; do not reuse addresses.

---

## Permission Levels (Summary)

| Actor | Can Trade | Can Withdraw | Can Query | Notes |
|-------|-----------|--------------|-----------|-------|
| Master wallet | Yes | Yes | Yes (own address) | Full access |
| Agent wallet | Yes (signs L1 actions) | No | No (must use master address for queries) | Trading delegation only |
| Sub-account | N/A (no key) | N/A | Yes (sub-account address) | Signed by master via vaultAddress |
| Vault | N/A (no key) | N/A | Yes (vault address) | Signed by master via vaultAddress |

---

## Rate Limits

### IP-Based Limits (per IP address)

**REST Endpoint Weight Budget:** 1200 weight per minute.

#### Exchange endpoint (POST /exchange) weight:
| Request type | Weight |
|-------------|--------|
| Single action (batch_length = 1) | 1 |
| Batched action (batch_length = n) | `1 + floor(n / 40)` |

#### Info endpoint (POST /info) weight:

| Request type | Weight |
|-------------|--------|
| `l2Book`, `allMids`, `clearinghouseState`, `orderStatus`, `spotClearinghouseState`, `exchangeStatus` | 2 |
| Most other info requests (openOrders, userFills, candleSnapshot, etc.) | 20 |
| `userRole` | 60 |
| `recentTrades`, `historicalOrders`, `userFills`, `userFillsByTime`, `fundingHistory`, `userFunding`, `nonUserFundingUpdates`, `twapHistory`, `userTwapSliceFills`, `userTwapSliceFillsByTime`, `delegatorHistory`, `delegatorRewards`, `validatorStats` | 20 base + 1 per 20 items returned |
| `candleSnapshot` | 20 base + 1 per 60 candles returned |

#### Explorer API weight:
| Request type | Weight |
|-------------|--------|
| Standard explorer request | 40 |
| `blockList` | 40 + 1 per block |

### WebSocket Limits (per IP address)

| Limit | Value |
|-------|-------|
| Max simultaneous connections | 10 |
| Max new connections per minute | 30 |
| Max subscriptions per IP | 1000 |
| Max unique users in user-specific subscriptions per IP | 10 |
| Max messages sent to Hyperliquid per minute | 2000 |
| Max simultaneous inflight POST messages | 100 |

### EVM JSON-RPC Limits

- 100 requests per minute to `rpc.hyperliquid.xyz/evm`

---

### Address-Based Limits (per user address)

This is a separate limit system that applies on top of IP limits.

| Parameter | Value |
|-----------|-------|
| Initial request buffer | 10,000 requests |
| Accrual rate | 1 request per 1 USDC traded cumulatively |
| Throttled rate (when buffer exhausted) | 1 request per 10 seconds |
| Cancel request multiplier | `min(limit + 100,000, limit * 2)` — cancels have a higher allowance |
| Max open orders (base) | 1,000 |
| Max open orders (volume-based) | +1 per 5,000,000 USDC cumulative volume |
| Max open orders (hard cap) | 5,000 |

**Batched requests:** Treated as 1 request for IP-based rate limiting, but as N requests for address-based rate limiting (where N = number of orders in the batch).

**expiresAfter penalty:** If a request reaches the server after its `expiresAfter` timestamp, it incurs a 5x rate limit penalty.

---

## Environment Details

| Environment | REST Base URL | WebSocket URL |
|-------------|--------------|---------------|
| Mainnet | `https://api.hyperliquid.xyz` | `wss://api.hyperliquid.xyz/ws` |
| Testnet | `https://api.hyperliquid-testnet.xyz` | `wss://api.hyperliquid-testnet.xyz/ws` |

---

## Implementation Notes for Rust/V5 Connector

1. No API key header needed — every request is signed with the private key.
2. The signing flow requires msgpack serialization for L1 actions — the `rmp-serde` or `rmpv` crate is needed.
3. EIP-712 encoding requires keccak256 hashing — `keccak256` from `tiny-keccak` or `ethereum-types`.
4. For ECDSA signing over secp256k1 — `k256` or `ethers` crate.
5. Nonce = `SystemTime::now().duration_since(UNIX_EPOCH).as_millis()` as u64.
6. `vaultAddress` field is omitted (not null) when trading on own account.
7. Address fields in the signed payload must be lowercase hex strings.
8. Field ordering in msgpack serialization must exactly match the SDK reference implementation.

---

## Sources

- [Signing | Hyperliquid Docs](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/signing)
- [Nonces and API Wallets | Hyperliquid Docs](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/nonces-and-api-wallets)
- [Rate Limits and User Limits | Hyperliquid Docs](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/rate-limits-and-user-limits)
- [Hyperliquid Authentication Guide | Chainstack](https://docs.chainstack.com/docs/hyperliquid-authentication-guide)
- [Turnkey x Hyperliquid: EIP-712 Signing](https://www.turnkey.com/blog/hyperliquid-secure-eip-712-signing)
