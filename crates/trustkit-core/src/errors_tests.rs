// Unit tests for domain error hierarchy and POSIX exit code categorization.

use super::*;

#[test]
fn test_posix_exit_code_mapping() {
    let fault = DomainViolation::InfrastructureBoundaryFault {
        subsystem: "sqlite".to_string(),
        details: "busy".to_string(),
    };
    assert_eq!(fault.posix_exit_code(), 1);

    let empty_backlog = DomainViolation::EmptyTaskBacklogFault {
        cycle_id: "CYC-001".to_string(),
        reason: "Zero tasks".to_string(),
    };
    assert_eq!(empty_backlog.posix_exit_code(), 1);

    let spec_inc = DomainViolation::SpecOrTraceabilityIncomplete {
        reason: "Missing Gherkin".to_string(),
        missing_artefact: "spec.md".to_string(),
    };
    assert_eq!(spec_inc.posix_exit_code(), 2);

    let missing_rules = DomainViolation::MissingBusinessRules {
        file_path: "prd.md".to_string(),
        details: "No RN found".to_string(),
    };
    assert_eq!(missing_rules.posix_exit_code(), 2);

    let tdd = DomainViolation::TddLifecycleViolation {
        task_id: "TASK-001".to_string(),
        reason: "No red evidence".to_string(),
    };
    assert_eq!(tdd.posix_exit_code(), 3);

    let zero_tests = DomainViolation::ZeroFilteredTestsFault {
        task_id: "TASK-001".to_string(),
        reason: "0 passed; 0 failed".to_string(),
    };
    assert_eq!(zero_tests.posix_exit_code(), 3);

    let compile_fail = DomainViolation::CompilationFailureInRedPhaseFault {
        task_id: "TASK-001".to_string(),
        reason: "error[E0425]".to_string(),
    };
    assert_eq!(compile_fail.posix_exit_code(), 3);

    let gap = DomainViolation::TraceabilityGap {
        orphan_rule: "RN-002".to_string(),
        message: "Orphan rule without task".to_string(),
    };
    assert_eq!(gap.posix_exit_code(), 3);

    let invariant = DomainViolation::AstIntegrityOrInvariantViolation {
        invariant_id: "RN-006".to_string(),
        intercepted_command: "git add .".to_string(),
        reason: "Sweeping add forbidden".to_string(),
        remediation: "Add explicit files".to_string(),
    };
    assert_eq!(invariant.posix_exit_code(), 4);

    let cycle = DomainViolation::CycleDetected {
        cycle_path: vec!["T1".into(), "T2".into(), "T1".into()],
        message: "Cycle".to_string(),
    };
    assert_eq!(cycle.posix_exit_code(), 4);

    let critic = DomainViolation::CriticSwarmOrPlaywrightFailed {
        layer: "E2E".to_string(),
        attempt: 2,
        failure_summary: "Timeout".to_string(),
    };
    assert_eq!(critic.posix_exit_code(), 5);

    let cb = DomainViolation::CircuitBreakerTripped {
        cycle_id: "cycle-1".to_string(),
        consecutive_failures: 5,
        remediation: "Human review".to_string(),
    };
    assert_eq!(cb.posix_exit_code(), 6);
}

#[test]
fn test_error_display() {
    let err = DomainViolation::TddLifecycleViolation {
        task_id: "TASK-005".to_string(),
        reason: "Green before Red".to_string(),
    };
    assert!(format!("{err}").contains("TASK-005"));
}
