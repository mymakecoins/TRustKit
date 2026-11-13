// Integration tests for Dual-Axis Presenter (TASK-004, RN-004, RF-004).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use trustkit_cli::dual_axis_presenter::DualAxisPresenter;
use trustkit_core::entities::{
    DualAxisFinding, DualAxisFindingKind, DualAxisReviewReport, GateVerdict, SpecComplianceDefect,
};

fn create_sample_failing_spec_report() -> DualAxisReviewReport {
    let spec_finding = DualAxisFinding {
        file: "crates/trustkit-cli/src/lib.rs".to_string(),
        line: 45,
        kind: DualAxisFindingKind::SpecCompliance {
            defect: SpecComplianceDefect::ScopeCreep,
            requirement_reference: "RF-004".to_string(),
            details: "Extraneous public endpoint introduced without spec coverage".to_string(),
        },
        summary: "Scope creep detected".to_string(),
        recommendation: "Remove unapproved endpoint or update spec".to_string(),
    };

    DualAxisReviewReport::new("TASK-004".to_string(), vec![], vec![spec_finding])
}

#[test]
fn test_dual_axis_report_non_compensatory_verdict() {
    let report = create_sample_failing_spec_report();

    assert_eq!(report.standards_axis.verdict, GateVerdict::Pass);
    assert_eq!(report.spec_axis.verdict, GateVerdict::Fail);
    assert_eq!(report.overall_verdict(), GateVerdict::Fail);
    assert!(!report.is_pass());
}

#[test]
fn test_dual_axis_presenter_json_format() {
    let report = create_sample_failing_spec_report();
    let resp = DualAxisPresenter::to_json_response(&report);

    assert_eq!(resp.task_id, "TASK-004");
    assert_eq!(resp.verdict, "FAIL");
    assert_eq!(resp.overall_verdict, "FAIL");

    assert_eq!(resp.standards_axis.axis, "STANDARDS_AND_SMELLS");
    assert_eq!(resp.standards_axis.verdict, "PASS");
    assert_eq!(resp.standards_axis.findings_count, 0);
    assert!(resp.standards_axis.findings.is_empty());

    assert_eq!(resp.spec_axis.axis, "SPEC_COMPLIANCE");
    assert_eq!(resp.spec_axis.verdict, "FAIL");
    assert_eq!(resp.spec_axis.findings_count, 1);
    assert_eq!(
        resp.spec_axis.findings[0].file,
        "crates/trustkit-cli/src/lib.rs"
    );

    let json_str = DualAxisPresenter::format_json(&report);
    assert!(json_str.contains("\"overall_verdict\":\"FAIL\""));
    assert!(json_str.contains("\"standards_axis\""));
    assert!(json_str.contains("\"spec_axis\""));
}

#[test]
fn test_dual_axis_presenter_ascii_format() {
    let report = create_sample_failing_spec_report();
    let ascii = DualAxisPresenter::format_ascii(&report);

    assert!(ascii.contains("=== DUAL-AXIS CODE REVIEW: TASK-004 ==="));
    assert!(ascii.contains("Overall Verdict: FAIL"));
    assert!(ascii.contains("[Standards & Fowler Smells]"));
    assert!(ascii.contains("Verdict: PASS"));
    assert!(ascii.contains("[Spec Compliance & Scope Creep]"));
    assert!(ascii.contains("Verdict: FAIL"));
    assert!(ascii.contains("Scope creep detected"));
}

#[test]
fn test_dual_axis_presenter_both_passing() {
    let report = DualAxisReviewReport::new("TASK-004".to_string(), vec![], vec![]);

    assert_eq!(report.overall_verdict(), GateVerdict::Pass);
    assert!(report.is_pass());

    let resp = DualAxisPresenter::to_json_response(&report);
    assert_eq!(resp.verdict, "PASS");
    assert_eq!(resp.overall_verdict, "PASS");
    assert_eq!(resp.standards_axis.verdict, "PASS");
    assert_eq!(resp.spec_axis.verdict, "PASS");

    let ascii = DualAxisPresenter::format_ascii(&report);
    assert!(ascii.contains("Overall Verdict: PASS"));
    assert!(ascii.contains("(no findings)"));
}

#[test]
fn test_dual_axis_presenter_standards_failing_spec_passing() {
    let std_finding = DualAxisFinding {
        file: "crates/trustkit-cli/src/main.rs".to_string(),
        line: 12,
        kind: DualAxisFindingKind::StandardsAndSmells(
            trustkit_core::entities::StandardsFindingKind::RepositoryStandardViolation {
                rule_id: "RN-001".to_string(),
                description: "Prohibited emoji detected".to_string(),
            },
        ),
        summary: "Emoji detected in CLI output".to_string(),
        recommendation: "Remove emoji".to_string(),
    };

    let report = DualAxisReviewReport::new("TASK-004".to_string(), vec![std_finding], vec![]);

    assert_eq!(report.standards_axis.verdict, GateVerdict::Fail);
    assert_eq!(report.spec_axis.verdict, GateVerdict::Pass);
    assert_eq!(report.overall_verdict(), GateVerdict::Fail);

    let resp = DualAxisPresenter::to_json_response(&report);
    assert_eq!(resp.verdict, "FAIL");
    assert_eq!(resp.overall_verdict, "FAIL");
    assert_eq!(resp.standards_axis.verdict, "FAIL");
    assert_eq!(resp.spec_axis.verdict, "PASS");
}
