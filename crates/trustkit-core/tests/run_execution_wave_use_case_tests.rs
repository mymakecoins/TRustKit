// Unit tests for RunExecutionWaveUseCase (TASK-004).
// Constitutional Principle III: Test-First & Anti-Cheat.

use std::sync::Mutex;
use trustkit_core::entities::{
    ArtifactSeal, AtomicTask, CycleStatus, ExecutionEvidence, GateStage, MethodologyProfile,
    SddCycle, TaskNode, WaveSchedule,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::{CycleEvidenceRepositoryPort, WorktreeIsolationPort, WorktreeStatus};
use trustkit_core::special_cases::{
    ActiveCycleLookup, IssueLinkage, SpecLockState, TddLifecycleState, WorktreeAllocation,
};
use trustkit_core::use_cases::run_execution_wave::{
    self, RunExecutionWaveRequest, validate_commit_message,
};

struct InMemoryWaveRepo {
    cycle: Mutex<Option<SddCycle>>,
    tasks: Mutex<Vec<AtomicTask>>,
    evidences: Mutex<Vec<ExecutionEvidence>>,
}

impl CycleEvidenceRepositoryPort for InMemoryWaveRepo {
    fn save_cycle(&self, cycle: &SddCycle) -> Result<(), DomainViolation> {
        *self.cycle.lock().unwrap() = Some(cycle.clone());
        Ok(())
    }

    fn find_active_cycle(&self) -> Result<ActiveCycleLookup, DomainViolation> {
        let c = self.cycle.lock().unwrap();
        match &*c {
            Some(cycle) => Ok(ActiveCycleLookup::Found(Box::new(cycle.clone()))),
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
        _c: &str,
        _t: &[TaskNode],
        _w: &WaveSchedule,
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
        allocation: &WorktreeAllocation,
    ) -> Result<(), DomainViolation> {
        let mut list = self.tasks.lock().unwrap();
        if let Some(t) = list.iter_mut().find(|t| t.task_id == task_id) {
            t.worktree_allocation = allocation.clone();
        }
        Ok(())
    }
}

struct MockWorktreePort {
    provisioned_paths: Mutex<Vec<String>>,
    merged_paths: Mutex<Vec<String>>,
}

impl WorktreeIsolationPort for MockWorktreePort {
    fn provision_worktree(
        &self,
        _cycle_id: &str,
        task_id: &str,
        _base_branch: &str,
    ) -> Result<WorktreeAllocation, DomainViolation> {
        let path = format!("/tmp/worktrees/{}", task_id);
        self.provisioned_paths.lock().unwrap().push(path.clone());
        Ok(WorktreeAllocation::Provisioned {
            worktree_path: path,
            branch_name: format!("worktree/{}", task_id),
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
        worktree_path: &str,
        _target_branch: &str,
    ) -> Result<String, DomainViolation> {
        self.merged_paths
            .lock()
            .unwrap()
            .push(worktree_path.to_string());
        Ok("merge_sha_abcdef1234567890".to_string())
    }
}

fn build_cycle(spec_state: SpecLockState) -> SddCycle {
    SddCycle {
        cycle_id: "cycle-wave-1".to_string(),
        issue_linkage: IssueLinkage::LinkedIssue {
            issue_number: 45,
            issue_url: "https://github.com/org/repo/issues/45".to_string(),
            project_board_id: "board-1".to_string(),
            project_item_id: "item-1".to_string(),
            board_column: trustkit_core::special_cases::BoardColumn::InProgress,
        },
        methodology_profile: MethodologyProfile::StandardGauntletV2,
        current_gate: GateStage::Gate3WavePlanned,
        lifecycle_status: CycleStatus::Active,
        gauntlet_attempt: 0,
        spec_dir: "specs/004".to_string(),
        spec_file_path: "specs/004/spec.md".to_string(),
        spec_seal_state: spec_state,
        work_branch: "feat/45-worktree-tdd".to_string(),
        base_commit_sha: "000000".to_string(),
        pre_existing_dirty_files: vec![],
        created_at: "2026-10-28T12:00:00Z".to_string(),
        updated_at: "2026-10-28T12:00:00Z".to_string(),
    }
}

fn build_task(task_id: &str, wave_index: u32) -> AtomicTask {
    AtomicTask {
        cycle_id: "cycle-wave-1".to_string(),
        task_id: task_id.to_string(),
        title: format!("Task {}", task_id),
        architectural_layer: "Core".to_string(),
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
fn test_wave_execution_fails_when_spec_draft_unverified() {
    let repo = InMemoryWaveRepo {
        cycle: Mutex::new(Some(build_cycle(SpecLockState::DraftUnverified))),
        tasks: Mutex::new(vec![build_task("TASK-001", 1)]),
        evidences: Mutex::new(vec![]),
    };
    let worktree_port = MockWorktreePort {
        provisioned_paths: Mutex::new(vec![]),
        merged_paths: Mutex::new(vec![]),
    };

    let req = RunExecutionWaveRequest {
        cycle_id: "cycle-wave-1".to_string(),
        wave_index: 1,
        now_iso: "2026-10-28T12:00:00Z".to_string(),
    };

    let res = run_execution_wave::execute(req, &repo, &worktree_port);
    assert!(matches!(
        res,
        Err(DomainViolation::SpecOrTraceabilityIncomplete { .. })
    ));
}

#[test]
fn test_wave_execution_provisions_and_merges_two_tasks() {
    let frozen = SpecLockState::Frozen {
        spec_path: "specs/004/spec.md".to_string(),
        blake3_digest: "hash".to_string(),
    };
    let repo = InMemoryWaveRepo {
        cycle: Mutex::new(Some(build_cycle(frozen))),
        tasks: Mutex::new(vec![
            build_task("TASK-001", 1),
            build_task("TASK-002", 1),
            build_task("TASK-003", 2), // wave 2 should not run
        ]),
        evidences: Mutex::new(vec![]),
    };
    let worktree_port = MockWorktreePort {
        provisioned_paths: Mutex::new(vec![]),
        merged_paths: Mutex::new(vec![]),
    };

    let req = RunExecutionWaveRequest {
        cycle_id: "cycle-wave-1".to_string(),
        wave_index: 1,
        now_iso: "2026-10-28T12:00:00Z".to_string(),
    };

    let outcome = run_execution_wave::execute(req, &repo, &worktree_port)
        .expect("wave execution should succeed");

    assert_eq!(outcome.wave_index, 1);
    assert_eq!(outcome.tasks_executed.len(), 2);
    assert_eq!(worktree_port.provisioned_paths.lock().unwrap().len(), 2);
    assert_eq!(worktree_port.merged_paths.lock().unwrap().len(), 2);
    // Evidences: 2 provision + 2 merge = 4
    assert_eq!(repo.evidences.lock().unwrap().len(), 4);
}

#[test]
fn test_commit_message_validation_nominal_and_violations() {
    // Valid patterns
    assert!(validate_commit_message("feat: #45 implementa worktrees").is_ok());
    assert!(validate_commit_message("fix: #102 corrige erro de validacao").is_ok());
    assert!(validate_commit_message("test: #3 adiciona testes unitarios").is_ok());

    // Invalid patterns
    assert!(validate_commit_message("wip commit").is_err());
    assert!(validate_commit_message("feat: sem numero de issue").is_err());
    assert!(validate_commit_message("foo: #45 tipo invalido").is_err());
}
