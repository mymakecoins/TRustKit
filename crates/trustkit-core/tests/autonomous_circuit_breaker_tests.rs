// Tests for Autonomous Circuit Breaker and Diagnostic Treatment (TASK-005).
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use std::sync::Mutex;
use trustkit_core::entities::{
    ArtifactSeal, CycleStatus, ExecutionEvidence, GateStage, MethodologyProfile, SddCycle,
    SpecLockState, TaskNode, WaveSchedule,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::{
    ArtifactHasherPort, CycleEvidenceRepositoryPort, SpecValidationReport, SpecValidatorPort,
    WaveExecutionPlannerPort,
};
use trustkit_core::special_cases::{ActiveCycleLookup, IssueLinkage};
use trustkit_core::use_cases::run_autonomous_cycle::{
    self, AutonomousCycleRequest, check_circuit_breaker_safety, diagnose_failure,
};

struct MockRepo {
    active_cycle: Option<SddCycle>,
    locks: Mutex<Vec<(String, String, String)>>,
}

impl MockRepo {
    fn new(active_cycle: Option<SddCycle>) -> Self {
        Self {
            active_cycle,
            locks: Mutex::new(Vec::new()),
        }
    }
}

impl CycleEvidenceRepositoryPort for MockRepo {
    fn save_cycle(&self, _cycle: &SddCycle) -> Result<(), DomainViolation> {
        Ok(())
    }
    fn find_active_cycle(&self) -> Result<ActiveCycleLookup, DomainViolation> {
        match self.active_cycle.as_ref() {
            Some(c) => Ok(ActiveCycleLookup::Found(Box::new(c.clone()))),
            None => Ok(ActiveCycleLookup::NoActiveCycle),
        }
    }
    fn save_artifact_seal(&self, _seal: &ArtifactSeal) -> Result<u64, DomainViolation> {
        Ok(1)
    }
    fn list_artifact_seals(&self, _cycle_id: &str) -> Result<Vec<ArtifactSeal>, DomainViolation> {
        Ok(vec![])
    }
    fn append_evidence(&self, _evidence: &ExecutionEvidence) -> Result<u64, DomainViolation> {
        Ok(1)
    }
    fn list_evidences_for_cycle(
        &self,
        _cycle_id: &str,
    ) -> Result<Vec<ExecutionEvidence>, DomainViolation> {
        Ok(vec![])
    }
    fn update_cycle_spec_lock(
        &self,
        cycle_id: &str,
        blake3_digest: &str,
        locked_state: &str,
    ) -> Result<(), DomainViolation> {
        self.locks.lock().unwrap().push((
            cycle_id.to_string(),
            blake3_digest.to_string(),
            locked_state.to_string(),
        ));
        Ok(())
    }
    fn save_atomic_tasks(
        &self,
        _cycle_id: &str,
        _tasks: &[TaskNode],
        _wave_schedule: &WaveSchedule,
    ) -> Result<(), DomainViolation> {
        Ok(())
    }
}

struct MockHasher;
impl ArtifactHasherPort for MockHasher {
    fn compute_file_blake3(&self, _path: &str) -> Result<String, DomainViolation> {
        Ok("mock_digest".to_string())
    }
    fn compute_bytes_blake3(&self, _bytes: &[u8]) -> String {
        "mock_digest".to_string()
    }
}

struct MockValidator {
    fail_spec: bool,
}
impl SpecValidatorPort for MockValidator {
    fn validate_specification(
        &self,
        _spec_content: &str,
    ) -> Result<SpecValidationReport, DomainViolation> {
        if self.fail_spec {
            Err(DomainViolation::SpecOrTraceabilityIncomplete {
                reason: "Mermaid syntax error detected in section 2".to_string(),
                missing_artefact: "valid Mermaid flowchart".to_string(),
            })
        } else {
            Ok(SpecValidationReport {
                mermaid_valid: true,
                gherkin_valid: true,
                orphan_rules: vec![],
                total_rules: 5,
                total_requirements: 5,
            })
        }
    }
}

struct MockPlanner;
impl WaveExecutionPlannerPort for MockPlanner {
    fn plan_waves(&self, _tasks: &[TaskNode]) -> Result<WaveSchedule, DomainViolation> {
        Ok(WaveSchedule {
            cycle_id: "test-cycle".to_string(),
            total_tasks: 0,
            waves: vec![],
        })
    }
}

fn create_tripped_cycle(cycle_id: &str, attempts: u8) -> SddCycle {
    SddCycle {
        cycle_id: cycle_id.to_string(),
        issue_linkage: IssueLinkage::UnlinkedLocalCycle {
            directory_slug: "test".to_string(),
        },
        methodology_profile: MethodologyProfile::StandardGauntletV2,
        current_gate: GateStage::Gate5GauntletCleared,
        lifecycle_status: CycleStatus::CircuitBreakerTripped,
        gauntlet_attempt: attempts,
        spec_dir: "docs/specs/test".to_string(),
        spec_file_path: "docs/specs/test/spec.md".to_string(),
        spec_seal_state: SpecLockState::DraftUnverified,
        work_branch: "feat/test".to_string(),
        base_commit_sha: "0000000000000000000000000000000000000000".to_string(),
        pre_existing_dirty_files: vec![],
        created_at: "2026-11-05T12:00:00Z".to_string(),
        updated_at: "2026-11-05T12:00:00Z".to_string(),
    }
}

#[test]
fn test_circuit_breaker_trips_when_cycle_already_in_circuit_breaker_tripped() {
    let cycle = create_tripped_cycle("issue-10-test", 5);
    let repo = MockRepo::new(Some(cycle));
    let hasher = MockHasher;
    let validator = MockValidator { fail_spec: false };
    let planner = MockPlanner;

    let req = AutonomousCycleRequest {
        issue: Some("10".to_string()),
        spec_path: None,
        target_branch: "develop".to_string(),
        runtime_mode: "auto".to_string(),
        agent_command: "".to_string(),
        now_iso: "2026-11-05T12:00:00Z".to_string(),
        timestamp_slug: "test".to_string(),
        spec_content: "# Spec Valid".to_string(),
        tasks: vec![],
    };

    let result = run_autonomous_cycle::execute(req, &repo, &hasher, &validator, &planner);
    assert!(result.is_err());
    let err = result.unwrap_err();

    assert_eq!(err.posix_exit_code(), 6);
    match err {
        DomainViolation::CircuitBreakerTripped {
            cycle_id,
            consecutive_failures,
            remediation,
        } => {
            assert_eq!(cycle_id, "issue-10-test");
            assert_eq!(consecutive_failures, 5);
            assert!(remediation.contains("trustkit doctor"));
        }
        other => panic!("Expected CircuitBreakerTripped, got: {other:?}"),
    }
}

#[test]
fn test_circuit_breaker_safety_check_direct() {
    let cycle = create_tripped_cycle("issue-10-test", 5);
    let repo = MockRepo::new(Some(cycle));
    let result = check_circuit_breaker_safety("issue-10-test", &repo);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().posix_exit_code(), 6);
}

#[test]
fn test_gate_1_spec_failure_returns_semantic_exit_code_2() {
    let repo = MockRepo::new(None);
    let hasher = MockHasher;
    let validator = MockValidator { fail_spec: true };
    let planner = MockPlanner;

    let req = AutonomousCycleRequest {
        issue: Some("10".to_string()),
        spec_path: None,
        target_branch: "develop".to_string(),
        runtime_mode: "auto".to_string(),
        agent_command: "".to_string(),
        now_iso: "2026-11-05T12:00:00Z".to_string(),
        timestamp_slug: "test".to_string(),
        spec_content: "# Spec with invalid Mermaid".to_string(),
        tasks: vec![],
    };

    let result = run_autonomous_cycle::execute(req, &repo, &hasher, &validator, &planner);
    assert!(result.is_err());
    let err = result.unwrap_err();

    assert_eq!(err.posix_exit_code(), 2);
    let diag = diagnose_failure("issue-10-test", &err);
    assert_eq!(diag.semantic_exit_code, 2);
    assert!(!diag.is_tripped);
    assert!(diag.root_cause.contains("Gate 1 spec incomplete"));
    assert!(diag.remediation.contains("trustkit spec lint"));
}

#[test]
fn test_structured_diagnostics_produced_for_all_semantic_categories() {
    // Exit code 6: Circuit breaker
    let cb_err = DomainViolation::CircuitBreakerTripped {
        cycle_id: "cycle-cb".to_string(),
        consecutive_failures: 5,
        remediation: "Run trustkit doctor".to_string(),
    };
    let cb_diag = diagnose_failure("cycle-cb", &cb_err);
    assert_eq!(cb_diag.semantic_exit_code, 6);
    assert!(cb_diag.is_tripped);
    assert_eq!(cb_diag.consecutive_failures, 5);

    // Exit code 3: TDD violation
    let tdd_err = DomainViolation::TddLifecycleViolation {
        task_id: "TASK-003".to_string(),
        reason: "MissingRedPhaseProof".to_string(),
    };
    let tdd_diag = diagnose_failure("cycle-tdd", &tdd_err);
    assert_eq!(tdd_diag.semantic_exit_code, 3);
    assert!(!tdd_diag.is_tripped);
    assert!(
        tdd_diag
            .root_cause
            .contains("Gate 4 TDD lifecycle violation")
    );

    // Exit code 4: Invariant violation
    let inv_err = DomainViolation::AstIntegrityOrInvariantViolation {
        invariant_id: "RN-013".to_string(),
        intercepted_command: "git commit".to_string(),
        reason: "Forbidden emoji found".to_string(),
        remediation: "Remove emoji".to_string(),
    };
    let inv_diag = diagnose_failure("cycle-inv", &inv_err);
    assert_eq!(inv_diag.semantic_exit_code, 4);
    assert!(inv_diag.remediation.contains("Remove emoji"));
}
