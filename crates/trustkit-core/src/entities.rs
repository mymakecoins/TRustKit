// Plain Old Rust Structs (PORS) Domain Entities.
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle VI: Ubiquitous Language & Focused Modular Clean Code (< 250 LOC).

pub use crate::ast_entities::*;
pub use crate::gate_entities::*;
pub use crate::gauntlet_entities::*;
pub use crate::graph_entities::*;
pub use crate::knowledge_entities::*;
pub use crate::semantic_review_entities::*;
pub use crate::spec_entities::*;
pub use crate::spec_scaffold_entities::*;
pub use crate::special_cases::{CircuitBreakerState, SpecLockState, WorktreeAllocation};
use crate::errors::DomainViolation;
use crate::special_cases::{IssueLinkage, TddLifecycleState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SddCycle {
    pub cycle_id: String,
    pub issue_linkage: IssueLinkage,
    pub methodology_profile: MethodologyProfile,
    pub current_gate: GateStage,
    pub lifecycle_status: CycleStatus,
    pub gauntlet_attempt: u8,
    pub spec_dir: String,
    pub spec_file_path: String,
    pub spec_seal_state: SpecLockState,
    pub work_branch: String,
    pub base_commit_sha: String,
    pub pre_existing_dirty_files: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl SddCycle {
    pub fn record_gauntlet_failure(&mut self, diagnostic: impl Into<String>) -> CircuitBreakerState {
        self.gauntlet_attempt = self.gauntlet_attempt.saturating_add(1);
        let diag = diagnostic.into();
        let cb = CircuitBreakerState::evaluate(self.gauntlet_attempt, diag);
        if cb.is_tripped() {
            self.lifecycle_status = CycleStatus::CircuitBreakerTripped;
        }
        cb
    }

    pub fn reset_gauntlet_attempts_on_success(&mut self) {
        self.gauntlet_attempt = 0;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomicTask {
    pub cycle_id: String,
    pub task_id: String,
    pub title: String,
    pub architectural_layer: String,
    pub wave_index: u32,
    pub depends_on: Vec<String>,
    pub expected_files: Vec<String>,
    pub target_test_file: String,
    pub tdd_lifecycle_state: TddLifecycleState,
    pub red_evidence_id: u64,
    pub green_evidence_id: u64,
    pub worktree_allocation: WorktreeAllocation,
    pub commit_sha: String,
}

impl AtomicTask {
    pub fn advance_red_phase(&mut self, proof: &TddExecutionProof) -> Result<(), DomainViolation> {
        if proof.phase != TddPhase::Red {
            return Err(DomainViolation::TddLifecycleViolation {
                task_id: self.task_id.clone(),
                reason: "Phase proof must be Red for red phase record".to_string(),
            });
        }
        if proof.exit_code == 0 {
            return Err(DomainViolation::TddLifecycleViolation {
                task_id: self.task_id.clone(),
                reason: "Red phase requires a failing test execution (non-zero exit code)".to_string(),
            });
        }
        self.tdd_lifecycle_state = self
            .tdd_lifecycle_state
            .clone()
            .transition_to_red(proof.evidence_id)
            .map_err(|e| match e {
                DomainViolation::TddLifecycleViolation { reason, .. } => {
                    DomainViolation::TddLifecycleViolation {
                        task_id: self.task_id.clone(),
                        reason,
                    }
                }
                other => other,
            })?;
        self.red_evidence_id = proof.evidence_id;
        Ok(())
    }

    pub fn advance_green_phase(&mut self, proof: &TddExecutionProof) -> Result<(), DomainViolation> {
        if proof.phase != TddPhase::Green {
            return Err(DomainViolation::TddLifecycleViolation {
                task_id: self.task_id.clone(),
                reason: "Phase proof must be Green for green phase record".to_string(),
            });
        }
        if proof.exit_code != 0 {
            return Err(DomainViolation::TddLifecycleViolation {
                task_id: self.task_id.clone(),
                reason: "Green phase requires a successful test execution (exit code 0)".to_string(),
            });
        }
        self.tdd_lifecycle_state = self
            .tdd_lifecycle_state
            .clone()
            .transition_to_green(proof.evidence_id)
            .map_err(|e| match e {
                DomainViolation::TddLifecycleViolation { reason, .. } => {
                    DomainViolation::TddLifecycleViolation {
                        task_id: self.task_id.clone(),
                        reason,
                    }
                }
                other => other,
            })?;
        self.green_evidence_id = proof.evidence_id;
        Ok(())
    }

    pub fn advance_committed(&mut self, commit_sha: impl Into<String>) -> Result<(), DomainViolation> {
        let sha = commit_sha.into();
        self.tdd_lifecycle_state = self
            .tdd_lifecycle_state
            .clone()
            .transition_to_committed(&sha)
            .map_err(|e| match e {
                DomainViolation::TddLifecycleViolation { reason, .. } => {
                    DomainViolation::TddLifecycleViolation {
                        task_id: self.task_id.clone(),
                        reason,
                    }
                }
                other => other,
            })?;
        self.commit_sha = sha;
        Ok(())
    }

    pub fn allocate_worktree(
        &mut self,
        worktree_path: impl Into<String>,
        branch_name: impl Into<String>,
    ) -> Result<(), DomainViolation> {
        self.worktree_allocation = self
            .worktree_allocation
            .clone()
            .provision(worktree_path, branch_name)?;
        Ok(())
    }

    pub fn merge_and_prune_worktree(&mut self, commit_sha: impl Into<String>) -> Result<(), DomainViolation> {
        let sha = commit_sha.into();
        self.worktree_allocation = self
            .worktree_allocation
            .clone()
            .merge_and_prune(&sha)?;
        self.commit_sha = sha;
        Ok(())
    }
}

#[cfg(test)]
#[path = "entities_tests.rs"]
mod tests;
