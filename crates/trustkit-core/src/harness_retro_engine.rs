// Harness Retrospective Engine over SQLite WAL events (TASK-004, TASK-005, RF-004, RF-005).
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::entities::{
    EvidenceKind, ExecutionEvidence, GateStage, GateVerdict, HarnessRetroItem, HarnessRetroReport,
};

pub struct HarnessRetroEngine;

impl HarnessRetroEngine {
    pub fn analyze_events(cycle_id: &str, evidences: &[ExecutionEvidence]) -> HarnessRetroReport {
        let mut items = Vec::new();

        let mut has_gauntlet_fail = false;
        let mut has_review_fail = false;
        let mut nav_fail_count = 0;
        let mut cmd_exec_count = 0;

        for ev in evidences {
            if ev.evidence_kind == EvidenceKind::CmdExec {
                cmd_exec_count += 1;
            }

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

        if cmd_exec_count >= 5 {
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
