//! Live probe: HyperLiquid SPOT wire-symbol resolution for REST market-data.
//!
//! Validates:
//!  1. get_klines  — regression for MU/USDC + PURR/USDC (existing path)
//!  2. get_orderbook — new Spot resolution
//!  3. get_price    — new Spot resolution (allMids keyed "@N")
//!  4. get_recent_trades — new Spot resolution
//!  5. get_ticker   — new Spot path (allMids + l2Book, no perp universe lookup)
//!
//! Also validates perp regression: BTC FuturesCross get_klines still works.
//!
//! Run: cargo run -p digdigdig3 --example hl_spot_wire_probe

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::{AccountType, ExchangeId};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let hub = ExchangeHub::new();
    hub.connect_public(ExchangeId::HyperLiquid, false).await?;
    let conn = hub.rest(ExchangeId::HyperLiquid).expect("connector");

    let mut pass = 0usize;
    let mut fail = 0usize;

    macro_rules! check {
        ($label:expr, $result:expr) => {
            match $result {
                Ok(v) => {
                    println!("PASS {}: {:?}", $label, v);
                    pass += 1;
                }
                Err(e) => {
                    println!("FAIL {}: {}", $label, e);
                    fail += 1;
                }
            }
        };
        // Variant: expect Ok and non-empty
        ($label:expr, $result:expr, nonempty) => {
            match $result {
                Ok(v) if !v.is_empty() => {
                    println!("PASS {}: {} items", $label, v.len());
                    pass += 1;
                }
                Ok(_) => {
                    println!("FAIL {}: returned empty", $label);
                    fail += 1;
                }
                Err(e) => {
                    println!("FAIL {}: {}", $label, e);
                    fail += 1;
                }
            }
        };
    }

    // ─── 1. get_klines regression ────────────────────────────────────────────
    let klines_mu = conn
        .get_klines("MU/USDC".into(), "1h", Some(5), AccountType::Spot, None)
        .await;
    check!("get_klines MU/USDC Spot", klines_mu, nonempty);

    let klines_purr = conn
        .get_klines("PURR/USDC".into(), "1h", Some(5), AccountType::Spot, None)
        .await;
    check!("get_klines PURR/USDC Spot", klines_purr, nonempty);

    // ─── 2. get_orderbook ────────────────────────────────────────────────────
    let ob_mu = conn
        .get_orderbook("MU/USDC".into(), None, AccountType::Spot)
        .await
        .map(|ob| format!("bids={} asks={}", ob.bids.len(), ob.asks.len()));
    check!("get_orderbook MU/USDC Spot", ob_mu);

    let ob_purr = conn
        .get_orderbook("PURR/USDC".into(), None, AccountType::Spot)
        .await
        .map(|ob| format!("bids={} asks={}", ob.bids.len(), ob.asks.len()));
    check!("get_orderbook PURR/USDC Spot", ob_purr);

    // ─── 3. get_price ────────────────────────────────────────────────────────
    // Price is a type alias for f64 on HyperLiquid
    let price_mu = conn
        .get_price("MU/USDC".into(), AccountType::Spot)
        .await
        .map(|p| format!("price={p}"));
    check!("get_price MU/USDC Spot", price_mu);

    let price_purr = conn
        .get_price("PURR/USDC".into(), AccountType::Spot)
        .await
        .map(|p| format!("price={p}"));
    check!("get_price PURR/USDC Spot", price_purr);

    // ─── 4. get_recent_trades ────────────────────────────────────────────────
    let trades_mu = conn
        .get_recent_trades("MU/USDC".into(), Some(5), AccountType::Spot)
        .await;
    check!("get_recent_trades MU/USDC Spot", trades_mu, nonempty);

    let trades_purr = conn
        .get_recent_trades("PURR/USDC".into(), Some(5), AccountType::Spot)
        .await;
    check!("get_recent_trades PURR/USDC Spot", trades_purr, nonempty);

    // ─── 5. get_ticker ────────────────────────────────────────────────────────
    let ticker_mu = conn
        .get_ticker("MU/USDC".into(), AccountType::Spot)
        .await
        .map(|t| format!("last={} bid={:?} ask={:?}", t.last_price, t.bid_price, t.ask_price));
    check!("get_ticker MU/USDC Spot", ticker_mu);

    let ticker_purr = conn
        .get_ticker("PURR/USDC".into(), AccountType::Spot)
        .await
        .map(|t| format!("last={} bid={:?} ask={:?}", t.last_price, t.bid_price, t.ask_price));
    check!("get_ticker PURR/USDC Spot", ticker_purr);

    // ─── 6. Perp regression: BTC FuturesCross ────────────────────────────────
    let klines_btc = conn
        .get_klines("BTC".into(), "1h", Some(3), AccountType::FuturesCross, None)
        .await;
    check!("get_klines BTC FuturesCross (perp regression)", klines_btc, nonempty);

    let price_btc = conn
        .get_price("BTC".into(), AccountType::FuturesCross)
        .await
        .map(|p| format!("price={p}"));
    check!("get_price BTC FuturesCross (perp regression)", price_btc);

    println!("\n--- SUMMARY: {pass} PASS / {fail} FAIL ---");
    if fail > 0 {
        Err(format!("{fail} probe(s) failed").into())
    } else {
        Ok(())
    }
}
