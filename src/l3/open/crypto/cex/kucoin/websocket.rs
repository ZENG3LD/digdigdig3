//! KuCoinWebSocket — thin wrapper around UniversalWsTransport<KuCoinProtocol>.
//!
//! ## KuCoin pre-connect: bullet-public token fetch
//!
//! KuCoin requires a REST POST to `/api/v1/bullet-public` (spot) or the
//! futures equivalent before opening the WebSocket. The response provides:
//! - A token (baked into the WS URL as `?token=<token>`)
//! - The actual WS endpoint URL (`data.instanceServers[0].endpoint`)
//! - The recommended ping interval (`pingInterval` ms)
//!
//! This fetch happens in `KuCoinWebSocket::new()`. The resolved URL is passed
//! into `KuCoinProtocol`, whose `endpoint()` returns it directly.
//!
//! ## Usage
//!
//! ```ignore
//! let ws = KuCoinWebSocket::new(None, false, AccountType::Spot).await?;
//! ws.connect(AccountType::Spot).await?;
//! ws.subscribe(SubscriptionRequest::ticker(Symbol::new("BTC", "USDT"))).await?;
//! let stream = ws.event_stream();
//! ```

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use futures_util::Stream;
use serde_json::json;
use tokio::sync::Mutex as TokioMutex;
use url::Url;
use uuid::Uuid;

use crate::core::{
    AccountType, Credentials, ExchangeError, ExchangeResult,
};
use crate::core::traits::WebSocketConnector;
use crate::core::types::{
    ConnectionStatus, OrderbookCapabilities, StreamEvent, SubscriptionRequest,
    WebSocketResult, WsBookChannel,
};
use crate::core::websocket::{StreamSpec, UniversalWsTransport};
use crate::core::HttpClient;

use super::endpoints::{KuCoinEndpoint, KuCoinUrls};
use super::protocol::KuCoinProtocol;

// ─────────────────────────────────────────────────────────────────────────────
// KuCoinWebSocket
// ─────────────────────────────────────────────────────────────────────────────

/// KuCoin WebSocket connector backed by UniversalWsTransport.
///
/// Construction fetches the bullet-public token and resolves the WS URL.
/// Call `connect()` afterward to open the actual WebSocket.
pub struct KuCoinWebSocket {
    inner: UniversalWsTransport<KuCoinProtocol>,
    _account_type: AccountType,
}

impl KuCoinWebSocket {
    /// Create a new connector.
    ///
    /// Fetches the bullet-public token to resolve the WS URL. Does NOT
    /// open the WebSocket yet — call `connect()` for that.
    ///
    /// - `credentials` — `None` for public streams.
    /// - `testnet`      — `true` for sandbox endpoints.
    /// - `account_type` — determines spot vs futures bullet endpoint + topics.
    pub async fn new(
        credentials: Option<Credentials>,
        testnet: bool,
        account_type: AccountType,
    ) -> ExchangeResult<Self> {
        let (resolved_url, ping_interval_ms) =
            fetch_bullet_token(testnet, account_type, credentials.as_ref()).await?;

        let protocol = KuCoinProtocol::new(account_type, testnet, resolved_url, ping_interval_ms);
        let inner = UniversalWsTransport::new(protocol, account_type, testnet, credentials);

        Ok(Self { inner, _account_type: account_type })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WebSocketConnector impl — delegates to inner transport
// ─────────────────────────────────────────────────────────────────────────────

#[async_trait]
impl WebSocketConnector for KuCoinWebSocket {
    async fn connect(&self, _account_type: AccountType) -> WebSocketResult<()> {
        self.inner.connect().await
    }

    async fn disconnect(&self) -> WebSocketResult<()> {
        self.inner.disconnect().await
    }

    fn connection_status(&self) -> ConnectionStatus {
        self.inner.connection_status()
    }

    async fn subscribe(&self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let spec = StreamSpec::try_from(request)?;
        self.inner.subscribe(spec).await
    }

    async fn unsubscribe(&self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let spec = StreamSpec::try_from(request)?;
        self.inner.unsubscribe(spec).await
    }

    fn event_stream(&self) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>> {
        Box::pin(self.inner.event_stream())
    }

    fn active_subscriptions(&self) -> Vec<SubscriptionRequest> {
        self.inner
            .active_subscriptions()
            .into_iter()
            .map(SubscriptionRequest::from)
            .collect()
    }

    fn ping_rtt_handle(&self) -> Option<Arc<TokioMutex<u64>>> {
        None
    }

    fn orderbook_capabilities(&self, account_type: AccountType) -> OrderbookCapabilities {
        static SPOT_CHANNELS: &[WsBookChannel] = &[
            WsBookChannel::snapshot("spotMarket/level2Depth5",  5,  100),
            WsBookChannel::snapshot("spotMarket/level2Depth50", 50, 100),
            WsBookChannel::delta("market/level2", None, None),
        ];
        static FUTURES_CHANNELS: &[WsBookChannel] = &[
            WsBookChannel::snapshot("contractMarket/level2Depth5",  5,  100),
            WsBookChannel::snapshot("contractMarket/level2Depth50", 50, 100),
            WsBookChannel::delta("contractMarket/level2", None, None),
        ];

        match account_type {
            AccountType::FuturesCross | AccountType::FuturesIsolated => OrderbookCapabilities {
                ws_depths: &[5, 50],
                ws_default_depth: Some(50),
                rest_max_depth: Some(100),
                rest_depth_values: &[20, 100],
                supports_snapshot: true,
                supports_delta: true,
                update_speeds_ms: &[],
                default_speed_ms: None,
                ws_channels: FUTURES_CHANNELS,
                checksum: None,
                has_sequence: true,
                has_prev_sequence: false,
                supports_aggregation: false,
                aggregation_levels: &[],
            },
            _ => OrderbookCapabilities {
                ws_depths: &[5, 50],
                ws_default_depth: Some(50),
                rest_max_depth: None,
                rest_depth_values: &[20, 100],
                supports_snapshot: true,
                supports_delta: true,
                update_speeds_ms: &[],
                default_speed_ms: None,
                ws_channels: SPOT_CHANNELS,
                checksum: None,
                has_sequence: true,
                has_prev_sequence: false,
                supports_aggregation: false,
                aggregation_levels: &[],
            },
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Bullet-public token fetch
// ─────────────────────────────────────────────────────────────────────────────

/// POST to KuCoin bullet-public (or bullet-private for credentialed users).
///
/// Returns `(resolved_ws_url, ping_interval_ms)`.
async fn fetch_bullet_token(
    testnet: bool,
    account_type: AccountType,
    credentials: Option<&Credentials>,
) -> ExchangeResult<(Url, u64)> {
    let urls = if testnet { KuCoinUrls::TESTNET } else { KuCoinUrls::MAINNET };
    let base = urls.rest_url(account_type);
    let use_private = credentials.is_some();

    let endpoint_path = if use_private {
        KuCoinEndpoint::WsPrivateToken.path()
    } else {
        KuCoinEndpoint::WsPublicToken.path()
    };

    let url = format!("{}{}", base, endpoint_path);
    let http = HttpClient::new(30_000)?;

    let response = if use_private {
        let creds = credentials.expect("checked above");
        use super::auth::KuCoinAuth;
        let auth = KuCoinAuth::new(creds)?;
        let body = json!({});
        let headers = auth.sign_request("POST", endpoint_path, &body.to_string());
        http.post(&url, &body, &headers).await?
    } else {
        http.post(&url, &json!({}), &HashMap::new()).await?
    };

    // Check KuCoin response code
    let code = response
        .get("code")
        .and_then(|c| c.as_str())
        .unwrap_or("500000");
    if code != "200000" {
        let msg = response
            .get("msg")
            .and_then(|m| m.as_str())
            .unwrap_or("failed to get WebSocket token");
        return Err(ExchangeError::Api {
            code: code.parse().unwrap_or(-1),
            message: msg.to_string(),
        });
    }

    let data = response
        .get("data")
        .ok_or_else(|| ExchangeError::Parse("bullet-public: missing 'data'".into()))?;

    let token = data
        .get("token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ExchangeError::Parse("bullet-public: missing 'token'".into()))?;

    let servers = data
        .get("instanceServers")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ExchangeError::Parse("bullet-public: missing 'instanceServers'".into()))?;

    let server = servers
        .first()
        .ok_or_else(|| ExchangeError::Parse("bullet-public: no instance servers".into()))?;

    let endpoint = server
        .get("endpoint")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ExchangeError::Parse("bullet-public: missing 'endpoint'".into()))?;

    let ping_interval_ms = server
        .get("pingInterval")
        .and_then(|v| v.as_u64())
        .unwrap_or(18_000);

    let connect_id = Uuid::new_v4().to_string().replace('-', "");
    let ws_url_str = format!("{}?token={}&connectId={}", endpoint, token, connect_id);

    let ws_url = Url::parse(&ws_url_str).map_err(|e| {
        ExchangeError::Parse(format!("bullet-public: invalid WS URL '{}': {}", ws_url_str, e))
    })?;

    Ok((ws_url, ping_interval_ms))
}
