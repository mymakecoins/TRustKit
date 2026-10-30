use super::*;

#[test]
fn test_effort_level_parsing_and_caps() {
    assert_eq!(
        CodeReviewEffortLevel::from_str_case_insensitive("low"),
        Some(CodeReviewEffortLevel::Low)
    );
    assert_eq!(
        CodeReviewEffortLevel::from_str_case_insensitive("MEDIUM"),
        Some(CodeReviewEffortLevel::Medium)
    );
    assert_eq!(
        CodeReviewEffortLevel::from_str_case_insensitive("high"),
        Some(CodeReviewEffortLevel::High)
    );
    assert_eq!(
        CodeReviewEffortLevel::from_str_case_insensitive("max"),
        Some(CodeReviewEffortLevel::Max)
    );
    assert_eq!(
        CodeReviewEffortLevel::from_str_case_insensitive("invalid"),
        None
    );

    assert_eq!(CodeReviewEffortLevel::Low.default_max_findings(), 4);
    assert_eq!(CodeReviewEffortLevel::Medium.default_max_findings(), 8);
    assert_eq!(CodeReviewEffortLevel::High.default_max_findings(), 10);
    assert_eq!(CodeReviewEffortLevel::Max.default_max_findings(), 15);
}

#[test]
fn test_security_review_verdict_skipped_preserves_typed_reason_without_null() {
    let verdict = SecurityReviewVerdict::SkippedByDeveloper {
        reason: SecuritySkipReason::DeveloperPromptOptOut,
    };

    assert!(verdict.is_pass_or_skipped());
    assert_eq!(
        verdict.report_status_line(),
        "SKIPPED (Opt-out pelo desenvolvedor)"
    );
    match verdict {
        SecurityReviewVerdict::SkippedByDeveloper { reason } => {
            assert_eq!(reason, SecuritySkipReason::DeveloperPromptOptOut);
            assert_eq!(reason.as_str(), "DEVELOPER_PROMPT_OPT_OUT");
        }
        other => panic!("Expected SkippedByDeveloper, got {other:?}"),
    }
}

#[test]
fn test_semantic_finding_confidence_threshold_and_priority() {
    let finding = SemanticReviewFinding {
        file: "src/db.rs".to_string(),
        line: 42,
        severity: ReviewSeverity::High,
        category_or_angle: ReviewAngle::LogicCorrectness.as_str().to_string(),
        confidence: 9,
        summary: "Off-by-one boundary error".to_string(),
        scenario: "When slice is empty, index 0 panics".to_string(),
        recommendation: "Use .first() instead of direct index".to_string(),
    };

    assert!(finding.meets_confidence_threshold(8));
    assert!(!finding.meets_confidence_threshold(10));
    assert!(finding.is_correctness_priority());
}

#[test]
fn test_security_review_verdict_vulnerabilities_detected_report_line() {
    let finding = SemanticReviewFinding {
        file: "src/query.rs".to_string(),
        line: 15,
        severity: ReviewSeverity::High,
        category_or_angle: AppSecCategory::InputValidationInjection.as_str().to_string(),
        confidence: 9,
        summary: "SQL injection on filter param".to_string(),
        scenario: "Unsanitized input concatenated into SQL query".to_string(),
        recommendation: "Use parameterized rusqlite::params! macro".to_string(),
    };
    let verdict = SecurityReviewVerdict::VulnerabilitiesDetected {
        findings: vec![finding],
    };

    assert!(!verdict.is_pass_or_skipped());
    assert_eq!(
        verdict.report_status_line(),
        "FAIL (1 vulnerabilidades detectadas)"
    );
}
