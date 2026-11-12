// Integration tests for TASK-006: RedVerifier and TDD Red Output Verifier.
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use std::sync::Mutex;
use trustkit_core::entities::{
    ArtifactSeal, AtomicTask, ExecutionEvidence, SddCycle, TaskNode, TddPhase, WaveSchedule,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::{
    AsyncExecutionSupervisorPort, CycleEvidenceRepositoryPort, SupervisedExecutionOutcome,
    SupervisedExecutionRequest,
};
use trustkit_core::special_cases::{ActiveCycleLookup, TddLifecycleState, WorktreeAllocation};
use trustkit_core::use_cases::record_tdd_phase::{self, RecordTddPhaseRequest, RedVerifier};

struct InMemoryRepo {
    tasks: Mutex<Vec<AtomicTask>>,
    evidences: Mutex<Vec<ExecutionEvidence>>,
}

impl CycleEvidenceRepositoryPort for InMemoryRepo {
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
    fn find_atomic_task(
        &self,
        _cycle_id: &str,
        task_id: &str,
    ) -> Result<Option<AtomicTask>, DomainViolation> {
        let list = self.tasks.lock().unwrap();
        Ok(list.iter().find(|t| t.task_id == task_id).cloned())
    }
    fn update_task_tdd_phase(
        &self,
        _cycle_id: &str,
        task_id: &str,
        state: &TddLifecycleState,
    ) -> Result<(), DomainViolation> {
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
    fn execute_supervised(
        &self,
        _request: &SupervisedExecutionRequest,
    ) -> Result<SupervisedExecutionOutcome, DomainViolation> {
        Ok(SupervisedExecutionOutcome {
            exit_code: self.exit_code,
            duration_ms: 15,
            stdout_tail: self.stdout.clone(),
            stderr_tail: self.stderr.clone(),
            output_blake3_digest: "hash-006".to_string(),
        })
    }
}

fn sample_task(task_id: &str, state: TddLifecycleState) -> AtomicTask {
    AtomicTask {
        cycle_id: "cycle-1".to_string(),
        task_id: task_id.to_string(),
        title: "TDD Red Output Verifier".to_string(),
        architectural_layer: "CORE".to_string(),
        wave_index: 2,
        depends_on: vec![],
        expected_files: vec![],
        target_test_file: "crates/trustkit-core/tests/tdd_red_output_verifier_tests.rs".to_string(),
        tdd_lifecycle_state: state,
        red_evidence_id: 0,
        green_evidence_id: 0,
        worktree_allocation: WorktreeAllocation::NotProvisioned,
        commit_sha: "UNCOMMITTED".to_string(),
    }
}

#[test]
fn test_red_verifier_approves_valid_assertion_failure() {
    let stdout = "running 1 test\ntest test_contract ... FAILED\n\nfailures:\n    test_contract\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out";
    let stderr = "thread 'main' panicked at 'assertion failed: false', tests/sample.rs:10:5";

    let result = RedVerifier::verify("TASK-006", 101, stdout, stderr);
    assert!(result.is_ok());
}

#[test]
fn test_red_verifier_rejects_zero_filtered_tests() {
    let stdout = "running 0 tests\n\ntest result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out";
    let stderr = "";

    let result = RedVerifier::verify("TASK-006", 0, stdout, stderr);
    assert!(matches!(
        result,
        Err(DomainViolation::ZeroFilteredTestsFault { ref task_id, .. }) if task_id == "TASK-006"
    ));
}

#[test]
fn test_red_verifier_rejects_compilation_error_e() {
    let stdout = "";
    let stderr = "error[E0425]: cannot find value 'unresolved_symbol' in this scope\n --> tests/sample.rs:4:5";

    let result = RedVerifier::verify("TASK-006", 101, stdout, stderr);
    assert!(matches!(
        result,
        Err(DomainViolation::CompilationFailureInRedPhaseFault { ref task_id, .. }) if task_id == "TASK-006"
    ));
}

#[test]
fn test_red_verifier_rejects_could_not_compile_error() {
    let stdout = "";
    let stderr = "error: could not compile 'trustkit-core' (test 'tdd_red_output_verifier_tests') due to 1 previous error";

    let result = RedVerifier::verify("TASK-006", 101, stdout, stderr);
    assert!(matches!(
        result,
        Err(DomainViolation::CompilationFailureInRedPhaseFault { ref task_id, .. }) if task_id == "TASK-006"
    ));
}

#[test]
fn test_red_verifier_rejects_exit_code_zero_in_red_phase() {
    let stdout = "running 1 test\ntest test_ok ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out";
    let stderr = "";

    let result = RedVerifier::verify("TASK-006", 0, stdout, stderr);
    assert!(matches!(
        result,
        Err(DomainViolation::TddLifecycleViolation { ref task_id, .. }) if task_id == "TASK-006"
    ));
}

#[test]
fn test_record_tdd_phase_red_executes_and_blocks_compilation_failure() {
    let repo = InMemoryRepo {
        tasks: Mutex::new(vec![sample_task(
            "TASK-006",
            TddLifecycleState::AwaitingRedPhase,
        )]),
        evidences: Mutex::new(vec![]),
    };
    let supervisor = MockSupervisor {
        exit_code: 101,
        stdout: "".to_string(),
        stderr: "error[E0425]: cannot find function".to_string(),
    };

    let req = RecordTddPhaseRequest {
        cycle_id: "cycle-1".to_string(),
        task_id: "TASK-006".to_string(),
        phase: TddPhase::Red,
        test_command: "cargo".to_string(),
        test_args: vec!["test".to_string()],
        working_dir: ".".to_string(),
        timeout_ms: 10_000,
        now_iso: "2026-11-11T19:00:00Z".to_string(),
    };

    let res = record_tdd_phase::execute(req, &repo, &supervisor);
    assert!(matches!(
        res,
        Err(DomainViolation::CompilationFailureInRedPhaseFault { .. })
    ));
}

#[test]
fn test_record_tdd_phase_red_executes_and_blocks_zero_filtered_tests() {
    let repo = InMemoryRepo {
        tasks: Mutex::new(vec![sample_task(
            "TASK-006",
            TddLifecycleState::AwaitingRedPhase,
        )]),
        evidences: Mutex::new(vec![]),
    };
    let supervisor = MockSupervisor {
        exit_code: 0,
        stdout: "test result: ok. 0 passed; 0 failed".to_string(),
        stderr: "".to_string(),
    };

    let req = RecordTddPhaseRequest {
        cycle_id: "cycle-1".to_string(),
        task_id: "TASK-006".to_string(),
        phase: TddPhase::Red,
        test_command: "cargo".to_string(),
        test_args: vec!["test".to_string()],
        working_dir: ".".to_string(),
        timeout_ms: 10_000,
        now_iso: "2026-11-11T19:00:00Z".to_string(),
    };

    let res = record_tdd_phase::execute(req, &repo, &supervisor);
    assert!(matches!(
        res,
        Err(DomainViolation::ZeroFilteredTestsFault { .. })
    ));
}

#[test]
fn test_record_tdd_phase_red_succeeds_on_assertion_failure() {
    let repo = InMemoryRepo {
        tasks: Mutex::new(vec![sample_task(
            "TASK-006",
            TddLifecycleState::AwaitingRedPhase,
        )]),
        evidences: Mutex::new(vec![]),
    };
    let supervisor = MockSupervisor {
        exit_code: 101,
        stdout: "running 1 test\ntest test_something ... FAILED\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored".to_string(),
        stderr: "panicked at 'assertion failed'".to_string(),
    };

    let req = RecordTddPhaseRequest {
        cycle_id: "cycle-1".to_string(),
        task_id: "TASK-006".to_string(),
        phase: TddPhase::Red,
        test_command: "cargo".to_string(),
        test_args: vec!["test".to_string()],
        working_dir: ".".to_string(),
        timeout_ms: 10_000,
        now_iso: "2026-11-11T19:00:00Z".to_string(),
    };

    let outcome = record_tdd_phase::execute(req, &repo, &supervisor)
        .expect("should record red phase successfully");
    assert!(outcome.new_state.is_red_verified());
    assert_eq!(outcome.evidence_id, 1);
}

#[test]
fn test_task_006_red_phase_contract() {
    let result = RedVerifier::verify(
        "TASK-006",
        101,
        "FAILED. 0 passed; 1 failed",
        "assertion failed",
    );
    assert!(result.is_ok(), "TASK GREEN ASSERTION: TASK-006 verified");
}
