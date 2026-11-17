// E2E Task Lifecycle and Commit Integrity Integration Tests (TASK-004 / RF-004 / RN-004).
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
use trustkit_core::ports::CycleEvidenceRepositoryPort;
use trustkit_core::special_cases::{BoardColumn, IssueLinkage, SpecLockState, TddLifecycleState};
use trustkit_db::sqlite_adapter::SqliteWalRepositoryAdapter;

fn setup_e2e_repository(dir: &std::path::Path) -> (std::path::PathBuf, SqliteWalRepositoryAdapter) {
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

    fs::write(dir.join("README.md"), "# E2E Repo\n").expect("write README");
    fs::write(dir.join(".gitignore"), ".trustkit/\n*.db*\n").expect("write gitignore");
    Command::new("git")
        .args(["add", "README.md", ".gitignore"])
        .current_dir(dir)
        .output()
        .expect("git add");
    Command::new("git")
        .args(["commit", "-m", "chore: #0 initial commit"])
        .current_dir(dir)
        .output()
        .expect("git commit");

    let db_dir = dir.join(".trustkit");
    fs::create_dir_all(&db_dir).expect("create .trustkit");
    let db_path = db_dir.join("state.db");
    let repo = SqliteWalRepositoryAdapter::open(&db_path).expect("open db");

    let cycle = SddCycle {
        cycle_id: "cycle-e2e-integrity".to_string(),
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
        "TASK-004".to_string(),
        "Integridade E2E".to_string(),
        "CLI".to_string(),
        Vec::new(),
        vec!["crates/trustkit-cli/tests/e2e_task_lifecycle_commit_integrity_tests.rs".to_string()],
        "crates/trustkit-cli/tests/e2e_task_lifecycle_commit_integrity_tests.rs".to_string(),
    );

    let schedule = WaveSchedule {
        cycle_id: cycle.cycle_id.clone(),
        total_tasks: 1,
        waves: vec![ExecutionWave {
            wave_index: 4,
            tasks: vec![task.clone()],
        }],
    };
    repo.save_atomic_tasks(&cycle.cycle_id, &[task], &schedule)
        .expect("save tasks");

    repo.update_task_tdd_phase(
        &cycle.cycle_id,
        "TASK-004",
        &TddLifecycleState::GreenVerified {
            red_evidence_id: 1,
            green_evidence_id: 2,
        },
    )
    .expect("update tdd phase");

    (db_path, repo)
}

#[test]
fn test_e2e_task_lifecycle_commit_integrity_full_cycle() {
    let dir = tempdir().expect("tempdir");
    let (db_path, repo) = setup_e2e_repository(dir.path());

    // 1. Verify dirty working tree is rejected
    fs::write(dir.path().join("dirty.txt"), "uncommitted\n").expect("write dirty");
    let args = TaskCompleteArgs {
        task_id: "TASK-004".to_string(),
        code_review: None,
        json: true,
    };
    let dirty_result = task_cmd::execute_complete_with_db(&args, true, &db_path);
    assert!(dirty_result.is_err(), "Dirty tree must be rejected");

    // 2. Commit the change properly
    Command::new("git")
        .args(["add", "dirty.txt"])
        .current_dir(dir.path())
        .output()
        .expect("git add");
    Command::new("git")
        .args(["commit", "-m", "feat: #18 commit dirty file clean"])
        .current_dir(dir.path())
        .output()
        .expect("git commit");

    let head_output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(dir.path())
        .output()
        .expect("git rev-parse HEAD");
    let expected_sha = String::from_utf8_lossy(&head_output.stdout)
        .trim()
        .to_string();

    // 3. Complete task cleanly
    let clean_result = task_cmd::execute_complete_with_db(&args, true, &db_path);
    assert!(clean_result.is_ok(), "Clean tree must succeed");

    let task = repo
        .find_atomic_task("cycle-e2e-integrity", "TASK-004")
        .expect("find task")
        .expect("task exists");
    assert!(task.tdd_lifecycle_state.is_committed());
    assert_eq!(task.commit_sha, expected_sha);

    // 4. Verify working tree remains completely clean
    let status_output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(dir.path())
        .output()
        .expect("git status");
    let status_str = String::from_utf8_lossy(&status_output.stdout);
    assert!(
        status_str.trim().is_empty(),
        "Working tree must be clean without orphan files"
    );
    assert!(!expected_sha.is_empty(), "Valid commit SHA must be recorded");
}
