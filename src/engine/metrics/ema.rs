//! Exponential moving average.

/// An exponential moving average over a stream of samples.
///
/// The smoothing factor `alpha` is clamped to `(0.0, 1.0]`; higher values
/// react faster, lower values smooth harder.
#[derive(Debug, Clone, Copy)]
pub struct Ema {
    alpha: f64,
    value: Option<f64>,
}

impl Ema {
    /// Creates a new EMA with the given smoothing factor.
    #[must_use]
    pub fn new(alpha: f64) -> Self {
        Self {
            alpha: alpha.clamp(f64::EPSILON, 1.0),
            value: None,
        }
    }

    /// Feeds a sample and returns the updated average.
    ///
    /// The first sample seeds the average directly.
    pub fn update(&mut self, sample: f64) -> f64 {
        let next = match self.value {
            Some(current) => current + self.alpha * (sample - current),
            None => sample,
        };
        self.value = Some(next);
        next
    }

    /// The current average, if at least one sample was recorded.
    #[must_use]
    pub fn value(&self) -> Option<f64> {
        self.value
    }

    /// Clears all recorded state, keeping the smoothing factor.
    pub fn reset(&mut self) {
        self.value = None;
    }
}
