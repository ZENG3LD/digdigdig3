//! # HTTP Module
//!
//! Чистый HTTP транспорт без бизнес-логики.
//!
//! ## Компоненты
//! - `HttpClient` - HTTP клиент с retry и rate limiting
//! - `RetryConfig` - Конфигурация retry логики
//! - `GraphQlClient` - GraphQL клиент поверх HttpClient (POST + JSON body)

mod client;
pub mod graphql;

pub use client::{HttpClient, RetryConfig};
pub use graphql::GraphQlClient;
