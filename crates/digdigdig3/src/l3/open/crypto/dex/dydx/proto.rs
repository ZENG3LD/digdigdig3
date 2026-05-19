//! # dYdX v4 Protobuf Message Types
//!
//! Hand-written prost structs for dYdX v4 order placement and cancellation.
//! These mirror the canonical proto definitions from:
//!   `proto/dydxprotocol/clob/tx.proto`
//!   `proto/dydxprotocol/clob/order.proto`
//!   `proto/dydxprotocol/subaccounts/types.proto`
//!
//! Only the types required for order placement and cancellation are included.
//! Full Cosmos SDK transaction wrapping (TxBody, AuthInfo, TxRaw) is not
//! included here — that requires `cosmrs` or a separate signing crate.
//!
//! ## Feature gate
//! Everything in this module is compiled only when `features = ["grpc"]`.

#[cfg(feature = "grpc")]
pub mod dydxprotocol {
    // ─────────────────────────────────────────────────────────────────────────
    // SubaccountId
    //   proto path: dydxprotocol.subaccounts.SubaccountId
    //   field 1: owner  (string)  — bech32 dYdX chain address
    //   field 2: number (uint32)  — subaccount index (0 for default)
    // ─────────────────────────────────────────────────────────────────────────

    /// Identifies a dYdX v4 subaccount.
    ///
    /// `owner` is the bech32 chain address (e.g. `"dydx1abc…"`).
    /// `number` is the subaccount index — `0` for the default subaccount.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct SubaccountId {
        /// bech32 address of the owner account.
        #[prost(string, tag = "1")]
        pub owner: ::prost::alloc::string::String,

        /// Subaccount index (0 = default).
        #[prost(uint32, tag = "2")]
        pub number: u32,
    }

    // ─────────────────────────────────────────────────────────────────────────
    // OrderId
    //   proto path: dydxprotocol.clob.OrderId
    //   field 1: subaccount_id (SubaccountId)
    //   field 2: client_id     (uint32) — caller-assigned, arbitrary
    //   field 3: order_flags   (uint32) — 0=SHORT_TERM, 64=LONG_TERM, 32=CONDITIONAL
    //   field 4: clob_pair_id  (uint32) — market ID (0=BTC-USD, 1=ETH-USD, …)
    // ─────────────────────────────────────────────────────────────────────────

    /// Uniquely identifies an order on dYdX v4.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct OrderId {
        /// The subaccount placing the order.
        #[prost(message, optional, tag = "1")]
        pub subaccount_id: ::core::option::Option<SubaccountId>,

        /// Client-assigned identifier. Chosen by the caller; random u32 works.
        #[prost(uint32, tag = "2")]
        pub client_id: u32,

        /// Order flags encoding the order category:
        /// - `0`  → `SHORT_TERM`
        /// - `32` → `CONDITIONAL`
        /// - `64` → `LONG_TERM`
        #[prost(uint32, tag = "3")]
        pub order_flags: u32,

        /// CLOB pair ID — maps to a market (e.g. `0` = BTC-USD, `1` = ETH-USD).
        #[prost(uint32, tag = "4")]
        pub clob_pair_id: u32,
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Order.Side enum
    // ─────────────────────────────────────────────────────────────────────────

    /// Order side: buy or sell.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum OrderSide {
        /// Default / unspecified (do not use).
        Unspecified = 0,
        /// Buy side.
        Buy = 1,
        /// Sell side.
        Sell = 2,
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Order.TimeInForce enum
    // ─────────────────────────────────────────────────────────────────────────

    /// Time-in-force for a dYdX order.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum OrderTimeInForce {
        /// Good-Till-Cancel (unspecified maps to GTC).
        Unspecified = 0,
        /// Immediate-or-Cancel.
        Ioc = 1,
        /// Post-Only — rejected if it would immediately cross.
        PostOnly = 2,
        /// Fill-Or-Kill.
        FillOrKill = 3,
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Order.ConditionType enum
    // ─────────────────────────────────────────────────────────────────────────

    /// Condition type for conditional orders.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum OrderConditionType {
        /// No condition (standard order).
        Unspecified = 0,
        /// Stop-loss trigger.
        StopLoss = 1,
        /// Take-profit trigger.
        TakeProfit = 2,
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Order.GoodTilOneof  — oneof expiry encoding
    //
    // prost represents protobuf `oneof` as Option fields.  Both fields share
    // proto field numbers 5 and 6; callers set exactly one.
    // ─────────────────────────────────────────────────────────────────────────

    /// A full dYdX v4 `Order` message.
    ///
    /// Embedded inside `MsgPlaceOrder`.
    ///
    /// **SHORT_TERM constraint:** `good_til_block` must satisfy
    /// `currentBlockHeight < goodTilBlock <= currentBlockHeight + 20`.
    ///
    /// **LONG_TERM / CONDITIONAL:** set `good_til_block_time` (UTC epoch
    /// seconds as `fixed32`).
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Order {
        /// Unique identifier for this order.
        #[prost(message, optional, tag = "1")]
        pub order_id: ::core::option::Option<OrderId>,

        /// Buy or sell.
        #[prost(enumeration = "OrderSide", tag = "2")]
        pub side: i32,

        /// Size in base quantums (size / stepBaseQuantum, integer).
        #[prost(uint64, tag = "3")]
        pub quantums: u64,

        /// Price in subticks (price * subticksPerTick, integer).
        #[prost(uint64, tag = "4")]
        pub subticks: u64,

        /// SHORT_TERM expiry: block height after which the order is invalid.
        /// Set this OR `good_til_block_time`, not both.
        #[prost(uint32, optional, tag = "5")]
        pub good_til_block: ::core::option::Option<u32>,

        /// LONG_TERM / CONDITIONAL expiry: UTC timestamp in seconds (fixed32 on
        /// wire, represented as u32 in Rust because prost maps `fixed32` to `u32`).
        /// Set this OR `good_til_block`, not both.
        #[prost(fixed32, optional, tag = "6")]
        pub good_til_block_time: ::core::option::Option<u32>,

        /// Time-in-force (maps to `OrderTimeInForce` enum values).
        #[prost(enumeration = "OrderTimeInForce", tag = "7")]
        pub time_in_force: i32,

        /// `1` = reduce-only, `0` = normal.
        #[prost(uint32, tag = "8")]
        pub reduce_only: u32,

        /// Client metadata field (arbitrary u32 for tagging, usually 0).
        #[prost(uint32, tag = "9")]
        pub client_metadata: u32,

        /// Condition type for conditional orders (UNSPECIFIED for regular orders).
        #[prost(enumeration = "OrderConditionType", tag = "10")]
        pub condition_type: i32,

        /// Trigger price in subticks for conditional orders (0 for non-conditional).
        #[prost(uint64, tag = "11")]
        pub conditional_order_trigger_subticks: u64,
    }

    // ─────────────────────────────────────────────────────────────────────────
    // MsgPlaceOrder
    //   proto path: dydxprotocol.clob.MsgPlaceOrder
    //   Broadcast to: cosmos.tx.v1beta1.Service/BroadcastTx
    // ─────────────────────────────────────────────────────────────────────────

    /// Transaction message to place an order on dYdX v4.
    ///
    /// This message must be wrapped in a signed Cosmos SDK `TxRaw` before
    /// broadcasting.  The gRPC service path is:
    /// `/dydxprotocol.clob.Msg/PlaceOrder`
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct MsgPlaceOrder {
        /// The order to place.
        #[prost(message, optional, tag = "1")]
        pub order: ::core::option::Option<Order>,
    }

    // ─────────────────────────────────────────────────────────────────────────
    // MsgPlaceOrderResponse
    // ─────────────────────────────────────────────────────────────────────────

    /// Response returned by the validator after `MsgPlaceOrder`.
    ///
    /// For SHORT_TERM orders the response is empty on success; the order is
    /// identified by `OrderId` in the request.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct MsgPlaceOrderResponse {}

    // ─────────────────────────────────────────────────────────────────────────
    // MsgCancelOrder
    //   proto path: dydxprotocol.clob.MsgCancelOrder
    // ─────────────────────────────────────────────────────────────────────────

    /// Transaction message to cancel an existing order on dYdX v4.
    ///
    /// The `good_til_*` fields must be set to the same value that was used when
    /// the order was originally placed.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct MsgCancelOrder {
        /// Identifies the order to cancel.
        #[prost(message, optional, tag = "1")]
        pub order_id: ::core::option::Option<OrderId>,

        /// SHORT_TERM: block height used when placing the order.
        /// Set this OR `good_til_block_time`.
        #[prost(uint32, optional, tag = "2")]
        pub good_til_block: ::core::option::Option<u32>,

        /// LONG_TERM / CONDITIONAL: UTC timestamp in seconds used when placing.
        /// Set this OR `good_til_block`.
        #[prost(fixed32, optional, tag = "3")]
        pub good_til_block_time: ::core::option::Option<u32>,
    }

    // ─────────────────────────────────────────────────────────────────────────
    // MsgCancelOrderResponse
    // ─────────────────────────────────────────────────────────────────────────

    /// Response returned after `MsgCancelOrder`.  Empty on success.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct MsgCancelOrderResponse {}

    // ─────────────────────────────────────────────────────────────────────────
    // Constants — order flag values
    // ─────────────────────────────────────────────────────────────────────────

    /// Order flag for SHORT_TERM orders (expire within 20 blocks).
    pub const ORDER_FLAG_SHORT_TERM: u32 = 0;

    /// Order flag for LONG_TERM orders (expire at an absolute UTC timestamp).
    pub const ORDER_FLAG_LONG_TERM: u32 = 64;

    /// Order flag for CONDITIONAL orders (stop/take-profit triggers).
    pub const ORDER_FLAG_CONDITIONAL: u32 = 32;

    // ─────────────────────────────────────────────────────────────────────────
    // Cosmos BroadcastTx wrapper types
    //   proto path: cosmos.tx.v1beta1
    //   Only the fields needed for broadcasting a pre-signed TxRaw are included.
    // ─────────────────────────────────────────────────────────────────────────

    /// Broadcast mode for a Cosmos SDK transaction.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum BroadcastMode {
        /// Default / unspecified (not recommended).
        Unspecified = 0,
        /// Broadcast and wait until the transaction is included in a block.
        Block = 1,
        /// Broadcast and return after the transaction is accepted into the mempool.
        Sync = 2,
        /// Fire and forget — return immediately without mempool confirmation.
        Async = 3,
    }

    /// Request to broadcast a signed transaction to a Cosmos validator node.
    ///
    /// `tx_bytes` is the protobuf-serialized `TxRaw` message (body_bytes +
    /// auth_info_bytes + signatures).
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct BroadcastTxRequest {
        /// Serialized `TxRaw` bytes.
        #[prost(bytes = "vec", tag = "1")]
        pub tx_bytes: ::prost::alloc::vec::Vec<u8>,

        /// How to wait for confirmation.
        #[prost(enumeration = "BroadcastMode", tag = "2")]
        pub mode: i32,
    }

    /// Result of an individual transaction that was broadcast.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct TxResponse {
        /// Block height at which the transaction was included (`0` if async/sync).
        #[prost(int64, tag = "1")]
        pub height: i64,

        /// Transaction hash (hex string).
        #[prost(string, tag = "2")]
        pub txhash: ::prost::alloc::string::String,

        /// Cosmos SDK error code (`0` = success).
        #[prost(uint32, tag = "3")]
        pub code: u32,

        /// Human-readable error log (empty on success).
        #[prost(string, tag = "6")]
        pub raw_log: ::prost::alloc::string::String,
    }

    /// Response from `cosmos.tx.v1beta1.Service/BroadcastTx`.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct BroadcastTxResponse {
        /// Transaction execution result.
        #[prost(message, optional, tag = "1")]
        pub tx_response: ::core::option::Option<TxResponse>,
    }
}
