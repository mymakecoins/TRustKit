// Special Case algebraic types eliminating NULL and representing domain alternatives.
// Constitutional Principle II: Absolute Prohibition of NULL & Special Case Objects.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::entities::SddCycle;
use crate::errors::DomainViolation;
pub use crate::gauntlet_entities::CircuitBreakerState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardColumn {
    Backlog,
    Refinement,
    Ready,
    Todo,
    Planning,
    InProgress,
    InReview,
    AwaitStaging,
    InStaging,
    ReadyForTest,
    Testing,
    Done,
    Blocked,
}

impl BoardColumn {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Backlog => "Backlog",
            Self::Refinement => "Refinement",
            Self::Ready => "Ready",
            Self::Todo => "Todo",
            Self::Planning => "Planning",
            Self::InProgress => "In progress",
            Self::InReview => "In review",
            Self::AwaitStaging => "Await Staging",
            Self::InStaging => "In Staging",
            Self::ReadyForTest => "Ready for test",
            Self::Testing => "Testing",
            Self::Done => "Done",
            Self::Blocked => "Blocked",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name.trim().to_lowercase().as_str() {
            "backlog" => Some(Self::Backlog),
            "refinement" => Some(Self::Refinement),
            "ready" => Some(Self::Ready),
            "todo" => Some(Self::Todo),
            "planning" => Some(Self::Planning),
            "in progress" | "inprogress" => Some(Self::InProgress),
            "in review" | "inreview" => Some(Self::InReview),
            "await staging" | "awaitstaging" => Some(Self::AwaitStaging),
            "in staging" | "instaging" => Some(Self::InStaging),
            "ready for test" | "readyfortest" => Some(Self::ReadyForTest),
            "testing" => Some(Self::Testing),
            "done" => Some(Self::Done),
            "blocked" => Some(Self::Blocked),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssueLinkage {
    LinkedIssue {
        issue_number: u64,
        issue_url: String,
        project_board_id: String,
        project_item_id: String,
        board_column: BoardColumn,
    },
    UnlinkedLocalCycle {
        directory_slug: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecLockState {
    DraftUnverified,
    Frozen {
        spec_path: String,
        blake3_digest: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TddLifecycleState {
    AwaitingRedPhase,
    RedVerified {
        failing_evidence_id: u64,
    },
    GreenVerified {
        red_evidence_id: u64,
        green_evidence_id: u64,
    },
    Committed {
        commit_sha: String,
    },
}

impl TddLifecycleState {
    pub const fn is_awaiting_red(&self) -> bool {
        matches!(self, Self::AwaitingRedPhase)
    }

    pub const fn is_red_verified(&self) -> bool {
        matches!(self, Self::RedVerified { .. })
    }

    pub const fn is_green_verified(&self) -> bool {
        matches!(self, Self::GreenVerified { .. })
    }

    pub const fn is_committed(&self) -> bool {
        matches!(self, Self::Committed { .. })
    }

    pub fn transition_to_red(self, failing_evidence_id: u64) -> Result<Self, DomainViolation> {
        match self {
            Self::AwaitingRedPhase => Ok(Self::RedVerified {
                failing_evidence_id,
            }),
            Self::RedVerified { .. } | Self::GreenVerified { .. } | Self::Committed { .. } => {
                Err(DomainViolation::TddLifecycleViolation {
                    task_id: String::new(),
                    reason: "Red phase already verified for this task".to_string(),
                })
            }
        }
    }

    pub fn transition_to_green(self, green_evidence_id: u64) -> Result<Self, DomainViolation> {
        match self {
            Self::RedVerified { failing_evidence_id } => Ok(Self::GreenVerified {
                red_evidence_id: failing_evidence_id,
                green_evidence_id,
            }),
            Self::AwaitingRedPhase => Err(DomainViolation::TddLifecycleViolation {
                task_id: String::new(),
                reason: "MissingRedPhaseProof: cannot transition to GreenVerified without prior RedVerified proof (RN-002)".to_string(),
            }),
            Self::GreenVerified { .. } | Self::Committed { .. } => {
                Err(DomainViolation::TddLifecycleViolation {
                    task_id: String::new(),
                    reason: "Green phase already verified or committed for this task".to_string(),
                })
            }
        }
    }

    pub fn transition_to_committed(
        self,
        commit_sha: impl Into<String>,
    ) -> Result<Self, DomainViolation> {
        match self {
            Self::GreenVerified { .. } => Ok(Self::Committed {
                commit_sha: commit_sha.into(),
            }),
            _ => Err(DomainViolation::TddLifecycleViolation {
                task_id: String::new(),
                reason: "Cannot commit task before reaching GreenVerified state".to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoardSyncOutcome {
    Synchronized {
        column: BoardColumn,
    },
    SkippedUnlinkedCycle,
    DeferredOffline {
        queued_transition: BoardColumn,
        retry_after_ms: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractUnlockState {
    Sealed,
    UnlockedByHuman { justification: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraceVerificationState {
    PendingGate5,
    VerifiedPass,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorktreeAllocation {
    NotProvisioned,
    Provisioned {
        worktree_path: String,
        branch_name: String,
        base_branch: String,
    },
    MergedAndPruned {
        commit_sha: String,
    },
}

impl WorktreeAllocation {
    pub const fn is_not_provisioned(&self) -> bool {
        matches!(self, Self::NotProvisioned)
    }

    pub const fn is_provisioned(&self) -> bool {
        matches!(self, Self::Provisioned { .. })
    }

    pub const fn is_merged_and_pruned(&self) -> bool {
        matches!(self, Self::MergedAndPruned { .. })
    }

    pub fn worktree_path(&self) -> Option<&str> {
        match self {
            Self::Provisioned { worktree_path, .. } => Some(worktree_path.as_str()),
            _ => None,
        }
    }

    pub fn branch_name(&self) -> Option<&str> {
        match self {
            Self::Provisioned { branch_name, .. } => Some(branch_name.as_str()),
            _ => None,
        }
    }

    pub fn base_branch(&self) -> Option<&str> {
        match self {
            Self::Provisioned { base_branch, .. } => Some(base_branch.as_str()),
            _ => None,
        }
    }

    pub fn provision(
        self,
        worktree_path: impl Into<String>,
        branch_name: impl Into<String>,
        base_branch: impl Into<String>,
    ) -> Result<Self, DomainViolation> {
        match self {
            Self::NotProvisioned => Ok(Self::Provisioned {
                worktree_path: worktree_path.into(),
                branch_name: branch_name.into(),
                base_branch: base_branch.into(),
            }),
            Self::Provisioned { .. } => Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "worktree_allocation".to_string(),
                details: "Worktree already provisioned for this task".to_string(),
            }),
            Self::MergedAndPruned { .. } => Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "worktree_allocation".to_string(),
                details: "Worktree already merged and pruned; cannot re-provision".to_string(),
            }),
        }
    }

    pub fn merge_and_prune(self, commit_sha: impl Into<String>) -> Result<Self, DomainViolation> {
        match self {
            Self::Provisioned { .. } => Ok(Self::MergedAndPruned {
                commit_sha: commit_sha.into(),
            }),
            Self::NotProvisioned => Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "worktree_allocation".to_string(),
                details: "Cannot merge worktree that was never provisioned".to_string(),
            }),
            Self::MergedAndPruned { .. } => Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "worktree_allocation".to_string(),
                details: "Worktree has already been merged and pruned".to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActiveCycleLookup {
    Found(Box<SddCycle>),
    NoActiveCycle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentityVerification {
    Verified {
        operator_handle: String,
    },
    Rejected {
        detected_handle: String,
        expected_handle: String,
    },
    GhUnavailable {
        details: String,
    },
}

#[cfg(test)]
#[path = "special_cases_tests.rs"]
mod tests;
