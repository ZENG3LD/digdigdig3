//! ReconnectConfig + BackoffState — reconnect backoff logic.

use std::time::Duration;

/// Reconnect backoff configuration.
#[derive(Debug, Clone)]
pub struct ReconnectConfig {
    /// 0 = infinite
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
    /// 0.2 = ±20% randomization
    pub jitter_factor: f64,
    pub connection_timeout_ms: u64,
    /// Delay after auth failure before retry (longer than normal backoff).
    pub auth_failure_delay_ms: u64,
    /// Silent-stream threshold = ping_interval × silent_multiplier.
    /// Watchdog fires a forced reconnect when no frames arrive for this duration.
    /// Default 2. Lower for chatty streams (1), raise for deliberately quiet feeds (5).
    pub silent_multiplier: u32,
    /// Broadcast queue depth that triggers a lag warning.
    /// If `event_tx.len() > lag_threshold`, `tracing::warn!` fires on target `dig3::ws::lag`.
    /// Default 512 (half of the 4096 broadcast capacity).
    pub lag_threshold: usize,
    /// How often (ms) the lag-check task polls `event_tx.len()`.
    /// Default 5000 (5 s).
    pub lag_check_interval_ms: u64,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            max_attempts: 0,
            initial_delay_ms: 1_000,
            max_delay_ms: 30_000,
            backoff_multiplier: 2.0,
            jitter_factor: 0.2,
            connection_timeout_ms: 10_000,
            auth_failure_delay_ms: 5_000,
            silent_multiplier: 2,
            lag_threshold: 512,
            lag_check_interval_ms: 5_000,
        }
    }
}

/// Mutable backoff state — held inside the driver task, never shared.
pub(super) struct BackoffState {
    cfg: ReconnectConfig,
    pub attempt: u32,
    current_delay_ms: u64,
}

impl BackoffState {
    pub fn new(cfg: ReconnectConfig) -> Self {
        let initial = cfg.initial_delay_ms;
        Self {
            cfg,
            attempt: 0,
            current_delay_ms: initial,
        }
    }

    /// Returns the next sleep duration, then advances the state.
    pub fn next_delay(&mut self) -> Duration {
        let base = self.current_delay_ms as f64;
        let jitter_range = base * self.cfg.jitter_factor;
        // Deterministic jitter using attempt count (avoids rand dependency in this module).
        // Simple alternating +/- based on attempt parity.
        let jitter = if self.attempt % 2 == 0 {
            jitter_range * 0.5
        } else {
            -jitter_range * 0.5
        };
        let delayed = (base + jitter).max(0.0) as u64;
        let delay_ms = delayed.min(self.cfg.max_delay_ms);

        // Advance
        self.attempt += 1;
        let next = (self.current_delay_ms as f64 * self.cfg.backoff_multiplier) as u64;
        self.current_delay_ms = next.min(self.cfg.max_delay_ms);

        Duration::from_millis(delay_ms)
    }

    pub fn reset(&mut self) {
        self.attempt = 0;
        self.current_delay_ms = self.cfg.initial_delay_ms;
    }

    pub fn max_attempts(&self) -> u32 {
        self.cfg.max_attempts
    }

    pub fn auth_failure_delay(&self) -> Duration {
        Duration::from_millis(self.cfg.auth_failure_delay_ms)
    }

    pub fn connection_timeout(&self) -> Duration {
        Duration::from_millis(self.cfg.connection_timeout_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_increases() {
        let cfg = ReconnectConfig {
            initial_delay_ms: 1_000,
            max_delay_ms: 30_000,
            backoff_multiplier: 2.0,
            jitter_factor: 0.0,
            ..Default::default()
        };
        let mut state = BackoffState::new(cfg);
        let d0 = state.next_delay().as_millis();
        let d1 = state.next_delay().as_millis();
        let d2 = state.next_delay().as_millis();
        assert!(d1 > d0, "delay should increase: d1={d1} d0={d0}");
        assert!(d2 > d1, "delay should increase: d2={d2} d1={d1}");
    }

    #[test]
    fn backoff_caps_at_max() {
        let cfg = ReconnectConfig {
            initial_delay_ms: 1_000,
            max_delay_ms: 5_000,
            backoff_multiplier: 100.0,
            jitter_factor: 0.0,
            ..Default::default()
        };
        let mut state = BackoffState::new(cfg);
        for _ in 0..10 {
            let d = state.next_delay().as_millis();
            assert!(d <= 5_000, "delay should not exceed max: {d}");
        }
    }

    #[test]
    fn backoff_reset() {
        let cfg = ReconnectConfig::default();
        let mut state = BackoffState::new(cfg);
        state.next_delay();
        state.next_delay();
        state.reset();
        assert_eq!(state.attempt, 0);
    }
}
