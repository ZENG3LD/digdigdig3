//! # HTTP Client - абстракция для REST запросов
//!
//! Универсальный HTTP клиент с:
//! - Retry логикой с exponential backoff
//! - Rate limiting (429 handling)
//! - Timeout handling
//! - Error mapping
//! - Debug logging

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use reqwest::{Client, Method, Response, header::HeaderMap};
use serde_json::Value;

use crate::core::types::{ExchangeError, ExchangeResult};

// ═══════════════════════════════════════════════════════════════════════════════
// CONFIGURATION
// ═══════════════════════════════════════════════════════════════════════════════

/// Конфигурация retry логики
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Максимальное количество попыток
    pub max_attempts: u32,
    /// Начальная задержка для exponential backoff (ms)
    pub initial_backoff_ms: u64,
    /// Максимальная задержка (ms)
    pub max_backoff_ms: u64,
    /// Множитель для exponential backoff
    pub backoff_multiplier: f64,
    /// Jitter factor (0.0-1.0) для рандомизации задержек
    /// 0.0 = без jitter, 1.0 = до 100% рандомизации
    pub jitter_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_backoff_ms: 100,
            max_backoff_ms: 5000,
            backoff_multiplier: 2.0,
            jitter_factor: 0.0,
        }
    }
}

impl RetryConfig {
    /// Конфигурация для нестабильных API (например, Bithumb)
    /// Больше попыток, короткие таймауты, jitter для избежания thundering herd
    pub fn unreliable_api() -> Self {
        Self {
            max_attempts: 7,
            initial_backoff_ms: 500,
            max_backoff_ms: 8000,
            backoff_multiplier: 1.5,
            jitter_factor: 0.3, // 30% jitter
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// HTTP CLIENT
// ═══════════════════════════════════════════════════════════════════════════════

/// HTTP клиент для REST API с retry и rate limiting
pub struct HttpClient {
    client: Client,
    timeout: Duration,
    retry_config: RetryConfig,
    debug: bool,
    /// Total number of HTTP requests attempted (including retries)
    pub requests_total: Arc<AtomicU64>,
    /// Total number of HTTP errors encountered
    pub errors_total: Arc<AtomicU64>,
    /// Latency of the most recently completed request in milliseconds
    pub last_latency_ms: Arc<AtomicU64>,
}

impl HttpClient {
    /// Создать новый HTTP клиент
    pub fn new(timeout_ms: u64) -> ExchangeResult<Self> {
        Self::with_config(timeout_ms, RetryConfig::default())
    }

    /// Создать HTTP клиент с кастомной конфигурацией retry
    pub fn with_config(timeout_ms: u64, retry_config: RetryConfig) -> ExchangeResult<Self> {
        let client = Client::builder()
            .timeout(Duration::from_millis(timeout_ms))
            .build()
            .map_err(|e| ExchangeError::Network(format!("Failed to create HTTP client: {}", e)))?;

        let debug = std::env::var("DEBUG_API").is_ok();

        Ok(Self {
            client,
            timeout: Duration::from_millis(timeout_ms),
            retry_config,
            debug,
            requests_total: Arc::new(AtomicU64::new(0)),
            errors_total: Arc::new(AtomicU64::new(0)),
            last_latency_ms: Arc::new(AtomicU64::new(0)),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // PUBLIC API
    // ═══════════════════════════════════════════════════════════════════════════

    /// GET запрос с retry
    pub async fn get(
        &self,
        url: &str,
        params: &HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        self.request_with_retry(Method::GET, url, params, &HashMap::new(), None).await
    }

    /// GET запрос с заголовками и retry
    pub async fn get_with_headers(
        &self,
        url: &str,
        params: &HashMap<String, String>,
        headers: &HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        self.request_with_retry(Method::GET, url, params, headers, None).await
    }

    /// POST запрос с retry
    pub async fn post(
        &self,
        url: &str,
        body: &Value,
        headers: &HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        self.request_with_retry(Method::POST, url, &HashMap::new(), headers, Some(body)).await
    }

    /// POST с query params и retry
    pub async fn post_with_params(
        &self,
        url: &str,
        params: &HashMap<String, String>,
        body: &Value,
        headers: &HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        self.request_with_retry(Method::POST, url, params, headers, Some(body)).await
    }

    /// DELETE запрос с retry
    pub async fn delete(
        &self,
        url: &str,
        params: &HashMap<String, String>,
        headers: &HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        self.request_with_retry(Method::DELETE, url, params, headers, None).await
    }

    /// DELETE запрос с JSON body и retry (for APIs like Paradex batch cancel)
    pub async fn delete_with_body(
        &self,
        url: &str,
        body: &Value,
        headers: &HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        self.request_with_retry(Method::DELETE, url, &HashMap::new(), headers, Some(body)).await
    }

    /// PUT запрос с retry
    pub async fn put(
        &self,
        url: &str,
        body: &Value,
        headers: &HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        self.request_with_retry(Method::PUT, url, &HashMap::new(), headers, Some(body)).await
    }

    /// PATCH запрос с retry
    pub async fn patch(
        &self,
        url: &str,
        body: &Value,
        headers: &HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        self.request_with_retry(Method::PATCH, url, &HashMap::new(), headers, Some(body)).await
    }

    /// GET запрос с заголовками, возвращает тело и заголовки ответа
    ///
    /// Используется когда нужен доступ к response headers (например, X-MBX-USED-WEIGHT-1M у Binance).
    pub async fn get_with_response_headers(
        &self,
        url: &str,
        params: &HashMap<String, String>,
        headers: &HashMap<String, String>,
    ) -> ExchangeResult<(Value, HeaderMap)> {
        self.request_returning_headers(Method::GET, url, params, headers, None).await
    }

    /// POST запрос, возвращает тело и заголовки ответа
    pub async fn post_with_response_headers(
        &self,
        url: &str,
        body: &Value,
        headers: &HashMap<String, String>,
    ) -> ExchangeResult<(Value, HeaderMap)> {
        self.request_returning_headers(Method::POST, url, &HashMap::new(), headers, Some(body)).await
    }

    /// DELETE запрос, возвращает тело и заголовки ответа
    pub async fn delete_with_response_headers(
        &self,
        url: &str,
        params: &HashMap<String, String>,
        headers: &HashMap<String, String>,
    ) -> ExchangeResult<(Value, HeaderMap)> {
        self.request_returning_headers(Method::DELETE, url, params, headers, None).await
    }

    /// GET запрос для бинарных данных (без JSON парсинга)
    ///
    /// Используется для скачивания файлов (например, .bi5 для Dukascopy)
    pub async fn get_bytes(&self, url: &str) -> ExchangeResult<Vec<u8>> {
        let response = self.client
            .get(url)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Failed to download {}: {}", url, e)))?;

        if !response.status().is_success() {
            return Err(ExchangeError::Api {
                code: response.status().as_u16() as i32,
                message: format!("HTTP {} for {}", response.status(), url),
            });
        }

        response
            .bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| ExchangeError::Network(format!("Failed to read bytes: {}", e)))
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // METRICS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Returns (requests_total, errors_total, last_latency_ms)
    pub fn stats(&self) -> (u64, u64, u64) {
        (
            self.requests_total.load(Ordering::Relaxed),
            self.errors_total.load(Ordering::Relaxed),
            self.last_latency_ms.load(Ordering::Relaxed),
        )
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // RETRY LOGIC
    // ═══════════════════════════════════════════════════════════════════════════

    /// Выполнить запрос с retry логикой
    async fn request_with_retry(
        &self,
        method: Method,
        url: &str,
        params: &HashMap<String, String>,
        headers: &HashMap<String, String>,
        body: Option<&Value>,
    ) -> ExchangeResult<Value> {
        let mut last_error = ExchangeError::Network("No attempts made".to_string());
        let mut current_backoff = self.retry_config.initial_backoff_ms;

        for attempt in 0..self.retry_config.max_attempts {
            if attempt > 0 {
                self.log_retry(attempt, current_backoff, &last_error);
                tokio::time::sleep(Duration::from_millis(current_backoff)).await;
            }

            self.requests_total.fetch_add(1, Ordering::Relaxed);
            let start = std::time::Instant::now();

            match self.execute_request(&method, url, params, headers, body).await {
                Ok(response) => {
                    let elapsed = start.elapsed().as_millis() as u64;
                    self.last_latency_ms.store(elapsed, Ordering::Relaxed);
                    return self.handle_response(response, attempt).await;
                }
                Err(e) => {
                    let elapsed = start.elapsed().as_millis() as u64;
                    self.last_latency_ms.store(elapsed, Ordering::Relaxed);
                    self.errors_total.fetch_add(1, Ordering::Relaxed);
                    last_error = e;

                    // Проверяем, стоит ли ретраить
                    if !self.should_retry(&last_error) {
                        return Err(last_error);
                    }

                    // Exponential backoff
                    current_backoff = self.calculate_next_backoff(current_backoff);
                }
            }
        }

        // Все попытки исчерпаны
        Err(ExchangeError::Network(format!(
            "Max retries ({}) exceeded. Last error: {}",
            self.retry_config.max_attempts, last_error
        )))
    }

    /// Выполнить запрос с retry, возвращая тело и заголовки ответа
    ///
    /// Идентично `request_with_retry`, но сохраняет заголовки ответа перед
    /// потреблением тела. Используется когда нужны response headers (например,
    /// `X-MBX-USED-WEIGHT-1M` для Binance rate limiting).
    async fn request_returning_headers(
        &self,
        method: Method,
        url: &str,
        params: &HashMap<String, String>,
        headers: &HashMap<String, String>,
        body: Option<&Value>,
    ) -> ExchangeResult<(Value, HeaderMap)> {
        let mut last_error = ExchangeError::Network("No attempts made".to_string());
        let mut current_backoff = self.retry_config.initial_backoff_ms;

        for attempt in 0..self.retry_config.max_attempts {
            if attempt > 0 {
                self.log_retry(attempt, current_backoff, &last_error);
                tokio::time::sleep(Duration::from_millis(current_backoff)).await;
            }

            self.requests_total.fetch_add(1, Ordering::Relaxed);
            let start = std::time::Instant::now();

            match self.execute_request(&method, url, params, headers, body).await {
                Ok(response) => {
                    let elapsed = start.elapsed().as_millis() as u64;
                    self.last_latency_ms.store(elapsed, Ordering::Relaxed);
                    return self.handle_response_with_headers(response, attempt).await;
                }
                Err(e) => {
                    let elapsed = start.elapsed().as_millis() as u64;
                    self.last_latency_ms.store(elapsed, Ordering::Relaxed);
                    self.errors_total.fetch_add(1, Ordering::Relaxed);
                    last_error = e;

                    if !self.should_retry(&last_error) {
                        return Err(last_error);
                    }

                    current_backoff = self.calculate_next_backoff(current_backoff);
                }
            }
        }

        Err(ExchangeError::Network(format!(
            "Max retries ({}) exceeded. Last error: {}",
            self.retry_config.max_attempts, last_error
        )))
    }

    /// Выполнить один HTTP запрос
    async fn execute_request(
        &self,
        method: &Method,
        url: &str,
        params: &HashMap<String, String>,
        headers: &HashMap<String, String>,
        body: Option<&Value>,
    ) -> Result<Response, ExchangeError> {
        let mut request = self.client.request(method.clone(), url);

        // Query params
        if !params.is_empty() {
            request = request.query(params);
        }

        // Headers
        for (key, value) in headers {
            request = request.header(key.as_str(), value.as_str());
        }

        // Body для POST/PUT
        if let Some(body) = body {
            request = request.json(body);
        }

        self.log_request(method, url, params);

        request
            .send()
            .await
            .map_err(|e| self.map_reqwest_error(e))
    }

    /// Обработка ответа с учётом retry
    async fn handle_response(&self, response: Response, _attempt: u32) -> ExchangeResult<Value> {
        let status = response.status();
        let status_code = status.as_u16();

        // Извлекаем Retry-After заголовок ДО потребления body
        let retry_after = self.extract_retry_after(&response);

        let body = response
            .text()
            .await
            .map_err(|e| ExchangeError::Network(format!("Failed to read response: {}", e)))?;

        self.log_response(status_code, &body);

        // Rate limit - особая обработка
        if status_code == 429 {
            return Err(ExchangeError::RateLimitExceeded {
                retry_after,
                message: self.extract_error_message(&body),
            });
        }

        // Успешный ответ
        if status.is_success() {
            return serde_json::from_str(&body)
                .map_err(|e| ExchangeError::ParseError(format!("Invalid JSON: {} - Body: {}", e, body)));
        }

        // Другие ошибки
        let json: Option<Value> = serde_json::from_str(&body).ok();
        Err(self.map_http_error(status_code, json.as_ref(), &body))
    }

    /// Обработка ответа с сохранением заголовков
    ///
    /// Извлекает заголовки из ответа до потребления тела, затем обрабатывает
    /// статус и парсит JSON — аналогично `handle_response`.
    async fn handle_response_with_headers(
        &self,
        response: Response,
        _attempt: u32,
    ) -> ExchangeResult<(Value, HeaderMap)> {
        let status = response.status();
        let status_code = status.as_u16();

        // Извлекаем все заголовки ДО потребления тела
        let resp_headers = response.headers().clone();

        // Извлекаем Retry-After заголовок ДО потребления body
        let retry_after = self.extract_retry_after(&response);

        let body = response
            .text()
            .await
            .map_err(|e| ExchangeError::Network(format!("Failed to read response: {}", e)))?;

        self.log_response(status_code, &body);

        // Rate limit - особая обработка
        if status_code == 429 {
            return Err(ExchangeError::RateLimitExceeded {
                retry_after,
                message: self.extract_error_message(&body),
            });
        }

        // Успешный ответ
        if status.is_success() {
            let value = serde_json::from_str(&body)
                .map_err(|e| ExchangeError::ParseError(format!("Invalid JSON: {} - Body: {}", e, body)))?;
            return Ok((value, resp_headers));
        }

        // Другие ошибки
        let json: Option<Value> = serde_json::from_str(&body).ok();
        Err(self.map_http_error(status_code, json.as_ref(), &body))
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // RETRY HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Определить, стоит ли ретраить эту ошибку
    fn should_retry(&self, error: &ExchangeError) -> bool {
        match error {
            // Сетевые ошибки - ретраим
            ExchangeError::Network(_) => true,
            ExchangeError::Timeout(_) => true,
            // Rate limit - ретраим (после ожидания)
            ExchangeError::RateLimitExceeded { .. } => true,
            // Server errors (5xx) - ретраим
            ExchangeError::Api { code, .. } if *code >= 500 => true,
            // Всё остальное - не ретраим
            _ => false,
        }
    }

    /// Вычислить следующую задержку (exponential backoff с jitter)
    fn calculate_next_backoff(&self, current: u64) -> u64 {
        let next = (current as f64 * self.retry_config.backoff_multiplier) as u64;
        let next = next.min(self.retry_config.max_backoff_ms);

        // Применяем jitter, если настроен
        if self.retry_config.jitter_factor > 0.0 {
            self.apply_jitter(next)
        } else {
            next
        }
    }

    /// Применить jitter к задержке для избежания thundering herd
    fn apply_jitter(&self, backoff_ms: u64) -> u64 {
        use std::collections::hash_map::RandomState;
        use std::hash::{BuildHasher, Hasher};

        // Используем RandomState для генерации псевдослучайного числа
        let random_state = RandomState::new();
        let mut hasher = random_state.build_hasher();
        hasher.write_u64(crate::core::timestamp_millis());
        let random_value = hasher.finish();

        // Преобразуем в диапазон 0.0-1.0
        let random_factor = (random_value % 10000) as f64 / 10000.0;

        // Применяем jitter: backoff * (1 - jitter_factor * random_factor)
        let jitter = self.retry_config.jitter_factor * random_factor;
        let multiplier = 1.0 - jitter;

        (backoff_ms as f64 * multiplier).max(self.retry_config.initial_backoff_ms as f64) as u64
    }

    /// Извлечь Retry-After заголовок
    fn extract_retry_after(&self, response: &Response) -> Option<u64> {
        response
            .headers()
            .get("Retry-After")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ERROR MAPPING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Маппинг ошибок reqwest
    fn map_reqwest_error(&self, error: reqwest::Error) -> ExchangeError {
        if error.is_timeout() {
            ExchangeError::Timeout(format!("Request timed out after {:?}", self.timeout))
        } else if error.is_connect() {
            ExchangeError::Network(format!("Connection failed: {}", error))
        } else if error.is_request() {
            ExchangeError::InvalidRequest(format!("Invalid request: {}", error))
        } else {
            ExchangeError::Network(format!("HTTP error: {}", error))
        }
    }

    /// Маппинг HTTP ошибок
    fn map_http_error(&self, status: u16, json: Option<&Value>, raw_body: &str) -> ExchangeError {
        let message = json
            .and_then(|j| self.extract_error_from_json(j))
            .unwrap_or_else(|| raw_body.to_string());

        let code = json
            .and_then(|j| j.get("code").and_then(|v| v.as_i64()))
            .map(|c| c as i32)
            .unwrap_or(status as i32);

        match status {
            401 => ExchangeError::InvalidCredentials(message),
            403 => ExchangeError::PermissionDenied(message),
            429 => ExchangeError::RateLimitExceeded {
                retry_after: None,
                message,
            },
            400 | 422 => ExchangeError::InvalidRequest(message),
            404 => ExchangeError::Api { code, message: format!("Not found: {}", message) },
            500..=599 => ExchangeError::Api { code, message: format!("Server error: {}", message) },
            _ => ExchangeError::Api { code, message },
        }
    }

    /// Извлечь сообщение об ошибке из JSON (пробуем разные поля)
    fn extract_error_from_json(&self, json: &Value) -> Option<String> {
        // Пробуем разные поля, которые используют разные биржи
        json.get("msg")
            .or_else(|| json.get("message"))
            .or_else(|| json.get("error"))
            .or_else(|| json.get("err_msg"))
            .or_else(|| json.get("retMsg"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    /// Извлечь сообщение об ошибке из тела ответа
    fn extract_error_message(&self, body: &str) -> String {
        serde_json::from_str::<Value>(body)
            .ok()
            .and_then(|j| self.extract_error_from_json(&j))
            .unwrap_or_else(|| body.to_string())
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // DEBUG LOGGING
    // ═══════════════════════════════════════════════════════════════════════════

    fn log_request(&self, method: &Method, url: &str, params: &HashMap<String, String>) {
        if self.debug {
            eprintln!("[HTTP] {} {} params={:?}", method, url, params);
        }
    }

    fn log_response(&self, status: u16, body: &str) {
        if self.debug {
            eprintln!("[HTTP] Status: {}", status);
            // Обрезаем длинные ответы
            if body.len() > 500 {
                eprintln!("[HTTP] Response: {}...", &body[..500]);
            } else {
                eprintln!("[HTTP] Response: {}", body);
            }
        }
    }

    fn log_retry(&self, attempt: u32, backoff: u64, error: &ExchangeError) {
        if self.debug {
            eprintln!(
                "[HTTP] Retry attempt {} after {}ms. Error: {}",
                attempt, backoff, error
            );
        }
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new(10_000).expect("Failed to create default HTTP client")
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.initial_backoff_ms, 100);
        assert_eq!(config.max_backoff_ms, 5000);
    }

    #[test]
    fn test_calculate_backoff() {
        let client = HttpClient::new(1000).unwrap();

        // 100 -> 200 -> 400 -> 800 -> 1600 -> 3200 -> 5000 (capped)
        assert_eq!(client.calculate_next_backoff(100), 200);
        assert_eq!(client.calculate_next_backoff(200), 400);
        assert_eq!(client.calculate_next_backoff(3000), 5000); // capped
        assert_eq!(client.calculate_next_backoff(5000), 5000); // stays at max
    }

    #[test]
    fn test_should_retry() {
        let client = HttpClient::new(1000).unwrap();

        // Should retry
        assert!(client.should_retry(&ExchangeError::Network("test".into())));
        assert!(client.should_retry(&ExchangeError::Timeout("test".into())));
        assert!(client.should_retry(&ExchangeError::RateLimitExceeded { retry_after: None, message: "test".into() }));
        assert!(client.should_retry(&ExchangeError::Api { code: 500, message: "test".into() }));
        assert!(client.should_retry(&ExchangeError::Api { code: 503, message: "test".into() }));

        // Should NOT retry
        assert!(!client.should_retry(&ExchangeError::InvalidCredentials("test".into())));
        assert!(!client.should_retry(&ExchangeError::PermissionDenied("test".into())));
        assert!(!client.should_retry(&ExchangeError::InvalidRequest("test".into())));
        assert!(!client.should_retry(&ExchangeError::Api { code: 400, message: "test".into() }));
    }
}
