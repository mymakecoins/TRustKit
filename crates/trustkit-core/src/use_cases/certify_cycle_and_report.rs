// Certify Cycle and Report Use Case (TASK-006).
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::entities::{
    CertificationSummary, CircuitBreakerState, CriticFinding, CriticKind, CriticVerdict,
    CycleStatus, EvidenceKind, GateStage, GateVerdict, SpecificationV2,
};
use crate::errors::DomainViolation;
use crate::ports::{CycleCertificationPort, CycleEvidenceRepositoryPort};
use crate::special_cases::ActiveCycleLookup;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CertifyCycleRequest {
    pub cycle_id: String,
    pub spec: SpecificationV2,
    pub now_iso: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CertifyCycleOutcome {
    pub cycle_id: String,
    pub report_markdown: String,
    pub completion_comment: String,
    pub certified_at: String,
}

pub fn execute(
    req: CertifyCycleRequest,
    repo: &dyn CycleEvidenceRepositoryPort,
    cert_port: &dyn CycleCertificationPort,
) -> Result<CertifyCycleOutcome, DomainViolation> {
    let cycle = match repo.find_active_cycle()? {
        ActiveCycleLookup::Found(c) => *c,
        ActiveCycleLookup::NoActiveCycle => {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "certification".to_string(),
                details: format!("No active SDD cycle found for cycle_id '{}'", req.cycle_id),
            });
        }
    };

    let evidences = repo.list_certification_evidences(&cycle.cycle_id)?;

    // Check Layer 0 pass
    let layer0_passed = evidences.iter().rev().any(|e| {
        e.evidence_kind == EvidenceKind::AstAudit && e.verdict == GateVerdict::Pass
    });

    if !layer0_passed {
        return Err(DomainViolation::InfrastructureBoundaryFault {
            subsystem: "certification".to_string(),
            details: "Cannot certify cycle: Layer 0 AST Audit did not pass.".to_string(),
        });
    }

    // Check 4 Critics pass
    let critics = [
        CriticKind::DomainAdversarial,
        CriticKind::ContractCompliance,
        CriticKind::StaticSecurity,
        CriticKind::PlaywrightVisual,
    ];

    let mut critic_findings = Vec::with_capacity(critics.len());
    for kind in critics {
        let critic_role = format!("CRITIC:{}", kind.as_str());
        let ev = evidences
            .iter()
            .rev()
            .find(|e| e.actor_role == critic_role && e.evidence_kind == EvidenceKind::CriticVerdict);

        match ev {
            Some(e) if e.verdict == GateVerdict::Pass => {
                critic_findings.push(CriticFinding {
                    kind,
                    verdict: CriticVerdict::pass(),
                    duration_ms: e.duration_ms,
                });
            }
            Some(e) => {
                return Err(DomainViolation::InfrastructureBoundaryFault {
                    subsystem: "certification".to_string(),
                    details: format!("Critic {} failed with exit code {}", kind.as_str(), e.exit_code),
                });
            }
            None => {
                return Err(DomainViolation::InfrastructureBoundaryFault {
                    subsystem: "certification".to_string(),
                    details: format!("Missing evidence for critic {}", kind.as_str()),
                });
            }
        }
    }

    let summary = CertificationSummary {
        cycle_id: cycle.cycle_id.clone(),
        layer0_clean_pass: true,
        critic_findings,
        total_evidences_recorded: evidences.len(),
        circuit_breaker_state: CircuitBreakerState::Closed,
        certified_at: req.now_iso.clone(),
    };

    if !summary.is_fully_certified() {
        return Err(DomainViolation::InfrastructureBoundaryFault {
            subsystem: "certification".to_string(),
            details: "Certification conditions not fully met.".to_string(),
        });
    }

    let report_markdown = cert_port.render_gauntlet_report(&summary, &evidences)?;
    let completion_comment = cert_port.format_issue_completion_comment(&summary, &req.spec)?;

    // Advance status to Certified and gate to Gate6Certified
    repo.update_cycle_status(&cycle.cycle_id, CycleStatus::Certified, 0)?;
    repo.advance_cycle_gate(&cycle.cycle_id, GateStage::Gate6Certified)?;

    Ok(CertifyCycleOutcome {
        cycle_id: cycle.cycle_id,
        report_markdown,
        completion_comment,
        certified_at: req.now_iso,
    })
}
