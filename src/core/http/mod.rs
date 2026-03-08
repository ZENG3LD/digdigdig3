//! # HTTP Module
//!
//! Чистый HTTP транспорт без бизнес-логики.
//!
//! ## Компоненты
//! - `HttpClient` - HTTP клиент с retry и rate limiting
//! - `RetryConfig` - Конфигурация retry логики

mod client;

pub use client::{HttpClient, RetryConfig};
