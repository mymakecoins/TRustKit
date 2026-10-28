// Audit AST Layer 0 Use Case (TASK-011).
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::entities::{
    AstAuditVerdict, EvidenceKind, ExecutionEvidence, GateStage, GateVerdict,
};
use crate::errors::DomainViolation;
use crate::ports::{AstAuditorPort, CycleEvidenceRepositoryPort};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditAstLayer0Request {
    pub cycle_id: String,
    pub files: Vec<String>,
    pub now_iso: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditAstLayer0Outcome {
    pub cycle_id: String,
    pub verdict: AstAuditVerdict,
    pub total_violations: usize,
    pub recorded_evidence_id: u64,
}

pub fn execute(
    req: AuditAstLayer0Request,
    repo: &dyn CycleEvidenceRepositoryPort,
    auditor: &dyn AstAuditorPort,
) -> Result<AuditAstLayer0Outcome, DomainViolation> {
    let reference_seals = repo.list_artifact_seals(&req.cycle_id)?;
    let verdict = auditor.audit_source_tree(&req.files, &reference_seals)?;

    let total_violations = verdict.violations().len();
    let gate_verdict = if verdict.is_pass() {
        GateVerdict::Pass
    } else {
        GateVerdict::Fail
    };

    let sanitized_stdout = if verdict.is_pass() {
        "AstAuditVerdict::CleanPass".to_string()
    } else {
        format!("AstAuditVerdict::Rejected ({} violations)", total_violations)
    };

    let evidence = ExecutionEvidence {
        evidence_id: 0,
        cycle_id: req.cycle_id.clone(),
        gate_stage: GateStage::Gate5GauntletCleared,
        gauntlet_attempt: 0,
        actor_role: "AUDITOR:AST_LAYER0".to_string(),
        evidence_kind: EvidenceKind::AstAudit,
        executed_command: "trustkit gauntlet ast-audit".to_string(),
        exit_code: if verdict.is_pass() { 0 } else { 4 },
        duration_ms: 1,
        sanitized_stdout_tail: sanitized_stdout,
        sanitized_stderr_tail: "".to_string(),
        output_blake3_digest: "".to_string(),
        verdict: gate_verdict,
        recorded_at: req.now_iso,
    };

    let recorded_evidence_id = repo.append_evidence(&evidence)?;

    Ok(AuditAstLayer0Outcome {
        cycle_id: req.cycle_id,
        verdict,
        total_violations,
        recorded_evidence_id,
    })
}
