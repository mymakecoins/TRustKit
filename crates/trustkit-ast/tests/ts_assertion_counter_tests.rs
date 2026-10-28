// Unit tests for TsAssertionCounter (TASK-006).
// Constitutional Principle III: Test-First & Anti-Cheat.

use trustkit_ast::TsAssertionCounter;

#[test]
fn test_count_expect_assertions_in_typescript() {
    let counter = TsAssertionCounter::new();
    let code = r#"
        test('validates user', () => {
            const user = getUser();
            expect(user).toBeDefined();
            expect(user.name).toBe('Alice');
            expect(user.age).toBeGreaterThan(18);
        });
    "#;

    let result = counter.analyze_test_file(code);
    assert_eq!(result.assertion_count, 3);
    assert!(!result.has_skipped_tests);
}

#[test]
fn test_detect_skipped_tests_in_typescript() {
    let counter = TsAssertionCounter::new();
    let code = r#"
        test.skip('skipped test', () => {
            expect(1).toBe(1);
        });
    "#;

    let result = counter.analyze_test_file(code);
    assert!(result.has_skipped_tests);
}

#[test]
fn test_no_assertions_in_ts() {
    let counter = TsAssertionCounter::new();
    let code = "export const PI = 3.14;";
    let result = counter.analyze_test_file(code);
    assert_eq!(result.assertion_count, 0);
    assert!(!result.has_skipped_tests);
}
