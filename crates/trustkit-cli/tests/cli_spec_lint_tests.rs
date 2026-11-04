// Integration tests for `trustkit spec lint` CLI command (TASK-005).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use clap::Parser;
use std::fs;
use std::path::Path;
use tempfile::tempdir;
use trustkit_cli::cli_contract::{Cli, Command, SpecArgs, SpecLintArgs, SpecSubcommand};
use trustkit_cli::commands::spec_lint_cmd;
use trustkit_core::errors::DomainViolation;

const VALID_SPEC: &str = r#"# Especificacao: 009-sample-feature

## 1. Product Trio

- **PM**: Regras de negocio definidas.
- **Tech Lead**: Arquitetura hexagonal.
- **Design / QA**: Criterios de aceitacao BDD.

## 2. Diagrama de Fluxo (Mermaid)

```mermaid
flowchart LR
    Node1[Entrada] --> Node2[Saida]
```

## 3. Criterios de Aceitacao (Gherkin)

### Cenario 1: Execucao Nominal

- **Dado** ambiente pronto
- **Quando** executa
- **Entao** conclui com sucesso

## 4. Matriz de Rastreabilidade

| Regra Negocio | Requisito Funcional | Tarefa | No Mermaid | Teste Unitario / Integracao | Evidencia / Verificacao |
| :--- | :--- | :--- | :--- | :--- | :--- |
| RN-001 | RF-001 | TASK-001 | Node1 | tests/unit.rs | EV-001.json |
"#;

#[test]
fn test_cli_contract_parse_spec_lint() {
    let args = vec![
        "trustkit",
        "spec",
        "lint",
        "--file",
        "specs/008-feature/spec.md",
        "--json",
    ];

    let cli = Cli::try_parse_from(args).expect("parse spec lint");
    match cli.command {
        Command::Spec(SpecArgs {
            action: SpecSubcommand::Lint(cmd_args),
        }) => {
            assert_eq!(cmd_args.file, Some("specs/008-feature/spec.md".to_string()));
            assert!(cmd_args.json);
        }
        other => panic!("Expected Spec lint command, got {other:?}"),
    }
}

#[test]
fn test_spec_lint_execution_clean_pass() {
    let dir = tempdir().expect("tempdir");
    let spec_path = dir.path().join("spec.md");
    fs::write(&spec_path, VALID_SPEC).expect("write valid spec");

    let args = SpecLintArgs {
        file: Some(spec_path.to_str().unwrap().to_string()),
        json: true,
    };

    let dummy_db = Path::new("target/dummy_state.db");
    let outcome = spec_lint_cmd::execute_with_db(&args, true, dummy_db);
    assert!(outcome.is_ok());
}

#[test]
fn test_spec_lint_execution_structural_violation_returns_incomplete_code() {
    let dir = tempdir().expect("tempdir");
    let spec_path = dir.path().join("broken_spec.md");
    fs::write(
        &spec_path,
        "# Especificacao: Incompleta\nSem secoes obrigatorias\n",
    )
    .expect("write broken");

    let args = SpecLintArgs {
        file: Some(spec_path.to_str().unwrap().to_string()),
        json: true,
    };

    let dummy_db = Path::new("target/dummy_state.db");
    let outcome = spec_lint_cmd::execute_with_db(&args, true, dummy_db);
    match outcome {
        Err(DomainViolation::SpecOrTraceabilityIncomplete { .. }) => (),
        other => panic!("Expected SpecOrTraceabilityIncomplete, got {other:?}"),
    }
}
