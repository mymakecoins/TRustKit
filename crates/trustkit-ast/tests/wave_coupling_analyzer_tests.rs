// Integration tests for WaveCouplingAnalyzer and WaveExecutionPlanner (TASK-005).
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use trustkit_ast::wave_coupling_analyzer::WaveCouplingAnalyzer;
use trustkit_ast::wave_planner::WaveExecutionPlanner;
use trustkit_core::entities::{
    EdgeConfidence, EdgeRelationKind, StructuralEdge, TaskNode,
};

fn make_task(id: &str, layer: &str, deps: &[&str], files: &[&str]) -> TaskNode {
    TaskNode {
        task_id: id.to_string(),
        title: format!("Task {id}"),
        architectural_layer: layer.to_string(),
        depends_on: deps.iter().map(|s| s.to_string()).collect(),
        expected_files: files.iter().map(|s| s.to_string()).collect(),
        target_test_file: format!("tests/{id}_tests.rs"),
    }
}

#[test]
fn test_wave_planner_serializes_ast_coupled_tasks() {
    let tasks = vec![
        make_task("TASK-001", "CORE", &[], &["crates/trustkit-core/src/ports.rs"]),
        make_task("TASK-002", "AST", &[], &["crates/trustkit-ast/src/ast_adapter.rs"]),
    ];

    let edges = vec![StructuralEdge::new(
        "crates/trustkit-ast/src/ast_adapter.rs",
        "crates/trustkit-core/src/ports.rs",
        EdgeRelationKind::Imports,
        EdgeConfidence::Extracted,
        "crates/trustkit-ast/src/ast_adapter.rs",
    )];

    let planner = WaveExecutionPlanner::with_edges(edges);
    let schedule = planner
        .plan_waves(&tasks)
        .expect("Wave planning should succeed");

    assert_eq!(schedule.waves.len(), 2);
    assert_eq!(schedule.waves[0].tasks[0].task_id, "TASK-001");
    assert_eq!(schedule.waves[1].tasks[0].task_id, "TASK-002");
}

#[test]
fn test_wave_planner_parallel_when_ast_independent() {
    let tasks = vec![
        make_task("TASK-001", "CORE", &[], &["crates/alpha.rs"]),
        make_task("TASK-002", "CORE", &[], &["crates/beta.rs"]),
    ];

    // Edges exist, but neither connects alpha to beta
    let edges = vec![
        StructuralEdge::new("crates/alpha.rs", "crates/gamma.rs", EdgeRelationKind::Imports, EdgeConfidence::Extracted, "crates/alpha.rs"),
        StructuralEdge::new("crates/beta.rs", "crates/delta.rs", EdgeRelationKind::Imports, EdgeConfidence::Extracted, "crates/beta.rs"),
    ];

    let planner = WaveExecutionPlanner::with_edges(edges);
    let schedule = planner
        .plan_waves(&tasks)
        .expect("Wave planning should succeed");

    assert_eq!(schedule.waves.len(), 1);
    assert_eq!(schedule.waves[0].tasks.len(), 2);
    assert_eq!(schedule.waves[0].tasks[0].task_id, "TASK-001");
    assert_eq!(schedule.waves[0].tasks[1].task_id, "TASK-002");
}

#[test]
fn test_wave_planner_serializes_god_node_colliding_tasks() {
    let tasks = vec![
        make_task("TASK-010", "CORE", &[], &["crates/contract_a.rs"]),
        make_task("TASK-020", "CORE", &[], &["crates/contract_b.rs"]),
    ];

    // god_node.rs imports multiple contracts (high degree), consuming both contract_a and contract_b
    let edges = vec![
        StructuralEdge::new("crates/god_node.rs", "crates/contract_a.rs", EdgeRelationKind::Imports, EdgeConfidence::Extracted, "crates/god_node.rs"),
        StructuralEdge::new("crates/god_node.rs", "crates/contract_b.rs", EdgeRelationKind::Imports, EdgeConfidence::Extracted, "crates/god_node.rs"),
        StructuralEdge::new("crates/god_node.rs", "crates/contract_c.rs", EdgeRelationKind::Imports, EdgeConfidence::Extracted, "crates/god_node.rs"),
    ];

    // With god_node_threshold = 3, god_node has total_degree 3 (or out/in degree >= 3)
    let planner = WaveExecutionPlanner::with_edges_and_threshold(edges, 3);
    let schedule = planner
        .plan_waves(&tasks)
        .expect("Wave planning should succeed");

    assert_eq!(schedule.waves.len(), 2);
    assert_eq!(schedule.waves[0].tasks[0].task_id, "TASK-010");
    assert_eq!(schedule.waves[1].tasks[0].task_id, "TASK-020");
}

#[test]
fn test_wave_coupling_analyzer_preserves_acyclicity() {
    let tasks = vec![
        make_task("TASK-001", "CORE", &["TASK-002"], &["crates/f1.rs"]),
        make_task("TASK-002", "CORE", &[], &["crates/f2.rs"]),
    ];

    // Suppose f1 imports f2, which suggests TASK-002 before TASK-001,
    // which aligns with explicit dependency (TASK-001 depends on TASK-002).
    let edges = vec![StructuralEdge::new(
        "crates/f1.rs",
        "crates/f2.rs",
        EdgeRelationKind::Imports,
        EdgeConfidence::Extracted,
        "crates/f1.rs",
    )];

    let analyzer = WaveCouplingAnalyzer::new(3);
    let deps = analyzer.detect_coupling_dependencies(&tasks, &edges);
    assert!(!deps.is_empty());
}
