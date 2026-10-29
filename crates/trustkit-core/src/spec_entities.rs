// Domain entities representing specifications and planning.
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle VI: Ubiquitous Language & Focused Modular Clean Code (< 250 LOC).

use crate::special_cases::SpecLockState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BusinessRule {
    pub rule_id: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionalRequirement {
    pub req_id: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpstreamDocument {
    pub path: String,
    pub title: String,
    pub topic_slug: String,
    pub business_rules: Vec<BusinessRule>,
    pub functional_requirements: Vec<FunctionalRequirement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GherkinScenario {
    pub name: String,
    pub given: Vec<String>,
    pub when: Vec<String>,
    pub then: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceabilityRow {
    pub business_rule_id: String,
    pub functional_req_id: String,
    pub task_id: String,
    pub mermaid_node: String,
    pub playwright_spec_path: String,
    pub evidence_screenshot: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecificationV2 {
    pub cycle_id: String,
    pub feature_slug: String,
    pub spec_file_path: String,
    pub business_rules: Vec<BusinessRule>,
    pub functional_requirements: Vec<FunctionalRequirement>,
    pub mermaid_content: String,
    pub gherkin_scenarios: Vec<GherkinScenario>,
    pub traceability_matrix: Vec<TraceabilityRow>,
    pub lock_state: SpecLockState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskNode {
    pub task_id: String,
    pub title: String,
    pub architectural_layer: String,
    pub depends_on: Vec<String>,
    pub expected_files: Vec<String>,
    pub target_test_file: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionWave {
    pub wave_index: usize,
    pub tasks: Vec<TaskNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaveSchedule {
    pub cycle_id: String,
    pub total_tasks: usize,
    pub waves: Vec<ExecutionWave>,
}
