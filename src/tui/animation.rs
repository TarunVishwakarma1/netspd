//! Animation primitives: spinner frames and smooth value interpolation.

/// Braille spinner frames, one per UI tick.
const SPINNER_FRAMES: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// Returns the spinner glyph for a given UI tick.
#[must_use]
pub fn spinner_frame(tick: u64) -> &'static str {
    SPINNER_FRAMES[(tick as usize) % SPINNER_FRAMES.len()]
}

/// A value that eases smoothly toward its target.
///
/// Uses framerate-independent exponential smoothing, so counters and bars
/// animate identically at any refresh rate.
#[derive(Debug, Clone, Copy)]
pub struct AnimatedValue {
    current: f64,
    target: f64,
    /// Approach rate: higher values converge faster.
    rate: f64,
}

impl AnimatedValue {
    /// Creates a value at rest at zero with the given approach rate.
    #[must_use]
    pub fn new(rate: f64) -> Self {
        Self {
            current: 0.0,
            target: 0.0,
            rate: rate.max(0.1),
        }
    }

    /// Sets a new target to ease toward.
    pub fn set_target(&mut self, target: f64) {
        if target.is_finite() {
            self.target = target;
        }
    }

    /// Snaps immediately to a value (used on resets).
    pub fn snap(&mut self, value: f64) {
        self.current = value;
        self.target = value;
    }

    /// Advances the animation by `dt` seconds and returns the new value.
    pub fn tick(&mut self, dt: f64) -> f64 {
        let blend = 1.0 - (-self.rate * dt.max(0.0)).exp();
        self.current += (self.target - self.current) * blend;
        if (self.target - self.current).abs() < 1e-9 {
            self.current = self.target;
        }
        self.current
    }

    /// The current animated value.
    #[must_use]
    pub fn value(&self) -> f64 {
        self.current
    }

    /// Whether the value is still visibly moving toward its target.
    #[must_use]
    pub fn is_animating(&self) -> bool {
        (self.target - self.current).abs() > f64::EPSILON * self.target.abs().max(1.0)
    }
}
