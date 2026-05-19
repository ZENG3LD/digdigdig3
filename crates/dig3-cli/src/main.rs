use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use digdigdig3_station::{
    AccountType, Event, ExchangeId, Station, Stream, SubscriptionSet,
};

#[derive(Parser, Debug)]
#[command(name = "dig3", version, about = "digdigdig3 unified CLI")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Subscribe and print live events (trades / kline / orderbook / ...).
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
        /// Exchange name (e.g. binance, bybit, okx, kraken). Case-insensitive.
        exchange: String,
        /// Symbol — accepts "BTC-USDT", "BTC/USDT", "BTC_USDT", or "BTCUSDT".
        symbol: String,
        /// Account scope: spot | margin | futures_cross | futures_isolated.
        #[arg(long, default_value = "spot")]
        account: String,
        /// Stop after N seconds (omit to run until Ctrl-C).
        #[arg(long)]
        duration: Option<u64>,
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
            WatchKind::Trades { exchange, symbol, account, duration } => {
                run_watch_trades(&exchange, &symbol, &account, duration).await?;
            }
        },
        Cmd::Persist => println!("dig3 persist: not yet implemented in Phase 1"),
        Cmd::Replay => println!("dig3 replay: not yet implemented in Phase 1"),
        Cmd::Matrix => println!(
            "dig3 matrix: not yet implemented in Phase 1 (use `cargo run --example e2e_smoke -p digdigdig3-core --release`)"
        ),
        Cmd::Inspect => println!("dig3 inspect: not yet implemented in Phase 1"),
        Cmd::Capture => println!("dig3 capture: not yet implemented in Phase 1"),
        Cmd::Benchmark => println!("dig3 benchmark: not yet implemented in Phase 1"),
    }

    Ok(())
}

async fn run_watch_trades(
    exchange: &str,
    symbol: &str,
    account: &str,
    duration: Option<u64>,
) -> Result<()> {
    let exch = ExchangeId::from_str(exchange)
        .ok_or_else(|| anyhow!("unknown exchange `{exchange}`"))?;
    let acct = parse_account(account)?;

    let station = Station::builder().build().await.context("Station::build")?;

    let set = SubscriptionSet::new().add(exch, symbol, acct, [Stream::Trade]);

    let mut handle = station
        .subscribe(set)
        .await
        .context("Station::subscribe")?;

    eprintln!(
        "dig3 watch trades: exchange={exchange} symbol={symbol} account={account} duration={duration:?}"
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
        }
    }

    Ok(())
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
