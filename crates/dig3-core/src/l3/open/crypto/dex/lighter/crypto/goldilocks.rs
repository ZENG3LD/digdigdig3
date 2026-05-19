//! Goldilocks field GF(p) where p = 2^64 - 2^32 + 1.
//!
//! Uses Montgomery representation internally (R = 2^64).
//! All values are stored as x*R mod p, canonical range 0..p-1.
//! Constant-time operations throughout.

use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// An element of GF(p), p = 2^64 - 2^32 + 1.
/// Stored in Montgomery form: internal value = real_value * R mod p, where R = 2^64.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GFp(pub(crate) u64);

impl GFp {
    /// GF(p) modulus: p = 2^64 - 2^32 + 1
    pub const MOD: u64 = 0xFFFF_FFFF_0000_0001;

    /// Element 0
    pub const ZERO: GFp = GFp::from_u64_reduce(0);

    /// Element 1
    pub const ONE: GFp = GFp::from_u64_reduce(1);

    /// Element -1 mod p
    pub const MINUS_ONE: GFp = GFp::from_u64_reduce(GFp::MOD - 1);

    // R^2 mod p = 2^128 mod p = 0xFFFFFFFE00000001
    const R2: u64 = 0xFFFF_FFFE_0000_0001;

    /// Montgomery reduction: given x in [0, p*2^64 - 1], returns x/2^64 mod p.
    /// The result is in [0, p-1].
    #[inline(always)]
    pub(crate) const fn montyred(x: u128) -> u64 {
        // For p = 2^64 - 2^32 + 1 with R = 2^64:
        // Write x = xl + xh*2^64 where xl = x mod 2^64, xh = x >> 64
        // Then x/R mod p = xh - xl*(2^32 - 1)/2^64 mod p
        // But the efficient Goldilocks reduction uses the special structure of p.
        let xl = x as u64;
        let xh = (x >> 64) as u64;
        // Compute xl * (R/p) = xl * (2^32 - 1) but in a special way
        let (a, e) = xl.overflowing_add(xl << 32);
        let b = a.wrapping_sub(a >> 32).wrapping_sub(e as u64);
        let (r, c) = xh.overflowing_sub(b);
        r.wrapping_sub(0u32.wrapping_sub(c as u32) as u64)
    }

    /// Create from a u64 integer with implicit reduction mod p.
    /// Uses Montgomery form: stores v * R mod p.
    #[inline(always)]
    pub const fn from_u64_reduce(v: u64) -> GFp {
        GFp(GFp::montyred((v as u128) * (GFp::R2 as u128)))
    }

    /// Create from u64, returning (element, mask).
    /// If v < p: element = v as GFp, mask = 0xFFFF_FFFF_FFFF_FFFF.
    /// If v >= p: element = ZERO, mask = 0.
    pub fn from_u64_checked(v: u64) -> (GFp, u64) {
        let z = v.wrapping_sub(GFp::MOD);
        let c = ((v & !z) >> 63).wrapping_sub(1);
        (GFp::from_u64_reduce(v & c), c)
    }

    /// Decode from a raw u64 value (canonical form, must be < p).
    /// Converts to Montgomery form. Used for field element construction from transaction fields.
    #[inline(always)]
    pub fn from_canonical_u64(v: u64) -> GFp {
        // For transaction field encoding, values come in as raw u64 bit patterns.
        // We trust they are < p (Goldilocks field is large enough for all tx fields).
        GFp::from_u64_reduce(v)
    }

    /// Get the canonical integer representation in [0, p-1].
    #[inline(always)]
    pub const fn to_u64(self) -> u64 {
        GFp::montyred(self.0 as u128)
    }

    /// Encode to 8 bytes little-endian (canonical form).
    #[inline]
    pub fn to_le_bytes(self) -> [u8; 8] {
        self.to_u64().to_le_bytes()
    }

    /// Decode from 8 bytes little-endian. Returns (element, mask).
    /// mask = 0xFFFF_FFFF_FFFF_FFFF if valid (value < p), else 0.
    pub fn from_le_bytes(b: &[u8; 8]) -> (GFp, u64) {
        let v = u64::from_le_bytes(*b);
        GFp::from_u64_checked(v)
    }

    // --- Field arithmetic (Montgomery form) ---

    /// Addition mod p.
    #[inline(always)]
    const fn add(self, rhs: Self) -> Self {
        // self.0 and rhs.0 are both in [0, p-1]
        // We want (self.0 + rhs.0) mod p
        // Trick: compute x1 = self.0 - (p - rhs.0) which might underflow
        let (x1, c1) = self.0.overflowing_sub(GFp::MOD - rhs.0);
        let adj = 0u32.wrapping_sub(c1 as u32);
        GFp(x1.wrapping_sub(adj as u64))
    }

    /// Subtraction mod p.
    #[inline(always)]
    const fn sub(self, rhs: Self) -> Self {
        let (x1, c1) = self.0.overflowing_sub(rhs.0);
        let adj = 0u32.wrapping_sub(c1 as u32);
        GFp(x1.wrapping_sub(adj as u64))
    }

    /// Negation mod p.
    #[inline(always)]
    pub const fn neg(self) -> Self {
        GFp::ZERO.sub(self)
    }

    /// Halving (multiply by 2^{-1} mod p).
    #[inline(always)]
    pub const fn half(self) -> Self {
        GFp((self.0 >> 1).wrapping_add(
            (self.0 & 1).wrapping_neg() & 0x7FFF_FFFF_8000_0001,
        ))
    }

    /// Doubling (multiply by 2).
    #[inline(always)]
    pub const fn double(self) -> Self {
        self.add(self)
    }

    /// Multiplication by small integer (< 2^31). Faster than full Montgomery mul.
    #[inline(always)]
    pub const fn mul_small(self, rhs: u32) -> Self {
        let x = (self.0 as u128) * (rhs as u128);
        let xl = x as u64;
        let xh = (x >> 64) as u64;
        let (r, c) = xl.overflowing_sub(GFp::MOD - ((xh << 32) - xh));
        GFp(r.wrapping_sub(0u32.wrapping_sub(c as u32) as u64))
    }

    /// Full Montgomery multiplication.
    #[inline(always)]
    pub(crate) const fn mul(self, rhs: Self) -> Self {
        GFp(GFp::montyred((self.0 as u128) * (rhs.0 as u128)))
    }

    /// Squaring.
    #[inline(always)]
    pub const fn square(self) -> Self {
        self.mul(self)
    }

    /// Repeated squaring: return self^(2^n).
    pub fn msquare(self, n: u32) -> Self {
        let mut x = self;
        for _ in 0..n {
            x = x.square();
        }
        x
    }

    /// Inversion using Fermat's little theorem: 1/x = x^(p-2) mod p.
    /// p - 2 = 0xFFFF_FFFE_FFFF_FFFF
    /// Returns 0 if self == 0.
    pub fn invert(self) -> Self {
        // Addition chain for p-2 = 2^64 - 2^32 - 1
        // From Pornin's ecgfp5 reference implementation
        let x = self;
        let x2 = x * x.square();         // x^(2^2 - 1)  = x^3
        let x4 = x2 * x2.msquare(2);     // x^(2^4 - 1)  = x^15
        let x5 = x * x4.square();        // x^(2^5 - 1)  = x^31
        let x10 = x5 * x5.msquare(5);    // x^(2^10 - 1)
        let x15 = x5 * x10.msquare(5);   // x^(2^15 - 1)
        let x16 = x * x15.square();      // x^(2^16 - 1)
        let x31 = x15 * x16.msquare(15); // x^(2^31 - 1)
        let x32 = x * x31.square();      // x^(2^32 - 1)
        x32 * x31.msquare(33)            // x^(2^64 - 2^32 - 1) = x^(p-2)
    }

    /// Legendre symbol: returns x^((p-1)/2).
    /// Result is: 0 if x==0, ONE if QR, MINUS_ONE if non-QR.
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
    // g = 7^(2^32-1) mod p = 1753635133440165772 (primitive 2^32-th root of unity).
    // GG[i] = g^(2^i) for i = 0 to 31.
    pub(crate) const GG: [GFp; 32] = [
        GFp::from_u64_reduce(1753635133440165772),
        GFp::from_u64_reduce(4614640910117430873),
        GFp::from_u64_reduce(9123114210336311365),
        GFp::from_u64_reduce(16116352524544190054),
        GFp::from_u64_reduce(6414415596519834757),
        GFp::from_u64_reduce(1213594585890690845),
        GFp::from_u64_reduce(17096174751763063430),
        GFp::from_u64_reduce(5456943929260765144),
        GFp::from_u64_reduce(9713644485405565297),
        GFp::from_u64_reduce(16905767614792059275),
        GFp::from_u64_reduce(5416168637041100469),
        GFp::from_u64_reduce(17654865857378133588),
        GFp::from_u64_reduce(3511170319078647661),
        GFp::from_u64_reduce(18146160046829613826),
        GFp::from_u64_reduce(9306717745644682924),
        GFp::from_u64_reduce(12380578893860276750),
        GFp::from_u64_reduce(6115771955107415310),
        GFp::from_u64_reduce(17776499369601055404),
        GFp::from_u64_reduce(16207902636198568418),
        GFp::from_u64_reduce(1532612707718625687),
        GFp::from_u64_reduce(17492915097719143606),
        GFp::from_u64_reduce(455906449640507599),
        GFp::from_u64_reduce(11353340290879379826),
        GFp::from_u64_reduce(1803076106186727246),
        GFp::from_u64_reduce(13797081185216407910),
        GFp::from_u64_reduce(17870292113338400769),
        GFp::from_u64_reduce(549755813888),
        GFp::from_u64_reduce(70368744161280),
        GFp::from_u64_reduce(17293822564807737345),
        GFp::from_u64_reduce(18446744069397807105),
        GFp::from_u64_reduce(281474976710656),
        GFp::from_u64_reduce(18446744069414584320),
    ];

    /// Square root in GF(p). Returns (sqrt, mask).
    /// mask = 0xFFFF_FFFF_FFFF_FFFF if self is a QR, else 0.
    /// Uses constant-time Tonelli-Shanks (p = q*2^32 + 1, q = 2^32 - 1).
    pub fn sqrt(self) -> (Self, u64) {
        let x = self;
        // Compute x^((q+1)/2) = x^(2^31) as starting point
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

    /// Returns 0xFFFF_FFFF_FFFF_FFFF if self == 0, else 0.
    #[inline(always)]
    pub const fn iszero(self) -> u64 {
        !((((self.0 | self.0.wrapping_neg()) as i64) >> 63) as u64)
    }

    /// Returns 0xFFFF_FFFF_FFFF_FFFF if self == ONE, else 0.
    #[inline(always)]
    pub const fn isone(self) -> u64 {
        self.equals(GFp::ONE)
    }

    /// Returns 0xFFFF_FFFF_FFFF_FFFF if self == MINUS_ONE, else 0.
    #[inline(always)]
    pub const fn isminusone(self) -> u64 {
        self.equals(GFp::MINUS_ONE)
    }

    /// Returns 0xFFFF_FFFF_FFFF_FFFF if self == rhs, else 0.
    #[inline(always)]
    pub const fn equals(self, rhs: Self) -> u64 {
        let t = self.0 ^ rhs.0;
        !((((t | t.wrapping_neg()) as i64) >> 63) as u64)
    }

    /// Constant-time select: returns x0 if c == 0, x1 if c == 0xFFFF_FFFF_FFFF_FFFF.
    #[inline(always)]
    pub fn select(c: u64, x0: GFp, x1: GFp) -> GFp {
        GFp(x0.0 ^ (c & (x0.0 ^ x1.0)))
    }
}

// --- Operator trait implementations ---

impl Add for GFp {
    type Output = GFp;
    #[inline(always)]
    fn add(self, rhs: GFp) -> GFp { GFp::add(self, rhs) }
}
impl Add<&GFp> for GFp {
    type Output = GFp;
    #[inline(always)]
    fn add(self, rhs: &GFp) -> GFp { GFp::add(self, *rhs) }
}
impl AddAssign for GFp {
    #[inline(always)]
    fn add_assign(&mut self, rhs: GFp) { *self = GFp::add(*self, rhs); }
}
impl AddAssign<&GFp> for GFp {
    #[inline(always)]
    fn add_assign(&mut self, rhs: &GFp) { *self = GFp::add(*self, *rhs); }
}

impl Sub for GFp {
    type Output = GFp;
    #[inline(always)]
    fn sub(self, rhs: GFp) -> GFp { GFp::sub(self, rhs) }
}
impl Sub<&GFp> for GFp {
    type Output = GFp;
    #[inline(always)]
    fn sub(self, rhs: &GFp) -> GFp { GFp::sub(self, *rhs) }
}
impl SubAssign for GFp {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: GFp) { *self = GFp::sub(*self, rhs); }
}
impl SubAssign<&GFp> for GFp {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: &GFp) { *self = GFp::sub(*self, *rhs); }
}

impl Mul for GFp {
    type Output = GFp;
    #[inline(always)]
    fn mul(self, rhs: GFp) -> GFp { GFp::mul(self, rhs) }
}
impl Mul<&GFp> for GFp {
    type Output = GFp;
    #[inline(always)]
    fn mul(self, rhs: &GFp) -> GFp { GFp::mul(self, *rhs) }
}
impl MulAssign for GFp {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: GFp) { *self = GFp::mul(*self, rhs); }
}
impl MulAssign<&GFp> for GFp {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: &GFp) { *self = GFp::mul(*self, *rhs); }
}

impl Neg for GFp {
    type Output = GFp;
    #[inline(always)]
    fn neg(self) -> GFp { GFp::neg(self) }
}
impl Neg for &GFp {
    type Output = GFp;
    #[inline(always)]
    fn neg(self) -> GFp { GFp::neg(*self) }
}

/// Compute x^7 for S-box (Poseidon2).
pub fn pow7(x: GFp) -> GFp {
    let x2 = x.square();
    let x4 = x2.square();
    x4 * x2 * x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_arithmetic() {
        let a = GFp::from_u64_reduce(5);
        let b = GFp::from_u64_reduce(3);
        let c = a * b;
        assert_eq!(c.to_u64(), 15, "5 * 3 = 15");
    }

    #[test]
    fn test_inverse() {
        let a = GFp::from_u64_reduce(5);
        let inv_a = a.invert();
        let product = a * inv_a;
        assert_eq!(product.to_u64(), 1, "5 * inv(5) = 1");
    }

    #[test]
    fn test_zero_inverse() {
        let z = GFp::ZERO;
        let inv_z = z.invert();
        assert_eq!(inv_z.to_u64(), 0, "inv(0) = 0");
    }

    #[test]
    fn test_add_sub() {
        let a = GFp::from_u64_reduce(100);
        let b = GFp::from_u64_reduce(200);
        let s = a + b;
        let d = s - b;
        assert_eq!(d.to_u64(), a.to_u64(), "add then sub roundtrip");
    }

    #[test]
    fn test_sqrt() {
        // 4 is a perfect square: sqrt(4) = 2
        let a = GFp::from_u64_reduce(4);
        let (s, mask) = a.sqrt();
        assert_eq!(mask, 0xFFFF_FFFF_FFFF_FFFF, "4 is a QR");
        // sqrt can return either 2 or p-2
        let v = s.to_u64();
        assert!(v == 2 || v == GFp::MOD - 2, "sqrt(4) = ±2, got {}", v);
        assert_eq!((s * s).to_u64(), 4, "sqrt(4)^2 = 4");
    }

    #[test]
    fn test_pow7() {
        let a = GFp::from_u64_reduce(3);
        let a7 = pow7(a);
        // 3^7 = 2187
        assert_eq!(a7.to_u64(), 2187, "3^7 = 2187");
    }

    #[test]
    fn test_from_canonical_roundtrip() {
        let v: u64 = 12345678901234567u64;
        let e = GFp::from_canonical_u64(v);
        assert_eq!(e.to_u64(), v, "roundtrip through Montgomery form");
    }
}
