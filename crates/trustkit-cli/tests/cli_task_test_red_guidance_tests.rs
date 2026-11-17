// Tests for CLI task test RED phase diagnostic and remediation guidance (TASK-003).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use tempfile::tempdir;
use trustkit_cli::cli_contract::{TaskTestArgs, TddPhaseArg};
use trustkit_cli::commands::task_cmd;
use trustkit_core::entities::{
    CycleStatus, ExecutionWave, GateStage, MethodologyProfile, SddCycle, TaskNode, WaveSchedule,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::{
    AsyncExecutionSupervisorPort, CycleEvidenceRepositoryPort, SupervisedExecutionOutcome,
    SupervisedExecutionRequest,
};
use trustkit_core::special_cases::{IssueLinkage, SpecLockState};
use trustkit_db::sqlite_adapter::SqliteWalRepositoryAdapter;

struct PassingExecutionSupervisor;

impl AsyncExecutionSupervisorPort for PassingExecutionSupervisor {
    fn execute_supervised(
        &self,
        _request: &SupervisedExecutionRequest,
    ) -> Result<SupervisedExecutionOutcome, DomainViolation> {
        Ok(SupervisedExecutionOutcome {
            exit_code: 0,
            duration_ms: 5,
            stdout_tail:
                "running 1 test\ntest sample ... ok\ntest result: ok. 1 passed; 0 failed\n"
                    .to_string(),
            stderr_tail: String::new(),
            output_blake3_digest: "0".repeat(64),
        })
    }
}

fn setup_cycle_with_task(
    dir: &std::path::Path,
    task_id: &str,
    target_test_file: &str,
) -> (std::path::PathBuf, SqliteWalRepositoryAdapter) {
    let db_path = dir.join("state.db");
    let repo = SqliteWalRepositoryAdapter::open(&db_path).expect("Failed to open test db");

    let cycle = SddCycle {
        cycle_id: "cycle-red-guidance".to_string(),
        issue_linkage: IssueLinkage::LinkedIssue {
            issue_number: 45,
            issue_url: "https://github.com/org/repo/issues/45".to_string(),
            project_board_id: "board-1".to_string(),
            project_item_id: "item-1".to_string(),
            board_column: trustkit_core::special_cases::BoardColumn::InProgress,
        },
        methodology_profile: MethodologyProfile::StandardGauntletV2,
        current_gate: GateStage::Gate4TasksImplemented,
        lifecycle_status: CycleStatus::Active,
        gauntlet_attempt: 0,
        spec_dir: dir.to_str().unwrap().to_string(),
        spec_file_path: dir.join("spec.md").to_str().unwrap().to_string(),
        spec_seal_state: SpecLockState::Frozen {
            spec_path: dir.join("spec.md").to_str().unwrap().to_string(),
            blake3_digest: "0".repeat(64),
        },
        work_branch: "feat/45-core".to_string(),
        base_commit_sha: "1234567890123456789012345678901234567890".to_string(),
        pre_existing_dirty_files: Vec::new(),
        created_at: "2026-10-28T00:00:00Z".to_string(),
        updated_at: "2026-10-28T00:00:00Z".to_string(),
    };

    repo.save_cycle(&cycle).expect("Failed to save cycle");

    let task = TaskNode::new(
        task_id.to_string(),
        "Test Task".to_string(),
        "CLI".to_string(),
        Vec::new(),
        vec!["crates/test.rs".to_string()],
        target_test_file.to_string(),
    );

    let schedule = WaveSchedule {
        cycle_id: cycle.cycle_id.clone(),
        total_tasks: 1,
        waves: vec![ExecutionWave {
            wave_index: 1,
            tasks: vec![task.clone()],
        }],
    };

    repo.save_atomic_tasks(&cycle.cycle_id, &[task], &schedule)
        .expect("Failed to save tasks");

    (db_path, repo)
}

#[test]
fn test_task_test_red_phase_passing_suite_emits_remediation_guidance() {
    let dir = tempdir().expect("tempdir");
    let target_test_file = "crates/trustkit-cli/tests/cli_task_test_red_guidance_tests.rs";
    let (db_path, _) = setup_cycle_with_task(dir.path(), "TASK-003", target_test_file);

    let args = TaskTestArgs {
        task_id: "TASK-003".to_string(),
        phase: TddPhaseArg::Red,
        json: true,
    };

    let supervisor = PassingExecutionSupervisor;
    let result = task_cmd::execute_test_with_deps(&args, true, &db_path, &supervisor);
    assert!(
        result.is_err(),
        "Expected error when suite passes in RED phase"
    );

    match result.err().unwrap() {
        DomainViolation::TddLifecycleViolation { task_id, reason } => {
            assert_eq!(task_id, "TASK-003");
            assert!(
                reason.contains("Author a failing test assertion")
                    && reason.contains(target_test_file),
                "Error reason must include remediation guidance with target file: {reason}"
            );
        }
        other => panic!("Expected TddLifecycleViolation, got: {other:?}"),
    }
}
