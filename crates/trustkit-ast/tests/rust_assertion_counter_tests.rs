// Unit tests for RustAssertionCounter (TASK-005).
// Constitutional Principle III: Test-First & Anti-Cheat.

use trustkit_ast::RustAssertionCounter;

#[test]
fn test_count_assertions_in_rust_test_function() {
    let counter = RustAssertionCounter::new();
    let code = r#"
        #[test]
        fn test_sample() {
            let x = 1 + 1;
            assert_eq!(x, 2);
            assert!(x > 0);
            assert_ne!(x, 0);
        }
    "#;

    let count = counter.count_assertions(code);
    assert_eq!(count, 3);
}

#[test]
fn test_count_ignores_other_macros() {
    let counter = RustAssertionCounter::new();
    let code = r#"
        #[test]
        fn test_sample() {
            println!("hello");
            eprintln!("debug: {:?}", 42);
            let v = vec![1, 2, 3];
            assert_eq!(v.len(), 3);
        }
    "#;

    let count = counter.count_assertions(code);
    assert_eq!(count, 1);
}

#[test]
fn test_count_empty_or_no_assertions() {
    let counter = RustAssertionCounter::new();
    let code = "fn main() { println!(\"no tests\"); }";
    let count = counter.count_assertions(code);
    assert_eq!(count, 0);
}
