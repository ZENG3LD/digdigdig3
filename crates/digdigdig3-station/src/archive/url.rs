//! Day-file URL construction + backward day iteration for archive specs.

use chrono::{Duration, NaiveDate, Utc};
use digdigdig3::core::types::ArchiveSpec;

/// Substitute `{symbol}` and `{date}` (`YYYY-MM-DD`) into `spec.url_pattern`
/// and assemble the full `https://{host}{path}` URL for one day file.
///
/// `symbol` is used verbatim (exchange-native form, e.g. `"BTCUSDT"`) — no
/// normalization is applied here, matching the connector's raw-symbol
/// convention.
pub fn url_for_day(spec: &ArchiveSpec, symbol: &str, date: NaiveDate) -> String {
    let date_str = date.format("%Y-%m-%d").to_string();
    let path = spec
        .url_pattern
        .replace("{symbol}", symbol)
        .replace("{date}", &date_str);
    format!("https://{}{}", spec.host, path)
}

/// Walks day dates backward from "yesterday UTC" (archives are typically
/// not same-day-available) down to and including the day containing
/// `since_ms`. One day = one ramp "page" for archive-phase `RampState`
/// purposes.
///
/// Exhausted (`next()` returns `None`) once the walk has yielded the day
/// containing `since_ms`, or immediately if `since_ms` is in the future
/// relative to yesterday (degenerate case — yields nothing).
#[derive(Debug, Clone)]
pub struct DayIterator {
    /// Next date to yield, walking backward. `None` once exhausted.
    cursor: Option<NaiveDate>,
    /// Inclusive floor — the day containing `since_ms`.
    floor: NaiveDate,
}

impl DayIterator {
    /// Build an iterator starting at "yesterday UTC" and walking backward
    /// to the day containing `since_ms` (inclusive).
    pub fn new(since_ms: i64) -> Self {
        Self::from_start(Utc::now().date_naive() - Duration::days(1), since_ms)
    }

    /// Build an iterator starting at an explicit `start` date (useful for
    /// deterministic tests / resuming a partial walk) and walking backward
    /// to the day containing `since_ms` (inclusive).
    pub fn from_start(start: NaiveDate, since_ms: i64) -> Self {
        let floor = day_from_ms(since_ms);
        let cursor = if start < floor { None } else { Some(start) };
        Self { cursor, floor }
    }

    /// Build an iterator resuming a walk that already loaded trades down to
    /// `oldest_loaded_ms` — starts at the day BEFORE the day containing that
    /// timestamp (the day itself may only be partially covered by whatever
    /// already loaded it, so re-walking it is skipped) and continues
    /// backward to the day containing `since_ms` (inclusive).
    pub fn resume_before(oldest_loaded_ms: i64, since_ms: i64) -> Self {
        let day = day_from_ms(oldest_loaded_ms);
        let start = day.pred_opt().unwrap_or(day);
        Self::from_start(start, since_ms)
    }
}

impl Iterator for DayIterator {
    type Item = NaiveDate;

    fn next(&mut self) -> Option<NaiveDate> {
        let current = self.cursor?;
        self.cursor = if current > self.floor {
            Some(current - Duration::days(1))
        } else {
            None
        };
        Some(current)
    }
}

fn day_from_ms(ms: i64) -> NaiveDate {
    let secs = ms.div_euclid(1000);
    chrono::DateTime::from_timestamp(secs, 0)
        .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap_or_default())
        .date_naive()
}

#[cfg(test)]
mod tests {
    use super::*;
    use digdigdig3::core::types::{ArchiveFormat, ArchiveSchema};

    fn bybit_spot_spec() -> ArchiveSpec {
        ArchiveSpec {
            host: "public.bybit.com",
            url_pattern: "/spot/{symbol}/{symbol}_{date}.csv.gz",
            format: ArchiveFormat::CsvGz,
            schema: ArchiveSchema::BybitSpot,
            since_ms: 1_667_260_800_000,
        }
    }

    fn bybit_futures_spec() -> ArchiveSpec {
        ArchiveSpec {
            host: "public.bybit.com",
            url_pattern: "/trading/{symbol}/{symbol}{date}.csv.gz",
            format: ArchiveFormat::CsvGz,
            schema: ArchiveSchema::BybitTrading,
            since_ms: 1_585_094_400_000,
        }
    }

    fn binance_futures_spec() -> ArchiveSpec {
        ArchiveSpec {
            host: "data.binance.vision",
            url_pattern: "/data/futures/um/daily/aggTrades/{symbol}/{symbol}-aggTrades-{date}.zip",
            format: ArchiveFormat::ZipSingleCsv,
            schema: ArchiveSchema::BinanceAggTrades { header: true },
            since_ms: 1_577_836_800_000,
        }
    }

    #[test]
    fn bybit_spot_url_uses_underscore_separator() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 3).unwrap_or_default();
        let url = url_for_day(&bybit_spot_spec(), "TONUSDT", date);
        assert_eq!(url, "https://public.bybit.com/spot/TONUSDT/TONUSDT_2026-06-03.csv.gz");
    }

    #[test]
    fn bybit_futures_url_has_no_separator() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 3).unwrap_or_default();
        let url = url_for_day(&bybit_futures_spec(), "TONUSDT", date);
        assert_eq!(url, "https://public.bybit.com/trading/TONUSDT/TONUSDT2026-06-03.csv.gz");
    }

    #[test]
    fn binance_futures_url_builds_expected_path() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 3).unwrap_or_default();
        let url = url_for_day(&binance_futures_spec(), "BTCUSDT", date);
        assert_eq!(
            url,
            "https://data.binance.vision/data/futures/um/daily/aggTrades/BTCUSDT/BTCUSDT-aggTrades-2026-06-03.zip"
        );
    }

    #[test]
    fn day_iterator_walks_backward_inclusive_of_floor() {
        let start = NaiveDate::from_ymd_opt(2026, 6, 5).unwrap_or_default();
        let since_ms = NaiveDate::from_ymd_opt(2026, 6, 2)
            .unwrap_or_default()
            .and_hms_opt(0, 0, 0)
            .unwrap_or_default()
            .and_utc()
            .timestamp_millis();
        let days: Vec<NaiveDate> = DayIterator::from_start(start, since_ms).collect();
        assert_eq!(
            days,
            vec![
                NaiveDate::from_ymd_opt(2026, 6, 5).unwrap_or_default(),
                NaiveDate::from_ymd_opt(2026, 6, 4).unwrap_or_default(),
                NaiveDate::from_ymd_opt(2026, 6, 3).unwrap_or_default(),
                NaiveDate::from_ymd_opt(2026, 6, 2).unwrap_or_default(),
            ]
        );
    }

    #[test]
    fn day_iterator_yields_nothing_when_since_after_start() {
        let start = NaiveDate::from_ymd_opt(2026, 6, 2).unwrap_or_default();
        let since_ms = NaiveDate::from_ymd_opt(2026, 6, 5)
            .unwrap_or_default()
            .and_hms_opt(0, 0, 0)
            .unwrap_or_default()
            .and_utc()
            .timestamp_millis();
        let days: Vec<NaiveDate> = DayIterator::from_start(start, since_ms).collect();
        assert!(days.is_empty());
    }

    #[test]
    fn day_iterator_single_day_when_start_equals_floor() {
        let start = NaiveDate::from_ymd_opt(2026, 6, 2).unwrap_or_default();
        let since_ms = start.and_hms_opt(0, 0, 0).unwrap_or_default().and_utc().timestamp_millis();
        let days: Vec<NaiveDate> = DayIterator::from_start(start, since_ms).collect();
        assert_eq!(days, vec![start]);
    }

    #[test]
    fn resume_before_starts_one_day_before_oldest_loaded() {
        let oldest_loaded = NaiveDate::from_ymd_opt(2026, 6, 5)
            .unwrap_or_default()
            .and_hms_opt(12, 0, 0)
            .unwrap_or_default()
            .and_utc()
            .timestamp_millis();
        let since_ms = NaiveDate::from_ymd_opt(2026, 6, 2)
            .unwrap_or_default()
            .and_hms_opt(0, 0, 0)
            .unwrap_or_default()
            .and_utc()
            .timestamp_millis();
        let days: Vec<NaiveDate> = DayIterator::resume_before(oldest_loaded, since_ms).collect();
        assert_eq!(
            days,
            vec![
                NaiveDate::from_ymd_opt(2026, 6, 4).unwrap_or_default(),
                NaiveDate::from_ymd_opt(2026, 6, 3).unwrap_or_default(),
                NaiveDate::from_ymd_opt(2026, 6, 2).unwrap_or_default(),
            ],
            "resume must skip the (possibly partially-covered) day already loaded"
        );
    }

    #[test]
    fn resume_before_yields_nothing_once_day_before_oldest_is_below_floor() {
        let oldest_loaded = NaiveDate::from_ymd_opt(2026, 6, 2)
            .unwrap_or_default()
            .and_hms_opt(0, 0, 0)
            .unwrap_or_default()
            .and_utc()
            .timestamp_millis();
        let since_ms = NaiveDate::from_ymd_opt(2026, 6, 2)
            .unwrap_or_default()
            .and_hms_opt(0, 0, 0)
            .unwrap_or_default()
            .and_utc()
            .timestamp_millis();
        let days: Vec<NaiveDate> = DayIterator::resume_before(oldest_loaded, since_ms).collect();
        assert!(days.is_empty(), "day before 2026-06-02 (2026-06-01) is below the since_ms floor");
    }
}
