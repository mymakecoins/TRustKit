// Acceptance tests for deterministic task synthesis on filesystem (TASK-002, RN-002, RF-002).
// Zero Emojis and Zero AI Trailers.

use std::fs;
use tempfile::tempdir;
use trustkit_ast::task_parser;
use trustkit_core::entities::AtomicTaskDraft;
use trustkit_orch::TaskSynthesizer;

fn sample_drafts() -> Vec<AtomicTaskDraft> {
    vec![
        AtomicTaskDraft {
            task_id: "TASK-001".to_string(),
            title: "Matrix AST Atomizer".to_string(),
            architectural_layer: "AST".to_string(),
            depends_on: vec![],
            expected_files: vec![
                "crates/trustkit-ast/src/spec_matrix_atomizer.rs".to_string(),
                "crates/trustkit-ast/tests/spec_matrix_atomizer_tests.rs".to_string(),
            ],
            target_test_file: "crates/trustkit-ast/tests/spec_matrix_atomizer_tests.rs".to_string(),
            context_summary: "Extract atomic tasks from traceability matrix.".to_string(),
            given_clause: "a valid markdown spec".to_string(),
            when_clause: "spec atomizer runs".to_string(),
            then_clause: "drafts are produced".to_string(),
            business_rule_id: "RN-001".to_string(),
            functional_req_id: "RF-001".to_string(),
            mermaid_node: "AstMatrixParser".to_string(),
            evidence_artifact: "EV-001-spec-matrix-ast-atomization.json".to_string(),
        },
        AtomicTaskDraft {
            task_id: "TASK-002".to_string(),
            title: "Task Synthesizer".to_string(),
            architectural_layer: "ORCH".to_string(),
            depends_on: vec!["TASK-001".to_string()],
            expected_files: vec![
                "crates/trustkit-orch/src/task_synthesizer.rs".to_string(),
                "crates/trustkit-orch/tests/task_synthesis_fs_tests.rs".to_string(),
            ],
            target_test_file: "crates/trustkit-orch/tests/task_synthesis_fs_tests.rs".to_string(),
            context_summary: "Synthesize tasks without overwriting existing files.".to_string(),
            given_clause: "a list of drafts and clean cycle dir".to_string(),
            when_clause: "materialize_tasks_dir is called".to_string(),
            then_clause: "new files are created on disk".to_string(),
            business_rule_id: "RN-002".to_string(),
            functional_req_id: "RF-002".to_string(),
            mermaid_node: "TaskSynthesizer".to_string(),
            evidence_artifact: "EV-002-task-file-synthesis.json".to_string(),
        },
    ]
}

#[test]
fn test_materialize_creates_tasks_dir_and_files() {
    let temp = tempdir().expect("tempdir");
    let cycle_dir = temp.path();
    let drafts = sample_drafts();

    let created =
        TaskSynthesizer::materialize_tasks_dir(cycle_dir, &drafts).expect("synthesis success");

    assert_eq!(created.len(), 2);
    let tasks_dir = cycle_dir.join("tasks");
    assert!(tasks_dir.exists());
    assert!(tasks_dir.join("TASK-001.md").exists());
    assert!(tasks_dir.join("TASK-002.md").exists());

    // Verify task_parser can parse the generated directory
    let parsed = task_parser::parse_tasks_dir(&tasks_dir).expect("parse tasks dir");
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].task_id, "TASK-001");
    assert_eq!(parsed[0].architectural_layer, "AST");
    assert_eq!(parsed[1].task_id, "TASK-002");
    assert_eq!(parsed[1].depends_on, vec!["TASK-001"]);
}

#[test]
fn test_materialize_preserves_existing_custom_tasks_without_overwriting() {
    let temp = tempdir().expect("tempdir");
    let cycle_dir = temp.path();
    let tasks_dir = cycle_dir.join("tasks");
    fs::create_dir_all(&tasks_dir).expect("create tasks dir");

    let custom_task_1 = "# TASK-001: Custom Pre-existing Task\n\nlayer: CUSTOM\ndepends_on: []\ntarget_test_file: custom_test.rs\nexpected_files:\n  - custom.rs\n";
    fs::write(tasks_dir.join("TASK-001.md"), custom_task_1).expect("write custom task");

    let drafts = sample_drafts();
    let report = TaskSynthesizer::synthesize(cycle_dir, &drafts).expect("synthesize");

    assert_eq!(report.created_count, 1);
    assert_eq!(report.preserved_count, 1);

    // Verify TASK-001 content was preserved exactly
    let content_001 = fs::read_to_string(tasks_dir.join("TASK-001.md")).expect("read file");
    assert_eq!(content_001, custom_task_1);

    // Verify TASK-002 was newly created
    assert!(tasks_dir.join("TASK-002.md").exists());
}
