//! Live integration test: Bitstamp Stream::OrderbookL3 emits create + changed/delete events.
//!
//! Subscribes to BTC-USD on Bitstamp via `live_orders_btcusd` (after REST snapshot bootstrap)
//! and collects events for up to 60 seconds.
//!
//! Asserts:
//!   - At least 1 event with action == "create" (REST snapshot or live order_created)
//!   - At least 1 event with action == "changed" or "delete" (live incremental)
//!   - All events have symbol == "BTCUSD"
//!   - All events have timestamp > 0
//!   - All order_id fields are non-empty
//!
//! Gated with `--ignored`. Run with:
//!   cargo test -p digdigdig3-station --test bitstamp_l3_live -- --ignored --nocapture

use std::time::Duration;

use digdigdig3_station::{AccountType, Event, ExchangeId, Station, Stream, SubscriptionSet};

#[tokio::test]
#[ignore = "live API, requires network"]
async fn bitstamp_btcusd_l3_emits_create_and_changed_events() {
    let station = Station::builder().build().await.expect("Station::build");

    let set = SubscriptionSet::new().add_raw(
        ExchangeId::Bitstamp,
        "btcusd",
        AccountType::Spot,
        [Stream::OrderbookL3],
    );

    let mut report = station.subscribe(set).await.expect("subscribe");
    assert!(
        report.failed.is_empty(),
        "subscribe should not fail for Bitstamp BTC-USD OrderbookL3: {:?}",
        report.failed
    );

    let mut create_count = 0usize;
    let mut changed_count = 0usize;
    let mut delete_count = 0usize;
    let mut bad_symbol = 0usize;
    let mut zero_ts = 0usize;
    let mut empty_order_id = 0usize;
    let mut total = 0usize;

    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
    loop {
        if tokio::time::Instant::now() >= deadline {
            break;
        }
        let remaining = deadline - tokio::time::Instant::now();
        let r = tokio::time::timeout(
            remaining.min(Duration::from_secs(2)),
            report.handle.recv(),
        )
        .await;
        match r {
            Ok(Some(Event::OrderbookL3 { symbol, point, .. })) => {
                total += 1;
                // Station relabels events with the user's raw input string ("btcusd");
                // compare case-insensitively since the wire format uppercases but
                // add_raw() lowercases.
                if !symbol.eq_ignore_ascii_case("BTCUSD") {
                    bad_symbol += 1;
                }
                if point.ts_ms <= 0 {
                    zero_ts += 1;
                }
                if point.order_id.is_empty() {
                    empty_order_id += 1;
                }
                match point.action.as_str() {
                    "create" => create_count += 1,
                    "changed" => changed_count += 1,
                    "delete" => delete_count += 1,
                    other => eprintln!("unexpected action: {:?}", other),
                }
                println!(
                    "[{total}] action={} side={:?} order_id={} price={} qty={} ts={}",
                    point.action, point.side, point.order_id, point.price, point.quantity, point.ts_ms
                );
                // Stop early once we have at least one create and at least one changed/delete
                if create_count > 0 && (changed_count + delete_count) > 0 {
                    break;
                }
            }
            Ok(Some(_other_event)) => {
                // Not an OrderbookL3 event — ignore
            }
            Ok(None) => break, // channel closed
            Err(_) => continue, // timeout, keep looping until deadline
        }
    }

    println!(
        "total={total} create={create_count} changed={changed_count} delete={delete_count} \
         bad_symbol={bad_symbol} zero_ts={zero_ts} empty_order_id={empty_order_id}"
    );

    assert!(create_count > 0, "no 'create' events in 60s (got {total} total)");
    assert!(
        changed_count + delete_count > 0,
        "no 'changed' or 'delete' events in 60s (got {total} total, {create_count} creates)"
    );
    assert_eq!(bad_symbol, 0, "some events had wrong symbol (expected BTCUSD case-insensitive, got non-BTCUSD)");
    assert_eq!(zero_ts, 0, "some events had zero/negative timestamp");
    assert_eq!(empty_order_id, 0, "some events had empty order_id");
}
