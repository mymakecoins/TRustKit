// Unit tests for AST Layer 0 domain entities (TASK-001).
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle I: Zero External Dependencies.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use super::*;

#[test]
fn test_ast_audit_verdict_clean_pass() {
    let verdict = AstAuditVerdict::CleanPass { violations: vec![] };
    assert!(verdict.is_pass());
    assert_eq!(verdict.violations().len(), 0);
}

#[test]
fn test_ast_audit_verdict_rejected_with_violations() {
    let violation = AstViolation::TestDilution(TestIntegrityViolation {
        file_path: "tests/auth_test.rs".to_string(),
        test_name: "test_login".to_string(),
        expected_assertions: 3,
        actual_assertions: 1,
    });

    let emoji_violation = AstViolation::ForbiddenEmoji {
        file_path: "src/lib.rs".to_string(),
        line: 42,
        character: "🚀".to_string(),
    };

    let ai_violation = AstViolation::AiTrailer {
        file_path: "README.md".to_string(),
        line: 10,
        trailer: "Co-Authored-By: Claude".to_string(),
    };

    let untyped_any = AstViolation::UntypedAny {
        file_path: "src/types.ts".to_string(),
        line: 15,
    };

    let residual_probe = AstViolation::ResidualDebugProbe {
        file_path: "src/main.rs".to_string(),
        line: 99,
        probe: "println!(\"[DEBUG-a4f2]\")".to_string(),
    };

    let verdict = AstAuditVerdict::Rejected {
        violations: vec![
            violation,
            emoji_violation,
            ai_violation,
            untyped_any,
            residual_probe,
        ],
    };

    assert!(!verdict.is_pass());
    assert_eq!(verdict.violations().len(), 5);
}

#[test]
fn test_context_slice_request_and_result() {
    let request = ContextSliceRequest {
        target_files: vec!["crates/trustkit-core/src/entities.rs".to_string()],
        symbol_filters: vec!["AstAuditVerdict".to_string()],
        max_line_budget: 100,
    };

    let result = ContextSliceResult {
        sliced_content: "pub enum AstAuditVerdict { ... }".to_string(),
        preserved_symbols: vec!["AstAuditVerdict".to_string()],
        total_lines: 10,
    };

    assert_eq!(request.target_files.len(), 1);
    assert_eq!(request.max_line_budget, 100);
    assert_eq!(result.total_lines, 10);
    assert_eq!(result.preserved_symbols[0], "AstAuditVerdict");
}

#[test]
fn test_task_slice_kind_variants_and_parsing() {
    assert_eq!(
        TaskSliceKind::VerticalTracerBullet.as_str(),
        "VERTICAL_TRACER_BULLET"
    );
    assert_eq!(
        TaskSliceKind::WideRefactorExpand.as_str(),
        "WIDE_REFACTOR_EXPAND"
    );
    assert_eq!(
        TaskSliceKind::WideRefactorMigrateBatch.as_str(),
        "WIDE_REFACTOR_MIGRATE_BATCH"
    );
    assert_eq!(
        TaskSliceKind::WideRefactorContract.as_str(),
        "WIDE_REFACTOR_CONTRACT"
    );

    assert!(!TaskSliceKind::VerticalTracerBullet.is_wide_refactor());
    assert!(TaskSliceKind::WideRefactorExpand.is_wide_refactor());
    assert!(TaskSliceKind::WideRefactorMigrateBatch.is_wide_refactor());
    assert!(TaskSliceKind::WideRefactorContract.is_wide_refactor());

    assert_eq!(
        TaskSliceKind::from_str_loose("expand"),
        TaskSliceKind::WideRefactorExpand
    );
    assert_eq!(
        TaskSliceKind::from_str_loose("migrate_batch"),
        TaskSliceKind::WideRefactorMigrateBatch
    );
    assert_eq!(
        TaskSliceKind::from_str_loose("contract"),
        TaskSliceKind::WideRefactorContract
    );
    assert_eq!(
        TaskSliceKind::from_str_loose("other"),
        TaskSliceKind::VerticalTracerBullet
    );
}

#[test]
fn test_task_blast_radius_report_risk_evaluation() {
    let report_low = TaskBlastRadiusReport::new(
        "TASK-001".to_string(),
        TaskSliceKind::VerticalTracerBullet,
        2,
        2,
        vec![],
    );
    assert!(!report_low.is_high_risk());

    let report_high = TaskBlastRadiusReport::new(
        "TASK-002".to_string(),
        TaskSliceKind::WideRefactorExpand,
        10,
        9,
        vec!["crates/trustkit-core/src/ports.rs".to_string()],
    );
    assert!(report_high.is_high_risk());
    assert_eq!(report_high.touched_high_degree_nodes.len(), 1);
}
