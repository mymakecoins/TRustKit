// AST graph extractor for Rust, TypeScript/JS and WikiLinks (TASK-002).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use std::fs;
use std::path::Path;
use trustkit_core::entities::{
    EdgeConfidence, EdgeRelationKind, GraphNodeKind, ModuleDepthMetric, SeamClassification,
    StructuralEdge, StructuralNode,
};
use trustkit_core::errors::DomainViolation;

use crate::knowledge_link_extractor::{extract_frontmatter_and_body, extract_wikilinks};
use crate::module_depth_analyzer::ModuleDepthAnalyzer;
use crate::rust_graph_extractor::RustGraphExtractor;
use crate::ts_graph_extractor::TsGraphExtractor;

#[derive(Debug, Default, Clone, Copy)]
pub struct AstGraphExtractor;

impl AstGraphExtractor {
    pub const fn new() -> Self {
        Self
    }

    pub fn extract_from_source(
        &self,
        file_path: &str,
        content: &str,
    ) -> (Vec<StructuralNode>, Vec<StructuralEdge>) {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let layer = detect_layer(file_path);

        if is_markdown(file_path) {
            Self::extract_markdown(file_path, content, &layer, &mut nodes, &mut edges);
        } else if is_rust(file_path) {
            Self::add_file_node(file_path, &layer, &mut nodes);
            RustGraphExtractor::extract(file_path, content, &layer, &mut nodes, &mut edges);
        } else if is_typescript_or_js(file_path) {
            Self::add_file_node(file_path, &layer, &mut nodes);
            TsGraphExtractor::extract(file_path, content, &layer, &mut nodes, &mut edges);
        } else {
            Self::add_file_node(file_path, &layer, &mut nodes);
        }

        (nodes, edges)
    }

    pub fn extract_from_in_memory_files(
        &self,
        files: &[(String, String)],
    ) -> (Vec<StructuralNode>, Vec<StructuralEdge>) {
        let mut all_nodes = Vec::new();
        let mut all_edges = Vec::new();

        for (path, content) in files {
            let (nodes, edges) = self.extract_from_source(path, content);
            all_nodes.extend(nodes);
            all_edges.extend(edges);
        }

        (all_nodes, all_edges)
    }

    pub fn extract_from_files(
        &self,
        file_paths: &[String],
    ) -> Result<(Vec<StructuralNode>, Vec<StructuralEdge>), DomainViolation> {
        let mut all_nodes = Vec::new();
        let mut all_edges = Vec::new();

        for path in file_paths {
            let content = fs::read_to_string(path).map_err(|e| {
                DomainViolation::InfrastructureBoundaryFault {
                    subsystem: "ast_graph_extractor".into(),
                    details: format!("Failed reading file '{path}': {e}"),
                }
            })?;

            let (nodes, edges) = self.extract_from_source(path, &content);
            all_nodes.extend(nodes);
            all_edges.extend(edges);
        }

        Ok((all_nodes, all_edges))
    }

    pub fn classify_seams(
        &self,
        nodes: &[StructuralNode],
        edges: &[StructuralEdge],
    ) -> Vec<SeamClassification> {
        ModuleDepthAnalyzer::new().classify_seams(nodes, edges)
    }

    pub fn extract_module_depths_from_in_memory_files(
        &self,
        files: &[(String, String)],
    ) -> Vec<ModuleDepthMetric> {
        let analyzer = ModuleDepthAnalyzer::new();
        files
            .iter()
            .map(|(path, content)| analyzer.evaluate_source_depth(path, content))
            .collect()
    }

    fn add_file_node(file_path: &str, layer: &str, nodes: &mut Vec<StructuralNode>) {
        let label = Path::new(file_path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(file_path);

        nodes.push(StructuralNode::new(
            file_path,
            label,
            GraphNodeKind::CodeFile,
            file_path,
            layer,
        ));
    }

    fn extract_markdown(
        file_path: &str,
        content: &str,
        layer: &str,
        nodes: &mut Vec<StructuralNode>,
        edges: &mut Vec<StructuralEdge>,
    ) {
        let note = extract_frontmatter_and_body(content);
        let label = if note.title.is_empty() {
            Path::new(file_path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(file_path)
                .to_string()
        } else {
            note.title
        };

        nodes.push(StructuralNode::new(
            file_path,
            label,
            GraphNodeKind::KnowledgeNote,
            file_path,
            layer,
        ));

        let wikilinks = extract_wikilinks(file_path, &note.body);
        for link in wikilinks {
            edges.push(StructuralEdge::new(
                file_path,
                link.target_slug,
                EdgeRelationKind::WikiLinksTo,
                EdgeConfidence::Extracted,
                file_path,
            ));
        }
    }
}

pub fn detect_layer(file_path: &str) -> String {
    let lower = file_path.to_ascii_lowercase();
    if lower.contains("trustkit-core") || lower.contains("/core/") {
        "CORE".to_string()
    } else if lower.contains("trustkit-ast") || lower.contains("/ast/") {
        "AST".to_string()
    } else if lower.contains("trustkit-db") || lower.contains("/db/") {
        "DB".to_string()
    } else if lower.contains("trustkit-orch") || lower.contains("/orch/") {
        "ORCH".to_string()
    } else if lower.contains("trustkit-gh") || lower.contains("/gh/") {
        "GH".to_string()
    } else if lower.contains("trustkit-cli") || lower.contains("/cli/") {
        "CLI".to_string()
    } else if lower.contains(".trustkit/wiki") || lower.contains("/wiki/") {
        "WIKI".to_string()
    } else if lower.contains("specs/") {
        "SPEC".to_string()
    } else {
        "CODE".to_string()
    }
}

fn is_rust(path: &str) -> bool {
    path.ends_with(".rs")
}

fn is_typescript_or_js(path: &str) -> bool {
    path.ends_with(".ts")
        || path.ends_with(".tsx")
        || path.ends_with(".js")
        || path.ends_with(".jsx")
}

fn is_markdown(path: &str) -> bool {
    path.ends_with(".md")
}
