//! MarketFeed E2E demo.
//!
//! Connects Binance + Bybit (Spot), subscribes ticker for BTC, prints fan-out
//! events for 15 seconds. Demonstrates that consumers see a unified
//! `FeedEvent` stream regardless of upstream exchange.
//!
//! Run:
//!     cargo run --example feed_demo --release

use std::sync::Arc;
use std::time::Duration;

use digdigdig3_core::connector_manager::{ExchangeHub, MarketFeed};
use digdigdig3_core::core::types::{AccountType, ExchangeId};
use tokio::time::timeout;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt().with_ansi(false).try_init();

    let hub = Arc::new(ExchangeHub::new());

    // Bring two exchanges into the pool.
    hub.connect_full(ExchangeId::Binance, &[AccountType::Spot], false).await?;
    hub.connect_full(ExchangeId::Bybit,   &[AccountType::Spot], false).await?;

    // Connect their WS layers.
    hub.ws(ExchangeId::Binance, AccountType::Spot).unwrap()
        .connect(AccountType::Spot).await?;
    hub.ws(ExchangeId::Bybit, AccountType::Spot).unwrap()
        .connect(AccountType::Spot).await?;

    // Build the feed — at v0 we just toggle a few options to prove the
    // builder shape compiles; behaviour is wired in later phases.
    let feed = MarketFeed::builder(hub.clone())
        .with_storage(false)
        .cache_symbols(true)
        .broadcast_capacity(2048)
        .build();

    let mut h_bnb = feed
        .subscribe_ticker(ExchangeId::Binance, "BTCUSDT", AccountType::Spot)
        .await?;
    let mut h_byb = feed
        .subscribe_ticker(ExchangeId::Bybit, "BTCUSDT", AccountType::Spot)
        .await?;

    println!(
        "feed up: {} upstreams, listening 15s…",
        feed.active_upstreams().await
    );

    let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
    let mut total = 0usize;

    loop {
        if tokio::time::Instant::now() >= deadline { break }

        tokio::select! {
            ev = h_bnb.recv() => if let Some(ev) = ev {
                if total < 4 {
                    println!("[BNB ] {:?} {}: {:?}", ev.account_type, ev.symbol, ev.event);
                }
                total += 1;
            },
            ev = h_byb.recv() => if let Some(ev) = ev {
                if total < 8 {
                    println!("[BYBIT] {:?} {}: {:?}", ev.account_type, ev.symbol, ev.event);
                }
                total += 1;
            },
            _ = timeout(Duration::from_millis(200), futures_util::future::pending::<()>()) => {}
        }
    }

    println!("done: {total} events across {} upstreams", feed.active_upstreams().await);
    Ok(())
}
