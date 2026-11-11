// Dual-audience presenter emitting null-free JSON or emoji-free ASCII.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::next_step_presenter::{JsonNextStep, NextStepPresenter};
use serde::Serialize;
use std::path::Path;
use trustkit_core::entities::{AtomicTask, CycleStatus, MethodologyProfile, NextStepResolver};
use trustkit_core::special_cases::{
    ActiveCycleLookup, BoardSyncOutcome, IssueLinkage, SpecLockState,
};
use trustkit_core::use_cases::initialize_cycle::InitializeCycleOutcome;

#[derive(Debug, Serialize)]
pub struct JsonHookApprovalResponse {
    pub verdict: String,
    pub exit_code: i32,
    pub evaluated_command: String,
}

#[derive(Debug, Serialize)]
pub struct JsonInitResponse {
    pub verdict: String,
    pub gate: String,
    pub cycle_id: String,
    pub methodology_profile: String,
    pub spec_dir: String,
    pub spec_file_path: String,
    pub work_branch: String,
    pub issue_linkage: JsonIssueLinkage,
    pub pre_existing_dirty_files: Vec<String>,
    pub next_steps: Vec<JsonNextStep>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind")]
pub enum JsonIssueLinkage {
    #[serde(rename = "LINKED_ISSUE")]
    LinkedIssue {
        issue_number: u64,
        issue_url: String,
        board_column: String,
        sync_outcome: String,
    },
    #[serde(rename = "UNLINKED_LOCAL_CYCLE")]
    UnlinkedLocalCycle {
        directory_slug: String,
        sync_outcome: String,
    },
}

#[derive(Debug, Serialize)]
pub struct JsonStatusResponse {
    pub verdict: String,
    pub cycle_state: JsonCycleState,
    pub next_steps: Vec<JsonNextStep>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind")]
pub enum JsonCycleState {
    #[serde(rename = "ACTIVE_CYCLE")]
    ActiveCycle {
        cycle_id: String,
        current_gate: u8,
        gate_label: String,
        lifecycle_status: String,
        gauntlet_attempt: u8,
        spec_lock_state: JsonSpecLockState,
        work_branch: String,
        pre_existing_dirty_files: Vec<String>,
    },
    #[serde(rename = "NO_ACTIVE_CYCLE")]
    NoActiveCycle { remediation: String },
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind")]
pub enum JsonSpecLockState {
    #[serde(rename = "DRAFT_UNVERIFIED")]
    DraftUnverified,
    #[serde(rename = "FROZEN")]
    Frozen {
        spec_path: String,
        blake3_digest: String,
    },
}

pub struct Presenter;

impl Presenter {
    fn is_json_mode(json_mode: bool) -> bool {
        json_mode || std::env::var("TRUSTKIT_AGENT_MODE").as_deref() == Ok("1")
    }

    fn has_populated_file(path: &str) -> bool {
        std::fs::metadata(path)
            .map(|m| m.len() > 0)
            .unwrap_or(false)
    }

    fn has_task_files_in_spec_dir(spec_dir: &str) -> bool {
        let tasks_dir = Path::new(spec_dir).join("tasks");
        std::fs::read_dir(tasks_dir)
            .ok()
            .into_iter()
            .flatten()
            .flatten()
            .any(|e| {
                e.file_name()
                    .to_str()
                    .is_some_and(|n| n.starts_with("TASK-") && n.ends_with(".md"))
            })
    }

    pub fn render_init(outcome: &InitializeCycleOutcome, json_mode: bool) -> String {
        let has_populated_spec = Self::has_populated_file(&outcome.cycle.spec_file_path)
            || matches!(
                outcome.cycle.methodology_profile,
                MethodologyProfile::StructuredSdd { .. }
            );
        let steps = NextStepResolver::for_init(&outcome.cycle.spec_file_path, has_populated_spec);

        if Self::is_json_mode(json_mode) {
            let sync_outcome_str = match &outcome.sync_outcome {
                BoardSyncOutcome::Synchronized { .. } => "SYNCHRONIZED",
                BoardSyncOutcome::SkippedUnlinkedCycle => "SKIPPED_UNLINKED_CYCLE",
                BoardSyncOutcome::DeferredOffline { .. } => "DEFERRED_OFFLINE",
            };

            let linkage = match &outcome.cycle.issue_linkage {
                IssueLinkage::LinkedIssue {
                    issue_number,
                    issue_url,
                    board_column,
                    ..
                } => JsonIssueLinkage::LinkedIssue {
                    issue_number: *issue_number,
                    issue_url: issue_url.clone(),
                    board_column: board_column.as_str().to_string(),
                    sync_outcome: sync_outcome_str.to_string(),
                },
                IssueLinkage::UnlinkedLocalCycle { directory_slug } => {
                    JsonIssueLinkage::UnlinkedLocalCycle {
                        directory_slug: directory_slug.clone(),
                        sync_outcome: sync_outcome_str.to_string(),
                    }
                }
            };

            let methodology_profile_str = match &outcome.cycle.methodology_profile {
                MethodologyProfile::StandardGauntletV2 => "STANDARD_GAUNTLET_V2",
                MethodologyProfile::StructuredSdd { .. } => "STRUCTURED_SDD",
            };

            let payload = JsonInitResponse {
                verdict: "PASS".to_string(),
                gate: outcome.cycle.current_gate.label().to_string(),
                cycle_id: outcome.cycle.cycle_id.clone(),
                methodology_profile: methodology_profile_str.to_string(),
                spec_dir: outcome.cycle.spec_dir.clone(),
                spec_file_path: outcome.cycle.spec_file_path.clone(),
                work_branch: outcome.cycle.work_branch.clone(),
                issue_linkage: linkage,
                pre_existing_dirty_files: outcome.cycle.pre_existing_dirty_files.clone(),
                next_steps: NextStepPresenter::to_json_vec(&steps),
            };

            serde_json::to_string_pretty(&payload)
                .unwrap_or_else(|_| "{\"verdict\":\"FAIL\"}".to_string())
        } else {
            let mut out = String::new();
            out.push_str(&format!(
                "[PASS] [GATE-0] SDD Cycle Initialized: {}\n",
                outcome.cycle.cycle_id
            ));
            out.push_str("------------------------------------------------------------\n");
            out.push_str(&format!("Cycle ID          : {}\n", outcome.cycle.cycle_id));
            out.push_str(&format!(
                "Gate Stage        : {}\n",
                outcome.cycle.current_gate.label()
            ));
            out.push_str(&format!(
                "Methodology Profile: {}\n",
                match &outcome.cycle.methodology_profile {
                    MethodologyProfile::StandardGauntletV2 => "STANDARD_GAUNTLET_V2",
                    MethodologyProfile::StructuredSdd { .. } => "STRUCTURED_SDD",
                }
            ));
            out.push_str(&format!("Spec Directory    : {}\n", outcome.cycle.spec_dir));
            out.push_str(&format!(
                "Spec File         : {}\n",
                outcome.cycle.spec_file_path
            ));
            out.push_str(&format!(
                "Work Branch       : {}\n",
                outcome.cycle.work_branch
            ));
            out.push_str(&format!(
                "Dirty Baseline    : {} files",
                outcome.cycle.pre_existing_dirty_files.len()
            ));
            out.push_str(&NextStepPresenter::render_ascii_block(&steps));
            out
        }
    }

    pub fn render_hook_pre_tool(evaluated_command: &str, json_mode: bool) -> String {
        if Self::is_json_mode(json_mode) {
            let payload = JsonHookApprovalResponse {
                verdict: "PASS".to_string(),
                exit_code: 0,
                evaluated_command: evaluated_command.to_string(),
            };
            serde_json::to_string_pretty(&payload)
                .unwrap_or_else(|_| "{\"verdict\":\"PASS\",\"exit_code\":0}".to_string())
        } else {
            format!("[PASS] Command approved: {evaluated_command}")
        }
    }

    pub fn render_status(lookup: &ActiveCycleLookup, json_mode: bool) -> String {
        Self::render_status_with_tasks(lookup, &[], json_mode)
    }

    pub fn render_status_with_tasks(
        lookup: &ActiveCycleLookup,
        tasks: &[AtomicTask],
        json_mode: bool,
    ) -> String {
        let steps = match lookup {
            ActiveCycleLookup::Found(cycle) => {
                let has_spec = Self::has_populated_file(&cycle.spec_file_path);
                let has_tasks =
                    !tasks.is_empty() || Self::has_task_files_in_spec_dir(&cycle.spec_dir);
                NextStepResolver::for_cycle_status(cycle, tasks, has_spec, has_tasks)
            }
            ActiveCycleLookup::NoActiveCycle => NextStepResolver::for_no_active_cycle(),
        };

        if Self::is_json_mode(json_mode) {
            let cycle_state = match lookup {
                ActiveCycleLookup::Found(cycle) => {
                    let spec_lock_state = match &cycle.spec_seal_state {
                        SpecLockState::DraftUnverified => JsonSpecLockState::DraftUnverified,
                        SpecLockState::Frozen {
                            spec_path,
                            blake3_digest,
                        } => JsonSpecLockState::Frozen {
                            spec_path: spec_path.clone(),
                            blake3_digest: blake3_digest.clone(),
                        },
                    };
                    let lifecycle_str = match cycle.lifecycle_status {
                        CycleStatus::Active => "ACTIVE",
                        CycleStatus::Paused => "PAUSED",
                        CycleStatus::CircuitBreakerTripped => "CIRCUIT_BREAKER_TRIPPED",
                        CycleStatus::Certified => "CERTIFIED",
                        CycleStatus::Merged => "MERGED",
                    };
                    JsonCycleState::ActiveCycle {
                        cycle_id: cycle.cycle_id.clone(),
                        current_gate: cycle.current_gate.as_u8(),
                        gate_label: cycle.current_gate.label().to_string(),
                        lifecycle_status: lifecycle_str.to_string(),
                        gauntlet_attempt: cycle.gauntlet_attempt,
                        spec_lock_state,
                        work_branch: cycle.work_branch.clone(),
                        pre_existing_dirty_files: cycle.pre_existing_dirty_files.clone(),
                    }
                }
                ActiveCycleLookup::NoActiveCycle => JsonCycleState::NoActiveCycle {
                    remediation:
                        "Run 'trustkit init [--issue <ID>]' to initialize a new SDD cycle."
                            .to_string(),
                },
            };

            serde_json::to_string_pretty(&JsonStatusResponse {
                verdict: "PASS".to_string(),
                cycle_state,
                next_steps: NextStepPresenter::to_json_vec(&steps),
            })
            .unwrap_or_else(|_| "{\"verdict\":\"FAIL\"}".to_string())
        } else {
            match lookup {
                ActiveCycleLookup::Found(cycle) => {
                    let lock_state_str = match &cycle.spec_seal_state {
                        SpecLockState::DraftUnverified => "DRAFT_UNVERIFIED",
                        SpecLockState::Frozen { .. } => "FROZEN",
                    };
                    let status_str = match cycle.lifecycle_status {
                        CycleStatus::Active => "ACTIVE",
                        CycleStatus::Paused => "PAUSED",
                        CycleStatus::CircuitBreakerTripped => "CIRCUIT_BREAKER_TRIPPED",
                        CycleStatus::Certified => "CERTIFIED",
                        CycleStatus::Merged => "MERGED",
                    };
                    let gate_prefix = format!("[GATE-{}]", cycle.current_gate.as_u8());
                    let mut out = String::new();
                    out.push_str(&format!("[PASS] {} Active Cycle Status\n", gate_prefix));
                    out.push_str("------------------------------------------------------------\n");
                    out.push_str(&format!("Cycle ID          : {}\n", cycle.cycle_id));
                    out.push_str(&format!(
                        "Current Gate      : {} ({})\n",
                        cycle.current_gate.as_u8(),
                        cycle.current_gate.label()
                    ));
                    out.push_str(&format!("Lifecycle Status  : {}\n", status_str));
                    out.push_str(&format!(
                        "Gauntlet Attempts : {} / 5\n",
                        cycle.gauntlet_attempt
                    ));
                    out.push_str(&format!("Spec Lock State   : {}\n", lock_state_str));
                    out.push_str(&format!("Work Branch       : {}\n", cycle.work_branch));
                    out.push_str(&format!(
                        "Dirty Baseline    : {} files",
                        cycle.pre_existing_dirty_files.len()
                    ));
                    out.push_str(&NextStepPresenter::render_ascii_block(&steps));
                    out
                }
                ActiveCycleLookup::NoActiveCycle => {
                    let mut out = "[PASS] No active cycle found.\nRun 'trustkit init [--issue <ID>]' to initialize a new SDD cycle.".to_string();
                    out.push_str(&NextStepPresenter::render_ascii_block(&steps));
                    out
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "presenter_tests.rs"]
mod tests;
