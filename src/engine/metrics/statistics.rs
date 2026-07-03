//! Basic statistical functions over `f64` slices.
//!
//! All functions are total: empty input yields `None` (or an empty vector)
//! instead of panicking.

/// Arithmetic mean.
#[must_use]
pub fn mean(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    Some(values.iter().sum::<f64>() / values.len() as f64)
}

/// Median of the values (the input does not need to be sorted).
#[must_use]
pub fn median(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 0 {
        Some((sorted[mid - 1] + sorted[mid]) / 2.0)
    } else {
        Some(sorted[mid])
    }
}

/// Population standard deviation.
#[must_use]
pub fn std_dev(values: &[f64]) -> Option<f64> {
    let avg = mean(values)?;
    let variance = values.iter().map(|v| (v - avg).powi(2)).sum::<f64>() / values.len() as f64;
    Some(variance.sqrt())
}

/// Returns the values sorted ascending with `fraction` trimmed from each
/// end, which removes outliers before averaging.
///
/// `fraction` is clamped to `0.0..=0.25`. Trimming never removes every
/// sample: at least one value is always kept.
#[must_use]
pub fn trimmed(values: &[f64], fraction: f64) -> Vec<f64> {
    if values.is_empty() {
        return Vec::new();
    }
    let fraction = fraction.clamp(0.0, 0.25);
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let cut = ((sorted.len() as f64) * fraction).floor() as usize;
    let end = sorted.len() - cut;
    if cut >= end {
        // Degenerate window; keep the median element.
        return vec![sorted[sorted.len() / 2]];
    }
    sorted[cut..end].to_vec()
}

/// Mean absolute difference between successive samples.
///
/// This is the classic jitter definition for latency streams. Requires at
/// least two samples.
#[must_use]
pub fn successive_diff_mean(values: &[f64]) -> Option<f64> {
    if values.len() < 2 {
        return None;
    }
    let sum: f64 = values.windows(2).map(|w| (w[1] - w[0]).abs()).sum();
    Some(sum / (values.len() - 1) as f64)
}
