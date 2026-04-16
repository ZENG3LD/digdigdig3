//! GF(p^5) — quintic extension of the Goldilocks field.
//!
//! Elements are polynomials a[0] + a[1]*z + a[2]*z^2 + a[3]*z^3 + a[4]*z^4
//! where z^5 = 3 (the reduction polynomial is x^5 - 3).

use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use super::goldilocks::GFp;

/// An element of GF(p^5), represented as a degree-4 polynomial over GF(p).
/// Internal Montgomery form is used (inherits from GFp).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GFp5(pub [GFp; 5]);

impl GFp5 {
    /// The zero element.
    pub const ZERO: GFp5 = GFp5([GFp::ZERO; 5]);

    /// The one element.
    pub const ONE: GFp5 = GFp5([GFp::ONE, GFp::ZERO, GFp::ZERO, GFp::ZERO, GFp::ZERO]);

    /// Construct from five u64 values, each implicitly reduced mod p.
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

    /// Construct from a GFp base field element (sets higher coefficients to 0).
    #[inline(always)]
    pub fn from_base(x: GFp) -> Self {
        GFp5([x, GFp::ZERO, GFp::ZERO, GFp::ZERO, GFp::ZERO])
    }

    /// Decode from exactly 40 bytes (5 little-endian u64 values, each < p).
    /// Returns (element, mask). mask = 0xFFFF_FFFF_FFFF_FFFF if valid, 0 otherwise.
    pub fn decode(buf: &[u8]) -> (Self, u64) {
        if buf.len() != 40 {
            return (GFp5::ZERO, 0);
        }
        let mut arr = [0u64; 5];
        for i in 0..5 {
            arr[i] = u64::from_le_bytes(buf[8 * i..8 * i + 8].try_into().unwrap());
        }
        let mut ok = !0u64;
        let mut coeffs = [GFp::ZERO; 5];
        for i in 0..5 {
            let (c, m) = GFp::from_u64_checked(arr[i]);
            coeffs[i] = c;
            ok &= m;
        }
        (GFp5(coeffs), ok)
    }

    /// Decode from a fixed 40-byte slice.
    pub fn from_le_bytes(buf: &[u8; 40]) -> (Self, u64) {
        GFp5::decode(buf)
    }

    /// Encode to 40 bytes (5 little-endian u64 values in canonical form).
    pub fn encode(self) -> [u8; 40] {
        let mut r = [0u8; 40];
        for i in 0..5 {
            r[8 * i..8 * i + 8].copy_from_slice(&self.0[i].to_u64().to_le_bytes());
        }
        r
    }

    /// Return the underlying 5 canonical u64 values.
    pub fn to_basefield_array(self) -> [u64; 5] {
        [
            self.0[0].to_u64(),
            self.0[1].to_u64(),
            self.0[2].to_u64(),
            self.0[3].to_u64(),
            self.0[4].to_u64(),
        ]
    }

    // --- Arithmetic ---

    #[inline]
    pub(crate) fn set_add(&mut self, rhs: &Self) {
        for i in 0..5 {
            self.0[i] += rhs.0[i];
        }
    }

    #[inline]
    pub(crate) fn set_sub(&mut self, rhs: &Self) {
        for i in 0..5 {
            self.0[i] -= rhs.0[i];
        }
    }

    #[inline]
    pub(crate) fn set_neg(&mut self) {
        for i in 0..5 {
            self.0[i] = -self.0[i];
        }
    }

    /// Multiply by a small constant (< 2^31).
    pub fn mul_small(self, rhs: u32) -> Self {
        GFp5([
            self.0[0].mul_small(rhs),
            self.0[1].mul_small(rhs),
            self.0[2].mul_small(rhs),
            self.0[3].mul_small(rhs),
            self.0[4].mul_small(rhs),
        ])
    }

    /// Multiply by a base field element (coefficient 0 only).
    pub fn mul_k0(self, rhs: GFp) -> GFp5 {
        GFp5([
            self.0[0] * rhs,
            self.0[1] * rhs,
            self.0[2] * rhs,
            self.0[3] * rhs,
            self.0[4] * rhs,
        ])
    }

    /// Multiply by v*z where v is a small integer (< 2^29).
    /// Uses z^5 = 3: z * a[4]*z^4 = 3*a[4].
    pub fn mul_small_k1(self, rhs: u32) -> Self {
        GFp5([
            self.0[4].mul_small(rhs * 3),
            self.0[0].mul_small(rhs),
            self.0[1].mul_small(rhs),
            self.0[2].mul_small(rhs),
            self.0[3].mul_small(rhs),
        ])
    }

    // GF(p^5) multiplication coefficient helpers.
    // c[k] = sum over i+j==k of a[i]*b[j] + 3 * sum over i+j==k+5 of a[i]*b[j]
    // The z^5 = 3 reduction means cross terms of degree >= 5 pick up factor 3.

    #[inline(always)]
    fn mul_to_k0(&self, rhs: &Self) -> GFp {
        // c[0] = a0*b0 + 3*(a1*b4 + a2*b3 + a3*b2 + a4*b1)
        let pp0 = (self.0[0].0 as u128) * (rhs.0[0].0 as u128);
        let pp1 = (self.0[1].0 as u128) * (rhs.0[4].0 as u128);
        let pp2 = (self.0[2].0 as u128) * (rhs.0[3].0 as u128);
        let pp3 = (self.0[3].0 as u128) * (rhs.0[2].0 as u128);
        let pp4 = (self.0[4].0 as u128) * (rhs.0[1].0 as u128);
        let zhi = (pp0 >> 64)
            + 3 * ((pp1 >> 64) + (pp2 >> 64) + (pp3 >> 64) + (pp4 >> 64));
        let zlo = (pp0 as u64 as u128)
            + 3 * ((pp1 as u64 as u128)
                + (pp2 as u64 as u128)
                + (pp3 as u64 as u128)
                + (pp4 as u64 as u128));
        GFp(GFp::montyred(zlo + (zhi << 32) - zhi))
    }

    #[inline(always)]
    fn mul_to_k1(&self, rhs: &Self) -> GFp {
        // c[1] = a0*b1 + a1*b0 + 3*(a2*b4 + a3*b3 + a4*b2)
        let pp0 = (self.0[0].0 as u128) * (rhs.0[1].0 as u128);
        let pp1 = (self.0[1].0 as u128) * (rhs.0[0].0 as u128);
        let pp2 = (self.0[2].0 as u128) * (rhs.0[4].0 as u128);
        let pp3 = (self.0[3].0 as u128) * (rhs.0[3].0 as u128);
        let pp4 = (self.0[4].0 as u128) * (rhs.0[2].0 as u128);
        let zhi = (pp0 >> 64) + (pp1 >> 64) + 3 * ((pp2 >> 64) + (pp3 >> 64) + (pp4 >> 64));
        let zlo = (pp0 as u64 as u128)
            + (pp1 as u64 as u128)
            + 3 * ((pp2 as u64 as u128) + (pp3 as u64 as u128) + (pp4 as u64 as u128));
        GFp(GFp::montyred(zlo + (zhi << 32) - zhi))
    }

    #[inline(always)]
    fn mul_to_k2(&self, rhs: &Self) -> GFp {
        // c[2] = a0*b2 + a1*b1 + a2*b0 + 3*(a3*b4 + a4*b3)
        let pp0 = (self.0[0].0 as u128) * (rhs.0[2].0 as u128);
        let pp1 = (self.0[1].0 as u128) * (rhs.0[1].0 as u128);
        let pp2 = (self.0[2].0 as u128) * (rhs.0[0].0 as u128);
        let pp3 = (self.0[3].0 as u128) * (rhs.0[4].0 as u128);
        let pp4 = (self.0[4].0 as u128) * (rhs.0[3].0 as u128);
        let zhi = (pp0 >> 64)
            + (pp1 >> 64)
            + (pp2 >> 64)
            + 3 * ((pp3 >> 64) + (pp4 >> 64));
        let zlo = (pp0 as u64 as u128)
            + (pp1 as u64 as u128)
            + (pp2 as u64 as u128)
            + 3 * ((pp3 as u64 as u128) + (pp4 as u64 as u128));
        GFp(GFp::montyred(zlo + (zhi << 32) - zhi))
    }

    #[inline(always)]
    fn mul_to_k3(&self, rhs: &Self) -> GFp {
        // c[3] = a0*b3 + a1*b2 + a2*b1 + a3*b0 + 3*(a4*b4)
        let pp0 = (self.0[0].0 as u128) * (rhs.0[3].0 as u128);
        let pp1 = (self.0[1].0 as u128) * (rhs.0[2].0 as u128);
        let pp2 = (self.0[2].0 as u128) * (rhs.0[1].0 as u128);
        let pp3 = (self.0[3].0 as u128) * (rhs.0[0].0 as u128);
        let pp4 = (self.0[4].0 as u128) * (rhs.0[4].0 as u128);
        let zhi = (pp0 >> 64) + (pp1 >> 64) + (pp2 >> 64) + (pp3 >> 64) + 3 * (pp4 >> 64);
        let zlo = (pp0 as u64 as u128)
            + (pp1 as u64 as u128)
            + (pp2 as u64 as u128)
            + (pp3 as u64 as u128)
            + 3 * (pp4 as u64 as u128);
        GFp(GFp::montyred(zlo + (zhi << 32) - zhi))
    }

    #[inline(always)]
    fn mul_to_k4(&self, rhs: &Self) -> GFp {
        // c[4] = a0*b4 + a1*b3 + a2*b2 + a3*b1 + a4*b0
        let pp0 = (self.0[0].0 as u128) * (rhs.0[4].0 as u128);
        let pp1 = (self.0[1].0 as u128) * (rhs.0[3].0 as u128);
        let pp2 = (self.0[2].0 as u128) * (rhs.0[2].0 as u128);
        let pp3 = (self.0[3].0 as u128) * (rhs.0[1].0 as u128);
        let pp4 = (self.0[4].0 as u128) * (rhs.0[0].0 as u128);
        let zhi = (pp0 >> 64) + (pp1 >> 64) + (pp2 >> 64) + (pp3 >> 64) + (pp4 >> 64);
        let zlo = (pp0 as u64 as u128)
            + (pp1 as u64 as u128)
            + (pp2 as u64 as u128)
            + (pp3 as u64 as u128)
            + (pp4 as u64 as u128);
        GFp(GFp::montyred(zlo + (zhi << 32) - zhi))
    }

    #[inline]
    pub(crate) fn set_mul(&mut self, rhs: &Self) {
        let d0 = self.mul_to_k0(rhs);
        let d1 = self.mul_to_k1(rhs);
        let d2 = self.mul_to_k2(rhs);
        let d3 = self.mul_to_k3(rhs);
        let d4 = self.mul_to_k4(rhs);
        self.0[0] = d0;
        self.0[1] = d1;
        self.0[2] = d2;
        self.0[3] = d3;
        self.0[4] = d4;
    }

    // Squaring: optimized using symmetry.
    // k0 = a0^2 + 6*(a1*a4 + a2*a3)
    // k1 = 2*a0*a1 + 6*a2*a4 + 3*a3^2
    // k2 = 2*a0*a2 + a1^2 + 6*a3*a4
    // k3 = 2*(a0*a3 + a1*a2) + 3*a4^2
    // k4 = 2*(a0*a4 + a1*a3) + a2^2

    #[inline(always)]
    fn square_to_k0(&self) -> GFp {
        let pp0 = (self.0[0].0 as u128) * (self.0[0].0 as u128);
        let pp1 = (self.0[1].0 as u128) * (self.0[4].0 as u128);
        let pp2 = (self.0[2].0 as u128) * (self.0[3].0 as u128);
        let zhi = (pp0 >> 64) + 6 * ((pp1 >> 64) + (pp2 >> 64));
        let zlo = (pp0 as u64 as u128) + 6 * ((pp1 as u64 as u128) + (pp2 as u64 as u128));
        GFp(GFp::montyred(zlo + (zhi << 32) - zhi))
    }

    #[inline(always)]
    fn square_to_k1(&self) -> GFp {
        let pp0 = (self.0[0].0 as u128) * (self.0[1].0 as u128);
        let pp2 = (self.0[2].0 as u128) * (self.0[4].0 as u128);
        let pp3 = (self.0[3].0 as u128) * (self.0[3].0 as u128);
        let zhi = 2 * (pp0 >> 64) + 6 * (pp2 >> 64) + 3 * (pp3 >> 64);
        let zlo =
            2 * (pp0 as u64 as u128) + 6 * (pp2 as u64 as u128) + 3 * (pp3 as u64 as u128);
        GFp(GFp::montyred(zlo + (zhi << 32) - zhi))
    }

    #[inline(always)]
    fn square_to_k2(&self) -> GFp {
        let pp0 = (self.0[0].0 as u128) * (self.0[2].0 as u128);
        let pp1 = (self.0[1].0 as u128) * (self.0[1].0 as u128);
        let pp3 = (self.0[3].0 as u128) * (self.0[4].0 as u128);
        let zhi = 2 * (pp0 >> 64) + (pp1 >> 64) + 6 * (pp3 >> 64);
        let zlo = 2 * (pp0 as u64 as u128) + (pp1 as u64 as u128) + 6 * (pp3 as u64 as u128);
        GFp(GFp::montyred(zlo + (zhi << 32) - zhi))
    }

    #[inline(always)]
    fn square_to_k3(&self) -> GFp {
        let pp0 = (self.0[0].0 as u128) * (self.0[3].0 as u128);
        let pp1 = (self.0[1].0 as u128) * (self.0[2].0 as u128);
        let pp4 = (self.0[4].0 as u128) * (self.0[4].0 as u128);
        let zhi = 2 * ((pp0 >> 64) + (pp1 >> 64)) + 3 * (pp4 >> 64);
        let zlo =
            2 * ((pp0 as u64 as u128) + (pp1 as u64 as u128)) + 3 * (pp4 as u64 as u128);
        GFp(GFp::montyred(zlo + (zhi << 32) - zhi))
    }

    #[inline(always)]
    fn square_to_k4(&self) -> GFp {
        let pp0 = (self.0[0].0 as u128) * (self.0[4].0 as u128);
        let pp1 = (self.0[1].0 as u128) * (self.0[3].0 as u128);
        let pp2 = (self.0[2].0 as u128) * (self.0[2].0 as u128);
        let zhi = 2 * ((pp0 >> 64) + (pp1 >> 64)) + (pp2 >> 64);
        let zlo = 2 * ((pp0 as u64 as u128) + (pp1 as u64 as u128)) + (pp2 as u64 as u128);
        GFp(GFp::montyred(zlo + (zhi << 32) - zhi))
    }

    /// Squaring.
    pub fn square(self) -> Self {
        GFp5([
            self.square_to_k0(),
            self.square_to_k1(),
            self.square_to_k2(),
            self.square_to_k3(),
            self.square_to_k4(),
        ])
    }

    /// Repeated squaring: self^(2^n).
    pub fn msquare(self, n: u32) -> Self {
        let mut r = self;
        for _ in 0..n {
            r = r.square();
        }
        r
    }

    // Frobenius endomorphisms (raise to power p, p^2).
    // These multiply coefficient k by the k-th power of the Frobenius image of z.
    // Constants: DTH_ROOT = 1041288259238279555, frob2 variants.
    pub(crate) fn set_frob1(&mut self) {
        self.0[1] *= GFp::from_u64_reduce(1_041_288_259_238_279_555);
        self.0[2] *= GFp::from_u64_reduce(15_820_824_984_080_659_046);
        self.0[3] *= GFp::from_u64_reduce(211_587_555_138_949_697);
        self.0[4] *= GFp::from_u64_reduce(1_373_043_270_956_696_022);
    }

    pub(crate) fn set_frob2(&mut self) {
        self.0[1] *= GFp::from_u64_reduce(15_820_824_984_080_659_046);
        self.0[2] *= GFp::from_u64_reduce(1_373_043_270_956_696_022);
        self.0[3] *= GFp::from_u64_reduce(1_041_288_259_238_279_555);
        self.0[4] *= GFp::from_u64_reduce(211_587_555_138_949_697);
    }

    /// Inversion using the Itoh-Tsujii algorithm.
    /// Returns 0 if self == 0.
    pub fn invert(self) -> Self {
        let mut r = self;
        r.set_invert();
        r
    }

    pub(crate) fn set_invert(&mut self) {
        // x^(r-1) = x^(p + p^2 + p^3 + p^4) via Frobenius composition.
        let t0 = *self;
        self.set_frob1();
        let mut t1 = *self;
        t1.set_frob1();
        self.set_mul(&t1);
        t1 = *self;
        self.set_frob2();
        self.set_mul(&t1);
        // Now *self = x^(p+p^2+p^3+p^4), and x * (*self) = x^r (in GF(p))
        let x_norm = self.mul_to_k0(&t0); // norm = x^r in base field
        self.set_mul_k0(x_norm.invert());
    }

    pub(crate) fn set_mul_k0(&mut self, rhs: GFp) {
        for i in 0..5 {
            self.0[i] *= rhs;
        }
    }

    /// Legendre symbol: returns x^((p^5-1)/2) as a base field element.
    /// 0 if x==0, ONE if square, MINUS_ONE if non-square.
    pub fn legendre(self) -> GFp {
        let mut t0 = self;
        t0.set_frob1();
        let mut t1 = t0;
        t1.set_frob1();
        t0.set_mul(&t1);
        t1 = t0;
        t1.set_frob2();
        t0.set_mul(&t1);
        // t0 = self^(p+p^2+p^3+p^4), t0 * self = self^r (in GF(p))
        let x = self.mul_to_k0(&t0);
        x.legendre()
    }

    /// Square root in GF(p^5). Returns (sqrt, mask).
    pub fn sqrt(self) -> (GFp5, u64) {
        let mut r = self;
        let c = r.set_sqrt();
        (r, c)
    }

    pub(crate) fn set_sqrt(&mut self) -> u64 {
        // sqrt(x) = sqrt(x^r) / x^((r-1)/2)
        // where x^r is in GF(p)
        let mut t = *self;
        // Compute x^((p+1)/2) = x^(2^31) steps
        let mut y = t;
        y = y.square();
        t.set_mul(&y);             // t = x^3
        y = t;
        y = y.msquare(2);
        t.set_mul(&y);             // t = x^(2^4-1)
        y = t;
        y = y.msquare(4);
        t.set_mul(&y);             // t = x^(2^8-1)
        y = t;
        y = y.msquare(8);
        t.set_mul(&y);             // t = x^(2^16-1)
        y = t;
        y = y.msquare(16);
        t.set_mul(&y);             // t = x^(2^32-1)
        t = t.msquare(31);
        let mut tmp = t;
        tmp.set_mul(self);         // tmp = x^((p+1)/2)
        y = tmp;
        y.set_frob2();
        tmp.set_mul(&y);
        tmp.set_frob1();           // tmp = x^((r-1)/2)
        y = tmp;
        y = y.square();
        let a = self.mul_to_k0(&y); // a = x^r in GF(p)
        let (s, cc) = a.sqrt();
        *self = tmp;
        self.set_invert();
        self.set_mul_k0(s);
        cc
    }

    /// Returns 0xFFFF_FFFF_FFFF_FFFF if self == 0, else 0.
    #[inline]
    pub fn iszero(self) -> u64 {
        let z = self.0[0].0 | self.0[1].0 | self.0[2].0 | self.0[3].0 | self.0[4].0;
        !((((z | z.wrapping_neg()) as i64) >> 63) as u64)
    }

    /// Returns 0xFFFF_FFFF_FFFF_FFFF if self == rhs, else 0.
    #[inline]
    pub fn equals(self, rhs: Self) -> u64 {
        let z = (self.0[0].0 ^ rhs.0[0].0)
            | (self.0[1].0 ^ rhs.0[1].0)
            | (self.0[2].0 ^ rhs.0[2].0)
            | (self.0[3].0 ^ rhs.0[3].0)
            | (self.0[4].0 ^ rhs.0[4].0);
        !((((z | z.wrapping_neg()) as i64) >> 63) as u64)
    }

    /// Constant-time select: returns x0 if c==0, x1 if c==0xFFFF_FFFF_FFFF_FFFF.
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

    /// Partial lookup (for windowed scalar multiplication).
    /// If c == 0xFFFF..., sets self = y. If c == 0, self should be zero.
    #[inline(always)]
    pub(crate) fn set_partial_lookup(&mut self, y: GFp5, c: u64) {
        self.0[0].0 |= c & y.0[0].0;
        self.0[1].0 |= c & y.0[1].0;
        self.0[2].0 |= c & y.0[2].0;
        self.0[3].0 |= c & y.0[3].0;
        self.0[4].0 |= c & y.0[4].0;
    }

    /// Halving (divide by 2).
    pub fn half(self) -> Self {
        GFp5([
            self.0[0].half(),
            self.0[1].half(),
            self.0[2].half(),
            self.0[3].half(),
            self.0[4].half(),
        ])
    }

    /// Doubling.
    pub fn double(self) -> Self {
        GFp5([
            self.0[0].double(),
            self.0[1].double(),
            self.0[2].double(),
            self.0[3].double(),
            self.0[4].double(),
        ])
    }
}

// --- Operator trait implementations ---

impl Add for GFp5 {
    type Output = GFp5;
    fn add(mut self, rhs: GFp5) -> GFp5 {
        self.set_add(&rhs);
        self
    }
}
impl Add<&GFp5> for GFp5 {
    type Output = GFp5;
    fn add(mut self, rhs: &GFp5) -> GFp5 {
        self.set_add(rhs);
        self
    }
}
impl AddAssign for GFp5 {
    fn add_assign(&mut self, rhs: GFp5) { self.set_add(&rhs); }
}
impl AddAssign<&GFp5> for GFp5 {
    fn add_assign(&mut self, rhs: &GFp5) { self.set_add(rhs); }
}

impl Sub for GFp5 {
    type Output = GFp5;
    fn sub(mut self, rhs: GFp5) -> GFp5 {
        self.set_sub(&rhs);
        self
    }
}
impl Sub<&GFp5> for GFp5 {
    type Output = GFp5;
    fn sub(mut self, rhs: &GFp5) -> GFp5 {
        self.set_sub(rhs);
        self
    }
}
impl SubAssign for GFp5 {
    fn sub_assign(&mut self, rhs: GFp5) { self.set_sub(&rhs); }
}

impl Mul for GFp5 {
    type Output = GFp5;
    fn mul(mut self, rhs: GFp5) -> GFp5 {
        self.set_mul(&rhs);
        self
    }
}
impl Mul<&GFp5> for GFp5 {
    type Output = GFp5;
    fn mul(mut self, rhs: &GFp5) -> GFp5 {
        self.set_mul(rhs);
        self
    }
}
impl MulAssign for GFp5 {
    fn mul_assign(&mut self, rhs: GFp5) { self.set_mul(&rhs); }
}

impl Neg for GFp5 {
    type Output = GFp5;
    fn neg(mut self) -> GFp5 {
        self.set_neg();
        self
    }
}
impl Neg for &GFp5 {
    type Output = GFp5;
    fn neg(self) -> GFp5 {
        let mut r = *self;
        r.set_neg();
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mul_identity() {
        let a = GFp5::from_u64_reduce(123, 456, 789, 0, 1);
        let r = a * GFp5::ONE;
        // a * 1 = a
        for i in 0..5 {
            assert_eq!(r.0[i].to_u64(), a.0[i].to_u64(), "mul_identity coeff {}", i);
        }
    }

    #[test]
    fn test_add_neg() {
        let a = GFp5::from_u64_reduce(1000, 2000, 3000, 4000, 5000);
        let neg_a = -a;
        let sum = a + neg_a;
        assert_eq!(sum.iszero(), !0u64, "a + (-a) = 0");
    }

    #[test]
    fn test_invert() {
        let a = GFp5::from_u64_reduce(42, 1, 0, 1, 2);
        let inv_a = a.invert();
        let product = a * inv_a;
        // product should be 1
        assert_eq!(product.0[0].to_u64(), 1);
        for i in 1..5 {
            assert_eq!(product.0[i].to_u64(), 0, "coeff {} should be 0", i);
        }
    }

    #[test]
    fn test_encode_decode() {
        let a = GFp5::from_u64_reduce(1, 2, 3, 4, 5);
        let bytes = a.encode();
        let (b, mask) = GFp5::decode(&bytes);
        assert_eq!(mask, !0u64, "decode should succeed");
        for i in 0..5 {
            assert_eq!(a.0[i].to_u64(), b.0[i].to_u64(), "coeff {} roundtrip", i);
        }
    }
}
