// E2E tests for `trustkit hook pre-tool`.
// Constitutional Principle III: Test-First, Anti-Cheat & Deterministic Equivalence.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use assert_cmd::Command;
use predicates::prelude::*;
use std::time::Instant;

#[test]
fn test_hook_pre_tool_blocks_forbidden_command_json_mode() {
    let mut cmd = Command::cargo_bin("trustkit").expect("find trustkit binary");
    let assert = cmd
        .arg("hook")
        .arg("pre-tool")
        .arg("--command")
        .arg("git stash")
        .arg("--json")
        .assert()
        .failure()
        .code(4)
        .stderr(predicate::str::contains("BLOCKED"))
        .stderr(predicate::str::contains("FORBIDDEN_COMMAND_INTERCEPTED"))
        .stderr(predicate::str::contains("RN-006"));

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("valid utf8");
    assert!(
        !stderr.contains(": null"),
        "JSON payload must contain zero ': null'"
    );
}

#[test]
fn test_hook_pre_tool_blocks_sweeping_git_add_json_mode() {
    let mut cmd = Command::cargo_bin("trustkit").expect("find trustkit binary");
    let assert = cmd
        .arg("hook")
        .arg("pre-tool")
        .arg("--command")
        .arg("git add .")
        .arg("--json")
        .assert()
        .failure()
        .code(4)
        .stderr(predicate::str::contains("BLOCKED"))
        .stderr(predicate::str::contains("RN-006"))
        .stderr(predicate::str::contains("git add ."));

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("valid utf8");
    assert!(
        !stderr.contains(": null"),
        "JSON payload must contain zero ': null'"
    );
}

#[test]
fn test_hook_pre_tool_blocks_unicode_emoji_json_mode() {
    let mut cmd = Command::cargo_bin("trustkit").expect("find trustkit binary");
    let assert = cmd
        .arg("hook")
        .arg("pre-tool")
        .arg("--command")
        .arg("git commit -m 'feat: \u{1F680} blast off'")
        .arg("--json")
        .assert()
        .failure()
        .code(4)
        .stderr(predicate::str::contains("BLOCKED"))
        .stderr(predicate::str::contains("RN-007"));

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("valid utf8");
    assert!(
        !stderr.contains(": null"),
        "JSON payload must contain zero ': null'"
    );
}

#[test]
fn test_hook_pre_tool_approves_safe_command_json_mode() {
    let mut cmd = Command::cargo_bin("trustkit").expect("find trustkit binary");
    let assert = cmd
        .arg("hook")
        .arg("pre-tool")
        .arg("--command")
        .arg("cargo test -p trustkit-core")
        .arg("--json")
        .assert()
        .success()
        .code(0);

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("valid utf8");
    assert!(stdout.contains("\"verdict\": \"PASS\"") || stdout.contains("\"verdict\":\"PASS\""));
    assert!(stdout.contains("cargo test -p trustkit-core"));
    assert!(
        !stdout.contains(": null"),
        "JSON payload must contain zero ': null'"
    );
}

#[test]
fn test_hook_pre_tool_approves_safe_command_ascii_mode() {
    let mut cmd = Command::cargo_bin("trustkit").expect("find trustkit binary");
    let assert = cmd
        .arg("hook")
        .arg("pre-tool")
        .arg("--command")
        .arg("cargo build --release")
        .assert()
        .success()
        .code(0);

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("valid utf8");
    assert!(stdout.contains("[PASS]"));
    assert!(
        stdout.is_ascii(),
        "Output must be 100% ASCII without Unicode emojis"
    );
}

#[test]
fn test_hook_pre_tool_execution_responsiveness() {
    let mut cmd = Command::cargo_bin("trustkit").expect("find trustkit binary");
    let start = Instant::now();
    cmd.arg("hook")
        .arg("pre-tool")
        .arg("--command")
        .arg("cargo test")
        .arg("--json")
        .assert()
        .success()
        .code(0);
    let elapsed = start.elapsed();
    // Verify fast responsiveness (including process invocation in test harness)
    assert!(
        elapsed.as_millis() < 500,
        "Hook process execution must be responsive"
    );
}
