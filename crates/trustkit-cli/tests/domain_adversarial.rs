// Domain Adversarial Critic integration tests.
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_domain_adversarial_rejects_empty_cycle_operations() {
    let mut cmd = Command::cargo_bin("trustkit").expect("trustkit binary");
    cmd.args(["task", "test", "NON_EXISTENT_TASK", "--phase", "red"])
        .assert()
        .failure();
}

#[test]
fn test_domain_adversarial_forbids_arbitrary_gate_skipping() {
    let mut cmd = Command::cargo_bin("trustkit").expect("trustkit binary");
    cmd.args(["task", "complete", "TASK-999"])
        .assert()
        .failure();
}

#[test]
fn test_domain_adversarial_doctor_survives_adversarial_calls() {
    let mut cmd = Command::cargo_bin("trustkit").expect("trustkit binary");
    cmd.args(["doctor", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"verdict\":"));
}
