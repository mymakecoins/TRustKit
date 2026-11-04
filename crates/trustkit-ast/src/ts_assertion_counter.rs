// Tree-Sitter AST assertion counter for TypeScript source code (TASK-006).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use tree_sitter::{Node, Parser};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TsTestAnalysis {
    pub assertion_count: u32,
    pub has_skipped_tests: bool,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct TsAssertionCounter;

impl TsAssertionCounter {
    pub const fn new() -> Self {
        Self
    }

    pub fn analyze_test_file(&self, source_code: &str) -> TsTestAnalysis {
        let mut parser = Parser::new();
        let language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT;
        if parser.set_language(&language.into()).is_err() {
            return TsTestAnalysis::default();
        }

        let Some(tree) = parser.parse(source_code, None) else {
            return TsTestAnalysis::default();
        };

        let mut analysis = TsTestAnalysis::default();
        Self::traverse_node(tree.root_node(), source_code.as_bytes(), &mut analysis);
        analysis
    }

    fn traverse_node(node: Node, source_bytes: &[u8], analysis: &mut TsTestAnalysis) {
        if node.kind() == "call_expression" {
            let func_text_opt = node
                .child_by_field_name("function")
                .and_then(|fn_node| fn_node.utf8_text(source_bytes).ok());

            if let Some(func_text) = func_text_opt {
                if func_text == "expect" {
                    analysis.assertion_count += 1;
                }

                if func_text.contains(".skip") || func_text.contains(".todo") {
                    analysis.has_skipped_tests = true;
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::traverse_node(child, source_bytes, analysis);
        }
    }
}
