// Unit tests for TestIntegrityAuditor (TASK-007).
// Constitutional Principle III: Test-First & Anti-Cheat.

use trustkit_ast::TestIntegrityAuditor;
use trustkit_core::entities::{ArtifactCategory, ArtifactSeal, AstViolation};
use trustkit_core::special_cases::ContractUnlockState;

#[test]
fn test_auditor_detects_removed_assertions_in_rust() {
    let auditor = TestIntegrityAuditor::new();
    let file_path = "tests/auth_test.rs";
    let code_with_less_assertions = r#"
        #[test]
        fn test_login() {
            assert_eq!(1, 1);
        }
    "#;

    let seal = ArtifactSeal {
        seal_id: 1,
        cycle_id: "c1".to_string(),
        file_path: file_path.to_string(),
        artifact_category: ArtifactCategory::PreExistingTest,
        blake3_digest: "hash".to_string(),
        ast_assertion_count: 3, // Esperava 3, tem apenas 1
        contract_unlock_state: ContractUnlockState::Sealed,
    };

    let violations = auditor.audit_file(file_path, code_with_less_assertions, Some(&seal));
    assert_eq!(violations.len(), 1);
    match &violations[0] {
        AstViolation::TestDilution(dilution) => {
            assert_eq!(dilution.file_path, file_path);
            assert_eq!(dilution.expected_assertions, 3);
            assert_eq!(dilution.actual_assertions, 1);
        }
        other => panic!("Expected TestDilution, got: {other:?}"),
    }
}

#[test]
fn test_auditor_detects_skipped_tests_in_typescript() {
    let auditor = TestIntegrityAuditor::new();
    let file_path = "tests/payment.spec.ts";
    let code_with_skip = r#"
        test.skip('pay test', () => {
            expect(1).toBe(1);
        });
    "#;

    let seal = ArtifactSeal {
        seal_id: 2,
        cycle_id: "c1".to_string(),
        file_path: file_path.to_string(),
        artifact_category: ArtifactCategory::PreExistingTest,
        blake3_digest: "hash".to_string(),
        ast_assertion_count: 1,
        contract_unlock_state: ContractUnlockState::Sealed,
    };

    let violations = auditor.audit_file(file_path, code_with_skip, Some(&seal));
    assert_eq!(violations.len(), 1);
    match &violations[0] {
        AstViolation::TestDilution(dilution) => {
            assert_eq!(dilution.file_path, file_path);
        }
        other => panic!("Expected TestDilution for skip, got: {other:?}"),
    }
}

#[test]
fn test_auditor_passes_when_assertions_equal_or_increased() {
    let auditor = TestIntegrityAuditor::new();
    let file_path = "tests/math_test.rs";
    let code = r#"
        #[test]
        fn test_math() {
            assert_eq!(2 + 2, 4);
            assert!(true);
        }
    "#;

    let seal = ArtifactSeal {
        seal_id: 3,
        cycle_id: "c1".to_string(),
        file_path: file_path.to_string(),
        artifact_category: ArtifactCategory::PreExistingTest,
        blake3_digest: "hash".to_string(),
        ast_assertion_count: 2,
        contract_unlock_state: ContractUnlockState::Sealed,
    };

    let violations = auditor.audit_file(file_path, code, Some(&seal));
    assert!(violations.is_empty());
}
