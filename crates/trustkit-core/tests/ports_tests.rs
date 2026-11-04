// Unit tests for domain ports (TASK-002).
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle I: Zero External Dependencies.

use trustkit_core::entities::{
    AstAuditVerdict, CertificationSummary, CircuitBreakerState, ContextSliceRequest,
    ContextSliceResult, CycleStatus, ExecutionEvidence, GateStage, SpecificationV2, TaskNode,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::{
    AstAuditorPort, AsyncExecutionSupervisorPort, ContextSlicerPort, CycleCertificationPort,
    CycleEvidenceRepositoryPort, ForeignScreenshotInspection, ProjectBoardGatewayPort,
    PullRequestDetails, PullRequestOutcome, SupervisedExecutionOutcome, SupervisedExecutionRequest,
    VisualEvidencePort, WorktreeIsolationPort, WorktreeStatus,
};
use trustkit_core::special_cases::{
    ActiveCycleLookup, BoardColumn, BoardSyncOutcome, IdentityVerification, SpecLockState,
    TddLifecycleState, WorktreeAllocation,
};

struct MockAstAuditor;

impl AstAuditorPort for MockAstAuditor {
    fn audit_source_tree(
        &self,
        _files: &[String],
        _reference_seals: &[trustkit_core::entities::ArtifactSeal],
    ) -> Result<AstAuditVerdict, DomainViolation> {
        Ok(AstAuditVerdict::CleanPass { violations: vec![] })
    }
}

struct MockContextSlicer;

impl ContextSlicerPort for MockContextSlicer {
    fn slice_context(
        &self,
        request: &ContextSliceRequest,
    ) -> Result<ContextSliceResult, DomainViolation> {
        Ok(ContextSliceResult {
            sliced_content: "pub fn mock() {}".to_string(),
            preserved_symbols: request.symbol_filters.clone(),
            total_lines: 1,
        })
    }
}

struct MockWorktreeIsolation;

impl WorktreeIsolationPort for MockWorktreeIsolation {
    fn provision_worktree(
        &self,
        _cycle_id: &str,
        task_id: &str,
        _base_branch: &str,
    ) -> Result<WorktreeAllocation, DomainViolation> {
        Ok(WorktreeAllocation::Provisioned {
            worktree_path: format!(".trustkit/worktrees/{task_id}"),
            branch_name: format!("task/{task_id}"),
        })
    }

    fn check_worktree_status(
        &self,
        worktree_path: &str,
    ) -> Result<WorktreeStatus, DomainViolation> {
        Ok(WorktreeStatus {
            worktree_path: worktree_path.to_string(),
            branch_name: "task/TASK-001".to_string(),
            is_clean: true,
            head_commit_sha: "abcdef1234567890".to_string(),
        })
    }

    fn merge_and_prune_worktree(
        &self,
        _task_id: &str,
        _worktree_path: &str,
        _target_branch: &str,
    ) -> Result<String, DomainViolation> {
        Ok("commit_sha_merged_123".to_string())
    }
}

struct MockExecutionSupervisor;

impl AsyncExecutionSupervisorPort for MockExecutionSupervisor {
    fn execute_supervised(
        &self,
        request: &SupervisedExecutionRequest,
    ) -> Result<SupervisedExecutionOutcome, DomainViolation> {
        Ok(SupervisedExecutionOutcome {
            exit_code: 0,
            duration_ms: 42,
            stdout_tail: format!("executed: {}", request.command),
            stderr_tail: String::new(),
            output_blake3_digest: "blake3_mock".to_string(),
        })
    }
}

struct MockVisualEvidenceGuard;

impl VisualEvidencePort for MockVisualEvidenceGuard {
    fn restore_foreign_screenshots(
        &self,
        active_issue_prefix: &str,
        _evidence_dir: &str,
    ) -> Result<ForeignScreenshotInspection, DomainViolation> {
        Ok(ForeignScreenshotInspection {
            active_issue_prefix: active_issue_prefix.to_string(),
            foreign_screenshots_detected: vec!["issue-12-screen.png".to_string()],
            restored_screenshots: vec!["issue-12-screen.png".to_string()],
        })
    }
}

struct MockCycleCertification;

impl CycleCertificationPort for MockCycleCertification {
    fn render_gauntlet_report(
        &self,
        summary: &CertificationSummary,
        _evidences: &[ExecutionEvidence],
    ) -> Result<String, DomainViolation> {
        Ok(format!(
            "# Certification Report for {}\nFully certified: {}",
            summary.cycle_id,
            summary.is_fully_certified()
        ))
    }

    fn format_issue_completion_comment(
        &self,
        summary: &CertificationSummary,
        spec: &SpecificationV2,
    ) -> Result<String, DomainViolation> {
        Ok(format!(
            "Cycle {} for {} completed successfully.",
            summary.cycle_id, spec.feature_slug
        ))
    }
}

struct MockProjectBoard;

impl ProjectBoardGatewayPort for MockProjectBoard {
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
        _issue_number: u64,
        target: BoardColumn,
    ) -> Result<BoardSyncOutcome, DomainViolation> {
        Ok(BoardSyncOutcome::Synchronized { column: target })
    }

    fn create_pull_request(
        &self,
        details: &PullRequestDetails,
    ) -> Result<PullRequestOutcome, DomainViolation> {
        Ok(PullRequestOutcome {
            pr_number: details.issue_number,
            pr_url: format!("https://github.com/org/repo/pull/{}", details.issue_number),
            target_column: BoardColumn::InReview,
        })
    }
}

struct MockCycleEvidenceRepo;

impl CycleEvidenceRepositoryPort for MockCycleEvidenceRepo {
    fn save_cycle(
        &self,
        _cycle: &trustkit_core::entities::SddCycle,
    ) -> Result<(), DomainViolation> {
        Ok(())
    }
    fn find_active_cycle(&self) -> Result<ActiveCycleLookup, DomainViolation> {
        Ok(ActiveCycleLookup::NoActiveCycle)
    }
    fn save_artifact_seal(
        &self,
        _seal: &trustkit_core::entities::ArtifactSeal,
    ) -> Result<u64, DomainViolation> {
        Ok(1)
    }
    fn list_artifact_seals(
        &self,
        _cycle_id: &str,
    ) -> Result<Vec<trustkit_core::entities::ArtifactSeal>, DomainViolation> {
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
        _wave_schedule: &trustkit_core::entities::WaveSchedule,
    ) -> Result<(), DomainViolation> {
        Ok(())
    }
}

#[test]
fn test_mock_ast_auditor_port() {
    let auditor = MockAstAuditor;
    let verdict = auditor
        .audit_source_tree(&["src/lib.rs".to_string()], &[])
        .expect("mock audit should succeed");
    assert!(verdict.is_pass());
}

#[test]
fn test_mock_context_slicer_port() {
    let slicer = MockContextSlicer;
    let req = ContextSliceRequest {
        target_files: vec!["src/lib.rs".to_string()],
        symbol_filters: vec!["mock".to_string()],
        max_line_budget: 50,
    };
    let res = slicer
        .slice_context(&req)
        .expect("mock slice should succeed");
    assert_eq!(res.preserved_symbols, vec!["mock".to_string()]);
    assert_eq!(res.total_lines, 1);
}

#[test]
fn test_mock_worktree_isolation_port() {
    let wt = MockWorktreeIsolation;
    let alloc = wt
        .provision_worktree("cycle-1", "TASK-001", "develop")
        .expect("provision worktree");
    match alloc {
        WorktreeAllocation::Provisioned {
            worktree_path,
            branch_name,
        } => {
            assert_eq!(worktree_path, ".trustkit/worktrees/TASK-001");
            assert_eq!(branch_name, "task/TASK-001");
        }
        _ => panic!("Expected provisioned worktree"),
    }

    let status = wt
        .check_worktree_status(".trustkit/worktrees/TASK-001")
        .expect("check status");
    assert!(status.is_clean);
    assert_eq!(status.head_commit_sha, "abcdef1234567890");

    let sha = wt
        .merge_and_prune_worktree("TASK-001", ".trustkit/worktrees/TASK-001", "develop")
        .expect("merge and prune");
    assert_eq!(sha, "commit_sha_merged_123");
}

#[test]
fn test_mock_async_execution_supervisor_port() {
    let supervisor = MockExecutionSupervisor;
    let outcome = supervisor
        .execute_supervised(&SupervisedExecutionRequest {
            command: "cargo test".to_string(),
            args: vec!["--lib".to_string()],
            working_dir: ".".to_string(),
            timeout_ms: 10000,
        })
        .expect("execute supervised");
    assert_eq!(outcome.exit_code, 0);
    assert_eq!(outcome.duration_ms, 42);
    assert!(outcome.stdout_tail.contains("cargo test"));
}

#[test]
fn test_mock_visual_evidence_port() {
    let guard = MockVisualEvidenceGuard;
    let inspection = guard
        .restore_foreign_screenshots("issue-45", "evidence/")
        .expect("restore foreign screenshots");
    assert_eq!(inspection.active_issue_prefix, "issue-45");
    assert_eq!(inspection.restored_screenshots, vec!["issue-12-screen.png"]);
}

#[test]
fn test_mock_cycle_certification_port() {
    let cert = MockCycleCertification;
    let summary = CertificationSummary {
        cycle_id: "cycle-45".to_string(),
        layer0_clean_pass: true,
        critic_findings: vec![],
        total_evidences_recorded: 10,
        circuit_breaker_state: CircuitBreakerState::Closed,
        certified_at: "2026-10-28T18:00:00Z".to_string(),
    };
    let report = cert
        .render_gauntlet_report(&summary, &[])
        .expect("render report");
    assert!(report.contains("Certification Report for cycle-45"));

    let spec = SpecificationV2 {
        cycle_id: "cycle-45".to_string(),
        feature_slug: "004-worktree-tdd".to_string(),
        spec_file_path: "specs/004/spec.md".to_string(),
        business_rules: vec![],
        functional_requirements: vec![],
        mermaid_content: String::new(),
        gherkin_scenarios: vec![],
        traceability_matrix: vec![],
        lock_state: SpecLockState::DraftUnverified,
    };
    let comment = cert
        .format_issue_completion_comment(&summary, &spec)
        .expect("format comment");
    assert!(comment.contains("Cycle cycle-45 for 004-worktree-tdd completed successfully"));
}

#[test]
fn test_mock_project_board_create_pr() {
    let board = MockProjectBoard;
    let pr = board
        .create_pull_request(&PullRequestDetails {
            issue_number: 45,
            title: "feat: #45 worktree isolation".to_string(),
            body: "PR details".to_string(),
            base_branch: "develop".to_string(),
            head_branch: "feat/45".to_string(),
            reviewers: vec!["reviewer1".to_string()],
            assignees: vec!["dev1".to_string()],
        })
        .expect("create pr");
    assert_eq!(pr.pr_number, 45);
    assert_eq!(pr.target_column, BoardColumn::InReview);
}

#[test]
fn test_mock_cycle_evidence_repo_defaults() {
    let repo = MockCycleEvidenceRepo;
    assert!(
        repo.update_task_tdd_phase("c", "t", &TddLifecycleState::AwaitingRedPhase)
            .is_ok()
    );
    assert!(
        repo.update_task_worktree("c", "t", &WorktreeAllocation::NotProvisioned)
            .is_ok()
    );
    assert!(repo.find_atomic_task("c", "t").unwrap().is_none());
    assert!(repo.list_atomic_tasks("c").unwrap().is_empty());
    assert!(
        repo.update_cycle_status("c", CycleStatus::Active, 0)
            .is_ok()
    );
    assert!(
        repo.advance_cycle_gate("c", GateStage::Gate1SpecDrafting)
            .is_ok()
    );
    assert!(repo.list_certification_evidences("c").unwrap().is_empty());
}
