// Integration tests for GraphContextSlicer multi-hop traversal (TASK-004).
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use std::collections::HashMap;
use trustkit_ast::context_slicer::AstContextSlicer;
use trustkit_ast::graph_context_slicer::GraphContextSlicer;
use trustkit_core::entities::{
    EdgeConfidence, EdgeRelationKind, GraphNodeKind, GraphTraversalOutcome, StructuralEdge,
    StructuralNode, TraversalMode,
};

#[test]
fn test_graph_slicer_bfs_one_hop_service_ports_entities() {
    let nodes = vec![
        StructuralNode::new(
            "crates/service.rs",
            "service.rs",
            GraphNodeKind::CodeFile,
            "crates/service.rs",
            "CORE",
        ),
        StructuralNode::new(
            "crates/ports.rs",
            "ports.rs",
            GraphNodeKind::CodeFile,
            "crates/ports.rs",
            "CORE",
        ),
        StructuralNode::new(
            "crates/entities.rs",
            "entities.rs",
            GraphNodeKind::CodeFile,
            "crates/entities.rs",
            "CORE",
        ),
    ];

    let edges = vec![
        StructuralEdge::new(
            "crates/service.rs",
            "crates/ports.rs",
            EdgeRelationKind::Imports,
            EdgeConfidence::Extracted,
            "crates/service.rs",
        ),
        StructuralEdge::new(
            "crates/service.rs",
            "crates/entities.rs",
            EdgeRelationKind::Imports,
            EdgeConfidence::Extracted,
            "crates/service.rs",
        ),
    ];

    let mut sources = HashMap::new();
    sources.insert(
        "crates/service.rs".to_string(),
        "pub struct ServiceOrder;\npub fn execute_service() {}\n".to_string(),
    );
    sources.insert(
        "crates/ports.rs".to_string(),
        "pub trait OrderRepository {\n    fn save(&self);\n}\n".to_string(),
    );
    sources.insert(
        "crates/entities.rs".to_string(),
        "pub struct OrderEntity {\n    pub id: u64,\n}\n".to_string(),
    );

    let slicer = GraphContextSlicer::new();
    let outcome = slicer.slice_graph_with_sources(
        &nodes,
        &edges,
        &sources,
        "crates/service.rs",
        TraversalMode::Bfs,
        1,
        80,
    );

    match outcome {
        GraphTraversalOutcome::VisitedSlice(slice) => {
            assert_eq!(slice.visited_nodes.len(), 3);
            assert_eq!(slice.visited_nodes[0], "crates/service.rs");
            assert!(slice.visited_nodes.contains(&"crates/ports.rs".to_string()));
            assert!(
                slice
                    .visited_nodes
                    .contains(&"crates/entities.rs".to_string())
            );
            assert!(slice.consumed_lines <= 80);
            assert!(slice.sliced_content.contains("pub struct ServiceOrder"));
            assert!(slice.sliced_content.contains("OrderRepository"));
            assert!(slice.sliced_content.contains("pub struct OrderEntity"));
            assert_eq!(slice.traversed_edges.len(), 2);
        }
        GraphTraversalOutcome::EmptyGraphSlice => {
            panic!("Expected VisitedSlice but got EmptyGraphSlice");
        }
    }
}

#[test]
fn test_graph_slicer_bfs_vs_dfs_order_in_three_level_graph() {
    let nodes = vec![
        StructuralNode::new(
            "root.rs",
            "root.rs",
            GraphNodeKind::CodeFile,
            "root.rs",
            "CORE",
        ),
        StructuralNode::new(
            "lvl1_a.rs",
            "lvl1_a.rs",
            GraphNodeKind::CodeFile,
            "lvl1_a.rs",
            "CORE",
        ),
        StructuralNode::new(
            "lvl1_b.rs",
            "lvl1_b.rs",
            GraphNodeKind::CodeFile,
            "lvl1_b.rs",
            "CORE",
        ),
        StructuralNode::new(
            "lvl2_a.rs",
            "lvl2_a.rs",
            GraphNodeKind::CodeFile,
            "lvl2_a.rs",
            "CORE",
        ),
    ];

    let edges = vec![
        StructuralEdge::new(
            "root.rs",
            "lvl1_a.rs",
            EdgeRelationKind::Imports,
            EdgeConfidence::Extracted,
            "root.rs",
        ),
        StructuralEdge::new(
            "root.rs",
            "lvl1_b.rs",
            EdgeRelationKind::Imports,
            EdgeConfidence::Extracted,
            "root.rs",
        ),
        StructuralEdge::new(
            "lvl1_a.rs",
            "lvl2_a.rs",
            EdgeRelationKind::Imports,
            EdgeConfidence::Extracted,
            "lvl1_a.rs",
        ),
    ];

    let mut sources = HashMap::new();
    sources.insert("root.rs".to_string(), "pub struct Root;\n".to_string());
    sources.insert("lvl1_a.rs".to_string(), "pub struct Lvl1A;\n".to_string());
    sources.insert("lvl1_b.rs".to_string(), "pub struct Lvl1B;\n".to_string());
    sources.insert("lvl2_a.rs".to_string(), "pub struct Lvl2A;\n".to_string());

    let slicer = GraphContextSlicer::new();

    // BFS order: root -> lvl1_a, lvl1_b -> lvl2_a
    let bfs_outcome = slicer.slice_graph_with_sources(
        &nodes,
        &edges,
        &sources,
        "root.rs",
        TraversalMode::Bfs,
        3,
        100,
    );

    // DFS order: root -> lvl1_a -> lvl2_a -> lvl1_b
    let dfs_outcome = slicer.slice_graph_with_sources(
        &nodes,
        &edges,
        &sources,
        "root.rs",
        TraversalMode::Dfs,
        3,
        100,
    );

    let bfs_nodes = match bfs_outcome {
        GraphTraversalOutcome::VisitedSlice(s) => s.visited_nodes,
        _ => panic!("Expected BFS VisitedSlice"),
    };

    let dfs_nodes = match dfs_outcome {
        GraphTraversalOutcome::VisitedSlice(s) => s.visited_nodes,
        _ => panic!("Expected DFS VisitedSlice"),
    };

    assert_eq!(
        bfs_nodes,
        vec!["root.rs", "lvl1_a.rs", "lvl1_b.rs", "lvl2_a.rs"]
    );
    assert_eq!(
        dfs_nodes,
        vec!["root.rs", "lvl1_a.rs", "lvl2_a.rs", "lvl1_b.rs"]
    );
}

#[test]
fn test_graph_slicer_prioritizes_extracted_over_inferred() {
    let nodes = vec![
        StructuralNode::new(
            "root.rs",
            "root.rs",
            GraphNodeKind::CodeFile,
            "root.rs",
            "CORE",
        ),
        StructuralNode::new(
            "a_inferred.rs",
            "a_inferred.rs",
            GraphNodeKind::CodeFile,
            "a_inferred.rs",
            "CORE",
        ),
        StructuralNode::new(
            "z_extracted.rs",
            "z_extracted.rs",
            GraphNodeKind::CodeFile,
            "z_extracted.rs",
            "CORE",
        ),
    ];

    // Note: a_inferred is alphabetically earlier than z_extracted, but z_extracted is Extracted!
    let edges = vec![
        StructuralEdge::new(
            "root.rs",
            "a_inferred.rs",
            EdgeRelationKind::ReferencesSymbol,
            EdgeConfidence::Inferred,
            "root.rs",
        ),
        StructuralEdge::new(
            "root.rs",
            "z_extracted.rs",
            EdgeRelationKind::Imports,
            EdgeConfidence::Extracted,
            "root.rs",
        ),
    ];

    let mut sources = HashMap::new();
    sources.insert("root.rs".to_string(), "pub struct Root;\n".to_string());
    sources.insert(
        "a_inferred.rs".to_string(),
        "pub struct Inferred;\n".to_string(),
    );
    sources.insert(
        "z_extracted.rs".to_string(),
        "pub struct Extracted;\n".to_string(),
    );

    let slicer = GraphContextSlicer::new();
    let outcome = slicer.slice_graph_with_sources(
        &nodes,
        &edges,
        &sources,
        "root.rs",
        TraversalMode::Bfs,
        1,
        100,
    );

    match outcome {
        GraphTraversalOutcome::VisitedSlice(s) => {
            assert_eq!(s.visited_nodes[0], "root.rs");
            assert_eq!(s.visited_nodes[1], "z_extracted.rs");
            assert_eq!(s.visited_nodes[2], "a_inferred.rs");
        }
        _ => panic!("Expected VisitedSlice"),
    }
}

#[test]
fn test_graph_slicer_empty_slice_when_root_unknown() {
    let slicer = GraphContextSlicer::new();
    let outcome = slicer.slice_graph_with_sources(
        &[],
        &[],
        &HashMap::new(),
        "non_existent.rs",
        TraversalMode::Bfs,
        2,
        50,
    );
    assert_eq!(outcome, GraphTraversalOutcome::EmptyGraphSlice);
}

#[test]
fn test_ast_context_slicer_backward_compatibility() {
    let slicer = AstContextSlicer::new();
    let req = trustkit_core::entities::ContextSliceRequest {
        target_files: vec!["test.rs".to_string()],
        max_line_budget: 10,
        symbol_filters: vec![],
    };
    let res = slicer.slice_code("test.rs", "pub struct Foo;\nfn private() {}", &req);
    assert!(res.sliced_content.contains("pub struct Foo;"));
    assert!(!res.sliced_content.contains("private"));
}
