// Data transfer objects and value objects for domain ports.
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::special_cases::BoardColumn;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecValidationReport {
    pub mermaid_valid: bool,
    pub gherkin_valid: bool,
    pub orphan_rules: Vec<String>,
    pub total_rules: usize,
    pub total_requirements: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PullRequestDetails {
    pub issue_number: u64,
    pub title: String,
    pub body: String,
    pub base_branch: String,
    pub head_branch: String,
    pub reviewers: Vec<String>,
    pub assignees: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PullRequestOutcome {
    pub pr_number: u64,
    pub pr_url: String,
    pub target_column: BoardColumn,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorktreeStatus {
    pub worktree_path: String,
    pub branch_name: String,
    pub is_clean: bool,
    pub head_commit_sha: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupervisedExecutionRequest {
    pub command: String,
    pub args: Vec<String>,
    pub working_dir: String,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupervisedExecutionOutcome {
    pub exit_code: i32,
    pub duration_ms: u64,
    pub stdout_tail: String,
    pub stderr_tail: String,
    pub output_blake3_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForeignScreenshotInspection {
    pub active_issue_prefix: String,
    pub foreign_screenshots_detected: Vec<String>,
    pub restored_screenshots: Vec<String>,
}
