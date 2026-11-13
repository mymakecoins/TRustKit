// Integration tests for Dual-Axis Semantic Review Entities (TASK-001).
// Constitutional Principle I: Zero external dependencies (std only in core).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use trustkit_core::entities::{
    DualAxisFinding, DualAxisFindingKind, DualAxisReviewReport, FowlerCodeSmell, GateVerdict,
    ReviewAxis, SpecComplianceDefect, StandardsFindingKind,
};

#[test]
fn test_instantiate_standards_and_smells_fowler_finding() {
    let finding = DualAxisFinding {
        file: "crates/trustkit-core/src/lib.rs".to_string(),
        line: 42,
        kind: DualAxisFindingKind::StandardsAndSmells(StandardsFindingKind::FowlerSmell {
            smell: FowlerCodeSmell::RepeatedSwitches,
            description: "Repeated switch statements on entity variants".to_string(),
        }),
        summary: "Potential repeated switch smell".to_string(),
        recommendation: "Refactor to polymorphic dispatcher".to_string(),
    };

    assert_eq!(finding.axis(), ReviewAxis::StandardsAndSmells);
    assert!(!finding.is_scope_creep());
}

#[test]
fn test_instantiate_repository_standard_violation() {
    let finding = DualAxisFinding {
        file: "crates/trustkit-cli/src/main.rs".to_string(),
        line: 10,
        kind: DualAxisFindingKind::StandardsAndSmells(
            StandardsFindingKind::RepositoryStandardViolation {
                rule_id: "RN-013".to_string(),
                description: "Prohibition of NULL values".to_string(),
            },
        ),
        summary: "Hard repository rule violation".to_string(),
        recommendation: "Use Option or Result".to_string(),
    };

    assert_eq!(finding.axis(), ReviewAxis::StandardsAndSmells);
}

#[test]
fn test_instantiate_spec_compliance_defects() {
    let finding = DualAxisFinding {
        file: "crates/trustkit-core/src/service.rs".to_string(),
        line: 100,
        kind: DualAxisFindingKind::SpecCompliance {
            defect: SpecComplianceDefect::ScopeCreep,
            requirement_reference: "RF-002".to_string(),
            details: "Added unrequested public flag".to_string(),
        },
        summary: "Scope creep detected against frozen spec".to_string(),
        recommendation: "Remove unrequested flag".to_string(),
    };

    assert_eq!(finding.axis(), ReviewAxis::SpecCompliance);
    assert!(finding.is_scope_creep());
}

#[test]
fn test_fowler_code_smells_all_twelve_variants() {
    let smells = [
        FowlerCodeSmell::MysteriousName,
        FowlerCodeSmell::DuplicatedCode,
        FowlerCodeSmell::FeatureEnvy,
        FowlerCodeSmell::DataClumps,
        FowlerCodeSmell::PrimitiveObsession,
        FowlerCodeSmell::RepeatedSwitches,
        FowlerCodeSmell::ShotgunSurgery,
        FowlerCodeSmell::DivergentChange,
        FowlerCodeSmell::SpeculativeGenerality,
        FowlerCodeSmell::MessageChains,
        FowlerCodeSmell::MiddleMan,
        FowlerCodeSmell::RefusedBequest,
    ];

    assert_eq!(smells.len(), 12);
    for smell in smells {
        assert!(!smell.as_str().is_empty());
    }
}

#[test]
fn test_dual_axis_review_report_non_compensatory() {
    let standards_finding = DualAxisFinding {
        file: "crates/trustkit-core/src/lib.rs".to_string(),
        line: 1,
        kind: DualAxisFindingKind::StandardsAndSmells(StandardsFindingKind::FowlerSmell {
            smell: FowlerCodeSmell::DuplicatedCode,
            description: "Duplicated block".to_string(),
        }),
        summary: "Duplication".to_string(),
        recommendation: "Extract helper".to_string(),
    };

    let report_with_smell =
        DualAxisReviewReport::new("TASK-001".to_string(), vec![standards_finding], vec![]);

    assert_eq!(report_with_smell.standards_axis.verdict, GateVerdict::Fail);
    assert_eq!(report_with_smell.spec_axis.verdict, GateVerdict::Pass);
    assert_eq!(report_with_smell.overall_verdict(), GateVerdict::Fail);
    assert!(!report_with_smell.is_pass());

    let clean_report = DualAxisReviewReport::new("TASK-001".to_string(), vec![], vec![]);
    assert_eq!(clean_report.overall_verdict(), GateVerdict::Pass);
    assert!(clean_report.is_pass());
}

#[test]
fn test_green_phase_nominal_verification() {
    let report = DualAxisReviewReport::new("TASK-001".to_string(), vec![], vec![]);
    assert_eq!(report.overall_verdict(), GateVerdict::Pass);
    assert!(report.is_pass());
}
