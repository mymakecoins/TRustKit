// Unit tests for Structural Graph Domain Entities and Ports (TASK-001).
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle I: Zero External Dependencies (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use super::*;
use crate::errors::DomainViolation;
use crate::ports::{StructuralGraphAnalyzerPort, StructuralGraphRepositoryPort};

struct InMemoryGraphRepository {
    nodes: std::sync::Mutex<Vec<StructuralNode>>,
    edges: std::sync::Mutex<Vec<StructuralEdge>>,
    manifests: std::sync::Mutex<Vec<FileGraphManifest>>,
}

impl InMemoryGraphRepository {
    fn new() -> Self {
        Self {
            nodes: std::sync::Mutex::new(Vec::new()),
            edges: std::sync::Mutex::new(Vec::new()),
            manifests: std::sync::Mutex::new(Vec::new()),
        }
    }
}

impl StructuralGraphRepositoryPort for InMemoryGraphRepository {
    fn detect_changed_files(
        &self,
        current_manifests: &[FileGraphManifest],
    ) -> Result<Vec<String>, DomainViolation> {
        let existing = self.manifests.lock().unwrap();
        let mut changed = Vec::new();
        for manifest in current_manifests {
            let unchanged = existing.iter().any(|m| {
                m.file_path == manifest.file_path && m.blake3_digest == manifest.blake3_digest
            });
            if !unchanged {
                changed.push(manifest.file_path.clone());
            }
        }
        Ok(changed)
    }

    fn save_graph_snapshot(
        &self,
        nodes: &[StructuralNode],
        edges: &[StructuralEdge],
        manifests: &[FileGraphManifest],
        force: bool,
    ) -> Result<GraphHealthVerdict, DomainViolation> {
        let mut nodes_guard = self.nodes.lock().unwrap();
        if !force && nodes.len() < nodes_guard.len() {
            return Ok(GraphHealthVerdict::ShrinkBlocked);
        }
        *nodes_guard = nodes.to_vec();
        *self.edges.lock().unwrap() = edges.to_vec();
        *self.manifests.lock().unwrap() = manifests.to_vec();
        Ok(GraphHealthVerdict::Healthy)
    }

    fn load_nodes(&self) -> Result<Vec<StructuralNode>, DomainViolation> {
        Ok(self.nodes.lock().unwrap().clone())
    }

    fn load_edges(&self) -> Result<Vec<StructuralEdge>, DomainViolation> {
        Ok(self.edges.lock().unwrap().clone())
    }

    fn load_manifests(&self) -> Result<Vec<FileGraphManifest>, DomainViolation> {
        Ok(self.manifests.lock().unwrap().clone())
    }
}

struct MockGraphAnalyzer;

impl StructuralGraphAnalyzerPort for MockGraphAnalyzer {
    fn audit_graph_health(
        &self,
        nodes: &[StructuralNode],
        edges: &[StructuralEdge],
        _top_god_nodes: usize,
    ) -> Result<GraphHealthReport, DomainViolation> {
        let mut self_loops = Vec::new();
        for edge in edges {
            if edge.source_id == edge.target_id {
                self_loops.push(edge.clone());
            }
        }
        let verdict = if self_loops.is_empty() {
            GraphHealthVerdict::Healthy
        } else {
            GraphHealthVerdict::DegradedWarning
        };
        Ok(GraphHealthReport {
            total_nodes: nodes.len(),
            total_edges: edges.len(),
            dangling_edges: Vec::new(),
            self_loop_edges: self_loops,
            collapsed_edges_count: 0,
            shrink_guard_triggered: false,
            god_nodes: Vec::new(),
            surprising_connections: Vec::new(),
            verdict,
        })
    }

    fn find_shortest_path(
        &self,
        _nodes: &[StructuralNode],
        edges: &[StructuralEdge],
        source_id: &str,
        target_id: &str,
    ) -> Result<ShortestPathOutcome, DomainViolation> {
        if edges
            .iter()
            .any(|e| e.source_id == source_id && e.target_id == target_id)
        {
            Ok(ShortestPathOutcome::PathFound(vec![
                source_id.to_string(),
                target_id.to_string(),
            ]))
        } else {
            Ok(ShortestPathOutcome::NoPathConnected)
        }
    }

    fn slice_graph_context(
        &self,
        _nodes: &[StructuralNode],
        edges: &[StructuralEdge],
        root_file: &str,
        _mode: TraversalMode,
        _max_hops: usize,
        _max_line_budget: usize,
    ) -> Result<GraphTraversalOutcome, DomainViolation> {
        if root_file.is_empty() {
            return Ok(GraphTraversalOutcome::EmptyGraphSlice);
        }
        Ok(GraphTraversalOutcome::VisitedSlice(GraphContextSlice {
            sliced_content: format!("// slice for {root_file}"),
            visited_nodes: vec![root_file.to_string()],
            traversed_edges: edges.to_vec(),
            consumed_lines: 1,
        }))
    }
}

#[test]
fn test_structural_edge_confidence_classifications_preserve_metadata_without_nulls() {
    let extracted = StructuralEdge::new(
        "crates/trustkit-ast/src/lib.rs",
        "crates/trustkit-core/src/ports.rs",
        EdgeRelationKind::Imports,
        EdgeConfidence::Extracted,
        "crates/trustkit-ast/src/lib.rs",
    );
    let inferred = StructuralEdge::new(
        "crates/trustkit-cli/src/main.rs",
        "crates/trustkit-cli/src/bootstrap.rs",
        EdgeRelationKind::ReferencesSymbol,
        EdgeConfidence::Inferred,
        "crates/trustkit-cli/src/main.rs",
    );
    let ambiguous = StructuralEdge::new(
        "specs/006/spec.md",
        "adrs/unknown-target",
        EdgeRelationKind::WikiLinksTo,
        EdgeConfidence::Ambiguous,
        "specs/006/spec.md",
    );

    assert_eq!(extracted.confidence, EdgeConfidence::Extracted);
    assert_eq!(extracted.confidence.as_str(), "EXTRACTED");
    assert_eq!(inferred.confidence, EdgeConfidence::Inferred);
    assert_eq!(inferred.confidence.as_str(), "INFERRED");
    assert_eq!(ambiguous.confidence, EdgeConfidence::Ambiguous);
    assert_eq!(ambiguous.confidence.as_str(), "AMBIGUOUS");
    assert!(extracted.confidence.priority_weight() > inferred.confidence.priority_weight());
    assert!(inferred.confidence.priority_weight() > ambiguous.confidence.priority_weight());

    for edge in [&extracted, &inferred, &ambiguous] {
        let json = edge.to_json();
        assert!(!json.contains("null"));
        assert!(json.contains(edge.confidence.as_str()));
        assert!(json.contains(edge.relation.as_str()));
        assert!(json.contains(&edge.source_file));
    }
}

#[test]
fn test_shortest_path_outcome_no_path_connected_serializes_with_empty_array_and_no_null() {
    let disconnected = ShortestPathOutcome::from_path(vec![]);
    assert_eq!(disconnected, ShortestPathOutcome::NoPathConnected);
    assert!(!disconnected.is_connected());
    assert_eq!(disconnected.as_slice().len(), 0);
    assert_eq!(disconnected.variant_label(), "NO_PATH_CONNECTED");
    assert_eq!(disconnected.as_json_array(), "[]");

    let json = disconnected.to_json();
    assert!(json.contains("\"outcome\":\"NO_PATH_CONNECTED\""));
    assert!(json.contains("\"path\":[]"));
    assert!(!json.contains("null"));

    let connected = ShortestPathOutcome::from_path(vec![
        "node_a".to_string(),
        "node_b".to_string(),
        "node_c".to_string(),
    ]);
    assert!(connected.is_connected());
    assert_eq!(connected.variant_label(), "PATH_FOUND");
    assert_eq!(connected.as_slice().len(), 3);
    assert!(!connected.to_json().contains("null"));
}

#[test]
fn test_graph_traversal_outcome_and_health_report_zero_null_invariants() {
    let empty_slice = GraphTraversalOutcome::EmptyGraphSlice;
    assert!(empty_slice.is_empty());
    assert_eq!(empty_slice.variant_label(), "EMPTY_GRAPH_SLICE");
    let empty_json = empty_slice.to_json();
    assert!(empty_json.contains("\"outcome\":\"EMPTY_GRAPH_SLICE\""));
    assert!(empty_json.contains("\"visited_nodes\":[]"));
    assert!(!empty_json.contains("null"));

    let god_node = GodNodeMetric::new(
        "crates/trustkit-core/src/ports.rs",
        14,
        2,
        vec!["crates/trustkit-ast/src/lib.rs".to_string()],
    );
    assert_eq!(god_node.total_degree, 16);
    assert!(!god_node.to_json().contains("null"));

    let surprising = SurprisingEdge::new(
        "crates/trustkit-core/src/bad.rs",
        "crates/trustkit-db/src/lib.rs",
        "CORE",
        "DB",
        "Core domain must not depend on database adapter",
    );
    assert!(!surprising.to_json().contains("null"));

    let report = GraphHealthReport {
        total_nodes: 12,
        total_edges: 20,
        dangling_edges: vec![],
        self_loop_edges: vec![],
        collapsed_edges_count: 1,
        shrink_guard_triggered: false,
        god_nodes: vec![god_node],
        surprising_connections: vec![surprising],
        verdict: GraphHealthVerdict::DegradedWarning,
    };
    let report_json = report.to_json();
    assert!(report_json.contains("\"verdict\":\"DEGRADED_WARNING\""));
    assert!(!report_json.contains("null"));
}

#[test]
fn test_structural_graph_repository_and_analyzer_ports_contract() {
    let repo = InMemoryGraphRepository::new();
    let analyzer = MockGraphAnalyzer;

    let node_a = StructuralNode::new(
        "crates/trustkit-core/src/ports.rs",
        "ports.rs",
        GraphNodeKind::CodeFile,
        "crates/trustkit-core/src/ports.rs",
        "CORE",
    );
    let node_b = StructuralNode::new(
        "crates/trustkit-ast/src/lib.rs",
        "lib.rs",
        GraphNodeKind::CodeFile,
        "crates/trustkit-ast/src/lib.rs",
        "AST",
    );
    let edge = StructuralEdge::new(
        "crates/trustkit-ast/src/lib.rs",
        "crates/trustkit-core/src/ports.rs",
        EdgeRelationKind::Imports,
        EdgeConfidence::Extracted,
        "crates/trustkit-ast/src/lib.rs",
    );
    let manifest = FileGraphManifest::new(
        "crates/trustkit-core/src/ports.rs",
        "blake3_hash_ports",
        1,
        "2026-10-29T22:00:00Z",
    );

    let verdict = repo
        .save_graph_snapshot(&[node_a.clone(), node_b.clone()], std::slice::from_ref(&edge), std::slice::from_ref(&manifest), false)
        .expect("initial snapshot should succeed");
    assert_eq!(verdict, GraphHealthVerdict::Healthy);

    let shrink_verdict = repo
        .save_graph_snapshot(std::slice::from_ref(&node_a), &[], &[manifest], false)
        .expect("shrink check should return verdict");
    assert_eq!(shrink_verdict, GraphHealthVerdict::ShrinkBlocked);
    assert_eq!(repo.load_nodes().unwrap().len(), 2);

    let path_res = analyzer
        .find_shortest_path(
            &[node_a, node_b],
            &[edge],
            "crates/trustkit-ast/src/lib.rs",
            "crates/trustkit-core/src/ports.rs",
        )
        .expect("path query should succeed");
    assert!(path_res.is_connected());
}
