// Unit tests for TDD Lifecycle Use Cases (TASK-003).
// Constitutional Principle III: Test-First & Anti-Cheat.

use std::sync::Mutex;
use trustkit_core::entities::{
    ArtifactSeal, AtomicTask, ExecutionEvidence, SddCycle, TaskNode, TddPhase, WaveSchedule,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::{
    AsyncExecutionSupervisorPort, CycleEvidenceRepositoryPort, SupervisedExecutionRequest,
    SupervisedExecutionOutcome,
};
use trustkit_core::special_cases::{ActiveCycleLookup, TddLifecycleState, WorktreeAllocation};
use trustkit_core::use_cases::complete_task::{self, CompleteTaskRequest};
use trustkit_core::use_cases::record_tdd_phase::{self, RecordTddPhaseRequest};

struct InMemoryRepo {
    tasks: Mutex<Vec<AtomicTask>>,
    evidences: Mutex<Vec<ExecutionEvidence>>,
}

impl CycleEvidenceRepositoryPort for InMemoryRepo {
    fn save_cycle(&self, _cycle: &SddCycle) -> Result<(), DomainViolation> { Ok(()) }
    fn find_active_cycle(&self) -> Result<ActiveCycleLookup, DomainViolation> {
        Ok(ActiveCycleLookup::NoActiveCycle)
    }
    fn save_artifact_seal(&self, _seal: &ArtifactSeal) -> Result<u64, DomainViolation> { Ok(1) }
    fn list_artifact_seals(&self, _cycle_id: &str) -> Result<Vec<ArtifactSeal>, DomainViolation> { Ok(vec![]) }
    fn append_evidence(&self, evidence: &ExecutionEvidence) -> Result<u64, DomainViolation> {
        let mut list = self.evidences.lock().unwrap();
        list.push(evidence.clone());
        Ok(list.len() as u64)
    }
    fn list_evidences_for_cycle(&self, _cycle_id: &str) -> Result<Vec<ExecutionEvidence>, DomainViolation> {
        Ok(self.evidences.lock().unwrap().clone())
    }
    fn update_cycle_spec_lock(&self, _cycle_id: &str, _digest: &str, _state: &str) -> Result<(), DomainViolation> { Ok(()) }
    fn save_atomic_tasks(&self, _c: &str, _t: &[TaskNode], _w: &WaveSchedule) -> Result<(), DomainViolation> { Ok(()) }

    fn find_atomic_task(&self, _cycle_id: &str, task_id: &str) -> Result<Option<AtomicTask>, DomainViolation> {
        let list = self.tasks.lock().unwrap();
        Ok(list.iter().find(|t| t.task_id == task_id).cloned())
    }

    fn update_task_tdd_phase(&self, _cycle_id: &str, task_id: &str, state: &TddLifecycleState) -> Result<(), DomainViolation> {
        let mut list = self.tasks.lock().unwrap();
        if let Some(t) = list.iter_mut().find(|t| t.task_id == task_id) {
            t.tdd_lifecycle_state = state.clone();
        }
        Ok(())
    }
}

struct MockSupervisor {
    exit_code: i32,
    stdout: String,
    stderr: String,
}

impl AsyncExecutionSupervisorPort for MockSupervisor {
    fn execute_supervised(&self, _request: &SupervisedExecutionRequest) -> Result<SupervisedExecutionOutcome, DomainViolation> {
        Ok(SupervisedExecutionOutcome {
            exit_code: self.exit_code,
            duration_ms: 42,
            stdout_tail: self.stdout.clone(),
            stderr_tail: self.stderr.clone(),
            output_blake3_digest: "blake3_hash".to_string(),
        })
    }
}

fn sample_task(task_id: &str, state: TddLifecycleState) -> AtomicTask {
    AtomicTask {
        cycle_id: "cycle-100".to_string(),
        task_id: task_id.to_string(),
        title: "Sample Task".to_string(),
        architectural_layer: "Domain".to_string(),
        wave_index: 1,
        depends_on: vec![],
        expected_files: vec!["crates/lib.rs".to_string()],
        target_test_file: "tests/test.rs".to_string(),
        tdd_lifecycle_state: state,
        red_evidence_id: 0,
        green_evidence_id: 0,
        worktree_allocation: WorktreeAllocation::NotProvisioned,
        commit_sha: "".to_string(),
    }
}

#[test]
fn test_green_phase_rejected_when_awaiting_red_phase() {
    let repo = InMemoryRepo {
        tasks: Mutex::new(vec![sample_task("TASK-001", TddLifecycleState::AwaitingRedPhase)]),
        evidences: Mutex::new(vec![]),
    };
    let supervisor = MockSupervisor {
        exit_code: 0,
        stdout: "ok".to_string(),
        stderr: "".to_string(),
    };

    let req = RecordTddPhaseRequest {
        cycle_id: "cycle-100".to_string(),
        task_id: "TASK-001".to_string(),
        phase: TddPhase::Green,
        test_command: "cargo".to_string(),
        test_args: vec!["test".to_string()],
        working_dir: ".".to_string(),
        timeout_ms: 10_000,
        now_iso: "2026-10-28T12:00:00Z".to_string(),
    };

    let result = record_tdd_phase::execute(req, &repo, &supervisor);
    match result {
        Err(DomainViolation::TddLifecycleViolation { task_id, reason }) => {
            assert_eq!(task_id, "TASK-001");
            assert!(reason.contains("MissingRedPhaseProof"));
        }
        other => panic!("Expected TddLifecycleViolation with MissingRedPhaseProof, got: {other:?}"),
    }
}

#[test]
fn test_red_phase_advances_to_red_verified_on_test_failure() {
    let repo = InMemoryRepo {
        tasks: Mutex::new(vec![sample_task("TASK-002", TddLifecycleState::AwaitingRedPhase)]),
        evidences: Mutex::new(vec![]),
    };
    let supervisor = MockSupervisor {
        exit_code: 101, // Rust panic exit code
        stdout: "running 1 test\ntest failed".to_string(),
        stderr: "assertion failed".to_string(),
    };

    let req = RecordTddPhaseRequest {
        cycle_id: "cycle-100".to_string(),
        task_id: "TASK-002".to_string(),
        phase: TddPhase::Red,
        test_command: "cargo".to_string(),
        test_args: vec!["test".to_string()],
        working_dir: ".".to_string(),
        timeout_ms: 10_000,
        now_iso: "2026-10-28T12:00:00Z".to_string(),
    };

    let outcome = record_tdd_phase::execute(req, &repo, &supervisor).expect("red phase should succeed");
    assert_eq!(outcome.phase, TddPhase::Red);
    assert!(outcome.new_state.is_red_verified());
    assert_eq!(repo.evidences.lock().unwrap().len(), 1);
}

#[test]
fn test_red_phase_fails_when_test_passes() {
    let repo = InMemoryRepo {
        tasks: Mutex::new(vec![sample_task("TASK-002", TddLifecycleState::AwaitingRedPhase)]),
        evidences: Mutex::new(vec![]),
    };
    let supervisor = MockSupervisor {
        exit_code: 0,
        stdout: "test ok".to_string(),
        stderr: "".to_string(),
    };

    let req = RecordTddPhaseRequest {
        cycle_id: "cycle-100".to_string(),
        task_id: "TASK-002".to_string(),
        phase: TddPhase::Red,
        test_command: "cargo".to_string(),
        test_args: vec!["test".to_string()],
        working_dir: ".".to_string(),
        timeout_ms: 10_000,
        now_iso: "2026-10-28T12:00:00Z".to_string(),
    };

    let result = record_tdd_phase::execute(req, &repo, &supervisor);
    assert!(matches!(result, Err(DomainViolation::TddLifecycleViolation { .. })));
}

#[test]
fn test_green_phase_advances_to_green_verified_on_test_success() {
    let repo = InMemoryRepo {
        tasks: Mutex::new(vec![sample_task("TASK-003", TddLifecycleState::RedVerified { failing_evidence_id: 1 })]),
        evidences: Mutex::new(vec![]),
    };
    let supervisor = MockSupervisor {
        exit_code: 0,
        stdout: "running 1 test\ntest result: ok".to_string(),
        stderr: "".to_string(),
    };

    let req = RecordTddPhaseRequest {
        cycle_id: "cycle-100".to_string(),
        task_id: "TASK-003".to_string(),
        phase: TddPhase::Green,
        test_command: "cargo".to_string(),
        test_args: vec!["test".to_string()],
        working_dir: ".".to_string(),
        timeout_ms: 10_000,
        now_iso: "2026-10-28T12:00:00Z".to_string(),
    };

    let outcome = record_tdd_phase::execute(req, &repo, &supervisor).expect("green phase should succeed");
    assert_eq!(outcome.phase, TddPhase::Green);
    assert!(outcome.new_state.is_green_verified());
}

#[test]
fn test_complete_task_advances_to_committed() {
    let repo = InMemoryRepo {
        tasks: Mutex::new(vec![sample_task("TASK-004", TddLifecycleState::GreenVerified { red_evidence_id: 1, green_evidence_id: 2 })]),
        evidences: Mutex::new(vec![]),
    };

    let req = CompleteTaskRequest {
        cycle_id: "cycle-100".to_string(),
        task_id: "TASK-004".to_string(),
        commit_sha: "abcdef1234567890abcdef1234567890abcdef12".to_string(),
        now_iso: "2026-10-28T12:00:00Z".to_string(),
    };

    let outcome = complete_task::execute(req, &repo).expect("complete task should succeed");
    assert!(outcome.new_state.is_committed());
}

#[test]
fn test_complete_task_fails_when_not_green() {
    let repo = InMemoryRepo {
        tasks: Mutex::new(vec![sample_task("TASK-004", TddLifecycleState::RedVerified { failing_evidence_id: 1 })]),
        evidences: Mutex::new(vec![]),
    };

    let req = CompleteTaskRequest {
        cycle_id: "cycle-100".to_string(),
        task_id: "TASK-004".to_string(),
        commit_sha: "abcdef1234567890abcdef1234567890abcdef12".to_string(),
        now_iso: "2026-10-28T12:00:00Z".to_string(),
    };

    let result = complete_task::execute(req, &repo);
    assert!(matches!(result, Err(DomainViolation::TddLifecycleViolation { .. })));
}
