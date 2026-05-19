//! dig3-cure — ad-hoc dataset inspection and deduplication CLI.
//!
//! Usage:
//!   dig3-cure --root <PATH> --exchange <NAME> --account <ACCOUNT>
//!             --symbol <SYM> --stream <KIND>
//!             [--dry-run] [--from <ts_ms>] [--to <ts_ms>]
//!             [--time-gap <ms>]

use std::path::PathBuf;

use digdigdig3_core::core::storage::{StorageConfig, StorageManager, StreamKey};

fn usage() -> ! {
    eprintln!(
        "Usage: dig3-cure \
         --root <PATH> \
         --exchange <NAME> \
         --account <ACCOUNT> \
         --symbol <SYM> \
         --stream <KIND> \
         [--dry-run] \
         [--from <ts_ms>] \
         [--to <ts_ms>] \
         [--time-gap <ms>]"
    );
    std::process::exit(1);
}

struct Args {
    root: PathBuf,
    exchange: String,
    account: String,
    symbol: String,
    stream: String,
    dry_run: bool,
    from_ms: i64,
    to_ms: i64,
    time_gap_ms: i64,
}

fn parse_args() -> Args {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    let mut root: Option<PathBuf> = None;
    let mut exchange: Option<String> = None;
    let mut account: Option<String> = None;
    let mut symbol: Option<String> = None;
    let mut stream: Option<String> = None;
    let mut dry_run = false;
    let mut from_ms: i64 = 0;
    let mut to_ms: i64 = i64::MAX;
    let mut time_gap_ms: i64 = 60_000;

    while i < raw.len() {
        match raw[i].as_str() {
            "--root" => {
                i += 1;
                root = Some(PathBuf::from(raw.get(i).unwrap_or_else(|| usage())));
            }
            "--exchange" => {
                i += 1;
                exchange = Some(raw.get(i).cloned().unwrap_or_else(|| usage()));
            }
            "--account" => {
                i += 1;
                account = Some(raw.get(i).cloned().unwrap_or_else(|| usage()));
            }
            "--symbol" => {
                i += 1;
                symbol = Some(raw.get(i).cloned().unwrap_or_else(|| usage()));
            }
            "--stream" => {
                i += 1;
                stream = Some(raw.get(i).cloned().unwrap_or_else(|| usage()));
            }
            "--dry-run" => {
                dry_run = true;
            }
            "--from" => {
                i += 1;
                from_ms = raw
                    .get(i)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_else(|| usage());
            }
            "--to" => {
                i += 1;
                to_ms = raw
                    .get(i)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_else(|| usage());
            }
            "--time-gap" => {
                i += 1;
                time_gap_ms = raw
                    .get(i)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_else(|| usage());
            }
            _ => {
                eprintln!("Unknown flag: {}", raw[i]);
                usage();
            }
        }
        i += 1;
    }

    Args {
        root: root.unwrap_or_else(|| usage()),
        exchange: exchange.unwrap_or_else(|| usage()),
        account: account.unwrap_or_else(|| usage()),
        symbol: symbol.unwrap_or_else(|| usage()),
        stream: stream.unwrap_or_else(|| usage()),
        dry_run,
        from_ms,
        to_ms,
        time_gap_ms,
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = parse_args();

    let config = StorageConfig {
        root: args.root,
        ..StorageConfig::default()
    };
    let storage = StorageManager::new(config)?;

    let key = StreamKey {
        exchange: args.exchange,
        account: args.account,
        symbol: args.symbol,
        stream_kind: args.stream,
    };

    // Use IntegrityChecker directly so we can apply the custom time-gap threshold,
    // then run Deduper + GapDetector for the full picture.
    let integrity = digdigdig3_core::core::cure::integrity::IntegrityChecker::new(&storage)
        .with_time_gap_threshold(args.time_gap_ms)
        .check(&key, args.from_ms, args.to_ms)
        .await?;

    let (kept, removed) = if args.dry_run {
        (integrity.record_count, integrity.duplicate_count)
    } else {
        digdigdig3_core::core::cure::dedup::Deduper::new(&storage)
            .dedup(&key, args.from_ms, args.to_ms)
            .await?
    };

    let kind_lower = key.stream_kind.to_lowercase();
    let ob_gaps = if kind_lower.contains("orderbook") || kind_lower.contains("delta") {
        digdigdig3_core::core::cure::gap::GapDetector::new(&storage)
            .detect(&key, args.from_ms, args.to_ms)
            .await?
    } else {
        vec![]
    };

    // ── Print report ──────────────────────────────────────────────────────────
    println!("=== dig3-cure report ===");
    println!(
        "stream:        {}/{}/{}/{}",
        integrity.stream.exchange,
        integrity.stream.account,
        integrity.stream.symbol,
        integrity.stream.stream_kind
    );
    println!(
        "range:         {} – {}",
        integrity.from_ms, integrity.to_ms
    );
    println!("records:       {}", integrity.record_count);
    println!(
        "first/last ts: {:?} / {:?}",
        integrity.first_ts, integrity.last_ts
    );
    if let Some(avg) = integrity.avg_interval_ms {
        println!("avg interval:  {:.1} ms", avg);
    }
    println!("duplicates:    {}", integrity.duplicate_count);
    println!("out-of-order:  {}", integrity.out_of_order_count);
    println!("parse errors:  {}", integrity.parse_errors);
    println!("time gaps:     {} (threshold {}ms)", integrity.time_gaps.len(), args.time_gap_ms);
    for g in &integrity.time_gaps {
        println!("  gap {} ms at [{} – {}]", g.duration_ms, g.start_ms, g.end_ms);
    }
    println!("seq gaps (raw JSON): {}", integrity.sequence_gaps.len());
    for g in &integrity.sequence_gaps {
        println!("  seq jump {}->{} at {}", g.from_seq, g.to_seq, g.ts_ms);
    }

    println!();
    if args.dry_run {
        println!("dedup (dry-run): would keep {}, remove {}", kept, removed);
    } else {
        println!("dedup:           kept {}, removed {}", kept, removed);
        println!("  output stream: {}_deduped", key.stream_kind);
    }

    println!("orderbook gaps (tracker): {}", ob_gaps.len());
    for g in &ob_gaps {
        println!(
            "  expected {} got {} at {}",
            g.expected, g.got, g.ts_ms
        );
    }

    Ok(())
}
