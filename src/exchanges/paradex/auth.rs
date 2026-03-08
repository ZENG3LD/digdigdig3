//! # Paradex Authentication
//!
//! Реализация JWT-based аутентификации для Paradex API.
//!
//! ## Алгоритм
//!
//! Paradex использует JWT tokens для авторизации:
//! 1. JWT token получается через POST /v1/auth с StarkNet signature
//! 2. Token живет 5 минут, рекомендуется обновлять каждые 3 минуты
//! 3. Для WebSocket: один раз authenticate, потом не требуется повторная авторизация
//!
//! ## Headers для приватных endpoint'ов
//!
//! - `Authorization: Bearer {jwt_token}`
//!
//! ## Note
//!
//! Создание JWT токена требует StarkNet криптографии (sign с private key).
//! В этой реализации мы ПРЕДПОЛАГАЕМ, что JWT token уже получен и передан через credentials.
//! Для полной реализации нужна интеграция с starknet-rs или аналогом.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::{
    Credentials, ExchangeResult, ExchangeError,
};

/// Paradex аутентификация (JWT-based)
#[derive(Clone)]
pub struct ParadexAuth {
    /// JWT token (pre-obtained, или будет получен через StarkNet signing)
    jwt_token: Arc<RwLock<Option<String>>>,

    /// StarkNet account address (если используем StarkNet signing)
    #[allow(dead_code)]
    account_address: Option<String>,

    /// Time offset: server_time - local_time (milliseconds)
    time_offset_ms: Arc<RwLock<i64>>,
}

impl ParadexAuth {
    /// Создать новый auth handler
    ///
    /// ВАЖНО: Paradex требует JWT token для приватных endpoint'ов.
    ///
    /// # Варианты использования:
    ///
    /// 1. **JWT token передан через credentials.api_key**:
    ///    ```ignore
    ///    let creds = Credentials::new("jwt_token_here", "");
    ///    let auth = ParadexAuth::new(&creds)?;
    ///    ```
    ///
    /// 2. **StarkNet account address + signing (TODO)**:
    ///    Требует реализации StarkNet signature generation
    pub fn new(credentials: &Credentials) -> ExchangeResult<Self> {
        // API key = JWT token (упрощенный вариант)
        let jwt_token = if !credentials.api_key.is_empty() {
            Some(credentials.api_key.clone())
        } else {
            None
        };

        // Если есть api_secret, считаем это StarkNet account address (для будущего)
        let account_address = if !credentials.api_secret.is_empty() {
            Some(credentials.api_secret.clone())
        } else {
            None
        };

        Ok(Self {
            jwt_token: Arc::new(RwLock::new(jwt_token)),
            account_address,
            time_offset_ms: Arc::new(RwLock::new(0)),
        })
    }

    /// Sync time with server
    pub async fn sync_time(&self, server_time_ms: i64) {
        let local_time = Self::timestamp_millis() as i64;
        let mut offset = self.time_offset_ms.write().await;
        *offset = server_time_ms - local_time;
    }

    /// Get adjusted timestamp
    async fn get_timestamp(&self) -> u64 {
        let local = Self::timestamp_millis() as i64;
        let offset = *self.time_offset_ms.read().await;
        (local + offset) as u64
    }

    /// Current timestamp in milliseconds
    fn timestamp_millis() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time is before UNIX epoch")
            .as_millis() as u64
    }

    /// Установить JWT token (например, после refresh)
    pub async fn set_jwt_token(&self, token: String) {
        let mut jwt = self.jwt_token.write().await;
        *jwt = Some(token);
    }

    /// Получить JWT token
    pub async fn get_jwt_token(&self) -> ExchangeResult<String> {
        let jwt = self.jwt_token.read().await;
        jwt.clone().ok_or_else(|| ExchangeError::Auth(
            "JWT token not set. Paradex requires authentication.".to_string()
        ))
    }

    /// Подписать запрос и вернуть headers
    ///
    /// Для Paradex это просто добавление Authorization header с JWT token
    pub async fn sign_request(
        &self,
        _method: &str,
        _endpoint: &str,
        _body: &str,
    ) -> ExchangeResult<HashMap<String, String>> {
        let jwt = self.get_jwt_token().await?;

        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), format!("Bearer {}", jwt));
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        Ok(headers)
    }

    /// Получить timestamp для подписи (если нужно)
    pub async fn get_timestamp_header(&self) -> u64 {
        self.get_timestamp().await
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// STARKNET SIGNING (TODO)
// ═══════════════════════════════════════════════════════════════════════════════

// NOTE: Для полноценной реализации нужно добавить:
//
// 1. Зависимость starknet-rs или аналог
// 2. Функцию generate_jwt_token(&self) -> Result<String>
//    - Создает StarkNet signature
//    - Отправляет POST /v1/auth
//    - Парсит jwt_token из ответа
// 3. Автоматический refresh токена каждые 3 минуты
//
// Пример структуры (псевдокод):
//
// ```rust
// pub async fn generate_jwt_token(&self, private_key: &str) -> ExchangeResult<String> {
//     let timestamp = Self::timestamp_millis() / 1000; // seconds
//
//     // StarkNet signature
//     let message = create_auth_message(timestamp);
//     let signature = sign_with_starknet(private_key, &message)?;
//
//     // POST /v1/auth
//     let response = client.post("/v1/auth")
//         .header("PARADEX-STARKNET-ACCOUNT", &self.account_address)
//         .header("PARADEX-STARKNET-SIGNATURE", signature_to_json(&signature))
//         .header("PARADEX-TIMESTAMP", timestamp.to_string())
//         .send()
//         .await?;
//
//     let jwt: JwtResponse = response.json().await?;
//     Ok(jwt.jwt_token)
// }
// ```

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_jwt_token() {
        let credentials = Credentials::new("test_jwt_token", "");
        let auth = ParadexAuth::new(&credentials).unwrap();

        let token = auth.get_jwt_token().await.unwrap();
        assert_eq!(token, "test_jwt_token");
    }

    #[tokio::test]
    async fn test_sign_request() {
        let credentials = Credentials::new("test_jwt_token", "");
        let auth = ParadexAuth::new(&credentials).unwrap();

        let headers = auth.sign_request("GET", "/account", "").await.unwrap();

        assert!(headers.contains_key("Authorization"));
        assert_eq!(
            headers.get("Authorization"),
            Some(&"Bearer test_jwt_token".to_string())
        );
    }

    #[tokio::test]
    async fn test_set_jwt_token() {
        let credentials = Credentials::new("", "");
        let auth = ParadexAuth::new(&credentials).unwrap();

        // Initially no token
        assert!(auth.get_jwt_token().await.is_err());

        // Set token
        auth.set_jwt_token("new_token".to_string()).await;

        // Now should work
        let token = auth.get_jwt_token().await.unwrap();
        assert_eq!(token, "new_token");
    }
}
