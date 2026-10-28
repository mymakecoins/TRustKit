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

    let verdict = AstAuditVerdict::Rejected {
        violations: vec![violation, emoji_violation, ai_violation, untyped_any],
    };

    assert!(!verdict.is_pass());
    assert_eq!(verdict.violations().len(), 4);
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
