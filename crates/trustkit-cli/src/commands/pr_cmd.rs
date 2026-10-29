// Handler for `trustkit pr` (TASK-014).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::cli_contract::PrArgs;
use crate::error_handler::DomainOutcome;
use crate::gate_presenter::{GatePresenter, JsonPrResponse};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::CycleEvidenceRepositoryPort;
use trustkit_core::special_cases::{ActiveCycleLookup, IssueLinkage};
use trustkit_core::use_cases::open_pull_request::{self, OpenPullRequestRequest};
use trustkit_db::sqlite_adapter::SqliteWalRepositoryAdapter;
use trustkit_gh::ProjectBoardSynchronizer;

fn current_timestamp_iso() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{now}")
}

pub fn execute(args: &PrArgs, json_mode: bool) -> Result<DomainOutcome, DomainViolation> {
    let db_path = Path::new(".trustkit").join("state.db");
    execute_with_db(args, json_mode, &db_path)
}

pub fn execute_with_db(
    args: &PrArgs,
    json_mode: bool,
    db_path: &Path,
) -> Result<DomainOutcome, DomainViolation> {
    if !db_path.exists() {
        return Err(DomainViolation::InfrastructureBoundaryFault {
            subsystem: "pull_request".to_string(),
            details: "No active SDD cycle found in state.db. Run 'trustkit init' first.".to_string(),
        });
    }

    let repo = SqliteWalRepositoryAdapter::open(db_path)?;
    let active_cycle = match repo.find_active_cycle()? {
        ActiveCycleLookup::Found(cycle) => *cycle,
        ActiveCycleLookup::NoActiveCycle => {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "pull_request".to_string(),
                details: "No active SDD cycle found. Run 'trustkit init' first.".to_string(),
            });
        }
    };

    let board_gateway = ProjectBoardSynchronizer::new();
    let base_branch = args
        .target
        .clone()
        .unwrap_or_else(|| "develop".to_string());
    let now_iso = current_timestamp_iso();

    let (title, custom_body) = match &active_cycle.issue_linkage {
        IssueLinkage::LinkedIssue { issue_number, .. } => (
            format!("feat: #{} cycle implementation", issue_number),
            Some(format!(
                "Implementation for cycle {}.\n\nRef #{}",
                active_cycle.cycle_id, issue_number
            )),
        ),
        IssueLinkage::UnlinkedLocalCycle { .. } => (
            format!("feat: {} cycle implementation", active_cycle.cycle_id),
            Some(format!(
                "Implementation for local cycle {}.",
                active_cycle.cycle_id
            )),
        ),
    };

    let request = OpenPullRequestRequest {
        cycle_id: active_cycle.cycle_id.clone(),
        human_authorized: true,
        base_branch: base_branch.clone(),
        head_branch: active_cycle.work_branch.clone(),
        title,
        custom_body,
        now_iso,
    };

    let outcome = open_pull_request::execute(request, &repo, &board_gateway)?;

    let response = JsonPrResponse {
        verdict: "PASS".to_string(),
        cycle_id: active_cycle.cycle_id,
        pr_url: outcome.pr_url,
        pr_number: outcome.pr_number,
        target_branch: base_branch,
        board_column: format!("{:?}", outcome.target_column),
    };

    println!("{}", GatePresenter::render_pr(&response, json_mode));
    Ok(DomainOutcome::Success)
}
