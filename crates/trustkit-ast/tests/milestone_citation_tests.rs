// Integration tests for Milestone Citation Resolver and Validator (TASK-003).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use std::fs;
use tempfile::tempdir;
use trustkit_ast::milestone_citation::CitationValidator;
use trustkit_core::entities::{
    Milestone, MilestoneCapability, MilestoneStatus, SourceCitation, TechnicalConstraint,
};

#[test]
fn test_parse_citation_string_nominal() {
    let parsed = CitationValidator::parse_citation_string("docs/PRD_TRustKit.md#L10-L25")
        .expect("Should parse valid range citation");
    assert_eq!(parsed.file_path, "docs/PRD_TRustKit.md");
    assert_eq!(parsed.start_line, 10);
    assert_eq!(parsed.end_line, 25);

    let single = CitationValidator::parse_citation_string("docs/ADR.md#L42")
        .expect("Should parse single line citation");
    assert_eq!(single.file_path, "docs/ADR.md");
    assert_eq!(single.start_line, 42);
    assert_eq!(single.end_line, 42);
}

#[test]
fn test_parse_citation_invalid_format() {
    assert!(CitationValidator::parse_citation_string("invalid_format_without_hash").is_err());
    assert!(CitationValidator::parse_citation_string("docs/file.md#abc").is_err());
    assert!(CitationValidator::parse_citation_string("docs/file.md#L20-L10").is_err());
}

#[test]
fn test_validate_citation_file_bounds() {
    let dir = tempdir().expect("tempdir");
    let test_file = dir.path().join("sample.md");
    let content = "Linha 1\nLinha 2\nLinha 3\nLinha 4\nLinha 5\n";
    fs::write(&test_file, content).expect("write file");

    let valid_citation = SourceCitation::new(test_file.to_str().unwrap(), 2, 4, "Secao");
    let result = CitationValidator::validate_citation(&valid_citation, dir.path())
        .expect("Should validate within bounds");
    assert!(result.is_valid);
    assert_eq!(result.total_file_lines, 5);
    assert_eq!(result.extracted_lines.len(), 3);
    assert_eq!(result.extracted_lines[0], "Linha 2");
    assert_eq!(result.extracted_lines[2], "Linha 4");

    // Exceeding bounds
    let invalid_citation = SourceCitation::new(test_file.to_str().unwrap(), 3, 10, "Secao");
    let err = CitationValidator::validate_citation(&invalid_citation, dir.path());
    assert!(err.is_err());

    // Non-existent file
    let missing_citation = SourceCitation::new("non_existent.md", 1, 2, "Secao");
    let err_missing = CitationValidator::validate_citation(&missing_citation, dir.path());
    assert!(err_missing.is_err());
}

#[test]
fn test_format_citation_block() {
    let dir = tempdir().expect("tempdir");
    let test_file = dir.path().join("doc.md");
    let content = "# Titulo\nTexto da especificacao na linha 2.\nMais texto na linha 3.\n";
    fs::write(&test_file, content).expect("write");

    let citation = SourceCitation::new(test_file.to_str().unwrap(), 2, 3, "Especificacao");
    let block = CitationValidator::format_citation_block(&citation, dir.path())
        .expect("Should format citation block");

    assert!(block.markdown.contains("doc.md#L2-L3"));
    assert!(block.markdown.contains("Especificacao"));
    assert!(
        block
            .markdown
            .contains("Texto da especificacao na linha 2.")
    );
}

#[test]
fn test_validate_milestone_all_citations() {
    let dir = tempdir().expect("tempdir");
    let prd_file = dir.path().join("PRD.md");
    let adr_file = dir.path().join("ADR.md");
    fs::write(&prd_file, "RF-001: Gestao de tarefas\nRF-002: Metricas\n").expect("prd");
    fs::write(&adr_file, "RN-001: SQLite WAL\nRN-002: Zero Panic\n").expect("adr");

    let cap = MilestoneCapability::new(
        "CAP-001",
        "Gestao de Tarefas",
        "Atende RF-001",
        SourceCitation::new(prd_file.to_str().unwrap(), 1, 1, "Requisitos"),
    );
    let con = TechnicalConstraint::new(
        "CON-001",
        "Persistencia SQLite",
        "Restricao arquitetural de WAL",
        SourceCitation::new(adr_file.to_str().unwrap(), 1, 1, "Arquitetura"),
    );

    let milestone = Milestone {
        milestone_id: "M01".to_string(),
        plan_id: "PLAN-1".to_string(),
        sequence_index: 1,
        title: "Fundacao".to_string(),
        slug: "fundacao".to_string(),
        file_path: "docs/milestones/M01-fundacao.md".to_string(),
        goal: "Objetivo de fundacao".to_string(),
        dependencies: vec![],
        capabilities: vec![cap],
        technical_constraints: vec![con],
        status: MilestoneStatus::Planned,
        created_at: "2026-11-06T00:00:00Z".to_string(),
        updated_at: "2026-11-06T00:00:00Z".to_string(),
    };

    let report = CitationValidator::validate_milestone(&milestone, dir.path())
        .expect("Should validate milestone");
    assert_eq!(report.total_citations, 2);
    assert_eq!(report.valid_citations, 2);
    assert!(report.all_valid);
}
