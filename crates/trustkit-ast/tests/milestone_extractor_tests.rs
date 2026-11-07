// Unit tests for PRD and ADR Milestone Extractor (TASK-001).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use std::fs;
use tempfile::tempdir;
use trustkit_ast::PrdAdrMilestoneExtractor;
use trustkit_core::ports::MilestoneExtractorPort;

#[test]
fn test_parse_blocks_line_tracking() {
    let content = "# Titulo Principal\n\n## 1. Visao\n- Item 1\n- Item 2\n\n| ID | Descricao |\n| :--- | :--- |\n| RN-001 | Regra 1 |\n";
    let blocks = PrdAdrMilestoneExtractor::parse_blocks(content, "docs/PRD.md");

    assert!(!blocks.is_empty());
    let h1 = blocks
        .iter()
        .find(|b| b.title == "Titulo Principal")
        .unwrap();
    assert_eq!(h1.start_line, 1);
    assert_eq!(h1.end_line, 1);

    let h2 = blocks.iter().find(|b| b.title == "1. Visao").unwrap();
    assert_eq!(h2.start_line, 3);

    let table_row = blocks.iter().find(|b| b.title == "RN-001").unwrap();
    assert_eq!(table_row.start_line, 9);
    assert!(table_row.is_table_row);
}

#[test]
fn test_extract_plan_from_content_nominal() {
    let prd_content = r#"# Sample System PRD

## 1. Visao do Produto
Fornecer um motor confiavel e rapido.

## 2. Requisitos de Negocio

| ID | Descricao |
| :--- | :--- |
| RN-001 | Autenticacao multifator obrigatoria |
| RN-002 | Auditoria de operacoes criticas |
| RF-001 | Login com OAuth2 e TOTP |
| RF-002 | Exportacao de relatorios criptografados |
"#;

    let adr_content = r#"# ADR-001: Decisoes Arquiteturais

## 1. Questao ou Problema
Escalabilidade e baixo acoplamento.

## 2. Decisao
Adotar arquitetura hexagonal com Rust puro no core.

## 3. Justificativa
Velocidade sub-milissegundo e isolamento contra quebras.
"#;

    let plan = PrdAdrMilestoneExtractor::extract_plan_from_content(
        prd_content,
        adr_content,
        "docs/PRD.md",
        "docs/ADR.md",
    )
    .expect("extraction should succeed");

    assert!(!plan.milestones.is_empty());
    let m1 = &plan.milestones[0];
    assert_eq!(m1.milestone_id, "M01");
    assert!(!m1.capabilities.is_empty());

    // Check line citations in capabilities
    let cap1 = &m1.capabilities[0];
    assert_eq!(cap1.citation.file_path, "docs/PRD.md");
    assert!(cap1.citation.start_line > 0);
    assert_eq!(
        cap1.citation.format_citation(),
        format!("docs/PRD.md#L{}", cap1.citation.start_line)
    );

    // Check technical constraints citations
    assert!(!m1.technical_constraints.is_empty());
    let tc1 = &m1.technical_constraints[0];
    assert_eq!(tc1.citation.file_path, "docs/ADR.md");
    assert!(tc1.citation.start_line > 0);
}

#[test]
fn test_extract_milestone_plan_from_files() {
    let dir = tempdir().expect("create temp dir");
    let prd_file = dir.path().join("PRD.md");
    let adr_file = dir.path().join("ADR.md");

    fs::write(
        &prd_file,
        "# Test PRD\n\n## 1. Proposito\nConstruir motor de milestones.\n\n| ID | Descricao |\n| :--- | :--- |\n| RN-001 | Regra basica de planejamento |\n",
    )
    .expect("write prd");

    fs::write(
        &adr_file,
        "# Test ADR\n\n## 1. Decisao\nUtilizar crates modulares.\n",
    )
    .expect("write adr");

    let extractor = PrdAdrMilestoneExtractor;
    let plan = extractor
        .extract_milestone_plan(prd_file.to_str().unwrap(), adr_file.to_str().unwrap())
        .expect("plan extraction should succeed");

    assert_eq!(plan.prd_path, prd_file.to_str().unwrap());
    assert_eq!(plan.adr_path, adr_file.to_str().unwrap());
    assert_eq!(plan.milestones.len(), 1);
    assert_eq!(plan.milestones[0].milestone_id, "M01");
}

#[test]
fn test_extract_fails_on_missing_files() {
    let extractor = PrdAdrMilestoneExtractor;
    let result = extractor.extract_milestone_plan("non_existent_prd.md", "non_existent_adr.md");
    assert!(result.is_err());
}
