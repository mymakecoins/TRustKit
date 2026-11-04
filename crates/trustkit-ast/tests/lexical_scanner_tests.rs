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
        AstViolation::ForbiddenEmoji {
            file_path,
            line,
            character,
        } => {
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
        AstViolation::AiTrailer {
            file_path,
            line,
            trailer,
        } => {
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

#[test]
fn test_scanner_verifies_bilingual_readmes_and_diagrams_have_zero_violations() {
    let scanner = InvariantLexicalScanner::new();
    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root should exist");

    for rel in [
        "README.md",
        "README.en.md",
        "docs/diagrams/01-trustkit-architecture.json",
        "docs/diagrams/02-trustkit-7gates-workflow.json",
        "docs/diagrams/03-trustkit-task-tdd-sequence.json",
        "docs/diagrams/05-trustkit-feature-lifecycle.json",
    ] {
        let full_path = repo_root.join(rel);
        let content = std::fs::read_to_string(&full_path)
            .unwrap_or_else(|e| panic!("Failed reading {}: {e}", full_path.display()));
        let violations = scanner.scan_content(rel, &content);
        assert!(
            violations.is_empty(),
            "Expected zero lexical violations in {rel}, found: {violations:?}"
        );
    }

    for html_rel in [
        "docs/diagrams/01-trustkit-architecture.html",
        "docs/diagrams/02-trustkit-7gates-workflow.html",
        "docs/diagrams/03-trustkit-task-tdd-sequence.html",
        "docs/diagrams/05-trustkit-feature-lifecycle.html",
    ] {
        let html_path = repo_root.join(html_rel);
        if html_path.is_file() {
            let content = std::fs::read_to_string(&html_path)
                .unwrap_or_else(|e| panic!("Failed reading {}: {e}", html_path.display()));
            let violations = scanner.scan_content(html_rel, &content);
            assert!(
                violations.is_empty(),
                "Expected zero lexical violations in {html_rel}, found: {violations:?}"
            );
        }
    }
}
