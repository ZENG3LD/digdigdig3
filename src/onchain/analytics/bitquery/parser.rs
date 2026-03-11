//! # Bitquery Response Parsers
//!
//! Parse GraphQL responses to domain types.
//!
//! ## Response Structure
//!
//! All GraphQL responses have this format:
//! ```json
//! {
//!   "data": {
//!     "EVM": {
//!       "CubeName": [ ... ]
//!     }
//!   },
//!   "errors": [ ... ]
//! }
//! ```

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::{ExchangeError, ExchangeResult};

// ═══════════════════════════════════════════════════════════════════════════════
// RESPONSE WRAPPER
// ═══════════════════════════════════════════════════════════════════════════════

/// GraphQL response wrapper
#[derive(Debug, Clone, Deserialize)]
pub struct BitqueryResponse<T> {
    pub data: Option<T>,
    pub errors: Option<Vec<GraphQLError>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GraphQLError {
    pub message: String,
    pub extensions: Option<Value>,
}

impl<T> BitqueryResponse<T> {
    /// Check if response has errors
    pub fn has_errors(&self) -> bool {
        self.errors.is_some()
    }

    /// Get error messages
    pub fn error_messages(&self) -> Vec<String> {
        self.errors
            .as_ref()
            .map(|errs| errs.iter().map(|e| e.message.clone()).collect())
            .unwrap_or_default()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// DEX TRADES DATA STRUCTURES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DexTradesData {
    #[serde(rename = "EVM")]
    pub evm: DexTradesEvm,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DexTradesEvm {
    #[serde(rename = "DEXTrades")]
    pub dex_trades: Vec<DexTrade>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DexTrade {
    #[serde(rename = "Trade")]
    pub trade: TradeInfo,
    #[serde(rename = "Transaction")]
    pub transaction: TransactionRef,
    #[serde(rename = "Block")]
    pub block: BlockRef,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TradeInfo {
    #[serde(rename = "Buy")]
    pub buy: TradeSide,
    #[serde(rename = "Sell")]
    pub sell: TradeSide,
    #[serde(rename = "Dex")]
    pub dex: DexInfo,
    #[serde(rename = "Index")]
    pub index: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TradeSide {
    #[serde(rename = "Amount")]
    pub amount: Option<f64>,
    #[serde(rename = "Price")]
    pub price: Option<f64>,
    #[serde(rename = "PriceInUSD")]
    pub price_in_usd: Option<f64>,
    #[serde(rename = "Currency")]
    pub currency: Currency,
    #[serde(rename = "Buyer")]
    pub buyer: Option<String>,
    #[serde(rename = "Seller")]
    pub seller: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DexInfo {
    #[serde(rename = "ProtocolName")]
    pub protocol_name: Option<String>,
    #[serde(rename = "ProtocolFamily")]
    pub protocol_family: Option<String>,
    #[serde(rename = "SmartContract")]
    pub smart_contract: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Currency {
    #[serde(rename = "Symbol")]
    pub symbol: Option<String>,
    #[serde(rename = "Name")]
    pub name: Option<String>,
    #[serde(rename = "SmartContract")]
    pub smart_contract: Option<String>,
    #[serde(rename = "Decimals")]
    pub decimals: Option<i32>,
    #[serde(rename = "Fungible")]
    pub fungible: Option<bool>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// TOKEN TRANSFERS DATA STRUCTURES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransfersData {
    #[serde(rename = "EVM")]
    pub evm: TransfersEvm,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransfersEvm {
    #[serde(rename = "Transfers")]
    pub transfers: Vec<TokenTransfer>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenTransfer {
    #[serde(rename = "Transfer")]
    pub transfer: TransferInfo,
    #[serde(rename = "Transaction")]
    pub transaction: TransactionRef,
    #[serde(rename = "Block")]
    pub block: BlockRef,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransferInfo {
    #[serde(rename = "Amount")]
    pub amount: Option<f64>,
    #[serde(rename = "Sender")]
    pub sender: Option<String>,
    #[serde(rename = "Receiver")]
    pub receiver: Option<String>,
    #[serde(rename = "Currency")]
    pub currency: Currency,
    #[serde(rename = "Type")]
    pub transfer_type: Option<String>,
    #[serde(rename = "Id")]
    pub id: Option<String>, // NFT token ID
}

// ═══════════════════════════════════════════════════════════════════════════════
// BALANCE UPDATES DATA STRUCTURES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BalanceUpdatesData {
    #[serde(rename = "EVM")]
    pub evm: BalanceUpdatesEvm,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BalanceUpdatesEvm {
    #[serde(rename = "BalanceUpdates")]
    pub balance_updates: Vec<BalanceUpdate>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BalanceUpdate {
    #[serde(rename = "BalanceUpdate")]
    pub balance_update: BalanceUpdateInfo,
    #[serde(rename = "Transaction")]
    pub transaction: TransactionRef,
    #[serde(rename = "Block")]
    pub block: BlockRef,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BalanceUpdateInfo {
    #[serde(rename = "Address")]
    pub address: Option<String>,
    #[serde(rename = "Amount")]
    pub amount: Option<f64>,
    #[serde(rename = "Type")]
    pub update_type: Option<String>,
    #[serde(rename = "Currency")]
    pub currency: Currency,
}

// ═══════════════════════════════════════════════════════════════════════════════
// BLOCKS DATA STRUCTURES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BlocksData {
    #[serde(rename = "EVM")]
    pub evm: BlocksEvm,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BlocksEvm {
    #[serde(rename = "Blocks")]
    pub blocks: Vec<BlockData>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BlockData {
    #[serde(rename = "Block")]
    pub block: BlockInfo,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BlockInfo {
    #[serde(rename = "Number")]
    pub number: Option<i64>,
    #[serde(rename = "Time")]
    pub time: Option<String>,
    #[serde(rename = "Hash")]
    pub hash: Option<String>,
    #[serde(rename = "GasLimit")]
    pub gas_limit: Option<i64>,
    #[serde(rename = "GasUsed")]
    pub gas_used: Option<i64>,
    #[serde(rename = "BaseFee")]
    pub base_fee: Option<i64>,
    #[serde(rename = "Coinbase")]
    pub coinbase: Option<String>,
    #[serde(rename = "Difficulty")]
    pub difficulty: Option<i64>,
    #[serde(rename = "Size")]
    pub size: Option<i64>,
    #[serde(rename = "TxCount")]
    pub tx_count: Option<i64>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRANSACTIONS DATA STRUCTURES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransactionsData {
    #[serde(rename = "EVM")]
    pub evm: TransactionsEvm,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransactionsEvm {
    #[serde(rename = "Transactions")]
    pub transactions: Vec<TransactionData>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransactionData {
    #[serde(rename = "Transaction")]
    pub transaction: TransactionInfo,
    #[serde(rename = "Receipt")]
    pub receipt: Option<TransactionReceipt>,
    #[serde(rename = "Block")]
    pub block: BlockRef,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransactionInfo {
    #[serde(rename = "Hash")]
    pub hash: Option<String>,
    #[serde(rename = "From")]
    pub from: Option<String>,
    #[serde(rename = "To")]
    pub to: Option<String>,
    #[serde(rename = "Value")]
    pub value: Option<f64>,
    #[serde(rename = "Gas")]
    pub gas: Option<i64>,
    #[serde(rename = "GasPrice")]
    pub gas_price: Option<i64>,
    #[serde(rename = "GasUsed")]
    pub gas_used: Option<i64>,
    #[serde(rename = "Nonce")]
    pub nonce: Option<i64>,
    #[serde(rename = "Index")]
    pub index: Option<i64>,
    #[serde(rename = "Type")]
    pub tx_type: Option<i64>,
    #[serde(rename = "Cost")]
    pub cost: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransactionReceipt {
    #[serde(rename = "Status")]
    pub status: Option<i64>,
    #[serde(rename = "GasUsed")]
    pub gas_used: Option<i64>,
    #[serde(rename = "EffectiveGasPrice")]
    pub effective_gas_price: Option<i64>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// SMART CONTRACT EVENTS DATA STRUCTURES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EventsData {
    #[serde(rename = "EVM")]
    pub evm: EventsEvm,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EventsEvm {
    #[serde(rename = "Events")]
    pub events: Vec<SmartContractEvent>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SmartContractEvent {
    #[serde(rename = "Log")]
    pub log: LogInfo,
    #[serde(rename = "Arguments")]
    pub arguments: Option<Vec<EventArgument>>,
    #[serde(rename = "Transaction")]
    pub transaction: TransactionRef,
    #[serde(rename = "Block")]
    pub block: BlockRef,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LogInfo {
    #[serde(rename = "Signature")]
    pub signature: Option<String>,
    #[serde(rename = "SignatureName")]
    pub signature_name: Option<String>,
    #[serde(rename = "SmartContract")]
    pub smart_contract: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EventArgument {
    #[serde(rename = "Name")]
    pub name: Option<String>,
    #[serde(rename = "Type")]
    pub arg_type: Option<String>,
    #[serde(rename = "Value")]
    pub value: Option<Value>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// COMMON REFERENCE TYPES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransactionRef {
    #[serde(rename = "Hash")]
    pub hash: Option<String>,
    #[serde(rename = "From")]
    pub from: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BlockRef {
    #[serde(rename = "Time")]
    pub time: Option<String>,
    #[serde(rename = "Number")]
    pub number: Option<i64>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// PARSER
// ═══════════════════════════════════════════════════════════════════════════════

pub struct BitqueryParser;

impl BitqueryParser {
    /// Parse GraphQL response and check for errors
    pub fn parse_response<T>(response: &Value) -> ExchangeResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        // First, try to deserialize into BitqueryResponse wrapper
        let wrapped: BitqueryResponse<T> = serde_json::from_value(response.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse response: {}", e)))?;

        // Check for GraphQL errors
        if wrapped.has_errors() {
            let error_msgs = wrapped.error_messages().join("; ");
            return Err(ExchangeError::Api {
                code: 0,
                message: format!("GraphQL errors: {}", error_msgs)
            });
        }

        // Extract data
        wrapped.data.ok_or_else(|| {
            ExchangeError::Parse("Response has no data field".to_string())
        })
    }

    /// Parse DEX trades response
    pub fn parse_dex_trades(response: &Value) -> ExchangeResult<Vec<DexTrade>> {
        let data: DexTradesData = Self::parse_response(response)?;
        Ok(data.evm.dex_trades)
    }

    /// Parse token transfers response
    pub fn parse_transfers(response: &Value) -> ExchangeResult<Vec<TokenTransfer>> {
        let data: TransfersData = Self::parse_response(response)?;
        Ok(data.evm.transfers)
    }

    /// Parse balance updates response
    pub fn parse_balance_updates(response: &Value) -> ExchangeResult<Vec<BalanceUpdate>> {
        let data: BalanceUpdatesData = Self::parse_response(response)?;
        Ok(data.evm.balance_updates)
    }

    /// Parse blocks response
    pub fn parse_blocks(response: &Value) -> ExchangeResult<Vec<BlockData>> {
        let data: BlocksData = Self::parse_response(response)?;
        Ok(data.evm.blocks)
    }

    /// Parse transactions response
    pub fn parse_transactions(response: &Value) -> ExchangeResult<Vec<TransactionData>> {
        let data: TransactionsData = Self::parse_response(response)?;
        Ok(data.evm.transactions)
    }

    /// Parse smart contract events response
    pub fn parse_events(response: &Value) -> ExchangeResult<Vec<SmartContractEvent>> {
        let data: EventsData = Self::parse_response(response)?;
        Ok(data.evm.events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_error_response() {
        let response = json!({
            "errors": [
                {
                    "message": "Unauthorized: Invalid or missing access token"
                }
            ]
        });

        let result: Result<DexTradesData, _> = BitqueryParser::parse_response(&response);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unauthorized"));
    }

    #[test]
    fn test_parse_empty_data() {
        let response = json!({
            "data": {
                "EVM": {
                    "DEXTrades": []
                }
            }
        });

        let trades = BitqueryParser::parse_dex_trades(&response).unwrap();
        assert_eq!(trades.len(), 0);
    }
}
