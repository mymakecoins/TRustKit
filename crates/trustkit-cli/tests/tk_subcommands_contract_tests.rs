// Contract tests for tk subcommands parity (TASK-003 / RF-003 / RN-003).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_task_003_tk_status_json_is_null_free_and_clean() {
    let mut cmd = Command::cargo_bin("tk").expect("find tk binary");
    let assert = cmd.args(["status", "--json"]).assert();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(
        !stdout.contains("null"),
        "JSON contract violation: null detected in tk status --json"
    );
    assert!(
        stdout.contains("\"verdict\": \"PASS\"") || stdout.contains("\"verdict\":\"PASS\""),
        "Expected PASS verdict in tk status"
    );
}

#[test]
fn test_task_003_tk_doctor_json_is_null_free() {
    let mut cmd = Command::cargo_bin("tk").expect("find tk binary");
    let assert = cmd.args(["doctor", "--json"]).assert();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(
        !stdout.contains("null"),
        "JSON contract violation: null detected in tk doctor --json"
    );
}

#[test]
fn test_task_003_tk_wave_help_exits_zero() {
    let mut cmd = Command::cargo_bin("tk").expect("find tk binary");
    cmd.args(["wave", "--help"])
        .assert()
        .success()
        .code(0)
        .stdout(predicate::str::contains("Usage: tk wave"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn test_task_003_tk_task_help_exits_zero() {
    let mut cmd = Command::cargo_bin("tk").expect("find tk binary");
    cmd.args(["task", "--help"])
        .assert()
        .success()
        .code(0)
        .stdout(predicate::str::contains("Usage: tk task"))
        .stderr(predicate::str::is_empty());
}
