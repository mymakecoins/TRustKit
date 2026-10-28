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
    #[serde(skip_serializing_if = "String::is_empty")]
    status: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    error_kind: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    cycle_path: Vec<String>,
    #[serde(skip_serializing_if = "String::is_empty")]
    rejection_kind: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    orphan_rule: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    message: String,
    violation: JsonViolationDetails,
}

pub struct CliBoundaryErrorHandler;

impl CliBoundaryErrorHandler {
    pub fn handle_result(result: Result<DomainOutcome, DomainViolation>, json_mode: bool) -> i32 {
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
                    let _ = writeln!(writer, "[FAIL] [EXIT-{exit_code}] {violation}");
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
            DomainViolation::MissingBusinessRules { file_path, details } => (
                "SPEC_INGESTION_ERROR".to_string(),
                String::new(),
                String::new(),
                format!("Missing business rules in {file_path}: {details}"),
                "Ensure upstream PRD contains required Business Rules (RN) and Functional Requirements (RF) tables.".to_string(),
            ),
            DomainViolation::TraceabilityGap { orphan_rule, message } => (
                "TraceabilityGap".to_string(),
                orphan_rule.clone(),
                String::new(),
                message.clone(),
                "Map all business rules to tasks and E2E test scenarios in the traceability matrix.".to_string(),
            ),
            DomainViolation::CycleDetected { cycle_path, message } => (
                "DagValidationError::CycleDetected".to_string(),
                String::new(),
                String::new(),
                format!("{message} Cycle: {}", cycle_path.join(" -> ")),
                "Break circular dependencies in task definitions.".to_string(),
            ),
        };

        let verdict = if exit_code == 4 {
            "BLOCKED".to_string()
        } else {
            "FAIL".to_string()
        };

        let (status, error_kind, cycle_path, rejection_kind, orphan_rule, message) = match violation
        {
            DomainViolation::TraceabilityGap {
                orphan_rule,
                message,
            } => (
                "SPEC_LOCK_REJECTED".to_string(),
                String::new(),
                vec![],
                "TraceabilityGap".to_string(),
                orphan_rule.clone(),
                message.clone(),
            ),
            DomainViolation::CycleDetected {
                cycle_path,
                message,
            } => (
                "CYCLE_DETECTED".to_string(),
                "DagValidationError::CycleDetected".to_string(),
                cycle_path.clone(),
                String::new(),
                String::new(),
                message.clone(),
            ),
            _ => (
                String::new(),
                String::new(),
                vec![],
                String::new(),
                String::new(),
                String::new(),
            ),
        };

        JsonErrorEnvelope {
            verdict,
            exit_code,
            status,
            error_kind,
            cycle_path,
            rejection_kind,
            orphan_rule,
            message,
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
#[path = "error_handler_tests.rs"]
mod tests;
