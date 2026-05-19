# Lighter.xyz Transaction Signing Format

**Research Date:** 2026-03-12
**Sources:** elliottech/lighter-go, elliottech/lighter-python, robustfengbin/lighter-sdk (Rust)

---

## Critical Correction: NOT secp256k1/ECDSA

The initial assumption that Lighter uses ECDSA secp256k1 is **wrong**.

Lighter uses a completely custom cryptographic stack:

| Layer | Algorithm |
|-------|-----------|
| Elliptic curve | **ECgFp5** (curve over GF(p^5), p = 2^64 - 2^32 + 1 = Goldilocks prime) |
| Hash function | **Poseidon2** over Goldilocks field |
| Signature scheme | **Schnorr** signatures over ECgFp5 |
| Private key size | **40 bytes** (not 32 like secp256k1) |
| Signature size | **80 bytes** (two ECgFp5 scalars: s || e, 40 bytes each) |

This is a ZK-native crypto stack designed for the Lighter ZK rollup (plonky2-based).

---

## Network Constants

| Network | Chain ID | Base URL |
|---------|----------|----------|
| Mainnet | `304` | `https://mainnet.zklighter.elliot.ai` |
| Testnet | `300` | `https://testnet.zklighter.elliot.ai` |

---

## Transaction Type Values

From `types/txtypes/constants.go` in lighter-go:

```
TxTypeL2CreateOrder  = 14
TxTypeL2CancelOrder  = 15
TxTypeL2ModifyOrder  = 19 (for reference)
TxTypeL2CancelAllOrders = 18
TxTypeL2Transfer     = 13
TxTypeL2Withdraw     = 12
```

---

## Signing Architecture Overview

```
Transaction Fields
      |
      v
[Build array of GoldilocksField elements in exact order]
      |
      v
[Poseidon2 hash → HashToQuinticExtension → GFp5 (5 u64s)]
      |
      v
[Schnorr sign with ECgFp5 private key → Signature{s: ECgFp5Scalar, e: ECgFp5Scalar}]
      |
      v
[sig.to_bytes() → 80 bytes → base64 encode → "sig" field in JSON]
[hash.to_le_bytes() → hex encode → "signed_hash" field (NOT sent to API)]
```

---

## GoldilocksField Encoding

The Goldilocks prime is `p = 2^64 - 2^32 + 1`.

Field elements are encoded from transaction values as follows (from `goldilocks.rs`):

```rust
// From u32 value (boolean flags, enums, u8 indices)
GoldilocksField::from_u32(value: u32) -> GoldilocksField {
    GoldilocksField::new(value as u64)
}

// From i64 value (account_index, nonce, amounts, timestamps)
GoldilocksField::from_i64(value: i64) -> GoldilocksField {
    if value >= 0 {
        GoldilocksField::new(value as u64)
    } else {
        GoldilocksField::ZERO - GoldilocksField::new((-value) as u64)
    }
}
```

Internal storage is in **Montgomery form**, but `to_le_bytes()` converts back to canonical form first.

---

## L2CreateOrder (tx_type = 14): Hash Input

From `create_order.go` (lighter-go) and `signer.rs` (lighter-sdk Rust):

The Poseidon2 hash input is a vector of **16 GoldilocksField elements** in this exact order:

| Index | Field | Type | Encoding |
|-------|-------|------|----------|
| 0 | `chain_id` | u32 | `from_u32` |
| 1 | `tx_type = 14` | u32 | `from_u32` |
| 2 | `nonce` | i64 | `from_i64` |
| 3 | `expired_at` | i64 | `from_i64` |
| 4 | `account_index` | i64 | `from_i64` |
| 5 | `api_key_index` | u8 → u32 | `from_u32` |
| 6 | `market_index` | i16 → u32 | `from_u32` |
| 7 | `client_order_index` | i64 | `from_i64` |
| 8 | `base_amount` | i64 | `from_i64` |
| 9 | `price` | u32 | `from_u32` |
| 10 | `is_ask` | bool → u32 (0 or 1) | `from_u32` |
| 11 | `order_type` | u8 → u32 | `from_u32` |
| 12 | `time_in_force` | u8 → u32 | `from_u32` |
| 13 | `reduce_only` | bool → u32 (0 or 1) | `from_u32` |
| 14 | `trigger_price` | u32 | `from_u32` |
| 15 | `order_expiry` | i64 | `from_i64` |

Then: `hash = Poseidon2::hash_to_quintic_extension(elements)` → GFp5

If L2TxAttributes (integrator fees) are present, then:
`final_hash = aggregate_tx_hash(tx_hash, attributes_hash)`

For simple orders with no integrator: `final_hash = tx_hash` directly.

---

## L2CancelOrder (tx_type = 15): Hash Input

From `cancel_order.go` (lighter-go) and `signer.rs` (lighter-sdk Rust):

The Poseidon2 hash input is a vector of **8 GoldilocksField elements**:

| Index | Field | Type | Encoding |
|-------|-------|------|----------|
| 0 | `chain_id` | u32 | `from_u32` |
| 1 | `tx_type = 15` | u32 | `from_u32` |
| 2 | `nonce` | i64 | `from_i64` |
| 3 | `expired_at` | i64 | `from_i64` |
| 4 | `account_index` | i64 | `from_i64` |
| 5 | `api_key_index` | u8 → u32 | `from_u32` |
| 6 | `market_index` | i16 → u32 | `from_u32` |
| 7 | `index` (order_id or client_order_id) | i64 | `from_i64` |

Then: `hash = Poseidon2::hash_to_quintic_extension(elements)` → GFp5

---

## Schnorr Signature Scheme

From `schnorr.rs` (lighter-sdk Rust, cross-referenced with lighter-go):

```rust
// Sign a pre-computed Poseidon2 hash
fn schnorr_sign_hashed_message(hashed_msg: GFp5, sk: &ECgFp5Scalar) -> Signature {
    // 1. Sample random scalar k (nonce, not the tx nonce)
    let k = ECgFp5Scalar::sample_random();

    // 2. Compute r = k * G  (ECgFp5 generator point, encoded as GFp5)
    let r = ECgFp5Point::generator().mul(&k).encode();  // GFp5

    // 3. Compute e = H(r || hashed_msg)  — 10 field elements total
    let mut pre_image = Vec::with_capacity(10);
    pre_image.extend(r.to_basefield_array());           // 5 u64s
    pre_image.extend(hashed_msg.to_basefield_array());  // 5 u64s
    let e_gfp5 = hash_to_quintic_extension(&pre_image); // Poseidon2 again
    let e = ECgFp5Scalar::from_gfp5(e_gfp5);

    // 4. Compute s = k - e * sk
    let s = k.sub(&e.mul(sk));

    Signature { s, e }
}
```

### Signature Serialization (80 bytes)

```rust
fn to_bytes(&self) -> [u8; 80] {
    let mut result = [0u8; 80];
    result[..40].copy_from_slice(&self.s.to_le_bytes());  // s: first 40 bytes
    result[40..].copy_from_slice(&self.e.to_le_bytes());  // e: last 40 bytes
    result
}
```

Each ECgFp5Scalar is serialized as **40 bytes little-endian** (5 u64s in little-endian = 5 × 8 bytes).

The 80-byte signature is then **base64-encoded** for the JSON payload.

---

## sendTx HTTP Request Format

**Endpoint:** `POST /api/v1/sendTx`

The request uses **multipart/form-data** (NOT JSON body), with two fields:

```
tx_type: "14"   (string, the integer tx type)
tx_info: "{...}"  (JSON-serialized signed transaction)
```

Optional authentication:
- Query param: `?auth=<auth_token>`
- Header: `Authorization: <auth_token>`

### tx_info JSON for CreateOrder (tx_type=14)

```json
{
  "account_index": 12345,
  "api_key_index": 0,
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

Note: `signed_hash` is computed but **NOT included** in the JSON sent to the API (`skip_serializing`).

### tx_info JSON for CancelOrder (tx_type=15)

```json
{
  "account_index": 12345,
  "api_key_index": 0,
  "market_index": 0,
  "index": 9876,
  "expired_at": 1741000000000,
  "nonce": 43,
  "sig": "BASE64_ENCODED_80_BYTE_SIGNATURE"
}
```

---

## Field Value Constraints

From `constants.go` (lighter-go):

| Field | Min | Max |
|-------|-----|-----|
| `account_index` | 0 | (1 << 48) - 2 |
| `api_key_index` | 0 | 255 (u8) |
| `market_index` | varies by market type | — |
| `base_amount` | 1 | (1 << 48) - 1 |
| `price` | 1 | (1 << 32) - 1 |
| `client_order_index` | 0 | (1 << 48) - 1 |
| `nonce` | 0 | unbounded (non-negative) |
| `expired_at` | 0 | (1 << 48) - 1 (timestamp in ms) |
| `trigger_price` | 0 | (1 << 32) - 1 (0 = no trigger) |
| `order_expiry` | 0 | (1 << 48) - 1 (0 = no expiry) |

### Order Type Enum

```
0 = LimitOrder
1 = MarketOrder
2 = StopLossOrder
3 = StopLossLimitOrder
4 = TakeProfitOrder
5 = TakeProfitLimitOrder
6 = TWAPOrder
```

### TimeInForce Enum

```
0 = GoodTillTime (GTT)
1 = ImmediateOrCancel (IOC)
2 = PostOnly
```

---

## Nonce Management

- Nonce must be **monotonically increasing** per (account_index, api_key_index) pair
- Fetch current nonce: `GET /api/v1/nextNonce?account_index={n}&api_key_index={k}`
- Response: `{ "code": 200, "nonce": 42 }`
- Increment by 1 for each transaction
- `expired_at` = current_unix_ms + expiry_window (SDK uses 10 minutes - 1 second by default)

---

## Private Key Format

- **40 bytes** (not 32)
- Hex-encoded string in SDK (with or without `0x` prefix)
- This is an **ECgFp5 scalar**, not an Ethereum private key
- The public key is derived as: `pk = sk * G` where G is the ECgFp5 generator point
- Public key is a GFp5 element (5 × u64 = 40 bytes), hex-encoded for API registration

---

## Auth Token Format

For authenticated WebSocket connections and some REST endpoints:

```
"{deadline}:{account_index}:{api_key_index}:{base64_sig}"
```

Where:
- `deadline` = unix timestamp (seconds) when token expires (default: +7 hours from now)
- The string `"{deadline}:{account_index}:{api_key_index}"` is hashed with Poseidon2
- Then Schnorr-signed with the API private key
- Signature is base64-encoded and appended

---

## Rust Implementation Requirements

To implement `sign_transaction()` in Rust, you need:

### Required Crates

```toml
# No standard crypto crates — must implement or import Lighter-specific crypto

# Option 1: Use robustfengbin/lighter-sdk (pure Rust, no FFI)
lighter-sdk = { git = "https://github.com/robustfengbin/lighter-sdk" }

# Option 2: Use goldilocks-crypto crate
goldilocks-crypto = "0.1"  # Has Poseidon2 + ECgFp5 + Schnorr

# For HTTP
reqwest = { version = "0.11", features = ["multipart", "json"] }
base64 = "0.21"
serde_json = "1.0"
```

### Implementation Skeleton (based on robustfengbin/lighter-sdk pattern)

```rust
use goldilocks_field::GoldilocksField;
use poseidon2::hash_to_quintic_extension;
use ecgfp5::{ECgFp5Scalar, schnorr_sign_hashed_message};
use base64::{engine::general_purpose::STANDARD, Engine};

const CHAIN_ID_MAINNET: u32 = 304;
const TX_TYPE_CREATE_ORDER: u32 = 14;
const TX_TYPE_CANCEL_ORDER: u32 = 15;

fn sign_create_order(
    sk: &ECgFp5Scalar,
    chain_id: u32,
    account_index: i64,
    api_key_index: u8,
    market_index: i16,
    client_order_index: i64,
    base_amount: i64,
    price: u32,
    is_ask: bool,
    order_type: u8,
    time_in_force: u8,
    reduce_only: bool,
    trigger_price: u32,
    order_expiry: i64,
    expired_at: i64,
    nonce: i64,
) -> String {
    let elements = vec![
        GoldilocksField::from_u32(chain_id),
        GoldilocksField::from_u32(TX_TYPE_CREATE_ORDER),
        GoldilocksField::from_i64(nonce),
        GoldilocksField::from_i64(expired_at),
        GoldilocksField::from_i64(account_index),
        GoldilocksField::from_u32(api_key_index as u32),
        GoldilocksField::from_u32(market_index as u32),
        GoldilocksField::from_i64(client_order_index),
        GoldilocksField::from_i64(base_amount),
        GoldilocksField::from_u32(price),
        GoldilocksField::from_u32(is_ask as u32),
        GoldilocksField::from_u32(order_type as u32),
        GoldilocksField::from_u32(time_in_force as u32),
        GoldilocksField::from_u32(reduce_only as u32),
        GoldilocksField::from_u32(trigger_price),
        GoldilocksField::from_i64(order_expiry),
    ];

    let hash = hash_to_quintic_extension(&elements); // GFp5
    let sig = schnorr_sign_hashed_message(hash, sk);
    let sig_base64 = STANDARD.encode(sig.to_bytes()); // 80 bytes → base64

    serde_json::json!({
        "account_index": account_index,
        "api_key_index": api_key_index,
        "market_index": market_index,
        "client_order_index": client_order_index,
        "base_amount": base_amount,
        "price": price,
        "is_ask": is_ask,
        "order_type": order_type,
        "time_in_force": time_in_force,
        "reduce_only": reduce_only,
        "trigger_price": trigger_price,
        "order_expiry": order_expiry,
        "expired_at": expired_at,
        "nonce": nonce,
        "sig": sig_base64,
    }).to_string()
}

fn sign_cancel_order(
    sk: &ECgFp5Scalar,
    chain_id: u32,
    account_index: i64,
    api_key_index: u8,
    market_index: i16,
    order_index: i64,
    expired_at: i64,
    nonce: i64,
) -> String {
    let elements = vec![
        GoldilocksField::from_u32(chain_id),
        GoldilocksField::from_u32(TX_TYPE_CANCEL_ORDER),
        GoldilocksField::from_i64(nonce),
        GoldilocksField::from_i64(expired_at),
        GoldilocksField::from_i64(account_index),
        GoldilocksField::from_u32(api_key_index as u32),
        GoldilocksField::from_u32(market_index as u32),
        GoldilocksField::from_i64(order_index),
    ];

    let hash = hash_to_quintic_extension(&elements);
    let sig = schnorr_sign_hashed_message(hash, sk);
    let sig_base64 = STANDARD.encode(sig.to_bytes());

    serde_json::json!({
        "account_index": account_index,
        "api_key_index": api_key_index,
        "market_index": market_index,
        "index": order_index,
        "expired_at": expired_at,
        "nonce": nonce,
        "sig": sig_base64,
    }).to_string()
}
```

---

## HTTP Submission

```rust
async fn send_tx(
    client: &reqwest::Client,
    base_url: &str,
    tx_type: u8,
    tx_info: String,
    auth_token: Option<&str>,
) -> Result<(), Error> {
    let mut url = format!("{}/api/v1/sendTx", base_url);
    if let Some(token) = auth_token {
        url = format!("{}?auth={}", url, token);
    }

    let form = reqwest::multipart::Form::new()
        .text("tx_type", tx_type.to_string())
        .text("tx_info", tx_info);

    let resp = client
        .post(&url)
        .multipart(form)
        .send()
        .await?;

    // Check response: { "code": 200, "message": "..." }
    Ok(())
}
```

---

## Key Findings Summary

1. **Lighter does NOT use secp256k1/ECDSA** — it uses Schnorr over ECgFp5 (Goldilocks-based curve)
2. **Hash = Poseidon2** `hash_to_quintic_extension()` over Goldilocks field elements
3. **Signature = 80 bytes** (two 40-byte ECgFp5 scalars: s and e), base64-encoded
4. **Private key = 40 bytes** ECgFp5 scalar (hex-encoded)
5. **tx_info is a JSON string** sent as multipart form field `tx_info` alongside `tx_type`
6. **CreateOrder hash has 16 elements**, CancelOrder hash has 8 elements
7. **Field element order matters exactly** — chain_id first, then tx_type, nonce, expired_at, then tx-specific fields
8. **`signed_hash` is NOT sent** to the API (skip_serializing in Rust SDK)
9. **Nonce endpoint**: `GET /api/v1/nextNonce?account_index=N&api_key_index=K`
10. **Existing Rust implementation**: `robustfengbin/lighter-sdk` is a pure-Rust reference implementation with no FFI

---

## Recommended Approach for Rust Implementation

Rather than reimplementing all the crypto from scratch, use or vendor the `robustfengbin/lighter-sdk` crate which provides:
- Complete ECgFp5 / Goldilocks / Poseidon2 / Schnorr implementations in pure Rust
- `LighterSigner` struct with `sign_create_order()`, `sign_cancel_order()` methods
- No external `.so`/`.dll` dependencies (unlike the Python SDK which loads shared libs)

Alternatively, the `goldilocks-crypto` crate on crates.io may provide compatible primitives.

---

## Sources

- [elliottech/lighter-python (GitHub)](https://github.com/elliottech/lighter-python)
- [elliottech/lighter-go (GitHub)](https://github.com/elliottech/lighter-go)
- [robustfengbin/lighter-sdk Rust SDK (GitHub)](https://github.com/robustfengbin/lighter-sdk)
- [0xvasanth/lighter-rs (GitHub)](https://github.com/0xvasanth/lighter-rs)
- [Lighter API Docs - sendTx](https://apidocs.lighter.xyz/reference/sendtx)
- [Lighter API Docs - Get Started For Programmers](https://apidocs.lighter.xyz/docs/get-started-for-programmers-1)
- [goldilocks-crypto crate (crates.io)](https://crates.io/crates/goldilocks-crypto)
- [ECgFp5 paper - Thomas Pornin](https://eprint.iacr.org/2022/274.pdf)
- [elliottech/lighter-go WASM main.go](https://github.com/elliottech/lighter-go/blob/main/wasm/main.go)
- [lighter-go types/txtypes/create_order.go](https://github.com/elliottech/lighter-go/blob/main/types/txtypes/create_order.go)
- [lighter-go types/txtypes/cancel_order.go](https://github.com/elliottech/lighter-go/blob/main/types/txtypes/cancel_order.go)
