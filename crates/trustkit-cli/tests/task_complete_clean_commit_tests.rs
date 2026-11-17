// Tests for task completion clean commit verification (TASK-003 / RF-003 / RN-003).
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
use trustkit_core::use_cases::complete_task::{self, CompleteTaskRequest};
use trustkit_db::sqlite_adapter::SqliteWalRepositoryAdapter;

fn setup_clean_git_repo_with_task(
    dir: &std::path::Path,
    task_id: &str,
) -> (std::path::PathBuf, SqliteWalRepositoryAdapter, String) {
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

    fs::write(dir.join("README.md"), "# Clean Repo\n").expect("write README");
    fs::write(dir.join(".gitignore"), ".trustkit/\n*.db*\n").expect("write gitignore");
    Command::new("git")
        .args(["add", "README.md", ".gitignore"])
        .current_dir(dir)
        .output()
        .expect("git add");
    Command::new("git")
        .args(["commit", "-m", "chore: #0 initial setup"])
        .current_dir(dir)
        .output()
        .expect("git commit");

    // Task commit
    fs::write(dir.join("feature.txt"), "Completed feature logic\n").expect("write feature");
    Command::new("git")
        .args(["add", "feature.txt"])
        .current_dir(dir)
        .output()
        .expect("git add feature");
    Command::new("git")
        .args(["commit", "-m", "feat: #18 implement task 3 feature"])
        .current_dir(dir)
        .output()
        .expect("git commit feature");

    let head_output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(dir)
        .output()
        .expect("git rev-parse HEAD");
    let head_sha = String::from_utf8_lossy(&head_output.stdout)
        .trim()
        .to_string();

    let db_dir = dir.join(".trustkit");
    fs::create_dir_all(&db_dir).expect("create .trustkit");
    let db_path = db_dir.join("state.db");
    let repo = SqliteWalRepositoryAdapter::open(&db_path).expect("open db");

    let cycle = SddCycle {
        cycle_id: "cycle-task-003".to_string(),
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
        "Validacao de Commit Limpo".to_string(),
        "CLI".to_string(),
        Vec::new(),
        vec!["crates/trustkit-cli/src/commands/task_cmd.rs".to_string()],
        "crates/trustkit-cli/tests/task_complete_clean_commit_tests.rs".to_string(),
    );

    let schedule = WaveSchedule {
        cycle_id: cycle.cycle_id.clone(),
        total_tasks: 1,
        waves: vec![ExecutionWave {
            wave_index: 3,
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

    (db_path, repo, head_sha)
}

#[test]
fn test_task_complete_success_with_clean_working_tree_and_valid_commit() {
    let dir = tempdir().expect("tempdir");
    let (db_path, repo, head_sha) = setup_clean_git_repo_with_task(dir.path(), "TASK-003");

    let args = TaskCompleteArgs {
        task_id: "TASK-003".to_string(),
        code_review: None,
        json: true,
    };

    let result = task_cmd::execute_complete_with_db(&args, true, &db_path);
    assert!(
        result.is_ok(),
        "Expected task completion to succeed on clean repo"
    );

    let task = repo
        .find_atomic_task("cycle-task-003", "TASK-003")
        .expect("find task")
        .expect("task exists");

    assert!(task.tdd_lifecycle_state.is_committed());
    assert_eq!(task.commit_sha, head_sha);
}

#[test]
fn test_task_complete_rejected_with_invalid_or_zero_commit_sha() {
    let dir = tempdir().expect("tempdir");
    let (_, repo, _) = setup_clean_git_repo_with_task(dir.path(), "TASK-003");

    let req = CompleteTaskRequest {
        cycle_id: "cycle-task-003".to_string(),
        task_id: "TASK-003".to_string(),
        commit_sha: "0".repeat(40),
        now_iso: "2026-11-17T00:00:00Z".to_string(),
    };

    let result = complete_task::execute(req, &repo);

    // In RED phase: complete_task::execute currently does not validate non-zero commit_sha, so this fails.
    assert!(
        result.is_err(),
        "Expected complete_task::execute to reject zero commit_sha"
    );

    match result.err().unwrap() {
        DomainViolation::TddLifecycleViolation { task_id, reason } => {
            assert_eq!(task_id, "TASK-003");
            assert!(
                reason.contains("commit SHA") || reason.contains("RN-003"),
                "Expected reason to mention commit SHA/RN-003, got: {reason}"
            );
        }
        other => panic!("Expected TddLifecycleViolation, got: {other:?}"),
    }
}
