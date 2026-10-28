// Directed Acyclic Graph (DAG) wave execution planner using petgraph.
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use petgraph::Direction;
use petgraph::algo::has_path_connecting;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;
use trustkit_core::entities::{ExecutionWave, TaskNode, WaveSchedule};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::WaveExecutionPlannerPort;

use crate::task_parser;

#[derive(Debug, Default, Clone, Copy)]
pub struct WaveExecutionPlanner;

impl WaveExecutionPlanner {
    pub fn parse_tasks_dir(dir_path: &Path) -> Result<Vec<TaskNode>, DomainViolation> {
        task_parser::parse_tasks_dir(dir_path)
    }

    fn find_cycle(graph: &DiGraph<String, ()>, nodes: &[NodeIndex]) -> Option<Vec<String>> {
        #[derive(Clone, Copy, PartialEq, Eq)]
        enum VisitStatus {
            Unvisited,
            Visiting,
            Visited,
        }

        let mut status = vec![VisitStatus::Unvisited; graph.node_count()];
        let mut path: Vec<NodeIndex> = Vec::new();

        fn dfs(
            u: NodeIndex,
            graph: &DiGraph<String, ()>,
            status: &mut [VisitStatus],
            path: &mut Vec<NodeIndex>,
        ) -> Option<Vec<NodeIndex>> {
            status[u.index()] = VisitStatus::Visiting;
            path.push(u);

            let mut neighbors: Vec<NodeIndex> = graph.neighbors(u).collect();
            neighbors.sort_by_key(|n| graph[*n].clone());

            for v in neighbors {
                match status[v.index()] {
                    VisitStatus::Visiting => {
                        let mut cycle = Vec::new();
                        let start_pos = path.iter().position(|&node| node == v).unwrap_or(0);
                        cycle.extend_from_slice(&path[start_pos..]);
                        cycle.push(v);
                        return Some(cycle);
                    }
                    VisitStatus::Unvisited => {
                        if let Some(c) = dfs(v, graph, status, path) {
                            return Some(c);
                        }
                    }
                    VisitStatus::Visited => {}
                }
            }

            path.pop();
            status[u.index()] = VisitStatus::Visited;
            None
        }

        for &n in nodes {
            if status[n.index()] == VisitStatus::Unvisited {
                let cycle_opt = dfs(n, graph, &mut status, &mut path);
                if let Some(cycle_nodes) = cycle_opt {
                    let path_names = cycle_nodes
                        .into_iter()
                        .map(|idx| graph[idx].clone())
                        .collect();
                    return Some(path_names);
                }
            }
        }

        None
    }
}

impl WaveExecutionPlannerPort for WaveExecutionPlanner {
    fn plan_waves(&self, tasks: &[TaskNode]) -> Result<WaveSchedule, DomainViolation> {
        let mut graph = DiGraph::<String, ()>::new();
        let mut node_indices = HashMap::new();
        let mut task_map = HashMap::new();

        let mut sorted_tasks = tasks.to_vec();
        sorted_tasks.sort_by(|a, b| a.task_id.cmp(&b.task_id));

        for task in &sorted_tasks {
            let idx = graph.add_node(task.task_id.clone());
            node_indices.insert(task.task_id.clone(), idx);
            task_map.insert(task.task_id.clone(), task.clone());
        }

        // Add explicit dependency edges (A -> B means A must execute before B)
        for task in &sorted_tasks {
            let target_idx = node_indices[&task.task_id];
            for dep in &task.depends_on {
                if let Some(&dep_idx) = node_indices.get(dep) {
                    graph.add_edge(dep_idx, target_idx, ());
                }
            }
        }

        let all_nodes: Vec<NodeIndex> = sorted_tasks
            .iter()
            .map(|t| node_indices[&t.task_id])
            .collect();

        // Check for circular dependencies
        if let Some(cycle_path) = Self::find_cycle(&graph, &all_nodes) {
            let message = format!(
                "Dependencia circular detectada entre tarefas: {}",
                cycle_path.join(" -> ")
            );
            return Err(DomainViolation::CycleDetected {
                cycle_path,
                message,
            });
        }

        // Shared files conflict resolution:
        // If two tasks share any expected file, the one with lower TASK-NNN precedes the other
        for i in 0..sorted_tasks.len() {
            for j in (i + 1)..sorted_tasks.len() {
                let t_i = &sorted_tasks[i];
                let t_j = &sorted_tasks[j];

                let files_i: HashSet<&String> = t_i.expected_files.iter().collect();
                let has_shared_file = t_j.expected_files.iter().any(|f| files_i.contains(f));

                if has_shared_file {
                    let idx_i = node_indices[&t_i.task_id];
                    let idx_j = node_indices[&t_j.task_id];

                    let (earlier, later) = if t_i.task_id <= t_j.task_id {
                        (idx_i, idx_j)
                    } else {
                        (idx_j, idx_i)
                    };

                    if !has_path_connecting(&graph, later, earlier, None)
                        && !has_path_connecting(&graph, earlier, later, None)
                    {
                        graph.add_edge(earlier, later, ());
                    }
                }
            }
        }

        let topo_order =
            petgraph::algo::toposort(&graph, None).map_err(|_| DomainViolation::CycleDetected {
                cycle_path: vec![],
                message: "Dependencia circular detectada apos resolucao de arquivos".to_string(),
            })?;

        let mut wave_indices = HashMap::new();
        for &node in &topo_order {
            let mut max_parent_wave = 0;
            for edge in graph.edges_directed(node, Direction::Incoming) {
                let parent = edge.source();
                let parent_wave = wave_indices.get(&parent).copied().unwrap_or(1);
                if parent_wave > max_parent_wave {
                    max_parent_wave = parent_wave;
                }
            }
            wave_indices.insert(node, max_parent_wave + 1);
        }

        let mut grouped: BTreeMap<usize, Vec<TaskNode>> = BTreeMap::new();
        for task in sorted_tasks {
            let node_idx = node_indices[&task.task_id];
            let wave_num = wave_indices.get(&node_idx).copied().unwrap_or(1);
            grouped.entry(wave_num).or_default().push(task);
        }

        let mut waves = Vec::new();
        for (wave_index, mut wave_tasks) in grouped {
            wave_tasks.sort_by(|a, b| a.task_id.cmp(&b.task_id));
            waves.push(ExecutionWave {
                wave_index,
                tasks: wave_tasks,
            });
        }

        Ok(WaveSchedule {
            cycle_id: "ACTIVE_CYCLE".to_string(),
            total_tasks: tasks.len(),
            waves,
        })
    }
}
