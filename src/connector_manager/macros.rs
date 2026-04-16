//! # Trait Delegation Macros
//!
//! Declarative macros for generating trait implementations on AnyConnector.
//!
//! ## Architecture
//!
//! Instead of manually writing many-arm match statements for each trait method,
//! these macros generate the boilerplate automatically.
//!
//! ## Usage
//!
//! ```ignore
//! impl_sync_method!(
//!     AnyConnector,
//!     ExchangeIdentity,
//!     exchange_id,
//!     () -> ExchangeId
//! );
//! ```
//!
//! This generates:
//! ```ignore
//! fn exchange_id(&self) -> ExchangeId {
//!     match self {
//!         Self::Binance(c) => c.exchange_id(),
//!         Self::Bybit(c) => c.exchange_id(),
//!         // ... 49 more arms
//!     }
//! }
//! ```

/// Generate a synchronous trait method delegation for AnyConnector
///
/// # Arguments
/// - `$trait_name` - The trait being implemented (e.g., ExchangeIdentity)
/// - `$method` - The method name (e.g., exchange_id)
/// - `$params` - Method parameters (e.g., `(symbol: Symbol)`)
/// - `$ret` - Return type (e.g., `ExchangeId`)
///
/// # Example
/// ```ignore
/// impl_sync_method!(ExchangeIdentity, exchange_id, () -> ExchangeId);
/// impl_sync_method!(ExchangeIdentity, is_testnet, () -> bool);
/// impl_sync_method!(ExchangeIdentity, supported_account_types, () -> Vec<AccountType>);
/// ```
#[macro_export]
macro_rules! impl_sync_method {
    ($trait_name:ident, $method:ident, ($($param_name:ident: $param_type:ty),*) -> $ret:ty) => {
        fn $method(&self $(, $param_name: $param_type)*) -> $ret {
            match self {
                // CEX (18)
                Self::Binance(c) => c.$method($($param_name),*),
                Self::Bybit(c) => c.$method($($param_name),*),
                Self::OKX(c) => c.$method($($param_name),*),
                Self::KuCoin(c) => c.$method($($param_name),*),
                Self::Kraken(c) => c.$method($($param_name),*),
                Self::Coinbase(c) => c.$method($($param_name),*),
                Self::GateIO(c) => c.$method($($param_name),*),
                Self::Bitfinex(c) => c.$method($($param_name),*),
                Self::Bitstamp(c) => c.$method($($param_name),*),
                Self::Gemini(c) => c.$method($($param_name),*),
                Self::MEXC(c) => c.$method($($param_name),*),
                Self::HTX(c) => c.$method($($param_name),*),
                Self::Bitget(c) => c.$method($($param_name),*),
                Self::BingX(c) => c.$method($($param_name),*),
                Self::CryptoCom(c) => c.$method($($param_name),*),
                Self::Upbit(c) => c.$method($($param_name),*),
                Self::Deribit(c) => c.$method($($param_name),*),
                Self::HyperLiquid(c) => c.$method($($param_name),*),

                // DEX (3)
                Self::Lighter(c) => c.$method($($param_name),*),
                Self::Paradex(c) => c.$method($($param_name),*),
                Self::Dydx(c) => c.$method($($param_name),*),

                // Stocks US (5)
                Self::Polygon(c) => c.$method($($param_name),*),
                Self::Finnhub(c) => c.$method($($param_name),*),
                Self::Tiingo(c) => c.$method($($param_name),*),
                Self::Twelvedata(c) => c.$method($($param_name),*),
                Self::Alpaca(c) => c.$method($($param_name),*),

                // Stocks India (5)
                Self::AngelOne(c) => c.$method($($param_name),*),
                Self::Zerodha(c) => c.$method($($param_name),*),
                Self::Upstox(c) => c.$method($($param_name),*),
                Self::Dhan(c) => c.$method($($param_name),*),
                Self::Fyers(c) => c.$method($($param_name),*),

                // Stocks Other (4)
                Self::JQuants(c) => c.$method($($param_name),*),
                Self::Krx(c) => c.$method($($param_name),*),
                Self::Moex(c) => c.$method($($param_name),*),
                Self::Tinkoff(c) => c.$method($($param_name),*),

                // Forex (3)
                Self::Oanda(c) => c.$method($($param_name),*),
                Self::Dukascopy(c) => c.$method($($param_name),*),
                Self::AlphaVantage(c) => c.$method($($param_name),*),

                // On-chain Analytics (2)
                Self::WhaleAlert(c) => c.$method($($param_name),*),
                Self::Bitquery(c) => c.$method($($param_name),*),

                // Brokers (1) + Data Feeds (2)
                Self::IB(c) => c.$method($($param_name),*),
                Self::YahooFinance(c) => c.$method($($param_name),*),
                Self::CryptoCompare(c) => c.$method($($param_name),*),
            }
        }
    };
}

/// Generate an async trait method delegation for AnyConnector
///
/// # Arguments
/// - `$trait_name` - The trait being implemented (e.g., MarketData)
/// - `$method` - The method name (e.g., get_price)
/// - `$params` - Method parameters (e.g., `(symbol: Symbol, account_type: AccountType)`)
/// - `$ret` - Return type (e.g., `ExchangeResult<Price>`)
///
/// # Example
/// ```ignore
/// impl_async_method!(
///     MarketData,
///     get_price,
///     (symbol: Symbol, account_type: AccountType) -> ExchangeResult<Price>
/// );
/// ```
#[macro_export]
macro_rules! impl_async_method {
    ($trait_name:ident, $method:ident, ($($param_name:ident: $param_type:ty),*) -> $ret:ty) => {
        async fn $method(&self $(, $param_name: $param_type)*) -> $ret {
            match self {
                // CEX (18)
                Self::Binance(c) => c.$method($($param_name),*).await,
                Self::Bybit(c) => c.$method($($param_name),*).await,
                Self::OKX(c) => c.$method($($param_name),*).await,
                Self::KuCoin(c) => c.$method($($param_name),*).await,
                Self::Kraken(c) => c.$method($($param_name),*).await,
                Self::Coinbase(c) => c.$method($($param_name),*).await,
                Self::GateIO(c) => c.$method($($param_name),*).await,
                Self::Bitfinex(c) => c.$method($($param_name),*).await,
                Self::Bitstamp(c) => c.$method($($param_name),*).await,
                Self::Gemini(c) => c.$method($($param_name),*).await,
                Self::MEXC(c) => c.$method($($param_name),*).await,
                Self::HTX(c) => c.$method($($param_name),*).await,
                Self::Bitget(c) => c.$method($($param_name),*).await,
                Self::BingX(c) => c.$method($($param_name),*).await,
                Self::CryptoCom(c) => c.$method($($param_name),*).await,
                Self::Upbit(c) => c.$method($($param_name),*).await,
                Self::Deribit(c) => c.$method($($param_name),*).await,
                Self::HyperLiquid(c) => c.$method($($param_name),*).await,

                // DEX (3)
                Self::Lighter(c) => c.$method($($param_name),*).await,
                Self::Paradex(c) => c.$method($($param_name),*).await,
                Self::Dydx(c) => c.$method($($param_name),*).await,

                // Stocks US (5)
                Self::Polygon(c) => c.$method($($param_name),*).await,
                Self::Finnhub(c) => c.$method($($param_name),*).await,
                Self::Tiingo(c) => c.$method($($param_name),*).await,
                Self::Twelvedata(c) => c.$method($($param_name),*).await,
                Self::Alpaca(c) => c.$method($($param_name),*).await,

                // Stocks India (5)
                Self::AngelOne(c) => c.$method($($param_name),*).await,
                Self::Zerodha(c) => c.$method($($param_name),*).await,
                Self::Upstox(c) => c.$method($($param_name),*).await,
                Self::Dhan(c) => c.$method($($param_name),*).await,
                Self::Fyers(c) => c.$method($($param_name),*).await,

                // Stocks Other (4)
                Self::JQuants(c) => c.$method($($param_name),*).await,
                Self::Krx(c) => c.$method($($param_name),*).await,
                Self::Moex(c) => c.$method($($param_name),*).await,
                Self::Tinkoff(c) => c.$method($($param_name),*).await,

                // Forex (3)
                Self::Oanda(c) => c.$method($($param_name),*).await,
                Self::Dukascopy(c) => c.$method($($param_name),*).await,
                Self::AlphaVantage(c) => c.$method($($param_name),*).await,

                // On-chain Analytics (2)
                Self::WhaleAlert(c) => c.$method($($param_name),*).await,
                Self::Bitquery(c) => c.$method($($param_name),*).await,

                // Brokers (1) + Data Feeds (2)
                Self::IB(c) => c.$method($($param_name),*).await,
                Self::YahooFinance(c) => c.$method($($param_name),*).await,
                Self::CryptoCompare(c) => c.$method($($param_name),*).await,
            }
        }
    };
}

// ═══════════════════════════════════════════════════════════════════════════════
// USAGE EXAMPLES
// ═══════════════════════════════════════════════════════════════════════════════

// Example macro expansions - see connector.rs for actual usage
//
// These macros are used to generate trait implementations like:
//
// impl ExchangeIdentity for AnyConnector {
//     impl_sync_method!(ExchangeIdentity, exchange_id, () -> ExchangeId);
// }
//
// #[async_trait]
// impl MarketData for AnyConnector {
//     impl_async_method!(MarketData, get_price, (symbol: Symbol, account_type: AccountType) -> ExchangeResult<Price>);
// }

// ═══════════════════════════════════════════════════════════════════════════════
// TESTING NOTE
// ═══════════════════════════════════════════════════════════════════════════════
//
// Macro functionality is tested indirectly via connector.rs unit tests.
// The macros generate trait implementations that are verified by:
// - test_exchange_identity_trait() - Tests ExchangeIdentity delegation
// - test_market_data_trait_delegates() - Tests MarketData delegation
// - test_any_connector_pattern_match() - Tests match arm generation
//
// If macros fail to generate correct code, the connector.rs tests will fail.
