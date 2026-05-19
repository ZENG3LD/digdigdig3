//! # exchange_hub_demo — full high-level building of dig3 ExchangeHub.
//!
//! Demonstrates every public surface reachable through the unified hub:
//! REST trait dispatch, MarketDataPublic, capability discovery, WebSocket,
//! and exchange-specific inherent methods via as_any() downcast.
//!
//! Run:
//!     cargo run --example exchange_hub_demo
//!
//! No API keys required — public endpoints only.

use digdigdig3_core::connector_manager::ExchangeHub;
use digdigdig3_core::core::traits::{HasCapabilities, MarketData, MarketDataPublic};
use digdigdig3_core::core::types::{AccountType, ExchangeId, Symbol};
use digdigdig3_core::l3::open::crypto::cex::binance::BinanceConnector;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("── ExchangeHub demo ────────────────────────────────────────");

    // ── 1. Build hub: one call per exchange wires REST + WS ────────────
    let hub = ExchangeHub::new();
    println!("\n[1. connect_full — single call wires REST + WS]");
    for id in [ExchangeId::Binance, ExchangeId::Bybit, ExchangeId::OKX] {
        match hub.connect_full(id, &[AccountType::Spot], false).await {
            Ok(()) => println!("  + {:?} connected", id),
            Err(e) => println!("  ! {:?} failed: {}", id, e),
        }
    }
    println!("  hub: rest={}, ws={}", hub.len_rest(), hub.len_ws());

    // ── 2. Discovery: ask what each exchange supports ──────────────────
    println!("\n[2. capability discovery before any call]");
    for id in hub.ids() {
        if let Some(caps) = hub.capabilities(id) {
            println!(
                "  {:?}: batch_place={} (max={}), funding_history={}, transfers={}, ws={}",
                id,
                caps.has_batch_place,
                caps.max_batch_place_size,
                caps.has_funding_rate_history,
                caps.has_transfers,
                caps.has_websocket,
            );
        }
    }

    // ── 3. REST: tickers via &dyn MarketData (vtable dispatch) ─────────
    println!("\n[3. REST: get_ticker via Arc<dyn CoreConnector>]");
    let btc = Symbol::new("BTC", "USDT");
    let btc_str = btc.to_concat();
    for id in hub.ids() {
        if let Some(rest) = hub.rest(id) {
            match MarketData::get_ticker(&*rest, btc_str.as_str().into(), AccountType::Spot).await {
                Ok(t) => println!("  {:?} ticker.last = {}", id, t.last_price),
                Err(e) => println!("  {:?} ticker err: {}", id, e),
            }
        }
    }

    // ── 4. MarketDataPublic: funding rate via the same Arc<dyn CoreConnector> ──
    println!("\n[4. MarketDataPublic: funding rate via same trait object]");
    let funding_symbols = [
        (ExchangeId::Binance, Symbol::new("BTC", "USDT")),
        (ExchangeId::Bybit, Symbol::new("BTC", "USDT")),
        (
            ExchangeId::OKX,
            Symbol {
                base: "BTC".into(),
                quote: "USDT".into(),
                raw: Some("BTC-USDT-SWAP".into()),
            },
        ),
    ];
    for (id, sym) in funding_symbols {
        if let Some(rest) = hub.rest(id) {
            let sym_concat = sym.to_concat();
            let sym_str = sym.raw.as_deref().unwrap_or(&sym_concat);
            match MarketDataPublic::get_funding_rate_history(
                &*rest,
                sym_str.into(),
                None,
                None,
                Some(3),
                AccountType::FuturesCross,
            )
            .await
            {
                Ok(rates) if !rates.is_empty() => {
                    println!("  {:?} funding[0].rate = {}", id, rates[0].rate)
                }
                Ok(_) => println!("  {:?} funding: 0 items", id),
                Err(e) => println!("  {:?} funding err: {}", id, e),
            }
        }
    }

    // ── 5. as_any() escape hatch: Binance-specific inherent method ─────
    println!("\n[5. as_any downcast: Binance.get_basis_history]");
    if let Some(rest) = hub.rest(ExchangeId::Binance) {
        if let Some(binance) = rest.as_any().downcast_ref::<BinanceConnector>() {
            match binance
                .get_basis_history("BTCUSDT", "PERPETUAL", "5m", Some(3), None, None)
                .await
            {
                Ok(v) => println!(
                    "  basis_history: {} items",
                    v.as_array().map(|a| a.len()).unwrap_or(0)
                ),
                Err(e) => println!("  basis_history err: {}", e),
            }
        } else {
            println!("  downcast to BinanceConnector failed");
        }
    }

    // ── 6. Capability check via HasCapabilities directly on rest handle ─
    println!("\n[6. HasCapabilities direct check on rest handle]");
    for id in hub.ids() {
        if let Some(rest) = hub.rest(id) {
            let caps = HasCapabilities::capabilities(&*rest);
            println!("  {:?}: has_ticker={} has_klines={}", id, caps.has_ticker, caps.has_klines);
        }
    }

    // ── 7. WebSocket: separate handle via same hub ─────────────────────
    println!("\n[7. WS via hub.ws(id, AccountType)]");
    for id in hub.ids() {
        if let Some(ws) = hub.ws(id, AccountType::Spot) {
            println!("  {:?} ws status={:?}", id, ws.connection_status());
        } else {
            println!("  {:?} ws: no entry (WS factory skipped)", id);
        }
    }

    // ── 8. Shutdown ────────────────────────────────────────────────────
    println!("\n[8. shutdown — REST + all WS for one exchange]");
    hub.shutdown(ExchangeId::Bybit);
    println!(
        "  after shutdown(Bybit): rest_pool.len={}, ws_pool.len={}",
        hub.len_rest(),
        hub.len_ws()
    );

    // ── 9. Hub conveniences ────────────────────────────────────────────
    println!("\n[9. hub conveniences: list_connected + is_connected]");
    println!("  list_connected() = {:?}", hub.list_connected());
    println!("  is_connected(Binance) = {}", hub.is_connected(ExchangeId::Binance));
    println!("  is_connected(KuCoin)  = {}", hub.is_connected(ExchangeId::KuCoin));

    println!("\n── Done. One hub, all surfaces. ──");
    Ok(())
}
