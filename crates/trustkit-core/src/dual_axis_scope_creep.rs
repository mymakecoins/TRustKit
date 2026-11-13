// Sub-evaluator for Axis 2: Spec Compliance and Scope Creep (RN-002 / RF-002).
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::dual_axis_entities::{
    AxisReviewResult, DualAxisFinding, DualAxisFindingKind, ReviewAxis, SpecComplianceDefect,
};
use crate::gate_entities::GateVerdict;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SealedSpecContext {
    pub spec_id: String,
    pub declared_rules: Vec<String>,
    pub declared_requirements: Vec<String>,
}

impl SealedSpecContext {
    pub fn new(
        spec_id: impl Into<String>,
        declared_rules: Vec<String>,
        declared_requirements: Vec<String>,
    ) -> Self {
        Self {
            spec_id: spec_id.into(),
            declared_rules,
            declared_requirements,
        }
    }

    pub fn has_rule_or_requirement(&self, reference: &str) -> bool {
        self.declared_rules.iter().any(|r| r == reference)
            || self.declared_requirements.iter().any(|r| r == reference)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicDiffElement {
    pub file: String,
    pub line: u32,
    pub element_name: String,
    pub raw_snippet: String,
    pub rule_reference: Option<String>,
}

impl PublicDiffElement {
    pub fn new(
        file: impl Into<String>,
        line: u32,
        element_name: impl Into<String>,
        raw_snippet: impl Into<String>,
        rule_reference: Option<String>,
    ) -> Self {
        Self {
            file: file.into(),
            line,
            element_name: element_name.into(),
            raw_snippet: raw_snippet.into(),
            rule_reference,
        }
    }
}

pub struct SpecComplianceEvaluator;

impl SpecComplianceEvaluator {
    pub fn evaluate_scope_creep(
        spec: &SealedSpecContext,
        elements: &[PublicDiffElement],
    ) -> Vec<DualAxisFinding> {
        let mut findings = Vec::new();

        for element in elements {
            let is_covered = match &element.rule_reference {
                Some(ref_name) => spec.has_rule_or_requirement(ref_name),
                None => false,
            };

            if !is_covered {
                let req_ref = element
                    .rule_reference
                    .clone()
                    .unwrap_or_else(|| "NONE".to_string());

                findings.push(DualAxisFinding {
                    file: element.file.clone(),
                    line: element.line,
                    kind: DualAxisFindingKind::SpecCompliance {
                        defect: SpecComplianceDefect::ScopeCreep,
                        requirement_reference: req_ref,
                        details: format!(
                            "Unrequested public element '{}': {}",
                            element.element_name, element.raw_snippet
                        ),
                    },
                    summary: format!("Scope creep detected in {}", element.element_name),
                    recommendation: format!(
                        "Remove '{}' or obtain spec approval via new RN/RF rule",
                        element.element_name
                    ),
                });
            }
        }

        findings
    }

    pub fn evaluate(spec: &SealedSpecContext, elements: &[PublicDiffElement]) -> AxisReviewResult {
        let findings = Self::evaluate_scope_creep(spec, elements);
        let verdict = if findings.is_empty() {
            GateVerdict::Pass
        } else {
            GateVerdict::Fail
        };

        AxisReviewResult {
            axis: ReviewAxis::SpecCompliance,
            verdict,
            findings,
        }
    }
}
