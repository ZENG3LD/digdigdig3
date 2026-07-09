//! Bulk daily trade-archive fetcher — third trade-layer producer alongside
//! REST pagination (`backfill.rs`) and live WS.
//!
//! Scope (Ф1 2026-07-09 + Ф4 2026-07-09): URL construction, day-file
//! iteration, decompress + parse into `TradePoint`. Pure logic (URL
//! builder, day iterator, CSV parsers) has no network dependency and is
//! unit-tested directly. The network fetch path (`fetch_archive_day`) runs
//! on BOTH native and wasm32 — native dials the archive host directly;
//! wasm routes through a CORS proxy (browsers cannot cross-origin GET
//! `public.bybit.com` / `data.binance.vision`), see `fetch::proxied_url`
//! and the `proxy_base` parameter. Neither transport is exercised by unit
//! tests here (no live network in unit tests — live verification is a
//! separate gate); the pure `proxied_url` URL-rewrite is unit-tested in
//! `fetch.rs`.
//!
//! No archive retention anywhere: every fetch is stream-and-discard, per
//! the client-only data doctrine (`nemo/docs/mlc/plans/
//! unified-data-plane-2026-07-09.md`). Callers normalize the returned rows
//! into `TradePoint`/`AggTradePoint` and prepend into the existing series
//! pipeline; this module never persists archive bytes.

mod parse;
mod url;
mod fetch;

pub use parse::{parse_binance_agg_trades_row, parse_bybit_spot_row, parse_bybit_trading_row, RawTrade};
pub use url::{url_for_day, DayIterator};
pub use fetch::{fetch_archive_day, ArchiveDayError};
