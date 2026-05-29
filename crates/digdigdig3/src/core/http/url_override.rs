//! # REST URL assembly with optional override
//!
//! Centralises the three URL-building modes used by wasm-eligible connectors:
//!
//! 1. **Encoded-proxy template** — override contains the literal `{url}`:
//!    percent-encode the full real target (`real_base + path + query`) and
//!    substitute it for `{url}`.
//!    Example: `"https://api.allorigins.win/raw?url={url}"` →
//!    `"https://api.allorigins.win/raw?url=https%3A%2F%2Fapi.binance.com%2Fapi%2Fv3%2Fklines%3F..."`.
//!
//! 2. **Prefix mode** — override does NOT contain `{url}`:
//!    `override + path + query`.
//!    Preserves the old behaviour exactly.
//!
//! 3. **No override** — use real base:
//!    `real_base + path + query`.

use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};

/// Characters that must be percent-encoded when a full URL is used as a query
/// parameter value.  This is the "query" encode set from the WHATWG URL spec
/// minus the characters that `CONTROLS` already covers, plus `: / ? & = # + @`.
///
/// We intentionally encode `:`, `/`, `?`, `&`, `=` so that the proxy can
/// parse its own `?url=` parameter correctly.
const URL_COMPONENT: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'<')
    .add(b'>')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'`')
    .add(b'{')
    .add(b'|')
    .add(b'}')
    // path/query delimiters that must be encoded inside a query param value
    .add(b':')
    .add(b'/')
    .add(b'?')
    .add(b'&')
    .add(b'=')
    .add(b'+')
    .add(b'@');

/// Assemble the final REST URL honouring an optional base override.
///
/// - `override_base` contains `"{url}"` → **encoded-proxy template**: percent-encode
///   the full real target (`real_base + path + query`) and substitute.
/// - `override_base` without `"{url}"` → **prefix mode**: `override + path + query`
///   (unchanged behaviour for existing callers).
/// - `None` → `real_base + path + query`.
///
/// `query` must be either an empty string or already include the leading `?`
/// (e.g. `"?symbol=BTCUSDT&limit=1"`).
pub fn assemble_rest_url(
    override_base: Option<&str>,
    real_base: &str,
    path: &str,
    query: &str,
) -> String {
    match override_base {
        Some(ov) if ov.contains("{url}") => {
            let target = format!("{real_base}{path}{query}");
            let encoded = utf8_percent_encode(&target, URL_COMPONENT).to_string();
            ov.replace("{url}", &encoded)
        }
        Some(ov) => format!("{ov}{path}{query}"),
        None => format!("{real_base}{path}{query}"),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    const REAL_BASE: &str = "https://api.binance.com";
    const PATH: &str = "/api/v3/klines";
    const QUERY: &str = "?symbol=BTCUSDT&interval=1m&limit=1";

    // ── No override ────────────────────────────────────────────────────────────

    #[test]
    fn no_override_returns_real_url() {
        let url = assemble_rest_url(None, REAL_BASE, PATH, QUERY);
        assert_eq!(url, "https://api.binance.com/api/v3/klines?symbol=BTCUSDT&interval=1m&limit=1");
    }

    #[test]
    fn no_override_empty_path_query() {
        let url = assemble_rest_url(None, REAL_BASE, "", "");
        assert_eq!(url, "https://api.binance.com");
    }

    // ── Prefix mode (no {url}) ─────────────────────────────────────────────────

    #[test]
    fn prefix_mode_prepends_override() {
        let ov = "https://corsproxy.io/?https%3A%2F%2Fapi.binance.com";
        let url = assemble_rest_url(Some(ov), REAL_BASE, PATH, QUERY);
        assert_eq!(
            url,
            "https://corsproxy.io/?https%3A%2F%2Fapi.binance.com/api/v3/klines?symbol=BTCUSDT&interval=1m&limit=1"
        );
    }

    #[test]
    fn prefix_mode_empty_query() {
        let ov = "https://my-proxy.example.com";
        let url = assemble_rest_url(Some(ov), REAL_BASE, "/api/v3/time", "");
        assert_eq!(url, "https://my-proxy.example.com/api/v3/time");
    }

    // ── Encoded-proxy template mode ({url}) ────────────────────────────────────

    #[test]
    fn encoded_proxy_encodes_full_target() {
        let ov = "https://api.allorigins.win/raw?url={url}";
        let url = assemble_rest_url(Some(ov), REAL_BASE, PATH, QUERY);
        // Full target: "https://api.binance.com/api/v3/klines?symbol=BTCUSDT&interval=1m&limit=1"
        // After percent-encoding that becomes:
        //   https%3A%2F%2Fapi.binance.com%2Fapi%2Fv3%2Fklines%3Fsymbol%3DBTCUSDT%26interval%3D1m%26limit%3D1
        assert!(url.starts_with("https://api.allorigins.win/raw?url=https%3A%2F%2F"), "unexpected: {url}");
        assert!(url.contains("%2Fapi%2Fv3%2Fklines"), "path not encoded: {url}");
        assert!(url.contains("%3Fsymbol%3DBTCUSDT"), "query not encoded: {url}");
        assert!(url.contains("%26interval%3D1m"), "& not encoded: {url}");
    }

    #[test]
    fn encoded_proxy_no_query() {
        let ov = "https://api.allorigins.win/raw?url={url}";
        let url = assemble_rest_url(Some(ov), REAL_BASE, "/api/v3/time", "");
        assert_eq!(
            url,
            "https://api.allorigins.win/raw?url=https%3A%2F%2Fapi.binance.com%2Fapi%2Fv3%2Ftime"
        );
    }

    #[test]
    fn encoded_proxy_placeholder_replaced_not_doubled() {
        let ov = "https://proxy.example/{url}";
        let url = assemble_rest_url(Some(ov), "https://ex.com", "/path", "");
        // Must contain exactly one occurrence of the encoded base, no literal {url}
        assert!(!url.contains("{url}"), "literal {{url}} must be replaced");
        assert!(url.contains("https%3A%2F%2Fex.com"), "encoded base must appear");
    }

    // ── Colon / slash encoding verification ────────────────────────────────────

    #[test]
    fn colon_and_slash_are_encoded() {
        let ov = "https://p.test?url={url}";
        let url = assemble_rest_url(Some(ov), "https://api.ex.com", "/v1/data", "?a=1&b=2");
        // ':' → %3A, '/' → %2F, '?' → %3F, '&' → %26, '=' → %3D
        assert!(url.contains("%3A"), "colon must be encoded");
        assert!(url.contains("%2F"), "slash must be encoded");
        assert!(url.contains("%3F"), "? must be encoded");
        assert!(url.contains("%26"), "& must be encoded");
        assert!(url.contains("%3D"), "= must be encoded");
    }
}
