//! # V5 Types
//!
//! Типы данных для V5 коннекторов.
//! Независимые от V4, упрощённые для использования агентами.

mod common;
mod market_data;
mod trading;
mod websocket;
mod responses;
pub mod onchain;

pub use common::*;
pub use market_data::*;
pub use trading::*;
pub use websocket::*;
pub use responses::{
    OrderResult, CancelAllResponse, BracketResponse, OcoResponse,
    AlgoOrderResponse, TransferResponse, DepositAddress, WithdrawResponse,
    FundsRecord, FeeInfo, PlaceOrderResponse,
    MarginBorrowResponse, MarginRepayResponse, MarginInterestRecord,
    EarnProduct, EarnPosition, ConvertQuote,
    ClosedPnlRecord, LongShortRatio,
    FundingPayment, FundingFilter,
    LedgerEntry, LedgerEntryType, LedgerFilter,
};
