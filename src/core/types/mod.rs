//! # V5 Types
//!
//! Типы данных для V5 коннекторов.
//! Независимые от V4, упрощённые для использования агентами.

mod common;
mod market_data;
mod trading;
mod websocket;
pub mod trading_v2;
pub mod responses_v2;

pub use common::*;
pub use market_data::*;
pub use trading::*;
pub use websocket::*;

// V2 type re-exports — coexist with V1 types
pub use trading_v2::{
    OrderTypeV2, TimeInForceV2, OrderRequest, CancelScope, CancelRequest,
    AmendFields, AmendRequest, OrderHistoryFilter, OrdersQuery,
    PositionModification, PositionQuery, BalanceQuery,
    TransferRequest, TransferHistoryFilter,
    SubAccountOperation, SubAccountResult, SubAccount,
    WithdrawRequest, FundsHistoryFilter, FundsRecordType,
    ExchangeCredentials,
};
pub use responses_v2::{
    OrderResult, CancelAllResponse, BracketResponse, OcoResponse,
    AlgoOrderResponse, TransferResponse, DepositAddress, WithdrawResponse,
    FundsRecord, FeeInfo, PlaceOrderResponse,
};
