// Deterministic Contextual Next-Step Guidance Resolver.
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::entities::TddPhase;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NextStepPriority {
    Primary,
    Secondary,
    Optional,
}

impl NextStepPriority {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Primary => "PRIMARY",
            Self::Secondary => "SECONDARY",
            Self::Optional => "OPTIONAL",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NextStepRecommendation {
    pub priority: NextStepPriority,
    pub target_gate: String,
    pub command: String,
    pub description: String,
}

impl NextStepRecommendation {
    pub fn new(
        priority: NextStepPriority,
        target_gate: impl Into<String>,
        command: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            priority,
            target_gate: target_gate.into(),
            command: command.into(),
            description: description.into(),
        }
    }
}

pub struct NextStepResolver;

impl NextStepResolver {
    pub fn for_no_active_cycle() -> Vec<NextStepRecommendation> {
        vec![
            NextStepRecommendation::new(
                NextStepPriority::Primary,
                "GATE-0",
                "trustkit init --issue <ISSUE_ID>",
                "Initialize a new SDD cycle linked to an issue or local feature.",
            ),
            NextStepRecommendation::new(
                NextStepPriority::Optional,
                "GATE-0",
                "trustkit milestones plan --prd <PRD_PATH> --adr <ADR_PATH>",
                "Extract implementation milestones from PRD and ADR documents before initializing.",
            ),
        ]
    }

    pub fn for_init(spec_file_path: &str, has_populated_spec: bool) -> Vec<NextStepRecommendation> {
        if has_populated_spec {
            vec![
                NextStepRecommendation::new(
                    NextStepPriority::Primary,
                    "GATE-1",
                    format!("trustkit spec lint --file {spec_file_path}"),
                    "Validate the initialized specification against structural and constitutional rules.",
                ),
                NextStepRecommendation::new(
                    NextStepPriority::Secondary,
                    "GATE-2",
                    format!("trustkit spec lock --spec {spec_file_path}"),
                    "Freeze the specification with a BLAKE3 cryptographic seal after lint approval.",
                ),
            ]
        } else {
            vec![
                NextStepRecommendation::new(
                    NextStepPriority::Primary,
                    "GATE-1",
                    "trustkit spec prompt --idea \"<FEATURE_DESCRIPTION>\" --context-hops 2",
                    "Generate the specification scaffold and AST context slice for Gate 1.",
                ),
                NextStepRecommendation::new(
                    NextStepPriority::Secondary,
                    "GATE-1",
                    format!("trustkit spec lint --file {spec_file_path}"),
                    "Validate the specification structure after writing spec.md.",
                ),
            ]
        }
    }

    pub fn for_spec_init(generated_spec_file: &str) -> Vec<NextStepRecommendation> {
        vec![
            NextStepRecommendation::new(
                NextStepPriority::Primary,
                "GATE-1",
                format!("trustkit spec lint --file {generated_spec_file}"),
                "Validate the generated specification structure and traceability matrix.",
            ),
            NextStepRecommendation::new(
                NextStepPriority::Secondary,
                "GATE-2",
                format!("trustkit spec lock --spec {generated_spec_file}"),
                "Freeze the validated specification with a BLAKE3 cryptographic seal.",
            ),
        ]
    }

    pub fn for_spec_prompt(spec_file_path: &str) -> Vec<NextStepRecommendation> {
        let target = if spec_file_path.trim().is_empty() {
            "<SPEC_FILE_PATH>"
        } else {
            spec_file_path
        };
        vec![
            NextStepRecommendation::new(
                NextStepPriority::Primary,
                "GATE-1",
                format!("trustkit spec lint --file {target}"),
                "Save the drafted specification to disk and run static structural validation.",
            ),
            NextStepRecommendation::new(
                NextStepPriority::Secondary,
                "GATE-2",
                format!("trustkit spec lock --spec {target}"),
                "Seal the specification with BLAKE3 once linter reports CleanPass.",
            ),
        ]
    }

    pub fn for_spec_lint(spec_file_path: &str, is_clean_pass: bool) -> Vec<NextStepRecommendation> {
        if is_clean_pass {
            vec![NextStepRecommendation::new(
                NextStepPriority::Primary,
                "GATE-2",
                format!("trustkit spec lock --spec {spec_file_path}"),
                "Freeze the validated specification with a BLAKE3 cryptographic seal.",
            )]
        } else {
            vec![NextStepRecommendation::new(
                NextStepPriority::Primary,
                "GATE-1",
                format!("trustkit spec lint --file {spec_file_path}"),
                "Remediate reported specification violations and re-run linter.",
            )]
        }
    }

    pub fn for_spec_lock(spec_dir: &str, has_task_files: bool) -> Vec<NextStepRecommendation> {
        let tasks_dir = if spec_dir.trim().is_empty() {
            "tasks".to_string()
        } else {
            format!("{}/tasks", spec_dir.trim_end_matches('/'))
        };
        if has_task_files {
            vec![NextStepRecommendation::new(
                NextStepPriority::Primary,
                "GATE-3",
                "trustkit wave plan",
                "Build the dependency DAG and plan parallel execution waves from existing atomic tasks.",
            )]
        } else {
            vec![NextStepRecommendation::new(
                NextStepPriority::Primary,
                "GATE-3",
                format!("trustkit wave plan --tasks-dir {tasks_dir}"),
                "Create atomic task files (TASK-NNN.md) in the tasks directory and plan execution waves.",
            )]
        }
    }

    pub fn for_wave_plan(first_wave_index: usize) -> Vec<NextStepRecommendation> {
        vec![NextStepRecommendation::new(
            NextStepPriority::Primary,
            "GATE-4",
            format!("trustkit wave run --wave {first_wave_index}"),
            "Provision isolated Git worktrees and dispatch execution for the planned wave.",
        )]
    }

    pub fn for_wave_run(
        wave_index: u32,
        failed_tasks: &[String],
        next_wave_index: Option<u32>,
        first_pending_task: Option<&str>,
    ) -> Vec<NextStepRecommendation> {
        if let Some(failed) = failed_tasks.first() {
            return vec![NextStepRecommendation::new(
                NextStepPriority::Primary,
                "GATE-4",
                format!("trustkit task test {failed} --phase red"),
                "Inspect and resolve failing task in current wave before advancing.",
            )];
        }
        if let Some(task_id) = first_pending_task {
            return vec![
                NextStepRecommendation::new(
                    NextStepPriority::Primary,
                    "GATE-4",
                    format!("trustkit task test {task_id} --phase red"),
                    "Record failing RED test evidence for the active task.",
                ),
                NextStepRecommendation::new(
                    NextStepPriority::Secondary,
                    "GATE-4",
                    format!("trustkit task test {task_id} --phase green"),
                    "Implement production logic and record passing GREEN test evidence.",
                ),
            ];
        }
        if let Some(next_wave) = next_wave_index {
            return vec![NextStepRecommendation::new(
                NextStepPriority::Primary,
                "GATE-4",
                format!("trustkit wave run --wave {next_wave}"),
                format!("Wave {wave_index} completed; execute next dependency wave {next_wave}."),
            )];
        }
        Self::for_all_waves_completed()
    }

    pub fn for_task_test(task_id: &str, phase: TddPhase) -> Vec<NextStepRecommendation> {
        match phase {
            TddPhase::Red => vec![NextStepRecommendation::new(
                NextStepPriority::Primary,
                "GATE-4",
                format!("trustkit task test {task_id} --phase green"),
                "Implement minimal production code to pass the test and verify GREEN phase.",
            )],
            TddPhase::Green => vec![
                NextStepRecommendation::new(
                    NextStepPriority::Primary,
                    "GATE-4",
                    format!("trustkit task complete {task_id}"),
                    "Commit verified changes and mark the atomic task as completed.",
                ),
                NextStepRecommendation::new(
                    NextStepPriority::Optional,
                    "GATE-4",
                    format!("trustkit task review {task_id}"),
                    "Run semantic code review on the task diff prior to completion.",
                ),
            ],
        }
    }

    pub fn for_task_complete(
        next_pending_task_in_wave: Option<&str>,
        next_wave_index: Option<u32>,
    ) -> Vec<NextStepRecommendation> {
        if let Some(next_task) = next_pending_task_in_wave {
            return vec![NextStepRecommendation::new(
                NextStepPriority::Primary,
                "GATE-4",
                format!("trustkit task test {next_task} --phase red"),
                "Start TDD RED phase for the next pending task in the current wave.",
            )];
        }
        if let Some(next_wave) = next_wave_index {
            return vec![NextStepRecommendation::new(
                NextStepPriority::Primary,
                "GATE-4",
                format!("trustkit wave run --wave {next_wave}"),
                "Current wave completed; provision and execute the next wave.",
            )];
        }
        Self::for_all_waves_completed()
    }

    pub fn for_all_waves_completed() -> Vec<NextStepRecommendation> {
        vec![
            NextStepRecommendation::new(
                NextStepPriority::Primary,
                "GATE-5",
                "trustkit gauntlet ast-audit",
                "Execute Layer 0 lexical, AST, and anti-cheat assertion audit.",
            ),
            NextStepRecommendation::new(
                NextStepPriority::Secondary,
                "GATE-5",
                "trustkit gauntlet run",
                "Execute Layer 1 Gauntlet critic swarm after Layer 0 CleanPass.",
            ),
        ]
    }
}

#[cfg(test)]
#[path = "next_step_guidance_tests.rs"]
mod tests;
