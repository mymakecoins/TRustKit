use std::fs;
use tempfile::tempdir;
use trustkit_ast::WaveExecutionPlanner;
use trustkit_core::entities::TaskNode;
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::WaveExecutionPlannerPort;

#[test]
fn test_plan_waves_simple_dag() {
    let planner = WaveExecutionPlanner;
    let tasks = vec![
        TaskNode {
            task_id: "TASK-001".to_string(),
            title: "Core Domain".to_string(),
            architectural_layer: "CORE".to_string(),
            depends_on: vec![],
            expected_files: vec!["crates/core/src/lib.rs".to_string()],
            target_test_file: "tests/core_test.rs".to_string(),
        },
        TaskNode {
            task_id: "TASK-002".to_string(),
            title: "Database Adapter".to_string(),
            architectural_layer: "DB".to_string(),
            depends_on: vec!["TASK-001".to_string()],
            expected_files: vec!["crates/db/src/lib.rs".to_string()],
            target_test_file: "tests/db_test.rs".to_string(),
        },
        TaskNode {
            task_id: "TASK-003".to_string(),
            title: "AST Parser".to_string(),
            architectural_layer: "AST".to_string(),
            depends_on: vec!["TASK-001".to_string()],
            expected_files: vec!["crates/ast/src/lib.rs".to_string()],
            target_test_file: "tests/ast_test.rs".to_string(),
        },
    ];

    let schedule = planner
        .plan_waves(&tasks)
        .expect("DAG planning should succeed");
    assert_eq!(schedule.total_tasks, 3);
    assert_eq!(schedule.waves.len(), 2);

    assert_eq!(schedule.waves[0].wave_index, 1);
    assert_eq!(schedule.waves[0].tasks.len(), 1);
    assert_eq!(schedule.waves[0].tasks[0].task_id, "TASK-001");

    assert_eq!(schedule.waves[1].wave_index, 2);
    assert_eq!(schedule.waves[1].tasks.len(), 2);
    let wave_2_ids: Vec<&str> = schedule.waves[1]
        .tasks
        .iter()
        .map(|t| t.task_id.as_str())
        .collect();
    assert!(wave_2_ids.contains(&"TASK-002"));
    assert!(wave_2_ids.contains(&"TASK-003"));
}

#[test]
fn test_plan_waves_detects_cycle() {
    let planner = WaveExecutionPlanner;
    let tasks = vec![
        TaskNode {
            task_id: "TASK-001".to_string(),
            title: "Task 1".to_string(),
            architectural_layer: "CORE".to_string(),
            depends_on: vec!["TASK-002".to_string()],
            expected_files: vec!["crates/a.rs".to_string()],
            target_test_file: "tests/a_test.rs".to_string(),
        },
        TaskNode {
            task_id: "TASK-002".to_string(),
            title: "Task 2".to_string(),
            architectural_layer: "CORE".to_string(),
            depends_on: vec!["TASK-001".to_string()],
            expected_files: vec!["crates/b.rs".to_string()],
            target_test_file: "tests/b_test.rs".to_string(),
        },
    ];

    let result = planner.plan_waves(&tasks);
    match result {
        Err(DomainViolation::CycleDetected {
            cycle_path,
            message,
        }) => {
            assert!(cycle_path.contains(&"TASK-001".to_string()));
            assert!(cycle_path.contains(&"TASK-002".to_string()));
            assert!(message.contains("TASK-001") || message.contains("circular"));
        }
        other => panic!("Expected DomainViolation::CycleDetected, got: {other:?}"),
    }
}

#[test]
fn test_plan_waves_shared_files_isolated_by_task_number() {
    let planner = WaveExecutionPlanner;
    let tasks = vec![
        TaskNode {
            task_id: "TASK-001".to_string(),
            title: "Modelo de Dados".to_string(),
            architectural_layer: "CORE".to_string(),
            depends_on: vec![],
            expected_files: vec!["src/auth.rs".to_string()],
            target_test_file: "".to_string(),
        },
        TaskNode {
            task_id: "TASK-002".to_string(),
            title: "Validador TOTP".to_string(),
            architectural_layer: "CORE".to_string(),
            depends_on: vec!["TASK-001".to_string()],
            expected_files: vec!["src/totp.rs".to_string()],
            target_test_file: "".to_string(),
        },
        TaskNode {
            task_id: "TASK-003".to_string(),
            title: "Logger de Auditoria".to_string(),
            architectural_layer: "CORE".to_string(),
            depends_on: vec![],
            expected_files: vec!["src/auth.rs".to_string()],
            target_test_file: "".to_string(),
        },
    ];

    let schedule = planner
        .plan_waves(&tasks)
        .expect("Should resolve shared file conflicts deterministically");

    // TASK-001 must precede TASK-003 because both touch src/auth.rs and 001 < 003
    let mut wave_of_001 = 0;
    let mut wave_of_003 = 0;
    for wave in &schedule.waves {
        for t in &wave.tasks {
            if t.task_id == "TASK-001" {
                wave_of_001 = wave.wave_index;
            }
            if t.task_id == "TASK-003" {
                wave_of_003 = wave.wave_index;
            }
        }
    }

    assert!(wave_of_001 > 0);
    assert!(wave_of_003 > 0);
    assert!(
        wave_of_001 < wave_of_003,
        "TASK-001 must be in an earlier wave than TASK-003 due to shared file src/auth.rs"
    );
}

#[test]
fn test_parse_tasks_dir_and_plan() {
    let dir = tempdir().unwrap();
    let tasks_dir = dir.path();

    let task1_content = r#"# TASK-001: Data Model
depends_on: []
Arquivos Esperados:
- src/auth.rs
"#;
    let task2_content = r#"# TASK-002: TOTP Validator
depends_on:
- TASK-001
Arquivos Esperados:
- src/totp.rs
"#;
    let task3_content = r#"# TASK-003: Audit Logger
depends_on: []
Arquivos Esperados:
- src/auth.rs
"#;

    fs::write(tasks_dir.join("TASK-001.md"), task1_content).unwrap();
    fs::write(tasks_dir.join("TASK-002.md"), task2_content).unwrap();
    fs::write(tasks_dir.join("TASK-003.md"), task3_content).unwrap();

    let tasks =
        WaveExecutionPlanner::parse_tasks_dir(tasks_dir).expect("Parsing tasks dir should succeed");
    assert_eq!(tasks.len(), 3);

    let planner = WaveExecutionPlanner;
    let schedule = planner
        .plan_waves(&tasks)
        .expect("Planning waves should succeed");
    assert_eq!(schedule.total_tasks, 3);
}
