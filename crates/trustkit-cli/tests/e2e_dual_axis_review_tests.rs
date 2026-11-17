// E2E Integration tests for Gate 4 Dual-Axis Code Review & SQLite WAL Persistence (TASK-005, RN-005, RF-005).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use tempfile::tempdir;
use trustkit_cli::e2e_dual_axis_review::DualAxisGate4Reviewer;
use trustkit_core::entities::{
    CycleStatus, DualAxisFinding, DualAxisFindingKind, DualAxisReviewReport, EvidenceKind,
    GateStage, GateVerdict, MethodologyProfile, SddCycle, SpecComplianceDefect,
};
use trustkit_core::ports::CycleEvidenceRepositoryPort;
use trustkit_core::special_cases::{IssueLinkage, SpecLockState};
use trustkit_db::sqlite_adapter::SqliteWalRepositoryAdapter;

fn setup_test_cycle(cycle_id: &str) -> (tempfile::TempDir, SqliteWalRepositoryAdapter) {
    let dir = tempdir().expect("create tempdir");
    let db_path = dir.path().join("state.db");
    let repo = SqliteWalRepositoryAdapter::open(&db_path).expect("open sqlite db");

    let cycle = SddCycle {
        cycle_id: cycle_id.to_string(),
        issue_linkage: IssueLinkage::UnlinkedLocalCycle {
            directory_slug: cycle_id.to_string(),
        },
        methodology_profile: MethodologyProfile::StandardGauntletV2,
        current_gate: GateStage::Gate4TasksImplemented,
        lifecycle_status: CycleStatus::Active,
        gauntlet_attempt: 0,
        spec_dir: dir.path().to_str().unwrap().to_string(),
        spec_file_path: dir.path().join("spec.md").to_str().unwrap().to_string(),
        spec_seal_state: SpecLockState::DraftUnverified,
        work_branch: "develop".to_string(),
        base_commit_sha: "0".repeat(40),
        pre_existing_dirty_files: vec![],
        created_at: "2026-11-12T20:00:00Z".to_string(),
        updated_at: "2026-11-12T20:00:00Z".to_string(),
    };
    repo.save_cycle(&cycle).expect("save cycle");
    (dir, repo)
}

#[test]
fn test_gate4_dual_axis_review_persistence_pass() {
    let (_dir, repo) = setup_test_cycle("cycle-dual-pass");
    let report = DualAxisReviewReport::new("TASK-005".to_string(), vec![], vec![]);

    let evidence_id = DualAxisGate4Reviewer::execute_and_record(
        "cycle-dual-pass",
        "TASK-005",
        &repo,
        &report,
        "2026-11-12T21:00:00Z",
    )
    .expect("record dual axis evidence");

    assert!(
        evidence_id > 0,
        "Evidence ID must be strictly greater than 0"
    );

    let evidences = repo
        .list_evidences_for_cycle("cycle-dual-pass")
        .expect("list evidences");

    assert_eq!(evidences.len(), 1);
    let ev = &evidences[0];
    assert_eq!(ev.evidence_kind, EvidenceKind::CodeReview);
    assert_eq!(ev.verdict, GateVerdict::Pass);
    assert_eq!(ev.exit_code, 0);
    assert!(ev.sanitized_stdout_tail.contains("STANDARDS_AND_SMELLS"));
    assert!(ev.sanitized_stdout_tail.contains("SPEC_COMPLIANCE"));
}

#[test]
fn test_gate4_dual_axis_review_persistence_fail_when_either_axis_fails() {
    let (_dir, repo) = setup_test_cycle("cycle-dual-fail");
    let spec_finding = DualAxisFinding {
        file: "crates/trustkit-cli/src/lib.rs".to_string(),
        line: 10,
        kind: DualAxisFindingKind::SpecCompliance {
            defect: SpecComplianceDefect::ScopeCreep,
            requirement_reference: "RF-005".to_string(),
            details: "Scope creep detected in CLI".to_string(),
        },
        summary: "Scope creep in CLI".to_string(),
        recommendation: "Remove unspec code".to_string(),
    };

    let report = DualAxisReviewReport::new("TASK-005".to_string(), vec![], vec![spec_finding]);

    let evidence_id = DualAxisGate4Reviewer::execute_and_record(
        "cycle-dual-fail",
        "TASK-005",
        &repo,
        &report,
        "2026-11-12T21:00:00Z",
    )
    .expect("record dual axis evidence");

    assert!(
        evidence_id > 0,
        "Evidence ID must be strictly greater than 0"
    );

    let evidences = repo
        .list_evidences_for_cycle("cycle-dual-fail")
        .expect("list evidences");

    assert_eq!(evidences.len(), 1);
    let ev = &evidences[0];
    assert_eq!(ev.evidence_kind, EvidenceKind::CodeReview);
    assert_eq!(ev.verdict, GateVerdict::Fail);
    assert_eq!(ev.exit_code, 5);
}
