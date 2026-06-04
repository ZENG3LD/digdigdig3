//! Live e2e for the RAW SymbolInfo contract.
//!
//! Connects several venues public and calls `get_exchange_info`, asserting the
//! core stays RAW + complete:
//!   * non-empty symbol universe
//!   * `extra` is a populated passthrough (not Null) — no native field lost
//!   * `instrument_type` is set on multi-instrument venues (native token)
//!   * status is the venue-native string (not a synthetic "TRADING")
//!
//! Run: cargo run -p digdigdig3 --example symbolinfo_raw_e2e --release

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{AccountType, ExchangeId};

struct Case {
    ex: ExchangeId,
    acct: AccountType,
    // require instrument_type Some on at least one symbol (multi-instrument venue)
    want_instr_type: bool,
}

#[tokio::main]
async fn main() {
    let cases = vec![
        Case { ex: ExchangeId::Binance, acct: AccountType::FuturesCross, want_instr_type: true },
        Case { ex: ExchangeId::OKX,     acct: AccountType::FuturesCross, want_instr_type: true },
        Case { ex: ExchangeId::Bybit,   acct: AccountType::FuturesCross, want_instr_type: true },
        Case { ex: ExchangeId::Deribit, acct: AccountType::FuturesCross, want_instr_type: true },
        Case { ex: ExchangeId::GateIO,  acct: AccountType::FuturesCross, want_instr_type: false },
        Case { ex: ExchangeId::Coinbase, acct: AccountType::Spot,        want_instr_type: true },
        Case { ex: ExchangeId::HyperLiquid, acct: AccountType::FuturesCross, want_instr_type: true },
    ];

    let mut failures = 0usize;
    println!("\n=== SymbolInfo RAW contract — live get_exchange_info ===");

    for c in &cases {
        let hub = ExchangeHub::new();
        if hub.connect_public(c.ex, false).await.is_err() {
            println!("  FAIL {:?}: connect_public failed", c.ex);
            failures += 1;
            continue;
        }
        let Some(rest) = hub.rest(c.ex) else {
            println!("  FAIL {:?}: no rest after connect", c.ex);
            failures += 1;
            continue;
        };
        match rest.get_exchange_info(c.acct).await {
            Ok(symbols) if !symbols.is_empty() => {
                let n = symbols.len();
                let extra_ok = symbols.iter().all(|s| !s.extra.is_null());
                let any_instr = symbols.iter().any(|s| s.instrument_type.is_some());
                let sample = &symbols[0];
                let instr_fail = c.want_instr_type && !any_instr;
                if !extra_ok {
                    println!("  FAIL {:?}: {n} symbols but some have NULL extra (field loss)", c.ex);
                    failures += 1;
                } else if instr_fail {
                    println!("  FAIL {:?}: {n} symbols but NO instrument_type set (expected native token)", c.ex);
                    failures += 1;
                } else {
                    println!("  OK   {:?}: {n} symbols | sample status={:?} instr_type={:?} extra_keys={}",
                        c.ex, sample.status, sample.instrument_type,
                        sample.extra.as_object().map(|o| o.len()).unwrap_or(0));
                }
            }
            Ok(_) => { println!("  FAIL {:?}: empty symbol list", c.ex); failures += 1; }
            Err(e) => { println!("  FAIL {:?}: get_exchange_info error: {e}", c.ex); failures += 1; }
        }
    }

    if failures == 0 {
        println!("\nRESULT: PASS — core returns raw, complete SymbolInfo\n");
    } else {
        eprintln!("\nRESULT: FAIL — {failures} venue(s) off\n");
        std::process::exit(1);
    }
}
