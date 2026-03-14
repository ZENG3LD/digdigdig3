# Lighter Connector — Auth Implementation Audit

**Date**: 2026-03-15
**Source files audited**:
- `src/crypto/dex/lighter/auth.rs`
- `src/crypto/dex/lighter/connector.rs`
- `src/crypto/dex/lighter/endpoints.rs`
- `src/crypto/dex/lighter/research/tx_signing_format.md`
- `src/crypto/dex/lighter/research/authentication.md`
- `src/crypto/dex/lighter/research/trading-research/auth_levels.md`
- `src/crypto/dex/lighter/research/LIGHTER_OPEN_ORDERS_RESEARCH.md`

---

## 1. What Works Today

### Public Market Data (fully functional — no auth required)

All read-only market data endpoints work without any signing:

| Method | Endpoint used |
|--------|--------------|
| `get_price` | `GET /api/v1/orderBookDetails` |
| `get_ticker` | `GET /api/v1/orderBookDetails` |
| `get_orderbook` | `GET /api/v1/orderBookOrders` |
| `get_klines` | `GET /api/v1/candles` |
| `ping` | `GET /` |
| `get_exchange_info` | static map (geo-blocked endpoint bypassed) |
| `get_recent_trades` | `GET /api/v1/recentTrades` |
| `get_exchange_stats` | `GET /api/v1/exchangeStats` |
| `get_current_height` | `GET /api/v1/currentHeight` |
| `get_funding_rates` | `GET /api/v1/funding-rates` |
| `get_exchange_metrics` | `GET /api/v1/exchangeMetrics` |

### Account Queries (work without auth token — server currently marks auth optional)

These pass `account_index` as a query param. The server marks both `auth` and `Authorization` as `required: false` in the current API spec:

| Method | Endpoint used |
|--------|--------------|
| `get_balance` | `GET /api/v1/account?by=index&value=N` |
| `get_account_info` | `GET /api/v1/account` |
| `get_positions` | `GET /api/v1/account` (positions embedded in response) |
| `get_funding_rate` | `GET /api/v1/fundings` |
| `get_open_orders` | `GET /api/v1/accountActiveOrders` |
| `get_order_history` | `GET /api/v1/accountInactiveOrders` |
| `get_fees` | `GET /api/v1/orderBooks` |

### k256-signing Feature: secp256k1 ECDSA Path

When compiled with `--features k256-signing`, the connector provides a **partial** signing path:

- `sign_l2_transaction(tx_hash: &[u8; 32])` — secp256k1 ECDSA signature (k256 crate), returns 64-byte r||s compact signature
- `build_create_order_hash(...)` — SHA-256 over a canonical query-string of fields
- `build_cancel_order_hash(...)` — SHA-256 over a canonical query-string of fields
- `place_order_signed(req)` — full place-order flow using the above
- `cancel_order_signed(req)` — full cancel-order flow using the above
- `fetch_next_nonce(account_index)` — working nonce fetch from `GET /api/v1/nextNonce`

These compile and run without panics. They will be rejected by the Lighter server because the server expects **ECgFp5+Poseidon2 Schnorr signatures**, not secp256k1 ECDSA.

---

## 2. What Is Stubbed (ECgFp5 Path — Returns ExchangeError::Auth)

Three methods in `auth.rs` immediately return `ExchangeError::Auth`:

### `generate_auth_token(expiry_seconds: u64) -> ExchangeResult<String>`

```
Err: "Lighter auth token generation requires ZK-native Schnorr+ECgFp5+Poseidon2 signing
      over the Goldilocks field. This is incompatible with standard ECDSA libraries."
```

The token this method should produce has the format:
```
{deadline}:{account_index}:{api_key_index}:{base64_schnorr_signature}
```
Where `deadline` is a Unix timestamp and the signed message is `Poseidon2("{deadline}:{account_index}:{api_key_index}")`.

### `generate_readonly_token(expiry_seconds: u64, scope: &str) -> ExchangeResult<String>`

```
Err: "Lighter read-only token generation requires ZK-native Schnorr+ECgFp5+Poseidon2 signing."
```

Token format: `ro:{account_index}:{single|all}:{expiry_unix}:{base64_schnorr_signature}`

### `sign_transaction(tx_type: u8, tx_data: &HashMap<String, Value>) -> ExchangeResult<String>`

```
Err: "Lighter transaction signing requires ZK-native Schnorr+ECgFp5+Poseidon2 signing
      (NOT standard ECDSA/secp256k1)."
```

This is the generic stub for L2 transaction signing. The k256 path (`place_order_signed`, `cancel_order_signed`) bypasses this method entirely and calls `build_create_order_hash` + `sign_l2_transaction` directly.

---

## 3. Exact Field Layout for Order Hashes

### The WRONG approach (current k256 path)

The current `build_create_order_hash` constructs a **canonical query string** and hashes it with SHA-256:

```
"account_index={}&base_amount={}&is_ask={}&market_id={}&nonce={}&order_type={}&price={}&time_in_force={}&tx_type=14"
```

This is incorrect. The server will reject it.

### The CORRECT approach (ECgFp5 + Poseidon2)

From `research/tx_signing_format.md` (sourced from elliottech/lighter-go and robustfengbin/lighter-sdk):

#### L2CreateOrder (tx_type = 14) — 16 GoldilocksField elements in exact order:

| Index | Field | Type | Encoding |
|-------|-------|------|----------|
| 0 | `chain_id` | u32 | `from_u32` |
| 1 | `tx_type = 14` | u32 | `from_u32` |
| 2 | `nonce` | i64 | `from_i64` |
| 3 | `expired_at` | i64 (unix ms) | `from_i64` |
| 4 | `account_index` | i64 | `from_i64` |
| 5 | `api_key_index` | u8 → u32 | `from_u32` |
| 6 | `market_index` | i16 → u32 | `from_u32` |
| 7 | `client_order_index` | i64 | `from_i64` |
| 8 | `base_amount` | i64 | `from_i64` |
| 9 | `price` | u32 | `from_u32` |
| 10 | `is_ask` | bool → u32 (0 or 1) | `from_u32` |
| 11 | `order_type` | u8 → u32 | `from_u32` |
| 12 | `time_in_force` | u8 → u32 | `from_u32` |
| 13 | `reduce_only` | bool → u32 | `from_u32` |
| 14 | `trigger_price` | u32 | `from_u32` |
| 15 | `order_expiry` | i64 | `from_i64` |

Then: `hash = Poseidon2::hash_to_quintic_extension(elements)` → GFp5 (5 × u64)

#### L2CancelOrder (tx_type = 15) — 8 GoldilocksField elements:

| Index | Field | Type | Encoding |
|-------|-------|------|----------|
| 0 | `chain_id` | u32 | `from_u32` |
| 1 | `tx_type = 15` | u32 | `from_u32` |
| 2 | `nonce` | i64 | `from_i64` |
| 3 | `expired_at` | i64 (unix ms) | `from_i64` |
| 4 | `account_index` | i64 | `from_i64` |
| 5 | `api_key_index` | u8 → u32 | `from_u32` |
| 6 | `market_index` | i16 → u32 | `from_u32` |
| 7 | `index` (order_id or client_order_id) | i64 | `from_i64` |

Then: `hash = Poseidon2::hash_to_quintic_extension(elements)` → GFp5

#### Key differences from the current implementation:

1. `chain_id` is the FIRST field (mainnet = 304, testnet = 300). The current k256 path has no chain_id.
2. `expired_at` (transaction deadline in unix milliseconds) is required. The current k256 path has no expiry.
3. `api_key_index` is a required field in the hash. The current k256 path omits it.
4. `client_order_index` is part of the CreateOrder hash. The current k256 path omits it.
5. `reduce_only`, `trigger_price`, `order_expiry` are in the CreateOrder hash. The current k256 path omits all three.
6. The hash function is **Poseidon2 `hash_to_quintic_extension`**, not SHA-256.
7. Input elements are **GoldilocksField elements** (u64 values in Goldilocks field arithmetic), not UTF-8 strings.

#### GoldilocksField encoding rules:

```rust
// For u32 values (booleans, enums, u8 indices):
GoldilocksField::new(value as u64)

// For i64 values (account_index, nonce, amounts, timestamps):
if value >= 0 {
    GoldilocksField::new(value as u64)
} else {
    // Negative: use field subtraction: 0 - abs(value)
    GoldilocksField::ZERO - GoldilocksField::new((-value) as u64)
}
```

Internal representation is Montgomery form, but `to_le_bytes()` converts back to canonical first.

---

## 4. The API Flow: Nonce → Hash → Sign → POST

### Step 1: Get Nonce

```
GET /api/v1/nextNonce?account_index={n}&api_key_index={k}
Response: { "code": 200, "nonce": 42 }
```

The nonce is per (account_index, api_key_index) pair. `fetch_next_nonce` in `connector.rs` **only passes `account_index`** — it should also pass `api_key_index`. This is a bug in the current implementation.

### Step 2: Build expiry

```
expired_at = current_unix_ms + 599_000   // 10 minutes minus 1 second (SDK default)
```

### Step 3: Build field element array

See the exact tables above. Build a `Vec<GoldilocksField>` with all 16 (create) or 8 (cancel) elements in the exact documented order.

### Step 4: Hash with Poseidon2

```rust
let hash: GFp5 = Poseidon2::hash_to_quintic_extension(&elements);
// GFp5 = [u64; 5], a quintic extension field element over Goldilocks
```

### Step 5: Schnorr sign

```rust
// Sign pre-computed GFp5 hash with ECgFp5 private key
fn schnorr_sign_hashed_message(hashed_msg: GFp5, sk: &ECgFp5Scalar) -> Signature {
    let k = ECgFp5Scalar::sample_random();                    // random nonce
    let r = ECgFp5Point::generator().mul(&k).encode();        // r = k*G as GFp5

    // e = H(r || msg) — second Poseidon2 call over 10 elements
    let mut pre_image = r.to_basefield_array();               // 5 u64s
    pre_image.extend(hashed_msg.to_basefield_array());        // 5 u64s
    let e_gfp5 = hash_to_quintic_extension(&pre_image);
    let e = ECgFp5Scalar::from_gfp5(e_gfp5);

    let s = k.sub(&e.mul(sk));                                // s = k - e*sk
    Signature { s, e }
}
```

### Step 6: Serialize signature

```rust
let mut sig_bytes = [0u8; 80];
sig_bytes[..40].copy_from_slice(&sig.s.to_le_bytes());   // s: first 40 bytes
sig_bytes[40..].copy_from_slice(&sig.e.to_le_bytes());   // e: last 40 bytes
let sig_base64 = base64::encode(&sig_bytes);
```

Each ECgFp5Scalar is **40 bytes** (5 × u64 little-endian). Total signature: **80 bytes**, base64-encoded.

### Step 7: Build tx_info JSON

For CreateOrder (tx_type=14):
```json
{
  "account_index": 12345,
  "api_key_index": 3,
  "market_index": 0,
  "client_order_index": 1,
  "base_amount": 100000000,
  "price": 200000,
  "is_ask": false,
  "order_type": 0,
  "time_in_force": 0,
  "reduce_only": false,
  "trigger_price": 0,
  "order_expiry": 0,
  "expired_at": 1741000000000,
  "nonce": 42,
  "sig": "BASE64_ENCODED_80_BYTE_SIGNATURE"
}
```

Note: `signed_hash` is computed internally but **NOT included** in the JSON (skip_serializing in Rust SDK).

For CancelOrder (tx_type=15):
```json
{
  "account_index": 12345,
  "api_key_index": 3,
  "market_index": 0,
  "index": 9876,
  "expired_at": 1741000000000,
  "nonce": 43,
  "sig": "BASE64_ENCODED_80_BYTE_SIGNATURE"
}
```

### Step 8: POST as multipart/form-data

**Critical**: The endpoint uses `multipart/form-data`, NOT `application/json`:

```rust
let form = reqwest::multipart::Form::new()
    .text("tx_type", "14")         // string, the integer tx type
    .text("tx_info", tx_info_json); // JSON-serialized signed transaction as string

client.post("https://mainnet.zklighter.elliot.ai/api/v1/sendTx")
    .multipart(form)
    .send()
    .await?
```

The current `post()` method in `connector.rs` sends `application/json` body. This is wrong — it will be rejected by the server.

---

## 5. What the Server Expects in the Authorization Header

For authenticated read endpoints (account data, WebSocket subscriptions):

```
Authorization: {expiry_unix}:{account_index}:{api_key_index}:{base64_schnorr_signature}
```

Or as a query parameter:
```
?auth={expiry_unix}:{account_index}:{api_key_index}:{base64_schnorr_signature}
```

The signed message for the auth token is: Poseidon2 hash of the string `"{expiry_unix}:{account_index}:{api_key_index}"` treated as GoldilocksField elements. The result is Schnorr-signed with the API private key, producing the same 80-byte ECgFp5 signature, base64-encoded.

**For write operations**: Auth is embedded inside the signed `tx_info` JSON via the `sig` field. An additional `?auth=...` query parameter (or `Authorization` header) may also be passed to `sendTx` for permission verification, but the transaction signature itself is the primary auth mechanism.

**Read-only tokens**:
```
ro:{account_index}:{single|all}:{expiry_unix}:{base64_schnorr_signature}
```
Max expiry: 10 years. Suitable for read-only WebSocket subscriptions.

---

## 6. Private Key Format and Cryptographic Parameters

### Private Key
- **40 bytes** (not 32 like secp256k1)
- Hex-encoded string (with or without `0x` prefix)
- This is an **ECgFp5 scalar** — a value in the Goldilocks scalar field
- Public key = `sk * G` where G is the ECgFp5 generator point
- Public key is a GFp5 element (5 × u64 = 40 bytes), hex-encoded for API registration

### Goldilocks Field
- Prime: `p = 2^64 - 2^32 + 1 = 18446744069414584321`
- All field arithmetic is mod p
- Montgomery form used internally; canonical form for serialization

### ECgFp5 Curve
- Defined over GF(p^5) (quintic extension of Goldilocks field)
- 5-isogeny of the "Goldilocks" elliptic curve
- Designed for ZK-SNARK (plonky2) compatibility
- Paper: Thomas Pornin, "ECgFp5: A SNARK-Friendly Elliptic Curve" (ePrint 2022/274)

### Poseidon2 Hash
- ZK-friendly sponge construction over Goldilocks field
- `hash_to_quintic_extension(inputs: &[GoldilocksField]) -> GFp5`
- Output is a GFp5 element (5 × u64), used directly as the message to sign
- The number of rounds and state width are defined in the `plonky2` / `lighter-sdk` implementation
- Two Poseidon2 invocations occur per signature: one for the tx hash, one inside Schnorr (for `e = H(r || msg)`)

### Network Constants
| Network | Chain ID |
|---------|----------|
| Mainnet | 304 |
| Testnet | 300 |

---

## 7. Bugs and Gaps in the Current k256 Implementation

### Bug 1: Wrong POST content type

`connector.rs` `post()` sends `Content-Type: application/json`. The `sendTx` endpoint requires `multipart/form-data` with fields `tx_type` (string) and `tx_info` (JSON string). The current implementation wraps both in a single JSON object — this is wrong.

### Bug 2: fetch_next_nonce missing api_key_index

`fetch_next_nonce(account_index)` only sends `account_index` to `GET /api/v1/nextNonce`. The endpoint requires both `account_index` AND `api_key_index` because each API key has its own nonce counter.

### Bug 3: Missing fields in hash construction

`build_create_order_hash` is missing: `chain_id`, `expired_at`, `api_key_index`, `client_order_index`, `reduce_only`, `trigger_price`, `order_expiry`. These fields are all required in the Poseidon2 hash input.

### Bug 4: Wrong hash algorithm

SHA-256 over UTF-8 query string is used. Correct: Poseidon2 `hash_to_quintic_extension` over GoldilocksField elements.

### Bug 5: Wrong signature size and encoding

Current: 64-byte secp256k1 ECDSA, hex-encoded, field key `"signature"`.
Correct: 80-byte ECgFp5 Schnorr, base64-encoded, field key `"sig"`.

### Bug 6: cancel_order_signed tx_info field name

Current uses `"order_index"` in the JSON. Correct field name per API spec is `"index"`.

### Bug 7: place_order_signed has no expired_at

The tx_info JSON sent by `place_order_signed` has no `expired_at` field. The server requires it (transaction deadline in unix milliseconds).

---

## 8. Implementation Path to Real Signing

### Option A: Use robustfengbin/lighter-sdk (pure Rust, recommended)

```toml
# Cargo.toml
lighter-sdk = { git = "https://github.com/robustfengbin/lighter-sdk" }
base64 = "0.21"
```

The SDK provides `LighterSigner` with:
- `sign_create_order(...)` — returns signed tx_info JSON string
- `sign_cancel_order(...)` — returns signed tx_info JSON string
- `create_auth_token(deadline, account_index, api_key_index)` — returns auth token string
- All underlying ECgFp5 / Goldilocks / Poseidon2 / Schnorr implementations

### Option B: Vendor the crypto primitives directly

Port from `robustfengbin/lighter-sdk` into the codebase:
- `goldilocks.rs` — GoldilocksField arithmetic
- `ecgfp5.rs` — ECgFp5 curve, point multiplication, encoding
- `poseidon2.rs` — Poseidon2 sponge, `hash_to_quintic_extension`
- `schnorr.rs` — `schnorr_sign_hashed_message`

### Option C: Lighter-rs crate on crates.io

```toml
lighter-rs = "0.1"  # check current version
```

Provides compatible signing but dependency on external crate with unknown maintenance status.

---

## 9. Summary

| Layer | Current State | What Is Needed |
|-------|--------------|----------------|
| Public market data | Working | Nothing |
| Account data (no auth) | Working (server currently allows) | Auth token for production |
| Auth token generation | Stub — returns ExchangeError::Auth | ECgFp5+Poseidon2 Schnorr signing |
| Read-only token generation | Stub — returns ExchangeError::Auth | ECgFp5+Poseidon2 Schnorr signing |
| Order placement | Compiles, sends wrong data | Poseidon2 hash, ECgFp5 Schnorr, multipart POST |
| Order cancellation | Compiles, sends wrong data | Poseidon2 hash, ECgFp5 Schnorr, multipart POST |
| Nonce fetch | Working (missing api_key_index param) | Add api_key_index to query |
| `sign_transaction()` | Stub — returns ExchangeError::Auth | Full ECgFp5 implementation |

The entire signing stack needs to be replaced. The k256 path exists as scaffolding that demonstrates the correct API call structure (nonce fetch → hash → sign → POST), but every cryptographic primitive in it is wrong for Lighter's actual protocol.

---

## 10. References

- `src/crypto/dex/lighter/auth.rs` — LighterAuth struct, all methods
- `src/crypto/dex/lighter/connector.rs` — place_order_signed, cancel_order_signed, fetch_next_nonce
- `src/crypto/dex/lighter/research/tx_signing_format.md` — exact field layouts from lighter-go and lighter-sdk
- `src/crypto/dex/lighter/research/authentication.md` — auth token formats, nonce management
- `src/crypto/dex/lighter/research/trading-research/auth_levels.md` — rate limits, key indices
- https://github.com/robustfengbin/lighter-sdk — pure Rust reference implementation
- https://github.com/elliottech/lighter-go — official Go SDK (source of field layouts)
- https://eprint.iacr.org/2022/274.pdf — ECgFp5 paper (Thomas Pornin)
- https://apidocs.lighter.xyz/reference/sendtx — sendTx API reference
