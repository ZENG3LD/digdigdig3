//! TopicRegistry — maps (StreamKind, AccountType) → (TopicPattern, ParserFn).
//!
//! ## Matching strategy
//!
//! Single-star wildcard covering one or more contiguous characters.
//! Examples:
//!   - `*@trade`         — Binance trades
//!   - `publicTrade.*`   — Bybit trades
//!   - `market.*.depth`  — HTX depth topics

use std::collections::HashMap;

use serde_json::Value;

use crate::core::types::{AccountType, StreamEvent, WebSocketError, WebSocketResult};

use super::stream_kind::StreamKind;

// ─────────────────────────────────────────────────────────────────────────────
// TopicKey
// ─────────────────────────────────────────────────────────────────────────────

/// Raw topic string extracted from a frame (e.g. "btcusdt@trade", "trades.BTC-USDT-SWAP").
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TopicKey(pub String);

impl TopicKey {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for TopicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// TopicPattern
// ─────────────────────────────────────────────────────────────────────────────

/// A pattern used to register parsers.  Supports a single `*` wildcard.
///
/// Examples:
///   `"*@trade"`          — matches any Binance trade topic
///   `"publicTrade.*"`    — matches any Bybit trade topic
///   `"market.*.depth"`   — matches any HTX depth topic with symbol in segment 2
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TopicPattern(pub String);

impl TopicPattern {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Returns true if this pattern matches the given key.
    /// At most one `*` wildcard is supported.
    pub fn matches(&self, key: &TopicKey) -> bool {
        topic_pattern_matches(&self.0, &key.0)
    }
}

impl std::fmt::Display for TopicPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Pattern matching supporting multiple `*` wildcards.
///
/// Each `*` matches zero or more characters (greedy left-to-right).
/// If no `*`, requires exact equality.
///
/// Examples:
///   `"*@trade"`         matches `"btcusdt@trade"`
///   `"*@depth20@*"`     matches `"btcusdt@depth20@100ms"`
///   `"market.*.depth"`  matches `"market.BTC-USDT.depth"`
pub fn topic_pattern_matches(pattern: &str, key: &str) -> bool {
    // Split pattern on `*` to get literal segments.
    // Invariant: segments[0] is a prefix anchor, segments[last] is a suffix anchor,
    // inner segments must appear in order (greedy scan).
    let segments: Vec<&str> = pattern.split('*').collect();

    match segments.len() {
        // No `*` — exact match.
        1 => pattern == key,

        n => {
            // First segment: must be an anchored prefix.
            let prefix = segments[0];
            if !key.starts_with(prefix) {
                return false;
            }

            // Last segment: must be an anchored suffix.
            let suffix = segments[n - 1];
            if !suffix.is_empty() && !key.ends_with(suffix) {
                return false;
            }

            // Middle segments (if any): scan left-to-right from after prefix.
            let mut pos = prefix.len();
            // The suffix will be consumed from the right — don't scan past it.
            let limit = key.len().saturating_sub(suffix.len());

            for seg in &segments[1..n - 1] {
                if seg.is_empty() {
                    // Consecutive `*`s — skip (empty segment matches nothing extra).
                    continue;
                }
                // Find `seg` in key[pos..limit].
                match key[pos..limit].find(seg) {
                    Some(off) => pos += off + seg.len(),
                    None => return false,
                }
            }

            // Verify the consumed prefix + middle + suffix regions don't overlap.
            pos <= limit + suffix.len()
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ParserFn
// ─────────────────────────────────────────────────────────────────────────────

/// Parse a raw JSON frame into a StreamEvent.
/// Receives the full frame Value so parsers can read any field.
pub type ParserFn = fn(&Value) -> WebSocketResult<StreamEvent>;

// ─────────────────────────────────────────────────────────────────────────────
// RegistryKey / RegistryEntry
// ─────────────────────────────────────────────────────────────────────────────

/// Registry key: (stream kind, account type).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RegistryKey {
    pub kind: StreamKind,
    pub account_type: AccountType,
}

/// One registered entry: the wire topic pattern + parser function.
#[derive(Clone)]
pub struct RegistryEntry {
    pub pattern: TopicPattern,
    pub parser: ParserFn,
}

// ─────────────────────────────────────────────────────────────────────────────
// TopicRegistry
// ─────────────────────────────────────────────────────────────────────────────

/// Maps (StreamKind, AccountType) → (TopicPattern, ParserFn).
///
/// Also maintains a flat dispatch list of (TopicPattern, ParserFn) for O(patterns)
/// per-frame dispatch (typically 5-40 patterns per exchange).
///
/// Immutable after construction — built once via `TopicRegistryBuilder`.
pub struct TopicRegistry {
    /// Primary map: per (kind, account) what pattern + parser.
    entries: HashMap<RegistryKey, RegistryEntry>,

    /// Flattened list of (pattern, parser) for frame dispatch.
    /// Built once at construction; not mutated at runtime.
    dispatch: Vec<(TopicPattern, ParserFn)>,
}

impl TopicRegistry {
    pub fn builder() -> TopicRegistryBuilder {
        TopicRegistryBuilder::default()
    }

    /// Look up a parser for an incoming frame's topic key.
    /// Returns the first pattern that matches.  O(patterns).
    pub fn dispatch(&self, key: &TopicKey) -> Option<ParserFn> {
        for (pattern, parser) in &self.dispatch {
            if pattern.matches(key) {
                return Some(*parser);
            }
        }
        None
    }

    /// Look up ALL parsers whose pattern matches the topic key.
    ///
    /// Used when multiple StreamKind entries share the same wire topic
    /// (e.g. Bybit linear `tickers.*` carries Ticker + MarkPrice + FundingRate + OpenInterest).
    /// Returns parsers in registration order (de-duplicated by pointer identity).
    pub fn dispatch_all(&self, key: &TopicKey) -> Vec<ParserFn> {
        let mut out: Vec<ParserFn> = Vec::new();
        for (pattern, parser) in &self.dispatch {
            if pattern.matches(key) {
                // De-duplicate by function pointer (avoids calling same fn twice when
                // the same parser is registered under multiple StreamKind keys).
                let ptr = *parser as usize;
                if !out.iter().any(|p| *p as usize == ptr) {
                    out.push(*parser);
                }
            }
        }
        out
    }

    /// Returns true if (kind, account) has a registered parser.
    pub fn supports(&self, kind: &StreamKind, account: AccountType) -> bool {
        let key = RegistryKey {
            kind: kind.clone(),
            account_type: account,
        };
        self.entries.contains_key(&key)
    }

    /// Returns all (kind, account) pairs with Native support.
    pub fn native_pairs(&self) -> impl Iterator<Item = (&StreamKind, AccountType)> + '_ {
        self.entries
            .keys()
            .map(|k| (&k.kind, k.account_type))
    }

    /// Returns a reference to all raw entries (for capability introspection).
    pub fn entries(&self) -> &HashMap<RegistryKey, RegistryEntry> {
        &self.entries
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// TopicRegistryBuilder
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct TopicRegistryBuilder {
    entries: Vec<(RegistryKey, RegistryEntry)>,
}

impl TopicRegistryBuilder {
    /// Register a (kind, account_type, pattern, parser) entry.
    pub fn register(
        mut self,
        kind: StreamKind,
        account_type: AccountType,
        pattern: impl Into<String>,
        parser: ParserFn,
    ) -> Self {
        let key = RegistryKey { kind, account_type };
        let entry = RegistryEntry {
            pattern: TopicPattern::new(pattern),
            parser,
        };
        self.entries.push((key, entry));
        self
    }

    pub fn build(self) -> TopicRegistry {
        // Build dispatch list (de-dup patterns that appear multiple times is intentional
        // — same pattern for spot + futures is allowed; first match wins per dispatch).
        let mut dispatch: Vec<(TopicPattern, ParserFn)> = Vec::new();
        let mut map: HashMap<RegistryKey, RegistryEntry> = HashMap::new();

        for (key, entry) in self.entries {
            // Add to dispatch list (all patterns, including duplicates from different keys)
            dispatch.push((entry.pattern.clone(), entry.parser));
            map.insert(key, entry);
        }

        TopicRegistry {
            entries: map,
            dispatch,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Error type helper (used to create WebSocketError in parser return)
// ─────────────────────────────────────────────────────────────────────────────

/// Helper: create a parse error for a field that is missing.
pub fn missing_field(field: &str) -> WebSocketError {
    WebSocketError::Parse(format!("missing field: {}", field))
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_match() {
        assert!(topic_pattern_matches("spot.trades", "spot.trades"));
        assert!(!topic_pattern_matches("spot.trades", "spot.trade"));
    }

    #[test]
    fn suffix_wildcard() {
        // "*@trade" matches "btcusdt@trade" and "ethusdt@trade"
        assert!(topic_pattern_matches("*@trade", "btcusdt@trade"));
        assert!(topic_pattern_matches("*@trade", "ethusdt@trade"));
        assert!(!topic_pattern_matches("*@trade", "btcusdt@kline_1m"));
    }

    #[test]
    fn prefix_wildcard() {
        // "publicTrade.*" matches "publicTrade.BTCUSDT"
        assert!(topic_pattern_matches("publicTrade.*", "publicTrade.BTCUSDT"));
        assert!(topic_pattern_matches("publicTrade.*", "publicTrade.ETHUSDT"));
        assert!(!topic_pattern_matches("publicTrade.*", "orderbook.BTCUSDT"));
    }

    #[test]
    fn mid_wildcard() {
        // "market.*.trade.detail" matches "market.BTC-USDT.trade.detail"
        assert!(topic_pattern_matches(
            "market.*.trade.detail",
            "market.BTC-USDT.trade.detail"
        ));
        assert!(!topic_pattern_matches(
            "market.*.trade.detail",
            "market.BTC-USDT.depth"
        ));
    }

    #[test]
    fn topic_key_display() {
        let key = TopicKey::new("btcusdt@trade");
        assert_eq!(key.to_string(), "btcusdt@trade");
    }

    #[test]
    fn registry_dispatch() {
        fn dummy_parser(_v: &Value) -> WebSocketResult<StreamEvent> {
            Err(WebSocketError::Parse("test".into()))
        }

        let registry = TopicRegistry::builder()
            .register(
                StreamKind::Trade,
                AccountType::Spot,
                "*@trade",
                dummy_parser,
            )
            .build();

        let key = TopicKey::new("btcusdt@trade");
        assert!(registry.dispatch(&key).is_some());

        let miss = TopicKey::new("btcusdt@kline_1m");
        assert!(registry.dispatch(&miss).is_none());
    }

    #[test]
    fn registry_supports() {
        fn dummy_parser(_v: &Value) -> WebSocketResult<StreamEvent> {
            Err(WebSocketError::Parse("test".into()))
        }

        let registry = TopicRegistry::builder()
            .register(
                StreamKind::Trade,
                AccountType::Spot,
                "*@trade",
                dummy_parser,
            )
            .build();

        assert!(registry.supports(&StreamKind::Trade, AccountType::Spot));
        assert!(!registry.supports(&StreamKind::Ticker, AccountType::Spot));
    }
}
