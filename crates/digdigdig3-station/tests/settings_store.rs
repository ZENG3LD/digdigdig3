//! Native round-trip tests for `SettingsStore` (file-backed).
//!
//! Run with:
//!   cargo test -p digdigdig3-station --test settings_store

use digdigdig3_station::{SettingsError, SettingsStore};
use serde::{Deserialize, Serialize};

/// Small struct for round-trip testing of arbitrary types.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Layout {
    columns: u8,
    rows: u8,
    pinned: bool,
    label: String,
}

// ─── helpers ──────────────────────────────────────────────────────────────────

/// Unique temp dir per test invocation (pid + test name suffix).
fn tmp_dir(suffix: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "dig3-settings-{}-{}",
        std::process::id(),
        suffix
    ))
}

// ─── Test 1: basic round-trip ─────────────────────────────────────────────────

#[tokio::test]
async fn settings_round_trip() {
    let dir = tmp_dir("round-trip");
    let _ = std::fs::remove_dir_all(&dir);

    let layout = Layout {
        columns: 3,
        rows: 4,
        pinned: true,
        label: "main".to_string(),
    };

    // Write phase.
    {
        let mut store = SettingsStore::open(&dir, "dig3-settings-test")
            .await
            .expect("open must succeed");

        store.set("theme", &"dark".to_string()).expect("set theme");
        store.set("max_bars", &5000u32).expect("set max_bars");
        store.set("layout", &layout).expect("set layout");
        store.save().await.expect("save must succeed");
    }

    // Read-back phase.
    {
        let store = SettingsStore::open(&dir, "dig3-settings-test")
            .await
            .expect("re-open must succeed");

        assert_eq!(store.get::<String>("theme"), Some("dark".to_string()));
        assert_eq!(store.get::<u32>("max_bars"), Some(5000u32));
        assert_eq!(store.get::<Layout>("layout"), Some(layout));
    }

    // Cleanup.
    let _ = std::fs::remove_dir_all(&dir);
}

// ─── Test 2: remove / contains / keys ────────────────────────────────────────

#[tokio::test]
async fn settings_remove_contains_keys() {
    let dir = tmp_dir("remove-keys");
    let _ = std::fs::remove_dir_all(&dir);

    let mut store = SettingsStore::open(&dir, "dig3-settings-test")
        .await
        .expect("open");

    store.set("a", &1u32).unwrap();
    store.set("b", &2u32).unwrap();
    store.set("c", &3u32).unwrap();

    assert!(store.contains("a"));
    assert!(store.contains("b"));
    assert!(store.contains("c"));

    let mut ks = store.keys();
    ks.sort();
    assert_eq!(ks, vec!["a", "b", "c"]);

    // Remove "b".
    assert!(store.remove("b"), "remove must return true for existing key");
    assert!(!store.contains("b"), "b must be gone");
    assert!(!store.remove("b"), "second remove must return false");

    let mut ks2 = store.keys();
    ks2.sort();
    assert_eq!(ks2, vec!["a", "c"]);

    let _ = std::fs::remove_dir_all(&dir);
}

// ─── Test 3: missing key → None, wrong type → None ───────────────────────────

#[tokio::test]
async fn settings_missing_and_wrong_type() {
    let dir = tmp_dir("none-cases");
    let _ = std::fs::remove_dir_all(&dir);

    let mut store = SettingsStore::open(&dir, "dig3-settings-test")
        .await
        .expect("open");

    store.set("num", &42u32).unwrap();

    // Missing key → None.
    assert_eq!(store.get::<String>("no-such-key"), None);

    // Wrong type (stored u32, read as String) → None, no panic.
    assert_eq!(store.get::<String>("num"), None);

    let _ = std::fs::remove_dir_all(&dir);
}

// ─── Test 4: corrupt file → Err, not silent wipe ─────────────────────────────

#[tokio::test]
async fn settings_corrupt_file_returns_err() {
    let dir = tmp_dir("corrupt");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let file_path = dir.join("dig3-settings-test.json");
    std::fs::write(&file_path, b"not valid json {{{{").unwrap();

    let result = SettingsStore::open(&dir, "dig3-settings-test").await;
    assert!(
        matches!(result, Err(SettingsError::Serde(_))),
        "corrupt file must return Serde error, got: {result:?}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

// ─── Test 5: persistence survives save + re-open ─────────────────────────────

#[tokio::test]
async fn settings_persist_across_reopen() {
    let dir = tmp_dir("persist");
    let _ = std::fs::remove_dir_all(&dir);

    // Write & save.
    {
        let mut s = SettingsStore::open(&dir, "dig3-settings-test")
            .await
            .unwrap();
        s.set("counter", &99u64).unwrap();
        s.save().await.unwrap();
    }

    // Drop and re-open — value must survive.
    {
        let s = SettingsStore::open(&dir, "dig3-settings-test")
            .await
            .unwrap();
        assert_eq!(s.get::<u64>("counter"), Some(99u64));
    }

    let _ = std::fs::remove_dir_all(&dir);
}
