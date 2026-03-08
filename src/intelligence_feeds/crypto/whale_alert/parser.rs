//! Whale Alert response parsers
//!
//! Parse JSON responses to domain types based on Whale Alert API formats.
//!
//! Note: Whale Alert is a blockchain transaction tracker, not a traditional exchange.
//! Most standard market data parsers will return UnsupportedOperation errors.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct WhaleAlertParser;

// ═══════════════════════════════════════════════════════════════════════════
// WHALE ALERT SPECIFIC DATA TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Whale Alert transaction (Enterprise API v2 format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhaleTransaction {
    pub height: u64,
    pub index_in_block: u64,
    pub timestamp: i64,
    pub hash: String,
    pub fee: String,
    pub fee_symbol: String,
    pub fee_symbol_price: String,
    pub sub_transactions: Vec<SubTransaction>,
}

/// Sub-transaction within a blockchain transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubTransaction {
    pub symbol: String,
    pub unit_price_usd: String,
    pub transaction_type: String,
    pub inputs: Vec<Address>,
    pub outputs: Vec<Address>,
}

/// Address information with attribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub amount: String,
    pub address: String,
    pub balance: String,
    pub locked: String,
    pub is_frozen: bool,
    pub owner: String,
    pub owner_type: String,
    pub address_type: String,
}

/// Owner attribution data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerAttribution {
    pub owner: String,
    pub owner_type: String,
    pub address_type: String,
    pub confidence: f64,
}

/// Block data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhaleBlock {
    pub blockchain: String,
    pub height: u64,
    pub timestamp: i64,
    pub transaction_count: u64,
    pub transactions: Vec<WhaleTransaction>,
}

/// Blockchain status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainStatus {
    pub blockchain: String,
    pub newest_block: u64,
    pub oldest_block: u64,
    pub status: String,
}

/// Status response (supported blockchains)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResponse {
    pub blockchains: std::collections::HashMap<String, Vec<String>>,
    pub status: Option<std::collections::HashMap<String, String>>,
}

/// Developer API v1 transaction format (deprecated)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhaleTransactionV1 {
    pub blockchain: String,
    pub symbol: String,
    pub id: String,
    pub transaction_type: String,
    pub hash: String,
    pub from: AddressV1,
    pub to: AddressV1,
    pub timestamp: i64,
    pub amount: f64,
    pub amount_usd: f64,
    pub transaction_count: u64,
}

/// Address format for v1 API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressV1 {
    pub address: String,
    pub owner: String,
    pub owner_type: String,
}

impl WhaleAlertParser {
    // ═══════════════════════════════════════════════════════════════════════
    // ENTERPRISE API V2 PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse status response
    pub fn parse_status(response: &Value) -> ExchangeResult<StatusResponse> {
        serde_json::from_value(response.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse status: {}", e)))
    }

    /// Parse blockchain status
    pub fn parse_blockchain_status(response: &Value) -> ExchangeResult<BlockchainStatus> {
        serde_json::from_value(response.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse blockchain status: {}", e)))
    }

    /// Parse single transaction
    pub fn parse_transaction(response: &Value) -> ExchangeResult<WhaleTransaction> {
        serde_json::from_value(response.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse transaction: {}", e)))
    }

    /// Parse multiple transactions
    pub fn parse_transactions(response: &Value) -> ExchangeResult<Vec<WhaleTransaction>> {
        if let Some(array) = response.as_array() {
            array.iter()
                .map(Self::parse_transaction)
                .collect()
        } else {
            Err(ExchangeError::Parse("Expected array of transactions".to_string()))
        }
    }

    /// Parse block data
    pub fn parse_block(response: &Value) -> ExchangeResult<WhaleBlock> {
        serde_json::from_value(response.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse block: {}", e)))
    }

    /// Parse address transactions
    pub fn parse_address_transactions(response: &Value) -> ExchangeResult<Vec<WhaleTransaction>> {
        Self::parse_transactions(response)
    }

    /// Parse owner attributions
    pub fn parse_owner_attributions(response: &Value) -> ExchangeResult<Vec<OwnerAttribution>> {
        let attributions = response
            .get("attributions")
            .ok_or_else(|| ExchangeError::Parse("Missing 'attributions' field".to_string()))?;

        if let Some(array) = attributions.as_array() {
            array.iter()
                .map(|attr| {
                    serde_json::from_value(attr.clone())
                        .map_err(|e| ExchangeError::Parse(format!("Failed to parse attribution: {}", e)))
                })
                .collect()
        } else {
            Err(ExchangeError::Parse("Expected array of attributions".to_string()))
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // DEVELOPER API V1 PARSERS (Deprecated)
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse v1 transaction
    pub fn parse_transaction_v1(response: &Value) -> ExchangeResult<WhaleTransactionV1> {
        serde_json::from_value(response.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse v1 transaction: {}", e)))
    }

    /// Parse v1 transactions list
    pub fn parse_transactions_v1(response: &Value) -> ExchangeResult<Vec<WhaleTransactionV1>> {
        let transactions = response
            .get("transactions")
            .ok_or_else(|| ExchangeError::Parse("Missing 'transactions' field".to_string()))?;

        if let Some(array) = transactions.as_array() {
            array.iter()
                .map(Self::parse_transaction_v1)
                .collect()
        } else {
            Err(ExchangeError::Parse("Expected array of transactions".to_string()))
        }
    }

    /// Get pagination cursor from v1 response
    pub fn parse_cursor(response: &Value) -> Option<String> {
        response
            .get("cursor")
            .and_then(|c| c.as_str())
            .map(|s| s.to_string())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // STANDARD MARKET DATA (NOT APPLICABLE - Returns UnsupportedOperation)
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse price response (NOT SUPPORTED)
    ///
    /// Whale Alert does not provide price data - it tracks transactions
    pub fn parse_price(_response: &Value) -> ExchangeResult<f64> {
        Err(ExchangeError::UnsupportedOperation(
            "Whale Alert does not provide price data - use a price oracle instead".to_string()
        ))
    }

    /// Parse ticker response (NOT SUPPORTED)
    pub fn parse_ticker(_response: &Value, _symbol: &str) -> ExchangeResult<crate::core::types::Ticker> {
        Err(ExchangeError::UnsupportedOperation(
            "Whale Alert does not provide ticker data - use a crypto exchange instead".to_string()
        ))
    }

    /// Parse klines/candles response (NOT SUPPORTED)
    pub fn parse_klines(_response: &Value) -> ExchangeResult<Vec<crate::core::types::Kline>> {
        Err(ExchangeError::UnsupportedOperation(
            "Whale Alert does not provide OHLCV data - use a crypto exchange instead".to_string()
        ))
    }

    /// Parse orderbook response (NOT SUPPORTED)
    pub fn parse_orderbook(_response: &Value) -> ExchangeResult<crate::core::types::OrderBook> {
        Err(ExchangeError::UnsupportedOperation(
            "Whale Alert does not provide orderbook data - use a crypto exchange instead".to_string()
        ))
    }

    /// Parse symbols list (returns supported blockchains instead)
    pub fn parse_symbols(response: &Value) -> ExchangeResult<Vec<String>> {
        // Return list of supported blockchains as "symbols"
        let status = Self::parse_status(response)?;
        Ok(status.blockchains.keys().cloned().collect())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    fn _require_f64(obj: &Value, field: &str) -> ExchangeResult<f64> {
        obj.get(field)
            .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn _get_f64(obj: &Value, field: &str) -> Option<f64> {
        obj.get(field)
            .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
    }

    fn _require_i64(obj: &Value, field: &str) -> ExchangeResult<i64> {
        obj.get(field)
            .and_then(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn _get_i64(obj: &Value, field: &str) -> Option<i64> {
        obj.get(field)
            .and_then(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
    }

    fn _get_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field).and_then(|v| v.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_status() {
        let response = json!({
            "blockchains": {
                "bitcoin": ["BTC", "USDT"],
                "ethereum": ["ETH", "USDC", "USDT"]
            },
            "status": {
                "bitcoin": "connected",
                "ethereum": "connected"
            }
        });

        let status = WhaleAlertParser::parse_status(&response).unwrap();
        assert!(status.blockchains.contains_key("bitcoin"));
        assert!(status.blockchains.contains_key("ethereum"));
    }

    #[test]
    fn test_parse_blockchain_status() {
        let response = json!({
            "blockchain": "ethereum",
            "newest_block": 18500000,
            "oldest_block": 18000000,
            "status": "connected"
        });

        let status = WhaleAlertParser::parse_blockchain_status(&response).unwrap();
        assert_eq!(status.blockchain, "ethereum");
        assert_eq!(status.newest_block, 18500000);
        assert_eq!(status.oldest_block, 18000000);
    }

    #[test]
    fn test_unsupported_operations() {
        let response = json!({});

        assert!(WhaleAlertParser::parse_price(&response).is_err());
        assert!(WhaleAlertParser::parse_ticker(&response, "BTCUSDT").is_err());
        assert!(WhaleAlertParser::parse_klines(&response).is_err());
        assert!(WhaleAlertParser::parse_orderbook(&response).is_err());
    }
}
