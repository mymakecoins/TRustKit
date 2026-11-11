// Handler for `trustkit doctor` (TASK-014).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::cli_contract::DoctorArgs;
use crate::error_handler::DomainOutcome;
use crate::gate_presenter::{GatePresenter, JsonDoctorResponse};
use std::path::Path;
use trustkit_core::entities::{CycleStatus, GateVerdict};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::CycleEvidenceRepositoryPort;
use trustkit_db::sqlite_adapter::SqliteWalRepositoryAdapter;

pub fn execute(args: &DoctorArgs, json_mode: bool) -> Result<DomainOutcome, DomainViolation> {
    let db_path = Path::new(".trustkit").join("state.db");
    execute_with_db(args, json_mode, &db_path)
}

pub fn execute_with_db(
    _args: &DoctorArgs,
    json_mode: bool,
    db_path: &Path,
) -> Result<DomainOutcome, DomainViolation> {
    if !db_path.exists() {
        return Err(DomainViolation::InfrastructureBoundaryFault {
            subsystem: "doctor".to_string(),
            details: "No active SDD cycle found in state.db. Run 'trustkit init' first."
                .to_string(),
        });
    }

    let repo = SqliteWalRepositoryAdapter::open(db_path)?;
    let cycle = match repo.find_latest_cycle()? {
        Some(c) => c,
        None => {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "doctor".to_string(),
                details: "No active or recorded SDD cycle found. Run 'trustkit init' first."
                    .to_string(),
            });
        }
    };

    let is_tripped =
        cycle.lifecycle_status == CycleStatus::CircuitBreakerTripped || cycle.gauntlet_attempt >= 5;

    let evidences = repo.list_evidences_for_cycle(&cycle.cycle_id)?;
    let mut diagnostics = Vec::new();

    for ev in evidences.iter().rev() {
        if ev.verdict == GateVerdict::Fail {
            diagnostics.push(format!(
                "Failure in stage {:?} by actor {}: command `{}` exited with {}",
                ev.gate_stage, ev.actor_role, ev.executed_command, ev.exit_code
            ));
            if diagnostics.len() >= 5 {
                break;
            }
        }
    }

    if diagnostics.is_empty() {
        diagnostics.push("No failing evidence recorded in active cycle.".to_string());
    }

    let remediation = if is_tripped {
        "Circuit breaker tripped after 5 consecutive failures. Intervene manually to fix failing tests and reset status.".to_string()
    } else {
        "Cycle is healthy. Continue TDD cycle or run gauntlet.".to_string()
    };

    let verdict = if is_tripped { "BLOCKED" } else { "PASS" }.to_string();

    let response = JsonDoctorResponse {
        verdict,
        circuit_breaker_tripped: is_tripped,
        consecutive_failures: cycle.gauntlet_attempt,
        diagnostics,
        remediation: remediation.clone(),
        next_steps: Vec::new(),
    };

    println!("{}", GatePresenter::render_doctor(&response, json_mode));

    if is_tripped {
        Err(DomainViolation::CircuitBreakerTripped {
            cycle_id: cycle.cycle_id,
            consecutive_failures: cycle.gauntlet_attempt,
            remediation,
        })
    } else {
        Ok(DomainOutcome::Success)
    }
}
