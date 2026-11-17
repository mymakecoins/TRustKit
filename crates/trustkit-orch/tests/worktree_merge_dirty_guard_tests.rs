// Worktree merge dirty guard tests (TASK-002 / RF-002 / RN-002).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;
use trustkit_core::ports::WorktreeIsolationPort;
use trustkit_core::special_cases::WorktreeAllocation;
use trustkit_orch::WorktreeIsolationProvider;

fn setup_test_git_repo(initial_branch: &str) -> tempfile::TempDir {
    let dir = tempdir().expect("tempdir");
    let path = dir.path();

    Command::new("git")
        .args(["init", "-b", initial_branch])
        .current_dir(path)
        .output()
        .expect("git init");
    Command::new("git")
        .args(["config", "user.name", "Tester"])
        .current_dir(path)
        .output()
        .expect("git config user.name");
    Command::new("git")
        .args(["config", "user.email", "tester@test.local"])
        .current_dir(path)
        .output()
        .expect("git config user.email");

    fs::write(path.join("README.md"), "# Test Repo\n").expect("write README");
    Command::new("git")
        .args(["add", "README.md"])
        .current_dir(path)
        .output()
        .expect("git add");
    Command::new("git")
        .args(["commit", "-m", "chore: #0 initial repository setup"])
        .current_dir(path)
        .output()
        .expect("git commit");

    dir
}

#[test]
fn test_merge_and_prune_rejected_when_worktree_is_dirty() {
    let repo_dir = setup_test_git_repo("main");
    let provider = WorktreeIsolationProvider::new(repo_dir.path());

    let alloc = provider
        .provision_worktree("cycle-1", "TASK-002", "main")
        .expect("provision task");
    let wt_path = match alloc {
        WorktreeAllocation::Provisioned { worktree_path, .. } => worktree_path,
        _ => panic!("Expected Provisioned allocation for TASK-002"),
    };

    fs::write(
        Path::new(&wt_path).join("dirty_worktree_file.txt"),
        "uncommitted worktree file\n",
    )
    .expect("write dirty file");

    let result = provider.merge_and_prune_worktree("TASK-002", &wt_path, "main");

    assert!(
        result.is_err(),
        "Expected merge_and_prune_worktree to fail when worktree has uncommitted files"
    );

    let err_str = format!("{:?}", result.err().unwrap());
    assert!(
        err_str.contains("uncommitted") || err_str.contains("RN-002") || err_str.contains("dirty"),
        "Expected error to mention uncommitted/RN-002/dirty, got: {err_str}"
    );
}
