// Rust AST graph extractor for structural nodes and dependency edges (TASK-002).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use tree_sitter::{Node, Parser};
use trustkit_core::entities::{
    EdgeConfidence, EdgeRelationKind, GraphNodeKind, StructuralEdge, StructuralNode,
};

pub struct RustGraphExtractor;

impl RustGraphExtractor {
    pub fn extract(
        file_path: &str,
        source_code: &str,
        layer: &str,
        nodes: &mut Vec<StructuralNode>,
        edges: &mut Vec<StructuralEdge>,
    ) {
        let mut parser = Parser::new();
        let language = tree_sitter_rust::LANGUAGE;
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
                "use_declaration" => {
                    Self::process_use_declaration(&child, bytes, file_path, edges);
                }
                "mod_item" => {
                    Self::process_mod_item(&child, bytes, file_path, layer, nodes, edges);
                }
                "struct_item" | "enum_item" | "trait_item" | "function_item" | "type_item" => {
                    Self::process_top_level_item(&child, bytes, file_path, layer, nodes, edges);
                }
                "impl_item" => {
                    Self::process_impl_item(&child, bytes, file_path, layer, nodes, edges);
                }
                _ => {}
            }
        }
    }

    fn is_public(node: &Node, bytes: &[u8]) -> bool {
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

    fn extract_name<'a>(node: &Node<'a>, bytes: &'a [u8]) -> Option<&'a str> {
        node.child_by_field_name("name")
            .and_then(|n| n.utf8_text(bytes).ok())
            .map(str::trim)
    }

    fn process_top_level_item(
        node: &Node,
        bytes: &[u8],
        file_path: &str,
        layer: &str,
        nodes: &mut Vec<StructuralNode>,
        edges: &mut Vec<StructuralEdge>,
    ) {
        if !Self::is_public(node, bytes) {
            return;
        }
        let Some(name) = Self::extract_name(node, bytes) else {
            return;
        };
        if name.is_empty() {
            return;
        }

        let node_id = format!("{file_path}::{name}");
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

    fn process_mod_item(
        node: &Node,
        bytes: &[u8],
        file_path: &str,
        layer: &str,
        nodes: &mut Vec<StructuralNode>,
        edges: &mut Vec<StructuralEdge>,
    ) {
        let Some(name) = Self::extract_name(node, bytes) else {
            return;
        };
        if name.is_empty() {
            return;
        }

        if Self::is_public(node, bytes) {
            let node_id = format!("{file_path}::{name}");
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

        // External module declaration (e.g. mod foo;) acts as import
        if node.child_by_field_name("body").is_none() {
            edges.push(StructuralEdge::new(
                file_path,
                name,
                EdgeRelationKind::Imports,
                EdgeConfidence::Extracted,
                file_path,
            ));
        }
    }

    fn process_impl_item(
        node: &Node,
        bytes: &[u8],
        file_path: &str,
        layer: &str,
        nodes: &mut Vec<StructuralNode>,
        edges: &mut Vec<StructuralEdge>,
    ) {
        let Some(body) = node.child_by_field_name("body") else {
            return;
        };
        let mut cursor = body.walk();
        for item in body.children(&mut cursor) {
            if item.kind() == "function_item"
                && Self::is_public(&item, bytes)
                && let Some(fn_name) = Self::extract_name(&item, bytes)
            {
                let node_id = format!("{file_path}::{fn_name}");
                if !nodes.iter().any(|n| n.node_id == node_id) {
                    nodes.push(StructuralNode::new(
                        &node_id,
                        fn_name,
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
        }
    }

    fn process_use_declaration(
        node: &Node,
        bytes: &[u8],
        file_path: &str,
        edges: &mut Vec<StructuralEdge>,
    ) {
        let arg_text = if let Some(arg) = node.child_by_field_name("argument") {
            arg.utf8_text(bytes).unwrap_or("").trim()
        } else {
            node.utf8_text(bytes)
                .unwrap_or("")
                .trim_start_matches("use ")
                .trim_end_matches(';')
                .trim()
        };

        if arg_text.is_empty() {
            return;
        }

        for path in Self::expand_use_tree(arg_text) {
            if !path.is_empty() {
                edges.push(StructuralEdge::new(
                    file_path,
                    path,
                    EdgeRelationKind::Imports,
                    EdgeConfidence::Extracted,
                    file_path,
                ));
            }
        }
    }

    fn expand_use_tree(raw: &str) -> Vec<String> {
        let clean = raw.trim().trim_end_matches(';').trim();
        if let Some((prefix, rest)) = clean.split_once('{') {
            let base = prefix.trim();
            let items = rest.trim_end_matches('}').trim();
            items
                .split(',')
                .map(|item| item.trim())
                .filter(|item| !item.is_empty())
                .map(|item| format!("{base}{item}"))
                .collect()
        } else {
            vec![clean.to_string()]
        }
    }
}
