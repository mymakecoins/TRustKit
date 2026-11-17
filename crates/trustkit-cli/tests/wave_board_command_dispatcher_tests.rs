// Integration tests for WaveBoard command dispatcher (TASK-003 / RF-003 / RN-003).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use tempfile::tempdir;
use trustkit_cli::cli_contract::{BoardDownArgs, BoardStatusArgs};
use trustkit_cli::wave_board_cli_commands::{WaveBoardAction, WaveBoardCliResponse};
use trustkit_cli::wave_board_command_dispatcher::{execute_down, execute_status};
use trustkit_orch::wave_board_port_allocator::WaveBoardMetadata;

#[test]
fn test_task_003_wave_board_dispatcher_status_stopped() {
    let args = BoardStatusArgs { json: true };
    let res = execute_status(&args, true);
    assert!(res.is_ok());
}

#[test]
fn test_task_003_wave_board_dispatcher_down_when_not_running() {
    let args = BoardDownArgs { json: false };
    let res = execute_down(&args, false);
    assert!(res.is_ok());
}

#[test]
fn test_task_003_wave_board_cli_response_formatting() {
    let resp = WaveBoardCliResponse::new(
        WaveBoardAction::Up,
        1234,
        8080,
        "http://127.0.0.1:8080",
        "Wave board daemon started successfully",
        vec!["Run tk board status".to_string()],
    );

    let json = resp.to_json_output();
    assert!(!json.contains("null"));
    assert!(json.contains("\"status\": \"PASS\""));
    assert!(json.contains("\"pid\": 1234"));
    assert!(json.contains("\"port\": 8080"));

    let tty = resp.to_tty_output();
    assert!(tty.contains("[PASS] Wave Board UP: Wave board daemon started successfully"));
    assert!(tty.contains("URL: http://127.0.0.1:8080"));
    assert!(tty.contains("PID: 1234 | Port: 8080"));
    assert!(tty.contains("NEXT STEP:\n  Run tk board status"));
    assert!(!tty.contains("null"));
}

#[test]
fn test_task_003_metadata_persistence_roundtrip() {
    let dir = tempdir().expect("Failed tempdir");
    let meta_file = dir.path().join("wave-board.json");

    let meta = WaveBoardMetadata::new(9999, 54321, "2026-11-17T15:00:00Z".to_string());
    meta.save_to_file(&meta_file).expect("Failed save");

    let loaded = WaveBoardMetadata::load_from_file(&meta_file).expect("Failed load");
    assert_eq!(loaded.pid, 9999);
    assert_eq!(loaded.port, 54321);
    assert_eq!(loaded.url, "http://127.0.0.1:54321");
    assert_eq!(loaded.started_at, "2026-11-17T15:00:00Z");
}
