//! Trade persistence — binary append-only with sparse index.
//!
//! Layout under `<storage_root>/trades/<exchange>/<account>/<symbol>/<YYYY-MM-DD>.dat`:
//! - `.dat`: 41 bytes per trade (LE)
//!     - `u64 ts_ms` (8) — exchange timestamp in ms
//!     - `f64 price` (8)
//!     - `f64 quantity` (8)
//!     - `u8  side` (1) — 0 Buy, 1 Sell, 2 Unknown
//!     - `u64 trade_id_hash` (8) — fnv1a-64 of the original trade-id string
//!     - 8 bytes reserved (zero-filled) — held for sequence number / flags
//! - `.idx`: every 1024th record emits `[u64 ts_ms, u64 file_offset]` for
//!   fast scan to a timestamp during step 5+ replay.
//!
//! UTC day rollover reopens both files.

use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use digdigdig3_core::core::types::TradeSide;
use serde::{Deserialize, Serialize};

const RECORD_SIZE: usize = 41;
const SPARSE_IDX_EVERY: u32 = 1024;

/// Builder-side configuration for trade persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceConfig {
    pub enabled: bool,
    pub trades: bool,
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        // Off by default — `dig3 watch` opts in explicitly.
        Self { enabled: false, trades: false }
    }
}

impl PersistenceConfig {
    /// Turn on with default sub-toggles (trades on).
    pub fn on() -> Self {
        Self { enabled: true, trades: true }
    }

    pub fn trades(mut self, on: bool) -> Self {
        self.trades = on;
        self
    }
}

/// Binary append-only writer for a single (exchange, account, symbol) stream.
pub struct TradeWriter {
    root: PathBuf,
    exchange: String,
    account: String,
    symbol: String,
    current_day: String, // YYYY-MM-DD UTC
    dat: BufWriter<File>,
    idx: BufWriter<File>,
    records: u32,
    file_offset: u64,
}

impl std::fmt::Debug for TradeWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TradeWriter")
            .field("root", &self.root)
            .field("exchange", &self.exchange)
            .field("account", &self.account)
            .field("symbol", &self.symbol)
            .field("current_day", &self.current_day)
            .field("records", &self.records)
            .field("file_offset", &self.file_offset)
            .finish()
    }
}

impl TradeWriter {
    pub fn new(
        storage_root: &Path,
        exchange: &str,
        account: &str,
        symbol: &str,
    ) -> std::io::Result<Self> {
        let exchange = exchange.to_lowercase();
        let account = account.to_lowercase();
        let symbol = symbol.to_lowercase();
        let day = utc_today();
        let (dat_path, idx_path) = paths(storage_root, &exchange, &account, &symbol, &day);
        let (dat, idx, offset) = open_pair(&dat_path, &idx_path)?;
        Ok(Self {
            root: storage_root.to_path_buf(),
            exchange,
            account,
            symbol,
            current_day: day,
            dat: BufWriter::new(dat),
            idx: BufWriter::new(idx),
            records: 0,
            file_offset: offset,
        })
    }

    pub fn append(
        &mut self,
        ts_ms: i64,
        price: f64,
        quantity: f64,
        side: TradeSide,
        trade_id: &str,
    ) -> std::io::Result<()> {
        self.rotate_if_new_day()?;

        let mut buf = [0u8; RECORD_SIZE];
        buf[0..8].copy_from_slice(&(ts_ms as u64).to_le_bytes());
        buf[8..16].copy_from_slice(&price.to_le_bytes());
        buf[16..24].copy_from_slice(&quantity.to_le_bytes());
        buf[24] = match side {
            TradeSide::Buy => 0,
            TradeSide::Sell => 1,
        };
        buf[25..33].copy_from_slice(&fnv1a_64(trade_id.as_bytes()).to_le_bytes());
        // bytes 33..41 reserved (already zero)
        self.dat.write_all(&buf)?;

        if self.records % SPARSE_IDX_EVERY == 0 {
            let mut idx_buf = [0u8; 16];
            idx_buf[0..8].copy_from_slice(&(ts_ms as u64).to_le_bytes());
            idx_buf[8..16].copy_from_slice(&self.file_offset.to_le_bytes());
            self.idx.write_all(&idx_buf)?;
        }

        self.records = self.records.wrapping_add(1);
        self.file_offset += RECORD_SIZE as u64;
        Ok(())
    }

    pub fn flush(&mut self) -> std::io::Result<()> {
        self.dat.flush()?;
        self.idx.flush()?;
        Ok(())
    }

    fn rotate_if_new_day(&mut self) -> std::io::Result<()> {
        let today = utc_today();
        if today == self.current_day {
            return Ok(());
        }
        self.flush()?;
        let (dat_path, idx_path) =
            paths(&self.root, &self.exchange, &self.account, &self.symbol, &today);
        let (dat, idx, offset) = open_pair(&dat_path, &idx_path)?;
        self.dat = BufWriter::new(dat);
        self.idx = BufWriter::new(idx);
        self.records = 0;
        self.file_offset = offset;
        self.current_day = today;
        Ok(())
    }
}

impl Drop for TradeWriter {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

fn paths(
    root: &Path,
    exchange: &str,
    account: &str,
    symbol: &str,
    day: &str,
) -> (PathBuf, PathBuf) {
    let dir = root.join("trades").join(exchange).join(account).join(symbol);
    let dat = dir.join(format!("{day}.dat"));
    let idx = dir.join(format!("{day}.idx"));
    (dat, idx)
}

fn open_pair(dat_path: &Path, idx_path: &Path) -> std::io::Result<(File, File, u64)> {
    if let Some(parent) = dat_path.parent() {
        create_dir_all(parent)?;
    }
    let dat = OpenOptions::new().create(true).append(true).open(dat_path)?;
    let offset = dat.metadata()?.len();
    let idx = OpenOptions::new().create(true).append(true).open(idx_path)?;
    Ok((dat, idx, offset))
}

fn utc_today() -> String {
    use chrono::Utc;
    Utc::now().format("%Y-%m-%d").to_string()
}

fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::read;

    #[test]
    fn writer_round_trip() {
        let tmp = std::env::temp_dir().join(format!("dig3-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);

        {
            let mut w = TradeWriter::new(&tmp, "binance", "spot", "btcusdt").unwrap();
            for i in 0..3 {
                w.append(1700000000000 + i, 70000.0 + i as f64, 0.1, TradeSide::Buy, "x").unwrap();
            }
            w.flush().unwrap();
        }

        let day = utc_today();
        let dat = tmp.join("trades/binance/spot/btcusdt").join(format!("{day}.dat"));
        let bytes = read(&dat).unwrap();
        assert_eq!(bytes.len(), 3 * RECORD_SIZE);

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
