//! Slim EVM crypto primitives for connectors that need EIP-712 signing
//! (HyperLiquid). Replaces the heavy `alloy` SDK with direct `k256` + `sha3`
//! usage — eliminates ~100 transitive deps and the rustls-webpki CVE chain.

use k256::ecdsa::{RecoveryId, Signature, SigningKey};
use sha3::{Digest, Keccak256};

/// keccak256 hash (NOT SHA3-256 — different padding!)
pub fn keccak256(data: &[u8]) -> [u8; 32] {
    let mut out = [0u8; 32];
    out.copy_from_slice(&Keccak256::digest(data));
    out
}

/// EVM wallet — secp256k1 keypair with Ethereum address derivation.
#[derive(Clone)]
pub struct EvmWallet {
    signing_key: SigningKey,
    address: [u8; 20],
}

impl EvmWallet {
    /// Parse from hex string (with or without "0x" prefix). Must be 64 hex chars.
    pub fn from_hex(hex_str: &str) -> Result<Self, String> {
        let s = hex_str.trim_start_matches("0x").trim_start_matches("0X");
        if s.len() != 64 {
            return Err(format!("evm wallet: expected 64 hex chars, got {}", s.len()));
        }
        let bytes = hex::decode(s).map_err(|e| format!("evm wallet: hex decode: {}", e))?;
        let arr: [u8; 32] = bytes.try_into().map_err(|_| "evm wallet: not 32 bytes".to_string())?;
        let signing_key = SigningKey::from_bytes(&arr.into())
            .map_err(|e| format!("evm wallet: invalid secret key: {}", e))?;
        let address = derive_address(&signing_key);
        Ok(Self { signing_key, address })
    }

    /// Raw 20-byte Ethereum address.
    pub fn address(&self) -> [u8; 20] {
        self.address
    }

    /// Lowercase hex with 0x prefix, no checksum.
    pub fn address_hex(&self) -> String {
        let mut s = String::with_capacity(42);
        s.push_str("0x");
        s.push_str(&hex::encode(self.address));
        s
    }

    /// Sign a 32-byte prehash. Returns 65-byte recoverable signature: r(32) || s(32) || v(1)
    /// where v = recovery_id + 27 (HyperLiquid / Ethereum convention).
    pub fn sign_prehash_recoverable(&self, hash: &[u8; 32]) -> Result<[u8; 65], String> {
        let (sig, rec_id): (Signature, RecoveryId) = self.signing_key
            .sign_prehash_recoverable(hash)
            .map_err(|e| format!("evm wallet: sign failed: {}", e))?;
        let mut out = [0u8; 65];
        let r_s = sig.to_bytes(); // 64 bytes: r(32) || s(32)
        out[0..64].copy_from_slice(&r_s);
        out[64] = rec_id.to_byte() + 27;
        Ok(out)
    }
}

/// Derive 20-byte Ethereum address from secp256k1 SigningKey.
/// Algorithm: uncompressed_pubkey[1..] (skip 0x04 prefix) → keccak256 → last 20 bytes.
fn derive_address(sk: &SigningKey) -> [u8; 20] {
    let vk = sk.verifying_key();
    let point = vk.to_encoded_point(false); // uncompressed: 65 bytes [0x04 || x || y]
    let pk_bytes = point.as_bytes();
    debug_assert_eq!(pk_bytes.len(), 65);
    debug_assert_eq!(pk_bytes[0], 0x04);
    let hash = keccak256(&pk_bytes[1..]);
    let mut addr = [0u8; 20];
    addr.copy_from_slice(&hash[12..]);
    addr
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evm_wallet_known_keypair() {
        let w = EvmWallet::from_hex("e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e").unwrap();
        let addr = w.address_hex();
        println!("derived address: {}", addr);
        assert_eq!(addr.len(), 42);
        assert!(addr.starts_with("0x"));
    }

    #[test]
    fn evm_wallet_l1_action_sign_mainnet() {
        let w = EvmWallet::from_hex("e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e").unwrap();
        let connection_id = hex::decode("de6c4037798a4434ca03cd05f00e3b803126221375cd1e7eaaaf041768be06eb").unwrap();
        let conn_id: [u8; 32] = connection_id.try_into().unwrap();

        let final_hash = compute_l1_eip712_hash(&conn_id, true);
        let sig = w.sign_prehash_recoverable(&final_hash).unwrap();
        let sig_hex = format!("0x{}", hex::encode(sig));
        let expected = "0xfa8a41f6a3fa728206df80801a83bcbfbab08649cd34d9c0bfba7c7b2f99340f53a00226604567b98a1492803190d65a201d6805e5831b7044f17fd530aec7841c";
        assert_eq!(sig_hex, expected, "L1 mainnet signature mismatch");
    }

    #[test]
    fn evm_wallet_l1_action_sign_testnet() {
        let w = EvmWallet::from_hex("e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e").unwrap();
        let connection_id = hex::decode("de6c4037798a4434ca03cd05f00e3b803126221375cd1e7eaaaf041768be06eb").unwrap();
        let conn_id: [u8; 32] = connection_id.try_into().unwrap();
        let final_hash = compute_l1_eip712_hash(&conn_id, false);
        let sig = w.sign_prehash_recoverable(&final_hash).unwrap();
        let sig_hex = format!("0x{}", hex::encode(sig));
        let expected = "0x1713c0fc661b792a50e8ffdd59b637b1ed172d9a3aa4d801d9d88646710fb74b33959f4d075a7ccbec9f2374a6da21ffa4448d58d0413a0d335775f680a881431c";
        assert_eq!(sig_hex, expected, "L1 testnet signature mismatch");
    }

    /// Compute HyperLiquid L1 action EIP-712 final hash.
    /// `is_mainnet=true` → source="a"; false → source="b".
    fn compute_l1_eip712_hash(connection_id: &[u8; 32], is_mainnet: bool) -> [u8; 32] {
        // Domain separator (chainId=1337 for L1, always)
        let domain_type_hash = keccak256(b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)");
        let name_hash = keccak256(b"Exchange");
        let version_hash = keccak256(b"1");
        let mut chain_id = [0u8; 32];
        chain_id[24..32].copy_from_slice(&1337u64.to_be_bytes());
        let verifying_contract = [0u8; 32];

        let mut domain_buf = Vec::with_capacity(160);
        domain_buf.extend_from_slice(&domain_type_hash);
        domain_buf.extend_from_slice(&name_hash);
        domain_buf.extend_from_slice(&version_hash);
        domain_buf.extend_from_slice(&chain_id);
        domain_buf.extend_from_slice(&verifying_contract);
        let domain_separator = keccak256(&domain_buf);

        // Agent struct hash — uses string source "a"/"b" per the spec
        let agent_type_hash = keccak256(b"Agent(string source,bytes32 connectionId)");
        let source_hash = if is_mainnet { keccak256(b"a") } else { keccak256(b"b") };
        let mut struct_buf = Vec::with_capacity(96);
        struct_buf.extend_from_slice(&agent_type_hash);
        struct_buf.extend_from_slice(&source_hash);
        struct_buf.extend_from_slice(connection_id);
        let struct_hash = keccak256(&struct_buf);

        // Final hash
        let mut final_buf = Vec::with_capacity(66);
        final_buf.push(0x19);
        final_buf.push(0x01);
        final_buf.extend_from_slice(&domain_separator);
        final_buf.extend_from_slice(&struct_hash);
        keccak256(&final_buf)
    }
}
