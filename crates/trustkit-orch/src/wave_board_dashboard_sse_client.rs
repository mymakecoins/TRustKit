// Production implementation for TASK-005 and TASK-003 (Issue #32)
// Verified under strict TDD
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::wave_board_snapshot_builder::BoardSnapshot;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KanbanCardMutation {
    pub task_id: String,
    pub previous_column: String,
    pub new_column: String,
    pub tdd_phase: String,
    pub timestamp_iso: String,
}

#[derive(Debug, Default)]
pub struct WaveBoardDashboardClient {
    pub active_cards: Vec<KanbanCardMutation>,
}

impl WaveBoardDashboardClient {
    pub fn new() -> Self {
        Self {
            active_cards: Vec::new(),
        }
    }

    pub fn load_snapshot(&mut self, snapshot: &BoardSnapshot) {
        self.active_cards.clear();
        for card in &snapshot.cards {
            self.active_cards.push(KanbanCardMutation {
                task_id: card.task_id.clone(),
                previous_column: card.gate_label.clone(),
                new_column: card.gate_label.clone(),
                tdd_phase: card.tdd_phase.clone(),
                timestamp_iso: String::new(),
            });
        }
    }

    pub fn apply_mutation(&mut self, mutation: KanbanCardMutation) {
        if let Some(pos) = self
            .active_cards
            .iter()
            .position(|c| c.task_id == mutation.task_id)
        {
            self.active_cards[pos] = mutation;
        } else {
            self.active_cards.push(mutation);
        }
    }

    pub fn card_count(&self) -> usize {
        self.active_cards.len()
    }

    pub fn column_card_count(&self, column: &str) -> usize {
        self.active_cards
            .iter()
            .filter(|c| c.new_column == column)
            .count()
    }

    pub fn find_card(&self, task_id: &str) -> Option<&KanbanCardMutation> {
        self.active_cards.iter().find(|c| c.task_id == task_id)
    }

    pub fn verify_dashboard_html_reactive_contract(html: &str) -> bool {
        html.contains("EventSource('/events')")
            && html.contains("/api/snapshot")
            && html.contains("renderSnapshot")
            && html.contains("applyCardMutation")
            && html.contains("sseStatusBadge")
    }
}
