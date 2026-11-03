// Tests for SupervisorRuntimeSelector and fallback behavior (TASK-005).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use std::path::PathBuf;
use trustkit_core::entities::RuntimeBackendMode;
use trustkit_core::errors::DomainViolation;
use trustkit_orch::supervisor_runtime_selector::SupervisorRuntimeSelector;

#[test]
fn test_selector_auto_fallback_to_native_when_socket_missing() {
    let selector = SupervisorRuntimeSelector::with_custom_socket(PathBuf::from("/tmp/non_existent_herdr_123.sock"));
    let resolved = selector.resolve(RuntimeBackendMode::Auto).expect("Auto should not fail");
    assert_eq!(resolved.active_mode, RuntimeBackendMode::Native);
    assert!(resolved.fallback_triggered);
    assert!(resolved.multiplexer.is_none());
}

#[test]
fn test_selector_explicit_native() {
    let selector = SupervisorRuntimeSelector::with_custom_socket(PathBuf::from("/tmp/non_existent_herdr_123.sock"));
    let resolved = selector.resolve(RuntimeBackendMode::Native).expect("Native should not fail");
    assert_eq!(resolved.active_mode, RuntimeBackendMode::Native);
    assert!(!resolved.fallback_triggered);
    assert!(resolved.multiplexer.is_none());
}

#[test]
fn test_selector_explicit_herdr_fails_when_socket_missing() {
    let selector = SupervisorRuntimeSelector::with_custom_socket(PathBuf::from("/tmp/non_existent_herdr_123.sock"));
    let err = selector.resolve(RuntimeBackendMode::Herdr);
    assert!(matches!(
        err,
        Err(DomainViolation::InfrastructureBoundaryFault { .. })
    ));
}

#[test]
fn test_selector_resolves_herdr_when_socket_exists() {
    let temp_dir = tempfile::tempdir().unwrap();
    let socket_path = temp_dir.path().join("mock_herdr.sock");
    std::fs::write(&socket_path, b"mock").unwrap();

    let selector = SupervisorRuntimeSelector::with_custom_socket(socket_path);
    let resolved = selector.resolve(RuntimeBackendMode::Herdr).expect("Should resolve Herdr");
    assert_eq!(resolved.active_mode, RuntimeBackendMode::Herdr);
    assert!(!resolved.fallback_triggered);
    assert!(resolved.multiplexer.is_some());
}
