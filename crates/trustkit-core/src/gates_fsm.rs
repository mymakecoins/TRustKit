// 7-Gate Typestate FSM with Circuit Breaker at 5 attempts.
// Constitutional Principle III: Deterministic Equivalence & Quality Gates.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::entities::GateStage;
use crate::errors::DomainViolation;
use crate::ports::SpecValidationReport;
use crate::special_cases::SpecLockState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gate0Discovery;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gate1SpecDrafting;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gate2SpecSealed;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gate3WavePlanned;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gate4TasksImplemented;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gate5GauntletCleared;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gate6Certified;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CircuitBreakerTripped;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateFsm<State> {
    pub cycle_id: String,
    pub gauntlet_attempt: u8,
    pub state: State,
}

impl GateFsm<Gate0Discovery> {
    pub fn new(cycle_id: String) -> Self {
        Self {
            cycle_id,
            gauntlet_attempt: 0,
            state: Gate0Discovery,
        }
    }

    pub fn advance_to_spec_drafting(self) -> GateFsm<Gate1SpecDrafting> {
        GateFsm {
            cycle_id: self.cycle_id,
            gauntlet_attempt: self.gauntlet_attempt,
            state: Gate1SpecDrafting,
        }
    }
}

impl GateFsm<Gate1SpecDrafting> {
    pub fn advance_to_spec_sealed(self) -> GateFsm<Gate2SpecSealed> {
        GateFsm {
            cycle_id: self.cycle_id,
            gauntlet_attempt: self.gauntlet_attempt,
            state: Gate2SpecSealed,
        }
    }

    pub fn validate_and_seal(
        self,
        report: &SpecValidationReport,
    ) -> Result<GateFsm<Gate2SpecSealed>, DomainViolation> {
        if !report.mermaid_valid {
            return Err(DomainViolation::SpecOrTraceabilityIncomplete {
                reason: "Invalid Mermaid syntax".to_string(),
                missing_artefact: "flowchart LR/TD".to_string(),
            });
        }
        if !report.gherkin_valid {
            return Err(DomainViolation::SpecOrTraceabilityIncomplete {
                reason: "Invalid Gherkin syntax".to_string(),
                missing_artefact: "Scenario / Given / When / Then".to_string(),
            });
        }
        if let Some(orphan) = report.orphan_rules.first() {
            return Err(DomainViolation::TraceabilityGap {
                orphan_rule: orphan.clone(),
                message: format!(
                    "Regra de negocio {orphan} nao possui TASK ou Screenshot correspondente na Matriz de Rastreabilidade"
                ),
            });
        }
        Ok(self.advance_to_spec_sealed())
    }
}

impl GateFsm<Gate2SpecSealed> {
    pub fn advance_to_wave_planned(self) -> GateFsm<Gate3WavePlanned> {
        GateFsm {
            cycle_id: self.cycle_id,
            gauntlet_attempt: self.gauntlet_attempt,
            state: Gate3WavePlanned,
        }
    }

    pub fn validate_and_plan_waves(
        self,
        lock_state: &SpecLockState,
    ) -> Result<GateFsm<Gate3WavePlanned>, DomainViolation> {
        match lock_state {
            SpecLockState::Frozen { .. } => Ok(self.advance_to_wave_planned()),
            SpecLockState::DraftUnverified => Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "wave_planner".to_string(),
                details: "Cannot plan execution waves for cycle with DRAFT_UNVERIFIED specification lock state. Run 'trustkit spec lock' first.".to_string(),
            }),
        }
    }
}

impl GateFsm<Gate3WavePlanned> {
    pub fn advance_to_tasks_implemented(self) -> GateFsm<Gate4TasksImplemented> {
        GateFsm {
            cycle_id: self.cycle_id,
            gauntlet_attempt: self.gauntlet_attempt,
            state: Gate4TasksImplemented,
        }
    }
}

pub enum GauntletResult {
    Pass(GateFsm<Gate5GauntletCleared>),
    Retry(GateFsm<Gate4TasksImplemented>),
    BreakerTripped(GateFsm<CircuitBreakerTripped>),
}

impl GateFsm<Gate4TasksImplemented> {
    pub fn evaluate_gauntlet(mut self, passed: bool) -> GauntletResult {
        if passed {
            GauntletResult::Pass(GateFsm {
                cycle_id: self.cycle_id,
                gauntlet_attempt: self.gauntlet_attempt,
                state: Gate5GauntletCleared,
            })
        } else {
            self.gauntlet_attempt = self.gauntlet_attempt.saturating_add(1);
            if self.gauntlet_attempt >= 5 {
                GauntletResult::BreakerTripped(GateFsm {
                    cycle_id: self.cycle_id,
                    gauntlet_attempt: self.gauntlet_attempt,
                    state: CircuitBreakerTripped,
                })
            } else {
                GauntletResult::Retry(self)
            }
        }
    }
}

impl GateFsm<Gate5GauntletCleared> {
    pub fn advance_to_certified(self) -> GateFsm<Gate6Certified> {
        GateFsm {
            cycle_id: self.cycle_id,
            gauntlet_attempt: self.gauntlet_attempt,
            state: Gate6Certified,
        }
    }
}

pub fn transition_gate(
    current: GateStage,
    attempt: u8,
    cycle_id: &str,
) -> Result<GateStage, DomainViolation> {
    if attempt >= 5 {
        return Err(DomainViolation::CircuitBreakerTripped {
            cycle_id: cycle_id.to_string(),
            consecutive_failures: attempt,
            remediation:
                "Circuit breaker tripped at 5 attempts. Human operator must inspect and unlock."
                    .to_string(),
        });
    }

    match current {
        GateStage::Gate0Discovery => Ok(GateStage::Gate1SpecDrafting),
        GateStage::Gate1SpecDrafting => Ok(GateStage::Gate2SpecSealed),
        GateStage::Gate2SpecSealed => Ok(GateStage::Gate3WavePlanned),
        GateStage::Gate3WavePlanned => Ok(GateStage::Gate4TasksImplemented),
        GateStage::Gate4TasksImplemented => Ok(GateStage::Gate5GauntletCleared),
        GateStage::Gate5GauntletCleared => Ok(GateStage::Gate6Certified),
        GateStage::Gate6Certified => Ok(GateStage::Gate6Certified),
    }
}

#[cfg(test)]
#[path = "gates_fsm_tests.rs"]
mod tests;
