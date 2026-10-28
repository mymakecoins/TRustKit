// AST context slicer for builder token budgeting (TASK-008).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use tree_sitter::{Node, Parser};
use trustkit_core::entities::{ContextSliceRequest, ContextSliceResult};

#[derive(Debug, Default, Clone, Copy)]
pub struct AstContextSlicer;

impl AstContextSlicer {
    pub const fn new() -> Self {
        Self
    }

    pub fn slice_code(
        &self,
        _file_path: &str,
        source_code: &str,
        request: &ContextSliceRequest,
    ) -> ContextSliceResult {
        let mut parser = Parser::new();
        let language = tree_sitter_rust::LANGUAGE;
        if parser.set_language(&language.into()).is_err() {
            return Self::fallback_slice(source_code, request);
        }

        let Some(tree) = parser.parse(source_code, None) else {
            return Self::fallback_slice(source_code, request);
        };

        let mut output_lines = Vec::new();
        let bytes = source_code.as_bytes();
        Self::collect_public_items(tree.root_node(), bytes, &mut output_lines);

        let mut preserved_symbols = Vec::new();
        for sym in &request.symbol_filters {
            if source_code.contains(sym) {
                preserved_symbols.push(sym.clone());
            }
        }

        if output_lines.is_empty() {
            return Self::fallback_slice(source_code, request);
        }

        let mut final_lines = Vec::new();
        let max_budget = if request.max_line_budget == 0 {
            100
        } else {
            request.max_line_budget
        };

        for line in output_lines {
            if final_lines.len() >= max_budget {
                final_lines.push("// [Context truncated to stay within budget]".to_string());
                break;
            }
            final_lines.push(line);
        }

        let total_lines = final_lines.len();
        let sliced_content = final_lines.join("\n");

        ContextSliceResult {
            sliced_content,
            preserved_symbols,
            total_lines,
        }
    }

    fn collect_public_items(node: Node, source_bytes: &[u8], lines: &mut Vec<String>) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let kind = child.kind();
            let text = child.utf8_text(source_bytes).unwrap_or("");
            let is_pub = text.trim().starts_with("pub ");

            if matches!(kind, "struct_item" | "enum_item" | "trait_item") && is_pub {
                for l in text.lines() {
                    lines.push(l.trim_end().to_string());
                }
            } else if kind == "impl_item" {
                if let Some(body) = child.child_by_field_name("body") {
                    let mut body_cursor = body.walk();
                    let mut pub_fns = Vec::new();
                    for item in body.children(&mut body_cursor) {
                        let item_text = item.utf8_text(source_bytes).unwrap_or("");
                        if item.kind() == "function_item" && item_text.trim().starts_with("pub ") {
                            let sig = item_text.split('{').next().unwrap_or(item_text).trim();
                            pub_fns.push(format!("    {sig} {{ /* ... */ }}"));
                        }
                    }
                    if !pub_fns.is_empty() {
                        let header = text.split('{').next().unwrap_or("").trim();
                        lines.push(format!("{header} {{"));
                        lines.extend(pub_fns);
                        lines.push("}".to_string());
                    }
                }
            } else if kind == "function_item" && is_pub {
                let sig = text.split('{').next().unwrap_or(text).trim();
                lines.push(format!("{sig} {{ /* ... */ }}"));
            }
        }
    }

    fn fallback_slice(source_code: &str, request: &ContextSliceRequest) -> ContextSliceResult {
        let max_budget = if request.max_line_budget == 0 {
            100
        } else {
            request.max_line_budget
        };

        let mut lines = Vec::new();
        for line in source_code.lines() {
            if lines.len() >= max_budget {
                lines.push("// [Context truncated to stay within budget]".to_string());
                break;
            }
            if line.trim().starts_with("pub ") {
                lines.push(line.trim_end().to_string());
            }
        }

        let total_lines = lines.len();
        let sliced_content = lines.join("\n");

        ContextSliceResult {
            sliced_content,
            preserved_symbols: request.symbol_filters.clone(),
            total_lines,
        }
    }
}
