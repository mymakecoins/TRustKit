// Handler for `trustkit spec lock`.
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Resilient Error Handling & Zero-Panic Policy.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Cryptographic Immutability ("The Event Log Proves").

use crate::cli_contract::SpecLockArgs;
use crate::error_handler::DomainOutcome;
use crate::next_step_presenter::{JsonNextStep, NextStepPresenter};
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use trustkit_ast::SpecValidator;
use trustkit_core::entities::NextStepResolver;
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::CycleEvidenceRepositoryPort;
use trustkit_core::special_cases::ActiveCycleLookup;
use trustkit_core::use_cases::lock_specification::{
    self, LockSpecificationOutcome, LockSpecificationRequest,
};
use trustkit_db::blake3_adapter::Blake3ParallelSealerAdapter;
use trustkit_db::sqlite_adapter::SqliteWalRepositoryAdapter;

#[derive(Debug, Serialize)]
pub struct JsonSpecLockSuccessResponse {
    pub status: String,
    pub cycle_id: String,
    pub spec_file_path: String,
    pub spec_lock_state: String,
    pub blake3_digest: String,
    pub locked_at: String,
    pub rules_count: usize,
    pub requirements_count: usize,
    pub tasks_count: usize,
    pub e2e_tests_count: usize,
    pub next_steps: Vec<JsonNextStep>,
}

pub fn execute(args: &SpecLockArgs, json_mode: bool) -> Result<DomainOutcome, DomainViolation> {
    let db_path = Path::new(".trustkit").join("state.db");

    if args.spec.is_none() && !db_path.exists() {
        return Err(DomainViolation::InfrastructureBoundaryFault {
            subsystem: "spec_lock".to_string(),
            details: "No active SDD cycle found and no --spec flag provided.".to_string(),
        });
    }

    if let Some(parent) = db_path.parent().filter(|p| !p.exists()) {
        let _ = fs::create_dir_all(parent);
    }

    let repo = SqliteWalRepositoryAdapter::open(&db_path)?;
    let hasher = Blake3ParallelSealerAdapter::new();
    let validator = SpecValidator;

    let target_spec_path = match &args.spec {
        Some(path) if !path.trim().is_empty() => path.clone(),
        _ => match repo.find_active_cycle()? {
            ActiveCycleLookup::Found(cycle) => cycle.spec_file_path,
            ActiveCycleLookup::NoActiveCycle => {
                return Err(DomainViolation::InfrastructureBoundaryFault {
                    subsystem: "spec_lock".to_string(),
                    details: "No active SDD cycle found and no --spec flag provided.".to_string(),
                });
            }
        },
    };

    let spec_path = Path::new(&target_spec_path);
    let file_content = fs::read_to_string(spec_path).map_err(|err| {
        DomainViolation::InfrastructureBoundaryFault {
            subsystem: "filesystem".to_string(),
            details: format!(
                "Failed reading specification file '{}': {err}",
                spec_path.display()
            ),
        }
    })?;

    let now_iso = current_iso_timestamp();

    let req = LockSpecificationRequest {
        spec_path: Some(target_spec_path),
        file_content,
        now_iso,
    };

    let outcome = lock_specification::execute(req, &repo, &hasher, &validator)?;

    let rendered = render_output(&outcome, json_mode);
    println!("{rendered}");

    Ok(DomainOutcome::Success)
}

fn render_output(outcome: &LockSpecificationOutcome, json_mode: bool) -> String {
    let spec_dir = Path::new(&outcome.spec_file_path)
        .parent()
        .and_then(|p| p.to_str())
        .unwrap_or("");
    let has_task_files = Path::new(spec_dir)
        .join("tasks")
        .read_dir()
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .any(|e| {
            e.file_name()
                .to_str()
                .is_some_and(|n| n.starts_with("TASK-") && n.ends_with(".md"))
        });
    let steps = NextStepResolver::for_spec_lock(spec_dir, has_task_files);

    if json_mode {
        let payload = JsonSpecLockSuccessResponse {
            status: "SPEC_FROZEN".to_string(),
            cycle_id: outcome.cycle_id.clone(),
            spec_file_path: outcome.spec_file_path.clone(),
            spec_lock_state: outcome.spec_lock_state.clone(),
            blake3_digest: outcome.blake3_digest.clone(),
            locked_at: outcome.locked_at.clone(),
            rules_count: outcome.rules_count,
            requirements_count: outcome.requirements_count,
            tasks_count: outcome.tasks_count,
            e2e_tests_count: outcome.e2e_tests_count,
            next_steps: NextStepPresenter::to_json_vec(&steps),
        };

        serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
    } else {
        let mut out = format!(
            "[SPEC FROZEN]\nCycle ID: {}\nFile: {}\nBLAKE3: {}\nLocked At: {}\nRules: {} | Requirements: {} | Tasks: {} | Tests: {}",
            outcome.cycle_id,
            outcome.spec_file_path,
            outcome.blake3_digest,
            outcome.locked_at,
            outcome.rules_count,
            outcome.requirements_count,
            outcome.tasks_count,
            outcome.e2e_tests_count
        );
        out.push_str(&NextStepPresenter::render_ascii_block(&steps));
        out
    }
}

fn current_iso_timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let sec = now % 60;
    let min = (now / 60) % 60;
    let hour = (now / 3600) % 24;
    let days = now / 86400;

    let z = days as i64 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    format!("{y:04}-{m:02}-{d:02}T{hour:02}:{min:02}:{sec:02}Z")
}
