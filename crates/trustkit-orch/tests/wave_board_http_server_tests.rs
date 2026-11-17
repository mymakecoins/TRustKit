// Integration tests for WaveBoard HTTP server with SSE and dashboard delivery (TASK-002 / RF-002 / RN-002).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use trustkit_orch::wave_board_http_server::{SseEvent, WaveBoardServerSse};

#[tokio::test]
async fn test_task_002_wave_board_http_root_serves_html() {
    let html =
        "<html><head><title>TRustKit Dashboard</title></head><body><h1>Kanban</h1></body></html>";
    let server = Arc::new(WaveBoardServerSse::new(html));

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind ephemeral listener");
    let addr = listener.local_addr().expect("Failed to get local addr");

    let srv = Arc::clone(&server);
    tokio::spawn(async move {
        let (socket, _) = listener.accept().await.expect("Accept failed");
        srv.handle_connection(socket).await.expect("Handle failed");
    });

    let mut client = TcpStream::connect(addr)
        .await
        .expect("Failed to connect client");

    let req = "GET / HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n";
    client
        .write_all(req.as_bytes())
        .await
        .expect("Write failed");

    let mut buf = vec![0u8; 4096];
    let n = client.read(&mut buf).await.expect("Read failed");
    let response = String::from_utf8_lossy(&buf[..n]);

    assert!(response.contains("HTTP/1.1 200 OK"));
    assert!(response.contains("Content-Type: text/html; charset=utf-8"));
    assert!(response.contains("TRustKit Dashboard"));
}

#[tokio::test]
async fn test_task_002_wave_board_http_events_stream_sse() {
    let html = "<html><body>Dashboard</body></html>";
    let server = Arc::new(WaveBoardServerSse::new(html));

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind ephemeral listener");
    let addr = listener.local_addr().expect("Failed to get local addr");

    let srv = Arc::clone(&server);
    tokio::spawn(async move {
        let (socket, _) = listener.accept().await.expect("Accept failed");
        let _ = srv.handle_connection(socket).await;
    });

    let mut client = TcpStream::connect(addr)
        .await
        .expect("Failed to connect client");

    let req = "GET /events HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n";
    client
        .write_all(req.as_bytes())
        .await
        .expect("Write failed");

    let mut buf = [0u8; 1024];
    let n = client.read(&mut buf).await.expect("Read failed");
    let initial_resp = String::from_utf8_lossy(&buf[..n]);

    assert!(initial_resp.contains("HTTP/1.1 200 OK"));
    assert!(initial_resp.contains("Content-Type: text/event-stream"));
    assert!(initial_resp.contains("event: connected"));

    // Broadcast a mutation event
    let mutation = SseEvent::new("cycle_mutation", "{\"gate\":4,\"verdict\":\"PASS\"}");
    let _ = server.broadcast(mutation);

    let n = client.read(&mut buf).await.expect("Read mutation failed");
    let mutation_resp = String::from_utf8_lossy(&buf[..n]);
    assert!(mutation_resp.contains("event: cycle_mutation"));
    assert!(mutation_resp.contains("{\"gate\":4,\"verdict\":\"PASS\"}"));
}

#[tokio::test]
async fn test_task_002_wave_board_http_not_found() {
    let html = "<html><body>Dashboard</body></html>";
    let server = Arc::new(WaveBoardServerSse::new(html));

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind ephemeral listener");
    let addr = listener.local_addr().expect("Failed to get local addr");

    let srv = Arc::clone(&server);
    tokio::spawn(async move {
        let (socket, _) = listener.accept().await.expect("Accept failed");
        let _ = srv.handle_connection(socket).await;
    });

    let mut client = TcpStream::connect(addr)
        .await
        .expect("Failed to connect client");

    let req = "GET /unknown-path HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n";
    client
        .write_all(req.as_bytes())
        .await
        .expect("Write failed");

    let mut buf = [0u8; 512];
    let n = client.read(&mut buf).await.expect("Read failed");
    let response = String::from_utf8_lossy(&buf[..n]);

    assert!(response.contains("HTTP/1.1 404 Not Found"));
}
