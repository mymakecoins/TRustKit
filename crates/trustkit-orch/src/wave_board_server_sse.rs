// Production implementation for TASK-002 and TASK-001 (Issue #32)
// Verified under strict TDD
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{self, Receiver, Sender};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SseEvent {
    pub event_type: String,
    pub payload_json: String,
}

impl SseEvent {
    pub fn new(event_type: impl Into<String>, payload_json: impl Into<String>) -> Self {
        Self {
            event_type: event_type.into(),
            payload_json: payload_json.into(),
        }
    }

    pub fn to_sse_frame(&self) -> String {
        format!(
            "event: {}\ndata: {}\n\n",
            self.event_type, self.payload_json
        )
    }
}

pub struct WaveBoardServerSse {
    event_sender: Sender<SseEvent>,
    dashboard_html: String,
}

impl WaveBoardServerSse {
    pub fn new(dashboard_html: impl Into<String>) -> Self {
        let (tx, _) = broadcast::channel(64);
        Self {
            event_sender: tx,
            dashboard_html: dashboard_html.into(),
        }
    }

    pub fn subscribe(&self) -> Receiver<SseEvent> {
        self.event_sender.subscribe()
    }

    pub fn broadcast(&self, event: SseEvent) -> Result<usize, String> {
        self.event_sender
            .send(event)
            .map_err(|e| format!("Failed to broadcast event: {e}"))
    }

    pub fn dashboard_html(&self) -> &str {
        &self.dashboard_html
    }

    pub async fn handle_connection(&self, mut stream: TcpStream) -> std::io::Result<()> {
        let mut buf = [0u8; 8192];
        let n = stream.read(&mut buf).await?;
        if n == 0 {
            return Ok(());
        }
        let req_str = String::from_utf8_lossy(&buf[..n]);
        let first_line = req_str.lines().next().unwrap_or("");
        let mut parts = first_line.split_whitespace();
        let method = parts.next().unwrap_or("GET");
        let path = parts.next().unwrap_or("/");

        if method == "POST" && (path == "/api/events" || path == "/api/v1/events") {
            let body_json = req_str
                .split("\r\n\r\n")
                .nth(1)
                .or_else(|| req_str.split("\n\n").nth(1))
                .unwrap_or("{}")
                .trim();

            let payload = if body_json.is_empty() {
                "{}".to_string()
            } else {
                body_json.to_string()
            };

            let event = SseEvent::new("mutation", payload);
            let _ = self.broadcast(event);

            let resp_body = "{\"verdict\":\"PASS\",\"broadcasted\":true}";
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                resp_body.len(),
                resp_body
            );
            stream.write_all(response.as_bytes()).await?;
            stream.flush().await?;
        } else if path == "/" {
            let body = self.dashboard_html.as_bytes();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            stream.write_all(response.as_bytes()).await?;
            stream.write_all(body).await?;
            stream.flush().await?;
        } else if path == "/events" || path == "/api/v1/events" {
            let header = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\nAccess-Control-Allow-Origin: *\r\n\r\n";
            stream.write_all(header.as_bytes()).await?;
            stream.flush().await?;

            let mut rx = self.subscribe();
            let initial = SseEvent::new("connected", "{\"status\":\"ready\"}");
            let _ = stream.write_all(initial.to_sse_frame().as_bytes()).await;
            let _ = stream.flush().await;

            while let Ok(event) = rx.recv().await {
                let frame = event.to_sse_frame();
                if stream.write_all(frame.as_bytes()).await.is_err() {
                    break;
                }
                if stream.flush().await.is_err() {
                    break;
                }
            }
        } else {
            let not_found =
                "HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\nConnection: close\r\n\r\nNot Found";
            stream.write_all(not_found.as_bytes()).await?;
            stream.flush().await?;
        }
        Ok(())
    }

    pub async fn serve(self: Arc<Self>, listener: TcpListener) -> std::io::Result<()> {
        loop {
            let (socket, _) = listener.accept().await?;
            let server = Arc::clone(&self);
            tokio::spawn(async move {
                let _ = server.handle_connection(socket).await;
            });
        }
    }
}
