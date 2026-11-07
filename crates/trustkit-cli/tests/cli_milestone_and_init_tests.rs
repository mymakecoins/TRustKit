// Integration tests for CLI milestone commands and init with --milestone.
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use std::fs;
use tempfile::tempdir;
use trustkit_cli::cli_contract::{
    Cli, Command, InitArgs, MilestonesArgs, MilestonesInspectArgs, MilestonesListArgs,
    MilestonesPlanArgs, MilestonesSubcommand,
};
use trustkit_cli::commands::{init_cmd, milestones_cmd};
use trustkit_core::errors::DomainViolation;

const SAMPLE_PRD: &str = r#"# Product Requirements Document: Demo System

## 1. Visao Geral
Este e um sistema de teste.

## 2. Capacidades de Negocio
- CAP-01: Autenticacao de usuarios com JWT.
- CAP-02: Gestao de sessoes ativas e renovacao.
- CAP-03: Auditoria transacional em tempo real.
- CAP-04: Painel de metricas e monitoramento.
- CAP-05: Exportacao de relatorios em PDF.
"#;

const SAMPLE_ADR: &str = r#"# Architecture Decision Record: Demo Architecture

## 1. Status
Aprovado

## 2. Decisao de Arquitetura
Utilizacao de arquitetura hexagonal com isolamento estrito de dominios.

## 3. Justificativa
Garantir desacoplamento completo entre adapters e logica pura de dominio.
"#;

#[test]
fn test_milestones_plan_and_disk_artifacts() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let prd_path = root.join("PRD.md");
    let adr_path = root.join("ADR.md");
    let out_dir = root.join("milestones");
    let db_path = root.join("state.db");

    fs::write(&prd_path, SAMPLE_PRD).unwrap();
    fs::write(&adr_path, SAMPLE_ADR).unwrap();

    let plan_args = MilestonesPlanArgs {
        prd: prd_path.to_string_lossy().to_string(),
        adr: adr_path.to_string_lossy().to_string(),
        output_dir: out_dir.to_string_lossy().to_string(),
        json: true,
    };

    let outcome = milestones_cmd::execute_plan_with_db(&plan_args, true, &db_path);
    assert!(
        outcome.is_ok(),
        "Planning should succeed: {:?}",
        outcome.err()
    );

    // Verify disk files
    assert!(out_dir.join("INDEX.md").exists());
    assert!(out_dir.join("M01-fundacao-e-modelos.md").exists());

    // Verify list command
    let list_args = MilestonesListArgs {
        plan_id: None,
        json: true,
    };
    let list_res = milestones_cmd::execute_list_with_db(&list_args, true, &db_path);
    assert!(list_res.is_ok());

    // Verify inspect command
    let inspect_args = MilestonesInspectArgs {
        milestone_id: "M01".to_string(),
        json: true,
    };
    let inspect_res = milestones_cmd::execute_inspect_with_db(&inspect_args, true, &db_path);
    assert!(inspect_res.is_ok());
}

#[test]
fn test_milestones_plan_fails_on_missing_files() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("state.db");
    let plan_args = MilestonesPlanArgs {
        prd: "nonexistent_prd.md".to_string(),
        adr: "nonexistent_adr.md".to_string(),
        output_dir: "docs/milestones".to_string(),
        json: true,
    };

    let res = milestones_cmd::execute_plan_with_db(&plan_args, true, &db_path);
    assert!(res.is_err());
    match res.unwrap_err() {
        DomainViolation::InfrastructureBoundaryFault { subsystem, .. } => {
            assert_eq!(subsystem, "milestone_extractor");
        }
        other => panic!("Expected InfrastructureBoundaryFault, got {:?}", other),
    }
}

#[test]
fn test_milestones_inspect_fails_on_missing_id() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let prd_path = root.join("PRD.md");
    let adr_path = root.join("ADR.md");
    let db_path = root.join("state.db");

    fs::write(&prd_path, SAMPLE_PRD).unwrap();
    fs::write(&adr_path, SAMPLE_ADR).unwrap();

    let plan_args = MilestonesPlanArgs {
        prd: prd_path.to_string_lossy().to_string(),
        adr: adr_path.to_string_lossy().to_string(),
        output_dir: root.join("ms").to_string_lossy().to_string(),
        json: true,
    };
    milestones_cmd::execute_plan_with_db(&plan_args, true, &db_path).unwrap();

    let inspect_args = MilestonesInspectArgs {
        milestone_id: "M99".to_string(),
        json: true,
    };
    let res = milestones_cmd::execute_inspect_with_db(&inspect_args, true, &db_path);
    assert!(res.is_err());
    match res.unwrap_err() {
        DomainViolation::InfrastructureBoundaryFault { subsystem, .. } => {
            assert_eq!(subsystem, "milestones_inspect");
        }
        other => panic!("Expected InfrastructureBoundaryFault, got {:?}", other),
    }
}

#[test]
fn test_init_fails_on_missing_milestone_file() {
    let init_args = InitArgs {
        issue: None,
        from_prd: None,
        milestone: Some("nonexistent_milestone.md".to_string()),
        json: true,
    };

    let res = init_cmd::execute(&init_args, true);
    assert!(res.is_err());
    match res.unwrap_err() {
        DomainViolation::InfrastructureBoundaryFault { subsystem, .. } => {
            assert_eq!(subsystem, "init_milestone");
        }
        other => panic!("Expected InfrastructureBoundaryFault, got {:?}", other),
    }
}

#[test]
fn test_cli_dispatch_milestones() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let prd_path = root.join("PRD.md");
    let adr_path = root.join("ADR.md");
    fs::write(&prd_path, SAMPLE_PRD).unwrap();
    fs::write(&adr_path, SAMPLE_ADR).unwrap();

    let ms_dir = root.join("ms");
    let ms_str = ms_dir.to_str().unwrap();
    let prd_str = prd_path.to_str().unwrap();
    let adr_str = adr_path.to_str().unwrap();

    let args = [
        "trustkit",
        "milestones",
        "plan",
        "--prd",
        prd_str,
        "--adr",
        adr_str,
        "--output-dir",
        ms_str,
        "--json",
    ];

    use clap::Parser;
    let cli = Cli::try_parse_from(args).unwrap();
    assert!(cli.is_json_mode());
    match cli.command {
        Command::Milestones(MilestonesArgs {
            action: MilestonesSubcommand::Plan(_),
        }) => {}
        _ => panic!("Expected milestones plan command"),
    }
}
