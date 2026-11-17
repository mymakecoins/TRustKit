// Handler for `trustkit wave plan`.
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Resilient Error Handling & Zero-Panic Policy.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::cli_contract::WavePlanArgs;
use crate::error_handler::DomainOutcome;
use crate::next_step_presenter::{JsonNextStep, NextStepPresenter};
use serde::Serialize;
use std::path::{Path, PathBuf};
use trustkit_ast::WaveExecutionPlanner;
use trustkit_core::entities::{NextStepResolver, WaveSchedule};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::CycleEvidenceRepositoryPort;
use trustkit_core::special_cases::ActiveCycleLookup;
use trustkit_core::use_cases::plan_execution_waves::{
    self, PlanExecutionWavesOutcome, PlanExecutionWavesRequest,
};
use trustkit_db::sqlite_adapter::SqliteWalRepositoryAdapter;

#[derive(Debug, Serialize)]
pub struct JsonWavePlanSuccessResponse {
    pub status: String,
    pub cycle_id: String,
    pub total_tasks: usize,
    pub total_waves: usize,
    pub waves: Vec<ExecutionWavePayload>,
    pub next_steps: Vec<JsonNextStep>,
}

#[derive(Debug, Serialize)]
pub struct ExecutionWavePayload {
    pub wave_index: usize,
    pub tasks: Vec<TaskPayload>,
}

#[derive(Debug, Serialize)]
pub struct TaskPayload {
    pub task_id: String,
    pub title: String,
    pub architectural_layer: String,
    pub depends_on: Vec<String>,
    pub expected_files: Vec<String>,
    pub slice_kind: String,
    pub blast_radius_score: u32,
}

impl From<&WaveSchedule> for JsonWavePlanSuccessResponse {
    fn from(schedule: &WaveSchedule) -> Self {
        let first_wave_idx = schedule.waves.first().map(|w| w.wave_index).unwrap_or(1);
        let steps = NextStepResolver::for_wave_plan(first_wave_idx);
        let waves = schedule
            .waves
            .iter()
            .map(|w| ExecutionWavePayload {
                wave_index: w.wave_index,
                tasks: w
                    .tasks
                    .iter()
                    .map(|t| TaskPayload {
                        task_id: t.task_id.clone(),
                        title: t.title.clone(),
                        architectural_layer: t.architectural_layer.clone(),
                        depends_on: t.depends_on.clone(),
                        expected_files: t.expected_files.clone(),
                        slice_kind: t.slice_kind.as_str().to_string(),
                        blast_radius_score: t.blast_radius_score,
                    })
                    .collect(),
            })
            .collect();

        Self {
            status: "WAVES_PLANNED".to_string(),
            cycle_id: schedule.cycle_id.clone(),
            total_tasks: schedule.total_tasks,
            total_waves: schedule.waves.len(),
            waves,
            next_steps: NextStepPresenter::to_json_vec(&steps),
        }
    }
}

pub fn execute(args: &WavePlanArgs, json_mode: bool) -> Result<DomainOutcome, DomainViolation> {
    let db_path = Path::new(".trustkit").join("state.db");
    if !db_path.exists() {
        return Err(DomainViolation::InfrastructureBoundaryFault {
            subsystem: "wave_plan".to_string(),
            details: "No active SDD cycle found in state.db. Run 'trustkit init' and 'trustkit spec lock' first.".to_string(),
        });
    }

    let repo = SqliteWalRepositoryAdapter::open(&db_path)?;
    let active_cycle = match repo.find_active_cycle()? {
        ActiveCycleLookup::Found(cycle) => *cycle,
        ActiveCycleLookup::NoActiveCycle => {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "wave_plan".to_string(),
                details:
                    "No active SDD cycle found. Run 'trustkit init' and 'trustkit spec lock' first."
                        .to_string(),
            });
        }
    };

    let tasks_dir = resolve_tasks_dir(args.tasks_dir.as_deref(), &active_cycle.spec_dir)?;
    let tasks = WaveExecutionPlanner::parse_tasks_dir(&tasks_dir)?;
    if tasks.is_empty() {
        return Err(DomainViolation::InfrastructureBoundaryFault {
            subsystem: "wave_plan".to_string(),
            details: format!(
                "No atomic task files (TASK-*.md) found in '{}'.",
                tasks_dir.display()
            ),
        });
    }

    let planner = WaveExecutionPlanner;
    let req = PlanExecutionWavesRequest { tasks };

    let outcome = plan_execution_waves::execute(req, &repo, &planner)?;

    let rendered = render_output(&outcome, json_mode);
    println!("{rendered}");

    Ok(DomainOutcome::Success)
}

fn resolve_tasks_dir(
    provided_dir: Option<&str>,
    spec_dir: &str,
) -> Result<PathBuf, DomainViolation> {
    if let Some(path_str) = provided_dir {
        let path = PathBuf::from(path_str);
        if path.exists() && path.is_dir() {
            return Ok(path);
        }
        return Err(DomainViolation::InfrastructureBoundaryFault {
            subsystem: "wave_plan".to_string(),
            details: format!(
                "Provided tasks directory does not exist or is not a directory: '{path_str}'"
            ),
        });
    }

    let candidates = [
        Path::new(spec_dir).join("tasks"),
        Path::new(spec_dir).to_path_buf(),
        PathBuf::from("tasks"),
    ];

    for candidate in candidates {
        if candidate.exists() && candidate.is_dir() {
            return Ok(candidate);
        }
    }

    Err(DomainViolation::InfrastructureBoundaryFault {
        subsystem: "wave_plan".to_string(),
        details: format!(
            "Could not locate tasks directory automatically under '{}'. Use --tasks-dir <DIR>.",
            spec_dir
        ),
    })
}

fn render_output(outcome: &PlanExecutionWavesOutcome, json_mode: bool) -> String {
    if json_mode {
        let payload = JsonWavePlanSuccessResponse::from(&outcome.wave_schedule);
        serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
    } else {
        let s = &outcome.wave_schedule;
        let first_wave_idx = s.waves.first().map(|w| w.wave_index).unwrap_or(1);
        let steps = NextStepResolver::for_wave_plan(first_wave_idx);
        let mut out = format!(
            "[WAVES PLANNED]\nCycle ID: {}\nTotal Tasks: {} | Total Waves: {}\n",
            s.cycle_id,
            s.total_tasks,
            s.waves.len()
        );
        for wave in &s.waves {
            out.push_str(&format!("\nWave {}:\n", wave.wave_index));
            for task in &wave.tasks {
                out.push_str(&format!(
                    "  - {} [{}] ({}) [slice: {} | blast_radius: {}]\n",
                    task.task_id,
                    task.architectural_layer,
                    task.title,
                    task.slice_kind.as_str(),
                    task.blast_radius_score
                ));
            }
        }
        out.push_str(&NextStepPresenter::render_ascii_block(&steps));
        out
    }
}
