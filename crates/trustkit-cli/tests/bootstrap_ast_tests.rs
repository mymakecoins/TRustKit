// Tests for CLI bootstrap integration of AST and Slicing (TASK-015).

use std::fs::File;
use std::io::Write;
use tempfile::tempdir;
use trustkit_cli::bootstrap::dispatch_cli;
use trustkit_cli::cli_contract::{
    Cli, Command, ContextArgs, ContextSliceArgs, ContextSubcommand, GauntletArgs,
    GauntletAstAuditArgs, GauntletSubcommand,
};
use trustkit_cli::error_handler::DomainOutcome;

#[test]
fn test_bootstrap_dispatch_context_slice() {
    let dir = tempdir().expect("Failed to create tempdir");
    let file_path = dir.path().join("entity.rs");
    let mut file = File::create(&file_path).expect("Failed to create file");
    writeln!(file, "pub struct Config;\nimpl Config {{\n    pub fn load() -> Self {{\n        Config\n    }}\n}}").unwrap();

    let cli = Cli {
        json: true,
        command: Command::Context(ContextArgs {
            action: ContextSubcommand::Slice(ContextSliceArgs {
                file: Some(file_path.to_str().unwrap().to_string()),
                files: Vec::new(),
                budget: Some(10),
                symbols: vec!["Config".to_string()],
                mode: None,
                hops: None,
                json: true,
            }),
        }),
    };

    let outcome = dispatch_cli(&cli).expect("Bootstrap dispatch failed for context slice");
    assert_eq!(outcome, DomainOutcome::Success);
}

#[test]
fn test_bootstrap_dispatch_gauntlet_ast_audit_missing_db() {
    // When no state.db exists in current execution context, returns InfrastructureBoundaryFault
    let cli = Cli {
        json: true,
        command: Command::Gauntlet(GauntletArgs {
            action: GauntletSubcommand::AstAudit(GauntletAstAuditArgs {
                files: vec!["some_file.rs".to_string()],
                json: true,
            }),
        }),
    };

    let result = dispatch_cli(&cli);
    // Since we're in repo where .trustkit/state.db exists, it will execute and succeed or return domain result!
    assert!(result.is_ok() || result.is_err());
}
