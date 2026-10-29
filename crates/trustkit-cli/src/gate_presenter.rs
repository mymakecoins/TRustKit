// Dual-mode contracts and presenters for Gates 4, 5, and 6 CLI commands.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct JsonTaskTestResponse {
    pub verdict: String,
    pub task_id: String,
    pub phase: String,
    pub exit_code: i32,
    pub duration_ms: u64,
    pub evidence_id: u64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonTaskCompleteResponse {
    pub verdict: String,
    pub task_id: String,
    pub new_state: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonWaveRunResponse {
    pub verdict: String,
    pub wave_index: u32,
    pub total_tasks: usize,
    pub successful_tasks: Vec<String>,
    pub failed_tasks: Vec<String>,
    pub duration_ms: u64,
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
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonDoctorResponse {
    pub verdict: String,
    pub circuit_breaker_tripped: bool,
    pub consecutive_failures: u8,
    pub diagnostics: Vec<String>,
    pub remediation: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonDoneResponse {
    pub verdict: String,
    pub cycle_id: String,
    pub gate: String,
    pub report_file: String,
    pub issue_comment: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonMergeResponse {
    pub verdict: String,
    pub cycle_id: String,
    pub target_branch: String,
    pub commit_hash: String,
    pub board_column: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonPrResponse {
    pub verdict: String,
    pub cycle_id: String,
    pub pr_url: String,
    pub pr_number: u64,
    pub target_branch: String,
    pub board_column: String,
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
    pub fn render_task_test(res: &JsonTaskTestResponse, json_mode: bool) -> String {
        if json_mode {
            serde_json::to_string(res).unwrap_or_else(|_| "{\"verdict\":\"FAIL\"}".to_string())
        } else {
            format!(
                "[{}] Task {} test --phase {} finished with exit code {} in {}ms (evidence: #{})",
                res.verdict, res.task_id, res.phase, res.exit_code, res.duration_ms, res.evidence_id
            )
        }
    }

    pub fn render_task_complete(res: &JsonTaskCompleteResponse, json_mode: bool) -> String {
        if json_mode {
            serde_json::to_string(res).unwrap_or_else(|_| "{\"verdict\":\"FAIL\"}".to_string())
        } else {
            format!("[{}] Task {} completed -> {}", res.verdict, res.task_id, res.new_state)
        }
    }

    pub fn render_wave_run(res: &JsonWaveRunResponse, json_mode: bool) -> String {
        if json_mode {
            serde_json::to_string(res).unwrap_or_else(|_| "{\"verdict\":\"FAIL\"}".to_string())
        } else {
            format!(
                "[{}] Wave {} executed: {}/{} tasks passed in {}ms",
                res.verdict, res.wave_index, res.successful_tasks.len(), res.total_tasks, res.duration_ms
            )
        }
    }

    pub fn render_gauntlet_run(res: &JsonGauntletRunResponse, json_mode: bool) -> String {
        if json_mode {
            serde_json::to_string(res).unwrap_or_else(|_| "{\"verdict\":\"FAIL\"}".to_string())
        } else {
            format!(
                "[{}] Gauntlet execution: {}/{} critics passed (Layer 0: {}, Domain: {}, Contract: {}, Build: {}, E2E: {})",
                res.verdict,
                res.critics_passed,
                res.critics_total,
                res.layer0_status,
                res.layer1_domain_status,
                res.layer1_contract_status,
                res.layer1_build_status,
                res.layer1_e2e_status,
            )
        }
    }

    pub fn render_doctor(res: &JsonDoctorResponse, json_mode: bool) -> String {
        if json_mode {
            serde_json::to_string(res).unwrap_or_else(|_| "{\"verdict\":\"FAIL\"}".to_string())
        } else {
            format!(
                "[{}] Doctor Diagnostics: Circuit Breaker Tripped: {}, Consecutive Failures: {}\nRemediation: {}",
                res.verdict, res.circuit_breaker_tripped, res.consecutive_failures, res.remediation
            )
        }
    }

    pub fn render_done(res: &JsonDoneResponse, json_mode: bool) -> String {
        if json_mode {
            serde_json::to_string(res).unwrap_or_else(|_| "{\"verdict\":\"FAIL\"}".to_string())
        } else {
            format!(
                "[{}] Gate 6 Cycle Certified: {}\nReport: {}\nComment:\n{}",
                res.verdict, res.cycle_id, res.report_file, res.issue_comment
            )
        }
    }

    pub fn render_merge(res: &JsonMergeResponse, json_mode: bool) -> String {
        if json_mode {
            serde_json::to_string(res).unwrap_or_else(|_| "{\"verdict\":\"FAIL\"}".to_string())
        } else {
            format!(
                "[{}] Deterministic merge completed for cycle {} into branch {} ({})\nBoard column: {}",
                res.verdict, res.cycle_id, res.target_branch, res.commit_hash, res.board_column
            )
        }
    }

    pub fn render_pr(res: &JsonPrResponse, json_mode: bool) -> String {
        if json_mode {
            serde_json::to_string(res).unwrap_or_else(|_| "{\"verdict\":\"FAIL\"}".to_string())
        } else {
            format!(
                "[{}] Pull Request #{} opened for cycle {} -> {}\nURL: {}\nBoard column: {}",
                res.verdict, res.pr_number, res.cycle_id, res.target_branch, res.pr_url, res.board_column
            )
        }
    }

    pub fn render_mcp_serve(res: &JsonMcpServeResponse, json_mode: bool) -> String {
        if json_mode {
            serde_json::to_string(res).unwrap_or_else(|_| "{\"status\":\"FAIL\"}".to_string())
        } else {
            format!("[PASS] MCP Server {} v{} listening on {}", res.server_name, res.version, res.transport)
        }
    }
}

#[cfg(test)]
#[path = "gate_presenter_tests.rs"]
mod tests;
