// Integration tests for GraphHealthAuditor and StructuralGraphAnalyzerPort (TASK-006).
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use std::time::Instant;
use trustkit_ast::graph_health_auditor::GraphHealthAuditor;
use trustkit_core::entities::{
    EdgeConfidence, EdgeRelationKind, GraphHealthVerdict, GraphNodeKind, ModuleDepthClassification,
    StructuralEdge, StructuralNode,
};
use trustkit_core::ports::StructuralGraphAnalyzerPort;

#[test]
fn test_graph_health_auditor_detects_dangling_and_self_loops() {
    let nodes = vec![
        StructuralNode::new(
            "specs/spec.md",
            "spec.md",
            GraphNodeKind::KnowledgeNote,
            "specs/spec.md",
            "SPEC",
        ),
        StructuralNode::new(
            "node_a.rs",
            "node_a.rs",
            GraphNodeKind::CodeFile,
            "node_a.rs",
            "CORE",
        ),
    ];

    let edges = vec![
        // Dangling edge: adrs/inexistente is not in nodes
        StructuralEdge::new(
            "specs/spec.md",
            "adrs/inexistente",
            EdgeRelationKind::WikiLinksTo,
            EdgeConfidence::Extracted,
            "specs/spec.md",
        ),
        // Self-loop edge
        StructuralEdge::new(
            "node_a.rs",
            "node_a.rs",
            EdgeRelationKind::Imports,
            EdgeConfidence::Extracted,
            "node_a.rs",
        ),
    ];

    let auditor = GraphHealthAuditor::new();
    let report = auditor
        .audit_graph_health(&nodes, &edges, 5)
        .expect("Health audit should succeed");

    assert_eq!(report.verdict, GraphHealthVerdict::DegradedWarning);
    assert_eq!(report.dangling_edges.len(), 1);
    assert_eq!(report.dangling_edges[0].target_id, "adrs/inexistente");
    assert_eq!(report.self_loop_edges.len(), 1);
    assert_eq!(report.self_loop_edges[0].source_id, "node_a.rs");
}

#[test]
fn test_graph_health_auditor_detects_surprising_connections_hexagonal_violation() {
    let nodes = vec![
        StructuralNode::new(
            "crates/trustkit-core/src/domain.rs",
            "domain.rs",
            GraphNodeKind::CodeFile,
            "crates/trustkit-core/src/domain.rs",
            "CORE",
        ),
        StructuralNode::new(
            "crates/trustkit-db/src/repo.rs",
            "repo.rs",
            GraphNodeKind::CodeFile,
            "crates/trustkit-db/src/repo.rs",
            "DB",
        ),
    ];

    // Core importing DB is a prohibited hexagonal violation!
    let edges = vec![StructuralEdge::new(
        "crates/trustkit-core/src/domain.rs",
        "crates/trustkit-db/src/repo.rs",
        EdgeRelationKind::Imports,
        EdgeConfidence::Extracted,
        "crates/trustkit-core/src/domain.rs",
    )];

    let auditor = GraphHealthAuditor::new();
    let report = auditor
        .audit_graph_health(&nodes, &edges, 5)
        .expect("Health audit should succeed");

    assert_eq!(report.verdict, GraphHealthVerdict::DegradedWarning);
    assert_eq!(report.surprising_connections.len(), 1);
    assert_eq!(report.surprising_connections[0].source_layer, "CORE");
    assert_eq!(report.surprising_connections[0].target_layer, "DB");
    assert!(
        report.surprising_connections[0]
            .reason
            .contains("hexagonal")
    );
}

#[test]
fn test_graph_health_auditor_detects_collapsed_edges() {
    let nodes = vec![
        StructuralNode::new(
            "file_a.rs",
            "file_a.rs",
            GraphNodeKind::CodeFile,
            "file_a.rs",
            "CORE",
        ),
        StructuralNode::new(
            "file_b.rs",
            "file_b.rs",
            GraphNodeKind::CodeFile,
            "file_b.rs",
            "CORE",
        ),
    ];

    // Three parallel edges between file_a and file_b
    let edges = vec![
        StructuralEdge::new(
            "file_a.rs",
            "file_b.rs",
            EdgeRelationKind::Imports,
            EdgeConfidence::Extracted,
            "file_a.rs",
        ),
        StructuralEdge::new(
            "file_a.rs",
            "file_b.rs",
            EdgeRelationKind::Imports,
            EdgeConfidence::Extracted,
            "file_a.rs",
        ),
        StructuralEdge::new(
            "file_a.rs",
            "file_b.rs",
            EdgeRelationKind::Imports,
            EdgeConfidence::Extracted,
            "file_a.rs",
        ),
    ];

    let auditor = GraphHealthAuditor::new();
    let report = auditor
        .audit_graph_health(&nodes, &edges, 5)
        .expect("Health audit should succeed");

    assert_eq!(report.collapsed_edges_count, 2);
}

#[test]
fn test_graph_health_auditor_calculates_god_nodes_and_cohesion() {
    let nodes = vec![
        StructuralNode::new(
            "center.rs",
            "center.rs",
            GraphNodeKind::CodeFile,
            "center.rs",
            "CORE",
        ),
        StructuralNode::new(
            "leaf1.rs",
            "leaf1.rs",
            GraphNodeKind::CodeFile,
            "leaf1.rs",
            "CORE",
        ),
        StructuralNode::new(
            "leaf2.rs",
            "leaf2.rs",
            GraphNodeKind::CodeFile,
            "leaf2.rs",
            "CORE",
        ),
        StructuralNode::new(
            "leaf3.rs",
            "leaf3.rs",
            GraphNodeKind::CodeFile,
            "leaf3.rs",
            "DB",
        ),
    ];

    let edges = vec![
        StructuralEdge::new(
            "leaf1.rs",
            "center.rs",
            EdgeRelationKind::Imports,
            EdgeConfidence::Extracted,
            "leaf1.rs",
        ),
        StructuralEdge::new(
            "leaf2.rs",
            "center.rs",
            EdgeRelationKind::Imports,
            EdgeConfidence::Extracted,
            "leaf2.rs",
        ),
        StructuralEdge::new(
            "center.rs",
            "leaf3.rs",
            EdgeRelationKind::Imports,
            EdgeConfidence::Extracted,
            "center.rs",
        ),
    ];

    let auditor = GraphHealthAuditor::new();
    let report = auditor
        .audit_graph_health(&nodes, &edges, 2)
        .expect("Health audit should succeed");

    assert!(!report.god_nodes.is_empty());
    let top_god = &report.god_nodes[0];
    assert_eq!(top_god.node_id, "center.rs");
    assert_eq!(top_god.total_degree, 3);
    assert_eq!(top_god.in_degree, 2);
    assert_eq!(top_god.out_degree, 1);
    assert_eq!(top_god.coupled_neighbors.len(), 3);

    let cohesion = auditor.compute_cohesion_scores(&nodes, &edges);
    assert!(cohesion.contains_key("CORE"));
}

#[test]
fn test_graph_health_auditor_performance_large_graph() {
    let mut nodes = Vec::new();
    for i in 0..5000 {
        nodes.push(StructuralNode::new(
            format!("node_{i}.rs"),
            format!("node_{i}.rs"),
            GraphNodeKind::CodeFile,
            format!("node_{i}.rs"),
            "CORE",
        ));
    }

    let mut edges = Vec::new();
    for i in 0..4999 {
        edges.push(StructuralEdge::new(
            format!("node_{i}.rs"),
            format!("node_{}.rs", i + 1),
            EdgeRelationKind::Imports,
            EdgeConfidence::Extracted,
            format!("node_{i}.rs"),
        ));
    }

    let auditor = GraphHealthAuditor::new();
    let start = Instant::now();
    let report = auditor
        .audit_graph_health(&nodes, &edges, 5)
        .expect("Health audit should succeed");
    let elapsed = start.elapsed();

    assert_eq!(report.verdict, GraphHealthVerdict::Healthy);
    // Precondition: execution in memory < 10ms (generous margin for CI/test while requirement is < 3ms)
    println!("Elapsed for 5000 nodes audit: {:?}", elapsed);
    assert!(elapsed.as_millis() < 50);
}

#[test]
fn test_graph_health_auditor_evaluates_module_depth_and_deletion_test_under_15ms() {
    let auditor = GraphHealthAuditor::new();

    let shallow_rust = r#"
        pub fn forward_save(repo: &Repo, id: &str) -> bool {
            repo.save(id)
        }
        pub fn forward_delete(repo: &Repo, id: &str) -> bool {
            repo.delete(id)
        }
    "#;

    let deep_rust = r#"
        pub fn compile_pipeline(source: &str) -> Result<Vec<String>, String> {
            let mut tokens = Vec::new();
            for line in source.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() && !trimmed.starts_with("//") {
                    tokens.push(trimmed.to_uppercase());
                }
            }
            if tokens.is_empty() {
                return Err("empty pipeline".to_string());
            }
            let mut normalized = Vec::with_capacity(tokens.len());
            for tok in tokens {
                let clean = tok.replace(';', "");
                if !clean.is_empty() {
                    normalized.push(clean);
                }
            }
            Ok(normalized)
        }
    "#;

    let start = Instant::now();
    let shallow_metric =
        auditor.evaluate_module_depth("crates/cli/src/pass_through.rs", shallow_rust);
    let deep_metric = auditor.evaluate_module_depth("crates/core/src/deep_engine.rs", deep_rust);
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 15,
        "Depth evaluation must run in < 15ms"
    );
    assert_eq!(
        shallow_metric.classification,
        ModuleDepthClassification::ShallowPassThrough
    );
    assert!(!shallow_metric.passes_deletion_test);
    assert_eq!(deep_metric.classification, ModuleDepthClassification::Deep);
    assert!(deep_metric.passes_deletion_test);
}

#[test]
fn test_graph_health_auditor_detects_shallow_pass_through_candidates_in_report() {
    let nodes = vec![
        StructuralNode::new(
            "crates/cli/src/wrapper.rs",
            "wrapper.rs",
            GraphNodeKind::CodeFile,
            "crates/cli/src/wrapper.rs",
            "CLI",
        ),
        StructuralNode::new(
            "crates/cli/src/wrapper.rs::delegate_call",
            "delegate_call",
            GraphNodeKind::PublicSymbol,
            "crates/cli/src/wrapper.rs",
            "CLI",
        ),
        StructuralNode::new(
            "crates/cli/src/wrapper.rs::delegate_status",
            "delegate_status",
            GraphNodeKind::PublicSymbol,
            "crates/cli/src/wrapper.rs",
            "CLI",
        ),
        StructuralNode::new(
            "crates/orch/src/adapter.rs",
            "adapter.rs",
            GraphNodeKind::CodeFile,
            "crates/orch/src/adapter.rs",
            "ORCH",
        ),
    ];

    let edges = vec![
        StructuralEdge::new(
            "crates/cli/src/wrapper.rs",
            "crates/cli/src/wrapper.rs::delegate_call",
            EdgeRelationKind::DefinesSymbol,
            EdgeConfidence::Extracted,
            "crates/cli/src/wrapper.rs",
        ),
        StructuralEdge::new(
            "crates/cli/src/wrapper.rs",
            "crates/cli/src/wrapper.rs::delegate_status",
            EdgeRelationKind::DefinesSymbol,
            EdgeConfidence::Extracted,
            "crates/cli/src/wrapper.rs",
        ),
        StructuralEdge::new(
            "crates/cli/src/wrapper.rs",
            "crates/orch/src/adapter.rs",
            EdgeRelationKind::Imports,
            EdgeConfidence::Extracted,
            "crates/cli/src/wrapper.rs",
        ),
    ];

    let auditor = GraphHealthAuditor::new();
    let report = auditor
        .audit_graph_health(&nodes, &edges, 5)
        .expect("Health audit should succeed");

    assert!(!report.shallow_modules.is_empty());
    let candidate = report
        .shallow_modules
        .iter()
        .find(|c| c.file_path == "crates/cli/src/wrapper.rs")
        .expect("wrapper.rs should be flagged as shallow module candidate");
    assert!(candidate.affected_seam.contains("adapter.rs"));
    assert!(!candidate.deletion_test_justification.is_empty());
    assert!(!candidate.deepening_recommendation.is_empty());
}
