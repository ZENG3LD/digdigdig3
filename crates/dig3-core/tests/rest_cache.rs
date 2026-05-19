//! Phase λ.C — TTL cache unit tests.
//!
//! Pure logic tests — no network. TTL expiry tested via tokio::time::sleep.
//!
//! Run with: cargo test --test rest_cache -- --nocapture

use digdigdig3_core::RestCache;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn get_returns_none_on_miss() {
    let cache: RestCache<String, u32> = RestCache::new(Duration::from_secs(60));
    assert!(cache.get(&"missing".to_string()).is_none());
}

#[tokio::test]
async fn insert_then_get_returns_value() {
    let cache: RestCache<String, u32> = RestCache::new(Duration::from_secs(60));
    cache.insert("key".to_string(), 42);
    assert_eq!(cache.get(&"key".to_string()), Some(42));
}

#[tokio::test]
async fn expired_returns_none() {
    let cache: RestCache<String, u32> = RestCache::new(Duration::from_millis(50));
    cache.insert("key".to_string(), 99);
    assert_eq!(cache.get(&"key".to_string()), Some(99));
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(cache.get(&"key".to_string()).is_none());
}

#[tokio::test]
async fn get_or_fetch_calls_loader_on_miss() {
    let cache: RestCache<String, u32> = RestCache::new(Duration::from_secs(60));
    let call_count = Arc::new(AtomicUsize::new(0));
    let cc = call_count.clone();

    let result: Result<u32, String> = cache
        .get_or_fetch("key".to_string(), || async move {
            cc.fetch_add(1, Ordering::SeqCst);
            Ok(7)
        })
        .await;

    assert_eq!(result.unwrap(), 7);
    assert_eq!(call_count.load(Ordering::SeqCst), 1);
    // value now cached
    assert_eq!(cache.get(&"key".to_string()), Some(7));
}

#[tokio::test]
async fn get_or_fetch_skips_loader_on_hit() {
    let cache: RestCache<String, u32> = RestCache::new(Duration::from_secs(60));
    cache.insert("key".to_string(), 5);

    let call_count = Arc::new(AtomicUsize::new(0));
    let cc = call_count.clone();

    let result: Result<u32, String> = cache
        .get_or_fetch("key".to_string(), || async move {
            cc.fetch_add(1, Ordering::SeqCst);
            Ok(99)
        })
        .await;

    assert_eq!(result.unwrap(), 5);
    assert_eq!(call_count.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn invalidate_removes_key() {
    let cache: RestCache<String, u32> = RestCache::new(Duration::from_secs(60));
    cache.insert("key".to_string(), 1);
    assert!(cache.get(&"key".to_string()).is_some());
    cache.invalidate(&"key".to_string());
    assert!(cache.get(&"key".to_string()).is_none());
}

#[tokio::test]
async fn sweep_expired_removes_old_entries() {
    let cache: RestCache<String, u32> = RestCache::new(Duration::from_millis(50));
    cache.insert("a".to_string(), 1);
    cache.insert("b".to_string(), 2);
    cache.insert_with_ttl("c".to_string(), 3, Duration::from_secs(60));

    assert_eq!(cache.len(), 3);
    tokio::time::sleep(Duration::from_millis(100)).await;

    let removed = cache.sweep_expired();
    assert_eq!(removed, 2); // "a" and "b" expired; "c" still fresh
    assert_eq!(cache.len(), 1);
    assert_eq!(cache.get(&"c".to_string()), Some(3));
}
