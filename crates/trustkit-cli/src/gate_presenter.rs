// Dual-mode contracts and presenters for Gates 1, 4, 5, and 6 CLI commands.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::next_step_presenter::{JsonNextStep, NextStepPresenter};
use serde::{Deserialize, Serialize};
use trustkit_core::entities::{NextStepResolver, TddPhase};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorktreeProvisionInfo {
    pub task_id: String,
    pub branch_name: String,
    pub worktree_path: String,
    pub base_branch: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonDesignItTwiceBrief {
    pub brief_id: String,
    pub strategy_code: String,
    pub strategy_name: String,
    pub design_constraint: String,
    pub target_seam_focus: String,
    pub subagent_prompt: String,
    pub expected_tradeoffs: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonSpecPromptResponse {
    pub verdict: String,
    pub idea: String,
    pub token_estimate: usize,
    pub system_instructions: String,
    pub scaffold_markdown: String,
    pub context_summary: String,
    #[serde(default)]
    pub design_it_twice: bool,
    #[serde(default)]
    pub design_it_twice_briefs: Vec<JsonDesignItTwiceBrief>,
    #[serde(default)]
    pub next_steps: Vec<JsonNextStep>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonSpecLintViolation {
    pub code: String,
    pub message: String,
    pub line: usize,
    pub remediation: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonSpecLintResponse {
    pub verdict: String,
    pub spec_file: String,
    pub violations_count: usize,
    pub violations: Vec<JsonSpecLintViolation>,
    #[serde(default)]
    pub next_steps: Vec<JsonNextStep>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonTaskTestResponse {
    pub verdict: String,
    pub task_id: String,
    pub phase: String,
    pub exit_code: i32,
    pub duration_ms: u64,
    pub evidence_id: u64,
    pub status: String,
    #[serde(default)]
    pub next_steps: Vec<JsonNextStep>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonTaskCompleteResponse {
    pub verdict: String,
    pub task_id: String,
    pub new_state: String,
    pub timestamp: String,
    #[serde(default)]
    pub next_steps: Vec<JsonNextStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonWaveRunResponse {
    pub verdict: String,
    pub wave_index: u32,
    pub total_tasks: usize,
    pub successful_tasks: Vec<String>,
    pub failed_tasks: Vec<String>,
    pub duration_ms: u64,
    #[serde(default)]
    pub runtime_used: String,
    #[serde(default)]
    pub provisioned_worktrees: Vec<WorktreeProvisionInfo>,
    #[serde(default)]
    pub next_steps: Vec<JsonNextStep>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonGauntletRunResponse {
    pub verdict: String,
    pub layer0_status: String,
    pub layer1_domain_status: String,
    pub layer1_contract_status: String,
    pub layer1_build_status: String,
    pub layer1_e2e_status: String,
    pub critics_passed: usize,
    pub critics_total: usize,
    pub circuit_breaker_tripped: bool,
    #[serde(default)]
    pub next_steps: Vec<JsonNextStep>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonDoctorResponse {
    pub verdict: String,
    pub circuit_breaker_tripped: bool,
    pub consecutive_failures: u8,
    #[serde(default)]
    pub sccache_detected: bool,
    #[serde(default)]
    pub compilation_cache_status: String,
    #[serde(default)]
    pub batch_execution_enabled: bool,
    pub diagnostics: Vec<String>,
    pub remediation: String,
    #[serde(default)]
    pub next_steps: Vec<JsonNextStep>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonDoneResponse {
    pub verdict: String,
    pub cycle_id: String,
    pub gate: String,
    pub report_file: String,
    pub issue_comment: String,
    #[serde(default)]
    pub next_steps: Vec<JsonNextStep>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonCloseResponse {
    pub verdict: String,
    pub cycle_id: String,
    pub board_column: String,
    #[serde(default)]
    pub next_steps: Vec<JsonNextStep>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonMergeResponse {
    pub verdict: String,
    pub cycle_id: String,
    pub target_branch: String,
    pub commit_hash: String,
    pub board_column: String,
    #[serde(default)]
    pub next_steps: Vec<JsonNextStep>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonPrResponse {
    pub verdict: String,
    pub cycle_id: String,
    pub pr_url: String,
    pub pr_number: u64,
    pub target_branch: String,
    pub board_column: String,
    #[serde(default)]
    pub next_steps: Vec<JsonNextStep>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonMcpServeResponse {
    pub status: String,
    pub transport: String,
    pub server_name: String,
    pub version: String,
}

pub struct GatePresenter;

impl GatePresenter {
    fn ensure_steps(
        existing: &[JsonNextStep],
        fallback: Vec<trustkit_core::entities::NextStepRecommendation>,
    ) -> Vec<JsonNextStep> {
        if existing.is_empty() {
            NextStepPresenter::to_json_vec(&fallback)
        } else {
            existing.to_vec()
        }
    }

    pub fn render_task_test(res: &JsonTaskTestResponse, json_mode: bool) -> String {
        let phase = if res.phase.eq_ignore_ascii_case("green") {
            TddPhase::Green
        } else {
            TddPhase::Red
        };
        let steps = Self::ensure_steps(
            &res.next_steps,
            NextStepResolver::for_task_test(&res.task_id, phase),
        );
        if json_mode {
            let mut payload = res.clone();
            payload.next_steps = steps;
            serde_json::to_string(&payload).unwrap_or_else(|_| "{\"verdict\":\"FAIL\"}".to_string())
        } else {
            let mut out = format!(
                "[{}] Task {} test --phase {} finished with exit code {} in {}ms (evidence: #{})",
                res.verdict,
                res.task_id,
                res.phase,
                res.exit_code,
                res.duration_ms,
                res.evidence_id
            );
            out.push_str(&NextStepPresenter::render_json_steps_ascii_block(&steps));
            out
        }
    }

    pub fn render_task_complete(res: &JsonTaskCompleteResponse, json_mode: bool) -> String {
        let steps = Self::ensure_steps(
            &res.next_steps,
            NextStepResolver::for_task_complete(None, None),
        );
        if json_mode {
            let mut payload = res.clone();
            payload.next_steps = steps;
            serde_json::to_string(&payload).unwrap_or_else(|_| "{\"verdict\":\"FAIL\"}".to_string())
        } else {
            let mut out = format!(
                "[{}] Task {} completed -> {}",
                res.verdict, res.task_id, res.new_state
            );
            out.push_str(&NextStepPresenter::render_json_steps_ascii_block(&steps));
            out
        }
    }

    pub fn render_worktree_provision_notification(
        branch_name: &str,
        worktree_path: &str,
        base_branch: &str,
    ) -> String {
        format!(
            "[INFO] Branch '{branch_name}' criada com sucesso.\n[INFO] Checkout realizado no diretorio isolado: {worktree_path}\n[INFO] Branch ativa do repositorio raiz preservada: '{base_branch}'"
        )
    }

    pub fn render_wave_run(res: &JsonWaveRunResponse, json_mode: bool) -> String {
        let first_pending = res.successful_tasks.first().map(String::as_str);
        let steps = Self::ensure_steps(
            &res.next_steps,
            NextStepResolver::for_wave_run(res.wave_index, &res.failed_tasks, None, first_pending),
        );
        if json_mode {
            let mut payload = res.clone();
            payload.next_steps = steps;
            if payload.provisioned_worktrees.is_empty() && !payload.successful_tasks.is_empty() {
                payload.provisioned_worktrees = payload
                    .successful_tasks
                    .iter()
                    .map(|id| WorktreeProvisionInfo {
                        task_id: id.clone(),
                        branch_name: format!("worktree/{id}"),
                        worktree_path: format!(".trustkit/worktrees/{id}"),
                        base_branch: "develop".to_string(),
                    })
                    .collect();
            }
            serde_json::to_string(&payload).unwrap_or_else(|_| "{\"verdict\":\"FAIL\"}".to_string())
        } else {
            let mut out = String::new();
            let wts = if res.provisioned_worktrees.is_empty() {
                res.successful_tasks
                    .iter()
                    .map(|id| WorktreeProvisionInfo {
                        task_id: id.clone(),
                        branch_name: format!("worktree/{id}"),
                        worktree_path: format!(".trustkit/worktrees/{id}"),
                        base_branch: "develop".to_string(),
                    })
                    .collect()
            } else {
                res.provisioned_worktrees.clone()
            };
            for wt in &wts {
                out.push_str(&Self::render_worktree_provision_notification(
                    &wt.branch_name,
                    &wt.worktree_path,
                    &wt.base_branch,
                ));
                out.push('\n');
            }
            out.push_str(&format!(
                "[{}] Wave {} executed: {}/{} tasks passed in {}ms",
                res.verdict,
                res.wave_index,
                res.successful_tasks.len(),
                res.total_tasks,
                res.duration_ms
            ));
            out.push_str(&NextStepPresenter::render_json_steps_ascii_block(&steps));
            out
        }
    }

    pub fn render_gauntlet_run(res: &JsonGauntletRunResponse, json_mode: bool) -> String {
        let steps = Self::ensure_steps(
            &res.next_steps,
            NextStepResolver::for_gauntlet_run(res.circuit_breaker_tripped),
        );
        if json_mode {
            let mut payload = res.clone();
            payload.next_steps = steps;
            serde_json::to_string(&payload).unwrap_or_else(|_| "{\"verdict\":\"FAIL\"}".to_string())
        } else {
            let mut out = format!(
                "[{}] Gauntlet execution: {}/{} critics passed (Layer 0: {}, Domain: {}, Contract: {}, Build: {}, E2E: {})",
                res.verdict,
                res.critics_passed,
                res.critics_total,
                res.layer0_status,
                res.layer1_domain_status,
                res.layer1_contract_status,
                res.layer1_build_status,
                res.layer1_e2e_status,
            );
            out.push_str(&NextStepPresenter::render_json_steps_ascii_block(&steps));
            out
        }
    }

    pub fn render_doctor(res: &JsonDoctorResponse, json_mode: bool) -> String {
        let steps = Self::ensure_steps(
            &res.next_steps,
            NextStepResolver::for_doctor(res.circuit_breaker_tripped),
        );
        if json_mode {
            let mut payload = res.clone();
            payload.next_steps = steps;
            serde_json::to_string(&payload).unwrap_or_else(|_| "{\"verdict\":\"FAIL\"}".to_string())
        } else {
            let cache_status = if res.compilation_cache_status.is_empty() {
                "AUTO"
            } else {
                res.compilation_cache_status.as_str()
            };
            let mut out = format!(
                "[{}] Doctor Diagnostics: Circuit Breaker Tripped: {}, Consecutive Failures: {}\nCompilation Cache: {} (sccache_detected: {}, batching_enabled: {})\nRemediation: {}",
                res.verdict,
                res.circuit_breaker_tripped,
                res.consecutive_failures,
                cache_status,
                res.sccache_detected,
                res.batch_execution_enabled,
                res.remediation
            );
            out.push_str(&NextStepPresenter::render_json_steps_ascii_block(&steps));
            out
        }
    }

    pub fn render_done(res: &JsonDoneResponse, json_mode: bool) -> String {
        let steps = Self::ensure_steps(&res.next_steps, NextStepResolver::for_done("develop"));
        if json_mode {
            let mut payload = res.clone();
            payload.next_steps = steps;
            serde_json::to_string(&payload).unwrap_or_else(|_| "{\"verdict\":\"FAIL\"}".to_string())
        } else {
            let mut out = format!(
                "[{}] Gate 6 Cycle Certified: {}\nReport: {}\nComment:\n{}",
                res.verdict, res.cycle_id, res.report_file, res.issue_comment
            );
            out.push_str(&NextStepPresenter::render_json_steps_ascii_block(&steps));
            out
        }
    }

    pub fn render_close(res: &JsonCloseResponse, json_mode: bool) -> String {
        let steps = Self::ensure_steps(&res.next_steps, NextStepResolver::for_close("develop"));
        if json_mode {
            let mut payload = res.clone();
            payload.next_steps = steps;
            serde_json::to_string(&payload).unwrap_or_else(|_| "{\"verdict\":\"FAIL\"}".to_string())
        } else {
            let mut out = format!(
                "[{}] SDD cycle governance closed for cycle {}\nBoard column: {}",
                res.verdict, res.cycle_id, res.board_column
            );
            out.push_str(&NextStepPresenter::render_json_steps_ascii_block(&steps));
            out
        }
    }

    pub fn render_merge(res: &JsonMergeResponse, json_mode: bool) -> String {
        let steps = Self::ensure_steps(
            &res.next_steps,
            NextStepResolver::for_merge(&res.target_branch),
        );
        if json_mode {
            let mut payload = res.clone();
            payload.next_steps = steps;
            serde_json::to_string(&payload).unwrap_or_else(|_| "{\"verdict\":\"FAIL\"}".to_string())
        } else {
            let mut out = format!(
                "[{}] Deterministic merge completed for cycle {} into branch {} ({})\nBoard column: {}",
                res.verdict, res.cycle_id, res.target_branch, res.commit_hash, res.board_column
            );
            out.push_str(&NextStepPresenter::render_json_steps_ascii_block(&steps));
            out
        }
    }

    pub fn render_pr(res: &JsonPrResponse, json_mode: bool) -> String {
        let steps = Self::ensure_steps(&res.next_steps, NextStepResolver::for_pr());
        if json_mode {
            let mut payload = res.clone();
            payload.next_steps = steps;
            serde_json::to_string(&payload).unwrap_or_else(|_| "{\"verdict\":\"FAIL\"}".to_string())
        } else {
            let mut out = format!(
                "[{}] Pull Request #{} opened for cycle {} -> {}\nURL: {}\nBoard column: {}",
                res.verdict,
                res.pr_number,
                res.cycle_id,
                res.target_branch,
                res.pr_url,
                res.board_column
            );
            out.push_str(&NextStepPresenter::render_json_steps_ascii_block(&steps));
            out
        }
    }

    pub fn render_spec_prompt(res: &JsonSpecPromptResponse, json_mode: bool) -> String {
        let steps = Self::ensure_steps(&res.next_steps, NextStepResolver::for_spec_prompt(""));
        if json_mode {
            let mut payload = res.clone();
            payload.next_steps = steps;
            serde_json::to_string(&payload).unwrap_or_else(|_| "{\"verdict\":\"FAIL\"}".to_string())
        } else {
            let mut out = format!(
                "[{}] Spec Prompt Generated (~{} tokens)\n\n--- INSTRUCOES CONSTITUCIONAIS ---\n{}\n\n--- CONTEXTO ---\n{}\n\n--- ESQUELETO SPEC.MD ---\n{}",
                res.verdict,
                res.token_estimate,
                res.system_instructions,
                res.context_summary,
                res.scaffold_markdown
            );
            out.push_str(&NextStepPresenter::render_json_steps_ascii_block(&steps));
            out
        }
    }

    pub fn render_spec_lint(res: &JsonSpecLintResponse, json_mode: bool) -> String {
        let steps = Self::ensure_steps(
            &res.next_steps,
            NextStepResolver::for_spec_lint(&res.spec_file, res.violations.is_empty()),
        );
        if json_mode {
            let mut payload = res.clone();
            payload.next_steps = steps;
            serde_json::to_string(&payload).unwrap_or_else(|_| "{\"verdict\":\"FAIL\"}".to_string())
        } else if res.violations.is_empty() {
            let mut out = format!(
                "[PASS] Specification '{}' is compliant with all structural and constitutional rules.",
                res.spec_file
            );
            out.push_str(&NextStepPresenter::render_json_steps_ascii_block(&steps));
            out
        } else {
            let mut out = format!(
                "[FAIL] Specification '{}' has {} violations:\n",
                res.spec_file, res.violations_count
            );
            for v in &res.violations {
                out.push_str(&format!(
                    "  Line {}: [{}] {} (Remediacao: {})\n",
                    v.line, v.code, v.message, v.remediation
                ));
            }
            out.push_str(&NextStepPresenter::render_json_steps_ascii_block(&steps));
            out
        }
    }

    pub fn render_mcp_serve(res: &JsonMcpServeResponse, json_mode: bool) -> String {
        if json_mode {
            serde_json::to_string(res).unwrap_or_else(|_| "{\"status\":\"FAIL\"}".to_string())
        } else {
            format!(
                "[PASS] MCP Server {} v{} listening on {}",
                res.server_name, res.version, res.transport
            )
        }
    }
}

#[cfg(test)]
#[path = "gate_presenter_tests.rs"]
mod tests;
