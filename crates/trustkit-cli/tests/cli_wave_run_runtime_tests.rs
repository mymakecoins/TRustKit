// Tests for CLI wave run with runtime options (TASK-007).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use clap::Parser;
use trustkit_cli::cli_contract::{Cli, Command, WaveSubcommand};

#[test]
fn test_parse_wave_run_with_runtime_auto_and_agent_cmd() {
    let args = vec![
        "trustkit",
        "wave",
        "run",
        "--wave",
        "1",
        "--runtime",
        "herdr",
        "--agent-cmd",
        "claude",
        "--json",
    ];

    let cli = Cli::try_parse_from(args).expect("Should parse wave run arguments");
    match cli.command {
        Command::Wave(wave_cmd) => match wave_cmd.action {
            WaveSubcommand::Run(run_args) => {
                assert_eq!(run_args.wave, 1);
                assert_eq!(run_args.runtime, "herdr");
                assert_eq!(run_args.agent_cmd, "claude");
                assert!(run_args.json);
            }
            _ => panic!("Expected WaveSubcommand::Run"),
        },
        _ => panic!("Expected Command::Wave"),
    }
}

#[test]
fn test_parse_wave_run_defaults() {
    let args = vec!["trustkit", "wave", "run", "--wave", "2"];
    let cli = Cli::try_parse_from(args).expect("Should parse default wave run arguments");
    match cli.command {
        Command::Wave(wave_cmd) => match wave_cmd.action {
            WaveSubcommand::Run(run_args) => {
                assert_eq!(run_args.wave, 2);
                assert_eq!(run_args.runtime, "auto");
                assert_eq!(run_args.agent_cmd, "");
                assert!(!run_args.json);
            }
            _ => panic!("Expected WaveSubcommand::Run"),
        },
        _ => panic!("Expected Command::Wave"),
    }
}
