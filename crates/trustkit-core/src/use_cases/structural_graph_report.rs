// Deterministic GRAPH_REPORT.md generator (TASK-007, RF-031, RN-007).
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL & Emojis.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::entities::{FileGraphManifest, GraphHealthReport, StructuralEdge, StructuralNode};
use std::collections::{BTreeMap, HashMap};

pub fn render_graph_report(
    nodes: &[StructuralNode],
    edges: &[StructuralEdge],
    health: &GraphHealthReport,
    manifests: &[FileGraphManifest],
    now_iso: &str,
) -> String {
    let mut out = String::with_capacity(2048);
    out.push_str("# Structural Graph Report\n\n");
    out.push_str(&format!("- Generated At: {}\n", now_iso));
    out.push_str(&format!("- Total Nodes: {}\n", nodes.len()));
    out.push_str(&format!("- Total Edges: {}\n", edges.len()));
    out.push_str(&format!("- Manifested Files: {}\n", manifests.len()));
    out.push_str(&format!(
        "- Health Verdict: {}\n\n",
        health.verdict.as_str()
    ));

    // 1. God Nodes
    out.push_str("## God Nodes\n\n");
    if health.god_nodes.is_empty() {
        out.push_str("No God Nodes detected.\n\n");
    } else {
        out.push_str("| Node ID | In-Degree | Out-Degree | Total Degree | Coupled Neighbors |\n");
        out.push_str("| :--- | :--- | :--- | :--- | :--- |\n");
        for g in &health.god_nodes {
            let coupled = if g.coupled_neighbors.is_empty() {
                "none".to_string()
            } else {
                g.coupled_neighbors.join(", ")
            };
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                g.node_id, g.in_degree, g.out_degree, g.total_degree, coupled
            ));
        }
        out.push('\n');
    }

    // 2. Surprising Connections
    out.push_str("## Surprising Connections\n\n");
    if health.surprising_connections.is_empty() {
        out.push_str("No surprising connections detected.\n\n");
    } else {
        out.push_str("| Source | Target | Source Layer | Target Layer | Reason |\n");
        out.push_str("| :--- | :--- | :--- | :--- | :--- |\n");
        for s in &health.surprising_connections {
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                s.source_id, s.target_id, s.source_layer, s.target_layer, s.reason
            ));
        }
        out.push('\n');
    }

    // 3. Module Cohesion Scores
    out.push_str("## Module Cohesion Scores\n\n");
    let cohesion = compute_module_cohesion(nodes, edges);
    if cohesion.is_empty() {
        out.push_str("No module cohesion data available.\n\n");
    } else {
        out.push_str("| Module / Layer | Internal Edges | Total Edges | Cohesion Ratio |\n");
        out.push_str("| :--- | :--- | :--- | :--- |\n");
        for (layer, (internal, total, ratio)) in &cohesion {
            out.push_str(&format!(
                "| {} | {} | {} | {:.4} |\n",
                layer, internal, total, ratio
            ));
        }
        out.push('\n');
    }

    // 4. Graph Health Audit
    out.push_str("## Graph Health Audit\n\n");
    out.push_str(&format!("- Total Nodes: {}\n", health.total_nodes));
    out.push_str(&format!("- Total Edges: {}\n", health.total_edges));
    out.push_str(&format!(
        "- Dangling Edges Count: {}\n",
        health.dangling_edges.len()
    ));
    out.push_str(&format!(
        "- Self-Loop Edges Count: {}\n",
        health.self_loop_edges.len()
    ));
    out.push_str(&format!(
        "- Collapsed Edges Count: {}\n",
        health.collapsed_edges_count
    ));
    out.push_str(&format!(
        "- Shrink-Guard Triggered: {}\n",
        health.shrink_guard_triggered
    ));
    out.push_str(&format!("- Audit Verdict: {}\n", health.verdict.as_str()));

    out
}

pub fn compute_module_cohesion(
    nodes: &[StructuralNode],
    edges: &[StructuralEdge],
) -> BTreeMap<String, (usize, usize, f64)> {
    let node_layer: HashMap<&str, &str> = nodes
        .iter()
        .map(|n| (n.node_id.as_str(), n.architectural_layer.as_str()))
        .collect();

    let mut internal_counts: HashMap<String, usize> = HashMap::new();
    let mut total_counts: HashMap<String, usize> = HashMap::new();

    for edge in edges {
        let s_layer = node_layer
            .get(edge.source_id.as_str())
            .copied()
            .unwrap_or("");
        let t_layer = node_layer
            .get(edge.target_id.as_str())
            .copied()
            .unwrap_or("");

        if !s_layer.is_empty() {
            *total_counts.entry(s_layer.to_string()).or_default() += 1;
        }
        if !t_layer.is_empty() && t_layer != s_layer {
            *total_counts.entry(t_layer.to_string()).or_default() += 1;
        }
        if !s_layer.is_empty() && s_layer == t_layer {
            *internal_counts.entry(s_layer.to_string()).or_default() += 1;
        }
    }

    let mut result = BTreeMap::new();
    for (layer, total) in total_counts {
        let internal = internal_counts.get(&layer).copied().unwrap_or(0);
        let ratio = if total > 0 {
            internal as f64 / total as f64
        } else {
            1.0
        };
        result.insert(layer, (internal, total, ratio));
    }
    result
}
