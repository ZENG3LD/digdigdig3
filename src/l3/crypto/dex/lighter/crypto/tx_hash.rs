//! Transaction hashing for Lighter DEX.
//!
//! Packs transaction fields into Goldilocks field elements and hashes them
//! with Poseidon2's `hash_to_quintic_extension` to produce a GFp5 message hash.
//!
//! ## Transaction Type Constants
//! From `lighter-go/types/txtypes/constants.go`:
//! - CreateOrder: 14
//! - CancelOrder: 15
//! - Transfer: 12
//! - Withdraw: 13
//! - ChangePubKey: 8
//!
//! ## Chain IDs
//! - Mainnet: 304
//! - Testnet: 300

use super::goldilocks::GFp;
use super::gfp5::GFp5;
use super::poseidon2::{hash_to_quintic_extension, bytes_to_field_elements};

// Transaction type constants (from lighter-go/types/txtypes/constants.go)
pub const TX_TYPE_CREATE_ORDER: u32 = 14;
pub const TX_TYPE_CANCEL_ORDER: u32 = 15;
pub const TX_TYPE_CANCEL_ALL_ORDERS: u32 = 16;
pub const TX_TYPE_MODIFY_ORDER: u32 = 17;
pub const TX_TYPE_TRANSFER: u32 = 12;
pub const TX_TYPE_WITHDRAW: u32 = 13;
pub const TX_TYPE_CHANGE_PUB_KEY: u32 = 8;
pub const TX_TYPE_CREATE_SUB_ACCOUNT: u32 = 9;
pub const TX_TYPE_MINT_SHARES: u32 = 18;
pub const TX_TYPE_BURN_SHARES: u32 = 19;
pub const TX_TYPE_UPDATE_LEVERAGE: u32 = 20;
pub const TX_TYPE_UPDATE_MARGIN: u32 = 29;
pub const TX_TYPE_STAKE_ASSETS: u32 = 35;
pub const TX_TYPE_UNSTAKE_ASSETS: u32 = 36;
pub const TX_TYPE_APPROVE_INTEGRATOR: u32 = 45;
pub const TX_TYPE_CREATE_GROUPED_ORDERS: u32 = 28;

// Chain IDs
pub const CHAIN_ID_MAINNET: u32 = 304;
pub const CHAIN_ID_TESTNET: u32 = 300;

// Special sentinel values
/// Nil client order index (use when no client order index provided)
pub const NIL_CLIENT_ORDER_INDEX: i64 = (1i64 << 48) - 1;

// Order types
pub const ORDER_TYPE_LIMIT: u8 = 0;
pub const ORDER_TYPE_MARKET: u8 = 1;
pub const ORDER_TYPE_STOP_LOSS: u8 = 2;
pub const ORDER_TYPE_STOP_LOSS_LIMIT: u8 = 3;
pub const ORDER_TYPE_TAKE_PROFIT: u8 = 4;
pub const ORDER_TYPE_TAKE_PROFIT_LIMIT: u8 = 5;
pub const ORDER_TYPE_TWAP: u8 = 6;

// Time-in-force values
pub const TIF_IMMEDIATE_OR_CANCEL: u8 = 0;
pub const TIF_GOOD_TILL_TIME: u8 = 1;
pub const TIF_POST_ONLY: u8 = 2;

// --- Field element encoding helpers ---

/// Create a Goldilocks field element from a u32 value.
/// Go SDK: `g.FromUint32(v)` — cast to u64, put into field.
#[inline]
pub fn from_u32(v: u32) -> GFp {
    GFp::from_canonical_u64(v as u64)
}

/// Create a Goldilocks field element from an i64 value.
/// Go SDK: `g.FromInt64(v)` — reinterpret bit pattern as u64 (wrapping cast).
/// This means negative values become large positive values via two's complement.
#[inline]
pub fn from_i64(v: i64) -> GFp {
    // Reinterpret as u64 bit pattern (Go's int64-to-uint64 cast is a bit reinterpretation)
    GFp::from_canonical_u64(v as u64)
}

/// Create a Goldilocks field element from a u64 value.
#[inline]
pub fn from_u64(v: u64) -> GFp {
    GFp::from_canonical_u64(v)
}

// --- Create Order ---

/// Fields for an L2 CreateOrder transaction.
#[derive(Debug, Clone)]
pub struct CreateOrderFields {
    pub chain_id: u32,
    pub nonce: i64,
    pub expired_at: i64,          // Unix milliseconds
    pub account_index: i64,
    pub api_key_index: u8,
    pub market_index: i16,
    pub client_order_index: i64,   // Use NIL_CLIENT_ORDER_INDEX if unset
    pub base_amount: i64,
    pub price: u32,
    pub is_ask: bool,
    pub order_type: u8,
    pub time_in_force: u8,
    pub reduce_only: bool,
    pub trigger_price: u32,
    pub order_expiry: i64,         // Unix milliseconds; -1 = now + 28 days
}

/// Hash a CreateOrder transaction to a GFp5 element.
/// Packs 16 Goldilocks field elements in exact order.
pub fn hash_create_order(fields: &CreateOrderFields) -> GFp5 {
    let elems: [GFp; 16] = [
        from_u32(fields.chain_id),                          // [0] chain_id = 304
        from_u32(TX_TYPE_CREATE_ORDER),                      // [1] tx_type = 14
        from_i64(fields.nonce),                             // [2] nonce
        from_i64(fields.expired_at),                        // [3] expired_at (unix ms)
        from_i64(fields.account_index),                     // [4] account_index
        from_u32(fields.api_key_index as u32),              // [5] api_key_index
        from_u32(fields.market_index as u32),               // [6] market_index
        from_i64(fields.client_order_index),                // [7] client_order_index
        from_i64(fields.base_amount),                       // [8] base_amount
        from_u32(fields.price),                             // [9] price
        from_u32(fields.is_ask as u32),                     // [10] is_ask (0 or 1)
        from_u32(fields.order_type as u32),                 // [11] order_type
        from_u32(fields.time_in_force as u32),              // [12] time_in_force
        from_u32(fields.reduce_only as u32),                // [13] reduce_only
        from_u32(fields.trigger_price),                     // [14] trigger_price
        from_i64(fields.order_expiry),                      // [15] order_expiry
    ];
    hash_to_quintic_extension(&elems)
}

/// Hash a CreateOrder transaction and return the 40-byte little-endian hash.
pub fn hash_create_order_bytes(fields: &CreateOrderFields) -> [u8; 40] {
    hash_create_order(fields).encode()
}

// --- Cancel Order ---

/// Fields for an L2 CancelOrder transaction.
#[derive(Debug, Clone)]
pub struct CancelOrderFields {
    pub chain_id: u32,
    pub nonce: i64,
    pub expired_at: i64,   // Unix milliseconds
    pub account_index: i64,
    pub api_key_index: u8,
    pub market_index: i16,
    pub index: i64,        // Client Order Index OR server Order Index
}

/// Hash a CancelOrder transaction to a GFp5 element.
/// Packs 8 Goldilocks field elements.
pub fn hash_cancel_order(fields: &CancelOrderFields) -> GFp5 {
    let elems: [GFp; 8] = [
        from_u32(fields.chain_id),                         // [0] chain_id = 304
        from_u32(TX_TYPE_CANCEL_ORDER),                     // [1] tx_type = 15
        from_i64(fields.nonce),                            // [2] nonce
        from_i64(fields.expired_at),                       // [3] expired_at (unix ms)
        from_i64(fields.account_index),                    // [4] account_index
        from_u32(fields.api_key_index as u32),             // [5] api_key_index
        from_u32(fields.market_index as u32),              // [6] market_index
        from_i64(fields.index),                            // [7] order_index
    ];
    hash_to_quintic_extension(&elems)
}

/// Hash a CancelOrder transaction and return the 40-byte little-endian hash.
pub fn hash_cancel_order_bytes(fields: &CancelOrderFields) -> [u8; 40] {
    hash_cancel_order(fields).encode()
}

// --- Auth Token ---

/// Hash the auth token message string to a GFp5 element.
///
/// The message format is: `"{deadline_unix_seconds}:{account_index}:{api_key_index}"`
/// The string bytes are split into 8-byte chunks → Goldilocks field elements,
/// then hashed with Poseidon2.
///
/// # Arguments
/// - `deadline`: Unix timestamp in seconds (NOT milliseconds)
/// - `account_index`: Account index
/// - `api_key_index`: API key index
pub fn hash_auth_token(deadline: u64, account_index: i64, api_key_index: u8) -> GFp5 {
    let message = format!("{}:{}:{}", deadline, account_index, api_key_index);
    let elems = bytes_to_field_elements(message.as_bytes());
    hash_to_quintic_extension(&elems)
}

/// Hash the auth token message bytes and return 40-byte hash.
pub fn hash_auth_token_bytes(deadline: u64, account_index: i64, api_key_index: u8) -> [u8; 40] {
    hash_auth_token(deadline, account_index, api_key_index).encode()
}

// --- Withdraw ---

/// Fields for an L2 Withdraw transaction.
#[derive(Debug, Clone)]
pub struct WithdrawFields {
    pub chain_id: u32,
    pub nonce: i64,
    pub expired_at: i64,
    pub from_account_index: i64,
    pub api_key_index: u8,
    pub asset_index: i16,
    pub route_type: u8,
    pub amount: u64,  // Split into two 32-bit Goldilocks elements
}

/// Hash a Withdraw transaction to a GFp5 element.
pub fn hash_withdraw(fields: &WithdrawFields) -> GFp5 {
    let elems: [GFp; 10] = [
        from_u32(fields.chain_id),
        from_u32(TX_TYPE_WITHDRAW),
        from_i64(fields.nonce),
        from_i64(fields.expired_at),
        from_i64(fields.from_account_index),
        from_u32(fields.api_key_index as u32),
        from_u32(fields.asset_index as u32),
        from_u32(fields.route_type as u32),
        from_u64(fields.amount & 0xFFFFFFFF),   // lower 32 bits
        from_u64(fields.amount >> 32),           // upper 32 bits
    ];
    hash_to_quintic_extension(&elems)
}

// --- ChangePubKey ---

/// Hash a ChangePubKey transaction to a GFp5 element.
/// The new public key (40 bytes) is encoded as 5 Goldilocks field elements.
pub fn hash_change_pub_key(
    chain_id: u32,
    nonce: i64,
    expired_at: i64,
    account_index: i64,
    api_key_index: u8,
    new_pub_key: &[u8; 40],
) -> GFp5 {
    let mut elems = Vec::with_capacity(11);
    elems.push(from_u32(chain_id));
    elems.push(from_u32(TX_TYPE_CHANGE_PUB_KEY));
    elems.push(from_i64(nonce));
    elems.push(from_i64(expired_at));
    elems.push(from_i64(account_index));
    elems.push(from_u32(api_key_index as u32));

    // Encode 40-byte public key as 5 Goldilocks elements (8 bytes each, little-endian)
    for i in 0..5 {
        let chunk: [u8; 8] = new_pub_key[i * 8..(i + 1) * 8].try_into().unwrap();
        let v = u64::from_le_bytes(chunk);
        elems.push(GFp::from_canonical_u64(v));
    }

    hash_to_quintic_extension(&elems)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_i64_negative() {
        // Negative i64 values are reinterpreted as u64 bit patterns
        let v = from_i64(-1i64);
        // -1 as u64 = 0xFFFFFFFFFFFFFFFF — but this exceeds p, so it wraps
        // Actually from_canonical_u64 uses from_u64_reduce which handles this correctly
        // The Montgomery multiplication will reduce mod p
        let canonical = v.to_u64();
        // -1 mod p = p - 1
        assert_eq!(canonical, GFp::MOD - 1, "from_i64(-1) = p - 1");
    }

    #[test]
    fn test_hash_create_order_deterministic() {
        let fields = CreateOrderFields {
            chain_id: CHAIN_ID_MAINNET,
            nonce: 1,
            expired_at: 1741000000000,
            account_index: 12345,
            api_key_index: 3,
            market_index: 0,
            client_order_index: NIL_CLIENT_ORDER_INDEX,
            base_amount: 10000,
            price: 400000,
            is_ask: false,
            order_type: ORDER_TYPE_LIMIT,
            time_in_force: TIF_POST_ONLY,
            reduce_only: false,
            trigger_price: 0,
            order_expiry: 1741000000000 + 28 * 24 * 60 * 60 * 1000,
        };

        let hash1 = hash_create_order_bytes(&fields);
        let hash2 = hash_create_order_bytes(&fields);
        assert_eq!(hash1, hash2, "hash is deterministic");
    }

    #[test]
    fn test_hash_cancel_order_deterministic() {
        let fields = CancelOrderFields {
            chain_id: CHAIN_ID_MAINNET,
            nonce: 2,
            expired_at: 1741000000000,
            account_index: 12345,
            api_key_index: 3,
            market_index: 0,
            index: 9876,
        };

        let hash1 = hash_cancel_order_bytes(&fields);
        let hash2 = hash_cancel_order_bytes(&fields);
        assert_eq!(hash1, hash2, "cancel hash is deterministic");
    }

    #[test]
    fn test_hash_auth_token_format() {
        let deadline: u64 = 1741000000;
        let account: i64 = 12345;
        let api_key: u8 = 1;

        let hash = hash_auth_token_bytes(deadline, account, api_key);
        // Just check it produces a non-zero result
        assert_ne!(hash, [0u8; 40], "auth token hash is non-zero");
    }

    #[test]
    fn test_nil_client_order_index() {
        assert_eq!(NIL_CLIENT_ORDER_INDEX, 281474976710655, "NIL is (1<<48)-1");
    }
}
