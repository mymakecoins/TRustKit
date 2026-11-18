// End-to-End Integration tests for CLI to Wave Board SSE Real-Time Synchronization (TASK-004 / RF-004 / RN-004).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;
use tokio::net::TcpListener;
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
use trustkit_core::special_cases::{BoardColumn, IssueLinkage, SpecLockState};
use trustkit_db::sqlite_adapter::SqliteWalRepositoryAdapter;
use trustkit_orch::wave_board_port_allocator::WaveBoardMetadata;
use trustkit_orch::wave_board_server_sse::WaveBoardServerSse;

struct MockTddSupervisor {
    exit_code: i32,
    stdout: String,
}

impl AsyncExecutionSupervisorPort for MockTddSupervisor {
    fn execute_supervised(
        &self,
        _request: &SupervisedExecutionRequest,
    ) -> Result<SupervisedExecutionOutcome, DomainViolation> {
        Ok(SupervisedExecutionOutcome {
            exit_code: self.exit_code,
            duration_ms: 12,
            stdout_tail: self.stdout.clone(),
            stderr_tail: String::new(),
            output_blake3_digest: "deadbeef".to_string(),
        })
    }
}

fn setup_cycle_in_db(db_path: &std::path::Path) -> SqliteWalRepositoryAdapter {
    let repo = SqliteWalRepositoryAdapter::open(db_path).expect("Open db");
    let cycle = SddCycle {
        cycle_id: "issue-32-e2e".to_string(),
        issue_linkage: IssueLinkage::LinkedIssue {
            issue_number: 32,
            issue_url: "https://github.com/org/repo/issues/32".to_string(),
            project_board_id: "PVT_1".to_string(),
            project_item_id: "PVTI_1".to_string(),
            board_column: BoardColumn::InProgress,
        },
        methodology_profile: MethodologyProfile::StandardGauntletV2,
        current_gate: GateStage::Gate4TasksImplemented,
        lifecycle_status: CycleStatus::Active,
        gauntlet_attempt: 0,
        spec_dir: ".".to_string(),
        spec_file_path: "spec.md".to_string(),
        spec_seal_state: SpecLockState::Frozen {
            blake3_digest: "7eda7cb07795ba6edb0ccc43c771bec85fc18a86b17d12912ded4b773444bfa7"
                .to_string(),
            spec_path: "spec.md".to_string(),
        },
        work_branch: "mymakecoins/32".to_string(),
        base_commit_sha: "ab9617befc208776513eaf5a11a16d3d7c156cf7".to_string(),
        pre_existing_dirty_files: vec![],
        created_at: "2026-11-18T09:39:00Z".to_string(),
        updated_at: "2026-11-18T09:45:38Z".to_string(),
    };
    repo.save_cycle(&cycle).expect("Save cycle");

    let t1 = TaskNode::new(
        "TASK-001".to_string(),
        "E2E Realtime Task".to_string(),
        "CLI".to_string(),
        vec![],
        vec!["crates/trustkit-cli/src/commands/task_cmd.rs".to_string()],
        "crates/trustkit-cli/tests/wave_board_realtime_cli_e2e_tests.rs".to_string(),
    );
    let schedule = WaveSchedule {
        cycle_id: cycle.cycle_id.clone(),
        total_tasks: 1,
        waves: vec![ExecutionWave {
            wave_index: 1,
            tasks: vec![t1.clone()],
        }],
    };
    repo.save_atomic_tasks(&cycle.cycle_id, &[t1], &schedule)
        .expect("Save tasks");
    repo
}

#[tokio::test]
async fn test_task_004_e2e_task_test_publishes_sse_mutations_to_daemon() {
    let dir = tempdir().expect("Tempdir");
    let tk_dir = dir.path().join(".trustkit");
    std::fs::create_dir_all(&tk_dir).expect("Create .trustkit");
    let db_path = tk_dir.join("state.db");
    let meta_path = tk_dir.join("wave-board.json");

    let repo = Arc::new(setup_cycle_in_db(&db_path));
    let server =
        Arc::new(WaveBoardServerSse::new("<html><body>Board</body></html>").with_repository(repo));
    let mut rx = server.subscribe();

    let listener = TcpListener::bind("127.0.0.1:0").await.expect("Bind");
    let port = listener.local_addr().expect("Addr").port();

    let meta = WaveBoardMetadata::new(9999, port, "2026-11-18T10:00:00Z".to_string());
    meta.save_to_file(&meta_path).expect("Save meta");

    let srv = Arc::clone(&server);
    tokio::spawn(async move {
        for _ in 0..2 {
            if let Ok((socket, _)) = listener.accept().await {
                let s = Arc::clone(&srv);
                tokio::spawn(async move {
                    let _ = s.handle_connection(socket).await;
                });
            }
        }
    });

    // 1. Execute RED phase via task_cmd
    let db_clone = db_path.clone();
    tokio::task::spawn_blocking(move || {
        let red_args = TaskTestArgs {
            task_id: "TASK-001".to_string(),
            phase: TddPhaseArg::Red,
            json: true,
        };
        let sup = MockTddSupervisor {
            exit_code: 101,
            stdout: "test result: FAILED. 0 passed; 1 failed;".to_string(),
        };
        task_cmd::execute_test_with_deps(&red_args, true, &db_clone, &sup)
            .expect("RED execution should succeed");
    })
    .await
    .expect("Join RED");

    let ev_red = tokio::time::timeout(Duration::from_millis(500), rx.recv())
        .await
        .expect("Timed out waiting for RED SSE event from task_cmd")
        .expect("Channel closed");
    assert_eq!(ev_red.event_type, "mutation");
    assert!(ev_red.payload_json.contains("TASK-001"));
    assert!(ev_red.payload_json.contains("RED"));

    // 2. Execute GREEN phase via task_cmd
    let db_clone2 = db_path.clone();
    tokio::task::spawn_blocking(move || {
        let green_args = TaskTestArgs {
            task_id: "TASK-001".to_string(),
            phase: TddPhaseArg::Green,
            json: true,
        };
        let sup = MockTddSupervisor {
            exit_code: 0,
            stdout: "test result: ok. 1 passed; 0 failed;".to_string(),
        };
        task_cmd::execute_test_with_deps(&green_args, true, &db_clone2, &sup)
            .expect("GREEN execution should succeed");
    })
    .await
    .expect("Join GREEN");

    let ev_green = tokio::time::timeout(Duration::from_millis(500), rx.recv())
        .await
        .expect("Timed out waiting for GREEN SSE event from task_cmd")
        .expect("Channel closed");
    assert_eq!(ev_green.event_type, "mutation");
    assert!(ev_green.payload_json.contains("TASK-001"));
    assert!(ev_green.payload_json.contains("GREEN"));
}

#[test]
fn test_task_004_e2e_graceful_decoupling_when_daemon_stopped() {
    let dir = tempdir().expect("Tempdir");
    let tk_dir = dir.path().join(".trustkit");
    std::fs::create_dir_all(&tk_dir).expect("Create .trustkit");
    let db_path = tk_dir.join("state.db");
    let _repo = setup_cycle_in_db(&db_path);

    // No wave-board.json exists -> must complete cleanly without error
    let red_args = TaskTestArgs {
        task_id: "TASK-001".to_string(),
        phase: TddPhaseArg::Red,
        json: true,
    };
    let sup = MockTddSupervisor {
        exit_code: 101,
        stdout: "test result: FAILED. 0 passed; 1 failed;".to_string(),
    };
    let res = task_cmd::execute_test_with_deps(&red_args, true, &db_path, &sup);
    assert!(
        res.is_ok(),
        "CLI command must succeed when daemon is not running"
    );
}
