// Handler for `trustkit close` (TASK-003).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::cli_contract::CloseArgs;
use crate::error_handler::DomainOutcome;
use crate::gate_presenter::JsonCloseResponse;
use crate::presenter::Presenter;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::CycleEvidenceRepositoryPort;
use trustkit_core::special_cases::ActiveCycleLookup;
use trustkit_core::use_cases::close_cycle::{self, CloseCycleRequest};
use trustkit_db::sqlite_adapter::SqliteWalRepositoryAdapter;
use trustkit_gh::ProjectBoardSynchronizer;

fn current_timestamp_iso() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{now}")
}

pub fn execute(args: &CloseArgs, json_mode: bool) -> Result<DomainOutcome, DomainViolation> {
    let db_path = Path::new(".trustkit").join("state.db");
    execute_with_db(args, json_mode, &db_path)
}

pub fn execute_with_db(
    args: &CloseArgs,
    json_mode: bool,
    db_path: &Path,
) -> Result<DomainOutcome, DomainViolation> {
    if !db_path.exists() {
        return Err(DomainViolation::InfrastructureBoundaryFault {
            subsystem: "close".to_string(),
            details: "No active SDD cycle found in state.db. Run 'trustkit init' first."
                .to_string(),
        });
    }

    let repo = SqliteWalRepositoryAdapter::open(db_path)?;
    let active_cycle = match repo.find_active_cycle()? {
        ActiveCycleLookup::Found(cycle) => *cycle,
        ActiveCycleLookup::NoActiveCycle => {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "close".to_string(),
                details: "No active SDD cycle found. Run 'trustkit init' first.".to_string(),
            });
        }
    };

    let board_gateway = ProjectBoardSynchronizer::new();
    let now_iso = current_timestamp_iso();

    let request = CloseCycleRequest {
        cycle_id: active_cycle.cycle_id.clone(),
        now_iso,
    };

    let outcome = close_cycle::execute(request, &repo, &board_gateway)?;
    let board_column_str = format!("{:?}", outcome.board_sync_outcome);

    let response = JsonCloseResponse {
        verdict: "PASS".to_string(),
        cycle_id: outcome.cycle_id,
        board_column: board_column_str,
        next_steps: Vec::new(),
    };

    println!(
        "{}",
        Presenter::render_close(&response, json_mode || args.json)
    );
    Ok(DomainOutcome::Success)
}
