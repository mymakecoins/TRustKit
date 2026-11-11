// Dual-mode presenter for Next-Step Guidance (ASCII footer and null-free JSON array).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use serde::{Deserialize, Serialize};
use trustkit_core::entities::NextStepRecommendation;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonNextStep {
    pub priority: String,
    pub target_gate: String,
    pub command: String,
    pub description: String,
}

impl From<&NextStepRecommendation> for JsonNextStep {
    fn from(rec: &NextStepRecommendation) -> Self {
        Self {
            priority: rec.priority.as_str().to_string(),
            target_gate: rec.target_gate.clone(),
            command: rec.command.clone(),
            description: rec.description.clone(),
        }
    }
}

pub struct NextStepPresenter;

impl NextStepPresenter {
    pub fn to_json_vec(steps: &[NextStepRecommendation]) -> Vec<JsonNextStep> {
        steps.iter().map(JsonNextStep::from).collect()
    }

    pub fn render_ascii_block(steps: &[NextStepRecommendation]) -> String {
        let json_steps = Self::to_json_vec(steps);
        Self::render_json_steps_ascii_block(&json_steps)
    }

    pub fn render_json_steps_ascii_block(steps: &[JsonNextStep]) -> String {
        if steps.is_empty() {
            return String::new();
        }
        let mut out = String::new();
        out.push_str("\n------------------------------------------------------------\n");
        out.push_str("NEXT STEP:\n");
        for step in steps {
            out.push_str(&format!(
                "  [{}] ({}) {}\n",
                step.priority, step.target_gate, step.command
            ));
            out.push_str(&format!("            {}\n", step.description));
        }
        out.trim_end().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use trustkit_core::entities::NextStepResolver;

    #[test]
    fn test_next_step_presenter_json_and_ascii_invariants() {
        let recs = NextStepResolver::for_spec_lint("docs/specs/001/spec.md", true);
        let json_vec = NextStepPresenter::to_json_vec(&recs);
        assert_eq!(json_vec.len(), 1);
        assert_eq!(json_vec[0].priority, "PRIMARY");
        assert_eq!(json_vec[0].target_gate, "GATE-2");

        let serialized = serde_json::to_string(&json_vec).expect("serialize");
        assert!(!serialized.contains("null"));
        assert!(serialized.is_ascii());

        let ascii = NextStepPresenter::render_ascii_block(&recs);
        assert!(ascii.contains("NEXT STEP:"));
        assert!(
            ascii.contains("[PRIMARY] (GATE-2) trustkit spec lock --spec docs/specs/001/spec.md")
        );
        assert!(ascii.is_ascii());
    }
}
