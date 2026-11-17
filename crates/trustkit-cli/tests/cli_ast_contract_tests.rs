// Tests for CLI contracts: gauntlet ast-audit and context slice (TASK-013).

use clap::Parser;
use trustkit_cli::cli_contract::{
    Cli, Command, ContextArgs, ContextSliceArgs, ContextSubcommand, GauntletArgs,
    GauntletAstAuditArgs, GauntletSubcommand,
};

#[test]
fn test_parse_gauntlet_ast_audit_command_default() {
    let args = Cli::try_parse_from(["trustkit", "gauntlet", "ast-audit", "--json"])
        .expect("Failed to parse gauntlet ast-audit");

    assert!(args.is_json_mode());
    assert_eq!(
        args.command,
        Command::Gauntlet(GauntletArgs {
            action: GauntletSubcommand::AstAudit(GauntletAstAuditArgs {
                files: Vec::new(),
                json: true,
            })
        })
    );
}

#[test]
fn test_parse_gauntlet_audit_alias() {
    let args = Cli::try_parse_from(["trustkit", "gauntlet", "audit", "--json"])
        .expect("Failed to parse gauntlet audit alias");

    assert!(args.is_json_mode());
    assert_eq!(
        args.command,
        Command::Gauntlet(GauntletArgs {
            action: GauntletSubcommand::AstAudit(GauntletAstAuditArgs {
                files: Vec::new(),
                json: true,
            })
        })
    );
}

#[test]
fn test_parse_gauntlet_ast_audit_command_with_files() {
    let args = Cli::try_parse_from([
        "trustkit",
        "gauntlet",
        "ast-audit",
        "--files",
        "src/lib.rs",
        "tests/suite.rs",
    ])
    .expect("Failed to parse gauntlet ast-audit with files");

    assert!(!args.is_json_mode());
    assert_eq!(
        args.command,
        Command::Gauntlet(GauntletArgs {
            action: GauntletSubcommand::AstAudit(GauntletAstAuditArgs {
                files: vec!["src/lib.rs".to_string(), "tests/suite.rs".to_string()],
                json: false,
            })
        })
    );
}

#[test]
fn test_parse_context_slice_command() {
    let args = Cli::try_parse_from([
        "trustkit",
        "context",
        "slice",
        "--file",
        "crates/trustkit-core/src/lib.rs",
        "--budget",
        "150",
        "--symbols",
        "SddCycle,ArtifactSeal",
        "--json",
    ])
    .expect("Failed to parse context slice");

    assert!(args.is_json_mode());
    assert_eq!(
        args.command,
        Command::Context(ContextArgs {
            action: ContextSubcommand::Slice(ContextSliceArgs {
                file: Some("crates/trustkit-core/src/lib.rs".to_string()),
                files: Vec::new(),
                budget: Some(150),
                symbols: vec!["SddCycle".to_string(), "ArtifactSeal".to_string()],
                mode: None,
                hops: None,
                json: true,
            })
        })
    );
}

#[test]
fn test_parse_context_slice_command_with_mode_and_hops() {
    let args = Cli::try_parse_from([
        "trustkit",
        "context",
        "slice",
        "--file",
        "crates/trustkit-core/src/use_cases.rs",
        "--mode",
        "bfs",
        "--hops",
        "2",
        "--budget",
        "120",
        "--json",
    ])
    .expect("Failed to parse context slice with mode and hops");

    assert!(args.is_json_mode());
    assert_eq!(
        args.command,
        Command::Context(ContextArgs {
            action: ContextSubcommand::Slice(ContextSliceArgs {
                file: Some("crates/trustkit-core/src/use_cases.rs".to_string()),
                files: Vec::new(),
                budget: Some(120),
                symbols: Vec::new(),
                mode: Some("bfs".to_string()),
                hops: Some(2),
                json: true,
            })
        })
    );
}
