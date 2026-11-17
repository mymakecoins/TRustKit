// Unit tests for NextStepStatusPolicy red authoring guidance (TASK-002).
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use super::*;
use crate::entities::{AtomicTask, CycleStatus, GateStage, MethodologyProfile, SddCycle};
use crate::next_step_guidance::NextStepPriority;
use crate::special_cases::{IssueLinkage, SpecLockState, TddLifecycleState, WorktreeAllocation};

fn sample_cycle(gate: GateStage, status: CycleStatus, lock_state: SpecLockState) -> SddCycle {
    SddCycle {
        cycle_id: "042-auth".to_string(),
        issue_linkage: IssueLinkage::UnlinkedLocalCycle {
            directory_slug: "042-auth".to_string(),
        },
        methodology_profile: MethodologyProfile::StandardGauntletV2,
        current_gate: gate,
        lifecycle_status: status,
        gauntlet_attempt: 0,
        spec_dir: "docs/specs/042-auth".to_string(),
        spec_file_path: "docs/specs/042-auth/spec.md".to_string(),
        spec_seal_state: lock_state,
        work_branch: "feat/042-auth".to_string(),
        base_commit_sha: "0".repeat(40),
        pre_existing_dirty_files: Vec::new(),
        created_at: "2026-10-28T00:00:00Z".to_string(),
        updated_at: "2026-10-28T00:00:00Z".to_string(),
    }
}

fn sample_task(task_id: &str, wave_index: u32, state: TddLifecycleState) -> AtomicTask {
    AtomicTask {
        cycle_id: "042-auth".to_string(),
        task_id: task_id.to_string(),
        title: format!("Implement {task_id}"),
        architectural_layer: "CORE".to_string(),
        wave_index,
        depends_on: Vec::new(),
        expected_files: vec!["src/lib.rs".to_string()],
        target_test_file: "crates/trustkit-core/src/next_step_status_policy_tests.rs".to_string(),
        tdd_lifecycle_state: state,
        red_evidence_id: 0,
        green_evidence_id: 0,
        worktree_allocation: WorktreeAllocation::NotProvisioned,
        commit_sha: String::new(),
    }
}

#[test]
fn test_for_cycle_status_awaiting_red_authoring_guidance() {
    let cycle = sample_cycle(
        GateStage::Gate4TasksImplemented,
        CycleStatus::Active,
        SpecLockState::Frozen {
            spec_path: "docs/specs/042-auth/spec.md".to_string(),
            blake3_digest: "a".repeat(64),
        },
    );
    let mut task = sample_task("TASK-002", 1, TddLifecycleState::AwaitingRedPhase);
    task.target_test_file = "crates/trustkit-core/src/next_step_status_policy_tests.rs".to_string();
    task.worktree_allocation = WorktreeAllocation::Provisioned {
        worktree_path: ".trustkit/worktrees/TASK-002".to_string(),
        branch_name: "worktree/TASK-002".to_string(),
    };

    let steps = NextStepResolver::for_cycle_status(&cycle, &[task], true, true);
    assert_eq!(steps[0].priority, NextStepPriority::Primary);
    assert_eq!(steps[0].target_gate, "GATE-4");
    assert_eq!(steps[0].command, "trustkit task test TASK-002 --phase red");
    assert!(
        steps[0]
            .description
            .contains("Author failing test assertion")
            && steps[0]
                .description
                .contains("before running task test --phase red"),
        "Primary recommendation must explicitly guide authoring failing test assertion in target_test_file before running task test --phase red: {}",
        steps[0].description
    );
}
