// Contract tests for tk CLI parity and argv0 dynamic resolution (TASK-002 / RF-002 / RN-002).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_task_002_tk_help_shows_tk_usage() {
    let mut cmd = Command::cargo_bin("tk").expect("find tk binary");
    cmd.arg("--help")
        .assert()
        .success()
        .code(0)
        .stdout(predicate::str::contains("Usage: tk"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn test_task_002_tk_version_shows_tk_version() {
    let mut cmd = Command::cargo_bin("tk").expect("find tk binary");
    cmd.arg("--version")
        .assert()
        .success()
        .code(0)
        .stdout(predicate::str::starts_with("tk "))
        .stderr(predicate::str::is_empty());
}

#[test]
fn test_task_002_trustkit_help_preserves_trustkit_usage() {
    let mut cmd = Command::cargo_bin("trustkit").expect("find trustkit binary");
    cmd.arg("--help")
        .assert()
        .success()
        .code(0)
        .stdout(predicate::str::contains("Usage: trustkit"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn test_task_002_trustkit_version_preserves_trustkit_version() {
    let mut cmd = Command::cargo_bin("trustkit").expect("find trustkit binary");
    cmd.arg("--version")
        .assert()
        .success()
        .code(0)
        .stdout(predicate::str::starts_with("trustkit "))
        .stderr(predicate::str::is_empty());
}
