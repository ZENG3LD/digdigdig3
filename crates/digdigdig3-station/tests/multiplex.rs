#![cfg(not(target_arch = "wasm32"))]
//! Phase 2 in-process multi-consumer test.
//!
//! Two SubscriptionHandles from the SAME Station to the same (exchange, symbol,
//! kind) must share one underlying WS — Station::active_streams() == 1 even
//! when 2 handles are alive. When all handles drop, the multiplexer shuts down
//! and active_streams() returns to 0.

use digdigdig3_station::{AccountType, ExchangeId, Station, Stream, SubscriptionSet};

#[tokio::test]
async fn two_handles_share_one_multiplex_actor() {
    let station = Station::builder()
        .build()
        .await
        .expect("Station::build");

    let s1 = SubscriptionSet::new().add(
        ExchangeId::Binance,
        "BTC-USDT",
        AccountType::Spot,
        [Stream::Trade],
    );
    let s2 = SubscriptionSet::new().add(
        ExchangeId::Binance,
        "BTC-USDT",
        AccountType::Spot,
        [Stream::Trade],
    );

    let h1 = station.subscribe(s1).await.expect("subscribe 1").handle;
    let h2 = station.subscribe(s2).await.expect("subscribe 2").handle;

    // Both consumers, one shared multiplex actor.
    assert_eq!(station.active_streams(), 1, "expected 1 shared mux after 2 subscribes");

    drop(h1);
    // Give the drop's release_consumer a tick to apply.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    assert_eq!(station.active_streams(), 1, "mux still alive while h2 holds it");

    drop(h2);
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    assert_eq!(station.active_streams(), 0, "all handles dropped — mux should retire");
}
