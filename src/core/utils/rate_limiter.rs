//! # Rate Limiter Utilities
//!
//! Four rate limiter implementations for different exchange rate limit strategies:
//!
//! 1. **SimpleRateLimiter** - For exchanges using simple request counting
//!    - Used by: BingX (100/10s), Bitfinex (10-90/min), Bithumb (10/s)
//!
//! 2. **WeightRateLimiter** - For exchanges using weight-based systems
//!    - Used by: Binance (6000 weight/min), KuCoin (4000/30s)
//!
//! 3. **DecayingRateLimiter** - For exchanges using continuous counter decay
//!    - Used by: Kraken Spot (max=15/20, decay=0.33/1.0 per second), Deribit (credits)
//!
//! 4. **GroupRateLimiter** - For exchanges with multiple independent rate limit pools
//!    - Used by: Phemex (CONTRACT/SPOTORDER/OTHERS), Upbit, Kraken Futures
//!
//! ## Example Usage
//!
//! ### SimpleRateLimiter
//! ```
//! use std::time::Duration;
//! use connectors_v5::core::utils::{SimpleRateLimiter};
//!
//! // BingX: 100 requests per 10 seconds
//! let mut limiter = SimpleRateLimiter::new(100, Duration::from_secs(10));
//!
//! if limiter.try_acquire() {
//!     // Make API request
//!     println!("Request allowed, {} remaining", limiter.remaining());
//! } else {
//!     let wait = limiter.time_until_ready();
//!     println!("Rate limited, wait {:?}", wait);
//! }
//! ```
//!
//! ### WeightRateLimiter
//! ```
//! use std::time::Duration;
//! use connectors_v5::core::utils::{WeightRateLimiter};
//!
//! // Binance: 6000 weight per minute
//! let mut limiter = WeightRateLimiter::new(6000, Duration::from_secs(60));
//!
//! // Endpoint has weight of 5
//! if limiter.try_acquire(5) {
//!     // Make API request
//!     println!("Request allowed, {} weight remaining", limiter.remaining());
//!
//!     // Update from server header if available
//!     // limiter.update_from_server(used_weight_from_header);
//! } else {
//!     let wait = limiter.time_until_ready(5);
//!     println!("Rate limited, wait {:?}", wait);
//! }
//! ```

use std::collections::HashMap;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Simple rate limiter: X requests per time window
///
/// Tracks individual requests within a sliding time window.
/// Each request is counted equally regardless of complexity.
///
/// ## Used by exchanges:
/// - **BingX**: 100 requests / 10 seconds
/// - **Bitfinex**: 10-90 requests / minute (varies by tier)
/// - **Bithumb**: 10 requests / second
///
/// ## Behavior:
/// - Maintains a queue of request timestamps
/// - Automatically cleans up requests outside the time window
/// - Rejects requests when limit is reached
/// - Provides wait time until next request can be made
#[derive(Debug, Clone)]
pub struct SimpleRateLimiter {
    /// Maximum number of requests allowed in the time window
    max_requests: u32,
    /// Time window duration
    window: Duration,
    /// Queue of request timestamps (oldest first)
    requests: VecDeque<Instant>,
}

impl SimpleRateLimiter {
    /// Create a new simple rate limiter
    ///
    /// # Arguments
    /// * `max_requests` - Maximum number of requests allowed in the window
    /// * `window` - Time window duration
    ///
    /// # Example
    /// ```
    /// use std::time::Duration;
    /// use connectors_v5::core::utils::SimpleRateLimiter;
    ///
    /// // BingX: 100 requests per 10 seconds
    /// let limiter = SimpleRateLimiter::new(100, Duration::from_secs(10));
    /// ```
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            max_requests,
            window,
            requests: VecDeque::with_capacity(max_requests as usize),
        }
    }

    /// Try to acquire permission for a request
    ///
    /// If the request can be made (under the limit), it is recorded and returns `true`.
    /// Otherwise returns `false`.
    ///
    /// # Returns
    /// - `true` if request is allowed and has been recorded
    /// - `false` if rate limit is exceeded
    pub fn try_acquire(&mut self) -> bool {
        self.cleanup();

        if self.requests.len() < self.max_requests as usize {
            self.requests.push_back(Instant::now());
            true
        } else {
            false
        }
    }

    /// Get time to wait before next request is allowed
    ///
    /// # Returns
    /// - `Duration::ZERO` if a request can be made now
    /// - Otherwise, the time to wait until the oldest request expires
    pub fn time_until_ready(&self) -> Duration {
        if self.requests.len() < self.max_requests as usize {
            return Duration::ZERO;
        }

        // Oldest request must expire before we can make a new one
        if let Some(&oldest) = self.requests.front() {
            let elapsed = oldest.elapsed();
            if elapsed >= self.window {
                Duration::ZERO
            } else {
                self.window - elapsed
            }
        } else {
            Duration::ZERO
        }
    }

    /// Get current number of requests in the active window (evicts expired entries first)
    pub fn current_count(&mut self) -> u32 {
        self.cleanup();
        self.requests.len() as u32
    }

    /// Get the maximum requests allowed per window
    pub fn max_requests(&self) -> u32 {
        self.max_requests
    }

    /// Get remaining request capacity in the current window (evicts expired entries first)
    pub fn remaining(&mut self) -> u32 {
        self.cleanup();
        self.max_requests.saturating_sub(self.requests.len() as u32)
    }

    /// Update from server-reported remaining count.
    /// Rebuilds internal tracking to match server state.
    ///
    /// # Arguments
    /// * `remaining` - Number of requests the server reports as still available
    pub fn update_from_server(&mut self, remaining: u32) {
        let used = self.max_requests.saturating_sub(remaining);
        self.requests.clear();
        let now = Instant::now();
        for _ in 0..used {
            self.requests.push_back(now);
        }
    }

    /// Remove requests that are outside the time window
    fn cleanup(&mut self) {
        let now = Instant::now();
        while let Some(&oldest) = self.requests.front() {
            if now.duration_since(oldest) >= self.window {
                self.requests.pop_front();
            } else {
                break;
            }
        }
    }
}

/// Weight-based rate limiter: total weight per time window
///
/// Tracks cumulative weight of requests within a sliding time window.
/// Each request has an associated weight based on its computational cost.
///
/// ## Used by exchanges:
/// - **Binance**: 6000 weight / minute (varies by endpoint)
/// - **KuCoin**: 4000 weight / 30 seconds
///
/// ## Features:
/// - Tracks request weights instead of simple counts
/// - Supports server-side weight updates via response headers
/// - Corrects client-side tracking drift using server data
/// - Provides wait time based on requested weight
///
/// ## Weight Assignment:
/// Each endpoint is assigned a weight by the exchange:
/// - Simple queries (e.g., ping): weight 1
/// - Order book (depth 5): weight 1-5
/// - Order book (depth 500+): weight 10-50
/// - Account operations: weight 5-20
#[derive(Debug, Clone)]
pub struct WeightRateLimiter {
    /// Maximum total weight allowed in the time window
    max_weight: u32,
    /// Time window duration
    window: Duration,
    /// Queue of (timestamp, weight) entries (oldest first)
    entries: VecDeque<(Instant, u32)>,
    /// Last known used weight from server header
    last_server_used: Option<u32>,
    /// Timestamp of last server update
    last_server_update: Option<Instant>,
}

impl WeightRateLimiter {
    /// Create a new weight-based rate limiter
    ///
    /// # Arguments
    /// * `max_weight` - Maximum total weight allowed in the window
    /// * `window` - Time window duration
    ///
    /// # Example
    /// ```
    /// use std::time::Duration;
    /// use connectors_v5::core::utils::WeightRateLimiter;
    ///
    /// // Binance: 6000 weight per minute
    /// let limiter = WeightRateLimiter::new(6000, Duration::from_secs(60));
    /// ```
    pub fn new(max_weight: u32, window: Duration) -> Self {
        Self {
            max_weight,
            window,
            entries: VecDeque::new(),
            last_server_used: None,
            last_server_update: None,
        }
    }

    /// Try to acquire permission for a request with given weight
    ///
    /// If the request can be made (total weight under limit), it is recorded and returns `true`.
    /// Otherwise returns `false`.
    ///
    /// # Arguments
    /// * `weight` - Weight of the request (as defined by exchange API docs)
    ///
    /// # Returns
    /// - `true` if request is allowed and has been recorded
    /// - `false` if adding this weight would exceed the limit
    pub fn try_acquire(&mut self, weight: u32) -> bool {
        self.cleanup();

        let current = self.current_weight();
        if current + weight <= self.max_weight {
            self.entries.push_back((Instant::now(), weight));
            true
        } else {
            false
        }
    }

    /// Get time to wait before a request with given weight is allowed
    ///
    /// # Arguments
    /// * `weight` - Weight of the pending request
    ///
    /// # Returns
    /// - `Duration::ZERO` if the request can be made now
    /// - Otherwise, the time to wait until enough weight capacity is available
    pub fn time_until_ready(&mut self, weight: u32) -> Duration {
        let current = self.current_weight();
        if current + weight <= self.max_weight {
            return Duration::ZERO;
        }

        // Need to wait for enough old entries to expire
        let needed = current + weight - self.max_weight;
        let mut accumulated = 0;

        for &(timestamp, entry_weight) in &self.entries {
            accumulated += entry_weight;
            if accumulated >= needed {
                let elapsed = timestamp.elapsed();
                if elapsed >= self.window {
                    return Duration::ZERO;
                } else {
                    return self.window - elapsed;
                }
            }
        }

        Duration::ZERO
    }

    /// Update used weight from server response header
    ///
    /// Many exchanges return the current used weight in response headers:
    /// - Binance: `X-MBX-USED-WEIGHT-1M`
    /// - KuCoin: Similar header in responses
    ///
    /// This allows correcting client-side tracking to match server state.
    ///
    /// # Arguments
    /// * `used_weight` - Current used weight reported by server
    ///
    /// # Example
    /// ```no_run
    /// # use std::time::Duration;
    /// # use connectors_v5::core::utils::WeightRateLimiter;
    /// let mut limiter = WeightRateLimiter::new(6000, Duration::from_secs(60));
    ///
    /// // After making a request, parse the header
    /// // let used_weight = response.headers()
    /// //     .get("X-MBX-USED-WEIGHT-1M")
    /// //     .and_then(|v| v.to_str().ok())
    /// //     .and_then(|v| v.parse::<u32>().ok());
    ///
    /// // if let Some(weight) = used_weight {
    /// //     limiter.update_from_server(weight);
    /// // }
    /// ```
    pub fn update_from_server(&mut self, used_weight: u32) {
        self.last_server_used = Some(used_weight);
        self.last_server_update = Some(Instant::now());
    }

    /// Get current total weight in the active window (evicts expired entries first)
    ///
    /// If server data is available and recent (within the window), prefers server data.
    /// Otherwise uses client-side tracking.
    pub fn current_weight(&mut self) -> u32 {
        self.cleanup();
        // If we have recent server data, use it
        if let (Some(server_weight), Some(server_time)) =
            (self.last_server_used, self.last_server_update)
        {
            if server_time.elapsed() < self.window {
                return server_weight;
            }
        }

        // Otherwise calculate from our tracked entries
        self.entries.iter().map(|(_, weight)| weight).sum()
    }

    /// Get the maximum weight allowed per window
    pub fn max_weight(&self) -> u32 {
        self.max_weight
    }

    /// Get remaining weight capacity in the current window (evicts expired entries first)
    pub fn remaining(&mut self) -> u32 {
        self.max_weight.saturating_sub(self.current_weight())
    }

    /// Remove entries that are outside the time window
    fn cleanup(&mut self) {
        let now = Instant::now();
        while let Some(&(timestamp, _)) = self.entries.front() {
            if now.duration_since(timestamp) >= self.window {
                self.entries.pop_front();
            } else {
                break;
            }
        }

        // Clear server data if it's too old
        if let Some(server_time) = self.last_server_update {
            if now.duration_since(server_time) >= self.window {
                self.last_server_used = None;
                self.last_server_update = None;
            }
        }
    }
}

/// Decaying rate limiter: counter increases per request, decays continuously.
///
/// Unlike windowed limiters, this uses continuous exponential-style decay.
/// The counter increases by `cost` per request and decreases at `decay_rate` per second.
///
/// ## Used by exchanges:
/// - **Kraken Spot**: max=15 (Starter) or 20 (Pro), decay=0.33/s or 1.0/s
/// - **Deribit**: max=10000 credits, refill=10000 credits/s, cost=500/request
#[derive(Debug, Clone)]
pub struct DecayingRateLimiter {
    max_counter: f64,
    /// Units per second that decay from the counter
    decay_rate: f64,
    counter: f64,
    last_update: Instant,
}

impl DecayingRateLimiter {
    /// Create a new decaying rate limiter
    ///
    /// # Arguments
    /// * `max_counter` - Maximum counter value before requests are blocked
    /// * `decay_rate` - Units per second that decay from the counter
    ///
    /// # Example
    /// ```
    /// // Kraken Spot Starter tier: max=15, decay=0.33/s
    /// use connectors_v5::core::utils::DecayingRateLimiter;
    /// let limiter = DecayingRateLimiter::new(15.0, 0.33);
    /// ```
    pub fn new(max_counter: f64, decay_rate: f64) -> Self {
        Self {
            max_counter,
            decay_rate,
            counter: 0.0,
            last_update: Instant::now(),
        }
    }

    /// Apply decay based on elapsed time since last update
    fn apply_decay(&mut self) {
        let elapsed = self.last_update.elapsed().as_secs_f64();
        self.counter = (self.counter - self.decay_rate * elapsed).max(0.0);
        self.last_update = Instant::now();
    }

    /// Try to acquire permission for a request with the given cost
    ///
    /// # Arguments
    /// * `cost` - Cost of the request (increases the counter)
    ///
    /// # Returns
    /// - `true` if request is allowed and cost has been added to the counter
    /// - `false` if adding this cost would exceed `max_counter`
    pub fn try_acquire(&mut self, cost: f64) -> bool {
        self.apply_decay();
        if self.counter + cost <= self.max_counter {
            self.counter += cost;
            true
        } else {
            false
        }
    }

    /// Get time to wait before a request with the given cost is allowed
    ///
    /// # Arguments
    /// * `cost` - Cost of the pending request
    ///
    /// # Returns
    /// - `Duration::ZERO` if the request can be made now
    /// - Otherwise, the time to wait until enough counter capacity decays
    pub fn time_until_ready(&mut self, cost: f64) -> Duration {
        self.apply_decay();
        if self.counter + cost <= self.max_counter {
            return Duration::ZERO;
        }
        let excess = self.counter + cost - self.max_counter;
        let wait_secs = excess / self.decay_rate;
        Duration::from_secs_f64(wait_secs)
    }

    /// Get current counter level after applying decay
    pub fn current_level(&mut self) -> f64 {
        self.apply_decay();
        self.counter
    }

    /// Get the maximum counter level
    pub fn max_level(&self) -> f64 {
        self.max_counter
    }

    /// Get remaining capacity after applying decay
    pub fn remaining(&mut self) -> f64 {
        self.apply_decay();
        (self.max_counter - self.counter).max(0.0)
    }
}

/// Group-based rate limiter: multiple independent pools keyed by name.
///
/// Each group is an independent `WeightRateLimiter` with its own max/window.
/// Requests are routed to the appropriate group by the connector.
///
/// ## Used by exchanges:
/// - **Phemex**: CONTRACT, SPOTORDER, OTHERS groups
/// - **Upbit**: market, account, order groups
/// - **Kraken Futures**: derivatives (500/10s), history (100/600s)
#[derive(Debug, Clone)]
pub struct GroupRateLimiter {
    groups: HashMap<&'static str, WeightRateLimiter>,
}

impl GroupRateLimiter {
    /// Create a new empty group rate limiter
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
        }
    }

    /// Add a named rate limit group
    ///
    /// # Arguments
    /// * `name` - Group name (must be `'static` str, e.g. `"public"`)
    /// * `max_weight` - Maximum total weight for this group's window
    /// * `window` - Time window duration for this group
    pub fn add_group(&mut self, name: &'static str, max_weight: u32, window: Duration) {
        self.groups
            .insert(name, WeightRateLimiter::new(max_weight, window));
    }

    /// Try to acquire weight from a specific group
    ///
    /// # Returns
    /// - `true` if weight is within the group's limit (or group is unknown)
    /// - `false` if adding this weight would exceed the group's limit
    pub fn try_acquire(&mut self, group: &str, weight: u32) -> bool {
        if let Some(limiter) = self.groups.get_mut(group) {
            limiter.try_acquire(weight)
        } else {
            true // unknown group = no limit applied
        }
    }

    /// Get wait time for a specific group
    ///
    /// # Returns
    /// - `Duration::ZERO` if the group is unknown or request fits within limits
    /// - Otherwise, the time to wait until enough weight capacity is available
    pub fn time_until_ready(&mut self, group: &str, weight: u32) -> Duration {
        if let Some(limiter) = self.groups.get_mut(group) {
            limiter.time_until_ready(weight)
        } else {
            Duration::ZERO
        }
    }

    /// Update server-reported used weight for a specific group
    ///
    /// # Arguments
    /// * `group` - Group name
    /// * `used_weight` - Current used weight reported by server for this group
    pub fn update_from_server(&mut self, group: &str, used_weight: u32) {
        if let Some(limiter) = self.groups.get_mut(group) {
            limiter.update_from_server(used_weight);
        }
    }

    /// Get `(current_weight, max_weight)` for a specific group
    ///
    /// Returns `None` if the group does not exist.
    pub fn group_stats(&mut self, group: &str) -> Option<(u32, u32)> {
        self.groups
            .get_mut(group)
            .map(|l| (l.current_weight(), l.max_weight()))
    }

    /// Get stats for all groups as `Vec` of `(name, current_weight, max_weight)`
    pub fn all_stats(&mut self) -> Vec<(&str, u32, u32)> {
        self.groups
            .iter_mut()
            .map(|(name, l)| (*name, l.current_weight(), l.max_weight()))
            .collect()
    }

    /// Get primary group stats for backwards-compatible display
    ///
    /// Returns `(current_weight, max_weight)` of an arbitrary group,
    /// or `(0, 0)` when no groups have been added.
    pub fn primary_stats(&mut self) -> (u32, u32) {
        self.groups
            .values_mut()
            .next()
            .map(|l| (l.current_weight(), l.max_weight()))
            .unwrap_or((0, 0))
    }
}

impl Default for GroupRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// RUNTIME LIMITER — unified enum wrapping the four implementations
// ═══════════════════════════════════════════════════════════════════════════════

use crate::core::types::{RateLimitCapabilities, LimitModel};

/// Unified runtime limiter — wraps one of the four implementations.
///
/// Built once per connector via `RuntimeLimiter::from_caps(&RateLimitCapabilities)`.
/// Held behind `Arc<Mutex<RuntimeLimiter>>` in the connector struct.
pub enum RuntimeLimiter {
    Simple(SimpleRateLimiter),
    Weight(WeightRateLimiter),
    Decaying(DecayingRateLimiter),
    Group(GroupRateLimiter),
    Unlimited,
}

impl RuntimeLimiter {
    /// Build from capabilities. Unknown/Unlimited models return `Unlimited`.
    pub fn from_caps(caps: &RateLimitCapabilities) -> Self {
        match caps.model {
            LimitModel::Unlimited => Self::Unlimited,

            LimitModel::Simple => {
                if caps.rest_pools.is_empty() {
                    return Self::Unlimited;
                }
                let pool = &caps.rest_pools[0];
                Self::Simple(SimpleRateLimiter::new(
                    pool.max_budget,
                    Duration::from_secs(pool.window_seconds as u64),
                ))
            }

            LimitModel::Weight => {
                if caps.rest_pools.is_empty() {
                    return Self::Unlimited;
                }
                let pool = &caps.rest_pools[0];
                Self::Weight(WeightRateLimiter::new(
                    pool.max_budget,
                    Duration::from_secs(pool.window_seconds as u64),
                ))
            }

            LimitModel::Decaying => {
                if let Some(cfg) = caps.decaying {
                    Self::Decaying(DecayingRateLimiter::new(cfg.max_counter, cfg.decay_rate_per_sec))
                } else {
                    Self::Unlimited
                }
            }

            LimitModel::Group => {
                let mut g = GroupRateLimiter::new();
                for pool in caps.rest_pools {
                    g.add_group(
                        pool.name,
                        pool.max_budget,
                        Duration::from_secs(pool.window_seconds as u64),
                    );
                }
                Self::Group(g)
            }
        }
    }

    /// Try to acquire budget. Returns `true` if allowed.
    pub fn try_acquire(&mut self, group: &str, weight: u32) -> bool {
        match self {
            Self::Unlimited => true,
            Self::Simple(l) => l.try_acquire(),
            Self::Weight(l) => l.try_acquire(weight),
            Self::Decaying(l) => l.try_acquire(weight as f64),
            Self::Group(l) => l.try_acquire(group, weight),
        }
    }

    /// Time to wait before `weight` can be acquired.
    pub fn time_until_ready(&mut self, group: &str, weight: u32) -> Duration {
        match self {
            Self::Unlimited => Duration::ZERO,
            Self::Simple(l) => l.time_until_ready(),
            Self::Weight(l) => l.time_until_ready(weight),
            Self::Decaying(l) => l.time_until_ready(weight as f64),
            Self::Group(l) => l.time_until_ready(group, weight),
        }
    }

    /// Sync from server-reported used/remaining value.
    pub fn update_from_server(&mut self, group: &str, value: u32) {
        match self {
            Self::Weight(l) => l.update_from_server(value),
            Self::Group(l) => l.update_from_server(group, value),
            Self::Simple(l) => l.update_from_server(value),
            _ => {}
        }
    }

    /// Snapshot: `(used, max)` for the primary pool. For `ConnectorStats`.
    pub fn primary_stats(&mut self) -> (u32, u32) {
        match self {
            Self::Unlimited => (0, 0),
            Self::Simple(l) => (l.current_count(), l.max_requests()),
            Self::Weight(l) => (l.current_weight(), l.max_weight()),
            Self::Decaying(l) => (l.current_level() as u32, l.max_level() as u32),
            Self::Group(l) => l.primary_stats(),
        }
    }

    /// All group stats for `ConnectorStats::rate_groups`.
    pub fn group_stats(&mut self) -> Vec<(String, u32, u32)> {
        match self {
            Self::Group(l) => l
                .all_stats()
                .into_iter()
                .map(|(name, used, max)| (name.to_string(), used, max))
                .collect(),
            _ => vec![],
        }
    }

    /// Utilization ratio in `[0.0, 1.0]`. Used by threshold monitor.
    pub fn utilization(&mut self) -> f32 {
        let (used, max) = self.primary_stats();
        if max == 0 {
            return 0.0;
        }
        (used as f32 / max as f32).min(1.0)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// RATE LIMIT PRESSURE MONITORING
// ═══════════════════════════════════════════════════════════════════════════════

/// Threshold levels for rate limit pressure.
///
/// Two thresholds:
/// - **75%** — Warning: notify user, everything still passes.
/// - **90%** — Cutoff: non-essential requests (market data) are dropped.
///   Last 10% of budget is reserved exclusively for trading operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RateLimitPressure {
    /// < 75% utilization — normal operation.
    Normal,
    /// >= 75% — warning notification; all requests still pass.
    Warning,
    /// >= 90% — non-essential requests dropped; last 10% reserved for trading.
    Cutoff,
}

impl RateLimitPressure {
    /// Determine pressure level from utilization ratio.
    pub fn from_utilization(ratio: f32) -> Self {
        if ratio >= 0.90 {
            Self::Cutoff
        } else if ratio >= 0.75 {
            Self::Warning
        } else {
            Self::Normal
        }
    }
}

/// Lightweight monitor — called inline before every request.
///
/// No channels, no background task. Logs once per pressure transition.
pub struct RateLimitMonitor {
    last_pressure: RateLimitPressure,
    exchange_name: &'static str,
}

impl RateLimitMonitor {
    pub fn new(exchange_name: &'static str) -> Self {
        Self {
            last_pressure: RateLimitPressure::Normal,
            exchange_name,
        }
    }

    /// Evaluate current pressure and return level.
    /// Logs once when pressure level changes.
    pub fn check(&mut self, limiter: &mut RuntimeLimiter) -> RateLimitPressure {
        let ratio = limiter.utilization();
        let pressure = RateLimitPressure::from_utilization(ratio);
        if pressure != self.last_pressure {
            match pressure {
                RateLimitPressure::Warning => {
                    tracing::warn!(
                        exchange = self.exchange_name,
                        utilization = format!("{:.0}%", ratio * 100.0),
                        "Rate limit warning: 75%+ budget used"
                    );
                }
                RateLimitPressure::Cutoff => {
                    tracing::error!(
                        exchange = self.exchange_name,
                        utilization = format!("{:.0}%", ratio * 100.0),
                        "Rate limit cutoff: 90%+ used — dropping non-essential, last 10% reserved for trading"
                    );
                }
                RateLimitPressure::Normal => {
                    tracing::info!(
                        exchange = self.exchange_name,
                        utilization = format!("{:.0}%", ratio * 100.0),
                        "Rate limit pressure eased — back to normal"
                    );
                }
            }
            self.last_pressure = pressure;
        }
        pressure
    }

    /// Current pressure without re-checking.
    pub fn current_pressure(&self) -> RateLimitPressure {
        self.last_pressure
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_simple_rate_limiter_allows_under_limit() {
        let mut limiter = SimpleRateLimiter::new(5, Duration::from_secs(1));

        // Should allow first 5 requests
        for i in 0..5 {
            assert!(limiter.try_acquire(), "Request {} should be allowed", i + 1);
        }

        assert_eq!(limiter.current_count(), 5);
        assert_eq!(limiter.remaining(), 0);
    }

    #[test]
    fn test_simple_rate_limiter_blocks_over_limit() {
        let mut limiter = SimpleRateLimiter::new(3, Duration::from_secs(1));

        // Allow first 3
        assert!(limiter.try_acquire());
        assert!(limiter.try_acquire());
        assert!(limiter.try_acquire());

        // Block 4th
        assert!(!limiter.try_acquire(), "4th request should be blocked");
        assert_eq!(limiter.current_count(), 3);
    }

    #[test]
    fn test_simple_rate_limiter_allows_after_window() {
        let mut limiter = SimpleRateLimiter::new(2, Duration::from_millis(100));

        // Fill the limit
        assert!(limiter.try_acquire());
        assert!(limiter.try_acquire());
        assert!(!limiter.try_acquire());

        // Wait for window to pass
        thread::sleep(Duration::from_millis(110));

        // Should allow new requests
        assert!(
            limiter.try_acquire(),
            "Request should be allowed after window expires"
        );
    }

    #[test]
    fn test_simple_rate_limiter_time_until_ready() {
        let mut limiter = SimpleRateLimiter::new(1, Duration::from_secs(1));

        // First request allowed
        assert!(limiter.try_acquire());

        // Check wait time (should be close to 1 second, but less due to elapsed time)
        let wait = limiter.time_until_ready();
        assert!(
            wait > Duration::from_millis(900),
            "Wait time should be close to 1 second"
        );
        assert!(
            wait <= Duration::from_secs(1),
            "Wait time should not exceed window"
        );
    }

    #[test]
    fn test_simple_rate_limiter_time_until_ready_when_available() {
        let mut limiter = SimpleRateLimiter::new(5, Duration::from_secs(1));

        assert!(limiter.try_acquire());

        // Still have capacity, should return zero
        let wait = limiter.time_until_ready();
        assert_eq!(
            wait,
            Duration::ZERO,
            "Should return zero wait when capacity available"
        );
    }

    #[test]
    fn test_weight_rate_limiter_allows_under_limit() {
        let mut limiter = WeightRateLimiter::new(100, Duration::from_secs(1));

        // Various weights under limit
        assert!(limiter.try_acquire(10));
        assert!(limiter.try_acquire(20));
        assert!(limiter.try_acquire(30));
        assert_eq!(limiter.current_weight(), 60);
        assert_eq!(limiter.remaining(), 40);
    }

    #[test]
    fn test_weight_rate_limiter_blocks_over_limit() {
        let mut limiter = WeightRateLimiter::new(50, Duration::from_secs(1));

        assert!(limiter.try_acquire(30));
        assert!(limiter.try_acquire(15));
        assert_eq!(limiter.current_weight(), 45);

        // This would exceed limit (45 + 10 = 55 > 50)
        assert!(
            !limiter.try_acquire(10),
            "Request should be blocked when it would exceed limit"
        );
        assert_eq!(
            limiter.current_weight(),
            45,
            "Weight should not increase after blocked request"
        );
    }

    #[test]
    fn test_weight_rate_limiter_allows_after_window() {
        let mut limiter = WeightRateLimiter::new(50, Duration::from_millis(100));

        // Fill to limit
        assert!(limiter.try_acquire(50));
        assert!(!limiter.try_acquire(1));

        // Wait for window
        thread::sleep(Duration::from_millis(110));

        // Should allow new requests
        assert!(
            limiter.try_acquire(50),
            "Request should be allowed after window expires"
        );
    }

    #[test]
    fn test_weight_rate_limiter_time_until_ready() {
        let mut limiter = WeightRateLimiter::new(100, Duration::from_secs(1));

        assert!(limiter.try_acquire(100));

        // Check wait time for a 1-weight request
        let wait = limiter.time_until_ready(1);
        assert!(
            wait > Duration::from_millis(900),
            "Wait time should be close to 1 second"
        );
        assert!(
            wait <= Duration::from_secs(1),
            "Wait time should not exceed window"
        );
    }

    #[test]
    fn test_weight_rate_limiter_partial_expiry() {
        let mut limiter = WeightRateLimiter::new(100, Duration::from_millis(100));

        // Add weights at different times
        assert!(limiter.try_acquire(50));
        thread::sleep(Duration::from_millis(60));
        assert!(limiter.try_acquire(40));

        // First entry should be close to expiring
        thread::sleep(Duration::from_millis(50));

        // First 50 should have expired, so 50 + new request should work
        assert!(
            limiter.try_acquire(50),
            "Should allow request after partial expiry"
        );
    }

    #[test]
    fn test_weight_rate_limiter_server_update() {
        let mut limiter = WeightRateLimiter::new(1000, Duration::from_secs(60));

        // Make some client-side tracked requests
        assert!(limiter.try_acquire(100));
        assert!(limiter.try_acquire(50));
        assert_eq!(limiter.current_weight(), 150);

        // Server reports different weight (could be from other clients/instances)
        limiter.update_from_server(500);
        assert_eq!(
            limiter.current_weight(),
            500,
            "Should use server-reported weight"
        );
        assert_eq!(limiter.remaining(), 500);
    }

    #[test]
    fn test_weight_rate_limiter_server_update_expires() {
        let mut limiter = WeightRateLimiter::new(1000, Duration::from_millis(100));

        limiter.update_from_server(500);
        assert_eq!(limiter.current_weight(), 500);

        // Wait for server data to expire
        thread::sleep(Duration::from_millis(110));

        // Should fall back to client tracking (0 since we haven't made tracked requests)
        limiter.cleanup();
        assert_eq!(
            limiter.current_weight(),
            0,
            "Should revert to client tracking after server data expires"
        );
    }

    #[test]
    fn test_weight_rate_limiter_different_weights() {
        let mut limiter = WeightRateLimiter::new(100, Duration::from_secs(1));

        // Simulate different endpoint weights (like Binance)
        assert!(limiter.try_acquire(1)); // ping
        assert!(limiter.try_acquire(1)); // simple query
        assert!(limiter.try_acquire(5)); // order book depth 100
        assert!(limiter.try_acquire(10)); // order book depth 500
        assert!(limiter.try_acquire(50)); // order book depth 5000

        assert_eq!(limiter.current_weight(), 67);
        assert_eq!(limiter.remaining(), 33);

        // Can still fit a 33-weight request
        assert!(limiter.try_acquire(33));
        assert!(!limiter.try_acquire(1), "Should be at capacity");
    }

    // --- SimpleRateLimiter::update_from_server ---

    #[test]
    fn test_simple_rate_limiter_update_from_server() {
        let mut limiter = SimpleRateLimiter::new(10, Duration::from_secs(60));

        // Server reports only 3 remaining out of 10 → 7 are considered used
        limiter.update_from_server(3);
        assert_eq!(limiter.remaining(), 3);
        assert_eq!(limiter.current_count(), 7);

        // Can still acquire up to 3 more
        for _ in 0..3 {
            assert!(
                limiter.try_acquire(),
                "Should allow request within remaining capacity"
            );
        }
        // 11th would exceed limit (10 used + 1 new > 10)
        assert!(
            !limiter.try_acquire(),
            "Should block when remaining exhausted"
        );
    }

    // --- DecayingRateLimiter ---

    #[test]
    fn test_decaying_rate_limiter_allows_under_limit() {
        // Kraken Starter: max=15, decay=0.33/s, each request costs 1
        let mut limiter = DecayingRateLimiter::new(15.0, 0.33);

        for i in 0..15 {
            assert!(
                limiter.try_acquire(1.0),
                "Request {} should be allowed",
                i + 1
            );
        }
        assert!(limiter.current_level() <= 15.0);
    }

    #[test]
    fn test_decaying_rate_limiter_blocks_over_limit() {
        let mut limiter = DecayingRateLimiter::new(10.0, 1.0);

        // Fill to max with a single large cost
        assert!(
            limiter.try_acquire(10.0),
            "Should allow request at exactly max"
        );

        // Next request should be blocked
        assert!(
            !limiter.try_acquire(1.0),
            "Should block when counter is at max"
        );
    }

    #[test]
    fn test_decaying_rate_limiter_decays_over_time() {
        let mut limiter = DecayingRateLimiter::new(10.0, 10.0); // decays 10 units/sec

        // Fill to max
        assert!(limiter.try_acquire(10.0));
        let level_before = limiter.current_level();
        assert!(
            level_before > 9.0,
            "Counter should be near 10 right after request"
        );

        // Wait 200ms → decay of ~2.0 units expected
        thread::sleep(Duration::from_millis(200));
        let level_after = limiter.current_level();

        assert!(
            level_after < level_before,
            "Counter should decay over time: before={}, after={}",
            level_before,
            level_after
        );
    }

    #[test]
    fn test_decaying_rate_limiter_time_until_ready() {
        // decay_rate = 10/s, cost = 5, max = 10
        let mut limiter = DecayingRateLimiter::new(10.0, 10.0);

        // Fill to max
        assert!(limiter.try_acquire(10.0));

        // Need to wait for 5 units to decay at 10/s → ~0.5 seconds
        let wait = limiter.time_until_ready(5.0);
        assert!(
            wait > Duration::from_millis(400),
            "Wait should be roughly 0.5s, got {:?}",
            wait
        );
        assert!(
            wait <= Duration::from_secs(1),
            "Wait should not exceed 1s, got {:?}",
            wait
        );
    }

    // --- GroupRateLimiter ---

    #[test]
    fn test_group_rate_limiter_independent_groups() {
        let mut limiter = GroupRateLimiter::new();
        limiter.add_group("public", 100, Duration::from_secs(10));
        limiter.add_group("private", 20, Duration::from_secs(10));

        // Exhaust the private group
        assert!(limiter.try_acquire("private", 20));
        assert!(
            !limiter.try_acquire("private", 1),
            "private group should be at capacity"
        );

        // Public group must still be independent and available
        assert!(
            limiter.try_acquire("public", 50),
            "public group should be unaffected"
        );
    }

    #[test]
    fn test_group_rate_limiter_unknown_group_allows() {
        let mut limiter = GroupRateLimiter::new();
        limiter.add_group("public", 100, Duration::from_secs(10));

        // Requests to an unknown group should always be allowed (no limit configured)
        assert!(
            limiter.try_acquire("nonexistent", 9999),
            "Unknown group should return true"
        );
        assert_eq!(
            limiter.time_until_ready("nonexistent", 9999),
            Duration::ZERO,
            "Unknown group wait time should be zero"
        );
    }

    #[test]
    fn test_group_rate_limiter_all_stats() {
        let mut limiter = GroupRateLimiter::new();
        limiter.add_group("spot", 50, Duration::from_secs(10));
        limiter.add_group("futures", 200, Duration::from_secs(10));

        limiter.try_acquire("spot", 10);
        limiter.try_acquire("futures", 40);

        let stats = limiter.all_stats();
        assert_eq!(
            stats.len(),
            2,
            "all_stats should return one entry per group"
        );

        for (name, current, max) in &stats {
            match *name {
                "spot" => {
                    assert_eq!(*max, 50);
                    assert_eq!(*current, 10);
                }
                "futures" => {
                    assert_eq!(*max, 200);
                    assert_eq!(*current, 40);
                }
                other => panic!("Unexpected group name: {}", other),
            }
        }
    }
}
