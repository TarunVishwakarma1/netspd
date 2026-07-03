//! Latency aggregation: trimmed mean, jitter and loss.

use crate::engine::models::LatencyStats;

use super::statistics;

/// Fraction of samples trimmed from each end before averaging.
const TRIM_FRACTION: f64 = 0.1;

/// Collects individual latency samples and failures, then produces
/// [`LatencyStats`].
#[derive(Debug, Clone, Default)]
pub struct LatencyCalculator {
    samples: Vec<f64>,
    failures: u32,
}

impl LatencyCalculator {
    /// Creates an empty calculator.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a successful round-trip time, in milliseconds.
    pub fn record(&mut self, latency_ms: f64) {
        if latency_ms.is_finite() && latency_ms >= 0.0 {
            self.samples.push(latency_ms);
        }
    }

    /// Records a failed sample (counts toward packet loss).
    pub fn record_failure(&mut self) {
        self.failures += 1;
    }

    /// Number of successful samples recorded so far.
    #[must_use]
    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    /// Produces aggregated statistics.
    ///
    /// Returns `None` when no successful sample was recorded. The average
    /// uses a trimmed mean to discard outliers; jitter is the mean absolute
    /// successive difference over the raw sample order.
    #[must_use]
    pub fn stats(&self) -> Option<LatencyStats> {
        let trimmed = statistics::trimmed(&self.samples, TRIM_FRACTION);
        let average_ms = statistics::mean(&trimmed)?;
        let jitter_ms = statistics::successive_diff_mean(&self.samples).unwrap_or(0.0);
        let min_ms = self.samples.iter().copied().fold(f64::INFINITY, f64::min);
        let max_ms = self
            .samples
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        let total = self.samples.len() as f64 + f64::from(self.failures);
        let packet_loss_pct = if total > 0.0 {
            f64::from(self.failures) / total * 100.0
        } else {
            0.0
        };
        Some(LatencyStats {
            average_ms,
            jitter_ms,
            min_ms,
            max_ms,
            samples: self.samples.len(),
            packet_loss_pct,
        })
    }
}
