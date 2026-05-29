//! OPFS `SettingsStore` wasm32 round-trip test.
//!
//! Verifies that `SettingsStore::open`, `set`, `save`, and re-`open` + `get`
//! produce consistent results against the Origin Private File System in a
//! real browser context.
//!
//! Run with:
//!   cargo test --target wasm32-unknown-unknown -p digdigdig3-station \
//!       --test wasm_settings_store
//!
//! Requires: dig2-wasm-test runner (configured in .cargo/config.toml) +
//!           a browser with OPFS support (Chrome 86+, Firefox 111+, Safari 15.2+).

#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

use digdigdig3_station::SettingsStore;
use serde::{Deserialize, Serialize};

/// Small struct to exercise typed round-trips over OPFS.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct PanelConfig {
    visible: bool,
    width: u32,
    title: String,
}

// ─── Test: full OPFS settings round-trip ─────────────────────────────────────

/// Write several typed values, save to OPFS, re-open same namespace, assert
/// all values survive.
///
/// Uses a distinctive namespace so it does not collide with other tests. OPFS
/// persists across page reloads; re-running this test over stale data is safe
/// because the test overwrites (not appends) on each `save()`.
#[wasm_bindgen_test]
async fn settings_opfs_round_trip() {
    let ns = "dig3-settings-wasm-test-v1";

    let panel = PanelConfig {
        visible: true,
        width: 320,
        title: "indicators".to_string(),
    };

    // Write phase.
    {
        let mut store = SettingsStore::open(ns)
            .await
            .expect("SettingsStore::open must succeed in browser OPFS");

        store.set("theme", &"dark").expect("set theme");
        store.set("zoom", &150u32).expect("set zoom");
        store.set("panel", &panel).expect("set panel");

        store.save().await.expect("save must succeed");

        web_sys::console::log_1(
            &format!("[wasm_settings_store] saved: theme=dark zoom=150 panel={panel:?}").into(),
        );
    }

    // Read-back phase (new store instance, same namespace).
    {
        let store = SettingsStore::open(ns)
            .await
            .expect("SettingsStore::open (re-open) must succeed");

        let theme: Option<String> = store.get("theme");
        let zoom: Option<u32> = store.get("zoom");
        let panel_back: Option<PanelConfig> = store.get("panel");

        web_sys::console::log_1(
            &format!(
                "[wasm_settings_store] read back: theme={theme:?} zoom={zoom:?} panel={panel_back:?}"
            )
            .into(),
        );

        assert_eq!(
            theme,
            Some("dark".to_string()),
            "theme must survive OPFS round-trip"
        );
        assert_eq!(zoom, Some(150u32), "zoom must survive OPFS round-trip");
        assert_eq!(
            panel_back,
            Some(panel),
            "panel struct must survive OPFS round-trip"
        );
    }
}
