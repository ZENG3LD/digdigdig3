//! # lossless_verify — prove the RAW pump loses nothing.
//!
//! Live REST harness: for each venue, fetch the public feeds whose payloads were
//! enriched (trade / kline / funding / mark / open-interest / ticker) and ASSERT
//! that the venue-specific Option fields are populated where the wire carries them.
//! cargo-check ≠ proof — this hits the real exchanges.
//!
//! Run:
//!     cargo run --example lossless_verify --release
//!     cargo run --example lossless_verify --release -- Binance OKX   # subset
//!
//! Exit code is non-zero if any expected field came back empty (the pump dropped it).
//! No API keys — public endpoints only.

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{AccountType, ExchangeId};

/// One field expectation: name + whether it was populated.
struct Check {
    field: &'static str,
    populated: bool,
}

fn check(field: &'static str, populated: bool) -> Check {
    Check { field, populated }
}

/// Result of probing one (venue, endpoint).
struct Probe {
    venue: &'static str,
    endpoint: &'static str,
    checks: Vec<Check>,
    error: Option<String>,
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let want = |v: &str| args.is_empty() || args.iter().any(|a| a.eq_ignore_ascii_case(v));

    let hub = ExchangeHub::new();
    let mut probes: Vec<Probe> = Vec::new();

    // ── BINANCE: trade quote_qty/is_best_match; kline taker_buy; funding mark_price; OI sums ──
    if want("Binance") {
        let _ = hub.connect_public(ExchangeId::Binance, false).await;
        if let Some(c) = hub.rest(ExchangeId::Binance) {
            match c.get_recent_trades("BTCUSDT".into(), Some(5), AccountType::Spot).await {
                Ok(ts) => {
                    let t = ts.first();
                    probes.push(Probe {
                        venue: "Binance", endpoint: "recent_trades",
                        checks: vec![
                            check("quote_qty", t.map_or(false, |t| t.quote_qty.is_some())),
                            check("is_best_match", t.map_or(false, |t| t.is_best_match.is_some())),
                        ],
                        error: None,
                    });
                }
                Err(e) => probes.push(err("Binance", "recent_trades", e)),
            }
            match c.get_klines("BTCUSDT".into(), "1m", Some(2), AccountType::Spot, None).await {
                Ok(ks) => {
                    let k = ks.first();
                    probes.push(Probe {
                        venue: "Binance", endpoint: "klines",
                        checks: vec![
                            check("taker_buy_base_volume", k.map_or(false, |k| k.taker_buy_base_volume.is_some())),
                            check("taker_buy_quote_volume", k.map_or(false, |k| k.taker_buy_quote_volume.is_some())),
                        ],
                        error: None,
                    });
                }
                Err(e) => probes.push(err("Binance", "klines", e)),
            }
            match c.get_funding_rate_history("BTCUSDT".into(), None, None, Some(2), AccountType::FuturesCross).await {
                Ok(fs) => probes.push(Probe {
                    venue: "Binance", endpoint: "funding_history",
                    checks: vec![check("mark_price", fs.first().map_or(false, |f| f.mark_price.is_some()))],
                    error: None,
                }),
                Err(e) => probes.push(err("Binance", "funding_history", e)),
            }
        }
    }

    // ── OKX: funding rich (premium/interest/sett/impact); OI ccy/usd ──
    if want("OKX") {
        let _ = hub.connect_public(ExchangeId::OKX, false).await;
        if let Some(c) = hub.rest(ExchangeId::OKX) {
            match c.get_funding_rate_history("BTC-USDT-SWAP".into(), None, None, Some(2), AccountType::FuturesCross).await {
                Ok(fs) => probes.push(Probe {
                    venue: "OKX", endpoint: "funding_history",
                    checks: vec![check("realized_rate", fs.first().map_or(false, |f| f.realized_rate.is_some()))],
                    error: None,
                }),
                Err(e) => probes.push(err("OKX", "funding_history", e)),
            }
            match c.get_open_interest_history("BTC-USDT-SWAP".into(), "5m", None, None, Some(2), AccountType::FuturesCross).await {
                Ok(ois) => probes.push(Probe {
                    venue: "OKX", endpoint: "open_interest",
                    checks: vec![
                        check("open_interest_ccy", ois.first().map_or(false, |o| o.open_interest_ccy.is_some())),
                        check("open_interest_usd", ois.first().map_or(false, |o| o.open_interest_usd.is_some())),
                    ],
                    error: None,
                }),
                Err(e) => probes.push(err("OKX", "open_interest", e)),
            }
        }
    }

    // ── BITMEX: trade notional×3/tickDirection/trdType ──
    if want("BitMEX") {
        let _ = hub.connect_public(ExchangeId::Bitmex, false).await;
        if let Some(c) = hub.rest(ExchangeId::Bitmex) {
            match c.get_recent_trades("XBTUSDT".into(), Some(5), AccountType::FuturesCross).await {
                Ok(ts) => {
                    let t = ts.first();
                    probes.push(Probe {
                        venue: "BitMEX", endpoint: "recent_trades",
                        checks: vec![
                            check("gross_value", t.map_or(false, |t| t.gross_value.is_some())),
                            check("home_notional", t.map_or(false, |t| t.home_notional.is_some())),
                            check("foreign_notional", t.map_or(false, |t| t.foreign_notional.is_some())),
                            check("tick_direction", t.map_or(false, |t| t.tick_direction.is_some())),
                        ],
                        error: None,
                    });
                }
                Err(e) => probes.push(err("BitMEX", "recent_trades", e)),
            }
        }
    }

    // ── DERIBIT: trade index/mark/contracts/trade_seq/tick_direction ──
    if want("Deribit") {
        let _ = hub.connect_public(ExchangeId::Deribit, false).await;
        if let Some(c) = hub.rest(ExchangeId::Deribit) {
            match c.get_recent_trades("BTC-PERPETUAL".into(), Some(5), AccountType::FuturesCross).await {
                Ok(ts) => {
                    let t = ts.first();
                    probes.push(Probe {
                        venue: "Deribit", endpoint: "recent_trades",
                        checks: vec![
                            check("index_price", t.map_or(false, |t| t.index_price.is_some())),
                            check("mark_price", t.map_or(false, |t| t.mark_price.is_some())),
                            check("trade_seq", t.map_or(false, |t| t.trade_seq.is_some())),
                            check("tick_direction", t.map_or(false, |t| t.tick_direction.is_some())),
                        ],
                        error: None,
                    });
                }
                Err(e) => probes.push(err("Deribit", "recent_trades", e)),
            }
        }
    }

    // ── GATEIO: contract_stats bundle drained → LSR top_* + Taker long/short + LiqAggregate ──
    if want("GateIO") {
        let _ = hub.connect_public(ExchangeId::GateIO, false).await;
        if let Some(c) = hub.rest(ExchangeId::GateIO) {
            match c.get_long_short_ratio_history("BTC_USDT".into(), "5m", None, None, Some(2), AccountType::FuturesCross).await {
                Ok(ls) => probes.push(Probe {
                    venue: "GateIO", endpoint: "long_short_ratio",
                    checks: vec![
                        check("top_lsr_size", ls.first().map_or(false, |l| l.top_lsr_size.is_some())),
                        check("long_users", ls.first().map_or(false, |l| l.long_users.is_some())),
                    ],
                    error: None,
                }),
                Err(e) => probes.push(err("GateIO", "long_short_ratio", e)),
            }
            match c.get_liquidation_aggregate_history("BTC_USDT".into(), "5m", None, None, Some(2), AccountType::FuturesCross).await {
                Ok(la) => probes.push(Probe {
                    venue: "GateIO", endpoint: "liquidation_aggregate",
                    // liq sizes are often 0 in a quiet bucket; we only assert the call succeeds and returns rows.
                    checks: vec![check("rows_returned", !la.is_empty())],
                    error: None,
                }),
                Err(e) => probes.push(err("GateIO", "liquidation_aggregate", e)),
            }
        }
    }

    // ── BYBIT: ticker 35-field derivative union ──
    if want("Bybit") {
        let _ = hub.connect_public(ExchangeId::Bybit, false).await;
        if let Some(c) = hub.rest(ExchangeId::Bybit) {
            match c.get_ticker("BTCUSDT".into(), AccountType::FuturesCross).await {
                Ok(t) => probes.push(Probe {
                    venue: "Bybit", endpoint: "ticker(linear)",
                    checks: vec![
                        check("mark_price", t.mark_price.is_some()),
                        check("open_interest", t.open_interest.is_some()),
                        check("funding_rate", t.funding_rate.is_some()),
                        check("funding_interval_hour", t.funding_interval_hour.is_some()),
                    ],
                    error: None,
                }),
                Err(e) => probes.push(err("Bybit", "ticker(linear)", e)),
            }
        }
    }

    // ── Report ──
    println!("\n── lossless_verify ─ live RAW-pump field coverage ──────────────");
    let mut total = 0usize;
    let mut populated = 0usize;
    let mut errors = 0usize;
    for p in &probes {
        if let Some(e) = &p.error {
            println!("  [ERR ] {:<8} {:<22} {}", p.venue, p.endpoint, e);
            errors += 1;
            continue;
        }
        for c in &p.checks {
            total += 1;
            let mark = if c.populated { populated += 1; "ok " } else { "MISS" };
            println!("  [{}] {:<8} {:<22} {}", mark, p.venue, p.endpoint, c.field);
        }
    }
    let missed = total - populated;
    println!("────────────────────────────────────────────────────────────────");
    println!("  fields populated: {populated}/{total}   missed: {missed}   call-errors: {errors}");

    if missed > 0 {
        eprintln!("\nFAIL: {missed} enriched field(s) came back empty — the pump dropped data.");
        std::process::exit(1);
    }
    if populated == 0 {
        eprintln!("\nFAIL: no fields verified (all calls errored?) — not a proof of losslessness.");
        std::process::exit(2);
    }
    println!("\nPASS: every probed enriched field is populated from the live wire.");
}

fn err(venue: &'static str, endpoint: &'static str, e: impl std::fmt::Display) -> Probe {
    Probe { venue, endpoint, checks: Vec::new(), error: Some(e.to_string()) }
}
