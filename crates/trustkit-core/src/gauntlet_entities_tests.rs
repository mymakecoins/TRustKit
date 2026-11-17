// Unit tests for gauntlet entities.
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use super::*;
use crate::special_cases::CircuitBreakerState;

#[test]
fn test_tdd_phase_representation_and_parsing() {
    assert_eq!(TddPhase::Red.as_str(), "RED");
    assert_eq!(TddPhase::Green.as_str(), "GREEN");
    assert_eq!(
        TddPhase::from_str_case_insensitive("red"),
        Some(TddPhase::Red)
    );
    assert_eq!(
        TddPhase::from_str_case_insensitive(" GREEN "),
        Some(TddPhase::Green)
    );
    assert_eq!(TddPhase::from_str_case_insensitive("invalid"), None);
}

#[test]
fn test_tdd_execution_proof_validations() {
    let red_fail = TddExecutionProof::new(TddPhase::Red, 1, 120, 10);
    assert!(red_fail.validates_phase_expectation());

    let red_pass = TddExecutionProof::new(TddPhase::Red, 0, 100, 11);
    assert!(!red_pass.validates_phase_expectation());

    let green_pass = TddExecutionProof::new(TddPhase::Green, 0, 95, 12);
    assert!(green_pass.validates_phase_expectation());

    let green_fail = TddExecutionProof::new(TddPhase::Green, 2, 80, 13);
    assert!(!green_fail.validates_phase_expectation());
}

#[test]
fn test_critic_kind_and_verdict() {
    let critics = [
        (
            CriticKind::DomainAdversarial,
            "DOMAIN_ADVERSARIAL",
            "Domain/Adversarial Critic",
        ),
        (
            CriticKind::ContractCompliance,
            "CONTRACT_COMPLIANCE",
            "Contract Compliance Critic",
        ),
        (
            CriticKind::StaticSecurity,
            "STATIC_SECURITY",
            "Static Security Critic",
        ),
        (
            CriticKind::PlaywrightVisual,
            "PLAYWRIGHT_VISUAL",
            "Playwright Visual Critic",
        ),
    ];
    for (critic, label, display) in critics {
        assert_eq!(critic.as_str(), label);
        assert_eq!(critic.display_name(), display);
    }

    let pass = CriticVerdict::pass();
    assert!(pass.is_pass());
    assert!(!pass.is_fail());

    let fail = CriticVerdict::fail("Syntax violation", "crates/trustkit-core/src/lib.rs", 42);
    assert!(fail.is_fail());
    assert!(!fail.is_pass());
    if let CriticVerdict::Fail { reason, file, line } = fail {
        assert_eq!(reason, "Syntax violation");
        assert_eq!(file, "crates/trustkit-core/src/lib.rs");
        assert_eq!(line, 42);
    }
}

#[test]
fn test_certification_summary_full_pass() {
    let findings = vec![
        CriticFinding {
            kind: CriticKind::DomainAdversarial,
            verdict: CriticVerdict::pass(),
            duration_ms: 150,
        },
        CriticFinding {
            kind: CriticKind::ContractCompliance,
            verdict: CriticVerdict::pass(),
            duration_ms: 120,
        },
        CriticFinding {
            kind: CriticKind::StaticSecurity,
            verdict: CriticVerdict::pass(),
            duration_ms: 80,
        },
        CriticFinding {
            kind: CriticKind::PlaywrightVisual,
            verdict: CriticVerdict::pass(),
            duration_ms: 500,
        },
    ];

    let summary = CertificationSummary {
        cycle_id: "cycle-test".to_string(),
        layer0_clean_pass: true,
        critic_findings: findings,
        total_evidences_recorded: 4,
        circuit_breaker_state: CircuitBreakerState::Closed,
        certified_at: "2026-10-28T17:00:00Z".to_string(),
    };

    assert!(summary.is_fully_certified());
}

#[test]
fn test_certification_summary_fails_on_tripped_circuit_breaker() {
    let findings = vec![CriticFinding {
        kind: CriticKind::DomainAdversarial,
        verdict: CriticVerdict::pass(),
        duration_ms: 100,
    }];

    let summary = CertificationSummary {
        cycle_id: "cycle-test".to_string(),
        layer0_clean_pass: true,
        critic_findings: findings,
        total_evidences_recorded: 1,
        circuit_breaker_state: CircuitBreakerState::Tripped {
            consecutive_failures: 5,
            diagnostic: "Max gauntlet attempts exceeded".to_string(),
        },
        certified_at: "2026-10-28T17:00:00Z".to_string(),
    };

    assert!(!summary.is_fully_certified());
}

#[test]
fn test_certification_summary_fails_on_critic_failure() {
    let findings = vec![CriticFinding {
        kind: CriticKind::PlaywrightVisual,
        verdict: CriticVerdict::fail("Visual diff mismatch", "evidence/shot.png", 0),
        duration_ms: 200,
    }];

    let summary = CertificationSummary {
        cycle_id: "cycle-test".to_string(),
        layer0_clean_pass: true,
        critic_findings: findings,
        total_evidences_recorded: 1,
        circuit_breaker_state: CircuitBreakerState::Closed,
        certified_at: "2026-10-28T17:00:00Z".to_string(),
    };

    assert!(!summary.is_fully_certified());
}

#[test]
fn test_diagnostic_bug_loop_plan_six_phases_and_validation() {
    let valid_plan = DiagnosticBugLoopPlan {
        red_loop_command: "cargo test -p trustkit-core test_fail".to_string(),
        minimized_repro: "Isolated unit test reproducing state mismatch".to_string(),
        hypotheses: vec![
            "Hypothesis 1: WAL lock contention".to_string(),
            "Hypothesis 2: Stale BLAKE3 digest".to_string(),
            "Hypothesis 3: Unsorted task DAG".to_string(),
        ],
        debug_tag_prefix: "[DEBUG-a4f2]".to_string(),
        seam_fix_strategy: "Fix state transition in domain entity seam".to_string(),
        cleanup_steps: vec![
            "Remove [DEBUG-a4f2] instrumentation logs".to_string(),
            "Re-run full workspace test suite".to_string(),
        ],
    };

    assert_eq!(valid_plan.phase_count(), 6);
    assert!(valid_plan.validate().is_ok());

    let invalid_hypotheses_plan = DiagnosticBugLoopPlan {
        hypotheses: vec!["Only one".to_string()],
        ..valid_plan.clone()
    };
    assert!(invalid_hypotheses_plan.validate().is_err());

    let invalid_tag_plan = DiagnosticBugLoopPlan {
        debug_tag_prefix: "LOG-1234".to_string(),
        ..valid_plan
    };
    assert!(invalid_tag_plan.validate().is_err());
}
