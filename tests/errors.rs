//! Tests for user-facing error messages.

use netspd::errors::EngineError;

#[test]
fn no_samples_error_includes_failure_reason() {
    let err = EngineError::NoSamples {
        reason: "invalid peer certificate: NotValidForName".to_owned(),
    };
    let message = err.to_string();
    assert!(message.contains("no measurement samples were collected"));
    assert!(message.contains("invalid peer certificate: NotValidForName"));
}

#[test]
fn no_samples_error_is_not_cancellation() {
    let err = EngineError::NoSamples {
        reason: "request timed out".to_owned(),
    };
    assert!(!err.is_cancelled());
}
