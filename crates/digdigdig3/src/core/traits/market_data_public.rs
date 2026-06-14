//! Extended public market data trait (funding, liquidations, OI, premium, etc.).
//!
//! All methods have default impls returning `UnsupportedOperation`. Each exchange
//! overrides only the methods it natively supports. Callers consume via
//! `Arc<dyn CoreConnector>` through dynamic dispatch.


use crate::core::types::{
    AccountType, AggTrade, Basis, ExchangeError, ExchangeResult, FundingRate, HistoricalVolatility,
    Kline, Liquidation, LongShortRatio, MarkPrice, OpenInterest, PublicTrade, SymbolInput,
    TakerVolume,
};

/// Extended public market data — derivatives analytics, liquidations, OI, funding history.
///
/// All methods default to `UnsupportedOperation`. Connectors override only
/// the methods they natively support. Callers use this trait via `Arc<dyn CoreConnector>`.
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
pub trait MarketDataPublic: Send + Sync {
    /// Resolve the caller-facing display symbol to the wire id used for WS
    /// subscribe frames and REST market-data parameters.
    ///
    /// Equals the display symbol everywhere **except** venues whose subscription
    /// id differs from the display name (HyperLiquid spot: display "MU/USDC",
    /// wire "@107"). The default implementation is a passthrough — every
    /// connector that does not override this has zero extra cost.
    ///
    /// Called by the Station subscribe path after the canonical/raw pair is
    /// determined, so WS subscribe frames carry the correct wire id. REST
    /// market-data methods that self-resolve internally (e.g. `get_klines`)
    /// do not need to call this externally — Station uses it only for the
    /// subscribe frame and the SeriesKey symbol (so event routing back to
    /// the subscriber works despite the display-vs-wire mismatch).
    async fn resolve_market_symbol(&self, symbol: &str, account_type: AccountType) -> String {
        let _ = account_type;
        symbol.to_string()
    }

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

    /// Aggregated trades — server-side compression of fills (same price / side /
    /// taker order) into one record carrying the underlying fill-id range.
    ///
    /// Override ONLY on venues whose aggTrade feed is materially richer or
    /// deeper than `get_recent_trades`: Binance spot (`/api/v3/aggTrades`,
    /// history to 2017 via `from_id`), Binance USDⓈ-M (`/fapi/v1/aggTrades`),
    /// MEXC spot (`/api/v3/aggTrades`). On venues where the "aggTrade" channel
    /// is byte-identical to the raw trade feed (Bybit/OKX/Bitget/MEXC-fut/dYdX),
    /// do NOT override — callers fall back to `get_recent_trades`.
    ///
    /// Returns `AggTrade` (NOT `PublicTrade`) — carries `first_trade_id` /
    /// `last_trade_id` (the merged-fill range = the whole point of aggregation),
    /// plus `non_rpi_qty` / `is_best_match` where the venue provides them.
    /// `from_id` paginates by aggregate id (Binance cursor). Default: `UnsupportedOperation`.
    async fn get_agg_trades(
        &self,
        symbol: SymbolInput<'_>,
        limit: Option<u32>,
        from_id: Option<u64>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<AggTrade>> {
        let _ = (symbol, limit, from_id, account_type);
        Err(ExchangeError::UnsupportedOperation(
            "get_agg_trades not supported".into(),
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

    /// Historical futures basis (futures − spot index) time series.
    async fn get_basis_history(
        &self,
        symbol: SymbolInput<'_>,
        period: &str,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Basis>> {
        let _ = (symbol, period, start_time, end_time, limit, account_type);
        Err(ExchangeError::UnsupportedOperation(
            "get_basis_history not supported".into(),
        ))
    }

    /// Historical taker buy/sell volume time series (aggressor-side flow).
    async fn get_taker_volume_history(
        &self,
        symbol: SymbolInput<'_>,
        period: &str,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<TakerVolume>> {
        let _ = (symbol, period, start_time, end_time, limit, account_type);
        Err(ExchangeError::UnsupportedOperation(
            "get_taker_volume_history not supported".into(),
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
