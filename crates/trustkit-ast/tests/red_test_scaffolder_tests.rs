// Integration tests for TASK-005: RedTestScaffolder.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use std::fs;
use tempfile::tempdir;
use trustkit_ast::{RedTestScaffolder, ScaffoldingTarget};
use trustkit_core::errors::DomainViolation;

#[test]
fn test_scaffold_creates_compilable_test_file_when_missing() {
    let temp = tempdir().expect("tempdir");
    let target = ScaffoldingTarget {
        task_id: "TASK-005".to_string(),
        target_test_file: "crates/trustkit-ast/tests/example_scaffolded_tests.rs".to_string(),
        criteria_summary: "Deve criar arquivo compilavel com falha de assercao".to_string(),
    };

    let generated_path = RedTestScaffolder::scaffold(temp.path(), &target)
        .expect("scaffold should create new test file");

    assert!(generated_path.exists());
    let content = fs::read_to_string(&generated_path).expect("read file");
    assert!(content.contains("test_task_005_red_phase_contract"));
    assert!(content.contains("TASK RED ASSERTION: TASK-005"));
}

#[test]
fn test_scaffold_does_not_overwrite_existing_file() {
    let temp = tempdir().expect("tempdir");
    let relative_path = "tests/pre_existing_test.rs";
    let full_path = temp.path().join(relative_path);
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent).expect("mkdir");
    }
    fs::write(&full_path, "// custom pre-existing content").expect("write pre-existing");

    let target = ScaffoldingTarget {
        task_id: "TASK-005".to_string(),
        target_test_file: relative_path.to_string(),
        criteria_summary: "Do not overwrite".to_string(),
    };

    let result_path = RedTestScaffolder::scaffold(temp.path(), &target)
        .expect("should return existing file without failure");

    assert_eq!(result_path, full_path);
    let content = fs::read_to_string(&result_path).expect("read");
    assert_eq!(content, "// custom pre-existing content");
}

#[test]
fn test_scaffold_rejects_empty_identifiers() {
    let temp = tempdir().expect("tempdir");
    let empty_task = ScaffoldingTarget {
        task_id: "".to_string(),
        target_test_file: "tests/some_test.rs".to_string(),
        criteria_summary: "Empty task".to_string(),
    };
    let res1 = RedTestScaffolder::scaffold(temp.path(), &empty_task);
    assert!(matches!(
        res1,
        Err(DomainViolation::AstIntegrityOrInvariantViolation { .. })
    ));

    let empty_file = ScaffoldingTarget {
        task_id: "TASK-005".to_string(),
        target_test_file: "".to_string(),
        criteria_summary: "Empty file".to_string(),
    };
    let res2 = RedTestScaffolder::scaffold(temp.path(), &empty_file);
    assert!(matches!(
        res2,
        Err(DomainViolation::AstIntegrityOrInvariantViolation { .. })
    ));
}
