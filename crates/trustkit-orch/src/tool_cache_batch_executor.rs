// Tool caching, command batching and AST wave analysis cache (TASK-002 / RN-002 / RF-002).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use std::collections::HashMap;
use trustkit_core::entities::{
    AstAuditVerdict, BatchExecutionConfig, BatchGroupingMode, StructuralEdge, StructuralNode,
    ToolCachingPolicy,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::{
    AsyncExecutionSupervisorPort, SupervisedExecutionOutcome, SupervisedExecutionRequest,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskCommandSpec {
    pub task_id: String,
    pub crate_name: String,
    pub wave_index: u32,
    pub request: SupervisedExecutionRequest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedCommandBatch {
    pub group_key: String,
    pub task_ids: Vec<String>,
    pub combined_request: SupervisedExecutionRequest,
    pub individual_requests: Vec<(String, SupervisedExecutionRequest)>,
}

pub struct WaveBatchCommandPlanner;

impl WaveBatchCommandPlanner {
    pub fn plan_batches(
        specs: &[TaskCommandSpec],
        config: &BatchExecutionConfig,
    ) -> Vec<PlannedCommandBatch> {
        if specs.is_empty() {
            return Vec::new();
        }

        if !config.is_batching_active() || specs.len() == 1 {
            return specs.iter().map(Self::single_batch).collect();
        }

        let mut groups: Vec<(String, Vec<&TaskCommandSpec>)> = Vec::new();
        for spec in specs {
            let subcmd = spec
                .request
                .args
                .first()
                .map(String::as_str)
                .unwrap_or("cmd");
            let key = match config.grouping_mode {
                BatchGroupingMode::PerCrate => format!(
                    "w{}:{}:{}:{}:{}",
                    spec.wave_index,
                    spec.crate_name,
                    spec.request.command,
                    subcmd,
                    spec.request.working_dir
                ),
                BatchGroupingMode::PerWave => format!(
                    "w{}:{}:{}:{}",
                    spec.wave_index, spec.request.command, subcmd, spec.request.working_dir
                ),
                BatchGroupingMode::Disabled => spec.task_id.clone(),
            };

            if let Some((_, bucket)) = groups
                .iter_mut()
                .find(|(k, b)| k == &key && b.len() < config.max_batch_size)
            {
                bucket.push(spec);
            } else {
                groups.push((key, vec![spec]));
            }
        }

        groups
            .into_iter()
            .map(|(key, bucket)| Self::combine_bucket(key, &bucket))
            .collect()
    }

    fn single_batch(spec: &TaskCommandSpec) -> PlannedCommandBatch {
        PlannedCommandBatch {
            group_key: spec.task_id.clone(),
            task_ids: vec![spec.task_id.clone()],
            combined_request: spec.request.clone(),
            individual_requests: vec![(spec.task_id.clone(), spec.request.clone())],
        }
    }

    fn combine_bucket(group_key: String, bucket: &[&TaskCommandSpec]) -> PlannedCommandBatch {
        if bucket.len() == 1 {
            return Self::single_batch(bucket[0]);
        }

        let first = bucket[0];
        let mut combined_args = first.request.args.clone();
        let mut task_ids = Vec::with_capacity(bucket.len());
        let mut individual_requests = Vec::with_capacity(bucket.len());

        for (idx, spec) in bucket.iter().enumerate() {
            task_ids.push(spec.task_id.clone());
            individual_requests.push((spec.task_id.clone(), spec.request.clone()));

            if idx > 0 {
                let mut iter = spec.request.args.iter();
                while let Some(arg) = iter.next() {
                    if arg == "--test" || arg == "-p" {
                        if let Some(val) = iter.next() {
                            let pair_exists = combined_args
                                .windows(2)
                                .any(|w| &w[0] == arg && &w[1] == val);
                            if !pair_exists {
                                combined_args.push(arg.clone());
                                combined_args.push(val.clone());
                            }
                        }
                    }
                }
            }
        }

        let max_timeout = bucket
            .iter()
            .map(|s| s.request.timeout_ms)
            .max()
            .unwrap_or(60_000);

        PlannedCommandBatch {
            group_key,
            task_ids,
            combined_request: SupervisedExecutionRequest {
                command: first.request.command.clone(),
                args: combined_args,
                working_dir: first.request.working_dir.clone(),
                timeout_ms: max_timeout,
            },
            individual_requests,
        }
    }

    pub fn execute_batches(
        batches: &[PlannedCommandBatch],
        supervisor: &dyn AsyncExecutionSupervisorPort,
    ) -> Result<Vec<(String, SupervisedExecutionOutcome)>, DomainViolation> {
        let mut results = Vec::new();

        for batch in batches {
            let outcome = supervisor.execute_supervised(&batch.combined_request)?;
            if batch.task_ids.len() == 1 {
                results.push((batch.task_ids[0].clone(), outcome));
                continue;
            }

            if outcome.exit_code == 0 {
                let batch_size = batch.task_ids.len();
                for task_id in &batch.task_ids {
                    let mut tagged = outcome.clone();
                    tagged.stdout_tail = if tagged.stdout_tail.is_empty() {
                        format!("BATCH_EXEC={batch_size} task={task_id}")
                    } else {
                        format!(
                            "BATCH_EXEC={batch_size} task={task_id}\n{}",
                            tagged.stdout_tail
                        )
                    };
                    results.push((task_id.clone(), tagged));
                }
            } else {
                for (task_id, single_req) in &batch.individual_requests {
                    let single_outcome = supervisor.execute_supervised(single_req)?;
                    results.push((task_id.clone(), single_outcome));
                }
            }
        }

        Ok(results)
    }
}

#[derive(Debug, Clone)]
pub struct AstWaveAnalysisCache {
    policy: ToolCachingPolicy,
    ast_entries: HashMap<String, (String, AstAuditVerdict)>,
    coupling_entries: HashMap<String, (String, (Vec<StructuralNode>, Vec<StructuralEdge>))>,
    hits: usize,
    misses: usize,
}

impl AstWaveAnalysisCache {
    pub fn new(policy: ToolCachingPolicy) -> Self {
        Self {
            policy,
            ast_entries: HashMap::new(),
            coupling_entries: HashMap::new(),
            hits: 0,
            misses: 0,
        }
    }

    pub fn compute_content_digest(content: &str) -> String {
        blake3::hash(content.as_bytes()).to_hex().to_string()
    }

    pub fn get_or_analyze_ast<F>(
        &mut self,
        file_path: &str,
        content: &str,
        analyzer: F,
    ) -> Result<AstAuditVerdict, DomainViolation>
    where
        F: FnOnce() -> Result<AstAuditVerdict, DomainViolation>,
    {
        if !self.policy.ast_analysis_cache_enabled {
            self.misses += 1;
            return analyzer();
        }

        let digest = Self::compute_content_digest(content);
        if let Some((cached_digest, cached_verdict)) = self.ast_entries.get(file_path)
            && cached_digest == &digest
        {
            self.hits += 1;
            return Ok(cached_verdict.clone());
        }

        self.misses += 1;
        let verdict = analyzer()?;
        self.ast_entries
            .insert(file_path.to_string(), (digest, verdict.clone()));
        Ok(verdict)
    }

    pub fn get_or_extract_structural_coupling<F>(
        &mut self,
        file_path: &str,
        content: &str,
        extractor: F,
    ) -> Result<(Vec<StructuralNode>, Vec<StructuralEdge>), DomainViolation>
    where
        F: FnOnce() -> Result<(Vec<StructuralNode>, Vec<StructuralEdge>), DomainViolation>,
    {
        if !self.policy.structural_coupling_cache_enabled {
            self.misses += 1;
            return extractor();
        }

        let digest = Self::compute_content_digest(content);
        if let Some((cached_digest, cached_graph)) = self.coupling_entries.get(file_path)
            && cached_digest == &digest
        {
            self.hits += 1;
            return Ok(cached_graph.clone());
        }

        self.misses += 1;
        let graph = extractor()?;
        self.coupling_entries
            .insert(file_path.to_string(), (digest, graph.clone()));
        Ok(graph)
    }

    pub const fn cache_hits(&self) -> usize {
        self.hits
    }

    pub const fn cache_misses(&self) -> usize {
        self.misses
    }

    pub const fn reuse_rate_pct(&self) -> u8 {
        let total = self.hits + self.misses;
        if total == 0 {
            0
        } else {
            ((self.hits * 100) / total) as u8
        }
    }

    pub fn cached_file_count(&self) -> usize {
        self.ast_entries.len() + self.coupling_entries.len()
    }

    pub fn policy(&self) -> &ToolCachingPolicy {
        &self.policy
    }
}
