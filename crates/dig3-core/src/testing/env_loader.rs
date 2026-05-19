//! # Env Loader — Load API credentials from `.env` file
//!
//! Reads `CARGO_MANIFEST_DIR/.env` and parses all exchange credentials
//! using the `{EXCHANGE}_API_KEY` / `{EXCHANGE}_API_SECRET` pattern.

use std::collections::HashMap;
use std::path::Path;

use crate::core::types::ExchangeId;
use crate::core::traits::Credentials;

// ═══════════════════════════════════════════════════════════════════════════════
// PUBLIC API
// ═══════════════════════════════════════════════════════════════════════════════

/// Load all available credentials from `.env` file at `CARGO_MANIFEST_DIR/.env`.
///
/// Parses `KEY=VALUE` lines (ignoring comments and blank lines).
/// For each known `ExchangeId`, checks whether `{NAME}_API_KEY` and
/// `{NAME}_API_SECRET` are present. If yes, builds a `Credentials` struct.
/// Also loads optional `{NAME}_PASSPHRASE`.
///
/// Returns a map from `ExchangeId` to `Credentials` for every exchange whose
/// keys are found.
pub fn load_credentials() -> HashMap<ExchangeId, Credentials> {
    let env_vars = load_env_file();
    let mut result = HashMap::new();

    for id in all_exchange_ids() {
        let prefix = env_prefix(id);

        let key_var = format!("{}_API_KEY", prefix);
        let secret_var = format!("{}_API_SECRET", prefix);

        let api_key = match env_vars.get(&key_var) {
            Some(k) if !k.is_empty() => k.clone(),
            _ => continue,
        };

        let api_secret = match env_vars.get(&secret_var) {
            Some(s) if !s.is_empty() => s.clone(),
            _ => continue,
        };

        let mut creds = Credentials::new(api_key, api_secret);

        let passphrase_var = format!("{}_PASSPHRASE", prefix);
        if let Some(p) = env_vars.get(&passphrase_var) {
            if !p.is_empty() {
                creds = creds.with_passphrase(p.clone());
            }
        }

        let testnet_var = format!("{}_TESTNET", prefix);
        if let Some(val) = env_vars.get(&testnet_var) {
            if val.eq_ignore_ascii_case("true") || val == "1" {
                creds = creds.with_testnet(true);
            }
        }

        result.insert(id, creds);
    }

    result
}

/// Check if a specific exchange has credentials available in `.env`.
pub fn has_credentials(id: ExchangeId) -> bool {
    load_credentials().contains_key(&id)
}

// ═══════════════════════════════════════════════════════════════════════════════
// INTERNAL HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Return the uppercase env-var prefix for a given `ExchangeId`.
///
/// Matches the convention used by `ConnectorConfigManager::load_from_env()`:
/// `exchange_id.as_str().to_uppercase()`, with one exception for
/// `CryptoCom` whose `as_str()` returns `"crypto_com"` → `"CRYPTO_COM"`.
fn env_prefix(id: ExchangeId) -> String {
    id.as_str().to_uppercase()
}

/// Parse `KEY=VALUE` pairs from a `.env` file at `CARGO_MANIFEST_DIR/.env`.
///
/// Lines starting with `#` or blank lines are ignored.
/// Values may optionally be quoted with `"` or `'`.
fn load_env_file() -> HashMap<String, String> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let env_path = Path::new(&manifest_dir).join(".env");

    let contents = match std::fs::read_to_string(&env_path) {
        Ok(c) => c,
        Err(_) => return HashMap::new(),
    };

    let mut map = HashMap::new();

    for line in contents.lines() {
        let line = line.trim();

        // Skip comments and blank lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Split on first '='
        let Some(eq_pos) = line.find('=') else {
            continue;
        };

        let key = line[..eq_pos].trim().to_string();
        let raw_value = line[eq_pos + 1..].trim();

        // Strip optional surrounding quotes
        let value = strip_quotes(raw_value).to_string();

        if !key.is_empty() {
            map.insert(key, value);
        }
    }

    map
}

/// Strip leading/trailing `"` or `'` from a value string.
fn strip_quotes(s: &str) -> &str {
    if (s.starts_with('"') && s.ends_with('"'))
        || (s.starts_with('\'') && s.ends_with('\''))
    {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

/// Return all known `ExchangeId` variants (mirrors the list in
/// `ConnectorConfigManager::load_from_env()`).
fn all_exchange_ids() -> Vec<ExchangeId> {
    vec![
        ExchangeId::Binance,
        ExchangeId::Bybit,
        ExchangeId::OKX,
        ExchangeId::KuCoin,
        ExchangeId::Kraken,
        ExchangeId::Coinbase,
        ExchangeId::GateIO,
        ExchangeId::Bitfinex,
        ExchangeId::Bitstamp,
        ExchangeId::Gemini,
        ExchangeId::MEXC,
        ExchangeId::HTX,
        ExchangeId::Bitget,
        ExchangeId::BingX,
        ExchangeId::CryptoCom,
        ExchangeId::Upbit,
        ExchangeId::Deribit,
        ExchangeId::HyperLiquid,
        ExchangeId::Lighter,
        ExchangeId::Dydx,
        ExchangeId::Polymarket,
        ExchangeId::Polygon,
        ExchangeId::Finnhub,
        ExchangeId::Tiingo,
        ExchangeId::Twelvedata,
        ExchangeId::Coinglass,
        ExchangeId::CryptoCompare,
        ExchangeId::WhaleAlert,
        ExchangeId::DefiLlama,
        ExchangeId::Bitquery,
        ExchangeId::Oanda,
        ExchangeId::AlphaVantage,
        ExchangeId::Dukascopy,
        ExchangeId::AngelOne,
        ExchangeId::Zerodha,
        ExchangeId::Fyers,
        ExchangeId::Dhan,
        ExchangeId::Upstox,
        ExchangeId::Alpaca,
        ExchangeId::JQuants,
        ExchangeId::Tinkoff,
        ExchangeId::Moex,
        ExchangeId::Krx,
        ExchangeId::Futu,
        ExchangeId::Fred,
        ExchangeId::Bls,
        ExchangeId::YahooFinance,
        ExchangeId::Ib,
    ]
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_quotes_double() {
        assert_eq!(strip_quotes("\"hello\""), "hello");
    }

    #[test]
    fn test_strip_quotes_single() {
        assert_eq!(strip_quotes("'hello'"), "hello");
    }

    #[test]
    fn test_strip_quotes_none() {
        assert_eq!(strip_quotes("hello"), "hello");
    }

    #[test]
    fn test_env_prefix_binance() {
        assert_eq!(env_prefix(ExchangeId::Binance), "BINANCE");
    }

    #[test]
    fn test_env_prefix_kucoin() {
        assert_eq!(env_prefix(ExchangeId::KuCoin), "KUCOIN");
    }

    #[test]
    fn test_env_prefix_crypto_com() {
        // as_str() = "crypto_com" → uppercase = "CRYPTO_COM"
        assert_eq!(env_prefix(ExchangeId::CryptoCom), "CRYPTO_COM");
    }

    #[test]
    fn test_env_prefix_okx() {
        assert_eq!(env_prefix(ExchangeId::OKX), "OKX");
    }

    #[test]
    fn test_load_credentials_empty_when_no_env_file() {
        // Without a .env file containing credentials, the result is empty
        // (or only has what environment provides — we don't control that here)
        let _creds = load_credentials();
        // Simply ensure it doesn't panic
    }
}
