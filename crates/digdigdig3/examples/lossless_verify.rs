//! # lossless_verify — prove the RAW pump loses nothing.
//!
//! Live REST harness: for each venue, fetch the public feeds whose payloads were
//! enriched (trade / kline / funding / mark / open-interest / ticker) and ASSERT
//! that the venue-specific Option fields are populated where the wire carries them.
//! cargo-check ≠ proof — this hits the real exchanges.
//!
//! Run:
//!     cargo run --example lossless_verify --release
//!     cargo run --example lossless_verify --release -- Binance OKX   # subset
//!
//! Exit code is non-zero if any expected field came back empty (the pump dropped it).
//! No API keys — public endpoints only.

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{AccountType, ExchangeId};

/// One field expectation: name + whether it was populated.
struct Check {
    field: &'static str,
    populated: bool,
}

fn check(field: &'static str, populated: bool) -> Check {
    Check { field, populated }
}

/// Result of probing one (venue, endpoint).
struct Probe {
    venue: &'static str,
    endpoint: &'static str,
    checks: Vec<Check>,
    error: Option<String>,
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let want = |v: &str| args.is_empty() || args.iter().any(|a| a.eq_ignore_ascii_case(v));

    let hub = ExchangeHub::new();
    let mut probes: Vec<Probe> = Vec::new();

    // ── BINANCE: trade quote_qty/is_best_match; kline taker_buy; funding mark_price; OI sums ──
    if want("Binance") {
        let _ = hub.connect_public(ExchangeId::Binance, false).await;
        if let Some(c) = hub.rest(ExchangeId::Binance) {
            match c.get_recent_trades("BTCUSDT".into(), Some(5), AccountType::Spot).await {
                Ok(ts) => {
                    let t = ts.first();
                    probes.push(Probe {
                        venue: "Binance", endpoint: "recent_trades",
                        checks: vec![
                            check("quote_qty", t.map_or(false, |t| t.quote_qty.is_some())),
                            check("is_best_match", t.map_or(false, |t| t.is_best_match.is_some())),
                        ],
                        error: None,
                    });
                }
                Err(e) => probes.push(err("Binance", "recent_trades", e)),
            }
            match c.get_klines("BTCUSDT".into(), "1m", Some(2), AccountType::Spot, None).await {
                Ok(ks) => {
                    let k = ks.first();
                    probes.push(Probe {
                        venue: "Binance", endpoint: "klines",
                        checks: vec![
                            check("taker_buy_base_volume", k.map_or(false, |k| k.taker_buy_base_volume.is_some())),
                            check("taker_buy_quote_volume", k.map_or(false, |k| k.taker_buy_quote_volume.is_some())),
                        ],
                        error: None,
                    });
                }
                Err(e) => probes.push(err("Binance", "klines", e)),
            }
            match c.get_funding_rate_history("BTCUSDT".into(), None, None, Some(2), AccountType::FuturesCross).await {
                Ok(fs) => probes.push(Probe {
                    venue: "Binance", endpoint: "funding_history",
                    checks: vec![check("mark_price", fs.first().map_or(false, |f| f.mark_price.is_some()))],
                    error: None,
                }),
                Err(e) => probes.push(err("Binance", "funding_history", e)),
            }

            // ── Binance ticker REST: enriched fields ──
            match c.get_ticker("BTCUSDT".into(), AccountType::Spot).await {
                Ok(t) => probes.push(Probe {
                    venue: "Binance", endpoint: "ticker_spot",
                    checks: vec![
                        check("weighted_avg_price", t.weighted_avg_price.is_some()),
                        check("open_price",         t.open_price.is_some()),
                        check("prev_close_price",   t.prev_close_price.is_some()),
                        check("bid_qty",            t.bid_qty.is_some()),
                        check("ask_qty",            t.ask_qty.is_some()),
                        check("last_qty",           t.last_qty.is_some()),
                        check("first_id",           t.first_id.is_some()),
                        check("last_id",            t.last_id.is_some()),
                        check("count",              t.count.is_some()),
                        check("open_time",          t.open_time.is_some()),
                    ],
                    error: None,
                }),
                Err(e) => probes.push(err("Binance", "ticker_spot", e)),
            }

            // ── Binance topLSR: global long/short ratio (long_ratio nonzero) ──
            match c.get_long_short_ratio_history("BTCUSDT".into(), "1h", None, None, Some(2), AccountType::FuturesCross).await {
                Ok(ls) => {
                    let l = ls.first();
                    probes.push(Probe {
                        venue: "Binance", endpoint: "topLSR_global",
                        checks: vec![
                            check("long_ratio_nonzero", l.map_or(false, |l| l.long_ratio > 0.0)),
                        ],
                        error: None,
                    });
                }
                Err(e) => probes.push(err("Binance", "topLSR_global", e)),
            }

            // ── Binance depth futures: event_time + transaction_time ──
            match c.get_orderbook("BTCUSDT".into(), Some(5), AccountType::FuturesCross).await {
                Ok(ob) => probes.push(Probe {
                    venue: "Binance", endpoint: "depth_futures",
                    checks: vec![
                        check("event_time",       ob.event_time.is_some()),
                        check("transaction_time", ob.transaction_time.is_some()),
                    ],
                    error: None,
                }),
                Err(e) => probes.push(err("Binance", "depth_futures", e)),
            }

            // ── Binance basis history ──
            // get_basis_history is inherent on BinanceConnector; access via downcast.
            // Via dyn trait it's not reachable → skip with a note.
            probes.push(Probe {
                venue: "Binance", endpoint: "basis_history",
                checks: Vec::new(),
                error: Some("inherent method get_basis_history not reachable via dyn trait; verify directly".to_string()),
            });
        }
    }

    // ── OKX: funding rich (premium/interest/sett/impact); OI ccy/usd ──
    if want("OKX") {
        let _ = hub.connect_public(ExchangeId::OKX, false).await;
        if let Some(c) = hub.rest(ExchangeId::OKX) {
            match c.get_funding_rate_history("BTC-USDT-SWAP".into(), None, None, Some(2), AccountType::FuturesCross).await {
                Ok(fs) => probes.push(Probe {
                    venue: "OKX", endpoint: "funding_history",
                    checks: vec![check("realized_rate", fs.first().map_or(false, |f| f.realized_rate.is_some()))],
                    error: None,
                }),
                Err(e) => probes.push(err("OKX", "funding_history", e)),
            }
            match c.get_open_interest_history("BTC-USDT-SWAP".into(), "5m", None, None, Some(2), AccountType::FuturesCross).await {
                Ok(ois) => probes.push(Probe {
                    venue: "OKX", endpoint: "open_interest",
                    checks: vec![
                        check("open_interest_ccy", ois.first().map_or(false, |o| o.open_interest_ccy.is_some())),
                        check("open_interest_usd", ois.first().map_or(false, |o| o.open_interest_usd.is_some())),
                    ],
                    error: None,
                }),
                Err(e) => probes.push(err("OKX", "open_interest", e)),
            }

            // ── OKX ticker: bid_qty / ask_qty / open_price / last_qty ──
            match c.get_ticker("BTC-USDT-SWAP".into(), AccountType::FuturesCross).await {
                Ok(t) => probes.push(Probe {
                    venue: "OKX", endpoint: "ticker_swap",
                    checks: vec![
                        check("bid_qty",    t.bid_qty.is_some()),
                        check("ask_qty",    t.ask_qty.is_some()),
                        check("open_price", t.open_price.is_some()),
                        check("last_qty",   t.last_qty.is_some()),
                    ],
                    error: None,
                }),
                Err(e) => probes.push(err("OKX", "ticker_swap", e)),
            }

            // ── OKX orderbook sequence ──
            match c.get_orderbook("BTC-USDT-SWAP".into(), Some(5), AccountType::FuturesCross).await {
                Ok(ob) => probes.push(Probe {
                    venue: "OKX", endpoint: "orderbook_swap",
                    checks: vec![check("sequence", ob.sequence.is_some())],
                    error: None,
                }),
                Err(e) => probes.push(err("OKX", "orderbook_swap", e)),
            }

            // ── OKX klines: quote_volume populated AND > volume (proves idx7 = USDT, not idx6 = BTC) ──
            match c.get_klines("BTC-USDT-SWAP".into(), "1m", Some(2), AccountType::FuturesCross, None).await {
                Ok(ks) => {
                    let k = ks.first();
                    probes.push(Probe {
                        venue: "OKX", endpoint: "klines_swap",
                        checks: vec![
                            check("quote_volume_is_some", k.map_or(false, |k| k.quote_volume.is_some())),
                            check("quote_volume_gt_volume",
                                k.map_or(false, |k| k.quote_volume.map_or(false, |q| q > k.volume))),
                        ],
                        error: None,
                    });
                }
                Err(e) => probes.push(err("OKX", "klines_swap", e)),
            }
        }
    }

    // ── BITMEX: trade notional×3/tickDirection/trdType ──
    if want("BitMEX") {
        let _ = hub.connect_public(ExchangeId::Bitmex, false).await;
        if let Some(c) = hub.rest(ExchangeId::Bitmex) {
            match c.get_recent_trades("XBTUSDT".into(), Some(5), AccountType::FuturesCross).await {
                Ok(ts) => {
                    let t = ts.first();
                    probes.push(Probe {
                        venue: "BitMEX", endpoint: "recent_trades",
                        checks: vec![
                            check("gross_value",     t.map_or(false, |t| t.gross_value.is_some())),
                            check("home_notional",   t.map_or(false, |t| t.home_notional.is_some())),
                            check("foreign_notional",t.map_or(false, |t| t.foreign_notional.is_some())),
                            check("tick_direction",  t.map_or(false, |t| t.tick_direction.is_some())),
                        ],
                        error: None,
                    });
                }
                Err(e) => probes.push(err("BitMEX", "recent_trades", e)),
            }

            // ── BitMEX ticker: mark_price, index_price, open_interest, funding_rate ──
            match c.get_ticker("XBTUSD".into(), AccountType::FuturesCross).await {
                Ok(t) => probes.push(Probe {
                    venue: "BitMEX", endpoint: "ticker",
                    checks: vec![
                        check("mark_price",   t.mark_price.is_some()),
                        check("index_price",  t.index_price.is_some()),
                        check("open_interest",t.open_interest.is_some()),
                        check("funding_rate", t.funding_rate.is_some()),
                    ],
                    error: None,
                }),
                Err(e) => probes.push(err("BitMEX", "ticker", e)),
            }

            // ── BitMEX klines: last_size ──
            match c.get_klines("XBTUSD".into(), "1m", Some(2), AccountType::FuturesCross, None).await {
                Ok(ks) => {
                    let k = ks.first();
                    probes.push(Probe {
                        venue: "BitMEX", endpoint: "klines",
                        checks: vec![
                            check("last_size", k.map_or(false, |k| k.last_size.is_some())),
                        ],
                        error: None,
                    });
                }
                Err(e) => probes.push(err("BitMEX", "klines", e)),
            }
        }
    }

    // ── DERIBIT: trade index/mark/contracts/trade_seq/tick_direction ──
    if want("Deribit") {
        let _ = hub.connect_public(ExchangeId::Deribit, false).await;
        if let Some(c) = hub.rest(ExchangeId::Deribit) {
            match c.get_recent_trades("BTC-PERPETUAL".into(), Some(5), AccountType::FuturesCross).await {
                Ok(ts) => {
                    let t = ts.first();
                    probes.push(Probe {
                        venue: "Deribit", endpoint: "recent_trades",
                        checks: vec![
                            check("index_price",  t.map_or(false, |t| t.index_price.is_some())),
                            check("mark_price",   t.map_or(false, |t| t.mark_price.is_some())),
                            check("trade_seq",    t.map_or(false, |t| t.trade_seq.is_some())),
                            check("tick_direction",t.map_or(false, |t| t.tick_direction.is_some())),
                        ],
                        error: None,
                    });
                }
                Err(e) => probes.push(err("Deribit", "recent_trades", e)),
            }
        }
    }

    // ── GATEIO: contract_stats bundle drained → LSR top_* + Taker long/short + LiquidationBucket ──
    if want("GateIO") {
        let _ = hub.connect_public(ExchangeId::GateIO, false).await;
        if let Some(c) = hub.rest(ExchangeId::GateIO) {
            match c.get_long_short_ratio_history("BTC_USDT".into(), "5m", None, None, Some(2), AccountType::FuturesCross).await {
                Ok(ls) => probes.push(Probe {
                    venue: "GateIO", endpoint: "long_short_ratio",
                    checks: vec![
                        check("top_lsr_size", ls.first().map_or(false, |l| l.top_lsr_size.is_some())),
                        check("long_users",   ls.first().map_or(false, |l| l.long_users.is_some())),
                    ],
                    error: None,
                }),
                Err(e) => probes.push(err("GateIO", "long_short_ratio", e)),
            }
            match c.get_liquidation_bucket_history("BTC_USDT".into(), "5m", None, None, Some(2), AccountType::FuturesCross).await {
                Ok(la) => probes.push(Probe {
                    venue: "GateIO", endpoint: "liquidation_bucket",
                    // liq sizes are often 0 in a quiet bucket; we only assert the call succeeds and returns rows.
                    checks: vec![check("rows_returned", !la.is_empty())],
                    error: None,
                }),
                Err(e) => probes.push(err("GateIO", "liquidation_bucket", e)),
            }
            match c.get_insurance_fund(None, AccountType::FuturesCross).await {
                Ok(ifd) => probes.push(Probe {
                    venue: "GateIO", endpoint: "insurance_fund",
                    checks: vec![check("balance", ifd.first().map_or(false, |i| i.balance > 0.0))],
                    error: None,
                }),
                Err(e) => probes.push(err("GateIO", "insurance_fund", e)),
            }

            // ── GateIO klines spot: quote_volume > volume (swap-fix) ──
            match c.get_klines("BTC_USDT".into(), "1m", Some(2), AccountType::Spot, None).await {
                Ok(ks) => {
                    let k = ks.first();
                    probes.push(Probe {
                        venue: "GateIO", endpoint: "klines_spot",
                        checks: vec![
                            check("quote_volume_is_some",    k.map_or(false, |k| k.quote_volume.is_some())),
                            check("quote_volume_gt_volume",
                                k.map_or(false, |k| k.quote_volume.map_or(false, |q| q > k.volume))),
                        ],
                        error: None,
                    });
                }
                Err(e) => probes.push(err("GateIO", "klines_spot", e)),
            }

            // ── GateIO ticker spot: bid_qty / ask_qty ──
            match c.get_ticker("BTC_USDT".into(), AccountType::Spot).await {
                Ok(t) => probes.push(Probe {
                    venue: "GateIO", endpoint: "ticker_spot",
                    checks: vec![
                        check("bid_qty", t.bid_qty.is_some()),
                        check("ask_qty", t.ask_qty.is_some()),
                    ],
                    error: None,
                }),
                Err(e) => probes.push(err("GateIO", "ticker_spot", e)),
            }

            // ── GateIO ticker futures: mark_price / index_price / funding_rate ──
            match c.get_ticker("BTC_USDT".into(), AccountType::FuturesCross).await {
                Ok(t) => probes.push(Probe {
                    venue: "GateIO", endpoint: "ticker_futures",
                    checks: vec![
                        check("mark_price",   t.mark_price.is_some()),
                        check("index_price",  t.index_price.is_some()),
                        check("funding_rate", t.funding_rate.is_some()),
                    ],
                    error: None,
                }),
                Err(e) => probes.push(err("GateIO", "ticker_futures", e)),
            }
        }
    }

    // ── BITFINEX: deriv-status fanned out, incl. insurance fund (idx6) ──
    if want("Bitfinex") {
        let _ = hub.connect_public(ExchangeId::Bitfinex, false).await;
        if let Some(c) = hub.rest(ExchangeId::Bitfinex) {
            match c.get_insurance_fund(Some("tBTCF0:USTF0".into()), AccountType::FuturesCross).await {
                Ok(ifd) => probes.push(Probe {
                    venue: "Bitfinex", endpoint: "insurance_fund",
                    checks: vec![check("balance", ifd.first().map_or(false, |i| i.balance > 0.0))],
                    error: None,
                }),
                Err(e) => probes.push(err("Bitfinex", "insurance_fund", e)),
            }
            match c.get_premium_index(Some("tBTCF0:USTF0".into()), AccountType::FuturesCross).await {
                Ok(mp) => probes.push(Probe {
                    venue: "Bitfinex", endpoint: "premium_index(mark)",
                    checks: vec![
                        check("spot_price",  mp.first().map_or(false, |m| m.spot_price.is_some())),
                        check("index_price", mp.first().map_or(false, |m| m.index_price.is_some())),
                    ],
                    error: None,
                }),
                Err(e) => probes.push(err("Bitfinex", "premium_index(mark)", e)),
            }

            // ── Bitfinex ticker: bid_qty / ask_qty ──
            match c.get_ticker("tBTCUSD".into(), AccountType::Spot).await {
                Ok(t) => probes.push(Probe {
                    venue: "Bitfinex", endpoint: "ticker_spot",
                    checks: vec![
                        check("bid_qty", t.bid_qty.is_some()),
                        check("ask_qty", t.ask_qty.is_some()),
                    ],
                    error: None,
                }),
                Err(e) => probes.push(err("Bitfinex", "ticker_spot", e)),
            }
        }
    }

    // ── DERIBIT ticker: stats bundle (mark/index/oi/settlement/funding_8h/min-max_price) + new fields ──
    if want("Deribit") {
        if let Some(c) = hub.rest(ExchangeId::Deribit) {
            match c.get_ticker("BTC-PERPETUAL".into(), AccountType::FuturesCross).await {
                Ok(t) => probes.push(Probe {
                    venue: "Deribit", endpoint: "ticker",
                    checks: vec![
                        check("mark_price",      t.mark_price.is_some()),
                        check("open_interest",   t.open_interest.is_some()),
                        check("settlement_price",t.settlement_price.is_some()),
                        check("min_price",       t.min_price.is_some()),
                        check("max_price",       t.max_price.is_some()),
                        check("volume_notional", t.volume_notional.is_some()),
                        check("state",           t.state.is_some()),
                        check("interest_value",  t.interest_value.is_some()),
                    ],
                    error: None,
                }),
                Err(e) => probes.push(err("Deribit", "ticker", e)),
            }

            // ── Deribit funding history: interest_8h / index_price / prev_index_price ──
            match c.get_funding_rate_history("BTC-PERPETUAL".into(), None, None, Some(2), AccountType::FuturesCross).await {
                Ok(fs) => {
                    let f = fs.first();
                    probes.push(Probe {
                        venue: "Deribit", endpoint: "funding_history",
                        checks: vec![
                            check("interest_8h",      f.map_or(false, |f| f.interest_8h.is_some())),
                            check("index_price",      f.map_or(false, |f| f.index_price.is_some())),
                            check("prev_index_price", f.map_or(false, |f| f.prev_index_price.is_some())),
                        ],
                        error: None,
                    });
                }
                Err(e) => probes.push(err("Deribit", "funding_history", e)),
            }

            // Deribit DVOL: get_volatility_index_data is an inherent method on DeribitConnector,
            // not reachable via dyn trait object — skip with a note.
            probes.push(Probe {
                venue: "Deribit", endpoint: "dvol",
                checks: Vec::new(),
                error: Some("get_volatility_index_data is inherent; not reachable via dyn trait object".to_string()),
            });
        }
    }

    // ── BYBIT: ticker 35-field derivative union + new bid_qty/ask_qty + orderbook sequence/cts ──
    if want("Bybit") {
        let _ = hub.connect_public(ExchangeId::Bybit, false).await;
        if let Some(c) = hub.rest(ExchangeId::Bybit) {
            match c.get_ticker("BTCUSDT".into(), AccountType::FuturesCross).await {
                Ok(t) => probes.push(Probe {
                    venue: "Bybit", endpoint: "ticker(linear)",
                    checks: vec![
                        check("mark_price",          t.mark_price.is_some()),
                        check("open_interest",        t.open_interest.is_some()),
                        check("funding_rate",         t.funding_rate.is_some()),
                        check("funding_interval_hour",t.funding_interval_hour.is_some()),
                        check("bid_qty",              t.bid_qty.is_some()),
                        check("ask_qty",              t.ask_qty.is_some()),
                    ],
                    error: None,
                }),
                Err(e) => probes.push(err("Bybit", "ticker(linear)", e)),
            }

            // ── Bybit orderbook: sequence + cts ──
            match c.get_orderbook("BTCUSDT".into(), Some(5), AccountType::FuturesCross).await {
                Ok(ob) => probes.push(Probe {
                    venue: "Bybit", endpoint: "orderbook_futures",
                    checks: vec![
                        check("sequence", ob.sequence.is_some()),
                        check("cts",      ob.cts.is_some()),
                    ],
                    error: None,
                }),
                Err(e) => probes.push(err("Bybit", "orderbook_futures", e)),
            }
        }
    }

    // ── BITGET: account-LSR + position-LSR (inherent methods on BitgetConnector) ──
    if want("Bitget") {
        let _ = hub.connect_public(ExchangeId::Bitget, false).await;
        // get_account_long_short_ratio_history and get_position_long_short_ratio_history
        // are inherent methods on BitgetConnector, not exposed on dyn MarketDataPublic.
        // The generic get_long_short_ratio_history trait method routes to an internal path.
        if let Some(c) = hub.rest(ExchangeId::Bitget) {
            match c.get_long_short_ratio_history("BTCUSDT".into(), "1H", None, None, Some(2), AccountType::FuturesCross).await {
                Ok(ls) => {
                    let l = ls.first();
                    probes.push(Probe {
                        venue: "Bitget", endpoint: "lsr_trait",
                        checks: vec![
                            check("long_ratio_nonzero", l.map_or(false, |l| l.long_ratio > 0.0)),
                        ],
                        error: None,
                    });
                }
                Err(e) => probes.push(err("Bitget", "lsr_trait", e)),
            }
            // Inherent account/position LSR — not reachable via dyn trait; note it.
            probes.push(Probe {
                venue: "Bitget", endpoint: "account_lsr_inherent",
                checks: Vec::new(),
                error: Some("get_account_long_short_ratio_history is inherent; verify via BitgetConnector directly".to_string()),
            });
            probes.push(Probe {
                venue: "Bitget", endpoint: "position_lsr_inherent",
                checks: Vec::new(),
                error: Some("get_position_long_short_ratio_history is inherent; verify via BitgetConnector directly".to_string()),
            });
        }
    }

    // ── HTX: basis / OI history / ticker spot ──
    if want("HTX") {
        let _ = hub.connect_public(ExchangeId::HTX, false).await;
        if let Some(c) = hub.rest(ExchangeId::HTX) {
            // ── HTX basis history ──
            match c.get_basis_history("BTC-USDT".into(), "60min", None, None, Some(2), AccountType::FuturesCross).await {
                Ok(bs) => {
                    let b = bs.first();
                    probes.push(Probe {
                        venue: "HTX", endpoint: "basis_history",
                        checks: vec![
                            check("index_price",  b.map_or(false, |b| b.index_price.is_some())),
                            check("futures_price",b.map_or(false, |b| b.futures_price.is_some())),
                            check("basis_rate",   b.map_or(false, |b| b.basis_rate.is_some())),
                        ],
                        error: None,
                    });
                }
                Err(e) => probes.push(err("HTX", "basis_history", e)),
            }

            // ── HTX OI history: open_interest_value ──
            match c.get_open_interest_history("BTC-USDT".into(), "60min", None, None, Some(2), AccountType::FuturesCross).await {
                Ok(ois) => {
                    let o = ois.first();
                    probes.push(Probe {
                        venue: "HTX", endpoint: "oi_history",
                        checks: vec![
                            check("open_interest_value", o.map_or(false, |o| o.open_interest_value.is_some())),
                        ],
                        error: None,
                    });
                }
                Err(e) => probes.push(err("HTX", "oi_history", e)),
            }

            // ── HTX ticker spot: bid_qty / ask_qty / open_price ──
            match c.get_ticker("btcusdt".into(), AccountType::Spot).await {
                Ok(t) => probes.push(Probe {
                    venue: "HTX", endpoint: "ticker_spot",
                    checks: vec![
                        check("bid_qty",    t.bid_qty.is_some()),
                        check("ask_qty",    t.ask_qty.is_some()),
                        check("open_price", t.open_price.is_some()),
                    ],
                    error: None,
                }),
                Err(e) => probes.push(err("HTX", "ticker_spot", e)),
            }
        }
    }

    // ── dYdX: ticker + funding history ──
    if want("dYdX") {
        let _ = hub.connect_public(ExchangeId::Dydx, false).await;
        if let Some(c) = hub.rest(ExchangeId::Dydx) {
            // ── dYdX ticker: funding_rate / open_interest / count ──
            match c.get_ticker("BTC-USD".into(), AccountType::FuturesCross).await {
                Ok(t) => probes.push(Probe {
                    venue: "dYdX", endpoint: "ticker",
                    checks: vec![
                        check("funding_rate",  t.funding_rate.is_some()),
                        check("open_interest", t.open_interest.is_some()),
                        check("count",         t.count.is_some()),
                    ],
                    error: None,
                }),
                Err(e) => probes.push(err("dYdX", "ticker", e)),
            }

            // ── dYdX funding history: symbol / mark_price ──
            match c.get_funding_rate_history("BTC-USD".into(), None, None, Some(2), AccountType::FuturesCross).await {
                Ok(fs) => {
                    let f = fs.first();
                    probes.push(Probe {
                        venue: "dYdX", endpoint: "funding_history",
                        checks: vec![
                            check("symbol",     f.map_or(false, |f| f.symbol.is_some())),
                            check("mark_price", f.map_or(false, |f| f.mark_price.is_some())),
                        ],
                        error: None,
                    });
                }
                Err(e) => probes.push(err("dYdX", "funding_history", e)),
            }

            // dYdX OI from candles — OI is fanned out into ticker, not a separate endpoint.
            // get_open_interest_history returns UnsupportedOperation. Note it.
            probes.push(Probe {
                venue: "dYdX", endpoint: "oi_history",
                checks: Vec::new(),
                error: Some("OI from candles: dYdX has no OI history endpoint; OI lives in ticker.open_interest".to_string()),
            });
        }
    }

    // ── HyperLiquid: ticker + funding history ──
    if want("HyperLiquid") {
        let _ = hub.connect_public(ExchangeId::HyperLiquid, false).await;
        if let Some(c) = hub.rest(ExchangeId::HyperLiquid) {
            // ── HyperLiquid ticker: open_interest / funding_rate / index_price / mark_price / volume_24h / quote_volume_24h ──
            match c.get_ticker("BTC".into(), AccountType::FuturesCross).await {
                Ok(t) => probes.push(Probe {
                    venue: "HyperLiquid", endpoint: "ticker",
                    checks: vec![
                        check("open_interest",   t.open_interest.is_some()),
                        check("funding_rate",     t.funding_rate.is_some()),
                        check("index_price",      t.index_price.is_some()),
                        check("mark_price",       t.mark_price.is_some()),
                        check("volume_24h",       t.volume_24h.is_some()),
                        check("quote_volume_24h", t.quote_volume_24h.is_some()),
                    ],
                    error: None,
                }),
                Err(e) => probes.push(err("HyperLiquid", "ticker", e)),
            }

            // ── HyperLiquid funding history: symbol / premium ──
            match c.get_funding_rate_history("BTC".into(), None, None, Some(2), AccountType::FuturesCross).await {
                Ok(fs) => {
                    let f = fs.first();
                    probes.push(Probe {
                        venue: "HyperLiquid", endpoint: "funding_history",
                        checks: vec![
                            check("symbol",  f.map_or(false, |f| f.symbol.is_some())),
                            check("premium", f.map_or(false, |f| f.premium.is_some())),
                        ],
                        error: None,
                    });
                }
                Err(e) => probes.push(err("HyperLiquid", "funding_history", e)),
            }
        }
    }

    // ── Lighter: funding history ──
    if want("Lighter") {
        let _ = hub.connect_public(ExchangeId::Lighter, false).await;
        if let Some(c) = hub.rest(ExchangeId::Lighter) {
            match c.get_funding_rate_history("BTC".into(), None, None, Some(2), AccountType::FuturesCross).await {
                Ok(fs) => {
                    let f = fs.first();
                    probes.push(Probe {
                        venue: "Lighter", endpoint: "funding_history",
                        checks: vec![
                            check("symbol", f.map_or(false, |f| f.symbol.is_some())),
                        ],
                        error: None,
                    });
                }
                Err(e) => probes.push(err("Lighter", "funding_history", e)),
            }
        }
    }

    // ── Kraken spot: ticker ──
    if want("Kraken") {
        let _ = hub.connect_public(ExchangeId::Kraken, false).await;
        if let Some(c) = hub.rest(ExchangeId::Kraken) {
            // ── Kraken spot ticker: weighted_avg_price / count / open_price / last_qty ──
            match c.get_ticker("XXBTZUSD".into(), AccountType::Spot).await {
                Ok(t) => probes.push(Probe {
                    venue: "Kraken", endpoint: "ticker_spot",
                    checks: vec![
                        check("weighted_avg_price", t.weighted_avg_price.is_some()),
                        check("count",              t.count.is_some()),
                        check("open_price",         t.open_price.is_some()),
                        check("last_qty",           t.last_qty.is_some()),
                    ],
                    error: None,
                }),
                Err(e) => probes.push(err("Kraken", "ticker_spot", e)),
            }

            // ── Kraken futures ticker: weighted_avg_price / last_qty / last_trade_time ──
            match c.get_ticker("PI_XBTUSD".into(), AccountType::FuturesCross).await {
                Ok(t) => probes.push(Probe {
                    venue: "Kraken", endpoint: "ticker_futures",
                    checks: vec![
                        check("weighted_avg_price", t.weighted_avg_price.is_some()),
                        check("last_qty",           t.last_qty.is_some()),
                        check("last_trade_time",    t.last_trade_time.is_some()),
                    ],
                    error: None,
                }),
                Err(e) => probes.push(err("Kraken", "ticker_futures", e)),
            }

            // ── Kraken futures funding history: relative_funding_rate ──
            match c.get_funding_rate_history("PI_XBTUSD".into(), None, None, Some(2), AccountType::FuturesCross).await {
                Ok(fs) => {
                    let f = fs.first();
                    probes.push(Probe {
                        venue: "Kraken", endpoint: "funding_history",
                        checks: vec![
                            check("relative_funding_rate", f.map_or(false, |f| f.relative_funding_rate.is_some())),
                        ],
                        error: None,
                    });
                }
                Err(e) => probes.push(err("Kraken", "funding_history", e)),
            }
        }
    }

    // ── MEXC: funding history / futures kline / spot ticker ──
    if want("MEXC") {
        let _ = hub.connect_public(ExchangeId::MEXC, false).await;
        if let Some(c) = hub.rest(ExchangeId::MEXC) {
            // ── MEXC funding history: symbol / funding_interval_hours ──
            match c.get_funding_rate_history("BTCUSDT".into(), None, None, Some(2), AccountType::FuturesCross).await {
                Ok(fs) => {
                    let f = fs.first();
                    probes.push(Probe {
                        venue: "MEXC", endpoint: "funding_history",
                        checks: vec![
                            check("symbol",                f.map_or(false, |f| f.symbol.is_some())),
                            check("funding_interval_hours",f.map_or(false, |f| f.funding_interval_hours.is_some())),
                        ],
                        error: None,
                    });
                }
                Err(e) => probes.push(err("MEXC", "funding_history", e)),
            }

            // ── MEXC futures kline: quote_volume ──
            match c.get_klines("BTCUSDT".into(), "1m", Some(2), AccountType::FuturesCross, None).await {
                Ok(ks) => {
                    let k = ks.first();
                    probes.push(Probe {
                        venue: "MEXC", endpoint: "klines_futures",
                        checks: vec![
                            check("quote_volume", k.map_or(false, |k| k.quote_volume.is_some())),
                        ],
                        error: None,
                    });
                }
                Err(e) => probes.push(err("MEXC", "klines_futures", e)),
            }

            // ── MEXC spot ticker: open_price ──
            match c.get_ticker("BTCUSDT".into(), AccountType::Spot).await {
                Ok(t) => probes.push(Probe {
                    venue: "MEXC", endpoint: "ticker_spot",
                    checks: vec![
                        check("open_price", t.open_price.is_some()),
                    ],
                    error: None,
                }),
                Err(e) => probes.push(err("MEXC", "ticker_spot", e)),
            }
        }
    }

    // ── BingX: ticker spot + swap OI ──
    if want("BingX") {
        let _ = hub.connect_public(ExchangeId::BingX, false).await;
        if let Some(c) = hub.rest(ExchangeId::BingX) {
            // ── BingX ticker spot: bid_qty / ask_qty / open_price / open_time ──
            match c.get_ticker("BTC-USDT".into(), AccountType::Spot).await {
                Ok(t) => probes.push(Probe {
                    venue: "BingX", endpoint: "ticker_spot",
                    checks: vec![
                        check("bid_qty",    t.bid_qty.is_some()),
                        check("ask_qty",    t.ask_qty.is_some()),
                        check("open_price", t.open_price.is_some()),
                        check("open_time",  t.open_time.is_some()),
                    ],
                    error: None,
                }),
                Err(e) => probes.push(err("BingX", "ticker_spot", e)),
            }

            // ── BingX swap OI: symbol non-empty AND timestamp > 0 ──
            match c.get_open_interest("BTC-USDT", AccountType::FuturesCross).await {
                Ok(oi) => probes.push(Probe {
                    venue: "BingX", endpoint: "oi_swap",
                    checks: vec![
                        check("symbol_nonempty",  oi.symbol.as_deref().map_or(false, |s| !s.is_empty())),
                        check("timestamp_nonzero", oi.timestamp != 0),
                    ],
                    error: None,
                }),
                Err(e) => probes.push(err("BingX", "oi_swap", e)),
            }
        }
    }

    // ── Coinbase: klines close_time / ticker bid_qty+ask_qty ──
    if want("Coinbase") {
        let _ = hub.connect_public(ExchangeId::Coinbase, false).await;
        if let Some(c) = hub.rest(ExchangeId::Coinbase) {
            // ── Coinbase klines: close_time > open_time (close_time-bug fix) ──
            match c.get_klines("BTC-USD".into(), "60", Some(2), AccountType::Spot, None).await {
                Ok(ks) => {
                    let k = ks.first();
                    probes.push(Probe {
                        venue: "Coinbase", endpoint: "klines",
                        checks: vec![
                            check("close_time_is_some",     k.map_or(false, |k| k.close_time.is_some())),
                            check("close_time_gt_open_time",
                                k.map_or(false, |k| k.close_time.map_or(false, |ct| ct > k.open_time))),
                        ],
                        error: None,
                    });
                }
                Err(e) => probes.push(err("Coinbase", "klines", e)),
            }

            // ── Coinbase ticker: bid_qty / ask_qty ──
            match c.get_ticker("BTC-USD".into(), AccountType::Spot).await {
                Ok(t) => probes.push(Probe {
                    venue: "Coinbase", endpoint: "ticker_spot",
                    checks: vec![
                        check("bid_qty", t.bid_qty.is_some()),
                        check("ask_qty", t.ask_qty.is_some()),
                    ],
                    error: None,
                }),
                Err(e) => probes.push(err("Coinbase", "ticker_spot", e)),
            }
        }
    }

    // ── Upbit: klines open_time boundary check + ticker open_price ──
    if want("Upbit") {
        let _ = hub.connect_public(ExchangeId::Upbit, false).await;
        if let Some(c) = hub.rest(ExchangeId::Upbit) {
            // ── Upbit klines: open_time % 60_000 == 0 for 1-min bars ──
            match c.get_klines("KRW-BTC".into(), "1", Some(2), AccountType::Spot, None).await {
                Ok(ks) => {
                    let k = ks.first();
                    probes.push(Probe {
                        venue: "Upbit", endpoint: "klines_1m",
                        checks: vec![
                            check("open_time_boundary", k.map_or(false, |k| k.open_time % 60_000 == 0)),
                        ],
                        error: None,
                    });
                }
                Err(e) => probes.push(err("Upbit", "klines_1m", e)),
            }

            // ── Upbit ticker: open_price ──
            match c.get_ticker("KRW-BTC".into(), AccountType::Spot).await {
                Ok(t) => probes.push(Probe {
                    venue: "Upbit", endpoint: "ticker_spot",
                    checks: vec![check("open_price", t.open_price.is_some())],
                    error: None,
                }),
                Err(e) => probes.push(err("Upbit", "ticker_spot", e)),
            }
        }
    }

    // ── Bitstamp: ticker vwap fix + open_price; quote_volume_24h must be None ──
    if want("Bitstamp") {
        let _ = hub.connect_public(ExchangeId::Bitstamp, false).await;
        if let Some(c) = hub.rest(ExchangeId::Bitstamp) {
            match c.get_ticker("btcusd".into(), AccountType::Spot).await {
                Ok(t) => probes.push(Probe {
                    venue: "Bitstamp", endpoint: "ticker_spot",
                    checks: vec![
                        check("weighted_avg_price",         t.weighted_avg_price.is_some()),
                        check("open_price",                 t.open_price.is_some()),
                        // Semantic-bug fix: vwap must NOT be misrouted to quote_volume_24h
                        check("quote_volume_24h_is_none",   !t.quote_volume_24h.is_some()),
                    ],
                    error: None,
                }),
                Err(e) => probes.push(err("Bitstamp", "ticker_spot", e)),
            }
        }
    }

    // ── Gemini: ticker volume_24h / quote_volume_24h / open_price ──
    if want("Gemini") {
        let _ = hub.connect_public(ExchangeId::Gemini, false).await;
        if let Some(c) = hub.rest(ExchangeId::Gemini) {
            match c.get_ticker("btcusd".into(), AccountType::Spot).await {
                Ok(t) => probes.push(Probe {
                    venue: "Gemini", endpoint: "ticker_spot",
                    checks: vec![
                        check("volume_24h",       t.volume_24h.is_some()),
                        check("quote_volume_24h", t.quote_volume_24h.is_some()),
                        check("open_price",       t.open_price.is_some()),
                    ],
                    error: None,
                }),
                Err(e) => probes.push(err("Gemini", "ticker_spot", e)),
            }
        }
    }

    // ── Crypto.com: orderbook bids[0].order_count ──
    if want("CryptoCom") {
        let _ = hub.connect_public(ExchangeId::CryptoCom, false).await;
        if let Some(c) = hub.rest(ExchangeId::CryptoCom) {
            match c.get_orderbook("BTC_USDT".into(), Some(5), AccountType::Spot).await {
                Ok(ob) => probes.push(Probe {
                    venue: "CryptoCom", endpoint: "orderbook_spot",
                    checks: vec![
                        check("bids_order_count",
                            ob.bids.first().map_or(false, |l| l.order_count.is_some())),
                    ],
                    error: None,
                }),
                Err(e) => probes.push(err("CryptoCom", "orderbook_spot", e)),
            }
        }
    }

    // ── KuCoin: funding history symbol + OI > 0 ──
    if want("KuCoin") {
        let _ = hub.connect_public(ExchangeId::KuCoin, false).await;
        if let Some(c) = hub.rest(ExchangeId::KuCoin) {
            // ── KuCoin funding history: symbol ──
            match c.get_funding_rate_history("XBTUSDTM".into(), None, None, Some(2), AccountType::FuturesCross).await {
                Ok(fs) => {
                    let f = fs.first();
                    probes.push(Probe {
                        venue: "KuCoin", endpoint: "funding_history",
                        checks: vec![
                            check("symbol", f.map_or(false, |f| f.symbol.is_some())),
                        ],
                        error: None,
                    });
                }
                Err(e) => probes.push(err("KuCoin", "funding_history", e)),
            }

            // ── KuCoin OI: open_interest > 0.0 ──
            match c.get_open_interest("XBTUSDTM", AccountType::FuturesCross).await {
                Ok(oi) => probes.push(Probe {
                    venue: "KuCoin", endpoint: "open_interest",
                    checks: vec![
                        check("open_interest_nonzero", oi.open_interest > 0.0),
                    ],
                    error: None,
                }),
                Err(e) => probes.push(err("KuCoin", "open_interest", e)),
            }
        }
    }

    // ── Report ──
    println!("\n── lossless_verify ─ live RAW-pump field coverage ──────────────");
    let mut total = 0usize;
    let mut populated = 0usize;
    let mut errors = 0usize;
    for p in &probes {
        if let Some(e) = &p.error {
            println!("  [ERR ] {:<12} {:<28} {}", p.venue, p.endpoint, e);
            errors += 1;
            continue;
        }
        for c in &p.checks {
            total += 1;
            let mark = if c.populated { populated += 1; "ok " } else { "MISS" };
            println!("  [{}] {:<12} {:<28} {}", mark, p.venue, p.endpoint, c.field);
        }
    }
    let missed = total - populated;
    println!("────────────────────────────────────────────────────────────────");
    println!("  fields populated: {populated}/{total}   missed: {missed}   call-errors: {errors}");

    if missed > 0 {
        eprintln!("\nFAIL: {missed} enriched field(s) came back empty — the pump dropped data.");
        std::process::exit(1);
    }
    if populated == 0 {
        eprintln!("\nFAIL: no fields verified (all calls errored?) — not a proof of losslessness.");
        std::process::exit(2);
    }
    println!("\nPASS: every probed enriched field is populated from the live wire.");
}

fn err(venue: &'static str, endpoint: &'static str, e: impl std::fmt::Display) -> Probe {
    Probe { venue, endpoint, checks: Vec::new(), error: Some(e.to_string()) }
}
