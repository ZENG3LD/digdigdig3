//! REST base URL override plumbing test — Workstream A
//!
//! Verifies that `ExchangeHub::set_rest_base_override` is stored, survives
//! through `connect_public`, and is readable back via `get_rest_base_override`.
//! No live network calls are made — connectors are constructed but never called.

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::ExchangeId;

/// Sets an override, constructs a Binance connector via the hub, and asserts
/// the override value is preserved in the hub map.
#[tokio::test]
async fn rest_override_binance_plumbing() {
    let hub = ExchangeHub::new();

    let override_url = "https://my-proxy.test".to_string();
    hub.set_rest_base_override(ExchangeId::Binance, override_url.clone());

    // Override must be readable before connect.
    assert_eq!(
        hub.get_rest_base_override(ExchangeId::Binance),
        Some(override_url.clone()),
        "override must be stored in hub map before connect"
    );

    // connect_public reads the DashMap and passes override to factory.
    // The bogus URL will never be contacted — we only verify construction succeeds
    // (the override is stored in the connector, not used until a request is made).
    let result = hub.connect_public(ExchangeId::Binance, false).await;
    assert!(
        result.is_ok(),
        "connect_public must succeed with override set: {:?}",
        result
    );

    // Override must still be in the hub map after connect.
    assert_eq!(
        hub.get_rest_base_override(ExchangeId::Binance),
        Some(override_url),
        "override must persist in hub map after connect"
    );
}

/// Cross-check: OKX override also threads through the factory.
#[tokio::test]
async fn rest_override_okx_plumbing() {
    let hub = ExchangeHub::new();

    let override_url = "https://okx-proxy.test".to_string();
    hub.set_rest_base_override(ExchangeId::OKX, override_url.clone());

    assert_eq!(
        hub.get_rest_base_override(ExchangeId::OKX),
        Some(override_url.clone()),
        "OKX override must be stored"
    );

    let result = hub.connect_public(ExchangeId::OKX, false).await;
    assert!(
        result.is_ok(),
        "connect_public(OKX) must succeed with override set: {:?}",
        result
    );

    assert_eq!(
        hub.get_rest_base_override(ExchangeId::OKX),
        Some(override_url),
        "OKX override must persist after connect"
    );
}

/// Verifies clear_rest_base_override removes the entry.
#[tokio::test]
async fn rest_override_clear() {
    let hub = ExchangeHub::new();
    hub.set_rest_base_override(ExchangeId::Binance, "https://proxy.test".to_string());
    hub.clear_rest_base_override(ExchangeId::Binance);
    assert_eq!(
        hub.get_rest_base_override(ExchangeId::Binance),
        None,
        "cleared override must be None"
    );
}

/// Verifies that connect_full also picks up the override.
#[tokio::test]
async fn rest_override_connect_full_plumbing() {
    use digdigdig3::core::types::AccountType;

    let hub = ExchangeHub::new();
    let override_url = "https://bybit-proxy.test".to_string();
    hub.set_rest_base_override(ExchangeId::Bybit, override_url.clone());

    let result = hub.connect_full(ExchangeId::Bybit, &[AccountType::Spot], false).await;
    assert!(
        result.is_ok(),
        "connect_full must succeed with override set: {:?}",
        result
    );

    assert_eq!(
        hub.get_rest_base_override(ExchangeId::Bybit),
        Some(override_url),
        "Bybit override must persist after connect_full"
    );
}
