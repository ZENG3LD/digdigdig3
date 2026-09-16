//! Forex brokers and data providers

#[cfg(feature = "exchange-oanda")]
pub mod oanda;
#[cfg(feature = "exchange-dukascopy")]
pub mod dukascopy;
