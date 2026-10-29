// Handler for `trustkit wave run` (TASK-014).
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
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::CycleEvidenceRepositoryPort;
use trustkit_core::special_cases::ActiveCycleLookup;
use trustkit_core::use_cases::run_execution_wave::{self, RunExecutionWaveRequest};
use trustkit_db::sqlite_adapter::SqliteWalRepositoryAdapter;
use trustkit_orch::WorktreeIsolationProvider;

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
            details: "No active SDD cycle found in state.db. Run 'trustkit init' first.".to_string(),
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

    let request = RunExecutionWaveRequest {
        cycle_id: active_cycle.cycle_id,
        wave_index: args.wave,
        now_iso,
    };

    let outcome = run_execution_wave::execute(request, &repo, &worktree_provider)?;
    let duration_ms = start.elapsed().as_millis() as u64;

    let successful_tasks: Vec<String> = outcome
        .tasks_executed
        .iter()
        .map(|t| t.task_id.clone())
        .collect();

    let response = JsonWaveRunResponse {
        verdict: "PASS".to_string(),
        wave_index: outcome.wave_index,
        total_tasks: outcome.tasks_executed.len(),
        successful_tasks,
        failed_tasks: Vec::new(),
        duration_ms,
    };

    println!("{}", GatePresenter::render_wave_run(&response, json_mode));
    Ok(DomainOutcome::Success)
}
