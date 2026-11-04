// Unit tests for Certification, Merge and Pull Request Use Cases (TASK-006).
// Constitutional Principle III: Test-First & Anti-Cheat.

use std::sync::Mutex;
use trustkit_core::entities::{
    ArtifactSeal, CertificationSummary, CriticKind, CycleStatus, EvidenceKind, ExecutionEvidence,
    GateStage, GateVerdict, MethodologyProfile, SddCycle, SpecificationV2, TaskNode, WaveSchedule,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::{
    CycleCertificationPort, CycleEvidenceRepositoryPort, ProjectBoardGatewayPort,
    PullRequestDetails, PullRequestOutcome,
};
use trustkit_core::special_cases::{
    ActiveCycleLookup, BoardColumn, BoardSyncOutcome, IdentityVerification, IssueLinkage,
    SpecLockState,
};
use trustkit_core::use_cases::certify_cycle_and_report::{self, CertifyCycleRequest};
use trustkit_core::use_cases::merge_cycle::{self, MergeCycleRequest};
use trustkit_core::use_cases::open_pull_request::{self, OpenPullRequestRequest};

struct InMemoryCertRepo {
    cycle: Mutex<Option<SddCycle>>,
    evidences: Mutex<Vec<ExecutionEvidence>>,
}

impl CycleEvidenceRepositoryPort for InMemoryCertRepo {
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

    fn list_certification_evidences(
        &self,
        _cycle_id: &str,
    ) -> Result<Vec<ExecutionEvidence>, DomainViolation> {
        Ok(self.evidences.lock().unwrap().clone())
    }
}

struct MockCertPort;

impl CycleCertificationPort for MockCertPort {
    fn render_gauntlet_report(
        &self,
        summary: &CertificationSummary,
        _evidences: &[ExecutionEvidence],
    ) -> Result<String, DomainViolation> {
        assert!(!summary.cycle_id.is_empty());
        Ok(format!("# Gauntlet Report for {}", summary.cycle_id))
    }

    fn format_issue_completion_comment(
        &self,
        summary: &CertificationSummary,
        _spec: &SpecificationV2,
    ) -> Result<String, DomainViolation> {
        Ok(format!(
            "## Cycle {} Certified Successfully",
            summary.cycle_id
        ))
    }
}

struct MockBoardGateway {
    last_pr: Mutex<Option<PullRequestDetails>>,
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
        details: &PullRequestDetails,
    ) -> Result<PullRequestOutcome, DomainViolation> {
        *self.last_pr.lock().unwrap() = Some(details.clone());
        Ok(PullRequestOutcome {
            pr_number: 101,
            pr_url: "https://github.com/org/repo/pull/101".to_string(),
            target_column: BoardColumn::InReview,
        })
    }
}

fn build_cycle(status: CycleStatus) -> SddCycle {
    SddCycle {
        cycle_id: "cycle-cert-1".to_string(),
        issue_linkage: IssueLinkage::LinkedIssue {
            issue_number: 45,
            issue_url: "https://github.com/org/repo/issues/45".to_string(),
            project_board_id: "board-1".to_string(),
            project_item_id: "item-1".to_string(),
            board_column: BoardColumn::InProgress,
        },
        methodology_profile: MethodologyProfile::StandardGauntletV2,
        current_gate: GateStage::Gate5GauntletCleared,
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
        created_at: "2026-10-28T12:00:00Z".to_string(),
        updated_at: "2026-10-28T12:00:00Z".to_string(),
    }
}

fn make_passed_evidences() -> Vec<ExecutionEvidence> {
    let mut list = vec![ExecutionEvidence {
        evidence_id: 1,
        cycle_id: "cycle-cert-1".to_string(),
        gate_stage: GateStage::Gate5GauntletCleared,
        gauntlet_attempt: 0,
        actor_role: "AUDITOR:AST_LAYER0".to_string(),
        evidence_kind: EvidenceKind::AstAudit,
        executed_command: "ast-audit".to_string(),
        exit_code: 0,
        duration_ms: 5,
        sanitized_stdout_tail: "".to_string(),
        sanitized_stderr_tail: "".to_string(),
        output_blake3_digest: "".to_string(),
        verdict: GateVerdict::Pass,
        recorded_at: "2026-10-28T12:00:00Z".to_string(),
    }];

    let critics = [
        CriticKind::DomainAdversarial,
        CriticKind::ContractCompliance,
        CriticKind::StaticSecurity,
        CriticKind::PlaywrightVisual,
    ];

    for (idx, kind) in critics.iter().enumerate() {
        list.push(ExecutionEvidence {
            evidence_id: (idx + 2) as u64,
            cycle_id: "cycle-cert-1".to_string(),
            gate_stage: GateStage::Gate5GauntletCleared,
            gauntlet_attempt: 0,
            actor_role: format!("CRITIC:{}", kind.as_str()),
            evidence_kind: EvidenceKind::CriticVerdict,
            executed_command: "critic run".to_string(),
            exit_code: 0,
            duration_ms: 10,
            sanitized_stdout_tail: "".to_string(),
            sanitized_stderr_tail: "".to_string(),
            output_blake3_digest: "".to_string(),
            verdict: GateVerdict::Pass,
            recorded_at: "2026-10-28T12:00:00Z".to_string(),
        });
    }

    list
}

fn sample_spec() -> SpecificationV2 {
    SpecificationV2 {
        cycle_id: "cycle-cert-1".to_string(),
        feature_slug: "feature".to_string(),
        spec_file_path: "specs/spec.md".to_string(),
        business_rules: vec![],
        functional_requirements: vec![],
        mermaid_content: "".to_string(),
        gherkin_scenarios: vec![],
        traceability_matrix: vec![],
        lock_state: SpecLockState::DraftUnverified,
    }
}

#[test]
fn test_certify_cycle_and_report_success() {
    let repo = InMemoryCertRepo {
        cycle: Mutex::new(Some(build_cycle(CycleStatus::Active))),
        evidences: Mutex::new(make_passed_evidences()),
    };
    let cert_port = MockCertPort;

    let req = CertifyCycleRequest {
        cycle_id: "cycle-cert-1".to_string(),
        spec: sample_spec(),
        now_iso: "2026-10-28T12:00:00Z".to_string(),
    };

    let outcome = certify_cycle_and_report::execute(req, &repo, &cert_port)
        .expect("certification should succeed");

    assert!(!outcome.report_markdown.contains("null"));
    assert!(!outcome.completion_comment.contains("null"));
    assert_eq!(
        repo.cycle
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .lifecycle_status,
        CycleStatus::Certified
    );
    assert_eq!(
        repo.cycle.lock().unwrap().as_ref().unwrap().current_gate,
        GateStage::Gate6Certified
    );
}

#[test]
fn test_merge_cycle_fails_when_not_certified() {
    let repo = InMemoryCertRepo {
        cycle: Mutex::new(Some(build_cycle(CycleStatus::Active))),
        evidences: Mutex::new(vec![]),
    };
    let board = MockBoardGateway {
        last_pr: Mutex::new(None),
        last_transition: Mutex::new(None),
    };

    let req = MergeCycleRequest {
        cycle_id: "cycle-cert-1".to_string(),
        target_branch: "develop".to_string(),
        now_iso: "2026-10-28T12:00:00Z".to_string(),
    };

    let res = merge_cycle::execute(req, &repo, &board);
    assert!(matches!(
        res,
        Err(DomainViolation::InfrastructureBoundaryFault { .. })
    ));
}

#[test]
fn test_merge_cycle_succeeds_when_certified() {
    let repo = InMemoryCertRepo {
        cycle: Mutex::new(Some(build_cycle(CycleStatus::Certified))),
        evidences: Mutex::new(vec![]),
    };
    let board = MockBoardGateway {
        last_pr: Mutex::new(None),
        last_transition: Mutex::new(None),
    };

    let req = MergeCycleRequest {
        cycle_id: "cycle-cert-1".to_string(),
        target_branch: "develop".to_string(),
        now_iso: "2026-10-28T12:00:00Z".to_string(),
    };

    let outcome = merge_cycle::execute(req, &repo, &board).expect("merge should succeed");
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
fn test_open_pull_request_success_with_reviewers_assignees_and_ref() {
    let repo = InMemoryCertRepo {
        cycle: Mutex::new(Some(build_cycle(CycleStatus::Certified))),
        evidences: Mutex::new(vec![]),
    };
    let board = MockBoardGateway {
        last_pr: Mutex::new(None),
        last_transition: Mutex::new(None),
    };

    let req = OpenPullRequestRequest {
        cycle_id: "cycle-cert-1".to_string(),
        human_authorized: true,
        base_branch: "develop".to_string(),
        head_branch: "feat/45".to_string(),
        title: "feat: #45 implementa worktrees".to_string(),
        custom_body: None,
        now_iso: "2026-10-28T12:00:00Z".to_string(),
    };

    let outcome =
        open_pull_request::execute(req, &repo, &board).expect("pr creation should succeed");
    assert_eq!(outcome.pr_number, 101);
    assert_eq!(outcome.target_column, BoardColumn::InReview);

    let pr_details = board.last_pr.lock().unwrap().clone().unwrap();
    assert_eq!(pr_details.reviewers, Vec::<String>::new());
    assert_eq!(pr_details.assignees, Vec::<String>::new());
    assert!(pr_details.body.contains("Ref #45"));
}

#[test]
fn test_open_pull_request_forbids_auto_closing_keywords() {
    let repo = InMemoryCertRepo {
        cycle: Mutex::new(Some(build_cycle(CycleStatus::Certified))),
        evidences: Mutex::new(vec![]),
    };
    let board = MockBoardGateway {
        last_pr: Mutex::new(None),
        last_transition: Mutex::new(None),
    };

    let req = OpenPullRequestRequest {
        cycle_id: "cycle-cert-1".to_string(),
        human_authorized: true,
        base_branch: "develop".to_string(),
        head_branch: "feat/45".to_string(),
        title: "feat: #45".to_string(),
        custom_body: Some("This PR Closes #45 automatically".to_string()),
        now_iso: "2026-10-28T12:00:00Z".to_string(),
    };

    let res = open_pull_request::execute(req, &repo, &board);
    match res {
        Err(DomainViolation::AstIntegrityOrInvariantViolation { invariant_id, .. }) => {
            assert_eq!(invariant_id, "RN-012");
        }
        other => panic!("Expected InvariantViolation for RN-012, got: {other:?}"),
    }
}

#[test]
fn test_open_pull_request_requires_human_authorization() {
    let repo = InMemoryCertRepo {
        cycle: Mutex::new(Some(build_cycle(CycleStatus::Certified))),
        evidences: Mutex::new(vec![]),
    };
    let board = MockBoardGateway {
        last_pr: Mutex::new(None),
        last_transition: Mutex::new(None),
    };

    let req = OpenPullRequestRequest {
        cycle_id: "cycle-cert-1".to_string(),
        human_authorized: false,
        base_branch: "develop".to_string(),
        head_branch: "feat/45".to_string(),
        title: "feat: #45".to_string(),
        custom_body: None,
        now_iso: "2026-10-28T12:00:00Z".to_string(),
    };

    let res = open_pull_request::execute(req, &repo, &board);
    assert!(matches!(
        res,
        Err(DomainViolation::InfrastructureBoundaryFault { .. })
    ));
}
