# ECgFp5 Reference Implementation — Thomas Pornin (pornin/ecgfp5)

Source: https://github.com/pornin/ecgfp5
Fetched: 2026-03-15

---

## Repository Overview

| Field | Value |
|-------|-------|
| Author | Thomas Pornin |
| License | MIT (Copyright 2022 Thomas Pornin) |
| Language | Rust 2018 edition |
| `no_std` | YES — `#![no_std]` in lib.rs |
| `unsafe` | NO — zero unsafe code |
| Dependencies | None (empty `[dependencies]`) |
| API style | Structs with methods + full std operator traits |
| Constant-time | YES — all secret-path operations |

---

## Crate Structure

```
rust/
├── Cargo.toml
├── src/
│   ├── lib.rs       — crate root, #![no_std], PRNG for tests
│   ├── field.rs     — GFp and GFp5 field arithmetic (1546 lines)
│   ├── scalar.rs    — Scalar arithmetic, Signed161, Signed640 (886 lines)
│   ├── curve.rs     — Point, PointAffine, curve operations (1240 lines)
│   └── multab.rs    — Precomputed generator point tables G0..G280 (799 lines)
└── benches/
    ├── field.rs
    ├── curve.rs
    └── scalar.rs
```

All modules are `pub mod` — all types are `pub`.

---

## src/lib.rs

```rust
#![no_std]

pub mod field;
pub mod curve;
pub mod scalar;
pub mod multab;

// A custom PRNG; not cryptographically secure, but good enough for tests.
#[cfg(test)]
struct PRNG(u128);

#[cfg(test)]
impl PRNG {
    const A: u128 = 87981536952642681582438141175044346919;
    const B: u128 = 331203846847999889118488772711684568729;

    fn next_u64(&mut self) -> u64 {
        self.0 = PRNG::A.wrapping_mul(self.0).wrapping_add(PRNG::B);
        (self.0 >> 64) as u64
    }

    fn next(&mut self, buf: &mut [u8]) {
        let mut acc: u64 = 0;
        for i in 0..buf.len() {
            if (i & 7) == 0 { acc = self.next_u64(); }
            buf[i] = acc as u8;
            acc >>= 8;
        }
    }
}
```

---

## src/field.rs — Complete Source

### GFp struct (Goldilocks field element)

```rust
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use core::convert::TryFrom;

// ========================================================================
// GF(p)

/// An element of GF(p).
#[derive(Clone, Copy, Debug)]
pub struct GFp(u64);

impl GFp {

    // IMPLEMENTATION NOTES:
    // ---------------------
    //
    // Let R = 2^64 mod p. Element x is represented by x*R mod p, in the
    // 0..p-1 range (Montgomery representation). Values are never outside
    // of that range.
    //
    // Everything is constant-time. There are specialized "Boolean"
    // functions such as iszero() and equals() that output a u64 which
    // happens, in practice, to have value 0 (for "false") or 2^64-1
    // (for "true").

    /// GF(p) modulus: p = 2^64 - 2^32 + 1
    pub const MOD: u64 = 0xFFFFFFFF00000001;

    /// Element 0 in GF(p).
    pub const ZERO: GFp = GFp::from_u64_reduce(0);

    /// Element 1 in GF(p).
    pub const ONE: GFp = GFp::from_u64_reduce(1);

    /// Element -1 in GF(p).
    pub const MINUS_ONE: GFp = GFp::from_u64_reduce(GFp::MOD - 1);

    // 2^128 mod p.
    const R2: u64 = 0xFFFFFFFE00000001;

    // Montgomery reduction: given x <= p*2^64 - 1 = 2^128 - 2^96 + 2^64 - 1,
    // return x/2^64 mod p (in the 0 to p-1 range).
    #[inline(always)]
    const fn montyred(x: u128) -> u64 {
        // Write:
        //  x = x0 + x1*2^32 + xh*2^64
        // with x0 and x1 over 32 bits each (in the 0..2^32-1 range),
        // and 0 <= xh <= p-1 (since x < p*2^64).
        // Then:
        //  x/2^64 = xh + ((x0 + x1*2^32) / 2^64) mod p
        let xl = x as u64;
        let xh = (x >> 64) as u64;
        let (a, e) = xl.overflowing_add(xl << 32);
        let b = a.wrapping_sub(a >> 32).wrapping_sub(e as u64);
        let (r, c) = xh.overflowing_sub(b);
        r.wrapping_sub(0u32.wrapping_sub(c as u32) as u64)
    }

    /// Build a GF(p) element from a 64-bit integer. Returns (r, c).
    /// If v < modulus: r = v as GFp, c = 0xFFFFFFFFFFFFFFFF.
    /// Otherwise: r = zero, c = 0.
    pub fn from_u64(v: u64) -> (GFp, u64) {
        let z = v.wrapping_sub(GFp::MOD);
        let c = ((v & !z) >> 63).wrapping_sub(1);
        (GFp::from_u64_reduce(v & c), c)
    }

    /// Build a GF(p) element from a 64-bit integer (implicitly reduced mod p).
    #[inline(always)]
    pub const fn from_u64_reduce(v: u64) -> GFp {
        GFp(GFp::montyred((v as u128) * (GFp::R2 as u128)))
    }

    /// Get the element as an integer, normalized in the 0..p-1 range.
    #[inline(always)]
    pub const fn to_u64(self) -> u64 {
        GFp::montyred(self.0 as u128)
    }

    /// Addition in GF(p)
    #[inline(always)]
    const fn add(self, rhs: Self) -> Self {
        let (x1, c1) = self.0.overflowing_sub(GFp::MOD - rhs.0);
        let adj = 0u32.wrapping_sub(c1 as u32);
        GFp(x1.wrapping_sub(adj as u64))
    }

    /// Subtraction in GF(p)
    #[inline(always)]
    const fn sub(self, rhs: Self) -> Self {
        let (x1, c1) = self.0.overflowing_sub(rhs.0);
        let adj = 0u32.wrapping_sub(c1 as u32);
        GFp(x1.wrapping_sub(adj as u64))
    }

    /// Negation in GF(p)
    #[inline(always)]
    const fn neg(self) -> Self {
        GFp::ZERO.sub(self)
    }

    /// Halving in GF(p) (division by 2).
    #[inline(always)]
    pub const fn half(self) -> Self {
        GFp((self.0 >> 1).wrapping_add(
            (self.0 & 1).wrapping_neg() & 0x7FFFFFFF80000001))
    }

    /// Doubling in GF(p) (multiplication by 2).
    #[inline(always)]
    pub const fn double(self) -> Self {
        self.add(self)
    }

    /// Multiplication in GF(p) by a small integer (less than 2^31).
    #[inline(always)]
    pub const fn mul_small(self, rhs: u32) -> Self {
        let x = (self.0 as u128) * (rhs as u128);
        let xl = x as u64;
        let xh = (x >> 64) as u64;
        let (r, c) = xl.overflowing_sub(GFp::MOD - ((xh << 32) - xh));
        GFp(r.wrapping_sub(0u32.wrapping_sub(c as u32) as u64))
    }

    /// Multiplication in GF(p)
    #[inline(always)]
    const fn mul(self, rhs: Self) -> Self {
        GFp(GFp::montyred((self.0 as u128) * (rhs.0 as u128)))
    }

    /// Squaring in GF(p)
    #[inline(always)]
    pub const fn square(self) -> Self {
        self.mul(self)
    }

    /// Multiple squarings in GF(p): return x^(2^n)
    pub fn msquare(self, n: u32) -> Self {
        let mut x = self;
        for _ in 0..n { x = x.square(); }
        x
    }

    /// Inversion in GF(p); if zero, returns zero.
    /// Uses Fermat's little theorem: 1/x = x^(p-2) mod p.
    /// p-2 = 0xFFFFFFFEFFFFFFFF
    pub fn invert(self) -> Self {
        let x = self;
        let x2 = x * x.square();
        let x4 = x2 * x2.msquare(2);
        let x5 = x * x4.square();
        let x10 = x5 * x5.msquare(5);
        let x15 = x5 * x10.msquare(5);
        let x16 = x * x15.square();
        let x31 = x15 * x16.msquare(15);
        let x32 = x * x31.square();
        return x32 * x31.msquare(33);
    }

    fn div(self, rhs: Self) -> Self {
        self * rhs.invert()
    }

    /// Returns 0xFFFFFFFFFFFFFFFF if self == 0, else 0.
    #[inline(always)]
    pub const fn iszero(self) -> u64 {
        !((((self.0 | self.0.wrapping_neg()) as i64) >> 63) as u64)
    }

    /// Returns 0xFFFFFFFFFFFFFFFF if self == 1, else 0.
    #[inline(always)]
    pub const fn isone(self) -> u64 {
        self.equals(GFp::ONE)
    }

    /// Returns 0xFFFFFFFFFFFFFFFF if self == -1 mod p, else 0.
    #[inline(always)]
    pub const fn isminusone(self) -> u64 {
        self.equals(GFp::MINUS_ONE)
    }

    /// Returns 0xFFFFFFFFFFFFFFFF if self == rhs, else 0.
    #[inline(always)]
    pub const fn equals(self, rhs: Self) -> u64 {
        let t = self.0 ^ rhs.0;
        !((((t | t.wrapping_neg()) as i64) >> 63) as u64)
    }

    /// Legendre symbol: return x^((p-1)/2) as a GF(p) element.
    /// (p-1)/2 = 0x7FFFFFFF80000000
    pub fn legendre(self) -> GFp {
        let x = self;
        let x2 = x * x.square();
        let x4 = x2 * x2.msquare(2);
        let x8 = x4 * x4.msquare(4);
        let x16 = x8 * x8.msquare(8);
        let x32 = x16 * x16.msquare(16);
        x32.msquare(31)
    }

    // Precomputed roots of unity for Tonelli-Shanks sqrt.
    // For g = 7^(2^32-1) mod p = 1753635133440165772 (primitive 2^32 root of 1),
    // GG[i] = g^(2^i) for i = 0 to 31.
    const GG: [GFp; 32] = [
        GFp::from_u64_reduce( 1753635133440165772),
        GFp::from_u64_reduce( 4614640910117430873),
        GFp::from_u64_reduce( 9123114210336311365),
        GFp::from_u64_reduce(16116352524544190054),
        GFp::from_u64_reduce( 6414415596519834757),
        GFp::from_u64_reduce( 1213594585890690845),
        GFp::from_u64_reduce(17096174751763063430),
        GFp::from_u64_reduce( 5456943929260765144),
        GFp::from_u64_reduce( 9713644485405565297),
        GFp::from_u64_reduce(16905767614792059275),
        GFp::from_u64_reduce( 5416168637041100469),
        GFp::from_u64_reduce(17654865857378133588),
        GFp::from_u64_reduce( 3511170319078647661),
        GFp::from_u64_reduce(18146160046829613826),
        GFp::from_u64_reduce( 9306717745644682924),
        GFp::from_u64_reduce(12380578893860276750),
        GFp::from_u64_reduce( 6115771955107415310),
        GFp::from_u64_reduce(17776499369601055404),
        GFp::from_u64_reduce(16207902636198568418),
        GFp::from_u64_reduce( 1532612707718625687),
        GFp::from_u64_reduce(17492915097719143606),
        GFp::from_u64_reduce(  455906449640507599),
        GFp::from_u64_reduce(11353340290879379826),
        GFp::from_u64_reduce( 1803076106186727246),
        GFp::from_u64_reduce(13797081185216407910),
        GFp::from_u64_reduce(17870292113338400769),
        GFp::from_u64_reduce(        549755813888),
        GFp::from_u64_reduce(      70368744161280),
        GFp::from_u64_reduce(17293822564807737345),
        GFp::from_u64_reduce(18446744069397807105),
        GFp::from_u64_reduce(     281474976710656),
        GFp::from_u64_reduce(18446744069414584320)
    ];

    /// Square root in GF(p); returns (r, cc):
    ///  - If square: r = sqrt(self), cc = 0xFFFFFFFFFFFFFFFF
    ///  - If not square: r = zero, cc = 0
    /// Uses constant-time Tonelli-Shanks.
    /// p = q*2^n + 1 with q = 2^32 - 1 and n = 32.
    pub fn sqrt(self) -> (Self, u64) {
        let x = self;
        let x2 = x * x.square();
        let x4 = x2 * x2.msquare(2);
        let x5 = x * x4.square();
        let x10 = x5 * x5.msquare(5);
        let x15 = x5 * x10.msquare(5);
        let x16 = x * x15.square();
        let x31 = x15 * x16.msquare(15);
        let mut r = x * x31;
        let mut v = x * x31.square();

        for i in (1..32).rev() {
            let w = v.msquare((i - 1) as u32);
            let cc = w.equals(GFp::MINUS_ONE);
            v = GFp(v.0 ^ (cc & (v.0 ^ (v * GFp::GG[32 - i]).0)));
            r = GFp(r.0 ^ (cc & (r.0 ^ (r * GFp::GG[31 - i]).0)));
        }
        let m = v.iszero() | v.equals(GFp::ONE);
        (GFp(r.0 & m), m)
    }

    /// Constant-time select: returns x0 if c == 0, x1 if c == 0xFFFFFFFFFFFFFFFF.
    #[inline(always)]
    pub fn select(c: u64, x0: GFp, x1: GFp) -> GFp {
        GFp(x0.0 ^ (c & (x0.0 ^ x1.0)))
    }
}

// All arithmetic operator traits (Add, Sub, Mul, Div, Neg, *Assign)
// are implemented for GFp, supporting both value and reference operands.
```

### GFp5 struct (quintic extension field)

```rust
// ========================================================================
// GF(p^5)

/// An element of GF(p^5).
/// Represented as x0 + x1*z + x2*z^2 + x3*z^3 + x4*z^4
/// where the modulus polynomial is z^5 - 3 (i.e. z^5 = 3 in GF(p^5)).
#[derive(Clone, Copy, Debug)]
pub struct GFp5(pub [GFp; 5]);

impl GFp5 {

    /// Value zero in GF(p^5).
    pub const ZERO: GFp5 = GFp5([GFp::ZERO, GFp::ZERO, GFp::ZERO, GFp::ZERO, GFp::ZERO]);

    /// Value one in GF(p^5).
    pub const ONE: GFp5 = GFp5([GFp::ONE, GFp::ZERO, GFp::ZERO, GFp::ZERO, GFp::ZERO]);

    /// Create from five u64 coefficients (implicitly reduced mod p).
    #[inline(always)]
    pub const fn from_u64_reduce(x0: u64, x1: u64, x2: u64, x3: u64, x4: u64) -> Self {
        GFp5([
            GFp::from_u64_reduce(x0),
            GFp::from_u64_reduce(x1),
            GFp::from_u64_reduce(x2),
            GFp::from_u64_reduce(x3),
            GFp::from_u64_reduce(x4),
        ])
    }

    /// Create from five u64 coefficients (strict range check).
    /// Returns (element, 0xFFFF...) on success, (zero, 0) on failure.
    pub fn from_u64(x0: u64, x1: u64, x2: u64, x3: u64, x4: u64) -> (Self, u64) {
        let (w0, c0) = GFp::from_u64(x0);
        let (w1, c1) = GFp::from_u64(x1);
        let (w2, c2) = GFp::from_u64(x2);
        let (w3, c3) = GFp::from_u64(x3);
        let (w4, c4) = GFp::from_u64(x4);
        let c = c0 & c1 & c2 & c3 & c4;
        (GFp5([GFp(w0.0 & c), GFp(w1.0 & c), GFp(w2.0 & c), GFp(w3.0 & c), GFp(w4.0 & c)]), c)
    }

    /// Decode from exactly 40 bytes (5 little-endian u64 coefficients).
    /// Returns (element, 0xFFFF...) on success, (zero, 0) on failure.
    pub fn decode(buf: &[u8]) -> (Self, u64) {
        if buf.len() != 40 {
            return (GFp5::ZERO, 0);
        }
        GFp5::from_u64(
            u64::from_le_bytes(*<&[u8; 8]>::try_from(&buf[ 0.. 8]).unwrap()),
            u64::from_le_bytes(*<&[u8; 8]>::try_from(&buf[ 8..16]).unwrap()),
            u64::from_le_bytes(*<&[u8; 8]>::try_from(&buf[16..24]).unwrap()),
            u64::from_le_bytes(*<&[u8; 8]>::try_from(&buf[24..32]).unwrap()),
            u64::from_le_bytes(*<&[u8; 8]>::try_from(&buf[32..40]).unwrap()))
    }

    /// Encode to exactly 40 bytes (5 little-endian u64 coefficients).
    pub fn encode(self) -> [u8; 40] {
        let mut r = [0u8; 40];
        for i in 0..5 {
            r[8*i..8*i+8].copy_from_slice(&self.0[i].to_u64().to_le_bytes());
        }
        r
    }

    // ---- Internal set_* mutation functions ----

    #[inline]
    pub(crate) fn set_add(&mut self, rhs: &Self) {
        self.0[0] += rhs.0[0]; self.0[1] += rhs.0[1]; self.0[2] += rhs.0[2];
        self.0[3] += rhs.0[3]; self.0[4] += rhs.0[4];
    }

    #[inline]
    pub(crate) fn set_sub(&mut self, rhs: &Self) {
        self.0[0] -= rhs.0[0]; self.0[1] -= rhs.0[1]; self.0[2] -= rhs.0[2];
        self.0[3] -= rhs.0[3]; self.0[4] -= rhs.0[4];
    }

    #[inline]
    pub(crate) fn set_neg(&mut self) {
        self.0[0] = -self.0[0]; self.0[1] = -self.0[1]; self.0[2] = -self.0[2];
        self.0[3] = -self.0[3]; self.0[4] = -self.0[4];
    }

    pub fn half(self) -> Self { let mut r = self; r.set_half(); r }
    pub(crate) fn set_half(&mut self) {
        self.0[0] = self.0[0].half(); self.0[1] = self.0[1].half();
        self.0[2] = self.0[2].half(); self.0[3] = self.0[3].half();
        self.0[4] = self.0[4].half();
    }

    pub fn double(self) -> Self { let mut r = self; r.set_double(); r }
    pub(crate) fn set_double(&mut self) {
        self.0[0] = self.0[0].double(); self.0[1] = self.0[1].double();
        self.0[2] = self.0[2].double(); self.0[3] = self.0[3].double();
        self.0[4] = self.0[4].double();
    }

    pub fn mul_small(self, rhs: u32) -> Self { let mut r = self; r.set_mul_small(rhs); r }
    pub(crate) fn set_mul_small(&mut self, rhs: u32) {
        self.0[0] = self.0[0].mul_small(rhs); self.0[1] = self.0[1].mul_small(rhs);
        self.0[2] = self.0[2].mul_small(rhs); self.0[3] = self.0[3].mul_small(rhs);
        self.0[4] = self.0[4].mul_small(rhs);
    }

    /// Multiply by v*z where v is a small integer (< 2^29).
    /// Uses the identity z^5 = 3, so z * x4*z^4 = 3*x4.
    pub fn mul_small_k1(self, rhs: u32) -> Self { let mut r = self; r.set_mul_small_k1(rhs); r }
    pub(crate) fn set_mul_small_k1(&mut self, rhs: u32) {
        let d0 = self.0[4].mul_small(rhs * 3);
        let d1 = self.0[0].mul_small(rhs);
        let d2 = self.0[1].mul_small(rhs);
        let d3 = self.0[2].mul_small(rhs);
        let d4 = self.0[3].mul_small(rhs);
        self.0[0] = d0; self.0[1] = d1; self.0[2] = d2; self.0[3] = d3; self.0[4] = d4;
    }

    /// Multiply by (v1*z - v0), both v0, v1 < 2^28.
    pub fn mul_small_kn01(self, v0: u32, v1: u32) -> Self { let mut r = self; r.set_mul_small_kn01(v0, v1); r }
    pub(crate) fn set_mul_small_kn01(&mut self, v0: u32, v1: u32) {
        let d0 = self.0[4].mul_small(3 * v1) - self.0[0].mul_small(v0);
        let d1 = self.0[0].mul_small(v1) - self.0[1].mul_small(v0);
        let d2 = self.0[1].mul_small(v1) - self.0[2].mul_small(v0);
        let d3 = self.0[2].mul_small(v1) - self.0[3].mul_small(v0);
        let d4 = self.0[3].mul_small(v1) - self.0[4].mul_small(v0);
        self.0[0] = d0; self.0[1] = d1; self.0[2] = d2; self.0[3] = d3; self.0[4] = d4;
    }

    /// Multiply by a GF(p) element (coefficient 0 only).
    pub fn mul_k0(self, rhs: GFp) -> GFp5 { let mut r = self; r.set_mul_k0(rhs); r }
    pub(crate) fn set_mul_k0(&mut self, rhs: GFp) {
        self.0[0] *= rhs; self.0[1] *= rhs; self.0[2] *= rhs;
        self.0[3] *= rhs; self.0[4] *= rhs;
    }

    // ---- GF(p^5) multiplication using the z^5 = 3 reduction rule ----
    // Coefficient k of product self * rhs:
    //   k0 = a0*b0 + 3*(a1*b4 + a2*b3 + a3*b2 + a4*b1)
    //   k1 = a0*b1 + a1*b0 + 3*(a2*b4 + a3*b3 + a4*b2)
    //   k2 = a0*b2 + a1*b1 + a2*b0 + 3*(a3*b4 + a4*b3)
    //   k3 = a0*b3 + a1*b2 + a2*b1 + a3*b0 + 3*(a4*b4)
    //   k4 = a0*b4 + a1*b3 + a2*b2 + a3*b1 + a4*b0

    #[inline(always)]
    fn mul_to_k0(&self, rhs: &Self) -> GFp {
        let pp0 = (self.0[0].0 as u128) * (rhs.0[0].0 as u128);
        let pp1 = (self.0[1].0 as u128) * (rhs.0[4].0 as u128);
        let pp2 = (self.0[2].0 as u128) * (rhs.0[3].0 as u128);
        let pp3 = (self.0[3].0 as u128) * (rhs.0[2].0 as u128);
        let pp4 = (self.0[4].0 as u128) * (rhs.0[1].0 as u128);
        let zhi = (pp0 >> 64) + 3 * ((pp1 >> 64) + (pp2 >> 64) + (pp3 >> 64) + (pp4 >> 64));
        let zlo = ((pp0 as u64) as u128) + 3 * (((pp1 as u64) as u128)
            + ((pp2 as u64) as u128) + ((pp3 as u64) as u128) + ((pp4 as u64) as u128));
        GFp(GFp::montyred(zlo + (zhi << 32) - zhi))
    }

    // ... (mul_to_k1 through mul_to_k4 follow the same pattern)

    #[inline]
    pub(crate) fn set_mul(&mut self, rhs: &Self) {
        let d0 = self.mul_to_k0(rhs);
        let d1 = self.mul_to_k1(rhs);
        let d2 = self.mul_to_k2(rhs);
        let d3 = self.mul_to_k3(rhs);
        let d4 = self.mul_to_k4(rhs);
        self.0[0] = d0; self.0[1] = d1; self.0[2] = d2; self.0[3] = d3; self.0[4] = d4;
    }

    // ---- Squaring (optimized coefficients) ----
    // k0 = a0^2 + 6*(a1*a4 + a2*a3)
    // k1 = 2*a0*a1 + 6*a2*a4 + 3*a3^2
    // k2 = 2*a0*a2 + a1^2 + 6*a3*a4
    // k3 = 2*(a0*a3 + a1*a2) + 3*a4^2
    // k4 = 2*(a0*a4 + a1*a3) + a2^2

    pub fn square(self) -> Self { let mut r = self; r.set_square(); r }
    pub(crate) fn set_square(&mut self) {
        let d0 = self.square_to_k0(); let d1 = self.square_to_k1();
        let d2 = self.square_to_k2(); let d3 = self.square_to_k3();
        let d4 = self.square_to_k4();
        self.0[0] = d0; self.0[1] = d1; self.0[2] = d2; self.0[3] = d3; self.0[4] = d4;
    }

    pub fn msquare(self, n: u32) -> Self { let mut r = self; r.set_msquare(n); r }
    pub(crate) fn set_msquare(&mut self, n: u32) {
        for _ in 0..n { self.set_square(); }
    }

    /// Returns 0xFFFFFFFFFFFFFFFF if self == rhs, else 0.
    #[inline]
    pub fn equals(self, rhs: Self) -> u64 {
        let z = (self.0[0].0 ^ rhs.0[0].0) | (self.0[1].0 ^ rhs.0[1].0)
            | (self.0[2].0 ^ rhs.0[2].0) | (self.0[3].0 ^ rhs.0[3].0)
            | (self.0[4].0 ^ rhs.0[4].0);
        ((z | z.wrapping_neg()) >> 63).wrapping_sub(1)
    }

    /// Returns 0xFFFFFFFFFFFFFFFF if self == 0, else 0.
    #[inline]
    pub fn iszero(self) -> u64 {
        let z = self.0[0].0 | self.0[1].0 | self.0[2].0 | self.0[3].0 | self.0[4].0;
        ((z | z.wrapping_neg()) >> 63).wrapping_sub(1)
    }

    // ---- Frobenius operators ----
    // raise to power p: multiply coefficient k by z_k^p for k=1..4
    // Precomputed constants are p-th powers of the symbolic variable z.
    fn set_frob1(&mut self) {
        self.0[1] *= GFp::from_u64_reduce( 1041288259238279555);
        self.0[2] *= GFp::from_u64_reduce(15820824984080659046);
        self.0[3] *= GFp::from_u64_reduce(  211587555138949697);
        self.0[4] *= GFp::from_u64_reduce( 1373043270956696022);
    }

    // raise to power p^2
    fn set_frob2(&mut self) {
        self.0[1] *= GFp::from_u64_reduce(15820824984080659046);
        self.0[2] *= GFp::from_u64_reduce( 1373043270956696022);
        self.0[3] *= GFp::from_u64_reduce( 1041288259238279555);
        self.0[4] *= GFp::from_u64_reduce(  211587555138949697);
    }

    /// Inversion in GF(p^5). Uses Itoh-Tsujii algorithm.
    /// If zero, returns zero.
    pub fn invert(self) -> GFp5 { let mut r = self; r.set_invert(); r }

    pub(crate) fn set_invert(&mut self) {
        // x^(r-1) = x^(p + p^2 + p^3 + p^4)
        //         = phi1(x) * phi1(phi1(x)) * phi2(phi1(x) * phi1(phi1(x)))
        // then 1/x = x^(r-1) / x^r  where x^r is in GF(p)
        let t0 = *self;
        self.set_frob1();
        let mut t1 = *self;
        t1.set_frob1();
        self.set_mul(&t1);
        t1 = *self;
        self.set_frob2();
        self.set_mul(&t1);
        let x = self.mul_to_k0(&t0);
        self.set_mul_k0(x.invert());
    }

    pub(crate) fn set_div(&mut self, rhs: &Self) {
        let mut d = *rhs;
        d.set_invert();
        self.set_mul(&d);
    }

    /// Return x^((p^5-1)/2) as a GF(p) element.
    /// 0 if x == 0, 1 if x is a non-zero square, -1 otherwise.
    pub fn legendre(self) -> GFp {
        let mut t0 = self;
        t0.set_frob1();
        let mut t1 = t0;
        t1.set_frob1();
        t0.set_mul(&t1);
        t1 = t0;
        t1.set_frob2();
        t0.set_mul(&t1);
        let x = self.mul_to_k0(&t0);
        x.legendre()
    }

    /// Square root in GF(p^5); returns (s, cc):
    ///  - If square: s = sqrt(self), cc = 0xFFFFFFFFFFFFFFFF
    ///  - If not square: s = zero, cc = 0
    pub fn sqrt(self) -> (GFp5, u64) {
        let mut r = self;
        let c = r.set_sqrt();
        (r, c)
    }

    pub(crate) fn set_sqrt(&mut self) -> u64 {
        // sqrt(x) = sqrt(x^r) / x^((r-1)/2)
        // where x^r is in GF(p), computed via:
        //   d <- x^((p+1)/2)
        //   e <- frob1(d * frob2(d))  => x^((r-1)/2)
        //   x^r = x * e^2
        let mut t = *self;
        let mut y = t; y.set_square(); t.set_mul(&y);     // t = x^3
        y = t; y.set_msquare( 2); t.set_mul(&y);           // t = x^(2^4-1)
        y = t; y.set_msquare( 4); t.set_mul(&y);           // t = x^(2^8-1)
        y = t; y.set_msquare( 8); t.set_mul(&y);           // t = x^(2^16-1)
        y = t; y.set_msquare(16); t.set_mul(&y);           // t = x^(2^32-1)
        t.set_msquare(31);
        t.set_mul(self);                                    // t = x^((p+1)/2)
        y = t;
        y.set_frob2();
        t.set_mul(&y);
        t.set_frob1();                                      // t = x^((r-1)/2)
        y = t;
        y.set_square();
        let a = self.mul_to_k0(&y);                        // a = x^r (in GF(p))
        let (s, cc) = a.sqrt();
        *self = t;
        self.set_invert();
        self.set_mul_k0(s);
        cc
    }

    /// Constant-time select: returns x0 if c == 0, x1 if c == 0xFFFFFFFFFFFFFFFF.
    #[inline(always)]
    pub fn select(c: u64, x0: GFp5, x1: GFp5) -> GFp5 {
        GFp5([
            GFp::select(c, x0.0[0], x1.0[0]),
            GFp::select(c, x0.0[1], x1.0[1]),
            GFp::select(c, x0.0[2], x1.0[2]),
            GFp::select(c, x0.0[3], x1.0[3]),
            GFp::select(c, x0.0[4], x1.0[4]),
        ])
    }

    /// Partial constant-time lookup (for windowed scalar multiplication).
    /// Precondition: either self is zero, or c == 0.
    /// If c == 0xFFFFFFFFFFFFFFFF, sets self = y.
    #[inline(always)]
    pub(crate) fn set_partial_lookup(&mut self, y: GFp5, c: u64) {
        self.0[0].0 |= c & y.0[0].0;
        self.0[1].0 |= c & y.0[1].0;
        self.0[2].0 |= c & y.0[2].0;
        self.0[3].0 |= c & y.0[3].0;
        self.0[4].0 |= c & y.0[4].0;
    }
}

// All arithmetic operator traits (Add, Sub, Mul, Div, Neg, *Assign)
// are implemented for GFp5, supporting both value and reference operands.
```

---

## src/scalar.rs — Complete Source

```rust
use core::ops::{Add, AddAssign, Neg, Sub, SubAssign, Mul, MulAssign};

/// A scalar (integer modulo the prime group order n).
/// Stored as five 64-bit limbs in normal (non-Montgomery) representation.
#[derive(Clone, Copy, Debug)]
pub struct Scalar([u64; 5]);

impl Scalar {

    pub const ZERO: Self = Self([0, 0, 0, 0, 0]);
    pub const ONE: Self = Self([1, 0, 0, 0, 0]);

    // Group order n (319-bit prime):
    const N: Self = Self([
        0xE80FD996948BFFE1,
        0xE8885C39D724A09C,
        0x7FFFFFE6CFB80639,
        0x7FFFFFF100000016,
        0x7FFFFFFD80000007,
    ]);

    // -1/N[0] mod 2^64 (Montgomery constant)
    const N0I: u64 = 0xD78BEF72057B7BDF;

    // 2^640 mod n (Montgomery R^2 for converting to Montgomery form)
    const R2: Self = Self([
        0xA01001DCE33DC739,
        0x6C3228D33F62ACCF,
        0xD1D796CC91CF8525,
        0xAADFFF5D1574C1D8,
        0x4ACA13B28CA251F5,
    ]);

    // 2^632 mod n (used for decode_reduce chunking)
    const T632: Self = Self([
        0x2B0266F317CA91B3,
        0xEC1D26528E984773,
        0x8651D7865E12DB94,
        0xDA2ADFF5941574D0,
        0x53CACA12110CA256,
    ]);

    // raw addition (no reduction)
    fn add_inner(self, a: Self) -> Self { ... }

    // raw subtraction, returns (result, borrow_mask)
    fn sub_inner(self, a: Self) -> (Self, u64) { ... }

    /// Constant-time select: returns a0 if c==0, a1 if c==0xFFFF...
    pub fn select(c: u64, a0: Self, a1: Self) -> Self { ... }

    // Scalar addition (mod n)
    fn add(self, rhs: Self) -> Self {
        let r0 = self.add_inner(rhs);
        let (r1, c) = r0.sub_inner(Self::N);
        Self::select(c, r1, r0)
    }

    // Scalar subtraction (mod n)
    fn sub(self, rhs: Self) -> Self {
        let (r0, c) = self.sub_inner(rhs);
        let r1 = r0.add_inner(Self::N);
        Self::select(c, r0, r1)
    }

    // Scalar negation
    fn neg(self) -> Self { Self::ZERO.sub(self) }

    // Montgomery multiplication: returns (self*rhs)/2^320 mod n
    // 'self' MUST be < n
    fn montymul(self, rhs: Self) -> Self { ... }

    // Full scalar multiplication (mod n)
    fn mul(self, rhs: Self) -> Self {
        self.montymul(Self::R2).montymul(rhs)
    }

    /// Decode bytes (little-endian unsigned) into scalar.
    /// Returns (s, c): if decoded value < n, s is value and c = 0xFFFF...
    /// Otherwise s = ZERO, c = 0.
    pub fn decode(buf: &[u8]) -> (Self, u64) { ... }

    /// Decode bytes and REDUCE mod n. Never fails.
    /// Processes 312-bit (39-byte) chunks in high-to-low order,
    /// multiplying accumulated result by 2^312 = T632 each step.
    pub fn decode_reduce(buf: &[u8]) -> Self { ... }

    /// Encode to exactly 40 bytes (little-endian).
    pub fn encode(self) -> [u8; 40] {
        let mut r = [0u8; 40];
        for i in 0..5 {
            r[8*i..8*i+8].copy_from_slice(&self.0[i].to_le_bytes());
        }
        r
    }

    /// Recode scalar into signed integers for windowed scalar mul.
    /// Window width w (2..=10), digits in range -(2^(w-1)) to +2^(w-1).
    pub(crate) fn recode_signed(self, ss: &mut [i32], w: i32) { ... }

    /// Lagrange decomposition: represent scalar k as (v0, v1)
    /// such that k = v0/v1 mod n. Uses algorithm 4 from
    /// https://eprint.iacr.org/2020/454
    /// NOT constant-time — use only on public values.
    pub fn lagrange(self) -> (Signed161, Signed161) { ... }

    /// Returns 0xFFFFFFFFFFFFFFFF if self == 0, else 0.
    pub fn iszero(self) -> u64 { ... }

    /// Returns 0xFFFFFFFFFFFFFFFF if self == rhs (mod n), else 0.
    pub fn equals(self, rhs: Self) -> u64 { ... }
}

// Operator traits: Add, AddAssign, Sub, SubAssign, Neg, Mul, MulAssign
// all implemented for Scalar.

/// 161-bit signed integer (for Lagrange decomposition).
/// Two's complement, truncated to 161 bits.
/// WARNING: all operations are variable-time.
#[derive(Clone, Copy, Debug)]
pub struct Signed161([u64; 3]);

impl Signed161 {
    fn from_scalar(s: Scalar) -> Self { Self([s.0[0], s.0[1], s.0[2]]) }

    /// Convert to scalar (mod n). Variable-time.
    pub fn to_scalar_vartime(self) -> Scalar { ... }

    /// Export as 192-bit integer (sign-extended).
    pub fn to_u192(self) -> [u64; 3] { ... }

    /// Recode into 33 signed digits for 5-bit window.
    pub(crate) fn recode_signed_5(self) -> [i32; 33] { ... }

    fn add_shifted(&mut self, v: &Signed161, s: i32) { ... }
    fn sub_shifted(&mut self, v: &Signed161, s: i32) { ... }
}

/// 640-bit signed integer (for Lagrange decomposition internals).
/// WARNING: all operations are variable-time.
#[derive(Clone, Copy, Debug)]
struct Signed640([u64; 10]);

impl Signed640 {
    fn from_nsquared() -> Self { ... }       // Returns n^2
    fn from_mul_scalars(a: Scalar, b: Scalar) -> Self { ... }  // Returns a*b
    fn add1(&mut self) { ... }               // Increment by 1
    fn is_nonnegative(&self) -> bool { ... }
    fn lt_unsigned(&self, rhs: &Self) -> bool { ... }
    fn bitlength(&self) -> i32 { ... }
    fn add_shifted(&mut self, v: &Signed640, s: i32) { ... }
    fn sub_shifted(&mut self, v: &Signed640, s: i32) { ... }
}
```

---

## src/curve.rs — Complete Source

### Key constants

```rust
impl Point {
    // Curve equation: y^2 = x*(x^2 + a*x + b)
    // a = 2 (in GF(p^5), constant coefficient only)
    const A: GFp5 = GFp5([GFp::from_u64_reduce(2), GFp::ZERO, GFp::ZERO, GFp::ZERO, GFp::ZERO]);

    // b = 263*z (coefficient of z term)
    const B1: u32 = 263;

    // 4*b
    const B_MUL4: GFp5 = GFp5([GFp::ZERO, GFp::from_u64_reduce(4 * 263), ...]);

    /// The neutral point (identity element): (X:Z:U:T) = (0:1:0:1)
    pub const NEUTRAL: Self = Self { X: GFp5::ZERO, Z: GFp5::ONE, U: GFp5::ZERO, T: GFp5::ONE };

    /// The conventional generator (corresponds to encoding w = 4).
    /// w = y/x = 4  =>  y = 4x
    pub const GENERATOR: Self = Self {
        X: GFp5::from_u64_reduce(
            12883135586176881569, 4356519642755055268,
            5248930565894896907, 2165973894480315022, 2448410071095648785),
        Z: GFp5::ONE,
        U: GFp5::from_u64_reduce(13835058052060938241, 0, 0, 0, 0),
        T: GFp5::ONE,
    };
}
```

### Fractional coordinates

The point uses (X/Z, U/T) fractional coordinates where:
- x = X/Z is the x-coordinate
- u = U/T = x/y (the "u" coordinate, reciprocal of w)
- The neutral N has u = 0 (U = 0)
- Encoding: w = 1/u = y/x

### Encoding/Decoding

```rust
impl Point {
    /// Encode point to a GFp5 field element: w = 1/u = y/x.
    /// Neutral maps to w = 0.
    pub fn encode(self) -> GFp5 {
        self.T / self.U
    }

    /// Test if a GFp5 value can be decoded as a valid point.
    /// Returns 0xFFFF... if valid, 0 otherwise.
    pub fn validate(w: GFp5) -> u64 {
        // w = 0 is valid (neutral), or (w^2 - a)^2 - 4*b must be a QR
        let e = w.square() - Self::A;
        let delta = e.square() - Self::B_MUL4;
        w.iszero() | delta.legendre().isone()
    }

    /// Decode GFp5 field element to point. Returns (P, c).
    /// Solves: x^2 - (w^2 - a)*x + b = 0, selects non-square root.
    pub fn decode(w: GFp5) -> (Self, u64) {
        let e = w.square() - Self::A;
        let delta = e.square() - Self::B_MUL4;
        let (r, c) = delta.sqrt();
        let x1 = (e + r).half();
        let x2 = (e - r).half();
        // exactly one of x1, x2 is a square in GF(p^5)
        let x = GFp5::select(x1.legendre().isone(), x1, x2);
        let X = GFp5::select(c, GFp5::ZERO, x);
        let Z = GFp5::ONE;
        let U = GFp5::select(c, GFp5::ZERO, GFp5::ONE);
        let T = GFp5::select(c, GFp5::ONE, w);
        (Self { X, Z, U, T }, c | w.iszero())
    }
}
```

### Point addition formulas

```rust
impl Point {
    // General point addition. Complete (no special cases). Cost: 10M.
    fn set_add(&mut self, rhs: &Self) {
        let (X1, Z1, U1, T1) = (&self.X, &self.Z, &self.U, &self.T);
        let (X2, Z2, U2, T2) = (&rhs.X, &rhs.Z, &rhs.U, &rhs.T);
        let t1 = X1 * X2;
        let t2 = Z1 * Z2;
        let t3 = U1 * U2;
        let t4 = T1 * T2;
        let t5 = (X1 + Z1) * (X2 + Z2) - t1 - t2;
        let t6 = (U1 + T1) * (U2 + T2) - t3 - t4;
        let t7 = t1 + t2.mul_small_k1(Self::B1);
        let t8 = t4 * t7;
        let t9 = t3 * (t5.mul_small_k1(2 * Self::B1) + t7.double());
        let t10 = (t4 + t3.double()) * (t5 + t7);
        self.X = (t10 - t8).mul_small_k1(Self::B1);
        self.Z = t8 - t9;
        self.U = t6 * (t2.mul_small_k1(Self::B1) - t1);
        self.T = t8 + t9;
    }

    // Add a point in affine coordinates. Cost: 8M.
    fn set_add_affine(&mut self, rhs: &PointAffine) {
        let (X1, Z1, U1, T1) = (&self.X, &self.Z, &self.U, &self.T);
        let (x2, u2) = (&rhs.x, &rhs.u);
        let t1 = X1 * x2;
        let t2 = Z1;
        let t3 = U1 * u2;
        let t4 = T1;
        let t5 = X1 + x2 * Z1;
        let t6 = U1 + u2 * T1;
        let t7 = t1 + t2.mul_small_k1(Self::B1);
        let t8 = t4 * t7;
        let t9 = t3 * (t5.mul_small_k1(2 * Self::B1) + t7.double());
        let t10 = (t4 + t3.double()) * (t5 + t7);
        self.X = (t10 - t8).mul_small_k1(Self::B1);
        self.U = t6 * (t2.mul_small_k1(Self::B1) - t1);
        self.Z = t8 - t9;
        self.T = t8 + t9;
    }

    // Negation: negate u coordinate
    fn set_neg(&mut self) { self.U.set_neg(); }

    /// Specialized doubling. Cost: 4M+5S.
    fn set_double(&mut self) {
        let (X, Z, U, T) = (&self.X, &self.Z, &self.U, &self.T);
        let t1 = Z * T;
        let t2 = t1 * T;
        let X1 = t2.square();
        let Z1 = t1 * U;
        let t3 = U.square();
        let W1 = t2 - (X + Z).double() * t3;
        let t4 = Z1.square();
        self.X = t4.mul_small_k1(4 * Self::B1);
        self.Z = W1.square();
        self.U = (W1 + Z1).square() - t4 - self.Z;
        self.T = X1.double() - t4.mul_small(4) - self.Z;
    }

    /// n successive doublings (faster than n calls to double()).
    /// Cost: n*(2M+5S) + 2M+1S
    pub fn mdouble(self, n: u32) -> Self { let mut r = self; r.set_mdouble(n); r }

    /// Test if neutral: returns 0xFFFF... if neutral, 0 otherwise.
    pub fn isneutral(self) -> u64 { self.U.iszero() }

    /// Equality: returns 0xFFFF... if equal, 0 otherwise.
    pub fn equals(self, rhs: Self) -> u64 {
        (self.U * rhs.T).equals(rhs.U * self.T)
    }
}
```

### Scalar multiplication

```rust
impl Point {
    // 5-bit signed window scalar multiplication.
    const WINDOW: usize = 5;
    const WIN_SIZE: usize = 1 << ((Self::WINDOW - 1) as i32);  // = 16

    fn set_mul(&mut self, s: &Scalar) {
        let win = self.make_window_affine();  // 16 affine precomputed multiples
        let mut ss = [0i32; (319 + Self::WINDOW) / Self::WINDOW];
        s.recode_signed(&mut ss, Self::WINDOW as i32);
        let n = ss.len() - 1;
        *self = PointAffine::lookup(&win, ss[n]).to_point();
        for i in (0..n).rev() {
            self.set_mdouble(Self::WINDOW as u32);
            *self += PointAffine::lookup(&win, ss[i]);
        }
    }

    /// Multiply the conventional generator by a scalar.
    /// FASTER than using * on GENERATOR — uses 8 precomputed tables.
    /// Tables cover 40-bit windows: G0, G40, G80, G120, G160, G200, G240, G280
    /// Gk[i] = (i+1)*(2^k)*G for i = 0..15
    pub fn mulgen(s: Scalar) -> Self {
        let mut ss = [0i32; 64];
        s.recode_signed(&mut ss, 5);
        let mut P = PointAffine::lookup(&G0, ss[7]).to_point();
        P += PointAffine::lookup(&G40, ss[15]);
        P += PointAffine::lookup(&G80, ss[23]);
        P += PointAffine::lookup(&G120, ss[31]);
        P += PointAffine::lookup(&G160, ss[39]);
        P += PointAffine::lookup(&G200, ss[47]);
        P += PointAffine::lookup(&G240, ss[55]);
        P += PointAffine::lookup(&G280, ss[63]);
        for i in (0..7).rev() {
            P.set_mdouble(5);
            P += PointAffine::lookup(&G0,   ss[i]);
            P += PointAffine::lookup(&G40,  ss[i + 8]);
            P += PointAffine::lookup(&G80,  ss[i + 16]);
            P += PointAffine::lookup(&G120, ss[i + 24]);
            P += PointAffine::lookup(&G160, ss[i + 32]);
            P += PointAffine::lookup(&G200, ss[i + 40]);
            P += PointAffine::lookup(&G240, ss[i + 48]);
            P += PointAffine::lookup(&G280, ss[i + 56]);
        }
        P
    }

    /// Schnorr signature verification: verify s*G + k*Q = R.
    /// Uses Lagrange decomposition of k for efficiency.
    /// NOT constant-time — use only on public data.
    pub fn verify_muladd_vartime(self, s: Scalar, k: Scalar, R: Self) -> bool {
        let (c0, c1) = k.lagrange();
        let t = s * c1.to_scalar_vartime();
        let mut tt = [0i32; 64];
        t.recode_signed(&mut tt, 5);
        let tt0 = &tt[..32];
        let tt1 = &tt[32..];
        let ss0 = c0.recode_signed_5();
        let ss1 = c1.recode_signed_5();
        let winQ = self.make_window_5();
        let winR = (-R).make_window_5();
        let mut P = Self::lookup_vartime(&winQ, ss0[32]);
        if ss1[32] != 0 { P += Self::lookup_vartime(&winR, ss1[32]); }
        for i in (0..32).rev() {
            P.set_mdouble(5);
            if tt0[i] != 0 { P += PointAffine::lookup_vartime(&G0,   tt0[i]); }
            if tt1[i] != 0 { P += PointAffine::lookup_vartime(&G160, tt1[i]); }
            if ss0[i] != 0 { P += Self::lookup_vartime(&winQ, ss0[i]); }
            if ss1[i] != 0 { P += Self::lookup_vartime(&winR, ss1[i]); }
        }
        P.isneutral() != 0
    }
}
```

### PointAffine struct

```rust
// Internal type for precomputed windows in scalar multiplication.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PointAffine {
    pub(crate) x: GFp5,
    pub(crate) u: GFp5,
}

impl PointAffine {
    const NEUTRAL: Self = Self { x: GFp5::ZERO, u: GFp5::ZERO };

    fn to_point(self) -> Point {
        Point { X: self.x, Z: GFp5::ONE, U: self.u, T: GFp5::ONE }
    }

    // Constant-time lookup in window [-n..+n] -> k*P.
    fn lookup(win: &[Self], k: i32) -> Self { ... }

    // Variable-time lookup (for public scalars only).
    fn lookup_vartime(win: &[Self], k: i32) -> Self { ... }
}
```

### Operator traits

Full `Add`, `Sub`, `Mul` (by `Scalar`), `Neg`, `AddAssign`, `SubAssign`, `MulAssign` are implemented for `Point`, `&Point`, `PointAffine`, and `&PointAffine` in all valid combinations.

---

## src/multab.rs — Precomputed Generator Tables

```rust
use super::field::GFp5;
use super::curve::PointAffine;

// For k = 40*j (j = 0 to 7), constant Gk[] is an array of 16 points
// in affine coordinates, with Gk[i] = (i+1)*(2^k)*G for generator G.

pub(crate) const G0: [PointAffine; 16] = [
    PointAffine {   /* 1*G */
        x: GFp5::from_u64_reduce(0xB2CA178ECF4453A1, 0x3C757788836D3EA4,
          0x48D7F28A26DAFD0B, 0x1E0F15C7FD44C28E, 0x21FA7FFCC8252211),
        u: GFp5::from_u64_reduce(0xBFFFFFFF40000001, 0x0000000000000000,
          0x0000000000000000, 0x0000000000000000, 0x0000000000000000),
    },
    // ... 15 more entries
];

pub(crate) const G40:  [PointAffine; 16] = [ ... ];  // (i+1)*(2^40)*G
pub(crate) const G80:  [PointAffine; 16] = [ ... ];  // (i+1)*(2^80)*G
pub(crate) const G120: [PointAffine; 16] = [ ... ];  // (i+1)*(2^120)*G
pub(crate) const G160: [PointAffine; 16] = [ ... ];  // (i+1)*(2^160)*G
pub(crate) const G200: [PointAffine; 16] = [ ... ];  // (i+1)*(2^200)*G
pub(crate) const G240: [PointAffine; 16] = [ ... ];  // (i+1)*(2^240)*G
pub(crate) const G280: [PointAffine; 16] = [ ... ];  // (i+1)*(2^280)*G
```

---

## Cargo.toml

```toml
[package]
name = "ecgfp5"
version = "0.1.0"
edition = "2018"

[dependencies]
# None! Zero external dependencies.

[[bench]]
name = "field"
path = "benches/field.rs"
harness = false

[[bench]]
name = "curve"
path = "benches/curve.rs"
harness = false

[[bench]]
name = "scalar"
path = "benches/scalar.rs"
harness = false
```

---

## API Summary

### Public types

| Type | Location | Description |
|------|----------|-------------|
| `GFp` | `field::GFp` | GF(p) element, p = 2^64 - 2^32 + 1 (Goldilocks prime) |
| `GFp5` | `field::GFp5` | GF(p^5) element, polynomial basis with z^5 = 3 |
| `Scalar` | `scalar::Scalar` | Integer mod n (319-bit group order) |
| `Point` | `curve::Point` | Curve point in (X:Z:U:T) fractional coordinates |
| `Signed161` | `scalar::Signed161` | 161-bit signed integer for Lagrange decomposition |

### Key API calls

```rust
// Field arithmetic
let a = GFp::from_u64_reduce(42_u64);
let b: GFp = a + a;
let c: GFp = a.square();
let (d, ok) = a.sqrt();          // ok = 0xFFFF... if QR
let e: GFp = a.invert();         // 0 if a == 0
let f: GFp = a.legendre();       // 0, 1, or -1

// Extension field
let x = GFp5::from_u64_reduce(1, 2, 3, 4, 5);
let y: GFp5 = x * x;
let z: GFp5 = x.invert();
let (s, ok) = x.sqrt();          // ok = 0xFFFF... if square
let bytes: [u8; 40] = x.encode();
let (x2, ok) = GFp5::decode(&bytes);

// Scalar
let s = Scalar::decode_reduce(&some_bytes);
let (s2, ok) = Scalar::decode(&exactly_40_bytes);
let bytes = s.encode();          // always 40 bytes
let (v0, v1) = s.lagrange();     // NOT constant-time

// Curve points
let G = Point::GENERATOR;
let N = Point::NEUTRAL;
let P = Point::mulgen(s);        // fast generator mul using tables
let Q = G * s;                   // slow (no precomputed tables)
let R = P + Q;
let ok = P.isneutral();
let ok = P.equals(Q);
let w: GFp5 = P.encode();        // w = y/x
let (P2, ok) = Point::decode(w);
let ok = Point::validate(w);
let verified = Q.verify_muladd_vartime(s, k, R);  // Schnorr verification
```

---

## Implementation Notes

### `no_std` Compatibility
YES. `#![no_std]` at crate root. Only uses `core::ops` and `core::convert::TryFrom`.

### `unsafe` Usage
NONE. Zero unsafe code anywhere in the codebase.

### Constant-Time Guarantees
- All secret-path operations in `GFp`, `GFp5`, `Scalar`, and `Point` are constant-time.
- `lagrange()`, `lookup_vartime()`, `verify_muladd_vartime()` are explicitly NOT constant-time and intended for public data only.
- Boolean "mask" convention: `true = 0xFFFFFFFFFFFFFFFF`, `false = 0x0000000000000000`.

### Montgomery Representation
- `GFp`: Values stored as `x*R mod p` where `R = 2^64 mod p`.
- `Scalar`: Values stored in normal (non-Montgomery) representation for efficient encode/decode.

### Performance Characteristics (Intel Coffee Lake)
- GFp multiplication: ~10-cycle latency (mulx + montyred)
- GFp5 multiplication: 25 GFp multiplications
- Point addition (complete): 10 GFp5 multiplications
- Point doubling: 4 GFp5 multiplications + 5 squarings
- `mulgen()`: ~2x faster than generic multiplication due to precomputed tables

### Curve Parameters
- Curve: `y^2 = x*(x^2 + 2*x + 263*z)` over GF(p^5)
- `a = 2`, `b = 263*z`
- Group order n: 319-bit prime
- Generator G: u = 13835058052060938241 (u = x/y, first GFp coefficient only)
- Generator encoding (w = y/x = 4): `GFp5::from_u64_reduce(4, 0, 0, 0, 0)` — but the GENERATOR constant stores full coordinates
- Neutral element: w = 0

---

## License

```
MIT License

Copyright (c) 2022 Thomas Pornin

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

---

## Sources

- [pornin/ecgfp5 on GitHub](https://github.com/pornin/ecgfp5)
- [EcGFp5 paper on ePrint](https://eprint.iacr.org/2022/274)
- [Lagrange decomposition algorithm](https://eprint.iacr.org/2020/454)
