//! # Coinglass Connector
//!
//! Реализация коннектора для Coinglass API V4.
//!
//! ## Important Notes
//!
//! Coinglass is a DERIVATIVES ANALYTICS provider, not a trading exchange:
//! - NO standard price/OHLC data (use exchanges for that)
//! - NO trading operations (Trading trait returns UnsupportedOperation)
//! - NO account balances (Account trait returns UnsupportedOperation)
//! - Focus: Liquidations, Open Interest, Funding Rates, Long/Short Ratios
//!
//! ## Custom Methods
//!
//! Since Coinglass doesn't fit standard MarketData/Trading/Account patterns,
//! custom methods are provided as direct connector methods.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use serde_json::{Value};

use crate::core::{
    HttpClient, Credentials,
    ExchangeId, AccountType, Symbol,
    ExchangeError, ExchangeResult,
    Price, Kline, Ticker, OrderBook,
    SymbolInfo,
};
use crate::core::traits::{
    ExchangeIdentity, MarketData,
};
use crate::core::utils::WeightRateLimiter;

use super::endpoints::{CoinglassUrls, CoinglassEndpoint};
use super::auth::CoinglassAuth;
use super::parser::{
    CoinglassParser,
    // Market discovery
    ExchangePairInfo, PairsMarketData, CoinMarketInfo,
    // Liquidations
    LiquidationData, LiquidationHeatmapPoint, LiquidationMapEntry, LiquidationMaxPainData,
    // Open Interest
    OpenInterestOhlc, OpenInterestHistory, OpenInterestVolRatio,
    // Funding rates
    FundingRateData, CurrentFundingRate, FundingRateAggregated,
    // Long/Short
    LongShortRatio, TopLongShortRatio, TakerBuySellVolume,
    // Order book
    BidAskRange, OrderbookHeatmapPoint, LargeOrder,
    // Volume & flows
    CvdPoint, NetFlowPoint, FootprintPoint,
    // Options
    OptionsMaxPain, OptionsOiHistory, OptionsVolumeHistory,
    // On-chain
    ExchangeReserve, ExchangeBalanceHistory, Erc20Transfer, WhaleTransfer,
    TokenUnlock, TokenVesting,
    // ETF
    EtfFlowData, GrayscalePremiumData,
    // HyperLiquid
    HlWhaleAlert, HlWhalePosition, HlWalletPosition, HlPositionDistribution,
    // Technical indicators
    RsiData, MovingAverageData,
};

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Coinglass коннектор
pub struct CoinglassConnector {
    /// HTTP клиент
    http: HttpClient,
    /// Аутентификация
    auth: CoinglassAuth,
    /// URL'ы
    urls: CoinglassUrls,
    /// Rate limiter (varies by subscription tier)
    rate_limiter: Arc<Mutex<WeightRateLimiter>>,
}

impl CoinglassConnector {
    /// Создать новый коннектор
    ///
    /// # Arguments
    /// * `credentials` - API credentials (requires api_key)
    /// * `rate_limit_per_min` - Rate limit (30 for Hobbyist, 80 for Startup, etc.)
    pub async fn new(credentials: Credentials, rate_limit_per_min: u32) -> ExchangeResult<Self> {
        let auth = CoinglassAuth::new(&credentials)?;
        let urls = CoinglassUrls::MAINNET;
        let http = HttpClient::new(30_000)?; // 30 sec timeout

        // Initialize rate limiter: rate_limit requests per 60 seconds
        let rate_limiter = Arc::new(Mutex::new(
            WeightRateLimiter::new(rate_limit_per_min, Duration::from_secs(60))
        ));

        Ok(Self {
            http,
            auth,
            urls,
            rate_limiter,
        })
    }

    /// Create connector with default rate limit (30 req/min - Hobbyist tier)
    pub async fn new_with_default_limit(credentials: Credentials) -> ExchangeResult<Self> {
        Self::new(credentials, 30).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Wait for rate limit if needed
    async fn rate_limit_wait(&self, weight: u32) {
        loop {
            // Scope the lock to ensure it's dropped before await
            let wait_time = {
                let mut limiter = self.rate_limiter.lock().expect("Mutex poisoned");
                if limiter.try_acquire(weight) {
                    return; // Successfully acquired, exit early
                }
                limiter.time_until_ready(weight)
            }; // Lock is dropped here

            if wait_time > Duration::ZERO {
                tokio::time::sleep(wait_time).await;
            }
        }
    }

    /// GET запрос
    async fn get(
        &self,
        endpoint: CoinglassEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        // Wait for rate limit
        self.rate_limit_wait(1).await;

        let base_url = self.urls.rest_url();
        let path = endpoint.path();

        // Build query string
        let query = if params.is_empty() {
            String::new()
        } else {
            let qs: Vec<String> = params.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            format!("?{}", qs.join("&"))
        };

        let url = format!("{}{}{}", base_url, path, query);

        // Add auth headers
        let headers = self.auth.get_headers();

        let response = self.http.get_with_headers(&url, &HashMap::new(), &headers).await?;

        // Check for API errors
        if !CoinglassParser::is_success(&response) {
            let error_msg = CoinglassParser::extract_error(&response);
            let error_code = response.get("code")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(0);
            return Err(ExchangeError::Api {
                code: error_code,
                message: error_msg,
            });
        }

        Ok(response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CUSTOM METHODS - MARKET DISCOVERY
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get list of supported coins
    pub async fn get_supported_coins(&self) -> ExchangeResult<Vec<String>> {
        let response = self.get(CoinglassEndpoint::SupportedCoins, HashMap::new()).await?;
        CoinglassParser::parse_supported_coins(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CUSTOM METHODS - LIQUIDATIONS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get liquidation history
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol (e.g., "BTC", "ETH")
    /// * `interval` - Time interval ("1m", "5m", "15m", "1h", "4h", "12h", "1d")
    /// * `limit` - Number of data points (optional)
    pub async fn get_liquidation_history(
        &self,
        symbol: &str,
        interval: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<LiquidationData>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("interval".to_string(), interval.to_string());

        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }

        let response = self.get(CoinglassEndpoint::LiquidationHistory, params).await?;
        CoinglassParser::parse_liquidations(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CUSTOM METHODS - OPEN INTEREST
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get Open Interest OHLC aggregated history
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol (e.g., "BTC", "ETH")
    /// * `interval` - Time interval ("1m", "5m", "15m", "1h", "4h", "12h", "1d")
    /// * `limit` - Number of data points (optional)
    pub async fn get_open_interest_ohlc(
        &self,
        symbol: &str,
        interval: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<OpenInterestOhlc>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("interval".to_string(), interval.to_string());

        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }

        let response = self.get(CoinglassEndpoint::OpenInterestOhlc, params).await?;
        CoinglassParser::parse_oi_ohlc(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CUSTOM METHODS - FUNDING RATES
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get funding rate history
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol (e.g., "BTC", "ETH")
    /// * `exchange` - Exchange name (optional, e.g., "Binance")
    /// * `limit` - Number of data points (optional)
    pub async fn get_funding_rate_history(
        &self,
        symbol: &str,
        exchange: Option<&str>,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<FundingRateData>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());

        if let Some(exchange) = exchange {
            params.insert("exchange".to_string(), exchange.to_string());
        }

        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }

        let response = self.get(CoinglassEndpoint::FundingRateHistory, params).await?;
        CoinglassParser::parse_funding_rates(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CUSTOM METHODS - LONG/SHORT RATIOS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get long/short ratio history
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol (e.g., "BTC", "ETH")
    /// * `interval` - Time interval ("1m", "5m", "15m", "1h", "4h", "12h", "1d")
    /// * `limit` - Number of data points (optional)
    pub async fn get_long_short_ratio(
        &self,
        symbol: &str,
        interval: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<LongShortRatio>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("interval".to_string(), interval.to_string());

        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }

        let response = self.get(CoinglassEndpoint::LongShortRateHistory, params).await?;
        CoinglassParser::parse_long_short_ratio(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CUSTOM METHODS - MARKET DISCOVERY
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get supported exchange pairs
    pub async fn get_supported_exchange_pairs(&self) -> ExchangeResult<Vec<ExchangePairInfo>> {
        let response = self.get(CoinglassEndpoint::SupportedExchangePairs, HashMap::new()).await?;
        CoinglassParser::parse_exchange_pairs(&response)
    }

    /// Get pairs markets data
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol (e.g., "BTC")
    pub async fn get_pairs_markets(&self, symbol: &str) -> ExchangeResult<Vec<PairsMarketData>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        let response = self.get(CoinglassEndpoint::PairsMarkets, params).await?;
        CoinglassParser::parse_pairs_markets(&response)
    }

    /// Get coins markets data
    pub async fn get_coins_markets(&self) -> ExchangeResult<Vec<CoinMarketInfo>> {
        let response = self.get(CoinglassEndpoint::CoinsMarkets, HashMap::new()).await?;
        CoinglassParser::parse_coins_markets(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CUSTOM METHODS - LIQUIDATIONS (additional)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get liquidation heatmap
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol (e.g., "BTC", "ETH")
    /// * `interval` - Time interval
    /// * `limit` - Number of data points (optional)
    pub async fn get_liquidation_heatmap(
        &self,
        symbol: &str,
        interval: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<LiquidationHeatmapPoint>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("interval".to_string(), interval.to_string());
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }
        let response = self.get(CoinglassEndpoint::LiquidationHeatmap, params).await?;
        CoinglassParser::parse_liquidation_heatmap(&response)
    }

    /// Get liquidation map (price levels with liquidation volumes)
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol (e.g., "BTC", "ETH")
    pub async fn get_liquidation_map(&self, symbol: &str) -> ExchangeResult<Vec<LiquidationMapEntry>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        let response = self.get(CoinglassEndpoint::LiquidationMap, params).await?;
        CoinglassParser::parse_liquidation_map(&response)
    }

    /// Get liquidation max pain price
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol (e.g., "BTC", "ETH")
    pub async fn get_liquidation_max_pain(&self, symbol: &str) -> ExchangeResult<LiquidationMaxPainData> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        let response = self.get(CoinglassEndpoint::LiquidationMaxPain, params).await?;
        CoinglassParser::parse_liquidation_max_pain(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CUSTOM METHODS - OPEN INTEREST (additional)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get aggregated Open Interest OHLC snapshot
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol
    /// * `interval` - Time interval
    pub async fn get_open_interest_aggregated(
        &self,
        symbol: &str,
        interval: &str,
    ) -> ExchangeResult<Vec<OpenInterestOhlc>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("interval".to_string(), interval.to_string());
        let response = self.get(CoinglassEndpoint::OpenInterestAggregated, params).await?;
        CoinglassParser::parse_oi_aggregated(&response)
    }

    /// Get Open Interest history
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol
    /// * `interval` - Time interval
    /// * `limit` - Number of data points (optional)
    pub async fn get_open_interest_history(
        &self,
        symbol: &str,
        interval: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<OpenInterestHistory>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("interval".to_string(), interval.to_string());
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }
        let response = self.get(CoinglassEndpoint::OpenInterestHistory, params).await?;
        CoinglassParser::parse_oi_history(&response)
    }

    /// Get Open Interest / Volume ratio
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol
    /// * `interval` - Time interval
    /// * `limit` - Number of data points (optional)
    pub async fn get_open_interest_vol_ratio(
        &self,
        symbol: &str,
        interval: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<OpenInterestVolRatio>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("interval".to_string(), interval.to_string());
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }
        let response = self.get(CoinglassEndpoint::OpenInterestVolRatio, params).await?;
        CoinglassParser::parse_oi_vol_ratio(&response)
    }

    /// Get Open Interest by coin (chart)
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol
    /// * `interval` - Time interval
    /// * `limit` - Number of data points (optional)
    pub async fn get_open_interest_by_coin(
        &self,
        symbol: &str,
        interval: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<OpenInterestHistory>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("interval".to_string(), interval.to_string());
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }
        let response = self.get(CoinglassEndpoint::OpenInterestByCoin, params).await?;
        CoinglassParser::parse_oi_by_coin(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CUSTOM METHODS - FUNDING RATES (additional)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get current funding rates across exchanges
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol (e.g., "BTC")
    pub async fn get_funding_rate_current(&self, symbol: &str) -> ExchangeResult<Vec<CurrentFundingRate>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        let response = self.get(CoinglassEndpoint::FundingRateCurrent, params).await?;
        CoinglassParser::parse_current_funding_rates(&response)
    }

    /// Get aggregated (OHLC) funding rate
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol
    /// * `interval` - Time interval
    /// * `limit` - Number of data points (optional)
    pub async fn get_funding_rate_aggregated(
        &self,
        symbol: &str,
        interval: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<FundingRateAggregated>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("interval".to_string(), interval.to_string());
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }
        let response = self.get(CoinglassEndpoint::FundingRateAggregated, params).await?;
        CoinglassParser::parse_funding_rate_aggregated(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CUSTOM METHODS - LONG/SHORT RATIOS (additional)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get long/short account ratio
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol
    /// * `exchange` - Exchange name (optional)
    /// * `interval` - Time interval
    /// * `limit` - Number of data points (optional)
    pub async fn get_long_short_account(
        &self,
        symbol: &str,
        exchange: Option<&str>,
        interval: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<LongShortRatio>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("interval".to_string(), interval.to_string());
        if let Some(exchange) = exchange {
            params.insert("exchange".to_string(), exchange.to_string());
        }
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }
        let response = self.get(CoinglassEndpoint::LongShortAccountRatio, params).await?;
        CoinglassParser::parse_long_short_account(&response)
    }

    /// Get global long/short account ratio chart
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol
    /// * `interval` - Time interval
    /// * `limit` - Number of data points (optional)
    pub async fn get_global_long_short_account(
        &self,
        symbol: &str,
        interval: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<LongShortRatio>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("interval".to_string(), interval.to_string());
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }
        let response = self.get(CoinglassEndpoint::LongShortGlobalAccountRatio, params).await?;
        CoinglassParser::parse_global_long_short(&response)
    }

    /// Get top traders long/short position ratio
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol
    /// * `exchange` - Exchange name (optional)
    /// * `interval` - Time interval
    /// * `limit` - Number of data points (optional)
    pub async fn get_top_long_short_position(
        &self,
        symbol: &str,
        exchange: Option<&str>,
        interval: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<TopLongShortRatio>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("interval".to_string(), interval.to_string());
        if let Some(exchange) = exchange {
            params.insert("exchange".to_string(), exchange.to_string());
        }
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }
        let response = self.get(CoinglassEndpoint::TopLongShortPositionRatio, params).await?;
        CoinglassParser::parse_top_long_short_position(&response)
    }

    /// Get top traders long/short account ratio
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol
    /// * `exchange` - Exchange name (optional)
    /// * `interval` - Time interval
    /// * `limit` - Number of data points (optional)
    pub async fn get_top_long_short_account(
        &self,
        symbol: &str,
        exchange: Option<&str>,
        interval: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<TopLongShortRatio>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("interval".to_string(), interval.to_string());
        if let Some(exchange) = exchange {
            params.insert("exchange".to_string(), exchange.to_string());
        }
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }
        let response = self.get(CoinglassEndpoint::TopLongShortAccountRatio, params).await?;
        CoinglassParser::parse_top_long_short_account(&response)
    }

    /// Get taker buy/sell volume chart
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol
    /// * `interval` - Time interval
    /// * `limit` - Number of data points (optional)
    pub async fn get_taker_buy_sell_volume(
        &self,
        symbol: &str,
        interval: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<TakerBuySellVolume>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("interval".to_string(), interval.to_string());
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }
        let response = self.get(CoinglassEndpoint::TakerBuySellVolume, params).await?;
        CoinglassParser::parse_taker_buy_sell_volume(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CUSTOM METHODS - ORDER BOOK ANALYTICS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get bid/ask range data
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol
    /// * `exchange` - Exchange name (optional)
    pub async fn get_bid_ask_range(
        &self,
        symbol: &str,
        exchange: Option<&str>,
    ) -> ExchangeResult<Vec<BidAskRange>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        if let Some(exchange) = exchange {
            params.insert("exchange".to_string(), exchange.to_string());
        }
        let response = self.get(CoinglassEndpoint::BidAskRange, params).await?;
        CoinglassParser::parse_bid_ask_range(&response)
    }

    /// Get orderbook heatmap
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol
    /// * `exchange` - Exchange name (optional)
    /// * `interval` - Time interval
    /// * `limit` - Number of data points (optional)
    pub async fn get_orderbook_heatmap(
        &self,
        symbol: &str,
        exchange: Option<&str>,
        interval: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<OrderbookHeatmapPoint>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("interval".to_string(), interval.to_string());
        if let Some(exchange) = exchange {
            params.insert("exchange".to_string(), exchange.to_string());
        }
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }
        let response = self.get(CoinglassEndpoint::OrderbookHeatmap, params).await?;
        CoinglassParser::parse_orderbook_heatmap(&response)
    }

    /// Get large orders
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol
    /// * `exchange` - Exchange name (optional)
    pub async fn get_large_orders(
        &self,
        symbol: &str,
        exchange: Option<&str>,
    ) -> ExchangeResult<Vec<LargeOrder>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        if let Some(exchange) = exchange {
            params.insert("exchange".to_string(), exchange.to_string());
        }
        let response = self.get(CoinglassEndpoint::LargeOrders, params).await?;
        CoinglassParser::parse_large_orders(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CUSTOM METHODS - VOLUME & FLOWS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get Cumulative Volume Delta (CVD) chart
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol
    /// * `exchange` - Exchange name (optional)
    /// * `interval` - Time interval
    /// * `limit` - Number of data points (optional)
    pub async fn get_cvd(
        &self,
        symbol: &str,
        exchange: Option<&str>,
        interval: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<CvdPoint>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("interval".to_string(), interval.to_string());
        if let Some(exchange) = exchange {
            params.insert("exchange".to_string(), exchange.to_string());
        }
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }
        let response = self.get(CoinglassEndpoint::CumulativeVolumeDelta, params).await?;
        CoinglassParser::parse_cvd(&response)
    }

    /// Get net flow indicator
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol
    /// * `interval` - Time interval
    /// * `limit` - Number of data points (optional)
    pub async fn get_net_flow(
        &self,
        symbol: &str,
        interval: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<NetFlowPoint>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("interval".to_string(), interval.to_string());
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }
        let response = self.get(CoinglassEndpoint::NetFlowIndicator, params).await?;
        CoinglassParser::parse_net_flow(&response)
    }

    /// Get footprint chart data
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol
    /// * `exchange` - Exchange name (optional)
    /// * `interval` - Time interval
    /// * `limit` - Number of data points (optional)
    pub async fn get_footprint(
        &self,
        symbol: &str,
        exchange: Option<&str>,
        interval: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<FootprintPoint>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("interval".to_string(), interval.to_string());
        if let Some(exchange) = exchange {
            params.insert("exchange".to_string(), exchange.to_string());
        }
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }
        let response = self.get(CoinglassEndpoint::FootprintChart, params).await?;
        CoinglassParser::parse_footprint(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CUSTOM METHODS - OPTIONS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get options max pain prices
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol (e.g., "BTC", "ETH")
    pub async fn get_options_max_pain(&self, symbol: &str) -> ExchangeResult<Vec<OptionsMaxPain>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        let response = self.get(CoinglassEndpoint::OptionsMaxPain, params).await?;
        CoinglassParser::parse_options_max_pain(&response)
    }

    /// Get options Open Interest history
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol
    /// * `interval` - Time interval
    /// * `limit` - Number of data points (optional)
    pub async fn get_options_oi_history(
        &self,
        symbol: &str,
        interval: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<OptionsOiHistory>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("interval".to_string(), interval.to_string());
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }
        let response = self.get(CoinglassEndpoint::OptionsOiHistory, params).await?;
        CoinglassParser::parse_options_oi_history(&response)
    }

    /// Get options volume history
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol
    /// * `interval` - Time interval
    /// * `limit` - Number of data points (optional)
    pub async fn get_options_volume_history(
        &self,
        symbol: &str,
        interval: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<OptionsVolumeHistory>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("interval".to_string(), interval.to_string());
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }
        let response = self.get(CoinglassEndpoint::OptionsVolumeHistory, params).await?;
        CoinglassParser::parse_options_volume_history(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CUSTOM METHODS - ON-CHAIN
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get exchange reserve history (on-chain)
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol
    /// * `exchange` - Exchange name (optional)
    /// * `interval` - Time interval
    /// * `limit` - Number of data points (optional)
    pub async fn get_exchange_reserve(
        &self,
        symbol: &str,
        exchange: Option<&str>,
        interval: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<ExchangeReserve>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("interval".to_string(), interval.to_string());
        if let Some(exchange) = exchange {
            params.insert("exchange".to_string(), exchange.to_string());
        }
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }
        let response = self.get(CoinglassEndpoint::ExchangeReserve, params).await?;
        CoinglassParser::parse_exchange_reserve(&response)
    }

    /// Get exchange balance history (on-chain)
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol
    /// * `exchange` - Exchange name (optional)
    /// * `interval` - Time interval
    /// * `limit` - Number of data points (optional)
    pub async fn get_exchange_balance_history(
        &self,
        symbol: &str,
        exchange: Option<&str>,
        interval: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<ExchangeBalanceHistory>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("interval".to_string(), interval.to_string());
        if let Some(exchange) = exchange {
            params.insert("exchange".to_string(), exchange.to_string());
        }
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }
        let response = self.get(CoinglassEndpoint::ExchangeBalanceHistory, params).await?;
        CoinglassParser::parse_exchange_balance_history(&response)
    }

    /// Get ERC-20 large transfers
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol (e.g., "USDT", "USDC")
    /// * `limit` - Number of records (optional)
    pub async fn get_erc20_transfers(
        &self,
        symbol: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<Erc20Transfer>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }
        let response = self.get(CoinglassEndpoint::Erc20Transfers, params).await?;
        CoinglassParser::parse_erc20_transfers(&response)
    }

    /// Get whale transfers
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol (optional)
    /// * `limit` - Number of records (optional)
    pub async fn get_whale_transfers(
        &self,
        symbol: Option<&str>,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<WhaleTransfer>> {
        let mut params = HashMap::new();
        if let Some(symbol) = symbol {
            params.insert("symbol".to_string(), symbol.to_uppercase());
        }
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }
        let response = self.get(CoinglassEndpoint::WhaleTransfers, params).await?;
        CoinglassParser::parse_whale_transfers(&response)
    }

    /// Get upcoming token unlocks
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol (optional, to filter by token)
    /// * `limit` - Number of records (optional)
    pub async fn get_token_unlocks(
        &self,
        symbol: Option<&str>,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<TokenUnlock>> {
        let mut params = HashMap::new();
        if let Some(symbol) = symbol {
            params.insert("symbol".to_string(), symbol.to_uppercase());
        }
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }
        let response = self.get(CoinglassEndpoint::TokenUnlocks, params).await?;
        CoinglassParser::parse_token_unlocks(&response)
    }

    /// Get token vesting schedule
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol
    pub async fn get_token_vesting(&self, symbol: &str) -> ExchangeResult<Vec<TokenVesting>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        let response = self.get(CoinglassEndpoint::TokenVesting, params).await?;
        CoinglassParser::parse_token_vesting(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CUSTOM METHODS - ETF
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get Bitcoin ETF daily flows
    ///
    /// # Arguments
    /// * `limit` - Number of days (optional)
    pub async fn get_btc_etf_flow(&self, limit: Option<u32>) -> ExchangeResult<Vec<EtfFlowData>> {
        let mut params = HashMap::new();
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }
        let response = self.get(CoinglassEndpoint::BtcEtfFlow, params).await?;
        CoinglassParser::parse_etf_flow(&response)
    }

    /// Get Ethereum ETF daily flows
    ///
    /// # Arguments
    /// * `limit` - Number of days (optional)
    pub async fn get_eth_etf_flow(&self, limit: Option<u32>) -> ExchangeResult<Vec<EtfFlowData>> {
        let mut params = HashMap::new();
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }
        let response = self.get(CoinglassEndpoint::EthEtfFlow, params).await?;
        CoinglassParser::parse_etf_flow(&response)
    }

    /// Get Solana ETF daily flows
    ///
    /// # Arguments
    /// * `limit` - Number of days (optional)
    pub async fn get_sol_etf_flow(&self, limit: Option<u32>) -> ExchangeResult<Vec<EtfFlowData>> {
        let mut params = HashMap::new();
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }
        let response = self.get(CoinglassEndpoint::SolEtfFlow, params).await?;
        CoinglassParser::parse_etf_flow(&response)
    }

    /// Get XRP ETF daily flows
    ///
    /// # Arguments
    /// * `limit` - Number of days (optional)
    pub async fn get_xrp_etf_flow(&self, limit: Option<u32>) -> ExchangeResult<Vec<EtfFlowData>> {
        let mut params = HashMap::new();
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }
        let response = self.get(CoinglassEndpoint::XrpEtfFlow, params).await?;
        CoinglassParser::parse_etf_flow(&response)
    }

    /// Get Hong Kong ETF daily flows
    ///
    /// # Arguments
    /// * `limit` - Number of days (optional)
    pub async fn get_hk_etf_flow(&self, limit: Option<u32>) -> ExchangeResult<Vec<EtfFlowData>> {
        let mut params = HashMap::new();
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }
        let response = self.get(CoinglassEndpoint::HkEtfFlow, params).await?;
        CoinglassParser::parse_etf_flow(&response)
    }

    /// Get Grayscale trust premium/discount data
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol (e.g., "BTC", "ETH")
    /// * `limit` - Number of data points (optional)
    pub async fn get_grayscale_premium(
        &self,
        symbol: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<GrayscalePremiumData>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }
        let response = self.get(CoinglassEndpoint::GrayscalePremium, params).await?;
        CoinglassParser::parse_grayscale_premium(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CUSTOM METHODS - HYPERLIQUID
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get HyperLiquid whale alerts
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol (optional)
    /// * `limit` - Number of alerts (optional)
    pub async fn get_hl_whale_alerts(
        &self,
        symbol: Option<&str>,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<HlWhaleAlert>> {
        let mut params = HashMap::new();
        if let Some(symbol) = symbol {
            params.insert("symbol".to_string(), symbol.to_uppercase());
        }
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }
        let response = self.get(CoinglassEndpoint::HyperLiquidWhaleAlert, params).await?;
        CoinglassParser::parse_hl_whale_alerts(&response)
    }

    /// Get HyperLiquid whale positions
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol (optional)
    pub async fn get_hl_whale_positions(&self, symbol: Option<&str>) -> ExchangeResult<Vec<HlWhalePosition>> {
        let mut params = HashMap::new();
        if let Some(symbol) = symbol {
            params.insert("symbol".to_string(), symbol.to_uppercase());
        }
        let response = self.get(CoinglassEndpoint::HyperLiquidWhalePositions, params).await?;
        CoinglassParser::parse_hl_whale_positions(&response)
    }

    /// Get HyperLiquid wallet positions for a specific address
    ///
    /// # Arguments
    /// * `address` - Wallet address
    pub async fn get_hl_wallet_positions(&self, address: &str) -> ExchangeResult<Vec<HlWalletPosition>> {
        let mut params = HashMap::new();
        params.insert("address".to_string(), address.to_string());
        let response = self.get(CoinglassEndpoint::HyperLiquidWalletPositions, params).await?;
        CoinglassParser::parse_hl_wallet_positions(&response)
    }

    /// Get HyperLiquid position distribution for a symbol
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol (e.g., "BTC", "ETH")
    pub async fn get_hl_position_distribution(&self, symbol: &str) -> ExchangeResult<Vec<HlPositionDistribution>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        let response = self.get(CoinglassEndpoint::HyperLiquidPositionDistribution, params).await?;
        CoinglassParser::parse_hl_position_distribution(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CUSTOM METHODS - TECHNICAL INDICATORS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get RSI (Relative Strength Index) data
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol
    /// * `interval` - Time interval
    /// * `period` - RSI period (optional, default 14)
    /// * `limit` - Number of data points (optional)
    pub async fn get_rsi(
        &self,
        symbol: &str,
        interval: &str,
        period: Option<u32>,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<RsiData>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("interval".to_string(), interval.to_string());
        if let Some(period) = period {
            params.insert("period".to_string(), period.to_string());
        }
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }
        let response = self.get(CoinglassEndpoint::Rsi, params).await?;
        CoinglassParser::parse_rsi(&response)
    }

    /// Get Moving Average data
    ///
    /// # Arguments
    /// * `symbol` - Crypto symbol
    /// * `interval` - Time interval
    /// * `ma_type` - MA type (optional, e.g., "SMA", "EMA")
    /// * `period` - MA period (optional, e.g., 20, 50, 200)
    /// * `limit` - Number of data points (optional)
    pub async fn get_moving_average(
        &self,
        symbol: &str,
        interval: &str,
        ma_type: Option<&str>,
        period: Option<u32>,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<MovingAverageData>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("interval".to_string(), interval.to_string());
        if let Some(ma_type) = ma_type {
            params.insert("type".to_string(), ma_type.to_string());
        }
        if let Some(period) = period {
            params.insert("period".to_string(), period.to_string());
        }
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }
        let response = self.get(CoinglassEndpoint::MovingAverage, params).await?;
        CoinglassParser::parse_moving_average(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT IMPLEMENTATIONS
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for CoinglassConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Coinglass
    }

    fn is_testnet(&self) -> bool {
        false // Coinglass only has mainnet
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        // Coinglass is a data provider, doesn't support traditional account types
        vec![]
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// UNSUPPORTED TRAITS
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for CoinglassConnector {
    async fn ping(&self) -> ExchangeResult<()> {
        // Test with supported-coins endpoint (simplest endpoint)
        match self.get(CoinglassEndpoint::SupportedCoins, HashMap::new()).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    async fn get_exchange_info(&self, _account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        let coins = self.get_supported_coins().await?;

        let infos = coins
            .into_iter()
            .map(|coin| SymbolInfo {
                symbol: coin.clone(),
                base_asset: coin,
                quote_asset: "USD".to_string(), // Coinglass tracks derivatives quoted in USD
                status: "TRADING".to_string(),
                price_precision: 2,
                quantity_precision: 0,
                min_quantity: None,
                max_quantity: None,
                step_size: None,
                min_notional: None,
            })
            .collect();

        Ok(infos)
    }

    async fn get_price(&self, _symbol: Symbol, _account_type: AccountType) -> ExchangeResult<Price> {
        Err(ExchangeError::UnsupportedOperation(
            "Coinglass does not provide standard price data. Use get_open_interest_ohlc() or other custom methods.".to_string()
        ))
    }

    async fn get_orderbook(&self, _symbol: Symbol, _depth: Option<u16>, _account_type: AccountType) -> ExchangeResult<OrderBook> {
        Err(ExchangeError::UnsupportedOperation(
            "Coinglass does not provide orderbook data.".to_string()
        ))
    }

    async fn get_klines(&self, _symbol: Symbol, _interval: &str, _limit: Option<u16>, _account_type: AccountType, _end_time: Option<i64>) -> ExchangeResult<Vec<Kline>> {
        Err(ExchangeError::UnsupportedOperation(
            "Coinglass does not provide standard klines. Use get_open_interest_ohlc() for OI OHLC data.".to_string()
        ))
    }

    async fn get_ticker(&self, _symbol: Symbol, _account_type: AccountType) -> ExchangeResult<Ticker> {
        Err(ExchangeError::UnsupportedOperation(
            "Coinglass does not provide ticker data.".to_string()
        ))
    }
}






