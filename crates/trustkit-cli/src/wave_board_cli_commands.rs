// Production implementation for TASK-003
// Verified under strict TDD
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WaveBoardAction {
    Up,
    Down,
    Status,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WaveBoardCliResponse {
    pub status: String,
    pub action: String,
    pub pid: u32,
    pub port: u16,
    pub url: String,
    pub message: String,
    pub next_steps: Vec<String>,
}

impl WaveBoardCliResponse {
    pub fn new(
        action: WaveBoardAction,
        pid: u32,
        port: u16,
        url: impl Into<String>,
        message: impl Into<String>,
        next_steps: Vec<String>,
    ) -> Self {
        let action_str = match action {
            WaveBoardAction::Up => "up",
            WaveBoardAction::Down => "down",
            WaveBoardAction::Status => "status",
        };
        Self {
            status: "PASS".to_string(),
            action: action_str.to_string(),
            pid,
            port,
            url: url.into(),
            message: message.into(),
            next_steps,
        }
    }

    pub fn to_json_output(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn to_tty_output(&self) -> String {
        format!(
            "[PASS] Wave Board {}: {}\nURL: {}\nPID: {} | Port: {}\n------------------------------------------------------------\nNEXT STEP:\n  {}",
            self.action.to_uppercase(),
            self.message,
            self.url,
            self.pid,
            self.port,
            self.next_steps.join("\n  ")
        )
    }
}
