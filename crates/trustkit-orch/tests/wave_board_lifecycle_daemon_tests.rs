// Integration tests for WaveBoard daemon lifecycle management (TASK-004 / RF-004 / RN-004).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use tempfile::tempdir;
use trustkit_orch::wave_board_lifecycle_daemon::WaveBoardLifecycleManager;
use trustkit_orch::wave_board_port_allocator::WaveBoardMetadata;

#[test]
fn test_task_004_wave_board_lifecycle_daemon_full_cycle() {
    let dir = tempdir().expect("Failed to create tempdir");
    let meta_file = dir.path().join("wave-board.json");

    // 1. Initial state: not running
    let status_before = WaveBoardLifecycleManager::inspect_status(&meta_file);
    assert!(!status_before.is_running);
    assert_eq!(status_before.pid, None);
    assert_eq!(status_before.port, None);
    assert_eq!(status_before.url, None);

    // 2. Metadata recorded: active
    let meta = WaveBoardMetadata::new(7777, 8888, "2026-11-17T12:00:00Z".to_string());
    meta.save_to_file(&meta_file).expect("Failed to save metadata");

    let status_running = WaveBoardLifecycleManager::inspect_status(&meta_file);
    assert!(status_running.is_running);
    assert_eq!(status_running.pid, Some(7777));
    assert_eq!(status_running.port, Some(8888));
    assert_eq!(status_running.url, Some("http://127.0.0.1:8888".to_string()));

    // 3. Graceful shutdown: returns true and removes metadata file
    let shutdown_success = WaveBoardLifecycleManager::graceful_shutdown(&meta_file)
        .expect("Shutdown failed");
    assert!(shutdown_success);
    assert!(!meta_file.exists());

    // 4. Post-shutdown: not running
    let status_after = WaveBoardLifecycleManager::inspect_status(&meta_file);
    assert!(!status_after.is_running);
    assert_eq!(status_after.pid, None);

    // 5. Subsequent shutdown returns false gracefully
    let shutdown_repeat = WaveBoardLifecycleManager::graceful_shutdown(&meta_file)
        .expect("Repeat shutdown failed");
    assert!(!shutdown_repeat);
}
