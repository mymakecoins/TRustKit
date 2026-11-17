// TypeScript/JavaScript AST graph extractor (TASK-002).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use std::path::Path;
use tree_sitter::{Node, Parser};
use trustkit_core::entities::{
    EdgeConfidence, EdgeRelationKind, GraphNodeKind, StructuralEdge, StructuralNode,
};

pub struct TsGraphExtractor;

impl TsGraphExtractor {
    pub fn extract(
        file_path: &str,
        source_code: &str,
        layer: &str,
        nodes: &mut Vec<StructuralNode>,
        edges: &mut Vec<StructuralEdge>,
    ) {
        let mut parser = Parser::new();
        let language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT;
        if parser.set_language(&language.into()).is_err() {
            return;
        }

        let Some(tree) = parser.parse(source_code, None) else {
            return;
        };

        let bytes = source_code.as_bytes();
        let root = tree.root_node();
        let mut cursor = root.walk();

        for child in root.children(&mut cursor) {
            match child.kind() {
                "import_statement" => {
                    Self::process_import(&child, bytes, file_path, edges);
                }
                "export_statement" => {
                    Self::process_export(&child, bytes, file_path, layer, nodes, edges);
                }
                _ => {}
            }
        }
    }

    fn process_import(node: &Node, bytes: &[u8], file_path: &str, edges: &mut Vec<StructuralEdge>) {
        let Some(source_node) = node.child_by_field_name("source") else {
            return;
        };
        let Ok(raw_spec) = source_node.utf8_text(bytes) else {
            return;
        };
        let specifier = raw_spec.trim().trim_matches(['\'', '"']);
        if specifier.is_empty() {
            return;
        }

        let is_relative = specifier.starts_with("./") || specifier.starts_with("../");
        let (target_id, confidence) = if is_relative {
            let resolved = Self::resolve_relative_path(file_path, specifier);
            let has_ext = specifier.ends_with(".ts")
                || specifier.ends_with(".tsx")
                || specifier.ends_with(".js")
                || specifier.ends_with(".jsx");
            let conf = if has_ext {
                EdgeConfidence::Extracted
            } else {
                EdgeConfidence::Inferred
            };
            (resolved, conf)
        } else {
            (specifier.to_string(), EdgeConfidence::Extracted)
        };

        edges.push(StructuralEdge::new(
            file_path,
            target_id,
            EdgeRelationKind::Imports,
            confidence,
            file_path,
        ));
    }

    fn process_export(
        node: &Node,
        bytes: &[u8],
        file_path: &str,
        layer: &str,
        nodes: &mut Vec<StructuralNode>,
        edges: &mut Vec<StructuralEdge>,
    ) {
        if let Some(decl) = node.child_by_field_name("declaration") {
            Self::process_declaration(&decl, bytes, file_path, layer, nodes, edges);
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "export_clause" {
                let mut clause_cursor = child.walk();
                for spec in child.children(&mut clause_cursor) {
                    if spec.kind() == "export_specifier"
                        && let Some(name_node) = spec.child_by_field_name("name")
                        && let Ok(name) = name_node.utf8_text(bytes)
                    {
                        Self::register_symbol(name.trim(), file_path, layer, nodes, edges);
                    }
                }
            }
        }
    }

    fn process_declaration(
        decl: &Node,
        bytes: &[u8],
        file_path: &str,
        layer: &str,
        nodes: &mut Vec<StructuralNode>,
        edges: &mut Vec<StructuralEdge>,
    ) {
        match decl.kind() {
            "interface_declaration"
            | "type_alias_declaration"
            | "class_declaration"
            | "function_declaration" => {
                if let Some(name_node) = decl.child_by_field_name("name")
                    && let Ok(name) = name_node.utf8_text(bytes)
                {
                    let clean_name = name.trim();
                    Self::register_symbol(clean_name, file_path, layer, nodes, edges);
                    if decl.kind() == "class_declaration" {
                        Self::extract_ts_implements(decl, bytes, file_path, clean_name, edges);
                    }
                }
            }
            "lexical_declaration" => {
                let mut cursor = decl.walk();
                for child in decl.children(&mut cursor) {
                    if child.kind() == "variable_declarator"
                        && let Some(name_node) = child.child_by_field_name("name")
                        && let Ok(name) = name_node.utf8_text(bytes)
                    {
                        Self::register_symbol(name.trim(), file_path, layer, nodes, edges);
                    }
                }
            }
            _ => {}
        }
    }

    fn register_symbol(
        name: &str,
        file_path: &str,
        layer: &str,
        nodes: &mut Vec<StructuralNode>,
        edges: &mut Vec<StructuralEdge>,
    ) {
        if name.is_empty() {
            return;
        }
        let node_id = format!("{file_path}::{name}");
        if !nodes.iter().any(|n| n.node_id == node_id) {
            nodes.push(StructuralNode::new(
                &node_id,
                name,
                GraphNodeKind::PublicSymbol,
                file_path,
                layer,
            ));
            edges.push(StructuralEdge::new(
                file_path,
                &node_id,
                EdgeRelationKind::DefinesSymbol,
                EdgeConfidence::Extracted,
                file_path,
            ));
        }
    }

    fn extract_ts_implements(
        class_node: &Node,
        bytes: &[u8],
        file_path: &str,
        class_name: &str,
        edges: &mut Vec<StructuralEdge>,
    ) {
        let Ok(raw) = class_node.utf8_text(bytes) else {
            return;
        };
        let header = raw.split('{').next().unwrap_or("");
        if let Some((_, after_impl)) = header.split_once("implements ") {
            for iface in after_impl.split(',') {
                let clean = iface.split('<').next().unwrap_or(iface).trim();
                if !clean.is_empty() {
                    edges.push(StructuralEdge::new(
                        format!("{file_path}::{class_name}"),
                        clean,
                        EdgeRelationKind::ReferencesSymbol,
                        EdgeConfidence::Extracted,
                        file_path,
                    ));
                }
            }
        }
    }

    fn resolve_relative_path(base_file: &str, relative: &str) -> String {
        let parent = Path::new(base_file)
            .parent()
            .unwrap_or_else(|| Path::new(""));
        let joined = parent.join(relative);
        let mut components = Vec::new();
        for comp in joined.components() {
            match comp {
                std::path::Component::ParentDir => {
                    components.pop();
                }
                std::path::Component::CurDir => {}
                std::path::Component::Normal(c) => {
                    components.push(c.to_string_lossy().to_string());
                }
                _ => {}
            }
        }
        components.join("/")
    }
}
