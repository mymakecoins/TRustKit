// Run Gauntlet Layer 1 Use Case (TASK-005).
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::entities::{
    CriticFinding, CriticKind, CriticVerdict, CycleStatus, EvidenceKind, ExecutionEvidence,
    GateStage, GateVerdict,
};
use crate::errors::DomainViolation;
use crate::ports::{
    AsyncExecutionSupervisorPort, CycleEvidenceRepositoryPort, SupervisedExecutionRequest,
    VisualEvidencePort,
};
use crate::special_cases::{ActiveCycleLookup, IssueLinkage};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CriticCommandOverride {
    pub kind: CriticKind,
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunGauntletLayer1Request {
    pub cycle_id: String,
    pub working_dir: String,
    pub command_overrides: Vec<CriticCommandOverride>,
    pub now_iso: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunGauntletLayer1Outcome {
    pub cycle_id: String,
    pub unanimous_pass: bool,
    pub findings: Vec<CriticFinding>,
    pub gauntlet_attempt: u8,
}

fn resolve_critic_command(
    kind: CriticKind,
    overrides: &[CriticCommandOverride],
) -> (String, Vec<String>) {
    if let Some(c) = overrides.iter().find(|o| o.kind == kind) {
        return (c.command.clone(), c.args.clone());
    }
    match kind {
        CriticKind::DomainAdversarial => ("cargo".to_string(), vec!["test".to_string(), "--test".to_string(), "domain_adversarial".to_string()]),
        CriticKind::ContractCompliance => ("cargo".to_string(), vec!["test".to_string(), "--test".to_string(), "contract_compliance".to_string()]),
        CriticKind::StaticSecurity => ("cargo".to_string(), vec!["clippy".to_string(), "--".to_string(), "-D".to_string(), "warnings".to_string()]),
        CriticKind::PlaywrightVisual => ("npx".to_string(), vec!["playwright".to_string(), "test".to_string()]),
    }
}

pub fn execute(
    req: RunGauntletLayer1Request,
    repo: &dyn CycleEvidenceRepositoryPort,
    supervisor: &dyn AsyncExecutionSupervisorPort,
    visual_port: &dyn VisualEvidencePort,
) -> Result<RunGauntletLayer1Outcome, DomainViolation> {
    let mut cycle = match repo.find_active_cycle()? {
        ActiveCycleLookup::Found(c) => *c,
        ActiveCycleLookup::NoActiveCycle => {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "gauntlet_layer1".to_string(),
                details: format!("No active SDD cycle found for cycle_id '{}'", req.cycle_id),
            });
        }
    };

    if cycle.lifecycle_status == CycleStatus::CircuitBreakerTripped || cycle.gauntlet_attempt >= 5 {
        return Err(DomainViolation::CircuitBreakerTripped {
            cycle_id: cycle.cycle_id.clone(),
            consecutive_failures: cycle.gauntlet_attempt,
            remediation: "Circuit breaker is tripped. Execute 'trustkit doctor' to diagnose.".to_string(),
        });
    }

    // Step 1: Verify Layer 0 clean pass
    let evidences = repo.list_evidences_for_cycle(&cycle.cycle_id)?;
    let layer0_passed = evidences.iter().rev().any(|e| {
        e.evidence_kind == EvidenceKind::AstAudit && e.verdict == GateVerdict::Pass
    });

    if !layer0_passed {
        return Err(DomainViolation::CriticSwarmOrPlaywrightFailed {
            layer: "Layer0Ast".to_string(),
            attempt: cycle.gauntlet_attempt,
            failure_summary: "Layer 0 AST Audit did not pass clean. Layer 1 critics cannot be dispatched.".to_string(),
        });
    }

    // Step 2: Dispatch Swarm of 4 Critics
    let critics = [
        CriticKind::DomainAdversarial,
        CriticKind::ContractCompliance,
        CriticKind::StaticSecurity,
        CriticKind::PlaywrightVisual,
    ];

    let mut findings = Vec::with_capacity(critics.len());

    for kind in critics {
        let (cmd, args) = resolve_critic_command(kind, &req.command_overrides);
        let exec_req = SupervisedExecutionRequest {
            command: cmd.clone(),
            args: args.clone(),
            working_dir: req.working_dir.clone(),
            timeout_ms: 120_000,
        };

        let outcome = supervisor.execute_supervised(&exec_req)?;
        let verdict = if outcome.exit_code == 0 {
            CriticVerdict::pass()
        } else {
            let reason = if outcome.stderr_tail.is_empty() {
                outcome.stdout_tail.clone()
            } else {
                outcome.stderr_tail.clone()
            };
            CriticVerdict::fail(reason, kind.as_str().to_string(), 0)
        };

        let evidence = ExecutionEvidence {
            evidence_id: 0,
            cycle_id: cycle.cycle_id.clone(),
            gate_stage: GateStage::Gate5GauntletCleared,
            gauntlet_attempt: cycle.gauntlet_attempt,
            actor_role: format!("CRITIC:{}", kind.as_str()),
            evidence_kind: EvidenceKind::CriticVerdict,
            executed_command: format!("{} {}", cmd, args.join(" ")),
            exit_code: outcome.exit_code,
            duration_ms: outcome.duration_ms,
            sanitized_stdout_tail: outcome.stdout_tail,
            sanitized_stderr_tail: outcome.stderr_tail,
            output_blake3_digest: outcome.output_blake3_digest,
            verdict: if verdict.is_pass() { GateVerdict::Pass } else { GateVerdict::Fail },
            recorded_at: req.now_iso.clone(),
        };
        repo.append_evidence(&evidence)?;

        findings.push(CriticFinding {
            kind,
            verdict,
            duration_ms: outcome.duration_ms,
        });
    }

    // Step 3: Restore foreign screenshots
    let issue_prefix = match &cycle.issue_linkage {
        IssueLinkage::LinkedIssue { issue_number, .. } => format!("issue-{}", issue_number),
        IssueLinkage::UnlinkedLocalCycle { directory_slug } => directory_slug.clone(),
    };
    visual_port.restore_foreign_screenshots(&issue_prefix, &format!("{}/tests/screenshots", req.working_dir))?;

    // Step 4: Validate unanimity (RN-003)
    let unanimous_pass = findings.iter().all(|f| f.verdict.is_pass());

    if !unanimous_pass {
        let cb_state = cycle.record_gauntlet_failure("One or more Gauntlet Layer 1 critics failed.");
        repo.update_cycle_status(&cycle.cycle_id, cycle.lifecycle_status, cycle.gauntlet_attempt)?;

        if cb_state.is_tripped() {
            return Err(DomainViolation::CircuitBreakerTripped {
                cycle_id: cycle.cycle_id,
                consecutive_failures: cycle.gauntlet_attempt,
                remediation: "Circuit breaker tripped after 5 failures. Run 'trustkit doctor'.".to_string(),
            });
        }

        return Err(DomainViolation::CriticSwarmOrPlaywrightFailed {
            layer: "Layer1Critics".to_string(),
            attempt: cycle.gauntlet_attempt,
            failure_summary: "Gauntlet Layer 1 failed unanimity requirement (RN-003)".to_string(),
        });
    }

    cycle.reset_gauntlet_attempts_on_success();
    repo.update_cycle_status(&cycle.cycle_id, CycleStatus::Active, 0)?;
    repo.advance_cycle_gate(&cycle.cycle_id, GateStage::Gate5GauntletCleared)?;

    Ok(RunGauntletLayer1Outcome {
        cycle_id: cycle.cycle_id,
        unanimous_pass: true,
        findings,
        gauntlet_attempt: cycle.gauntlet_attempt,
    })
}
