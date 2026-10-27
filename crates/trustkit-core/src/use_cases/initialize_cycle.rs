// initialize_cycle use case.
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL & Special Cases.
// Constitutional Principle VI: Ubiquitous Language & Focused Modular Clean Code (< 250 LOC).

use crate::entities::{
    CycleStatus, EvidenceKind, ExecutionEvidence, GateStage, GateVerdict,
    MethodologyProfile, SddCycle,
};
use crate::errors::DomainViolation;
use crate::ports::{CycleEvidenceRepositoryPort, ProjectBoardGatewayPort};
use crate::special_cases::{
    BoardColumn, BoardSyncOutcome, IdentityVerification, IssueLinkage, SpecLockState,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitializeCycleRequest {
    pub issue_param: Option<String>,
    pub from_prd: Option<String>,
    pub work_branch: String,
    pub base_commit_sha: String,
    pub pre_existing_dirty_files: Vec<String>,
    pub now_iso: String,
    pub timestamp_slug: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitializeCycleOutcome {
    pub cycle: SddCycle,
    pub sync_outcome: BoardSyncOutcome,
}

pub fn execute(
    req: InitializeCycleRequest,
    repo: &dyn CycleEvidenceRepositoryPort,
    board: &dyn ProjectBoardGatewayPort,
) -> Result<InitializeCycleOutcome, DomainViolation> {
    let linkage = resolve_linkage(&req.issue_param, &req.work_branch, &req.timestamp_slug);
    let (cycle_id, sync_outcome) = match &linkage {
        IssueLinkage::LinkedIssue { issue_number, .. } => {
            match board.verify_active_identity("marcos-rodoind")? {
                IdentityVerification::Verified { .. } => {}
                IdentityVerification::Rejected { detected_handle, expected_handle } => {
                    return Err(DomainViolation::InfrastructureBoundaryFault {
                        subsystem: "identity_guard".to_string(),
                        details: format!("Operator mismatch: '{detected_handle}', expected '{expected_handle}'"),
                    });
                }
                IdentityVerification::GhUnavailable { details } => {
                    return Err(DomainViolation::InfrastructureBoundaryFault {
                        subsystem: "identity_guard".to_string(),
                        details: format!("GitHub CLI unavailable: {details}"),
                    });
                }
            }
            let sync = board.transition_issue_column(*issue_number, BoardColumn::InProgress)?;
            (format!("issue-{issue_number}-{}", req.timestamp_slug), sync)
        }
        IssueLinkage::UnlinkedLocalCycle { .. } => (
            format!("no-issue-{}", req.timestamp_slug),
            BoardSyncOutcome::SkippedUnlinkedCycle,
        ),
    };

    let spec_dir = format!("docs/specs/{cycle_id}");
    let spec_file_path = req.from_prd.unwrap_or_else(|| format!("{spec_dir}/spec.md"));
    let cycle = SddCycle {
        cycle_id: cycle_id.clone(),
        issue_linkage: linkage,
        methodology_profile: MethodologyProfile::SpecKitIntegrated { feature_dir: spec_dir.clone() },
        current_gate: GateStage::Gate0Discovery,
        lifecycle_status: CycleStatus::Active,
        gauntlet_attempt: 0,
        spec_dir,
        spec_file_path,
        spec_seal_state: SpecLockState::DraftUnverified,
        work_branch: req.work_branch,
        base_commit_sha: req.base_commit_sha,
        pre_existing_dirty_files: req.pre_existing_dirty_files,
        created_at: req.now_iso.clone(),
        updated_at: req.now_iso.clone(),
    };

    repo.save_cycle(&cycle)?;
    let evidence = ExecutionEvidence {
        evidence_id: 0,
        cycle_id: cycle.cycle_id.clone(),
        gate_stage: GateStage::Gate0Discovery,
        gauntlet_attempt: 0,
        actor_role: "ORCHESTRATOR".to_string(),
        evidence_kind: match &sync_outcome {
            BoardSyncOutcome::Synchronized { .. } | BoardSyncOutcome::DeferredOffline { .. } => EvidenceKind::BoardSync,
            _ => EvidenceKind::CmdExec,
        },
        executed_command: "trustkit init".to_string(),
        exit_code: 0,
        duration_ms: 0,
        sanitized_stdout_tail: format!("Cycle initialized: {}", cycle.cycle_id),
        sanitized_stderr_tail: String::new(),
        output_blake3_digest: String::new(),
        verdict: GateVerdict::Pass,
        recorded_at: req.now_iso,
    };
    repo.append_evidence(&evidence)?;
    Ok(InitializeCycleOutcome { cycle, sync_outcome })
}

fn resolve_linkage(param: &Option<String>, branch: &str, ts: &str) -> IssueLinkage {
    if let Some((p, num)) = param.as_ref().and_then(|p| parse_num(p).map(|num| (p, num))) {
        let url = if p.starts_with("http") { p.clone() } else { format!("https://github.com/TibaldiHolding/TRustKit/issues/{num}") };
        return IssueLinkage::LinkedIssue {
            issue_number: num, issue_url: url,
            project_board_id: "default".to_string(), project_item_id: format!("item_{num}"),
            board_column: BoardColumn::InProgress,
        };
    }
    if let Some(num) = parse_branch_num(branch) {
        return IssueLinkage::LinkedIssue {
            issue_number: num, issue_url: format!("https://github.com/TibaldiHolding/TRustKit/issues/{num}"),
            project_board_id: "default".to_string(), project_item_id: format!("item_{num}"),
            board_column: BoardColumn::InProgress,
        };
    }
    IssueLinkage::UnlinkedLocalCycle { directory_slug: format!("no-issue-{ts}") }
}

fn parse_num(s: &str) -> Option<u64> {
    s.rfind("/issues/").map(|pos| &s[(pos + 8)..]).unwrap_or(s).trim().parse().ok()
}

fn parse_branch_num(b: &str) -> Option<u64> {
    let p: Vec<&str> = b.split('/').collect();
    if p.len() >= 2 && matches!(p[0], "feat" | "fix" | "chore") {
        p[1].split('-').next()?.parse().ok()
    } else if p.len() >= 3 {
        p[1].parse().ok()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::ArtifactSeal;
    use crate::special_cases::ActiveCycleLookup;
    use std::cell::RefCell;

    struct FakeRepo(RefCell<Vec<SddCycle>>, RefCell<Vec<ExecutionEvidence>>);
    impl CycleEvidenceRepositoryPort for FakeRepo {
        fn save_cycle(&self, c: &SddCycle) -> Result<(), DomainViolation> { self.0.borrow_mut().push(c.clone()); Ok(()) }
        fn find_active_cycle(&self) -> Result<ActiveCycleLookup, DomainViolation> { Ok(ActiveCycleLookup::NoActiveCycle) }
        fn save_artifact_seal(&self, _: &ArtifactSeal) -> Result<u64, DomainViolation> { Ok(1) }
        fn list_artifact_seals(&self, _: &str) -> Result<Vec<ArtifactSeal>, DomainViolation> { Ok(vec![]) }
        fn append_evidence(&self, e: &ExecutionEvidence) -> Result<u64, DomainViolation> { self.1.borrow_mut().push(e.clone()); Ok(1) }
        fn list_evidences_for_cycle(&self, _: &str) -> Result<Vec<ExecutionEvidence>, DomainViolation> { Ok(self.1.borrow().clone()) }
    }

    struct FakeBoard(IdentityVerification, BoardSyncOutcome);
    impl ProjectBoardGatewayPort for FakeBoard {
        fn verify_active_identity(&self, _: &str) -> Result<IdentityVerification, DomainViolation> { Ok(self.0.clone()) }
        fn transition_issue_column(&self, _: u64, _: BoardColumn) -> Result<BoardSyncOutcome, DomainViolation> { Ok(self.1.clone()) }
    }

    fn sample_req(issue: Option<&str>, prd: Option<&str>, branch: &str) -> InitializeCycleRequest {
        InitializeCycleRequest {
            issue_param: issue.map(String::from),
            from_prd: prd.map(String::from),
            work_branch: branch.to_string(),
            base_commit_sha: "a".repeat(40),
            pre_existing_dirty_files: vec!["dirty.txt".to_string()],
            now_iso: "2026-10-27T14:15:00Z".to_string(),
            timestamp_slug: "202610271415".to_string(),
        }
    }

    #[test]
    fn test_linked_issue_and_evidence() {
        let repo = FakeRepo(RefCell::new(vec![]), RefCell::new(vec![]));
        let board = FakeBoard(IdentityVerification::Verified { operator_handle: "marcos-rodoind".into() }, BoardSyncOutcome::Synchronized { column: BoardColumn::InProgress });
        let res = execute(sample_req(Some("45"), None, "feat/45-cli"), &repo, &board).unwrap();
        assert_eq!(res.cycle.cycle_id, "issue-45-202610271415");
        assert_eq!(res.cycle.spec_file_path, "docs/specs/issue-45-202610271415/spec.md");
        assert_eq!(repo.0.borrow().len(), 1);
        assert_eq!(repo.1.borrow().len(), 1);
    }

    #[test]
    fn test_unlinked_fallback_and_from_prd() {
        let repo = FakeRepo(RefCell::new(vec![]), RefCell::new(vec![]));
        let board = FakeBoard(IdentityVerification::Verified { operator_handle: "marcos-rodoind".into() }, BoardSyncOutcome::SkippedUnlinkedCycle);
        let res = execute(sample_req(None, Some("docs/PRD.md"), "main"), &repo, &board).unwrap();
        assert_eq!(res.cycle.cycle_id, "no-issue-202610271415");
        assert_eq!(res.cycle.spec_file_path, "docs/PRD.md");
        assert_eq!(res.sync_outcome, BoardSyncOutcome::SkippedUnlinkedCycle);
    }

    #[test]
    fn test_identity_rejection_and_offline_sync() {
        let repo = FakeRepo(RefCell::new(vec![]), RefCell::new(vec![]));
        let rej = FakeBoard(IdentityVerification::Rejected { detected_handle: "other".into(), expected_handle: "marcos-rodoind".into() }, BoardSyncOutcome::SkippedUnlinkedCycle);
        assert!(execute(sample_req(Some("45"), None, "feat/45"), &repo, &rej).is_err());

        let off = FakeBoard(IdentityVerification::Verified { operator_handle: "marcos-rodoind".into() }, BoardSyncOutcome::DeferredOffline { queued_transition: BoardColumn::InProgress, retry_after_ms: 5000 });
        let res = execute(sample_req(Some("45"), None, "feat/45"), &repo, &off).unwrap();
        assert!(matches!(res.sync_outcome, BoardSyncOutcome::DeferredOffline { .. }));
    }
}
