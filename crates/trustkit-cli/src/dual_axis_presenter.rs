// Dual-Axis Review Presenter (TASK-004, RN-004, RF-004).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use serde::Serialize;
use std::fmt::Write as _;
use trustkit_core::entities::{
    AxisReviewResult, DualAxisFinding, DualAxisFindingKind, DualAxisReviewReport, GateVerdict,
    StandardsFindingKind,
};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct JsonFinding {
    pub file: String,
    pub line: u32,
    pub category: String,
    pub summary: String,
    pub recommendation: String,
}

impl From<&DualAxisFinding> for JsonFinding {
    fn from(finding: &DualAxisFinding) -> Self {
        let category = match &finding.kind {
            DualAxisFindingKind::StandardsAndSmells(std_kind) => match std_kind {
                StandardsFindingKind::RepositoryStandardViolation { rule_id, .. } => {
                    format!("RepositoryStandardViolation: {rule_id}")
                }
                StandardsFindingKind::FowlerSmell { smell, .. } => {
                    format!("FowlerSmell: {}", smell.as_str())
                }
            },
            DualAxisFindingKind::SpecCompliance {
                defect,
                requirement_reference,
                ..
            } => {
                format!(
                    "SpecComplianceDefect: {} ({requirement_reference})",
                    defect.as_str()
                )
            }
        };

        Self {
            file: finding.file.clone(),
            line: finding.line,
            category,
            summary: finding.summary.clone(),
            recommendation: finding.recommendation.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct JsonAxisReviewResult {
    pub axis: String,
    pub verdict: String,
    pub findings_count: usize,
    pub findings: Vec<JsonFinding>,
}

impl From<&AxisReviewResult> for JsonAxisReviewResult {
    fn from(axis_result: &AxisReviewResult) -> Self {
        let findings: Vec<JsonFinding> =
            axis_result.findings.iter().map(JsonFinding::from).collect();
        let findings_count = findings.len();
        let verdict = match axis_result.verdict {
            GateVerdict::Pass => "PASS".to_string(),
            GateVerdict::Fail => "FAIL".to_string(),
            GateVerdict::Info => "INFO".to_string(),
            GateVerdict::Blocked => "BLOCKED".to_string(),
        };

        Self {
            axis: axis_result.axis.as_str().to_string(),
            verdict,
            findings_count,
            findings,
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct JsonDualAxisReviewResponse {
    pub verdict: String,
    pub task_id: String,
    pub overall_verdict: String,
    pub standards_axis: JsonAxisReviewResult,
    pub spec_axis: JsonAxisReviewResult,
}

pub struct DualAxisPresenter;

impl DualAxisPresenter {
    pub fn to_json_response(report: &DualAxisReviewReport) -> JsonDualAxisReviewResponse {
        let overall_verdict = match report.overall_verdict() {
            GateVerdict::Pass => "PASS".to_string(),
            GateVerdict::Fail => "FAIL".to_string(),
            GateVerdict::Info => "INFO".to_string(),
            GateVerdict::Blocked => "BLOCKED".to_string(),
        };

        JsonDualAxisReviewResponse {
            verdict: overall_verdict.clone(),
            task_id: report.task_id.clone(),
            overall_verdict,
            standards_axis: JsonAxisReviewResult::from(&report.standards_axis),
            spec_axis: JsonAxisReviewResult::from(&report.spec_axis),
        }
    }

    pub fn format_json(report: &DualAxisReviewReport) -> String {
        let resp = Self::to_json_response(report);
        serde_json::to_string(&resp).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn format_ascii(report: &DualAxisReviewReport) -> String {
        let mut out = String::new();
        let overall = match report.overall_verdict() {
            GateVerdict::Pass => "PASS",
            GateVerdict::Fail => "FAIL",
            GateVerdict::Info => "INFO",
            GateVerdict::Blocked => "BLOCKED",
        };

        let _ = writeln!(out, "=== DUAL-AXIS CODE REVIEW: {} ===", report.task_id);
        let _ = writeln!(out, "Overall Verdict: {overall}\n");

        Self::write_axis_section(
            &mut out,
            "[Standards & Fowler Smells]",
            &report.standards_axis,
        );
        let _ = writeln!(out);
        Self::write_axis_section(
            &mut out,
            "[Spec Compliance & Scope Creep]",
            &report.spec_axis,
        );

        out
    }

    fn write_axis_section(out: &mut String, header: &str, axis: &AxisReviewResult) {
        let verdict = match axis.verdict {
            GateVerdict::Pass => "PASS",
            GateVerdict::Fail => "FAIL",
            GateVerdict::Info => "INFO",
            GateVerdict::Blocked => "BLOCKED",
        };
        let _ = writeln!(out, "{header}");
        let _ = writeln!(
            out,
            "Verdict: {verdict} (findings: {})",
            axis.findings.len()
        );

        if axis.findings.is_empty() {
            let _ = writeln!(out, "  (no findings)");
        } else {
            for finding in &axis.findings {
                let _ = writeln!(
                    out,
                    "  - {}:{} - {} (rec: {})",
                    finding.file, finding.line, finding.summary, finding.recommendation
                );
            }
        }
    }
}
