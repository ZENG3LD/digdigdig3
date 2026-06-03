//! Extended public market data trait (funding, liquidations, OI, premium, etc.).
//!
//! All methods have default impls returning `UnsupportedOperation`. Each exchange
//! overrides only the methods it natively supports. Callers consume via
//! `Arc<dyn CoreConnector>` through dynamic dispatch.


use crate::core::types::{
    AccountType, ExchangeError, ExchangeResult, FundingRate, HistoricalVolatility, Kline,
    Liquidation, LongShortRatio, MarkPrice, OpenInterest, PublicTrade, SymbolInput,
};

/// Extended public market data — derivatives analytics, liquidations, OI, funding history.
///
/// All methods default to `UnsupportedOperation`. Connectors override only
/// the methods they natively support. Callers use this trait via `Arc<dyn CoreConnector>`.
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
pub trait MarketDataPublic: Send + Sync {
    /// Recent public trades for a symbol.
    async fn get_recent_trades(
        &self,
        symbol: SymbolInput<'_>,
        limit: Option<u32>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<PublicTrade>> {
        let _ = (symbol, limit, account_type);
        Err(ExchangeError::UnsupportedOperation(
            "get_recent_trades not supported".into(),
        ))
    }

    /// Historical liquidation events, optionally filtered by symbol and time range.
    async fn get_liquidation_history(
        &self,
        symbol: Option<SymbolInput<'_>>,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Liquidation>> {
        let _ = (symbol, start_time, end_time, limit, account_type);
        Err(ExchangeError::UnsupportedOperation(
            "get_liquidation_history not supported".into(),
        ))
    }

    /// Historical open interest snapshots.
    async fn get_open_interest_history(
        &self,
        symbol: SymbolInput<'_>,
        period: &str,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<OpenInterest>> {
        let _ = (symbol, period, start_time, end_time, limit, account_type);
        Err(ExchangeError::UnsupportedOperation(
            "get_open_interest_history not supported".into(),
        ))
    }

    /// Mark price and index price snapshot(s) for a symbol.
    ///
    /// `symbol` is `None` to retrieve data for all symbols.
    async fn get_premium_index(
        &self,
        symbol: Option<SymbolInput<'_>>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<MarkPrice>> {
        let _ = (symbol, account_type);
        Err(ExchangeError::UnsupportedOperation(
            "get_premium_index not supported".into(),
        ))
    }

    /// Historical long/short ratio snapshots.
    async fn get_long_short_ratio_history(
        &self,
        symbol: SymbolInput<'_>,
        period: &str,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<LongShortRatio>> {
        let _ = (symbol, period, start_time, end_time, limit, account_type);
        Err(ExchangeError::UnsupportedOperation(
            "get_long_short_ratio_history not supported".into(),
        ))
    }

    /// Mark price klines (OHLCV based on mark price).
    async fn get_mark_price_klines(
        &self,
        symbol: SymbolInput<'_>,
        interval: &str,
        limit: Option<u32>,
        account_type: AccountType,
        end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let _ = (symbol, interval, limit, account_type, end_time);
        Err(ExchangeError::UnsupportedOperation(
            "get_mark_price_klines not supported".into(),
        ))
    }

    /// Index price klines (OHLCV based on index/spot price).
    async fn get_index_price_klines(
        &self,
        symbol: SymbolInput<'_>,
        interval: &str,
        limit: Option<u32>,
        account_type: AccountType,
        end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let _ = (symbol, interval, limit, account_type, end_time);
        Err(ExchangeError::UnsupportedOperation(
            "get_index_price_klines not supported".into(),
        ))
    }

    /// Premium-index klines (OHLCV of the funding-basis premium index).
    async fn get_premium_index_klines(
        &self,
        symbol: SymbolInput<'_>,
        interval: &str,
        limit: Option<u32>,
        account_type: AccountType,
        end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let _ = (symbol, interval, limit, account_type, end_time);
        Err(ExchangeError::UnsupportedOperation(
            "get_premium_index_klines not supported".into(),
        ))
    }

    /// Historical realized volatility series — trailing window (≈15 days, hourly buckets).
    ///
    /// `currency` is the base asset in exchange-native form (e.g. `"BTC"`, `"ETH"`
    /// for Deribit). Other exchanges may accept a symbol or index name instead.
    ///
    /// Default: `UnsupportedOperation` — only Deribit overrides this method.
    async fn get_historical_volatility(
        &self,
        currency: &str,
    ) -> ExchangeResult<Vec<HistoricalVolatility>> {
        let _ = currency;
        Err(ExchangeError::UnsupportedOperation(
            "get_historical_volatility: not available on this exchange \
             (Deribit-only: public/get_historical_volatility)"
                .into(),
        ))
    }

    /// Historical funding rate payments.
    async fn get_funding_rate_history(
        &self,
        symbol: SymbolInput<'_>,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<FundingRate>> {
        let _ = (symbol, start_time, end_time, limit, account_type);
        Err(ExchangeError::UnsupportedOperation(
            "get_funding_rate_history not supported".into(),
        ))
    }
}
