// Integration tests for TASK-003: EmptyTaskBacklogFault Circuit Breaker at Gate 3.
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use std::sync::Mutex;
use trustkit_core::entities::{
    ArtifactSeal, AtomicTask, CycleStatus, ExecutionEvidence, ExecutionWave, GateStage,
    MethodologyProfile, SddCycle, TaskNode, WaveSchedule,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::{CycleEvidenceRepositoryPort, WaveExecutionPlannerPort};
use trustkit_core::special_cases::{
    ActiveCycleLookup, IssueLinkage, SpecLockState, TddLifecycleState, WorktreeAllocation,
};
use trustkit_core::use_cases::plan_execution_waves::{
    self, PlanExecutionWavesRequest, enforce_non_empty_backlog, guard_gate_transition_with_backlog,
    verify_cycle_has_atomic_tasks,
};

struct FakeRepo {
    active_cycle: Mutex<Option<SddCycle>>,
    saved_tasks: Mutex<Vec<TaskNode>>,
    persisted_atomic_tasks: Mutex<Vec<AtomicTask>>,
}

impl FakeRepo {
    fn new(cycle: Option<SddCycle>, persisted_atomic_tasks: Vec<AtomicTask>) -> Self {
        Self {
            active_cycle: Mutex::new(cycle),
            saved_tasks: Mutex::new(Vec::new()),
            persisted_atomic_tasks: Mutex::new(persisted_atomic_tasks),
        }
    }
}

impl CycleEvidenceRepositoryPort for FakeRepo {
    fn save_cycle(&self, c: &SddCycle) -> Result<(), DomainViolation> {
        *self.active_cycle.lock().unwrap() = Some(c.clone());
        Ok(())
    }

    fn find_active_cycle(&self) -> Result<ActiveCycleLookup, DomainViolation> {
        match &*self.active_cycle.lock().unwrap() {
            Some(c) => Ok(ActiveCycleLookup::Found(Box::new(c.clone()))),
            None => Ok(ActiveCycleLookup::NoActiveCycle),
        }
    }

    fn save_artifact_seal(&self, _: &ArtifactSeal) -> Result<u64, DomainViolation> {
        Ok(1)
    }

    fn list_artifact_seals(&self, _: &str) -> Result<Vec<ArtifactSeal>, DomainViolation> {
        Ok(vec![])
    }

    fn append_evidence(&self, _: &ExecutionEvidence) -> Result<u64, DomainViolation> {
        Ok(1)
    }

    fn list_evidences_for_cycle(&self, _: &str) -> Result<Vec<ExecutionEvidence>, DomainViolation> {
        Ok(vec![])
    }

    fn update_cycle_spec_lock(&self, _: &str, _: &str, _: &str) -> Result<(), DomainViolation> {
        Ok(())
    }

    fn save_atomic_tasks(
        &self,
        _: &str,
        tasks: &[TaskNode],
        _: &WaveSchedule,
    ) -> Result<(), DomainViolation> {
        self.saved_tasks.lock().unwrap().extend_from_slice(tasks);
        Ok(())
    }

    fn list_atomic_tasks(&self, _: &str) -> Result<Vec<AtomicTask>, DomainViolation> {
        Ok(self.persisted_atomic_tasks.lock().unwrap().clone())
    }
}

struct FakePlanner;

impl WaveExecutionPlannerPort for FakePlanner {
    fn plan_waves(&self, tasks: &[TaskNode]) -> Result<WaveSchedule, DomainViolation> {
        Ok(WaveSchedule {
            cycle_id: "CYC-TEST".to_string(),
            total_tasks: tasks.len(),
            waves: if tasks.is_empty() {
                vec![]
            } else {
                vec![ExecutionWave {
                    wave_index: 1,
                    tasks: tasks.to_vec(),
                }]
            },
        })
    }
}

fn frozen_cycle(cycle_id: &str) -> SddCycle {
    SddCycle {
        cycle_id: cycle_id.to_string(),
        issue_linkage: IssueLinkage::UnlinkedLocalCycle {
            directory_slug: "test-empty-cb".to_string(),
        },
        methodology_profile: MethodologyProfile::StandardGauntletV2,
        current_gate: GateStage::Gate2SpecSealed,
        lifecycle_status: CycleStatus::Active,
        gauntlet_attempt: 0,
        spec_dir: "specs/010".to_string(),
        spec_file_path: "specs/010/spec.md".to_string(),
        spec_seal_state: SpecLockState::Frozen {
            spec_path: "specs/010/spec.md".to_string(),
            blake3_digest: "db4955e682843bd0eb13e92b22272a7952e2c637b38e408f0ef09af015eb33ba"
                .to_string(),
        },
        work_branch: "feat/empty-cb".to_string(),
        base_commit_sha: "0000000000000000000000000000000000000000".to_string(),
        pre_existing_dirty_files: vec![],
        created_at: "2026-11-11T10:00:00Z".to_string(),
        updated_at: "2026-11-11T10:00:00Z".to_string(),
    }
}

#[test]
fn test_plan_execution_waves_rejects_empty_backlog_with_circuit_breaker_fault() {
    let repo = FakeRepo::new(Some(frozen_cycle("CYC-EMPTY-001")), vec![]);
    let planner = FakePlanner;
    let req = PlanExecutionWavesRequest { tasks: vec![] };

    let result = plan_execution_waves::execute(req, &repo, &planner);
    assert!(
        result.is_err(),
        "Planning waves with zero atomic tasks must fail"
    );

    let err = result.unwrap_err();
    assert_eq!(
        err.posix_exit_code(),
        1,
        "EmptyTaskBacklogFault must map to non-zero exit code 1"
    );
    match err {
        DomainViolation::EmptyTaskBacklogFault { cycle_id, reason } => {
            assert_eq!(cycle_id, "CYC-EMPTY-001");
            assert!(reason.contains("zero") || reason.contains("empty") || reason.contains("0"));
        }
        other => panic!("Expected DomainViolation::EmptyTaskBacklogFault, got: {other:?}"),
    }
    assert!(
        repo.saved_tasks.lock().unwrap().is_empty(),
        "Repository must not persist empty task schedule"
    );
}

#[test]
fn test_enforce_non_empty_backlog_blocks_zero_tasks_and_allows_positive_tasks() {
    let err = enforce_non_empty_backlog("CYC-003", 0).unwrap_err();
    assert_eq!(err.posix_exit_code(), 1);
    assert!(matches!(
        err,
        DomainViolation::EmptyTaskBacklogFault { ref cycle_id, .. } if cycle_id == "CYC-003"
    ));

    let ok = enforce_non_empty_backlog("CYC-003", 2);
    assert!(ok.is_ok());
}

#[test]
fn test_guard_gate_transition_blocks_gates_4_5_and_6_when_backlog_empty() {
    for blocked_gate in [
        GateStage::Gate4TasksImplemented,
        GateStage::Gate5GauntletCleared,
        GateStage::Gate6Certified,
    ] {
        let res = guard_gate_transition_with_backlog("CYC-GATE-GUARD", blocked_gate, 0);
        assert!(
            res.is_err(),
            "Transition to {blocked_gate:?} must be blocked when total_tasks == 0"
        );
        let err = res.unwrap_err();
        assert_eq!(err.posix_exit_code(), 1);
        assert!(matches!(err, DomainViolation::EmptyTaskBacklogFault { .. }));
    }

    assert!(
        guard_gate_transition_with_backlog("CYC-GATE-GUARD", GateStage::Gate1SpecDrafting, 0)
            .is_ok()
    );
    assert!(
        guard_gate_transition_with_backlog("CYC-GATE-GUARD", GateStage::Gate2SpecSealed, 0).is_ok()
    );
    assert!(
        guard_gate_transition_with_backlog("CYC-GATE-GUARD", GateStage::Gate4TasksImplemented, 3)
            .is_ok()
    );
}

#[test]
fn test_verify_cycle_has_atomic_tasks_inspects_repository() {
    let empty_repo = FakeRepo::new(Some(frozen_cycle("CYC-REPO-EMPTY")), vec![]);
    let err = verify_cycle_has_atomic_tasks("CYC-REPO-EMPTY", &empty_repo).unwrap_err();
    assert!(matches!(err, DomainViolation::EmptyTaskBacklogFault { .. }));

    let populated_task = AtomicTask {
        cycle_id: "CYC-REPO-OK".to_string(),
        task_id: "TASK-001".to_string(),
        title: "Sample Task".to_string(),
        architectural_layer: "CORE".to_string(),
        wave_index: 1,
        depends_on: vec![],
        expected_files: vec!["src/lib.rs".to_string()],
        target_test_file: "tests/test.rs".to_string(),
        tdd_lifecycle_state: TddLifecycleState::AwaitingRedPhase,
        red_evidence_id: 0,
        green_evidence_id: 0,
        worktree_allocation: WorktreeAllocation::NotProvisioned,
        commit_sha: String::new(),
    };
    let ok_repo = FakeRepo::new(Some(frozen_cycle("CYC-REPO-OK")), vec![populated_task]);
    let count = verify_cycle_has_atomic_tasks("CYC-REPO-OK", &ok_repo).expect("Should pass");
    assert_eq!(count, 1);
}
