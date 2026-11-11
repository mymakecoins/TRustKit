// Presenter and JSON response serialization for milestone operations.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::next_step_presenter::{JsonNextStep, NextStepPresenter};
use serde::Serialize;
use trustkit_core::entities::{
    Milestone, NextStepPriority, NextStepRecommendation, NextStepResolver,
};
use trustkit_core::use_cases::plan_milestones::PlanMilestonesOutcome;

#[derive(Debug, Clone, Serialize)]
pub struct JsonMilestoneSummary {
    pub milestone_id: String,
    pub title: String,
    pub slug: String,
    pub status: String,
    pub dependencies: Vec<String>,
    pub capabilities_count: usize,
    pub technical_constraints_count: usize,
    pub file_path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonMilestonesPlanResponse {
    pub verdict: String,
    pub status: String,
    pub plan_id: String,
    pub total_milestones: usize,
    pub total_capabilities: usize,
    pub generated_files: Vec<String>,
    pub milestones: Vec<JsonMilestoneSummary>,
    pub next_steps: Vec<JsonNextStep>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonMilestonesListResponse {
    pub verdict: String,
    pub status: String,
    pub total_milestones: usize,
    pub milestones: Vec<JsonMilestoneSummary>,
    pub next_steps: Vec<JsonNextStep>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonMilestoneCapability {
    pub capability_id: String,
    pub title: String,
    pub description: String,
    pub citation: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonTechnicalConstraint {
    pub constraint_id: String,
    pub title: String,
    pub description: String,
    pub citation: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonMilestoneDetail {
    pub milestone_id: String,
    pub plan_id: String,
    pub sequence_index: usize,
    pub title: String,
    pub slug: String,
    pub file_path: String,
    pub goal: String,
    pub dependencies: Vec<String>,
    pub capabilities: Vec<JsonMilestoneCapability>,
    pub technical_constraints: Vec<JsonTechnicalConstraint>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonMilestonesInspectResponse {
    pub verdict: String,
    pub status: String,
    pub milestone: JsonMilestoneDetail,
    pub next_steps: Vec<JsonNextStep>,
}

pub struct MilestonesPresenter;

impl MilestonesPresenter {
    pub fn resolve_plan_next_steps(milestones: &[Milestone]) -> Vec<NextStepRecommendation> {
        let first_file = milestones.first().map(|m| m.file_path.as_str());
        let first_id = milestones.first().map(|m| m.milestone_id.as_str());
        NextStepResolver::for_milestones_plan(first_file, first_id)
    }

    pub fn resolve_inspect_next_steps(m: &Milestone) -> Vec<NextStepRecommendation> {
        vec![NextStepRecommendation::new(
            NextStepPriority::Primary,
            "GATE-0",
            format!("trustkit init --milestone {}", m.file_path),
            format!("Initialize an SDD cycle from milestone {}.", m.milestone_id),
        )]
    }

    pub fn render_plan_text(outcome: &PlanMilestonesOutcome) -> String {
        let steps = Self::resolve_plan_next_steps(&outcome.plan.milestones);
        let mut out = String::new();
        out.push_str("[PASS] Milestones planned successfully\n");
        out.push_str("------------------------------------------------------------\n");
        out.push_str(&format!("Plan ID            : {}\n", outcome.plan.plan_id));
        out.push_str(&format!("PRD Path           : {}\n", outcome.plan.prd_path));
        out.push_str(&format!("ADR Path           : {}\n", outcome.plan.adr_path));
        out.push_str(&format!(
            "Total Milestones   : {}\n",
            outcome.plan.milestones.len()
        ));
        out.push_str(&format!(
            "Total Capabilities : {}\n",
            outcome.plan.total_capabilities()
        ));
        out.push_str(&format!(
            "Generated Files    : {}\n",
            outcome.generated_artifacts.len()
        ));
        for m in &outcome.plan.milestones {
            out.push_str(&format!(
                "- {} [{}] {}\n",
                m.milestone_id,
                m.status.as_str(),
                m.title
            ));
        }
        out.push_str(&NextStepPresenter::render_ascii_block(&steps));
        out
    }

    pub fn render_list_text(milestones: &[Milestone]) -> String {
        let steps = Self::resolve_plan_next_steps(milestones);
        let mut out = String::new();
        out.push_str(&format!(
            "[PASS] Milestones List ({} items)\n",
            milestones.len()
        ));
        out.push_str(
            "--------------------------------------------------------------------------------\n",
        );
        out.push_str("| ID  | Status     | Title                                | Dependencies\n");
        out.push_str(
            "--------------------------------------------------------------------------------\n",
        );
        for m in milestones {
            let deps = if m.dependencies.is_empty() {
                "None".to_string()
            } else {
                m.dependencies.join(", ")
            };
            out.push_str(&format!(
                "| {:<3} | {:<10} | {:<36} | {}\n",
                m.milestone_id,
                m.status.as_str(),
                m.title,
                deps
            ));
        }
        out.push_str(&NextStepPresenter::render_ascii_block(&steps));
        out
    }

    pub fn render_inspect_text(m: &Milestone) -> String {
        let steps = Self::resolve_inspect_next_steps(m);
        let mut out = String::new();
        out.push_str(&format!(
            "[PASS] Milestone: {} ({})\n",
            m.milestone_id, m.title
        ));
        out.push_str("------------------------------------------------------------\n");
        out.push_str(&format!("Status      : {}\n", m.status.as_str()));
        out.push_str(&format!("Slug        : {}\n", m.slug));
        out.push_str(&format!("File        : {}\n", m.file_path));
        out.push_str(&format!("Goal        : {}\n", m.goal));
        out.push_str(&format!(
            "Dependencies: {}\n",
            if m.dependencies.is_empty() {
                "None".to_string()
            } else {
                m.dependencies.join(", ")
            }
        ));
        out.push_str(&format!("Capabilities: {} items\n", m.capabilities.len()));
        for cap in &m.capabilities {
            out.push_str(&format!(
                "  - {} {}: {} ({})\n",
                cap.capability_id,
                cap.title,
                cap.description,
                cap.citation.format_citation()
            ));
        }
        out.push_str(&format!(
            "Constraints : {} items\n",
            m.technical_constraints.len()
        ));
        for tc in &m.technical_constraints {
            out.push_str(&format!(
                "  - {} {}: {} ({})\n",
                tc.constraint_id,
                tc.title,
                tc.description,
                tc.citation.format_citation()
            ));
        }
        out.push_str(&NextStepPresenter::render_ascii_block(&steps));
        out
    }
}
