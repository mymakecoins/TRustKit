// Test integrity anti-cheat auditor for AST Layer 0 (TASK-007).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::rust_assertion_counter::RustAssertionCounter;
use crate::ts_assertion_counter::TsAssertionCounter;
use trustkit_core::entities::{ArtifactSeal, AstViolation, TestIntegrityViolation};

#[derive(Debug, Default, Clone, Copy)]
pub struct TestIntegrityAuditor {
    rust_counter: RustAssertionCounter,
    ts_counter: TsAssertionCounter,
}

impl TestIntegrityAuditor {
    pub const fn new() -> Self {
        Self {
            rust_counter: RustAssertionCounter::new(),
            ts_counter: TsAssertionCounter::new(),
        }
    }

    pub fn audit_file(
        &self,
        file_path: &str,
        source_code: &str,
        reference_seal: Option<&ArtifactSeal>,
    ) -> Vec<AstViolation> {
        let mut violations = Vec::new();

        let Some(seal) = reference_seal else {
            return violations;
        };

        if file_path.ends_with(".rs") {
            let actual = self.rust_counter.count_assertions(source_code);
            if actual < seal.ast_assertion_count {
                violations.push(AstViolation::TestDilution(TestIntegrityViolation {
                    file_path: file_path.to_string(),
                    test_name: "rust_test_suite".to_string(),
                    expected_assertions: seal.ast_assertion_count,
                    actual_assertions: actual,
                }));
            }
        } else if file_path.ends_with(".ts") || file_path.ends_with(".tsx") {
            let analysis = self.ts_counter.analyze_test_file(source_code);
            if analysis.has_skipped_tests {
                violations.push(AstViolation::TestDilution(TestIntegrityViolation {
                    file_path: file_path.to_string(),
                    test_name: "skipped_or_todo_tests_detected".to_string(),
                    expected_assertions: seal.ast_assertion_count,
                    actual_assertions: analysis.assertion_count,
                }));
            } else if analysis.assertion_count < seal.ast_assertion_count {
                violations.push(AstViolation::TestDilution(TestIntegrityViolation {
                    file_path: file_path.to_string(),
                    test_name: "typescript_test_suite".to_string(),
                    expected_assertions: seal.ast_assertion_count,
                    actual_assertions: analysis.assertion_count,
                }));
            }
        }

        violations
    }
}
