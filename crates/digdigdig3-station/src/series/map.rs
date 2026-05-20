use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use super::{DataPoint, Series, SeriesKey};

/// `Arc<RwLock<HashMap<Key, Arc<RwLock<Series<T>>>>>>` — MLC dual-read pattern.
///
/// Outer RwLock guards the map structure (only locked briefly during
/// get_or_create). Inner RwLock per-series allows independent reads/writes
/// across keys without serializing on the map mutex.
pub struct SharedSeriesMap<T: DataPoint> {
    inner: Arc<RwLock<HashMap<SeriesKey, Arc<RwLock<Series<T>>>>>>,
    default_capacity: usize,
}

impl<T: DataPoint> Clone for SharedSeriesMap<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            default_capacity: self.default_capacity,
        }
    }
}

impl<T: DataPoint> SharedSeriesMap<T> {
    pub fn new(default_capacity: usize) -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            default_capacity: default_capacity.max(1),
        }
    }

    /// Return existing series or create + insert an empty one.
    pub async fn get_or_create(&self, key: SeriesKey) -> Arc<RwLock<Series<T>>> {
        if let Some(s) = self.get(&key).await {
            return s;
        }
        let mut guard = self.inner.write().await;
        guard
            .entry(key)
            .or_insert_with(|| Arc::new(RwLock::new(Series::new(self.default_capacity))))
            .clone()
    }

    pub async fn get(&self, key: &SeriesKey) -> Option<Arc<RwLock<Series<T>>>> {
        self.inner.read().await.get(key).cloned()
    }

    pub async fn remove(&self, key: &SeriesKey) -> Option<Arc<RwLock<Series<T>>>> {
        self.inner.write().await.remove(key)
    }

    pub async fn len(&self) -> usize {
        self.inner.read().await.len()
    }

    pub async fn keys(&self) -> Vec<SeriesKey> {
        self.inner.read().await.keys().cloned().collect()
    }
}
