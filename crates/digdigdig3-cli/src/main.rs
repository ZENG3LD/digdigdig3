use std::path::PathBuf;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use digdigdig3_station::{
    AccountType, Event, ExchangeId, GapHealConfig, PersistenceConfig, Station, Stream,
    SubscriptionSet,
};
use digdigdig3::core::websocket::KlineInterval;

#[derive(Parser, Debug)]
#[command(name = "dig3", version, about = "digdigdig3 unified CLI")]
struct Cli {
    /// Root directory for Station-managed artefacts (trades / bars / snapshots).
    /// Resolves: --storage-root > DIG3_STORAGE_ROOT > ./dig3_storage
    #[arg(long, global = true)]
    storage_root: Option<PathBuf>,

    /// Emit last-N points from disk before live stream takes over (0 = off).
    #[arg(long, global = true, default_value_t = 0)]
    warm_start: usize,

    /// Persist each point to <storage-root>/<kind>/<exch>/<acct>/<sym>/<date>.dat
    #[arg(long, global = true, default_value_t = true, action = clap::ArgAction::Set)]
    persist: bool,

    /// Proactive REST gap-heal when live timestamps jump past threshold.
    /// Currently effective for `trades` and `kline` kinds (the only ones with
    /// a sensible public REST history endpoint).
    #[arg(long, global = true, default_value_t = false, action = clap::ArgAction::Set)]
    gap_heal: bool,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Subscribe and print live events.
    Watch {
        #[command(subcommand)]
        what: WatchKind,
    },
    Persist,
    Replay,
    Matrix,
    Inspect,
    Capture,
    Benchmark,
}

#[derive(Subcommand, Debug)]
enum WatchKind {
    /// Live trade tape.
    Trades { exchange: String, symbol: String, #[arg(long, default_value = "spot")] account: String, #[arg(long)] duration: Option<u64> },
    /// Live aggregated-trade tape.
    AggTrades { exchange: String, symbol: String, #[arg(long, default_value = "spot")] account: String, #[arg(long)] duration: Option<u64> },
    /// Live L2 orderbook ladder (top-N levels, refreshed in place).
    Orderbook { exchange: String, symbol: String, #[arg(long, default_value = "spot")] account: String, #[arg(long)] duration: Option<u64>, #[arg(long, default_value_t = 10)] depth: usize },
    /// Live OHLCV candle stream.
    Kline { exchange: String, symbol: String, #[arg(long, default_value = "spot")] account: String, #[arg(long, default_value = "1m")] interval: String, #[arg(long)] duration: Option<u64> },
    /// Live ticker / 24h stats.
    Ticker { exchange: String, symbol: String, #[arg(long, default_value = "spot")] account: String, #[arg(long)] duration: Option<u64> },
    /// Live mark price (futures).
    Mark { exchange: String, symbol: String, #[arg(long, default_value = "cross")] account: String, #[arg(long)] duration: Option<u64> },
    /// Live funding rate (futures).
    Funding { exchange: String, symbol: String, #[arg(long, default_value = "cross")] account: String, #[arg(long)] duration: Option<u64> },
    /// Live open interest (futures).
    OpenInterest { exchange: String, symbol: String, #[arg(long, default_value = "cross")] account: String, #[arg(long)] duration: Option<u64> },
    /// Live liquidation events (futures).
    Liquidations { exchange: String, symbol: String, #[arg(long, default_value = "cross")] account: String, #[arg(long)] duration: Option<u64> },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();
    let opts = WatchOpts {
        storage_root: cli.storage_root,
        warm_start: cli.warm_start,
        persist: cli.persist,
        gap_heal: cli.gap_heal,
    };

    match cli.cmd {
        Cmd::Watch { what } => run_watch(what, opts).await?,
        Cmd::Persist => println!("dig3 persist: not yet implemented"),
        Cmd::Replay => println!("dig3 replay: not yet implemented"),
        Cmd::Matrix => println!("dig3 matrix: not yet implemented (use `cargo run --example e2e_smoke -p digdigdig3 --release`)"),
        Cmd::Inspect => println!("dig3 inspect: not yet implemented"),
        Cmd::Capture => println!("dig3 capture: not yet implemented"),
        Cmd::Benchmark => println!("dig3 benchmark: not yet implemented"),
    }
    Ok(())
}

#[derive(Clone)]
struct WatchOpts {
    storage_root: Option<PathBuf>,
    warm_start: usize,
    persist: bool,
    gap_heal: bool,
}

async fn build_station(opts: &WatchOpts) -> Result<Station> {
    let mut b = Station::builder();
    if let Some(root) = &opts.storage_root { b = b.storage_root(root.clone()); }
    if opts.warm_start > 0 { b = b.warm_start(opts.warm_start); }
    if opts.persist { b = b.persistence(PersistenceConfig::on()); }
    if opts.gap_heal { b = b.gap_heal(GapHealConfig::on()); }
    b.build().await.context("Station::build")
}

async fn run_watch(kind: WatchKind, opts: WatchOpts) -> Result<()> {
    use WatchKind::*;
    match kind {
        Trades { exchange, symbol, account, duration } => watch_one(&exchange, &symbol, &account, duration, Stream::Trade, &opts, 0).await,
        AggTrades { exchange, symbol, account, duration } => watch_one(&exchange, &symbol, &account, duration, Stream::AggTrade, &opts, 0).await,
        Orderbook { exchange, symbol, account, duration, depth } => watch_one(&exchange, &symbol, &account, duration, Stream::Orderbook, &opts, depth).await,
        Kline { exchange, symbol, account, interval, duration } => watch_one(&exchange, &symbol, &account, duration, Stream::Kline(KlineInterval::new(interval)), &opts, 0).await,
        Ticker { exchange, symbol, account, duration } => watch_one(&exchange, &symbol, &account, duration, Stream::Ticker, &opts, 0).await,
        Mark { exchange, symbol, account, duration } => watch_one(&exchange, &symbol, &account, duration, Stream::MarkPrice, &opts, 0).await,
        Funding { exchange, symbol, account, duration } => watch_one(&exchange, &symbol, &account, duration, Stream::FundingRate, &opts, 0).await,
        OpenInterest { exchange, symbol, account, duration } => watch_one(&exchange, &symbol, &account, duration, Stream::OpenInterest, &opts, 0).await,
        Liquidations { exchange, symbol, account, duration } => watch_one(&exchange, &symbol, &account, duration, Stream::Liquidation, &opts, 0).await,
    }
}

async fn watch_one(
    exchange: &str,
    symbol: &str,
    account: &str,
    duration: Option<u64>,
    stream: Stream,
    opts: &WatchOpts,
    ob_depth: usize,
) -> Result<()> {
    let exch = ExchangeId::from_str(exchange).ok_or_else(|| anyhow!("unknown exchange `{exchange}`"))?;
    let acct = parse_account(account)?;
    let station = build_station(opts).await?;

    let set = SubscriptionSet::new().add(exch, symbol, acct, [stream.clone()]);
    let mut handle = station.subscribe(set).await.context("Station::subscribe")?;

    eprintln!(
        "dig3 watch {:?}: exchange={exchange} symbol={symbol} account={account} warm_start={} persist={} gap_heal={} duration={duration:?} storage_root={}",
        stream, opts.warm_start, opts.persist, opts.gap_heal, station.storage_root().display()
    );

    let deadline = duration.map(|d| tokio::time::Instant::now() + Duration::from_secs(d));
    let mut seq = 0u64;
    loop {
        let event = match deadline {
            Some(at) => tokio::select! { _ = tokio::time::sleep_until(at) => break, ev = handle.recv() => ev },
            None => handle.recv().await,
        };
        let Some(event) = event else { break };
        print_event(&event, ob_depth, seq);
        seq += 1;
    }
    Ok(())
}

fn print_event(event: &Event, ob_depth: usize, seq: u64) {
    match event {
        Event::Trade { exchange, symbol, point } => {
            println!("{ts} {ex:?} {sym} TRADE {side:>4} px={px} qty={qty}",
                ts = point.ts_ms, ex = exchange, sym = symbol,
                side = point.side_label(), px = point.price, qty = point.quantity);
        }
        Event::AggTrade { exchange, symbol, point } => {
            let side = if point.side == 0 { "Buy" } else { "Sell" };
            println!("{ts} {ex:?} {sym} AGG   {side:>4} px={px} qty={qty} id={id}",
                ts = point.ts_ms, ex = exchange, sym = symbol,
                side = side, px = point.price, qty = point.quantity, id = point.agg_id);
        }
        Event::Bar { exchange, symbol, timeframe, point } => {
            println!("{ts} {ex:?} {sym} KLINE [{tf}] O={o} H={h} L={l} C={c} V={v}",
                ts = point.open_time, ex = exchange, sym = symbol, tf = timeframe,
                o = point.open, h = point.high, l = point.low, c = point.close, v = point.volume);
        }
        Event::Ticker { exchange, symbol, point } => {
            println!("{ts} {ex:?} {sym} TICKER last={last} bid={bid} ask={ask} 24h={ch:.4}%",
                ts = point.ts_ms, ex = exchange, sym = symbol,
                last = point.last, bid = point.bid, ask = point.ask, ch = point.change_pct_24h);
        }
        Event::OrderbookSnapshot { exchange, symbol, point } => {
            print_ladder(*exchange, symbol, point.ts_ms, &point.bids, &point.asks, ob_depth.max(1), seq);
        }
        Event::MarkPrice { exchange, symbol, point } => {
            println!("{ts} {ex:?} {sym} MARK mark={m} index={i}",
                ts = point.ts_ms, ex = exchange, sym = symbol, m = point.mark, i = point.index);
        }
        Event::FundingRate { exchange, symbol, point } => {
            println!("{ts} {ex:?} {sym} FUNDING rate={r} next={n}",
                ts = point.ts_ms, ex = exchange, sym = symbol, r = point.rate, n = point.next_funding_time_ms);
        }
        Event::OpenInterest { exchange, symbol, point } => {
            println!("{ts} {ex:?} {sym} OI oi={o} oi_val={v}",
                ts = point.ts_ms, ex = exchange, sym = symbol, o = point.open_interest, v = point.open_interest_value);
        }
        Event::Liquidation { exchange, symbol, point } => {
            let side = if point.side == 0 { "Buy" } else { "Sell" };
            println!("{ts} {ex:?} {sym} LIQ {side:>4} px={px} qty={qty} val={v}",
                ts = point.ts_ms, ex = exchange, sym = symbol, side = side,
                px = point.price, qty = point.quantity, v = point.value);
        }
    }
}

fn print_ladder(
    exchange: ExchangeId,
    symbol: &str,
    ts: i64,
    bids: &[(f64, f64)],
    asks: &[(f64, f64)],
    depth: usize,
    seq: u64,
) {
    let top_bid = bids.first().map(|(p, _)| *p).unwrap_or(0.0);
    let top_ask = asks.first().map(|(p, _)| *p).unwrap_or(0.0);
    let spread = if top_bid > 0.0 && top_ask > 0.0 { top_ask - top_bid } else { 0.0 };
    println!(
        "--- {ex:?} {sym} ts={ts} seq={seq} spread={spread:.4} bid={top_bid} ask={top_ask} ---",
        ex = exchange, sym = symbol, ts = ts, seq = seq, spread = spread, top_bid = top_bid, top_ask = top_ask
    );
    for i in 0..depth {
        let bid = bids.get(i);
        let ask = asks.get(i);
        match (bid, ask) {
            (Some((bp, bq)), Some((ap, aq))) => {
                println!("  {bq:>12.4} @ {bp:<10.4}  |  {ap:>10.4} @ {aq:<12.4}",
                    bp = bp, bq = bq, ap = ap, aq = aq);
            }
            (Some((bp, bq)), None) => println!("  {bq:>12.4} @ {bp:<10.4}  |", bp = bp, bq = bq),
            (None, Some((ap, aq))) => println!("                              |  {ap:>10.4} @ {aq:<12.4}", ap = ap, aq = aq),
            (None, None) => break,
        }
    }
}

fn parse_account(s: &str) -> Result<AccountType> {
    Ok(match s.to_lowercase().as_str() {
        "spot" => AccountType::Spot,
        "margin" => AccountType::Margin,
        "futures_cross" | "cross" => AccountType::FuturesCross,
        "futures_isolated" | "isolated" => AccountType::FuturesIsolated,
        "earn" => AccountType::Earn,
        "options" => AccountType::Options,
        other => return Err(anyhow!("unknown account type `{other}`")),
    })
}
