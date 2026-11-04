// Tests for run_interactive_wave use case (TASK-006).
// Constitutional Principle I: Zero external dependencies (std only in core).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use std::sync::Mutex;
use trustkit_core::entities::{
    AgentSemanticState, ArtifactSeal, AtomicTask, BlockReasonKind, CycleStatus, ExecutionEvidence,
    GateStage, GateVerdict, MethodologyProfile, MultiplexedTerminalHandle,
    MultiplexedTerminalRequest, RuntimeBackendMode, SddCycle, TaskNode, WaveSchedule,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::{
    CycleEvidenceRepositoryPort, TerminalMultiplexerPort, WorktreeIsolationPort, WorktreeStatus,
};
use trustkit_core::special_cases::{
    ActiveCycleLookup, BoardColumn, IssueLinkage, SpecLockState, TddLifecycleState,
    WorktreeAllocation,
};
use trustkit_core::use_cases::run_interactive_wave::{RunInteractiveWaveRequest, execute};

#[derive(Default)]
struct MockCycleRepo {
    cycle: Mutex<Option<SddCycle>>,
    tasks: Mutex<Vec<AtomicTask>>,
    evidences: Mutex<Vec<ExecutionEvidence>>,
}

impl CycleEvidenceRepositoryPort for MockCycleRepo {
    fn save_cycle(&self, cycle: &SddCycle) -> Result<(), DomainViolation> {
        *self.cycle.lock().unwrap() = Some(cycle.clone());
        Ok(())
    }
    fn find_active_cycle(&self) -> Result<ActiveCycleLookup, DomainViolation> {
        let guard = self.cycle.lock().unwrap();
        match guard.as_ref() {
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
    fn append_evidence(&self, evidence: &ExecutionEvidence) -> Result<u64, DomainViolation> {
        let mut list = self.evidences.lock().unwrap();
        list.push(evidence.clone());
        Ok(list.len() as u64)
    }
    fn list_evidences_for_cycle(
        &self,
        _cycle_id: &str,
    ) -> Result<Vec<ExecutionEvidence>, DomainViolation> {
        Ok(self.evidences.lock().unwrap().clone())
    }
    fn update_cycle_spec_lock(
        &self,
        _cycle_id: &str,
        _digest: &str,
        _state: &str,
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
    fn update_task_worktree(
        &self,
        _cycle_id: &str,
        task_id: &str,
        alloc: &WorktreeAllocation,
    ) -> Result<(), DomainViolation> {
        let mut list = self.tasks.lock().unwrap();
        if let Some(t) = list.iter_mut().find(|t| t.task_id == task_id) {
            t.worktree_allocation = alloc.clone();
        }
        Ok(())
    }
}

struct MockWorktreePort;

impl WorktreeIsolationPort for MockWorktreePort {
    fn provision_worktree(
        &self,
        _cycle_id: &str,
        task_id: &str,
        base_branch: &str,
    ) -> Result<WorktreeAllocation, DomainViolation> {
        Ok(WorktreeAllocation::Provisioned {
            worktree_path: format!("/tmp/worktree-{}", task_id),
            branch_name: format!("{}-branch", base_branch),
        })
    }
    fn check_worktree_status(
        &self,
        worktree_path: &str,
    ) -> Result<WorktreeStatus, DomainViolation> {
        Ok(WorktreeStatus {
            worktree_path: worktree_path.to_string(),
            branch_name: "branch".to_string(),
            is_clean: true,
            head_commit_sha: "head123".to_string(),
        })
    }
    fn merge_and_prune_worktree(
        &self,
        _task_id: &str,
        _path: &str,
        _target: &str,
    ) -> Result<String, DomainViolation> {
        Ok("commit-sha-123456".to_string())
    }
}

struct MockMultiplexerPort {
    simulated_state: AgentSemanticState,
}

impl TerminalMultiplexerPort for MockMultiplexerPort {
    fn spawn_terminal(
        &self,
        request: &MultiplexedTerminalRequest,
    ) -> Result<MultiplexedTerminalHandle, DomainViolation> {
        Ok(MultiplexedTerminalHandle::new(
            format!("term-{}", request.task_id),
            request.task_id.clone(),
            1234,
            "2026-11-03T17:00:00Z",
        ))
    }
    fn query_terminal_state(
        &self,
        _handle: &MultiplexedTerminalHandle,
    ) -> Result<AgentSemanticState, DomainViolation> {
        Ok(self.simulated_state.clone())
    }
    fn send_terminal_input(
        &self,
        _handle: &MultiplexedTerminalHandle,
        _input: &str,
    ) -> Result<(), DomainViolation> {
        Ok(())
    }
    fn read_terminal_buffer(
        &self,
        _handle: &MultiplexedTerminalHandle,
        _lines: usize,
    ) -> Result<String, DomainViolation> {
        Ok("Mock buffer output\n".to_string())
    }
    fn terminate_terminal(
        &self,
        _handle: &MultiplexedTerminalHandle,
    ) -> Result<(), DomainViolation> {
        Ok(())
    }
}

fn create_test_cycle() -> SddCycle {
    SddCycle {
        cycle_id: "test-cycle-1".to_string(),
        issue_linkage: IssueLinkage::LinkedIssue {
            issue_number: 42,
            issue_url: "https://github.com/org/repo/issues/42".to_string(),
            project_board_id: "board-1".to_string(),
            project_item_id: "item-1".to_string(),
            board_column: BoardColumn::InProgress,
        },
        methodology_profile: MethodologyProfile::StandardGauntletV2,
        current_gate: GateStage::Gate4TasksImplemented,
        lifecycle_status: CycleStatus::Active,
        gauntlet_attempt: 0,
        spec_dir: "specs".to_string(),
        spec_file_path: "specs/spec.md".to_string(),
        spec_seal_state: SpecLockState::Frozen {
            blake3_digest: "blake3-test".to_string(),
            spec_path: "specs/spec.md".to_string(),
        },
        work_branch: "feature/42".to_string(),
        base_commit_sha: "base123".to_string(),
        pre_existing_dirty_files: vec![],
        created_at: "2026-11-03".to_string(),
        updated_at: "2026-11-03".to_string(),
    }
}

fn create_test_task(task_id: &str, wave_index: u32) -> AtomicTask {
    AtomicTask {
        cycle_id: "test-cycle-1".to_string(),
        task_id: task_id.to_string(),
        title: "Test Task".to_string(),
        architectural_layer: "CORE".to_string(),
        wave_index,
        depends_on: vec![],
        expected_files: vec![],
        target_test_file: "".to_string(),
        tdd_lifecycle_state: TddLifecycleState::AwaitingRedPhase,
        red_evidence_id: 0,
        green_evidence_id: 0,
        worktree_allocation: WorktreeAllocation::NotProvisioned,
        commit_sha: "".to_string(),
    }
}

#[test]
fn test_run_interactive_wave_success_with_multiplexer() {
    let repo = MockCycleRepo::default();
    repo.save_cycle(&create_test_cycle()).unwrap();
    *repo.tasks.lock().unwrap() = vec![
        create_test_task("TASK-001", 1),
        create_test_task("TASK-002", 1),
    ];

    let worktree_port = MockWorktreePort;
    let mux_port = MockMultiplexerPort {
        simulated_state: AgentSemanticState::Done { exit_code: 0 },
    };

    let req = RunInteractiveWaveRequest {
        cycle_id: "test-cycle-1".to_string(),
        wave_index: 1,
        runtime_mode: RuntimeBackendMode::Pty,
        agent_command: "claude".to_string(),
        now_iso: "2026-11-03T17:00:00Z".to_string(),
    };

    let outcome =
        execute(req, &repo, &worktree_port, Some(&mux_port)).expect("Should execute wave");
    assert_eq!(outcome.wave_index, 1);
    assert_eq!(outcome.tasks_dispatched.len(), 2);
    assert_eq!(outcome.overall_verdict, GateVerdict::Pass);
    assert_eq!(
        outcome.tasks_dispatched[0].merged_commit_sha,
        "commit-sha-123456"
    );
}

#[test]
fn test_run_interactive_wave_blocked_stops_merge() {
    let repo = MockCycleRepo::default();
    repo.save_cycle(&create_test_cycle()).unwrap();
    *repo.tasks.lock().unwrap() = vec![create_test_task("TASK-001", 1)];

    let worktree_port = MockWorktreePort;
    let mux_port = MockMultiplexerPort {
        simulated_state: AgentSemanticState::Blocked {
            kind: BlockReasonKind::Permission,
            detail: "Permission needed".to_string(),
        },
    };

    let req = RunInteractiveWaveRequest {
        cycle_id: "test-cycle-1".to_string(),
        wave_index: 1,
        runtime_mode: RuntimeBackendMode::Pty,
        agent_command: "claude".to_string(),
        now_iso: "2026-11-03T17:00:00Z".to_string(),
    };

    let outcome =
        execute(req, &repo, &worktree_port, Some(&mux_port)).expect("Should return outcome");
    assert_eq!(outcome.overall_verdict, GateVerdict::Fail);
    assert!(outcome.tasks_dispatched[0].state.is_blocked());
    assert_eq!(outcome.tasks_dispatched[0].merged_commit_sha, "");
}
