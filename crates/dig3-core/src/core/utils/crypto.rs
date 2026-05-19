//! # Crypto Utilities
//!
//! HMAC и hashing функции.

use hmac::{Hmac, Mac};
use sha2::{Sha256, Sha384, Sha512, Digest};

/// HMAC-SHA256
pub fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key)
        .expect("HMAC can take key of any size");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

/// HMAC-SHA256 with lowercase hex encoding
/// Used by Bybit and other exchanges that require hex signatures
pub fn hmac_sha256_hex(key: &[u8], data: &[u8]) -> String {
    let bytes = hmac_sha256(key, data);
    hex::encode(bytes)
}

/// HMAC-SHA384
pub fn hmac_sha384(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = Hmac::<Sha384>::new_from_slice(key)
        .expect("HMAC can take key of any size");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

/// HMAC-SHA512
pub fn hmac_sha512(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = Hmac::<Sha512>::new_from_slice(key)
        .expect("HMAC can take key of any size");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

/// SHA256 hash
pub fn sha256(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// SHA512 hash
pub fn sha512(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha512::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hmac_sha256() {
        let key = b"secret";
        let data = b"message";
        let result = hmac_sha256(key, data);
        assert_eq!(result.len(), 32); // SHA256 = 32 bytes
    }

    #[test]
    fn test_hmac_sha512() {
        let key = b"secret";
        let data = b"message";
        let result = hmac_sha512(key, data);
        assert_eq!(result.len(), 64); // SHA512 = 64 bytes
    }

    #[test]
    fn test_sha256() {
        let data = b"hello";
        let result = sha256(data);
        assert_eq!(result.len(), 32);
    }

    #[test]
    fn test_sha512() {
        let data = b"hello";
        let result = sha512(data);
        assert_eq!(result.len(), 64);
    }
}
