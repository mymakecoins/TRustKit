// Tree-Sitter AST assertion counter for Rust source code (TASK-005).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use tree_sitter::{Node, Parser};

#[derive(Debug, Default, Clone, Copy)]
pub struct RustAssertionCounter;

impl RustAssertionCounter {
    pub const fn new() -> Self {
        Self
    }

    pub fn count_assertions(&self, source_code: &str) -> u32 {
        let mut parser = Parser::new();
        let language = tree_sitter_rust::LANGUAGE;
        if parser.set_language(&language.into()).is_err() {
            return 0;
        }

        let Some(tree) = parser.parse(source_code, None) else {
            return 0;
        };

        let mut count = 0;
        Self::traverse_node(tree.root_node(), source_code.as_bytes(), &mut count);
        count
    }

    fn traverse_node(node: Node, source_bytes: &[u8], count: &mut u32) {
        if node.kind() == "macro_invocation" {
            let found_name = if let Some(macro_name_node) = node.child_by_field_name("macro") {
                macro_name_node.utf8_text(source_bytes).ok()
            } else if let Some(first) = node.named_child(0) {
                first.utf8_text(source_bytes).ok()
            } else {
                None
            };

            if found_name.is_some_and(Self::is_assertion_macro) {
                *count += 1;
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::traverse_node(child, source_bytes, count);
        }
    }

    fn is_assertion_macro(name: &str) -> bool {
        matches!(
            name.trim_end_matches('!'),
            "assert"
                | "assert_eq"
                | "assert_ne"
                | "debug_assert"
                | "debug_assert_eq"
                | "debug_assert_ne"
        )
    }
}
