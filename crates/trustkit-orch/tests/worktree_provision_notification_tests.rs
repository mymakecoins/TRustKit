// Worktree Provision Notification Integration Tests (TASK-001 / RF-001 / RN-001).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle IV: Zero Panic.
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
fn test_worktree_provision_structured_details_and_root_preservation() {
    let repo_dir = setup_test_git_repo("develop");
    let provider = WorktreeIsolationProvider::new(repo_dir.path());

    let alloc = provider
        .provision_worktree("cycle-1", "TASK-001", "develop")
        .expect("provision worktree");

    assert!(alloc.is_provisioned());
    match &alloc {
        WorktreeAllocation::Provisioned {
            worktree_path,
            branch_name,
            base_branch,
        } => {
            assert!(worktree_path.contains(".trustkit/worktrees/TASK-001"));
            assert_eq!(branch_name, "worktree/TASK-001");
            assert_eq!(base_branch, "develop");
        }
        _ => panic!("Expected Provisioned allocation"),
    }

    assert_eq!(alloc.branch_name(), Some("worktree/TASK-001"));
    assert_eq!(alloc.base_branch(), Some("develop"));
    assert!(
        alloc
            .worktree_path()
            .expect("worktree path")
            .contains(".trustkit/worktrees/TASK-001")
    );

    // Verify root repo active branch is untouched and preserved
    let root_branch_out = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(repo_dir.path())
        .output()
        .expect("git rev-parse HEAD in root");
    let root_branch = String::from_utf8_lossy(&root_branch_out.stdout)
        .trim()
        .to_string();
    assert_eq!(root_branch, "develop");

    // Verify worktree exists on filesystem and is valid git worktree
    let wt_path = alloc.worktree_path().unwrap();
    assert!(Path::new(wt_path).exists());
    let status = provider
        .check_worktree_status(wt_path)
        .expect("check status");
    assert!(status.is_clean);
    assert_eq!(status.branch_name, "worktree/TASK-001");
}
