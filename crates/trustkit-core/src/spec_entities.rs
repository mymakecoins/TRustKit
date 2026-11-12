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

use crate::ast_entities::TaskSliceKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskNode {
    pub task_id: String,
    pub title: String,
    pub architectural_layer: String,
    pub depends_on: Vec<String>,
    pub expected_files: Vec<String>,
    pub target_test_file: String,
    pub slice_kind: TaskSliceKind,
    pub blast_radius_score: u32,
}

impl TaskNode {
    pub fn new(
        task_id: String,
        title: String,
        architectural_layer: String,
        depends_on: Vec<String>,
        expected_files: Vec<String>,
        target_test_file: String,
    ) -> Self {
        Self {
            task_id,
            title,
            architectural_layer,
            depends_on,
            expected_files,
            target_test_file,
            slice_kind: TaskSliceKind::VerticalTracerBullet,
            blast_radius_score: 0,
        }
    }

    pub fn with_slice_kind(mut self, slice_kind: TaskSliceKind) -> Self {
        self.slice_kind = slice_kind;
        self
    }

    pub fn with_blast_radius(mut self, blast_radius_score: u32) -> Self {
        self.blast_radius_score = blast_radius_score;
        self
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AtomizationSource {
    SpecificationMarkdown {
        spec_path: String,
        markdown_content: String,
    },
    GitHubIssueBody {
        issue_number: u64,
        title: String,
        body_markdown: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomicTaskDraft {
    pub task_id: String,
    pub title: String,
    pub architectural_layer: String,
    pub depends_on: Vec<String>,
    pub expected_files: Vec<String>,
    pub target_test_file: String,
    pub context_summary: String,
    pub given_clause: String,
    pub when_clause: String,
    pub then_clause: String,
    pub business_rule_id: String,
    pub functional_req_id: String,
    pub mermaid_node: String,
    pub evidence_artifact: String,
}

impl AtomicTaskDraft {
    pub fn to_task_node(&self) -> TaskNode {
        TaskNode::new(
            self.task_id.clone(),
            self.title.clone(),
            self.architectural_layer.clone(),
            self.depends_on.clone(),
            self.expected_files.clone(),
            self.target_test_file.clone(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedTestScaffold {
    pub task_id: String,
    pub target_test_file: String,
    pub generated_rust_code: String,
}
