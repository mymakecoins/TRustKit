// Tests for TASK-003: CLI update subcommand, Dual-Audience presenter and tk parity.
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use assert_cmd::Command as AssertCommand;
use clap::Parser;
use predicates::prelude::*;
use serde_json::Value;
use trustkit_cli::cli_contract::{Cli, Command, UpdateArgs};

#[test]
fn test_parse_update_command_flags() {
    let cli_default = Cli::try_parse_from(["trustkit", "update"]).expect("parse default update");
    assert!(!cli_default.is_json_mode());
    assert_eq!(
        cli_default.command,
        Command::Update(UpdateArgs {
            check: false,
            force: false,
            json: false,
        })
    );

    let cli_flags = Cli::try_parse_from(["trustkit", "update", "--check", "--force", "--json"])
        .expect("parse update flags");
    assert!(cli_flags.is_json_mode());
    assert_eq!(
        cli_flags.command,
        Command::Update(UpdateArgs {
            check: true,
            force: true,
            json: true,
        })
    );
}

#[test]
fn test_trustkit_and_tk_update_help_parity_without_emojis() {
    for bin in ["trustkit", "tk"] {
        let mut cmd = AssertCommand::cargo_bin(bin).expect("find binary");
        let assert = cmd
            .args(["update", "--help"])
            .assert()
            .success()
            .code(0)
            .stdout(predicate::str::contains(format!("Usage: {bin} update")))
            .stdout(predicate::str::contains("--check"))
            .stdout(predicate::str::contains("--force"))
            .stdout(predicate::str::contains("--json"))
            .stderr(predicate::str::is_empty());

        let out = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
        assert!(
            out.is_ascii(),
            "Help output for {bin} update must be pure ASCII"
        );
    }
}

#[test]
fn test_update_check_json_and_tty_output_modes() {
    let mut json_cmd = AssertCommand::cargo_bin("tk").expect("find tk binary");
    let json_assert = json_cmd
        .env("TRUSTKIT_UPDATE_SIMULATE", "update_available")
        .args(["update", "--check", "--json"])
        .assert()
        .success()
        .code(0);
    let stdout_json = String::from_utf8(json_assert.get_output().stdout.clone()).unwrap();
    assert!(
        !stdout_json.contains("null"),
        "JSON output must be strictly null-free: {stdout_json}"
    );
    let parsed: Value = serde_json::from_str(&stdout_json).expect("valid JSON");
    assert_eq!(parsed["verdict"], "PASS");
    assert_eq!(parsed["status"], "UPDATE_AVAILABLE");
    assert_eq!(parsed["check_only"], true);
    assert!(
        parsed["next_steps"]
            .as_array()
            .is_some_and(|arr| !arr.is_empty()),
        "next_steps must be populated"
    );

    let mut tty_cmd = AssertCommand::cargo_bin("trustkit").expect("find trustkit binary");
    tty_cmd
        .env_remove("TRUSTKIT_AGENT_MODE")
        .env("TRUSTKIT_UPDATE_SIMULATE", "updated")
        .args(["update"])
        .assert()
        .success()
        .code(0)
        .stdout(predicate::str::contains("[INFO]"))
        .stdout(predicate::str::contains("[UPDATE]"))
        .stdout(predicate::str::contains("[PASS]"))
        .stdout(predicate::str::contains("NEXT STEP:"));
}
