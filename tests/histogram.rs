//! Tests for the ping histogram shown on the results screen.

use netspd::engine::metrics::Sampler;

#[test]
fn sampler_min_and_max() {
    let mut s = Sampler::new(10);
    s.push(30.0);
    s.push(10.0);
    s.push(20.0);
    assert_eq!(s.min(), Some(10.0));
    assert_eq!(s.max(), Some(30.0));
}

#[test]
fn sampler_min_max_empty() {
    let s = Sampler::new(10);
    assert_eq!(s.min(), None);
    assert_eq!(s.max(), None);
}

#[test]
fn sampler_min_max_single() {
    let mut s = Sampler::new(10);
    s.push(42.0);
    assert_eq!(s.min(), Some(42.0));
    assert_eq!(s.max(), Some(42.0));
}

#[test]
fn evenly_distributed_samples_fill_all_bins() {
    // Push one sample per bin of a 0–80 ms range (8 bins of 10 ms each).
    let mut s = Sampler::new(20);
    for i in 0..8u32 {
        s.push(f64::from(i) * 10.0 + 5.0); // 5, 15, 25, 35, 45, 55, 65, 75
    }
    // All 8 bins should receive exactly 1 sample.
    assert_eq!(s.len(), 8);
    let lo = s.min().unwrap_or(0.0);
    let hi = s.max().unwrap_or(80.0);
    assert!(lo < hi);
}

#[test]
fn all_identical_samples_produce_near_zero_spread() {
    let mut s = Sampler::new(10);
    for _ in 0..10 {
        s.push(12.0);
    }
    let lo = s.min().unwrap_or(0.0);
    let hi = s.max().unwrap_or(0.0);
    // Spread < 0.5 ms means no histogram is shown.
    assert!(hi - lo < 0.5);
}

#[test]
fn rolling_window_evicts_oldest() {
    let mut s = Sampler::new(3);
    s.push(1.0);
    s.push(2.0);
    s.push(3.0);
    s.push(100.0); // evicts 1.0
    assert_eq!(s.min(), Some(2.0));
    assert_eq!(s.len(), 3);
}
