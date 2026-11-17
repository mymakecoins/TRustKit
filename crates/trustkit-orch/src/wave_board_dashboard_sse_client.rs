// Production implementation for TASK-005
// Verified under strict TDD
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

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

    pub fn find_card(&self, task_id: &str) -> Option<&KanbanCardMutation> {
        self.active_cards.iter().find(|c| c.task_id == task_id)
    }
}
