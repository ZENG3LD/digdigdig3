//! # canonical_demo — Phase λ.A canonical event normalization demo.
//!
//! Demonstrates canonical typed fields from constructed StreamEvents,
//! and shows how to wire normalization into a live WS event loop.
//!
//! Run:
//!     cargo run --example canonical_demo
//!
//! No API keys required — uses synthesized events to show the API.

use digdigdig3::core::normalization::{CanonicalEvent, Canonicalize};
use digdigdig3::core::types::{PublicTrade, StreamEvent, Ticker, TradeSide};

fn main() {
    println!("── canonical_demo: Phase λ.A normalization ─────────────────\n");

    // Synthesize a trade event (as if from any exchange WebSocket)
    let raw_trade = StreamEvent::Trade {
        symbol: "BTCUSDT".to_string(),
        trade: PublicTrade {
            id: "1234567890".to_string(),
            price: 67_432.1,
            quantity: 0.015,
            side: TradeSide::Buy,
            // Exchange sends 10-digit seconds — normalization converts to ms
            timestamp: 1_700_000_000,
        },
    };

    if let Some(CanonicalEvent::Trade(t)) = raw_trade.canonicalize() {
        println!("[trade]");
        println!("  symbol       = {}", t.symbol);
        println!("  price        = {}", t.price);   // Decimal — exact, no f64 rounding
        println!("  quantity     = {}", t.quantity);
        println!("  side         = {:?}", t.side);
        println!("  timestamp_ms = {}", t.timestamp_ms); // was 1_700_000_000 s → 1_700_000_000_000 ms
        println!("  trade_id     = {:?}\n", t.trade_id);
    }

    // Synthesize a ticker event (bid/ask optional — some exchanges omit them)
    let raw_ticker = StreamEvent::Ticker {
        symbol: "ETHUSDT".to_string(),
        ticker: Ticker {
            last_price: 3_215.5,
            bid_price: Some(3_215.0),
            ask_price: Some(3_216.0),
            high_24h: None,
            low_24h: None,
            volume_24h: Some(45_321.7),
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h: None,
            // Exchange sends 16-digit microseconds — normalization converts to ms
            timestamp: 1_700_000_000_000_000,
        },
    };

    if let Some(CanonicalEvent::Ticker(t)) = raw_ticker.canonicalize() {
        println!("[ticker]");
        println!("  symbol       = {}", t.symbol);
        println!("  last_price   = {}", t.last_price);
        println!("  bid_price    = {:?}", t.bid_price);
        println!("  ask_price    = {:?}", t.ask_price);
        println!("  volume_24h   = {:?}", t.volume_24h);
        println!("  timestamp_ms = {}\n", t.timestamp_ms); // was µs → ms
    }

    // Show the Other variant for events we don't canonicalize yet
    let raw_funding = StreamEvent::FundingRate {
        symbol: "BTCUSDT".to_string(),
        rate: 0.0001,
        next_funding_time: None,
        timestamp: 1_700_000_000_000,
    };

    match raw_funding.canonicalize() {
        Some(CanonicalEvent::Other) => println!("[funding_rate] → CanonicalEvent::Other (not yet mapped)"),
        other => println!("[funding_rate] unexpected: {:?}", other),
    }

    println!("\n── Done. ────────────────────────────────────────────────────");
    println!("To wire into live WS events:");
    println!("  use futures_util::StreamExt;");
    println!("  while let Some(Ok(event)) = ws.event_stream().next().await {{");
    println!("      if let Some(canonical) = event.canonicalize() {{ ... }}");
    println!("  }}");
}
