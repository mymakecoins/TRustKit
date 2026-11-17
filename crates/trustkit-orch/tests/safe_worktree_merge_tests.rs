// Safe Worktree Merge Integration Tests (TASK-008 / RF-008 / RN-008).
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
fn test_safe_accumulative_worktree_merge_preserves_all_commits() {
    let repo_dir = setup_test_git_repo("main");
    let provider = WorktreeIsolationProvider::new(repo_dir.path());

    let alloc1 = provider
        .provision_worktree("cycle-1", "TASK-008A", "main")
        .expect("provision task A");
    let wt1_path = match alloc1 {
        WorktreeAllocation::Provisioned { worktree_path, .. } => worktree_path,
        _ => panic!("Expected Provisioned allocation for TASK-008A"),
    };

    let alloc2 = provider
        .provision_worktree("cycle-1", "TASK-008B", "main")
        .expect("provision task B");
    let wt2_path = match alloc2 {
        WorktreeAllocation::Provisioned { worktree_path, .. } => worktree_path,
        _ => panic!("Expected Provisioned allocation for TASK-008B"),
    };

    // Commit in worktree 1
    fs::write(
        Path::new(&wt1_path).join("task_a.txt"),
        "Content from Task A\n",
    )
    .expect("write task_a.txt");
    Command::new("git")
        .args(["add", "task_a.txt"])
        .current_dir(&wt1_path)
        .output()
        .expect("git add wt1");
    Command::new("git")
        .args(["commit", "-m", "feat: #8 implement task A"])
        .current_dir(&wt1_path)
        .output()
        .expect("git commit wt1");

    // Commit in worktree 2
    fs::write(
        Path::new(&wt2_path).join("task_b.txt"),
        "Content from Task B\n",
    )
    .expect("write task_b.txt");
    Command::new("git")
        .args(["add", "task_b.txt"])
        .current_dir(&wt2_path)
        .output()
        .expect("git add wt2");
    Command::new("git")
        .args(["commit", "-m", "feat: #8 implement task B"])
        .current_dir(&wt2_path)
        .output()
        .expect("git commit wt2");

    // Merge task A into main
    let sha1 = provider
        .merge_and_prune_worktree("TASK-008A", &wt1_path, "main")
        .expect("merge task A");
    assert!(!sha1.is_empty());
    assert!(!Path::new(&wt1_path).exists());

    // Merge task B into main
    let sha2 = provider
        .merge_and_prune_worktree("TASK-008B", &wt2_path, "main")
        .expect("merge task B");
    assert!(!sha2.is_empty());
    assert!(!Path::new(&wt2_path).exists());

    // Verify main has files from both tasks
    assert!(repo_dir.path().join("task_a.txt").exists());
    assert!(repo_dir.path().join("task_b.txt").exists());

    // Verify git log on main contains both commits
    let log_output = Command::new("git")
        .args(["log", "--oneline", "main"])
        .current_dir(repo_dir.path())
        .output()
        .expect("git log");
    let log_str = String::from_utf8_lossy(&log_output.stdout);
    assert!(log_str.contains("feat: #8 implement task A"));
    assert!(log_str.contains("feat: #8 implement task B"));
}

#[test]
fn test_safe_worktree_merge_detects_conflicts() {
    let repo_dir = setup_test_git_repo("main");
    let provider = WorktreeIsolationProvider::new(repo_dir.path());

    let alloc1 = provider
        .provision_worktree("cycle-1", "TASK-008C", "main")
        .expect("provision task C");
    let wt1_path = match alloc1 {
        WorktreeAllocation::Provisioned { worktree_path, .. } => worktree_path,
        _ => panic!("Expected Provisioned allocation"),
    };

    let alloc2 = provider
        .provision_worktree("cycle-1", "TASK-008D", "main")
        .expect("provision task D");
    let wt2_path = match alloc2 {
        WorktreeAllocation::Provisioned { worktree_path, .. } => worktree_path,
        _ => panic!("Expected Provisioned allocation"),
    };

    // Both worktrees modify shared_file.txt with conflicting content
    fs::write(Path::new(&wt1_path).join("shared_file.txt"), "Version C\n").expect("write C");
    Command::new("git")
        .args(["add", "shared_file.txt"])
        .current_dir(&wt1_path)
        .output()
        .expect("git add C");
    Command::new("git")
        .args(["commit", "-m", "feat: #8 conflicting commit from C"])
        .current_dir(&wt1_path)
        .output()
        .expect("git commit C");

    fs::write(Path::new(&wt2_path).join("shared_file.txt"), "Version D\n").expect("write D");
    Command::new("git")
        .args(["add", "shared_file.txt"])
        .current_dir(&wt2_path)
        .output()
        .expect("git add D");
    Command::new("git")
        .args(["commit", "-m", "feat: #8 conflicting commit from D"])
        .current_dir(&wt2_path)
        .output()
        .expect("git commit D");

    // Merge task C succeeds
    let sha_c = provider
        .merge_and_prune_worktree("TASK-008C", &wt1_path, "main")
        .expect("merge task C");
    assert!(!sha_c.is_empty());

    // Merge task D must detect conflict and fail with DomainViolation
    let merge_d_result = provider.merge_and_prune_worktree("TASK-008D", &wt2_path, "main");
    assert!(merge_d_result.is_err());
    let err_msg = format!("{:?}", merge_d_result.err().unwrap());
    assert!(err_msg.contains("Merge conflict") || err_msg.contains("conflict"));
}
