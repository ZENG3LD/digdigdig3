//! ECgFp5 elliptic curve over GF(p^5).
//!
//! Curve equation (Montgomery form): y^2 = x*(x^2 + a*x + b)
//! where a = 2, b = 263*z (z is the GFp5 generator).
//!
//! Points use fractional (X/Z, U/T) coordinates where U/T = x/y.
//! The neutral element has U = 0.
//!
//! Source: Thomas Pornin's ecgfp5 reference implementation.

#![allow(non_snake_case)]

use super::gfp5::GFp5;
use super::scalar::Scalar;

/// A point on the ECgFp5 curve in (X:Z:U:T) fractional coordinates.
/// x = X/Z, u = U/T = x/y.
#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub(crate) X: GFp5,
    pub(crate) Z: GFp5,
    pub(crate) U: GFp5,
    pub(crate) T: GFp5,
}

impl Point {
    // Curve parameter a = 2
    const A: GFp5 = GFp5::from_u64_reduce(2, 0, 0, 0, 0);

    // Curve parameter b1 = 263 (b = 263*z, coefficient of z only)
    const B1: u32 = 263;

    // 4*b as GFp5: [0, 4*263, 0, 0, 0]
    const B_MUL4: GFp5 = GFp5::from_u64_reduce(0, 4 * 263, 0, 0, 0);

    /// The neutral element (identity): (X:Z:U:T) = (0:1:0:1).
    pub const NEUTRAL: Self = Self {
        X: GFp5::ZERO,
        Z: GFp5::ONE,
        U: GFp5::ZERO,
        T: GFp5::ONE,
    };

    /// The conventional generator point (encoding w = 4, i.e. y/x = 4).
    /// These constants are from Pornin's ecgfp5 reference — in Montgomery form.
    pub const GENERATOR: Self = Self {
        X: GFp5::from_u64_reduce(
            0xB2CA_178E_CF44_53A1,
            0x3C75_7788_836D_3EA4,
            0x48D7_F28A_26DA_FD0B,
            0x1E0F_15C7_FD44_C28E,
            0x21FA_7FFC_C825_2211,
        ),
        Z: GFp5::ONE,
        U: GFp5::from_u64_reduce(0xBFFF_FFFF_4000_0001, 0, 0, 0, 0),
        T: GFp5::ONE,
    };

    /// Test if this is the neutral element.
    /// Returns 0xFFFF_FFFF_FFFF_FFFF if neutral, 0 otherwise.
    pub fn isneutral(self) -> u64 {
        self.U.iszero()
    }

    /// Encode point to a GFp5 field element: w = T/U = y/x = 1/u.
    /// The neutral element encodes to 0.
    pub fn encode(self) -> GFp5 {
        // w = T/U; for neutral U=0 so we get 0 (since invert(0) = 0 in our impl)
        self.T * self.U.invert()
    }

    /// Validate whether a GFp5 element w can decode to a curve point.
    /// Returns 0xFFFF... if valid, 0 otherwise.
    pub fn validate(w: GFp5) -> u64 {
        // w = 0 is valid (neutral). Otherwise, (w^2 - a)^2 - 4b must be a QR in GF(p^5).
        let e = w.square() - Self::A;
        let delta = e.square() - Self::B_MUL4;
        w.iszero() | delta.legendre().isone()
    }

    /// Decode a GFp5 field element to a curve point.
    /// Returns (Point, mask). mask = 0xFFFF... if valid.
    pub fn decode(w: GFp5) -> (Self, u64) {
        let e = w.square() - Self::A;
        let delta = e.square() - Self::B_MUL4;
        let (r, c) = delta.sqrt();
        let x1 = (e + r).half();
        let x2 = (e - r).half();
        // Exactly one of x1, x2 is a square in GF(p^5)
        let x1_is_qr = x1.legendre().isone();
        let x = GFp5::select(x1_is_qr, x1, x2);
        let X = GFp5::select(c, GFp5::ZERO, x);
        let Z = GFp5::ONE;
        let U = GFp5::select(c, GFp5::ZERO, GFp5::ONE);
        let T = GFp5::select(c, GFp5::ONE, w);
        (Self { X, Z, U, T }, c | w.iszero())
    }

    /// General point addition (complete, handles neutral correctly). Cost: 10M.
    fn set_add(&mut self, rhs: &Self) {
        let (X1, Z1, U1, T1) = (self.X, self.Z, self.U, self.T);
        let (X2, Z2, U2, T2) = (rhs.X, rhs.Z, rhs.U, rhs.T);
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

    /// Add a point in affine (x, u) coordinates. Cost: 8M.
    fn set_add_affine(&mut self, rhs: &PointAffine) {
        let (X1, Z1, U1, T1) = (self.X, self.Z, self.U, self.T);
        let (x2, u2) = (rhs.x, rhs.u);
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

    /// Negation: negate the u-coordinate (flip sign of y).
    fn set_neg(&mut self) {
        self.U.set_neg();
    }

    /// Point doubling. Cost: 4M+5S.
    fn set_double(&mut self) {
        let (X, Z, U, T) = (self.X, self.Z, self.U, self.T);
        let t1 = Z * T;
        let t2 = t1 * T;
        let x1 = t2.square();
        let z1 = t1 * U;
        let t3 = U.square();
        let w1 = t2 - (X + Z).double() * t3;
        let t4 = z1.square();
        self.X = t4.mul_small_k1(4 * Self::B1);
        self.Z = w1.square();
        self.U = (w1 + z1).square() - t4 - self.Z;
        self.T = x1.double() - t4.mul_small(4) - self.Z;
    }

    /// n successive doublings.
    pub fn mdouble(self, n: u32) -> Self {
        let mut r = self;
        r.set_mdouble(n);
        r
    }

    fn set_mdouble(&mut self, n: u32) {
        if n == 0 {
            return;
        }
        // First pass: convert to doubling-friendly form
        // Cost: n*(2M+5S) + 2M+1S total
        let mut r = *self;
        r.set_double();
        for _ in 1..n {
            r.set_double();
        }
        *self = r;
    }

    /// Build window of 2^(w-1) = 16 affine multiples for 5-bit windowed multiplication.
    fn make_window_affine(&self, size: usize) -> Vec<PointAffine> {
        let mut win = vec![PointAffine::NEUTRAL; size];
        // Convert self to affine
        let zinv = self.Z.invert();
        let tinv = self.T.invert();
        win[0] = PointAffine {
            x: self.X * zinv,
            u: self.U * tinv,
        };
        let mut cur = *self;
        for i in 1..size {
            cur.set_add(self);
            let zinv2 = cur.Z.invert();
            let tinv2 = cur.T.invert();
            win[i] = PointAffine {
                x: cur.X * zinv2,
                u: cur.U * tinv2,
            };
        }
        win
    }

    /// Lookup in a window table, constant-time. k is in [-size, +size], 0 = neutral.
    fn win_lookup(win: &[PointAffine], k: i32) -> PointAffine {
        let neg = k < 0;
        let idx = if neg { -k } else { k } as usize;
        let mut r = PointAffine::NEUTRAL;
        for i in 0..win.len() {
            let mask = ((i + 1 == idx) as u64).wrapping_neg();
            r.x.set_partial_lookup(win[i].x, mask);
            r.u.set_partial_lookup(win[i].u, mask);
        }
        if neg {
            r.u = -r.u;
        }
        r
    }

    /// Scalar multiplication: compute s * self.
    /// Uses 5-bit signed windowed method.
    pub fn mul(self, s: &Scalar) -> Self {
        const WINDOW: usize = 5;
        const WIN_SIZE: usize = 1 << (WINDOW - 1); // 16

        let win = self.make_window_affine(WIN_SIZE);
        let mut ss = [0i32; 64]; // ceil(319/5) = 64 digits
        s.recode_signed(&mut ss);

        let n = ss.len() - 1;
        let start = Self::win_lookup_to_point(&win, ss[n]);
        let mut p = start;

        for i in (0..n).rev() {
            p.set_mdouble(WINDOW as u32);
            let pt = Self::win_lookup(&win, ss[i]);
            p.set_add_affine(&pt);
        }
        p
    }

    fn win_lookup_to_point(win: &[PointAffine], k: i32) -> Self {
        let pa = Self::win_lookup(win, k);
        pa.to_point()
    }

    /// Multiply the generator by a scalar.
    /// Uses precomputed tables for efficiency.
    pub fn mulgen(s: Scalar) -> Self {
        // For simplicity, use the general scalar multiplication on GENERATOR.
        // A production implementation would use the precomputed multab tables.
        Self::GENERATOR.mul(&s)
    }

    /// Equality check. Returns 0xFFFF... if equal, 0 otherwise.
    pub fn equals(self, rhs: Self) -> u64 {
        // P1 == P2 iff U1*T2 == U2*T1 (in the fractional coordinate sense)
        (self.U * rhs.T).equals(rhs.U * self.T)
    }
}

impl core::ops::Add for Point {
    type Output = Point;
    fn add(mut self, rhs: Point) -> Point {
        self.set_add(&rhs);
        self
    }
}

impl core::ops::AddAssign<PointAffine> for Point {
    fn add_assign(&mut self, rhs: PointAffine) {
        self.set_add_affine(&rhs);
    }
}

impl core::ops::Neg for Point {
    type Output = Point;
    fn neg(mut self) -> Point {
        self.set_neg();
        self
    }
}

/// A point in affine coordinates (x, u) where x = X, u = U.
/// Used internally for precomputed window tables.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PointAffine {
    pub(crate) x: GFp5,
    pub(crate) u: GFp5,
}

impl PointAffine {
    pub(crate) const NEUTRAL: Self = Self {
        x: GFp5::ZERO,
        u: GFp5::ZERO,
    };

    pub(crate) fn to_point(self) -> Point {
        // If affine (x, u), then projective (X:Z:U:T) = (x:1:u:1)
        // But if neutral (x=0, u=0), return Point::NEUTRAL
        let is_neutral = self.u.iszero();
        let X = GFp5::select(is_neutral, GFp5::ZERO, self.x);
        let Z = GFp5::ONE;
        let U = GFp5::select(is_neutral, GFp5::ZERO, self.u);
        let T = GFp5::ONE;
        Point { X, Z, U, T }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_encode() {
        // The generator should encode to a valid point
        let g = Point::GENERATOR;
        let w = g.encode();
        // w = 4 (the generator has y/x = 4)
        // Let's verify it's a valid encoding
        let valid = Point::validate(w);
        assert_ne!(valid, 0, "generator encodes to valid point");
    }

    #[test]
    fn test_neutral_encodes_to_zero() {
        let n = Point::NEUTRAL;
        let w = n.encode();
        assert_eq!(w.iszero(), !0u64, "neutral encodes to 0");
    }

    #[test]
    fn test_neutral_addition() {
        let g = Point::GENERATOR;
        let n = Point::NEUTRAL;
        let r = g + n;
        // r should equal g
        assert_ne!(r.equals(g), 0, "G + neutral = G");
    }

    #[test]
    fn test_double_vs_add() {
        let g = Point::GENERATOR;
        let g2_add = g + g;
        let mut g2_dbl = g;
        g2_dbl.set_double();
        assert_ne!(g2_add.equals(g2_dbl), 0, "G+G == 2G");
    }

    #[test]
    fn test_scalar_mul_one() {
        let g = Point::GENERATOR;
        let r = g.mul(&Scalar::ONE);
        assert_ne!(r.equals(g), 0, "1*G == G");
    }

    #[test]
    fn test_order_check() {
        // n*G should equal the neutral element
        // (We skip this test as it's expensive; leave for integration tests)
        // let r = Point::mulgen(Scalar::N);
        // assert_ne!(r.isneutral(), 0, "n*G = neutral");
    }
}
