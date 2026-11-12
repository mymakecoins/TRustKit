// Close Cycle Use Case (TASK-001).
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::entities::CycleStatus;
use crate::errors::DomainViolation;
use crate::ports::{CycleEvidenceRepositoryPort, ProjectBoardGatewayPort};
use crate::special_cases::{ActiveCycleLookup, BoardColumn, BoardSyncOutcome, IssueLinkage};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloseCycleRequest {
    pub cycle_id: String,
    pub now_iso: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloseCycleOutcome {
    pub cycle_id: String,
    pub cycle_status: CycleStatus,
    pub board_sync_outcome: BoardSyncOutcome,
}

pub fn execute(
    req: CloseCycleRequest,
    repo: &dyn CycleEvidenceRepositoryPort,
    board_gateway: &dyn ProjectBoardGatewayPort,
) -> Result<CloseCycleOutcome, DomainViolation> {
    let cycle = match repo.find_active_cycle()? {
        ActiveCycleLookup::Found(c) => *c,
        ActiveCycleLookup::NoActiveCycle => {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "close_cycle".to_string(),
                details: format!("No active SDD cycle found for cycle_id '{}'", req.cycle_id),
            });
        }
    };

    if cycle.lifecycle_status != CycleStatus::Certified {
        return Err(DomainViolation::InfrastructureBoundaryFault {
            subsystem: "close_gate".to_string(),
            details: format!(
                "Cannot close cycle '{}': cycle is not Certified (current status: {:?}). Gate 6 certification required (RN-002).",
                cycle.cycle_id, cycle.lifecycle_status
            ),
        });
    }

    repo.update_cycle_status(&cycle.cycle_id, CycleStatus::Merged, cycle.gauntlet_attempt)?;

    let board_sync_outcome = match cycle.issue_linkage {
        IssueLinkage::LinkedIssue { issue_number, .. } => {
            board_gateway.transition_issue_column(issue_number, BoardColumn::ReadyForTest)?
        }
        IssueLinkage::UnlinkedLocalCycle { .. } => BoardSyncOutcome::SkippedUnlinkedCycle,
    };

    Ok(CloseCycleOutcome {
        cycle_id: cycle.cycle_id,
        cycle_status: CycleStatus::Merged,
        board_sync_outcome,
    })
}

#[cfg(test)]
#[path = "close_cycle_tests.rs"]
mod close_cycle_tests;
