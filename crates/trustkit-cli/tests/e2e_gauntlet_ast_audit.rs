// E2E tests for `trustkit gauntlet ast-audit` and `trustkit context slice` (TASK-016).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use assert_cmd::Command;
use serde_json::Value;
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
    let readme = path.join("README.md");
    fs::write(&readme, "# Test Repo\n").expect("write readme");
    run(&["add", "README.md"]);
    run(&["commit", "-m", "chore: initial commit"]);
}

#[test]
fn test_e2e_context_slice_success() {
    let dir = tempdir().expect("create temp dir");
    let file_path = dir.path().join("service.rs");
    fs::write(
        &file_path,
        r#"
pub struct PaymentService {
    pub api_key: String,
}

impl PaymentService {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    pub fn process_charge(&self, amount: u64) -> bool {
        amount > 0
    }
}
"#,
    )
    .expect("write rust file");

    let mut cmd = Command::cargo_bin("trustkit").expect("find trustkit binary");
    let assert = cmd
        .args([
            "context",
            "slice",
            "--file",
            file_path.to_str().unwrap(),
            "--budget",
            "10",
            "--symbols",
            "PaymentService",
            "--json",
        ])
        .assert()
        .success()
        .code(0);

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: Value = serde_json::from_str(&stdout).expect("valid json output");
    assert_eq!(json["status"], "CONTEXT_SLICED");
    assert!(json["total_lines"].as_u64().unwrap() <= 10);
    assert!(!stdout.contains("null"));
}

#[test]
fn test_e2e_gauntlet_ast_audit_clean_pass() {
    let dir = tempdir().expect("create temp dir");
    let base_dir = dir.path();
    init_git_repo(base_dir);

    // 1. Initialize cycle at base_dir
    let mut init_cmd = Command::cargo_bin("trustkit").expect("find trustkit binary");
    init_cmd
        .current_dir(base_dir)
        .args(["init", "--json"])
        .assert()
        .success()
        .code(0);

    // 2. Create clean file
    let clean_path = base_dir.join("clean.rs");
    fs::write(
        &clean_path,
        "pub fn calculate_sum(a: i32, b: i32) -> i32 {\n    a + b\n}\n",
    )
    .expect("write clean code");

    // 3. Run gauntlet ast-audit
    let mut audit_cmd = Command::cargo_bin("trustkit").expect("find trustkit binary");
    let assert = audit_cmd
        .current_dir(base_dir)
        .args([
            "gauntlet",
            "ast-audit",
            "--files",
            clean_path.to_str().unwrap(),
            "--json",
        ])
        .assert()
        .success()
        .code(0);

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: Value = serde_json::from_str(&stdout).expect("valid json output");
    assert_eq!(json["status"], "PASS");
    assert_eq!(json["verdict"], "CLEAN_PASS");
    assert_eq!(json["total_violations"], 0);
    assert!(!stdout.contains("null"));
}

#[test]
fn test_e2e_gauntlet_ast_audit_forbidden_emoji_fails_with_code_4() {
    let dir = tempdir().expect("create temp dir");
    let base_dir = dir.path();
    init_git_repo(base_dir);

    let mut init_cmd = Command::cargo_bin("trustkit").expect("find trustkit binary");
    init_cmd
        .current_dir(base_dir)
        .args(["init", "--json"])
        .assert()
        .success()
        .code(0);

    let dirty_path = base_dir.join("emoji.rs");
    fs::write(
        &dirty_path,
        "// Invalid token: rocket 🚀\npub fn check() {}\n",
    )
    .expect("write dirty code");

    let mut audit_cmd = Command::cargo_bin("trustkit").expect("find trustkit binary");
    let assert = audit_cmd
        .current_dir(base_dir)
        .args([
            "gauntlet",
            "ast-audit",
            "--files",
            dirty_path.to_str().unwrap(),
            "--json",
        ])
        .assert()
        .failure()
        .code(4);

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let json: Value = serde_json::from_str(&stderr).expect("valid json error envelope");
    assert_eq!(json["exit_code"], 4);
    assert_eq!(json["verdict"], "BLOCKED");
    assert!(!stderr.contains("null"));
}

#[test]
fn test_e2e_gauntlet_ast_audit_ai_trailer_fails_with_code_4() {
    let dir = tempdir().expect("create temp dir");
    let base_dir = dir.path();
    init_git_repo(base_dir);

    let mut init_cmd = Command::cargo_bin("trustkit").expect("find trustkit binary");
    init_cmd
        .current_dir(base_dir)
        .args(["init", "--json"])
        .assert()
        .success()
        .code(0);

    let trailer_path = base_dir.join("trailer.rs");
    fs::write(
        &trailer_path,
        "// Co-Authored-By: Claude <noreply@anthropic.com>\npub fn safe() {}\n",
    )
    .expect("write trailer code");

    let mut audit_cmd = Command::cargo_bin("trustkit").expect("find trustkit binary");
    let assert = audit_cmd
        .current_dir(base_dir)
        .args([
            "gauntlet",
            "ast-audit",
            "--files",
            trailer_path.to_str().unwrap(),
            "--json",
        ])
        .assert()
        .failure()
        .code(4);

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let json: Value = serde_json::from_str(&stderr).expect("valid json error envelope");
    assert_eq!(json["exit_code"], 4);
    assert!(!stderr.contains("null"));
}
