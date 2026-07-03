//! Throughput measurement with EMA smoothing.

use std::time::Instant;

use super::Ema;

/// A point-in-time throughput reading, in bits per second.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThroughputSnapshot {
    /// EMA-smoothed instantaneous speed.
    pub current_bps: f64,
    /// Average speed since measurement began.
    pub average_bps: f64,
    /// Highest smoothed speed observed.
    pub peak_bps: f64,
}

/// Converts a monotonically-growing byte counter into smoothed speed
/// readings.
///
/// Feed it the cumulative byte total at regular intervals via
/// [`ThroughputMeter::update`]; it derives instantaneous, average and peak
/// speeds. Time is passed explicitly so the meter is deterministic in tests.
#[derive(Debug, Clone)]
pub struct ThroughputMeter {
    started: Instant,
    last_time: Instant,
    last_bytes: u64,
    ema: Ema,
    peak_bps: f64,
}

impl ThroughputMeter {
    /// Creates a meter starting at `now` with the given EMA smoothing
    /// factor.
    #[must_use]
    pub fn new(now: Instant, alpha: f64) -> Self {
        Self {
            started: now,
            last_time: now,
            last_bytes: 0,
            ema: Ema::new(alpha),
            peak_bps: 0.0,
        }
    }

    /// Records the cumulative byte total at `now` and returns updated
    /// speed readings.
    pub fn update(&mut self, total_bytes: u64, now: Instant) -> ThroughputSnapshot {
        let interval = now.duration_since(self.last_time).as_secs_f64();
        let delta_bytes = total_bytes.saturating_sub(self.last_bytes);

        let current_bps = if interval > 0.0 {
            let raw_bps = (delta_bytes as f64 * 8.0) / interval;
            self.last_time = now;
            self.last_bytes = total_bytes;
            self.ema.update(raw_bps)
        } else {
            self.ema.value().unwrap_or(0.0)
        };

        let elapsed = now.duration_since(self.started).as_secs_f64();
        let average_bps = if elapsed > 0.0 {
            (total_bytes as f64 * 8.0) / elapsed
        } else {
            0.0
        };
        self.peak_bps = self.peak_bps.max(current_bps);

        ThroughputSnapshot {
            current_bps,
            average_bps,
            peak_bps: self.peak_bps,
        }
    }
}
