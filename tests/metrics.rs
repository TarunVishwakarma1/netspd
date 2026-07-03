//! Unit tests for the metric calculators.

use std::time::{Duration, Instant};

use netspd::engine::metrics::{statistics, Ema, LatencyCalculator, Sampler, ThroughputMeter};

const TOLERANCE: f64 = 1e-9;

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < TOLERANCE
}

#[test]
fn ema_seeds_with_first_sample() {
    let mut ema = Ema::new(0.5);
    assert!(ema.value().is_none());
    assert!(close(ema.update(10.0), 10.0));
}

#[test]
fn ema_smooths_toward_new_samples() {
    let mut ema = Ema::new(0.5);
    ema.update(10.0);
    assert!(close(ema.update(20.0), 15.0));
    assert!(close(ema.update(20.0), 17.5));
}

#[test]
fn ema_reset_clears_state() {
    let mut ema = Ema::new(0.3);
    ema.update(42.0);
    ema.reset();
    assert!(ema.value().is_none());
}

#[test]
fn ema_clamps_invalid_alpha() {
    let mut ema = Ema::new(5.0);
    ema.update(10.0);
    // Alpha clamped to 1.0: follows samples exactly.
    assert!(close(ema.update(30.0), 30.0));
}

#[test]
fn sampler_evicts_oldest_beyond_capacity() {
    let mut sampler = Sampler::new(3);
    for value in [1.0, 2.0, 3.0, 4.0] {
        sampler.push(value);
    }
    assert_eq!(sampler.len(), 3);
    let values: Vec<f64> = sampler.iter().collect();
    assert_eq!(values, vec![2.0, 3.0, 4.0]);
    assert_eq!(sampler.latest(), Some(4.0));
    assert_eq!(sampler.max(), Some(4.0));
}

#[test]
fn sampler_clear_keeps_capacity_working() {
    let mut sampler = Sampler::new(2);
    sampler.push(1.0);
    sampler.clear();
    assert!(sampler.is_empty());
    sampler.push(5.0);
    assert_eq!(sampler.latest(), Some(5.0));
}

#[test]
fn statistics_mean_median_stddev() {
    let values = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
    assert!(close(statistics::mean(&values).unwrap_or(0.0), 5.0));
    assert!(close(statistics::median(&values).unwrap_or(0.0), 4.5));
    assert!(close(statistics::std_dev(&values).unwrap_or(0.0), 2.0));
}

#[test]
fn statistics_empty_inputs_yield_none() {
    assert!(statistics::mean(&[]).is_none());
    assert!(statistics::median(&[]).is_none());
    assert!(statistics::std_dev(&[]).is_none());
    assert!(statistics::successive_diff_mean(&[1.0]).is_none());
    assert!(statistics::trimmed(&[], 0.1).is_empty());
}

#[test]
fn statistics_trimmed_removes_outliers() {
    let values = [100.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 0.0];
    let trimmed = statistics::trimmed(&values, 0.1);
    assert_eq!(trimmed.len(), 8);
    assert!(!trimmed.contains(&100.0));
    assert!(!trimmed.contains(&0.0));
}

#[test]
fn statistics_jitter_is_mean_successive_difference() {
    let values = [10.0, 12.0, 9.0, 13.0];
    // |12-10| + |9-12| + |13-9| = 2 + 3 + 4 = 9; 9 / 3 = 3.
    assert!(close(
        statistics::successive_diff_mean(&values).unwrap_or(0.0),
        3.0
    ));
}

#[test]
fn latency_calculator_aggregates_and_counts_loss() {
    let mut calc = LatencyCalculator::new();
    for sample in [10.0, 11.0, 12.0, 13.0] {
        calc.record(sample);
    }
    calc.record_failure();
    let stats = calc.stats();
    assert!(stats.is_some());
    let Some(stats) = stats else { return };
    assert_eq!(stats.samples, 4);
    assert!(close(stats.min_ms, 10.0));
    assert!(close(stats.max_ms, 13.0));
    assert!(close(stats.packet_loss_pct, 20.0));
    assert!(stats.jitter_ms > 0.0);
}

#[test]
fn latency_calculator_rejects_invalid_samples() {
    let mut calc = LatencyCalculator::new();
    calc.record(f64::NAN);
    calc.record(-5.0);
    assert!(calc.stats().is_none());
    assert_eq!(calc.sample_count(), 0);
}

#[test]
fn throughput_meter_computes_speeds() {
    let start = Instant::now();
    let mut meter = ThroughputMeter::new(start, 1.0);
    // 1_000_000 bytes over exactly one second = 8 Mbps.
    let snapshot = meter.update(1_000_000, start + Duration::from_secs(1));
    assert!(close(snapshot.current_bps, 8_000_000.0));
    assert!(close(snapshot.average_bps, 8_000_000.0));
    assert!(close(snapshot.peak_bps, 8_000_000.0));
}

#[test]
fn throughput_meter_tracks_peak() {
    let start = Instant::now();
    let mut meter = ThroughputMeter::new(start, 1.0);
    meter.update(2_000_000, start + Duration::from_secs(1));
    let slower = meter.update(2_500_000, start + Duration::from_secs(2));
    assert!(slower.current_bps < slower.peak_bps);
    assert!(close(slower.peak_bps, 16_000_000.0));
}
