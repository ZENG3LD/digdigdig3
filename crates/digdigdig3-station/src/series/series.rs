use std::collections::VecDeque;
use std::sync::Arc;

use tokio::sync::RwLock;

use super::DataPoint;

/// Bounded in-memory ring of `T`. MLC-style: SharedSeriesMap maps each Key to
/// `Arc<RwLock<Series<T>>>` so multiple readers + one writer don't serialize
/// against each other across keys.
pub struct Series<T: DataPoint> {
    capacity: usize,
    ring: VecDeque<T>,
    /// Number of records since the last `flush()`. The DiskStore consumes this.
    dirty: usize,
}

impl<T: DataPoint> Series<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            ring: VecDeque::with_capacity(capacity),
            dirty: 0,
        }
    }

    /// Append one record. Evicts oldest if at capacity.
    pub fn push(&mut self, point: T) {
        if self.ring.len() == self.capacity {
            self.ring.pop_front();
        }
        self.ring.push_back(point);
        self.dirty = self.dirty.saturating_add(1);
    }

    /// Append a batch (e.g. REST backfill). Maintains capacity bound.
    pub fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for p in iter {
            self.push(p);
        }
    }

    /// Insert-or-replace by timestamp. If a record with the same
    /// `timestamp_ms()` already exists in the ring, it is replaced in place
    /// (and `dirty` ticks). Otherwise behaves like [`push`].
    ///
    /// Used for kline gap-heal: REST may return a bar with the same
    /// `open_time` as one we already have but with the canonical OHLC values;
    /// the live half-formed copy must be overwritten.
    pub fn upsert_by_ts(&mut self, point: T) {
        let ts = point.timestamp_ms();
        if let Some(pos) = self.ring.iter().rposition(|p| p.timestamp_ms() == ts) {
            self.ring[pos] = point;
            self.dirty = self.dirty.saturating_add(1);
            return;
        }
        self.push(point);
    }

    /// Last N points, oldest → newest (clone). For `Series::take_tail`.
    pub fn tail(&self, n: usize) -> Vec<T> {
        let take = n.min(self.ring.len());
        let start = self.ring.len() - take;
        self.ring.iter().skip(start).cloned().collect()
    }

    /// All points oldest → newest.
    pub fn snapshot(&self) -> Vec<T> {
        self.ring.iter().cloned().collect()
    }

    /// Most recent record.
    pub fn last(&self) -> Option<&T> {
        self.ring.back()
    }

    pub fn len(&self) -> usize {
        self.ring.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ring.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn dirty_count(&self) -> usize {
        self.dirty
    }

    pub fn mark_clean(&mut self) {
        self.dirty = 0;
    }

    /// Replace the ring with `points` oldest→newest. Used by `seed_from_disk`.
    /// Resets `dirty` to 0 (seeded data isn't dirty — it already lives on disk).
    pub fn seed(&mut self, points: Vec<T>) {
        self.ring.clear();
        let take = points.len().min(self.capacity);
        let start = points.len() - take;
        for p in points.into_iter().skip(start) {
            self.ring.push_back(p);
        }
        self.dirty = 0;
    }
}

pub type SharedSeries<T> = Arc<RwLock<Series<T>>>;
