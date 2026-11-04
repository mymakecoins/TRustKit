use trustkit_core::entities::{
    CliSecurityReviewFlag, SecurityReviewDecision, SecurityReviewMode, SecuritySkipReason,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::security_review_policy::{
    SecurityReviewPolicyConfig, parse_security_review_policy, resolve_security_review_decision,
};

#[test]
fn test_parse_security_review_policy_toml_prompt_and_threshold() {
    let toml_content = r#"
[governance.gauntlet]
security_review_mode = "prompt"
min_confidence_threshold = 8
"#;

    let policy = parse_security_review_policy(toml_content).expect("should parse valid toml");
    assert_eq!(policy.mode, SecurityReviewMode::Prompt);
    assert_eq!(policy.min_confidence_threshold, 8);

    let decision_tty =
        resolve_security_review_decision(CliSecurityReviewFlag::Unspecified, &policy, true, false);
    assert_eq!(decision_tty, SecurityReviewDecision::PromptUser);

    let decision_json =
        resolve_security_review_decision(CliSecurityReviewFlag::Unspecified, &policy, true, true);
    assert_eq!(
        decision_json,
        SecurityReviewDecision::Skip {
            reason: SecuritySkipReason::HeadlessNonInteractiveDefault,
        }
    );
}

#[test]
fn test_parse_security_review_policy_rejects_out_of_range_threshold() {
    let toml_low = r#"
[governance.gauntlet]
security_review_mode = "always"
min_confidence_threshold = 5
"#;

    let res = parse_security_review_policy(toml_low);
    assert!(res.is_err());
    match res.unwrap_err() {
        DomainViolation::AstIntegrityOrInvariantViolation { invariant_id, .. } => {
            assert_eq!(invariant_id, "RN-002");
        }
        other => panic!("Expected AstIntegrityOrInvariantViolation, got {other:?}"),
    }
}

#[test]
fn test_cli_flags_override_policy_and_tty_state() {
    let policy_disabled = SecurityReviewPolicyConfig::new(SecurityReviewMode::Disabled, 9).unwrap();
    let decision_force_exec = resolve_security_review_decision(
        CliSecurityReviewFlag::ForceExecute,
        &policy_disabled,
        false,
        true,
    );
    assert_eq!(decision_force_exec, SecurityReviewDecision::Execute);

    let policy_always = SecurityReviewPolicyConfig::new(SecurityReviewMode::Always, 8).unwrap();
    let decision_force_skip = resolve_security_review_decision(
        CliSecurityReviewFlag::ForceSkip,
        &policy_always,
        true,
        false,
    );
    assert_eq!(
        decision_force_skip,
        SecurityReviewDecision::Skip {
            reason: SecuritySkipReason::CliFlagSkip,
        }
    );
}
