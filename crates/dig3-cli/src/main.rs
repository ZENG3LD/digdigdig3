use clap::{Parser, Subcommand};

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
    /// Run persistence daemon.
    Persist,
    /// Replay persisted history.
    Replay,
    /// Run the e2e coverage matrix.
    Matrix,
    /// Inspect symbols / capabilities / channels.
    Inspect,
    /// Capture raw frames to disk.
    Capture,
    /// Run microbenchmarks.
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
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match cli.cmd {
        Cmd::Watch { what } => match what {
            WatchKind::Trades { exchange, symbol, account, duration } => {
                println!(
                    "dig3 watch trades: exchange={exchange} symbol={symbol} account={account} duration={duration:?}"
                );
                println!("(phase 1 skeleton — Station::subscribe + persistence wiring lands in step 6)");
            }
        },
        Cmd::Persist => println!("dig3 persist: not yet implemented in Phase 1"),
        Cmd::Replay => println!("dig3 replay: not yet implemented in Phase 1"),
        Cmd::Matrix => println!("dig3 matrix: not yet implemented in Phase 1 (use cargo run --example e2e_smoke -p digdigdig3-core)"),
        Cmd::Inspect => println!("dig3 inspect: not yet implemented in Phase 1"),
        Cmd::Capture => println!("dig3 capture: not yet implemented in Phase 1"),
        Cmd::Benchmark => println!("dig3 benchmark: not yet implemented in Phase 1"),
    }

    Ok(())
}
