//! Live probe: HyperLiquid SPOT klines must work with the DISPLAY pair name
//! that `get_exchange_info(Spot)` reports (e.g. "MU/USDC"). The connector
//! resolves it internally to the spotMeta wire name ("@N") before hitting
//! candleSnapshot — without that resolution HL returns no candles and chart
//! consumers hang on "Loading...".
//!
//! Run: cargo run -p digdigdig3 --example hl_spot_kline_probe

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::{AccountType, ExchangeId};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let hub = ExchangeHub::new();
    hub.connect_public(ExchangeId::HyperLiquid, false).await?;
    let conn = hub.rest(ExchangeId::HyperLiquid).expect("connector");

    let info = conn.get_exchange_info(AccountType::Spot).await?;
    println!("spot universe: {} symbols", info.len());

    // The user-reported pair + the one HL names by pair (not @N) + a liquid one.
    let wanted = ["MU/USDC", "PURR/USDC", "HYPE/USDC"];
    let mut picks: Vec<String> = info
        .iter()
        .map(|s| s.symbol.clone())
        .filter(|s| wanted.contains(&s.as_str()))
        .collect();
    if picks.is_empty() {
        picks = info.iter().take(3).map(|s| s.symbol.clone()).collect();
    }

    let mut failures = 0usize;
    for sym in &picks {
        match conn
            .get_klines(sym.as_str().into(), "1h", Some(10), AccountType::Spot, None)
            .await
        {
            Ok(klines) if !klines.is_empty() => {
                let last = klines.last().unwrap();
                println!(
                    "PASS {sym}: {} bars, last open_time={} close={}",
                    klines.len(),
                    last.open_time,
                    last.close
                );
            }
            Ok(_) => {
                failures += 1;
                println!("FAIL {sym}: 0 bars returned");
            }
            Err(e) => {
                failures += 1;
                println!("FAIL {sym}: {e}");
            }
        }
    }

    if failures > 0 {
        return Err(format!("{failures}/{} spot kline fetches failed", picks.len()).into());
    }
    println!("ALL PASS");
    Ok(())
}
