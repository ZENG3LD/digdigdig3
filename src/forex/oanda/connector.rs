//! # OANDA v20 Connector
//!
//! Implementation of core traits for OANDA forex broker.
//!
//! ## Traits Implemented
//! - `ExchangeIdentity` - Exchange identification
//! - `MarketData` - Market data operations
//! - `Trading` - Trading operations
//! - `Account` - Account information
//! - `Positions` - Position management
//!
//! ## Important Notes
//! - OANDA is a forex BROKER, not just a data provider
//! - Uses Bearer token authentication (not HMAC)
//! - HTTP streaming for real-time data (not WebSocket)
//! - Symbol format: "EUR_USD" (underscore separator)
//! - All numeric values returned as strings

use std::collections::HashMap;
use async_trait::async_trait;
use serde_json::{json, Value};

use crate::core::{
    HttpClient, Credentials,
    ExchangeId, ExchangeType, AccountType, Symbol,
    ExchangeError, ExchangeResult,
    Price, Quantity, Kline, Ticker, OrderBook,
    Order, OrderSide, OrderType, Balance, AccountInfo,
    Position, FundingRate, SymbolInfo,
};
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
};

use super::endpoints::{OandaUrls, OandaEndpoint, format_symbol, map_granularity};
use super::auth::OandaAuth;
use super::parser::OandaParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// OANDA v20 connector
pub struct OandaConnector {
    /// HTTP client
    http: HttpClient,
    /// Authentication
    auth: OandaAuth,
    /// URLs (practice/live)
    urls: OandaUrls,
    /// Practice mode flag
    practice: bool,
    /// Account ID (cached)
    account_id: Option<String>,
}

impl OandaConnector {
    /// Create new OANDA connector
    ///
    /// # Arguments
    /// - `credentials` - Bearer token (stored in api_key field)
    /// - `practice` - If true, use practice (demo) account
    pub async fn new(credentials: Credentials, practice: bool) -> ExchangeResult<Self> {
        let urls = if practice {
            OandaUrls::PRACTICE
        } else {
            OandaUrls::LIVE
        };

        let http = HttpClient::new(30_000)?; // 30 sec timeout
        let auth = OandaAuth::new(&credentials)?;

        Ok(Self {
            http,
            auth,
            urls,
            practice,
            account_id: None,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ACCOUNT ID MANAGEMENT
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get account ID (fetch and cache if needed)
    async fn get_account_id(&mut self) -> ExchangeResult<String> {
        if let Some(ref id) = self.account_id {
            return Ok(id.clone());
        }

        // Fetch account ID
        let endpoint = OandaEndpoint::ListAccounts;
        let response = self.get(endpoint, HashMap::new()).await?;
        let account_id = OandaParser::parse_account_id(&response)?;

        self.account_id = Some(account_id.clone());
        Ok(account_id)
    }

    /// Get account ID reference (must be cached)
    fn require_account_id(&self) -> ExchangeResult<&str> {
        self.account_id.as_deref()
            .ok_or_else(|| ExchangeError::Auth("Account ID not initialized".to_string()))
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// GET request
    async fn get(
        &self,
        endpoint: OandaEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        let base_url = self.urls.rest_url;
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
        let headers = self.auth.sign_request();

        let response = self.http.get_with_headers(&url, &HashMap::new(), &headers).await?;
        self.check_response(&response)?;
        Ok(response)
    }

    /// POST request
    async fn post(
        &self,
        endpoint: OandaEndpoint,
        body: Value,
    ) -> ExchangeResult<Value> {
        let base_url = self.urls.rest_url;
        let path = endpoint.path();
        let url = format!("{}{}", base_url, path);

        let headers = self.auth.sign_request();
        let response = self.http.post(&url, &body, &headers).await?;
        self.check_response(&response)?;
        Ok(response)
    }

    /// PUT request
    async fn put(
        &self,
        endpoint: OandaEndpoint,
        body: Value,
    ) -> ExchangeResult<Value> {
        let base_url = self.urls.rest_url;
        let path = endpoint.path();
        let url = format!("{}{}", base_url, path);

        let headers = self.auth.sign_request();
        let response = self.http.put(&url, &body, &headers).await?;
        self.check_response(&response)?;
        Ok(response)
    }

    /// Check response for errors
    fn check_response(&self, response: &Value) -> ExchangeResult<()> {
        // OANDA returns error in "errorMessage" field
        if let Some(error_msg) = response.get("errorMessage") {
            let msg = error_msg.as_str().unwrap_or("Unknown error");

            // Check for specific error codes
            if let Some(code) = response.get("errorCode").and_then(|c| c.as_str()) {
                return Err(ExchangeError::Api {
                    code: -1,
                    message: format!("{}: {}", code, msg),
                });
            }

            return Err(ExchangeError::Api {
                code: -1,
                message: msg.to_string(),
            });
        }

        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS (OANDA-specific)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get tradeable instruments for account
    pub async fn get_instruments(&mut self) -> ExchangeResult<Vec<String>> {
        let account_id = self.get_account_id().await?;
        let endpoint = OandaEndpoint::GetInstruments(account_id);

        let response = self.get(endpoint, HashMap::new()).await?;

        let instruments = response.get("instruments")
            .and_then(|i| i.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|inst| inst.get("name").and_then(|n| n.as_str()))
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default();

        Ok(instruments)
    }

    /// Close all positions for an instrument
    pub async fn close_all_positions(&mut self, symbol: Symbol) -> ExchangeResult<()> {
        let account_id = self.get_account_id().await?;
        let instrument = format_symbol(&symbol.base, &symbol.quote);

        let endpoint = OandaEndpoint::ClosePosition {
            account_id,
            instrument,
        };

        let body = json!({
            "longUnits": "ALL",
            "shortUnits": "ALL"
        });

        self.put(endpoint, body).await?;
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for OandaConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Oanda
    }

    fn is_testnet(&self) -> bool {
        self.practice
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        // OANDA is forex broker - uses Spot as default account type
        vec![AccountType::Spot]
    }

    fn exchange_type(&self) -> ExchangeType {
        ExchangeType::Cex
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MARKET DATA
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for OandaConnector {
    async fn get_price(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Price> {
        let account_id = self.require_account_id()?;
        let instrument = format_symbol(&symbol.base, &symbol.quote);

        let endpoint = OandaEndpoint::GetPricing(account_id.to_string());

        let mut params = HashMap::new();
        params.insert("instruments".to_string(), instrument);

        let response = self.get(endpoint, params).await?;
        OandaParser::parse_price(&response)
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        _depth: Option<u16>,
        _account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let account_id = self.require_account_id()?;
        let instrument = format_symbol(&symbol.base, &symbol.quote);

        let endpoint = OandaEndpoint::GetPricing(account_id.to_string());

        let mut params = HashMap::new();
        params.insert("instruments".to_string(), instrument);

        let response = self.get(endpoint, params).await?;
        OandaParser::parse_orderbook(&response)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        _account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let instrument = format_symbol(&symbol.base, &symbol.quote);
        let endpoint = OandaEndpoint::GetCandles(instrument.clone());

        let mut params = HashMap::new();
        params.insert("granularity".to_string(), map_granularity(interval).to_string());
        params.insert("count".to_string(), limit.unwrap_or(500).to_string());
        params.insert("price".to_string(), "M".to_string()); // Mid prices

        let response = self.get(endpoint, params).await?;
        OandaParser::parse_klines(&response)
    }

    async fn get_ticker(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let account_id = self.require_account_id()?;
        let instrument = format_symbol(&symbol.base, &symbol.quote);

        let endpoint = OandaEndpoint::GetPricing(account_id.to_string());

        let mut params = HashMap::new();
        params.insert("instruments".to_string(), instrument);

        let response = self.get(endpoint, params).await?;
        OandaParser::parse_ticker(&response)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        // List accounts is a simple ping test
        let endpoint = OandaEndpoint::ListAccounts;
        let response = self.get(endpoint, HashMap::new()).await?;
        self.check_response(&response)
    }

    async fn get_exchange_info(&self, _account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        // Step 1: fetch account ID (can't use cached value since we have &self, not &mut self)
        let accounts_response = self.get(OandaEndpoint::ListAccounts, HashMap::new()).await?;
        let account_id = OandaParser::parse_account_id(&accounts_response)?;

        // Step 2: fetch instruments for that account
        let endpoint = OandaEndpoint::GetInstruments(account_id);
        let response = self.get(endpoint, HashMap::new()).await?;

        let instruments = response
            .get("instruments")
            .and_then(|i| i.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'instruments' array".to_string()))?;

        let infos = instruments
            .iter()
            .filter_map(|inst| {
                let name = inst.get("name").and_then(|n| n.as_str())?; // e.g. "EUR_USD"
                let _inst_type = inst.get("type").and_then(|t| t.as_str()).unwrap_or("CURRENCY");
                let _display = inst.get("displayName").and_then(|d| d.as_str()).unwrap_or(name);

                // OANDA format: "EUR_USD" → base="EUR", quote="USD"
                let parts: Vec<&str> = name.splitn(2, '_').collect();
                let (base, quote) = if parts.len() == 2 {
                    (parts[0].to_string(), parts[1].to_string())
                } else {
                    (name.to_string(), "USD".to_string())
                };

                Some(SymbolInfo {
                    symbol: name.to_string(),
                    base_asset: base,
                    quote_asset: quote,
                    status: "TRADING".to_string(),
                    price_precision: 5, // Forex typically 5 decimal places
                    quantity_precision: 0,
                    min_quantity: Some(1.0),
                    max_quantity: None,
                    step_size: Some(1.0),
                    min_notional: None,
                })
            })
            .collect();

        Ok(infos)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING
// ═══════════════════════════════════════════════════════════════════════════════



// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════



// ═══════════════════════════════════════════════════════════════════════════════
// POSITIONS
// ═══════════════════════════════════════════════════════════════════════════════


