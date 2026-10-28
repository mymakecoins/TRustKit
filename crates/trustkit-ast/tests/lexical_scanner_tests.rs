// Unit tests for InvariantLexicalScanner (TASK-004).
// Constitutional Principle III: Test-First & Anti-Cheat.

use trustkit_ast::InvariantLexicalScanner;
use trustkit_core::entities::AstViolation;

#[test]
fn test_scanner_detects_forbidden_emojis() {
    let scanner = InvariantLexicalScanner::new();
    let content = "fn add(a: i32, b: i32) -> i32 {\n    // Super fast 🚀\n    a + b\n}";
    let violations = scanner.scan_content("src/calc.rs", content);

    assert_eq!(violations.len(), 1);
    match &violations[0] {
        AstViolation::ForbiddenEmoji { file_path, line, character } => {
            assert_eq!(file_path, "src/calc.rs");
            assert_eq!(*line, 2);
            assert_eq!(character, "🚀");
        }
        other => panic!("Expected ForbiddenEmoji, got: {other:?}"),
    }
}

#[test]
fn test_scanner_detects_ai_trailers() {
    let scanner = InvariantLexicalScanner::new();
    let content = "feat: #42 add feature\n\nCo-Authored-By: Claude <noreply@anthropic.com>";
    let violations = scanner.scan_content("commit.txt", content);

    assert_eq!(violations.len(), 1);
    match &violations[0] {
        AstViolation::AiTrailer { file_path, line, trailer } => {
            assert_eq!(file_path, "commit.txt");
            assert_eq!(*line, 3);
            assert!(trailer.contains("Co-Authored-By: Claude"));
        }
        other => panic!("Expected AiTrailer, got: {other:?}"),
    }
}

#[test]
fn test_scanner_detects_untyped_any_in_typescript() {
    let scanner = InvariantLexicalScanner::new();
    let content = "export function parse(input: any): any {\n    return input;\n}";
    let violations = scanner.scan_content("src/parser.ts", content);

    assert_eq!(violations.len(), 2);
    for v in violations {
        match v {
            AstViolation::UntypedAny { file_path, line } => {
                assert_eq!(file_path, "src/parser.ts");
                assert_eq!(line, 1);
            }
            other => panic!("Expected UntypedAny, got: {other:?}"),
        }
    }
}

#[test]
fn test_scanner_passes_clean_code() {
    let scanner = InvariantLexicalScanner::new();
    let content = "fn main() {\n    println!(\"Hello clean world\");\n}";
    let violations = scanner.scan_content("src/main.rs", content);
    assert!(violations.is_empty());
}
