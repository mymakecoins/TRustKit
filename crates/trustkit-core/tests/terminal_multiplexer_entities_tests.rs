// Tests for terminal multiplexer domain entities and validation (TASK-001).
// Constitutional Principle I: Zero external dependencies (std only in core).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use trustkit_core::entities::{
    AgentSemanticState, BlockReasonKind, MultiplexedTerminalHandle, MultiplexedTerminalRequest,
    RuntimeBackendMode,
};
use trustkit_core::errors::DomainViolation;

#[test]
fn test_multiplexed_terminal_request_valid() {
    let req = MultiplexedTerminalRequest::new(
        "TASK-001",
        "/tmp/worktree",
        vec!["claude".to_string(), "--help".to_string()],
        vec![("ENV_TEST".to_string(), "1".to_string())],
        30_000,
    );
    assert!(req.is_ok());
    let req = req.unwrap();
    assert_eq!(req.task_id, "TASK-001");
    assert_eq!(req.working_dir, "/tmp/worktree");
    assert_eq!(req.command_argv.len(), 2);
    assert_eq!(req.timeout_ms, 30_000);
}

#[test]
fn test_multiplexed_terminal_request_empty_fields_fail() {
    let empty_dir =
        MultiplexedTerminalRequest::new("TASK-001", "", vec!["claude".to_string()], vec![], 10_000);
    assert!(matches!(
        empty_dir,
        Err(DomainViolation::AstIntegrityOrInvariantViolation { .. })
    ));

    let empty_argv =
        MultiplexedTerminalRequest::new("TASK-001", "/tmp/worktree", vec![], vec![], 10_000);
    assert!(matches!(
        empty_argv,
        Err(DomainViolation::AstIntegrityOrInvariantViolation { .. })
    ));
}

#[test]
fn test_multiplexed_terminal_handle_and_states() {
    let handle = MultiplexedTerminalHandle::new("term-123", "TASK-001", 42, "2026-11-03T17:00:00Z");
    assert_eq!(handle.terminal_id, "term-123");
    assert_eq!(handle.pid, 42);

    let state_working = AgentSemanticState::Working;
    assert!(state_working.is_working());
    assert!(!state_working.is_terminal());

    let state_blocked = AgentSemanticState::Blocked {
        kind: BlockReasonKind::Permission,
        detail: "Execute bash command".to_string(),
    };
    assert!(state_blocked.is_blocked());
    assert!(!state_blocked.is_terminal());

    let state_done = AgentSemanticState::Done { exit_code: 0 };
    assert!(state_done.is_done());
    assert!(state_done.is_terminal());

    let state_failed = AgentSemanticState::Failed {
        reason: "Process exited with 1".to_string(),
    };
    assert!(state_failed.is_failed());
    assert!(state_failed.is_terminal());
}

#[test]
fn test_runtime_backend_mode_parsing() {
    assert_eq!(
        RuntimeBackendMode::from_str_name("auto"),
        RuntimeBackendMode::Auto
    );
    assert_eq!(
        RuntimeBackendMode::from_str_name("pty"),
        RuntimeBackendMode::Pty
    );
    assert_eq!(
        RuntimeBackendMode::from_str_name("native"),
        RuntimeBackendMode::Native
    );
    assert_eq!(
        RuntimeBackendMode::from_str_name("unknown"),
        RuntimeBackendMode::Auto
    );
}
