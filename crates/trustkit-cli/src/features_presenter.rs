// Presenter and JSON response serialization for feature operations.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::next_step_presenter::{JsonNextStep, NextStepPresenter};
use serde::Serialize;
use trustkit_core::entities::{NextStepRecommendation, NextStepResolver, PlannedFeature};
use trustkit_core::use_cases::plan_features::PlanFeaturesOutcome;

#[derive(Debug, Clone, Serialize)]
pub struct JsonFeatureSummary {
    pub feature_id: String,
    pub milestone_id: String,
    pub title: String,
    pub slug: String,
    pub status: String,
    pub dependencies: Vec<String>,
    pub tasks_count: usize,
    pub spec_file_path: String,
    pub linked_cycle_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonFeaturesPlanResponse {
    pub verdict: String,
    pub status: String,
    pub milestone_id: String,
    pub total_features: usize,
    pub total_tasks: usize,
    pub generated_files: Vec<String>,
    pub features: Vec<JsonFeatureSummary>,
    pub next_steps: Vec<JsonNextStep>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonFeaturesListResponse {
    pub verdict: String,
    pub status: String,
    pub total_features: usize,
    pub features: Vec<JsonFeatureSummary>,
    pub next_steps: Vec<JsonNextStep>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonTaskBlueprint {
    pub task_id: String,
    pub layer: String,
    pub title: String,
    pub business_rule_id: String,
    pub functional_req_id: String,
    pub target_test_file: String,
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonFeatureDetail {
    pub feature_id: String,
    pub milestone_id: String,
    pub sequence_index: usize,
    pub title: String,
    pub slug: String,
    pub spec_dir: String,
    pub spec_file_path: String,
    pub goal: String,
    pub dependencies: Vec<String>,
    pub task_blueprints: Vec<JsonTaskBlueprint>,
    pub status: String,
    pub linked_cycle_id: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonFeaturesInspectResponse {
    pub verdict: String,
    pub status: String,
    pub feature: JsonFeatureDetail,
    pub next_steps: Vec<JsonNextStep>,
}

pub struct FeaturesPresenter;

impl FeaturesPresenter {
    pub fn resolve_plan_next_steps(
        features: &[PlannedFeature],
        milestone_id: &str,
    ) -> Vec<NextStepRecommendation> {
        let first_id = features.first().map(|f| f.feature_id.as_str());
        NextStepResolver::for_features_plan(first_id, milestone_id)
    }

    pub fn resolve_inspect_next_steps(f: &PlannedFeature) -> Vec<NextStepRecommendation> {
        NextStepResolver::for_feature_inspect(&f.feature_id, &f.spec_file_path)
    }

    pub fn render_plan_text(outcome: &PlanFeaturesOutcome) -> String {
        let steps =
            Self::resolve_plan_next_steps(&outcome.plan.features, &outcome.plan.milestone_id);
        let mut out = String::new();
        out.push_str("[PASS] Features decomposed and V2 specifications synthesized\n");
        out.push_str("------------------------------------------------------------\n");
        out.push_str(&format!(
            "Milestone ID       : {}\n",
            outcome.plan.milestone_id
        ));
        out.push_str(&format!(
            "Milestone Title    : {}\n",
            outcome.plan.milestone_title
        ));
        out.push_str(&format!(
            "Total Features     : {}\n",
            outcome.plan.features.len()
        ));
        out.push_str(&format!(
            "Total Layered Tasks: {}\n",
            outcome.plan.total_tasks()
        ));
        for f in &outcome.plan.features {
            out.push_str(&format!(
                "- {} [{}] {} ({})\n",
                f.feature_id,
                f.status.as_str(),
                f.title,
                f.spec_file_path
            ));
        }
        out.push_str(&NextStepPresenter::render_ascii_block(&steps));
        out
    }

    pub fn render_list_text(features: &[PlannedFeature]) -> String {
        let m_id = features
            .first()
            .map(|f| f.milestone_id.as_str())
            .unwrap_or("M01");
        let steps = Self::resolve_plan_next_steps(features, m_id);
        let mut out = String::new();
        out.push_str(&format!(
            "[PASS] Features List ({} items)\n",
            features.len()
        ));
        out.push_str(
            "--------------------------------------------------------------------------------\n",
        );
        out.push_str("| ID      | Milestone | Status     | Tasks | Spec Path\n");
        out.push_str(
            "--------------------------------------------------------------------------------\n",
        );
        for f in features {
            out.push_str(&format!(
                "| {:<7} | {:<9} | {:<10} | {:<5} | {}\n",
                f.feature_id,
                f.milestone_id,
                f.status.as_str(),
                f.task_blueprints.len(),
                f.spec_file_path
            ));
        }
        out.push_str(&NextStepPresenter::render_ascii_block(&steps));
        out
    }

    pub fn render_inspect_text(f: &PlannedFeature) -> String {
        let steps = Self::resolve_inspect_next_steps(f);
        let mut out = String::new();
        out.push_str(&format!("[PASS] Feature: {} ({})\n", f.feature_id, f.title));
        out.push_str("------------------------------------------------------------\n");
        out.push_str(&format!("Milestone   : {}\n", f.milestone_id));
        out.push_str(&format!("Status      : {}\n", f.status.as_str()));
        out.push_str(&format!("Spec File   : {}\n", f.spec_file_path));
        out.push_str(&format!("Cycle ID    : {}\n", f.linked_cycle_id));
        out.push_str(&format!("Goal        : {}\n", f.goal));
        out.push_str("\nLayered Task Blueprints:\n");
        for bp in &f.task_blueprints {
            out.push_str(&format!(
                "  - {} [{}] {} -> {}\n",
                bp.task_id,
                bp.layer.as_str(),
                bp.title,
                bp.target_test_file
            ));
        }
        out.push_str(&NextStepPresenter::render_ascii_block(&steps));
        out
    }
}
