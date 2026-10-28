use super::*;

#[test]
fn test_handle_result_success_returns_zero() {
    let mut buf = Vec::new();
    let code = CliBoundaryErrorHandler::handle_result_with_writer(
        Ok(DomainOutcome::Success),
        false,
        &mut buf,
    );
    assert_eq!(code, 0);
    assert!(buf.is_empty());
}

#[test]
fn test_handle_result_invariant_violation_json() {
    let mut buf = Vec::new();
    let violation = DomainViolation::AstIntegrityOrInvariantViolation {
        invariant_id: "RN-006".to_string(),
        intercepted_command: "git add .".to_string(),
        reason: "Sweeping add forbidden".to_string(),
        remediation: "Stage explicit files".to_string(),
    };
    let code = CliBoundaryErrorHandler::handle_result_with_writer(Err(violation), true, &mut buf);
    assert_eq!(code, 4);
    let output = String::from_utf8(buf).expect("valid utf8");
    assert!(output.contains("\"verdict\": \"BLOCKED\""));
    assert!(output.contains("\"exit_code\": 4"));
    assert!(output.contains("\"invariant_id\": \"RN-006\""));
    assert!(!output.contains(": null"));
}

#[test]
fn test_handle_result_human_mode() {
    let mut buf = Vec::new();
    let violation = DomainViolation::TddLifecycleViolation {
        task_id: "TASK-001".to_string(),
        reason: "Missing red evidence".to_string(),
    };
    let code = CliBoundaryErrorHandler::handle_result_with_writer(Err(violation), false, &mut buf);
    assert_eq!(code, 3);
    let output = String::from_utf8(buf).expect("valid utf8");
    assert!(output.contains("[FAIL] [EXIT-3]"));
    assert!(output.contains("TASK-001"));
}

#[test]
fn test_handle_result_cycle_detected_json() {
    let mut buf = Vec::new();
    let violation = DomainViolation::CycleDetected {
        cycle_path: vec![
            "TASK-001".to_string(),
            "TASK-002".to_string(),
            "TASK-001".to_string(),
        ],
        message: "Dependencia circular detectada".to_string(),
    };
    let code = CliBoundaryErrorHandler::handle_result_with_writer(Err(violation), true, &mut buf);
    assert_eq!(code, 4);
    let output = String::from_utf8(buf).expect("valid utf8");
    assert!(output.contains("\"verdict\": \"BLOCKED\""));
    assert!(output.contains("\"exit_code\": 4"));
    assert!(output.contains("\"status\": \"CYCLE_DETECTED\""));
    assert!(output.contains("\"error_kind\": \"DagValidationError::CycleDetected\""));
    assert!(output.contains("\"TASK-001\""));
    assert!(!output.contains(": null"));
}
