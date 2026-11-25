// Handler for `trustkit features plan`, `list`, and `inspect`.
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::cli_contract::{
    FeaturesArgs, FeaturesInspectArgs, FeaturesListArgs, FeaturesPlanArgs, FeaturesSubcommand,
};
use crate::error_handler::DomainOutcome;
use crate::features_presenter::{
    FeaturesPresenter, JsonFeatureDetail, JsonFeatureSummary, JsonFeaturesInspectResponse,
    JsonFeaturesListResponse, JsonFeaturesPlanResponse, JsonTaskBlueprint,
};
use crate::next_step_presenter::NextStepPresenter;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use trustkit_ast::MilestoneFeatureExtractor;
use trustkit_core::entities::PlannedFeature;
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::{FeatureRepositoryPort, MilestoneRepositoryPort};
use trustkit_core::use_cases::plan_features::{self, PlanFeaturesRequest};
use trustkit_db::sqlite_adapter::SqliteWalRepositoryAdapter;

fn current_timestamp_iso() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{now}")
}

fn to_summary(f: &PlannedFeature) -> JsonFeatureSummary {
    JsonFeatureSummary {
        feature_id: f.feature_id.clone(),
        milestone_id: f.milestone_id.clone(),
        title: f.title.clone(),
        slug: f.slug.clone(),
        status: f.status.as_str().to_string(),
        dependencies: f.dependencies.clone(),
        tasks_count: f.task_blueprints.len(),
        spec_file_path: f.spec_file_path.clone(),
        linked_cycle_id: f.linked_cycle_id.clone(),
    }
}

pub fn execute(args: &FeaturesArgs, json_mode: bool) -> Result<DomainOutcome, DomainViolation> {
    let db_path = Path::new(".trustkit").join("state.db");
    execute_with_db(args, json_mode, &db_path)
}

pub fn execute_with_db(
    args: &FeaturesArgs,
    json_mode: bool,
    db_path: &Path,
) -> Result<DomainOutcome, DomainViolation> {
    match &args.action {
        FeaturesSubcommand::Plan(plan_args) => execute_plan_with_db(plan_args, json_mode, db_path),
        FeaturesSubcommand::List(list_args) => execute_list_with_db(list_args, json_mode, db_path),
        FeaturesSubcommand::Inspect(inspect_args) => {
            execute_inspect_with_db(inspect_args, json_mode, db_path)
        }
    }
}

pub fn resolve_or_ingest_milestone_id(
    repo: &SqliteWalRepositoryAdapter,
    milestone_ref: &str,
    now_iso: &str,
) -> Result<String, DomainViolation> {
    if repo.find_milestone_by_id(milestone_ref)?.is_some() {
        return Ok(milestone_ref.to_string());
    }
    let path = Path::new(milestone_ref);
    if path.exists() && path.is_file() {
        let content =
            fs::read_to_string(path).map_err(|e| DomainViolation::InfrastructureBoundaryFault {
                subsystem: "features_plan".to_string(),
                details: format!("Failed reading milestone file '{}': {e}", milestone_ref),
            })?;
        let parsed =
            MilestoneFeatureExtractor::parse_milestone_markdown(&content, milestone_ref, now_iso)?;
        let m_id = parsed.milestone_id.clone();
        repo.save_milestone(&parsed)?;
        return Ok(m_id);
    }
    Err(DomainViolation::InfrastructureBoundaryFault {
        subsystem: "features_plan".to_string(),
        details: format!(
            "Milestone '{}' not found in SQLite WAL or on disk",
            milestone_ref
        ),
    })
}

pub fn execute_plan_with_db(
    args: &FeaturesPlanArgs,
    json_mode: bool,
    db_path: &Path,
) -> Result<DomainOutcome, DomainViolation> {
    if let Some(parent) = db_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let repo = SqliteWalRepositoryAdapter::open(db_path)?;
    let now_iso = current_timestamp_iso();
    let milestone_id = resolve_or_ingest_milestone_id(&repo, &args.milestone, &now_iso)?;
    let extractor = MilestoneFeatureExtractor;

    let req = PlanFeaturesRequest {
        milestone_ref: milestone_id,
        output_dir: args.output_dir.clone(),
        now_iso,
    };

    let outcome = plan_features::execute(req, &repo, &extractor, Some(&repo))?;
    let summaries: Vec<JsonFeatureSummary> = outcome.plan.features.iter().map(to_summary).collect();
    let generated_files: Vec<String> = outcome
        .generated_artifacts
        .iter()
        .map(|a| a.file_path.clone())
        .collect();

    if json_mode || args.json {
        let steps = FeaturesPresenter::resolve_plan_next_steps(
            &outcome.plan.features,
            &outcome.plan.milestone_id,
        );
        let resp = JsonFeaturesPlanResponse {
            verdict: "PASS".to_string(),
            status: "FEATURES_PLANNED".to_string(),
            milestone_id: outcome.plan.milestone_id.clone(),
            total_features: outcome.plan.features.len(),
            total_tasks: outcome.plan.total_tasks(),
            generated_files,
            features: summaries,
            next_steps: NextStepPresenter::to_json_vec(&steps),
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&resp).unwrap_or_default()
        );
    } else {
        println!("{}", FeaturesPresenter::render_plan_text(&outcome));
    }

    Ok(DomainOutcome::Success)
}

pub fn execute_list_with_db(
    args: &FeaturesListArgs,
    json_mode: bool,
    db_path: &Path,
) -> Result<DomainOutcome, DomainViolation> {
    if let Some(parent) = db_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let repo = SqliteWalRepositoryAdapter::open(db_path)?;
    let features = repo.list_features(args.milestone.as_deref())?;
    let summaries: Vec<JsonFeatureSummary> = features.iter().map(to_summary).collect();
    let m_id = args
        .milestone
        .as_deref()
        .or_else(|| features.first().map(|f| f.milestone_id.as_str()))
        .unwrap_or("M01");

    if json_mode || args.json {
        let steps = FeaturesPresenter::resolve_plan_next_steps(&features, m_id);
        let resp = JsonFeaturesListResponse {
            verdict: "PASS".to_string(),
            status: "FEATURES_LISTED".to_string(),
            total_features: features.len(),
            features: summaries,
            next_steps: NextStepPresenter::to_json_vec(&steps),
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&resp).unwrap_or_default()
        );
    } else {
        println!("{}", FeaturesPresenter::render_list_text(&features));
    }

    Ok(DomainOutcome::Success)
}

pub fn execute_inspect_with_db(
    args: &FeaturesInspectArgs,
    json_mode: bool,
    db_path: &Path,
) -> Result<DomainOutcome, DomainViolation> {
    if let Some(parent) = db_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let repo = SqliteWalRepositoryAdapter::open(db_path)?;
    let feature = repo.find_feature_by_id(&args.feature_id)?.ok_or_else(|| {
        DomainViolation::InfrastructureBoundaryFault {
            subsystem: "features_inspect".to_string(),
            details: format!("Feature '{}' not found in database", args.feature_id),
        }
    })?;

    if json_mode || args.json {
        let detail = JsonFeatureDetail {
            feature_id: feature.feature_id.clone(),
            milestone_id: feature.milestone_id.clone(),
            sequence_index: feature.sequence_index,
            title: feature.title.clone(),
            slug: feature.slug.clone(),
            spec_dir: feature.spec_dir.clone(),
            spec_file_path: feature.spec_file_path.clone(),
            goal: feature.goal.clone(),
            dependencies: feature.dependencies.clone(),
            task_blueprints: feature
                .task_blueprints
                .iter()
                .map(|bp| JsonTaskBlueprint {
                    task_id: bp.task_id.clone(),
                    layer: bp.layer.as_str().to_string(),
                    title: bp.title.clone(),
                    business_rule_id: bp.business_rule_id.clone(),
                    functional_req_id: bp.functional_req_id.clone(),
                    target_test_file: bp.target_test_file.clone(),
                    depends_on: bp.depends_on.clone(),
                })
                .collect(),
            status: feature.status.as_str().to_string(),
            linked_cycle_id: feature.linked_cycle_id.clone(),
            created_at: feature.created_at.clone(),
            updated_at: feature.updated_at.clone(),
        };
        let steps = FeaturesPresenter::resolve_inspect_next_steps(&feature);
        let resp = JsonFeaturesInspectResponse {
            verdict: "PASS".to_string(),
            status: "FEATURE_INSPECTED".to_string(),
            feature: detail,
            next_steps: NextStepPresenter::to_json_vec(&steps),
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&resp).unwrap_or_default()
        );
    } else {
        println!("{}", FeaturesPresenter::render_inspect_text(&feature));
    }

    Ok(DomainOutcome::Success)
}
