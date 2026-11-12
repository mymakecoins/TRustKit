// Unit tests for Close Cycle Use Case (TASK-001 / TASK-002).
// Constitutional Principle III: Test-First & Anti-Cheat.

use super::{CloseCycleRequest, execute};
use crate::entities::{
    ArtifactSeal, CycleStatus, ExecutionEvidence, GateStage, MethodologyProfile, SddCycle,
    TaskNode, WaveSchedule,
};
use crate::errors::DomainViolation;
use crate::ports::{
    CycleEvidenceRepositoryPort, ProjectBoardGatewayPort, PullRequestDetails, PullRequestOutcome,
};
use crate::special_cases::{
    ActiveCycleLookup, BoardColumn, BoardSyncOutcome, IdentityVerification, IssueLinkage,
    SpecLockState,
};
use std::sync::Mutex;

struct InMemoryTestRepo {
    cycle: Mutex<Option<SddCycle>>,
}

impl CycleEvidenceRepositoryPort for InMemoryTestRepo {
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

struct MockBoardGateway {
    last_transition: Mutex<Option<(u64, BoardColumn)>>,
}

impl ProjectBoardGatewayPort for MockBoardGateway {
    fn verify_active_identity(
        &self,
        expected_handle: &str,
    ) -> Result<IdentityVerification, DomainViolation> {
        Ok(IdentityVerification::Verified {
            operator_handle: expected_handle.to_string(),
        })
    }

    fn transition_issue_column(
        &self,
        issue_number: u64,
        target: BoardColumn,
    ) -> Result<BoardSyncOutcome, DomainViolation> {
        *self.last_transition.lock().unwrap() = Some((issue_number, target));
        Ok(BoardSyncOutcome::Synchronized { column: target })
    }

    fn create_pull_request(
        &self,
        _details: &PullRequestDetails,
    ) -> Result<PullRequestOutcome, DomainViolation> {
        Ok(PullRequestOutcome {
            pr_number: 101,
            pr_url: "https://github.com/org/repo/pull/101".to_string(),
            target_column: BoardColumn::InReview,
        })
    }
}

fn build_cycle(status: CycleStatus) -> SddCycle {
    SddCycle {
        cycle_id: "cycle-test-1".to_string(),
        issue_linkage: IssueLinkage::LinkedIssue {
            issue_number: 45,
            issue_url: "https://github.com/org/repo/issues/45".to_string(),
            project_board_id: "board-1".to_string(),
            project_item_id: "item-1".to_string(),
            board_column: BoardColumn::InProgress,
        },
        methodology_profile: MethodologyProfile::StandardGauntletV2,
        current_gate: GateStage::Gate6Certified,
        lifecycle_status: status,
        gauntlet_attempt: 0,
        spec_dir: "specs".to_string(),
        spec_file_path: "specs/spec.md".to_string(),
        spec_seal_state: SpecLockState::Frozen {
            spec_path: "p".into(),
            blake3_digest: "h".into(),
        },
        work_branch: "feat/45".to_string(),
        base_commit_sha: "sha".to_string(),
        pre_existing_dirty_files: vec![],
        created_at: "2026-11-12T12:00:00Z".to_string(),
        updated_at: "2026-11-12T12:00:00Z".to_string(),
    }
}

#[test]
fn test_close_cycle_nominal_success() {
    let repo = InMemoryTestRepo {
        cycle: Mutex::new(Some(build_cycle(CycleStatus::Certified))),
    };
    let board = MockBoardGateway {
        last_transition: Mutex::new(None),
    };

    let req = CloseCycleRequest {
        cycle_id: "cycle-test-1".to_string(),
        now_iso: "2026-11-12T12:00:00Z".to_string(),
    };

    let outcome =
        execute(req, &repo, &board).expect("close_cycle should succeed for certified cycle");
    assert_eq!(outcome.cycle_id, "cycle-test-1");
    assert_eq!(outcome.cycle_status, CycleStatus::Merged);
    assert_eq!(
        repo.cycle
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .lifecycle_status,
        CycleStatus::Merged
    );
    assert_eq!(
        *board.last_transition.lock().unwrap(),
        Some((45, BoardColumn::ReadyForTest))
    );
}

#[test]
fn test_close_cycle_rejects_uncertified_cycle() {
    let repo = InMemoryTestRepo {
        cycle: Mutex::new(Some(build_cycle(CycleStatus::Active))),
    };
    let board = MockBoardGateway {
        last_transition: Mutex::new(None),
    };

    let req = CloseCycleRequest {
        cycle_id: "cycle-test-1".to_string(),
        now_iso: "2026-11-12T12:00:00Z".to_string(),
    };

    let err = execute(req, &repo, &board).expect_err("close_cycle must reject uncertified cycle");
    match err {
        DomainViolation::InfrastructureBoundaryFault { subsystem, details } => {
            assert_eq!(subsystem, "close_gate");
            assert!(details.contains("Gate 6 certification required (RN-002)"));
        }
        _ => panic!("Expected InfrastructureBoundaryFault for close_gate"),
    }
}
