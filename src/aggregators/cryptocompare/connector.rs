//! CryptoCompare connector implementation

use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{
    Symbol, AccountType, Asset, Price, Ticker, Kline, OrderBook,
    ExchangeId, ExchangeError, ExchangeResult,
    Order, OrderSide, Quantity, Balance, AccountInfo, Position, SymbolInfo,
};
use crate::core::traits::{ExchangeIdentity, MarketData, Trading, Account, Positions};

use super::endpoints::*;
use super::auth::*;
use super::parser::*;

/// CryptoCompare connector
pub struct CryptoCompareConnector {
    client: Client,
    auth: CryptoCompareAuth,
    endpoints: CryptoCompareEndpoints,
}

impl CryptoCompareConnector {
    /// Create new connector with explicit auth
    pub fn new(auth: CryptoCompareAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: CryptoCompareEndpoints::default(),
        }
    }

    /// Create connector from environment variables
    ///
    /// Looks for CRYPTOCOMPARE_API_KEY environment variable.
    pub fn from_env() -> Self {
        Self::new(CryptoCompareAuth::from_env())
    }

    /// Create connector without API key (public endpoints only, low rate limits)
    pub fn public() -> Self {
        Self::new(CryptoCompareAuth::public())
    }

    /// Internal: Make GET request
    async fn get(
        &self,
        endpoint: CryptoCompareEndpoint,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        // Add authentication (query parameter is preferred for CryptoCompare)
        self.auth.sign_query(&mut params);

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status_code = response.status().as_u16() as i32;
            let message = format!(
                "HTTP {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            );
            return Err(ExchangeError::Api { code: status_code, message });
        }

        response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT: ExchangeIdentity (ALWAYS implement)
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for CryptoCompareConnector {
    fn exchange_name(&self) -> &'static str {
        "cryptocompare"
    }

    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::CryptoCompare
    }

    fn is_testnet(&self) -> bool {
        false // CryptoCompare doesn't have testnet
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        // CryptoCompare is data provider for spot crypto only
        vec![AccountType::Spot]
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT: MarketData (Implement what makes sense)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for CryptoCompareConnector {
    /// Get current price
    async fn get_price(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Price> {
        let (fsym, tsym) = format_symbol(&symbol);

        let mut params = HashMap::new();
        params.insert("fsym".to_string(), fsym);
        params.insert("tsyms".to_string(), tsym.clone());

        let response = self.get(CryptoCompareEndpoint::Price, params).await?;
        CryptoCompareParser::parse_price(&response, &tsym)
    }

    /// Get ticker (24h stats)
    async fn get_ticker(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let (fsym, tsym) = format_symbol(&symbol);

        let mut params = HashMap::new();
        params.insert("fsyms".to_string(), fsym.clone());
        params.insert("tsyms".to_string(), tsym.clone());

        let response = self.get(CryptoCompareEndpoint::PriceMultiFull, params).await?;
        CryptoCompareParser::parse_ticker(&response, &fsym, &tsym)
    }

    /// Get orderbook
    ///
    /// NOTE: CryptoCompare orderbook data is PAID TIER ONLY (WebSocket Channel 16).
    /// Not available via REST API.
    async fn get_orderbook(
        &self,
        _symbol: Symbol,
        _depth: Option<u16>,
        _account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        Err(ExchangeError::UnsupportedOperation(
            "CryptoCompare orderbook data requires paid tier and WebSocket connection - not available via REST API".to_string()
        ))
    }

    /// Get klines/candles
    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        _account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let (fsym, tsym) = format_symbol(&symbol);
        let (endpoint, aggregate) = map_interval_aggregate(interval);

        let mut params = HashMap::new();
        params.insert("fsym".to_string(), fsym);
        params.insert("tsym".to_string(), tsym);
        params.insert("aggregate".to_string(), aggregate.to_string());

        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        } else {
            params.insert("limit".to_string(), "100".to_string());
        }

        let response = self.get(endpoint, params).await?;
        CryptoCompareParser::parse_klines(&response)
    }

    /// Ping endpoint (check connection)
    async fn ping(&self) -> ExchangeResult<()> {
        // CryptoCompare doesn't have a dedicated ping endpoint
        // We'll use a lightweight endpoint to verify connection
        let mut params = HashMap::new();
        params.insert("fsym".to_string(), "BTC".to_string());
        params.insert("tsyms".to_string(), "USD".to_string());

        let _ = self.get(CryptoCompareEndpoint::Price, params).await?;
        Ok(())
    }

    /// Get all coins listed on CryptoCompare
    async fn get_exchange_info(&self, _account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        let response = self.get(CryptoCompareEndpoint::CoinList, HashMap::new()).await?;
        let symbols = CryptoCompareParser::parse_symbols(&response)?;

        let infos = symbols
            .into_iter()
            .map(|symbol| SymbolInfo {
                symbol: symbol.clone(),
                base_asset: symbol,
                quote_asset: "USD".to_string(), // CryptoCompare tracks crypto vs USD by default
                status: "TRADING".to_string(),
                price_precision: 8,
                quantity_precision: 8,
                min_quantity: None,
                max_quantity: None,
                step_size: None,
                min_notional: None,
            })
            .collect();

        Ok(infos)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT: Trading (UnsupportedOperation for data providers)
// ═══════════════════════════════════════════════════════════════════════════════



// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT: Account (UnsupportedOperation for data providers)
// ═══════════════════════════════════════════════════════════════════════════════



// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT: Positions (UnsupportedOperation for data providers)
// ═══════════════════════════════════════════════════════════════════════════════



// ═══════════════════════════════════════════════════════════════════════════════
// EXTENDED METHODS (CryptoCompare-specific, not from traits)
// ═══════════════════════════════════════════════════════════════════════════════

impl CryptoCompareConnector {
    /// Get historical price at specific timestamp
    ///
    /// Returns price at end of day GMT for given timestamp.
    pub async fn get_historical_price(
        &self,
        symbol: Symbol,
        timestamp: i64,
    ) -> ExchangeResult<f64> {
        let (fsym, tsym) = format_symbol(&symbol);

        let mut params = HashMap::new();
        params.insert("fsym".to_string(), fsym);
        params.insert("tsyms".to_string(), tsym.clone());
        params.insert("ts".to_string(), (timestamp / 1000).to_string()); // Convert to seconds

        let response = self.get(CryptoCompareEndpoint::PriceHistorical, params).await?;
        CryptoCompareParser::parse_price(&response, &tsym)
    }

    /// Get top exchanges by volume for a trading pair
    pub async fn get_top_exchanges(
        &self,
        symbol: Symbol,
        limit: Option<u16>,
    ) -> ExchangeResult<serde_json::Value> {
        let (fsym, tsym) = format_symbol(&symbol);

        let mut params = HashMap::new();
        params.insert("fsym".to_string(), fsym);
        params.insert("tsym".to_string(), tsym);
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }

        self.get(CryptoCompareEndpoint::TopExchanges, params).await
    }

    /// Get rate limit status (requires API key)
    pub async fn get_rate_limit(&self) -> ExchangeResult<serde_json::Value> {
        if !self.auth.has_key() {
            return Err(ExchangeError::Auth(
                "API key required to check rate limits".to_string()
            ));
        }

        self.get(CryptoCompareEndpoint::RateLimit, HashMap::new()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exchange_identity() {
        let connector = CryptoCompareConnector::public();

        assert_eq!(connector.exchange_name(), "cryptocompare");
        assert!(!connector.is_testnet());
        assert_eq!(connector.supported_account_types(), vec![AccountType::Spot]);
    }
}
