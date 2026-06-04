//! # CORS Proxy REST-host Allowlist
//!
//! Canonical allowlist of REST API hosts for the 20 wasm-eligible venues in
//! `digdigdig3`. A CORS proxy that serves the wasm build to browsers MUST
//! enforce this allowlist — forward requests ONLY to these hosts, never to
//! arbitrary destinations. This kills both SSRF and open-relay attack classes.
//!
//! ## Enforcement model
//!
//! Two supported proxy shapes both terminate at the same gate:
//!
//! * **Prefix mode** — `override_base` is the proxy prefix, e.g.
//!   `"https://my-cors-proxy.example.com"`.  The proxy receives the path
//!   `/api.binance.com/api/v3/klines?...` and must extract the first path
//!   segment as the destination host before forwarding.
//! * **Encoded `{url}` mode** — `assemble_rest_url` encodes the full real URL
//!   into the `?url=…` parameter.  The proxy must decode that parameter and
//!   validate its host before forwarding.
//!
//! In both cases use [`is_allowed_rest_host`] to check the extracted host
//! (exact, case-insensitive, no suffix matching).
//!
//! ## Maintenance
//!
//! **This list is CURATED and MUST be kept in sync** whenever a connector's
//! `endpoints.rs` introduces or changes a base URL constant.  Source files to
//! watch (one entry per venue):
//!
//! | Venue | Source |
//! |---|---|
//! | Binance | `src/l3/open/crypto/cex/binance/endpoints.rs` |
//! | Bybit | `src/l3/open/crypto/cex/bybit/endpoints.rs` |
//! | OKX | `src/l3/open/crypto/cex/okx/endpoints.rs` |
//! | Bitget | `src/l3/open/crypto/cex/bitget/endpoints.rs` |
//! | Bitstamp | `src/l3/open/crypto/cex/bitstamp/endpoints.rs` |
//! | Coinbase | `src/l3/open/crypto/cex/coinbase/endpoints.rs` |
//! | Kraken | `src/l3/open/crypto/cex/kraken/endpoints.rs` |
//! | Deribit | `src/l3/open/crypto/cex/deribit/endpoints.rs` |
//! | HTX | `src/l3/open/crypto/cex/htx/endpoints.rs` |
//! | KuCoin | `src/l3/open/crypto/cex/kucoin/endpoints.rs` |
//! | MEXC | `src/l3/open/crypto/cex/mexc/endpoints.rs` |
//! | GateIO | `src/l3/open/crypto/cex/gateio/endpoints.rs` |
//! | Gemini | `src/l3/open/crypto/cex/gemini/endpoints.rs` |
//! | BingX | `src/l3/open/crypto/cex/bingx/endpoints.rs` |
//! | CryptoCom | `src/l3/open/crypto/cex/crypto_com/endpoints.rs` |
//! | Upbit | `src/l3/open/crypto/cex/upbit/endpoints.rs` |
//! | Bitfinex | `src/l3/open/crypto/cex/bitfinex/endpoints.rs` |
//! | HyperLiquid | `src/l3/open/crypto/dex/hyperliquid/endpoints.rs` |
//! | dYdX | `src/l3/open/crypto/dex/dydx/endpoints.rs` |
//! | Lighter | `src/l3/open/crypto/dex/lighter/endpoints.rs` |

/// Canonical REST-host allowlist for the 20 wasm-eligible venues.
///
/// Hosts are lowercase, scheme-stripped, path-stripped (host only).
/// Alphabetically sorted. Use [`is_allowed_rest_host`] to query.
pub const REST_HOST_ALLOWLIST: &[&str] = &[
    // BingX — all account types share one base
    "open-api.bingx.com",

    // Binance — spot + USDT-margined futures + coin-margined futures
    "api.binance.com",
    "dapi.binance.com",
    "fapi.binance.com",
    // Binance TESTNET (mark with comment; no separate bool flag needed at runtime)
    "testapi.binance.vision",    // testnet spot
    "testnet.binancefuture.com", // testnet futures (USDT-M + COIN-M)

    // Bitfinex — public REST + authenticated REST (split hosts)
    "api-pub.bitfinex.com",
    "api.bitfinex.com",

    // Bitget — single base for both spot and futures
    "api.bitget.com",

    // Bitstamp
    "www.bitstamp.net",

    // Bybit
    "api.bybit.com",
    "api-testnet.bybit.com", // testnet

    // Coinbase — brokerage REST + v2 (deposits/withdrawals) share the same apex
    "api.coinbase.com",

    // CryptoCom
    "api.crypto.com",
    "uat-api.3ona.co", // testnet (UAT sandbox)

    // Deribit
    "www.deribit.com",
    "test.deribit.com", // testnet

    // dYdX v4 Indexer
    "indexer.dydx.trade",
    "indexer.v4testnet.dydx.exchange", // testnet

    // GateIO — spot REST + futures REST
    "api.gateio.ws",
    "fx-api.gateio.ws",
    "api-testnet.gateapi.io",    // testnet spot
    "fx-api-testnet.gateio.ws",  // testnet futures

    // Gemini
    "api.gemini.com",
    "api.sandbox.gemini.com", // testnet

    // HTX (formerly Huobi) — spot REST + derivatives REST + AWS-optimised spot
    "api.hbdm.com",
    "api.huobi.pro",
    "api-aws.huobi.pro", // AWS-optimised endpoint

    // HyperLiquid
    "api.hyperliquid.xyz",
    "api.hyperliquid-testnet.xyz", // testnet

    // Kraken — spot REST + futures REST
    "api.kraken.com",
    "demo-futures.kraken.com", // futures testnet (demo)
    "futures.kraken.com",

    // KuCoin — spot REST + futures REST
    "api-futures.kucoin.com",
    "api-sandbox-futures.kucoin.com", // testnet futures
    "api.kucoin.com",
    "openapi-sandbox.kucoin.com", // testnet spot

    // Lighter
    "mainnet.zklighter.elliot.ai",
    "testnet.zklighter.elliot.ai", // testnet

    // MEXC — spot REST + futures REST
    "api.mexc.com",
    "contract.mexc.com",

    // OKX — mainnet and demo trading both use the same REST host
    "www.okx.com",

    // Upbit — Korea main + regional (Singapore, Indonesia, Thailand)
    "api.upbit.com",
    "id-api.upbit.com",
    "sg-api.upbit.com",
    "th-api.upbit.com",
];

/// Return the full allowlist slice.
///
/// Equivalent to dereferencing [`REST_HOST_ALLOWLIST`]; provided as a
/// function for callers that want a stable fn-pointer to pass around.
pub fn rest_host_allowlist() -> &'static [&'static str] {
    REST_HOST_ALLOWLIST
}

/// Return `true` iff `host` is an exact (case-insensitive) member of
/// [`REST_HOST_ALLOWLIST`].
///
/// **Security note**: this is an EXACT match, not a suffix match.
/// `"api.binance.com.evil.com"` is NOT accepted even though it contains
/// `"api.binance.com"` as a substring. Do not relax to suffix matching.
pub fn is_allowed_rest_host(host: &str) -> bool {
    if host.is_empty() {
        return false;
    }
    let lower = host.to_ascii_lowercase();
    REST_HOST_ALLOWLIST
        .iter()
        .any(|&allowed| allowed == lower.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn list_is_non_empty() {
        assert!(!REST_HOST_ALLOWLIST.is_empty());
    }

    #[test]
    fn no_duplicates() {
        let set: HashSet<&str> = REST_HOST_ALLOWLIST.iter().copied().collect();
        assert_eq!(
            set.len(),
            REST_HOST_ALLOWLIST.len(),
            "REST_HOST_ALLOWLIST contains duplicate entries"
        );
    }

    #[test]
    fn known_hosts_present() {
        assert!(
            is_allowed_rest_host("api.binance.com"),
            "api.binance.com must be in allowlist"
        );
        assert!(
            is_allowed_rest_host("api.kucoin.com"),
            "api.kucoin.com must be in allowlist"
        );
        assert!(
            is_allowed_rest_host("indexer.dydx.trade"),
            "indexer.dydx.trade must be in allowlist"
        );
    }

    #[test]
    fn case_insensitive_match() {
        assert!(is_allowed_rest_host("API.BINANCE.COM"));
        assert!(is_allowed_rest_host("Api.Kucoin.Com"));
        assert!(is_allowed_rest_host("INDEXER.DYDX.TRADE"));
    }

    #[test]
    fn ssrf_candidates_blocked() {
        // Private/link-local ranges
        assert!(!is_allowed_rest_host("169.254.169.254")); // AWS metadata
        assert!(!is_allowed_rest_host("localhost"));
        assert!(!is_allowed_rest_host("127.0.0.1"));

        // Arbitrary external host
        assert!(!is_allowed_rest_host("evil.com"));

        // Suffix-confusion attacks — must NOT match even though they contain
        // a known host as a substring
        assert!(!is_allowed_rest_host("api.binance.com.evil.com"));
        assert!(!is_allowed_rest_host("fake-api.binance.com"));

        // Empty string
        assert!(!is_allowed_rest_host(""));
    }

    #[test]
    fn all_entries_are_lowercase() {
        for &host in REST_HOST_ALLOWLIST {
            assert_eq!(
                host,
                host.to_ascii_lowercase(),
                "Entry '{host}' is not all-lowercase — allowlist must use lowercase hostnames"
            );
        }
    }
}
