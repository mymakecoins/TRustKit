// Integration tests for TDD lifecycle, worktrees and certification SQLite persistence (TASK-007).
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero-Panic & Resilient Database Handling.

use std::sync::Arc;
use std::thread;
use tempfile::tempdir;
use trustkit_core::entities::{
    CycleStatus, EvidenceKind, ExecutionEvidence, ExecutionWave, GateStage, GateVerdict,
    MethodologyProfile, SddCycle, TaskNode, WaveSchedule,
};
use trustkit_core::ports::CycleEvidenceRepositoryPort;
use trustkit_core::special_cases::{
    ActiveCycleLookup, IssueLinkage, SpecLockState, TddLifecycleState, WorktreeAllocation,
};
use trustkit_db::sqlite_adapter::SqliteWalRepositoryAdapter;

fn setup_test_cycle_and_task(repo: &SqliteWalRepositoryAdapter, cycle_id: &str, task_id: &str) {
    let cycle = SddCycle {
        cycle_id: cycle_id.to_string(),
        issue_linkage: IssueLinkage::UnlinkedLocalCycle {
            directory_slug: cycle_id.to_string(),
        },
        methodology_profile: MethodologyProfile::StandardGauntletV2,
        current_gate: GateStage::Gate3WavePlanned,
        lifecycle_status: CycleStatus::Active,
        gauntlet_attempt: 0,
        spec_dir: format!("specs/{cycle_id}"),
        spec_file_path: format!("specs/{cycle_id}/spec.md"),
        spec_seal_state: SpecLockState::Frozen {
            spec_path: format!("specs/{cycle_id}/spec.md"),
            blake3_digest: "0".repeat(64),
        },
        work_branch: "develop".to_string(),
        base_commit_sha: "0".repeat(40),
        pre_existing_dirty_files: vec![],
        created_at: "2026-10-28T10:00:00Z".to_string(),
        updated_at: "2026-10-28T10:00:00Z".to_string(),
    };
    repo.save_cycle(&cycle).expect("save cycle");

    let task = TaskNode::new(
        task_id.to_string(),
        "Test task implementation".to_string(),
        "CORE".to_string(),
        vec![],
        vec!["crates/trustkit-core/src/entities.rs".to_string()],
        "tests/task_tests.rs".to_string(),
    );
    let schedule = WaveSchedule {
        cycle_id: cycle_id.to_string(),
        total_tasks: 1,
        waves: vec![ExecutionWave {
            wave_index: 1,
            tasks: vec![task.clone()],
        }],
    };
    repo.save_atomic_tasks(cycle_id, &[task], &schedule)
        .expect("save atomic tasks");
}

#[test]
fn test_task_tdd_lifecycle_state_transitions_persistence() {
    let dir = tempdir().expect("temp dir");
    let db_path = dir.path().join("state.db");
    let repo = SqliteWalRepositoryAdapter::open(&db_path).expect("open repo");

    let cycle_id = "cycle-tdd-1";
    let task_id = "TASK-001";
    setup_test_cycle_and_task(&repo, cycle_id, task_id);

    // Initial state check
    let initial_task = repo
        .find_atomic_task(cycle_id, task_id)
        .expect("find task")
        .expect("task exists");
    assert_eq!(
        initial_task.tdd_lifecycle_state,
        TddLifecycleState::AwaitingRedPhase
    );
    assert_eq!(
        initial_task.worktree_allocation,
        WorktreeAllocation::NotProvisioned
    );

    // Transition to RedVerified with failing evidence id
    repo.update_task_tdd_phase(
        cycle_id,
        task_id,
        &TddLifecycleState::RedVerified {
            failing_evidence_id: 101,
        },
    )
    .expect("update to red");

    let red_task = repo
        .find_atomic_task(cycle_id, task_id)
        .expect("find task")
        .expect("task exists");
    assert_eq!(
        red_task.tdd_lifecycle_state,
        TddLifecycleState::RedVerified {
            failing_evidence_id: 101,
        }
    );

    // Transition to GreenVerified with both evidence ids
    repo.update_task_tdd_phase(
        cycle_id,
        task_id,
        &TddLifecycleState::GreenVerified {
            red_evidence_id: 101,
            green_evidence_id: 202,
        },
    )
    .expect("update to green");

    let green_task = repo
        .find_atomic_task(cycle_id, task_id)
        .expect("find task")
        .expect("task exists");
    assert_eq!(
        green_task.tdd_lifecycle_state,
        TddLifecycleState::GreenVerified {
            red_evidence_id: 101,
            green_evidence_id: 202,
        }
    );

    // Transition to Committed with commit sha
    repo.update_task_tdd_phase(
        cycle_id,
        task_id,
        &TddLifecycleState::Committed {
            commit_sha: "abcd1234efgh5678".to_string(),
        },
    )
    .expect("update to committed");

    let committed_task = repo
        .find_atomic_task(cycle_id, task_id)
        .expect("find task")
        .expect("task exists");
    assert_eq!(
        committed_task.tdd_lifecycle_state,
        TddLifecycleState::Committed {
            commit_sha: "abcd1234efgh5678".to_string(),
        }
    );
    assert_eq!(committed_task.commit_sha, "abcd1234efgh5678");
}

#[test]
fn test_task_worktree_allocation_persistence() {
    let dir = tempdir().expect("temp dir");
    let db_path = dir.path().join("state.db");
    let repo = SqliteWalRepositoryAdapter::open(&db_path).expect("open repo");

    let cycle_id = "cycle-wt-1";
    let task_id = "TASK-002";
    setup_test_cycle_and_task(&repo, cycle_id, task_id);

    // Provision worktree
    repo.update_task_worktree(
        cycle_id,
        task_id,
        &WorktreeAllocation::Provisioned {
            worktree_path: ".trustkit/worktrees/TASK-002".to_string(),
            branch_name: "task/TASK-002".to_string(),
        },
    )
    .expect("update worktree to provisioned");

    let prov_task = repo
        .find_atomic_task(cycle_id, task_id)
        .expect("find task")
        .expect("task exists");
    assert_eq!(
        prov_task.worktree_allocation,
        WorktreeAllocation::Provisioned {
            worktree_path: ".trustkit/worktrees/TASK-002".to_string(),
            branch_name: "task/TASK-002".to_string(),
        }
    );

    // Merge and prune worktree
    repo.update_task_worktree(
        cycle_id,
        task_id,
        &WorktreeAllocation::MergedAndPruned {
            commit_sha: "fedcba9876543210".to_string(),
        },
    )
    .expect("update worktree to merged");

    let merged_task = repo
        .find_atomic_task(cycle_id, task_id)
        .expect("find task")
        .expect("task exists");
    assert_eq!(
        merged_task.worktree_allocation,
        WorktreeAllocation::MergedAndPruned {
            commit_sha: "fedcba9876543210".to_string(),
        }
    );
}

#[test]
fn test_cycle_status_and_circuit_breaker_persistence() {
    let dir = tempdir().expect("temp dir");
    let db_path = dir.path().join("state.db");
    let repo = SqliteWalRepositoryAdapter::open(&db_path).expect("open repo");

    let cycle_id = "cycle-status-1";
    setup_test_cycle_and_task(&repo, cycle_id, "TASK-001");

    // Trip circuit breaker on 5th gauntlet failure
    repo.update_cycle_status(cycle_id, CycleStatus::CircuitBreakerTripped, 5)
        .expect("update to circuit breaker tripped");

    // Active cycle query should not find tripped cycle
    let active_lookup = repo.find_active_cycle().expect("find active cycle");
    assert_eq!(active_lookup, ActiveCycleLookup::NoActiveCycle);

    // Advance gate and certify
    repo.update_cycle_status(cycle_id, CycleStatus::Certified, 0)
        .expect("update to certified");
    repo.advance_cycle_gate(cycle_id, GateStage::Gate6Certified)
        .expect("advance gate to 6");

    // Mark as merged
    repo.update_cycle_status(cycle_id, CycleStatus::Merged, 0)
        .expect("update to merged");
}

#[test]
fn test_concurrent_evidence_writes_during_gauntlet_battery() {
    let dir = tempdir().expect("temp dir");
    let db_path = dir.path().join("state.db");
    let repo = Arc::new(SqliteWalRepositoryAdapter::open(&db_path).expect("open repo"));

    let cycle_id = "cycle-gauntlet-swarm";
    setup_test_cycle_and_task(&repo, cycle_id, "TASK-001");

    let mut handles = Vec::new();
    let actors = [
        "CRITIC:DOMAIN_ADVERSARIAL",
        "CRITIC:CONTRACT_COMPLIANCE",
        "CRITIC:STATIC_SECURITY",
        "CRITIC:PLAYWRIGHT_VISUAL",
        "BUILDER:WORKER_0",
    ];

    for (t_idx, actor) in actors.iter().enumerate() {
        let repo_clone = Arc::clone(&repo);
        let actor_role = actor.to_string();
        handles.push(thread::spawn(move || {
            for i in 0..10 {
                let ev = ExecutionEvidence {
                    evidence_id: 0,
                    cycle_id: cycle_id.to_string(),
                    gate_stage: GateStage::Gate5GauntletCleared,
                    gauntlet_attempt: 1,
                    actor_role: actor_role.clone(),
                    evidence_kind: EvidenceKind::CriticVerdict,
                    executed_command: format!("gauntlet verify --critic {} --step {}", t_idx, i),
                    exit_code: 0,
                    duration_ms: 15,
                    sanitized_stdout_tail: format!("Pass finding for step {}", i),
                    sanitized_stderr_tail: String::new(),
                    output_blake3_digest: format!("{:064x}", t_idx * 100 + i),
                    verdict: GateVerdict::Pass,
                    recorded_at: format!("2026-10-28T12:{:02}:{:02}Z", t_idx, i),
                };
                repo_clone
                    .append_evidence(&ev)
                    .expect("concurrent evidence write must not fail");
            }
        }));
    }

    for h in handles {
        h.join().expect("thread completed without panics");
    }

    let evidences = repo
        .list_evidences_for_cycle(cycle_id)
        .expect("list evidences");
    assert_eq!(evidences.len(), 50);
}

#[test]
fn test_certification_evidences_ordered_chronologically() {
    let dir = tempdir().expect("temp dir");
    let db_path = dir.path().join("state.db");
    let repo = SqliteWalRepositoryAdapter::open(&db_path).expect("open repo");

    let cycle_id = "cycle-cert-ordering";
    setup_test_cycle_and_task(&repo, cycle_id, "TASK-001");

    let ev1 = ExecutionEvidence {
        evidence_id: 0,
        cycle_id: cycle_id.to_string(),
        gate_stage: GateStage::Gate4TasksImplemented,
        gauntlet_attempt: 0,
        actor_role: "BUILDER".to_string(),
        evidence_kind: EvidenceKind::CmdExec,
        executed_command: "cargo test --phase red".to_string(),
        exit_code: 1,
        duration_ms: 100,
        sanitized_stdout_tail: "failed as expected".to_string(),
        sanitized_stderr_tail: String::new(),
        output_blake3_digest: "1".repeat(64),
        verdict: GateVerdict::Fail,
        recorded_at: "2026-10-28T10:00:00Z".to_string(),
    };
    let ev2 = ExecutionEvidence {
        evidence_id: 0,
        cycle_id: cycle_id.to_string(),
        gate_stage: GateStage::Gate4TasksImplemented,
        gauntlet_attempt: 0,
        actor_role: "BUILDER".to_string(),
        evidence_kind: EvidenceKind::CmdExec,
        executed_command: "cargo test --phase green".to_string(),
        exit_code: 0,
        duration_ms: 120,
        sanitized_stdout_tail: "passed".to_string(),
        sanitized_stderr_tail: String::new(),
        output_blake3_digest: "2".repeat(64),
        verdict: GateVerdict::Pass,
        recorded_at: "2026-10-28T10:05:00Z".to_string(),
    };
    let ev3 = ExecutionEvidence {
        evidence_id: 0,
        cycle_id: cycle_id.to_string(),
        gate_stage: GateStage::Gate5GauntletCleared,
        gauntlet_attempt: 1,
        actor_role: "CRITIC:PLAYWRIGHT_VISUAL".to_string(),
        evidence_kind: EvidenceKind::CriticVerdict,
        executed_command: "npx playwright test".to_string(),
        exit_code: 0,
        duration_ms: 500,
        sanitized_stdout_tail: "visual snapshots match".to_string(),
        sanitized_stderr_tail: String::new(),
        output_blake3_digest: "3".repeat(64),
        verdict: GateVerdict::Pass,
        recorded_at: "2026-10-28T10:10:00Z".to_string(),
    };

    repo.append_evidence(&ev1).unwrap();
    repo.append_evidence(&ev2).unwrap();
    repo.append_evidence(&ev3).unwrap();

    let cert_evidences = repo
        .list_certification_evidences(cycle_id)
        .expect("list certification evidences");

    assert_eq!(cert_evidences.len(), 3);
    assert_eq!(cert_evidences[0].recorded_at, "2026-10-28T10:00:00Z");
    assert_eq!(cert_evidences[1].recorded_at, "2026-10-28T10:05:00Z");
    assert_eq!(cert_evidences[2].recorded_at, "2026-10-28T10:10:00Z");
}
