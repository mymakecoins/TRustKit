// Sub-evaluator for Axis 1: Standards and 12 Fowler Smells (TASK-003, RN-003, RF-003).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use trustkit_core::entities::{
    AxisReviewResult, DualAxisFinding, DualAxisFindingKind, FowlerCodeSmell, GateVerdict,
    ReviewAxis, StandardsFindingKind,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryStandardRule {
    pub rule_id: String,
    pub description: String,
    pub endorsed_patterns: Vec<String>,
    pub suppressed_smells: Vec<FowlerCodeSmell>,
}

impl RepositoryStandardRule {
    pub fn new(
        rule_id: &str,
        description: &str,
        endorsed_patterns: Vec<String>,
        suppressed_smells: Vec<FowlerCodeSmell>,
    ) -> Self {
        Self {
            rule_id: rule_id.to_string(),
            description: description.to_string(),
            endorsed_patterns,
            suppressed_smells,
        }
    }

    pub fn suppresses_smell(&self, smell: FowlerCodeSmell) -> bool {
        self.suppressed_smells.contains(&smell)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeSmellCandidate {
    pub smell: FowlerCodeSmell,
    pub file_path: String,
    pub line: u32,
    pub snippet: String,
    pub rationale: String,
}

impl CodeSmellCandidate {
    pub fn new(
        smell: FowlerCodeSmell,
        file_path: &str,
        line: u32,
        snippet: &str,
        rationale: &str,
    ) -> Self {
        Self {
            smell,
            file_path: file_path.to_string(),
            line,
            snippet: snippet.to_string(),
            rationale: rationale.to_string(),
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct FowlerSmellsLexicalEvaluator;

impl FowlerSmellsLexicalEvaluator {
    pub const fn new() -> Self {
        Self
    }

    pub fn parse_rules_from_markdown(content: &str) -> Vec<RepositoryStandardRule> {
        let mut rules = Vec::new();
        let lower = content.to_lowercase();

        if lower.contains("match") && (lower.contains("exaustivo") || lower.contains("exhaustive"))
        {
            rules.push(RepositoryStandardRule::new(
                "RULE-EXHAUSTIVE-MATCH",
                "Repository enforces exhaustive pattern matching over dynamic polymorphism",
                vec!["match".to_string(), "exhaustive_match".to_string()],
                vec![FowlerCodeSmell::RepeatedSwitches],
            ));
        }

        if lower.contains("zero emojis") {
            rules.push(RepositoryStandardRule::new(
                "RULE-ZERO-EMOJIS",
                "Repository strictly forbids emojis in code and documentation",
                vec!["zero_emojis".to_string()],
                vec![],
            ));
        }

        rules
    }

    pub fn evaluate(
        &self,
        rules: &[RepositoryStandardRule],
        candidates: &[CodeSmellCandidate],
    ) -> AxisReviewResult {
        let mut findings = Vec::new();

        for candidate in candidates {
            let is_suppressed = rules
                .iter()
                .any(|rule| rule.suppresses_smell(candidate.smell));

            if !is_suppressed {
                findings.push(DualAxisFinding {
                    file: candidate.file_path.clone(),
                    line: candidate.line,
                    kind: DualAxisFindingKind::StandardsAndSmells(
                        StandardsFindingKind::FowlerSmell {
                            smell: candidate.smell,
                            description: candidate.rationale.clone(),
                        },
                    ),
                    summary: format!("{}: {}", candidate.smell.as_str(), candidate.snippet),
                    recommendation: "Refactor to satisfy Fowler clean code baseline".to_string(),
                });
            }
        }

        let verdict = if findings.is_empty() {
            GateVerdict::Pass
        } else {
            GateVerdict::Fail
        };

        AxisReviewResult {
            axis: ReviewAxis::StandardsAndSmells,
            verdict,
            findings,
        }
    }
}
