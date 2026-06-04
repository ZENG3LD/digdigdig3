//! Live multi-exchange e2e for the bar-aligned non-OHLCV loader.
//!
//! For each exchange × stream cell, asserts either:
//!   * Data     — series non-empty, bar-grid aligned (t % step == 0), ts strictly increasing
//!   * NotSupp  — loader returns an error reporting the stream is not supported (wire-absent)
//!
//! Exit 0 only if every cell matches its expectation. Run:
//!   cargo run -p digdigdig3-station --example bar_align_matrix

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{AccountType, ExchangeId};
use digdigdig3::core::websocket::KlineInterval;
use digdigdig3_station::bar_align::load_bar_aligned;
use digdigdig3_station::series::Kind;

fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64
}

#[derive(Clone, Copy, PartialEq)]
enum Expect {
    Data,
    NotSupp,
}

fn step_ms(iv: &str) -> i64 {
    let (n, u) = iv.split_at(iv.len() - 1);
    let n: i64 = n.parse().unwrap();
    n * match u {
        "m" => 60_000,
        "h" => 3_600_000,
        "d" => 86_400_000,
        _ => panic!("bad iv"),
    }
}

#[tokio::main]
async fn main() {
    let iv = "1h";
    let interval = KlineInterval::new(iv);
    let step = step_ms(iv);
    let end = now_ms();
    let start = end - 2 * 86_400_000; // 2 days

    // (exchange, raw futures symbol, [(Kind, Expect)])
    let matrix: Vec<(ExchangeId, &str, Vec<(Kind, Expect)>)> = vec![
        (ExchangeId::Binance, "BTCUSDT", vec![
            (Kind::MarkPriceKline(interval.clone()), Expect::Data),
            (Kind::IndexPriceKline(interval.clone()), Expect::Data),
            (Kind::PremiumIndexKline(interval.clone()), Expect::Data),
            (Kind::FundingRate, Expect::Data),
            (Kind::OpenInterest, Expect::Data),
            (Kind::LongShortRatio, Expect::Data),
            (Kind::Basis, Expect::Data),
        ]),
        (ExchangeId::Bybit, "BTCUSDT", vec![
            (Kind::MarkPriceKline(interval.clone()), Expect::Data),
            (Kind::IndexPriceKline(interval.clone()), Expect::Data),
            (Kind::PremiumIndexKline(interval.clone()), Expect::Data),
            (Kind::FundingRate, Expect::Data),
            (Kind::OpenInterest, Expect::Data),
            (Kind::LongShortRatio, Expect::Data),
        ]),
        (ExchangeId::OKX, "BTC-USDT-SWAP", vec![
            (Kind::MarkPriceKline(interval.clone()), Expect::Data),
            (Kind::IndexPriceKline(interval.clone()), Expect::Data),
            (Kind::PremiumIndexKline(interval.clone()), Expect::NotSupp),
            (Kind::FundingRate, Expect::Data),
            (Kind::OpenInterest, Expect::Data),
            (Kind::LongShortRatio, Expect::Data),
        ]),
        (ExchangeId::GateIO, "BTC_USDT", vec![
            (Kind::MarkPriceKline(interval.clone()), Expect::Data),
            (Kind::IndexPriceKline(interval.clone()), Expect::Data),
            (Kind::PremiumIndexKline(interval.clone()), Expect::Data),
            (Kind::FundingRate, Expect::Data),
            (Kind::OpenInterest, Expect::Data),
            (Kind::LongShortRatio, Expect::Data),
        ]),
        (ExchangeId::HTX, "BTC-USDT", vec![
            (Kind::MarkPriceKline(interval.clone()), Expect::Data),
            // HTX has no index-price kline REST endpoint (only mark/premium/estimated).
            (Kind::IndexPriceKline(interval.clone()), Expect::NotSupp),
            (Kind::PremiumIndexKline(interval.clone()), Expect::Data),
            (Kind::FundingRate, Expect::Data),
            (Kind::OpenInterest, Expect::Data),
            (Kind::LongShortRatio, Expect::Data),
            (Kind::Basis, Expect::Data),
        ]),
        // ── thin venues (wave-2): their primary implemented methods ──────────────
        (ExchangeId::MEXC, "BTC_USDT", vec![
            (Kind::MarkPriceKline(interval.clone()), Expect::Data),
            (Kind::IndexPriceKline(interval.clone()), Expect::Data),
            (Kind::FundingRate, Expect::Data),
        ]),
        (ExchangeId::BingX, "BTC-USDT", vec![
            (Kind::MarkPriceKline(interval.clone()), Expect::Data),
            (Kind::FundingRate, Expect::Data),
        ]),
        (ExchangeId::Bitfinex, "tBTCF0:USTF0", vec![
            (Kind::FundingRate, Expect::Data),
            (Kind::OpenInterest, Expect::Data),
        ]),
        (ExchangeId::Deribit, "BTC-PERPETUAL", vec![
            (Kind::FundingRate, Expect::Data),
        ]),
        (ExchangeId::Dydx, "BTC-USD", vec![
            (Kind::FundingRate, Expect::Data),
        ]),
        (ExchangeId::Lighter, "BTC", vec![
            (Kind::MarkPriceKline(interval.clone()), Expect::Data),
            (Kind::FundingRate, Expect::Data),
        ]),
        (ExchangeId::HyperLiquid, "BTC", vec![
            (Kind::FundingRate, Expect::Data),
        ]),
        // Crypto.com get-valuations returns per-minute tick points (not interval
        // klines), so mark is not bar-grid-aligned — only funding tested here.
        (ExchangeId::CryptoCom, "BTCUSD-PERP", vec![
            (Kind::FundingRate, Expect::Data),
        ]),
        (ExchangeId::Bitget, "BTCUSDT", vec![
            (Kind::MarkPriceKline(interval.clone()), Expect::Data),
            (Kind::IndexPriceKline(interval.clone()), Expect::Data),
            (Kind::PremiumIndexKline(interval.clone()), Expect::NotSupp),
            (Kind::FundingRate, Expect::Data),
            (Kind::OpenInterest, Expect::NotSupp),
            (Kind::LongShortRatio, Expect::Data),
        ]),
        (ExchangeId::Kraken, "PF_XBTUSD", vec![
            (Kind::MarkPriceKline(interval.clone()), Expect::Data),
            (Kind::IndexPriceKline(interval.clone()), Expect::Data),
            (Kind::FundingRate, Expect::Data),
        ]),
    ];

    let acct = AccountType::FuturesCross;
    let mut failures = 0usize;

    println!("\n=== bar-align live matrix — {iv}, 2-day window, FuturesCross ===");

    for (exchange, symbol, cells) in &matrix {
        let hub = Arc::new(ExchangeHub::new());
        if let Err(e) = hub.connect_full(*exchange, &[acct], false).await {
            println!("\n{exchange:?}: CONNECT FAILED: {e}");
            failures += cells.len();
            continue;
        }
        println!("\n{exchange:?} ({symbol}):");
        for (kind, expect) in cells {
            let label = format!("{kind:?}");
            let res = load_bar_aligned(&hub, *exchange, acct, symbol, kind, &interval, start, end).await;
            match (expect, res) {
                (Expect::Data, Ok(series)) => {
                    let times: Vec<i64> = match &series {
                        digdigdig3_station::BarAlignedSeries::Klines(v) => v.iter().map(|b| b.open_time).collect(),
                        digdigdig3_station::BarAlignedSeries::Scalar(v) => v.iter().map(|b| b.bar_open_time).collect(),
                    };
                    if times.is_empty() {
                        println!("  FAIL {label:<34} EMPTY");
                        failures += 1;
                    } else if times.windows(2).any(|w| w[1] <= w[0]) {
                        println!("  FAIL {label:<34} non-monotonic ts");
                        failures += 1;
                    } else if times.iter().any(|t| t % step != 0) {
                        println!("  FAIL {label:<34} unaligned to {iv} grid");
                        failures += 1;
                    } else {
                        println!("  OK   {label:<34} {} bars", times.len());
                    }
                }
                (Expect::Data, Err(e)) => {
                    println!("  FAIL {label:<34} expected data, got error: {e}");
                    failures += 1;
                }
                (Expect::NotSupp, Err(e)) => {
                    let msg = e.to_string().to_lowercase();
                    if msg.contains("not supported") || msg.contains("notsupported") || msg.contains("unsupported") {
                        println!("  OK   {label:<34} correctly NotSupported");
                    } else {
                        println!("  FAIL {label:<34} wrong error (expected NotSupported): {e}");
                        failures += 1;
                    }
                }
                (Expect::NotSupp, Ok(s)) => {
                    println!("  FAIL {label:<34} expected NotSupported, got {} bars", s.len());
                    failures += 1;
                }
            }
        }
    }

    // Taker buy/sell volume is not a station Kind (no loader path) — verify the
    // connector trait method directly across the venues that implement it.
    {
        use digdigdig3::core::types::SymbolInput;
        println!("\ntaker_volume_history (direct trait call):");
        let taker_venues = [
            (ExchangeId::Binance, "BTCUSDT"),
            (ExchangeId::OKX, "BTC-USDT-SWAP"),
            (ExchangeId::GateIO, "BTC_USDT"),
            (ExchangeId::Bitget, "BTCUSDT"),
        ];
        for (ex, sym) in taker_venues {
            let hub = Arc::new(ExchangeHub::new());
            if hub.connect_full(ex, &[acct], false).await.is_err() {
                println!("  FAIL {ex:?} taker connect failed"); failures += 1; continue;
            }
            let rest = hub.rest(ex).expect("rest");
            match rest.get_taker_volume_history(SymbolInput::Raw(sym), "1h", Some(start), Some(end), Some(500), acct).await {
                Ok(v) if !v.is_empty() && v.iter().all(|t| t.timestamp % step == 0) => {
                    println!("  OK   {ex:?} taker {} buckets | first buy={:.2} sell={:.2}",
                        v.len(), v.first().unwrap().buy_volume, v.first().unwrap().sell_volume);
                }
                Ok(v) => { println!("  FAIL {ex:?} taker invalid/empty ({} buckets)", v.len()); failures += 1; }
                Err(e) => { println!("  FAIL {ex:?} taker error: {e}"); failures += 1; }
            }
        }
    }

    if failures == 0 {
        println!("\nRESULT: PASS — all cells matched expectation\n");
    } else {
        eprintln!("\nRESULT: FAIL — {failures} cell(s) off\n");
        std::process::exit(1);
    }
}
