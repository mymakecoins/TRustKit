// Handler for `trustkit wave run` (TASK-014 / TASK-007).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::cli_contract::WaveRunArgs;
use crate::error_handler::DomainOutcome;
use crate::gate_presenter::{GatePresenter, JsonWaveRunResponse};
use std::path::Path;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use trustkit_core::entities::{GateVerdict, RuntimeBackendMode};
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
                cycle_id: active_cycle.cycle_id,
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
                cycle_id: active_cycle.cycle_id,
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

    let response = JsonWaveRunResponse {
        verdict: verdict_str,
        wave_index: args.wave,
        total_tasks: successful_tasks.len() + failed_tasks.len(),
        successful_tasks,
        failed_tasks,
        duration_ms,
        runtime_used: resolved.active_mode.as_str().to_string(),
    };

    println!("{}", GatePresenter::render_wave_run(&response, json_mode));
    Ok(DomainOutcome::Success)
}
