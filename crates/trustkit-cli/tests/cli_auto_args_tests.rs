// Tests for CLI auto command arguments and validation (TASK-001).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use clap::Parser;
use trustkit_cli::cli_contract::{AutoArgs, Cli, Command};

#[test]
fn test_task_001_parse_auto_with_issue() {
    let args = vec!["trustkit", "auto", "--issue", "42", "--json"];
    let cli = Cli::try_parse_from(args).expect("Should parse auto with issue");
    match cli.command {
        Command::Auto(auto_args) => {
            assert_eq!(auto_args.issue.as_deref(), Some("42"));
            assert_eq!(auto_args.spec, None);
            assert_eq!(auto_args.target, "develop");
            assert_eq!(auto_args.runtime, "auto");
            assert_eq!(auto_args.agent_cmd, "");
            assert!(auto_args.json);
            assert!(auto_args.validate().is_ok());
        }
        _ => panic!("Expected Command::Auto"),
    }
}

#[test]
fn test_task_001_parse_auto_with_spec_and_custom_options() {
    let args = vec![
        "trustkit",
        "auto",
        "--spec",
        "docs/specs/exemplo/spec.md",
        "--target",
        "main",
        "--runtime",
        "pty",
        "--agent-cmd",
        "gemini",
    ];
    let cli = Cli::try_parse_from(args).expect("Should parse auto with custom options");
    match cli.command {
        Command::Auto(auto_args) => {
            assert_eq!(
                auto_args.spec.as_deref(),
                Some("docs/specs/exemplo/spec.md")
            );
            assert_eq!(auto_args.issue, None);
            assert_eq!(auto_args.target, "main");
            assert_eq!(auto_args.runtime, "pty");
            assert_eq!(auto_args.agent_cmd, "gemini");
            assert!(!auto_args.json);
            assert!(auto_args.validate().is_ok());
        }
        _ => panic!("Expected Command::Auto"),
    }
}

#[test]
fn test_task_001_validate_auto_without_issue_or_spec_fails() {
    let auto_args = AutoArgs {
        issue: None,
        spec: None,
        target: "develop".to_string(),
        runtime: "auto".to_string(),
        agent_cmd: "".to_string(),
        json: false,
    };
    let result = auto_args.validate();
    assert!(result.is_err());
}
