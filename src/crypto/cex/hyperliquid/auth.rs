//! # Hyperliquid Authentication
//!
//! EIP-712 wallet-based authentication for Hyperliquid.
//!
//! ## Overview
//!
//! Hyperliquid uses Ethereum wallet signatures (EIP-712) instead of traditional API keys.
//! Two signing schemes:
//!
//! ### 1. L1 Actions (Phantom Agent)
//! Used for trading operations: place/cancel/modify orders, leverage, margin.
//! Uses msgpack serialization + keccak256 + EIP-712 phantom agent construction.
//!
//! ### 2. User-Signed Actions (Direct EIP-712)
//! Used for: withdrawals, internal USDC/spot transfers.
//!
//! ## Implementation Notes
//!
//! - Keccak256 via `alloy::primitives::keccak256` (already a direct dependency)
//! - ECDSA signing via `alloy::signers::local::PrivateKeySigner`
//! - Minimal msgpack encoder (subset needed for Hyperliquid actions)

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use alloy::primitives::keccak256 as alloy_keccak256;
use alloy::signers::local::PrivateKeySigner;
use alloy::signers::SignerSync;

use crate::core::{
    Credentials, ExchangeResult, ExchangeError,
    timestamp_millis,
};

// ═══════════════════════════════════════════════════════════════════════════════
// MINIMAL MSGPACK ENCODER
// ═══════════════════════════════════════════════════════════════════════════════

/// Minimal msgpack encoder for Hyperliquid L1 action hashing.
///
/// Only implements the subset of msgpack needed for Hyperliquid's canonical
/// serialization of order/cancel/leverage/margin actions.
/// Field ordering follows Python SDK (dict keys sorted lexicographically).
struct MsgpackEncoder {
    buf: Vec<u8>,
}

impl MsgpackEncoder {
    fn new() -> Self {
        Self { buf: Vec::with_capacity(256) }
    }

    fn finish(self) -> Vec<u8> {
        self.buf
    }

    fn write_nil(&mut self) {
        self.buf.push(0xc0);
    }

    fn write_bool(&mut self, v: bool) {
        self.buf.push(if v { 0xc3 } else { 0xc2 });
    }

    fn write_uint(&mut self, v: u64) {
        if v <= 0x7f {
            self.buf.push(v as u8);
        } else if v <= 0xff {
            self.buf.push(0xcc);
            self.buf.push(v as u8);
        } else if v <= 0xffff {
            self.buf.push(0xcd);
            self.buf.extend_from_slice(&(v as u16).to_be_bytes());
        } else if v <= 0xffff_ffff {
            self.buf.push(0xce);
            self.buf.extend_from_slice(&(v as u32).to_be_bytes());
        } else {
            self.buf.push(0xcf);
            self.buf.extend_from_slice(&v.to_be_bytes());
        }
    }

    fn write_int(&mut self, v: i64) {
        if v >= 0 {
            self.write_uint(v as u64);
        } else if v >= -32 {
            self.buf.push(v as u8); // negative fixint
        } else if v >= -128 {
            self.buf.push(0xd0);
            self.buf.push(v as u8);
        } else if v >= -32768 {
            self.buf.push(0xd1);
            self.buf.extend_from_slice(&(v as i16).to_be_bytes());
        } else if v >= -2_147_483_648 {
            self.buf.push(0xd2);
            self.buf.extend_from_slice(&(v as i32).to_be_bytes());
        } else {
            self.buf.push(0xd3);
            self.buf.extend_from_slice(&v.to_be_bytes());
        }
    }

    fn write_str(&mut self, s: &str) {
        let bytes = s.as_bytes();
        let len = bytes.len();
        if len <= 31 {
            self.buf.push(0xa0 | len as u8);
        } else if len <= 0xff {
            self.buf.push(0xd9);
            self.buf.push(len as u8);
        } else if len <= 0xffff {
            self.buf.push(0xda);
            self.buf.extend_from_slice(&(len as u16).to_be_bytes());
        } else {
            self.buf.push(0xdb);
            self.buf.extend_from_slice(&(len as u32).to_be_bytes());
        }
        self.buf.extend_from_slice(bytes);
    }

    fn write_bin(&mut self, data: &[u8]) {
        let len = data.len();
        if len <= 0xff {
            self.buf.push(0xc4);
            self.buf.push(len as u8);
        } else if len <= 0xffff {
            self.buf.push(0xc5);
            self.buf.extend_from_slice(&(len as u16).to_be_bytes());
        } else {
            self.buf.push(0xc6);
            self.buf.extend_from_slice(&(len as u32).to_be_bytes());
        }
        self.buf.extend_from_slice(data);
    }

    fn begin_map(&mut self, n: usize) {
        if n <= 15 {
            self.buf.push(0x80 | n as u8);
        } else if n <= 0xffff {
            self.buf.push(0xde);
            self.buf.extend_from_slice(&(n as u16).to_be_bytes());
        } else {
            self.buf.push(0xdf);
            self.buf.extend_from_slice(&(n as u32).to_be_bytes());
        }
    }

    fn begin_array(&mut self, n: usize) {
        if n <= 15 {
            self.buf.push(0x90 | n as u8);
        } else if n <= 0xffff {
            self.buf.push(0xdc);
            self.buf.extend_from_slice(&(n as u16).to_be_bytes());
        } else {
            self.buf.push(0xdd);
            self.buf.extend_from_slice(&(n as u32).to_be_bytes());
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// HYPERLIQUID ACTION STRUCTURES
// ═══════════════════════════════════════════════════════════════════════════════

/// Internal representation of a Hyperliquid order for msgpack encoding.
/// Fields encoded in sorted key order matching Python SDK.
#[derive(Debug, Clone)]
pub struct HlOrder {
    /// Asset index
    pub a: u32,
    /// Is buy
    pub b: bool,
    /// Price string (no trailing zeros)
    pub p: String,
    /// Size string (no trailing zeros)
    pub s: String,
    /// Reduce only
    pub r: bool,
    /// Order type
    pub t: HlOrderType,
    /// Client order ID (16-byte hex with 0x prefix, or None)
    pub c: Option<String>,
}

/// Order type for Hyperliquid
#[derive(Debug, Clone)]
pub enum HlOrderType {
    Limit { tif: HlTif },
    Trigger { trigger_px: String, is_market: bool, tpsl: String },
}

/// Time in force for Hyperliquid limit orders
#[derive(Debug, Clone, Copy)]
pub enum HlTif {
    Gtc,
    Alo,  // Post-only (Add-Liquidity-Only)
    Ioc,
    Fok,
}

impl HlTif {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Gtc => "Gtc",
            Self::Alo => "Alo",
            Self::Ioc => "Ioc",
            Self::Fok => "Fok",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MSGPACK SERIALIZERS FOR HYPERLIQUID ACTIONS
// ═══════════════════════════════════════════════════════════════════════════════

/// Encode an "order" action into msgpack.
/// Keys sorted: grouping, orders, type
pub fn msgpack_order_action(orders: &[HlOrder], grouping: &str) -> Vec<u8> {
    let mut enc = MsgpackEncoder::new();
    enc.begin_map(3);

    enc.write_str("grouping");
    enc.write_str(grouping);

    enc.write_str("orders");
    enc.begin_array(orders.len());
    for order in orders {
        msgpack_order(&mut enc, order);
    }

    enc.write_str("type");
    enc.write_str("order");

    enc.finish()
}

/// Encode a single order. Keys sorted: a, b, c, p, r, s, t
fn msgpack_order(enc: &mut MsgpackEncoder, order: &HlOrder) {
    let n = if order.c.is_some() { 7 } else { 6 };
    enc.begin_map(n);

    enc.write_str("a");
    enc.write_uint(order.a as u64);

    enc.write_str("b");
    enc.write_bool(order.b);

    if let Some(ref cloid) = order.c {
        enc.write_str("c");
        let hex_str = cloid.trim_start_matches("0x");
        if let Ok(bytes) = hex::decode(hex_str) {
            enc.write_bin(&bytes);
        } else {
            enc.write_nil();
        }
    }

    enc.write_str("p");
    enc.write_str(&order.p);

    enc.write_str("r");
    enc.write_bool(order.r);

    enc.write_str("s");
    enc.write_str(&order.s);

    enc.write_str("t");
    msgpack_order_type(enc, &order.t);
}

fn msgpack_order_type(enc: &mut MsgpackEncoder, ot: &HlOrderType) {
    match ot {
        HlOrderType::Limit { tif } => {
            enc.begin_map(1);
            enc.write_str("limit");
            enc.begin_map(1);
            enc.write_str("tif");
            enc.write_str(tif.as_str());
        }
        HlOrderType::Trigger { trigger_px, is_market, tpsl } => {
            enc.begin_map(1);
            enc.write_str("trigger");
            // Keys sorted: isMarket, tpsl, triggerPx
            enc.begin_map(3);
            enc.write_str("isMarket");
            enc.write_bool(*is_market);
            enc.write_str("tpsl");
            enc.write_str(tpsl);
            enc.write_str("triggerPx");
            enc.write_str(trigger_px);
        }
    }
}

/// Encode a "cancel" action. Keys sorted: cancels, type
pub fn msgpack_cancel_action(cancels: &[(u32, u64)]) -> Vec<u8> {
    let mut enc = MsgpackEncoder::new();
    enc.begin_map(2);

    enc.write_str("cancels");
    enc.begin_array(cancels.len());
    for &(asset, oid) in cancels {
        // Keys sorted: a, o
        enc.begin_map(2);
        enc.write_str("a");
        enc.write_uint(asset as u64);
        enc.write_str("o");
        enc.write_uint(oid);
    }

    enc.write_str("type");
    enc.write_str("cancel");

    enc.finish()
}

/// Encode a "modify" action. Keys sorted: oid, order, type
pub fn msgpack_modify_action(oid: u64, order: &HlOrder) -> Vec<u8> {
    let mut enc = MsgpackEncoder::new();
    enc.begin_map(3);

    enc.write_str("oid");
    enc.write_uint(oid);

    enc.write_str("order");
    msgpack_order(&mut enc, order);

    enc.write_str("type");
    enc.write_str("modify");

    enc.finish()
}

/// Encode a "batchModify" action. Keys sorted: modifies, type
/// Each modify entry: {oid, order} (keys sorted)
pub fn msgpack_batch_modify_action(modifies: &[(u64, &HlOrder)]) -> Vec<u8> {
    let mut enc = MsgpackEncoder::new();
    enc.begin_map(2);

    enc.write_str("modifies");
    enc.begin_array(modifies.len());
    for (oid, order) in modifies {
        enc.begin_map(2);
        enc.write_str("oid");
        enc.write_uint(*oid);
        enc.write_str("order");
        msgpack_order(&mut enc, order);
    }

    enc.write_str("type");
    enc.write_str("batchModify");

    enc.finish()
}

/// Encode an "updateLeverage" action. Keys sorted: asset, isCross, leverage, type
pub fn msgpack_update_leverage_action(asset: u32, is_cross: bool, leverage: u32) -> Vec<u8> {
    let mut enc = MsgpackEncoder::new();
    enc.begin_map(4);

    enc.write_str("asset");
    enc.write_uint(asset as u64);

    enc.write_str("isCross");
    enc.write_bool(is_cross);

    enc.write_str("leverage");
    enc.write_uint(leverage as u64);

    enc.write_str("type");
    enc.write_str("updateLeverage");

    enc.finish()
}

/// Encode an "updateIsolatedMargin" action. Keys sorted: asset, isBuy, ntli, type
pub fn msgpack_update_isolated_margin_action(asset: u32, is_buy: bool, ntli: i64) -> Vec<u8> {
    let mut enc = MsgpackEncoder::new();
    enc.begin_map(4);

    enc.write_str("asset");
    enc.write_uint(asset as u64);

    enc.write_str("isBuy");
    enc.write_bool(is_buy);

    enc.write_str("ntli");
    enc.write_int(ntli);

    enc.write_str("type");
    enc.write_str("updateIsolatedMargin");

    enc.finish()
}

/// Encode a "usdClassTransfer" action into msgpack.
///
/// Keys sorted: amount, toPerp, type
pub fn msgpack_usd_class_transfer_action(amount: &str, to_perp: bool) -> Vec<u8> {
    let mut enc = MsgpackEncoder::new();
    enc.begin_map(3);

    enc.write_str("amount");
    enc.write_str(amount);

    enc.write_str("toPerp");
    enc.write_bool(to_perp);

    enc.write_str("type");
    enc.write_str("usdClassTransfer");

    enc.finish()
}

/// Encode a "twapOrder" action into msgpack.
///
/// Outer dict keys sorted: twap, type
/// Inner twap dict keys sorted: a, b, m, r, s
pub fn msgpack_twap_action(
    asset: u32,
    is_buy: bool,
    size: &str,
    reduce_only: bool,
    duration_minutes: u64,
) -> Vec<u8> {
    let mut enc = MsgpackEncoder::new();
    // Outer map: { twap: {...}, type: "twapOrder" }
    enc.begin_map(2);

    enc.write_str("twap");
    // Inner map: { a, b, m, r, s } — 5 keys, sorted alphabetically
    enc.begin_map(5);
    enc.write_str("a");
    enc.write_uint(asset as u64);
    enc.write_str("b");
    enc.write_bool(is_buy);
    enc.write_str("m");
    enc.write_uint(duration_minutes);
    enc.write_str("r");
    enc.write_bool(reduce_only);
    enc.write_str("s");
    enc.write_str(size);

    enc.write_str("type");
    enc.write_str("twapOrder");

    enc.finish()
}

// ═══════════════════════════════════════════════════════════════════════════════
// EIP-712 DOMAIN AND HASHING
// ═══════════════════════════════════════════════════════════════════════════════

fn keccak(data: &[u8]) -> [u8; 32] {
    *alloy_keccak256(data)
}

/// EIP-712 domain separator for Hyperliquid
/// domain = { name: "Exchange", version: "1", chainId: <chain_id>, verifyingContract: 0x0000...0000 }
fn compute_domain_separator(chain_id: u64) -> [u8; 32] {
    let type_hash = keccak(
        b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"
    );
    let name_hash = keccak(b"Exchange");
    let version_hash = keccak(b"1");

    let mut chain_id_bytes = [0u8; 32];
    chain_id_bytes[24..].copy_from_slice(&chain_id.to_be_bytes());

    let verifying_contract = [0u8; 32]; // zero address

    let mut encoded = Vec::with_capacity(5 * 32);
    encoded.extend_from_slice(&type_hash);
    encoded.extend_from_slice(&name_hash);
    encoded.extend_from_slice(&version_hash);
    encoded.extend_from_slice(&chain_id_bytes);
    encoded.extend_from_slice(&verifying_contract);

    keccak(&encoded)
}

/// Compute connection ID from msgpack action bytes + nonce + optional vault address.
/// Python SDK: keccak256(action_bytes + nonce_be8 + vault_flag + [vault_bytes])
fn compute_connection_id(action_bytes: &[u8], nonce: u64, vault_address: Option<&[u8; 20]>) -> [u8; 32] {
    let mut data = Vec::with_capacity(action_bytes.len() + 8 + 21);
    data.extend_from_slice(action_bytes);
    data.extend_from_slice(&nonce.to_be_bytes());
    if let Some(vault) = vault_address {
        data.push(1u8);
        data.extend_from_slice(vault);
    } else {
        data.push(0u8);
    }
    keccak(&data)
}

/// Compute EIP-712 struct hash for Agent type.
/// Agent(address source, bytes32 connectionId)
///
/// "source" is the phantom address: 0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48
/// (USDC contract address — used as sentinel in Hyperliquid's phantom agent scheme)
fn compute_agent_hash(connection_id: &[u8; 32]) -> [u8; 32] {
    let type_hash = keccak(b"Agent(address source,bytes32 connectionId)");

    // Phantom source address: 0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48
    let phantom_source: [u8; 20] = [
        0xa0, 0xb8, 0x69, 0x91, 0xc6, 0x21, 0x8b, 0x36, 0xc1, 0xd1,
        0x9d, 0x4a, 0x2e, 0x9e, 0xb0, 0xce, 0x36, 0x06, 0xeb, 0x48,
    ];

    let mut encoded = Vec::with_capacity(3 * 32);
    encoded.extend_from_slice(&type_hash);
    // address padded to 32 bytes (left-padded with zeros)
    encoded.extend_from_slice(&[0u8; 12]);
    encoded.extend_from_slice(&phantom_source);
    encoded.extend_from_slice(connection_id);

    keccak(&encoded)
}

/// Compute final EIP-712 hash to sign: keccak256("\x19\x01" + domain_separator + struct_hash)
fn compute_eip712_hash(domain_separator: &[u8; 32], struct_hash: &[u8; 32]) -> [u8; 32] {
    let mut data = Vec::with_capacity(66);
    data.extend_from_slice(b"\x19\x01");
    data.extend_from_slice(domain_separator);
    data.extend_from_slice(struct_hash);
    keccak(&data)
}

// ═══════════════════════════════════════════════════════════════════════════════
// SIGNATURE COMPONENTS
// ═══════════════════════════════════════════════════════════════════════════════

/// EIP-712 signature components for Hyperliquid request body
#[derive(Debug, Clone)]
pub struct SignatureComponents {
    /// First 32 bytes of signature (0x-prefixed hex)
    pub r: String,
    /// Last 32 bytes of signature (0x-prefixed hex)
    pub s: String,
    /// Recovery ID: 27 or 28
    pub v: u8,
}

// ═══════════════════════════════════════════════════════════════════════════════
// HYPERLIQUID AUTH HANDLER
// ═══════════════════════════════════════════════════════════════════════════════

/// Hyperliquid authentication handler
///
/// Uses EIP-712 signing via `alloy::signers::local::PrivateKeySigner`.
/// Implements L1 action signing (phantom agent) for all trading operations.
#[derive(Clone)]
pub struct HyperliquidAuth {
    /// Ethereum wallet address (lowercase 0x-prefixed)
    wallet_address: String,
    /// Local signer wrapping secp256k1 private key
    signer: Arc<PrivateKeySigner>,
    /// Nonce counter (monotonically increasing timestamp-based)
    nonce_counter: Arc<AtomicU64>,
    /// Is testnet (affects chain ID used in EIP-712 domain)
    is_testnet: bool,
}

impl HyperliquidAuth {
    /// Create new auth handler from credentials.
    ///
    /// `credentials.api_secret` — 0x-prefixed private key hex string (32 bytes).
    /// `credentials.api_key` — 0x-prefixed wallet address (used for Info queries).
    ///   If empty, address is derived from the private key.
    pub fn new(credentials: &Credentials) -> ExchangeResult<Self> {
        if credentials.api_secret.is_empty() {
            return Err(ExchangeError::Auth(
                "Hyperliquid requires wallet private key in api_secret field".to_string()
            ));
        }

        let signer: PrivateKeySigner = credentials.api_secret.parse()
            .map_err(|e| ExchangeError::Auth(format!("Invalid private key: {}", e)))?;

        // Use provided address or derive from private key
        let wallet_address = if !credentials.api_key.is_empty() {
            credentials.api_key.to_lowercase()
        } else {
            format!("0x{}", hex::encode(signer.address().as_slice()))
        };

        let nonce_counter = Arc::new(AtomicU64::new(timestamp_millis()));

        Ok(Self {
            wallet_address,
            signer: Arc::new(signer),
            nonce_counter,
            is_testnet: false,
        })
    }

    /// Create auth handler for specific network
    pub fn new_with_network(credentials: &Credentials, is_testnet: bool) -> ExchangeResult<Self> {
        let mut auth = Self::new(credentials)?;
        auth.is_testnet = is_testnet;
        Ok(auth)
    }

    /// Get wallet address (lowercase 0x-prefixed)
    pub fn wallet_address(&self) -> &str {
        &self.wallet_address
    }

    /// Get next nonce (timestamp-based, monotonically increasing)
    pub fn get_next_nonce(&self) -> u64 {
        let now = timestamp_millis();
        self.nonce_counter.fetch_max(now, Ordering::SeqCst);
        self.nonce_counter.fetch_add(1, Ordering::SeqCst)
    }

    /// EIP-712 chain ID based on network
    fn chain_id(&self) -> u64 {
        if self.is_testnet { 421614 } else { 42161 }
    }

    /// Sign an EIP-712 hash (32 bytes) using the secp256k1 private key.
    /// Returns r, s, v components.
    fn sign_hash(&self, hash: [u8; 32]) -> ExchangeResult<SignatureComponents> {
        use alloy::primitives::B256;
        let hash_b256 = B256::from(hash);
        let sig = self.signer.sign_hash_sync(&hash_b256)
            .map_err(|e| ExchangeError::Auth(format!("Signing failed: {}", e)))?;

        let bytes = sig.as_bytes();
        let r = format!("0x{}", hex::encode(&bytes[..32]));
        let s = format!("0x{}", hex::encode(&bytes[32..64]));
        // alloy returns v as 0/1, Hyperliquid wants 27/28
        let v = if bytes[64] == 0 { 27u8 } else { 28u8 };

        Ok(SignatureComponents { r, s, v })
    }

    /// Sign an L1 action given its pre-encoded msgpack bytes.
    ///
    /// Flow:
    /// 1. connection_id = keccak256(action_bytes + nonce_be + vault_flag)
    /// 2. agent_hash = EIP-712 struct hash of Agent(phantom_source, connection_id)
    /// 3. final = EIP-712 message: keccak("\x19\x01" + domain_separator + agent_hash)
    /// 4. Sign final with secp256k1
    pub fn sign_l1_action(
        &self,
        action_bytes: &[u8],
        nonce: u64,
        vault_address: Option<&[u8; 20]>,
    ) -> ExchangeResult<SignatureComponents> {
        let chain_id = self.chain_id();
        let domain_separator = compute_domain_separator(chain_id);
        let connection_id = compute_connection_id(action_bytes, nonce, vault_address);
        let agent_hash = compute_agent_hash(&connection_id);
        let final_hash = compute_eip712_hash(&domain_separator, &agent_hash);
        self.sign_hash(final_hash)
    }

    /// Sign and build exchange request for a batch of orders
    pub fn sign_order_action(
        &self,
        orders: &[HlOrder],
        grouping: &str,
        vault_address: Option<&[u8; 20]>,
    ) -> ExchangeResult<serde_json::Value> {
        let nonce = self.get_next_nonce();
        let action_bytes = msgpack_order_action(orders, grouping);
        let sig = self.sign_l1_action(&action_bytes, nonce, vault_address)?;
        let action = build_order_action_json(orders, grouping);
        Ok(build_exchange_request(action, nonce, sig, vault_address))
    }

    /// Sign and build exchange request for cancellation
    pub fn sign_cancel_action(
        &self,
        cancels: &[(u32, u64)],
        vault_address: Option<&[u8; 20]>,
    ) -> ExchangeResult<serde_json::Value> {
        let nonce = self.get_next_nonce();
        let action_bytes = msgpack_cancel_action(cancels);
        let sig = self.sign_l1_action(&action_bytes, nonce, vault_address)?;
        let action = build_cancel_action_json(cancels);
        Ok(build_exchange_request(action, nonce, sig, vault_address))
    }

    /// Sign and build exchange request for batch-modifying multiple existing orders
    pub fn sign_batch_modify_action(
        &self,
        modifies: &[(u64, &HlOrder)],
        vault_address: Option<&[u8; 20]>,
    ) -> ExchangeResult<serde_json::Value> {
        let nonce = self.get_next_nonce();
        let action_bytes = msgpack_batch_modify_action(modifies);
        let sig = self.sign_l1_action(&action_bytes, nonce, vault_address)?;

        let modifies_json: Vec<serde_json::Value> = modifies.iter().map(|(oid, order)| {
            let order_type_json = match &order.t {
                HlOrderType::Limit { tif } => serde_json::json!({
                    "limit": { "tif": tif.as_str() }
                }),
                HlOrderType::Trigger { trigger_px, is_market, tpsl } => serde_json::json!({
                    "trigger": {
                        "triggerPx": trigger_px,
                        "isMarket": is_market,
                        "tpsl": tpsl,
                    }
                }),
            };
            let cloid_json = match &order.c {
                Some(c) => serde_json::Value::String(c.clone()),
                None => serde_json::Value::Null,
            };
            serde_json::json!({
                "oid": oid,
                "order": {
                    "a": order.a,
                    "b": order.b,
                    "p": order.p,
                    "s": order.s,
                    "r": order.r,
                    "t": order_type_json,
                    "c": cloid_json,
                }
            })
        }).collect();

        let action = serde_json::json!({
            "type": "batchModify",
            "modifies": modifies_json,
        });
        Ok(build_exchange_request(action, nonce, sig, vault_address))
    }

    /// Sign and build exchange request for modifying an existing order
    pub fn sign_modify_action(
        &self,
        oid: u64,
        order: &HlOrder,
        vault_address: Option<&[u8; 20]>,
    ) -> ExchangeResult<serde_json::Value> {
        let nonce = self.get_next_nonce();
        let action_bytes = msgpack_modify_action(oid, order);
        let sig = self.sign_l1_action(&action_bytes, nonce, vault_address)?;
        let order_type_json = match &order.t {
            HlOrderType::Limit { tif } => serde_json::json!({
                "limit": { "tif": tif.as_str() }
            }),
            HlOrderType::Trigger { trigger_px, is_market, tpsl } => serde_json::json!({
                "trigger": {
                    "triggerPx": trigger_px,
                    "isMarket": is_market,
                    "tpsl": tpsl,
                }
            }),
        };
        let cloid_json = match &order.c {
            Some(c) => serde_json::Value::String(c.clone()),
            None => serde_json::Value::Null,
        };
        let action = serde_json::json!({
            "type": "modify",
            "oid": oid,
            "order": {
                "a": order.a,
                "b": order.b,
                "p": order.p,
                "s": order.s,
                "r": order.r,
                "t": order_type_json,
                "c": cloid_json,
            }
        });
        Ok(build_exchange_request(action, nonce, sig, vault_address))
    }

    /// Sign and build exchange request for leverage update
    pub fn sign_update_leverage(
        &self,
        asset: u32,
        is_cross: bool,
        leverage: u32,
        vault_address: Option<&[u8; 20]>,
    ) -> ExchangeResult<serde_json::Value> {
        let nonce = self.get_next_nonce();
        let action_bytes = msgpack_update_leverage_action(asset, is_cross, leverage);
        let sig = self.sign_l1_action(&action_bytes, nonce, vault_address)?;
        let action = serde_json::json!({
            "type": "updateLeverage",
            "asset": asset,
            "isCross": is_cross,
            "leverage": leverage,
        });
        Ok(build_exchange_request(action, nonce, sig, vault_address))
    }

    /// Sign and build exchange request for isolated margin update
    pub fn sign_update_isolated_margin(
        &self,
        asset: u32,
        is_buy: bool,
        ntli: i64,
        vault_address: Option<&[u8; 20]>,
    ) -> ExchangeResult<serde_json::Value> {
        let nonce = self.get_next_nonce();
        let action_bytes = msgpack_update_isolated_margin_action(asset, is_buy, ntli);
        let sig = self.sign_l1_action(&action_bytes, nonce, vault_address)?;
        let action = serde_json::json!({
            "type": "updateIsolatedMargin",
            "asset": asset,
            "isBuy": is_buy,
            "ntli": ntli,
        });
        Ok(build_exchange_request(action, nonce, sig, vault_address))
    }

    /// Sign and build exchange request for a TWAP order.
    ///
    /// Hyperliquid TWAP uses the `twapOrder` action type, which is separate from
    /// the regular `order` action. The TWAP action includes:
    /// - `a`: asset index (u32)
    /// - `b`: is_buy (bool)
    /// - `s`: size as normalized string
    /// - `r`: reduce_only (bool)
    /// - `m`: duration in minutes (u64, converted from seconds)
    ///
    /// Signing follows the same L1 action mechanism — msgpack-encode the action
    /// dict and sign with EIP-712 phantom agent scheme.
    pub fn sign_twap_action(
        &self,
        asset: u32,
        is_buy: bool,
        size: &str,
        reduce_only: bool,
        duration_seconds: u64,
        vault_address: Option<&[u8; 20]>,
    ) -> ExchangeResult<serde_json::Value> {
        let nonce = self.get_next_nonce();
        // Duration in minutes (Hyperliquid TWAP takes minutes, minimum 5 minutes)
        let duration_minutes = (duration_seconds / 60).max(5);
        let action_bytes = msgpack_twap_action(asset, is_buy, size, reduce_only, duration_minutes);
        let sig = self.sign_l1_action(&action_bytes, nonce, vault_address)?;
        let action = serde_json::json!({
            "type": "twapOrder",
            "twap": {
                "a": asset,
                "b": is_buy,
                "s": size,
                "r": reduce_only,
                "m": duration_minutes,
            },
        });
        Ok(build_exchange_request(action, nonce, sig, vault_address))
    }

    /// Sign and build exchange request for a USD class transfer (spot ↔ perp).
    ///
    /// `to_perp = true`  → transfer USDC from Spot wallet to Perp wallet
    /// `to_perp = false` → transfer USDC from Perp wallet to Spot wallet
    pub fn sign_usd_class_transfer(
        &self,
        amount: &str,
        to_perp: bool,
        vault_address: Option<&[u8; 20]>,
    ) -> ExchangeResult<serde_json::Value> {
        let nonce = self.get_next_nonce();
        let action_bytes = msgpack_usd_class_transfer_action(amount, to_perp);
        let sig = self.sign_l1_action(&action_bytes, nonce, vault_address)?;
        let action = serde_json::json!({
            "type": "usdClassTransfer",
            "amount": amount,
            "toPerp": to_perp,
        });
        Ok(build_exchange_request(action, nonce, sig, vault_address))
    }

    /// Standard headers (no auth headers — Hyperliquid signs via request body)
    pub fn get_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// JSON REQUEST BUILDERS
// ═══════════════════════════════════════════════════════════════════════════════

fn build_order_action_json(orders: &[HlOrder], grouping: &str) -> serde_json::Value {
    let order_jsons: Vec<serde_json::Value> = orders.iter().map(|o| {
        let order_type_json = match &o.t {
            HlOrderType::Limit { tif } => serde_json::json!({
                "limit": { "tif": tif.as_str() }
            }),
            HlOrderType::Trigger { trigger_px, is_market, tpsl } => serde_json::json!({
                "trigger": {
                    "triggerPx": trigger_px,
                    "isMarket": is_market,
                    "tpsl": tpsl,
                }
            }),
        };

        let cloid_json = match &o.c {
            Some(c) => serde_json::Value::String(c.clone()),
            None => serde_json::Value::Null,
        };

        serde_json::json!({
            "a": o.a,
            "b": o.b,
            "p": o.p,
            "s": o.s,
            "r": o.r,
            "t": order_type_json,
            "c": cloid_json,
        })
    }).collect();

    serde_json::json!({
        "type": "order",
        "orders": order_jsons,
        "grouping": grouping,
    })
}

fn build_cancel_action_json(cancels: &[(u32, u64)]) -> serde_json::Value {
    let cancels_json: Vec<serde_json::Value> = cancels.iter()
        .map(|&(a, o)| serde_json::json!({ "a": a, "o": o }))
        .collect();

    serde_json::json!({
        "type": "cancel",
        "cancels": cancels_json,
    })
}

fn build_exchange_request(
    action: serde_json::Value,
    nonce: u64,
    sig: SignatureComponents,
    vault_address: Option<&[u8; 20]>,
) -> serde_json::Value {
    let vault_json = vault_address
        .map(|v| serde_json::Value::String(format!("0x{}", hex::encode(v))))
        .unwrap_or(serde_json::Value::Null);

    serde_json::json!({
        "action": action,
        "nonce": nonce,
        "signature": {
            "r": sig.r,
            "s": sig.s,
            "v": sig.v,
        },
        "vaultAddress": vault_json,
    })
}

// ═══════════════════════════════════════════════════════════════════════════════
// UTILITY FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════════

/// Get Hyperliquid chain ID
#[allow(dead_code)]
pub fn get_chain_id(is_testnet: bool) -> u64 {
    if is_testnet { 421614 } else { 42161 }
}

/// Normalize a float price to Hyperliquid string format (no trailing zeros).
/// e.g. 50000.0 → "50000.0", 0.12500 → "0.125"
pub fn normalize_price(price: f64) -> String {
    let s = format!("{:.8}", price);
    let s = s.trim_end_matches('0');
    if s.ends_with('.') {
        format!("{}0", s)
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_price() {
        assert_eq!(normalize_price(50000.0), "50000.0");
        assert_eq!(normalize_price(50000.5), "50000.5");
        assert_eq!(normalize_price(0.1), "0.1");
        assert_eq!(normalize_price(123.456), "123.456");
    }

    #[test]
    fn test_msgpack_bool() {
        let mut enc = MsgpackEncoder::new();
        enc.write_bool(true);
        enc.write_bool(false);
        let buf = enc.finish();
        assert_eq!(buf, vec![0xc3, 0xc2]);
    }

    #[test]
    fn test_msgpack_str() {
        let mut enc = MsgpackEncoder::new();
        enc.write_str("hello");
        let buf = enc.finish();
        assert_eq!(buf[0], 0xa5); // fixstr len=5
        assert_eq!(&buf[1..], b"hello");
    }

    #[test]
    fn test_msgpack_uint() {
        let mut enc = MsgpackEncoder::new();
        enc.write_uint(0);
        enc.write_uint(127);
        enc.write_uint(128);
        let buf = enc.finish();
        assert_eq!(buf[0], 0);
        assert_eq!(buf[1], 127);
        assert_eq!(buf[2], 0xcc);
        assert_eq!(buf[3], 128);
    }

    #[test]
    fn test_keccak_empty() {
        let result = keccak(b"");
        let expected = hex::decode(
            "c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470"
        ).unwrap();
        assert_eq!(result.as_slice(), expected.as_slice());
    }
}
