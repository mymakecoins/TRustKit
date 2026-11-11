// Module Depth (John Ousterhout) and Seam Classification (Michael Feathers) AST Analyzer.
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use tree_sitter::{Node, Parser};
use trustkit_core::entities::ModuleDepthMetric;

#[derive(Debug, Default, Clone, Copy)]
pub struct ModuleDepthAnalyzer;

impl ModuleDepthAnalyzer {
    pub const fn new() -> Self {
        Self
    }

    pub fn evaluate_source_depth(&self, file_path: &str, source_code: &str) -> ModuleDepthMetric {
        if file_path.ends_with(".ts")
            || file_path.ends_with(".tsx")
            || file_path.ends_with(".js")
            || file_path.ends_with(".jsx")
        {
            Self::evaluate_ts_depth(file_path, source_code)
        } else {
            Self::evaluate_rust_depth(file_path, source_code)
        }
    }

    fn evaluate_rust_depth(file_path: &str, source_code: &str) -> ModuleDepthMetric {
        let mut parser = Parser::new();
        if parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .is_err()
        {
            return ModuleDepthMetric::evaluate(file_path, 0, 1, false);
        }
        let Some(tree) = parser.parse(source_code, None) else {
            return ModuleDepthMetric::evaluate(file_path, 0, 1, false);
        };

        let bytes = source_code.as_bytes();
        let root = tree.root_node();
        let mut pub_surface = 0usize;
        let mut internal_units = 0usize;
        let mut pub_fn_count = 0usize;
        let mut trivial_pub_fn_count = 0usize;

        Self::walk_rust_node(
            &root,
            bytes,
            &mut pub_surface,
            &mut internal_units,
            &mut pub_fn_count,
            &mut trivial_pub_fn_count,
        );

        let is_pass_through = pub_fn_count > 0 && trivial_pub_fn_count == pub_fn_count;
        ModuleDepthMetric::evaluate(file_path, pub_surface, internal_units, is_pass_through)
    }

    fn walk_rust_node(
        node: &Node,
        bytes: &[u8],
        pub_surface: &mut usize,
        internal_units: &mut usize,
        pub_fn_count: &mut usize,
        trivial_pub_fn_count: &mut usize,
    ) {
        let kind = node.kind();
        match kind {
            "function_item" => {
                let is_pub = Self::is_rust_pub(node, bytes);
                let params = Self::count_rust_params(node);
                let body_complexity = node
                    .child_by_field_name("body")
                    .map(|b| Self::count_rust_block_complexity(&b))
                    .unwrap_or(0);
                if is_pub {
                    *pub_surface += 1 + params;
                    *pub_fn_count += 1;
                    if body_complexity <= 1 {
                        *trivial_pub_fn_count += 1;
                    }
                } else {
                    *internal_units += 2;
                }
                *internal_units += body_complexity;
                return;
            }
            "struct_item" | "enum_item" | "trait_item" | "type_item" => {
                if Self::is_rust_pub(node, bytes) {
                    *pub_surface += 1;
                } else {
                    *internal_units += 2;
                }
            }
            _ => {}
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::walk_rust_node(
                &child,
                bytes,
                pub_surface,
                internal_units,
                pub_fn_count,
                trivial_pub_fn_count,
            );
        }
    }

    fn is_rust_pub(node: &Node, bytes: &[u8]) -> bool {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "visibility_modifier"
                && let Ok(text) = child.utf8_text(bytes)
            {
                return text.trim().starts_with("pub");
            }
        }
        false
    }

    fn count_rust_params(fn_node: &Node) -> usize {
        let Some(params) = fn_node.child_by_field_name("parameters") else {
            return 0;
        };
        let mut count = 0;
        let mut cursor = params.walk();
        for child in params.children(&mut cursor) {
            if child.kind() == "parameter" {
                count += 1;
            }
        }
        count
    }

    fn count_rust_block_complexity(block: &Node) -> usize {
        let mut units = 0;
        Self::walk_rust_complexity(block, &mut units);
        units
    }

    fn walk_rust_complexity(node: &Node, units: &mut usize) {
        match node.kind() {
            "let_declaration"
            | "if_expression"
            | "match_expression"
            | "match_arm"
            | "for_expression"
            | "while_expression"
            | "loop_expression"
            | "return_expression"
            | "assignment_expression"
            | "compound_assignment_expr"
            | "macro_invocation" => {
                *units += 2;
            }
            "call_expression" | "method_call_expression" => {
                *units += 1;
            }
            _ => {}
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::walk_rust_complexity(&child, units);
        }
    }

    fn evaluate_ts_depth(file_path: &str, source_code: &str) -> ModuleDepthMetric {
        let mut parser = Parser::new();
        if parser
            .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
            .is_err()
        {
            return ModuleDepthMetric::evaluate(file_path, 0, 1, false);
        }
        let Some(tree) = parser.parse(source_code, None) else {
            return ModuleDepthMetric::evaluate(file_path, 0, 1, false);
        };

        let root = tree.root_node();
        let mut pub_surface = 0usize;
        let mut internal_units = 0usize;
        let mut pub_fn_count = 0usize;
        let mut trivial_pub_fn_count = 0usize;

        let mut cursor = root.walk();
        for child in root.children(&mut cursor) {
            if child.kind() == "export_statement" {
                pub_surface += 1;
                let comp = Self::count_ts_complexity(&child);
                internal_units += comp;
                pub_fn_count += 1;
                if comp <= 1 {
                    trivial_pub_fn_count += 1;
                }
            } else if child.kind() != "import_statement" {
                internal_units += Self::count_ts_complexity(&child) + 1;
            }
        }

        let is_pass_through = pub_fn_count > 0 && trivial_pub_fn_count == pub_fn_count;
        ModuleDepthMetric::evaluate(file_path, pub_surface, internal_units, is_pass_through)
    }

    fn count_ts_complexity(node: &Node) -> usize {
        let mut units = 0;
        Self::walk_ts_complexity(node, &mut units);
        units
    }

    fn walk_ts_complexity(node: &Node, units: &mut usize) {
        match node.kind() {
            "lexical_declaration"
            | "variable_declaration"
            | "if_statement"
            | "for_statement"
            | "while_statement"
            | "switch_statement"
            | "try_statement" => {
                *units += 2;
            }
            "call_expression" | "return_statement" => {
                *units += 1;
            }
            _ => {}
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::walk_ts_complexity(&child, units);
        }
    }
}
