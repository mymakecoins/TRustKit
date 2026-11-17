// Contract test for tk cargo binary manifest definition (TASK-001 / RF-001 / RN-001).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use assert_cmd::Command;

#[test]
fn test_task_001_tk_binary_exists_and_runs() {
    let mut cmd = Command::cargo_bin("tk").expect("find tk binary");
    cmd.arg("--version").assert().success().code(0);
}
