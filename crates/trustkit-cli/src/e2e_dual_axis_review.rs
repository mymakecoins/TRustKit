// E2E Gate 4 Dual-Axis Code Review & SQLite WAL Persistence (TASK-005, RN-005, RF-005).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::dual_axis_presenter::DualAxisPresenter;
use trustkit_core::entities::{DualAxisReviewReport, EvidenceKind, ExecutionEvidence, GateStage};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::CycleEvidenceRepositoryPort;

pub struct DualAxisGate4Reviewer;

impl DualAxisGate4Reviewer {
    pub fn execute_and_record(
        cycle_id: &str,
        task_id: &str,
        repo: &dyn CycleEvidenceRepositoryPort,
        report: &DualAxisReviewReport,
        now_iso: &str,
    ) -> Result<u64, DomainViolation> {
        let is_pass = report.is_pass();
        let json_tail = DualAxisPresenter::format_json(report);

        let evidence = ExecutionEvidence {
            evidence_id: 0,
            cycle_id: cycle_id.to_string(),
            gate_stage: GateStage::Gate4TasksImplemented,
            gauntlet_attempt: 0,
            actor_role: "AUDITOR:DUAL_AXIS_REVIEW".to_string(),
            evidence_kind: EvidenceKind::CodeReview,
            executed_command: format!("trustkit task review {task_id} --effort medium"),
            exit_code: if is_pass { 0 } else { 5 },
            duration_ms: 10,
            sanitized_stdout_tail: json_tail,
            sanitized_stderr_tail: String::new(),
            output_blake3_digest: String::new(),
            verdict: report.overall_verdict(),
            recorded_at: now_iso.to_string(),
        };

        repo.append_evidence(&evidence)
    }
}
