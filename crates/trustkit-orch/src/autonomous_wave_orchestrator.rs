// Autonomous Multi-Wave Execution Orchestrator (TASK-003).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use std::path::Path;
use trustkit_core::entities::{
    CodeReviewEffortLevel, GateStage, GateVerdict, SemanticReviewFinding, TddPhase,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::{
    AsyncExecutionSupervisorPort, BuilderAgentDispatcherPort, CycleEvidenceRepositoryPort,
    SemanticCodeReviewerPort, WorktreeIsolationPort,
};
use trustkit_core::special_cases::WorktreeAllocation;
use trustkit_core::use_cases::complete_task::{self, CompleteTaskRequest};
use trustkit_core::use_cases::record_tdd_phase::{self, RecordTddPhaseRequest};
use trustkit_core::use_cases::review_task_code::{self, ReviewTaskCodeRequest};

#[derive(Debug, Default, Clone)]
pub struct DefaultSemanticCodeReviewer;

impl DefaultSemanticCodeReviewer {
    pub fn new() -> Self {
        Self
    }
}

impl SemanticCodeReviewerPort for DefaultSemanticCodeReviewer {
    fn review_task_diff(
        &self,
        _cycle_id: &str,
        _task_id: &str,
        _effort_level: CodeReviewEffortLevel,
    ) -> Result<Vec<SemanticReviewFinding>, DomainViolation> {
        Ok(vec![])
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiWaveExecutionRequest<'a> {
    pub cycle_id: &'a str,
    pub wave_indices: &'a [u32],
    pub target_branch: &'a str,
    pub now_iso: &'a str,
}

pub struct MultiWaveDependencies<'a> {
    pub repo: &'a dyn CycleEvidenceRepositoryPort,
    pub worktree_port: &'a dyn WorktreeIsolationPort,
    pub supervisor: &'a dyn AsyncExecutionSupervisorPort,
    pub reviewer: &'a dyn SemanticCodeReviewerPort,
    pub builder_dispatcher: Option<&'a dyn BuilderAgentDispatcherPort>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskExecutionSummary {
    pub task_id: String,
    pub red_verified: bool,
    pub green_verified: bool,
    pub review_passed: bool,
    pub completed: bool,
    pub commit_sha: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiWaveExecutionOutcome {
    pub cycle_id: String,
    pub waves_executed: Vec<u32>,
    pub tasks_executed: Vec<TaskExecutionSummary>,
    pub overall_verdict: GateVerdict,
}

#[derive(Debug, Default, Clone)]
pub struct AutonomousWaveOrchestrator;

impl AutonomousWaveOrchestrator {
    pub fn new() -> Self {
        Self
    }

    fn resolve_test_args(target_test_file: &str, task_id: &str) -> Vec<String> {
        if !target_test_file.is_empty() {
            let path = Path::new(target_test_file);
            let pkg = if target_test_file.starts_with("crates/") {
                path.iter().nth(1).and_then(|s| s.to_str())
            } else {
                None
            };
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                let mut args = vec!["test".to_string()];
                if let Some(p) = pkg {
                    args.push("-p".to_string());
                    args.push(p.to_string());
                }
                if target_test_file.contains("/tests/") || target_test_file.starts_with("tests/") {
                    args.push("--test".to_string());
                    args.push(stem.to_string());
                } else if target_test_file.contains("/src/") || target_test_file.starts_with("src/")
                {
                    args.push("--lib".to_string());
                    let filter = stem
                        .trim_end_matches("_tests")
                        .trim_end_matches("_test")
                        .replace('-', "_");
                    args.push(if filter.is_empty() {
                        stem.replace('-', "_")
                    } else {
                        filter
                    });
                } else {
                    args.push("--".to_string());
                    args.push(stem.replace('-', "_"));
                }
                return args;
            }
        }
        vec![
            "test".to_string(),
            "--".to_string(),
            task_id.to_lowercase().replace('-', "_"),
        ]
    }

    pub fn execute_wave_loop(
        &self,
        req: &MultiWaveExecutionRequest<'_>,
        deps: &MultiWaveDependencies<'_>,
    ) -> Result<MultiWaveExecutionOutcome, DomainViolation> {
        if req.wave_indices.is_empty() {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "autonomous_wave_orchestrator".to_string(),
                details: "No wave indices specified for execution".to_string(),
            });
        }

        let all_tasks = deps.repo.list_atomic_tasks(req.cycle_id)?;
        let mut executed_summaries = Vec::new();

        for wave_idx in req.wave_indices {
            let wave_tasks: Vec<_> = all_tasks
                .iter()
                .filter(|t| t.wave_index == *wave_idx)
                .cloned()
                .collect();

            if wave_tasks.is_empty() {
                return Err(DomainViolation::InfrastructureBoundaryFault {
                    subsystem: "autonomous_wave_orchestrator".to_string(),
                    details: format!(
                        "No tasks found for wave index {wave_idx} in cycle '{}'",
                        req.cycle_id
                    ),
                });
            }

            for task in wave_tasks {
                let allocation = deps.worktree_port.provision_worktree(
                    req.cycle_id,
                    &task.task_id,
                    req.target_branch,
                )?;
                let worktree_path = match allocation {
                    WorktreeAllocation::Provisioned { worktree_path, .. } => worktree_path,
                    _ => format!(".trustkit/worktrees/{}", task.task_id),
                };

                let test_args = Self::resolve_test_args(&task.target_test_file, &task.task_id);

                // Step 1: Ensure RED phase is verified
                let current_task = deps
                    .repo
                    .find_atomic_task(req.cycle_id, &task.task_id)?
                    .unwrap_or(task.clone());

                if !current_task.tdd_lifecycle_state.is_red_verified() {
                    let target = trustkit_ast::ScaffoldingTarget {
                        task_id: task.task_id.clone(),
                        target_test_file: task.target_test_file.clone(),
                        criteria_summary: task.title.clone(),
                    };
                    let _ = trustkit_ast::RedTestScaffolder::scaffold(
                        Path::new(&worktree_path),
                        &target,
                    );

                    let red_req = RecordTddPhaseRequest {
                        cycle_id: req.cycle_id.to_string(),
                        task_id: task.task_id.clone(),
                        phase: TddPhase::Red,
                        test_command: "cargo".to_string(),
                        test_args: test_args.clone(),
                        working_dir: worktree_path.clone(),
                        timeout_ms: 60_000,
                        now_iso: req.now_iso.to_string(),
                    };
                    record_tdd_phase::execute(red_req, deps.repo, deps.supervisor)?;
                }

                // Step 1.5: Dispatch Builder Agent between RED and GREEN (TASK-007 / RF-007)
                if let Some(dispatcher) = deps.builder_dispatcher {
                    dispatcher.dispatch_builder_in_worktree(
                        req.cycle_id,
                        &task.task_id,
                        &worktree_path,
                        &task.target_test_file,
                        &task.expected_files,
                    )?;
                }

                // Step 2: GREEN phase
                let green_req = RecordTddPhaseRequest {
                    cycle_id: req.cycle_id.to_string(),
                    task_id: task.task_id.clone(),
                    phase: TddPhase::Green,
                    test_command: "cargo".to_string(),
                    test_args: test_args.clone(),
                    working_dir: worktree_path.clone(),
                    timeout_ms: 60_000,
                    now_iso: req.now_iso.to_string(),
                };
                record_tdd_phase::execute(green_req, deps.repo, deps.supervisor)?;

                // Step 3: Code Review
                let review_req = ReviewTaskCodeRequest {
                    cycle_id: req.cycle_id.to_string(),
                    task_id: task.task_id.clone(),
                    effort_level: CodeReviewEffortLevel::Medium,
                    max_findings_override: 0,
                    now_iso: req.now_iso.to_string(),
                };
                review_task_code::execute(review_req, deps.repo, deps.reviewer)?;

                // Step 4: Merge and Complete
                let merged_sha = deps.worktree_port.merge_and_prune_worktree(
                    &task.task_id,
                    &worktree_path,
                    req.target_branch,
                )?;

                let complete_req = CompleteTaskRequest {
                    cycle_id: req.cycle_id.to_string(),
                    task_id: task.task_id.clone(),
                    commit_sha: merged_sha.clone(),
                    now_iso: req.now_iso.to_string(),
                };
                complete_task::execute(complete_req, deps.repo)?;

                executed_summaries.push(TaskExecutionSummary {
                    task_id: task.task_id,
                    red_verified: true,
                    green_verified: true,
                    review_passed: true,
                    completed: true,
                    commit_sha: merged_sha,
                });
            }
        }

        deps.repo
            .advance_cycle_gate(req.cycle_id, GateStage::Gate4TasksImplemented)?;

        Ok(MultiWaveExecutionOutcome {
            cycle_id: req.cycle_id.to_string(),
            waves_executed: req.wave_indices.to_vec(),
            tasks_executed: executed_summaries,
            overall_verdict: GateVerdict::Pass,
        })
    }
}
