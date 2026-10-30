// Run Security Review Use Case for Gates 5 & 6 (TASK-006).
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL & Special Cases.
// Constitutional Principle IV: Resilient Error Handling & Zero-Panic Policy.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::entities::{
    EvidenceKind, ExecutionEvidence, GateStage, GateVerdict, SecurityReviewDecision,
    SecurityReviewVerdict, SecuritySkipReason, SemanticReviewFinding,
};
use crate::errors::DomainViolation;
use crate::ports::{CycleEvidenceRepositoryPort, SecurityDiffReviewerPort};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunSecurityReviewRequest {
    pub cycle_id: String,
    pub target_branch: String,
    pub min_confidence_threshold: u8,
    pub decision: SecurityReviewDecision,
    pub now_iso: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunSecurityReviewOutcome {
    pub cycle_id: String,
    pub verdict: SecurityReviewVerdict,
    pub surviving_findings: Vec<SemanticReviewFinding>,
    pub discarded_low_confidence_count: usize,
    pub recorded_evidence_id: u64,
}

pub fn execute(
    req: RunSecurityReviewRequest,
    repo: &dyn CycleEvidenceRepositoryPort,
    reviewer: &dyn SecurityDiffReviewerPort,
) -> Result<RunSecurityReviewOutcome, DomainViolation> {
    if !(8..=10).contains(&req.min_confidence_threshold) {
        return Err(DomainViolation::AstIntegrityOrInvariantViolation {
            invariant_id: "RN-006".to_string(),
            intercepted_command: "run_security_review".to_string(),
            reason: format!(
                "min_confidence_threshold {} is outside valid range 8..=10",
                req.min_confidence_threshold
            ),
            remediation: "Set min_confidence_threshold between 8 and 10.".to_string(),
        });
    }

    match req.decision {
        SecurityReviewDecision::Skip { reason } => record_skipped(req.cycle_id, reason, req.now_iso, repo),
        SecurityReviewDecision::PromptUser => record_skipped(
            req.cycle_id,
            SecuritySkipReason::HeadlessNonInteractiveDefault,
            req.now_iso,
            repo,
        ),
        SecurityReviewDecision::Execute => {
            let candidates = reviewer.review_branch_diff(&req.target_branch)?;
            let total_candidates = candidates.len();

            let mut surviving_findings: Vec<SemanticReviewFinding> = candidates
                .into_iter()
                .filter(|f| f.meets_confidence_threshold(req.min_confidence_threshold))
                .collect();

            surviving_findings.sort_by(|a, b| {
                b.severity
                    .rank_weight()
                    .cmp(&a.severity.rank_weight())
                    .then_with(|| b.confidence.cmp(&a.confidence))
            });

            let discarded_low_confidence_count =
                total_candidates.saturating_sub(surviving_findings.len());

            let (verdict, gate_verdict, exit_code, stdout_summary) =
                if surviving_findings.is_empty() {
                    (
                        SecurityReviewVerdict::CleanPass,
                        GateVerdict::Pass,
                        0,
                        format!(
                            "SecurityReviewVerdict::CleanPass (Confianca >= {})",
                            req.min_confidence_threshold
                        ),
                    )
                } else {
                    let count = surviving_findings.len();
                    (
                        SecurityReviewVerdict::VulnerabilitiesDetected {
                            findings: surviving_findings.clone(),
                        },
                        GateVerdict::Fail,
                        5,
                        format!(
                            "SecurityReviewVerdict::VulnerabilitiesDetected ({count} vulnerabilidades detectadas)"
                        ),
                    )
                };

            let evidence = ExecutionEvidence {
                evidence_id: 0,
                cycle_id: req.cycle_id.clone(),
                gate_stage: GateStage::Gate5GauntletCleared,
                gauntlet_attempt: 0,
                actor_role: "CRITIC:SECURITY_REVIEW".to_string(),
                evidence_kind: EvidenceKind::SecurityReview,
                executed_command: format!(
                    "trustkit security-review --target {} --min-confidence {}",
                    req.target_branch, req.min_confidence_threshold
                ),
                exit_code,
                duration_ms: 15,
                sanitized_stdout_tail: stdout_summary,
                sanitized_stderr_tail: "".to_string(),
                output_blake3_digest: "".to_string(),
                verdict: gate_verdict,
                recorded_at: req.now_iso,
            };

            let recorded_evidence_id = repo.append_evidence(&evidence)?;

            Ok(RunSecurityReviewOutcome {
                cycle_id: req.cycle_id,
                verdict,
                surviving_findings,
                discarded_low_confidence_count,
                recorded_evidence_id,
            })
        }
    }
}

fn record_skipped(
    cycle_id: String,
    reason: SecuritySkipReason,
    now_iso: String,
    repo: &dyn CycleEvidenceRepositoryPort,
) -> Result<RunSecurityReviewOutcome, DomainViolation> {
    let verdict = SecurityReviewVerdict::SkippedByDeveloper { reason };
    let evidence = ExecutionEvidence {
        evidence_id: 0,
        cycle_id: cycle_id.clone(),
        gate_stage: GateStage::Gate5GauntletCleared,
        gauntlet_attempt: 0,
        actor_role: "CRITIC:SECURITY_REVIEW".to_string(),
        evidence_kind: EvidenceKind::SecurityReview,
        executed_command: format!("trustkit security-review --skip ({})", reason.as_str()),
        exit_code: 0,
        duration_ms: 1,
        sanitized_stdout_tail: format!(
            "SecurityReviewVerdict::SkippedByDeveloper ({})",
            reason.as_str()
        ),
        sanitized_stderr_tail: "".to_string(),
        output_blake3_digest: "".to_string(),
        verdict: GateVerdict::Pass,
        recorded_at: now_iso,
    };

    let recorded_evidence_id = repo.append_evidence(&evidence)?;

    Ok(RunSecurityReviewOutcome {
        cycle_id,
        verdict,
        surviving_findings: Vec::new(),
        discarded_low_confidence_count: 0,
        recorded_evidence_id,
    })
}
