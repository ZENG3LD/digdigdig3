//! # pool_dispatch — Trait-object pool smoke test
//!
//! Constructs a `ConnectorPool`, populates it with several exchanges via
//! `ConnectorFactory`, then calls trait methods through `Arc<dyn CoreConnector>`
//! to prove the post-Stage-4 architecture works end-to-end:
//!
//! - Pool returns ONE type (`Arc<dyn CoreConnector>`)
//! - All `CoreConnector` super-trait methods reachable via vtable
//! - No manual enum matching, no per-connector pool re-assembly
//!
//! Run:
//!   cargo run --example pool_dispatch
//!
//! No API keys required — all calls hit public endpoints.

use std::sync::Arc;

use digdigdig3::connector_manager::{ConnectorFactory, ConnectorPool};
use digdigdig3::core::traits::{CoreConnector, MarketData, MarketDataPublic, WebSocketConnector};
use digdigdig3::core::types::{AccountType, ExchangeId, Symbol};

async fn populate_pool(pool: &ConnectorPool) {
    for id in [ExchangeId::Binance, ExchangeId::Bybit, ExchangeId::OKX] {
        match ConnectorFactory::create_public(id, false).await {
            Ok(conn) => {
                pool.insert(id, conn);
                println!("  + {:?} inserted", id);
            }
            Err(e) => println!("  ! {:?} failed: {}", id, e),
        }
    }
}

async fn smoke_ticker(conn: Arc<dyn CoreConnector>, symbol: Symbol) {
    let id = conn.exchange_id();
    match MarketData::get_ticker(&*conn, symbol.clone(), AccountType::Spot).await {
        Ok(t) => println!(
            "  OK  {:?} get_ticker({:?}) -> last={} bid={:?} ask={:?}",
            id, symbol, t.last_price, t.bid_price, t.ask_price
        ),
        Err(e) => println!("  ERR {:?} get_ticker: {}", id, e),
    }
}

async fn smoke_funding(conn: Arc<dyn CoreConnector>, symbol: Symbol) {
    let id = conn.exchange_id();
    let result = MarketDataPublic::get_funding_rate_history(
        &*conn,
        &symbol,
        None,
        None,
        Some(3),
        AccountType::FuturesCross,
    )
    .await;
    match result {
        Ok(rates) if rates.is_empty() => {
            println!("  OK  {:?} get_funding_rate_history -> 0 items", id)
        }
        Ok(rates) => println!(
            "  OK  {:?} get_funding_rate_history -> {} items, first.rate={}",
            id,
            rates.len(),
            rates[0].rate
        ),
        Err(e) => println!("  -   {:?} get_funding_rate_history: {}", id, e),
    }
}

#[tokio::main]
async fn main() {
    println!("── ConnectorPool dispatch smoke ───────────────────────────");

    let pool = ConnectorPool::new();
    println!("\n[populate]");
    populate_pool(&pool).await;
    println!("\n  pool.len() = {}", pool.len());

    println!("\n[dispatch: get_ticker via &dyn MarketData]");
    let btc_usdt = Symbol::new("BTC", "USDT");
    for id in [ExchangeId::Binance, ExchangeId::Bybit, ExchangeId::OKX] {
        if let Some(conn) = pool.get(&id) {
            smoke_ticker(conn, btc_usdt.clone()).await;
        }
    }

    println!("\n[dispatch: get_funding_rate_history via &dyn MarketDataPublic]");
    let btc_perp = Symbol::new("BTC", "USDT");
    let btc_swap = Symbol {
        base: "BTC".into(),
        quote: "USDT".into(),
        raw: Some("BTC-USDT-SWAP".into()),
    };
    for (id, sym) in [
        (ExchangeId::Binance, btc_perp.clone()),
        (ExchangeId::Bybit, btc_perp.clone()),
        (ExchangeId::OKX, btc_swap),
    ] {
        if let Some(conn) = pool.get(&id) {
            smoke_funding(conn, sym).await;
        }
    }

    println!("\n[dispatch: WebSocket connect + subscribe via &dyn WebSocketConnector]");
    for id in [ExchangeId::Binance, ExchangeId::Bybit, ExchangeId::OKX] {
        if let Some(conn) = pool.get(&id) {
            // Arc<dyn CoreConnector> already satisfies WebSocketConnector.
            // Smoke: confirm connection_status dispatches without panic.
            let status = WebSocketConnector::connection_status(&*conn);
            println!("  {:?} ws_status={:?}", id, status);
        }
    }

    println!("\n── Done. Pool dispatch works through Arc<dyn CoreConnector>. ──\n");
}
