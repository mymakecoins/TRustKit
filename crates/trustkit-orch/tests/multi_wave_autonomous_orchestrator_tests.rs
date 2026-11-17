// Tests for Autonomous Multi-Wave Execution Orchestrator (TASK-003).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use std::sync::Mutex;
use trustkit_core::entities::{
    ArtifactSeal, AtomicTask, ExecutionEvidence, GateStage, GateVerdict, SddCycle, TaskNode,
    WaveSchedule,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::{
    AsyncExecutionSupervisorPort, CycleEvidenceRepositoryPort, SupervisedExecutionOutcome,
    SupervisedExecutionRequest, WorktreeIsolationPort, WorktreeStatus,
};
use trustkit_core::special_cases::{ActiveCycleLookup, TddLifecycleState, WorktreeAllocation};
use trustkit_orch::autonomous_wave_orchestrator::{
    AutonomousWaveOrchestrator, DefaultSemanticCodeReviewer, MultiWaveDependencies,
    MultiWaveExecutionRequest,
};

struct MockEvidenceRepo {
    tasks: Mutex<Vec<AtomicTask>>,
    gate_advanced: Mutex<Option<GateStage>>,
}

impl MockEvidenceRepo {
    fn new(tasks: Vec<AtomicTask>) -> Self {
        Self {
            tasks: Mutex::new(tasks),
            gate_advanced: Mutex::new(None),
        }
    }
}

impl CycleEvidenceRepositoryPort for MockEvidenceRepo {
    fn save_cycle(&self, _cycle: &SddCycle) -> Result<(), DomainViolation> {
        Ok(())
    }
    fn find_active_cycle(&self) -> Result<ActiveCycleLookup, DomainViolation> {
        Ok(ActiveCycleLookup::NoActiveCycle)
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
        _cycle_id: &str,
        _blake3_digest: &str,
        _locked_state: &str,
    ) -> Result<(), DomainViolation> {
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
    fn list_atomic_tasks(&self, _cycle_id: &str) -> Result<Vec<AtomicTask>, DomainViolation> {
        Ok(self.tasks.lock().unwrap().clone())
    }
    fn find_atomic_task(
        &self,
        _cycle_id: &str,
        task_id: &str,
    ) -> Result<Option<AtomicTask>, DomainViolation> {
        let guard = self.tasks.lock().unwrap();
        Ok(guard.iter().find(|t| t.task_id == task_id).cloned())
    }
    fn update_task_tdd_phase(
        &self,
        _cycle_id: &str,
        task_id: &str,
        state: &TddLifecycleState,
    ) -> Result<(), DomainViolation> {
        let mut guard = self.tasks.lock().unwrap();
        if let Some(t) = guard.iter_mut().find(|t| t.task_id == task_id) {
            t.tdd_lifecycle_state = state.clone();
        }
        Ok(())
    }
    fn advance_cycle_gate(
        &self,
        _cycle_id: &str,
        next_gate: GateStage,
    ) -> Result<(), DomainViolation> {
        *self.gate_advanced.lock().unwrap() = Some(next_gate);
        Ok(())
    }
}

struct MockWorktreeProvider {
    provisioned: Mutex<Vec<String>>,
    merged: Mutex<Vec<String>>,
}

impl MockWorktreeProvider {
    fn new() -> Self {
        Self {
            provisioned: Mutex::new(Vec::new()),
            merged: Mutex::new(Vec::new()),
        }
    }
}

impl WorktreeIsolationPort for MockWorktreeProvider {
    fn provision_worktree(
        &self,
        _cycle_id: &str,
        task_id: &str,
        _base_branch: &str,
    ) -> Result<WorktreeAllocation, DomainViolation> {
        self.provisioned.lock().unwrap().push(task_id.to_string());
        Ok(WorktreeAllocation::Provisioned {
            worktree_path: format!(".trustkit/worktrees/{task_id}"),
            branch_name: format!("worktree/{task_id}"),
            base_branch: "main".to_string(),
        })
    }

    fn check_worktree_status(
        &self,
        _worktree_path: &str,
    ) -> Result<WorktreeStatus, DomainViolation> {
        Ok(WorktreeStatus {
            worktree_path: ".trustkit/worktrees/mock".to_string(),
            branch_name: "worktree/mock".to_string(),
            is_clean: true,
            head_commit_sha: "mock_sha_00000000000000000000000000000000".to_string(),
        })
    }

    fn merge_and_prune_worktree(
        &self,
        task_id: &str,
        _worktree_path: &str,
        _target_branch: &str,
    ) -> Result<String, DomainViolation> {
        self.merged.lock().unwrap().push(task_id.to_string());
        Ok("mock_merged_sha_1234567890abcdef12345678".to_string())
    }
}

struct MockSupervisor {
    simulate_red_pass: bool,
}

impl MockSupervisor {
    fn new(simulate_red_pass: bool) -> Self {
        Self { simulate_red_pass }
    }
}

impl AsyncExecutionSupervisorPort for MockSupervisor {
    fn execute_supervised(
        &self,
        request: &SupervisedExecutionRequest,
    ) -> Result<SupervisedExecutionOutcome, DomainViolation> {
        let exit_code = if self.simulate_red_pass {
            0
        } else if request.args.contains(&"test".to_string()) && !self.simulate_red_pass {
            // When simulating TDD:
            // First run is RED (should fail with exit code 1)
            // Second run is GREEN (should pass with exit code 0)
            1
        } else {
            0
        };

        Ok(SupervisedExecutionOutcome {
            exit_code,
            duration_ms: 5,
            stdout_tail: "test output".to_string(),
            stderr_tail: "".to_string(),
            output_blake3_digest: "mock_digest".to_string(),
        })
    }
}

struct ConfigurableSupervisor {
    calls: Mutex<usize>,
}

impl ConfigurableSupervisor {
    fn new() -> Self {
        Self {
            calls: Mutex::new(0),
        }
    }
}

impl AsyncExecutionSupervisorPort for ConfigurableSupervisor {
    fn execute_supervised(
        &self,
        _request: &SupervisedExecutionRequest,
    ) -> Result<SupervisedExecutionOutcome, DomainViolation> {
        let mut count = self.calls.lock().unwrap();
        *count += 1;
        // Even call = Green (exit 0), Odd call = Red (exit 1)
        let exit_code = if *count % 2 == 1 { 1 } else { 0 };

        Ok(SupervisedExecutionOutcome {
            exit_code,
            duration_ms: 5,
            stdout_tail: "test output".to_string(),
            stderr_tail: "".to_string(),
            output_blake3_digest: "mock_digest".to_string(),
        })
    }
}

fn create_task(task_id: &str, wave_index: u32, test_file: &str) -> AtomicTask {
    AtomicTask {
        cycle_id: "test-cycle".to_string(),
        task_id: task_id.to_string(),
        title: format!("Task {task_id}"),
        architectural_layer: "CORE".to_string(),
        wave_index,
        depends_on: vec![],
        expected_files: vec!["file.rs".to_string()],
        target_test_file: test_file.to_string(),
        tdd_lifecycle_state: TddLifecycleState::AwaitingRedPhase,
        red_evidence_id: 0,
        green_evidence_id: 0,
        worktree_allocation: WorktreeAllocation::NotProvisioned,
        commit_sha: "UNCOMMITTED".to_string(),
    }
}

#[test]
fn test_task_003_multi_wave_orchestrator_executes_sequential_waves() {
    let t1 = create_task("TASK-001", 1, "tests/task_001_tests.rs");
    let t2 = create_task("TASK-002", 1, "tests/task_002_tests.rs");
    let t3 = create_task("TASK-003", 2, "tests/task_003_tests.rs");

    let repo = MockEvidenceRepo::new(vec![t1, t2, t3]);
    let worktree_port = MockWorktreeProvider::new();
    let supervisor = ConfigurableSupervisor::new();
    let reviewer = DefaultSemanticCodeReviewer::new();

    let orchestrator = AutonomousWaveOrchestrator::new();
    let req = MultiWaveExecutionRequest {
        cycle_id: "test-cycle",
        wave_indices: &[1, 2],
        target_branch: "develop",
        now_iso: "2026-11-05T12:00:00Z",
    };
    let deps = MultiWaveDependencies {
        repo: &repo,
        worktree_port: &worktree_port,
        supervisor: &supervisor,
        reviewer: &reviewer,
        builder_dispatcher: None,
    };
    let outcome = orchestrator
        .execute_wave_loop(&req, &deps)
        .expect("multi wave execution must succeed");

    assert_eq!(outcome.overall_verdict, GateVerdict::Pass);
    assert_eq!(outcome.waves_executed, vec![1, 2]);
    assert_eq!(outcome.tasks_executed.len(), 3);
    assert_eq!(
        *repo.gate_advanced.lock().unwrap(),
        Some(GateStage::Gate4TasksImplemented)
    );
    assert_eq!(worktree_port.merged.lock().unwrap().len(), 3);
}

#[test]
fn test_task_003_orchestrator_rejects_empty_waves() {
    let repo = MockEvidenceRepo::new(vec![]);
    let worktree_port = MockWorktreeProvider::new();
    let supervisor = ConfigurableSupervisor::new();
    let reviewer = DefaultSemanticCodeReviewer::new();

    let orchestrator = AutonomousWaveOrchestrator::new();
    let req = MultiWaveExecutionRequest {
        cycle_id: "test-cycle",
        wave_indices: &[],
        target_branch: "develop",
        now_iso: "2026-11-05T12:00:00Z",
    };
    let deps = MultiWaveDependencies {
        repo: &repo,
        worktree_port: &worktree_port,
        supervisor: &supervisor,
        reviewer: &reviewer,
        builder_dispatcher: None,
    };
    let result = orchestrator.execute_wave_loop(&req, &deps);

    assert!(result.is_err());
    match result {
        Err(DomainViolation::InfrastructureBoundaryFault { details, .. }) => {
            assert!(details.contains("No wave indices"));
        }
        _ => panic!("Expected InfrastructureBoundaryFault for empty wave indices"),
    }
}

#[test]
fn test_task_003_orchestrator_fails_when_wave_tasks_missing() {
    let t1 = create_task("TASK-001", 1, "tests/task_001_tests.rs");
    let repo = MockEvidenceRepo::new(vec![t1]);
    let worktree_port = MockWorktreeProvider::new();
    let supervisor = ConfigurableSupervisor::new();
    let reviewer = DefaultSemanticCodeReviewer::new();

    let orchestrator = AutonomousWaveOrchestrator::new();
    let req = MultiWaveExecutionRequest {
        cycle_id: "test-cycle",
        wave_indices: &[99],
        target_branch: "develop",
        now_iso: "2026-11-05T12:00:00Z",
    };
    let deps = MultiWaveDependencies {
        repo: &repo,
        worktree_port: &worktree_port,
        supervisor: &supervisor,
        reviewer: &reviewer,
        builder_dispatcher: None,
    };
    let result = orchestrator.execute_wave_loop(&req, &deps);

    assert!(result.is_err());
    match result {
        Err(DomainViolation::InfrastructureBoundaryFault { details, .. }) => {
            assert!(details.contains("No tasks found for wave index 99"));
        }
        _ => panic!("Expected InfrastructureBoundaryFault for missing wave tasks"),
    }
}

#[test]
fn test_task_003_orchestrator_enforces_red_failure() {
    let t1 = create_task("TASK-001", 1, "tests/task_001_tests.rs");
    let repo = MockEvidenceRepo::new(vec![t1]);
    let worktree_port = MockWorktreeProvider::new();
    // Simulate Red test passing (exit code 0), which violates TDD red requirement
    let supervisor = MockSupervisor::new(true);
    let reviewer = DefaultSemanticCodeReviewer::new();

    let orchestrator = AutonomousWaveOrchestrator::new();
    let req = MultiWaveExecutionRequest {
        cycle_id: "test-cycle",
        wave_indices: &[1],
        target_branch: "develop",
        now_iso: "2026-11-05T12:00:00Z",
    };
    let deps = MultiWaveDependencies {
        repo: &repo,
        worktree_port: &worktree_port,
        supervisor: &supervisor,
        reviewer: &reviewer,
        builder_dispatcher: None,
    };
    let result = orchestrator.execute_wave_loop(&req, &deps);

    assert!(result.is_err());
    match result {
        Err(DomainViolation::TddLifecycleViolation { .. }) => {}
        other => panic!("Expected TddLifecycleViolation, got: {other:?}"),
    }
}
