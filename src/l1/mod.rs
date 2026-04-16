//! # L1 — Price / OHLCV data feeds only
//!
//! L1 connectors provide market data (prices, klines, tickers) but do NOT
//! include L2 orderbook data or execution capabilities.

pub mod free;
pub mod paid;
