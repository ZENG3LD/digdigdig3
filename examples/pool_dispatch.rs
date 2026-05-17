//! # pool_dispatch — Hub-based trait-object dispatch smoke test
//!
//! Constructs an `ExchangeHub`, populates it via `hub.connect_full()`, then
//! calls trait methods through `Arc<dyn CoreConnector>` to prove the
//! architecture works end-to-end:
//!
//! - Hub returns ONE type (`Arc<dyn CoreConnector>`) via `hub.rest()`
//! - All `CoreConnector` super-trait methods reachable via vtable
//! - No manual enum matching, no per-connector pool re-assembly
//!
//! Run:
//!   cargo run --example pool_dispatch
//!
//! No API keys required — all calls hit public endpoints.

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::traits::{CoreConnector, MarketData, MarketDataPublic, BatchOrders};
use digdigdig3::core::types::{AccountType, ExchangeId, Symbol};

async fn smoke_ticker(conn: std::sync::Arc<dyn CoreConnector>, symbol: Symbol) {
    let id = conn.exchange_id();
    let sym_str = symbol.to_concat();
    match MarketData::get_ticker(&*conn, &sym_str, AccountType::Spot).await {
        Ok(t) => println!(
            "  OK  {:?} get_ticker({:?}) -> last={} bid={:?} ask={:?}",
            id, symbol, t.last_price, t.bid_price, t.ask_price
        ),
        Err(e) => println!("  ERR {:?} get_ticker: {}", id, e),
    }
}

async fn smoke_funding(conn: std::sync::Arc<dyn CoreConnector>, symbol: Symbol) {
    let id = conn.exchange_id();
    let sym_concat = symbol.to_concat();
    let sym_str = symbol.raw.as_deref().unwrap_or(&sym_concat);
    let result = MarketDataPublic::get_funding_rate_history(
        &*conn,
        sym_str,
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
    println!("── Hub dispatch smoke ─────────────────────────────────────");

    let hub = ExchangeHub::new();
    println!("\n[connect_full — REST + WS]");
    for id in [ExchangeId::Binance, ExchangeId::Bybit, ExchangeId::OKX] {
        match hub.connect_full(id, &[AccountType::Spot], false).await {
            Ok(()) => println!("  + {:?} connected", id),
            Err(e) => println!("  ! {:?} failed: {}", id, e),
        }
    }
    println!("\n  hub: rest={}, ws={}", hub.len_rest(), hub.len_ws());

    println!("\n[dispatch: get_ticker via &dyn MarketData]");
    let btc_usdt = Symbol::new("BTC", "USDT");
    for id in [ExchangeId::Binance, ExchangeId::Bybit, ExchangeId::OKX] {
        if let Some(conn) = hub.rest(id) {
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
        if let Some(conn) = hub.rest(id) {
            smoke_funding(conn, sym).await;
        }
    }

    println!("\n[as_any downcast: exchange-specific Binance method]");
    if let Some(conn) = hub.rest(ExchangeId::Binance) {
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

    println!("\n[WebSocket dispatch via hub.ws()]");
    for id in [ExchangeId::Binance, ExchangeId::Bybit, ExchangeId::OKX] {
        if let Some(ws) = hub.ws(id, AccountType::Spot) {
            println!("  + {:?} ws status={:?}", id, ws.connection_status());
        } else {
            println!("  ! {:?} ws: no entry", id);
        }
    }
    println!("  hub ws entries = {}", hub.len_ws());

    println!("\n[trait dispatch: max_batch_place_size on Binance]");
    if let Some(conn) = hub.rest(ExchangeId::Binance) {
        let max = BatchOrders::max_batch_place_size(&*conn);
        println!("  Binance max_batch_place_size = {}", max);
    }

    println!("\n[capability discovery via hub.capabilities()]");
    for id in hub.list_connected() {
        if let Some(caps) = hub.capabilities(id) {
            println!(
                "  {:?}: batch_place={} (max={}), funding_history={}, transfers={}, ws={}",
                id, caps.has_batch_place, caps.max_batch_place_size,
                caps.has_funding_rate_history, caps.has_transfers, caps.has_websocket
            );
        }
    }

    println!("\n[is_connected check]");
    println!("  Binance connected: {}", hub.is_connected(ExchangeId::Binance));
    println!("  KuCoin connected:  {}", hub.is_connected(ExchangeId::KuCoin));

    println!("\n── Done. REST surface via Arc<dyn CoreConnector>, WS via hub. ──\n");
}
