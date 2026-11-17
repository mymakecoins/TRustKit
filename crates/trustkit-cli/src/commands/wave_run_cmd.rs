// Handler for `trustkit wave run` (TASK-014 / TASK-007).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::cli_contract::WaveRunArgs;
use crate::error_handler::DomainOutcome;
use crate::gate_presenter::{GatePresenter, JsonWaveRunResponse, WorktreeProvisionInfo};
use crate::next_step_presenter::NextStepPresenter;
use std::path::Path;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use trustkit_core::entities::{GateVerdict, NextStepResolver, RuntimeBackendMode};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::{CycleEvidenceRepositoryPort, TerminalMultiplexerPort};
use trustkit_core::special_cases::ActiveCycleLookup;
use trustkit_core::use_cases::run_execution_wave::{self, RunExecutionWaveRequest};
use trustkit_core::use_cases::run_interactive_wave::{self, RunInteractiveWaveRequest};
use trustkit_db::sqlite_adapter::SqliteWalRepositoryAdapter;
use trustkit_orch::{SupervisorRuntimeSelector, WorktreeIsolationProvider};

fn current_timestamp_iso() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{now}")
}

pub fn execute(args: &WaveRunArgs, json_mode: bool) -> Result<DomainOutcome, DomainViolation> {
    let db_path = Path::new(".trustkit").join("state.db");
    execute_with_db(args, json_mode, &db_path)
}

pub fn execute_with_db(
    args: &WaveRunArgs,
    json_mode: bool,
    db_path: &Path,
) -> Result<DomainOutcome, DomainViolation> {
    if !db_path.exists() {
        return Err(DomainViolation::InfrastructureBoundaryFault {
            subsystem: "wave_run".to_string(),
            details: "No active SDD cycle found in state.db. Run 'trustkit init' first."
                .to_string(),
        });
    }

    let repo = SqliteWalRepositoryAdapter::open(db_path)?;
    let active_cycle = match repo.find_active_cycle()? {
        ActiveCycleLookup::Found(cycle) => *cycle,
        ActiveCycleLookup::NoActiveCycle => {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "wave_run".to_string(),
                details: "No active SDD cycle found. Run 'trustkit init' first.".to_string(),
            });
        }
    };

    let start = Instant::now();
    let repo_root = Path::new(".");
    let worktree_provider = WorktreeIsolationProvider::new(repo_root);
    let now_iso = current_timestamp_iso();

    let selector = SupervisorRuntimeSelector::new();
    let mode = RuntimeBackendMode::from_str_name(&args.runtime);
    let resolved = selector.resolve(mode)?;

    let (successful_tasks, failed_tasks, verdict_str) =
        if resolved.multiplexer.is_some() || !args.agent_cmd.is_empty() {
            let req = RunInteractiveWaveRequest {
                cycle_id: active_cycle.cycle_id.clone(),
                wave_index: args.wave,
                runtime_mode: resolved.active_mode,
                agent_command: args.agent_cmd.clone(),
                now_iso,
            };
            let mux_ref = resolved
                .multiplexer
                .as_ref()
                .map(|m| m as &dyn TerminalMultiplexerPort);
            let outcome = run_interactive_wave::execute(req, &repo, &worktree_provider, mux_ref)?;
            let mut succ = Vec::new();
            let mut fail = Vec::new();
            for t in outcome.tasks_dispatched {
                if t.state.is_done() {
                    succ.push(t.task_id);
                } else {
                    fail.push(t.task_id);
                }
            }
            let v = match outcome.overall_verdict {
                GateVerdict::Pass => "PASS".to_string(),
                GateVerdict::Blocked => "BLOCKED".to_string(),
                GateVerdict::Fail => "FAIL".to_string(),
                GateVerdict::Info => "INFO".to_string(),
            };
            (succ, fail, v)
        } else {
            let request = RunExecutionWaveRequest {
                cycle_id: active_cycle.cycle_id.clone(),
                wave_index: args.wave,
                now_iso,
            };
            let outcome = run_execution_wave::execute(request, &repo, &worktree_provider)?;
            let succ: Vec<String> = outcome
                .tasks_executed
                .iter()
                .map(|t| t.task_id.clone())
                .collect();
            (succ, Vec::new(), "PASS".to_string())
        };

    let duration_ms = start.elapsed().as_millis() as u64;

    let all_tasks = repo
        .list_atomic_tasks(&active_cycle.cycle_id)
        .unwrap_or_default();
    let first_pending_task = all_tasks
        .iter()
        .find(|t| t.wave_index == args.wave && !t.tdd_lifecycle_state.is_committed())
        .map(|t| t.task_id.as_str());
    let next_wave_index = all_tasks
        .iter()
        .filter(|t| t.wave_index > args.wave && !t.tdd_lifecycle_state.is_committed())
        .map(|t| t.wave_index)
        .min();

    let steps = NextStepResolver::for_wave_run(
        args.wave,
        &failed_tasks,
        next_wave_index,
        first_pending_task,
    );

    let mut provisioned_worktrees = Vec::new();
    for task_id in &successful_tasks {
        if let Ok(Some(task)) = repo.find_atomic_task(&active_cycle.cycle_id, task_id) {
            let branch_name = task
                .worktree_allocation
                .branch_name()
                .map(String::from)
                .unwrap_or_else(|| format!("worktree/{task_id}"));
            let worktree_path = task
                .worktree_allocation
                .worktree_path()
                .map(String::from)
                .unwrap_or_else(|| format!(".trustkit/worktrees/{task_id}"));
            let base_branch = task
                .worktree_allocation
                .base_branch()
                .filter(|s| !s.is_empty())
                .map(String::from)
                .unwrap_or_else(|| active_cycle.work_branch.clone());

            provisioned_worktrees.push(WorktreeProvisionInfo {
                task_id: task_id.clone(),
                branch_name,
                worktree_path,
                base_branch,
            });
        } else {
            provisioned_worktrees.push(WorktreeProvisionInfo {
                task_id: task_id.clone(),
                branch_name: format!("worktree/{task_id}"),
                worktree_path: format!(".trustkit/worktrees/{task_id}"),
                base_branch: active_cycle.work_branch.clone(),
            });
        }
    }

    let response = JsonWaveRunResponse {
        verdict: verdict_str,
        wave_index: args.wave,
        total_tasks: successful_tasks.len() + failed_tasks.len(),
        successful_tasks,
        failed_tasks,
        duration_ms,
        runtime_used: resolved.active_mode.as_str().to_string(),
        provisioned_worktrees,
        next_steps: NextStepPresenter::to_json_vec(&steps),
    };

    println!(
        "{}",
        GatePresenter::render_wave_run(&response, json_mode || args.json)
    );
    Ok(DomainOutcome::Success)
}
