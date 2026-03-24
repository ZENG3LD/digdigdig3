//! Schnorr signatures over ECgFp5 using Poseidon2 as the hash function.
//!
//! ## Algorithm
//!
//! **Key generation:**
//! - Private key: ECgFp5Scalar (40 bytes, little-endian)
//! - Public key: GFp5 = GENERATOR.mul(sk).encode() (40 bytes)
//!
//! **Signing a pre-hashed message (hashed_msg: GFp5):**
//! 1. k = random ECgFp5Scalar (from OsRng)
//! 2. r = GENERATOR.mul(k).encode() as GFp5
//! 3. preimage = r[0..5] ++ hashed_msg[0..5]  (10 Goldilocks elements)
//! 4. e_gfp5 = Poseidon2.hash_to_quintic_extension(preimage)
//! 5. e = Scalar::from_gfp5(e_gfp5)
//! 6. s = k - e * sk  (mod n)
//! 7. Signature = (s, e) → 80 bytes: s[40] || e[40]
//!
//! **Verification:**
//! 1. r_v = (GENERATOR.mul(sig.s) + pk_point.mul(sig.e)).encode()
//! 2. Recompute e_v = Schnorr hash of (r_v, hashed_msg)
//! 3. Return e_v == sig.e

use rand::RngCore;
use super::ecgfp5::Point;
use super::gfp5::GFp5;
use super::goldilocks::GFp;
use super::scalar::Scalar;
use super::poseidon2::hash_to_quintic_extension;

/// A Schnorr signature over ECgFp5.
#[derive(Clone, Copy, Debug)]
pub struct Signature {
    /// s = k - e * sk (mod n)
    pub s: Scalar,
    /// e = Poseidon2(r || msg) reduced mod n
    pub e: Scalar,
}

impl Signature {
    /// Serialize to 80 bytes: s (40 bytes LE) || e (40 bytes LE).
    pub fn to_bytes(self) -> [u8; 80] {
        let mut r = [0u8; 80];
        r[..40].copy_from_slice(&self.s.encode());
        r[40..].copy_from_slice(&self.e.encode());
        r
    }

    /// Deserialize from 80 bytes.
    /// Returns (signature, mask). mask = 0xFFFF... if both components are valid (< n).
    pub fn from_bytes(buf: &[u8; 80]) -> (Self, u64) {
        let (s, ms) = Scalar::from_le_bytes(buf[..40].try_into().unwrap());
        let (e, me) = Scalar::from_le_bytes(buf[40..].try_into().unwrap());
        let mask = ms & me;
        (Signature { s, e }, mask)
    }
}

/// Derive the public key from a private key.
///
/// Public key = GENERATOR * private_key, encoded as GFp5 (40 bytes).
pub fn derive_public_key(private_key: &[u8; 40]) -> [u8; 40] {
    let (sk, _) = Scalar::from_le_bytes(private_key);
    let pk_point = Point::mulgen(sk);
    pk_point.encode().encode()
}

/// Sign a pre-hashed message (a GFp5 element) with the given private key.
///
/// Uses OsRng for nonce generation.
///
/// # Arguments
/// - `hashed_msg`: The GFp5 element (40 bytes) representing the Poseidon2 hash of the transaction.
/// - `private_key`: The 40-byte little-endian ECgFp5 scalar private key.
///
/// # Returns
/// 80-byte signature: s[0..40] || e[0..40]
pub fn sign(private_key: &[u8; 40], hashed_msg: GFp5) -> [u8; 80] {
    let (sk, _) = Scalar::from_le_bytes(private_key);

    // Generate random nonce k using OsRng (NOT thread_rng)
    let mut rng = rand::rngs::OsRng;
    let mut k_bytes = [0u8; 40];
    rng.fill_bytes(&mut k_bytes);
    let k = Scalar::from_random_bytes(&k_bytes);

    let sig = sign_with_nonce(sk, hashed_msg, k);
    sig.to_bytes()
}

/// Internal signing function with explicit nonce (for testing).
pub(crate) fn sign_with_nonce(sk: Scalar, hashed_msg: GFp5, k: Scalar) -> Signature {
    // r = k * G, encoded as GFp5
    let r_point = Point::mulgen(k);
    let r = r_point.encode();

    // Build preimage: r[0..5] ++ hashed_msg[0..5] (10 Goldilocks elements)
    let preimage = build_schnorr_preimage(r, hashed_msg);

    // e = Poseidon2(preimage) as scalar
    let e_gfp5 = hash_to_quintic_extension(&preimage);
    let e = Scalar::from_gfp5(e_gfp5);

    // s = k - e * sk (mod n)
    let s = k.sub(e.mul(sk));

    Signature { s, e }
}

/// Build the 10-element preimage for Schnorr challenge: r[5] ++ msg[5].
fn build_schnorr_preimage(r: GFp5, msg: GFp5) -> [GFp; 10] {
    let r_arr = r.to_basefield_array();
    let m_arr = msg.to_basefield_array();
    [
        GFp::from_canonical_u64(r_arr[0]),
        GFp::from_canonical_u64(r_arr[1]),
        GFp::from_canonical_u64(r_arr[2]),
        GFp::from_canonical_u64(r_arr[3]),
        GFp::from_canonical_u64(r_arr[4]),
        GFp::from_canonical_u64(m_arr[0]),
        GFp::from_canonical_u64(m_arr[1]),
        GFp::from_canonical_u64(m_arr[2]),
        GFp::from_canonical_u64(m_arr[3]),
        GFp::from_canonical_u64(m_arr[4]),
    ]
}

/// Verify a Schnorr signature.
///
/// # Arguments
/// - `public_key`: 40-byte encoded public key (GFp5 element)
/// - `hashed_msg`: The GFp5 hash of the message
/// - `signature`: 80-byte signature
///
/// # Returns
/// `true` if the signature is valid, `false` otherwise.
pub fn verify(public_key: &[u8; 40], hashed_msg: GFp5, signature: &[u8; 80]) -> bool {
    // Parse signature
    let (sig, sig_ok) = Signature::from_bytes(signature);
    if sig_ok == 0 {
        return false;
    }

    // Parse public key
    let (pk_w, pk_ok) = GFp5::from_le_bytes(public_key);
    if pk_ok == 0 {
        return false;
    }

    // Decode public key to curve point
    let (pk_point, decode_ok) = Point::decode(pk_w);
    if decode_ok == 0 {
        return false;
    }

    // Compute r_v = s*G + e*pk
    let sg = Point::mulgen(sig.s);
    let epk = pk_point.mul(&sig.e);
    let r_v = (sg + epk).encode();

    // Recompute challenge
    let preimage = build_schnorr_preimage(r_v, hashed_msg);
    let e_v_gfp5 = hash_to_quintic_extension(&preimage);
    let e_v = Scalar::from_gfp5(e_v_gfp5);

    // Check e_v == sig.e
    e_v.equals(sig.e) != 0
}

/// Sign pre-hashed bytes (40 bytes = a GFp5 element in little-endian).
/// This is the interface matching the Go SDK's `Sign(hashedMessage []byte)`.
pub fn sign_hashed_message(private_key: &[u8; 40], hashed_message_bytes: &[u8; 40]) -> [u8; 80] {
    let (msg_gfp5, _) = GFp5::from_le_bytes(hashed_message_bytes);
    sign(private_key, msg_gfp5)
}

/// Verify a signature against pre-hashed bytes.
pub fn verify_hashed_message(
    public_key: &[u8; 40],
    hashed_message_bytes: &[u8; 40],
    signature: &[u8; 80],
) -> bool {
    let (msg_gfp5, _) = GFp5::from_le_bytes(hashed_message_bytes);
    verify(public_key, msg_gfp5, signature)
}

