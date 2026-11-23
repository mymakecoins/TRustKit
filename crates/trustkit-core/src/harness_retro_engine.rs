// Harness Retrospective Engine over SQLite WAL events (TASK-004, TASK-005, TASK-001, RF-004, RF-005, RF-001).
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::entities::{
    EvidenceKind, ExecutionEvidence, GateStage, GateVerdict, HarnessRetroItem, HarnessRetroReport,
    ToolEconomyMetrics,
};

pub struct HarnessRetroEngine;

impl HarnessRetroEngine {
    pub fn quantify_tool_economy(evidences: &[ExecutionEvidence]) -> ToolEconomyMetrics {
        let mut total_cmd_executions = 0usize;
        let mut cached_executions = 0usize;
        let mut batched_executions = 0usize;
        let mut optimized_executions = 0usize;
        let mut total_duration_ms = 0u64;

        for ev in evidences {
            if ev.evidence_kind != EvidenceKind::CmdExec {
                continue;
            }
            total_cmd_executions += 1;
            total_duration_ms = total_duration_ms.saturating_add(ev.duration_ms);

            let cmd_lower = ev.executed_command.to_ascii_lowercase();
            let out_lower = ev.sanitized_stdout_tail.to_ascii_lowercase();

            let is_cached = cmd_lower.contains("sccache")
                || cmd_lower.contains("rustc_wrapper")
                || cmd_lower.contains(".trustkit/cache")
                || cmd_lower.contains("--target-dir")
                || out_lower.contains("cache_hit")
                || out_lower.contains("sccache");

            let test_flag_count = cmd_lower.matches("--test ").count();
            let is_batched = cmd_lower.contains("batch_exec")
                || out_lower.contains("batch_exec")
                || out_lower.contains("batched_tasks")
                || test_flag_count >= 2;

            if is_cached {
                cached_executions += 1;
            }
            if is_batched {
                batched_executions += 1;
            }
            if is_cached || is_batched {
                optimized_executions += 1;
            }
        }

        let cache_reuse_rate_pct = (optimized_executions * 100)
            .checked_div(total_cmd_executions)
            .unwrap_or(0)
            .min(100) as u8;

        let average_cycle_time_ms = total_duration_ms
            .checked_div(total_cmd_executions as u64)
            .unwrap_or(0);

        ToolEconomyMetrics {
            total_cmd_executions,
            cached_executions,
            batched_executions,
            cache_reuse_rate_pct,
            total_duration_ms,
            average_cycle_time_ms,
        }
    }

    pub fn analyze_events_with_metrics(
        cycle_id: &str,
        evidences: &[ExecutionEvidence],
    ) -> (HarnessRetroReport, ToolEconomyMetrics) {
        let metrics = Self::quantify_tool_economy(evidences);
        let report = Self::build_report(cycle_id, evidences, &metrics);
        (report, metrics)
    }

    pub fn analyze_events(cycle_id: &str, evidences: &[ExecutionEvidence]) -> HarnessRetroReport {
        let metrics = Self::quantify_tool_economy(evidences);
        Self::build_report(cycle_id, evidences, &metrics)
    }

    fn build_report(
        cycle_id: &str,
        evidences: &[ExecutionEvidence],
        metrics: &ToolEconomyMetrics,
    ) -> HarnessRetroReport {
        let mut items = Vec::new();

        let mut has_gauntlet_fail = false;
        let mut has_review_fail = false;
        let mut nav_fail_count = 0;

        for ev in evidences {
            let is_fail = ev.verdict == GateVerdict::Fail || ev.exit_code != 0;
            if is_fail {
                if ev.gate_stage == GateStage::Gate5GauntletCleared
                    || ev.evidence_kind == EvidenceKind::AstAudit
                    || ev.actor_role.contains("CRITIC")
                {
                    has_gauntlet_fail = true;
                }

                if ev.evidence_kind == EvidenceKind::CodeReview
                    || ev.evidence_kind == EvidenceKind::SecurityReview
                    || ev.actor_role.contains("REVIEWER")
                {
                    has_review_fail = true;
                }

                let combined_logs =
                    format!("{}\n{}", ev.sanitized_stdout_tail, ev.sanitized_stderr_tail)
                        .to_lowercase();

                if combined_logs.contains("no such file")
                    || combined_logs.contains("not found")
                    || combined_logs.contains("cannot find")
                {
                    nav_fail_count += 1;
                }
            }
        }

        if has_gauntlet_fail {
            items.push(HarnessRetroItem {
                category: "Novas Checagens Automatizadas".to_string(),
                severity: "HIGH".to_string(),
                recommendation: "Adicionar validacao pre-commit automatizada para evitar reprovacoes no Gauntlet.".to_string(),
            });
        }

        if has_review_fail {
            items.push(HarnessRetroItem {
                category: "Regras de Revisao".to_string(),
                severity: "MEDIUM".to_string(),
                recommendation: "Configurar regras de revisao semantica estrita para prevenir violacoes de padroes.".to_string(),
            });
        }

        if metrics.cache_reuse_rate_pct > 0 {
            items.push(HarnessRetroItem {
                category: "Economia de Ferramentas".to_string(),
                severity: "LOW".to_string(),
                recommendation: format!(
                    "Cache local e batching ativos: taxa de reaproveitamento de {}% e tempo medio de ciclo de {}ms em {} execucoes.",
                    metrics.cache_reuse_rate_pct,
                    metrics.average_cycle_time_ms,
                    metrics.total_cmd_executions
                ),
            });
        } else if metrics.total_cmd_executions >= 5 {
            items.push(HarnessRetroItem {
                category: "Economia de Ferramentas".to_string(),
                severity: "MEDIUM".to_string(),
                recommendation: "Implementar cache local e agrupamento de chamadas para otimizar tempo de execucao.".to_string(),
            });
        }

        if nav_fail_count > 0 {
            items.push(HarnessRetroItem {
                category: "Ponteiros de Navegacao".to_string(),
                severity: "LOW".to_string(),
                recommendation: "Atualizar indice de navegacao e grafos estruturais para facilitar a localizacao de arquivos.".to_string(),
            });
        }

        items.sort_by(|a, b| {
            Self::severity_rank(&b.severity)
                .cmp(&Self::severity_rank(&a.severity))
                .then_with(|| a.category.cmp(&b.category))
                .then_with(|| a.recommendation.cmp(&b.recommendation))
        });

        HarnessRetroReport::new(cycle_id, items)
    }

    fn severity_rank(sev: &str) -> u8 {
        match sev.to_ascii_uppercase().as_str() {
            "CRITICAL" | "HIGH" => 4,
            "MEDIUM" => 3,
            "LOW" => 2,
            _ => 1,
        }
    }
}
