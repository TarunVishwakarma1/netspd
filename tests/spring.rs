//! Tests for the damped spring animator behind the dial needle.

use netspd::tui::animation::SpringValue;

/// Advances the spring in 60 fps steps for `seconds`.
fn simulate(spring: &mut SpringValue, seconds: f64) -> Vec<f64> {
    let steps = (seconds * 60.0) as usize;
    (0..steps).map(|_| spring.tick(1.0 / 60.0)).collect()
}

#[test]
fn spring_converges_to_target() {
    let mut spring = SpringValue::new(60.0, 9.0);
    spring.set_target(100.0);
    simulate(&mut spring, 5.0);
    assert!((spring.value() - 100.0).abs() < 0.5);
}

#[test]
fn underdamped_spring_overshoots() {
    let mut spring = SpringValue::new(60.0, 9.0);
    spring.set_target(100.0);
    let path = simulate(&mut spring, 3.0);
    let max = path.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    assert!(max > 100.5, "needle should overshoot, peaked at {max}");
}

#[test]
fn snap_kills_motion() {
    let mut spring = SpringValue::new(60.0, 9.0);
    spring.set_target(500.0);
    simulate(&mut spring, 0.2);
    spring.snap(0.0);
    let path = simulate(&mut spring, 0.5);
    assert!(path.iter().all(|value| value.abs() < 1e-9));
}

#[test]
fn huge_frame_deltas_stay_stable() {
    let mut spring = SpringValue::new(60.0, 9.0);
    spring.set_target(100.0);
    // A stalled frame (e.g. terminal suspended) must not explode.
    let value = spring.tick(10.0);
    assert!(value.is_finite());
    assert!((spring.tick(5.0) - 100.0).abs() < 1.0);
}

#[test]
fn non_finite_targets_are_ignored() {
    let mut spring = SpringValue::new(60.0, 9.0);
    spring.set_target(f64::NAN);
    let value = spring.tick(0.1);
    assert!(value.is_finite());
    assert!(value.abs() < 1e-9);
}
