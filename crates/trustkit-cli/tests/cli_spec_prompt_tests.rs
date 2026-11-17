// Integration tests for `trustkit spec prompt` CLI command (TASK-003).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use clap::Parser;
use std::path::Path;
use trustkit_cli::cli_contract::{Cli, Command, SpecArgs, SpecPromptArgs, SpecSubcommand};
use trustkit_cli::commands::spec_prompt_cmd;

#[test]
fn test_cli_contract_parse_spec_prompt() {
    let args = vec![
        "trustkit",
        "spec",
        "prompt",
        "--idea",
        "Autenticacao biometrica",
        "--context-hops",
        "3",
        "--budget",
        "3000",
        "--json",
    ];

    let cli = Cli::try_parse_from(args).expect("parse spec prompt");
    match cli.command {
        Command::Spec(SpecArgs {
            action: SpecSubcommand::Prompt(cmd_args),
        }) => {
            assert_eq!(cmd_args.idea, "Autenticacao biometrica");
            assert_eq!(cmd_args.context_hops, 3);
            assert_eq!(cmd_args.budget, 3000);
            assert!(cmd_args.json);
        }
        other => panic!("Expected Spec prompt command, got {other:?}"),
    }
}

#[test]
fn test_spec_prompt_execution_dual_mode_without_null() {
    let args = SpecPromptArgs {
        idea: "Motor de busca deterministico".to_string(),
        context_hops: 2,
        budget: 4000,
        design_it_twice: false,
        json: true,
    };

    let dummy_root = Path::new("target");
    let outcome = spec_prompt_cmd::execute_from_root(&args, true, dummy_root);
    assert!(outcome.is_ok());
}
