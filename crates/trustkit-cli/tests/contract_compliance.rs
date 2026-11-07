// Contract Compliance Critic integration tests.
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_contract_compliance_help_exits_code_zero_on_stdout() {
    let mut cmd = Command::cargo_bin("trustkit").expect("trustkit binary");
    cmd.args(["--help"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("Usage: trustkit"));
}

#[test]
fn test_contract_compliance_version_exits_code_zero() {
    let mut cmd = Command::cargo_bin("trustkit").expect("trustkit binary");
    cmd.args(["--version"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("trustkit"));
}

#[test]
fn test_contract_compliance_json_output_null_free() {
    let mut cmd = Command::cargo_bin("trustkit").expect("trustkit binary");
    let assert = cmd.args(["status", "--json"]).assert();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(
        !stdout.contains("null"),
        "JSON contract violation: null detected in stdout"
    );
}

#[test]
fn test_contract_compliance_invalid_subcommand_exits_non_zero() {
    let mut cmd = Command::cargo_bin("trustkit").expect("trustkit binary");
    cmd.args(["invalid_subcommand_contract_check"])
        .assert()
        .failure();
}
