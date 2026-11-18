// Snapshot Reconciler from SQLite WAL for Wave Board (TASK-002).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use serde::{Deserialize, Serialize};
use trustkit_core::entities::{AtomicTask, SddCycle};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::CycleEvidenceRepositoryPort;
use trustkit_core::special_cases::{
    ActiveCycleLookup, SpecLockState, TddLifecycleState, WorktreeAllocation,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KanbanCardSnapshot {
    pub card_id: String,
    pub cycle_id: String,
    pub task_id: String,
    pub title: String,
    pub gate: u8,
    pub column_index: u8,
    pub gate_label: String,
    pub status_label: String,
    pub priority: String,
    pub tdd_phase: String,
    pub blake3_seal: String,
    pub worktree_branch: String,
    pub wave_label: String,
    pub gauntlet_attempt: u8,
    pub next_step_command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BoardMetricsSummary {
    pub active_cycles: usize,
    pub total_cards: usize,
    pub active_worktrees: usize,
    pub gauntlet_attempts: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BoardSnapshot {
    pub cycle_id: String,
    pub current_gate: u8,
    pub metrics: BoardMetricsSummary,
    pub cards: Vec<KanbanCardSnapshot>,
}

impl BoardSnapshot {
    pub fn empty() -> Self {
        Self {
            cycle_id: "NONE".to_string(),
            current_gate: 0,
            metrics: BoardMetricsSummary {
                active_cycles: 0,
                total_cards: 0,
                active_worktrees: 0,
                gauntlet_attempts: 0,
            },
            cards: Vec::new(),
        }
    }
}

pub struct WaveBoardSnapshotBuilder;

impl WaveBoardSnapshotBuilder {
    pub fn gate_labels(gate: u8) -> (&'static str, &'static str) {
        match gate {
            0 => ("GATE 0: DISCOVERY", "Em Descoberta / Triagem"),
            1 => ("GATE 1: SPEC DRAFTING", "Em Refinamento / Especificacao"),
            2 => ("GATE 2: SPEC SEALED", "Escopo Congelado / Aprovado"),
            3 => ("GATE 3: WAVE PLANNED", "Pronto para Execucao (Planejado)"),
            4 => ("GATE 4: TASKS IMPLEMENTED", "Em Desenvolvimento (TDD)"),
            5 => (
                "GATE 5: GAUNTLET CLEARED",
                "Em Auditoria / Validacao Estrita",
            ),
            _ => ("GATE 6: CERTIFIED", "Concluido / Certificado (Done)"),
        }
    }

    fn resolve_blake3_seal(cycle: &SddCycle) -> String {
        match &cycle.spec_seal_state {
            SpecLockState::Frozen { blake3_digest, .. } => blake3_digest.clone(),
            SpecLockState::DraftUnverified => "UNSEALED_DRAFT".to_string(),
        }
    }

    fn build_task_card(
        cycle: &SddCycle,
        task: &AtomicTask,
        total_waves: u32,
        blake3_seal: &str,
    ) -> KanbanCardSnapshot {
        let (tdd_phase, task_gate, next_step_command) = match &task.tdd_lifecycle_state {
            TddLifecycleState::AwaitingRedPhase => {
                let col = if matches!(
                    task.worktree_allocation,
                    WorktreeAllocation::Provisioned { .. }
                ) {
                    4
                } else {
                    cycle.current_gate.as_u8().clamp(3, 4)
                };
                (
                    "AWAITING_RED".to_string(),
                    col,
                    format!("tk task test {} --phase red --json", task.task_id),
                )
            }
            TddLifecycleState::RedVerified { .. } => (
                "RED".to_string(),
                4,
                format!("tk task test {} --phase green --json", task.task_id),
            ),
            TddLifecycleState::GreenVerified { .. } => (
                "GREEN".to_string(),
                4,
                format!("tk task complete {} --json", task.task_id),
            ),
            TddLifecycleState::Committed { .. } => {
                let col = if cycle.current_gate.as_u8() >= 5 {
                    cycle.current_gate.as_u8().min(6)
                } else {
                    4
                };
                let cmd = if col >= 6 {
                    "tk pr --target develop --json".to_string()
                } else {
                    "tk gauntlet run --json".to_string()
                };
                ("COMMITTED".to_string(), col, cmd)
            }
        };

        let (gate_label, status_label) = Self::gate_labels(task_gate);
        let worktree_branch = task
            .worktree_allocation
            .branch_name()
            .unwrap_or(&cycle.work_branch)
            .to_string();

        KanbanCardSnapshot {
            card_id: format!("{}-{}", cycle.cycle_id, task.task_id),
            cycle_id: cycle.cycle_id.clone(),
            task_id: task.task_id.clone(),
            title: task.title.clone(),
            gate: task_gate,
            column_index: task_gate,
            gate_label: gate_label.to_string(),
            status_label: status_label.to_string(),
            priority: "PRIMARY".to_string(),
            tdd_phase,
            blake3_seal: blake3_seal.to_string(),
            worktree_branch,
            wave_label: format!("Wave {}/{}", task.wave_index, total_waves.max(1)),
            gauntlet_attempt: cycle.gauntlet_attempt,
            next_step_command,
        }
    }

    pub fn build_from_repository(
        repo: &dyn CycleEvidenceRepositoryPort,
    ) -> Result<BoardSnapshot, DomainViolation> {
        let cycle_opt = match repo.find_active_cycle()? {
            ActiveCycleLookup::Found(c) => Some(*c),
            ActiveCycleLookup::NoActiveCycle => repo.find_latest_cycle()?,
        };

        let Some(cycle) = cycle_opt else {
            return Ok(BoardSnapshot::empty());
        };

        let blake3_seal = Self::resolve_blake3_seal(&cycle);
        let tasks = repo.list_atomic_tasks(&cycle.cycle_id)?;
        let total_waves = tasks.iter().map(|t| t.wave_index).max().unwrap_or(1);
        let active_worktrees = tasks
            .iter()
            .filter(|t| {
                matches!(
                    t.worktree_allocation,
                    WorktreeAllocation::Provisioned { .. }
                )
            })
            .count();

        let cards = if tasks.is_empty() {
            let gate = cycle.current_gate.as_u8().min(6);
            let (gate_label, status_label) = Self::gate_labels(gate);
            let next_cmd = match gate {
                0 => "tk spec prompt --idea \"...\" --context-hops 2 --json".to_string(),
                1 => format!("tk spec lock --spec {} --json", cycle.spec_file_path),
                2 => "tk wave plan --json".to_string(),
                _ => "tk status --json".to_string(),
            };
            vec![KanbanCardSnapshot {
                card_id: cycle.cycle_id.clone(),
                cycle_id: cycle.cycle_id.clone(),
                task_id: cycle.cycle_id.clone(),
                title: format!("Cycle {}", cycle.cycle_id),
                gate,
                column_index: gate,
                gate_label: gate_label.to_string(),
                status_label: status_label.to_string(),
                priority: "PRIMARY".to_string(),
                tdd_phase: if gate >= 2 {
                    "SEALED".to_string()
                } else {
                    "DRAFT".to_string()
                },
                blake3_seal,
                worktree_branch: cycle.work_branch.clone(),
                wave_label: "Wave 0/0".to_string(),
                gauntlet_attempt: cycle.gauntlet_attempt,
                next_step_command: next_cmd,
            }]
        } else {
            tasks
                .iter()
                .map(|t| Self::build_task_card(&cycle, t, total_waves, &blake3_seal))
                .collect()
        };

        Ok(BoardSnapshot {
            cycle_id: cycle.cycle_id,
            current_gate: cycle.current_gate.as_u8(),
            metrics: BoardMetricsSummary {
                active_cycles: 1,
                total_cards: cards.len(),
                active_worktrees,
                gauntlet_attempts: cycle.gauntlet_attempt,
            },
            cards,
        })
    }
}
