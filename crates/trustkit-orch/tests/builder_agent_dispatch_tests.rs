// Integration tests for TASK-007: Builder Agent Dispatch (RF-007 / RN-007).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tempfile::tempdir;
use trustkit_core::entities::{
    ArtifactSeal, AtomicTask, ExecutionEvidence, GateStage, GateVerdict, SddCycle, TaskNode,
    WaveSchedule,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::{
    AsyncExecutionSupervisorPort, BuilderAgentDispatcherPort, CycleEvidenceRepositoryPort,
    SupervisedExecutionOutcome, SupervisedExecutionRequest, WorktreeIsolationPort, WorktreeStatus,
};
use trustkit_core::special_cases::{ActiveCycleLookup, TddLifecycleState, WorktreeAllocation};
use trustkit_orch::autonomous_wave_orchestrator::{
    AutonomousWaveOrchestrator, DefaultSemanticCodeReviewer, MultiWaveDependencies,
    MultiWaveExecutionRequest,
};
use trustkit_orch::builder_agent_dispatcher::BuilderAgentDispatcher;

struct SpyBuilderDispatcher {
    called: Arc<AtomicBool>,
}

impl SpyBuilderDispatcher {
    fn new() -> (Self, Arc<AtomicBool>) {
        let flag = Arc::new(AtomicBool::new(false));
        (
            Self {
                called: Arc::clone(&flag),
            },
            flag,
        )
    }
}

impl BuilderAgentDispatcherPort for SpyBuilderDispatcher {
    fn dispatch_builder_in_worktree(
        &self,
        cycle_id: &str,
        task_id: &str,
        worktree_path: &str,
        _target_test_file: &str,
        _expected_files: &[String],
    ) -> Result<SupervisedExecutionOutcome, DomainViolation> {
        assert_eq!(cycle_id, "test-cycle");
        assert_eq!(task_id, "TASK-007");
        assert!(!worktree_path.is_empty());
        self.called.store(true, Ordering::SeqCst);
        Ok(SupervisedExecutionOutcome {
            exit_code: 0,
            duration_ms: 5,
            stdout_tail: "Builder dispatched successfully".to_string(),
            stderr_tail: "".to_string(),
            output_blake3_digest: "digest".to_string(),
        })
    }
}

struct MockEvidenceRepo {
    tasks: std::sync::Mutex<Vec<AtomicTask>>,
}

impl MockEvidenceRepo {
    fn new(tasks: Vec<AtomicTask>) -> Self {
        Self {
            tasks: std::sync::Mutex::new(tasks),
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
        _target_gate: GateStage,
    ) -> Result<(), DomainViolation> {
        Ok(())
    }
}

struct MockWorktreeProvider;

impl WorktreeIsolationPort for MockWorktreeProvider {
    fn provision_worktree(
        &self,
        _cycle_id: &str,
        task_id: &str,
        _target_branch: &str,
    ) -> Result<WorktreeAllocation, DomainViolation> {
        Ok(WorktreeAllocation::Provisioned {
            worktree_path: format!("/tmp/worktree-{}", task_id),
            branch_name: format!("worktree/{}", task_id),
        })
    }
    fn check_worktree_status(
        &self,
        _worktree_path: &str,
    ) -> Result<WorktreeStatus, DomainViolation> {
        Ok(WorktreeStatus {
            worktree_path: "/tmp/worktree-TASK-007".to_string(),
            branch_name: "worktree/TASK-007".to_string(),
            is_clean: true,
            head_commit_sha: "head_commit_sha".to_string(),
        })
    }
    fn merge_and_prune_worktree(
        &self,
        _task_id: &str,
        _worktree_path: &str,
        _target_branch: &str,
    ) -> Result<String, DomainViolation> {
        Ok("commit-sha-task-007".to_string())
    }
}

struct StepSensitiveSupervisor {
    call_count: std::sync::atomic::AtomicUsize,
}

impl StepSensitiveSupervisor {
    fn new() -> Self {
        Self {
            call_count: std::sync::atomic::AtomicUsize::new(0),
        }
    }
}

impl AsyncExecutionSupervisorPort for StepSensitiveSupervisor {
    fn execute_supervised(
        &self,
        _request: &SupervisedExecutionRequest,
    ) -> Result<SupervisedExecutionOutcome, DomainViolation> {
        let count = self.call_count.fetch_add(1, Ordering::SeqCst);
        let exit_code = if count == 0 { 101 } else { 0 };
        let stdout = if count == 0 {
            "test failed as expected in red phase".to_string()
        } else {
            "test passed in green phase".to_string()
        };

        Ok(SupervisedExecutionOutcome {
            exit_code,
            duration_ms: 10,
            stdout_tail: stdout,
            stderr_tail: "".to_string(),
            output_blake3_digest: "hash".to_string(),
        })
    }
}

#[test]
fn test_builder_agent_dispatcher_materializes_expected_files() {
    let dir = tempdir().expect("tempdir");
    let wt_path = dir.path().to_str().expect("path");
    let dispatcher = BuilderAgentDispatcher::new()
        .with_agent_command("echo builder-done")
        .with_cargo_target_dir_prefix("target/test_isolated");

    let expected = vec![
        "crates/trustkit-orch/src/new_mod.rs".to_string(),
        "crates/trustkit-orch/src/another_mod.rs".to_string(),
    ];

    let outcome = dispatcher
        .dispatch_builder_in_worktree("cyc-1", "TASK-007", wt_path, "tests/dummy.rs", &expected)
        .expect("dispatch should succeed");

    assert_eq!(outcome.exit_code, 0);
    assert!(
        dir.path()
            .join("crates/trustkit-orch/src/new_mod.rs")
            .exists()
    );
    assert!(
        dir.path()
            .join("crates/trustkit-orch/src/another_mod.rs")
            .exists()
    );
}

#[test]
fn test_builder_agent_dispatcher_fails_when_worktree_missing() {
    let dispatcher = BuilderAgentDispatcher::new();
    let result = dispatcher.dispatch_builder_in_worktree(
        "cyc-1",
        "TASK-007",
        "/path/that/does/not/exist/ever/12345",
        "tests/dummy.rs",
        &[],
    );

    assert!(result.is_err());
    match result {
        Err(DomainViolation::InfrastructureBoundaryFault { subsystem, details }) => {
            assert_eq!(subsystem, "builder_agent_dispatcher");
            assert!(details.contains("Worktree path does not exist"));
        }
        _ => panic!("Expected InfrastructureBoundaryFault"),
    }
}

#[test]
fn test_autonomous_wave_orchestrator_dispatches_builder_between_red_and_green() {
    let task = AtomicTask {
        cycle_id: "test-cycle".to_string(),
        task_id: "TASK-007".to_string(),
        title: "Dispatch do Agente Construtor".to_string(),
        architectural_layer: "ORCH".to_string(),
        wave_index: 3,
        depends_on: vec![],
        expected_files: vec!["crates/trustkit-orch/src/builder_agent_dispatcher.rs".to_string()],
        target_test_file: "crates/trustkit-orch/tests/builder_agent_dispatch_tests.rs".to_string(),
        tdd_lifecycle_state: TddLifecycleState::AwaitingRedPhase,
        red_evidence_id: 0,
        green_evidence_id: 0,
        worktree_allocation: WorktreeAllocation::NotProvisioned,
        commit_sha: "UNCOMMITTED".to_string(),
    };

    let repo = MockEvidenceRepo::new(vec![task]);
    let worktree_port = MockWorktreeProvider;
    let supervisor = StepSensitiveSupervisor::new();
    let reviewer = DefaultSemanticCodeReviewer::new();
    let (spy_dispatcher, called_flag) = SpyBuilderDispatcher::new();

    let orchestrator = AutonomousWaveOrchestrator::new();
    let req = MultiWaveExecutionRequest {
        cycle_id: "test-cycle",
        wave_indices: &[3],
        target_branch: "develop",
        now_iso: "2026-11-11T21:00:00Z",
    };
    let deps = MultiWaveDependencies {
        repo: &repo,
        worktree_port: &worktree_port,
        supervisor: &supervisor,
        reviewer: &reviewer,
        builder_dispatcher: Some(&spy_dispatcher),
    };

    let outcome = orchestrator
        .execute_wave_loop(&req, &deps)
        .expect("orchestrator execution must succeed");

    assert_eq!(outcome.overall_verdict, GateVerdict::Pass);
    assert!(
        called_flag.load(Ordering::SeqCst),
        "BuilderAgentDispatcherPort must have been called between RED and GREEN"
    );
}
