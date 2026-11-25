// Integration tests for CLI `tk features` (`plan`, `list`, `inspect`) and `tk auto` (`--feature`, `--milestone`, `--issue`, `--spec`, `--specs`).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use clap::Parser;
use std::fs;
use tempfile::tempdir;
use trustkit_cli::cli_contract::{
    Cli, Command, FeaturesArgs, FeaturesInspectArgs, FeaturesListArgs, FeaturesPlanArgs,
    FeaturesSubcommand, MilestonesPlanArgs,
};
use trustkit_cli::commands::{features_cmd, milestones_cmd};
use trustkit_core::ports::FeatureRepositoryPort;
use trustkit_db::SqliteWalRepositoryAdapter;

const SAMPLE_PRD: &str = r#"# Product Requirements Document: Payment Platform

## 1. Visao Geral
Sistema de pagamentos para comercio eletronico.

## 2. Capacidades de Negocio
| ID | Titulo | Descricao |
| :--- | :--- | :--- |
| RN-001 | Pagamento Basico | Processar cobranca de cartao de credito |
| RF-001 | API de Cobranca | Expor endpoint POST /charges |
| RN-002 | Validacao de Limite | Bloquear valores acima do limite diario |
| RF-002 | Regra de Teto | Retornar erro 422 quando ultrapassar teto |
"#;

const SAMPLE_ADR: &str = r#"# Architecture Decision Record: Hexagonal Engine

## 1. Decisao de Arquitetura
Isolamento hexagonal com persistencia SQLite WAL e Circuit Breaker.

## 2. Justificativa
Garantir testabilidade deterministica e resiliencia sob falha.
"#;

#[test]
fn test_cli_parse_features_and_singular_alias_feature() {
    let plan_cli = Cli::try_parse_from([
        "trustkit",
        "features",
        "plan",
        "--milestone",
        "M01",
        "--output-dir",
        "docs/features",
        "--json",
    ])
    .expect("parse features plan");

    match plan_cli.command {
        Command::Features(FeaturesArgs {
            action: FeaturesSubcommand::Plan(args),
        }) => {
            assert_eq!(args.milestone, "M01");
            assert_eq!(args.output_dir, "docs/features");
            assert!(args.json);
        }
        _ => panic!("Expected Command::Features Plan"),
    }

    let alias_list = Cli::try_parse_from([
        "trustkit",
        "feature",
        "list",
        "--milestone",
        "M01",
        "--json",
    ])
    .expect("parse feature list via singular alias");

    match alias_list.command {
        Command::Features(FeaturesArgs {
            action: FeaturesSubcommand::List(args),
        }) => {
            assert_eq!(args.milestone, Some("M01".to_string()));
            assert!(args.json);
        }
        _ => panic!("Expected Command::Features List"),
    }

    let alias_inspect =
        Cli::try_parse_from(["trustkit", "feature", "inspect", "M01-F01", "--json"])
            .expect("parse feature inspect via singular alias");

    match alias_inspect.command {
        Command::Features(FeaturesArgs {
            action: FeaturesSubcommand::Inspect(args),
        }) => {
            assert_eq!(args.feature_id, "M01-F01");
            assert!(args.json);
        }
        _ => panic!("Expected Command::Features Inspect"),
    }
}

#[test]
fn test_cli_parse_auto_retrocompatibility_and_new_flags() {
    // 1. --issue
    let auto_issue = Cli::try_parse_from(["trustkit", "auto", "--issue", "42", "--json"])
        .expect("parse --issue");
    if let Command::Auto(args) = auto_issue.command {
        assert!(args.validate().is_ok());
        assert_eq!(args.resolved_issue(), Some("42".to_string()));
    } else {
        panic!("Expected Command::Auto");
    }

    // 2. --spec
    let auto_spec = Cli::try_parse_from([
        "trustkit",
        "auto",
        "--spec",
        "docs/specs/001/spec.md",
        "--json",
    ])
    .expect("parse --spec");
    if let Command::Auto(args) = auto_spec.command {
        assert!(args.validate().is_ok());
        assert_eq!(
            args.resolved_spec(),
            Some("docs/specs/001/spec.md".to_string())
        );
    } else {
        panic!("Expected Command::Auto");
    }

    // 3. --specs (plural alias for high-throughput teams)
    let auto_specs = Cli::try_parse_from([
        "trustkit",
        "auto",
        "--specs",
        "docs/specs/002/spec.md",
        "--json",
    ])
    .expect("parse --specs alias");
    if let Command::Auto(args) = auto_specs.command {
        assert!(args.validate().is_ok());
        assert_eq!(
            args.resolved_spec(),
            Some("docs/specs/002/spec.md".to_string())
        );
    } else {
        panic!("Expected Command::Auto");
    }

    // 4. --feature
    let auto_feature = Cli::try_parse_from(["trustkit", "auto", "--feature", "M01-F01", "--json"])
        .expect("parse --feature");
    if let Command::Auto(args) = auto_feature.command {
        assert!(args.validate().is_ok());
        assert_eq!(args.feature, Some("M01-F01".to_string()));
    } else {
        panic!("Expected Command::Auto");
    }

    // 5. --milestone
    let auto_ms = Cli::try_parse_from(["trustkit", "auto", "--milestone", "M01", "--json"])
        .expect("parse --milestone");
    if let Command::Auto(args) = auto_ms.command {
        assert!(args.validate().is_ok());
        assert_eq!(args.milestone, Some("M01".to_string()));
    } else {
        panic!("Expected Command::Auto");
    }

    // 6. Positional deterministic resolution (Feature ID vs Milestone ID vs Issue ID)
    let pos_feat =
        Cli::try_parse_from(["trustkit", "auto", "M01-F01"]).expect("parse positional feature");
    if let Command::Auto(args) = pos_feat.command {
        assert_eq!(args.resolved_feature(), Some("M01-F01".to_string()));
        assert_eq!(args.resolved_milestone(), None);
        assert_eq!(args.resolved_issue(), None);
    } else {
        panic!("Expected Command::Auto");
    }

    let pos_ms =
        Cli::try_parse_from(["trustkit", "auto", "M01"]).expect("parse positional milestone");
    if let Command::Auto(args) = pos_ms.command {
        assert_eq!(args.resolved_milestone(), Some("M01".to_string()));
        assert_eq!(args.resolved_feature(), None);
        assert_eq!(args.resolved_issue(), None);
    } else {
        panic!("Expected Command::Auto");
    }
}

#[test]
fn test_end_to_end_milestone_to_features_plan_list_and_inspect() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let prd_path = root.join("PRD.md");
    let adr_path = root.join("ADR.md");
    let ms_dir = root.join("milestones");
    let feat_dir = root.join("features");
    let db_path = root.join("state.db");

    fs::write(&prd_path, SAMPLE_PRD).unwrap();
    fs::write(&adr_path, SAMPLE_ADR).unwrap();

    // Step 1: tk milestone plan
    let ms_args = MilestonesPlanArgs {
        prd: prd_path.to_string_lossy().to_string(),
        adr: adr_path.to_string_lossy().to_string(),
        output_dir: ms_dir.to_string_lossy().to_string(),
        json: true,
    };
    milestones_cmd::execute_plan_with_db(&ms_args, true, &db_path).expect("plan milestones");

    // Step 2: tk features plan --milestone M01
    let feat_args = FeaturesPlanArgs {
        milestone: "M01".to_string(),
        output_dir: feat_dir.to_string_lossy().to_string(),
        json: true,
    };
    features_cmd::execute_plan_with_db(&feat_args, true, &db_path).expect("plan features");

    let repo = SqliteWalRepositoryAdapter::open(&db_path).unwrap();
    let features = repo.list_features(Some("M01")).unwrap();
    assert!(!features.is_empty());
    for f in &features {
        assert!(std::path::Path::new(&f.spec_file_path).exists());
        assert_eq!(f.task_blueprints.len(), 4);
    }

    // Step 3: tk features list --milestone M01
    let list_args = FeaturesListArgs {
        milestone: Some("M01".to_string()),
        json: true,
    };
    features_cmd::execute_list_with_db(&list_args, true, &db_path).expect("list features");

    // Step 4: tk features inspect M01-F01
    let inspect_args = FeaturesInspectArgs {
        feature_id: "M01-F01".to_string(),
        json: true,
    };
    features_cmd::execute_inspect_with_db(&inspect_args, true, &db_path).expect("inspect feature");
}
