# Lighter DEX: ECgFp5 + Poseidon2 Signing — Complete Technical Reference

Research date: 2026-03-15
Target: Port Lighter transaction signing to pure Rust

---

## Table of Contents

1. [Overview](#overview)
2. [Cryptographic Primitives Stack](#cryptographic-primitives-stack)
3. [Goldilocks Field (GF(p))](#goldilocks-field-gfp)
4. [GFp5 — Quintic Extension Field](#gfp5--quintic-extension-field)
5. [ECgFp5 Elliptic Curve](#ecgfp5-elliptic-curve)
6. [ECgFp5 Scalar Field](#ecgfp5-scalar-field)
7. [Poseidon2 Hash Function](#poseidon2-hash-function)
8. [Schnorr Signature Scheme](#schnorr-signature-scheme)
9. [Transaction Hash Construction](#transaction-hash-construction)
10. [Auth Token Format](#auth-token-format)
11. [Transaction Type Reference](#transaction-type-reference)
12. [API Key and Private Key Format](#api-key-and-private-key-format)
13. [Existing Rust Implementations](#existing-rust-implementations)
14. [Porting Guide — Step by Step](#porting-guide--step-by-step)
15. [Sources](#sources)

---

## Overview

Lighter is an application-specific zk-rollup on Ethereum for perpetual futures. It uses a custom cryptographic stack for L2 transaction signing that is intentionally different from Ethereum's secp256k1/keccak256:

- **Curve**: ECgFp5 (elliptic curve over the quintic extension of the Goldilocks field)
- **Hash**: Poseidon2 over GF(2^64 - 2^32 + 1), WIDTH=12, the "Plonky2 Goldilocks" variant
- **Signatures**: Schnorr (s, e) over ECgFp5 scalar field
- **Key size**: 40 bytes private key, 40 bytes public key (both are GFp5 elements in little-endian)
- **Signature size**: 80 bytes (40 bytes S + 40 bytes E, little-endian)

The canonical reference implementation is the Go library at `github.com/elliottech/poseidon_crypto`, which is what the official `lighter-go` and `lighter-python` SDKs call via FFI.

---

## Cryptographic Primitives Stack

```
Transaction Fields
       |
       v  (field element encoding per type)
Goldilocks field elements []GoldilocksField
       |
       v  Poseidon2(WIDTH=12, RATE=8, 4+22+4 rounds, S-box=x^7)
GFp5 quintic extension element (5 × u64 = 40 bytes) = message hash
       |
       v  Schnorr over ECgFp5
Signature { S: ECgFp5Scalar [40 bytes], E: ECgFp5Scalar [40 bytes] }
       |
       v  concatenate S || E (little-endian)
80-byte signature
```

---

## Goldilocks Field (GF(p))

### Modulus

```
p = 2^64 - 2^32 + 1 = 0xFFFFFFFF00000001 = 18446744069414584321
```

### Montgomery Form

The reference Go implementation uses **Montgomery form** with `R = 2^64`:

- `R^2 mod p = 18446744065119617025`
- `q_inv_neg = -p^{-1} mod 2^64 = 18446744069414584319`
- `MONT_ONE = R mod p = 2^32 - 1 = 4294967295`

### Element Encoding

Each `GoldilocksField` element serializes to **8 bytes, little-endian**, after converting from Montgomery form back to canonical form (`x * R^{-1} mod p`).

### Field Element Construction from Transaction Fields

The Go SDK provides these constructors in `poseidon_crypto/field/goldilocks`:

```go
g.FromUint32(v uint32)  // v as u64, then into Montgomery
g.FromUint64(v uint64)  // v as u64, into Montgomery
g.FromInt64(v int64)    // v as u64 (wrapping cast), into Montgomery
```

For Rust port: encode integers directly as `u64` (no sign extension for int64 — treat as bit pattern), then multiply by `R^2 mod p` for Montgomery form.

---

## GFp5 — Quintic Extension Field

### Definition

```
GFp5 = GF(p)[x] / (x^5 - 3)
```

An element is a degree-4 polynomial: `a[0] + a[1]*x + a[2]*x^2 + a[3]*x^3 + a[4]*x^4`

where each `a[i]` is a `GoldilocksField` and the reduction polynomial is `x^5 = 3` (i.e., `W = 3`).

### Multiplication (polynomial mod x^5 - 3)

```
c[0] = a[0]*b[0] + 3*(a[1]*b[4] + a[2]*b[3] + a[3]*b[2] + a[4]*b[1])
c[1] = a[0]*b[1] + a[1]*b[0] + 3*(a[2]*b[4] + a[3]*b[3] + a[4]*b[2])
c[2] = a[0]*b[2] + a[1]*b[1] + a[2]*b[0] + 3*(a[3]*b[4] + a[4]*b[3])
c[3] = a[0]*b[3] + a[1]*b[2] + a[2]*b[1] + a[3]*b[0] + 3*a[4]*b[4]
c[4] = a[0]*b[4] + a[1]*b[3] + a[2]*b[2] + a[3]*b[1] + a[4]*b[0]
```

### Serialization

40 bytes: `a[0] || a[1] || a[2] || a[3] || a[4]` each 8 bytes little-endian.

### Frobenius / DTH_ROOT

- `DTH_ROOT = 1041288259238279555` (base field constant, used for Frobenius endomorphism)
- Frobenius: `Frob(a)[i] = a[i] * DTH_ROOT^i`

---

## ECgFp5 Elliptic Curve

### Curve Equation

Montgomery-form (used internally for the group law):

```
y^2 = x * (x^2 + a*x + b)
```

Parameters:
```
a = 2  (as GFp5 base element: [2, 0, 0, 0, 0])
b = 263*i  (i = the GFp5 generator: [0, 263, 0, 0, 0])
```

### Point Representations

Two internal representations are used:

**ECgFp5Point (fractional XU coordinates)**
Projective coordinates `(X/Z, U/T)` where `U = X/Y`:
```
{ x: GFp5, z: GFp5, u: GFp5, t: GFp5 }
```
Used for efficient scalar multiplication (windowed method, 5-bit window, 64 windows = 320 doublings).

**WeierstrassPoint (standard projective)**
Standard `(X/Z, Y/Z)`:
```
{ X: GFp5, Y: GFp5, Z: GFp5, IsInf: bool }
```
Used for `MulAdd2` in signature verification.

### Generator Point (ECgFp5Point fractional coords)

```go
GENERATOR_ECgFp5Point = ECgFp5Point{
    x: GFp5{12883135586176881569, 4356519642755055268, 5248930565894896907, 2165973894480315022, 2448410071095648785},
    z: GFp5_ONE,   // [1,0,0,0,0]
    u: GFp5_ONE,   // [1,0,0,0,0]
    t: GFp5{4, 0, 0, 0, 0},
}
```

NOTE: These are the raw `u64` values (already in Montgomery form as stored internally in the Go lib).

### Generator Point (WeierstrassPoint)

```go
GENERATOR_WEIERSTRASS = WeierstrassPoint{
    X: GFp5{11712523173042564207, 14090224426659529053, ...},
    Y: GFp5{14639054205878357578, 17426078571020221072, ...},
    IsInf: false
}
```

### Point Encoding

A point is encoded as a single `GFp5` element (40 bytes) using the `x`-coordinate with sign disambiguation via the `u = x/y` coordinate. The `Encode()` function returns `GFp5` directly.

### Key Derivation

```
PublicKey = sk * GENERATOR_ECgFp5Point
encoded_pubkey = PublicKey.Encode()  // GFp5, 40 bytes
```

---

## ECgFp5 Scalar Field

### Group Order (n)

```
n = 1067993516717146951041484916571792702745057740581727230159139685185762082554198619328292418486241
```

This is a 319-bit prime (the order of the ECgFp5 group).

In hex:
```
n = 0x33...  (see full value below)
```

Decimal breakdown: `~2^319.0`

### Scalar Representation

5 limbs of `u64` (little-endian), Montgomery form, 40 bytes total.

### `ScalarElementFromLittleEndianBytes`

Input: 40-byte little-endian encoding of an integer `< n`.
The Go implementation checks canonicality (value < n) during deserialization.

### `FromGfp5` conversion

Takes a `GFp5` element (5 × u64) and reduces modulo n to produce an `ECgFp5Scalar`. This is used to convert the Poseidon2 hash output into a scalar challenge `e`.

---

## Poseidon2 Hash Function

### Parameters (the "Plonky2 Goldilocks" variant)

| Parameter | Value |
|-----------|-------|
| Field | GF(2^64 - 2^32 + 1) |
| Width (t) | 12 |
| Rate | 8 |
| Capacity | 4 |
| Full rounds (Rf/2) | 4 start + 4 end = 8 total |
| Partial rounds (Rp) | 22 |
| S-box | x^7 |

Import path in Go: `github.com/elliottech/poseidon_crypto/hash/poseidon2_goldilocks_plonky2`
(NOTE: there is also `poseidon2_goldilocks` — verify which one lighter-go uses. The signer uses `poseidon2_goldilocks`)

### Round Constants

**External round constants** (8 rounds × 12 elements each):

```rust
const EXTERNAL_CONSTANTS: [[u64; 12]; 8] = [
    [15492826721047263190, 11728330187201910315, 8836021247773420868, 16777404051263952451,
     5510875212538051896,  6173089941271892285,  2927757366422211339, 10340958981325008808,
     8541987352684552425,  9739599543776434497, 15073950188101532019, 12084856431752384512],
    [4584713381960671270,  8807052963476652830,   54136601502601741,  4872702333905478703,
     5551030319979516287, 12889366755535460989, 16329242193178844328,   412018088475211848,
    10505784623379650541,  9758812378619434837,  7421979329386275117,   375240370024755551],
    [3331431125640721931, 15684937309956309981,   578521833432107983, 14379242000670861838,
    17922409828154900976,  8153494278429192257, 15904673920630731971, 11217863998460634216,
     3301540195510742136,  9937973023749922003,  3059102938155026419,  1895288289490976132],
    [5580912693628927540, 10064804080494788323,  9582481583369602410, 10186259561546797986,
      247426333829703916, 13193193905461376067,  6386232593701758044, 17954717245501896472,
     1531720443376282699,  2455761864255501970, 11234429217864304495,  4746959618548874102],
    [13571697342473846203, 17477857865056504753, 15963032953523553760, 16033593225279635898,
    14252634232868282405,  8219748254835277737,  7459165569491914711, 15855939513193752003,
    16788866461340278896,  7102224659693946577,  3024718005636976471, 13695468978618890430],
    [8214202050877825436,  2670727992739346204, 16259532062589659211, 11869922396257088411,
     3179482916972760137, 13525476046633427808,  3217337278042947412, 14494689598654046340,
    15837379330312175383,  8029037639801151344,  2153456285263517937,  8301106462311849241],
    [13294194396455217955, 17394768489610594315, 12847609130464867455, 14015739446356528640,
     5879251655839607853,  9747000124977436185,  8950393546890284269, 10765765936405694368,
    14695323910334139959, 16366254691123000864, 15292774414889043182, 10910394433429313384],
    [17253424460214596184,  3442854447664030446,  3005570425335613727, 10859158614900201063,
     9763230642109343539,  6647722546511515039,   909012944955815706, 18101204076790399111,
    11588128829349125809, 15863878496612806566,  5201119062417750399,   176665553780565743],
];
```

**Internal round constants** (22 rounds, only applied to state[0]):

```rust
const INTERNAL_CONSTANTS: [u64; 22] = [
    11921381764981422944, 10318423381711320787,  8291411502347000766,   229948027109387563,
     9152521390190983261,  7129306032690285515, 15395989607365232011,  8641397269074305925,
    17256848792241043600,  6046475228902245682, 12041608676381094092, 12785542378683951657,
    14546032085337914034,  3304199118235116851, 16499627707072547655, 10386478025625759321,
    13475579315436919170, 16042710511297532028,  1411266850385657080,  9024840976168649958,
    14047056970978379368,   838728605080212101,
];
```

**Internal diagonal matrix** (used in partial rounds linear layer):

```rust
const MATRIX_DIAG_12: [u64; 12] = [
    0xc3b6c08e23ba9300, 0xd84b5de94a324fb6, 0x0d0c371c5b35b84f, 0x7964f570e7188037,
    0x5daf18bbd996604b, 0x6743bc47b9595257, 0x5528b9362c59bb70, 0xac45e25b7127b68b,
    0xa2077d7dfbb606b5, 0xf3faac6faee378ae, 0x0c6388b51545e883, 0xd27dbb6944917b60,
];
```

NOTE: These diagonal matrix values are stored in Montgomery form internally in the Go lib. When porting to Rust, verify whether to use them as-is or convert.

### Permutation Structure

```
permute(state):
  external_linear_layer(state)    // initial MDS mix
  full_rounds(state, 0..4)        // 4 full rounds
  partial_rounds(state, 0..22)    // 22 partial rounds (S-box only on state[0])
  full_rounds(state, 4..8)        // 4 full rounds
```

**Full round:**
```
add_external_rc(state, round)  // add round constants to all 12 elements
state[i] = state[i]^7          // S-box on all elements
external_linear_layer(state)   // MDS matrix multiplication
```

**Partial round:**
```
state[0] += INTERNAL_CONSTANTS[round]
state[0] = state[0]^7          // S-box only on first element
internal_linear_layer(state)   // diagonal matrix + sum
```

**External linear layer:**
```
// Process in 3 groups of 4:
for each group [a,b,c,d]:
    t0 = a+b; t1 = c+d; t2 = t0+t1; t3 = t2+b; t4 = t2+d; t5=2*a; t6=2*c
    a' = t3+t0; b' = t6+t3; c' = t1+t4; d' = t5+t4
// Cross-group mixing: each element += sum of its column position across groups
```

**Internal linear layer:**
```
sum = state[0] + state[1] + ... + state[11]
state[i] = state[i] * MATRIX_DIAG_12[i] + sum
```

### Hash to Quintic Extension

The primary function used for signing:

```go
func HashToQuinticExtension(input []GoldilocksField) GFp5 {
    // Sponge: absorb input in chunks of RATE=8, squeeze 5 elements
    state := [12]GoldilocksField{}
    for chunk in input.chunks(8) {
        state[..chunk.len()] = chunk
        permute(&state)
    }
    // Squeeze: return state[0..5]
    return GFp5{state[0], state[1], state[2], state[3], state[4]}
}
```

**IMPORTANT**: The sponge absorbs one chunk, calls permute, then absorbs the next chunk (no XOR accumulation across the state — each absorption overwrites state[0..8]).

### Test Vectors

Permutation test vector (from Go `TestPermute`):

Input:
```
[5417613058500526590, 2481548824842427254, 6473243198879784792, 1720313757066167274,
 2806320291675974571, 7407976414706455446, 1105257841424046885, 7613435757403328049,
 3376066686066811538, 5888575799323675710, 6689309723188675948, 2468250420241012720]
```

Expected output:
```
[5364184781011389007, 15309475861242939136, 5983386513087443499,  886942118604446276,
14903657885227062600,  7742650891575941298,  1962182278500985790, 10213480816595178755,
 3510799061817443836,  4610029967627506430,  7566382334276534836,  2288460879362380348]
```

HashToQuinticExtension test vector (from Go `TestHashToQuinticExtension`):

Input (7 elements):
```
[3451004116618606032, 11263134342958518251, 10957204882857370932, 5369763041201481933,
 7695734348563036858,  1393419330378128434,  7387917082382606332]
```

Expected output (GFp5, 5 elements):
```
[17992684813643984528, 5243896189906434327, 7705560276311184368, 2785244775876017560, 14449776097783372302]
```

---

## Schnorr Signature Scheme

### Algorithm

Based on the reference implementation in `elliottech/poseidon_crypto/signature/schnorr/schnorr.go`:

**Key Generation:**
```
private_key sk: ECgFp5Scalar (40 bytes, uniform random < n)
public_key  pk: GFp5 = GENERATOR.mul(sk).encode()  // 40 bytes
```

**Signing a pre-hashed message:**

```
Input:  hashed_msg: GFp5  (40 bytes, the Poseidon2 hash of the transaction)
        sk: ECgFp5Scalar

1. k  = random ECgFp5Scalar  (uniform < n)
2. r  = GENERATOR.mul(k).encode()  // GFp5 (40 bytes)
3. preimage = r[0..5] ++ hashed_msg[0..5]  // 10 GoldilocksField elements
4. e_gfp5 = Poseidon2.HashToQuinticExtension(preimage)  // GFp5
5. e = ECgFp5Scalar::from_gfp5(e_gfp5)  // reduce mod n
6. s = k - e * sk  (mod n)
7. return Signature { S: s, E: e }
```

**Verification:**
```
Input:  pk: GFp5
        hashed_msg: GFp5
        sig: Signature { S, E }

1. Check sig.S and sig.E are canonical (< n)
2. pk_ws = WeierstrassPoint::decode(pk)
3. r_v = MulAdd2(GENERATOR_WEIERSTRASS, pk_ws, sig.S, sig.E).encode()
   // r_v = S*G + E*pk
4. preimage = r_v[0..5] ++ hashed_msg[0..5]
5. e_v = Schnorr.from_gfp5(HashToQuinticExtension(preimage))
6. return e_v == sig.E
```

### Signature Serialization

```
80 bytes: S_le_bytes[0..40] || E_le_bytes[0..40]
```

Both `S` and `E` are 40-byte little-endian representations of their 5-limb Montgomery scalars.

### Security Properties

- ECgFp5 has **prime order** (no cofactor) — no small subgroup attacks
- No cofactor clearing needed
- Canonical encoding prevents malleability
- All decoded points are valid group elements

---

## Transaction Hash Construction

### General Pattern

Every transaction type follows:

```go
func (tx *L2SomeTxInfo) Hash(lighterChainId uint32) ([]byte, error) {
    elems := []GoldilocksField{}

    // Standard prefix (always first 4 fields):
    elems = append(elems, g.FromUint32(lighterChainId))
    elems = append(elems, g.FromUint32(TX_TYPE_CONSTANT))
    elems = append(elems, g.FromInt64(tx.Nonce))
    elems = append(elems, g.FromInt64(tx.ExpiredAt))

    // Transaction-specific fields...

    txHash := p2.HashToQuinticExtension(elems)  // GFp5
    return AggregateTxHash(txHash)  // applies integrator fee hash if present
}
```

The returned `[]byte` is the `GFp5` element in little-endian (40 bytes). This is passed directly to `schnorr_sign_hashed_message`.

### Chain IDs

```
Mainnet chain_id = 304
Testnet chain_id = 300
```

Source: Python SDK `signer_client.py`:
```python
self.chain_id = 304 if ("mainnet" in url or "api" in url) else 300
```

### Transaction Type Constants

```go
TxTypeEmpty              = 0
// Layer 1
TxTypeL1Deposit          = 1
TxTypeL1Withdraw         = 2
// Layer 2
TxTypeL2Transfer         = 10
TxTypeL2Withdraw         = 11
TxTypeL2CreateOrder      = 12
TxTypeL2CancelOrder      = 13
TxTypeL2ModifyOrder      = 14
TxTypeL2UpdateLeverage   = 15
TxTypeL2UpdateMargin     = 16
TxTypeL2ChangePubKey     = 17
TxTypeL2CancelAllOrders  = 18
TxTypeL2CreateSubAccount = 19
TxTypeL2CreatePublicPool = 20
TxTypeL2MintShares       = 21
TxTypeL2BurnShares       = 22
TxTypeL2StakeAssets      = 23
TxTypeL2UnstakeAssets    = 24
TxTypeL2ApproveIntegrator= 45
```

### CreateOrder Hash Fields

```
[0]  lighterChainId       uint32
[1]  TxTypeL2CreateOrder  uint32  (= 12)
[2]  Nonce                int64
[3]  ExpiredAt            int64
[4]  AccountIndex         int64
[5]  ApiKeyIndex          uint32  (cast from uint8)
[6]  MarketIndex          uint32  (cast from uint16)
[7]  ClientOrderIndex     int64
[8]  BaseAmount           int64
[9]  Price                uint32
[10] IsAsk                uint32  (0 or 1)
[11] Type                 uint32  (order type enum)
[12] TimeInForce          uint32  (IOC=0, GTT=1, PostOnly=2)
[13] ReduceOnly           uint32  (0 or 1)
[14] TriggerPrice         uint32
[15] OrderExpiry          int64
```

Then: `txHash = Poseidon2.HashToQuinticExtension(elems)` — and optionally `AggregateTxHash` if integrator attributes are set.

### CancelOrder Hash Fields

```
[0]  lighterChainId       uint32
[1]  TxTypeL2CancelOrder  uint32  (= 13)
[2]  Nonce                int64
[3]  ExpiredAt            int64
[4]  AccountIndex         int64
[5]  ApiKeyIndex          uint32
[6]  MarketIndex          uint32
[7]  Index                int64   (order index on the book)
```

### Withdraw Hash Fields

```
[0]  lighterChainId       uint32
[1]  TxTypeL2Withdraw     uint32  (= 11)
[2]  Nonce                int64
[3]  ExpiredAt            int64
[4]  FromAccountIndex     int64
[5]  ApiKeyIndex          uint32
[6]  AssetIndex           uint32  (cast from int16)
[7]  RouteType            uint32  (cast from uint8)
[8]  Amount & 0xFFFFFFFF  uint64  (lower 32 bits)
[9]  Amount >> 32         uint64  (upper 32 bits)
```

NOTE: 64-bit Amount is split into two 32-bit Goldilocks elements.

### ChangePubKey Hash Fields

```
[0]  lighterChainId         uint32
[1]  TxTypeL2ChangePubKey   uint32  (= 17)
[2]  Nonce                  int64
[3]  ExpiredAt              int64
[4]  AccountIndex           int64
[5]  ApiKeyIndex            uint32
[6..] pubKey bytes encoded as GoldilocksField elements via ArrayFromCanonicalLittleEndianBytes
```

`ArrayFromCanonicalLittleEndianBytes([]byte)`: splits byte array into 8-byte chunks, each chunk becomes one `GoldilocksField` element (little-endian, must be < p). The public key is 40 bytes = 5 field elements.

### AggregateTxHash (Integrator Fee Extension)

If integrator fee attributes are present, the hash is further combined:

```go
func AggregateTxHash(txHash GFp5) []byte {
    if attr.IsEmpty() {
        return txHash.ToLittleEndianBytes()
    }
    attributesHash := attr.Hash()  // another Poseidon2 hash of fee parameters
    combined := txHash[0..5] ++ attributesHash[0..5]  // 10 elements
    return Poseidon2.HashToQuinticExtension(combined).ToLittleEndianBytes()
}
```

---

## Auth Token Format

Source: `elliottech/lighter-go/types/tx_request.go`, `ConstructAuthToken()`.

### Construction

```go
message := fmt.Sprintf("%v:%v:%v", deadline.Unix(), accountIndex, apiKeyIndex)
// e.g. "1741000000:12345:1"

msgInField := g.ArrayFromCanonicalLittleEndianBytes([]byte(message))
// The ASCII bytes of the message string are split into 8-byte chunks → GoldilocksField elements

msgHash := p2.HashToQuinticExtension(msgInField).ToLittleEndianBytes()
// 40-byte hash

signatureBytes := schnorr.Sign(msgHash, privateKey)
// 80 bytes

signature := hex.Encode(signatureBytes)

token := fmt.Sprintf("%v:%v", message, signature)
// Final format: "{deadline}:{accountIndex}:{apiKeyIndex}:{80-byte-sig-hex}"
```

### Token Format

```
{unix_timestamp}:{account_index}:{api_key_index}:{160-char-hex-signature}
```

Example:
```
1741000000:12345:1:a1b2c3...f0 (160 hex chars = 80 bytes)
```

### Token Lifetime

- Tokens are valid for **8 hours** from creation
- `deadline=0` → creates a token valid for ~7 hours from now
- Server enforces maximum deadline of `now + 8 hours`

### Usage

The token is passed as a query parameter or header in API requests that require authentication without full signing (e.g., read-only account data via WebSocket).

---

## Transaction Type Reference

### Order Types

```go
LimitOrder         = 0
MarketOrder        = 1
StopLossOrder      = 2  // perps only
TakeProfitOrder    = 3  // perps only
StopLossLimitOrder = 4  // perps only
TakeProfitLimitOrder = 5  // perps only
TWAPOrder          = 6
```

### Time In Force

```go
ImmediateOrCancel = 0  // IOC
GoodTillTime      = 1  // GTT
PostOnly          = 2
```

### Market Index Ranges

```go
MinPerpsMarketIndex = 0
MaxPerpsMarketIndex = 254

MinSpotMarketIndex  = 2048
MaxSpotMarketIndex  = 4094
```

### Field Constraints

| Field | Min | Max | Encoding |
|-------|-----|-----|----------|
| AccountIndex | 0 | 281,474,976,710,654 | int64 → GF |
| ApiKeyIndex | 0 | 254 | uint8 → uint32 → GF |
| Nonce | 0 | 2^63-1 | int64 → GF |
| Price | 1 | 2^32-1 | uint32 → GF |
| BaseAmount | 1 | 2^48-1 | int64 → GF |
| ExpiredAt | 0 | 2^48-1 | int64 → GF |
| ClientOrderIndex | 0 | 2^48-1 | int64 → GF (or NilClientOrderIndex = 2^48-1) |

### NilClientOrderIndex

```go
NilClientOrderIndex = MaxClientOrderIndex = (1 << 48) - 1
```

If client does not provide an order index, this sentinel value is used.

---

## API Key and Private Key Format

### Private Key

- 40 bytes, little-endian serialization of an `ECgFp5Scalar` element
- Commonly represented as 80-character hex string
- Must be `< n` (the group order)
- The `keyManager` in Go stores it as `curve.ECgFp5Scalar` and loads via `ScalarElementFromLittleEndianBytes(b)`

### Public Key

- 40 bytes, little-endian serialization of a `GFp5` element (the encoded EC point)
- `pk = GENERATOR.mul(sk).encode()`
- Also 80-char hex string

### API Key Index

- `uint8`, range 0–254
- Reserved: 0 = desktop, 1 = mobile PWA, 2 = mobile app
- User-generated keys: indices 3–254

### Key Generation

The Go shared library exports `GenerateAPIKey()`:

```go
// Returns (privateKey []byte, publicKey []byte)
sk := curve.SampleScalarCrypto()  // random scalar < n
pk := schnorr.SchnorrPkFromSk(sk).ToLittleEndianBytes()
return sk.ToLittleEndianBytes(), pk
```

The `SampleScalarCrypto()` function uses `crypto/rand` to fill 40 bytes and reduces modulo n.

---

## Existing Rust Implementations

### 1. `robustfengbin/lighter-sdk` (Pure Rust, MIT/Apache-2.0)

**URL**: https://github.com/robustfengbin/lighter-sdk

This is the most complete pure-Rust implementation found. It implements all cryptographic primitives from scratch:

```
src/crypto/
├── goldilocks.rs   — GF(2^64 - 2^32 + 1) with Montgomery form
├── gfp5.rs         — Quintic extension field
├── ecgfp5.rs       — Elliptic curve, points, scalars
├── poseidon2.rs    — Poseidon2 hash (WIDTH=12)
└── schnorr.rs      — Schnorr signing/verification
```

Status: Unaudited community port. The test vectors match the Go reference implementation.

### 2. `0xvasanth/lighter-rs` (lighter-rs on crates.io)

**URL**: https://github.com/0xvasanth/lighter-rs
**Crate**: https://crates.io/crates/lighter-rs

Production-focused Rust SDK. Uses `goldilocks-crypto` and `poseidon-hash` crates (ports of the official Go implementation). Claims 41 unit tests and 11+ mainnet transactions verified.

Dependencies:
```toml
goldilocks-crypto = "..."
poseidon-hash = "..."
```

### 3. `goldilocks-crypto` crate

**Docs**: https://docs.rs/goldilocks-crypto/latest/goldilocks_crypto/

Separate Rust crate for Goldilocks field arithmetic used by lighter-rs.

### 4. `elliottech/lighter-prover` (official, Rust, audit reports available)

**URL**: via elliottech GitHub

The official prover is in Rust. Contains the authoritative implementation. Has been through security audits (reports at https://docs.lighter.xyz/security/security-audits). Not publicly browsable but confirms the Go lib is the canonical signing reference.

---

## Porting Guide — Step by Step

### Step 1: Goldilocks Field

Implement `GoldilocksField` with Montgomery arithmetic:
- Modulus: `p = 0xFFFFFFFF00000001`
- Montgomery `R = 2^64`, `R^2 = 18446744065119617025`, `q_inv_neg = 18446744069414584319`
- Operations: `add`, `sub`, `mul` (Montgomery), `inverse` (Fermat), `pow7` (for S-box)

Test: `5 * 3 = 15`, `inverse(5) * 5 = 1`

### Step 2: GFp5

Implement polynomial arithmetic mod `x^5 - 3` over `GoldilocksField[5]`:
- `mul`: use the 5×5 convolution formulas above
- `inverse`: norm-based via Frobenius
- `sqrt`: Tonelli-Shanks in extension field

### Step 3: Poseidon2

Implement the permutation with the exact constants above.
Validate against the test vector from `TestPermute` and `TestHashToQuinticExtension`.

**Critical**: The sponge function absorbs each chunk by writing to `state[0..chunk_len]`, calls permute, then writes the next chunk (does NOT XOR into existing state — it overwrites).

### Step 4: ECgFp5 Scalar Field

- 5-limb `u64` Montgomery form
- Modulus `n` as a 320-bit integer
- Operations: `mul` (Montgomery), `add`, `sub`, `neg`
- `from_gfp5`: reduce a GFp5 element (interpret 5 u64 limbs as a 320-bit integer, reduce mod n)
- `to_le_bytes` / `from_le_bytes`: 40-byte little-endian

### Step 5: ECgFp5 Point

- Implement fractional `(x, u)` coordinates for scalar multiplication
- Implement Weierstrass `(X, Y, Z)` for `MulAdd2` (verification)
- Generator point: use the exact constants from the Go source
- `encode()`: returns a `GFp5` element from the point

### Step 6: Schnorr

```rust
fn sign(hashed_msg: GFp5, sk: ECgFp5Scalar) -> Signature {
    let k = ECgFp5Scalar::random();
    let r = Generator.mul(k).encode();
    let preimage = [r[0], r[1], r[2], r[3], r[4],
                    hashed_msg[0], hashed_msg[1], hashed_msg[2], hashed_msg[3], hashed_msg[4]];
    let e = ECgFp5Scalar::from_gfp5(poseidon2_hash_to_gfp5(&preimage));
    let s = k - e * sk;
    Signature { s, e }
}
```

### Step 7: Transaction Hashing

```rust
fn hash_create_order(params: &CreateOrderParams, chain_id: u32) -> [u8; 40] {
    let elems = vec![
        GoldilocksField::from_u32(chain_id),
        GoldilocksField::from_u32(12),  // TxTypeL2CreateOrder
        GoldilocksField::from_i64(params.nonce),
        GoldilocksField::from_i64(params.expired_at),
        GoldilocksField::from_i64(params.account_index),
        GoldilocksField::from_u32(params.api_key_index as u32),
        GoldilocksField::from_u32(params.market_index as u32),
        GoldilocksField::from_i64(params.client_order_index),
        GoldilocksField::from_i64(params.base_amount),
        GoldilocksField::from_u32(params.price),
        GoldilocksField::from_u32(params.is_ask as u32),
        GoldilocksField::from_u32(params.order_type as u32),
        GoldilocksField::from_u32(params.time_in_force as u32),
        GoldilocksField::from_u32(params.reduce_only as u32),
        GoldilocksField::from_u32(params.trigger_price),
        GoldilocksField::from_i64(params.order_expiry),
    ];
    poseidon2_hash_to_gfp5(&elems).to_le_bytes()
}
```

### Step 8: Full Signing Flow

```rust
// 1. Load private key
let sk = ECgFp5Scalar::from_le_bytes(&private_key_bytes);

// 2. Hash the transaction
let msg_hash_bytes = hash_create_order(&params, chain_id);

// 3. Convert to GFp5
let msg_hash = GFp5::from_le_bytes(&msg_hash_bytes);

// 4. Sign
let sig = schnorr_sign(msg_hash, sk);

// 5. Serialize
let sig_bytes = sig.to_bytes();  // 80 bytes: S[0..40] || E[0..40]
```

### Integer Encoding Notes

The Go `g.FromInt64` and `g.FromUint32` functions simply cast to `u64` and put into Montgomery form:
- `FromUint32(v)` → `(v as u64)` → `new(v as u64)`
- `FromInt64(v)` → `(v as u64)` (reinterpret bit pattern, not sign-extend to field)
- `FromUint64(v)` → same as above

For `Amount` split encoding in Withdraw:
```rust
let lo = GoldilocksField::from_u64(amount & 0xFFFFFFFF);
let hi = GoldilocksField::from_u64(amount >> 32);
```

---

## Sources

- [robustfengbin/lighter-sdk (Rust)](https://github.com/robustfengbin/lighter-sdk) — pure Rust port of all crypto primitives
- [elliottech/lighter-go (Go, official)](https://github.com/elliottech/lighter-go) — reference signing SDK
- [elliottech/poseidon_crypto (Go, official)](https://github.com/elliottech/poseidon_crypto) — canonical crypto library
- [elliottech/lighter-python (Python, official)](https://github.com/elliottech/lighter-python) — Python SDK with chain IDs
- [0xvasanth/lighter-rs (Rust)](https://github.com/0xvasanth/lighter-rs) — production Rust SDK
- [lighter-rs on crates.io](https://crates.io/crates/lighter-rs) — published crate
- [goldilocks-crypto on docs.rs](https://docs.rs/goldilocks-crypto/latest/goldilocks_crypto/) — Goldilocks Rust crate
- [ECgFp5 ecgfp5 Go package docs](https://pkg.go.dev/github.com/ppd0705/poseidon_crypto/curve/ecgfp5) — scalar field order
- [Lighter API docs](https://apidocs.lighter.xyz/docs/get-started-for-programmers-1) — transaction parameter specs
- [GitHub: pornin/ecgfp5](https://github.com/pornin/ecgfp5) — original ECgFp5 curve spec
- [ECgFp5 paper (eprint 2022/274)](https://eprint.iacr.org/2022/274) — Thomas Pornin's curve paper
- [Lighter Protocol whitepaper](https://assets.lighter.xyz/whitepaper.pdf) — protocol overview
- [DeepWiki: lighter-python SignerClient](https://deepwiki.com/elliottech/lighter-python/3.1-account-api) — signing flow analysis
