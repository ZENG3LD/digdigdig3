//! # Centralized Exchanges (CEX)
//!
//! Traditional centralized cryptocurrency exchanges with REST + WebSocket APIs.

pub mod binance;
pub mod bybit;
pub mod okx;
pub mod kucoin;
pub mod kraken;
pub mod coinbase;
pub mod gateio;
pub mod bitfinex;
pub mod bitstamp;
pub mod gemini;
pub mod mexc;
pub mod htx;
pub mod bitget;
pub mod bingx;
pub mod crypto_com;
pub mod upbit;
pub mod deribit;
pub mod hyperliquid;

// ═══════════════════════════════════════════════════════════════════════════════
// DISABLED EXCHANGES
// ═══════════════════════════════════════════════════════════════════════════════

// DISABLED: Bithumb has persistent infrastructure issues (SSL hangs, 403 geo-blocking)
// pub mod bithumb;
