//! Replay rate control — governs how fast stored events are emitted.

/// Controls emission cadence during replay.
#[derive(Debug, Clone, Copy)]
pub enum ReplayRate {
    /// Emit events at original inter-event timing (wall-clock sleeps between events).
    Realtime,
    /// Emit at `x` times real speed. `1.0` equals `Realtime`.
    Accelerated(f64),
    /// Emit as fast as possible — no sleeps.
    Instant,
}

impl ReplayRate {
    /// Compute how long to sleep before emitting the next event.
    ///
    /// `sim_elapsed_ms` — milliseconds elapsed in simulated time since the first event.
    /// `real_elapsed_ms` — milliseconds elapsed on the wall clock since replay started.
    ///
    /// Returns `None` when no sleep is needed (event is already "late").
    pub fn delay_for(
        &self,
        sim_elapsed_ms: i64,
        real_elapsed_ms: i64,
    ) -> Option<std::time::Duration> {
        match self {
            ReplayRate::Instant => None,
            ReplayRate::Realtime => {
                let remaining = sim_elapsed_ms - real_elapsed_ms;
                if remaining > 0 {
                    Some(std::time::Duration::from_millis(remaining as u64))
                } else {
                    None
                }
            }
            ReplayRate::Accelerated(x) => {
                if *x <= 0.0 {
                    return None;
                }
                let target_real_ms = (sim_elapsed_ms as f64 / x) as i64;
                let remaining = target_real_ms - real_elapsed_ms;
                if remaining > 0 {
                    Some(std::time::Duration::from_millis(remaining as u64))
                } else {
                    None
                }
            }
        }
    }
}
