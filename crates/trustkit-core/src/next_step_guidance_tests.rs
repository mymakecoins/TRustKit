// Unit tests for Deterministic Contextual Next-Step Guidance Resolver.
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use super::*;
use crate::entities::{AtomicTask, CycleStatus, GateStage, MethodologyProfile, SddCycle, TddPhase};
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
        target_test_file: "tests/test.rs".to_string(),
        tdd_lifecycle_state: state,
        red_evidence_id: 0,
        green_evidence_id: 0,
        worktree_allocation: WorktreeAllocation::NotProvisioned,
        commit_sha: String::new(),
    }
}

#[test]
fn test_for_init_contextual_populated_vs_empty_spec() {
    let empty_steps = NextStepResolver::for_init("docs/specs/042-auth/spec.md", false);
    assert_eq!(empty_steps[0].priority, NextStepPriority::Primary);
    assert_eq!(empty_steps[0].target_gate, "GATE-1");
    assert!(empty_steps[0].command.contains("trustkit spec prompt"));

    let populated_steps = NextStepResolver::for_init("docs/specs/042-auth/spec.md", true);
    assert_eq!(populated_steps[0].priority, NextStepPriority::Primary);
    assert_eq!(
        populated_steps[0].command,
        "trustkit spec lint --file docs/specs/042-auth/spec.md"
    );
    assert_eq!(
        populated_steps[1].command,
        "trustkit spec lock --spec docs/specs/042-auth/spec.md"
    );
}

#[test]
fn test_for_spec_lint_clean_pass_vs_violations() {
    let pass_steps = NextStepResolver::for_spec_lint("specs/001/spec.md", true);
    assert_eq!(pass_steps[0].target_gate, "GATE-2");
    assert_eq!(
        pass_steps[0].command,
        "trustkit spec lock --spec specs/001/spec.md"
    );

    let fail_steps = NextStepResolver::for_spec_lint("specs/001/spec.md", false);
    assert_eq!(fail_steps[0].target_gate, "GATE-1");
    assert_eq!(
        fail_steps[0].command,
        "trustkit spec lint --file specs/001/spec.md"
    );
}

#[test]
fn test_for_spec_lock_contextual_tasks_presence() {
    let with_tasks = NextStepResolver::for_spec_lock("docs/specs/042-auth", true);
    assert_eq!(with_tasks[0].target_gate, "GATE-3");
    assert_eq!(with_tasks[0].command, "trustkit wave plan");

    let without_tasks = NextStepResolver::for_spec_lock("docs/specs/042-auth", false);
    assert_eq!(without_tasks[0].target_gate, "GATE-3");
    assert_eq!(
        without_tasks[0].command,
        "trustkit wave plan --tasks-dir docs/specs/042-auth/tasks"
    );
}

#[test]
fn test_for_task_test_and_complete_state_aware_handoff() {
    let red_next = NextStepResolver::for_task_test("TASK-001", TddPhase::Red);
    assert_eq!(
        red_next[0].command,
        "trustkit task test TASK-001 --phase green"
    );

    let green_next = NextStepResolver::for_task_test("TASK-001", TddPhase::Green);
    assert_eq!(green_next[0].command, "trustkit task complete TASK-001");

    // Next task in same wave
    let next_in_wave = NextStepResolver::for_task_complete(Some("TASK-002"), Some(2));
    assert_eq!(
        next_in_wave[0].command,
        "trustkit task test TASK-002 --phase red"
    );

    // Wave finished, next wave available
    let next_wave = NextStepResolver::for_task_complete(None, Some(2));
    assert_eq!(next_wave[0].command, "trustkit wave run --wave 2");

    // All waves finished -> Gate 5 AST audit
    let all_done = NextStepResolver::for_task_complete(None, None);
    assert_eq!(all_done[0].target_gate, "GATE-5");
    assert_eq!(all_done[0].command, "trustkit gauntlet ast-audit");
}

#[test]
fn test_for_cycle_status_full_lifecycle_transitions() {
    // 1. Circuit breaker tripped
    let mut tripped = sample_cycle(
        GateStage::Gate4TasksImplemented,
        CycleStatus::CircuitBreakerTripped,
        SpecLockState::DraftUnverified,
    );
    tripped.gauntlet_attempt = 5;
    let steps = NextStepResolver::for_cycle_status(&tripped, &[], false, false);
    assert_eq!(steps[0].command, "trustkit doctor");

    // 2. Gate 4 with a task in RedVerified -> recommends GREEN phase
    let frozen = SpecLockState::Frozen {
        spec_path: "docs/specs/042-auth/spec.md".to_string(),
        blake3_digest: "a".repeat(64),
    };
    let gate4_cycle = sample_cycle(
        GateStage::Gate4TasksImplemented,
        CycleStatus::Active,
        frozen.clone(),
    );
    let tasks = vec![
        sample_task(
            "TASK-001",
            1,
            TddLifecycleState::Committed {
                commit_sha: "abc".to_string(),
            },
        ),
        sample_task(
            "TASK-002",
            1,
            TddLifecycleState::RedVerified {
                failing_evidence_id: 3,
            },
        ),
    ];
    let steps = NextStepResolver::for_cycle_status(&gate4_cycle, &tasks, true, true);
    assert_eq!(
        steps[0].command,
        "trustkit task test TASK-002 --phase green"
    );

    // 3. Gate 4 with all tasks committed -> recommends Gate 5 AST audit
    let committed_tasks = vec![sample_task(
        "TASK-001",
        1,
        TddLifecycleState::Committed {
            commit_sha: "abc".to_string(),
        },
    )];
    let steps = NextStepResolver::for_cycle_status(&gate4_cycle, &committed_tasks, true, true);
    assert_eq!(steps[0].command, "trustkit gauntlet ast-audit");

    // 4. Gate 6 Certified -> recommends merge
    let certified_cycle = sample_cycle(GateStage::Gate6Certified, CycleStatus::Certified, frozen);
    let steps = NextStepResolver::for_cycle_status(&certified_cycle, &committed_tasks, true, true);
    assert_eq!(steps[0].command, "trustkit merge --target develop");
}

#[test]
fn test_for_wave_run_pending_tasks_guidance_explicit_red_authoring() {
    let failed: Vec<String> = Vec::new();
    let steps = NextStepResolver::for_wave_run(1, &failed, None, Some("TASK-001"));
    assert_eq!(steps[0].priority, NextStepPriority::Primary);
    assert_eq!(steps[0].target_gate, "GATE-4");
    assert_eq!(steps[0].command, "trustkit task test TASK-001 --phase red");
    assert!(
        steps[0].description.contains("target_test_file")
            && steps[0].description.contains("failing"),
        "Primary recommendation must explicitly guide authoring failing test assertion in target_test_file before running task test --phase red: {}",
        steps[0].description
    );
}
