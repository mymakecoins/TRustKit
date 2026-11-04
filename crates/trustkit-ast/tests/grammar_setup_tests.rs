// Unit test for tree-sitter grammars initialization (TASK-003).
// Constitutional Principle III: Test-First & Anti-Cheat.

#[test]
fn test_tree_sitter_rust_grammar_initialization() {
    let mut parser = tree_sitter::Parser::new();
    let language = tree_sitter_rust::LANGUAGE;
    let set_lang_res = parser.set_language(&language.into());
    assert!(
        set_lang_res.is_ok(),
        "Failed to set Rust language in tree-sitter parser"
    );

    let source = "fn main() { assert_eq!(1, 1); }";
    let tree = parser
        .parse(source, None)
        .expect("Failed to parse Rust source code");
    let root = tree.root_node();
    assert_eq!(root.kind(), "source_file");
}

#[test]
fn test_tree_sitter_typescript_grammar_initialization() {
    let mut parser = tree_sitter::Parser::new();
    let language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT;
    let set_lang_res = parser.set_language(&language.into());
    assert!(
        set_lang_res.is_ok(),
        "Failed to set TypeScript language in tree-sitter parser"
    );

    let source = "test('example', () => { expect(true).toBe(true); });";
    let tree = parser
        .parse(source, None)
        .expect("Failed to parse TypeScript source code");
    let root = tree.root_node();
    assert_eq!(root.kind(), "program");
}
