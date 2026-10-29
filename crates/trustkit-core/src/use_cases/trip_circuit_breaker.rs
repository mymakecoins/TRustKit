// Trip Circuit Breaker Use Case (TASK-005).
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::errors::DomainViolation;
use crate::ports::CycleEvidenceRepositoryPort;
use crate::special_cases::{ActiveCycleLookup, CircuitBreakerState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TripCircuitBreakerRequest {
    pub cycle_id: String,
    pub diagnostic_reason: String,
    pub now_iso: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TripCircuitBreakerOutcome {
    pub cycle_id: String,
    pub gauntlet_attempt: u8,
    pub state: CircuitBreakerState,
    pub remediation_hint: String,
}

pub fn execute(
    req: TripCircuitBreakerRequest,
    repo: &dyn CycleEvidenceRepositoryPort,
) -> Result<TripCircuitBreakerOutcome, DomainViolation> {
    let mut cycle = match repo.find_active_cycle()? {
        ActiveCycleLookup::Found(c) => *c,
        ActiveCycleLookup::NoActiveCycle => {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "circuit_breaker".to_string(),
                details: format!("No active SDD cycle found for cycle_id '{}'", req.cycle_id),
            });
        }
    };

    let cb_state = cycle.record_gauntlet_failure(&req.diagnostic_reason);

    repo.update_cycle_status(&cycle.cycle_id, cycle.lifecycle_status, cycle.gauntlet_attempt)?;

    let remediation_hint = if cb_state.is_tripped() {
        "Circuit breaker tripped after 5 consecutive failures. Run 'trustkit doctor' to diagnose.".to_string()
    } else {
        format!("Failure recorded. Attempt {}/5.", cycle.gauntlet_attempt)
    };

    Ok(TripCircuitBreakerOutcome {
        cycle_id: cycle.cycle_id,
        gauntlet_attempt: cycle.gauntlet_attempt,
        state: cb_state,
        remediation_hint,
    })
}
