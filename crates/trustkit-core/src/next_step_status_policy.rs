// Contextual Next-Step Guidance for Gates 5, 6, Milestones, and Cycle Status.
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::entities::{AtomicTask, CycleStatus, GateStage, SddCycle};
use crate::next_step_guidance::{NextStepPriority, NextStepRecommendation, NextStepResolver};
use crate::special_cases::SpecLockState;

impl NextStepResolver {
    pub fn for_ast_audit() -> Vec<NextStepRecommendation> {
        vec![NextStepRecommendation::new(
            NextStepPriority::Primary,
            "GATE-5",
            "trustkit gauntlet run",
            "Execute Layer 1 Gauntlet critic swarm and E2E verification.",
        )]
    }

    pub fn for_gauntlet_run(circuit_breaker_tripped: bool) -> Vec<NextStepRecommendation> {
        if circuit_breaker_tripped {
            vec![NextStepRecommendation::new(
                NextStepPriority::Primary,
                "GATE-5",
                "trustkit doctor",
                "Inspect circuit breaker diagnostics and remediate consecutive failures.",
            )]
        } else {
            vec![NextStepRecommendation::new(
                NextStepPriority::Primary,
                "GATE-6",
                "trustkit done",
                "Consolidate verification evidence and generate Gate 6 certification report.",
            )]
        }
    }

    pub fn for_doctor(circuit_breaker_tripped: bool) -> Vec<NextStepRecommendation> {
        if circuit_breaker_tripped {
            vec![NextStepRecommendation::new(
                NextStepPriority::Primary,
                "GATE-5",
                "trustkit gauntlet ast-audit",
                "Fix failing tests or invariant violations and re-run Layer 0 AST audit.",
            )]
        } else {
            vec![NextStepRecommendation::new(
                NextStepPriority::Primary,
                "GATE-5",
                "trustkit gauntlet run",
                "Cycle is healthy; proceed with Gauntlet Layer 1 verification.",
            )]
        }
    }

    pub fn for_done(target_branch: &str) -> Vec<NextStepRecommendation> {
        let branch = if target_branch.trim().is_empty() {
            "develop"
        } else {
            target_branch
        };
        vec![
            NextStepRecommendation::new(
                NextStepPriority::Primary,
                "GATE-6",
                format!("trustkit merge --target {branch}"),
                "Execute deterministic merge of certified cycle into target branch.",
            ),
            NextStepRecommendation::new(
                NextStepPriority::Optional,
                "GATE-6",
                format!("trustkit pr --target {branch}"),
                "Open Pull Request on demand if human code review is requested.",
            ),
        ]
    }

    pub fn for_close(target_branch: &str) -> Vec<NextStepRecommendation> {
        let branch = if target_branch.trim().is_empty() {
            "develop"
        } else {
            target_branch
        };
        vec![
            NextStepRecommendation::new(
                NextStepPriority::Primary,
                "GATE-6",
                format!("trustkit pr --target {branch}"),
                "Open Pull Request on demand if required by repository governance.",
            ),
            NextStepRecommendation::new(
                NextStepPriority::Secondary,
                "GATE-0",
                "trustkit init --issue <ISSUE_ID>",
                "Start the next SDD cycle when ready.",
            ),
        ]
    }

    pub fn for_merge(target_branch: &str) -> Vec<NextStepRecommendation> {
        let branch = if target_branch.trim().is_empty() {
            "develop"
        } else {
            target_branch
        };
        vec![
            NextStepRecommendation::new(
                NextStepPriority::Optional,
                "GATE-6",
                format!("trustkit pr --target {branch}"),
                "Open Pull Request on demand if required by repository governance.",
            ),
            NextStepRecommendation::new(
                NextStepPriority::Secondary,
                "GATE-0",
                "trustkit init --issue <ISSUE_ID>",
                "Start the next SDD cycle when ready.",
            ),
        ]
    }

    pub fn for_pr() -> Vec<NextStepRecommendation> {
        vec![NextStepRecommendation::new(
            NextStepPriority::Primary,
            "COMPLETED",
            "trustkit status",
            "Pull Request opened; verify cycle status or start a new cycle with 'trustkit init'.",
        )]
    }

    pub fn for_milestones_plan(
        first_milestone_file: Option<&str>,
        first_milestone_id: Option<&str>,
    ) -> Vec<NextStepRecommendation> {
        let file = first_milestone_file.unwrap_or("docs/milestones/M01.md");
        let id = first_milestone_id.unwrap_or("M01");
        vec![
            NextStepRecommendation::new(
                NextStepPriority::Primary,
                "GATE-0",
                format!("trustkit init --milestone {file}"),
                format!("Initialize an SDD cycle from milestone {id}."),
            ),
            NextStepRecommendation::new(
                NextStepPriority::Optional,
                "GATE-0",
                format!("trustkit milestones inspect {id}"),
                format!("Inspect capabilities and technical constraints of milestone {id}."),
            ),
        ]
    }

    pub fn for_cycle_status(
        cycle: &SddCycle,
        tasks: &[AtomicTask],
        has_spec_populated: bool,
        has_task_files: bool,
    ) -> Vec<NextStepRecommendation> {
        if cycle.lifecycle_status == CycleStatus::CircuitBreakerTripped
            || cycle.gauntlet_attempt >= 5
        {
            return vec![NextStepRecommendation::new(
                NextStepPriority::Primary,
                "GATE-5",
                "trustkit doctor",
                "Inspect circuit breaker diagnostics and remediate consecutive Gauntlet failures.",
            )];
        }

        if cycle.lifecycle_status == CycleStatus::Merged {
            return Self::for_merge("develop");
        }

        if cycle.lifecycle_status == CycleStatus::Certified
            || cycle.current_gate == GateStage::Gate6Certified
        {
            return Self::for_done("develop");
        }

        match cycle.current_gate {
            GateStage::Gate0Discovery | GateStage::Gate1SpecDrafting => {
                match &cycle.spec_seal_state {
                    SpecLockState::Frozen { .. } => {
                        Self::for_spec_lock(&cycle.spec_dir, has_task_files)
                    }
                    SpecLockState::DraftUnverified => {
                        Self::for_init(&cycle.spec_file_path, has_spec_populated)
                    }
                }
            }
            GateStage::Gate2SpecSealed => Self::for_spec_lock(&cycle.spec_dir, has_task_files),
            GateStage::Gate3WavePlanned | GateStage::Gate4TasksImplemented => {
                Self::resolve_gate3_or_gate4_status(&cycle.spec_dir, tasks, has_task_files)
            }
            GateStage::Gate5GauntletCleared => Self::for_gauntlet_run(false),
            GateStage::Gate6Certified => Self::for_done("develop"),
        }
    }

    fn resolve_gate3_or_gate4_status(
        spec_dir: &str,
        tasks: &[AtomicTask],
        has_task_files: bool,
    ) -> Vec<NextStepRecommendation> {
        if tasks.is_empty() {
            return Self::for_spec_lock(spec_dir, has_task_files);
        }

        if let Some(red_task) = tasks
            .iter()
            .find(|t| t.tdd_lifecycle_state.is_red_verified())
        {
            return vec![NextStepRecommendation::new(
                NextStepPriority::Primary,
                "GATE-4",
                format!("trustkit task test {} --phase green", red_task.task_id),
                "Task has verified RED evidence; implement code and verify GREEN phase.",
            )];
        }

        if let Some(green_task) = tasks
            .iter()
            .find(|t| t.tdd_lifecycle_state.is_green_verified())
        {
            return vec![NextStepRecommendation::new(
                NextStepPriority::Primary,
                "GATE-4",
                format!("trustkit task complete {}", green_task.task_id),
                "Task has verified GREEN evidence; commit and complete atomic task.",
            )];
        }

        if let Some(pending_task) = tasks
            .iter()
            .find(|t| t.tdd_lifecycle_state.is_awaiting_red())
        {
            if pending_task.worktree_allocation.is_provisioned()
                || pending_task.worktree_allocation.is_merged_and_pruned()
            {
                return vec![NextStepRecommendation::new(
                    NextStepPriority::Primary,
                    "GATE-4",
                    format!("trustkit task test {} --phase red", pending_task.task_id),
                    "Record failing RED test evidence for the provisioned atomic task.",
                )];
            }
            return vec![
                NextStepRecommendation::new(
                    NextStepPriority::Primary,
                    "GATE-4",
                    format!("trustkit wave run --wave {}", pending_task.wave_index),
                    format!(
                        "Provision worktrees and execute wave {} containing pending task {}.",
                        pending_task.wave_index, pending_task.task_id
                    ),
                ),
                NextStepRecommendation::new(
                    NextStepPriority::Secondary,
                    "GATE-4",
                    format!("trustkit task test {} --phase red", pending_task.task_id),
                    "Record failing RED test evidence for the pending atomic task.",
                ),
            ];
        }

        Self::for_all_waves_completed()
    }
}
