//! Bar-aligned historical series for non-OHLCV streams.
//!
//! The mlq backtester warmup is a slice factory: per `(symbol, timeframe)` it
//! drives each indicator bar-by-bar. To feed the ~130 non-OHLCV mli indicators
//! it needs, per `(symbol, timeframe, stream)`, a series **time-aligned to the
//! OHLCV bar grid** — one value per bar — that warmup routes into the matching
//! mli `update_*` method. This module is dig3's side of that contract: a fetch
//! that returns, for `(exchange, account, symbol, kind, range)`, a bar-aligned
//! series pulled through the shared [`ExchangeHub`] REST surface.
//!
//! Fill policy is decided by the *nature* of the stream, not a global flag
//! (see [`Kind::fill_policy`]):
//!
//! * **State** streams (funding, OI, mark/index price, long/short ratio) are
//!   levels → **last-value carry-forward** onto the bar grid. A bar with no
//!   fresh observation carries the previous value (`filled = true`).
//! * **Flow** streams (liquidation, aggTrade) are event flows → **bucket-sum
//!   per bar**, gaps are a real `0.0` (`filled = false`).
//!
//! Kline-family kinds (mark/index/premium price klines) arrive already on the
//! bar grid; they are returned verbatim as [`BarPoint`]s, no resample needed.

use std::sync::Arc;

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{AccountType, ExchangeId, SymbolInput};
use digdigdig3::core::websocket::KlineInterval;

use crate::data::BarPoint;
use crate::error::{Result, StationError};
use crate::series::{Kind, SeriesKey};

/// How a stream's observations collapse onto a bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillPolicy {
    /// State/level stream — carry the last observation forward into empty bars.
    ForwardFill,
    /// Event-flow stream — sum observations falling inside the bar; empty = 0.
    ZeroFlow,
}

/// One bar's worth of a scalar non-OHLCV stream, aligned to the OHLCV grid.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScalarBar {
    /// Bar open time (ms) — aligned to the interval grid, matches the kline key.
    pub bar_open_time: i64,
    /// The stream's value for this bar (rate / OI / ratio / price / flow-sum).
    pub value: f64,
    /// `true` if this bar carried a prior value (ForwardFill) rather than
    /// observing a fresh one. Always `false` for ZeroFlow.
    pub filled: bool,
}

/// Bar-aligned series, shaped by the stream kind. Mirrors the kline path so
/// mlq keys it the same way and warmup feeds it to the matching `update_*`.
#[derive(Debug, Clone)]
pub enum BarAlignedSeries {
    /// Mark/index/premium price klines — native OHLCV bars.
    Klines(Vec<BarPoint>),
    /// Scalar state or flow stream collapsed onto the bar grid.
    Scalar(Vec<ScalarBar>),
}

impl BarAlignedSeries {
    /// Number of bars in the series.
    pub fn len(&self) -> usize {
        match self {
            BarAlignedSeries::Klines(v) => v.len(),
            BarAlignedSeries::Scalar(v) => v.len(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Kind {
    /// Fill policy for collapsing this stream's observations onto a bar.
    /// State streams carry forward; flow streams bucket-sum with zero gaps.
    pub fn fill_policy(&self) -> FillPolicy {
        match self {
            Kind::Liquidation | Kind::AggTrade | Kind::Trade => FillPolicy::ZeroFlow,
            _ => FillPolicy::ForwardFill,
        }
    }
}

/// Interval string → bar step in milliseconds. Supports the standard kline
/// intervals up to weekly. Calendar-month (`"1M"`) has no fixed ms width and
/// returns `None` (kline-family months don't need this path; scalar resample
/// at monthly is out of scope).
fn interval_millis(iv: &str) -> Option<i64> {
    let iv = iv.trim();
    let (num, unit) = iv.split_at(iv.len().saturating_sub(1));
    let n: i64 = num.parse().ok()?;
    let per = match unit {
        "m" => 60_000,
        "h" => 60 * 60_000,
        "d" | "D" => 24 * 60 * 60_000,
        "w" | "W" => 7 * 24 * 60 * 60_000,
        _ => return None,
    };
    Some(n * per)
}

/// Resample a sorted `(ts_ms, value)` source onto the `[start, end)` bar grid
/// of width `step` ms, per `policy`. The grid is aligned to interval boundaries
/// (`bar_open_time % step == 0`), matching exchange kline semantics.
///
/// * `ForwardFill`: each bar's value is the most recent observation known by
///   the bar's close (`ts < bar_open + step`); leading bars with no prior
///   observation are omitted (gap = none at the head).
/// * `ZeroFlow`: each bar's value is the sum of observations with
///   `ts ∈ [bar_open, bar_open + step)`; every bar in range is emitted (0 if
///   none).
fn resample(mut src: Vec<(i64, f64)>, start: i64, end: i64, step: i64, policy: FillPolicy) -> Vec<ScalarBar> {
    src.sort_unstable_by_key(|(ts, _)| *ts);
    let first_bar = start.div_euclid(step) * step;
    let mut out = Vec::new();

    match policy {
        FillPolicy::ForwardFill => {
            let mut idx = 0usize;
            let mut last: Option<f64> = None;
            let mut t = first_bar;
            while t < end {
                let bar_close = t + step;
                let mut fresh = false;
                while idx < src.len() && src[idx].0 < bar_close {
                    last = Some(src[idx].1);
                    if src[idx].0 >= t {
                        fresh = true;
                    }
                    idx += 1;
                }
                if let Some(v) = last {
                    out.push(ScalarBar { bar_open_time: t, value: v, filled: !fresh });
                }
                t += step;
            }
        }
        FillPolicy::ZeroFlow => {
            let mut idx = 0usize;
            let mut t = first_bar;
            while t < end {
                let bar_close = t + step;
                let mut sum = 0.0;
                while idx < src.len() && src[idx].0 < bar_close {
                    if src[idx].0 >= t {
                        sum += src[idx].1;
                    }
                    idx += 1;
                }
                out.push(ScalarBar { bar_open_time: t, value: sum, filled: false });
                t += step;
            }
        }
    }
    out
}

/// Bar-align an in-memory series of recorded [`DataPoint`]s onto the OHLCV grid
/// (Track C — daemon-recorded streams with no REST history: liquidations, L2,
/// aggTrades). The forward-recording daemon (`dig3 watch` / Station persistence)
/// writes typed points to a `DiskStore<T>`; a consumer reads them back
/// (`DiskStore::read_tail` / `Series`) and feeds the slice here with a scalar
/// projection to get the same bar-aligned shape the REST loader produces.
///
/// `project` extracts the scalar per point (e.g. liquidation notional, aggTrade
/// size). Fill policy is the caller's choice: `ZeroFlow` for event flows
/// (liq/aggTrade — sum per bar, gap=0), `ForwardFill` for recorded levels.
///
/// This is the in-process resample step; reading the points off disk and
/// choosing the range is the caller's job (the recording is forward-only — you
/// can only bar-align what the daemon has captured).
pub fn bar_align_points<T, F>(
    points: &[T],
    project: F,
    start_ms: i64,
    end_ms: i64,
    interval: &KlineInterval,
    policy: FillPolicy,
) -> Result<Vec<ScalarBar>>
where
    T: crate::series::DataPoint,
    F: Fn(&T) -> f64,
{
    let step = interval_millis(interval.as_str())
        .ok_or_else(|| StationError::Core(format!("interval {interval} has no fixed ms width for resample")))?;
    let src: Vec<(i64, f64)> = points
        .iter()
        .filter(|p| {
            let t = p.timestamp_ms();
            t >= start_ms && t < end_ms
        })
        .map(|p| (p.timestamp_ms(), project(p)))
        .collect();
    Ok(resample(src, start_ms, end_ms, step, policy))
}

/// Track-C read-path: read a recorded stream off its `DiskStore` and bar-align
/// it in one call. Reads the last `max_points` records (the daemon appends
/// forward-only, so the tail is the captured window), range-filters, and
/// resamples per `project`/`policy`. Native-only — the wasm OPFS store has a
/// divergent read error type; wasm consumers read points then call
/// [`bar_align_points`] directly.
#[cfg(not(target_arch = "wasm32"))]
pub async fn bar_align_from_disk<T, F>(
    store: &crate::series::DiskStore<T>,
    project: F,
    start_ms: i64,
    end_ms: i64,
    interval: &KlineInterval,
    policy: FillPolicy,
    max_points: usize,
) -> Result<Vec<ScalarBar>>
where
    T: crate::series::DataPoint,
    F: Fn(&T) -> f64,
{
    let pts = store
        .read_tail(max_points)
        .await
        .map_err(|e| StationError::Core(format!("DiskStore read_tail failed: {e}")))?;
    bar_align_points(&pts, project, start_ms, end_ms, interval, policy)
}

/// Cap a REST kline `limit` to the per-call exchange ceiling.
const KLINE_PAGE: u16 = 1000;

/// Paginate kline-family history backwards over `[start, end)` and return all
/// bars in `[start, end)`, oldest-first, deduped by `open_time`.
///
/// `fetch` pulls up to `KLINE_PAGE` bars ending at the given exclusive
/// `end_time` (oldest→newest). We walk `end_time` backwards until we reach
/// `start` or the exchange stops returning new bars.
async fn paginate_klines<F, Fut>(start: i64, end: i64, mut fetch: F) -> Result<Vec<BarPoint>>
where
    F: FnMut(i64) -> Fut,
    Fut: std::future::Future<Output = Result<Vec<BarPoint>>>,
{
    let mut all: Vec<BarPoint> = Vec::new();
    let mut cursor = end;
    // Safety bound: never loop forever on a misbehaving venue.
    for _ in 0..256 {
        // One retry to absorb a transient gateway error (e.g. Gate.io's
        // premium-index endpoint intermittently 504s on heavy pages).
        let page = match fetch(cursor).await {
            Ok(p) => p,
            Err(_) => fetch(cursor).await?,
        };
        if page.is_empty() {
            break;
        }
        let oldest = page.iter().map(|b| b.open_time).min().unwrap_or(cursor);
        all.extend(page);
        if oldest <= start {
            break;
        }
        // Next page ends just before the oldest bar we already have.
        let next = oldest - 1;
        if next >= cursor {
            break; // no progress
        }
        cursor = next;
    }
    all.retain(|b| b.open_time >= start && b.open_time < end);
    all.sort_unstable_by_key(|b| b.open_time);
    all.dedup_by_key(|b| b.open_time);
    Ok(all)
}

/// Load a bar-aligned historical series for one `(exchange, account, symbol,
/// kind, interval)` over `[start_ms, end_ms)`.
///
/// `symbol` must be exchange-native (already normalized) — matching the
/// `SubscriptionSet::add_raw` / `fetch_history` convention.
///
/// Supported now (REST-historical, no daemon):
/// * Kline-family: `Kline`, `MarkPriceKline`, `IndexPriceKline`,
///   `PremiumIndexKline` → [`BarAlignedSeries::Klines`].
/// * Scalar state: `FundingRate`, `OpenInterest`, `LongShortRatio`,
///   `MarkPrice`, `IndexPrice` → [`BarAlignedSeries::Scalar`].
///
/// Flow streams (`Liquidation`, `AggTrade`) and book streams require the
/// recording daemon and return `StreamNotSupported` here.
pub async fn load_bar_aligned(
    hub: &Arc<ExchangeHub>,
    exchange: ExchangeId,
    account: AccountType,
    symbol: &str,
    kind: &Kind,
    interval: &KlineInterval,
    start_ms: i64,
    end_ms: i64,
) -> Result<BarAlignedSeries> {
    let rest = hub
        .rest(exchange)
        .ok_or_else(|| StationError::Core(format!("{exchange:?} not connected in hub")))?;

    // ── Kline-family — native bar grid, just paginate + range-filter. ──────────
    if let Some(klines) = match kind {
        Kind::Kline(iv) => Some(("kline", iv.clone())),
        Kind::MarkPriceKline(iv) => Some(("mark", iv.clone())),
        Kind::IndexPriceKline(iv) => Some(("index", iv.clone())),
        Kind::PremiumIndexKline(iv) => Some(("premium", iv.clone())),
        _ => None,
    } {
        let (which, iv) = klines;
        let rest = rest.clone();
        let sym = symbol.to_string();
        let bars = paginate_klines(start_ms, end_ms, |cursor| {
            let rest = rest.clone();
            let sym = sym.clone();
            let ivs = iv.as_str().to_string();
            async move {
                let res = match which {
                    "kline" => rest
                        .get_klines(SymbolInput::Raw(&sym), &ivs, Some(KLINE_PAGE), account, Some(cursor))
                        .await,
                    "mark" => rest
                        .get_mark_price_klines(SymbolInput::Raw(&sym), &ivs, Some(KLINE_PAGE as u32), account, Some(cursor))
                        .await,
                    "index" => rest
                        .get_index_price_klines(SymbolInput::Raw(&sym), &ivs, Some(KLINE_PAGE as u32), account, Some(cursor))
                        .await,
                    _ => rest
                        .get_premium_index_klines(SymbolInput::Raw(&sym), &ivs, Some(KLINE_PAGE as u32), account, Some(cursor))
                        .await,
                };
                res.map(|ks| ks.iter().map(BarPoint::from_kline).collect())
                    .map_err(|e| StationError::Core(format!("{which} klines failed: {e}")))
            }
        })
        .await?;
        return Ok(BarAlignedSeries::Klines(bars));
    }

    // ── Scalar streams — fetch source observations then resample to the grid. ──
    let step = interval_millis(interval.as_str())
        .ok_or_else(|| StationError::Core(format!("interval {interval} has no fixed ms width for resample")))?;

    let src: Vec<(i64, f64)> = match kind {
        Kind::FundingRate => rest
            .get_funding_rate_history(SymbolInput::Raw(symbol), Some(start_ms), Some(end_ms), Some(1000), account)
            .await
            .map_err(|e| StationError::Core(format!("funding history failed: {e}")))?
            .into_iter()
            .map(|f| (f.timestamp, f.rate))
            .collect(),

        Kind::OpenInterest => rest
            .get_open_interest_history(SymbolInput::Raw(symbol), interval.as_str(), Some(start_ms), Some(end_ms), Some(500), account)
            .await
            .map_err(|e| StationError::Core(format!("open interest history failed: {e}")))?
            .into_iter()
            .map(|oi| (oi.timestamp, oi.open_interest))
            .collect(),

        Kind::Basis => rest
            .get_basis_history(SymbolInput::Raw(symbol), interval.as_str(), Some(start_ms), Some(end_ms), Some(500), account)
            .await
            .map_err(|e| StationError::Core(format!("basis history failed: {e}")))?
            .into_iter()
            .map(|b| (b.timestamp, b.basis))
            .collect(),

        Kind::LongShortRatio => rest
            .get_long_short_ratio_history(SymbolInput::Raw(symbol), interval.as_str(), Some(start_ms), Some(end_ms), Some(500), account)
            .await
            .map_err(|e| StationError::Core(format!("long/short ratio history failed: {e}")))?
            .into_iter()
            .map(|r| {
                // Prefer the exchange-provided combined ratio; else long/short.
                let v = r.ratio.unwrap_or_else(|| {
                    if r.short_ratio > 0.0 { r.long_ratio / r.short_ratio } else { r.long_ratio }
                });
                (r.timestamp, v)
            })
            .collect(),

        // Scalar mark/index price: project the close of the corresponding
        // derived kline (native bar grid — no separate resample).
        Kind::MarkPrice | Kind::IndexPrice => {
            let which = if matches!(kind, Kind::MarkPrice) { "mark" } else { "index" };
            let rest2 = rest.clone();
            let sym = symbol.to_string();
            let ivs = interval.as_str().to_string();
            let bars = paginate_klines(start_ms, end_ms, |cursor| {
                let rest2 = rest2.clone();
                let sym = sym.clone();
                let ivs = ivs.clone();
                async move {
                    let res = if which == "mark" {
                        rest2.get_mark_price_klines(SymbolInput::Raw(&sym), &ivs, Some(KLINE_PAGE as u32), account, Some(cursor)).await
                    } else {
                        rest2.get_index_price_klines(SymbolInput::Raw(&sym), &ivs, Some(KLINE_PAGE as u32), account, Some(cursor)).await
                    };
                    res.map(|ks| ks.iter().map(BarPoint::from_kline).collect())
                        .map_err(|e| StationError::Core(format!("{which} klines failed: {e}")))
                }
            })
            .await?;
            // Already bar-aligned; project close directly.
            let scalars = bars
                .into_iter()
                .map(|b| ScalarBar { bar_open_time: b.open_time, value: b.close, filled: false })
                .collect();
            return Ok(BarAlignedSeries::Scalar(scalars));
        }

        Kind::Liquidation | Kind::AggTrade => {
            return Err(StationError::StreamNotSupported(format!(
                "{kind:?} bar-aligned history needs the recording daemon (no usable REST history)"
            )));
        }

        other => {
            return Err(StationError::StreamNotSupported(format!(
                "bar-aligned loader does not yet support {other:?}"
            )));
        }
    };

    Ok(BarAlignedSeries::Scalar(resample(src, start_ms, end_ms, step, kind.fill_policy())))
}

/// Convenience wrapper keyed by [`SeriesKey`]. The interval is read from the
/// key's kline-family kind; non-kline scalar kinds require `interval` to be
/// passed via [`load_bar_aligned`] directly (they carry no interval in the key).
pub async fn load_for_key(
    hub: &Arc<ExchangeHub>,
    key: &SeriesKey,
    interval: &KlineInterval,
    start_ms: i64,
    end_ms: i64,
) -> Result<BarAlignedSeries> {
    let iv = match &key.kind {
        Kind::Kline(iv) | Kind::MarkPriceKline(iv) | Kind::IndexPriceKline(iv) | Kind::PremiumIndexKline(iv) => iv.clone(),
        _ => interval.clone(),
    };
    load_bar_aligned(hub, key.exchange, key.account_type, &key.symbol, &key.kind, &iv, start_ms, end_ms).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interval_parsing() {
        assert_eq!(interval_millis("1m"), Some(60_000));
        assert_eq!(interval_millis("5m"), Some(300_000));
        assert_eq!(interval_millis("1h"), Some(3_600_000));
        assert_eq!(interval_millis("4h"), Some(14_400_000));
        assert_eq!(interval_millis("1d"), Some(86_400_000));
        assert_eq!(interval_millis("1w"), Some(604_800_000));
        assert_eq!(interval_millis("1M"), None);
        assert_eq!(interval_millis("xx"), None);
    }

    #[test]
    fn forward_fill_carries_into_empty_bars() {
        // step 60s. Observations at t=0 (v=10) and t=130s (v=20).
        let step = 60_000;
        let src = vec![(0, 10.0), (130_000, 20.0)];
        let out = resample(src, 0, 240_000, step, FillPolicy::ForwardFill);
        // bars: 0,60k,120k,180k
        assert_eq!(out.len(), 4);
        assert_eq!(out[0], ScalarBar { bar_open_time: 0, value: 10.0, filled: false });
        // bar 60k: no fresh obs → carry 10.0, filled
        assert_eq!(out[1], ScalarBar { bar_open_time: 60_000, value: 10.0, filled: true });
        // bar 120k: obs at 130k is inside [120k,180k) → fresh 20.0
        assert_eq!(out[2], ScalarBar { bar_open_time: 120_000, value: 20.0, filled: false });
        // bar 180k: carry 20.0
        assert_eq!(out[3], ScalarBar { bar_open_time: 180_000, value: 20.0, filled: true });
    }

    #[test]
    fn forward_fill_omits_leading_gap() {
        let step = 60_000;
        // First observation only at 150s — bars 0 and 60k have nothing to carry.
        let src = vec![(150_000, 5.0)];
        let out = resample(src, 0, 240_000, step, FillPolicy::ForwardFill);
        assert_eq!(out.len(), 2); // only 120k and 180k
        assert_eq!(out[0].bar_open_time, 120_000);
        assert_eq!(out[0].value, 5.0);
        assert!(!out[0].filled);
        assert_eq!(out[1], ScalarBar { bar_open_time: 180_000, value: 5.0, filled: true });
    }

    #[test]
    fn zero_flow_buckets_and_zeros_gaps() {
        let step = 60_000;
        // Two events in bar 0, none in bar 60k, one in bar 120k.
        let src = vec![(1_000, 3.0), (50_000, 4.0), (125_000, 9.0)];
        let out = resample(src, 0, 180_000, step, FillPolicy::ZeroFlow);
        assert_eq!(out.len(), 3);
        assert_eq!(out[0], ScalarBar { bar_open_time: 0, value: 7.0, filled: false });
        assert_eq!(out[1], ScalarBar { bar_open_time: 60_000, value: 0.0, filled: false });
        assert_eq!(out[2], ScalarBar { bar_open_time: 120_000, value: 9.0, filled: false });
    }

    #[test]
    fn bar_align_points_flow_from_recorded() {
        use crate::data::LiquidationPoint;
        // Recorded liquidations: two in bar 0, none in bar 60k, one in bar 120k.
        let pts = vec![
            LiquidationPoint { ts_ms: 1_000, price: 100.0, quantity: 1.0, value: 5_000.0, side: 0 },
            LiquidationPoint { ts_ms: 40_000, price: 100.0, quantity: 1.0, value: 3_000.0, side: 1 },
            LiquidationPoint { ts_ms: 125_000, price: 100.0, quantity: 1.0, value: 9_000.0, side: 0 },
        ];
        let out = bar_align_points(
            &pts, |p| p.value, 0, 180_000, &KlineInterval::new("1m"), FillPolicy::ZeroFlow,
        ).unwrap();
        assert_eq!(out.len(), 3);
        assert_eq!(out[0], ScalarBar { bar_open_time: 0, value: 8_000.0, filled: false });
        assert_eq!(out[1], ScalarBar { bar_open_time: 60_000, value: 0.0, filled: false });
        assert_eq!(out[2], ScalarBar { bar_open_time: 120_000, value: 9_000.0, filled: false });
    }

    #[test]
    fn grid_aligns_to_interval_boundary() {
        let step = 60_000;
        // start mid-bar → grid still snaps to boundary below start.
        let src = vec![(70_000, 1.0)];
        let out = resample(src, 35_000, 130_000, step, FillPolicy::ForwardFill);
        // first_bar = floor(35000/60000)*60000 = 0
        assert_eq!(out[0].bar_open_time, 60_000); // bar 0 had no obs (omitted); 60k has obs at 70k
        assert_eq!(out[0].value, 1.0);
    }
}
