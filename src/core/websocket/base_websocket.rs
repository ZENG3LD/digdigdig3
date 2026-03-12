//! # BaseWebSocket - универсальный WebSocket клиент
//!
//! Реализует WebSocketConnector и SubscriptionManager трейты,
//! делегируя биржево-специфичную логику в WebSocketConfig.
//!
//! ## Бизнес-логика
//! - Автоматический reconnect loop с exponential backoff
//! - Connection timeout
//! - Subscription recovery после переподключения
//! - Ping/Pong heartbeat
//! - Debug logging

use std::collections::HashSet;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures_util::stream::Stream;
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::sync::{mpsc, RwLock};
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::core::types::{
    AccountType, ConnectionStatus, StreamEvent, SubscriptionRequest,
    WebSocketError, WebSocketResult,
};

use super::super::config::{IdentityConfig, WebSocketConfig, WsMessageType};
use super::super::traits::{SubscriptionManager, WebSocketConnector};

// ═══════════════════════════════════════════════════════════════════════════════
// CONFIGURATION
// ═══════════════════════════════════════════════════════════════════════════════

/// Конфигурация reconnect логики
#[derive(Debug, Clone)]
pub struct ReconnectConfig {
    /// Максимальное количество попыток reconnect (0 = бесконечно)
    pub max_attempts: u32,
    /// Начальная задержка для reconnect (ms)
    pub initial_delay_ms: u64,
    /// Максимальная задержка (ms)
    pub max_delay_ms: u64,
    /// Множитель для exponential backoff
    pub backoff_multiplier: f64,
    /// Таймаут на подключение (ms)
    pub connection_timeout_ms: u64,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            max_attempts: 0, // infinite
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
            connection_timeout_ms: 10000,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// COMMANDS
// ═══════════════════════════════════════════════════════════════════════════════

/// Команды для внутреннего WebSocket loop
enum WsCommand {
    Subscribe(SubscriptionRequest),
    Unsubscribe(SubscriptionRequest),
    Disconnect,
}

// ═══════════════════════════════════════════════════════════════════════════════
// ABSTRACT WEBSOCKET
// ═══════════════════════════════════════════════════════════════════════════════

/// Абстрактный WebSocket клиент с auto-reconnect
///
/// Параметризован конфигурацией `C: WebSocketConfig + IdentityConfig`.
/// Вся биржево-специфичная логика (форматы сообщений, парсинг) в `C`.
pub struct BaseWebSocket<C: WebSocketConfig + IdentityConfig> {
    config: Arc<C>,
    account_type: AccountType,
    testnet: bool,
    reconnect_config: ReconnectConfig,
    debug: bool,

    // Connection state
    status: Arc<RwLock<ConnectionStatus>>,
    subscriptions: Arc<RwLock<HashSet<SubscriptionRequest>>>,

    // Channels
    command_tx: Option<mpsc::UnboundedSender<WsCommand>>,
    event_rx: Option<Arc<tokio::sync::Mutex<mpsc::UnboundedReceiver<WebSocketResult<StreamEvent>>>>>,
}

impl<C: WebSocketConfig + IdentityConfig + 'static> BaseWebSocket<C> {
    /// Создать новый WebSocket клиент
    pub fn new(config: C, account_type: AccountType, testnet: bool) -> Self {
        Self::with_reconnect_config(config, account_type, testnet, ReconnectConfig::default())
    }

    /// Создать WebSocket клиент с кастомной конфигурацией reconnect
    pub fn with_reconnect_config(
        config: C,
        account_type: AccountType,
        testnet: bool,
        reconnect_config: ReconnectConfig,
    ) -> Self {
        let debug = std::env::var("DEBUG_WS").is_ok();

        Self {
            config: Arc::new(config),
            account_type,
            testnet,
            reconnect_config,
            debug,
            status: Arc::new(RwLock::new(ConnectionStatus::Disconnected)),
            subscriptions: Arc::new(RwLock::new(HashSet::new())),
            command_tx: None,
            event_rx: None,
        }
    }

    /// Получить WebSocket URL
    fn get_ws_url(&self) -> String {
        if self.testnet {
            self.config
                .testnet_ws_url()
                .map(|s| s.to_string())
                .unwrap_or_else(|| self.config.ws_base_url(self.account_type).to_string())
        } else {
            self.config.ws_base_url(self.account_type).to_string()
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // RECONNECT LOOP
    // ═══════════════════════════════════════════════════════════════════════════

    /// Запустить WebSocket с auto-reconnect loop
    async fn start_reconnect_loop(
        config: Arc<C>,
        account_type: AccountType,
        ws_url: String,
        reconnect_config: ReconnectConfig,
        debug: bool,
        status: Arc<RwLock<ConnectionStatus>>,
        subscriptions: Arc<RwLock<HashSet<SubscriptionRequest>>>,
        mut command_rx: mpsc::UnboundedReceiver<WsCommand>,
        event_tx: mpsc::UnboundedSender<WebSocketResult<StreamEvent>>,
    ) {
        let mut current_delay = reconnect_config.initial_delay_ms;
        let mut attempt = 0u32;

        loop {
            // Обновляем статус
            {
                let mut guard = status.write().await;
                *guard = if attempt == 0 {
                    ConnectionStatus::Connecting
                } else {
                    ConnectionStatus::Reconnecting
                };
            }

            if debug {
                eprintln!("[WS] Connecting to {} (attempt {})", ws_url, attempt + 1);
            }

            // Попытка подключения с таймаутом
            let connect_result = timeout(
                Duration::from_millis(reconnect_config.connection_timeout_ms),
                connect_async(&ws_url),
            )
            .await;

            match connect_result {
                Ok(Ok((ws_stream, _))) => {
                    // Успешно подключились
                    {
                        let mut guard = status.write().await;
                        *guard = ConnectionStatus::Connected;
                    }

                    if debug {
                        eprintln!("[WS] Connected successfully");
                    }

                    // Сбрасываем счётчики
                    attempt = 0;
                    current_delay = reconnect_config.initial_delay_ms;

                    // Восстанавливаем подписки
                    let subs_to_restore: Vec<_> = {
                        subscriptions.read().await.iter().cloned().collect()
                    };

                    // Запускаем основной loop
                    let loop_result = Self::run_message_loop(
                        config.clone(),
                        account_type,
                        ws_stream,
                        debug,
                        status.clone(),
                        subscriptions.clone(),
                        &mut command_rx,
                        &event_tx,
                        subs_to_restore,
                    )
                    .await;

                    // Проверяем причину выхода
                    match loop_result {
                        LoopExitReason::Disconnected => {
                            if debug {
                                eprintln!("[WS] Disconnected by user, stopping");
                            }
                            return;
                        }
                        LoopExitReason::Error(e) => {
                            if debug {
                                eprintln!("[WS] Connection error: {}, will reconnect", e);
                            }
                            let _ = event_tx.send(Err(WebSocketError::ConnectionError(e)));
                        }
                        LoopExitReason::ConnectionClosed => {
                            if debug {
                                eprintln!("[WS] Connection closed, will reconnect");
                            }
                        }
                    }
                }
                Ok(Err(e)) => {
                    // Ошибка подключения
                    if debug {
                        eprintln!("[WS] Connection failed: {}", e);
                    }
                    let _ = event_tx.send(Err(WebSocketError::ConnectionError(e.to_string())));
                }
                Err(_) => {
                    // Таймаут
                    if debug {
                        eprintln!(
                            "[WS] Connection timeout ({}ms)",
                            reconnect_config.connection_timeout_ms
                        );
                    }
                    let _ = event_tx.send(Err(WebSocketError::Timeout));
                }
            }

            // Проверяем лимит попыток
            attempt += 1;
            if reconnect_config.max_attempts > 0 && attempt >= reconnect_config.max_attempts {
                if debug {
                    eprintln!(
                        "[WS] Max reconnect attempts ({}) reached, stopping",
                        reconnect_config.max_attempts
                    );
                }
                let mut guard = status.write().await;
                *guard = ConnectionStatus::Disconnected;
                return;
            }

            // Exponential backoff
            if debug {
                eprintln!("[WS] Waiting {}ms before reconnect", current_delay);
            }
            tokio::time::sleep(Duration::from_millis(current_delay)).await;

            current_delay = ((current_delay as f64) * reconnect_config.backoff_multiplier) as u64;
            current_delay = current_delay.min(reconnect_config.max_delay_ms);
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MESSAGE LOOP
    // ═══════════════════════════════════════════════════════════════════════════

    /// Основной loop обработки сообщений
    async fn run_message_loop(
        config: Arc<C>,
        account_type: AccountType,
        ws_stream: tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        debug: bool,
        status: Arc<RwLock<ConnectionStatus>>,
        subscriptions: Arc<RwLock<HashSet<SubscriptionRequest>>>,
        command_rx: &mut mpsc::UnboundedReceiver<WsCommand>,
        event_tx: &mpsc::UnboundedSender<WebSocketResult<StreamEvent>>,
        subs_to_restore: Vec<SubscriptionRequest>,
    ) -> LoopExitReason {
        let (mut write, mut read) = ws_stream.split();

        // Восстанавливаем подписки после reconnect
        for sub in subs_to_restore {
            let msg = config.create_subscribe_message(&sub.symbol, &sub.stream_type, account_type);
            if let Err(e) = write.send(Message::Text(msg.to_string())).await {
                return LoopExitReason::Error(format!("Failed to restore subscription: {}", e));
            }
            if debug {
                eprintln!("[WS] Restored subscription: {:?}", sub);
            }
        }

        // Ping interval
        let ping_interval = config.ping_interval_ms();
        let mut ping_timer = tokio::time::interval(Duration::from_millis(ping_interval));

        loop {
            tokio::select! {
                // Входящие сообщения от биржи
                msg = read.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            if let Ok(json) = serde_json::from_str::<Value>(&text) {
                                match config.classify_message(&json) {
                                    WsMessageType::Data => {
                                        match config.parse_stream_event(json) {
                                            Ok(event) => {
                                                let _ = event_tx.send(Ok(event));
                                            }
                                            Err(e) => {
                                                if debug {
                                                    eprintln!("[WS] Parse error: {}", e);
                                                }
                                                let _ = event_tx.send(Err(e));
                                            }
                                        }
                                    }
                                    WsMessageType::SubscribeAck => {
                                        if debug {
                                            eprintln!("[WS] Subscribe acknowledged");
                                        }
                                    }
                                    WsMessageType::UnsubscribeAck => {
                                        if debug {
                                            eprintln!("[WS] Unsubscribe acknowledged");
                                        }
                                    }
                                    WsMessageType::Pong => {
                                        if debug {
                                            eprintln!("[WS] Pong received");
                                        }
                                    }
                                    WsMessageType::Error => {
                                        if debug {
                                            eprintln!("[WS] Error from exchange: {}", text);
                                        }
                                        let _ = event_tx.send(Err(WebSocketError::ProtocolError(
                                            format!("Exchange error: {}", text)
                                        )));
                                    }
                                    WsMessageType::Unknown => {
                                        if debug {
                                            eprintln!("[WS] Unknown message: {}", text);
                                        }
                                    }
                                }
                            }
                        }
                        Some(Ok(Message::Binary(data))) => {
                            // Некоторые биржи отправляют binary (gzip)
                            // TODO: добавить распаковку если нужно
                            if debug {
                                eprintln!("[WS] Binary message received ({} bytes)", data.len());
                            }
                        }
                        Some(Ok(Message::Ping(data))) => {
                            let _ = write.send(Message::Pong(data)).await;
                        }
                        Some(Ok(Message::Pong(_))) => {
                            if debug {
                                eprintln!("[WS] Pong received (native)");
                            }
                        }
                        Some(Ok(Message::Close(frame))) => {
                            if debug {
                                eprintln!("[WS] Close frame received: {:?}", frame);
                            }
                            let mut guard = status.write().await;
                            *guard = ConnectionStatus::Disconnected;
                            return LoopExitReason::ConnectionClosed;
                        }
                        Some(Ok(Message::Frame(_))) => {
                            // Raw frame, ignore
                        }
                        Some(Err(e)) => {
                            let mut guard = status.write().await;
                            *guard = ConnectionStatus::Disconnected;
                            return LoopExitReason::Error(e.to_string());
                        }
                        None => {
                            // Stream ended
                            let mut guard = status.write().await;
                            *guard = ConnectionStatus::Disconnected;
                            return LoopExitReason::ConnectionClosed;
                        }
                    }
                }

                // Команды от пользователя
                cmd = command_rx.recv() => {
                    match cmd {
                        Some(WsCommand::Subscribe(req)) => {
                            let msg = config.create_subscribe_message(
                                &req.symbol,
                                &req.stream_type,
                                account_type
                            );
                            if let Err(e) = write.send(Message::Text(msg.to_string())).await {
                                let _ = event_tx.send(Err(WebSocketError::ProtocolError(e.to_string())));
                            } else {
                                subscriptions.write().await.insert(req.clone());
                                if debug {
                                    eprintln!("[WS] Subscribed: {:?}", req);
                                }
                            }
                        }
                        Some(WsCommand::Unsubscribe(req)) => {
                            let msg = config.create_unsubscribe_message(
                                &req.symbol,
                                &req.stream_type,
                                account_type
                            );
                            if let Err(e) = write.send(Message::Text(msg.to_string())).await {
                                let _ = event_tx.send(Err(WebSocketError::ProtocolError(e.to_string())));
                            } else {
                                subscriptions.write().await.remove(&req);
                                if debug {
                                    eprintln!("[WS] Unsubscribed: {:?}", req);
                                }
                            }
                        }
                        Some(WsCommand::Disconnect) => {
                            let _ = write.close().await;
                            let mut guard = status.write().await;
                            *guard = ConnectionStatus::Disconnected;
                            return LoopExitReason::Disconnected;
                        }
                        None => {
                            // Command channel closed
                            return LoopExitReason::Disconnected;
                        }
                    }
                }

                // Ping timer
                _ = ping_timer.tick() => {
                    if let Some(ping_msg) = config.create_ping_message() {
                        if let Err(e) = write.send(Message::Text(ping_msg.to_string())).await {
                            if debug {
                                eprintln!("[WS] Failed to send ping: {}", e);
                            }
                        }
                    } else {
                        if let Err(e) = write.send(Message::Ping(vec![])).await {
                            if debug {
                                eprintln!("[WS] Failed to send ping: {}", e);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Причина выхода из message loop
enum LoopExitReason {
    /// Пользователь отключился
    Disconnected,
    /// Соединение закрыто сервером
    ConnectionClosed,
    /// Ошибка
    Error(String),
}

// ═══════════════════════════════════════════════════════════════════════════════
// WebSocketConnector Implementation
// ═══════════════════════════════════════════════════════════════════════════════

impl<C: WebSocketConfig + IdentityConfig + 'static> WebSocketConnector for BaseWebSocket<C> {
    async fn connect(&mut self) -> WebSocketResult<()> {
        // Проверяем текущий статус
        {
            let guard = self.status.read().await;
            if matches!(*guard, ConnectionStatus::Connected | ConnectionStatus::Connecting) {
                return Ok(());
            }
        }

        let ws_url = self.get_ws_url();

        // Создаем каналы
        let (command_tx, command_rx) = mpsc::unbounded_channel();
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        self.command_tx = Some(command_tx);
        self.event_rx = Some(Arc::new(tokio::sync::Mutex::new(event_rx)));

        // Запускаем reconnect loop
        let config = self.config.clone();
        let account_type = self.account_type;
        let reconnect_config = self.reconnect_config.clone();
        let debug = self.debug;
        let status = self.status.clone();
        let subscriptions = self.subscriptions.clone();

        tokio::spawn(async move {
            Self::start_reconnect_loop(
                config,
                account_type,
                ws_url,
                reconnect_config,
                debug,
                status,
                subscriptions,
                command_rx,
                event_tx,
            )
            .await;
        });

        // Ждем подключения с таймаутом
        let connect_timeout = Duration::from_millis(self.reconnect_config.connection_timeout_ms + 1000);
        let start = std::time::Instant::now();

        while start.elapsed() < connect_timeout {
            {
                let guard = self.status.read().await;
                match *guard {
                    ConnectionStatus::Connected => return Ok(()),
                    ConnectionStatus::Disconnected => {
                        // Может быть ошибка подключения
                        tokio::time::sleep(Duration::from_millis(50)).await;
                    }
                    _ => {
                        tokio::time::sleep(Duration::from_millis(50)).await;
                    }
                }
            }
        }

        Err(WebSocketError::Timeout)
    }

    async fn disconnect(&mut self) -> WebSocketResult<()> {
        if let Some(tx) = self.command_tx.take() {
            let _ = tx.send(WsCommand::Disconnect);
        }

        // Ждем отключения
        let start = std::time::Instant::now();
        while start.elapsed() < Duration::from_secs(2) {
            let guard = self.status.read().await;
            if matches!(*guard, ConnectionStatus::Disconnected) {
                break;
            }
            drop(guard);
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        self.event_rx = None;
        Ok(())
    }

    fn connection_status(&self) -> ConnectionStatus {
        // Используем try_read чтобы не блокировать
        // Если лок занят, возвращаем Disconnected как fallback
        match self.status.try_read() {
            Ok(guard) => guard.clone(),
            Err(_) => ConnectionStatus::Disconnected,
        }
    }

    async fn subscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        if let Some(tx) = &self.command_tx {
            tx.send(WsCommand::Subscribe(request))
                .map_err(|_| WebSocketError::ProtocolError("Channel closed".to_string()))?;
            Ok(())
        } else {
            Err(WebSocketError::ConnectionError("Not connected".to_string()))
        }
    }

    async fn unsubscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        if let Some(tx) = &self.command_tx {
            tx.send(WsCommand::Unsubscribe(request))
                .map_err(|_| WebSocketError::ProtocolError("Channel closed".to_string()))?;
            Ok(())
        } else {
            Err(WebSocketError::ConnectionError("Not connected".to_string()))
        }
    }

    fn event_stream(&self) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>> {
        let rx = self.event_rx.clone();

        Box::pin(futures_util::stream::unfold(rx, |rx| async move {
            if let Some(rx) = rx {
                let mut guard = rx.lock().await;
                match guard.recv().await {
                    Some(event) => Some((event, Some(rx.clone()))),
                    None => None,
                }
            } else {
                None
            }
        }))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SubscriptionManager Implementation
// ═══════════════════════════════════════════════════════════════════════════════

impl<C: WebSocketConfig + IdentityConfig + 'static> SubscriptionManager for BaseWebSocket<C> {
    fn active_subscriptions(&self) -> Vec<SubscriptionRequest> {
        // Используем try_read чтобы не блокировать
        match self.subscriptions.try_read() {
            Ok(guard) => guard.iter().cloned().collect(),
            Err(_) => Vec::new(),
        }
    }

    fn has_subscription(&self, request: &SubscriptionRequest) -> bool {
        // Используем try_read чтобы не блокировать
        match self.subscriptions.try_read() {
            Ok(guard) => guard.contains(request),
            Err(_) => false,
        }
    }

    async fn clear_subscriptions(&mut self) -> WebSocketResult<()> {
        let subs: Vec<_> = self.subscriptions.read().await.iter().cloned().collect();
        for sub in subs {
            self.unsubscribe(sub).await?;
        }
        Ok(())
    }

    async fn reconnect_with_subscriptions(&mut self) -> WebSocketResult<()> {
        // С новым auto-reconnect это происходит автоматически
        // Но для ручного reconnect:
        self.disconnect().await?;
        tokio::time::sleep(Duration::from_millis(500)).await;
        self.connect().await
        // Подписки восстановятся автоматически в reconnect loop
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reconnect_config_default() {
        let config = ReconnectConfig::default();
        assert_eq!(config.max_attempts, 0); // infinite
        assert_eq!(config.initial_delay_ms, 1000);
        assert_eq!(config.max_delay_ms, 30000);
        assert_eq!(config.connection_timeout_ms, 10000);
    }
}
