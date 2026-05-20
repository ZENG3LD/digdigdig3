use std::path::PathBuf;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use digdigdig3_station::{
    AccountType, Event, ExchangeId, PersistenceConfig, Station, Stream, SubscriptionSet,
};

#[derive(Parser, Debug)]
#[command(name = "dig3", version, about = "digdigdig3 unified CLI")]
struct Cli {
    /// Root directory for Station-managed artefacts (trades / bars / snapshots).
    /// Resolves: --storage-root > DIG3_STORAGE_ROOT > ./dig3_storage
    #[arg(long, global = true)]
    storage_root: Option<PathBuf>,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Subscribe and print live events (trades / orderbook / ...).
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
    Trades {
        exchange: String,
        symbol: String,
        #[arg(long, default_value = "spot")]
        account: String,
        #[arg(long)]
        duration: Option<u64>,
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        persist: bool,
    },
    /// Live L2 orderbook ladder (top-N levels, refreshed in place).
    Orderbook {
        exchange: String,
        symbol: String,
        #[arg(long, default_value = "spot")]
        account: String,
        #[arg(long)]
        duration: Option<u64>,
        /// Number of levels per side to print on each update.
        #[arg(long, default_value_t = 10)]
        depth: usize,
    },
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

    match cli.cmd {
        Cmd::Watch { what } => match what {
            WatchKind::Trades { exchange, symbol, account, duration, persist } => {
                run_watch_trades(&exchange, &symbol, &account, duration, persist, cli.storage_root)
                    .await?;
            }
            WatchKind::Orderbook { exchange, symbol, account, duration, depth } => {
                run_watch_orderbook(&exchange, &symbol, &account, duration, depth, cli.storage_root)
                    .await?;
            }
        },
        Cmd::Persist => println!("dig3 persist: not yet implemented"),
        Cmd::Replay => println!("dig3 replay: not yet implemented"),
        Cmd::Matrix => println!(
            "dig3 matrix: not yet implemented (use `cargo run --example e2e_smoke -p digdigdig3 --release`)"
        ),
        Cmd::Inspect => println!("dig3 inspect: not yet implemented"),
        Cmd::Capture => println!("dig3 capture: not yet implemented"),
        Cmd::Benchmark => println!("dig3 benchmark: not yet implemented"),
    }

    Ok(())
}

async fn run_watch_trades(
    exchange: &str,
    symbol: &str,
    account: &str,
    duration: Option<u64>,
    persist: bool,
    storage_root_override: Option<PathBuf>,
) -> Result<()> {
    let exch = ExchangeId::from_str(exchange)
        .ok_or_else(|| anyhow!("unknown exchange `{exchange}`"))?;
    let acct = parse_account(account)?;

    let mut builder = Station::builder();
    if let Some(root) = storage_root_override {
        builder = builder.storage_root(root);
    }
    if persist {
        builder = builder.persistence(PersistenceConfig::on());
    }
    let station = builder.build().await.context("Station::build")?;

    let set = SubscriptionSet::new().add(exch, symbol, acct, [Stream::Trade]);

    let mut handle = station
        .subscribe(set)
        .await
        .context("Station::subscribe")?;

    eprintln!(
        "dig3 watch trades: exchange={exchange} symbol={symbol} account={account} duration={duration:?} persist={persist} storage_root={}",
        station.storage_root().display()
    );

    let deadline = duration.map(|d| tokio::time::Instant::now() + Duration::from_secs(d));

    loop {
        let event = match deadline {
            Some(at) => tokio::select! {
                _ = tokio::time::sleep_until(at) => break,
                ev = handle.recv() => ev,
            },
            None => handle.recv().await,
        };

        let Some(event) = event else { break };
        match event {
            Event::Trade { exchange, symbol, price, quantity, side, timestamp } => {
                println!(
                    "{ts} {ex:?} {sym} {side:>4} px={px} qty={qty}",
                    ts = timestamp,
                    ex = exchange,
                    sym = symbol,
                    side = side,
                    px = price,
                    qty = quantity,
                );
            }
            Event::OrderbookSnapshot { .. } => {} // ignored on trades watch
        }
    }

    Ok(())
}

async fn run_watch_orderbook(
    exchange: &str,
    symbol: &str,
    account: &str,
    duration: Option<u64>,
    depth: usize,
    storage_root_override: Option<PathBuf>,
) -> Result<()> {
    let exch = ExchangeId::from_str(exchange)
        .ok_or_else(|| anyhow!("unknown exchange `{exchange}`"))?;
    let acct = parse_account(account)?;

    let mut builder = Station::builder();
    if let Some(root) = storage_root_override {
        builder = builder.storage_root(root);
    }
    let station = builder.build().await.context("Station::build")?;

    let set = SubscriptionSet::new().add(exch, symbol, acct, [Stream::Orderbook]);

    let mut handle = station
        .subscribe(set)
        .await
        .context("Station::subscribe")?;

    eprintln!(
        "dig3 watch orderbook: exchange={exchange} symbol={symbol} account={account} depth={depth} duration={duration:?}"
    );

    let deadline = duration.map(|d| tokio::time::Instant::now() + Duration::from_secs(d));
    let mut prints = 0u64;

    loop {
        let event = match deadline {
            Some(at) => tokio::select! {
                _ = tokio::time::sleep_until(at) => break,
                ev = handle.recv() => ev,
            },
            None => handle.recv().await,
        };
        let Some(event) = event else { break };
        if let Event::OrderbookSnapshot { exchange, symbol, bids, asks, timestamp } = event {
            print_ladder(exchange, &symbol, timestamp, &bids, &asks, depth, prints);
            prints += 1;
        }
    }

    Ok(())
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
        ex = exchange,
        sym = symbol,
        ts = ts,
        seq = seq,
        spread = spread,
        top_bid = top_bid,
        top_ask = top_ask,
    );
    let n = depth;
    for i in 0..n {
        let bid = bids.get(i);
        let ask = asks.get(i);
        match (bid, ask) {
            (Some((bp, bq)), Some((ap, aq))) => {
                println!("  {bq:>12.4} @ {bp:<10.2}  |  {ap:>10.2} @ {aq:<12.4}",
                    bp = bp, bq = bq, ap = ap, aq = aq);
            }
            (Some((bp, bq)), None) => {
                println!("  {bq:>12.4} @ {bp:<10.2}  |", bp = bp, bq = bq);
            }
            (None, Some((ap, aq))) => {
                println!("                              |  {ap:>10.2} @ {aq:<12.4}", ap = ap, aq = aq);
            }
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
