// Integration tests for decoupled pre-release `trustkit audit security` command (TASK-008).

use clap::Parser;
use trustkit_cli::bootstrap::dispatch_cli;
use trustkit_cli::cli_contract::Cli;

#[test]
fn test_audit_security_standard_profile_with_gitignored_dir_passes() {
    let temp_dir = std::env::temp_dir().join(format!(
        "trustkit_audit_sec_pass_{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).expect("temp dir should be created");
    std::fs::write(temp_dir.join(".gitignore"), ".trustkit/\nreports-ignored/\n")
        .expect(".gitignore should be written");

    let out_dir = temp_dir.join("reports-ignored").join("sec-run");
    let scope_str = temp_dir.to_string_lossy().to_string();
    let out_str = out_dir.to_string_lossy().to_string();

    let cli = Cli::try_parse_from([
        "trustkit",
        "audit",
        "security",
        "--profile",
        "standard",
        "--scope",
        &scope_str,
        "--output-dir",
        &out_str,
        "--json",
    ])
    .expect("CLI should parse audit security command");

    let outcome = dispatch_cli(&cli).expect("audit security should succeed for gitignored dir");
    match outcome {
        trustkit_cli::error_handler::DomainOutcome::PassWithDetails(payload) => {
            assert!(payload.contains("\"verdict\":\"PASS\""));
            assert!(payload.contains("\"profile\":\"standard\""));
            assert!(!payload.contains("null"));
        }
        other => panic!("Expected PassWithDetails, got: {other:?}"),
    }
    assert!(out_dir.join("coverage-ledger.json").is_file());
    assert!(out_dir.join("findings.json").is_file());

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_audit_security_rejects_unignored_output_dir_inside_repo() {
    let temp_dir = std::env::temp_dir().join(format!(
        "trustkit_audit_sec_reject_{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).expect("temp dir should be created");
    std::fs::write(temp_dir.join(".gitignore"), "target/\n").expect(".gitignore should be written");

    let unignored_out = temp_dir.join("tracked_audit_folder");
    let scope_str = temp_dir.to_string_lossy().to_string();
    let out_str = unignored_out.to_string_lossy().to_string();

    let cli = Cli::try_parse_from([
        "trustkit",
        "audit",
        "security",
        "--profile",
        "deep",
        "--scope",
        &scope_str,
        "--output-dir",
        &out_str,
        "--json",
    ])
    .expect("CLI should parse audit security command");

    let err = dispatch_cli(&cli).expect_err("unignored output dir inside repo must be rejected");
    assert_eq!(err.posix_exit_code(), 4);

    let _ = std::fs::remove_dir_all(&temp_dir);
}
