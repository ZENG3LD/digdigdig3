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

use digdigdig3::connector_manager::{ConnectorFactory, ConnectorPool, WebSocketPool};
use digdigdig3::core::traits::{CoreConnector, HasCapabilities, MarketData, MarketDataPublic, BatchOrders};
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

    println!("\n[as_any downcast: exchange-specific Binance method]");
    if let Some(conn) = pool.get(&ExchangeId::Binance) {
        use digdigdig3::l3::open::crypto::cex::binance::BinanceConnector;
        if let Some(binance) = conn.as_any().downcast_ref::<BinanceConnector>() {
            match binance.get_basis_history("BTCUSDT", "PERPETUAL", "5m", Some(3), None, None).await {
                Ok(v) => println!("  OK  Binance.get_basis_history -> {} items", v.as_array().map(|a| a.len()).unwrap_or(0)),
                Err(e) => println!("  ERR Binance.get_basis_history: {}", e),
            }
        } else {
            println!("  downcast to BinanceConnector failed");
        }
    }

    println!("\n[WebSocketPool dispatch]");
    let ws_pool = WebSocketPool::new();
    for id in [ExchangeId::Binance, ExchangeId::Bybit, ExchangeId::OKX] {
        match ConnectorFactory::create_websocket(id, AccountType::Spot, false).await {
            Ok(ws) => {
                ws_pool.insert(id, AccountType::Spot, ws.clone());
                println!("  + {:?} ws inserted, status={:?}", id, ws.connection_status());
            }
            Err(e) => println!("  ! {:?} ws factory failed: {}", id, e),
        }
    }
    println!("  ws_pool.len() = {}", ws_pool.len());

    println!("\n[trait dispatch via pool: max_batch_place_size on Binance]");
    if let Some(conn) = pool.get(&ExchangeId::Binance) {
        let max = BatchOrders::max_batch_place_size(&*conn);
        println!("  Binance max_batch_place_size = {}", max);
    }

    println!("\n[capability discovery via pool]");
    for id in [ExchangeId::Binance, ExchangeId::Bybit, ExchangeId::OKX] {
        if let Some(conn) = pool.get(&id) {
            let caps = HasCapabilities::capabilities(&*conn);
            println!(
                "  {:?}: batch_place={} (max={}), funding_history={}, transfers={}, ws={}",
                id, caps.has_batch_place, caps.max_batch_place_size,
                caps.has_funding_rate_history, caps.has_transfers, caps.has_websocket
            );
        }
    }

    println!("\n── Done. REST surface via Arc<dyn CoreConnector>, WS via separate pool. ──\n");
}
