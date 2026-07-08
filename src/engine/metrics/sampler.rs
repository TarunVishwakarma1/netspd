//! Fixed-capacity rolling sample buffer.

use std::collections::VecDeque;

/// A rolling window of the most recent `capacity` samples.
///
/// Useful for sparklines and rolling statistics; pushing beyond capacity
/// evicts the oldest sample. The buffer never reallocates after creation.
#[derive(Debug, Clone)]
pub struct Sampler {
    capacity: usize,
    samples: VecDeque<f64>,
}

impl Sampler {
    /// Creates a sampler holding at most `capacity` samples
    /// (a capacity of zero is bumped to one).
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        Self {
            capacity,
            samples: VecDeque::with_capacity(capacity),
        }
    }

    /// Records a sample, evicting the oldest one when full.
    pub fn push(&mut self, sample: f64) {
        if self.samples.len() == self.capacity {
            self.samples.pop_front();
        }
        self.samples.push_back(sample);
    }

    /// Iterates samples from oldest to newest.
    pub fn iter(&self) -> impl Iterator<Item = f64> + '_ {
        self.samples.iter().copied()
    }

    /// The most recently pushed sample.
    #[must_use]
    pub fn latest(&self) -> Option<f64> {
        self.samples.back().copied()
    }

    /// The smallest sample currently in the window.
    #[must_use]
    pub fn min(&self) -> Option<f64> {
        self.samples
            .iter()
            .copied()
            .fold(None, |acc, s| Some(acc.map_or(s, |a: f64| a.min(s))))
    }

    /// The largest sample currently in the window.
    #[must_use]
    pub fn max(&self) -> Option<f64> {
        self.samples
            .iter()
            .copied()
            .fold(None, |acc, s| Some(acc.map_or(s, |a: f64| a.max(s))))
    }

    /// Number of samples currently held.
    #[must_use]
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Whether the window is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Removes all samples, keeping the allocated capacity.
    pub fn clear(&mut self) {
        self.samples.clear();
    }
}
