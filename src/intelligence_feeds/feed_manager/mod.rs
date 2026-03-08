//! # Feed Manager
//!
//! Registry, metadata, and factory for all 88 intelligence feed connectors.
//!
//! This module is intentionally separate from [`crate::connector_manager`],
//! which handles exchange/stock/forex connectors that implement the core
//! `MarketData` / `Trading` traits.  Intelligence feeds have no shared trait,
//! so the factory returns `Box<dyn Any>` for caller-side downcasting.
//!
//! ## Modules
//!
//! | Module | Contents |
//! |--------|----------|
//! | [`feed_id`] | [`FeedId`] — 88-variant enum identifying every feed |
//! | [`registry`] | [`FeedMetadata`], [`FeedCategory`], [`FeedAuthType`], [`FeedRegistry`], `FEEDS` |
//! | [`factory`] | [`FeedCredentials`], [`FeedFactory`] |
//!
//! ## Quick Start
//!
//! ```ignore
//! use connectors_v5::intelligence_feeds::feed_manager::{
//!     FeedId, FeedFactory, FeedCredentials, FeedRegistry,
//! };
//!
//! // Enumerate all public feeds
//! for meta in FeedRegistry::public_feeds() {
//!     println!("{} — {}", meta.name, meta.description);
//! }
//!
//! // Create a feed that needs no auth
//! let any = FeedFactory::create_public(FeedId::HackerNews)?;
//! // downcast: let hn = any.downcast::<HackerNewsConnector>().unwrap();
//!
//! // Create an authenticated feed
//! let any = FeedFactory::create_authenticated(
//!     FeedId::Fred,
//!     FeedCredentials::ApiKey("my_fred_key".into()),
//! ).await?;
//! ```

pub mod feed_id;
pub mod registry;
pub mod factory;

pub use feed_id::FeedId;
pub use registry::{FeedAuthType, FeedCategory, FeedMetadata, FeedRegistry, FEEDS};
pub use factory::{FeedCredentials, FeedFactory};
