//! Live validation of the OI-history REST shape across the 4 wired venues:
//! `get_open_interest_history(Raw(native), "1h", None, None, Some(30), FuturesCross)`
//! — row count, ROW ORDERING (live-probed 2026-08-01: Binance/GateIO
//! oldest-first, Bybit/OKX NEWEST-first) and bucket alignment (hourly
//! buckets at :00). Consumers must never trust `.last()` as "newest".
//!
//! Run: `cargo run -p digdigdig3 --release --example oi_hist_probe`

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{AccountType, ExchangeId, SymbolInput};

#[tokio::main]
async fn main() {
    let hub = ExchangeHub::new();
    let venues: &[(ExchangeId, &str)] = &[
        (ExchangeId::Binance, "BTCUSDT"),
        (ExchangeId::Bybit, "BTCUSDT"),
        (ExchangeId::OKX, "BTC-USDT-SWAP"),
        (ExchangeId::GateIO, "BTC_USDT"),
    ];
    for (eid, sym) in venues {
        if let Err(e) = hub.connect_public(*eid, false).await {
            println!("{eid:?}: connect err {e:?}");
            continue;
        }
        let Some(conn) = hub.rest(*eid) else {
            println!("{eid:?}: no rest");
            continue;
        };
        match conn
            .get_open_interest_history(
                SymbolInput::Raw(sym),
                "1h",
                None,
                None,
                Some(30),
                AccountType::FuturesCross,
            )
            .await
        {
            Ok(rows) => {
                let n = rows.len();
                let first = rows.first().map(|r| (r.timestamp, r.open_interest));
                let last = rows.last().map(|r| (r.timestamp, r.open_interest));
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as i64;
                println!(
                    "{eid:?}: {n} rows, first={first:?} last={last:?} now={now} (first_age_min={:?} last_age_min={:?})",
                    first.map(|(t, _)| (now - t) / 60000),
                    last.map(|(t, _)| (now - t) / 60000),
                );
            }
            Err(e) => println!("{eid:?}: ERR {e:?}"),
        }
    }
}
