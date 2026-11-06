// Tests for CLI help and version exit code compliance (TASK-001 & TASK-004).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_task_001_root_help_flag_returns_exit_code_zero_on_stdout() {
    let mut cmd = Command::cargo_bin("trustkit").expect("find trustkit binary");
    cmd.arg("--help")
        .assert()
        .success()
        .code(0)
        .stdout(predicate::str::contains("Usage: trustkit"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn test_task_001_root_short_help_flag_returns_exit_code_zero() {
    let mut cmd = Command::cargo_bin("trustkit").expect("find trustkit binary");
    cmd.arg("-h")
        .assert()
        .success()
        .code(0)
        .stdout(predicate::str::contains("Usage: trustkit"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn test_task_001_root_version_flag_returns_exit_code_zero_on_stdout() {
    let mut cmd = Command::cargo_bin("trustkit").expect("find trustkit binary");
    cmd.arg("--version")
        .assert()
        .success()
        .code(0)
        .stdout(predicate::str::contains("trustkit"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn test_task_001_gauntlet_subcommand_help_returns_exit_code_zero() {
    let mut cmd = Command::cargo_bin("trustkit").expect("find trustkit binary");
    cmd.args(["gauntlet", "--help"])
        .assert()
        .success()
        .code(0)
        .stdout(predicate::str::contains("gauntlet"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn test_task_001_gauntlet_run_help_returns_exit_code_zero_on_stdout() {
    let mut cmd = Command::cargo_bin("trustkit").expect("find trustkit binary");
    cmd.args(["gauntlet", "run", "--help"])
        .assert()
        .success()
        .code(0)
        .stdout(predicate::str::contains("Usage: trustkit gauntlet run"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn test_task_001_auto_subcommand_help_returns_exit_code_zero() {
    let mut cmd = Command::cargo_bin("trustkit").expect("find trustkit binary");
    cmd.args(["auto", "--help"])
        .assert()
        .success()
        .code(0)
        .stdout(predicate::str::contains("Usage: trustkit auto"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn test_task_002_invalid_subcommand_fails_with_exit_code_and_stderr() {
    let mut cmd = Command::cargo_bin("trustkit").expect("find trustkit binary");
    cmd.arg("invalid-subcommand-xyz")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error: unrecognized subcommand"));
}

#[test]
fn test_task_002_invalid_subcommand_json_mode_emits_json_failure_on_stderr() {
    let mut cmd = Command::cargo_bin("trustkit").expect("find trustkit binary");
    cmd.args(["invalid-subcommand-xyz", "--json"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("\"kind\":\"CLI_PARSE_ERROR\""));
}

#[test]
fn test_task_002_invalid_flag_fails_with_exit_code_and_stderr() {
    let mut cmd = Command::cargo_bin("trustkit").expect("find trustkit binary");
    cmd.arg("--unknown-flag-123")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "error: unexpected argument '--unknown-flag-123'",
        ));
}

#[test]
fn test_task_002_auto_accepts_positional_issue_or_spec() {
    let mut cmd = Command::cargo_bin("trustkit").expect("find trustkit binary");
    // Invocando com --help em auto com argumento posicional sintatico ou verificando parsing de positional
    cmd.args([
        "auto",
        "https://github.com/mymakecoins/TRustKit/issues/7",
        "--help",
    ])
    .assert()
    .success()
    .code(0);
}

#[test]
fn test_task_004_all_primary_subcommands_support_help_and_short_help() {
    let subcommands: &[&[&str]] = &[
        &["init", "--help"],
        &["init", "-h"],
        &["status", "--help"],
        &["status", "-h"],
        &["spec", "--help"],
        &["spec", "-h"],
        &["spec", "lint", "--help"],
        &["spec", "lock", "--help"],
        &["spec", "prompt", "--help"],
        &["wave", "--help"],
        &["wave", "-h"],
        &["wave", "plan", "--help"],
        &["wave", "run", "--help"],
        &["task", "--help"],
        &["task", "-h"],
        &["task", "test", "--help"],
        &["task", "complete", "--help"],
        &["gauntlet", "--help"],
        &["gauntlet", "-h"],
        &["gauntlet", "ast-audit", "--help"],
        &["gauntlet", "run", "--help"],
        &["gauntlet", "run", "-h"],
        &["doctor", "--help"],
        &["doctor", "-h"],
        &["done", "--help"],
        &["done", "-h"],
        &["merge", "--help"],
        &["merge", "-h"],
        &["pr", "--help"],
        &["pr", "-h"],
        &["auto", "--help"],
        &["auto", "-h"],
    ];

    for args in subcommands {
        let mut cmd = Command::cargo_bin("trustkit").expect("find trustkit binary");
        cmd.args(*args)
            .assert()
            .success()
            .code(0)
            .stdout(predicate::str::contains("Usage: trustkit"))
            .stderr(predicate::str::is_empty());
    }
}
