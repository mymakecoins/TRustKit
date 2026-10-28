// Special Case algebraic types eliminating NULL and representing domain alternatives.
// Constitutional Principle II: Absolute Prohibition of NULL & Special Case Objects.

use crate::entities::SddCycle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardColumn {
    Backlog,
    Todo,
    Planning,
    InProgress,
    InReview,
    ReadyForTest,
    Testing,
    Done,
    Blocked,
}

impl BoardColumn {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Backlog => "Backlog",
            Self::Todo => "Todo",
            Self::Planning => "Planning",
            Self::InProgress => "In progress",
            Self::InReview => "In review",
            Self::ReadyForTest => "Ready for test",
            Self::Testing => "Testing",
            Self::Done => "Done",
            Self::Blocked => "Blocked",
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
    Provisioned { path: String },
    MergedAndPruned { sha: String },
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
mod tests {
    use super::*;

    #[test]
    fn test_board_column_representation() {
        assert_eq!(BoardColumn::InProgress.as_str(), "In progress");
        assert_eq!(BoardColumn::Done.as_str(), "Done");
    }

    #[test]
    fn test_issue_linkage_variants() {
        let linked = IssueLinkage::LinkedIssue {
            issue_number: 42,
            issue_url: "https://github.com/org/repo/issues/42".to_string(),
            project_board_id: "board_1".to_string(),
            project_item_id: "item_1".to_string(),
            board_column: BoardColumn::InProgress,
        };
        let unlinked = IssueLinkage::UnlinkedLocalCycle {
            directory_slug: "no-issue-202610271400".to_string(),
        };
        assert_ne!(linked, unlinked);
    }

    #[test]
    fn test_spec_lock_and_unlock_states() {
        let draft = SpecLockState::DraftUnverified;
        let frozen = SpecLockState::Frozen {
            spec_path: "docs/spec.md".to_string(),
            blake3_digest: "abcd1234efgh5678".to_string(),
        };
        assert_ne!(draft, frozen);

        let sealed = ContractUnlockState::Sealed;
        let unlocked = ContractUnlockState::UnlockedByHuman {
            justification: "Critical fix".to_string(),
        };
        assert_ne!(sealed, unlocked);
    }

    #[test]
    fn test_tdd_and_worktree_states() {
        let awaiting = TddLifecycleState::AwaitingRedPhase;
        let red = TddLifecycleState::RedVerified {
            failing_evidence_id: 1,
        };
        let green = TddLifecycleState::GreenVerified {
            red_evidence_id: 1,
            green_evidence_id: 2,
        };
        let committed = TddLifecycleState::Committed {
            commit_sha: "abcd".to_string(),
        };
        assert_ne!(awaiting, red);
        assert_ne!(red, green);
        assert_ne!(green, committed);

        let not_prov = WorktreeAllocation::NotProvisioned;
        let prov = WorktreeAllocation::Provisioned {
            path: "/tmp/wt".to_string(),
        };
        let merged = WorktreeAllocation::MergedAndPruned {
            sha: "1234".to_string(),
        };
        assert_ne!(not_prov, prov);
        assert_ne!(prov, merged);

        let trace_pending = TraceVerificationState::PendingGate5;
        let trace_pass = TraceVerificationState::VerifiedPass;
        assert_ne!(trace_pending, trace_pass);

        let sync_ok = BoardSyncOutcome::Synchronized {
            column: BoardColumn::Done,
        };
        let sync_skip = BoardSyncOutcome::SkippedUnlinkedCycle;
        let sync_defer = BoardSyncOutcome::DeferredOffline {
            queued_transition: BoardColumn::Testing,
            retry_after_ms: 1000,
        };
        assert_ne!(sync_ok, sync_skip);
        assert_ne!(sync_skip, sync_defer);
    }
}
