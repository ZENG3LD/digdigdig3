# Poseidon2, Goldilocks Field, ECgFp5, and Schnorr Signatures in Rust

## Research Goal

Find existing Rust crates for Lighter DEX transaction signing, which requires:
1. Poseidon2 hash over the Goldilocks prime field (p = 2^64 - 2^32 + 1)
2. ECgFp5 elliptic curve (degree-5 extension of Goldilocks)
3. Schnorr signatures over ECgFp5

---

## Background: The Cryptographic Stack

Lighter DEX uses a non-standard cryptographic stack derived from zero-knowledge proof systems:

| Component | Description |
|-----------|-------------|
| **Goldilocks field** | Prime field GF(p) where p = 2^64 - 2^32 + 1 |
| **GFp5 / Fp5** | Quintic extension field GF(p^5), degree-5 tower over Goldilocks |
| **Poseidon2** | Algebraic hash function defined over Goldilocks field elements |
| **ECgFp5** | Elliptic curve over GF(p^5), designed by Thomas Pornin (NCC Group) |
| **Schnorr** | Schnorr signature scheme using ECgFp5 as the signing curve and Poseidon2 as the hash |

This stack originates from the Miden VM (Polygon) and is used by Lighter for ZK-friendly transaction signing. The signing key is a Scalar (integer mod group order n of ECgFp5), the public key is a Point on ECgFp5, and the signature is 80 bytes.

---

## Section 1: Poseidon2 Rust Crates

### 1.1 `p3-poseidon2` + `p3-goldilocks` (Plonky3)

**The most production-grade option for pure Poseidon2 over Goldilocks.**

| Property | Value |
|----------|-------|
| Crate(s) | `p3-poseidon2`, `p3-goldilocks` |
| Latest version | 0.5.0 (as of early 2026) |
| Field | Goldilocks (p = 2^64 - 2^32 + 1), also BabyBear, KoalaBear, Mersenne31 |
| Poseidon2 specifically? | Yes — implements Poseidon2, NOT original Poseidon |
| Maintained by | Polygon / Plonky3 team, actively developed |
| License | MIT or Apache-2.0 |

**Key types in `p3-goldilocks`:**

```rust
// The Goldilocks field element
pub struct Goldilocks;   // GF(2^64 - 2^32 + 1)

// Pre-built Poseidon2 type alias
pub type Poseidon2Goldilocks<const WIDTH: usize>;

// Convenience constructors (from docs)
pub fn default_goldilocks_poseidon2_8()  -> Poseidon2Goldilocks<8>
pub fn default_goldilocks_poseidon2_12() -> Poseidon2Goldilocks<12>

// Internal layer structs
pub struct Poseidon2ExternalLayerGoldilocks;
pub struct Poseidon2InternalLayerGoldilocks;
pub struct MdsMatrixGoldilocks;
pub struct GenericPoseidon2LinearLayersGoldilocks;
```

**Usage pattern:**

```rust
use p3_goldilocks::{Goldilocks, default_goldilocks_poseidon2_8};
use p3_poseidon2::Poseidon2;

let hasher = default_goldilocks_poseidon2_8();
// hasher acts on [Goldilocks; 8] state arrays
// Wrap in a sponge via p3_symmetric::PaddingFreeSponge
```

**Important caveat:** `p3-poseidon2` provides the *permutation* primitive, not a high-level `hash(bytes)` function. To hash arbitrary data you must compose it with a sponge construction from `p3-symmetric` (e.g., `PaddingFreeSponge` or `TruncatedPermutation`). This is typical for ZK-framework usage but adds boilerplate.

**Notes on compatibility with Lighter:**
The Plonky3 Poseidon2 parameterization may differ from Lighter's Go implementation parameters. Lighter's go implementation uses `poseidon2_goldilocks_plonky2` parameters (Plonky2-compatible), not Plonky3. This distinction matters — the round constants and MDS matrix differ between Plonky2 and Plonky3 variants.

**Dependencies:** Large ZK framework — pulls in `p3-field`, `p3-symmetric`, `p3-matrix`, etc. Not a minimal dep set.

---

### 1.2 `poseidon-hash` (Lighter-native)

**The correct crate for Lighter DEX compatibility.**

| Property | Value |
|----------|-------|
| Crate | `poseidon-hash` |
| Version | 0.1.x (latest ~0.1.3 as of 2025) |
| Field | Goldilocks (p = 2^64 - 2^32 + 1) **only** |
| Poseidon2 specifically? | Yes — Poseidon2 sponge compatible with Plonky2 circuit outputs |
| Ported from | `lighter-go` (official Lighter Protocol Go implementation) |
| Maintained by | Community (SmartCrypto / Bvvvp009 / 0xvasanth) |
| License | Unspecified / open source |
| Audited? | No |

**Key API:**

```toml
[dependencies]
poseidon-hash = "0.1"
```

```rust
// Hash Goldilocks field elements (no padding)
pub fn hash_no_pad(inputs: Vec<Goldilocks>) -> [Goldilocks; 4]

// Hash to quintic extension field (for ECgFp5 public key derivation)
pub fn hash_to_quintic_extension(inputs: Vec<Goldilocks>) -> Fp5Element

// Included Fp5 type
pub struct Fp5Element { ... }  // element of GF(p^5)
```

**Why this is the right one for Lighter:**
- Directly ported from lighter-go, so parameters (round constants, MDS matrix, sponge rate) match
- Specifically includes `Fp5Element` which is required for ECgFp5 operations
- Used by all known Rust Lighter SDKs (`lighter-rs`, `0xvasanth/lighter-rs`)
- `no_std` compatible with `alloc`
- Returns `[Goldilocks; 4]` — standard Goldilocks hash output (4 × 64-bit = 256 bits)

**Limitations:**
- Not audited
- No benchmarks published
- Maintained by community, not a major org
- API is minimal (no streaming/incremental hashing)

---

### 1.3 `p3-poseidon` (Plonky3 original Poseidon)

| Property | Value |
|----------|-------|
| Crate | `p3-poseidon` |
| Field | Goldilocks + others |
| Poseidon2? | No — this is original Poseidon, NOT Poseidon2 |

**Do not use** for Lighter — wrong hash function variant.

---

### 1.4 `light-poseidon` (Light Protocol / Solana)

| Property | Value |
|----------|-------|
| Crate | `light-poseidon` |
| Field | BN254 (not Goldilocks) |
| Poseidon2? | No — original Poseidon, Circom-compatible |
| Audited? | Yes (Veridise) |

**Not applicable** for Lighter. BN254 field, wrong hash variant.

---

### 1.5 `taceo-poseidon2` (TACEO Labs)

| Property | Value |
|----------|-------|
| Crate | `taceo-poseidon2` |
| Field | BN254 only |
| Poseidon2? | Yes — Poseidon2 permutation |
| Based on | HorizenLabs reference implementation |

**Not applicable** for Lighter — BN254 field only.

---

### 1.6 `neptune` (Filecoin / Lurk Lab)

| Property | Value |
|----------|-------|
| Crate | `neptune` |
| Field | BLS12-381, Pallas/Vesta |
| Poseidon2? | No — original Poseidon |
| Audited? | Yes (ADBK Consulting) |

**Not applicable** — wrong field and wrong hash variant.

---

### 1.7 HorizenLabs `poseidon2` (Reference)

| Property | Value |
|----------|-------|
| Repo | `github.com/HorizenLabs/poseidon2` |
| Published crate? | No (not on crates.io) |
| Fields | BN254, BLS12-381, and Goldilocks (POSEIDON2_GOLDILOCKS_8_PARAMS, _12_PARAMS, _16_PARAMS, _20_PARAMS) |
| Poseidon2? | Yes — this is the canonical reference implementation from the original paper authors |

**Goldilocks parameters defined:** widths 8, 12, 16, 20

This is the reference but is **not a published crate**. Would require vendoring. Used as base by `taceo-poseidon2` (BN254 only).

---

### 1.8 Summary: Poseidon2 Crates

| Crate | Field | Poseidon2? | Goldilocks? | Published? | Audited? | Lighter-compatible? |
|-------|-------|-----------|-------------|------------|----------|---------------------|
| `p3-poseidon2` + `p3-goldilocks` | Goldilocks + more | Yes | Yes | Yes | Indirectly (Polygon) | Likely, but params may differ |
| `poseidon-hash` | Goldilocks only | Yes | Yes | Yes | No | Yes — ported from lighter-go |
| `p3-poseidon` | Goldilocks + more | No (Poseidon v1) | Yes | Yes | No | No |
| `light-poseidon` | BN254 | No | No | Yes | Yes | No |
| `taceo-poseidon2` | BN254 | Yes | No | Yes | No | No |
| `neptune` | BLS12-381 | No | No | Yes | Yes | No |
| HorizenLabs/poseidon2 | Multiple incl. Goldilocks | Yes | Yes | No | No | Possibly |

**Recommendation:** Use `poseidon-hash = "0.1"` for Lighter compatibility.

---

## Section 2: ECgFp5 Implementations

### 2.1 `pornin/ecgfp5` (Reference Implementation — NOT a crate)

**The canonical source, by the original author Thomas Pornin (NCC Group).**

| Property | Value |
|----------|-------|
| Repo | `github.com/pornin/ecgfp5` |
| Published on crates.io? | No |
| Language | C, Python, and Rust (in `rust/` subdirectory) |
| Field | GF(p^5) where p = 2^64 - 2^32 + 1 |
| Schnorr included? | No (by design — hash function choice is left to the caller) |
| `no_std`? | Yes (uses `core` only, no `std`) |
| Unsafe? | No — pure safe Rust |

**Rust API:**

```rust
// Field types
pub struct GFp;      // Element of GF(p = 2^64 - 2^32 + 1)
pub struct GFp5;     // Element of GF(p^5)

// Curve types
pub struct Point;    // ECgFp5 group element
pub struct Scalar;   // Integer mod group order n

// Key operations
impl Point {
    pub fn mulgen(s: &Scalar) -> Point;  // pubkey = s * G
    pub fn encode(&self) -> [u8; 40];    // compress to 40 bytes
    pub fn verify_muladd_vartime(r: &GFp5, s: &Scalar, p: &Point) -> bool; // verify: s*G + r*P == R
}

impl Scalar {
    pub fn decode(src: &[u8]) -> Option<Scalar>;
    pub fn decode_reduce(src: &[u8]) -> Scalar;  // reduce mod n
    pub fn encode(&self) -> [u8; 40];
}
```

**Schnorr signature construction (manual, using these primitives):**

```
Sign(privkey: Scalar, msg: [GFp; N]):
  1. k = random_scalar()
  2. R = k * G  (Point)
  3. e = Poseidon2_hash(R.encode() || pubkey.encode() || msg)  → Scalar
  4. s = k - e * privkey  (mod n)
  5. sig = (e.encode(), s.encode())  (total 80 bytes)

Verify(pubkey: Point, msg, sig=(e,s)):
  1. R' = s*G + e*pubkey
  2. e' = Poseidon2_hash(R'.encode() || pubkey.encode() || msg)
  3. Accept if e' == e
```

**Why Schnorr is NOT included:**
The README explicitly states: "Defining a signature scheme entails choosing a hash function, and it is expected that in-VM implementations will want to use a specialized hash function." This is correct — you must wire Poseidon2 yourself.

**Usage in practice:** Most Lighter Rust SDKs vendor or re-implement this code internally (e.g., `robustfengbin/lighter-sdk` has its own `ecgfp5.rs`).

---

### 2.2 `plonky2_ecgfp5` (Plonky2 SNARK gadgets)

| Property | Value |
|----------|-------|
| Crate | `plonky2_ecgfp5` |
| Version | 0.1.1 (last updated February 2023) |
| Purpose | Plonky2 SNARK circuit gadgets for EcGFp5 |
| Out-of-circuit impl? | Yes (but prototype) |
| Schnorr? | Not explicitly, but "efficient verification of signatures" mentioned |
| Production ready? | Explicitly NOT — "prototype, not constant-time, DO NOT USE IN PRODUCTION" |
| Maintained? | Abandoned (last update 2023) |

**Not suitable** for production signing — out of date and explicitly a prototype.

---

### 2.3 `goldilocks-crypto` (Lighter-native combined crate)

**The most complete ready-to-use option for Lighter DEX signing.**

| Property | Value |
|----------|-------|
| Crate | `goldilocks-crypto` |
| Version | 0.1.2 |
| Field | GF(p^5) over Goldilocks |
| Includes | ECgFp5 + Schnorr + Scalar operations |
| Depends on | `poseidon-hash` (for Poseidon2 + field arithmetic) |
| Ported from | `lighter-go` (official Lighter Protocol Go implementation) |
| Audited? | No — "NOT been audited, provided as-is" |
| `no_std`? | Yes (with `alloc`) |

**API surface:**

```toml
[dependencies]
goldilocks-crypto = "0.1"
```

```rust
use goldilocks_crypto::schnorr::{sign, verify_signature};
use goldilocks_crypto::keypair::KeyPair;
use goldilocks_crypto::signature::Signature;
use goldilocks_crypto::scalar_field::ScalarField;

// Generate or load keypair
let keypair = KeyPair::from_private_key_bytes(&privkey_bytes)?;

// Sign
let sig: Signature = sign(&privkey_bytes, &message)?;
// sign_with_nonce allows deterministic signing
let sig2: Signature = sign_with_nonce(&privkey_bytes, &message, &nonce)?;

// Verify
verify_signature(&signature_bytes, &message, &pubkey_bytes)?;

// Validate a public key point
validate_public_key(&pubkey_bytes)?;
```

**Signature format:** 80 bytes (40 bytes for `e` scalar + 40 bytes for `s` scalar, both GFp5-encoded)

**Batch verification:** `batch_verify` module for efficient multi-signature verification.

**Internal structure:**

```
goldilocks-crypto/
├── schnorr.rs       (sign, verify, sign_with_nonce)
├── keypair.rs       (KeyPair struct)
├── scalar_field.rs  (ScalarField = private key)
├── signature.rs     (Signature = 80-byte wrapper)
└── batch_verify.rs  (batch verification)

depends on poseidon-hash:
└── poseidon-hash/
    ├── goldilocks.rs    (GFp field arithmetic)
    ├── fp5.rs           (GFp5 quintic extension)
    └── poseidon2.rs     (hash function)
```

---

## Section 3: Alternative — Inline Cryptography in lighter-sdk

`robustfengbin/lighter-sdk` chose a different approach: implement all cryptographic primitives **inline** with zero external cryptographic dependencies.

**Structure:**

```
src/crypto/
├── goldilocks.rs   (GFp field arithmetic)
├── gfp5.rs         (quintic extension field)
├── poseidon2.rs    (Poseidon2 hash)
├── ecgfp5.rs       (elliptic curve operations)
└── schnorr.rs      (Schnorr sign + verify)
```

**Advantages:**
- No external crypto deps beyond `tokio`/`reqwest`
- Self-contained, auditable
- No version conflicts

**Disadvantages:**
- More code to maintain
- No independent validation
- Same audit status (none)

---

## Section 4: Schnorr over ECgFp5 vs Standard Schnorr

### Differences from ed25519/secp256k1 Schnorr

| Property | Standard Schnorr (secp256k1 / ed25519) | ECgFp5 Schnorr (Lighter) |
|----------|-----------------------------------------|--------------------------|
| Curve field | 256-bit prime (secp256k1) or 255-bit (ed25519) | GF(p^5) where p = 2^64-2^32+1 |
| Hash function | SHA-256 / SHA-512 | Poseidon2 over Goldilocks |
| Signature size | 64 bytes (ed25519) | 80 bytes (2 × GFp5 elements) |
| Key size | 32 bytes (ed25519) | 40 bytes (1 × GFp5 element) |
| ZK-friendly? | No (SHA-2 is expensive in circuits) | Yes (Poseidon2 is native in STARK circuits) |
| Nonce generation | RFC 6979 deterministic | Poseidon2-based deterministic from key+msg |
| Public key | 32 bytes compressed | 40 bytes (GFp5 point encoding) |

### Why ECgFp5?

ECgFp5 was designed specifically for Miden VM and similar STARK-based systems where arithmetic over GF(p) is native. Verifying an ECgFp5 Schnorr signature inside a STARK circuit costs ~10× fewer constraints than verifying ed25519. Lighter inherits this design for its ZK proofs.

### Can generic Schnorr libraries be adapted?

**No, not directly.** Generic Schnorr libraries (e.g., `RustCrypto/signatures`, `ZcashFoundation/frost`) operate on standard elliptic curves via the `elliptic-curve` crate traits. ECgFp5 does not implement these traits (it uses a custom GFp5 base field). The signing logic is simple enough (20 lines) that re-implementation is preferable to adaptation.

---

## Section 5: Related Crates Not Applicable

| Crate | Reason Not Applicable |
|-------|----------------------|
| `poseidon-rs` | Original Poseidon, BN128 field |
| `poseidon-merkle` | Starknet (felt252 field, different params) |
| `halo2_poseidon` | Halo2/BN254 circuits |
| `poseidon252` | Dusk Network, BN254 |
| `ed448-goldilocks` | Ed448-Goldilocks curve (different meaning of "Goldilocks" — it refers to Curve448, not the Goldilocks prime) |
| `starknet-crypto` | StarkNet felt252 field |
| `miden-crypto` (RPO/RPX) | Different hash function (Rescue Prime Optimized, not Poseidon2) — though runs on same Goldilocks field |

**Note on `ed448-goldilocks`:** This crate is for the Ed448 elliptic curve, which happens to be nicknamed "Goldilocks" due to its cofactor of 4, but operates over a different field (p = 2^448 - 2^224 - 1). It is **completely unrelated** to the Goldilocks prime used by Lighter.

---

## Section 6: Feasibility Assessment

### Option A: Use `goldilocks-crypto` + `poseidon-hash`

**Recommended approach.**

```toml
[dependencies]
goldilocks-crypto = "0.1"  # ECgFp5 + Schnorr + Poseidon2
poseidon-hash = "0.1"      # included transitively, but may want explicit dep
```

**Pros:**
- Complete stack in two small crates
- Ported from official lighter-go, parameter-compatible
- `no_std` compatible
- Minimal dependencies (no heavy ZK framework)
- 41 unit tests in dependent SDKs validate against mainnet

**Cons:**
- Not audited
- Version 0.1.x — API may change
- Maintained by community, not major org
- Single-author projects (risk of abandonment)

**Verdict: Viable for trading bot usage, not for custodial wallets.**

---

### Option B: Vendor `pornin/ecgfp5` + `poseidon-hash`

Use Thomas Pornin's reference Rust code (vendored, not from crates.io) for ECgFp5 arithmetic, and wire it to `poseidon-hash` for hashing.

```
vendor/ecgfp5/   (copy of github.com/pornin/ecgfp5/rust/)
poseidon-hash = "0.1"
```

**Pros:**
- Most trusted ECgFp5 implementation (by the inventor)
- Constant-time, no unsafe code
- Can inspect and audit directly

**Cons:**
- Manual Schnorr wiring required (~40 lines)
- Vendor maintenance burden
- Still depends on `poseidon-hash` for correct Lighter-compatible params

---

### Option C: Use `p3-poseidon2` + `p3-goldilocks` + vendored ECgFp5

Use Plonky3 for the hash (higher quality, more maintained) and vendor ECgFp5.

**Pros:**
- Plonky3 is production-quality, widely used
- Active Polygon maintenance

**Cons:**
- Plonky3's Poseidon2 parameters (Plonky3 variant) **may not match** Lighter's expected parameters (Plonky2 variant)
- Heavy dependency tree from `p3-*` crates
- Requires verifying parameter compatibility against lighter-go

**Verdict: Risky without parameter verification. Not recommended without comparison testing.**

---

### Option D: Implement from scratch

Given the simplicity of the components, full from-scratch is feasible:

| Component | Lines of code estimate | Complexity |
|-----------|------------------------|------------|
| GFp arithmetic | ~200 | Low (modular arithmetic) |
| GFp5 arithmetic | ~300 | Medium (extension field tower) |
| Poseidon2 permutation | ~150 | Low (fixed round constants) |
| ECgFp5 point arithmetic | ~400 | Medium-high (Weierstrass formulas) |
| Schnorr sign/verify | ~50 | Low (once primitives exist) |
| **Total** | **~1100** | |

**Cons:**
- High risk of subtle bugs (especially in GFp5 arithmetic and point encoding)
- Round constant generation must match Lighter exactly
- Not recommended unless doing a full security audit

---

## Section 7: Recommended Minimal Dependency Set

For integrating Lighter DEX signing into `digdigdig3`:

```toml
[dependencies]
# Option A (recommended — Lighter-native, minimal deps):
goldilocks-crypto = "0.1"

# Option B (reference-quality, more control):
# vendor: pornin/ecgfp5/rust/ into local crate
# poseidon-hash = "0.1"
```

**Cargo.toml transitive tree (Option A):**

```
goldilocks-crypto 0.1.2
└── poseidon-hash 0.1.x
    (no further crypto deps — pure Rust, no alloc beyond core)
```

This is a very lean dependency footprint compared to pulling in Plonky2 or Plonky3.

---

## Section 8: Open Questions

1. **Parameter compatibility:** Are the round constants in `poseidon-hash` identical to the ones in `lighter-go`? Needs verification by running `hash_no_pad([0, 1, 2, 3])` against the Go output.

2. **Plonky2 vs Plonky3 variant:** Lighter uses Plonky2-compatible Poseidon2 parameters. The `p3-poseidon2` crate uses Plonky3 parameters. These are DIFFERENT. Confirm before using `p3-poseidon2`.

3. **Schnorr nonce generation:** Does Lighter use deterministic nonces (RFC-style: k = Hash(privkey, msg))? If yes, what exact construction? This affects signing reproducibility and safety.

4. **Public key encoding format:** Is the 40-byte encoding a compressed GFp5 point (x-coordinate only) or uncompressed (x, y)? Matters for interoperability with the API.

5. **Message format:** What exactly is hashed? Raw transaction bytes, or structured field elements? The `poseidon-hash::hash_no_pad` takes `Vec<Goldilocks>` — bytes must be converted to field elements first.

---

## Sources

- [p3-poseidon2 on crates.io](https://crates.io/crates/p3-poseidon2)
- [Poseidon2Goldilocks in p3-goldilocks docs](https://docs.rs/p3-goldilocks/latest/p3_goldilocks/type.Poseidon2Goldilocks.html)
- [p3-goldilocks module docs](https://docs.rs/p3-goldilocks/latest/p3_goldilocks/)
- [Plonky3 GitHub repository](https://github.com/Plonky3/Plonky3)
- [poseidon-hash on crates.io](https://crates.io/crates/poseidon-hash)
- [goldilocks-crypto on crates.io](https://crates.io/crates/goldilocks-crypto)
- [goldilocks-crypto docs.rs](https://docs.rs/goldilocks-crypto/latest/goldilocks_crypto/)
- [pornin/ecgfp5 GitHub (reference implementation)](https://github.com/pornin/ecgfp5)
- [EcGFp5 paper by Thomas Pornin](https://eprint.iacr.org/2022/274.pdf)
- [Poseidon2 paper (HorizenLabs)](https://eprint.iacr.org/2023/323.pdf)
- [HorizenLabs/poseidon2 GitHub (reference)](https://github.com/HorizenLabs/poseidon2)
- [plonky2_ecgfp5 on lib.rs](https://lib.rs/crates/plonky2_ecgfp5)
- [robustfengbin/lighter-sdk GitHub](https://github.com/robustfengbin/lighter-sdk)
- [Bvvvp009/lighter-rust GitHub](https://github.com/Bvvvp009/lighter-rust)
- [0xvasanth/lighter-rs GitHub](https://github.com/0xvasanth/lighter-rs)
- [lighter-rs on crates.io](https://crates.io/crates/lighter-rs)
- [The Poseidon2 in Plonky3 (HackMD)](https://hackmd.io/@sin7y/r1VOOG8bR)
- [light-poseidon on crates.io](https://crates.io/crates/light-poseidon)
- [neptune on crates.io](https://crates.io/crates/neptune)
- [miden-crypto on crates.io](https://crates.io/crates/miden-crypto)
- [Elliptic Curves over Goldilocks (HackMD)](https://hackmd.io/@Wimet/S1R3RAY5yx)
- [slop-poseidon2 on lib.rs](https://lib.rs/crates/slop-poseidon2)
- [SmartCrypto Twitter — Rust signer announcement](https://x.com/smartcrypto0/status/1986700296350060615)
