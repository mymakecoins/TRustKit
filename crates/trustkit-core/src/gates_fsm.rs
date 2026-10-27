// 7-Gate Typestate FSM with Circuit Breaker at 5 attempts.
// Constitutional Principle III: Deterministic Equivalence & Quality Gates.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::entities::GateStage;
use crate::errors::DomainViolation;

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
}

impl GateFsm<Gate2SpecSealed> {
    pub fn advance_to_wave_planned(self) -> GateFsm<Gate3WavePlanned> {
        GateFsm {
            cycle_id: self.cycle_id,
            gauntlet_attempt: self.gauntlet_attempt,
            state: Gate3WavePlanned,
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
            remediation: "Circuit breaker tripped at 5 attempts. Human operator must inspect and unlock.".to_string(),
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
mod tests {
    use super::*;

    #[test]
    fn test_typestate_happy_path() {
        let fsm = GateFsm::new("cycle-101".to_string());
        let g1 = fsm.advance_to_spec_drafting();
        let g2 = g1.advance_to_spec_sealed();
        let g3 = g2.advance_to_wave_planned();
        let g4 = g3.advance_to_tasks_implemented();

        match g4.evaluate_gauntlet(true) {
            GauntletResult::Pass(g5) => {
                let g6 = g5.advance_to_certified();
                assert_eq!(g6.state, Gate6Certified);
            }
            _ => panic!("Expected pass"),
        }
    }

    #[test]
    fn test_typestate_circuit_breaker_after_5_failures() {
        let fsm = GateFsm::new("cycle-102".to_string());
        let mut g4 = fsm
            .advance_to_spec_drafting()
            .advance_to_spec_sealed()
            .advance_to_wave_planned()
            .advance_to_tasks_implemented();

        for i in 1..=4 {
            match g4.evaluate_gauntlet(false) {
                GauntletResult::Retry(next) => {
                    assert_eq!(next.gauntlet_attempt, i);
                    g4 = next;
                }
                _ => panic!("Expected retry at attempt {i}"),
            }
        }

        match g4.evaluate_gauntlet(false) {
            GauntletResult::BreakerTripped(breaker) => {
                assert_eq!(breaker.gauntlet_attempt, 5);
                assert_eq!(breaker.state, CircuitBreakerTripped);
            }
            _ => panic!("Expected breaker tripped at attempt 5"),
        }
    }

    #[test]
    fn test_transition_gate_function() {
        assert_eq!(
            transition_gate(GateStage::Gate0Discovery, 0, "c1").unwrap(),
            GateStage::Gate1SpecDrafting
        );
        let err = transition_gate(GateStage::Gate4TasksImplemented, 5, "c1");
        assert!(err.is_err());
        assert_eq!(err.unwrap_err().posix_exit_code(), 6);
    }
}
