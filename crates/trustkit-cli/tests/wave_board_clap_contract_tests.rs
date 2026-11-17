// Contract tests for Clap integration of 'board' and 'wave board' commands (TASK-001 / RF-001 / RN-001).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use clap::Parser;
use trustkit_cli::cli_contract::{
    BoardSubcommand, Cli, Command, WaveSubcommand,
};

#[test]
fn test_task_001_clap_contract_root_board_up() {
    let cli = Cli::try_parse_from(["tk", "board", "up", "--open", "--json"])
        .expect("Should parse tk board up --open --json");

    assert!(cli.is_json_mode());
    match cli.command {
        Command::Board(args) => match args.action {
            BoardSubcommand::Up(up_args) => {
                assert!(up_args.open);
                assert!(up_args.json);
            }
            _ => panic!("Expected BoardSubcommand::Up"),
        },
        _ => panic!("Expected Command::Board"),
    }
}

#[test]
fn test_task_001_clap_contract_root_board_down() {
    let cli = Cli::try_parse_from(["trustkit", "board", "down", "--json"])
        .expect("Should parse trustkit board down --json");

    assert!(cli.is_json_mode());
    match cli.command {
        Command::Board(args) => match args.action {
            BoardSubcommand::Down(down_args) => {
                assert!(down_args.json);
            }
            _ => panic!("Expected BoardSubcommand::Down"),
        },
        _ => panic!("Expected Command::Board"),
    }
}

#[test]
fn test_task_001_clap_contract_root_board_status() {
    let cli = Cli::try_parse_from(["tk", "board", "status"])
        .expect("Should parse tk board status");

    assert!(!cli.is_json_mode());
    match cli.command {
        Command::Board(args) => match args.action {
            BoardSubcommand::Status(status_args) => {
                assert!(!status_args.json);
            }
            _ => panic!("Expected BoardSubcommand::Status"),
        },
        _ => panic!("Expected Command::Board"),
    }
}

#[test]
fn test_task_001_clap_contract_wave_board_subcommands() {
    let cli = Cli::try_parse_from(["tk", "wave", "board", "up"])
        .expect("Should parse tk wave board up");

    match cli.command {
        Command::Wave(wave_args) => match wave_args.action {
            WaveSubcommand::Board(board_args) => match board_args.action {
                BoardSubcommand::Up(up_args) => {
                    assert!(!up_args.open);
                    assert!(!up_args.json);
                }
                _ => panic!("Expected BoardSubcommand::Up"),
            },
            _ => panic!("Expected WaveSubcommand::Board"),
        },
        _ => panic!("Expected Command::Wave"),
    }

    let cli_down = Cli::try_parse_from(["trustkit", "wave", "board", "down", "--json"])
        .expect("Should parse trustkit wave board down --json");
    assert!(cli_down.is_json_mode());

    let cli_status = Cli::try_parse_from(["tk", "wave", "board", "status"])
        .expect("Should parse tk wave board status");
    assert!(!cli_status.is_json_mode());
}

#[test]
fn test_task_001_clap_contract_serve_subcommand() {
    let cli = Cli::try_parse_from(["trustkit", "board", "serve", "--port", "9876"])
        .expect("Should parse trustkit board serve --port 9876");

    match cli.command {
        Command::Board(args) => match args.action {
            BoardSubcommand::Serve(serve_args) => {
                assert_eq!(serve_args.port, 9876);
            }
            _ => panic!("Expected BoardSubcommand::Serve"),
        },
        _ => panic!("Expected Command::Board"),
    }
}
