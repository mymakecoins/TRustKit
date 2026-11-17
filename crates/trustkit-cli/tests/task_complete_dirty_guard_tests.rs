// Tests for task completion dirty working tree guard (TASK-001 / RF-001 / RN-001).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use std::fs;
use std::process::Command;
use tempfile::tempdir;
use trustkit_cli::cli_contract::TaskCompleteArgs;
use trustkit_cli::commands::task_cmd;
use trustkit_core::entities::{
    CycleStatus, ExecutionWave, GateStage, MethodologyProfile, SddCycle, TaskNode, WaveSchedule,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::CycleEvidenceRepositoryPort;
use trustkit_core::special_cases::{BoardColumn, IssueLinkage, SpecLockState, TddLifecycleState};
use trustkit_db::sqlite_adapter::SqliteWalRepositoryAdapter;

fn setup_git_repo_with_task(
    dir: &std::path::Path,
    task_id: &str,
) -> (std::path::PathBuf, SqliteWalRepositoryAdapter) {
    Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(dir)
        .output()
        .expect("git init");
    Command::new("git")
        .args(["config", "user.name", "Tester"])
        .current_dir(dir)
        .output()
        .expect("git config user.name");
    Command::new("git")
        .args(["config", "user.email", "tester@test.local"])
        .current_dir(dir)
        .output()
        .expect("git config user.email");

    fs::write(dir.join("README.md"), "# Test Repo\n").expect("write README");
    Command::new("git")
        .args(["add", "README.md"])
        .current_dir(dir)
        .output()
        .expect("git add");
    Command::new("git")
        .args(["commit", "-m", "chore: #0 initial setup"])
        .current_dir(dir)
        .output()
        .expect("git commit");

    let db_path = dir.join("state.db");
    let repo = SqliteWalRepositoryAdapter::open(&db_path).expect("open db");

    let cycle = SddCycle {
        cycle_id: "cycle-task-001".to_string(),
        issue_linkage: IssueLinkage::LinkedIssue {
            issue_number: 18,
            issue_url: "https://github.com/TibaldiHolding/TRustKit/issues/18".to_string(),
            project_board_id: "board-18".to_string(),
            project_item_id: "item-18".to_string(),
            board_column: BoardColumn::InProgress,
        },
        methodology_profile: MethodologyProfile::StandardGauntletV2,
        current_gate: GateStage::Gate4TasksImplemented,
        lifecycle_status: CycleStatus::Active,
        gauntlet_attempt: 0,
        spec_dir: dir.to_str().unwrap().to_string(),
        spec_file_path: dir.join("spec.md").to_str().unwrap().to_string(),
        spec_seal_state: SpecLockState::Frozen {
            spec_path: dir.join("spec.md").to_str().unwrap().to_string(),
            blake3_digest: "0".repeat(64),
        },
        work_branch: "main".to_string(),
        base_commit_sha: "0".repeat(40),
        pre_existing_dirty_files: Vec::new(),
        created_at: "2026-11-17T00:00:00Z".to_string(),
        updated_at: "2026-11-17T00:00:00Z".to_string(),
    };
    repo.save_cycle(&cycle).expect("save cycle");

    let task = TaskNode::new(
        task_id.to_string(),
        "Auditoria de Commits".to_string(),
        "CLI".to_string(),
        Vec::new(),
        vec!["crates/trustkit-cli/src/commands/task_cmd.rs".to_string()],
        "crates/trustkit-cli/tests/task_complete_dirty_guard_tests.rs".to_string(),
    );

    let schedule = WaveSchedule {
        cycle_id: cycle.cycle_id.clone(),
        total_tasks: 1,
        waves: vec![ExecutionWave {
            wave_index: 1,
            tasks: vec![task.clone()],
        }],
    };
    repo.save_atomic_tasks(&cycle.cycle_id, &[task], &schedule)
        .expect("save tasks");

    repo.update_task_tdd_phase(
        &cycle.cycle_id,
        task_id,
        &TddLifecycleState::GreenVerified {
            red_evidence_id: 1,
            green_evidence_id: 2,
        },
    )
    .expect("update tdd phase");

    (db_path, repo)
}

#[test]
fn test_task_complete_rejected_when_working_tree_has_dirty_files() {
    let dir = tempdir().expect("tempdir");
    let (db_path, repo) = setup_git_repo_with_task(dir.path(), "TASK-001");

    fs::write(dir.path().join("dirty_uncommitted_file.txt"), "uncommitted work\n")
        .expect("write uncommitted file");

    let args = TaskCompleteArgs {
        task_id: "TASK-001".to_string(),
        code_review: None,
        json: true,
    };

    let result = task_cmd::execute_complete_with_db(&args, true, &db_path);

    assert!(
        result.is_err(),
        "Expected execute_complete_with_db to fail when working tree is dirty"
    );

    match result.err().unwrap() {
        DomainViolation::TddLifecycleViolation { task_id, reason } => {
            assert_eq!(task_id, "TASK-001");
            assert!(
                reason.contains("uncommitted") || reason.contains("RN-001") || reason.contains("dirty"),
                "Expected reason to mention uncommitted/RN-001, got: {reason}"
            );
        }
        other => panic!("Expected TddLifecycleViolation, got: {other:?}"),
    }

    let task = repo
        .find_atomic_task("cycle-task-001", "TASK-001")
        .expect("find task")
        .expect("task exists");
    assert!(!task.tdd_lifecycle_state.is_committed());
}
