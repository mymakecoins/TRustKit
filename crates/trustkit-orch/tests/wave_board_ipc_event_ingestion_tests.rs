// Integration tests for Wave Board IPC Event Ingestion and SSE Broadcast (TASK-001 / RF-001 / RN-001).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use std::sync::Arc;
use tempfile::tempdir;
use tokio::net::TcpListener;
use trustkit_orch::wave_board_dashboard_sse_client::KanbanCardMutation;
use trustkit_orch::wave_board_event_publisher::WaveBoardEventPublisher;
use trustkit_orch::wave_board_port_allocator::WaveBoardMetadata;
use trustkit_orch::wave_board_server_sse::WaveBoardServerSse;

#[tokio::test]
async fn test_task_001_post_api_events_ingests_and_broadcasts_mutation() {
    let server = Arc::new(WaveBoardServerSse::new("<html><body>Board</body></html>"));
    let mut rx = server.subscribe();

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed binding ephemeral port");
    let port = listener.local_addr().expect("Failed local_addr").port();

    let srv = Arc::clone(&server);
    tokio::spawn(async move {
        if let Ok((socket, _)) = listener.accept().await {
            let _ = srv.handle_connection(socket).await;
        }
    });

    let mutation = KanbanCardMutation {
        task_id: "TASK-001".to_string(),
        previous_column: "GATE_3_WAVE_PLANNED".to_string(),
        new_column: "GATE_4_TASKS_IMPLEMENTED".to_string(),
        tdd_phase: "RED".to_string(),
        timestamp_iso: "2026-11-18T09:45:00Z".to_string(),
    };

    let m_clone = mutation.clone();
    let published = tokio::task::spawn_blocking(move || {
        WaveBoardEventPublisher::publish_to_port(port, &m_clone)
    })
    .await
    .expect("Join failed")
    .expect("Publish returned error");

    assert!(published, "Expected IPC event publication to succeed");

    let event = rx.recv().await.expect("Expected broadcasted SSE event");
    assert_eq!(event.event_type, "mutation");
    assert!(event.payload_json.contains("TASK-001"));
    assert!(event.payload_json.contains("GATE_4_TASKS_IMPLEMENTED"));
    assert!(event.payload_json.contains("RED"));
}

#[tokio::test]
async fn test_task_001_notify_if_active_with_metadata_file_and_graceful_decoupling() {
    let dir = tempdir().expect("Failed creating tempdir");
    let meta_path = dir.path().join("wave-board.json");

    let mutation = KanbanCardMutation {
        task_id: "TASK-001".to_string(),
        previous_column: "GATE_4_TASKS_IMPLEMENTED".to_string(),
        new_column: "GATE_4_TASKS_IMPLEMENTED".to_string(),
        tdd_phase: "GREEN".to_string(),
        timestamp_iso: "2026-11-18T09:46:00Z".to_string(),
    };

    // 1. When metadata file does not exist, returns Ok(false) gracefully
    let inactive_res =
        WaveBoardEventPublisher::notify_if_active(&meta_path, &mutation).expect("Should not fail");
    assert!(!inactive_res);

    // 2. When metadata file exists and server is listening, publishes and broadcasts
    let server = Arc::new(WaveBoardServerSse::new("<html><body>Board</body></html>"));
    let mut rx = server.subscribe();

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed binding ephemeral port");
    let port = listener.local_addr().expect("Failed local_addr").port();

    let meta = WaveBoardMetadata::new(12345, port, "2026-11-18T09:45:00Z".to_string());
    meta.save_to_file(&meta_path).expect("Failed saving meta");

    let srv = Arc::clone(&server);
    tokio::spawn(async move {
        if let Ok((socket, _)) = listener.accept().await {
            let _ = srv.handle_connection(socket).await;
        }
    });

    let meta_clone = meta_path.clone();
    let m_clone = mutation.clone();
    let active_res = tokio::task::spawn_blocking(move || {
        WaveBoardEventPublisher::notify_if_active(&meta_clone, &m_clone)
    })
    .await
    .expect("Join failed")
    .expect("Notify failed");

    assert!(active_res, "Expected active daemon to receive notification");

    let received = rx.recv().await.expect("Expected SSE frame");
    assert_eq!(received.event_type, "mutation");
    assert!(received.payload_json.contains("GREEN"));
}
