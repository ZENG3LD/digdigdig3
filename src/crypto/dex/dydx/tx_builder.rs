//! # dYdX Cosmos Transaction Builder
//!
//! Builds and signs Cosmos SDK transactions for dYdX v4 order placement and
//! cancellation using [`cosmrs`].
//!
//! ## Feature gate
//!
//! This module requires both `onchain-cosmos` and `grpc` features:
//! - `grpc` — for the `MsgPlaceOrder` / `MsgCancelOrder` proto types from `proto.rs`
//! - `onchain-cosmos` — for `cosmrs` tx building / signing
//!
//! ## Transaction flow
//!
//! ```text
//! OrderParams
//!   └─ build_place_order_tx()   ──► TxRaw bytes
//!        ├─ MsgPlaceOrder (prost → Any)
//!        ├─ tx::Body
//!        ├─ AuthInfo  (SignerInfo::auth_info + Fee)
//!        └─ SignDoc::sign() ──► Raw → to_bytes()
//!                              └─ DydxConnector::place_order_grpc() ──► BroadcastTxResponse
//! ```
//!
//! ## Wire format notes
//!
//! dYdX v4 is a Cosmos SDK chain. A signed transaction on the wire is a
//! protobuf-encoded `TxRaw`:
//!
//! ```proto
//! message TxRaw {
//!   bytes body_bytes      = 1;  // proto-encoded TxBody
//!   bytes auth_info_bytes = 2;  // proto-encoded AuthInfo
//!   repeated bytes signatures = 3; // one per signer
//! }
//! ```
//!
//! `body_bytes` contains a `TxBody` with one `Any`-wrapped message:
//! - place:  `/dydxprotocol.clob.MsgPlaceOrder`
//! - cancel: `/dydxprotocol.clob.MsgCancelOrder`

#[cfg(all(feature = "onchain-cosmos", feature = "grpc"))]
pub use inner::*;

#[cfg(all(feature = "onchain-cosmos", feature = "grpc"))]
mod inner {
    use cosmrs::crypto::secp256k1::SigningKey;
    use cosmrs::tx::{self, Body, Fee, SignDoc, SignerInfo};
    use cosmrs::Any;
    use prost::Message as ProstMessage;

    use crate::core::types::ExchangeError;
    use super::super::proto::dydxprotocol::{
        MsgPlaceOrder, MsgCancelOrder, Order, OrderId, SubaccountId,
        OrderSide, OrderConditionType,
        ORDER_FLAG_SHORT_TERM, ORDER_FLAG_LONG_TERM, ORDER_FLAG_CONDITIONAL,
    };
    // OrderTimeInForce is re-exported for callers building ShortTermOrderParams / LongTermOrderParams.
    #[allow(unused_imports)]
    pub use super::super::proto::dydxprotocol::OrderTimeInForce;

    // ─────────────────────────────────────────────────────────────────────────
    // TYPE URL CONSTANTS
    // ─────────────────────────────────────────────────────────────────────────

    /// Protobuf type URL for `dydxprotocol.clob.MsgPlaceOrder`.
    const TYPE_URL_PLACE_ORDER: &str = "/dydxprotocol.clob.MsgPlaceOrder";

    /// Protobuf type URL for `dydxprotocol.clob.MsgCancelOrder`.
    const TYPE_URL_CANCEL_ORDER: &str = "/dydxprotocol.clob.MsgCancelOrder";

    // ─────────────────────────────────────────────────────────────────────────
    // ORDER PARAMETERS
    // ─────────────────────────────────────────────────────────────────────────

    /// Parameters for placing a SHORT_TERM order on dYdX v4.
    ///
    /// For LONG_TERM orders, use [`LongTermOrderParams`].
    /// For CONDITIONAL orders (stop/TP), use [`ConditionalOrderParams`].
    #[derive(Debug, Clone)]
    pub struct ShortTermOrderParams {
        /// bech32 dYdX chain address of the order owner (e.g. `"dydx1abc..."`).
        pub owner_address: String,
        /// Subaccount index (0 for the default subaccount).
        pub subaccount_number: u32,
        /// Client-assigned order ID (arbitrary `u32`, unique per subaccount).
        pub client_id: u32,
        /// CLOB pair ID — maps to a market (0 = BTC-USD, 1 = ETH-USD, …).
        pub clob_pair_id: u32,
        /// Buy (`true`) or sell (`false`).
        pub is_buy: bool,
        /// Order size in base quantums (size / stepBaseQuantum).
        pub quantums: u64,
        /// Order price in subticks (price * subticksPerTick).
        pub subticks: u64,
        /// Block height after which the order expires.
        ///
        /// Must satisfy: `current_block < good_til_block <= current_block + 20`.
        pub good_til_block: u32,
        /// Time-in-force flag (GTC = 0, IOC = 1, PostOnly = 2, FOK = 3).
        pub time_in_force: i32,
        /// Whether this is a reduce-only order (`1`) or normal (`0`).
        pub reduce_only: u32,
    }

    /// Parameters for placing a LONG_TERM order on dYdX v4.
    #[derive(Debug, Clone)]
    pub struct LongTermOrderParams {
        /// bech32 dYdX chain address of the order owner.
        pub owner_address: String,
        /// Subaccount index (0 for default).
        pub subaccount_number: u32,
        /// Client-assigned order ID (arbitrary `u32`, unique per subaccount).
        pub client_id: u32,
        /// CLOB pair ID — maps to a market.
        pub clob_pair_id: u32,
        /// Buy (`true`) or sell (`false`).
        pub is_buy: bool,
        /// Order size in base quantums.
        pub quantums: u64,
        /// Order price in subticks.
        pub subticks: u64,
        /// UTC timestamp in seconds when the order expires (`fixed32` wire type).
        pub good_til_block_time: u32,
        /// Time-in-force flag.
        pub time_in_force: i32,
        /// Whether this is a reduce-only order (`1`) or normal (`0`).
        pub reduce_only: u32,
    }

    /// Parameters for placing a CONDITIONAL order (stop-loss or take-profit) on dYdX v4.
    ///
    /// Conditional orders use `ORDER_FLAG_CONDITIONAL` (32) and require:
    /// - a `condition_type` — either `StopLoss` (1) or `TakeProfit` (2)
    /// - a `trigger_subticks` — the price (in subticks) that activates the order
    /// - the order `subticks` — the limit price to execute at after trigger
    ///   (for a stop-market style, set this equal to `trigger_subticks` or
    ///   use a large/small sentinel to simulate a market sweep)
    ///
    /// Conditional orders must use `good_til_block_time` (LONG_TERM expiry) per
    /// the dYdX protocol — they cannot be SHORT_TERM.
    #[derive(Debug, Clone)]
    pub struct ConditionalOrderParams {
        /// bech32 dYdX chain address of the order owner.
        pub owner_address: String,
        /// Subaccount index (0 for default).
        pub subaccount_number: u32,
        /// Client-assigned order ID.
        pub client_id: u32,
        /// CLOB pair ID.
        pub clob_pair_id: u32,
        /// Buy (`true`) or sell (`false`).
        pub is_buy: bool,
        /// Order size in base quantums.
        pub quantums: u64,
        /// Execution price in subticks (limit price after trigger fires).
        ///
        /// For a stop-market, set to a large value (buy) or 1 (sell) to
        /// simulate a market sweep at any available price.
        pub subticks: u64,
        /// UTC timestamp (seconds) when the order expires.
        pub good_til_block_time: u32,
        /// Time-in-force flag.
        pub time_in_force: i32,
        /// Whether this is a reduce-only order (`1`) or normal (`0`).
        pub reduce_only: u32,
        /// Condition type: `StopLoss` (1) or `TakeProfit` (2).
        pub condition_type: OrderConditionType,
        /// Trigger price in subticks — the order activates when the oracle
        /// price crosses this level.
        pub trigger_subticks: u64,
    }

    /// Parameters for cancelling an existing order on dYdX v4.
    #[derive(Debug, Clone)]
    pub struct CancelOrderParams {
        /// bech32 dYdX chain address of the order owner.
        pub owner_address: String,
        /// Subaccount index (0 for default).
        pub subaccount_number: u32,
        /// Client ID that was used when placing the order.
        pub client_id: u32,
        /// CLOB pair ID.
        pub clob_pair_id: u32,
        /// Order flags used when placing the order (`ORDER_FLAG_SHORT_TERM` or
        /// `ORDER_FLAG_LONG_TERM`).
        pub order_flags: u32,
        /// Same `good_til_block` that was used when placing (SHORT_TERM only).
        pub good_til_block: Option<u32>,
        /// Same `good_til_block_time` that was used when placing (LONG_TERM only).
        pub good_til_block_time: Option<u32>,
    }

    // ─────────────────────────────────────────────────────────────────────────
    // TX BUILDING
    // ─────────────────────────────────────────────────────────────────────────

    /// Encode a prost `Message` as a `cosmrs::Any` with the given `type_url`.
    fn encode_as_any<M: ProstMessage>(type_url: &str, msg: &M) -> Any {
        Any {
            type_url: type_url.to_string(),
            value: msg.encode_to_vec(),
        }
    }

    /// Build and sign a Cosmos `TxRaw` containing a `MsgPlaceOrder` for a
    /// SHORT_TERM order.
    ///
    /// ## Parameters
    ///
    /// - `params` — order parameters
    /// - `signing_key` — secp256k1 private key for signing
    /// - `account_number` — from `CosmosChain::query_account` (first field)
    /// - `sequence` — from `CosmosChain::next_sequence` (must not be reused)
    /// - `chain_id` — Cosmos chain identifier (e.g. `"dydx-mainnet-1"`)
    /// - `fee` — transaction fee; `None` uses zero fee (common for dYdX short-term orders)
    ///
    /// ## Returns
    ///
    /// Serialised `TxRaw` bytes ready to be passed to
    /// `DydxConnector::place_order_grpc`.
    pub fn build_place_order_tx(
        params: &ShortTermOrderParams,
        signing_key: &SigningKey,
        account_number: u64,
        sequence: u64,
        chain_id: &str,
        fee: Option<Fee>,
    ) -> Result<Vec<u8>, ExchangeError> {
        let msg = MsgPlaceOrder {
            order: Some(Order {
                order_id: Some(OrderId {
                    subaccount_id: Some(SubaccountId {
                        owner: params.owner_address.clone(),
                        number: params.subaccount_number,
                    }),
                    client_id: params.client_id,
                    order_flags: ORDER_FLAG_SHORT_TERM,
                    clob_pair_id: params.clob_pair_id,
                }),
                side: if params.is_buy {
                    OrderSide::Buy as i32
                } else {
                    OrderSide::Sell as i32
                },
                quantums: params.quantums,
                subticks: params.subticks,
                good_til_block: Some(params.good_til_block),
                good_til_block_time: None,
                time_in_force: params.time_in_force,
                reduce_only: params.reduce_only,
                client_metadata: 0,
                condition_type: OrderConditionType::Unspecified as i32,
                conditional_order_trigger_subticks: 0,
            }),
        };

        let any = encode_as_any(TYPE_URL_PLACE_ORDER, &msg);
        build_and_sign_tx(any, signing_key, account_number, sequence, chain_id, fee)
    }

    /// Build and sign a Cosmos `TxRaw` containing a `MsgPlaceOrder` for a
    /// LONG_TERM order.
    pub fn build_place_long_term_order_tx(
        params: &LongTermOrderParams,
        signing_key: &SigningKey,
        account_number: u64,
        sequence: u64,
        chain_id: &str,
        fee: Option<Fee>,
    ) -> Result<Vec<u8>, ExchangeError> {
        let msg = MsgPlaceOrder {
            order: Some(Order {
                order_id: Some(OrderId {
                    subaccount_id: Some(SubaccountId {
                        owner: params.owner_address.clone(),
                        number: params.subaccount_number,
                    }),
                    client_id: params.client_id,
                    order_flags: ORDER_FLAG_LONG_TERM,
                    clob_pair_id: params.clob_pair_id,
                }),
                side: if params.is_buy {
                    OrderSide::Buy as i32
                } else {
                    OrderSide::Sell as i32
                },
                quantums: params.quantums,
                subticks: params.subticks,
                good_til_block: None,
                good_til_block_time: Some(params.good_til_block_time),
                time_in_force: params.time_in_force,
                reduce_only: params.reduce_only,
                client_metadata: 0,
                condition_type: OrderConditionType::Unspecified as i32,
                conditional_order_trigger_subticks: 0,
            }),
        };

        let any = encode_as_any(TYPE_URL_PLACE_ORDER, &msg);
        build_and_sign_tx(any, signing_key, account_number, sequence, chain_id, fee)
    }

    /// Build and sign a Cosmos `TxRaw` containing a `MsgPlaceOrder` for a
    /// CONDITIONAL order (stop-loss or take-profit).
    ///
    /// Conditional orders use `ORDER_FLAG_CONDITIONAL` (32) and are identified by
    /// both a `condition_type` and a `conditional_order_trigger_subticks` that
    /// specifies the oracle-price level at which the order activates.
    ///
    /// ## Parameters
    ///
    /// - `params` — conditional order parameters
    /// - `signing_key` — secp256k1 private key for signing
    /// - `account_number` — from `CosmosChain::query_account`
    /// - `sequence` — from `CosmosChain::next_sequence`
    /// - `chain_id` — Cosmos chain identifier
    /// - `fee` — transaction fee; `None` uses zero fee
    pub fn build_place_conditional_order_tx(
        params: &ConditionalOrderParams,
        signing_key: &SigningKey,
        account_number: u64,
        sequence: u64,
        chain_id: &str,
        fee: Option<Fee>,
    ) -> Result<Vec<u8>, ExchangeError> {
        let msg = MsgPlaceOrder {
            order: Some(Order {
                order_id: Some(OrderId {
                    subaccount_id: Some(SubaccountId {
                        owner: params.owner_address.clone(),
                        number: params.subaccount_number,
                    }),
                    client_id: params.client_id,
                    order_flags: ORDER_FLAG_CONDITIONAL,
                    clob_pair_id: params.clob_pair_id,
                }),
                side: if params.is_buy {
                    OrderSide::Buy as i32
                } else {
                    OrderSide::Sell as i32
                },
                quantums: params.quantums,
                subticks: params.subticks,
                good_til_block: None,
                good_til_block_time: Some(params.good_til_block_time),
                time_in_force: params.time_in_force,
                reduce_only: params.reduce_only,
                client_metadata: 0,
                condition_type: params.condition_type as i32,
                conditional_order_trigger_subticks: params.trigger_subticks,
            }),
        };

        let any = encode_as_any(TYPE_URL_PLACE_ORDER, &msg);
        build_and_sign_tx(any, signing_key, account_number, sequence, chain_id, fee)
    }

    /// Build and sign a Cosmos `TxRaw` containing a `MsgCancelOrder`.
    ///
    /// ## Parameters
    ///
    /// - `params` — cancellation parameters (must match the original order's fields)
    /// - `signing_key` — secp256k1 private key
    /// - `account_number` — from `CosmosChain::query_account`
    /// - `sequence` — from `CosmosChain::next_sequence`
    /// - `chain_id` — Cosmos chain identifier
    /// - `fee` — transaction fee; `None` uses zero fee
    pub fn build_cancel_order_tx(
        params: &CancelOrderParams,
        signing_key: &SigningKey,
        account_number: u64,
        sequence: u64,
        chain_id: &str,
        fee: Option<Fee>,
    ) -> Result<Vec<u8>, ExchangeError> {
        let msg = MsgCancelOrder {
            order_id: Some(OrderId {
                subaccount_id: Some(SubaccountId {
                    owner: params.owner_address.clone(),
                    number: params.subaccount_number,
                }),
                client_id: params.client_id,
                order_flags: params.order_flags,
                clob_pair_id: params.clob_pair_id,
            }),
            good_til_block: params.good_til_block,
            good_til_block_time: params.good_til_block_time,
        };

        let any = encode_as_any(TYPE_URL_CANCEL_ORDER, &msg);
        build_and_sign_tx(any, signing_key, account_number, sequence, chain_id, fee)
    }

    /// Internal helper: wrap an `Any` message in a `Body`, build `AuthInfo`,
    /// sign with secp256k1, and serialise the resulting `Raw` (TxRaw).
    fn build_and_sign_tx(
        msg: Any,
        signing_key: &SigningKey,
        account_number: u64,
        sequence: u64,
        chain_id: &str,
        fee: Option<Fee>,
    ) -> Result<Vec<u8>, ExchangeError> {
        // ── TxBody ──────────────────────────────────────────────────────────
        let body = Body::new(vec![msg], "", 0u32);

        // ── AuthInfo ────────────────────────────────────────────────────────
        let public_key = signing_key.public_key();
        let signer_info = SignerInfo::single_direct(Some(public_key.into()), sequence);

        let fee = fee.unwrap_or_else(|| {
            // dYdX SHORT_TERM orders use zero fee
            Fee::from_amount_and_gas(
                cosmrs::Coin {
                    denom: "adydx".parse().expect("'adydx' is a valid Cosmos denom"),
                    amount: 0u128,
                },
                0u64,
            )
        });

        let auth_info = signer_info.auth_info(fee);

        // ── Chain ID (tendermint chain::Id) ──────────────────────────────────
        let chain_id_parsed: cosmrs::tendermint::chain::Id = chain_id.parse().map_err(|e| {
            ExchangeError::InvalidRequest(format!(
                "DydxTxBuilder: invalid chain_id '{}': {}",
                chain_id, e
            ))
        })?;

        // ── SignDoc & Signature ──────────────────────────────────────────────
        let sign_doc = SignDoc::new(&body, &auth_info, &chain_id_parsed, account_number)
            .map_err(|e| {
                ExchangeError::InvalidRequest(format!(
                    "DydxTxBuilder: SignDoc construction failed: {}",
                    e
                ))
            })?;

        let raw: tx::Raw = sign_doc.sign(signing_key).map_err(|e| {
            ExchangeError::Auth(format!(
                "DydxTxBuilder: signing failed: {}",
                e
            ))
        })?;

        // ── Serialise TxRaw ─────────────────────────────────────────────────
        let tx_bytes = raw.to_bytes().map_err(|e| {
            ExchangeError::InvalidRequest(format!(
                "DydxTxBuilder: TxRaw serialisation failed: {}",
                e
            ))
        })?;

        Ok(tx_bytes)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // SIGNING KEY HELPERS
    // ─────────────────────────────────────────────────────────────────────────

    /// Parse a secp256k1 `SigningKey` from a 32-byte raw private key (big-endian).
    ///
    /// This is a convenience wrapper around `cosmrs::crypto::secp256k1::SigningKey`.
    /// The raw bytes can be loaded from a hex-encoded string, a keyfile, or
    /// derived from a BIP-32 mnemonic via `cosmrs::bip32`.
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// let key_bytes = hex::decode("your_32_byte_hex_key")?;
    /// let signing_key = signing_key_from_bytes(&key_bytes)?;
    /// ```
    pub fn signing_key_from_bytes(key_bytes: &[u8]) -> Result<SigningKey, ExchangeError> {
        SigningKey::from_slice(key_bytes).map_err(|e| {
            ExchangeError::Auth(format!(
                "DydxTxBuilder: invalid signing key bytes: {}",
                e
            ))
        })
    }

    // ─────────────────────────────────────────────────────────────────────────
    // TESTS
    // ─────────────────────────────────────────────────────────────────────────

    #[cfg(test)]
    mod tests {
        use super::*;

        /// Generate a deterministic test key (non-zero bytes — required by secp256k1).
        fn test_signing_key() -> SigningKey {
            let key_bytes = [1u8; 32];
            SigningKey::from_slice(&key_bytes).expect("test key is valid")
        }

        #[test]
        fn test_build_place_order_tx_produces_bytes() {
            let params = ShortTermOrderParams {
                owner_address: "dydx1eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeepkfq2m".to_string(),
                subaccount_number: 0,
                client_id: 1,
                clob_pair_id: 0, // BTC-USD
                is_buy: true,
                quantums: 1_000_000,
                subticks: 30_000_000_000,
                good_til_block: 100,
                time_in_force: OrderTimeInForce::Unspecified as i32,
                reduce_only: 0,
            };

            let key = test_signing_key();
            let result = build_place_order_tx(
                &params,
                &key,
                42,   // account_number
                7,    // sequence
                "dydx-mainnet-1",
                None, // zero fee
            );

            assert!(result.is_ok(), "tx build should succeed: {:?}", result.err());
            let bytes = result.unwrap();
            assert!(!bytes.is_empty(), "serialised tx must not be empty");
        }

        #[test]
        fn test_build_cancel_order_tx_produces_bytes() {
            let params = CancelOrderParams {
                owner_address: "dydx1eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeepkfq2m".to_string(),
                subaccount_number: 0,
                client_id: 1,
                clob_pair_id: 0,
                order_flags: ORDER_FLAG_SHORT_TERM,
                good_til_block: Some(100),
                good_til_block_time: None,
            };

            let key = test_signing_key();
            let result = build_cancel_order_tx(
                &params,
                &key,
                42,
                8,
                "dydx-mainnet-1",
                None,
            );

            assert!(result.is_ok(), "cancel tx build should succeed: {:?}", result.err());
            let bytes = result.unwrap();
            assert!(!bytes.is_empty(), "serialised cancel tx must not be empty");
        }

        #[test]
        fn test_build_place_long_term_order_tx_produces_bytes() {
            let params = LongTermOrderParams {
                owner_address: "dydx1eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeepkfq2m".to_string(),
                subaccount_number: 0,
                client_id: 2,
                clob_pair_id: 1, // ETH-USD
                is_buy: false,
                quantums: 5_000_000,
                subticks: 2_000_000_000,
                good_til_block_time: 1_800_000_000u32,
                time_in_force: OrderTimeInForce::Unspecified as i32,
                reduce_only: 0,
            };

            let key = test_signing_key();
            let result = build_place_long_term_order_tx(
                &params,
                &key,
                42,
                9,
                "dydx-mainnet-1",
                None,
            );

            assert!(result.is_ok(), "long-term tx build should succeed: {:?}", result.err());
        }

        #[test]
        fn test_invalid_chain_id_returns_error() {
            let params = ShortTermOrderParams {
                owner_address: "dydx1eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeepkfq2m".to_string(),
                subaccount_number: 0,
                client_id: 1,
                clob_pair_id: 0,
                is_buy: true,
                quantums: 1_000_000,
                subticks: 30_000_000_000,
                good_til_block: 100,
                time_in_force: 0,
                reduce_only: 0,
            };

            let key = test_signing_key();
            // An empty chain ID is invalid
            let result = build_place_order_tx(&params, &key, 42, 7, "", None);
            assert!(result.is_err(), "empty chain ID should produce an error");
        }

        #[test]
        fn test_signing_key_from_bytes() {
            let key_bytes = [2u8; 32];
            let key = signing_key_from_bytes(&key_bytes);
            assert!(key.is_ok(), "valid 32-byte key should parse");
        }

        #[test]
        fn test_signing_key_from_invalid_bytes() {
            // A 31-byte slice is invalid for secp256k1
            let key_bytes = [1u8; 31];
            let key = signing_key_from_bytes(&key_bytes);
            assert!(key.is_err(), "31-byte key should fail");
        }
    }
}
