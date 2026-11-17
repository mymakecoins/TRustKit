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
    EmptyTaskBacklogFault {
        cycle_id: String,
        reason: String,
    },
    SpecOrTraceabilityIncomplete {
        reason: String,
        missing_artefact: String,
    },
    TddLifecycleViolation {
        task_id: String,
        reason: String,
    },
    ZeroFilteredTestsFault {
        task_id: String,
        reason: String,
    },
    CompilationFailureInRedPhaseFault {
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
            Self::InfrastructureBoundaryFault { .. } | Self::EmptyTaskBacklogFault { .. } => {
                ExitCodeCategory::InfrastructureBoundaryFault
            }
            Self::SpecOrTraceabilityIncomplete { .. } | Self::MissingBusinessRules { .. } => {
                ExitCodeCategory::SpecOrTraceabilityIncomplete
            }
            Self::TddLifecycleViolation { .. }
            | Self::ZeroFilteredTestsFault { .. }
            | Self::CompilationFailureInRedPhaseFault { .. }
            | Self::TraceabilityGap { .. } => ExitCodeCategory::TddLifecycleViolation,
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
            Self::EmptyTaskBacklogFault { cycle_id, reason } => {
                write!(f, "[EMPTY_TASK_BACKLOG:{cycle_id}] {reason}")
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
            Self::ZeroFilteredTestsFault { task_id, reason } => {
                write!(f, "[ZERO_FILTERED_TESTS:{task_id}] {reason}")
            }
            Self::CompilationFailureInRedPhaseFault { task_id, reason } => {
                write!(f, "[COMPILATION_FAILURE_IN_RED:{task_id}] {reason}")
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
#[path = "errors_tests.rs"]
mod tests;
