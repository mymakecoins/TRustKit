// Integration tests for Wave Board SQLite WAL Snapshot Reconciliation (TASK-002 / RF-002 / RN-002).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use std::sync::Arc;
use tempfile::tempdir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use trustkit_core::entities::{
    CycleStatus, ExecutionWave, GateStage, MethodologyProfile, SddCycle, TaskNode, WaveSchedule,
};
use trustkit_core::ports::CycleEvidenceRepositoryPort;
use trustkit_core::special_cases::{BoardColumn, IssueLinkage, SpecLockState, TddLifecycleState};
use trustkit_db::sqlite_adapter::SqliteWalRepositoryAdapter;
use trustkit_orch::wave_board_server_sse::WaveBoardServerSse;
use trustkit_orch::wave_board_snapshot_builder::WaveBoardSnapshotBuilder;

fn seed_sample_sqlite_db(db_path: &std::path::Path) -> SqliteWalRepositoryAdapter {
    let repo = SqliteWalRepositoryAdapter::open(db_path).expect("Failed opening SQLite WAL db");

    let cycle = SddCycle {
        cycle_id: "issue-32-202611180939".to_string(),
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
        gauntlet_attempt: 1,
        spec_dir: "docs/specs/issue-32-202611180939".to_string(),
        spec_file_path: "docs/specs/issue-32-202611180939/spec.md".to_string(),
        spec_seal_state: SpecLockState::Frozen {
            blake3_digest: "7eda7cb07795ba6edb0ccc43c771bec85fc18a86b17d12912ded4b773444bfa7"
                .to_string(),
            spec_path: "docs/specs/issue-32-202611180939/spec.md".to_string(),
        },
        work_branch: "mymakecoins/32".to_string(),
        base_commit_sha: "ab9617befc208776513eaf5a11a16d3d7c156cf7".to_string(),
        pre_existing_dirty_files: vec![],
        created_at: "2026-11-18T09:39:00Z".to_string(),
        updated_at: "2026-11-18T09:45:38Z".to_string(),
    };
    repo.save_cycle(&cycle).expect("Failed saving cycle");

    let t1 = TaskNode::new(
        "TASK-001".to_string(),
        "Publicador IPC de Mutacoes".to_string(),
        "ORCH".to_string(),
        vec![],
        vec!["crates/trustkit-orch/src/wave_board_event_publisher.rs".to_string()],
        "crates/trustkit-orch/tests/wave_board_ipc_event_ingestion_tests.rs".to_string(),
    );
    let t2 = TaskNode::new(
        "TASK-002".to_string(),
        "Reconciliador de Snapshot Inicial".to_string(),
        "ORCH".to_string(),
        vec!["TASK-001".to_string()],
        vec!["crates/trustkit-orch/src/wave_board_snapshot_builder.rs".to_string()],
        "crates/trustkit-orch/tests/wave_board_sqlite_snapshot_tests.rs".to_string(),
    );
    let schedule = WaveSchedule {
        cycle_id: cycle.cycle_id.clone(),
        total_tasks: 2,
        waves: vec![
            ExecutionWave {
                wave_index: 1,
                tasks: vec![t1.clone()],
            },
            ExecutionWave {
                wave_index: 2,
                tasks: vec![t2.clone()],
            },
        ],
    };
    repo.save_atomic_tasks(&cycle.cycle_id, &[t1, t2], &schedule)
        .expect("Failed saving atomic tasks");

    repo.update_task_tdd_phase(
        &cycle.cycle_id,
        "TASK-001",
        &TddLifecycleState::Committed {
            commit_sha: "ab9617befc208776513eaf5a11a16d3d7c156cf7".to_string(),
        },
    )
    .expect("Failed updating TASK-001");
    repo.update_task_tdd_phase(
        &cycle.cycle_id,
        "TASK-002",
        &TddLifecycleState::RedVerified {
            failing_evidence_id: 10,
        },
    )
    .expect("Failed updating TASK-002");

    repo
}

#[test]
fn test_task_002_build_snapshot_from_real_sqlite_wal() {
    let dir = tempdir().expect("Failed creating tempdir");
    let db_path = dir.path().join("state.db");
    let repo = seed_sample_sqlite_db(&db_path);

    let snapshot =
        WaveBoardSnapshotBuilder::build_from_repository(&repo).expect("Failed building snapshot");

    assert_eq!(snapshot.cycle_id, "issue-32-202611180939");
    assert_eq!(snapshot.metrics.active_cycles, 1);
    assert_eq!(snapshot.cards.len(), 2);

    let card1 = snapshot
        .cards
        .iter()
        .find(|c| c.task_id == "TASK-001")
        .expect("Expected TASK-001 card");
    assert_eq!(card1.tdd_phase, "COMMITTED");
    assert!(card1.blake3_seal.starts_with("7eda7cb0"));

    let card2 = snapshot
        .cards
        .iter()
        .find(|c| c.task_id == "TASK-002")
        .expect("Expected TASK-002 card");
    assert_eq!(card2.tdd_phase, "RED");
    assert_eq!(card2.column_index, 4);
    assert!(
        card2
            .next_step_command
            .contains("tk task test TASK-002 --phase green")
            || card2
                .next_step_command
                .contains("trustkit task test TASK-002 --phase green")
    );

    let json = serde_json::to_string(&snapshot).expect("Failed serializing snapshot");
    assert!(!json.contains("null"), "Snapshot JSON must be null-free");
}

#[tokio::test]
async fn test_task_002_http_get_api_snapshot_and_events_initial_snapshot_frame() {
    let dir = tempdir().expect("Failed creating tempdir");
    let db_path = dir.path().join("state.db");
    let repo = Arc::new(seed_sample_sqlite_db(&db_path));

    let server =
        Arc::new(WaveBoardServerSse::new("<html><body>Board</body></html>").with_repository(repo));

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed binding port");
    let addr = listener.local_addr().expect("Failed local_addr");

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

    // 1. Verify GET /api/snapshot returns HTTP 200 and full snapshot JSON
    let mut client1 = TcpStream::connect(addr).await.expect("Connect 1 failed");
    client1
        .write_all(b"GET /api/snapshot HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n")
        .await
        .expect("Write 1 failed");
    let mut buf1 = vec![0u8; 8192];
    let n1 = client1.read(&mut buf1).await.expect("Read 1 failed");
    let resp1 = String::from_utf8_lossy(&buf1[..n1]);
    assert!(resp1.contains("HTTP/1.1 200 OK"));
    assert!(resp1.contains("Content-Type: application/json"));
    assert!(resp1.contains("TASK-001"));
    assert!(resp1.contains("TASK-002"));
    assert!(!resp1.contains("null"));

    // 2. Verify GET /events emits initial snapshot event frame
    let mut client2 = TcpStream::connect(addr).await.expect("Connect 2 failed");
    client2
        .write_all(b"GET /events HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n")
        .await
        .expect("Write 2 failed");
    let mut buf2 = vec![0u8; 8192];
    let n2 = client2.read(&mut buf2).await.expect("Read 2 failed");
    let resp2 = String::from_utf8_lossy(&buf2[..n2]);
    assert!(resp2.contains("event: connected"));
    assert!(resp2.contains("event: snapshot"));
    assert!(resp2.contains("TASK-002"));
}
