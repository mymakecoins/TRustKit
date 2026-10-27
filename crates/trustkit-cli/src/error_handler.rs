// CLI boundary error handler translating domain outcomes and violations into POSIX exit codes.
// Constitutional Principle IV: Resilient Error Handling & Zero-Panic Policy.
// Constitutional Principle II: Absolute Prohibition of NULL.

use serde::Serialize;
use std::io::{self, Write};
use trustkit_core::errors::DomainViolation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainOutcome {
    Success,
    PassWithDetails(String),
}

#[derive(Debug, Serialize)]
struct JsonViolationDetails {
    kind: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    invariant_id: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    intercepted_command: String,
    reason: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    remediation: String,
}

#[derive(Debug, Serialize)]
struct JsonErrorEnvelope {
    verdict: String,
    exit_code: i32,
    violation: JsonViolationDetails,
}

pub struct CliBoundaryErrorHandler;

impl CliBoundaryErrorHandler {
    pub fn handle_result(
        result: Result<DomainOutcome, DomainViolation>,
        json_mode: bool,
    ) -> i32 {
        Self::handle_result_with_writer(result, json_mode, &mut io::stderr())
    }

    pub fn handle_result_with_writer<W: Write>(
        result: Result<DomainOutcome, DomainViolation>,
        json_mode: bool,
        writer: &mut W,
    ) -> i32 {
        match result {
            Ok(_) => 0,
            Err(violation) => {
                let exit_code = violation.posix_exit_code();
                if json_mode {
                    let payload = Self::build_json_payload(&violation, exit_code);
                    if let Ok(json_str) = serde_json::to_string_pretty(&payload) {
                        let _ = writeln!(writer, "{json_str}");
                    }
                } else {
                    let _ = writeln!(
                        writer,
                        "[FAIL] [EXIT-{exit_code}] {violation}"
                    );
                }
                exit_code
            }
        }
    }

    fn build_json_payload(violation: &DomainViolation, exit_code: i32) -> JsonErrorEnvelope {
        let (kind, invariant_id, intercepted_command, reason, remediation) = match violation {
            DomainViolation::InfrastructureBoundaryFault { subsystem, details } => (
                "INFRASTRUCTURE_BOUNDARY_FAULT".to_string(),
                String::new(),
                String::new(),
                format!("{subsystem}: {details}"),
                "Verify system permissions and filesystem state.".to_string(),
            ),
            DomainViolation::SpecOrTraceabilityIncomplete {
                reason,
                missing_artefact,
            } => (
                "SPEC_OR_TRACEABILITY_INCOMPLETE".to_string(),
                String::new(),
                String::new(),
                format!("{reason} (missing {missing_artefact})"),
                "Complete the specification and traceability matrix before continuing.".to_string(),
            ),
            DomainViolation::TddLifecycleViolation { task_id, reason } => (
                "TDD_LIFECYCLE_VIOLATION".to_string(),
                String::new(),
                String::new(),
                format!("Task {task_id}: {reason}"),
                "Capture RED failure evidence prior to implementing production code.".to_string(),
            ),
            DomainViolation::AstIntegrityOrInvariantViolation {
                invariant_id,
                intercepted_command,
                reason,
                remediation,
            } => (
                "FORBIDDEN_COMMAND_INTERCEPTED".to_string(),
                invariant_id.clone(),
                intercepted_command.clone(),
                reason.clone(),
                remediation.clone(),
            ),
            DomainViolation::CriticSwarmOrPlaywrightFailed {
                layer,
                attempt,
                failure_summary,
            } => (
                "CRITIC_SWARM_OR_PLAYWRIGHT_FAILED".to_string(),
                String::new(),
                String::new(),
                format!("Layer {layer} attempt {attempt}: {failure_summary}"),
                "Address test failures and re-run gauntlet.".to_string(),
            ),
            DomainViolation::CircuitBreakerTripped {
                cycle_id,
                consecutive_failures,
                remediation,
            } => (
                "CIRCUIT_BREAKER_TRIPPED".to_string(),
                String::new(),
                String::new(),
                format!("Cycle {cycle_id} failed {consecutive_failures} consecutive times."),
                remediation.clone(),
            ),
        };

        let verdict = if exit_code == 4 {
            "BLOCKED".to_string()
        } else {
            "FAIL".to_string()
        };

        JsonErrorEnvelope {
            verdict,
            exit_code,
            violation: JsonViolationDetails {
                kind,
                invariant_id,
                intercepted_command,
                reason,
                remediation,
            },
        }
    }
}

#[cfg(test)]
mod tests {
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
        let code = CliBoundaryErrorHandler::handle_result_with_writer(
            Err(violation),
            true,
            &mut buf,
        );
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
        let code = CliBoundaryErrorHandler::handle_result_with_writer(
            Err(violation),
            false,
            &mut buf,
        );
        assert_eq!(code, 3);
        let output = String::from_utf8(buf).expect("valid utf8");
        assert!(output.contains("[FAIL] [EXIT-3]"));
        assert!(output.contains("TASK-001"));
    }
}
