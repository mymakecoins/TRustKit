// Integration tests for 4-Layer MilestoneFeatureExtractor and Spec compatibility.
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use trustkit_ast::{
    MilestoneFeatureExtractor, PrdAdrMilestoneExtractor, SpecLinter, SpecMatrixAtomizer,
    SpecValidator,
};
use trustkit_core::entities::ArchitecturalTaskLayer;
use trustkit_core::ports::{FeatureExtractorPort, SpecValidatorPort};
use trustkit_core::use_cases::plan_features::render_feature_spec_markdown;
use trustkit_core::use_cases::plan_milestones::render_milestone_markdown;

const SAMPLE_PRD: &str = r#"# Product Requirements Document: Payment Engine

## 1. Visao e Proposito
Plataforma de pagamentos resiliente.

## 2. Capacidades e Requisitos
| ID | Titulo | Descricao |
| :--- | :--- | :--- |
| RN-001 | Pagamento Basico | Processar cobranca sincrona com cartao |
| RF-001 | Endpoint Charge | Receber payload de cobranca e retornar recibo |
| RN-002 | Validacao de Limite | Bloquear transacoes acima do teto diario |
| RF-002 | Regra Antifraude | Retornar erro estruturado quando exceder limite |
"#;

const SAMPLE_ADR: &str = r#"# Architecture Decision Record: Hexagonal Payments

## 1. Decisao de Arquitetura
Isolamento hexagonal com persistencia SQLite WAL e Circuit Breaker para alta disponibilidade.

## 2. Justificativa
Evitar acoplamento entre dominio de cobranca e gateways externos.
"#;

#[test]
fn test_extract_features_from_milestone_synthesizes_4_layers_and_passes_linter_and_atomizer() {
    let m_plan = PrdAdrMilestoneExtractor::extract_plan_from_content(
        SAMPLE_PRD, SAMPLE_ADR, "PRD.md", "ADR.md",
    )
    .expect("extract milestone plan");
    assert!(!m_plan.milestones.is_empty());

    let m01 = &m_plan.milestones[0];
    let extractor = MilestoneFeatureExtractor;
    let f_plan = extractor
        .extract_features_from_milestone(m01, "docs/features", "2026-11-25T00:00:00Z")
        .expect("extract feature plan");

    assert!(
        f_plan.features.len() >= 2,
        "Milestone with multiple capabilities should split into cohesive features (e.g. Caminho Feliz and Validacoes)"
    );

    for feat in &f_plan.features {
        assert_eq!(feat.task_blueprints.len(), 4);
        assert_eq!(
            feat.task_blueprints[0].layer,
            ArchitecturalTaskLayer::SchemaAndContract
        );
        assert_eq!(
            feat.task_blueprints[1].layer,
            ArchitecturalTaskLayer::CoreAndDomain
        );
        assert_eq!(
            feat.task_blueprints[2].layer,
            ArchitecturalTaskLayer::ApiUiAndAdapters
        );
        assert_eq!(
            feat.task_blueprints[3].layer,
            ArchitecturalTaskLayer::TelemetryAuditAndE2e
        );

        let spec_md = render_feature_spec_markdown(feat);

        // Must pass SpecLinter with CleanPass
        let lint_verdict = SpecLinter::lint_markdown(&spec_md);
        assert!(
            lint_verdict.is_clean(),
            "Generated feature spec must pass SpecLinter: {:?}",
            lint_verdict.violations()
        );

        // Must pass SpecValidator (Gate 2 Lock requirement)
        let validator = SpecValidator;
        let report = validator
            .validate_specification(&spec_md)
            .expect("Generated feature spec must pass SpecValidator");
        assert!(report.mermaid_valid);
        assert!(report.gherkin_valid);
        assert!(report.orphan_rules.is_empty());

        // Must atomize into 4 layered AtomicTaskDrafts with parallel Wave 2 (TASK-002 and TASK-003)
        let drafts = SpecMatrixAtomizer::atomize_markdown(&spec_md).expect("atomize feature spec");
        assert_eq!(drafts.len(), 4);
        assert_eq!(drafts[0].task_id, "TASK-001");
        assert!(drafts[0].depends_on.is_empty());
        assert_eq!(drafts[1].task_id, "TASK-002");
        assert_eq!(drafts[1].depends_on, vec!["TASK-001".to_string()]);
        assert_eq!(drafts[2].task_id, "TASK-003");
        assert_eq!(drafts[2].depends_on, vec!["TASK-001".to_string()]);
        assert_eq!(drafts[3].task_id, "TASK-004");
        assert_eq!(
            drafts[3].depends_on,
            vec!["TASK-002".to_string(), "TASK-003".to_string()]
        );
    }
}

#[test]
fn test_parse_milestone_from_markdown_file_roundtrip() {
    let m_plan = PrdAdrMilestoneExtractor::extract_plan_from_content(
        SAMPLE_PRD, SAMPLE_ADR, "PRD.md", "ADR.md",
    )
    .expect("extract milestone plan");
    let m01 = &m_plan.milestones[0];
    let md = render_milestone_markdown(m01);

    let parsed = MilestoneFeatureExtractor::parse_milestone_markdown(
        &md,
        "docs/milestones/M01-fundacao-e-modelos.md",
        "2026-11-25T00:00:00Z",
    )
    .expect("parse milestone markdown");
    assert_eq!(parsed.milestone_id, "M01");
    assert!(!parsed.capabilities.is_empty());
}

#[test]
fn test_stack_detection_and_scaffolder_for_typescript_pnpm_repo() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(
        root.join("package.json"),
        r#"{"name":"demo-app","devDependencies":{"vitest":"^2.0.0"}}"#,
    )
    .unwrap();
    std::fs::write(root.join("pnpm-lock.yaml"), "lockfileVersion: '9.0'\n").unwrap();

    let stack = MilestoneFeatureExtractor::detect_project_stack(root);
    assert_eq!(stack, trustkit_core::entities::ProjectStack::TypeScriptPnpm);

    let bps = MilestoneFeatureExtractor::build_layered_blueprints_for_stack(
        stack,
        "m01_caminho_feliz",
        &[],
        &[],
    );
    assert_eq!(bps.len(), 4);
    assert!(bps[0].target_test_file.ends_with(".test.ts"));
    assert!(!bps[0].target_test_file.contains("crates/trustkit-core"));

    let scaffolded = trustkit_ast::RedTestScaffolder::scaffold(
        root,
        &trustkit_ast::ScaffoldingTarget {
            task_id: "TASK-001".to_string(),
            target_test_file: bps[0].target_test_file.clone(),
            criteria_summary: "Schema contract".to_string(),
        },
    )
    .expect("scaffold ts test");
    let content = std::fs::read_to_string(scaffolded).unwrap();
    assert!(content.contains("from 'vitest'"));
    assert!(content.contains("TASK RED ASSERTION: TASK-001 not implemented yet"));
}
