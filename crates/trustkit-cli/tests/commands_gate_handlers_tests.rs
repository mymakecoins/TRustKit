// Tests for Gate 4, 5, and 6 CLI execution handlers and POSIX exit code mappings (TASK-014).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use tempfile::tempdir;
use trustkit_cli::cli_contract::{
    DoctorArgs, DoneArgs, MergeArgs, TaskCompleteArgs, TaskTestArgs, TddPhaseArg,
};
use trustkit_cli::commands::{doctor_cmd, done_cmd, merge_cmd, task_cmd};
use trustkit_cli::error_handler::CliBoundaryErrorHandler;
use trustkit_core::entities::{
    CriticKind, CycleStatus, EvidenceKind, ExecutionEvidence, ExecutionWave, GateStage,
    GateVerdict, MethodologyProfile, SddCycle, TaskNode, WaveSchedule,
};
use trustkit_core::ports::CycleEvidenceRepositoryPort;
use trustkit_core::special_cases::{IssueLinkage, SpecLockState, TddLifecycleState};
use trustkit_db::sqlite_adapter::SqliteWalRepositoryAdapter;

fn setup_cycle_with_task(
    dir: &std::path::Path,
    task_id: &str,
    status: CycleStatus,
) -> (std::path::PathBuf, SqliteWalRepositoryAdapter) {
    let db_path = dir.join("state.db");
    let repo = SqliteWalRepositoryAdapter::open(&db_path).expect("Failed to open test db");

    let cycle = SddCycle {
        cycle_id: "cycle-gate-handlers".to_string(),
        issue_linkage: IssueLinkage::LinkedIssue {
            issue_number: 45,
            issue_url: "https://github.com/org/repo/issues/45".to_string(),
            project_board_id: "board-1".to_string(),
            project_item_id: "item-1".to_string(),
            board_column: trustkit_core::special_cases::BoardColumn::InProgress,
        },
        methodology_profile: MethodologyProfile::StandardGauntletV2,
        current_gate: GateStage::Gate4TasksImplemented,
        lifecycle_status: status,
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
        "CORE".to_string(),
        Vec::new(),
        vec!["crates/test.rs".to_string()],
        "tests/task_test.rs".to_string(),
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
fn test_task_test_green_without_prior_red_fails_with_exit_code_3() {
    let dir = tempdir().expect("tempdir");
    let (db_path, _) = setup_cycle_with_task(dir.path(), "TASK-001", CycleStatus::Active);

    let args = TaskTestArgs {
        task_id: "TASK-001".to_string(),
        phase: TddPhaseArg::Green,
        json: true,
    };

    let result = task_cmd::execute_test_with_db(&args, true, &db_path);
    assert!(result.is_err());

    let mut stderr_output = Vec::new();
    let exit_code =
        CliBoundaryErrorHandler::handle_result_with_writer(result, true, &mut stderr_output);

    // Premissa e Criterio de Aceite: Exit Code 3 para violacao do ciclo TDD
    assert_eq!(exit_code, 3);

    let output_str = String::from_utf8_lossy(&stderr_output);
    assert!(output_str.contains("TDD_LIFECYCLE_VIOLATION"));
    assert!(output_str.contains("MissingRedPhaseProof"));
}

#[test]
fn test_done_cmd_persists_gauntlet_report_and_returns_exit_code_0() {
    let dir = tempdir().expect("tempdir");
    let (db_path, repo) = setup_cycle_with_task(dir.path(), "TASK-001", CycleStatus::Active);

    // Seed Layer 0 AST Audit PASS evidence
    let ast_ev = ExecutionEvidence {
        evidence_id: 0,
        cycle_id: "cycle-gate-handlers".to_string(),
        gate_stage: GateStage::Gate5GauntletCleared,
        gauntlet_attempt: 0,
        actor_role: "AUDITOR:AST_LAYER0".to_string(),
        evidence_kind: EvidenceKind::AstAudit,
        executed_command: "trustkit gauntlet ast-audit".to_string(),
        exit_code: 0,
        duration_ms: 10,
        sanitized_stdout_tail: "PASS".to_string(),
        sanitized_stderr_tail: "".to_string(),
        output_blake3_digest: "abcd1234abcd1234abcd1234abcd1234abcd1234".to_string(),
        verdict: GateVerdict::Pass,
        recorded_at: "2026-10-28T12:00:00Z".to_string(),
    };
    repo.append_evidence(&ast_ev).expect("append ast evidence");

    // Seed 4 Critics PASS evidences
    let critics = [
        CriticKind::DomainAdversarial,
        CriticKind::ContractCompliance,
        CriticKind::StaticSecurity,
        CriticKind::PlaywrightVisual,
    ];
    for kind in critics {
        let ev = ExecutionEvidence {
            evidence_id: 0,
            cycle_id: "cycle-gate-handlers".to_string(),
            gate_stage: GateStage::Gate5GauntletCleared,
            gauntlet_attempt: 0,
            actor_role: format!("CRITIC:{}", kind.as_str()),
            evidence_kind: EvidenceKind::CriticVerdict,
            executed_command: format!("critic {}", kind.as_str()),
            exit_code: 0,
            duration_ms: 15,
            sanitized_stdout_tail: "PASS".to_string(),
            sanitized_stderr_tail: "".to_string(),
            output_blake3_digest: "feedbeef1234567890abcdef".to_string(),
            verdict: GateVerdict::Pass,
            recorded_at: "2026-10-28T12:01:00Z".to_string(),
        };
        repo.append_evidence(&ev).expect("append critic evidence");
    }

    let args = DoneArgs {
        json: false,
        skip_security_review: true,
        ..Default::default()
    };
    let result = done_cmd::execute_with_db(&args, false, &db_path);
    assert!(result.is_ok());

    let exit_code = CliBoundaryErrorHandler::handle_result(result, false);
    assert_eq!(exit_code, 0);

    // Criterio de aceite: gauntlet-report.md salvo no diretorio da especificacao
    let report_path = dir.path().join("gauntlet-report.md");
    assert!(report_path.exists());

    let content = std::fs::read_to_string(&report_path).expect("read report");
    assert!(content.contains("# Gauntlet Certification Report: cycle-gate-handlers"));
    assert!(content.contains("Gate 6 Certified"));
    assert!(content.contains("Domain/Adversarial Critic"));
    assert!(content.contains("Contract Compliance Critic"));
    assert!(content.contains("Static Security Critic"));
    assert!(content.contains("Playwright Visual Critic"));
}

#[test]
fn test_doctor_cmd_tripped_circuit_breaker_returns_exit_code_6() {
    let dir = tempdir().expect("tempdir");
    let (db_path, repo) =
        setup_cycle_with_task(dir.path(), "TASK-001", CycleStatus::CircuitBreakerTripped);

    // Update cycle with 5 failed gauntlet attempts
    repo.update_cycle_status("cycle-gate-handlers", CycleStatus::CircuitBreakerTripped, 5)
        .expect("update status");

    let args = DoctorArgs { json: true };
    let result = doctor_cmd::execute_with_db(&args, true, &db_path);
    assert!(result.is_err());

    let mut stderr_output = Vec::new();
    let exit_code =
        CliBoundaryErrorHandler::handle_result_with_writer(result, true, &mut stderr_output);

    // Premissa: Circuit Breaker acionado retorna Exit Code 6
    assert_eq!(exit_code, 6);

    let output_str = String::from_utf8_lossy(&stderr_output);
    assert!(output_str.contains("CIRCUIT_BREAKER_TRIPPED"));
}

#[test]
fn test_merge_cmd_rejects_uncertified_cycle_with_exit_code_1() {
    let dir = tempdir().expect("tempdir");
    let (db_path, _) = setup_cycle_with_task(dir.path(), "TASK-001", CycleStatus::Active);

    let args = MergeArgs {
        target: Some("develop".to_string()),
        json: true,
    };

    let result = merge_cmd::execute_with_db(&args, true, &db_path);
    assert!(result.is_err());

    let mut stderr_output = Vec::new();
    let exit_code =
        CliBoundaryErrorHandler::handle_result_with_writer(result, true, &mut stderr_output);

    // Uncertified cycle merge attempt results in InfrastructureBoundaryFault (Exit Code 1)
    assert_eq!(exit_code, 1);
    let output_str = String::from_utf8_lossy(&stderr_output);
    assert!(output_str.contains("cycle is not Certified"));
}

#[test]
fn test_task_complete_advances_to_committed() {
    let dir = tempdir().expect("tempdir");
    let (db_path, repo) = setup_cycle_with_task(dir.path(), "TASK-001", CycleStatus::Active);

    // First advance to red verified then green verified
    let red_proof = trustkit_core::entities::TddExecutionProof::new(
        trustkit_core::entities::TddPhase::Red,
        1,
        100,
        1,
    );
    let green_proof = trustkit_core::entities::TddExecutionProof::new(
        trustkit_core::entities::TddPhase::Green,
        0,
        100,
        2,
    );
    let mut task = repo
        .find_atomic_task("cycle-gate-handlers", "TASK-001")
        .expect("find")
        .expect("some");
    task.advance_red_phase(&red_proof).expect("advance red");
    task.advance_green_phase(&green_proof)
        .expect("advance green");
    repo.update_task_tdd_phase("cycle-gate-handlers", "TASK-001", &task.tdd_lifecycle_state)
        .expect("update");

    let args = TaskCompleteArgs {
        task_id: "TASK-001".to_string(),
        code_review: None,
        json: true,
    };

    let result = task_cmd::execute_complete_with_db(&args, true, &db_path);
    assert!(result.is_ok());

    let updated_task = repo
        .find_atomic_task("cycle-gate-handlers", "TASK-001")
        .expect("find")
        .expect("some");
    assert!(matches!(
        updated_task.tdd_lifecycle_state,
        TddLifecycleState::Committed { .. }
    ));
}

#[test]
fn test_gauntlet_run_without_layer0_clean_pass_fails_with_exit_code_5() {
    let dir = tempdir().expect("tempdir");
    let (db_path, _) = setup_cycle_with_task(dir.path(), "TASK-001", CycleStatus::Active);

    let args = trustkit_cli::cli_contract::GauntletRunArgs {
        json: true,
        ..Default::default()
    };
    let result = trustkit_cli::commands::gauntlet_run_cmd::execute_with_db(&args, true, &db_path);
    assert!(result.is_err());

    let mut stderr_output = Vec::new();
    let exit_code =
        CliBoundaryErrorHandler::handle_result_with_writer(result, true, &mut stderr_output);

    // Premissa: Falha do Gauntlet (Layer 0 AST ausente/reprovada) retorna Exit Code 5
    assert_eq!(exit_code, 5);

    let output_str = String::from_utf8_lossy(&stderr_output);
    assert!(output_str.contains("CRITIC_SWARM_OR_PLAYWRIGHT_FAILED"));
}
