//! dig3-catcher — long-running multi-stream subscriber daemon.
//!
//! Subscribes to N exchanges × M streams concurrently via async tasks.
//! Each task is tracked by a JobId and maintains per-stream counters with
//! a 60-second sliding window. Exposes HTTP introspection endpoints and
//! prints periodic console reports.
//!
//! Usage:
//!   dig3-catcher [--config PATH] [--port PORT] [--duration SECS] [--report-every SECS]

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{error, info, warn};

use digdigdig3_core::connector_manager::ExchangeHub;
use digdigdig3_core::core::types::{
    AccountType, ExchangeId, StreamType, SubscriptionRequest, Symbol,
};
use digdigdig3_station::storage::{StorageConfig, StorageManager, StreamKey};

// ── JobId ─────────────────────────────────────────────────────────────────────

static JOB_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Unique identifier for one (exchange, account, stream, symbol) subscription task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JobId(pub u64);

impl JobId {
    fn next() -> Self {
        Self(JOB_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

// ── Sliding window ────────────────────────────────────────────────────────────

/// 60-second sliding window counter.
struct SlidingWindow {
    instants: VecDeque<Instant>,
}

impl SlidingWindow {
    fn new() -> Self {
        Self {
            instants: VecDeque::new(),
        }
    }

    fn push(&mut self) {
        self.instants.push_back(Instant::now());
    }

    fn count_last_minute(&mut self) -> u64 {
        let cutoff = Instant::now() - Duration::from_secs(60);
        while self.instants.front().map(|t| *t < cutoff).unwrap_or(false) {
            self.instants.pop_front();
        }
        self.instants.len() as u64
    }
}

// ── JobStats ──────────────────────────────────────────────────────────────────

pub struct JobStats {
    pub job_id: JobId,
    pub label: String,
    pub exchange: ExchangeId,
    pub account: AccountType,
    pub stream: StreamType,
    pub symbol: String,
    pub events_total: u64,
    pub last_event_at: Option<Instant>,
    pub subscribe_at: Instant,
    /// Ring-buffer of last 10 error messages.
    pub errors: VecDeque<String>,
    window: SlidingWindow,
}

impl JobStats {
    fn new(
        job_id: JobId,
        exchange: ExchangeId,
        account: AccountType,
        stream: StreamType,
        symbol: String,
    ) -> Self {
        let label = format!(
            "{}:{}:{:?}:{}",
            exchange.as_str(),
            account_str(account),
            stream,
            symbol
        );
        Self {
            job_id,
            label,
            exchange,
            account,
            stream,
            symbol,
            events_total: 0,
            last_event_at: None,
            subscribe_at: Instant::now(),
            errors: VecDeque::with_capacity(10),
            window: SlidingWindow::new(),
        }
    }

    fn record_event(&mut self) {
        self.events_total += 1;
        self.last_event_at = Some(Instant::now());
        self.window.push();
    }

    fn events_last_minute(&mut self) -> u64 {
        self.window.count_last_minute()
    }

    fn push_error(&mut self, msg: String) {
        if self.errors.len() >= 10 {
            self.errors.pop_front();
        }
        self.errors.push_back(msg);
    }
}

fn account_str(a: AccountType) -> &'static str {
    match a {
        AccountType::Spot => "spot",
        AccountType::Margin => "margin",
        AccountType::FuturesCross => "futures_cross",
        AccountType::FuturesIsolated => "futures_isolated",
        AccountType::Earn => "earn",
        AccountType::Lending => "lending",
        AccountType::Options => "options",
        AccountType::Convert => "convert",
    }
}

// ── CatcherState ──────────────────────────────────────────────────────────────

pub struct CatcherState {
    pub jobs: RwLock<HashMap<JobId, Arc<RwLock<JobStats>>>>,
    pub started_at: Instant,
}

impl CatcherState {
    fn new() -> Self {
        Self {
            jobs: RwLock::new(HashMap::new()),
            started_at: Instant::now(),
        }
    }
}

// ── Config ────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Default)]
struct StorageCatcherConfig {
    enabled: bool,
    root: Option<String>,
    #[serde(default = "default_retention_days")]
    retention_days: u32,
}

fn default_retention_days() -> u32 {
    30
}

#[derive(Debug, Deserialize)]
struct Config {
    #[serde(rename = "target")]
    targets: Vec<TargetConfig>,
    #[serde(default)]
    storage: StorageCatcherConfig,
}

#[derive(Debug, Clone, Deserialize)]
struct TargetConfig {
    exchange: String,
    account: String,
    symbol: String,
    streams: Vec<String>,
}

fn default_config() -> Config {
    let targets = vec![
        TargetConfig {
            exchange: "Binance".into(),
            account: "Spot".into(),
            symbol: "BTCUSDT".into(),
            streams: vec!["Trade".into(), "Ticker".into()],
        },
        TargetConfig {
            exchange: "Bybit".into(),
            account: "Spot".into(),
            symbol: "BTCUSDT".into(),
            streams: vec!["Trade".into(), "Ticker".into()],
        },
        TargetConfig {
            exchange: "OKX".into(),
            account: "Spot".into(),
            symbol: "BTC-USDT".into(),
            streams: vec!["Trade".into(), "Ticker".into()],
        },
        TargetConfig {
            exchange: "KuCoin".into(),
            account: "Spot".into(),
            symbol: "BTC-USDT".into(),
            streams: vec!["Trade".into(), "Ticker".into()],
        },
        TargetConfig {
            exchange: "Bitget".into(),
            account: "Spot".into(),
            symbol: "BTCUSDT".into(),
            streams: vec!["Trade".into(), "Ticker".into()],
        },
    ];
    Config { targets, storage: StorageCatcherConfig::default() }
}

fn parse_exchange(s: &str) -> Option<ExchangeId> {
    ExchangeId::from_str(&s.to_lowercase())
        .or_else(|| ExchangeId::from_str(s))
}

fn parse_account(s: &str) -> AccountType {
    match s.to_lowercase().as_str() {
        "spot" => AccountType::Spot,
        "margin" => AccountType::Margin,
        "futures" | "futures_cross" | "futurescross" => AccountType::FuturesCross,
        "futures_isolated" | "futuresiso" => AccountType::FuturesIsolated,
        "earn" => AccountType::Earn,
        "lending" => AccountType::Lending,
        "options" => AccountType::Options,
        "convert" => AccountType::Convert,
        _ => AccountType::Spot,
    }
}

fn parse_stream(s: &str) -> Option<StreamType> {
    match s.to_lowercase().as_str() {
        "ticker" => Some(StreamType::Ticker),
        "trade" | "trades" => Some(StreamType::Trade),
        "orderbook" | "book" => Some(StreamType::Orderbook),
        "orderbookdelta" | "bookdelta" => Some(StreamType::OrderbookDelta),
        "markprice" => Some(StreamType::MarkPrice),
        "fundingrate" | "funding" => Some(StreamType::FundingRate),
        "liquidation" => Some(StreamType::Liquidation),
        "openinterest" | "oi" => Some(StreamType::OpenInterest),
        "aggtrade" => Some(StreamType::AggTrade),
        _ => None,
    }
}

// ── CLI args ──────────────────────────────────────────────────────────────────

struct Args {
    config: Option<String>,
    port: u16,
    duration: Option<u64>,
    report_every: u64,
    /// Optional path to storage root dir (overrides `[storage]` in config).
    storage_dir: Option<String>,
}

fn parse_args() -> Args {
    let args: Vec<String> = std::env::args().collect();
    let mut config = None;
    let mut port = 18250u16;
    let mut duration = None;
    let mut report_every = 60u64;
    let mut storage_dir = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--config" if i + 1 < args.len() => {
                config = Some(args[i + 1].clone());
                i += 2;
            }
            "--port" if i + 1 < args.len() => {
                port = args[i + 1].parse().unwrap_or(18250);
                i += 2;
            }
            "--duration" if i + 1 < args.len() => {
                duration = args[i + 1].parse().ok();
                i += 2;
            }
            "--report-every" if i + 1 < args.len() => {
                report_every = args[i + 1].parse().unwrap_or(60);
                i += 2;
            }
            "--storage-dir" if i + 1 < args.len() => {
                storage_dir = Some(args[i + 1].clone());
                i += 2;
            }
            _ => {
                i += 1;
            }
        }
    }

    Args {
        config,
        port,
        duration,
        report_every,
        storage_dir,
    }
}

// ── HTTP server (hand-rolled) ─────────────────────────────────────────────────

/// Stats snapshot for JSON serialization (no Instant in JSON).
#[derive(Serialize)]
struct JobSnapshot {
    job_id: u64,
    label: String,
    exchange: String,
    account: String,
    symbol: String,
    events_total: u64,
    events_last_minute: u64,
    last_event_secs_ago: Option<u64>,
    uptime_secs: u64,
    errors: Vec<String>,
}

async fn collect_snapshots(state: &CatcherState) -> Vec<JobSnapshot> {
    let jobs = state.jobs.read().await;
    let mut out = Vec::with_capacity(jobs.len());
    for stats_lock in jobs.values() {
        let mut s = stats_lock.write().await;
        let last_event_secs_ago = s
            .last_event_at
            .map(|t| t.elapsed().as_secs());
        let elast_min = s.events_last_minute();
        out.push(JobSnapshot {
            job_id: s.job_id.0,
            label: s.label.clone(),
            exchange: s.exchange.as_str().to_string(),
            account: account_str(s.account).to_string(),
            symbol: s.symbol.clone(),
            events_total: s.events_total,
            events_last_minute: elast_min,
            last_event_secs_ago,
            uptime_secs: s.subscribe_at.elapsed().as_secs(),
            errors: s.errors.iter().cloned().collect(),
        });
    }
    out.sort_by_key(|s| s.job_id);
    out
}

async fn serve_http(port: u16, state: Arc<CatcherState>) {
    let addr = format!("127.0.0.1:{port}");
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => {
            info!("HTTP health on http://{addr}");
            l
        }
        Err(e) => {
            error!("HTTP bind failed on {addr}: {e}");
            return;
        }
    };

    loop {
        let Ok((stream, _peer)) = listener.accept().await else {
            continue;
        };
        let state = state.clone();
        tokio::spawn(async move {
            handle_http(stream, state).await;
        });
    }
}

async fn handle_http(stream: tokio::net::TcpStream, state: Arc<CatcherState>) {
    let (read_half, mut write_half) = stream.into_split();
    let mut reader = BufReader::new(read_half);
    let mut first_line = String::new();
    if reader.read_line(&mut first_line).await.is_err() {
        return;
    }

    // Parse "GET /path HTTP/1.1"
    let path = first_line
        .split_whitespace()
        .nth(1)
        .unwrap_or("/")
        .to_string();

    // Drain request headers (required before writing response)
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line).await {
            Ok(0) | Err(_) => break,
            Ok(_) if line.trim().is_empty() => break,
            _ => {}
        }
    }

    let (status, body) = match path.as_str() {
        "/health" => {
            let uptime = state.started_at.elapsed().as_secs();
            let job_count = state.jobs.read().await.len();
            let body = format!(
                r#"{{"status":"ok","uptime_secs":{uptime},"job_count":{job_count}}}"#
            );
            ("200 OK", body)
        }
        "/stats" => {
            let snaps = collect_snapshots(&state).await;
            let body = serde_json::to_string(&snaps).unwrap_or_else(|_| "[]".into());
            ("200 OK", body)
        }
        "/silent" => {
            let snaps = collect_snapshots(&state).await;
            let silent: Vec<_> = snaps
                .into_iter()
                .filter(|s| s.events_last_minute == 0)
                .collect();
            let body = serde_json::to_string(&silent).unwrap_or_else(|_| "[]".into());
            ("200 OK", body)
        }
        "/errors" => {
            let snaps = collect_snapshots(&state).await;
            let errored: Vec<_> = snaps
                .into_iter()
                .filter(|s| !s.errors.is_empty())
                .collect();
            let body = serde_json::to_string(&errored).unwrap_or_else(|_| "[]".into());
            ("200 OK", body)
        }
        _ => ("404 Not Found", r#"{"error":"not found"}"#.to_string()),
    };

    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = write_half.write_all(response.as_bytes()).await;
}

// ── Periodic report ───────────────────────────────────────────────────────────

async fn print_report(state: &CatcherState) {
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ");
    let uptime = state.started_at.elapsed().as_secs();
    let snaps = collect_snapshots(state).await;
    let job_count = snaps.len();
    let silent_count = snaps.iter().filter(|s| s.events_last_minute == 0).count();
    let active = job_count - silent_count;

    eprintln!(
        "[catcher {now}] uptime={uptime}s jobs={job_count} active={active}"
    );
    for s in &snaps {
        let status = if s.events_last_minute == 0 {
            "SILENT"
        } else {
            "OK"
        };
        eprintln!(
            "  {:<48} #{:<3} total={:<8} last_min={:<6} {}",
            s.label, s.job_id, s.events_total, s.events_last_minute, status
        );
    }
    for s in snaps.iter().filter(|s| s.events_last_minute == 0) {
        eprintln!(
            "SILENT: {} #{} (0 events last min)",
            s.label, s.job_id
        );
    }
    for s in snaps.iter().filter(|s| !s.errors.is_empty()) {
        let last = s.errors.last().map(|e| e.as_str()).unwrap_or("");
        eprintln!("ERROR:  {} #{} — {}", s.label, s.job_id, last);
    }
}

// ── Subscriber loop ───────────────────────────────────────────────────────────

/// Returns the stream_kind string used as the filename stem in EventLog.
fn stream_kind_str(stream: &StreamType) -> String {
    match stream {
        StreamType::Ticker => "ticker".into(),
        StreamType::Trade => "trade".into(),
        StreamType::Orderbook => "orderbook".into(),
        StreamType::OrderbookDelta => "orderbook_delta".into(),
        StreamType::Kline { interval } => format!("kline_{interval}"),
        StreamType::MarkPrice => "mark_price".into(),
        StreamType::FundingRate => "funding_rate".into(),
        StreamType::Liquidation => "liquidation".into(),
        StreamType::OpenInterest => "open_interest".into(),
        StreamType::AggTrade => "agg_trade".into(),
        other => format!("{other:?}").to_lowercase(),
    }
}

async fn run_subscriber_loop(
    hub: Arc<ExchangeHub>,
    exchange: ExchangeId,
    account: AccountType,
    stream: StreamType,
    symbol: String,
    stats: Arc<RwLock<JobStats>>,
    storage: Option<Arc<StorageManager>>,
) {
    let kind_str = stream_kind_str(&stream);

    // Exponential backoff caps at 30s
    let mut backoff = Duration::from_secs(1);

    loop {
        // Connect (idempotent — hub skips if already connected)
        if let Err(e) = hub.connect_full(exchange, &[account], false).await {
            let msg = format!("connect: {e}");
            warn!(%msg, exchange = exchange.as_str());
            stats.write().await.push_error(msg);
            tokio::time::sleep(backoff).await;
            backoff = (backoff * 2).min(Duration::from_secs(30));
            continue;
        }

        let ws = match hub.ws(exchange, account) {
            Some(ws) => ws,
            None => {
                let msg = "no ws handle after connect".to_string();
                warn!(%msg, exchange = exchange.as_str());
                stats.write().await.push_error(msg);
                tokio::time::sleep(backoff).await;
                backoff = (backoff * 2).min(Duration::from_secs(30));
                continue;
            }
        };

        let sym = Symbol::with_raw("", "", symbol.clone());
        let req = SubscriptionRequest {
            symbol: sym,
            stream_type: stream.clone(),
            account_type: account,
            depth: None,
            update_speed_ms: None,
        };

        if let Err(e) = ws.subscribe(req).await {
            let msg = format!("subscribe: {e}");
            warn!(%msg, exchange = exchange.as_str());
            stats.write().await.push_error(msg);
            tokio::time::sleep(backoff).await;
            backoff = (backoff * 2).min(Duration::from_secs(30));
            continue;
        }

        info!(
            exchange = exchange.as_str(),
            stream = ?stream,
            symbol = %symbol,
            "subscribed"
        );

        // Reset backoff on successful connect
        backoff = Duration::from_secs(1);

        let mut event_rx = ws.event_stream();
        loop {
            match event_rx.next().await {
                Some(Ok(event)) => {
                    stats.write().await.record_event();

                    // Persist via StorageManager if enabled — failure is non-fatal
                    if let Some(mgr) = &storage {
                        let ts_ms = chrono::Utc::now().timestamp_millis();
                        match serde_json::to_vec(&event) {
                            Ok(payload) => {
                                let key = StreamKey {
                                    exchange: exchange.as_str().to_string(),
                                    account: account_str(account).to_string(),
                                    symbol: symbol.clone(),
                                    stream_kind: kind_str.clone(),
                                };
                                if let Err(e) = mgr.append(&key, ts_ms, &payload).await {
                                    warn!(
                                        exchange = exchange.as_str(),
                                        symbol = %symbol,
                                        kind = %kind_str,
                                        "storage write failed: {e}"
                                    );
                                }
                            }
                            Err(e) => {
                                warn!(
                                    exchange = exchange.as_str(),
                                    "event serialize failed: {e}"
                                );
                            }
                        }
                    }
                }
                Some(Err(e)) => {
                    let msg = format!("recv: {e}");
                    warn!(%msg, exchange = exchange.as_str());
                    stats.write().await.push_error(msg);
                    // Break inner loop → reconnect
                    break;
                }
                None => {
                    // Stream ended
                    let msg = "stream ended".to_string();
                    warn!(%msg, exchange = exchange.as_str());
                    stats.write().await.push_error(msg);
                    break;
                }
            }
        }

        // Reconnect after delay
        tokio::time::sleep(backoff).await;
        backoff = (backoff * 2).min(Duration::from_secs(30));
    }
}

// ── main ──────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_target(false)
        .init();

    let args = parse_args();

    // Load config
    let config = match &args.config {
        Some(path) => {
            let text = tokio::fs::read_to_string(path).await.map_err(|e| {
                format!("cannot read config {path}: {e}")
            })?;
            toml::from_str::<Config>(&text).map_err(|e| format!("config parse error: {e}"))?
        }
        None => {
            eprintln!("[catcher] no --config given, using built-in 5-exchange default");
            default_config()
        }
    };

    let hub = Arc::new(ExchangeHub::new());
    let state = Arc::new(CatcherState::new());

    // Resolve storage root: CLI flag wins over config file.
    let storage_root: Option<String> = args.storage_dir.clone().or_else(|| {
        if config.storage.enabled {
            config.storage.root.clone()
        } else {
            None
        }
    });

    let retention_days = config.storage.retention_days;

    let storage: Option<Arc<StorageManager>> = match storage_root {
        Some(ref root) => {
            let cfg = StorageConfig {
                root: root.into(),
                default_retention_days: retention_days,
                ..StorageConfig::default()
            };
            match StorageManager::new(cfg) {
                Ok(mgr) => {
                    eprintln!("[catcher] storage enabled: {root} (retention={retention_days}d)");
                    Some(Arc::new(mgr))
                }
                Err(e) => {
                    eprintln!("[catcher] storage init failed ({root}): {e} — continuing without storage");
                    None
                }
            }
        }
        None => None,
    };

    // Spawn HTTP health server
    {
        let state = state.clone();
        tokio::spawn(async move {
            serve_http(args.port, state).await;
        });
    }

    // Spawn one task per (exchange × account × stream × symbol)
    let mut handles = Vec::new();

    for target in &config.targets {
        let exchange = match parse_exchange(&target.exchange) {
            Some(e) => e,
            None => {
                eprintln!("[catcher] unknown exchange {:?}, skipping", target.exchange);
                continue;
            }
        };
        let account = parse_account(&target.account);

        for stream_str in &target.streams {
            let stream = match parse_stream(stream_str) {
                Some(s) => s,
                None => {
                    eprintln!(
                        "[catcher] unknown stream {:?} for {}, skipping",
                        stream_str, target.exchange
                    );
                    continue;
                }
            };

            let job_id = JobId::next();
            let job_stats = Arc::new(RwLock::new(JobStats::new(
                job_id,
                exchange,
                account,
                stream.clone(),
                target.symbol.clone(),
            )));

            {
                let mut jobs = state.jobs.write().await;
                jobs.insert(job_id, job_stats.clone());
            }

            let hub = hub.clone();
            let symbol = target.symbol.clone();
            let storage_clone = storage.clone();
            let handle = tokio::spawn(async move {
                run_subscriber_loop(hub, exchange, account, stream, symbol, job_stats, storage_clone).await;
            });
            handles.push(handle);
        }
    }

    eprintln!(
        "[catcher] started {} jobs, HTTP on http://127.0.0.1:{}",
        handles.len(),
        args.port
    );

    // Periodic reporter
    {
        let state = state.clone();
        let report_every = args.report_every;
        tokio::spawn(async move {
            let mut tick = interval(Duration::from_secs(report_every));
            loop {
                tick.tick().await;
                print_report(&state).await;
            }
        });
    }

    // Periodic flush task — every 60 s, ensures BufWriters are flushed.
    if let Some(mgr) = storage.clone() {
        tokio::spawn(async move {
            let mut tick = interval(Duration::from_secs(60));
            loop {
                tick.tick().await;
                if let Err(e) = mgr.flush_all().await {
                    warn!("storage flush error: {e}");
                }
            }
        });
    }

    // Daily retention sweep — every 24 h, deletes files older than retention_days.
    if let Some(mgr) = storage.clone() {
        tokio::spawn(async move {
            let mut tick = interval(Duration::from_secs(86_400));
            // Skip first immediate tick.
            tick.tick().await;
            loop {
                tick.tick().await;
                match mgr.cleanup(chrono::Utc::now()) {
                    Ok(n) if n > 0 => info!("retention sweep deleted {n} files"),
                    Ok(_) => {}
                    Err(e) => warn!("retention sweep error: {e}"),
                }
            }
        });
    }

    // Wait for shutdown or duration
    if let Some(secs) = args.duration {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                eprintln!("[catcher] SIGINT received, shutting down");
            }
            _ = tokio::time::sleep(Duration::from_secs(secs)) => {
                eprintln!("[catcher] duration {secs}s reached, exiting");
            }
        }
    } else {
        tokio::signal::ctrl_c().await.ok();
        eprintln!("[catcher] SIGINT received, shutting down");
    }

    // Abort all tasks
    for h in &handles {
        h.abort();
    }

    // Final summary
    eprintln!("[catcher] final summary:");
    print_report(&state).await;

    Ok(())
}
