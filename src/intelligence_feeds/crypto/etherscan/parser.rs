//! Etherscan response parsers
//!
//! Parse JSON responses to domain types based on Etherscan API response formats.
//!
//! All Etherscan responses follow the format:
//! ```json
//! {
//!   "status": "1",
//!   "message": "OK",
//!   "result": [...]
//! }
//! ```
//! where status="1" means success, status="0" means error.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct EtherscanParser;

impl EtherscanParser {
    // ═══════════════════════════════════════════════════════════════════════
    // ETHERSCAN-SPECIFIC PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse ETH balance response
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "status": "1",
    ///   "message": "OK",
    ///   "result": "40807168566070000000000"
    /// }
    /// ```
    pub fn parse_balance(response: &EtherscanResponse<String>) -> ExchangeResult<String> {
        Ok(response.result.clone())
    }

    /// Parse multi-balance response
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "status": "1",
    ///   "message": "OK",
    ///   "result": [
    ///     {"account": "0x...", "balance": "40807168566070000000000"},
    ///     {"account": "0x...", "balance": "332567136222827062478"}
    ///   ]
    /// }
    /// ```
    pub fn parse_multi_balance(
        response: &EtherscanResponse<Vec<EthBalance>>,
    ) -> ExchangeResult<Vec<EthBalance>> {
        Ok(response.result.clone())
    }

    /// Parse transaction list
    pub fn parse_transactions(
        response: &EtherscanResponse<Vec<EthTransaction>>,
    ) -> ExchangeResult<Vec<EthTransaction>> {
        Ok(response.result.clone())
    }

    /// Parse token transfers
    pub fn parse_token_transfers(
        response: &EtherscanResponse<Vec<TokenTransfer>>,
    ) -> ExchangeResult<Vec<TokenTransfer>> {
        Ok(response.result.clone())
    }

    /// Parse ETH price
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "status": "1",
    ///   "message": "OK",
    ///   "result": {
    ///     "ethbtc": "0.03161",
    ///     "ethbtc_timestamp": "1577086564",
    ///     "ethusd": "127.50",
    ///     "ethusd_timestamp": "1577086561"
    ///   }
    /// }
    /// ```
    pub fn parse_eth_price(response: &EtherscanResponse<EthPrice>) -> ExchangeResult<EthPrice> {
        Ok(response.result.clone())
    }

    /// Parse gas oracle
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "status": "1",
    ///   "message": "OK",
    ///   "result": {
    ///     "LastBlock": "13053741",
    ///     "SafeGasPrice": "20",
    ///     "ProposeGasPrice": "22",
    ///     "FastGasPrice": "24",
    ///     "suggestBaseFee": "19.230609716",
    ///     "gasUsedRatio": "0.370119078777807,0.8954731,0.550911766666667"
    ///   }
    /// }
    /// ```
    pub fn parse_gas_oracle(
        response: &EtherscanResponse<GasOracle>,
    ) -> ExchangeResult<GasOracle> {
        Ok(response.result.clone())
    }

    /// Parse block reward
    pub fn parse_block_reward(
        response: &EtherscanResponse<BlockReward>,
    ) -> ExchangeResult<BlockReward> {
        Ok(response.result.clone())
    }

    /// Parse simple string result (ETH supply, chain size, latest block, token supply)
    pub fn parse_string_result(response: &EtherscanResponse<String>) -> ExchangeResult<String> {
        Ok(response.result.clone())
    }

    /// Parse contract ABI (returns raw JSON string)
    pub fn parse_contract_abi(response: &EtherscanResponse<String>) -> ExchangeResult<String> {
        Ok(response.result.clone())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ERROR HANDLING
    // ═══════════════════════════════════════════════════════════════════════

    /// Check Etherscan response for errors
    ///
    /// Status "1" = success
    /// Status "0" = error (message contains error description)
    pub fn check_response_generic(response: &Value) -> ExchangeResult<()> {
        let status = response
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("0");

        if status == "1" {
            Ok(())
        } else {
            let message = response
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");

            let result = response
                .get("result")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let error_msg = if result.is_empty() {
                message.to_string()
            } else {
                format!("{}: {}", message, result)
            };

            Err(ExchangeError::Api {
                code: 0,
                message: error_msg,
            })
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    fn _require_str<'a>(obj: &'a Value, field: &str) -> ExchangeResult<&'a str> {
        obj.get(field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn _get_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field).and_then(|v| v.as_str())
    }

    fn _require_i64(obj: &Value, field: &str) -> ExchangeResult<i64> {
        obj.get(field)
            .and_then(|v| v.as_i64())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn _get_i64(obj: &Value, field: &str) -> Option<i64> {
        obj.get(field).and_then(|v| v.as_i64())
    }

    fn _get_f64(obj: &Value, field: &str) -> Option<f64> {
        obj.get(field).and_then(|v| v.as_f64())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// ETHERSCAN-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Generic Etherscan API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EtherscanResponse<T> {
    pub status: String,
    pub message: String,
    pub result: T,
}

/// ETH balance for a single address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthBalance {
    pub account: String,
    pub balance: String, // Wei value as string
}

/// Ethereum transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EthTransaction {
    pub block_number: String,
    #[serde(rename = "timeStamp")]
    pub time_stamp: String,
    pub hash: String,
    pub from: String,
    pub to: String,
    pub value: String,
    pub gas: String,
    pub gas_price: String,
    pub gas_used: String,
    pub is_error: String,
    pub input: String,
    #[serde(default)]
    pub nonce: String,
    #[serde(default)]
    pub transaction_index: String,
    #[serde(default)]
    pub confirmations: String,
}

/// ERC20 token transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenTransfer {
    pub block_number: String,
    #[serde(rename = "timeStamp")]
    pub time_stamp: String,
    pub hash: String,
    pub from: String,
    pub to: String,
    pub value: String,
    pub token_name: String,
    pub token_symbol: String,
    pub token_decimal: String,
    pub contract_address: String,
    #[serde(default)]
    pub nonce: String,
    #[serde(default)]
    pub gas: String,
    #[serde(default)]
    pub gas_price: String,
    #[serde(default)]
    pub gas_used: String,
    #[serde(default)]
    pub transaction_index: String,
    #[serde(default)]
    pub confirmations: String,
}

/// ETH price (USD + BTC)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthPrice {
    pub ethbtc: String,
    pub ethbtc_timestamp: String,
    pub ethusd: String,
    pub ethusd_timestamp: String,
}

/// Gas price oracle
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GasOracle {
    pub last_block: String,
    pub safe_gas_price: String,
    pub propose_gas_price: String,
    pub fast_gas_price: String,
    #[serde(default)]
    pub suggest_base_fee: String,
    #[serde(default)]
    pub gas_used_ratio: String,
}

/// Block mining reward
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockReward {
    pub block_number: String,
    #[serde(rename = "timeStamp")]
    pub time_stamp: String,
    pub block_miner: String,
    pub block_reward: String,
    #[serde(default)]
    pub uncles: Vec<UncleReward>,
}

/// Uncle block reward
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UncleReward {
    pub miner: String,
    pub uncle_position: String,
    pub block_reward: String,
}
