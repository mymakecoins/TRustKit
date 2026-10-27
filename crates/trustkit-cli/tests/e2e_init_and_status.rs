// E2E tests for `trustkit init` and `trustkit status`.
// Constitutional Principle III: Test-First, Anti-Cheat & Deterministic Equivalence.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::process::Command as StdCommand;
use tempfile::tempdir;

fn init_git_repo(path: &std::path::Path) {
    let run = |args: &[&str]| {
        let status = StdCommand::new("git")
            .args(args)
            .current_dir(path)
            .status()
            .expect("git command failed");
        assert!(status.success());
    };

    run(&["init", "-b", "main"]);
    run(&["config", "user.email", "test@example.com"]);
    run(&["config", "user.name", "Test User"]);

    // Create initial commit
    let readme = path.join("README.md");
    fs::write(&readme, "# Test Repo\n").expect("write readme");
    run(&["add", "README.md"]);
    run(&["commit", "-m", "chore: initial commit"]);
}

#[test]
fn test_init_fails_in_non_git_repository() {
    let dir = tempdir().expect("create temp dir");
    let mut cmd = Command::cargo_bin("trustkit").expect("find trustkit binary");
    cmd.current_dir(dir.path())
        .arg("init")
        .arg("--json")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("INFRASTRUCTURE_BOUNDARY_FAULT"));
}

#[test]
fn test_init_unlinked_local_cycle_json_mode_and_dirty_file_preservation() {
    let dir = tempdir().expect("create temp dir");
    init_git_repo(dir.path());

    // Create an uncommitted dirty file
    let dirty_file = dir.path().join("dirty.txt");
    fs::write(&dirty_file, "pre-existing content\n").expect("write dirty file");

    let mut cmd = Command::cargo_bin("trustkit").expect("find trustkit binary");
    let assert = cmd
        .current_dir(dir.path())
        .arg("init")
        .arg("--json")
        .assert()
        .success()
        .code(0);

    let output = String::from_utf8(assert.get_output().stdout.clone()).expect("valid utf8");

    // Zero : null assertion per Constitutional Principle II
    assert!(!output.contains(": null"), "JSON must not contain ': null'");

    // Verify unlinked cycle payload
    assert!(output.contains("\"verdict\": \"PASS\"") || output.contains("\"verdict\":\"PASS\""));
    assert!(output.contains("UNLINKED_LOCAL_CYCLE"));
    assert!(output.contains("dirty.txt"));

    // Verify pre-existing dirty file remained completely untouched
    let content = fs::read_to_string(&dirty_file).expect("read dirty file");
    assert_eq!(content, "pre-existing content\n");

    // Verify cycle directory was created
    let specs_dir = dir.path().join("docs").join("specs");
    assert!(specs_dir.exists(), "docs/specs directory must be created");
}

#[test]
fn test_init_linked_cycle_with_issue_flag() {
    let dir = tempdir().expect("create temp dir");
    init_git_repo(dir.path());

    let mut cmd = Command::cargo_bin("trustkit").expect("find trustkit binary");
    let assert = cmd
        .current_dir(dir.path())
        .arg("init")
        .arg("--issue")
        .arg("45")
        .arg("--json")
        .assert()
        .success()
        .code(0);

    let output = String::from_utf8(assert.get_output().stdout.clone()).expect("valid utf8");
    assert!(!output.contains(": null"), "JSON must not contain ': null'");
    assert!(output.contains("LINKED_ISSUE"));
    assert!(output.contains("issue-45-"));

    // Verify cycle directory exists
    let entries = fs::read_dir(dir.path().join("docs").join("specs")).expect("read specs dir");
    let has_issue_dir = entries.filter_map(|e| e.ok()).any(|e| {
        e.file_name().to_string_lossy().starts_with("issue-45-")
    });
    assert!(has_issue_dir, "cycle directory issue-45-* must be created");
}

#[test]
fn test_status_uninitialized_repository_graceful_exit_code_zero() {
    let dir = tempdir().expect("create temp dir");
    init_git_repo(dir.path());

    // Test in --json mode
    let mut cmd_json = Command::cargo_bin("trustkit").expect("find trustkit binary");
    let assert_json = cmd_json
        .current_dir(dir.path())
        .arg("status")
        .arg("--json")
        .assert()
        .success()
        .code(0);

    let stdout_json = String::from_utf8(assert_json.get_output().stdout.clone()).expect("valid utf8");
    assert!(!stdout_json.contains(": null"), "JSON must not contain ': null'");
    assert!(stdout_json.contains("\"verdict\": \"PASS\"") || stdout_json.contains("\"verdict\":\"PASS\""));
    assert!(stdout_json.contains("NO_ACTIVE_CYCLE"));
    assert!(stdout_json.contains("Run 'trustkit init"));

    // Test in human ASCII mode
    let mut cmd_human = Command::cargo_bin("trustkit").expect("find trustkit binary");
    let assert_human = cmd_human
        .current_dir(dir.path())
        .arg("status")
        .assert()
        .success()
        .code(0);

    let stdout_human = String::from_utf8(assert_human.get_output().stdout.clone()).expect("valid utf8");
    assert!(stdout_human.contains("[PASS]"));
    assert!(stdout_human.contains("No active cycle found"));
    assert!(stdout_human.chars().all(|c| c.is_ascii() || c.is_whitespace()));
}

#[test]
fn test_status_active_gate_0_cycle_json_and_ascii() {
    let dir = tempdir().expect("create temp dir");
    init_git_repo(dir.path());

    // Initialize an active cycle first
    let mut init_cmd = Command::cargo_bin("trustkit").expect("find trustkit binary");
    init_cmd
        .current_dir(dir.path())
        .arg("init")
        .arg("--issue")
        .arg("45")
        .arg("--json")
        .assert()
        .success()
        .code(0);

    // Test status in --json mode
    let mut status_json = Command::cargo_bin("trustkit").expect("find trustkit binary");
    let assert_json = status_json
        .current_dir(dir.path())
        .arg("status")
        .arg("--json")
        .assert()
        .success()
        .code(0);

    let stdout_json = String::from_utf8(assert_json.get_output().stdout.clone()).expect("valid utf8");
    assert!(!stdout_json.contains(": null"), "JSON must not contain ': null'");
    assert!(stdout_json.contains("\"verdict\": \"PASS\"") || stdout_json.contains("\"verdict\":\"PASS\""));
    assert!(stdout_json.contains("ACTIVE_CYCLE") || stdout_json.contains("\"cycle_id\": \"issue-45-") || stdout_json.contains("\"cycle_id\":\"issue-45-"));
    assert!(stdout_json.contains("GATE_0_DISCOVERY"));
    assert!(stdout_json.contains("ACTIVE"));

    // Test status in human ASCII mode
    let mut status_human = Command::cargo_bin("trustkit").expect("find trustkit binary");
    let assert_human = status_human
        .current_dir(dir.path())
        .arg("status")
        .assert()
        .success()
        .code(0);

    let stdout_human = String::from_utf8(assert_human.get_output().stdout.clone()).expect("valid utf8");
    assert!(stdout_human.contains("[PASS] [GATE-0] Active Cycle Status"));
    assert!(stdout_human.contains("Cycle ID          : issue-45-"));
    assert!(stdout_human.contains("Current Gate      : 0 (GATE_0_DISCOVERY)"));
    assert!(stdout_human.contains("Lifecycle Status  : ACTIVE"));
    assert!(stdout_human.chars().all(|c| c.is_ascii() || c.is_whitespace()));
}
