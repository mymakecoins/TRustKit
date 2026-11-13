// Integration tests for Fowler Smells and Repository Standard Override (TASK-003, RN-003, RF-003).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use trustkit_ast::{CodeSmellCandidate, FowlerSmellsLexicalEvaluator, RepositoryStandardRule};
use trustkit_core::entities::{
    DualAxisFindingKind, FowlerCodeSmell, GateVerdict, ReviewAxis, StandardsFindingKind,
};

#[test]
fn test_repo_standard_overrides_heuristic_fowler_smell() {
    let evaluator = FowlerSmellsLexicalEvaluator::new();

    let rule = RepositoryStandardRule::new(
        "RULE-EXHAUSTIVE-MATCH",
        "Repository enforces exhaustive pattern matching over dynamic polymorphism",
        vec!["match".to_string()],
        vec![FowlerCodeSmell::RepeatedSwitches],
    );

    let candidates = vec![CodeSmellCandidate::new(
        FowlerCodeSmell::RepeatedSwitches,
        "crates/trustkit-core/src/state_machine.rs",
        45,
        "match state { State::A => ..., State::B => ... }",
        "Repeated switch/match pattern detected across lifecycle transitions",
    )];

    let result = evaluator.evaluate(&[rule], &candidates);

    assert_eq!(result.axis, ReviewAxis::StandardsAndSmells);
    assert_eq!(result.verdict, GateVerdict::Pass);
    assert_eq!(result.findings.len(), 0);
}

#[test]
fn test_unendorsed_fowler_smell_reported_as_finding() {
    let evaluator = FowlerSmellsLexicalEvaluator::new();

    let rule = RepositoryStandardRule::new(
        "RULE-EXHAUSTIVE-MATCH",
        "Repository enforces exhaustive pattern matching over dynamic polymorphism",
        vec!["match".to_string()],
        vec![FowlerCodeSmell::RepeatedSwitches],
    );

    let candidates = vec![CodeSmellCandidate::new(
        FowlerCodeSmell::PrimitiveObsession,
        "crates/trustkit-core/src/service.rs",
        12,
        "let user_raw_string = \"user_123\";",
        "Primitive string used instead of strongly-typed UserId domain value object",
    )];

    let result = evaluator.evaluate(&[rule], &candidates);

    assert_eq!(result.axis, ReviewAxis::StandardsAndSmells);
    assert_eq!(result.verdict, GateVerdict::Fail);
    assert_eq!(result.findings.len(), 1);

    match &result.findings[0].kind {
        DualAxisFindingKind::StandardsAndSmells(StandardsFindingKind::FowlerSmell {
            smell,
            ..
        }) => {
            assert_eq!(*smell, FowlerCodeSmell::PrimitiveObsession);
        }
        _ => panic!("Expected FowlerSmell finding kind"),
    }
}

#[test]
fn test_parse_rules_from_markdown_agents_guidance() {
    let doc = r#"
# Architecture Guidelines
We enforce match exaustivo across state transitions instead of dynamic dispatch.
Also zero emojis in code.
"#;
    let rules = FowlerSmellsLexicalEvaluator::parse_rules_from_markdown(doc);
    assert_eq!(rules.len(), 2);
    assert!(rules[0].suppresses_smell(FowlerCodeSmell::RepeatedSwitches));
}
