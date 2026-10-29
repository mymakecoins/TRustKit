// Complete Task Use Case (TASK-003).
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::errors::DomainViolation;
use crate::ports::CycleEvidenceRepositoryPort;
use crate::special_cases::TddLifecycleState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompleteTaskRequest {
    pub cycle_id: String,
    pub task_id: String,
    pub commit_sha: String,
    pub now_iso: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompleteTaskOutcome {
    pub cycle_id: String,
    pub task_id: String,
    pub commit_sha: String,
    pub new_state: TddLifecycleState,
}

pub fn execute(
    req: CompleteTaskRequest,
    repo: &dyn CycleEvidenceRepositoryPort,
) -> Result<CompleteTaskOutcome, DomainViolation> {
    let mut task = repo
        .find_atomic_task(&req.cycle_id, &req.task_id)?
        .ok_or_else(|| DomainViolation::TddLifecycleViolation {
            task_id: req.task_id.clone(),
            reason: format!("Atomic task '{}' not found in cycle '{}'", req.task_id, req.cycle_id),
        })?;

    task.advance_committed(&req.commit_sha)?;
    repo.update_task_tdd_phase(&req.cycle_id, &req.task_id, &task.tdd_lifecycle_state)?;

    Ok(CompleteTaskOutcome {
        cycle_id: req.cycle_id,
        task_id: req.task_id,
        commit_sha: req.commit_sha,
        new_state: task.tdd_lifecycle_state,
    })
}
