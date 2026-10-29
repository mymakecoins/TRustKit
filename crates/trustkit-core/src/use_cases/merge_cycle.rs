// Merge Cycle Use Case (TASK-006).
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::entities::CycleStatus;
use crate::errors::DomainViolation;
use crate::ports::{CycleEvidenceRepositoryPort, ProjectBoardGatewayPort};
use crate::special_cases::{ActiveCycleLookup, BoardColumn, BoardSyncOutcome, IssueLinkage};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeCycleRequest {
    pub cycle_id: String,
    pub target_branch: String,
    pub now_iso: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeCycleOutcome {
    pub cycle_id: String,
    pub target_branch: String,
    pub cycle_status: CycleStatus,
    pub board_sync_outcome: BoardSyncOutcome,
}

pub fn execute(
    req: MergeCycleRequest,
    repo: &dyn CycleEvidenceRepositoryPort,
    board_gateway: &dyn ProjectBoardGatewayPort,
) -> Result<MergeCycleOutcome, DomainViolation> {
    let cycle = match repo.find_active_cycle()? {
        ActiveCycleLookup::Found(c) => *c,
        ActiveCycleLookup::NoActiveCycle => {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "merge_cycle".to_string(),
                details: format!("No active SDD cycle found for cycle_id '{}'", req.cycle_id),
            });
        }
    };

    if cycle.lifecycle_status != CycleStatus::Certified {
        return Err(DomainViolation::InfrastructureBoundaryFault {
            subsystem: "merge_gate".to_string(),
            details: format!(
                "Cannot merge cycle '{}': cycle is not Certified (current status: {:?}). Gate 6 certification required (RN-011).",
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

    Ok(MergeCycleOutcome {
        cycle_id: cycle.cycle_id,
        target_branch: req.target_branch,
        cycle_status: CycleStatus::Merged,
        board_sync_outcome,
    })
}
