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
use trustkit_core::entities::{
    ExecutionWave, StructuralEdge, TaskNode, TaskSliceKind, WaveSchedule,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::WaveExecutionPlannerPort;

use crate::task_parser;
use crate::wave_coupling_analyzer::WaveCouplingAnalyzer;

#[derive(Debug, Default, Clone, Copy)]
pub struct WaveExecutionPlanner;

#[derive(Debug, Clone)]
pub struct ConfiguredWavePlanner {
    pub structural_edges: Vec<StructuralEdge>,
    pub god_node_threshold: usize,
}

impl ConfiguredWavePlanner {
    pub fn plan_waves(&self, tasks: &[TaskNode]) -> Result<WaveSchedule, DomainViolation> {
        WaveExecutionPlanner::plan_with_options(
            tasks,
            &self.structural_edges,
            self.god_node_threshold,
        )
    }
}
impl WaveExecutionPlannerPort for ConfiguredWavePlanner {
    fn plan_waves(&self, tasks: &[TaskNode]) -> Result<WaveSchedule, DomainViolation> {
        self.plan_waves(tasks)
    }
}

impl WaveExecutionPlanner {
    pub const fn new() -> Self {
        Self
    }

    pub fn with_edges(edges: Vec<StructuralEdge>) -> ConfiguredWavePlanner {
        ConfiguredWavePlanner {
            structural_edges: edges,
            god_node_threshold: 3,
        }
    }

    pub fn with_edges_and_threshold(
        edges: Vec<StructuralEdge>,
        god_node_threshold: usize,
    ) -> ConfiguredWavePlanner {
        ConfiguredWavePlanner {
            structural_edges: edges,
            god_node_threshold,
        }
    }

    pub fn parse_tasks_dir(dir_path: &Path) -> Result<Vec<TaskNode>, DomainViolation> {
        task_parser::parse_tasks_dir(dir_path)
    }

    pub fn validate_expand_contract_topology(tasks: &[TaskNode]) -> Result<(), DomainViolation> {
        let expands: Vec<&TaskNode> = tasks
            .iter()
            .filter(|t| t.slice_kind == TaskSliceKind::WideRefactorExpand)
            .collect();
        let migrates: Vec<&TaskNode> = tasks
            .iter()
            .filter(|t| t.slice_kind == TaskSliceKind::WideRefactorMigrateBatch)
            .collect();
        let contracts: Vec<&TaskNode> = tasks
            .iter()
            .filter(|t| t.slice_kind == TaskSliceKind::WideRefactorContract)
            .collect();

        if !contracts.is_empty() && !migrates.is_empty() {
            for contract in &contracts {
                for migrate in &migrates {
                    if !contract.depends_on.contains(&migrate.task_id) {
                        return Err(DomainViolation::AstIntegrityOrInvariantViolation {
                            invariant_id: "RN-004".to_string(),
                            intercepted_command: "trustkit wave plan".to_string(),
                            reason: format!(
                                "Tarefa Contract '{}' viola topologia Expand-Contract: nao declara dependencia do lote migrate '{}'",
                                contract.task_id, migrate.task_id
                            ),
                            remediation: format!(
                                "Declare '{}' no campo depends_on de '{}'.",
                                migrate.task_id, contract.task_id
                            ),
                        });
                    }
                }
            }
        }

        if !migrates.is_empty() && !expands.is_empty() {
            for migrate in &migrates {
                for expand in &expands {
                    if !migrate.depends_on.contains(&expand.task_id) {
                        return Err(DomainViolation::AstIntegrityOrInvariantViolation {
                            invariant_id: "RN-004".to_string(),
                            intercepted_command: "trustkit wave plan".to_string(),
                            reason: format!(
                                "Tarefa MigrateBatch '{}' viola topologia Expand-Contract: nao declara dependencia da tarefa expand '{}'",
                                migrate.task_id, expand.task_id
                            ),
                            remediation: format!(
                                "Declare '{}' no campo depends_on de '{}'.",
                                expand.task_id, migrate.task_id
                            ),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    pub fn plan_waves_with_edges(
        &self,
        tasks: &[TaskNode],
        edges: &[StructuralEdge],
    ) -> Result<WaveSchedule, DomainViolation> {
        Self::plan_with_options(tasks, edges, 3)
    }

    pub fn plan_with_options(
        tasks: &[TaskNode],
        edges: &[StructuralEdge],
        god_node_threshold: usize,
    ) -> Result<WaveSchedule, DomainViolation> {
        let mut sorted_tasks = tasks.to_vec();
        sorted_tasks.sort_by(|a, b| a.task_id.cmp(&b.task_id));

        Self::validate_expand_contract_topology(&sorted_tasks)?;

        let analyzer = WaveCouplingAnalyzer::new(god_node_threshold);
        for task in &mut sorted_tasks {
            let report = analyzer.analyze_task_blast_radius(task, edges);
            task.blast_radius_score = report.blast_radius_score;
            if task.slice_kind == TaskSliceKind::VerticalTracerBullet && report.is_high_risk() {
                task.slice_kind = TaskSliceKind::WideRefactorExpand;
            }
        }

        let mut graph = DiGraph::<String, ()>::new();
        let mut node_indices = HashMap::new();
        let mut task_map = HashMap::new();

        for task in &sorted_tasks {
            let idx = graph.add_node(task.task_id.clone());
            node_indices.insert(task.task_id.clone(), idx);
            task_map.insert(task.task_id.clone(), task.clone());
        }

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

        let analyzer = WaveCouplingAnalyzer::new(god_node_threshold);
        analyzer.apply_coupling_edges(&mut graph, &node_indices, &sorted_tasks, edges);

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
        Self::plan_with_options(tasks, &[], 3)
    }
}
