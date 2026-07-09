//! Per-venue CSV row parsers → normalized [`RawTrade`] shape.
//!
//! Column layouts and side polarity mapping verified live 2026-07-09
//! against real downloaded day files (see fixture rows in the unit tests
//! below — captured verbatim from the archive hosts, not synthesized).

/// One parsed trade row, normalized across every venue's archive schema.
///
/// `is_buyer_maker` follows the exact polarity `TradePoint::from_public` /
/// `agg_trade_point_from` use elsewhere in this crate (`backfill.rs`):
/// `true` → the trade's taker side was SELL (buyer was resting/maker);
/// `false` → taker side was BUY. Mirrored here so callers can build
/// `TradePoint { side: 0=Buy, 1=Sell }` / `AggTradePoint` with the same
/// convention as the REST path, with no separate polarity table to keep in
/// sync.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RawTrade {
    pub ts_ms: i64,
    pub price: f64,
    pub qty: f64,
    pub is_buyer_maker: bool,
}

/// Bybit derivatives/linear (`ArchiveSchema::BybitTrading`) row parser.
///
/// Columns: `timestamp,symbol,side,size,price,tickDirection,trdMatchID,
/// grossValue,homeNotional,foreignNotional,RPI`. `timestamp` is Unix
/// SECONDS with a fractional part (`1781481604.4`). `side` is the taker
/// side, capitalized `Buy`/`Sell`. Trailing columns beyond `price` are
/// ignored (tolerates `grossValue` scientific notation and any additional
/// trailing fields without needing to parse them).
pub fn parse_bybit_trading_row(line: &str) -> Option<RawTrade> {
    let mut cols = line.split(',');
    let ts_raw = cols.next()?;
    let _symbol = cols.next()?;
    let side_raw = cols.next()?;
    let size_raw = cols.next()?;
    let price_raw = cols.next()?;

    let ts_secs: f64 = ts_raw.trim().parse().ok()?;
    let ts_ms = (ts_secs * 1000.0).round() as i64;
    let price: f64 = price_raw.trim().parse().ok()?;
    let qty: f64 = size_raw.trim().parse().ok()?;
    // Taker side "Sell" → buyer was resting (maker) → is_buyer_maker = true.
    let is_buyer_maker = match side_raw.trim() {
        "Sell" => true,
        "Buy" => false,
        _ => return None,
    };

    Some(RawTrade { ts_ms, price, qty, is_buyer_maker })
}

/// Bybit spot (`ArchiveSchema::BybitSpot`) row parser.
///
/// Columns: `id,timestamp,price,volume,side,rpi`. `timestamp` is INTEGER
/// milliseconds. `side` is the taker side, lowercase `sell`/`buy`.
pub fn parse_bybit_spot_row(line: &str) -> Option<RawTrade> {
    let mut cols = line.split(',');
    let _id = cols.next()?;
    let ts_raw = cols.next()?;
    let price_raw = cols.next()?;
    let volume_raw = cols.next()?;
    let side_raw = cols.next()?;

    let ts_ms: i64 = ts_raw.trim().parse().ok()?;
    let price: f64 = price_raw.trim().parse().ok()?;
    let qty: f64 = volume_raw.trim().parse().ok()?;
    let is_buyer_maker = match side_raw.trim() {
        "sell" => true,
        "buy" => false,
        _ => return None,
    };

    Some(RawTrade { ts_ms, price, qty, is_buyer_maker })
}

/// Binance daily aggTrades (`ArchiveSchema::BinanceAggTrades`) row parser —
/// shared by both futures USDⓈ-M (7 cols, header present) and spot (8 cols,
/// no header, extra trailing `is_best_match`).
///
/// Columns: `agg_trade_id,price,quantity,first_trade_id,last_trade_id,
/// transact_time,is_buyer_maker[,is_best_match]`.
///
/// `transact_time` branches by magnitude: futures is always milliseconds
/// (13 digits); spot is milliseconds for older history (13 digits) but
/// MICROSECONDS for recent data (16 digits) — `>= 10^15` is treated as
/// microseconds and divided by 1000. `is_buyer_maker` is lowercase
/// (`true`/`false`) on futures, CAPITALIZED (`True`/`False`) on spot — both
/// accepted case-insensitively.
///
/// `is_buyer_maker` maps directly to `RawTrade::is_buyer_maker` — this is
/// already the exact field name/polarity Binance uses on the wire and the
/// same polarity `agg_trade_point_from` assumes elsewhere in this crate.
pub fn parse_binance_agg_trades_row(line: &str) -> Option<RawTrade> {
    let mut cols = line.split(',');
    let _agg_trade_id = cols.next()?;
    let price_raw = cols.next()?;
    let qty_raw = cols.next()?;
    let _first_trade_id = cols.next()?;
    let _last_trade_id = cols.next()?;
    let ts_raw = cols.next()?;
    let is_buyer_maker_raw = cols.next()?;

    let price: f64 = price_raw.trim().parse().ok()?;
    let qty: f64 = qty_raw.trim().parse().ok()?;
    let ts_raw_num: i64 = ts_raw.trim().parse().ok()?;
    // >= 10^15 ⇒ 16-digit microsecond timestamp (recent Binance spot dumps).
    let ts_ms = if ts_raw_num >= 1_000_000_000_000_000 {
        ts_raw_num / 1000
    } else {
        ts_raw_num
    };
    let is_buyer_maker = match is_buyer_maker_raw.trim().to_ascii_lowercase().as_str() {
        "true" => true,
        "false" => false,
        _ => return None,
    };

    Some(RawTrade { ts_ms, price, qty, is_buyer_maker })
}

/// True if `line` looks like a column-name header row rather than data —
/// used to skip the header line when `ArchiveSchema` declares one present.
/// Cheap heuristic: header rows start with an alphabetic column-name
/// character (`t`imestamp, `i`d, `a`gg_trade_id); data rows start with a
/// digit or `-` (negative not expected here, but harmless to allow).
pub fn looks_like_header(line: &str) -> bool {
    line.trim_start()
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Bybit derivatives/linear — verbatim sample row ────────────────────

    #[test]
    fn bybit_trading_row_parses_sell_taker() {
        let line = "1781481604.4,TONUSDT,Sell,3.2,1.7524,MinusTick,9560dc3e-902d-5ebf-af47-9376c75e538c,5.60768e+08,3.2,5.60768,0";
        let row = parse_bybit_trading_row(line).expect("should parse");
        assert_eq!(row.ts_ms, 1_781_481_604_400);
        assert_eq!(row.price, 1.7524);
        assert_eq!(row.qty, 3.2);
        assert!(row.is_buyer_maker, "Sell taker => buyer was maker");
    }

    #[test]
    fn bybit_trading_row_fractional_seconds_round_correctly() {
        let line = "1781481604.5766,TONUSDT,Buy,1.0,1.75,ZeroPlusTick,id,0,1.0,1.75,0";
        let row = parse_bybit_trading_row(line).expect("should parse");
        assert_eq!(row.ts_ms, 1_781_481_604_577);
        assert!(!row.is_buyer_maker, "Buy taker => buyer was taker, not maker");
    }

    #[test]
    fn bybit_trading_row_rejects_garbage_side() {
        let line = "1781481604.4,TONUSDT,Neutral,3.2,1.7524,MinusTick,id,0,3.2,5.6,0";
        assert!(parse_bybit_trading_row(line).is_none());
    }

    // ── Bybit spot — verbatim sample row ──────────────────────────────────

    #[test]
    fn bybit_spot_row_parses_sell_lowercase() {
        let line = "1,1783296639146,0.0045,229.3,sell,0";
        let row = parse_bybit_spot_row(line).expect("should parse");
        assert_eq!(row.ts_ms, 1_783_296_639_146);
        assert_eq!(row.price, 0.0045);
        assert_eq!(row.qty, 229.3);
        assert!(row.is_buyer_maker);
    }

    #[test]
    fn bybit_spot_row_parses_buy_lowercase() {
        let line = "2,1783296639200,0.0046,10.0,buy,0";
        let row = parse_bybit_spot_row(line).expect("should parse");
        assert!(!row.is_buyer_maker);
    }

    // ── Binance futures aggTrades — verbatim sample row, header present ───

    #[test]
    fn binance_futures_row_parses_ms_timestamp() {
        let line = "128590373,0.004509,1165.0,268529307,268529307,1783296000156,false";
        let row = parse_binance_agg_trades_row(line).expect("should parse");
        assert_eq!(row.ts_ms, 1_783_296_000_156);
        assert_eq!(row.price, 0.004509);
        assert_eq!(row.qty, 1165.0);
        assert!(!row.is_buyer_maker);
    }

    #[test]
    fn binance_futures_header_row_is_skipped_by_looks_like_header() {
        let header = "agg_trade_id,price,quantity,first_trade_id,last_trade_id,transact_time,is_buyer_maker";
        assert!(looks_like_header(header));
        let data = "128590373,0.004509,1165.0,268529307,268529307,1783296000156,false";
        assert!(!looks_like_header(data));
    }

    // ── Binance spot aggTrades — verbatim sample row, µs timestamp, no header ─

    #[test]
    fn binance_spot_row_branches_microseconds_to_ms() {
        let line = "36641855,0.00451000,24750.00000000,66239811,66239811,1783296012668208,True,True";
        let row = parse_binance_agg_trades_row(line).expect("should parse");
        // 1783296012668208 / 1000 = 1783296012668 (µs → ms)
        assert_eq!(row.ts_ms, 1_783_296_012_668);
        assert_eq!(row.price, 0.00451);
        assert_eq!(row.qty, 24750.0);
        assert!(row.is_buyer_maker, "True (capitalized) must parse case-insensitively");
    }

    #[test]
    fn binance_spot_row_legacy_ms_timestamp_not_divided() {
        // Historical (pre-2025) spot dumps use 13-digit ms timestamps —
        // must NOT be divided by 1000 (magnitude branch at 10^15).
        let line = "1,0.5,1.0,1,1,1700000000000,False,True";
        let row = parse_binance_agg_trades_row(line).expect("should parse");
        assert_eq!(row.ts_ms, 1_700_000_000_000);
        assert!(!row.is_buyer_maker);
    }

    #[test]
    fn binance_row_rejects_garbage_buyer_maker() {
        let line = "1,0.5,1.0,1,1,1700000000000,maybe";
        assert!(parse_binance_agg_trades_row(line).is_none());
    }
}
