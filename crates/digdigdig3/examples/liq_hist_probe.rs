//! Live probe for the public liquidation REST-history endpoints
//! (2026-08-01: BitMEX + Bitfinex `/v2/liquidations/hist` + GateIO
//! `/futures/{settle}/liq_orders`). Prints per-venue row counts and the
//! newest row so side/price/qty mapping can be eyeballed against the wire.
//!
//! Run: `cargo run --example liq_hist_probe --release`

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{AccountType, ExchangeId};
use digdigdig3::core::SymbolInput;

#[tokio::main]
async fn main() {
    let hub = ExchangeHub::new();
    let probes: &[(ExchangeId, Option<&str>, AccountType)] = &[
        // Bitfinex: venue-wide endpoint, no symbol filter → market-wide slice.
        (ExchangeId::Bitfinex, None, AccountType::Spot),
        // Bitfinex filtered to the BTC perp.
        (ExchangeId::Bitfinex, Some("tBTCF0:USTF0"), AccountType::Spot),
        (ExchangeId::GateIO, Some("BTC_USDT"), AccountType::FuturesCross),
        (ExchangeId::Bitmex, Some("XBTUSD"), AccountType::FuturesCross),
    ];
    for (ex, sym, at) in probes {
        if let Err(e) = hub.connect_public(*ex, false).await {
            println!("{ex:?}: connect_public failed: {e}");
            continue;
        }
        let Some(conn) = hub.rest(*ex) else {
            println!("{ex:?}: no REST connector");
            continue;
        };
        let res = conn
            .get_liquidation_history(
                sym.map(SymbolInput::Raw),
                None,
                None,
                Some(10),
                *at,
            )
            .await;
        match res {
            Ok(rows) => {
                println!(
                    "{ex:?} sym={:?}: {} rows{}",
                    sym,
                    rows.len(),
                    rows.first()
                        .map(|r| format!(
                            " | newest: {} {:?} px={} qty={} ts={}",
                            r.symbol, r.side, r.price, r.quantity, r.timestamp
                        ))
                        .unwrap_or_default()
                );
            }
            Err(e) => println!("{ex:?} sym={:?}: ERR {e}", sym),
        }
    }
}
