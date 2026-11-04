// Unit tests for Gauntlet Layer 1 and Circuit Breaker Use Cases (TASK-005).
// Constitutional Principle III: Test-First & Anti-Cheat.

use std::sync::Mutex;
use trustkit_core::entities::{
    ArtifactSeal, CycleStatus, EvidenceKind, ExecutionEvidence, GateStage, GateVerdict,
    MethodologyProfile, SddCycle, TaskNode, WaveSchedule,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::{
    AsyncExecutionSupervisorPort, CycleEvidenceRepositoryPort, ForeignScreenshotInspection,
    SupervisedExecutionOutcome, SupervisedExecutionRequest, VisualEvidencePort,
};
use trustkit_core::special_cases::{ActiveCycleLookup, IssueLinkage, SpecLockState};
use trustkit_core::use_cases::run_gauntlet_layer1::{self, RunGauntletLayer1Request};
use trustkit_core::use_cases::trip_circuit_breaker::{self, TripCircuitBreakerRequest};

struct InMemoryGauntletRepo {
    cycle: Mutex<Option<SddCycle>>,
    evidences: Mutex<Vec<ExecutionEvidence>>,
}

impl CycleEvidenceRepositoryPort for InMemoryGauntletRepo {
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

    fn update_cycle_status(
        &self,
        _cycle_id: &str,
        status: CycleStatus,
        gauntlet_attempt: u8,
    ) -> Result<(), DomainViolation> {
        let mut c = self.cycle.lock().unwrap();
        if let Some(cycle) = c.as_mut() {
            cycle.lifecycle_status = status;
            cycle.gauntlet_attempt = gauntlet_attempt;
        }
        Ok(())
    }

    fn advance_cycle_gate(
        &self,
        _cycle_id: &str,
        next_gate: GateStage,
    ) -> Result<(), DomainViolation> {
        let mut c = self.cycle.lock().unwrap();
        if let Some(cycle) = c.as_mut() {
            cycle.current_gate = next_gate;
        }
        Ok(())
    }
}

struct MockSupervisor {
    playwright_fails: bool,
    calls: Mutex<Vec<String>>,
}

impl AsyncExecutionSupervisorPort for MockSupervisor {
    fn execute_supervised(
        &self,
        request: &SupervisedExecutionRequest,
    ) -> Result<SupervisedExecutionOutcome, DomainViolation> {
        self.calls.lock().unwrap().push(request.command.clone());
        let fails = self.playwright_fails && request.args.contains(&"playwright".to_string());
        Ok(SupervisedExecutionOutcome {
            exit_code: if fails { 1 } else { 0 },
            duration_ms: 100,
            stdout_tail: "output".to_string(),
            stderr_tail: if fails {
                "Playwright snapshot assertion mismatch".to_string()
            } else {
                "".to_string()
            },
            output_blake3_digest: "hash".to_string(),
        })
    }
}

struct MockVisualPort {
    restored_called: Mutex<bool>,
}

impl VisualEvidencePort for MockVisualPort {
    fn restore_foreign_screenshots(
        &self,
        active_issue_prefix: &str,
        _evidence_dir: &str,
    ) -> Result<ForeignScreenshotInspection, DomainViolation> {
        *self.restored_called.lock().unwrap() = true;
        Ok(ForeignScreenshotInspection {
            active_issue_prefix: active_issue_prefix.to_string(),
            foreign_screenshots_detected: vec![],
            restored_screenshots: vec![],
        })
    }
}

fn build_cycle_for_gauntlet(attempt: u8) -> SddCycle {
    SddCycle {
        cycle_id: "cycle-gauntlet-1".to_string(),
        issue_linkage: IssueLinkage::LinkedIssue {
            issue_number: 45,
            issue_url: "url".to_string(),
            project_board_id: "b".to_string(),
            project_item_id: "i".to_string(),
            board_column: trustkit_core::special_cases::BoardColumn::InProgress,
        },
        methodology_profile: MethodologyProfile::StandardGauntletV2,
        current_gate: GateStage::Gate4TasksImplemented,
        lifecycle_status: CycleStatus::Active,
        gauntlet_attempt: attempt,
        spec_dir: "specs".to_string(),
        spec_file_path: "specs/spec.md".to_string(),
        spec_seal_state: SpecLockState::Frozen {
            spec_path: "p".into(),
            blake3_digest: "h".into(),
        },
        work_branch: "feat/45".to_string(),
        base_commit_sha: "sha".to_string(),
        pre_existing_dirty_files: vec![],
        created_at: "2026-10-28T12:00:00Z".to_string(),
        updated_at: "2026-10-28T12:00:00Z".to_string(),
    }
}

fn make_layer0_evidence(verdict: GateVerdict) -> ExecutionEvidence {
    ExecutionEvidence {
        evidence_id: 1,
        cycle_id: "cycle-gauntlet-1".to_string(),
        gate_stage: GateStage::Gate5GauntletCleared,
        gauntlet_attempt: 0,
        actor_role: "AUDITOR:AST_LAYER0".to_string(),
        evidence_kind: EvidenceKind::AstAudit,
        executed_command: "trustkit gauntlet ast-audit".to_string(),
        exit_code: if verdict == GateVerdict::Pass { 0 } else { 4 },
        duration_ms: 10,
        sanitized_stdout_tail: "".to_string(),
        sanitized_stderr_tail: "".to_string(),
        output_blake3_digest: "".to_string(),
        verdict,
        recorded_at: "2026-10-28T12:00:00Z".to_string(),
    }
}

#[test]
fn test_layer1_not_dispatched_if_layer0_failed() {
    let repo = InMemoryGauntletRepo {
        cycle: Mutex::new(Some(build_cycle_for_gauntlet(0))),
        evidences: Mutex::new(vec![make_layer0_evidence(GateVerdict::Fail)]),
    };
    let supervisor = MockSupervisor {
        playwright_fails: false,
        calls: Mutex::new(vec![]),
    };
    let visual = MockVisualPort {
        restored_called: Mutex::new(false),
    };

    let req = RunGauntletLayer1Request {
        cycle_id: "cycle-gauntlet-1".to_string(),
        working_dir: ".".to_string(),
        command_overrides: vec![],
        now_iso: "2026-10-28T12:00:00Z".to_string(),
    };

    let res = run_gauntlet_layer1::execute(req, &repo, &supervisor, &visual);
    assert!(matches!(
        res,
        Err(DomainViolation::CriticSwarmOrPlaywrightFailed { .. })
    ));
    // Layer 1 critics should NOT have been called
    assert_eq!(supervisor.calls.lock().unwrap().len(), 0);
    assert!(!*visual.restored_called.lock().unwrap());
}

#[test]
fn test_layer1_fails_when_playwright_fails_increments_attempt() {
    let repo = InMemoryGauntletRepo {
        cycle: Mutex::new(Some(build_cycle_for_gauntlet(0))),
        evidences: Mutex::new(vec![make_layer0_evidence(GateVerdict::Pass)]),
    };
    let supervisor = MockSupervisor {
        playwright_fails: true,
        calls: Mutex::new(vec![]),
    };
    let visual = MockVisualPort {
        restored_called: Mutex::new(false),
    };

    let req = RunGauntletLayer1Request {
        cycle_id: "cycle-gauntlet-1".to_string(),
        working_dir: ".".to_string(),
        command_overrides: vec![],
        now_iso: "2026-10-28T12:00:00Z".to_string(),
    };

    let res = run_gauntlet_layer1::execute(req, &repo, &supervisor, &visual);
    assert!(matches!(
        res,
        Err(DomainViolation::CriticSwarmOrPlaywrightFailed { attempt: 1, .. })
    ));
    assert_eq!(
        repo.cycle
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .gauntlet_attempt,
        1
    );
    assert!(*visual.restored_called.lock().unwrap());
}

#[test]
fn test_circuit_breaker_tripped_at_fifth_failure() {
    // Cycle starts at 4 failures; next failure must trip breaker
    let repo = InMemoryGauntletRepo {
        cycle: Mutex::new(Some(build_cycle_for_gauntlet(4))),
        evidences: Mutex::new(vec![make_layer0_evidence(GateVerdict::Pass)]),
    };
    let supervisor = MockSupervisor {
        playwright_fails: true,
        calls: Mutex::new(vec![]),
    };
    let visual = MockVisualPort {
        restored_called: Mutex::new(false),
    };

    let req = RunGauntletLayer1Request {
        cycle_id: "cycle-gauntlet-1".to_string(),
        working_dir: ".".to_string(),
        command_overrides: vec![],
        now_iso: "2026-10-28T12:00:00Z".to_string(),
    };

    let res = run_gauntlet_layer1::execute(req, &repo, &supervisor, &visual);
    match res {
        Err(DomainViolation::CircuitBreakerTripped {
            consecutive_failures,
            ..
        }) => {
            assert_eq!(consecutive_failures, 5);
        }
        other => panic!("Expected CircuitBreakerTripped, got: {other:?}"),
    }
    assert_eq!(
        repo.cycle
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .lifecycle_status,
        CycleStatus::CircuitBreakerTripped
    );
}

#[test]
fn test_unanimous_pass_clears_gauntlet_and_resets_counter() {
    let repo = InMemoryGauntletRepo {
        cycle: Mutex::new(Some(build_cycle_for_gauntlet(2))),
        evidences: Mutex::new(vec![make_layer0_evidence(GateVerdict::Pass)]),
    };
    let supervisor = MockSupervisor {
        playwright_fails: false,
        calls: Mutex::new(vec![]),
    };
    let visual = MockVisualPort {
        restored_called: Mutex::new(false),
    };

    let req = RunGauntletLayer1Request {
        cycle_id: "cycle-gauntlet-1".to_string(),
        working_dir: ".".to_string(),
        command_overrides: vec![],
        now_iso: "2026-10-28T12:00:00Z".to_string(),
    };

    let outcome = run_gauntlet_layer1::execute(req, &repo, &supervisor, &visual)
        .expect("unanimous pass expected");

    assert!(outcome.unanimous_pass);
    assert_eq!(outcome.findings.len(), 4);
    let cycle = repo.cycle.lock().unwrap().clone().unwrap();
    assert_eq!(cycle.gauntlet_attempt, 0);
    assert_eq!(cycle.current_gate, GateStage::Gate5GauntletCleared);
}

#[test]
fn test_trip_circuit_breaker_use_case() {
    let repo = InMemoryGauntletRepo {
        cycle: Mutex::new(Some(build_cycle_for_gauntlet(4))),
        evidences: Mutex::new(vec![]),
    };

    let req = TripCircuitBreakerRequest {
        cycle_id: "cycle-gauntlet-1".to_string(),
        diagnostic_reason: "Manual trip for test".to_string(),
        now_iso: "2026-10-28T12:00:00Z".to_string(),
    };

    let outcome = trip_circuit_breaker::execute(req, &repo).expect("trip use case execution");
    assert_eq!(outcome.gauntlet_attempt, 5);
    assert!(outcome.state.is_tripped());
    assert_eq!(
        repo.cycle
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .lifecycle_status,
        CycleStatus::CircuitBreakerTripped
    );
}
