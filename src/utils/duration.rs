//! Parsing of human-friendly duration strings.

use std::time::Duration;

/// The shortest accepted repeat interval; a full test takes ~25 s and
/// hammering servers faster than this helps nobody.
pub const MIN_INTERVAL: Duration = Duration::from_secs(30);

/// Parses durations like `45s`, `10m`, `2h` or plain seconds (`90`).
///
/// Compound forms (`1h30m`) are also accepted. Returns an error message
/// suitable for CLI display on anything unparseable or zero.
pub fn parse_duration(value: &str) -> Result<Duration, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err("empty duration".to_owned());
    }

    let mut total = Duration::ZERO;
    let mut digits = String::new();
    for ch in value.chars() {
        match ch {
            '0'..='9' => digits.push(ch),
            's' | 'm' | 'h' => {
                let amount: u64 = digits
                    .parse()
                    .map_err(|_| format!("invalid duration {value:?}"))?;
                digits.clear();
                let unit = match ch {
                    's' => 1,
                    'm' => 60,
                    _ => 3600,
                };
                total += Duration::from_secs(amount * unit);
            }
            _ => {
                return Err(format!(
                    "invalid duration {value:?}; use forms like 45s, 10m, 2h"
                ))
            }
        }
    }
    if !digits.is_empty() {
        // Bare number: seconds.
        let amount: u64 = digits
            .parse()
            .map_err(|_| format!("invalid duration {value:?}"))?;
        total += Duration::from_secs(amount);
    }
    if total.is_zero() {
        return Err("duration must be greater than zero".to_owned());
    }
    Ok(total)
}

/// Parses a repeat interval, enforcing the [`MIN_INTERVAL`] floor.
pub fn parse_interval(value: &str) -> Result<Duration, String> {
    Ok(parse_duration(value)?.max(MIN_INTERVAL))
}
