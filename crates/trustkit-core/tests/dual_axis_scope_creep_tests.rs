// Integration tests for Dual-Axis Spec Compliance and Scope Creep Evaluator (TASK-002).
// Constitutional Principle I: Zero external dependencies (std only in core).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use trustkit_core::entities::{
    DualAxisFindingKind, GateVerdict, PublicDiffElement, ReviewAxis, SealedSpecContext,
    SpecComplianceDefect, SpecComplianceEvaluator,
};

#[test]
fn test_detect_scope_creep_when_public_element_not_in_sealed_spec() {
    let spec = SealedSpecContext::new(
        "spec-05-dual-axis-review",
        vec!["RN-001".to_string(), "RN-002".to_string()],
        vec!["RF-001".to_string(), "RF-002".to_string()],
    );

    let diff_elements = vec![PublicDiffElement::new(
        "crates/trustkit-core/src/service.rs",
        42,
        "pub fn unrequested_backdoor()",
        "pub fn unrequested_backdoor() -> bool { true }",
        None,
    )];

    let result = SpecComplianceEvaluator::evaluate(&spec, &diff_elements);

    assert_eq!(result.axis, ReviewAxis::SpecCompliance);
    assert_eq!(result.verdict, GateVerdict::Fail);
    assert_eq!(result.findings.len(), 1);

    let finding = &result.findings[0];
    assert!(finding.is_scope_creep());
    assert_eq!(finding.file, "crates/trustkit-core/src/service.rs");
    assert_eq!(finding.line, 42);

    match &finding.kind {
        DualAxisFindingKind::SpecCompliance {
            defect,
            requirement_reference,
            details,
        } => {
            assert_eq!(*defect, SpecComplianceDefect::ScopeCreep);
            assert_eq!(requirement_reference, "NONE");
            assert!(details.contains("unrequested_backdoor"));
        }
        _ => panic!("Expected SpecCompliance finding kind"),
    }
}

#[test]
fn test_detect_scope_creep_when_referencing_non_existent_rule() {
    let spec = SealedSpecContext::new(
        "spec-05-dual-axis-review",
        vec!["RN-001".to_string()],
        vec!["RF-001".to_string()],
    );

    let diff_elements = vec![PublicDiffElement::new(
        "crates/trustkit-core/src/feature.rs",
        10,
        "pub struct UnplannedExtension",
        "pub struct UnplannedExtension;",
        Some("RN-999".to_string()),
    )];

    let result = SpecComplianceEvaluator::evaluate(&spec, &diff_elements);

    assert_eq!(result.axis, ReviewAxis::SpecCompliance);
    assert_eq!(result.verdict, GateVerdict::Fail);
    assert_eq!(result.findings.len(), 1);
    assert!(result.findings[0].is_scope_creep());
}

#[test]
fn test_pass_spec_compliance_when_all_elements_match_sealed_spec() {
    let spec = SealedSpecContext::new(
        "spec-05-dual-axis-review",
        vec!["RN-001".to_string(), "RN-002".to_string()],
        vec!["RF-001".to_string(), "RF-002".to_string()],
    );

    let diff_elements = vec![PublicDiffElement::new(
        "crates/trustkit-core/src/dual_axis_scope_creep.rs",
        15,
        "pub struct SpecComplianceEvaluator",
        "pub struct SpecComplianceEvaluator;",
        Some("RF-002".to_string()),
    )];

    let result = SpecComplianceEvaluator::evaluate(&spec, &diff_elements);

    assert_eq!(result.axis, ReviewAxis::SpecCompliance);
    assert_eq!(result.verdict, GateVerdict::Pass);
    assert!(result.findings.is_empty());
}
