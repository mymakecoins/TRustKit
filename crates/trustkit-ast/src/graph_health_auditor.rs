// Graph health auditor and hexagonal architecture boundary validator (TASK-006).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use trustkit_core::entities::{
    GodNodeMetric, GraphHealthReport, GraphHealthVerdict, GraphTraversalOutcome,
    ShortestPathOutcome, StructuralEdge, StructuralNode, SurprisingEdge, TraversalMode,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::StructuralGraphAnalyzerPort;

use crate::ast_graph_extractor::detect_layer;
use crate::graph_context_slicer::GraphContextSlicer;

#[derive(Debug, Default, Clone, Copy)]
pub struct GraphHealthAuditor;

impl GraphHealthAuditor {
    pub const fn new() -> Self {
        Self
    }

    pub fn compute_cohesion_scores(
        &self,
        nodes: &[StructuralNode],
        edges: &[StructuralEdge],
    ) -> BTreeMap<String, f64> {
        let node_layer: HashMap<&str, &str> = nodes
            .iter()
            .map(|n| (n.node_id.as_str(), n.architectural_layer.as_str()))
            .collect();

        let mut internal_counts: HashMap<String, usize> = HashMap::new();
        let mut total_counts: HashMap<String, usize> = HashMap::new();

        for edge in edges {
            let s_layer = node_layer.get(edge.source_id.as_str()).copied().unwrap_or("");
            let t_layer = node_layer.get(edge.target_id.as_str()).copied().unwrap_or("");

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
            let ratio = if total > 0 { internal as f64 / total as f64 } else { 1.0 };
            result.insert(layer, ratio);
        }
        result
    }
}

impl StructuralGraphAnalyzerPort for GraphHealthAuditor {
    fn audit_graph_health(
        &self,
        nodes: &[StructuralNode],
        edges: &[StructuralEdge],
        top_god_nodes: usize,
    ) -> Result<GraphHealthReport, DomainViolation> {
        let mut known: HashSet<&str> = HashSet::with_capacity(nodes.len() * 2);
        let mut layers: HashMap<&str, &str> = HashMap::with_capacity(nodes.len());
        for n in nodes {
            known.insert(n.node_id.as_str());
            known.insert(n.source_file.as_str());
            layers.insert(n.node_id.as_str(), n.architectural_layer.as_str());
        }

        let mut dangling = Vec::new();
        let mut loops = Vec::new();
        let mut counts: HashMap<(&str, &str), usize> = HashMap::with_capacity(edges.len());
        let mut in_deg: HashMap<&str, usize> = HashMap::with_capacity(nodes.len());
        let mut out_deg: HashMap<&str, usize> = HashMap::with_capacity(nodes.len());
        let mut surprising = Vec::new();

        for e in edges {
            let s_id = e.source_id.as_str();
            let t_id = e.target_id.as_str();
            if !known.contains(s_id) || !known.contains(t_id) {
                dangling.push(e.clone());
            }
            if s_id == t_id {
                loops.push(e.clone());
            }
            *counts.entry((s_id, t_id)).or_default() += 1;
            *out_deg.entry(s_id).or_default() += 1;
            *in_deg.entry(t_id).or_default() += 1;

            let s_str: String;
            let s_l = match layers.get(s_id) {
                Some(&l) => l,
                None => { s_str = detect_layer(s_id); &s_str }
            };
            let t_str: String;
            let t_l = match layers.get(t_id) {
                Some(&l) => l,
                None => { t_str = detect_layer(t_id); &t_str }
            };

            if let Some(r) = check_hexagonal_violation(s_l, t_l, t_id) {
                surprising.push(SurprisingEdge::new(s_id, t_id, s_l, t_l, r));
            }
        }

        let collapsed: usize = counts.values().filter(|&&c| c > 1).map(|c| c - 1).sum();

        // Sort nodes by degree first, then collect neighbors only for top God Nodes
        let mut node_indices: Vec<(&str, usize, usize, usize)> = nodes.iter().map(|n| {
            let id = n.node_id.as_str();
            let in_d = in_deg.get(id).copied().unwrap_or(0);
            let out_d = out_deg.get(id).copied().unwrap_or(0);
            (id, in_d, out_d, in_d + out_d)
        }).collect();

        node_indices.sort_by(|a, b| b.3.cmp(&a.3).then_with(|| a.0.cmp(b.0)));
        let limit = if top_god_nodes == 0 { 5 } else { top_god_nodes };
        node_indices.truncate(limit);

        let top_ids: HashSet<&str> = node_indices.iter().map(|x| x.0).collect();
        let mut god_neighbors: HashMap<&str, HashSet<&str>> = HashMap::new();
        for e in edges {
            let s = e.source_id.as_str();
            let t = e.target_id.as_str();
            if top_ids.contains(s) {
                god_neighbors.entry(s).or_default().insert(t);
            }
            if top_ids.contains(t) {
                god_neighbors.entry(t).or_default().insert(s);
            }
        }

        let gods: Vec<GodNodeMetric> = node_indices.into_iter().map(|(id, in_d, out_d, _)| {
            let mut coupled: Vec<String> = god_neighbors.get(id)
                .map(|set| set.iter().map(|&s| s.to_string()).collect())
                .unwrap_or_default();
            coupled.sort();
            GodNodeMetric::new(id, in_d, out_d, coupled)
        }).collect();

        let verdict = if !dangling.is_empty() || !loops.is_empty() || !surprising.is_empty() {
            GraphHealthVerdict::DegradedWarning
        } else {
            GraphHealthVerdict::Healthy
        };

        Ok(GraphHealthReport {
            total_nodes: nodes.len(),
            total_edges: edges.len(),
            dangling_edges: dangling,
            self_loop_edges: loops,
            collapsed_edges_count: collapsed,
            shrink_guard_triggered: false,
            god_nodes: gods,
            surprising_connections: surprising,
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
        if source_id == target_id {
            return Ok(ShortestPathOutcome::PathFound(vec![source_id.to_string()]));
        }
        let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
        for e in edges {
            adj.entry(e.source_id.as_str()).or_default().push(e.target_id.as_str());
        }
        let mut queue = VecDeque::new();
        let mut parent = HashMap::new();
        queue.push_back(source_id);
        parent.insert(source_id, source_id);

        while let Some(curr) = queue.pop_front() {
            if curr == target_id {
                let mut path = vec![target_id.to_string()];
                let mut c = target_id;
                while c != source_id {
                    let p = parent[c];
                    path.push(p.to_string());
                    c = p;
                }
                path.reverse();
                return Ok(ShortestPathOutcome::PathFound(path));
            }
            if let Some(nbrs) = adj.get(curr) {
                for &next in nbrs {
                    if !parent.contains_key(next) {
                        parent.insert(next, curr);
                        queue.push_back(next);
                    }
                }
            }
        }
        Ok(ShortestPathOutcome::NoPathConnected)
    }

    fn slice_graph_context(
        &self,
        nodes: &[StructuralNode],
        edges: &[StructuralEdge],
        root_file: &str,
        mode: TraversalMode,
        max_hops: usize,
        max_line_budget: usize,
    ) -> Result<GraphTraversalOutcome, DomainViolation> {
        Ok(GraphContextSlicer::new().slice_graph(
            nodes, edges, root_file, mode, max_hops, max_line_budget,
        ))
    }
}

fn check_hexagonal_violation(s_layer: &str, t_layer: &str, t_id: &str) -> Option<String> {
    if s_layer == "CORE" && t_layer != "CORE" && t_layer != "SPEC" {
        return Some(format!("Violacao hexagonal: CORE nao pode depender da camada externa '{t_layer}'"));
    }
    if t_layer == "CLI" && s_layer != "CLI" {
        return Some(format!("Violacao hexagonal: camada interna '{s_layer}' nao pode depender de CLI"));
    }
    if s_layer == "DB" && t_layer != "DB" && t_layer != "CORE" {
        return Some(format!("Violacao hexagonal: DB so pode depender de CORE, mas dependeu de '{t_layer}'"));
    }
    if matches!(s_layer, "CORE" | "AST" | "DB" | "ORCH" | "GH" | "CLI") && t_layer == "WIKI" {
        return Some(format!("Violacao de fronteira: codigo em '{s_layer}' nao pode importar wiki '{t_id}'"));
    }
    None
}
