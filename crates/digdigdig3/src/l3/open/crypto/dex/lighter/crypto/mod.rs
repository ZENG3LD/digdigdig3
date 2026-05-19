//! Cryptographic primitives for Lighter DEX transaction signing.
//!
//! ## Module structure
//!
//! - [`goldilocks`] — Goldilocks field GF(p), p = 2^64 - 2^32 + 1
//! - [`gfp5`] — Quintic extension field GF(p^5), z^5 = 3
//! - [`scalar`] — ECgFp5 scalar field (integers mod n, the group order)
//! - [`ecgfp5`] — ECgFp5 elliptic curve points, scalar multiplication
//! - [`poseidon2`] — Poseidon2 hash function over Goldilocks
//! - [`schnorr`] — Schnorr signatures over ECgFp5
//! - [`tx_hash`] — Transaction field packing and Poseidon2 hashing
//!
//! ## Usage example
//!
//! ```ignore
//! use crate::crypto::dex::lighter::crypto::{
//!     tx_hash::{CreateOrderFields, hash_create_order_bytes, CHAIN_ID_MAINNET},
//!     schnorr::{sign_hashed_message, derive_public_key},
//! };
//!
//! // 1. Hash the transaction
//! let fields = CreateOrderFields { ... };
//! let msg_hash = hash_create_order_bytes(&fields);
//!
//! // 2. Sign with private key (40 bytes, hex-decoded)
//! let sig = sign_hashed_message(&private_key, &msg_hash);
//! // sig is 80 bytes: s[40] || e[40]
//!
//! // 3. Base64-encode the signature for the API
//! let sig_b64 = base64::encode(&sig);
//! ```

pub mod goldilocks;
pub mod gfp5;
pub mod scalar;
pub mod ecgfp5;
pub mod poseidon2;
pub mod schnorr;
pub mod tx_hash;

// Re-export the most commonly used types
pub use goldilocks::GFp;
pub use gfp5::GFp5;
pub use scalar::Scalar;
pub use ecgfp5::Point;
pub use poseidon2::hash_to_quintic_extension;
pub use schnorr::{sign, verify, derive_public_key, sign_hashed_message, verify_hashed_message, Signature};
pub use tx_hash::{
    CreateOrderFields,
    CancelOrderFields,
    WithdrawFields,
    hash_create_order,
    hash_create_order_bytes,
    hash_cancel_order,
    hash_cancel_order_bytes,
    hash_auth_token,
    hash_auth_token_bytes,
    hash_change_pub_key,
    // Chain IDs
    CHAIN_ID_MAINNET,
    CHAIN_ID_TESTNET,
    // Transaction type constants
    TX_TYPE_CREATE_ORDER,
    TX_TYPE_CANCEL_ORDER,
    TX_TYPE_WITHDRAW,
    TX_TYPE_TRANSFER,
    TX_TYPE_CHANGE_PUB_KEY,
    // Order type constants
    ORDER_TYPE_LIMIT,
    ORDER_TYPE_MARKET,
    // Time-in-force constants
    TIF_IMMEDIATE_OR_CANCEL,
    TIF_GOOD_TILL_TIME,
    TIF_POST_ONLY,
    // Special values
    NIL_CLIENT_ORDER_INDEX,
};
