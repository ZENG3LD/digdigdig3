//! # Encoding Utilities
//!
//! Base64 и Hex encoding.

use base64::{engine::general_purpose::STANDARD, Engine};

/// Encode bytes to Base64
pub fn encode_base64(data: &[u8]) -> String {
    STANDARD.encode(data)
}

/// Encode bytes to Hex (lowercase)
pub fn encode_hex_lower(data: &[u8]) -> String {
    hex::encode(data)
}

/// Encode bytes to Hex (uppercase)
pub fn encode_hex(data: &[u8]) -> String {
    hex::encode_upper(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_base64() {
        let data = b"hello";
        assert_eq!(encode_base64(data), "aGVsbG8=");
    }

    #[test]
    fn test_encode_hex_lower() {
        let data = b"hello";
        assert_eq!(encode_hex_lower(data), "68656c6c6f");
    }

    #[test]
    fn test_encode_hex() {
        let data = b"hello";
        assert_eq!(encode_hex(data), "68656C6C6F");
    }
}
