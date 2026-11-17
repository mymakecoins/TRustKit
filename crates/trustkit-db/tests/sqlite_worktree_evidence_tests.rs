// SQLite Worktree Evidence Integration Tests (TASK-002 / RF-002 / RN-002).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use tempfile::tempdir;
use trustkit_core::entities::{
    CycleStatus, ExecutionEvidence, ExecutionWave, GateStage, GateVerdict, MethodologyProfile,
    SddCycle, SpecLockState, TaskNode, WaveSchedule,
};
use trustkit_core::ports::CycleEvidenceRepositoryPort;
use trustkit_core::special_cases::{IssueLinkage, WorktreeAllocation};
use trustkit_db::SqliteWalRepositoryAdapter;

fn setup_test_cycle_and_task(repo: &SqliteWalRepositoryAdapter, cycle_id: &str, task_id: &str) {
    let cycle = SddCycle {
        cycle_id: cycle_id.to_string(),
        issue_linkage: IssueLinkage::UnlinkedLocalCycle {
            directory_slug: cycle_id.to_string(),
        },
        methodology_profile: MethodologyProfile::StandardGauntletV2,
        current_gate: GateStage::Gate4TasksImplemented,
        lifecycle_status: CycleStatus::Active,
        gauntlet_attempt: 0,
        spec_dir: format!("specs/{cycle_id}"),
        spec_file_path: format!("specs/{cycle_id}/spec.md"),
        spec_seal_state: SpecLockState::Frozen {
            spec_path: format!("specs/{cycle_id}/spec.md"),
            blake3_digest: "0".repeat(64),
        },
        work_branch: "develop".to_string(),
        base_commit_sha: "0".repeat(40),
        pre_existing_dirty_files: vec![],
        created_at: "2026-11-17T10:00:00Z".to_string(),
        updated_at: "2026-11-17T10:00:00Z".to_string(),
    };
    repo.save_cycle(&cycle).expect("save cycle");

    let task = TaskNode::new(
        task_id.to_string(),
        "Persistencia de Evidencias de Worktree no SQLite".to_string(),
        "DB".to_string(),
        vec!["TASK-001".to_string()],
        vec![
            "crates/trustkit-core/src/entities.rs".to_string(),
            "crates/trustkit-db/src/sqlite_adapter.rs".to_string(),
        ],
        "crates/trustkit-db/tests/sqlite_worktree_evidence_tests.rs".to_string(),
    );
    let schedule = WaveSchedule {
        cycle_id: cycle_id.to_string(),
        total_tasks: 1,
        waves: vec![ExecutionWave {
            wave_index: 2,
            tasks: vec![task.clone()],
        }],
    };
    repo.save_atomic_tasks(cycle_id, &[task], &schedule)
        .expect("save atomic tasks");
}

#[test]
fn test_sqlite_worktree_evidence_persistence() {
    let dir = tempdir().expect("tempdir");
    let db_path = dir.path().join("state.db");
    let repo = SqliteWalRepositoryAdapter::open(&db_path).expect("open repo");
    let cycle_id = "cycle-wt-002";
    let task_id = "TASK-002";

    setup_test_cycle_and_task(&repo, cycle_id, task_id);

    // 1. Update task worktree allocation
    let allocation = WorktreeAllocation::Provisioned {
        worktree_path: ".trustkit/worktrees/TASK-002".to_string(),
        branch_name: "worktree/TASK-002".to_string(),
        base_branch: "develop".to_string(),
    };
    repo.update_task_worktree(cycle_id, task_id, &allocation)
        .expect("update task worktree");

    // 2. Append worktree provision evidence
    let evidence = ExecutionEvidence::for_worktree_provision(
        cycle_id,
        task_id,
        "worktree/TASK-002",
        ".trustkit/worktrees/TASK-002",
        "develop",
        "2026-11-17T10:05:00Z",
    );
    let evidence_id = repo.append_evidence(&evidence).expect("append evidence");
    assert!(evidence_id > 0);

    // 3. Query and verify evidence
    let found = repo
        .find_worktree_evidence(cycle_id, task_id)
        .expect("find worktree evidence")
        .expect("evidence must be present");

    assert_eq!(found.actor_role, "ORCH:WORKTREE_TASK-002");
    assert_eq!(found.verdict, GateVerdict::Pass);
    assert_eq!(found.exit_code, 0);
    assert!(
        found
            .executed_command
            .contains("git worktree add -B worktree/TASK-002 .trustkit/worktrees/TASK-002 develop")
    );
    assert!(
        found
            .sanitized_stdout_tail
            .contains("[INFO] Branch 'worktree/TASK-002' criada com sucesso.")
    );
    assert!(
        found.sanitized_stdout_tail.contains(
            "[INFO] Checkout realizado no diretorio isolado: .trustkit/worktrees/TASK-002"
        )
    );
    assert!(
        found
            .sanitized_stdout_tail
            .contains("[INFO] Branch ativa do repositorio raiz preservada: 'develop'")
    );

    // 4. Verify task state in SQLite
    let task = repo
        .find_atomic_task(cycle_id, task_id)
        .expect("find task")
        .expect("task exists");
    assert_eq!(
        task.worktree_allocation.branch_name(),
        Some("worktree/TASK-002")
    );
    assert_eq!(
        task.worktree_allocation.worktree_path(),
        Some(".trustkit/worktrees/TASK-002")
    );
    assert_eq!(task.worktree_allocation.base_branch(), Some("develop"));
}
