// Unit tests for domain entities.
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use super::*;
use crate::gauntlet_entities::{TddExecutionProof, TddPhase};
use crate::special_cases::{CircuitBreakerState, WorktreeAllocation};

#[test]
fn test_gate_stage_conversions() {
    assert_eq!(GateStage::from_u8(0), Some(GateStage::Gate0Discovery));
    assert_eq!(GateStage::from_u8(6), Some(GateStage::Gate6Certified));
    assert_eq!(GateStage::from_u8(7), None);
    assert_eq!(GateStage::Gate0Discovery.label(), "GATE_0_DISCOVERY");
}

#[test]
fn test_sdd_cycle_construction() {
    let cycle = SddCycle {
        cycle_id: "issue-45-202610271400".to_string(),
        issue_linkage: IssueLinkage::UnlinkedLocalCycle {
            directory_slug: "issue-45-202610271400".to_string(),
        },
        methodology_profile: MethodologyProfile::StandardGauntletV2,
        current_gate: GateStage::Gate0Discovery,
        lifecycle_status: CycleStatus::Active,
        gauntlet_attempt: 0,
        spec_dir: "specs/001".to_string(),
        spec_file_path: "specs/001/spec.md".to_string(),
        spec_seal_state: SpecLockState::DraftUnverified,
        work_branch: "feat/45-core".to_string(),
        base_commit_sha: "0123456789012345678901234567890123456789".to_string(),
        pre_existing_dirty_files: vec![],
        created_at: "2026-10-27T14:00:00Z".to_string(),
        updated_at: "2026-10-27T14:00:00Z".to_string(),
    };
    assert_eq!(cycle.gauntlet_attempt, 0);
}

#[test]
fn test_atomic_task_tdd_lifecycle_typed_safety() {
    let mut task = AtomicTask {
        cycle_id: "cycle-1".to_string(),
        task_id: "TASK-001".to_string(),
        title: "Test Task".to_string(),
        architectural_layer: "CORE".to_string(),
        wave_index: 1,
        depends_on: vec![],
        expected_files: vec![],
        target_test_file: "tests/test.rs".to_string(),
        tdd_lifecycle_state: TddLifecycleState::AwaitingRedPhase,
        red_evidence_id: 0,
        green_evidence_id: 0,
        worktree_allocation: WorktreeAllocation::NotProvisioned,
        commit_sha: "UNCOMMITTED".to_string(),
    };

    // Cannot advance to Green without Red
    let premature_green = TddExecutionProof::new(TddPhase::Green, 0, 100, 1);
    assert!(task.advance_green_phase(&premature_green).is_err());

    // Red phase requires non-zero exit code
    let invalid_red = TddExecutionProof::new(TddPhase::Red, 0, 100, 2);
    assert!(task.advance_red_phase(&invalid_red).is_err());

    // Valid Red phase advances state to RedVerified
    let valid_red = TddExecutionProof::new(TddPhase::Red, 1, 100, 3);
    assert!(task.advance_red_phase(&valid_red).is_ok());
    assert!(task.tdd_lifecycle_state.is_red_verified());
    assert_eq!(task.red_evidence_id, 3);

    // Green phase requires exit code 0
    let failing_green = TddExecutionProof::new(TddPhase::Green, 1, 100, 4);
    assert!(task.advance_green_phase(&failing_green).is_err());

    // Valid Green phase advances state to GreenVerified
    let valid_green = TddExecutionProof::new(TddPhase::Green, 0, 100, 5);
    assert!(task.advance_green_phase(&valid_green).is_ok());
    assert!(task.tdd_lifecycle_state.is_green_verified());
    assert_eq!(task.green_evidence_id, 5);

    // Commit advances to Committed
    assert!(task.advance_committed("sha-abc-123").is_ok());
    assert!(task.tdd_lifecycle_state.is_committed());
    assert_eq!(task.commit_sha, "sha-abc-123");
}

#[test]
fn test_atomic_task_worktree_allocation_lifecycle() {
    let mut task = AtomicTask {
        cycle_id: "cycle-1".to_string(),
        task_id: "TASK-001".to_string(),
        title: "Test Task".to_string(),
        architectural_layer: "CORE".to_string(),
        wave_index: 1,
        depends_on: vec![],
        expected_files: vec![],
        target_test_file: "tests/test.rs".to_string(),
        tdd_lifecycle_state: TddLifecycleState::AwaitingRedPhase,
        red_evidence_id: 0,
        green_evidence_id: 0,
        worktree_allocation: WorktreeAllocation::NotProvisioned,
        commit_sha: "UNCOMMITTED".to_string(),
    };

    assert!(task.worktree_allocation.is_not_provisioned());
    assert!(
        task.allocate_worktree(".trustkit/worktrees/wt-1", "branch-wt-1")
            .is_ok()
    );
    assert!(task.worktree_allocation.is_provisioned());

    assert!(task.merge_and_prune_worktree("merge-sha-789").is_ok());
    assert!(task.worktree_allocation.is_merged_and_pruned());
    assert_eq!(task.commit_sha, "merge-sha-789");
}

#[test]
fn test_sdd_cycle_circuit_breaker_transition_on_5_failures() {
    let mut cycle = SddCycle {
        cycle_id: "issue-45-202610271400".to_string(),
        issue_linkage: IssueLinkage::UnlinkedLocalCycle {
            directory_slug: "issue-45-202610271400".to_string(),
        },
        methodology_profile: MethodologyProfile::StandardGauntletV2,
        current_gate: GateStage::Gate4TasksImplemented,
        lifecycle_status: CycleStatus::Active,
        gauntlet_attempt: 0,
        spec_dir: "specs/001".to_string(),
        spec_file_path: "specs/001/spec.md".to_string(),
        spec_seal_state: SpecLockState::DraftUnverified,
        work_branch: "feat/45-core".to_string(),
        base_commit_sha: "0123456789012345678901234567890123456789".to_string(),
        pre_existing_dirty_files: vec![],
        created_at: "2026-10-27T14:00:00Z".to_string(),
        updated_at: "2026-10-27T14:00:00Z".to_string(),
    };

    // Attempts 1 to 4 should keep Circuit Breaker Closed and Cycle Active
    for i in 1..=4 {
        let cb = cycle.record_gauntlet_failure(format!("Gauntlet failure attempt {i}"));
        assert!(cb.is_closed());
        assert_eq!(cycle.gauntlet_attempt, i);
        assert_eq!(cycle.lifecycle_status, CycleStatus::Active);
    }

    // 5th attempt must trip the Circuit Breaker and transition CycleStatus to CircuitBreakerTripped
    let cb5 = cycle.record_gauntlet_failure("5th consecutive failure in gauntlet swarm");
    assert!(cb5.is_tripped());
    assert_eq!(cycle.gauntlet_attempt, 5);
    assert_eq!(cycle.lifecycle_status, CycleStatus::CircuitBreakerTripped);
    if let CircuitBreakerState::Tripped {
        consecutive_failures,
        diagnostic,
    } = cb5
    {
        assert_eq!(consecutive_failures, 5);
        assert!(diagnostic.contains("5th consecutive failure"));
    }
}
