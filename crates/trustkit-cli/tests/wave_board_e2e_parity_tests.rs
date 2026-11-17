// E2E Parity tests between 'trustkit' and 'tk' for board and wave board commands (TASK-005 / RF-005 / RN-005).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_task_005_e2e_parity_board_help() {
    let mut cmd_tk = Command::cargo_bin("tk").expect("find tk");
    cmd_tk
        .args(["board", "--help"])
        .assert()
        .success()
        .code(0)
        .stdout(predicate::str::contains("Usage: tk board"))
        .stderr(predicate::str::is_empty());

    let mut cmd_trustkit = Command::cargo_bin("trustkit").expect("find trustkit");
    cmd_trustkit
        .args(["board", "--help"])
        .assert()
        .success()
        .code(0)
        .stdout(predicate::str::contains("Usage: trustkit board"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn test_task_005_e2e_parity_wave_board_help() {
    let mut cmd_tk = Command::cargo_bin("tk").expect("find tk");
    cmd_tk
        .args(["wave", "board", "--help"])
        .assert()
        .success()
        .code(0)
        .stdout(predicate::str::contains("Usage: tk wave board"))
        .stderr(predicate::str::is_empty());

    let mut cmd_trustkit = Command::cargo_bin("trustkit").expect("find trustkit");
    cmd_trustkit
        .args(["wave", "board", "--help"])
        .assert()
        .success()
        .code(0)
        .stdout(predicate::str::contains("Usage: trustkit wave board"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn test_task_005_e2e_parity_board_status_json_null_free() {
    let mut cmd_tk = Command::cargo_bin("tk").expect("find tk");
    let assert_tk = cmd_tk.args(["board", "status", "--json"]).assert().success();
    let stdout_tk = String::from_utf8(assert_tk.get_output().stdout.clone()).unwrap();

    assert!(!stdout_tk.contains("null"), "tk board status contained null");
    assert!(stdout_tk.contains("\"status\": \"PASS\""));
    assert!(stdout_tk.contains("\"action\": \"status\""));

    let mut cmd_trustkit = Command::cargo_bin("trustkit").expect("find trustkit");
    let assert_trustkit = cmd_trustkit.args(["board", "status", "--json"]).assert().success();
    let stdout_trustkit = String::from_utf8(assert_trustkit.get_output().stdout.clone()).unwrap();

    assert!(!stdout_trustkit.contains("null"), "trustkit board status contained null");
    assert!(stdout_trustkit.contains("\"status\": \"PASS\""));
    assert!(stdout_trustkit.contains("\"action\": \"status\""));
}

#[test]
fn test_task_005_e2e_parity_board_down_json_null_free() {
    let mut cmd_tk = Command::cargo_bin("tk").expect("find tk");
    let assert_tk = cmd_tk.args(["board", "down", "--json"]).assert().success();
    let stdout_tk = String::from_utf8(assert_tk.get_output().stdout.clone()).unwrap();

    assert!(!stdout_tk.contains("null"), "tk board down contained null");
    assert!(stdout_tk.contains("\"action\": \"down\""));

    let mut cmd_trustkit = Command::cargo_bin("trustkit").expect("find trustkit");
    let assert_trustkit = cmd_trustkit.args(["board", "down", "--json"]).assert().success();
    let stdout_trustkit = String::from_utf8(assert_trustkit.get_output().stdout.clone()).unwrap();

    assert!(!stdout_trustkit.contains("null"), "trustkit board down contained null");
    assert!(stdout_trustkit.contains("\"action\": \"down\""));
}
