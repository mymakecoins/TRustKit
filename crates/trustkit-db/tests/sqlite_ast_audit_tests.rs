// Unit tests for SQLite AST audit and artifact seal persistence (TASK-010).
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle II: Absolute Prohibition of NULL.

use tempfile::tempdir;
use trustkit_core::entities::{
    ArtifactCategory, ArtifactSeal, CycleStatus, EvidenceKind, ExecutionEvidence, GateStage,
    GateVerdict, MethodologyProfile, SddCycle,
};
use trustkit_core::ports::CycleEvidenceRepositoryPort;
use trustkit_core::special_cases::{ContractUnlockState, IssueLinkage, SpecLockState};
use trustkit_db::sqlite_adapter::SqliteWalRepositoryAdapter;

fn insert_test_cycle(repo: &SqliteWalRepositoryAdapter, id: &str) {
    let cycle = SddCycle {
        cycle_id: id.to_string(),
        issue_linkage: IssueLinkage::UnlinkedLocalCycle {
            directory_slug: id.to_string(),
        },
        methodology_profile: MethodologyProfile::StandardGauntletV2,
        current_gate: GateStage::Gate0Discovery,
        lifecycle_status: CycleStatus::Active,
        gauntlet_attempt: 0,
        spec_dir: format!("specs/{id}"),
        spec_file_path: format!("specs/{id}/spec.md"),
        spec_seal_state: SpecLockState::DraftUnverified,
        work_branch: "develop".to_string(),
        base_commit_sha: "0".repeat(40),
        pre_existing_dirty_files: vec![],
        created_at: "2026-10-28T12:00:00Z".to_string(),
        updated_at: "2026-10-28T12:00:00Z".to_string(),
    };
    repo.save_cycle(&cycle).expect("save cycle");
}

#[test]
fn test_persist_ast_audit_evidence_roundtrip() {
    let dir = tempdir().expect("create temp dir");
    let db_path = dir.path().join("state.db");
    let repo = SqliteWalRepositoryAdapter::open(&db_path).expect("open db");
    insert_test_cycle(&repo, "cycle-ast-1");
    insert_test_cycle(&repo, "cycle-ast-1");

    let evidence = ExecutionEvidence {
        evidence_id: 0,
        cycle_id: "cycle-ast-1".to_string(),
        gate_stage: GateStage::Gate5GauntletCleared,
        gauntlet_attempt: 0,
        actor_role: "AUDITOR:AST_LAYER0".to_string(),
        evidence_kind: EvidenceKind::AstAudit,
        executed_command: "trustkit gauntlet ast-audit".to_string(),
        exit_code: 0,
        duration_ms: 3,
        sanitized_stdout_tail: "AstAuditVerdict::CleanPass".to_string(),
        sanitized_stderr_tail: "".to_string(),
        output_blake3_digest: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
            .to_string(),
        verdict: GateVerdict::Pass,
        recorded_at: "2026-10-28T12:00:00Z".to_string(),
    };

    let id = repo.append_evidence(&evidence).expect("append evidence");
    assert!(id > 0);

    let evidences = repo
        .list_evidences_for_cycle("cycle-ast-1")
        .expect("list evidences");
    assert_eq!(evidences.len(), 1);
    assert_eq!(evidences[0].evidence_kind, EvidenceKind::AstAudit);
    assert_eq!(evidences[0].verdict, GateVerdict::Pass);
    assert_eq!(evidences[0].exit_code, 0);
}

#[test]
fn test_persist_ast_assertion_count_in_artifact_seal() {
    let dir = tempdir().expect("create temp dir");
    let db_path = dir.path().join("state.db");
    let repo = SqliteWalRepositoryAdapter::open(&db_path).expect("open db");
    insert_test_cycle(&repo, "cycle-ast-1");

    let seal = ArtifactSeal {
        seal_id: 0,
        cycle_id: "cycle-ast-1".to_string(),
        file_path: "tests/auth_test.rs".to_string(),
        artifact_category: ArtifactCategory::PreExistingTest,
        blake3_digest: "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210"
            .to_string(),
        ast_assertion_count: 42,
        contract_unlock_state: ContractUnlockState::Sealed,
    };

    let id = repo.save_artifact_seal(&seal).expect("save seal");
    assert!(id > 0);

    let seals = repo
        .list_artifact_seals("cycle-ast-1")
        .expect("list artifact seals");
    assert_eq!(seals.len(), 1);
    assert_eq!(seals[0].ast_assertion_count, 42);
    assert_eq!(seals[0].file_path, "tests/auth_test.rs");
}
