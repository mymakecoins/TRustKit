// Tests for OSC 7501 telemetry encoder (TASK-002).
// Constitutional Principle I: Zero external dependencies (std only in core).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use trustkit_core::entities::{Osc7501Payload, encode_osc7501};
use trustkit_core::errors::DomainViolation;

#[test]
fn test_encode_osc7501_nominal() {
    let payload = Osc7501Payload {
        state: "working".to_string(),
        id: "task-001".to_string(),
        kind: "".to_string(),
        progress: Some(45),
        app: "trustkit".to_string(),
        title: "Building".to_string(),
        msg: "Compiling crates".to_string(),
    };

    let encoded = encode_osc7501(&payload).expect("Should encode nominal payload");
    assert!(encoded.starts_with("\x1b]7501;"));
    assert!(encoded.ends_with("\x1b\\"));
    assert!(encoded.contains("state=working"));
    assert!(encoded.contains("id=task-001"));
    assert!(encoded.contains("progress=45"));
    assert!(encoded.contains("app=trustkit"));
}

#[test]
fn test_encode_osc7501_blocked() {
    let payload = Osc7501Payload {
        state: "blocked".to_string(),
        id: "task-002".to_string(),
        kind: "permission".to_string(),
        progress: None,
        app: "trustkit".to_string(),
        title: "Waiting approval".to_string(),
        msg: "Run command".to_string(),
    };

    let encoded = encode_osc7501(&payload).expect("Should encode blocked payload");
    assert!(encoded.contains("state=blocked"));
    assert!(encoded.contains("kind=permission"));
    assert!(!encoded.contains("progress="));
}

#[test]
fn test_encode_osc7501_rejects_emojis() {
    let payload = Osc7501Payload {
        state: "working".to_string(),
        id: "task-003".to_string(),
        kind: "".to_string(),
        progress: Some(10),
        app: "trustkit".to_string(),
        title: "Test with emoji 🚀".to_string(),
        msg: "Message".to_string(),
    };

    let result = encode_osc7501(&payload);
    assert!(matches!(
        result,
        Err(DomainViolation::AstIntegrityOrInvariantViolation { .. })
    ));
}
