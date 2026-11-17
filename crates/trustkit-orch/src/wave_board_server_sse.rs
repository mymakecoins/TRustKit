// Production implementation for TASK-002
// Verified under strict TDD
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

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
}
