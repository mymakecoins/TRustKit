// Integration tests for SQLite WAL repository adapter.
// Constitutional Principle II: Absolute Prohibition of NULL & Special Cases.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use super::*;
use std::sync::Arc;
use std::thread;
use trustkit_core::entities::{
    ArtifactCategory, ArtifactSeal, CycleStatus, EvidenceKind, ExecutionEvidence,
    GateStage, GateVerdict, MethodologyProfile, SddCycle,
};
use trustkit_core::special_cases::{
    ContractUnlockState, IssueLinkage, SpecLockState,
};

fn create_test_adapter(name: &str) -> SqliteWalRepositoryAdapter {
    let temp_dir = std::env::temp_dir().join(format!(
        "trustkit_test_{}_{}",
        name,
        std::time::SystemTime::now().elapsed().unwrap_or_default().as_nanos()
    ));
    let db_path = temp_dir.join("state.db");
    SqliteWalRepositoryAdapter::open(&db_path).expect("open test db")
}

#[test]
fn test_schema_initialization_all_tables_exist() {
    let adapter = create_test_adapter("schema");
    let conn = adapter.conn.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
        .unwrap();
    let tables: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    assert!(tables.contains(&"sdd_cycles".to_string()));
    assert!(tables.contains(&"artifact_seals".to_string()));
    assert!(tables.contains(&"traceability_links".to_string()));
    assert!(tables.contains(&"atomic_tasks".to_string()));
    assert!(tables.contains(&"execution_evidences".to_string()));
}

#[test]
fn test_not_null_and_check_constraints() {
    let adapter = create_test_adapter("constraints");
    let conn = adapter.conn.lock().unwrap();

    // Direct NULL insertion into NOT NULL column must fail
    let null_res = conn.execute(
        "INSERT INTO sdd_cycles (cycle_id, issue_linkage_kind, methodology_profile, current_gate, lifecycle_status, spec_dir, spec_file_path, work_branch, base_commit_sha, created_at, updated_at)
         VALUES (NULL, 'UNLINKED_LOCAL_CYCLE', 'STANDARD_GAUNTLET_V2', 0, 'ACTIVE', 'dir', 'spec.md', 'main', '00', 'now', 'now')",
        [],
    );
    assert!(null_res.is_err());

    // Check constraint on invalid lifecycle_status must fail
    let check_res = conn.execute(
        "INSERT INTO sdd_cycles (cycle_id, issue_linkage_kind, methodology_profile, current_gate, lifecycle_status, spec_dir, spec_file_path, work_branch, base_commit_sha, created_at, updated_at)
         VALUES ('c1', 'UNLINKED_LOCAL_CYCLE', 'STANDARD_GAUNTLET_V2', 0, 'INVALID_STATUS', 'dir', 'spec.md', 'main', '00', 'now', 'now')",
        [],
    );
    assert!(check_res.is_err());
}

#[test]
fn test_artifact_seal_roundtrip() {
    let adapter = create_test_adapter("seal");
    let cycle = SddCycle {
        cycle_id: "issue-10-202610271500".to_string(),
        issue_linkage: IssueLinkage::UnlinkedLocalCycle {
            directory_slug: "issue-10-202610271500".to_string(),
        },
        methodology_profile: MethodologyProfile::StandardGauntletV2,
        current_gate: GateStage::Gate2SpecSealed,
        lifecycle_status: CycleStatus::Active,
        gauntlet_attempt: 0,
        spec_dir: "specs/010".to_string(),
        spec_file_path: "specs/010/spec.md".to_string(),
        spec_seal_state: SpecLockState::DraftUnverified,
        work_branch: "feat/10".to_string(),
        base_commit_sha: "a".repeat(40),
        pre_existing_dirty_files: vec![],
        created_at: "2026-10-27T15:00:00Z".to_string(),
        updated_at: "2026-10-27T15:00:00Z".to_string(),
    };
    adapter.save_cycle(&cycle).unwrap();

    let seal1 = ArtifactSeal {
        seal_id: 0,
        cycle_id: "issue-10-202610271500".to_string(),
        file_path: "specs/010/spec.md".to_string(),
        artifact_category: ArtifactCategory::Spec,
        blake3_digest: "1".repeat(64),
        ast_assertion_count: 12,
        contract_unlock_state: ContractUnlockState::Sealed,
    };
    let seal2 = ArtifactSeal {
        seal_id: 0,
        cycle_id: "issue-10-202610271500".to_string(),
        file_path: "specs/010/constitution.md".to_string(),
        artifact_category: ArtifactCategory::Constitution,
        blake3_digest: "2".repeat(64),
        ast_assertion_count: 0,
        contract_unlock_state: ContractUnlockState::UnlockedByHuman {
            justification: "Emergency hotfix".to_string(),
        },
    };

    adapter.save_artifact_seal(&seal1).unwrap();
    adapter.save_artifact_seal(&seal2).unwrap();

    let seals = adapter.list_artifact_seals("issue-10-202610271500").unwrap();
    assert_eq!(seals.len(), 2);
    assert_eq!(seals[0].file_path, "specs/010/spec.md");
    assert_eq!(seals[0].contract_unlock_state, ContractUnlockState::Sealed);
    assert_eq!(seals[1].file_path, "specs/010/constitution.md");
    assert_eq!(
        seals[1].contract_unlock_state,
        ContractUnlockState::UnlockedByHuman {
            justification: "Emergency hotfix".to_string(),
        }
    );
}

#[test]
fn test_concurrent_evidence_writes() {
    let adapter = Arc::new(create_test_adapter("concurrent"));
    let cycle = SddCycle {
        cycle_id: "conc-202610271500".to_string(),
        issue_linkage: IssueLinkage::UnlinkedLocalCycle {
            directory_slug: "conc-202610271500".to_string(),
        },
        methodology_profile: MethodologyProfile::StandardGauntletV2,
        current_gate: GateStage::Gate4TasksImplemented,
        lifecycle_status: CycleStatus::Active,
        gauntlet_attempt: 1,
        spec_dir: "specs/conc".to_string(),
        spec_file_path: "specs/conc/spec.md".to_string(),
        spec_seal_state: SpecLockState::DraftUnverified,
        work_branch: "feat/conc".to_string(),
        base_commit_sha: "b".repeat(40),
        pre_existing_dirty_files: vec![],
        created_at: "2026-10-27T15:00:00Z".to_string(),
        updated_at: "2026-10-27T15:00:00Z".to_string(),
    };
    adapter.save_cycle(&cycle).unwrap();

    let mut handles = Vec::new();
    for t in 0..5 {
        let ad = Arc::clone(&adapter);
        handles.push(thread::spawn(move || {
            for i in 0..20 {
                let ev = ExecutionEvidence {
                    evidence_id: 0,
                    cycle_id: "conc-202610271500".to_string(),
                    gate_stage: GateStage::Gate4TasksImplemented,
                    gauntlet_attempt: 1,
                    actor_role: format!("WORKER_{t}"),
                    evidence_kind: EvidenceKind::CmdExec,
                    executed_command: format!("cargo check --worker {t} --step {i}"),
                    exit_code: 0,
                    duration_ms: 10,
                    sanitized_stdout_tail: "ok".to_string(),
                    sanitized_stderr_tail: String::new(),
                    output_blake3_digest: "0".repeat(64),
                    verdict: GateVerdict::Pass,
                    recorded_at: "2026-10-27T15:00:00Z".to_string(),
                };
                ad.append_evidence(&ev).unwrap();
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    let evidences = adapter.list_evidences_for_cycle("conc-202610271500").unwrap();
    assert_eq!(evidences.len(), 100);
}
