// Review Task Code Multi-Angle Use Case for Gate 4 Refactor phase (TASK-003).
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL & Special Cases.
// Constitutional Principle IV: Resilient Error Handling & Zero-Panic Policy.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::entities::{
    CodeReviewEffortLevel, EvidenceKind, ExecutionEvidence, GateStage, GateVerdict, ReviewSeverity,
    SemanticReviewFinding,
};
use crate::errors::DomainViolation;
use crate::ports::{CycleEvidenceRepositoryPort, SemanticCodeReviewerPort};
use crate::special_cases::TddLifecycleState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewTaskCodeRequest {
    pub cycle_id: String,
    pub task_id: String,
    pub effort_level: CodeReviewEffortLevel,
    pub max_findings_override: usize,
    pub now_iso: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewTaskCodeOutcome {
    pub cycle_id: String,
    pub task_id: String,
    pub effort_level: CodeReviewEffortLevel,
    pub findings: Vec<SemanticReviewFinding>,
    pub clean_pass: bool,
    pub recorded_evidence_id: u64,
}

pub fn execute(
    req: ReviewTaskCodeRequest,
    repo: &dyn CycleEvidenceRepositoryPort,
    reviewer: &dyn SemanticCodeReviewerPort,
) -> Result<ReviewTaskCodeOutcome, DomainViolation> {
    let task = repo
        .find_atomic_task(&req.cycle_id, &req.task_id)?
        .ok_or_else(|| DomainViolation::TddLifecycleViolation {
            task_id: req.task_id.clone(),
            reason: format!(
                "Atomic task '{}' not found in cycle '{}'",
                req.task_id, req.cycle_id
            ),
        })?;

    let green_verified = matches!(
        task.tdd_lifecycle_state,
        TddLifecycleState::GreenVerified { .. } | TddLifecycleState::Committed { .. }
    );

    if !green_verified {
        return Err(DomainViolation::TddLifecycleViolation {
            task_id: req.task_id,
            reason: "Gate 4 Code Review (Refactor) requires prior GREEN phase verification."
                .to_string(),
        });
    }

    let raw_candidates =
        reviewer.review_task_diff(&req.cycle_id, &req.task_id, req.effort_level)?;

    let mut verified_findings: Vec<SemanticReviewFinding> = raw_candidates
        .into_iter()
        .filter(|f| f.meets_confidence_threshold(7))
        .collect();

    verified_findings.sort_by(|a, b| {
        b.is_correctness_priority()
            .cmp(&a.is_correctness_priority())
            .then_with(|| b.severity.rank_weight().cmp(&a.severity.rank_weight()))
            .then_with(|| b.confidence.cmp(&a.confidence))
    });

    let cap = if req.max_findings_override > 0 {
        req.max_findings_override
    } else {
        req.effort_level.default_max_findings()
    };

    if verified_findings.len() > cap {
        verified_findings.truncate(cap);
    }

    let has_blocking_bug = verified_findings
        .iter()
        .any(|f| f.is_correctness_priority() && f.severity == ReviewSeverity::High);
    let clean_pass = !has_blocking_bug;

    let evidence = ExecutionEvidence {
        evidence_id: 0,
        cycle_id: req.cycle_id.clone(),
        gate_stage: GateStage::Gate4TasksImplemented,
        gauntlet_attempt: 0,
        actor_role: "AUDITOR:CODE_REVIEW".to_string(),
        evidence_kind: EvidenceKind::CodeReview,
        executed_command: format!(
            "trustkit task review {} --effort {}",
            req.task_id,
            req.effort_level.as_str()
        ),
        exit_code: if clean_pass { 0 } else { 5 },
        duration_ms: 10,
        sanitized_stdout_tail: if clean_pass {
            format!(
                "CodeReview::Pass ({} findings, effort={})",
                verified_findings.len(),
                req.effort_level.as_str()
            )
        } else {
            format!(
                "CodeReview::BlockingFindings ({} findings, effort={})",
                verified_findings.len(),
                req.effort_level.as_str()
            )
        },
        sanitized_stderr_tail: "".to_string(),
        output_blake3_digest: "".to_string(),
        verdict: if clean_pass {
            GateVerdict::Pass
        } else {
            GateVerdict::Fail
        },
        recorded_at: req.now_iso,
    };

    let recorded_evidence_id = repo.append_evidence(&evidence)?;

    Ok(ReviewTaskCodeOutcome {
        cycle_id: req.cycle_id,
        task_id: req.task_id,
        effort_level: req.effort_level,
        findings: verified_findings,
        clean_pass,
        recorded_evidence_id,
    })
}
