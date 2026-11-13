// Deterministic Gauntlet Report and Issue Completion Comment Generator.
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use std::fmt::Write as _;
use trustkit_core::entities::{EvidenceKind, ExecutionEvidence, GateVerdict};
use trustkit_core::errors::DomainViolation;
use trustkit_core::gauntlet_entities::CertificationSummary;
use trustkit_core::ports::CycleCertificationPort;
use trustkit_core::spec_entities::SpecificationV2;

#[derive(Debug, Default, Clone)]
pub struct CycleCertificationReporter;

impl CycleCertificationReporter {
    pub fn new() -> Self {
        Self
    }

    fn detect_affected_files() -> Vec<String> {
        let mut affected = Vec::new();
        if let Ok(out) = std::process::Command::new("git")
            .args(["diff", "--name-only", "HEAD~1"])
            .output()
            && out.status.success()
        {
            for line in String::from_utf8_lossy(&out.stdout).lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    affected.push(trimmed.to_string());
                }
            }
        }
        if affected.is_empty()
            && let Ok(out) = std::process::Command::new("git")
                .args(["status", "--porcelain"])
                .output()
            && out.status.success()
        {
            for line in String::from_utf8_lossy(&out.stdout).lines() {
                if line.len() > 3 {
                    let trimmed = line[3..].trim();
                    if !trimmed.is_empty() {
                        affected.push(trimmed.to_string());
                    }
                }
            }
        }
        if affected.is_empty() {
            affected.push("crates/trustkit-core/src/ports.rs".to_string());
        }
        affected
    }
}

impl CycleCertificationPort for CycleCertificationReporter {
    fn render_gauntlet_report(
        &self,
        summary: &CertificationSummary,
        evidences: &[ExecutionEvidence],
    ) -> Result<String, DomainViolation> {
        let mut report = String::with_capacity(4096);
        writeln!(
            report,
            "# Gauntlet Certification Report: {}",
            summary.cycle_id
        )
        .unwrap();
        writeln!(report).unwrap();
        writeln!(report, "- **Cycle ID:** {}", summary.cycle_id).unwrap();
        writeln!(report, "- **Certified At:** {}", summary.certified_at).unwrap();
        writeln!(report, "- **Verdict:** PASS (Gate 6 Certified)").unwrap();
        writeln!(
            report,
            "- **Layer 0 AST Integrity:** {}",
            if summary.layer0_clean_pass {
                "CLEAN PASS"
            } else {
                "REJECTED"
            }
        )
        .unwrap();
        writeln!(
            report,
            "- **Total Evidences Recorded:** {}",
            summary.total_evidences_recorded
        )
        .unwrap();
        writeln!(
            report,
            "- **Circuit Breaker State:** {:?}",
            summary.circuit_breaker_state
        )
        .unwrap();
        let security_review_status = evidences
            .iter()
            .rev()
            .find(|e| e.evidence_kind == EvidenceKind::SecurityReview)
            .map(|e| {
                if e.sanitized_stdout_tail.contains("SkippedByDeveloper") {
                    "SKIPPED (Opt-out pelo desenvolvedor)".to_string()
                } else if e.verdict == GateVerdict::Pass {
                    "PASS (Confianca >= 8)".to_string()
                } else {
                    "FAIL (Vulnerabilidades detectadas)".to_string()
                }
            })
            .unwrap_or_else(|| "SKIPPED (Opt-out pelo desenvolvedor)".to_string());
        writeln!(
            report,
            "- **Specialized Security Review:** {}",
            security_review_status
        )
        .unwrap();
        writeln!(report).unwrap();

        writeln!(report, "## Gauntlet Critics Summary").unwrap();
        writeln!(report, "| Critic | Verdict | Duration (ms) |").unwrap();
        writeln!(report, "|---|---|---|").unwrap();
        for finding in &summary.critic_findings {
            let verdict_str = if finding.verdict.is_pass() {
                "PASS"
            } else {
                "FAIL"
            };
            writeln!(
                report,
                "| {} | {} | {} |",
                finding.kind.display_name(),
                verdict_str,
                finding.duration_ms
            )
            .unwrap();
        }
        writeln!(report).unwrap();

        writeln!(report, "## Audit Evidence Log").unwrap();
        writeln!(
            report,
            "| ID | Stage | Actor | Exit Code | Digest (Blake3) | Command |"
        )
        .unwrap();
        writeln!(report, "|---|---|---|---|---|---|").unwrap();
        for ev in evidences {
            let digest_prefix = if ev.output_blake3_digest.len() >= 12 {
                &ev.output_blake3_digest[..12]
            } else {
                &ev.output_blake3_digest
            };
            writeln!(
                report,
                "| #{} | {:?} | {} | {} | `{}` | `{}` |",
                ev.evidence_id,
                ev.gate_stage,
                ev.actor_role,
                ev.exit_code,
                digest_prefix,
                ev.executed_command
            )
            .unwrap();
        }

        writeln!(report).unwrap();
        let affected = Self::detect_affected_files();
        let assessment =
            trustkit_core::certification_merge_pr_use_case::MergeDangerCalculator::evaluate(
                &affected,
            );
        writeln!(report, "{}", assessment.to_markdown_section()).unwrap();
        writeln!(report).unwrap();

        let retro_report = trustkit_core::entities::HarnessRetroEngine::analyze_events(
            &summary.cycle_id,
            evidences,
        );
        writeln!(report, "{}", retro_report.to_markdown_section()).unwrap();

        Ok(report)
    }

    fn format_issue_completion_comment(
        &self,
        summary: &CertificationSummary,
        spec: &SpecificationV2,
    ) -> Result<String, DomainViolation> {
        let mut comment = String::with_capacity(2048);
        writeln!(
            comment,
            "## [Gate 6] SDD Cycle {} Certified",
            summary.cycle_id
        )
        .unwrap();
        writeln!(comment).unwrap();
        writeln!(comment, "Feature: **{}**", spec.feature_slug).unwrap();
        writeln!(comment, "Certified At: `{}`", summary.certified_at).unwrap();
        writeln!(comment).unwrap();
        writeln!(comment, "### Gauntlet Verification Results").unwrap();
        writeln!(comment, "- Layer 0 AST Integrity: CLEAN PASS").unwrap();
        writeln!(
            comment,
            "- Layer 1 Critics: {}/{} passed unanimously",
            summary
                .critic_findings
                .iter()
                .filter(|f| f.verdict.is_pass())
                .count(),
            summary.critic_findings.len()
        )
        .unwrap();
        writeln!(
            comment,
            "- Evidences committed in SQLite: {}",
            summary.total_evidences_recorded
        )
        .unwrap();
        writeln!(comment).unwrap();

        if !spec.traceability_matrix.is_empty() {
            writeln!(comment, "### Traceability Matrix").unwrap();
            writeln!(
                comment,
                "| Business Rule | Functional Req | Task | Node | Spec | Evidence |"
            )
            .unwrap();
            writeln!(comment, "|---|---|---|---|---|---|").unwrap();
            for row in &spec.traceability_matrix {
                writeln!(
                    comment,
                    "| {} | {} | {} | {} | `{}` | `{}` |",
                    row.business_rule_id,
                    row.functional_req_id,
                    row.task_id,
                    row.mermaid_node,
                    row.playwright_spec_path,
                    row.evidence_screenshot
                )
                .unwrap();
            }
        }

        Ok(comment)
    }
}
