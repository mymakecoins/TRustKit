// Tests for PtySocketAdapter IPC communication (TASK-003).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use std::path::PathBuf;
use tempfile::tempdir;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use trustkit_core::entities::{AgentSemanticState, BlockReasonKind, MultiplexedTerminalRequest};
use trustkit_core::ports::TerminalMultiplexerPort;
use trustkit_orch::PtySocketAdapter;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_pty_socket_adapter_mock_lifecycle() {
    let dir = tempdir().unwrap();
    let socket_path = dir.path().join("pty_mock.sock");

    let listener = UnixListener::bind(&socket_path).unwrap();

    // Spawn mock server
    tokio::spawn(async move {
        loop {
            let (stream, _) = match listener.accept().await {
                Ok(conn) => conn,
                Err(_) => break,
            };

            tokio::spawn(async move {
                let (reader, mut writer) = stream.into_split();
                let mut lines = BufReader::new(reader).lines();

                while let Ok(Some(line)) = lines.next_line().await {
                    let req: serde_json::Value = serde_json::from_str(&line).unwrap();
                    let id = req["id"].clone();
                    let method = req["method"].as_str().unwrap_or("");

                    let response = match method {
                        "terminal.run" => {
                            serde_json::json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "result": {
                                    "id": "term-abc-42",
                                    "pid": 9999
                                }
                            })
                        }
                        "terminal.get" => {
                            serde_json::json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "result": {
                                    "id": "term-abc-42",
                                    "state": "blocked",
                                    "kind": "permission",
                                    "detail": "Need approval"
                                }
                            })
                        }
                        "terminal.send" => {
                            serde_json::json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "result": { "status": "ok" }
                            })
                        }
                        "terminal.read" => {
                            serde_json::json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "result": { "buffer": "Line 1\nLine 2\n" }
                            })
                        }
                        "terminal.kill" => {
                            serde_json::json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "result": { "status": "killed" }
                            })
                        }
                        _ => {
                            serde_json::json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "error": { "code": -32601, "message": "Method not found" }
                            })
                        }
                    };

                    let mut resp_str = serde_json::to_string(&response).unwrap();
                    resp_str.push('\n');
                    let _ = writer.write_all(resp_str.as_bytes()).await;
                    let _ = writer.flush().await;
                }
            });
        }
    });

    let adapter = PtySocketAdapter::new(socket_path);

    let req = MultiplexedTerminalRequest::new(
        "TASK-001",
        "/tmp/worktree",
        vec!["claude".to_string()],
        vec![],
        30_000,
    )
    .unwrap();

    let handle = adapter.spawn_terminal(&req).expect("Should spawn terminal");
    assert_eq!(handle.terminal_id, "term-abc-42");
    assert_eq!(handle.pid, 9999);

    let state = adapter
        .query_terminal_state(&handle)
        .expect("Should query state");
    assert_eq!(
        state,
        AgentSemanticState::Blocked {
            kind: BlockReasonKind::Permission,
            detail: "Need approval".to_string(),
        }
    );

    adapter
        .send_terminal_input(&handle, "proceed\n")
        .expect("Should send input");

    let buffer = adapter
        .read_terminal_buffer(&handle, 10)
        .expect("Should read buffer");
    assert!(buffer.contains("Line 1"));

    adapter
        .terminate_terminal(&handle)
        .expect("Should kill terminal");
}

#[test]
fn test_pty_socket_adapter_missing_socket_fails_cleanly() {
    let adapter = PtySocketAdapter::new(PathBuf::from("/tmp/non_existent_pty.sock"));
    let req = MultiplexedTerminalRequest::new(
        "TASK-001",
        "/tmp/worktree",
        vec!["claude".to_string()],
        vec![],
        10_000,
    )
    .unwrap();

    let res = adapter.spawn_terminal(&req);
    assert!(res.is_err());
}
