// Multi-hop AST graph context slicer with BFS/DFS and token budgeting (TASK-004).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use trustkit_core::entities::{
    ContextSliceRequest, GraphContextSlice, GraphTraversalOutcome, StructuralEdge, StructuralNode,
    TraversalMode,
};

use crate::context_slicer::AstContextSlicer;

#[derive(Debug, Default, Clone, Copy)]
pub struct GraphContextSlicer {
    ast_slicer: AstContextSlicer,
}

impl GraphContextSlicer {
    pub const fn new() -> Self {
        Self {
            ast_slicer: AstContextSlicer::new(),
        }
    }

    pub fn slice_graph(
        &self,
        nodes: &[StructuralNode],
        edges: &[StructuralEdge],
        root_file: &str,
        mode: TraversalMode,
        max_hops: usize,
        max_line_budget: usize,
    ) -> GraphTraversalOutcome {
        self.slice_graph_with_sources(
            nodes,
            edges,
            &HashMap::new(),
            root_file,
            mode,
            max_hops,
            max_line_budget,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn slice_graph_with_sources(
        &self,
        nodes: &[StructuralNode],
        edges: &[StructuralEdge],
        sources: &HashMap<String, String>,
        root_file: &str,
        mode: TraversalMode,
        max_hops: usize,
        max_line_budget: usize,
    ) -> GraphTraversalOutcome {
        if root_file.trim().is_empty() {
            return GraphTraversalOutcome::EmptyGraphSlice;
        }

        let exists = sources.contains_key(root_file)
            || nodes
                .iter()
                .any(|n| n.source_file == root_file || n.node_id == root_file)
            || fs::metadata(root_file).is_ok();

        if !exists {
            return GraphTraversalOutcome::EmptyGraphSlice;
        }

        let (visited_nodes, traversed_edges) = match mode {
            TraversalMode::Bfs => self.traverse_bfs(nodes, edges, root_file, max_hops),
            TraversalMode::Dfs => self.traverse_dfs(nodes, edges, root_file, max_hops),
        };

        if visited_nodes.is_empty() {
            return GraphTraversalOutcome::EmptyGraphSlice;
        }

        let mut lines = Vec::new();
        let budget = if max_line_budget == 0 {
            100
        } else {
            max_line_budget
        };

        for path in &visited_nodes {
            if lines.len() >= budget {
                break;
            }
            let content = if let Some(src) = sources.get(path) {
                src.clone()
            } else if let Ok(src) = fs::read_to_string(path) {
                src
            } else {
                continue;
            };

            let remaining = budget.saturating_sub(lines.len());
            if remaining == 0 {
                break;
            }

            let req = ContextSliceRequest {
                target_files: vec![path.clone()],
                symbol_filters: vec![],
                max_line_budget: remaining,
            };

            let slice = self.ast_slicer.slice_code(path, &content, &req);
            for l in slice.sliced_content.lines() {
                if lines.len() >= budget {
                    break;
                }
                lines.push(l.to_string());
            }
        }

        let consumed_lines = lines.len();
        let sliced_content = lines.join("\n");
        GraphTraversalOutcome::VisitedSlice(GraphContextSlice {
            sliced_content,
            visited_nodes,
            traversed_edges,
            consumed_lines,
        })
    }

    fn traverse_bfs(
        &self,
        nodes: &[StructuralNode],
        edges: &[StructuralEdge],
        root_file: &str,
        max_hops: usize,
    ) -> (Vec<String>, Vec<StructuralEdge>) {
        let mut visited = HashSet::new();
        let mut order = Vec::new();
        let mut traversed = Vec::new();
        let mut queue = VecDeque::new();

        visited.insert(root_file.to_string());
        order.push(root_file.to_string());
        queue.push_back((root_file.to_string(), 0));

        while let Some((curr, hop)) = queue.pop_front() {
            if hop >= max_hops {
                continue;
            }
            for (edge, target) in self.get_sorted_edges(&curr, nodes, edges) {
                if visited.insert(target.clone()) {
                    order.push(target.clone());
                    traversed.push(edge);
                    queue.push_back((target, hop + 1));
                }
            }
        }
        (order, traversed)
    }

    fn traverse_dfs(
        &self,
        nodes: &[StructuralNode],
        edges: &[StructuralEdge],
        root_file: &str,
        max_hops: usize,
    ) -> (Vec<String>, Vec<StructuralEdge>) {
        let mut visited = HashSet::new();
        let mut order = Vec::new();
        let mut traversed = Vec::new();

        visited.insert(root_file.to_string());
        order.push(root_file.to_string());

        self.dfs_step(
            root_file,
            0,
            max_hops,
            nodes,
            edges,
            &mut visited,
            &mut order,
            &mut traversed,
        );
        (order, traversed)
    }

    #[allow(clippy::too_many_arguments)]
    fn dfs_step(
        &self,
        curr: &str,
        hop: usize,
        max_hops: usize,
        nodes: &[StructuralNode],
        edges: &[StructuralEdge],
        visited: &mut HashSet<String>,
        order: &mut Vec<String>,
        traversed: &mut Vec<StructuralEdge>,
    ) {
        if hop >= max_hops {
            return;
        }
        for (edge, target) in self.get_sorted_edges(curr, nodes, edges) {
            if visited.insert(target.clone()) {
                order.push(target.clone());
                traversed.push(edge);
                self.dfs_step(
                    &target,
                    hop + 1,
                    max_hops,
                    nodes,
                    edges,
                    visited,
                    order,
                    traversed,
                );
            }
        }
    }

    fn get_sorted_edges(
        &self,
        file_path: &str,
        nodes: &[StructuralNode],
        edges: &[StructuralEdge],
    ) -> Vec<(StructuralEdge, String)> {
        let mut outgoing = Vec::new();
        for edge in edges {
            if edge.source_file != file_path && edge.source_id != file_path {
                continue;
            }
            let target = resolve_target_file(&edge.target_id, nodes);
            if target == file_path {
                continue;
            }
            outgoing.push((edge.clone(), target));
        }

        outgoing.sort_by(|(e1, t1), (e2, t2)| {
            e2.confidence
                .priority_weight()
                .cmp(&e1.confidence.priority_weight())
                .then_with(|| t1.cmp(t2))
        });
        outgoing
    }
}

fn resolve_target_file(target_id: &str, nodes: &[StructuralNode]) -> String {
    for node in nodes {
        if node.node_id == target_id {
            return node.source_file.clone();
        }
    }
    target_id.to_string()
}
