// Unit tests for PRD specification ingestion.
// Constitutional Principle III: Test-First, Anti-Cheat & Deterministic Equivalence.
// Constitutional Principle II: Absolute Prohibition of NULL.

use std::fs;
use tempfile::tempdir;
use trustkit_ast::PrdSpecificationIngestor;
use trustkit_core::entities::SpecLockState;
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::PrdSpecificationIngestorPort;

#[test]
fn test_extract_rn_rf_and_slug_from_h1() {
    let dir = tempdir().expect("create temp dir");
    let prd_path = dir.path().join("PRD_Sample.md");

    let prd_content = r#"# Central Auth Service

## 1. Regras de Negocio

| ID | Descricao |
| :--- | :--- |
| RN-001 | Todo usuario administrativo deve utilizar autenticacao em dois fatores |
| RN-002 | Senhas devem expirar apos 90 dias de inatividade |

## 2. Requisitos Funcionais

| ID | Descricao |
| :--- | :--- |
| RF-001 | Validar token TOTP de 6 digitos no login |
| RF-002 | Bloquear sessao e disparar e-mail de redefinicao de senha aos 90 dias |
"#;

    fs::write(&prd_path, prd_content).expect("write prd file");

    let ingestor = PrdSpecificationIngestor;
    let spec = ingestor
        .ingest_from_prd(prd_path.to_str().expect("valid utf-8 path"), None)
        .expect("ingestion should succeed");

    assert_eq!(spec.feature_slug, "central-auth-service");
    assert!(
        spec.spec_file_path
            .ends_with("central-auth-service-spec.md"),
        "expected path ending with central-auth-service-spec.md, got: {}",
        spec.spec_file_path
    );
    assert_eq!(spec.lock_state, SpecLockState::DraftUnverified);
    assert!(!spec.mermaid_content.is_empty());
    assert!(!spec.gherkin_scenarios.is_empty());

    // Traceability Matrix assertions
    assert_eq!(spec.traceability_matrix.len(), 2);
    assert_eq!(spec.traceability_matrix[0].business_rule_id, "RN-001");
    assert_eq!(spec.traceability_matrix[0].functional_req_id, "RF-001");
    assert_eq!(spec.traceability_matrix[1].business_rule_id, "RN-002");
    assert_eq!(spec.traceability_matrix[1].functional_req_id, "RF-002");
}

#[test]
fn test_custom_slug_override() {
    let dir = tempdir().expect("create temp dir");
    let prd_path = dir.path().join("PRD_Sample.md");

    let prd_content = r#"# Central Auth Service

| ID | Descricao |
| :--- | :--- |
| RN-001 | Regra 1 |
"#;

    fs::write(&prd_path, prd_content).expect("write prd file");

    let ingestor = PrdSpecificationIngestor;
    let spec = ingestor
        .ingest_from_prd(
            prd_path.to_str().expect("valid utf-8 path"),
            Some("custom-auth-spec"),
        )
        .expect("ingestion should succeed");

    assert_eq!(spec.feature_slug, "custom-auth-spec");
    assert!(
        spec.spec_file_path.ends_with("custom-auth-spec-spec.md")
            || spec.spec_file_path.ends_with("custom-auth-spec.md")
    );
}

#[test]
fn test_missing_file_fails_with_missing_business_rules() {
    let ingestor = PrdSpecificationIngestor;
    let result = ingestor.ingest_from_prd("/non/existent/path/prd.md", None);

    match result {
        Err(DomainViolation::MissingBusinessRules { file_path, .. }) => {
            assert_eq!(file_path, "/non/existent/path/prd.md");
        }
        other => panic!("expected MissingBusinessRules, got: {:?}", other),
    }
}

#[test]
fn test_missing_rules_in_prd_fails() {
    let dir = tempdir().expect("create temp dir");
    let prd_path = dir.path().join("PRD_Without_Rules.md");

    let prd_content = r#"# Empty PRD

Este documento nao possui nenhuma tabela com RN ou RF.
Apenas notas e comentarios gerais.
"#;

    fs::write(&prd_path, prd_content).expect("write prd file");

    let ingestor = PrdSpecificationIngestor;
    let result = ingestor.ingest_from_prd(prd_path.to_str().expect("valid utf-8 path"), None);

    match result {
        Err(DomainViolation::MissingBusinessRules { file_path, .. }) => {
            assert!(file_path.contains("PRD_Without_Rules.md"));
        }
        other => panic!("expected MissingBusinessRules, got: {:?}", other),
    }
}
