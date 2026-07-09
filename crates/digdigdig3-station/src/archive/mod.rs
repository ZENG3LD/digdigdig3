//! Bulk daily trade-archive fetcher — third trade-layer producer alongside
//! REST pagination (`backfill.rs`) and live WS.
//!
//! Scope (Ф1, 2026-07-09): URL construction, day-file iteration, native
//! fetch + decompress + parse into `TradePoint`. Pure logic (URL builder,
//! day iterator, CSV parsers) has no network dependency and is unit-tested
//! directly; the network fetch path (`fetch_archive_day`) is native-only
//! (`cfg(not(target_arch = "wasm32"))`) and untested here (no live network
//! in unit tests — live verification is a separate gate).
//!
//! No archive retention anywhere: every fetch is stream-and-discard, per
//! the client-only data doctrine (`nemo/docs/mlc/plans/
//! unified-data-plane-2026-07-09.md`). Callers normalize the returned rows
//! into `TradePoint`/`AggTradePoint` and prepend into the existing series
//! pipeline; this module never persists archive bytes.

mod parse;
mod url;

#[cfg(not(target_arch = "wasm32"))]
mod fetch;

pub use parse::{parse_binance_agg_trades_row, parse_bybit_spot_row, parse_bybit_trading_row, RawTrade};
pub use url::{url_for_day, DayIterator};

#[cfg(not(target_arch = "wasm32"))]
pub use fetch::{fetch_archive_day, ArchiveDayError};
