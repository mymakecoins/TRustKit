// Unit tests for special cases.
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

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
fn test_worktree_allocation_lifecycle_transitions() {
    let initial = WorktreeAllocation::NotProvisioned;
    assert!(initial.is_not_provisioned());
    assert!(!initial.is_provisioned());
    assert!(!initial.is_merged_and_pruned());

    let provisioned = initial
        .provision("/path/to/worktree", "task/TASK-001")
        .expect("Provisioning should succeed from NotProvisioned");
    assert!(provisioned.is_provisioned());
    assert_eq!(
        provisioned,
        WorktreeAllocation::Provisioned {
            worktree_path: "/path/to/worktree".to_string(),
            branch_name: "task/TASK-001".to_string(),
        }
    );

    let merged = provisioned
        .merge_and_prune("abcdef1234567890")
        .expect("Merging should succeed from Provisioned");
    assert!(merged.is_merged_and_pruned());
    assert_eq!(
        merged,
        WorktreeAllocation::MergedAndPruned {
            commit_sha: "abcdef1234567890".to_string(),
        }
    );
}

#[test]
fn test_worktree_allocation_invalid_transitions() {
    let not_prov = WorktreeAllocation::NotProvisioned;
    assert!(not_prov.clone().merge_and_prune("sha").is_err());

    let prov = WorktreeAllocation::Provisioned {
        worktree_path: "/wt".to_string(),
        branch_name: "branch".to_string(),
    };
    assert!(prov.clone().provision("/wt2", "branch2").is_err());

    let merged = WorktreeAllocation::MergedAndPruned {
        commit_sha: "sha".to_string(),
    };
    assert!(merged.clone().provision("/wt", "branch").is_err());
    assert!(merged.clone().merge_and_prune("sha2").is_err());
}

#[test]
fn test_tdd_lifecycle_state_transitions() {
    let awaiting = TddLifecycleState::AwaitingRedPhase;
    assert!(awaiting.is_awaiting_red());

    // Cannot advance to Green without Red
    let green_err = awaiting.clone().transition_to_green(2);
    assert!(green_err.is_err());

    // Advance to Red
    let red = awaiting
        .transition_to_red(1)
        .expect("Should transition to RedVerified");
    assert!(red.is_red_verified());

    // Advance to Green
    let green = red
        .transition_to_green(2)
        .expect("Should transition to GreenVerified");
    assert!(green.is_green_verified());

    // Advance to Committed
    let committed = green
        .transition_to_committed("commit123")
        .expect("Should transition to Committed");
    assert!(committed.is_committed());
}

#[test]
fn test_circuit_breaker_state_evaluations() {
    assert_eq!(
        CircuitBreakerState::evaluate(0, ""),
        CircuitBreakerState::Closed
    );
    assert_eq!(
        CircuitBreakerState::evaluate(4, "Almost tripped"),
        CircuitBreakerState::Closed
    );

    let tripped = CircuitBreakerState::evaluate(5, "Max iterations reached (RN-005)");
    assert!(tripped.is_tripped());
    assert!(!tripped.is_closed());

    if let CircuitBreakerState::Tripped {
        consecutive_failures,
        diagnostic,
    } = tripped
    {
        assert_eq!(consecutive_failures, 5);
        assert_eq!(diagnostic, "Max iterations reached (RN-005)");
    }
}
