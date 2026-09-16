//! # Centralized Exchanges (CEX)
//!
//! Traditional centralized cryptocurrency exchanges with REST + WebSocket APIs.

#[cfg(feature = "exchange-bitmex")]
pub mod bitmex;
#[cfg(feature = "exchange-binance")]
pub mod binance;
#[cfg(feature = "exchange-bybit")]
pub mod bybit;
#[cfg(feature = "exchange-okx")]
pub mod okx;
#[cfg(feature = "exchange-kucoin")]
pub mod kucoin;
#[cfg(feature = "exchange-kraken")]
pub mod kraken;
#[cfg(feature = "exchange-coinbase")]
pub mod coinbase;
#[cfg(feature = "exchange-gateio")]
pub mod gateio;
#[cfg(feature = "exchange-bitfinex")]
pub mod bitfinex;
#[cfg(feature = "exchange-bitstamp")]
pub mod bitstamp;
#[cfg(feature = "exchange-gemini")]
pub mod gemini;
#[cfg(feature = "exchange-mexc")]
pub mod mexc;
#[cfg(feature = "exchange-htx")]
pub mod htx;
#[cfg(feature = "exchange-bitget")]
pub mod bitget;
#[cfg(feature = "exchange-bingx")]
pub mod bingx;
#[cfg(feature = "exchange-crypto-com")]
pub mod crypto_com;
#[cfg(feature = "exchange-upbit")]
pub mod upbit;
#[cfg(feature = "exchange-deribit")]
pub mod deribit;

// ═══════════════════════════════════════════════════════════════════════════════
// DISABLED EXCHANGES
// ═══════════════════════════════════════════════════════════════════════════════

// DISABLED: Bithumb has persistent infrastructure issues (SSL hangs, 403 geo-blocking)
// pub mod bithumb;
