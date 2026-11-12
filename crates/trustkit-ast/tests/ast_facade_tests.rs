// Tests for AST facade adapter (TASK-009).

use std::fs::File;
use std::io::Write;
use tempfile::tempdir;
use trustkit_ast::TreeSitterAstAuditorAdapter;
use trustkit_core::entities::{
    ArtifactCategory, ArtifactSeal, AstViolation, ContextSliceRequest, TestIntegrityViolation,
};
use trustkit_core::ports::{AstAuditorPort, ContextSlicerPort};
use trustkit_core::special_cases::ContractUnlockState;

#[test]
fn test_facade_audit_source_tree_clean_pass() {
    let dir = tempdir().expect("Failed to create tempdir");
    let file_path = dir.path().join("clean.rs");
    let mut file = File::create(&file_path).expect("Failed to create file");
    writeln!(file, "pub fn hello() -> &'static str {{\n    \"world\"\n}}").unwrap();

    let adapter = TreeSitterAstAuditorAdapter::new();
    let files = vec![file_path.to_str().unwrap().to_string()];
    let verdict = adapter
        .audit_source_tree(&files, &[])
        .expect("Audit failed");

    assert!(verdict.is_pass());
    assert!(verdict.violations().is_empty());
}

#[test]
fn test_facade_audit_source_tree_with_emoji_violation() {
    let dir = tempdir().expect("Failed to create tempdir");
    let file_path = dir.path().join("emoji.rs");
    let mut file = File::create(&file_path).expect("Failed to create file");
    writeln!(file, "// Error: rocket 🚀\npub fn hello() {{}}").unwrap();

    let adapter = TreeSitterAstAuditorAdapter::new();
    let files = vec![file_path.to_str().unwrap().to_string()];
    let verdict = adapter
        .audit_source_tree(&files, &[])
        .expect("Audit failed");

    assert!(!verdict.is_pass());
    let violations = verdict.violations();
    assert!(!violations.is_empty());
    assert!(
        violations
            .iter()
            .any(|v| matches!(v, AstViolation::ForbiddenEmoji { .. }))
    );
}

#[test]
fn test_facade_audit_source_tree_with_test_dilution() {
    let dir = tempdir().expect("Failed to create tempdir");
    let file_path = dir.path().join("test_diluted.rs");
    let mut file = File::create(&file_path).expect("Failed to create file");
    writeln!(
        file,
        r#"
#[test]
fn test_something() {{
    assert_eq!(1, 1);
}}
"#
    )
    .unwrap();

    let path_str = file_path.to_str().unwrap().to_string();
    let seal = ArtifactSeal {
        seal_id: 1,
        cycle_id: "test-cycle".to_string(),
        file_path: path_str.clone(),
        artifact_category: ArtifactCategory::PreExistingTest,
        blake3_digest: "dummy".to_string(),
        ast_assertion_count: 5,
        contract_unlock_state: ContractUnlockState::Sealed,
    };

    let adapter = TreeSitterAstAuditorAdapter::new();
    let verdict = adapter
        .audit_source_tree(&[path_str], &[seal])
        .expect("Audit failed");

    assert!(!verdict.is_pass());
    let violations = verdict.violations();
    assert!(violations.iter().any(|v| matches!(
        v,
        AstViolation::TestDilution(TestIntegrityViolation {
            expected_assertions: 5,
            actual_assertions: 1,
            ..
        })
    )));
}

#[test]
fn test_facade_slice_context() {
    let dir = tempdir().expect("Failed to create tempdir");
    let file_path = dir.path().join("service.rs");
    let mut file = File::create(&file_path).expect("Failed to create file");
    writeln!(
        file,
        r#"
pub struct Service {{
    pub name: String,
}}

impl Service {{
    pub fn execute(&self) -> bool {{
        true
    }}
}}
"#
    )
    .unwrap();

    let adapter = TreeSitterAstAuditorAdapter::new();
    let request = ContextSliceRequest {
        target_files: vec![file_path.to_str().unwrap().to_string()],
        max_line_budget: 10,
        symbol_filters: vec!["Service".to_string()],
    };

    let result = adapter.slice_context(&request).expect("Slice failed");
    assert!(result.total_lines > 0);
    assert!(result.total_lines <= 10);
    assert!(!result.sliced_content.is_empty());
}

#[test]
fn test_wave_planner_rejects_contract_without_all_migrate_dependencies() {
    use trustkit_ast::WaveExecutionPlanner;
    use trustkit_core::entities::{TaskNode, TaskSliceKind};
    use trustkit_core::errors::DomainViolation;

    let t_expand = TaskNode::new(
        "TASK-001".to_string(),
        "Expand ports".to_string(),
        "CORE".to_string(),
        vec![],
        vec!["crates/ports.rs".to_string()],
        "tests/t1.rs".to_string(),
    )
    .with_slice_kind(TaskSliceKind::WideRefactorExpand);

    let t_migrate_1 = TaskNode::new(
        "TASK-002".to_string(),
        "Migrate batch 1".to_string(),
        "CORE".to_string(),
        vec!["TASK-001".to_string()],
        vec!["crates/a.rs".to_string()],
        "tests/t2.rs".to_string(),
    )
    .with_slice_kind(TaskSliceKind::WideRefactorMigrateBatch);

    let t_migrate_2 = TaskNode::new(
        "TASK-003".to_string(),
        "Migrate batch 2".to_string(),
        "CORE".to_string(),
        vec!["TASK-001".to_string()],
        vec!["crates/b.rs".to_string()],
        "tests/t3.rs".to_string(),
    )
    .with_slice_kind(TaskSliceKind::WideRefactorMigrateBatch);

    // Contract missing dependency on TASK-003!
    let t_contract = TaskNode::new(
        "TASK-004".to_string(),
        "Contract old port".to_string(),
        "CORE".to_string(),
        vec!["TASK-002".to_string()],
        vec!["crates/ports.rs".to_string()],
        "tests/t4.rs".to_string(),
    )
    .with_slice_kind(TaskSliceKind::WideRefactorContract);

    let tasks = vec![t_expand, t_migrate_1, t_migrate_2, t_contract];

    let result = WaveExecutionPlanner::plan_with_options(&tasks, &[], 8);
    match result {
        Err(DomainViolation::AstIntegrityOrInvariantViolation {
            invariant_id,
            reason,
            ..
        }) => {
            assert_eq!(invariant_id, "RN-004");
            assert!(reason.contains("TASK-003"));
        }
        other => {
            panic!("Expected RN-004 violation for contract missing migrate dep, got: {other:?}")
        }
    }
}
