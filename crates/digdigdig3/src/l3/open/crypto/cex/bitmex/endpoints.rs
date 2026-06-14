//! BitMEX URL constants and endpoint helpers.

/// Mainnet WebSocket URL.
pub const WS_URL: &str = "wss://ws.bitmex.com/realtime";
/// Testnet WebSocket URL.
pub const WS_URL_TESTNET: &str = "wss://ws.testnet.bitmex.com/realtime";

/// Mainnet REST base URL.
pub const REST_URL: &str = "https://www.bitmex.com/api/v1";
/// Testnet REST base URL.
pub const REST_URL_TESTNET: &str = "https://testnet.bitmex.com/api/v1";

// ─────────────────────────────────────────────────────────────────────────────
// REST path constants (relative to REST_URL / REST_URL_TESTNET)
// ─────────────────────────────────────────────────────────────────────────────

/// Public recent trades — `GET /api/v1/trade`
///
/// Query params: `symbol`, `count`, `reverse=true`.
/// Each row: `{timestamp, symbol, side, size, price, trdMatchID, …}`.
/// `timestamp` is ISO-8601 string (NOT epoch ms).
pub const PATH_TRADE: &str = "/trade";

/// OHLCV bucketed trade data — `GET /api/v1/trade/bucketed`
///
/// Query params: `symbol`, `binSize` (1m/5m/1h/1d), `count`, `reverse=true`.
/// Each row: `{timestamp, open, high, low, close, volume, trades, homeNotional, …}`.
/// NOTE: BitMEX bucket `timestamp` = **end** of the period, NOT the open time.
/// We store it as `open_time` = `timestamp − binSize_ms` to match Kline convention.
pub const PATH_TRADE_BUCKETED: &str = "/trade/bucketed";

/// Historical funding rate snapshots — `GET /api/v1/funding`
///
/// Query params: `symbol`, `count`, `reverse=true`.
/// Each row: `{timestamp, symbol, fundingInterval, fundingRate, fundingRateDaily}`.
pub const PATH_FUNDING: &str = "/funding";

/// Historical liquidation events — `GET /api/v1/liquidation`
///
/// Query params: `symbol`, `count`, `reverse=true`.
/// The endpoint returns `[]` when no forced liquidations occurred recently —
/// that is normal BitMEX behaviour, not an error.
/// Each row: `{orderID, symbol, side, price, leavesQty}`.
pub const PATH_LIQUIDATION: &str = "/liquidation";

// ─────────────────────────────────────────────────────────────────────────────
// Interval → BitMEX binSize mapping
// ─────────────────────────────────────────────────────────────────────────────

/// Convert a canonical interval string to the BitMEX `binSize` query parameter.
///
/// Returns `None` for intervals not supported by BitMEX bucketed trade endpoint.
/// BitMEX supports only four bucket sizes: `1m`, `5m`, `1h`, `1d`.
pub fn interval_to_bin_size(interval: &str) -> Option<&'static str> {
    match interval {
        "1m"  => Some("1m"),
        "5m"  => Some("5m"),
        "1h"  => Some("1h"),
        "1d"  => Some("1d"),
        _ => None,
    }
}

/// Return the duration of a BitMEX binSize in milliseconds.
///
/// Used to convert the bucket-close timestamp to an open-time by subtracting
/// the bucket duration. Panics are impossible here because callers only pass
/// strings previously validated by `interval_to_bin_size`.
pub fn bin_size_duration_ms(bin_size: &str) -> i64 {
    match bin_size {
        "1m"  => 60_000,
        "5m"  => 5 * 60_000,
        "1h"  => 60 * 60_000,
        "1d"  => 24 * 60 * 60_000,
        _     => 60_000, // safe fallback — should not be reached
    }
}
