//! # HTX Authentication
//!
//! Implementation of request signing for HTX API.
//!
//! ## Signature Algorithm
//!
//! 1. Create pre-sign string: `METHOD\nHOST\nPATH\nQUERY_STRING`
//! 2. HMAC-SHA256 with API secret
//! 3. Base64 encode the result
//! 4. URL encode the Base64 string
//!
//! ## Required Parameters (in query string)
//!
//! - `AccessKeyId` - API key
//! - `SignatureMethod` - Always "HmacSHA256"
//! - `SignatureVersion` - Always "2"
//! - `Timestamp` - UTC timestamp in format `YYYY-MM-DDThh:mm:ss`
//! - `Signature` - Computed signature
//!
//! ## Key Differences from Other Exchanges
//!
//! - **Query params based**: Signature and auth params go in URL (not headers)
//! - **Timestamp format**: `YYYY-MM-DDThh:mm:ss` UTC (not milliseconds)
//! - **Wide window**: ±5 minutes (not ±5 seconds)
//! - **POST requests**: Auth params in query string, business params in JSON body
//! - **Pre-sign format**: `METHOD\nHOST\nPATH\nQUERY` (newline separated)
//! - **Encoding**: HMAC-SHA256 → Base64 → URL encode

use std::collections::HashMap;
use chrono::Utc;
use url::form_urlencoded;

use crate::core::{
    hmac_sha256, encode_base64,
    Credentials,
};

/// HTX authentication handler
#[derive(Clone)]
pub struct HtxAuth {
    api_key: String,
    api_secret: String,
    /// Time offset: server_time - local_time (milliseconds)
    /// HTX has ±5 minute tolerance, so sync is optional but recommended
    time_offset_ms: i64,
}

impl HtxAuth {
    /// Create new auth handler
    pub fn new(credentials: &Credentials) -> Self {
        Self {
            api_key: credentials.api_key.clone(),
            api_secret: credentials.api_secret.clone(),
            time_offset_ms: 0,
        }
    }

    /// Sync time with server
    /// Call this with server timestamp (milliseconds) from /v1/common/timestamp response
    pub fn sync_time(&mut self, server_time_ms: i64) {
        let local_time = Utc::now().timestamp_millis();
        self.time_offset_ms = server_time_ms - local_time;
    }

    /// Get adjusted timestamp in UTC format: YYYY-MM-DDThh:mm:ss
    fn get_timestamp(&self) -> String {
        let local = Utc::now().timestamp_millis();
        let adjusted = local + self.time_offset_ms;
        let dt = chrono::DateTime::from_timestamp(adjusted / 1000, ((adjusted % 1000) * 1_000_000) as u32)
            .unwrap_or_else(Utc::now);
        dt.format("%Y-%m-%dT%H:%M:%S").to_string()
    }

    /// Build query string with auth parameters
    ///
    /// # Arguments
    ///
    /// * `method` - HTTP method ("GET", "POST", etc.)
    /// * `host` - API host ("api.huobi.pro")
    /// * `path` - Endpoint path ("/v1/order/orders/place")
    /// * `params` - Additional query parameters (business params for GET, empty for POST)
    ///
    /// # Returns
    ///
    /// Sorted query string with signature appended
    ///
    /// # Example
    ///
    /// ```ignore
    /// let query = auth.build_signed_query("GET", "api.huobi.pro", "/v1/account/accounts", &params);
    /// // Returns: AccessKeyId=xxx&SignatureMethod=HmacSHA256&...&Signature=xxx
    /// ```
    pub fn build_signed_query(
        &self,
        method: &str,
        host: &str,
        path: &str,
        params: &HashMap<String, String>,
    ) -> String {
        let timestamp = self.get_timestamp();

        // Build auth parameters
        let mut all_params = params.clone();
        all_params.insert("AccessKeyId".to_string(), self.api_key.clone());
        all_params.insert("SignatureMethod".to_string(), "HmacSHA256".to_string());
        all_params.insert("SignatureVersion".to_string(), "2".to_string());
        all_params.insert("Timestamp".to_string(), timestamp);

        // Sort parameters in ASCII order
        let mut sorted_params: Vec<(&String, &String)> = all_params.iter().collect();
        sorted_params.sort_by(|a, b| a.0.cmp(b.0));

        // Build query string with URL encoding
        let query_string = form_urlencoded::Serializer::new(String::new())
            .extend_pairs(sorted_params.iter())
            .finish();

        // Build pre-sign string
        // Format: METHOD\nHOST\nPATH\nQUERY_STRING
        let pre_sign = format!(
            "{}\n{}\n{}\n{}",
            method.to_uppercase(),
            host,
            path,
            query_string
        );

        // Compute HMAC-SHA256 signature
        let signature_bytes = hmac_sha256(
            self.api_secret.as_bytes(),
            pre_sign.as_bytes(),
        );

        // Base64 encode
        let signature_b64 = encode_base64(&signature_bytes);

        // URL encode the signature and append
        let signature_encoded = form_urlencoded::byte_serialize(signature_b64.as_bytes())
            .collect::<String>();

        format!("{}&Signature={}", query_string, signature_encoded)
    }

    /// Sign WebSocket authentication request (for v2 private channels)
    ///
    /// # WebSocket Auth Format (v2.1)
    ///
    /// Pre-sign string: `GET\nHOST\n/ws/v2\nAccessKeyId=xxx&SignatureMethod=...`
    ///
    /// Returns: (api_key, timestamp, signature_method, signature_version, signature)
    pub fn sign_websocket_auth(&self, host: &str) -> (String, String, String, String, String) {
        let timestamp = self.get_timestamp();

        // Build auth parameters
        let mut params = HashMap::new();
        params.insert("AccessKeyId".to_string(), self.api_key.clone());
        params.insert("SignatureMethod".to_string(), "HmacSHA256".to_string());
        params.insert("SignatureVersion".to_string(), "2.1".to_string());
        params.insert("Timestamp".to_string(), timestamp.clone());

        // Sort parameters
        let mut sorted_params: Vec<(&String, &String)> = params.iter().collect();
        sorted_params.sort_by(|a, b| a.0.cmp(b.0));

        // Build query string
        let query_string = form_urlencoded::Serializer::new(String::new())
            .extend_pairs(sorted_params.iter())
            .finish();

        // Build pre-sign string
        let pre_sign = format!(
            "GET\n{}\n/ws/v2\n{}",
            host,
            query_string
        );

        // Compute signature
        let signature_bytes = hmac_sha256(
            self.api_secret.as_bytes(),
            pre_sign.as_bytes(),
        );

        let signature = encode_base64(&signature_bytes);

        (
            self.api_key.clone(),
            timestamp,
            "HmacSHA256".to_string(),
            "2.1".to_string(),
            signature,
        )
    }

    /// Get API key
    pub fn api_key(&self) -> &str {
        &self.api_key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_signed_query() {
        let credentials = Credentials::new("test_key", "test_secret");
        let auth = HtxAuth::new(&credentials);

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), "btcusdt".to_string());

        let query = auth.build_signed_query(
            "GET",
            "api.huobi.pro",
            "/v1/order/openOrders",
            &params,
        );

        // Should contain all required parameters
        assert!(query.contains("AccessKeyId=test_key"));
        assert!(query.contains("SignatureMethod=HmacSHA256"));
        assert!(query.contains("SignatureVersion=2"));
        assert!(query.contains("Timestamp="));
        assert!(query.contains("symbol=btcusdt"));
        assert!(query.contains("Signature="));

        // Parameters should be sorted (AccessKeyId comes before symbol)
        let access_pos = query.find("AccessKeyId").unwrap();
        let symbol_pos = query.find("symbol").unwrap();
        assert!(access_pos < symbol_pos, "Parameters should be sorted in ASCII order");
    }

    #[test]
    fn test_build_signed_query_post() {
        let credentials = Credentials::new("test_key", "test_secret");
        let auth = HtxAuth::new(&credentials);

        // POST requests have empty params (business params go in body)
        let params = HashMap::new();

        let query = auth.build_signed_query(
            "POST",
            "api.huobi.pro",
            "/v1/order/orders/place",
            &params,
        );

        // Should only contain auth parameters
        assert!(query.contains("AccessKeyId=test_key"));
        assert!(query.contains("SignatureMethod=HmacSHA256"));
        assert!(query.contains("SignatureVersion=2"));
        assert!(query.contains("Timestamp="));
        assert!(query.contains("Signature="));
    }

    #[test]
    fn test_sign_websocket_auth() {
        let credentials = Credentials::new("test_key", "test_secret");
        let auth = HtxAuth::new(&credentials);

        let (api_key, timestamp, method, version, signature) =
            auth.sign_websocket_auth("api.huobi.pro");

        assert_eq!(api_key, "test_key");
        assert!(!timestamp.is_empty());
        assert_eq!(method, "HmacSHA256");
        assert_eq!(version, "2.1");
        assert!(!signature.is_empty());

        // Timestamp should be in YYYY-MM-DDThh:mm:ss format
        assert!(timestamp.contains('T'));
        assert_eq!(timestamp.len(), 19); // 2023-01-20T12:34:56
    }

    #[test]
    fn test_timestamp_format() {
        let credentials = Credentials::new("test_key", "test_secret");
        let auth = HtxAuth::new(&credentials);

        let timestamp = auth.get_timestamp();

        // Should match YYYY-MM-DDThh:mm:ss format
        assert_eq!(timestamp.len(), 19);
        assert!(timestamp.contains('T'));

        // Try to parse it
        let parts: Vec<&str> = timestamp.split('T').collect();
        assert_eq!(parts.len(), 2);

        let date_parts: Vec<&str> = parts[0].split('-').collect();
        assert_eq!(date_parts.len(), 3); // Year-Month-Day

        let time_parts: Vec<&str> = parts[1].split(':').collect();
        assert_eq!(time_parts.len(), 3); // Hour:Minute:Second
    }
}
