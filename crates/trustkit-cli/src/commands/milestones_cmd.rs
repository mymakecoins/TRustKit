// Handler for `trustkit milestones plan`, `list`, and `inspect`.
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::cli_contract::{
    MilestonesArgs, MilestonesInspectArgs, MilestonesListArgs, MilestonesPlanArgs,
    MilestonesSubcommand,
};
use crate::error_handler::DomainOutcome;
use crate::milestones_presenter::{
    JsonMilestoneCapability, JsonMilestoneDetail, JsonMilestoneSummary,
    JsonMilestonesInspectResponse, JsonMilestonesListResponse, JsonMilestonesPlanResponse,
    JsonTechnicalConstraint, MilestonesPresenter,
};
use crate::next_step_presenter::NextStepPresenter;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use trustkit_ast::PrdAdrMilestoneExtractor;
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::MilestoneRepositoryPort;
use trustkit_core::use_cases::plan_milestones::{self, PlanMilestonesRequest};
use trustkit_db::sqlite_adapter::SqliteWalRepositoryAdapter;

fn current_timestamp_iso() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{now}")
}

pub fn execute(args: &MilestonesArgs, json_mode: bool) -> Result<DomainOutcome, DomainViolation> {
    let db_path = Path::new(".trustkit").join("state.db");
    execute_with_db(args, json_mode, &db_path)
}

pub fn execute_with_db(
    args: &MilestonesArgs,
    json_mode: bool,
    db_path: &Path,
) -> Result<DomainOutcome, DomainViolation> {
    match &args.action {
        MilestonesSubcommand::Plan(plan_args) => {
            execute_plan_with_db(plan_args, json_mode, db_path)
        }
        MilestonesSubcommand::List(list_args) => {
            execute_list_with_db(list_args, json_mode, db_path)
        }
        MilestonesSubcommand::Inspect(inspect_args) => {
            execute_inspect_with_db(inspect_args, json_mode, db_path)
        }
    }
}

pub fn execute_plan_with_db(
    args: &MilestonesPlanArgs,
    json_mode: bool,
    db_path: &Path,
) -> Result<DomainOutcome, DomainViolation> {
    if let Some(parent) = db_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let repo = SqliteWalRepositoryAdapter::open(db_path)?;
    let extractor = PrdAdrMilestoneExtractor;

    let req = PlanMilestonesRequest {
        prd_path: args.prd.clone(),
        adr_path: args.adr.clone(),
        output_dir: args.output_dir.clone(),
        now_iso: current_timestamp_iso(),
    };

    let outcome = plan_milestones::execute(req, &extractor, Some(&repo))?;

    let summaries: Vec<JsonMilestoneSummary> = outcome
        .plan
        .milestones
        .iter()
        .map(|m| JsonMilestoneSummary {
            milestone_id: m.milestone_id.clone(),
            title: m.title.clone(),
            slug: m.slug.clone(),
            status: m.status.as_str().to_string(),
            dependencies: m.dependencies.clone(),
            capabilities_count: m.capabilities.len(),
            technical_constraints_count: m.technical_constraints.len(),
            file_path: m.file_path.clone(),
        })
        .collect();

    let generated_files: Vec<String> = outcome
        .generated_artifacts
        .iter()
        .map(|a| a.file_path.clone())
        .collect();

    if json_mode || args.json {
        let steps = MilestonesPresenter::resolve_plan_next_steps(&outcome.plan.milestones);
        let resp = JsonMilestonesPlanResponse {
            verdict: "PASS".to_string(),
            status: "MILESTONES_PLANNED".to_string(),
            plan_id: outcome.plan.plan_id.clone(),
            total_milestones: outcome.plan.milestones.len(),
            total_capabilities: outcome.plan.total_capabilities(),
            generated_files,
            milestones: summaries,
            next_steps: NextStepPresenter::to_json_vec(&steps),
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&resp).unwrap_or_default()
        );
    } else {
        println!("{}", MilestonesPresenter::render_plan_text(&outcome));
    }

    Ok(DomainOutcome::Success)
}

pub fn execute_list_with_db(
    args: &MilestonesListArgs,
    json_mode: bool,
    db_path: &Path,
) -> Result<DomainOutcome, DomainViolation> {
    if !db_path.exists() {
        return Err(DomainViolation::InfrastructureBoundaryFault {
            subsystem: "milestones_list".to_string(),
            details: "No active state.db found. Run 'trustkit milestones plan' first.".to_string(),
        });
    }

    let repo = SqliteWalRepositoryAdapter::open(db_path)?;
    let milestones = repo.list_milestones(args.plan_id.as_deref())?;

    let summaries: Vec<JsonMilestoneSummary> = milestones
        .iter()
        .map(|m| JsonMilestoneSummary {
            milestone_id: m.milestone_id.clone(),
            title: m.title.clone(),
            slug: m.slug.clone(),
            status: m.status.as_str().to_string(),
            dependencies: m.dependencies.clone(),
            capabilities_count: m.capabilities.len(),
            technical_constraints_count: m.technical_constraints.len(),
            file_path: m.file_path.clone(),
        })
        .collect();

    if json_mode || args.json {
        let steps = MilestonesPresenter::resolve_plan_next_steps(&milestones);
        let resp = JsonMilestonesListResponse {
            verdict: "PASS".to_string(),
            status: "MILESTONES_LISTED".to_string(),
            total_milestones: summaries.len(),
            milestones: summaries,
            next_steps: NextStepPresenter::to_json_vec(&steps),
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&resp).unwrap_or_default()
        );
    } else {
        println!("{}", MilestonesPresenter::render_list_text(&milestones));
    }

    Ok(DomainOutcome::Success)
}

pub fn execute_inspect_with_db(
    args: &MilestonesInspectArgs,
    json_mode: bool,
    db_path: &Path,
) -> Result<DomainOutcome, DomainViolation> {
    if !db_path.exists() {
        return Err(DomainViolation::InfrastructureBoundaryFault {
            subsystem: "milestones_inspect".to_string(),
            details: "No active state.db found. Run 'trustkit milestones plan' first.".to_string(),
        });
    }

    let repo = SqliteWalRepositoryAdapter::open(db_path)?;
    let milestone = match repo.find_milestone_by_id(&args.milestone_id)? {
        Some(m) => m,
        None => {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "milestones_inspect".to_string(),
                details: format!("Milestone '{}' not found in state.db", args.milestone_id),
            });
        }
    };

    let detail = JsonMilestoneDetail {
        milestone_id: milestone.milestone_id.clone(),
        plan_id: milestone.plan_id.clone(),
        sequence_index: milestone.sequence_index,
        title: milestone.title.clone(),
        slug: milestone.slug.clone(),
        file_path: milestone.file_path.clone(),
        goal: milestone.goal.clone(),
        dependencies: milestone.dependencies.clone(),
        capabilities: milestone
            .capabilities
            .iter()
            .map(|c| JsonMilestoneCapability {
                capability_id: c.capability_id.clone(),
                title: c.title.clone(),
                description: c.description.clone(),
                citation: c.citation.format_citation(),
            })
            .collect(),
        technical_constraints: milestone
            .technical_constraints
            .iter()
            .map(|t| JsonTechnicalConstraint {
                constraint_id: t.constraint_id.clone(),
                title: t.title.clone(),
                description: t.description.clone(),
                citation: t.citation.format_citation(),
            })
            .collect(),
        status: milestone.status.as_str().to_string(),
        created_at: milestone.created_at.clone(),
        updated_at: milestone.updated_at.clone(),
    };

    if json_mode || args.json {
        let steps = MilestonesPresenter::resolve_inspect_next_steps(&milestone);
        let resp = JsonMilestonesInspectResponse {
            verdict: "PASS".to_string(),
            status: "MILESTONE_FOUND".to_string(),
            milestone: detail,
            next_steps: NextStepPresenter::to_json_vec(&steps),
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&resp).unwrap_or_default()
        );
    } else {
        println!("{}", MilestonesPresenter::render_inspect_text(&milestone));
    }

    Ok(DomainOutcome::Success)
}
