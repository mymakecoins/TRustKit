use tempfile::tempdir;
use trustkit_core::entities::{
    CycleStatus, EvidenceKind, ExecutionEvidence, GateStage, GateVerdict, MethodologyProfile,
    SddCycle, SecurityReviewVerdict, SecuritySkipReason,
};
use trustkit_core::ports::CycleEvidenceRepositoryPort;
use trustkit_core::special_cases::{IssueLinkage, SpecLockState};
use trustkit_db::sqlite_adapter::SqliteWalRepositoryAdapter;

#[test]
fn test_sqlite_persists_code_review_and_security_review_evidences_without_null() {
    let dir = tempdir().expect("tempdir");
    let db_path = dir.path().join("state.db");
    let repo = SqliteWalRepositoryAdapter::open(&db_path).expect("open db");

    let cycle = SddCycle {
        cycle_id: "cycle-sec-review".to_string(),
        issue_linkage: IssueLinkage::UnlinkedLocalCycle {
            directory_slug: "cycle-sec-review".to_string(),
        },
        methodology_profile: MethodologyProfile::StandardGauntletV2,
        current_gate: GateStage::Gate5GauntletCleared,
        lifecycle_status: CycleStatus::Active,
        gauntlet_attempt: 0,
        spec_dir: dir.path().to_str().unwrap().to_string(),
        spec_file_path: dir.path().join("spec.md").to_str().unwrap().to_string(),
        spec_seal_state: SpecLockState::DraftUnverified,
        work_branch: "feat/7-sec".to_string(),
        base_commit_sha: "0".repeat(40),
        pre_existing_dirty_files: vec![],
        created_at: "2026-10-30T10:00:00Z".to_string(),
        updated_at: "2026-10-30T10:00:00Z".to_string(),
    };
    repo.save_cycle(&cycle).expect("save cycle");

    let code_ev = ExecutionEvidence {
        evidence_id: 0,
        cycle_id: "cycle-sec-review".to_string(),
        gate_stage: GateStage::Gate4TasksImplemented,
        gauntlet_attempt: 0,
        actor_role: "AUDITOR:CODE_REVIEW".to_string(),
        evidence_kind: EvidenceKind::CodeReview,
        executed_command: "trustkit task review TASK-003 --effort MEDIUM".to_string(),
        exit_code: 0,
        duration_ms: 10,
        sanitized_stdout_tail: "CodeReview::Pass".to_string(),
        sanitized_stderr_tail: "".to_string(),
        output_blake3_digest: "a".repeat(64),
        verdict: GateVerdict::Pass,
        recorded_at: "2026-10-30T10:01:00Z".to_string(),
    };
    let code_id = repo.append_evidence(&code_ev).expect("append code review");
    assert_eq!(code_id, 1);

    let sec_ev = ExecutionEvidence {
        evidence_id: 0,
        cycle_id: "cycle-sec-review".to_string(),
        gate_stage: GateStage::Gate5GauntletCleared,
        gauntlet_attempt: 0,
        actor_role: "CRITIC:SECURITY_REVIEW".to_string(),
        evidence_kind: EvidenceKind::SecurityReview,
        executed_command: "trustkit security-review --skip (DEVELOPER_PROMPT_OPT_OUT)".to_string(),
        exit_code: 0,
        duration_ms: 1,
        sanitized_stdout_tail: "SecurityReviewVerdict::SkippedByDeveloper (DEVELOPER_PROMPT_OPT_OUT)"
            .to_string(),
        sanitized_stderr_tail: "".to_string(),
        output_blake3_digest: "b".repeat(64),
        verdict: GateVerdict::Pass,
        recorded_at: "2026-10-30T10:02:00Z".to_string(),
    };
    let sec_id = repo
        .append_evidence(&sec_ev)
        .expect("append security review");
    assert_eq!(sec_id, 2);

    let loaded = repo
        .list_evidences_for_cycle("cycle-sec-review")
        .expect("list evidences");
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0].evidence_kind, EvidenceKind::CodeReview);
    assert_eq!(loaded[0].actor_role, "AUDITOR:CODE_REVIEW");
    assert_eq!(loaded[1].evidence_kind, EvidenceKind::SecurityReview);
    assert_eq!(loaded[1].actor_role, "CRITIC:SECURITY_REVIEW");

    let verdict = SecurityReviewVerdict::SkippedByDeveloper {
        reason: SecuritySkipReason::DeveloperPromptOptOut,
    };
    assert_eq!(
        format!("- **Specialized Security Review:** {}", verdict.report_status_line()),
        "- **Specialized Security Review:** SKIPPED (Opt-out pelo desenvolvedor)"
    );
}
