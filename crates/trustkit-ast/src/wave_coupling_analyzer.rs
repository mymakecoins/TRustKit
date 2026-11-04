// Wave coupling analyzer for AST dependencies and God Node collisions (TASK-005).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use petgraph::algo::has_path_connecting;
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::{HashMap, HashSet};
use trustkit_core::entities::{EdgeConfidence, EdgeRelationKind, StructuralEdge, TaskNode};

#[derive(Debug, Clone, Copy)]
pub struct WaveCouplingAnalyzer {
    god_node_threshold: usize,
}

impl WaveCouplingAnalyzer {
    pub const fn new(god_node_threshold: usize) -> Self {
        Self {
            god_node_threshold: if god_node_threshold == 0 {
                3
            } else {
                god_node_threshold
            },
        }
    }

    pub fn detect_coupling_dependencies(
        &self,
        tasks: &[TaskNode],
        edges: &[StructuralEdge],
    ) -> Vec<(String, String)> {
        let mut deps = Vec::new();
        let mut seen = HashSet::new();

        // 1. Direct AST coupling: if task T_j's file imports/references task T_i's file
        // then T_i must precede T_j (T_i -> T_j).
        for (i, t_i) in tasks.iter().enumerate() {
            for (j, t_j) in tasks.iter().enumerate() {
                if i == j {
                    continue;
                }
                if Self::has_ast_coupling(t_j, t_i, edges) {
                    Self::insert_dep(&t_i.task_id, &t_j.task_id, &mut deps, &mut seen);
                }
            }
        }

        // 2. God Node collisions: tasks altering neighbors of the same God Node
        let god_nodes = self.identify_god_nodes(edges);
        for god_node in &god_nodes {
            for (i, t_i) in tasks.iter().enumerate() {
                for (j, t_j) in tasks.iter().enumerate() {
                    if i >= j {
                        continue;
                    }
                    let t_i_collides = Self::task_touches_god_node(t_i, god_node, edges);
                    let t_j_collides = Self::task_touches_god_node(t_j, god_node, edges);

                    if t_i_collides && t_j_collides {
                        let (earlier, later) = if t_i.task_id <= t_j.task_id {
                            (&t_i.task_id, &t_j.task_id)
                        } else {
                            (&t_j.task_id, &t_i.task_id)
                        };
                        Self::insert_dep(earlier, later, &mut deps, &mut seen);
                    }
                }
            }
        }

        deps
    }

    pub fn apply_coupling_edges(
        &self,
        graph: &mut DiGraph<String, ()>,
        node_indices: &HashMap<String, NodeIndex>,
        tasks: &[TaskNode],
        edges: &[StructuralEdge],
    ) {
        let deps = self.detect_coupling_dependencies(tasks, edges);
        for (earlier_id, later_id) in deps {
            if let (Some(&earlier_idx), Some(&later_idx)) =
                (node_indices.get(&earlier_id), node_indices.get(&later_id))
                && earlier_idx != later_idx
                && !has_path_connecting(&*graph, later_idx, earlier_idx, None)
                && !has_path_connecting(&*graph, earlier_idx, later_idx, None)
            {
                graph.add_edge(earlier_idx, later_idx, ());
            }
        }
    }

    fn insert_dep(
        from: &str,
        to: &str,
        deps: &mut Vec<(String, String)>,
        seen: &mut HashSet<(String, String)>,
    ) {
        let key = (from.to_string(), to.to_string());
        if !seen.contains(&key) && from != to {
            seen.insert(key.clone());
            deps.push(key);
        }
    }

    fn has_ast_coupling(
        consumer_task: &TaskNode,
        provider_task: &TaskNode,
        edges: &[StructuralEdge],
    ) -> bool {
        for consumer_file in &consumer_task.expected_files {
            for provider_file in &provider_task.expected_files {
                let coupled = edges.iter().any(|edge| {
                    let is_extracted = edge.confidence == EdgeConfidence::Extracted;
                    let is_ref = edge.relation == EdgeRelationKind::Imports
                        || edge.relation == EdgeRelationKind::ReferencesSymbol;

                    if !is_extracted || !is_ref {
                        return false;
                    }

                    let from_consumer =
                        &edge.source_file == consumer_file || &edge.source_id == consumer_file;
                    let to_provider = &edge.target_id == provider_file
                        || edge.target_id.starts_with(&format!("{provider_file}::"));

                    from_consumer && to_provider
                });

                if coupled {
                    return true;
                }
            }
        }
        false
    }

    fn identify_god_nodes(&self, edges: &[StructuralEdge]) -> HashSet<String> {
        let mut degrees: HashMap<String, usize> = HashMap::new();

        for edge in edges {
            *degrees.entry(edge.source_file.clone()).or_default() += 1;
            *degrees.entry(edge.target_id.clone()).or_default() += 1;
        }

        degrees
            .into_iter()
            .filter(|(_, deg)| *deg >= self.god_node_threshold)
            .map(|(node, _)| node)
            .collect()
    }

    fn task_touches_god_node(task: &TaskNode, god_node: &str, edges: &[StructuralEdge]) -> bool {
        for file in &task.expected_files {
            if file == god_node {
                return true;
            }
            let is_neighbor = edges.iter().any(|e| {
                (&e.source_file == file || &e.source_id == file) && e.target_id == god_node
                    || (e.source_file == god_node || e.source_id == god_node)
                        && (&e.target_id == file || e.target_id.starts_with(&format!("{file}::")))
            });
            if is_neighbor {
                return true;
            }
        }
        false
    }
}
