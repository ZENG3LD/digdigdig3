//! # Vertex Protocol Authentication
//!
//! EIP-712 signature-based authentication for Vertex Protocol.
//!
//! ## Key Features
//! - EIP-712 typed structured data signing
//! - Ethereum wallet-based authentication (no API keys)
//! - Keccak-256 hashing
//! - secp256k1 ECDSA signatures
//!
//! ## Subaccount Format
//! - bytes32 = address (20 bytes) + subaccount_name (12 bytes, hex-encoded + zero-padded)
//!
//! ## Signature Format
//! - 65 bytes: r (32) + s (32) + v (1)
//! - v = recovery_id + 27

use ethers::signers::{LocalWallet, Signer};
use ethers::types::transaction::eip712::{Eip712, TypedData};
use serde_json::{json, Value};

use crate::core::{Credentials, ExchangeError, ExchangeResult, timestamp_millis};

// ═══════════════════════════════════════════════════════════════════════════════
// VERTEX AUTH
// ═══════════════════════════════════════════════════════════════════════════════

/// Vertex Protocol authentication using EIP-712 signatures
#[derive(Clone)]
pub struct VertexAuth {
    /// Ethereum wallet for signing
    wallet: LocalWallet,
    /// Wallet address (0x...)
    address: String,
    /// Subaccount name (e.g., "default")
    subaccount: String,
    /// Chain ID (42161 for Arbitrum One, 421613 for Arbitrum Sepolia)
    chain_id: u64,
    /// Verifying contract address
    verifying_contract: String,
}

impl VertexAuth {
    /// Create new Vertex auth handler
    ///
    /// # Arguments
    /// * `credentials` - Must contain private_key in api_key field
    /// * `chain_id` - Network chain ID
    /// * `verifying_contract` - Vertex endpoint contract address
    /// * `subaccount` - Optional subaccount name (default: "default")
    pub fn new(
        credentials: &Credentials,
        chain_id: u64,
        verifying_contract: String,
        subaccount: Option<String>,
    ) -> ExchangeResult<Self> {
        // Private key is stored in api_key field for Vertex
        let private_key = &credentials.api_key;

        // Parse the wallet from private key
        let wallet = private_key
            .parse::<LocalWallet>()
            .map_err(|e| ExchangeError::Auth(format!("Invalid private key: {}", e)))?
            .with_chain_id(chain_id);

        let address = format!("{:?}", wallet.address());
        let subaccount = subaccount.unwrap_or_else(|| "default".to_string());

        Ok(Self {
            wallet,
            address,
            subaccount,
            chain_id,
            verifying_contract,
        })
    }

    /// Get the wallet address
    pub fn address(&self) -> &str {
        &self.address
    }

    /// Get the subaccount name
    pub fn subaccount(&self) -> &str {
        &self.subaccount
    }

    /// Convert address + subaccount to bytes32 sender format
    ///
    /// Format: address (20 bytes) + subaccount_hex (12 bytes, zero-padded)
    ///
    /// # Example
    /// Address: 0x7a5ec2748e9065794491a8d29dcf3f9edb8d7c43
    /// Subaccount: "default"
    /// Result: 0x7a5ec2748e9065794491a8d29dcf3f9edb8d7c43646566 61756c740000000000
    pub fn get_sender(&self) -> String {
        let addr_hex = self.address.strip_prefix("0x").unwrap_or(&self.address);
        let subaccount_bytes = self.subaccount.as_bytes();
        let subaccount_hex = hex::encode(subaccount_bytes);

        // Pad to 12 bytes (24 hex chars)
        let padding_len = 24 - subaccount_hex.len().min(24);
        let padding = "0".repeat(padding_len);

        format!("0x{}{}{}", addr_hex, subaccount_hex, padding)
    }

    /// Generate a unique nonce
    ///
    /// Uses timestamp_ms * 1000 + random(0-999) to ensure uniqueness
    pub fn generate_nonce(&self) -> u64 {
        let timestamp_ms = timestamp_millis();
        let random = rand::random::<u32>() as u64 % 1000;
        timestamp_ms * 1000 + random
    }

    /// Generate expiration timestamp with time-in-force flags
    ///
    /// # Time-in-Force Encoding (bits 62-63)
    /// - GTC: 0
    /// - IOC: bit 62 = 1
    /// - FOK: bit 63 = 1
    /// - POST_ONLY: bits 62-63 = 1
    ///
    /// # Arguments
    /// * `seconds_valid` - Validity duration in seconds
    /// * `tif` - Time-in-force type
    pub fn generate_expiration(&self, seconds_valid: u64, tif: TimeInForce) -> u64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("System time is before UNIX epoch")
            .as_secs();

        let expiration = now + seconds_valid;

        match tif {
            TimeInForce::Gtc => expiration,
            TimeInForce::Ioc => expiration | (1 << 62),
            TimeInForce::Fok => expiration | (1 << 63),
            TimeInForce::PostOnly => expiration | (1 << 62) | (1 << 63),
        }
    }

    /// Sign a place_order payload using EIP-712
    ///
    /// # Arguments
    /// * `product_id` - Product ID
    /// * `price_x18` - Price in X18 format (string)
    /// * `amount` - Amount in X18 format (string, negative for sell)
    /// * `expiration` - Expiration timestamp with TIF bits
    /// * `nonce` - Unique order nonce
    pub async fn sign_order(
        &self,
        _product_id: u32,
        price_x18: &str,
        amount: &str,
        expiration: u64,
        nonce: u64,
    ) -> ExchangeResult<(String, String)> {
        let sender = self.get_sender();

        // Create order struct
        let order = json!({
            "sender": sender,
            "priceX18": price_x18,
            "amount": amount,
            "expiration": expiration.to_string(),
            "nonce": nonce.to_string(),
        });

        // Sign the order
        let signature = self.sign_typed_data("Order", &order).await?;

        // Calculate digest (order hash)
        let digest = self.calculate_digest("Order", &order)?;

        Ok((signature, digest))
    }

    /// Sign a cancel_orders payload using EIP-712
    pub async fn sign_cancel(
        &self,
        product_ids: Vec<u32>,
        digests: Vec<String>,
        nonce: u64,
    ) -> ExchangeResult<String> {
        let sender = self.get_sender();

        let cancellation = json!({
            "sender": sender,
            "productIds": product_ids,
            "digests": digests,
            "nonce": nonce.to_string(),
        });

        self.sign_typed_data("Cancellation", &cancellation).await
    }

    /// Sign a cancel_product_orders payload using EIP-712
    pub async fn sign_cancel_products(
        &self,
        product_ids: Vec<u32>,
        nonce: u64,
    ) -> ExchangeResult<String> {
        let sender = self.get_sender();

        let cancellation = json!({
            "sender": sender,
            "productIds": product_ids,
            "nonce": nonce.to_string(),
        });

        self.sign_typed_data("CancellationProducts", &cancellation).await
    }

    /// Sign a withdraw_collateral payload using EIP-712
    pub async fn sign_withdraw(
        &self,
        product_id: u32,
        amount: &str,
        nonce: u64,
    ) -> ExchangeResult<String> {
        let sender = self.get_sender();

        let withdrawal = json!({
            "sender": sender,
            "productId": product_id,
            "amount": amount,
            "nonce": nonce.to_string(),
        });

        self.sign_typed_data("WithdrawCollateral", &withdrawal).await
    }

    /// Sign WebSocket authentication message
    pub async fn sign_ws_auth(&self, expiration_ms: u64) -> ExchangeResult<(Value, String)> {
        let sender = self.get_sender();

        let tx = json!({
            "sender": sender,
            "expiration": expiration_ms,
        });

        let signature = self.sign_typed_data("Authentication", &tx).await?;

        Ok((tx, signature))
    }

    /// Sign EIP-712 typed data
    ///
    /// # Arguments
    /// * `primary_type` - Primary struct type (e.g., "Order", "Cancellation")
    /// * `message` - Message data (JSON value)
    async fn sign_typed_data(
        &self,
        primary_type: &str,
        message: &Value,
    ) -> ExchangeResult<String> {
        // Build EIP-712 typed data
        let typed_data = self.build_typed_data(primary_type, message)?;

        // Sign using ethers
        let signature = self
            .wallet
            .sign_typed_data(&typed_data)
            .await
            .map_err(|e| ExchangeError::Auth(format!("Signing failed: {}", e)))?;

        // Format as hex string with 0x prefix
        Ok(format!("0x{}", hex::encode(signature.to_vec())))
    }

    /// Calculate digest (hash) of typed data without signing
    fn calculate_digest(&self, primary_type: &str, message: &Value) -> ExchangeResult<String> {
        let typed_data = self.build_typed_data(primary_type, message)?;

        // Encode the struct hash
        let hash = typed_data
            .encode_eip712()
            .map_err(|e| ExchangeError::Auth(format!("Failed to encode: {}", e)))?;

        Ok(format!("0x{}", hex::encode(hash)))
    }

    /// Build EIP-712 TypedData structure
    fn build_typed_data(&self, primary_type: &str, message: &Value) -> ExchangeResult<TypedData> {
        // Domain separator
        let domain = json!({
            "name": "Vertex",
            "version": "0.0.1",
            "chainId": self.chain_id,
            "verifyingContract": self.verifying_contract,
        });

        // Type definitions
        let types = self.get_type_definitions(primary_type);

        // Build typed data
        let typed_data_json = json!({
            "types": types,
            "primaryType": primary_type,
            "domain": domain,
            "message": message,
        });

        // Parse into TypedData
        serde_json::from_value(typed_data_json)
            .map_err(|e| ExchangeError::Auth(format!("Failed to build typed data: {}", e)))
    }

    /// Get EIP-712 type definitions for each struct
    fn get_type_definitions(&self, primary_type: &str) -> Value {
        match primary_type {
            "Order" => json!({
                "EIP712Domain": [
                    {"name": "name", "type": "string"},
                    {"name": "version", "type": "string"},
                    {"name": "chainId", "type": "uint256"},
                    {"name": "verifyingContract", "type": "address"}
                ],
                "Order": [
                    {"name": "sender", "type": "bytes32"},
                    {"name": "priceX18", "type": "int128"},
                    {"name": "amount", "type": "int128"},
                    {"name": "expiration", "type": "uint64"},
                    {"name": "nonce", "type": "uint64"}
                ]
            }),
            "Cancellation" => json!({
                "EIP712Domain": [
                    {"name": "name", "type": "string"},
                    {"name": "version", "type": "string"},
                    {"name": "chainId", "type": "uint256"},
                    {"name": "verifyingContract", "type": "address"}
                ],
                "Cancellation": [
                    {"name": "sender", "type": "bytes32"},
                    {"name": "productIds", "type": "uint32[]"},
                    {"name": "digests", "type": "bytes32[]"},
                    {"name": "nonce", "type": "uint64"}
                ]
            }),
            "CancellationProducts" => json!({
                "EIP712Domain": [
                    {"name": "name", "type": "string"},
                    {"name": "version", "type": "string"},
                    {"name": "chainId", "type": "uint256"},
                    {"name": "verifyingContract", "type": "address"}
                ],
                "CancellationProducts": [
                    {"name": "sender", "type": "bytes32"},
                    {"name": "productIds", "type": "uint32[]"},
                    {"name": "nonce", "type": "uint64"}
                ]
            }),
            "WithdrawCollateral" => json!({
                "EIP712Domain": [
                    {"name": "name", "type": "string"},
                    {"name": "version", "type": "string"},
                    {"name": "chainId", "type": "uint256"},
                    {"name": "verifyingContract", "type": "address"}
                ],
                "WithdrawCollateral": [
                    {"name": "sender", "type": "bytes32"},
                    {"name": "productId", "type": "uint32"},
                    {"name": "amount", "type": "uint128"},
                    {"name": "nonce", "type": "uint64"}
                ]
            }),
            "Authentication" => json!({
                "EIP712Domain": [
                    {"name": "name", "type": "string"},
                    {"name": "version", "type": "string"},
                    {"name": "chainId", "type": "uint256"},
                    {"name": "verifyingContract", "type": "address"}
                ],
                "Authentication": [
                    {"name": "sender", "type": "bytes32"},
                    {"name": "expiration", "type": "uint64"}
                ]
            }),
            _ => json!({}),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TIME IN FORCE
// ═══════════════════════════════════════════════════════════════════════════════

/// Time-in-force types for orders
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeInForce {
    /// Good-Till-Cancel
    GTC,
    /// Immediate-or-Cancel
    IOC,
    /// Fill-or-Kill
    FOK,
    /// Post-Only (maker-only)
    PostOnly,
}

// ═══════════════════════════════════════════════════════════════════════════════
// X18 CONVERSION UTILITIES
// ═══════════════════════════════════════════════════════════════════════════════

/// Convert f64 to X18 format (string)
///
/// # Example
/// `30000.0` → `"30000000000000000000000"`
pub fn to_x18(value: f64) -> String {
    ((value * 1e18) as i128).to_string()
}

/// Convert X18 format (string) to f64
///
/// # Example
/// `"30000000000000000000000"` → `30000.0`
pub fn from_x18(value: &str) -> ExchangeResult<f64> {
    let x18_value: i128 = value
        .parse()
        .map_err(|_| ExchangeError::Parse(format!("Invalid X18 value: {}", value)))?;
    Ok(x18_value as f64 / 1e18)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sender_format() {
        let credentials = Credentials::new(
            "0x0123456789012345678901234567890123456789012345678901234567890123",
            ""
        );

        let auth = VertexAuth::new(
            &credentials,
            42161,
            "0x0000000000000000000000000000000000000000".to_string(),
            Some("default".to_string()),
        ).unwrap();

        let sender = auth.get_sender();

        // Should be 66 characters (0x + 64 hex chars = 32 bytes)
        assert_eq!(sender.len(), 66);
        assert!(sender.starts_with("0x"));

        // Last 24 chars should be subaccount + padding
        // "default" hex = 64656661756c74 (14 chars) + 10 zeros = 24 chars
        assert!(sender.ends_with("64656661756c740000000000"));
    }

    #[test]
    fn test_x18_conversion() {
        // Price conversions
        assert_eq!(to_x18(1.0), "1000000000000000000");
        assert_eq!(to_x18(30000.0), "30000000000000000000000");
        assert_eq!(to_x18(0.5), "500000000000000000");

        // Reverse conversions
        assert!((from_x18("1000000000000000000").unwrap() - 1.0).abs() < 1e-10);
        assert!((from_x18("30000000000000000000000").unwrap() - 30000.0).abs() < 1e-6);
    }

    #[test]
    fn test_expiration_tif_encoding() {
        let credentials = Credentials::new(
            "0x0123456789012345678901234567890123456789012345678901234567890123",
            ""
        );

        let auth = VertexAuth::new(
            &credentials,
            42161,
            "0x0000000000000000000000000000000000000000".to_string(),
            None,
        ).unwrap();

        let base_expiration = auth.generate_expiration(300, TimeInForce::Gtc);
        let ioc_expiration = auth.generate_expiration(300, TimeInForce::Ioc);
        let fok_expiration = auth.generate_expiration(300, TimeInForce::Fok);
        let post_expiration = auth.generate_expiration(300, TimeInForce::PostOnly);

        // GTC should have no flags set
        assert_eq!(base_expiration & (1 << 62), 0);
        assert_eq!(base_expiration & (1 << 63), 0);

        // IOC should have bit 62 set
        assert_ne!(ioc_expiration & (1 << 62), 0);
        assert_eq!(ioc_expiration & (1 << 63), 0);

        // FOK should have bit 63 set
        assert_eq!(fok_expiration & (1 << 62), 0);
        assert_ne!(fok_expiration & (1 << 63), 0);

        // POST_ONLY should have both bits set
        assert_ne!(post_expiration & (1 << 62), 0);
        assert_ne!(post_expiration & (1 << 63), 0);
    }
}
