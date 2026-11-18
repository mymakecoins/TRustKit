// Dual-audience presenter for `trustkit update` and `tk update` (TASK-003).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::next_step_presenter::{JsonNextStep, NextStepPresenter};
use serde::Serialize;
use trustkit_core::entities::{NextStepResolver, SelfUpdateReport};

#[derive(Debug, Clone, Serialize)]
pub struct JsonUpdateResponse {
    pub verdict: String,
    pub status: String,
    pub strategy: String,
    pub check_only: bool,
    pub forced: bool,
    pub local_commit: String,
    pub remote_commit: String,
    pub target_branch: String,
    pub binaries_updated: Vec<String>,
    pub summary: String,
    pub next_steps: Vec<JsonNextStep>,
}

pub struct UpdatePresenter;

impl UpdatePresenter {
    pub fn render(
        report: &SelfUpdateReport,
        check_only: bool,
        forced: bool,
        json_mode: bool,
    ) -> String {
        let is_json = json_mode || std::env::var("TRUSTKIT_AGENT_MODE").as_deref() == Ok("1");
        let steps = NextStepResolver::for_update(check_only, report.status.is_update_available());
        let json_steps = NextStepPresenter::to_json_vec(&steps);

        if is_json {
            let payload = JsonUpdateResponse {
                verdict: "PASS".to_string(),
                status: report.status.as_str().to_string(),
                strategy: report.strategy.as_str().to_string(),
                check_only,
                forced,
                local_commit: report.local_commit.clone(),
                remote_commit: report.remote_commit.clone(),
                target_branch: report.target_branch.clone(),
                binaries_updated: report.binaries_updated.clone(),
                summary: report.summary.clone(),
                next_steps: json_steps,
            };
            serde_json::to_string_pretty(&payload)
                .unwrap_or_else(|_| "{\"verdict\":\"FAIL\"}".to_string())
        } else {
            let mut out = String::new();
            out.push_str(&format!(
                "[INFO] Strategy: {} | Branch: origin/{}\n",
                report.strategy.as_str(),
                report.target_branch
            ));
            out.push_str(&format!(
                "[UPDATE] Status: {} | Local: {} | Remote: {}\n",
                report.status.as_str(),
                report.local_commit,
                report.remote_commit
            ));
            if !report.binaries_updated.is_empty() {
                out.push_str(&format!(
                    "[PASS] Updated binaries: {}\n",
                    report.binaries_updated.join(", ")
                ));
            } else {
                out.push_str(&format!("[PASS] {}\n", report.summary));
            }
            out.push_str(&NextStepPresenter::render_json_steps_ascii_block(
                &json_steps,
            ));
            out
        }
    }
}
