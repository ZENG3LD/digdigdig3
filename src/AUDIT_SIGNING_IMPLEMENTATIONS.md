# Signing & Authentication Audit — digdigdig3

**Date**: 2026-03-14
**Scope**: All `auth.rs` files under `src/` plus standalone signing modules (`eip712.rs`)
**Total auth files scanned**: 145+

---

## Summary

| Category | Count | Status |
|----------|-------|--------|
| Standard HMAC-SHA256 | 22 | OK |
| HMAC-SHA512 | 3 | OK |
| HMAC-SHA384 | 2 | OK |
| JWT (HS512) | 2 | OK |
| JWT (ES256 / ECDSA P-256) | 1 | FLAG — nonce not injected into header |
| EIP-712 (full, alloy) | 2 | OK — correctly implemented |
| EIP-712 (ethers crate) | 1 | FLAG — ethers not in Cargo.toml; Vertex dead service |
| StarkNet ECDSA | 1 | CRITICAL FLAG — hardcoded k=1 nonce |
| ZK-native ECgFp5 / Poseidon2 | 1 | NOT IMPLEMENTED (documented stub) |
| OAuth2 Bearer / API Key passthrough | 80+ | OK (no signing needed) |
| Custom flows (TOTP, SHA-256 checksum) | 5 | OK |
| No-auth (public DEX/data) | 15+ | OK |

---

## 1. Complete Connector Table

### Crypto CEX

| Connector | Method | Notes | Status |
|-----------|--------|-------|--------|
| `binance` | HMAC-SHA256 | Standard, timestamp+query | OK |
| `bingx` | HMAC-SHA256 | Standard | OK |
| `bitfinex` | HMAC-SHA384 | Nonce=microseconds, hex output | OK |
| `bitget` | HMAC-SHA256 | Standard | OK |
| `bithumb` | HMAC-SHA256 | Params sorted alphabetically | OK |
| `bitstamp` | HMAC-SHA256 | Multi-line string, UUID nonce, uppercase hex | OK |
| `bybit` | HMAC-SHA256 | Standard | OK |
| `coinbase` | ES256 JWT | ECDSA P-256, PEM key, 2-min expiry | FLAG (see §4) |
| `crypto_com` | HMAC-SHA256 | Standard | OK |
| `deribit` | HMAC-SHA256 | `client_signature` grant, `ts\nnonce\ndata` | OK |
| `gateio` | HMAC-SHA256 | Standard | OK |
| `gemini` | HMAC-SHA384 | Base64-encoded JSON payload in header | OK |
| `htx` | HMAC-SHA256 | Standard | OK |
| `hyperliquid` | EIP-712 (alloy) | Phantom agent + msgpack + keccak256 | OK (see §3) |
| `hyperliquid/eip712` | EIP-712 (alloy) | User-signed: withdraw/transfer | OK (see §3) |
| `kraken` | HMAC-SHA512 | SHA256→HMAC-SHA512, base64 secret | OK |
| `kucoin` | HMAC-SHA256 | + passphrase HMAC | OK |
| `mexc` | HMAC-SHA256 | Standard | OK |
| `okx` | HMAC-SHA256 | ISO8601 timestamp | OK |
| `phemex` | HMAC-SHA256 | Standard | OK |
| `upbit` | HMAC-SHA512 JWT | HS512 JWT with SHA512 query hash | OK |
| `vertex` | EIP-712 (ethers) | SERVICE DEAD — ethers not in Cargo.toml | FLAG (see §5) |

### Crypto DEX

| Connector | Method | Notes | Status |
|-----------|--------|-------|--------|
| `dydx` | No auth (Indexer) | Cosmos signing planned (`onchain-cosmos` feature) | OK |
| `gmx` | No auth / Bearer | Public price data; optional wallet integration | OK |
| `jupiter` | No auth | Public DEX data | OK |
| `lighter` | ECgFp5 + Poseidon2 | NOT IMPLEMENTED — documented stub | FLAG (see §6) |
| `paradex` | StarkNet ECDSA + JWT | k=1 hardcoded nonce in `sign_auth_request` | CRITICAL (see §2) |

### Crypto Swap

| Connector | Method | Notes | Status |
|-----------|--------|-------|--------|
| `raydium` | No auth | Public DEX REST data | OK |
| `uniswap` | API key header | x-api-key; no signing | OK |

### Prediction Markets

| Connector | Method | Notes | Status |
|-----------|--------|-------|--------|
| `polymarket` | HMAC-SHA256 | Base64-URL secret decoded, URL-safe output | OK |

### On-chain Analytics

| Connector | Method | Notes | Status |
|-----------|--------|-------|--------|
| `bitquery` | OAuth2 Bearer | Static `ory_at_...` token | OK |
| `etherscan` | API key in URL | No signing | OK |
| `whale_alert` | API key | No signing | OK |

### Stocks — India

| Connector | Method | Notes | Status |
|-----------|--------|-------|--------|
| `angel_one` | TOTP + JWT Bearer | 3-factor: code+PIN+TOTP | OK |
| `dhan` | API key | No signing | OK |
| `fyers` | SHA-256 checksum + Bearer | OAuth flow, `SHA256(app_id:app_secret)` | OK |
| `upstox` | OAuth2 Bearer | Standard flow | OK |
| `zerodha` | SHA-256 checksum + token | Custom `token key:token` scheme, daily expiry | OK |

### Stocks — Russia

| Connector | Method | Notes | Status |
|-----------|--------|-------|--------|
| `moex` | No auth / Basic | Public data, optional basic auth | OK |
| `tinkoff` | Bearer token | Simple `t.xxx` static token | OK |

### Stocks — Other Regions

| Connector | Method | Notes | Status |
|-----------|--------|-------|--------|
| `futu` (China) | Bearer/API key | Standard | OK |
| `jquants` (Japan) | Bearer token (exchanged) | OK |
| `krx` (Korea) | API key | No signing | OK |
| `alpaca` (US) | Static key + secret headers | No signing required | OK |
| `finnhub` (US) | API key in URL | No signing | OK |
| `polygon` (US) | API key | No signing | OK |
| `tiingo` (US) | Bearer token | No signing | OK |
| `twelvedata` (US) | API key | No signing | OK |

### Forex

| Connector | Method | Notes | Status |
|-----------|--------|-------|--------|
| `alphavantage` | API key in URL | No signing | OK |
| `dukascopy` | No auth | Public historical data | OK |
| `oanda` | Bearer token | No signing | OK |

### Aggregators

| Connector | Method | Notes | Status |
|-----------|--------|-------|--------|
| `cryptocompare` | API key header | No signing | OK |
| `defillama` | No auth | Public data | OK |
| `ib` | Session token | No signing | OK |
| `yahoo` | No auth | Public data | OK |

### Intelligence Feeds (80+ connectors)

The vast majority use one of:
- No auth (open government/public APIs: GDELT, ACLED, USGS, NASA EONET, etc.)
- Static API key in header or URL (FRED, WorldBank, NOAA, ADS-B Exchange, etc.)
- OAuth2 client credentials (Sentinel Hub)
- Bearer token (Bitquery, Cloudflare Radar, etc.)
- Basic auth (OpenSky, some RIPE NCC endpoints)

None use custom cryptographic signing. All are **OK**.

---

## 2. CRITICAL FLAG — Paradex StarkNet: Hardcoded k=1 Nonce

**File**: `src/crypto/dex/paradex/auth.rs`, line 296

```rust
// WARNING: k = 1 is NOT safe for production — use RFC 6979 deterministic nonces.
let k = FieldElement::from(1u64);
let signature = sign(&private_key, &message, &k)
    .map_err(|e| ExchangeError::Auth(format!("StarkNet sign failed: {}", e)))?;
```

### Severity: CRITICAL

Using a fixed nonce `k = 1` in any deterministic ECDSA (including StarkNet's variant) is a
**catastrophic private key disclosure vulnerability**. Two signatures with the same `k` value
over different messages allow trivial algebraic recovery of the private key:

```
k = 1 (known)
r₁, s₁ = sign(msg₁, k, privkey)
r₂, s₂ = sign(msg₂, k, privkey)
privkey = (s₁ * z₂ - s₂ * z₁) / (r * (s₁ - s₂))  — recoverable directly
```

### Current Mitigations

- Gated behind `#[cfg(feature = "starknet")]` — disabled by default
- Comment in code explicitly warns against production use
- Method is only called via `refresh_if_needed()` which is also feature-gated

### Required Fix

Replace `k = FieldElement::from(1u64)` with RFC 6979 deterministic nonce generation.
The `starknet-crypto` crate (v0.6) provides `sign` but RFC 6979 nonce must be generated
externally or using a wrapper. Options:

1. Use `starknet-crypto`'s built-in RFC 6979 support if available in v0.6+
2. Port the RFC 6979 algorithm from the StarkNet Rust SDK
3. Use the `rfc6979` crate to derive `k` deterministically from `(privkey, msg_hash)`

**Until fixed**: the `starknet` / `onchain-starknet` feature must NOT be enabled
in production builds.

---

## 3. EIP-712 Correctness Assessment — Hyperliquid

### 3a. L1 Phantom Agent Signing (`auth.rs`)

**Domain**:
```
name: "Exchange"
version: "1"
chainId: 42161 (mainnet) or 421614 (testnet)
verifyingContract: 0x0000000000000000000000000000000000000000
```

**Flow**:
1. `msgpack(action)` → bytes
2. `keccak256(bytes + nonce_be8 + vault_flag)` → `connectionId`
3. `hashStruct(Agent(phantom_source=USDC_addr, connectionId))` → `agentHash`
4. `keccak256("\x19\x01" + domainSep + agentHash)` → `finalHash`
5. Sign `finalHash` with secp256k1

**Assessment**: CORRECT. Implementation matches the Hyperliquid Python SDK exactly:
- Domain separator fields in correct order (type_hash, name_hash, version_hash, chainId, verifyingContract)
- Address padding correct (12 zero bytes + 20 address bytes = 32)
- `"\x19\x01"` prefix present
- Phantom source address `0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48` is correct (USDC contract)
- `v = bytes[64] == 0 ? 27 : 28` is correct Ethereum convention

**Minor note**: The `eip712.rs` module re-implements `l1_domain_separator` and `l1_agent_struct_hash`
instead of importing from `auth.rs`. This is a code duplication issue (not a correctness bug), but
creates a risk of divergence if either copy is updated.

### 3b. User-Signed Actions (`eip712.rs`, `onchain-evm` feature)

**Domain**:
```
name: "HyperliquidSignTransaction"  ← different from L1 domain
version: "1"
chainId: 42161 or 421614
verifyingContract: zero address
```

**Type strings verified**:
- `WithdrawFromBridge2`: `HyperliquidTransaction:WithdrawFromBridge2(string hyperliquidChain,string destination,string amount,uint64 time)` — CORRECT
- `UsdSend`: `HyperliquidTransaction:UsdSend(string hyperliquidChain,string destination,string amount,uint64 time)` — CORRECT
- `SpotSend`: `HyperliquidTransaction:SpotSend(string hyperliquidChain,string destination,string token,string amount,uint64 time)` — CORRECT

**String encoding**: Each `string` field is hashed as `keccak256(field.as_bytes())` — CORRECT per EIP-712 spec.

**uint64 encoding**: Packed as 8 bytes big-endian in a 32-byte slot (`time_bytes[24..]`) — CORRECT.

**Assessment**: CORRECT. Both domain separators are properly differentiated (L1 vs user-signed),
field ordering is deterministic, all type hashes are correctly formed.

---

## 4. FLAG — Coinbase: JWT Nonce Not Injected into Header

**File**: `src/crypto/cex/coinbase/auth.rs`, lines 144–168

```rust
let header_json = serde_json::json!({
    "alg": "ES256",
    "typ": "JWT",
    "kid": self.api_key_name.clone(),
    "nonce": nonce,  // ← generated but NOT used
});

// jsonwebtoken doesn't support custom header fields (like nonce)
let token = encode(&header, &claims, &self.encoding_key)?;
let _ = header_json;  // ← discarded!
```

### Severity: MEDIUM

The Coinbase Advanced Trade API requires a `nonce` field in the JWT header for replay
protection. The current implementation generates a nonce but silently discards it because
`jsonwebtoken::Header` does not support custom fields.

The code comment acknowledges this: _"The nonce is generated but not added to the JWT for now"_.

### Consequence

- Requests may be rejected by Coinbase if nonce is strictly required
- Without nonce, repeated identical requests could potentially be replayed (within the 2-min window)

### Required Fix

Manually construct the JWT instead of using `jsonwebtoken::encode`:

```rust
// 1. Build header JSON with nonce field
let header_json = serde_json::json!({
    "alg": "ES256",
    "typ": "JWT",
    "kid": api_key_name,
    "nonce": nonce,
});

// 2. base64url(header) + "." + base64url(payload)
let signing_input = format!("{}.{}", base64url(header_json), base64url(claims));

// 3. Sign with ECDSA P-256
let signature = ecdsa_sign(signing_input.as_bytes(), &private_key)?;

// 4. Final JWT
let jwt = format!("{}.{}", signing_input, base64url(signature));
```

This requires using `p256` crate directly instead of `jsonwebtoken`.

---

## 5. FLAG — Vertex: Broken Import (ethers crate missing, service dead)

**File**: `src/crypto/cex/vertex/auth.rs`, lines 18–19

```rust
use ethers::signers::{LocalWallet, Signer};
use ethers::types::transaction::eip712::{Eip712, TypedData};
```

### Issues

1. **`ethers` is not declared in `Cargo.toml`** — this file will fail to compile if vertex is included
   without the `ethers` dependency being added.

2. **Vertex Protocol permanently shut down on August 14, 2025** (documented in `mod.rs`).
   The entire connector is dead code.

### Recommendation

The vertex connector can remain in the codebase as historical reference, but should be:
- Gated behind a `#[cfg(feature = "vertex-legacy")]` feature flag, OR
- The auth.rs file should have its `use ethers::...` lines removed and signing replaced with
  a stub returning `Err(ExchangeError::Unavailable("Vertex shut down 2025-08-14"))`

This prevents compilation failures if `cargo check` is run without the feature flag.

---

## 6. FLAG — Lighter: Signing Not Implemented (ECgFp5 + Poseidon2)

**File**: `src/crypto/dex/lighter/auth.rs`

All signing methods return `Err(ExchangeError::Auth(...))` with clear documentation explaining
why: Lighter uses **ECgFp5 Schnorr over the Goldilocks field with Poseidon2 hashing** — a
ZK-native cryptographic stack incompatible with standard Rust crates (`k256`, `secp256k1`).

The `k256-signing` feature provides secp256k1 ECDSA as a bridging option for L1/EVM
compatibility layers, but this does **not** satisfy Lighter's native L2 signing requirement.

### Current Impact

- Public market data endpoints: **fully functional**
- Authenticated endpoints (order placement, cancellation): **returns error at runtime**

### Recommendation

To enable Lighter trading, either:
1. Add the official `lighter-sdk` crate as an optional dependency when published
2. Port ECgFp5 Schnorr from the Lighter TypeScript SDK
3. Accept that this connector is data-only until the ZK crypto stack is available

The current state (documented stub with clear error messages) is acceptable for a data-only connector.

---

## 7. Additional Observations

### Kraken Base64 Decoder

`src/crypto/cex/kraken/auth.rs` contains a hand-rolled base64 decoder (lines 119–181) instead
of using the `base64` crate which is already a direct dependency. The implementation appears
correct but has a subtle issue: the `'='` padding character maps to `0x00` in the decode table
(index 61 = `=` → `DECODE_TABLE[61] = 0x00`), which could silently produce wrong output for
certain malformed inputs rather than erroring. **Low severity** — Kraken secrets are valid
base64 in practice.

**Recommendation**: Replace with `base64::engine::general_purpose::STANDARD.decode()`
from the existing `base64 = "0.22"` dependency.

### Bitfinex Clone Not Derived

`BitfinexAuth` uses `AtomicU64` for nonce tracking and does not derive `Clone`. The `Clone`
bound is required by several connector trait implementations. This is a compile-time issue
that would surface when cloning is attempted.

### Deribit `client_credentials` Sends Secret in Plaintext

The `client_credentials_params()` method sends `client_secret` directly in the request body:
```rust
params.insert("client_secret".to_string(), serde_json::json!(self.client_secret));
```
This is the documented Deribit behavior for the `client_credentials` grant type, but callers
should prefer `client_signature_params()` (which uses HMAC-SHA256 and never transmits the secret).
**Not a bug**, but worth documenting for operators.

### Bitstamp: Missing Content-Type When Body Present But Method Is GET

The `sign_request` method correctly adds `Content-Type: application/x-www-form-urlencoded` only
when `body` is non-empty. However, it always uses `www.bitstamp.net` as the host regardless of
whether the actual request targets a different host (e.g., testnet or international endpoints).
**Low severity** — Bitstamp only has one production host.

---

## 8. Hardcoded Secrets Check

Grep for hardcoded private keys / secrets in all auth.rs files:

- No actual private keys or API secrets found hardcoded in any auth.rs
- Test values like `"test_key"` / `"test_secret"` appear only in `#[cfg(test)]` blocks
- `AngelOneAuth` tests use example TOTP secret `"JBSWY3DPEHPK3PXP"` — this is the standard
  TOTP demo key from RFC 6238, not a real credential
- Paradex `sign_auth_request` has a `k = FieldElement::from(1u64)` — a cryptographic constant,
  not a hardcoded secret, but critically dangerous (see §2)

**No hardcoded production secrets found.**

---

## 9. Replay Attack Surface

| Connector | Replay Protection | Notes |
|-----------|------------------|-------|
| Binance / Bybit / OKX | Timestamp window | 5-second window, standard |
| Kraken | Strictly increasing nonce | Mutex-protected counter |
| Bitfinex | Strictly increasing nonce | AtomicU64 microseconds |
| Bitstamp | UUID v4 nonce | Unique per request |
| Gemini | Millisecond nonce | Standard |
| Hyperliquid L1 | Timestamp-based nonce | `AtomicU64::fetch_max(now)` |
| Hyperliquid user-signed | `time` field in struct | Timestamp in signed data |
| Coinbase | 2-min JWT expiry | Nonce in header missing (see §4) |
| Upbit | UUID v4 in JWT | OK |
| Deribit | Nonce in signature | Random 16-char alphanumeric |
| Polymarket | Millisecond timestamp | In signed message |
| Paradex | Timestamp signed | But k=1 negates all security (see §2) |

---

## 10. Priority Action Items

| Priority | Item | File | Action |
|----------|------|------|--------|
| P0 CRITICAL | Paradex k=1 nonce | `src/crypto/dex/paradex/auth.rs:296` | Replace with RFC 6979 deterministic nonce before enabling `starknet` feature in production |
| P1 HIGH | Coinbase JWT nonce not injected | `src/crypto/cex/coinbase/auth.rs:144–169` | Manual JWT construction with nonce in header |
| P2 MEDIUM | Vertex ethers import fails to compile | `src/crypto/cex/vertex/auth.rs:18–19` | Gate behind feature flag or replace with stub |
| P3 LOW | Lighter signing not implemented | `src/crypto/dex/lighter/auth.rs` | Add lighter-sdk when published (acceptable as data-only) |
| P4 LOW | Kraken hand-rolled base64 | `src/crypto/cex/kraken/auth.rs:119–181` | Replace with `base64` crate |
| P4 LOW | Hyperliquid EIP-712 code duplication | `auth.rs` vs `eip712.rs` | Extract shared helpers into `mod crypto` |
