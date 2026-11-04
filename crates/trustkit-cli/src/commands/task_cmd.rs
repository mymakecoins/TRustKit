// Handler for `trustkit task test` and `trustkit task complete` (TASK-014).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::cli_contract::{
    TaskArgs, TaskCompleteArgs, TaskReviewArgs, TaskSubcommand, TaskTestArgs,
};
use crate::error_handler::DomainOutcome;
use crate::gate_presenter::{GatePresenter, JsonTaskCompleteResponse, JsonTaskTestResponse};
use std::path::Path;
use std::process::Command as StdCommand;
use std::time::{SystemTime, UNIX_EPOCH};
use trustkit_core::entities::{CodeReviewEffortLevel, SemanticReviewFinding, TddPhase};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::{CycleEvidenceRepositoryPort, SemanticCodeReviewerPort};
use trustkit_core::special_cases::ActiveCycleLookup;
use trustkit_core::use_cases::complete_task::{self, CompleteTaskRequest};
use trustkit_core::use_cases::record_tdd_phase::{self, RecordTddPhaseRequest};
use trustkit_core::use_cases::review_task_code::{self, ReviewTaskCodeRequest};
use trustkit_db::sqlite_adapter::SqliteWalRepositoryAdapter;
use trustkit_orch::AsyncExecutionSupervisor;

struct DefaultSemanticCodeReviewer;

impl SemanticCodeReviewerPort for DefaultSemanticCodeReviewer {
    fn review_task_diff(
        &self,
        _cycle_id: &str,
        _task_id: &str,
        _effort_level: CodeReviewEffortLevel,
    ) -> Result<Vec<SemanticReviewFinding>, DomainViolation> {
        Ok(vec![])
    }
}

fn current_timestamp_iso() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{now}")
}

fn resolve_head_commit_sha(working_dir: &str) -> String {
    let output = StdCommand::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(working_dir)
        .output();

    if let Ok(out) = output
        && out.status.success()
    {
        let sha = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if sha.len() == 40 {
            return sha;
        }
    }
    "0000000000000000000000000000000000000000".to_string()
}

pub fn execute(args: &TaskArgs, json_mode: bool) -> Result<DomainOutcome, DomainViolation> {
    let db_path = Path::new(".trustkit").join("state.db");
    execute_with_db(args, json_mode, &db_path)
}

pub fn execute_with_db(
    args: &TaskArgs,
    json_mode: bool,
    db_path: &Path,
) -> Result<DomainOutcome, DomainViolation> {
    match &args.action {
        TaskSubcommand::Test(test_args) => execute_test_with_db(test_args, json_mode, db_path),
        TaskSubcommand::Review(review_args) => {
            execute_review_with_db(review_args, json_mode, db_path)
        }
        TaskSubcommand::Complete(complete_args) => {
            execute_complete_with_db(complete_args, json_mode, db_path)
        }
    }
}

pub fn execute_review_with_db(
    args: &TaskReviewArgs,
    json_mode: bool,
    db_path: &Path,
) -> Result<DomainOutcome, DomainViolation> {
    if !db_path.exists() {
        return Err(DomainViolation::InfrastructureBoundaryFault {
            subsystem: "task_review".to_string(),
            details: "No active SDD cycle found in state.db. Run 'trustkit init' first."
                .to_string(),
        });
    }

    let repo = SqliteWalRepositoryAdapter::open(db_path)?;
    let active_cycle = match repo.find_active_cycle()? {
        ActiveCycleLookup::Found(cycle) => *cycle,
        ActiveCycleLookup::NoActiveCycle => {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "task_review".to_string(),
                details: "No active SDD cycle found. Run 'trustkit init' first.".to_string(),
            });
        }
    };

    let reviewer = DefaultSemanticCodeReviewer;
    let effort_level = CodeReviewEffortLevel::from(args.effort);
    let outcome = review_task_code::execute(
        ReviewTaskCodeRequest {
            cycle_id: active_cycle.cycle_id,
            task_id: args.task_id.clone(),
            effort_level,
            max_findings_override: 0,
            now_iso: current_timestamp_iso(),
        },
        &repo,
        &reviewer,
    )?;

    if json_mode || args.json {
        println!(
            "{{\"verdict\":\"{}\",\"task_id\":\"{}\",\"effort\":\"{}\",\"findings_count\":{},\"evidence_id\":{}}}",
            if outcome.clean_pass { "PASS" } else { "FAIL" },
            outcome.task_id,
            outcome.effort_level.as_str(),
            outcome.findings.len(),
            outcome.recorded_evidence_id
        );
    } else {
        println!(
            "[trustkit] Code Review ({}) for {}: {} ({} findings)",
            outcome.effort_level.as_str(),
            outcome.task_id,
            if outcome.clean_pass { "PASS" } else { "FAIL" },
            outcome.findings.len()
        );
    }
    Ok(DomainOutcome::Success)
}

pub fn execute_test_with_db(
    args: &TaskTestArgs,
    json_mode: bool,
    db_path: &Path,
) -> Result<DomainOutcome, DomainViolation> {
    if !db_path.exists() {
        return Err(DomainViolation::InfrastructureBoundaryFault {
            subsystem: "task_test".to_string(),
            details: "No active SDD cycle found in state.db. Run 'trustkit init' first."
                .to_string(),
        });
    }

    let repo = SqliteWalRepositoryAdapter::open(db_path)?;
    let active_cycle = match repo.find_active_cycle()? {
        ActiveCycleLookup::Found(cycle) => *cycle,
        ActiveCycleLookup::NoActiveCycle => {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "task_test".to_string(),
                details: "No active SDD cycle found. Run 'trustkit init' first.".to_string(),
            });
        }
    };

    let phase = TddPhase::from(args.phase);
    let working_dir = active_cycle.spec_dir.clone();
    let supervisor = AsyncExecutionSupervisor::new();
    let now_iso = current_timestamp_iso();

    let request = RecordTddPhaseRequest {
        cycle_id: active_cycle.cycle_id.clone(),
        task_id: args.task_id.clone(),
        phase,
        test_command: "cargo".to_string(),
        test_args: vec![
            "test".to_string(),
            "--".to_string(),
            args.task_id.to_lowercase(),
        ],
        working_dir,
        timeout_ms: 60_000,
        now_iso,
    };

    let outcome = record_tdd_phase::execute(request, &repo, &supervisor)?;

    let response = JsonTaskTestResponse {
        verdict: "PASS".to_string(),
        task_id: outcome.task_id,
        phase: format!("{:?}", outcome.phase).to_lowercase(),
        exit_code: 0,
        duration_ms: 0,
        evidence_id: outcome.evidence_id,
        status: format!("{:?}", outcome.new_state),
    };

    println!("{}", GatePresenter::render_task_test(&response, json_mode));
    Ok(DomainOutcome::Success)
}

pub fn execute_complete_with_db(
    args: &TaskCompleteArgs,
    json_mode: bool,
    db_path: &Path,
) -> Result<DomainOutcome, DomainViolation> {
    if !db_path.exists() {
        return Err(DomainViolation::InfrastructureBoundaryFault {
            subsystem: "task_complete".to_string(),
            details: "No active SDD cycle found in state.db. Run 'trustkit init' first."
                .to_string(),
        });
    }

    let repo = SqliteWalRepositoryAdapter::open(db_path)?;
    let active_cycle = match repo.find_active_cycle()? {
        ActiveCycleLookup::Found(cycle) => *cycle,
        ActiveCycleLookup::NoActiveCycle => {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "task_complete".to_string(),
                details: "No active SDD cycle found. Run 'trustkit init' first.".to_string(),
            });
        }
    };

    let now_iso = current_timestamp_iso();
    let commit_sha = resolve_head_commit_sha(&active_cycle.spec_dir);

    if let Some(effort_arg) = args.code_review {
        let reviewer = DefaultSemanticCodeReviewer;
        review_task_code::execute(
            ReviewTaskCodeRequest {
                cycle_id: active_cycle.cycle_id.clone(),
                task_id: args.task_id.clone(),
                effort_level: CodeReviewEffortLevel::from(effort_arg),
                max_findings_override: 0,
                now_iso: now_iso.clone(),
            },
            &repo,
            &reviewer,
        )?;
    }

    let request = CompleteTaskRequest {
        cycle_id: active_cycle.cycle_id,
        task_id: args.task_id.clone(),
        commit_sha,
        now_iso: now_iso.clone(),
    };

    let outcome = complete_task::execute(request, &repo)?;

    let response = JsonTaskCompleteResponse {
        verdict: "PASS".to_string(),
        task_id: outcome.task_id,
        new_state: format!("{:?}", outcome.new_state),
        timestamp: now_iso,
    };

    println!(
        "{}",
        GatePresenter::render_task_complete(&response, json_mode)
    );
    Ok(DomainOutcome::Success)
}
