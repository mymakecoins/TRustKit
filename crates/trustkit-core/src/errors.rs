// Domain error hierarchy and POSIX exit code categorization.
// Constitutional Principle IV: Resilient Error Handling & Zero-Panic Policy.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCodeCategory {
    CleanPass = 0,
    InfrastructureBoundaryFault = 1,
    SpecOrTraceabilityIncomplete = 2,
    TddLifecycleViolation = 3,
    AstIntegrityOrInvariantViolation = 4,
    CriticSwarmOrPlaywrightFailed = 5,
    CircuitBreakerTripped = 6,
}

impl ExitCodeCategory {
    pub const fn as_i32(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainViolation {
    InfrastructureBoundaryFault {
        subsystem: String,
        details: String,
    },
    SpecOrTraceabilityIncomplete {
        reason: String,
        missing_artefact: String,
    },
    TddLifecycleViolation {
        task_id: String,
        reason: String,
    },
    AstIntegrityOrInvariantViolation {
        invariant_id: String,
        intercepted_command: String,
        reason: String,
        remediation: String,
    },
    CriticSwarmOrPlaywrightFailed {
        layer: String,
        attempt: u8,
        failure_summary: String,
    },
    CircuitBreakerTripped {
        cycle_id: String,
        consecutive_failures: u8,
        remediation: String,
    },
    MissingBusinessRules {
        file_path: String,
        details: String,
    },
    TraceabilityGap {
        orphan_rule: String,
        message: String,
    },
    CycleDetected {
        cycle_path: Vec<String>,
        message: String,
    },
}

impl DomainViolation {
    pub const fn exit_category(&self) -> ExitCodeCategory {
        match self {
            Self::InfrastructureBoundaryFault { .. } => {
                ExitCodeCategory::InfrastructureBoundaryFault
            }
            Self::SpecOrTraceabilityIncomplete { .. } | Self::MissingBusinessRules { .. } => {
                ExitCodeCategory::SpecOrTraceabilityIncomplete
            }
            Self::TddLifecycleViolation { .. } | Self::TraceabilityGap { .. } => {
                ExitCodeCategory::TddLifecycleViolation
            }
            Self::AstIntegrityOrInvariantViolation { .. } | Self::CycleDetected { .. } => {
                ExitCodeCategory::AstIntegrityOrInvariantViolation
            }
            Self::CriticSwarmOrPlaywrightFailed { .. } => {
                ExitCodeCategory::CriticSwarmOrPlaywrightFailed
            }
            Self::CircuitBreakerTripped { .. } => ExitCodeCategory::CircuitBreakerTripped,
        }
    }

    pub const fn posix_exit_code(&self) -> i32 {
        self.exit_category().as_i32()
    }
}

impl fmt::Display for DomainViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InfrastructureBoundaryFault { subsystem, details } => {
                write!(f, "[FAULT:{subsystem}] {details}")
            }
            Self::SpecOrTraceabilityIncomplete {
                reason,
                missing_artefact,
            } => {
                write!(
                    f,
                    "[SPEC_INCOMPLETE] {reason} (missing: {missing_artefact})"
                )
            }
            Self::TddLifecycleViolation { task_id, reason } => {
                write!(f, "[TDD_VIOLATION:{task_id}] {reason}")
            }
            Self::AstIntegrityOrInvariantViolation {
                invariant_id,
                intercepted_command,
                reason,
                ..
            } => {
                write!(
                    f,
                    "[INVARIANT_BLOCKED:{invariant_id}] Command '{intercepted_command}' blocked: {reason}"
                )
            }
            Self::CriticSwarmOrPlaywrightFailed {
                layer,
                attempt,
                failure_summary,
            } => {
                write!(
                    f,
                    "[GAUNTLET_FAILED:{layer}] Attempt {attempt}/5 failed: {failure_summary}"
                )
            }
            Self::CircuitBreakerTripped {
                cycle_id,
                consecutive_failures,
                remediation,
            } => {
                write!(
                    f,
                    "[CIRCUIT_BREAKER:{cycle_id}] Locked after {consecutive_failures} failures. Remediation: {remediation}"
                )
            }
            Self::MissingBusinessRules { file_path, details } => {
                write!(
                    f,
                    "[SPEC_INGESTION_ERROR] Missing business rules in '{file_path}': {details}"
                )
            }
            Self::TraceabilityGap {
                orphan_rule,
                message,
            } => {
                write!(f, "[TRACEABILITY_GAP] Rule '{orphan_rule}': {message}")
            }
            Self::CycleDetected {
                cycle_path,
                message,
            } => {
                write!(f, "[CYCLE_DETECTED] Path {:?}: {message}", cycle_path)
            }
        }
    }
}

impl std::error::Error for DomainViolation {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_posix_exit_code_mapping() {
        let fault = DomainViolation::InfrastructureBoundaryFault {
            subsystem: "sqlite".to_string(),
            details: "busy".to_string(),
        };
        assert_eq!(fault.posix_exit_code(), 1);

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
}
