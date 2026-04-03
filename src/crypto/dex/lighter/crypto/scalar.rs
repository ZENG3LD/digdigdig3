#![allow(clippy::all)]
//! ECgFp5 scalar field — integers modulo the curve group order n.
//!
//! The group order n is a 319-bit prime:
//! n = 1067993516717146951041484916571792702745057740581727230159139685185762082554198619328292418486241
//!
//! Stored as 5 × u64 limbs in little-endian (non-Montgomery) representation.

use super::gfp5::GFp5;

/// A scalar (integer mod n, the ECgFp5 group order).
/// Stored as 5 × u64 little-endian limbs in normal (non-Montgomery) representation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scalar(pub(crate) [u64; 5]);

impl Scalar {
    pub const ZERO: Self = Self([0, 0, 0, 0, 0]);
    pub const ONE: Self = Self([1, 0, 0, 0, 0]);

    /// Group order n (319-bit prime).
    pub const N: Self = Self([
        0xE80F_D996_948B_FFE1,
        0xE888_5C39_D724_A09C,
        0x7FFF_FFE6_CFB8_0639,
        0x7FFF_FFF1_0000_0016,
        0x7FFF_FFFD_8000_0007,
    ]);

    // Montgomery constant for scalar field: -1/N[0] mod 2^64
    const N0I: u64 = 0xD78B_EF72_057B_7BDF;

    // R^2 mod n (R = 2^320), for Montgomery multiplication
    const R2: Self = Self([
        0xA010_01DC_E33D_C739,
        0x6C32_28D3_3F62_ACCF,
        0xD1D7_96CC_91CF_8525,
        0xAADF_FF5D_1574_C1D8,
        0x4ACA_13B2_8CA2_51F5,
    ]);

    // 2^632 mod n (for decode_reduce)
    const T632: Self = Self([
        0x2B02_66F3_17CA_91B3,
        0xEC1D_2652_8E98_4773,
        0x8651_D786_5E12_DB94,
        0xDA2A_DFF5_9415_74D0,
        0x53CA_CA12_110C_A256,
    ]);

    // --- Raw 5-limb arithmetic ---

    fn add_inner(self, a: Self) -> Self {
        let mut r = [0u64; 5];
        let mut carry: u64 = 0;
        for i in 0..5 {
            let (s, c1) = self.0[i].overflowing_add(a.0[i]);
            let (s2, c2) = s.overflowing_add(carry);
            r[i] = s2;
            carry = (c1 as u64) + (c2 as u64);
        }
        Self(r)
    }

    // Returns (result, borrow_mask): borrow_mask = 0xFFFF... if borrow, else 0.
    fn sub_inner(self, a: Self) -> (Self, u64) {
        let mut r = [0u64; 5];
        let mut borrow: u64 = 0;
        for i in 0..5 {
            let (s, c1) = self.0[i].overflowing_sub(a.0[i]);
            let (s2, c2) = s.overflowing_sub(borrow);
            r[i] = s2;
            borrow = (c1 as u64) + (c2 as u64);
        }
        // borrow is 0 or 1; convert to mask
        let mask = borrow.wrapping_neg();
        (Self(r), mask)
    }

    /// Constant-time select: returns a0 if c==0, a1 if c==0xFFFF_FFFF_FFFF_FFFF.
    pub fn select(c: u64, a0: Self, a1: Self) -> Self {
        let mut r = [0u64; 5];
        for i in 0..5 {
            r[i] = a0.0[i] ^ (c & (a0.0[i] ^ a1.0[i]));
        }
        Self(r)
    }

    // --- Modular arithmetic ---

    /// Addition mod n.
    pub fn add(self, rhs: Self) -> Self {
        let r0 = self.add_inner(rhs);
        let (r1, c) = r0.sub_inner(Self::N);
        Self::select(c, r1, r0)
    }

    /// Subtraction mod n.
    pub fn sub(self, rhs: Self) -> Self {
        let (r0, c) = self.sub_inner(rhs);
        let r1 = r0.add_inner(Self::N);
        Self::select(c, r0, r1)
    }

    /// Negation mod n.
    pub fn neg(self) -> Self {
        Self::ZERO.sub(self)
    }

    /// Montgomery multiplication: computes (self * rhs) / R mod n.
    /// Both inputs should be in [0, n-1].
    fn montymul(self, rhs: Self) -> Self {
        let mut t = [0u64; 10];

        // Step 1: schoolbook multiply into 10 limbs
        for i in 0..5 {
            let mut carry: u128 = 0;
            for j in 0..5 {
                let prod = (self.0[i] as u128) * (rhs.0[j] as u128)
                    + t[i + j] as u128
                    + carry;
                t[i + j] = prod as u64;
                carry = prod >> 64;
            }
            t[i + 5] = carry as u64;
        }

        // Step 2: Montgomery reduction
        let mut r = [0u64; 5];
        let mut acc = [0u64; 10];
        acc.copy_from_slice(&t);

        for i in 0..5 {
            let q = acc[i].wrapping_mul(Self::N0I);
            let mut carry: u128 = 0;
            for j in 0..5 {
                let prod = (q as u128) * (Self::N.0[j] as u128)
                    + acc[i + j] as u128
                    + carry;
                acc[i + j] = prod as u64;
                carry = prod >> 64;
            }
            // propagate carry into remaining slots
            let mut k = i + 5;
            while carry != 0 && k < 10 {
                let s = acc[k] as u128 + carry;
                acc[k] = s as u64;
                carry = s >> 64;
                k += 1;
            }
            // carry is absorbed; any final overflow is handled by the conditional subtraction
        }

        for i in 0..5 {
            r[i] = acc[i + 5];
        }

        // Final subtraction if >= n
        let result = Self(r);
        let (reduced, c) = result.sub_inner(Self::N);
        Self::select(c, reduced, result)
    }

    /// Full scalar multiplication mod n.
    pub fn mul(self, rhs: Self) -> Self {
        self.montymul(Self::R2).montymul(rhs)
    }

    /// Decode little-endian bytes (canonical, must be < n).
    /// Returns (scalar, mask). mask = 0xFFFF... if valid, 0 otherwise.
    pub fn decode(buf: &[u8]) -> (Self, u64) {
        if buf.len() < 40 {
            return (Self::ZERO, 0);
        }
        let mut limbs = [0u64; 5];
        for i in 0..5 {
            limbs[i] = u64::from_le_bytes(buf[8 * i..8 * i + 8].try_into().unwrap());
        }
        let s = Self(limbs);
        // Check s < n by attempting subtraction
        let (_, borrow) = s.sub_inner(Self::N);
        // borrow == 0xFFFF... means s < n (no borrow means s >= n)
        // We want c = 0xFFFF... if s < n
        (s, borrow)
    }

    /// Decode from a fixed 40-byte array.
    pub fn from_le_bytes(buf: &[u8; 40]) -> (Self, u64) {
        Self::decode(buf)
    }

    /// Decode bytes and reduce mod n (no failure). Input can be any length.
    /// Processes the input in 39-byte chunks, accumulating into the scalar.
    pub fn decode_reduce(buf: &[u8]) -> Self {
        if buf.is_empty() {
            return Self::ZERO;
        }
        // Process in 39-byte chunks from low to high
        let mut offset = buf.len() % 39;
        if offset == 0 {
            offset = 39;
        }

        // First (possibly partial) chunk
        let first_chunk = &buf[..offset];
        let mut tmp = [0u8; 40];
        tmp[..first_chunk.len()].copy_from_slice(first_chunk);
        let mut limbs = [0u64; 5];
        for i in 0..5 {
            limbs[i] = u64::from_le_bytes(tmp[8 * i..8 * i + 8].try_into().unwrap());
        }
        let mut acc = Self::reduce_wide_inner(limbs);

        // Remaining 39-byte chunks
        let mut pos = offset;
        while pos + 39 <= buf.len() {
            // acc = acc * 2^(39*8) mod n = acc * T632 mod n
            acc = acc.mul(Self::T632);
            let chunk = &buf[pos..pos + 39];
            let mut tmp2 = [0u8; 40];
            tmp2[..39].copy_from_slice(chunk);
            let mut limbs2 = [0u64; 5];
            for i in 0..5 {
                limbs2[i] = u64::from_le_bytes(tmp2[8 * i..8 * i + 8].try_into().unwrap());
            }
            let chunk_scalar = Self::reduce_wide_inner(limbs2);
            acc = acc.add(chunk_scalar);
            pos += 39;
        }
        acc
    }

    /// Reduce a 5-limb value mod n (value may be larger than n).
    fn reduce_wide_inner(limbs: [u64; 5]) -> Self {
        let s = Self(limbs);
        // Try subtracting n repeatedly (at most a few times for normal inputs)
        let (r1, c1) = s.sub_inner(Self::N);
        let r = Self::select(c1, r1, s); // if c1 != 0, s < n, keep s; else keep r1
        let (r2, c2) = r.sub_inner(Self::N);
        Self::select(c2, r2, r)
    }

    /// Convert a GFp5 element to a scalar by treating its 5 limbs as a 320-bit integer
    /// and reducing mod n.
    pub fn from_gfp5(x: GFp5) -> Self {
        // Each GFp5 limb is a canonical u64 (already < p < 2^64).
        // We interpret the 5 limbs as a 320-bit little-endian integer and reduce mod n.
        let limbs = x.to_basefield_array();
        // The value is limbs[0] + limbs[1]*2^64 + ... + limbs[4]*2^256
        // We need to reduce this mod n.
        // Strategy: accumulate starting from highest limb
        let mut acc = Self([limbs[4], 0, 0, 0, 0]);
        // multiply by 2^64 = scalar with limb layout, then add next limb
        // 2^64 as a scalar: we need to reduce 2^64 mod n
        // Since n < 2^319, 2^64 < n, so 2^64 mod n = 2^64 is just [0,1,0,0,0]
        let pow64 = Self([0, 1, 0, 0, 0]);
        for i in (0..4).rev() {
            acc = acc.mul(pow64);
            let addend = Self([limbs[i], 0, 0, 0, 0]);
            acc = acc.add(addend);
        }
        acc
    }

    /// Encode to 40 bytes little-endian.
    pub fn encode(self) -> [u8; 40] {
        let mut r = [0u8; 40];
        for i in 0..5 {
            r[8 * i..8 * i + 8].copy_from_slice(&self.0[i].to_le_bytes());
        }
        r
    }

    /// Returns 0xFFFF_FFFF_FFFF_FFFF if self == 0, else 0.
    pub fn iszero(self) -> u64 {
        let z = self.0[0] | self.0[1] | self.0[2] | self.0[3] | self.0[4];
        !((((z | z.wrapping_neg()) as i64) >> 63) as u64)
    }

    /// Returns 0xFFFF_FFFF_FFFF_FFFF if self == rhs, else 0.
    pub fn equals(self, rhs: Self) -> u64 {
        let z = (self.0[0] ^ rhs.0[0])
            | (self.0[1] ^ rhs.0[1])
            | (self.0[2] ^ rhs.0[2])
            | (self.0[3] ^ rhs.0[3])
            | (self.0[4] ^ rhs.0[4]);
        !((((z | z.wrapping_neg()) as i64) >> 63) as u64)
    }

    /// Generate a random scalar using the provided random bytes (40 bytes).
    /// Reduces the 40-byte value mod n. For use with OsRng.
    pub fn from_random_bytes(buf: &[u8; 40]) -> Self {
        let mut limbs = [0u64; 5];
        for i in 0..5 {
            limbs[i] = u64::from_le_bytes(buf[8 * i..8 * i + 8].try_into().unwrap());
        }
        // Reduce mod n: if value >= n, subtract n
        let s = Self(limbs);
        let (r1, c) = s.sub_inner(Self::N);
        Self::select(c, r1, s)
    }

    /// Recode scalar into signed digits for 5-bit windowed scalar multiplication.
    /// Output: array of signed digits in range [-16, +16].
    /// Length: ceil(319/5) = 64 digits.
    pub(crate) fn recode_signed(&self, ss: &mut [i32]) {
        // Convert to non-adjacent form with 5-bit window
        // NAF-like recoding: each digit in range -(2^(w-1))..+(2^(w-1))
        let w = 5i32;
        let win = 1i32 << w;        // 32
        let half = 1i32 << (w - 1); // 16

        // Work with a mutable copy
        let mut t = self.0;

        let n = ss.len();
        for i in 0..n {
            // Extract w-bit digit
            let d = (t[0] & ((win as u64) - 1)) as i32;
            let d_signed = if d >= half { d - win } else { d };
            ss[i] = d_signed;

            // Subtract d_signed and right-shift by w
            let borrow: i64 = if d_signed < 0 {
                // t += -d_signed (which is positive)
                let add = (-d_signed) as u64;
                let mut carry: u64 = 0;
                for j in 0..5 {
                    let (s, c1) = t[j].overflowing_add(if j == 0 { add } else { 0 });
                    let (s2, c2) = s.overflowing_add(carry);
                    t[j] = s2;
                    carry = (c1 as u64) + (c2 as u64);
                }
                0
            } else {
                // t -= d_signed
                let sub = d_signed as u64;
                let mut borrow: u64 = 0;
                for j in 0..5 {
                    let (s, c1) = t[j].overflowing_sub(if j == 0 { sub } else { 0 });
                    let (s2, c2) = s.overflowing_sub(borrow);
                    t[j] = s2;
                    borrow = (c1 as u64) + (c2 as u64);
                }
                0
            };
            let _ = borrow;

            // Right-shift by w=5 bits across all 5 limbs
            for j in 0..4 {
                t[j] = (t[j] >> w) | (t[j + 1] << (64 - w));
            }
            t[4] >>= w;
        }
    }
}

impl core::ops::Add for Scalar {
    type Output = Scalar;
    fn add(self, rhs: Scalar) -> Scalar { Scalar::add(self, rhs) }
}
impl core::ops::Sub for Scalar {
    type Output = Scalar;
    fn sub(self, rhs: Scalar) -> Scalar { Scalar::sub(self, rhs) }
}
impl core::ops::Mul for Scalar {
    type Output = Scalar;
    fn mul(self, rhs: Scalar) -> Scalar { Scalar::mul(self, rhs) }
}
impl core::ops::Neg for Scalar {
    type Output = Scalar;
    fn neg(self) -> Scalar { Scalar::neg(self) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_zero() {
        let a = Scalar::ONE;
        let r = a.add(Scalar::ZERO);
        assert_eq!(r, Scalar::ONE);
    }

    #[test]
    fn test_sub_self() {
        let a = Scalar([12345, 678, 0, 0, 0]);
        let r = a.sub(a);
        assert_eq!(r.iszero(), !0u64, "a - a = 0");
    }

    #[test]
    fn test_mul_one() {
        let a = Scalar([999, 1, 0, 0, 0]);
        let r = a.mul(Scalar::ONE);
        assert_eq!(r, a, "a * 1 = a");
    }

    #[test]
    fn test_encode_decode() {
        let s = Scalar([1, 2, 3, 4, 5]);
        let bytes = s.encode();
        let (decoded, mask) = Scalar::decode(&bytes);
        assert_eq!(mask, !0u64, "decode should succeed");
        assert_eq!(decoded, s);
    }

    #[test]
    fn test_from_gfp5_zero() {
        let x = GFp5::ZERO;
        let s = Scalar::from_gfp5(x);
        assert_eq!(s.iszero(), !0u64, "from_gfp5(0) = 0 scalar");
    }
}
