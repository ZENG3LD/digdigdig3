//! Native day-file fetch: HTTP GET → decompress (gzip/zip) → CSV parse.
//!
//! `cfg(not(target_arch = "wasm32"))` — wasm fetch goes through the CORS
//! proxy + browser `DecompressionStream`, a separate (later) code path.
//! No retention: bytes are decoded, parsed into rows, and discarded —
//! nothing here writes to disk or keeps the day's bytes beyond this call.

use std::io::{BufRead, BufReader, Cursor, Read};
use std::time::Duration;

use chrono::NaiveDate;
use digdigdig3::core::types::{ArchiveFormat, ArchiveSchema, ArchiveSpec};
use flate2::read::GzDecoder;
use thiserror::Error;

use super::parse::{looks_like_header, parse_binance_agg_trades_row, parse_bybit_spot_row, parse_bybit_trading_row, RawTrade};
use super::url::url_for_day;

const FETCH_TIMEOUT: Duration = Duration::from_secs(300);

/// Failure fetching/decoding one archive day file. `NotFound` is a
/// DISTINCT variant from other failures — boundary detection (has the
/// archive genuinely ended, or is this a transient/data-gap failure) is
/// the caller's job, not this fn's; this fn just reports which case it saw.
#[derive(Debug, Error)]
pub enum ArchiveDayError {
    /// Day file does not exist upstream (HTTP 404). Caller decides whether
    /// this means "boundary reached" (date <= since_ms) or "data gap
    /// mid-range" (skip and continue).
    #[error("archive day file not found: {0}")]
    NotFound(String),
    /// Upstream returned a non-404 error status.
    #[error("archive day fetch failed with status {status}: {url}")]
    HttpStatus { status: u16, url: String },
    /// Transport-level failure (connect/timeout/DNS/TLS).
    #[error("archive day fetch transport error: {0}")]
    Transport(String),
    /// Response body could not be decompressed (corrupt gzip/zip, or a zip
    /// that isn't the expected single-CSV-entry shape).
    #[error("archive day decode error: {0}")]
    Decode(String),
}

impl From<reqwest::Error> for ArchiveDayError {
    fn from(e: reqwest::Error) -> Self {
        ArchiveDayError::Transport(e.to_string())
    }
}

/// Fetch, decompress, and parse one archive day file for `symbol` on
/// `date`. Returns rows oldest-to-newest as they appear in the source file
/// (no re-sorting — archive day files are already chronological on both
/// verified venues).
pub async fn fetch_archive_day(
    spec: &ArchiveSpec,
    symbol: &str,
    date: NaiveDate,
) -> Result<Vec<RawTrade>, ArchiveDayError> {
    let url = url_for_day(spec, symbol, date);

    let client = reqwest::Client::builder()
        .timeout(FETCH_TIMEOUT)
        .build()
        .map_err(|e| ArchiveDayError::Transport(e.to_string()))?;

    let resp = client.get(&url).send().await?;

    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(ArchiveDayError::NotFound(url));
    }
    if !resp.status().is_success() {
        return Err(ArchiveDayError::HttpStatus { status: resp.status().as_u16(), url });
    }

    let bytes = resp.bytes().await?;

    let lines: Vec<String> = match spec.format {
        ArchiveFormat::CsvGz => decode_gz_lines(&bytes).map_err(|e| ArchiveDayError::Decode(e.to_string()))?,
        ArchiveFormat::ZipSingleCsv => {
            decode_zip_single_csv_lines(&bytes).map_err(|e| ArchiveDayError::Decode(e.to_string()))?
        }
    };

    Ok(parse_lines(spec.schema, &lines))
}

/// Gunzip the whole body (bounded by one day's uncompressed size, up to
/// ~100MB per the plan's memory note — acceptable to materialize as lines
/// for v1; a streaming line-by-line variant can replace this without
/// changing the return shape if a venue's day file grows past what's
/// comfortable in memory).
fn decode_gz_lines(bytes: &[u8]) -> std::io::Result<Vec<String>> {
    let decoder = GzDecoder::new(Cursor::new(bytes));
    let reader = BufReader::new(decoder);
    reader.lines().collect()
}

/// Extract the single CSV entry from a zip archive and split it into
/// lines. Errors if the zip does not contain exactly one entry (the
/// verified Binance archive shape).
fn decode_zip_single_csv_lines(bytes: &[u8]) -> Result<Vec<String>, String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| e.to_string())?;
    if archive.len() != 1 {
        return Err(format!("expected exactly 1 zip entry, found {}", archive.len()));
    }
    let mut entry = archive.by_index(0).map_err(|e| e.to_string())?;
    let mut content = String::new();
    entry.read_to_string(&mut content).map_err(|e| e.to_string())?;
    Ok(content.lines().map(str::to_string).collect())
}

fn parse_lines(schema: ArchiveSchema, lines: &[String]) -> Vec<RawTrade> {
    let has_header = matches!(schema, ArchiveSchema::BinanceAggTrades { header: true });
    lines
        .iter()
        .enumerate()
        .filter_map(|(i, line)| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            if i == 0 && (has_header || (matches!(schema, ArchiveSchema::BybitTrading | ArchiveSchema::BybitSpot) && looks_like_header(line))) {
                return None;
            }
            match schema {
                ArchiveSchema::BybitTrading => parse_bybit_trading_row(line),
                ArchiveSchema::BybitSpot => parse_bybit_spot_row(line),
                ArchiveSchema::BinanceAggTrades { .. } => parse_binance_agg_trades_row(line),
            }
        })
        .collect()
}
