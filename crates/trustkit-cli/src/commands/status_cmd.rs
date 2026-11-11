// Handler for `trustkit status`.
// Constitutional Principle IV: Resilient Error Handling & Zero-Panic Policy.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::cli_contract::StatusArgs;
use crate::error_handler::DomainOutcome;
use crate::presenter::Presenter;
use std::path::Path;
use trustkit_core::errors::DomainViolation;
use trustkit_core::special_cases::ActiveCycleLookup;
use trustkit_core::use_cases::query_cycle_status;
use trustkit_db::sqlite_adapter::SqliteWalRepositoryAdapter;

pub fn execute(_args: &StatusArgs, json_mode: bool) -> Result<DomainOutcome, DomainViolation> {
    let db_path = Path::new(".trustkit").join("state.db");
    let (lookup, tasks) = if db_path.exists() {
        let repo = SqliteWalRepositoryAdapter::open(&db_path)?;
        let found = query_cycle_status::execute(&repo)?;
        let cycle_tasks = match &found {
            ActiveCycleLookup::Found(cycle) => {
                repo.list_atomic_tasks(&cycle.cycle_id).unwrap_or_default()
            }
            ActiveCycleLookup::NoActiveCycle => Vec::new(),
        };
        (found, cycle_tasks)
    } else {
        (ActiveCycleLookup::NoActiveCycle, Vec::new())
    };

    let rendered = Presenter::render_status_with_tasks(&lookup, &tasks, json_mode);
    println!("{rendered}");

    Ok(DomainOutcome::Success)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_execute_status_no_active_cycle() {
        let dir = tempdir().expect("create temp dir");
        let original_dir = std::env::current_dir().expect("get current dir");
        let _ = std::env::set_current_dir(dir.path());

        let args = StatusArgs { json: true };
        let res = execute(&args, true);

        let _ = std::env::set_current_dir(original_dir);
        assert!(res.is_ok());
    }
}
