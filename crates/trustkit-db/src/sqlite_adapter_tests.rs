// Integration tests for SQLite WAL repository adapter.
// Constitutional Principle II: Absolute Prohibition of NULL & Special Cases.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use super::*;
use std::sync::Arc;
use std::thread;
use trustkit_core::entities::{
    ArtifactCategory, ArtifactSeal, CycleStatus, EvidenceKind, ExecutionEvidence, ExecutionWave,
    GateStage, GateVerdict, MethodologyProfile, SddCycle, TaskNode, WaveSchedule,
};
use trustkit_core::special_cases::{
    ActiveCycleLookup, ContractUnlockState, IssueLinkage, SpecLockState,
};

fn create_test_adapter(name: &str) -> SqliteWalRepositoryAdapter {
    let temp_dir = std::env::temp_dir().join(format!(
        "trustkit_test_{}_{}",
        name,
        std::time::SystemTime::now()
            .elapsed()
            .unwrap_or_default()
            .as_nanos()
    ));
    SqliteWalRepositoryAdapter::open(&temp_dir.join("state.db")).expect("open test db")
}

fn mock_cycle(id: &str, gate: GateStage) -> SddCycle {
    SddCycle {
        cycle_id: id.to_string(),
        issue_linkage: IssueLinkage::UnlinkedLocalCycle {
            directory_slug: id.to_string(),
        },
        methodology_profile: MethodologyProfile::StandardGauntletV2,
        current_gate: gate,
        lifecycle_status: CycleStatus::Active,
        gauntlet_attempt: 0,
        spec_dir: format!("specs/{id}"),
        spec_file_path: format!("specs/{id}/spec.md"),
        spec_seal_state: SpecLockState::DraftUnverified,
        work_branch: "develop".to_string(),
        base_commit_sha: "0".repeat(40),
        pre_existing_dirty_files: vec![],
        created_at: "2026-10-27T15:00:00Z".to_string(),
        updated_at: "2026-10-27T15:00:00Z".to_string(),
    }
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
    let null_res = conn.execute(
        "INSERT INTO sdd_cycles (cycle_id, issue_linkage_kind, methodology_profile, current_gate, lifecycle_status, spec_dir, spec_file_path, work_branch, base_commit_sha, created_at, updated_at)
         VALUES (NULL, 'UNLINKED_LOCAL_CYCLE', 'STANDARD_GAUNTLET_V2', 0, 'ACTIVE', 'dir', 'spec.md', 'main', '00', 'now', 'now')", [],
    );
    assert!(null_res.is_err());
    let check_res = conn.execute(
        "INSERT INTO sdd_cycles (cycle_id, issue_linkage_kind, methodology_profile, current_gate, lifecycle_status, spec_dir, spec_file_path, work_branch, base_commit_sha, created_at, updated_at)
         VALUES ('c1', 'UNLINKED_LOCAL_CYCLE', 'STANDARD_GAUNTLET_V2', 0, 'INVALID_STATUS', 'dir', 'spec.md', 'main', '00', 'now', 'now')", [],
    );
    assert!(check_res.is_err());
}

#[test]
fn test_artifact_seal_roundtrip() {
    let adapter = create_test_adapter("seal");
    adapter
        .save_cycle(&mock_cycle("issue-10", GateStage::Gate2SpecSealed))
        .unwrap();

    let seal1 = ArtifactSeal {
        seal_id: 0,
        cycle_id: "issue-10".to_string(),
        file_path: "specs/010/spec.md".to_string(),
        artifact_category: ArtifactCategory::Spec,
        blake3_digest: "1".repeat(64),
        ast_assertion_count: 12,
        contract_unlock_state: ContractUnlockState::Sealed,
    };
    let seal2 = ArtifactSeal {
        seal_id: 0,
        cycle_id: "issue-10".to_string(),
        file_path: "specs/010/const.md".to_string(),
        artifact_category: ArtifactCategory::Constitution,
        blake3_digest: "2".repeat(64),
        ast_assertion_count: 0,
        contract_unlock_state: ContractUnlockState::UnlockedByHuman {
            justification: "Emergency fix".to_string(),
        },
    };
    adapter.save_artifact_seal(&seal1).unwrap();
    adapter.save_artifact_seal(&seal2).unwrap();

    let seals = adapter.list_artifact_seals("issue-10").unwrap();
    assert_eq!(seals.len(), 2);
    assert_eq!(seals[0].file_path, "specs/010/spec.md");
    assert_eq!(seals[0].contract_unlock_state, ContractUnlockState::Sealed);
    assert_eq!(
        seals[1].contract_unlock_state,
        ContractUnlockState::UnlockedByHuman {
            justification: "Emergency fix".to_string()
        }
    );
}

#[test]
fn test_concurrent_evidence_writes() {
    let adapter = Arc::new(create_test_adapter("concurrent"));
    adapter
        .save_cycle(&mock_cycle("conc-cycle", GateStage::Gate4TasksImplemented))
        .unwrap();

    let mut handles = Vec::new();
    for t in 0..5 {
        let ad = Arc::clone(&adapter);
        handles.push(thread::spawn(move || {
            for i in 0..20 {
                let ev = ExecutionEvidence {
                    evidence_id: 0,
                    cycle_id: "conc-cycle".to_string(),
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
    let evidences = adapter.list_evidences_for_cycle("conc-cycle").unwrap();
    assert_eq!(evidences.len(), 100);
}

#[test]
fn test_update_cycle_spec_lock_advances_gate_2() {
    let adapter = create_test_adapter("spec_lock");
    adapter
        .save_cycle(&mock_cycle("lock-test-cycle", GateStage::Gate1SpecDrafting))
        .unwrap();

    let digest = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    adapter
        .update_cycle_spec_lock("lock-test-cycle", digest, "FROZEN")
        .unwrap();

    let lookup = adapter.find_active_cycle().unwrap();
    match lookup {
        ActiveCycleLookup::Found(active) => {
            assert_eq!(active.current_gate, GateStage::Gate2SpecSealed);
            assert_eq!(
                active.spec_seal_state,
                SpecLockState::Frozen {
                    spec_path: "specs/lock-test-cycle/spec.md".to_string(),
                    blake3_digest: digest.to_string(),
                }
            );
        }
        _ => panic!("Expected active cycle found"),
    }
}

#[test]
fn test_save_atomic_tasks_advances_gate_3() {
    let adapter = create_test_adapter("atomic_tasks");
    adapter
        .save_cycle(&mock_cycle("wave-test-cycle", GateStage::Gate2SpecSealed))
        .unwrap();

    let t1 = TaskNode {
        task_id: "TASK-001".to_string(),
        title: "Domain entity".to_string(),
        architectural_layer: "CORE".to_string(),
        depends_on: vec![],
        expected_files: vec!["crates/trustkit-core/src/entities.rs".to_string()],
        target_test_file: "tests/entity.rs".to_string(),
    };
    let t2 = TaskNode {
        task_id: "TASK-002".to_string(),
        title: "Database adapter".to_string(),
        architectural_layer: "DB".to_string(),
        depends_on: vec!["TASK-001".to_string()],
        expected_files: vec!["crates/trustkit-db/src/sqlite_adapter.rs".to_string()],
        target_test_file: "tests/db.rs".to_string(),
    };
    let schedule = WaveSchedule {
        cycle_id: "wave-test-cycle".to_string(),
        total_tasks: 2,
        waves: vec![
            ExecutionWave {
                wave_index: 1,
                tasks: vec![t1.clone()],
            },
            ExecutionWave {
                wave_index: 2,
                tasks: vec![t2.clone()],
            },
        ],
    };

    adapter
        .save_atomic_tasks("wave-test-cycle", &[t1, t2], &schedule)
        .unwrap();
    let tasks = adapter.list_atomic_tasks("wave-test-cycle").unwrap();
    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0].task_id, "TASK-001");
    assert_eq!(tasks[0].wave_index, 1);
    assert_eq!(tasks[1].task_id, "TASK-002");
    assert_eq!(tasks[1].wave_index, 2);

    match adapter.find_active_cycle().unwrap() {
        ActiveCycleLookup::Found(active) => {
            assert_eq!(active.current_gate, GateStage::Gate3WavePlanned)
        }
        _ => panic!("Expected active cycle at gate 3"),
    }
}
