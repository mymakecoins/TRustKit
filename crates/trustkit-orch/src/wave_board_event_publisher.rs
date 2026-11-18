// IPC Event Publisher for Wave Board real-time synchronization (TASK-001).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::wave_board_dashboard_sse_client::KanbanCardMutation;
use crate::wave_board_port_allocator::WaveBoardMetadata;
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::path::Path;
use std::time::Duration;

pub struct WaveBoardEventPublisher;

impl WaveBoardEventPublisher {
    pub fn publish_to_port(port: u16, mutation: &KanbanCardMutation) -> Result<bool, String> {
        if port == 0 {
            return Ok(false);
        }
        let body = serde_json::to_string(mutation)
            .map_err(|e| format!("Failed serializing KanbanCardMutation: {e}"))?;
        Self::publish_raw_json_to_port(port, &body)
    }

    pub fn publish_raw_json_to_port(port: u16, body_json: &str) -> Result<bool, String> {
        if port == 0 {
            return Ok(false);
        }
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
        let timeout = Duration::from_millis(250);

        let mut stream = match TcpStream::connect_timeout(&addr, timeout) {
            Ok(s) => s,
            Err(_) => return Ok(false),
        };
        let _ = stream.set_read_timeout(Some(timeout));
        let _ = stream.set_write_timeout(Some(timeout));

        let request = format!(
            "POST /api/events HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body_json.len(),
            body_json
        );

        if stream.write_all(request.as_bytes()).is_err() {
            return Ok(false);
        }
        let _ = stream.flush();

        let mut resp_buf = [0u8; 512];
        let n = match stream.read(&mut resp_buf) {
            Ok(bytes) => bytes,
            Err(_) => return Ok(false),
        };
        let resp_str = String::from_utf8_lossy(&resp_buf[..n]);
        Ok(resp_str.contains("200 OK"))
    }

    pub fn notify_if_active<P: AsRef<Path>>(
        metadata_path: P,
        mutation: &KanbanCardMutation,
    ) -> Result<bool, String> {
        let p = metadata_path.as_ref();
        if !p.exists() {
            return Ok(false);
        }
        let meta = match WaveBoardMetadata::load_from_file(p) {
            Ok(m) => m,
            Err(_) => return Ok(false),
        };
        Self::publish_to_port(meta.port, mutation)
    }

    pub fn notify_workspace_mutation(
        workspace_root: &Path,
        task_id: &str,
        previous_column: &str,
        new_column: &str,
        tdd_phase: &str,
        timestamp_iso: &str,
    ) -> bool {
        let meta_path = workspace_root.join(".trustkit").join("wave-board.json");
        let mutation = KanbanCardMutation {
            task_id: task_id.to_string(),
            previous_column: previous_column.to_string(),
            new_column: new_column.to_string(),
            tdd_phase: tdd_phase.to_string(),
            timestamp_iso: timestamp_iso.to_string(),
        };
        Self::notify_if_active(meta_path, &mutation).unwrap_or(false)
    }
}
