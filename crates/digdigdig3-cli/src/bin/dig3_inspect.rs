//! Post-mortem analyzer for Station-persisted `.dat` files.
//!
//! Usage:
//!   dig3-inspect trades <path-to-.dat>
//!   dig3-inspect kline  <path-to-.dat>
//!   dig3-inspect ticker <path-to-.dat>
//!   dig3-inspect orderbook <path-to-.dat>
//!
//! Prints per-stream summary:
//!   - total records,
//!   - first / last ts,
//!   - total span,
//!   - gap distribution: count of gaps > 1s, 5s, 30s, 60s, max gap seen,
//!   - mean inter-arrival ms.
//!
//! Used to verify gap-heal behaviour after long live-data runs: any gap
//! ABOVE the configured threshold that the file shows means heal did NOT
//! patch it; gaps below threshold are normal market quiet.

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use digdigdig3_station::data::{BarPoint, ObSnapshotPoint, TickerPoint, TradePoint};
use digdigdig3_station::DataPoint;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: {} <kind> <path-to-.dat>", args[0]);
        eprintln!("  kind: trades | kline | ticker | orderbook");
        std::process::exit(2);
    }
    let kind = args[1].as_str();
    let path = PathBuf::from(&args[2]);

    let mut buf = Vec::new();
    File::open(&path)
        .with_context(|| format!("open {}", path.display()))?
        .read_to_end(&mut buf)?;
    println!("file: {} ({} bytes)", path.display(), buf.len());

    match kind {
        "trades" => analyze::<TradePoint>(&buf, "trade"),
        "kline" => analyze::<BarPoint>(&buf, "kline"),
        "ticker" => analyze::<TickerPoint>(&buf, "ticker"),
        "orderbook" => analyze::<ObSnapshotPoint>(&buf, "orderbook"),
        other => Err(anyhow!("unknown kind `{other}` (try trades/kline/ticker/orderbook)")),
    }
}

fn analyze<T: DataPoint>(buf: &[u8], label: &str) -> Result<()> {
    let size = T::RECORD_SIZE;
    if buf.len() % size != 0 {
        eprintln!("WARNING: byte len {} not multiple of record size {}", buf.len(), size);
    }
    let n = buf.len() / size;
    if n == 0 {
        println!("0 {label} records — nothing to analyze");
        return Ok(());
    }

    let mut timestamps: Vec<i64> = Vec::with_capacity(n);
    for chunk in buf.chunks_exact(size) {
        if let Some(p) = T::decode(chunk) {
            timestamps.push(p.timestamp_ms());
        }
    }

    let first = *timestamps.first().unwrap();
    let last = *timestamps.last().unwrap();
    let span_ms = (last - first).max(0);
    let span_secs = span_ms as f64 / 1000.0;

    println!("records: {}", n);
    println!("first ts: {} ms", first);
    println!("last  ts: {} ms", last);
    println!("span:     {:.1} s ({:.2} min)", span_secs, span_secs / 60.0);
    if n > 1 {
        let mean_iat = span_ms as f64 / (n - 1) as f64;
        println!("mean inter-arrival: {:.1} ms", mean_iat);
    }

    // Gap analysis. Pairwise diffs of consecutive timestamps.
    let mut diffs: Vec<i64> = Vec::with_capacity(n.saturating_sub(1));
    for w in timestamps.windows(2) {
        diffs.push((w[1] - w[0]).max(0));
    }
    diffs.sort_unstable();

    let max_gap = diffs.last().copied().unwrap_or(0);
    let bucket = |thresh_ms: i64| diffs.iter().filter(|d| **d > thresh_ms).count();

    println!();
    println!("gap distribution (consecutive ts diffs):");
    println!("  > 100 ms:  {}", bucket(100));
    println!("  > 1 s:     {}", bucket(1_000));
    println!("  > 5 s:     {}", bucket(5_000));
    println!("  > 10 s:    {}", bucket(10_000));
    println!("  > 30 s:    {}", bucket(30_000));
    println!("  > 60 s:    {}", bucket(60_000));
    println!("  max gap:   {} ms ({:.1} s)", max_gap, max_gap as f64 / 1000.0);

    // Show the 10 largest gaps with their positions, useful to correlate
    // with gap-heal log lines by timestamp.
    if diffs.len() > 0 {
        println!();
        println!("top 10 largest gaps (ms, with surrounding ts):");
        let mut indexed: Vec<(usize, i64)> = timestamps
            .windows(2)
            .enumerate()
            .map(|(i, w)| (i, (w[1] - w[0]).max(0)))
            .collect();
        indexed.sort_by(|a, b| b.1.cmp(&a.1));
        for (idx, gap) in indexed.iter().take(10) {
            if *gap < 1000 { break; }
            println!("  idx {idx:>6}  gap={gap:>8} ms  {} → {}", timestamps[*idx], timestamps[idx + 1]);
        }
    }
    Ok(())
}
