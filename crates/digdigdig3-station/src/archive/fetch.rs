//! Day-file fetch: HTTP GET → decompress (gzip/zip) → CSV parse.
//!
//! Runs on both native and wasm32. The transport differs (native = direct
//! `reqwest` against the archive host with a 300s timeout; wasm = `reqwest`'s
//! browser-`fetch` backend, which has no per-request timeout API, routed
//! through a CORS proxy since the browser cannot cross-origin GET
//! `public.bybit.com` / `data.binance.vision` directly) but the decode+parse
//! path (`decode_and_parse`) is fully shared. No retention: bytes are
//! decoded, parsed into rows, and discarded — nothing here writes to disk or
//! keeps the day's bytes beyond this call.

use std::io::{BufRead, BufReader, Cursor, Read};

use chrono::NaiveDate;
use digdigdig3::core::types::{ArchiveFormat, ArchiveSchema, ArchiveSpec};
use flate2::read::GzDecoder;
use thiserror::Error;

use super::parse::{looks_like_header, parse_binance_agg_trades_row, parse_bybit_spot_row, parse_bybit_trading_row, RawTrade};
use super::url::url_for_day;

#[cfg(not(target_arch = "wasm32"))]
const FETCH_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(300);

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

/// Rewrite a direct archive-host URL into the CORS proxy's prefix-mode
/// shape: `{proxy_base}/{host}/{path}[?{query}]`. Mirrors the extraction
/// contract in `digdigdig3::core::proxy_allowlist` / `digdigdig3-proxy`'s
/// `target::from_prefix` (the proxy's own destination parser) — this is the
/// client-side counterpart that BUILDS the URL the proxy expects, the same
/// relationship `core::http::url_override::assemble_rest_url` has to the
/// REST proxy path.
///
/// `direct_url` must be a well-formed `https://{host}{path}` URL (as
/// produced by [`url_for_day`]) — archive day URLs never carry a query
/// string, but the split is written generically in case that changes.
/// `proxy_base` should not have a trailing slash (a trailing slash is
/// tolerated: only one `/` ever separates it from `host`).
fn proxied_url(proxy_base: &str, direct_url: &str) -> String {
    let without_scheme = direct_url
        .strip_prefix("https://")
        .or_else(|| direct_url.strip_prefix("http://"))
        .unwrap_or(direct_url);

    let (host, rest) = match without_scheme.find('/') {
        Some(idx) => (&without_scheme[..idx], &without_scheme[idx..]),
        None => (without_scheme, ""),
    };

    let base = proxy_base.trim_end_matches('/');
    format!("{base}/{host}{rest}")
}

/// Fetch, decompress, and parse one archive day file for `symbol` on
/// `date`. Returns rows oldest-to-newest as they appear in the source file
/// (no re-sorting — archive day files are already chronological on both
/// verified venues).
///
/// `proxy_base` — `None` on native (direct HTTPS to the archive host, no
/// CORS concern off-browser); `Some(base)` on wasm, where the request is
/// rewritten through the CORS proxy's prefix-mode shape via [`proxied_url`]
/// (browsers cannot cross-origin GET `public.bybit.com` /
/// `data.binance.vision` directly — see `digdigdig3-proxy`, which already
/// allowlists both archive hosts). Native callers pass `None`; wasm callers
/// pass their configured proxy base (default `http://127.0.0.1:17800`).
pub async fn fetch_archive_day(
    spec: &ArchiveSpec,
    symbol: &str,
    date: NaiveDate,
    proxy_base: Option<&str>,
) -> Result<Vec<RawTrade>, ArchiveDayError> {
    let direct_url = url_for_day(spec, symbol, date);
    let fetch_url = match proxy_base {
        Some(base) => proxied_url(base, &direct_url),
        None => direct_url.clone(),
    };

    let client = build_client()?;
    let resp = client.get(&fetch_url).send().await?;

    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(ArchiveDayError::NotFound(direct_url));
    }
    if !resp.status().is_success() {
        return Err(ArchiveDayError::HttpStatus { status: resp.status().as_u16(), url: direct_url });
    }

    let bytes = resp.bytes().await?;

    decode_and_parse(&bytes, spec).await
}

/// Native transport: direct `reqwest` client with the 300s day-file
/// timeout (a compressed day file can be tens of MB; the venues' CDNs are
/// not always fast).
#[cfg(not(target_arch = "wasm32"))]
fn build_client() -> Result<reqwest::Client, ArchiveDayError> {
    reqwest::Client::builder()
        .timeout(FETCH_TIMEOUT)
        .build()
        .map_err(|e| ArchiveDayError::Transport(e.to_string()))
}

/// Wasm transport: `reqwest`'s browser-`fetch` backend. No per-request
/// timeout API exists on wasm32 (`ClientBuilder::timeout` panics if called
/// on this target) — the browser's own fetch semantics apply instead.
#[cfg(target_arch = "wasm32")]
fn build_client() -> Result<reqwest::Client, ArchiveDayError> {
    reqwest::Client::builder()
        .build()
        .map_err(|e| ArchiveDayError::Transport(e.to_string()))
}

/// Lines-per-slice for the wasm32 chunked path — chosen so each slice's
/// CPU work (decompress-read + parse) stays comfortably under a browser
/// frame budget even on a slow device, while keeping the number of
/// `TimeoutFuture` yields (and their macrotask-queue overhead) low for a
/// multi-million-line day file. Native ignores this — it runs the whole
/// file in one pass, unchanged from pre-chunking behavior.
#[cfg(target_arch = "wasm32")]
const WASM_CHUNK_LINES: usize = 75_000;

/// Shared decode+parse path for both transports: gunzip/unzip the body per
/// `spec.format`, then parse every line per `spec.schema`.
///
/// Native: runs synchronously start to finish, identical to the
/// pre-chunking implementation (no `.await` point actually yields control —
/// this fn is `async` only so callers share one signature across targets).
/// Wasm32: decompression + line-splitting + parsing are sliced into
/// [`WASM_CHUNK_LINES`]-line chunks with a macrotask yield between each, so
/// a multi-million-line day file never blocks the browser's single UI
/// thread for more than one chunk's worth of CPU time at a stretch.
async fn decode_and_parse(bytes: &[u8], spec: &ArchiveSpec) -> Result<Vec<RawTrade>, ArchiveDayError> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let lines: Vec<String> = match spec.format {
            ArchiveFormat::CsvGz => decode_gz_lines(bytes).map_err(|e| ArchiveDayError::Decode(e.to_string()))?,
            ArchiveFormat::ZipSingleCsv => {
                decode_zip_single_csv_lines(bytes).map_err(ArchiveDayError::Decode)?
            }
        };

        Ok(parse_lines(spec.schema, &lines))
    }

    #[cfg(target_arch = "wasm32")]
    {
        let lines: Vec<String> = match spec.format {
            ArchiveFormat::CsvGz => decode_gz_lines(bytes).map_err(|e| ArchiveDayError::Decode(e.to_string()))?,
            ArchiveFormat::ZipSingleCsv => {
                decode_zip_single_csv_lines(bytes).map_err(ArchiveDayError::Decode)?
            }
        };

        let mut rows = Vec::with_capacity(lines.len());
        for (chunk_idx, chunk) in lines.chunks(WASM_CHUNK_LINES).enumerate() {
            let base_line_idx = chunk_idx * WASM_CHUNK_LINES;
            rows.extend(parse_lines_chunk(spec.schema, chunk, base_line_idx));
            gloo_timers::future::TimeoutFuture::new(0).await;
        }
        Ok(rows)
    }
}

/// Gunzip the whole body (bounded by one day's uncompressed size, up to
/// ~100MB per the plan's memory note — acceptable to materialize as lines
/// for v1; a streaming line-by-line variant can replace this without
/// changing the return shape if a venue's day file grows past what's
/// comfortable in memory). Same on wasm — the body is already fully
/// buffered in `bytes` by the time this runs (browser `fetch` has no
/// streaming decompression API reqwest exposes here), so there is no extra
/// memory cost from running this codec on wasm32 vs native.
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

/// Parse every line in `lines` per `schema`. `base_line_idx` is this slice's
/// offset into the day file's full line sequence — only line 0 of the
/// WHOLE file can be a header, so the header-skip check below only fires
/// when this slice starts at the file's beginning (`base_line_idx == 0`).
/// Used directly (with `base_line_idx = 0` over the full line set) on
/// native; used per-[`WASM_CHUNK_LINES`]-chunk on wasm32.
fn parse_lines_chunk(schema: ArchiveSchema, lines: &[String], base_line_idx: usize) -> Vec<RawTrade> {
    let has_header = matches!(schema, ArchiveSchema::BinanceAggTrades { header: true });
    lines
        .iter()
        .enumerate()
        .filter_map(|(i, line)| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            if base_line_idx + i == 0
                && (has_header || (matches!(schema, ArchiveSchema::BybitTrading | ArchiveSchema::BybitSpot) && looks_like_header(line)))
            {
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

#[cfg(not(target_arch = "wasm32"))]
fn parse_lines(schema: ArchiveSchema, lines: &[String]) -> Vec<RawTrade> {
    parse_lines_chunk(schema, lines, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proxied_url_bybit_host_preserves_path() {
        let url = proxied_url(
            "http://127.0.0.1:17800",
            "https://public.bybit.com/spot/TONUSDT/TONUSDT_2026-06-03.csv.gz",
        );
        assert_eq!(
            url,
            "http://127.0.0.1:17800/public.bybit.com/spot/TONUSDT/TONUSDT_2026-06-03.csv.gz"
        );
    }

    #[test]
    fn proxied_url_binance_host_preserves_path() {
        let url = proxied_url(
            "http://127.0.0.1:17800",
            "https://data.binance.vision/data/futures/um/daily/aggTrades/BTCUSDT/BTCUSDT-aggTrades-2026-06-03.zip",
        );
        assert_eq!(
            url,
            "http://127.0.0.1:17800/data.binance.vision/data/futures/um/daily/aggTrades/BTCUSDT/BTCUSDT-aggTrades-2026-06-03.zip"
        );
    }

    #[test]
    fn proxied_url_strips_trailing_slash_on_proxy_base() {
        let url = proxied_url(
            "http://127.0.0.1:17800/",
            "https://public.bybit.com/spot/BTCUSDT/BTCUSDT_2026-06-03.csv.gz",
        );
        assert_eq!(
            url,
            "http://127.0.0.1:17800/public.bybit.com/spot/BTCUSDT/BTCUSDT_2026-06-03.csv.gz"
        );
    }

    #[test]
    fn proxied_url_handles_host_with_no_path() {
        let url = proxied_url("http://127.0.0.1:17800", "https://public.bybit.com");
        assert_eq!(url, "http://127.0.0.1:17800/public.bybit.com");
    }

    #[test]
    fn proxied_url_works_with_remote_https_proxy_base() {
        let url = proxied_url(
            "https://my-cors-proxy.example.com",
            "https://data.binance.vision/data/futures/um/daily/aggTrades/BTCUSDT/BTCUSDT-aggTrades-2026-06-03.zip",
        );
        assert_eq!(
            url,
            "https://my-cors-proxy.example.com/data.binance.vision/data/futures/um/daily/aggTrades/BTCUSDT/BTCUSDT-aggTrades-2026-06-03.zip"
        );
    }
}
