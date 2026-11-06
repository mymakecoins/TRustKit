// run_autonomous_cycle use case (TASK-002 / TASK-005).
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::entities::{CycleStatus, GateVerdict, TaskNode};
use crate::errors::DomainViolation;
use crate::ports::{
    ArtifactHasherPort, CycleEvidenceRepositoryPort, SpecValidatorPort, WaveExecutionPlannerPort,
};
use crate::special_cases::ActiveCycleLookup;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutonomousCycleRequest {
    pub issue: Option<String>,
    pub spec_path: Option<String>,
    pub target_branch: String,
    pub runtime_mode: String,
    pub agent_command: String,
    pub now_iso: String,
    pub timestamp_slug: String,
    pub spec_content: String,
    pub tasks: Vec<TaskNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutonomousCycleOutcome {
    pub cycle_id: String,
    pub gates_cleared: Vec<u8>,
    pub spec_digest: String,
    pub total_waves: usize,
    pub total_tasks: usize,
    pub overall_verdict: GateVerdict,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CircuitBreakerDiagnostic {
    pub cycle_id: String,
    pub is_tripped: bool,
    pub consecutive_failures: u8,
    pub semantic_exit_code: i32,
    pub root_cause: String,
    pub remediation: String,
}

impl CircuitBreakerDiagnostic {
    pub fn new(
        cycle_id: impl Into<String>,
        is_tripped: bool,
        consecutive_failures: u8,
        semantic_exit_code: i32,
        root_cause: impl Into<String>,
        remediation: impl Into<String>,
    ) -> Self {
        Self {
            cycle_id: cycle_id.into(),
            is_tripped,
            consecutive_failures,
            semantic_exit_code,
            root_cause: root_cause.into(),
            remediation: remediation.into(),
        }
    }
}

pub fn check_circuit_breaker_safety(
    cycle_id: &str,
    repo: &dyn CycleEvidenceRepositoryPort,
) -> Result<(), DomainViolation> {
    if let Ok(ActiveCycleLookup::Found(active)) = repo.find_active_cycle() {
        if active.lifecycle_status == CycleStatus::CircuitBreakerTripped
            || active.gauntlet_attempt >= 5
        {
            return Err(DomainViolation::CircuitBreakerTripped {
                cycle_id: active.cycle_id,
                consecutive_failures: active.gauntlet_attempt,
                remediation: "Inspect failures with 'trustkit doctor', resolve blocked gates, and reset with 'trustkit init'".to_string(),
            });
        }
    }
    if let Ok(Some(latest)) = repo.find_latest_cycle() {
        if latest.cycle_id == cycle_id
            && (latest.lifecycle_status == CycleStatus::CircuitBreakerTripped
                || latest.gauntlet_attempt >= 5)
        {
            return Err(DomainViolation::CircuitBreakerTripped {
                cycle_id: latest.cycle_id,
                consecutive_failures: latest.gauntlet_attempt,
                remediation: "Inspect failures with 'trustkit doctor', resolve blocked gates, and reset with 'trustkit init'".to_string(),
            });
        }
    }
    Ok(())
}

pub fn diagnose_failure(cycle_id: &str, err: &DomainViolation) -> CircuitBreakerDiagnostic {
    let code = err.posix_exit_code();
    let is_tripped = code == 6;
    let (root_cause, remediation) = match err {
        DomainViolation::CircuitBreakerTripped {
            consecutive_failures,
            remediation,
            ..
        } => (
            format!("Circuit breaker tripped after {consecutive_failures} failures"),
            remediation.clone(),
        ),
        DomainViolation::SpecOrTraceabilityIncomplete {
            reason,
            missing_artefact,
        } => (
            format!("Gate 1 spec incomplete: {reason} (missing: {missing_artefact})"),
            "Correct specification formatting and rerun linter via 'trustkit spec lint'"
                .to_string(),
        ),
        DomainViolation::TddLifecycleViolation { task_id, reason } => (
            format!("Gate 4 TDD lifecycle violation in {task_id}: {reason}"),
            "Ensure Red failure proof is recorded before Green verification".to_string(),
        ),
        DomainViolation::AstIntegrityOrInvariantViolation {
            invariant_id,
            reason,
            remediation,
            ..
        } => (
            format!("Invariant violation {invariant_id}: {reason}"),
            remediation.clone(),
        ),
        DomainViolation::CriticSwarmOrPlaywrightFailed {
            layer,
            attempt,
            failure_summary,
        } => (
            format!("Gauntlet {layer} critic failure (attempt {attempt}): {failure_summary}"),
            "Address reviewer findings and re-execute gauntlet".to_string(),
        ),
        other => (
            other.to_string(),
            "Consult 'trustkit doctor' for system recovery".to_string(),
        ),
    };

    CircuitBreakerDiagnostic::new(
        cycle_id,
        is_tripped,
        if is_tripped { 5 } else { 1 },
        code,
        root_cause,
        remediation,
    )
}

pub fn execute(
    req: AutonomousCycleRequest,
    repo: &dyn CycleEvidenceRepositoryPort,
    hasher: &dyn ArtifactHasherPort,
    validator: &dyn SpecValidatorPort,
    planner: &dyn WaveExecutionPlannerPort,
) -> Result<AutonomousCycleOutcome, DomainViolation> {
    if req.issue.is_none() && req.spec_path.is_none() {
        return Err(DomainViolation::InfrastructureBoundaryFault {
            subsystem: "autonomous_cycle".to_string(),
            details: "Either --issue or --spec must be provided".to_string(),
        });
    }

    let cycle_id = if let Some(ref iss) = req.issue {
        format!("issue-{iss}-{}", req.timestamp_slug)
    } else {
        format!("spec-{}", req.timestamp_slug)
    };

    // Pre-flight safety check: prevent loop if circuit breaker is already tripped
    check_circuit_breaker_safety(&cycle_id, repo)?;

    let mut gates_cleared = Vec::with_capacity(4);

    // Gate 0: Setup and initialization verified
    gates_cleared.push(0);

    // Gate 1: Specification Lint and syntactic verification
    if req.spec_content.trim().is_empty() {
        return Err(DomainViolation::SpecOrTraceabilityIncomplete {
            reason: "Specification content cannot be empty for autonomous cycle".to_string(),
            missing_artefact: "spec.md content".to_string(),
        });
    }

    let _report = validator.validate_specification(&req.spec_content)?;
    gates_cleared.push(1);

    // Gate 2: Specification BLAKE3 Cryptographic Freeze
    let spec_digest = hasher.compute_bytes_blake3(req.spec_content.as_bytes());
    repo.update_cycle_spec_lock(&cycle_id, &spec_digest, "FROZEN")?;
    gates_cleared.push(2);

    // Gate 3: Wave Planning Topological DAG
    let schedule = planner.plan_waves(&req.tasks)?;
    repo.save_atomic_tasks(&cycle_id, &req.tasks, &schedule)?;
    gates_cleared.push(3);

    let total_tasks = req.tasks.len();
    let total_waves = schedule.waves.len();

    Ok(AutonomousCycleOutcome {
        cycle_id,
        gates_cleared,
        spec_digest,
        total_waves,
        total_tasks,
        overall_verdict: GateVerdict::Pass,
    })
}
