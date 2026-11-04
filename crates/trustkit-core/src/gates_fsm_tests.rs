// Unit tests for 7-Gate Typestate FSM.
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use super::*;
use crate::ports::SpecValidationReport;
use crate::special_cases::SpecLockState;

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

#[test]
fn test_gate1_to_gate2_validations() {
    let fsm = GateFsm::new("c-g1".to_string()).advance_to_spec_drafting();

    // Invalid Mermaid
    let r_bad_mermaid = SpecValidationReport {
        mermaid_valid: false,
        gherkin_valid: true,
        orphan_rules: vec![],
        total_rules: 1,
        total_requirements: 1,
    };
    let err_mermaid = fsm.clone().validate_and_seal(&r_bad_mermaid);
    assert!(err_mermaid.is_err());
    assert_eq!(err_mermaid.unwrap_err().posix_exit_code(), 2);

    // Invalid Gherkin
    let r_bad_gherkin = SpecValidationReport {
        mermaid_valid: true,
        gherkin_valid: false,
        orphan_rules: vec![],
        total_rules: 1,
        total_requirements: 1,
    };
    let err_gherkin = fsm.clone().validate_and_seal(&r_bad_gherkin);
    assert!(err_gherkin.is_err());
    assert_eq!(err_gherkin.unwrap_err().posix_exit_code(), 2);

    // Orphan rule
    let r_orphan = SpecValidationReport {
        mermaid_valid: true,
        gherkin_valid: true,
        orphan_rules: vec!["RN-002".to_string()],
        total_rules: 2,
        total_requirements: 2,
    };
    let err_orphan = fsm.clone().validate_and_seal(&r_orphan);
    assert!(err_orphan.is_err());
    assert_eq!(err_orphan.unwrap_err().posix_exit_code(), 3);

    // Valid
    let r_ok = SpecValidationReport {
        mermaid_valid: true,
        gherkin_valid: true,
        orphan_rules: vec![],
        total_rules: 2,
        total_requirements: 2,
    };
    let sealed = fsm.validate_and_seal(&r_ok).unwrap();
    assert_eq!(sealed.state, Gate2SpecSealed);
}

#[test]
fn test_gate2_to_gate3_validations() {
    let fsm = GateFsm::new("c-g2".to_string())
        .advance_to_spec_drafting()
        .advance_to_spec_sealed();

    // Cannot advance if DraftUnverified
    let err = fsm
        .clone()
        .validate_and_plan_waves(&SpecLockState::DraftUnverified);
    assert!(err.is_err());
    assert_eq!(err.unwrap_err().posix_exit_code(), 1);

    // Can advance if Frozen
    let frozen = SpecLockState::Frozen {
        spec_path: "spec.md".to_string(),
        blake3_digest: "1234".to_string(),
    };
    let planned = fsm.validate_and_plan_waves(&frozen).unwrap();
    assert_eq!(planned.state, Gate3WavePlanned);
}
