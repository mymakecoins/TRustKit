use std::fs;
use tempfile::tempdir;
use trustkit_ast::WaveExecutionPlanner;
use trustkit_core::entities::{TaskNode, TaskSliceKind};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::WaveExecutionPlannerPort;

fn make_task(id: &str, layer: &str, deps: &[&str], files: &[&str]) -> TaskNode {
    TaskNode::new(
        id.to_string(),
        format!("Task {id}"),
        layer.to_string(),
        deps.iter().map(|s| s.to_string()).collect(),
        files.iter().map(|s| s.to_string()).collect(),
        format!("tests/{id}_test.rs"),
    )
}

#[test]
fn test_plan_waves_simple_dag() {
    let planner = WaveExecutionPlanner;
    let tasks = vec![
        make_task("TASK-001", "CORE", &[], &["crates/core/src/lib.rs"]),
        make_task("TASK-002", "DB", &["TASK-001"], &["crates/db/src/lib.rs"]),
        make_task("TASK-003", "AST", &["TASK-001"], &["crates/ast/src/lib.rs"]),
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
        make_task("TASK-001", "CORE", &["TASK-002"], &["crates/a.rs"]),
        make_task("TASK-002", "CORE", &["TASK-001"], &["crates/b.rs"]),
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
        make_task("TASK-001", "CORE", &[], &["src/auth.rs"]),
        make_task("TASK-002", "CORE", &["TASK-001"], &["src/totp.rs"]),
        make_task("TASK-003", "CORE", &[], &["src/auth.rs"]),
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

#[test]
fn test_wave_planner_expand_migrate_contract_strict_dag_ordering() {
    let t_expand = make_task("TASK-001", "CORE", &[], &["crates/ports.rs"])
        .with_slice_kind(TaskSliceKind::WideRefactorExpand);
    let t_migrate_1 = make_task("TASK-002", "CORE", &["TASK-001"], &["crates/adapter_a.rs"])
        .with_slice_kind(TaskSliceKind::WideRefactorMigrateBatch);
    let t_migrate_2 = make_task("TASK-003", "CORE", &["TASK-001"], &["crates/adapter_b.rs"])
        .with_slice_kind(TaskSliceKind::WideRefactorMigrateBatch);
    let t_contract = make_task(
        "TASK-004",
        "CORE",
        &["TASK-002", "TASK-003"],
        &["crates/ports.rs"],
    )
    .with_slice_kind(TaskSliceKind::WideRefactorContract);

    let tasks = vec![t_contract, t_migrate_1, t_expand, t_migrate_2];

    let planner = WaveExecutionPlanner;
    let schedule = planner.plan_waves(&tasks).expect("Should succeed");

    assert_eq!(schedule.waves.len(), 3);
    assert_eq!(schedule.waves[0].tasks.len(), 1);
    assert_eq!(schedule.waves[0].tasks[0].task_id, "TASK-001");
    assert_eq!(
        schedule.waves[0].tasks[0].slice_kind,
        TaskSliceKind::WideRefactorExpand
    );

    assert_eq!(schedule.waves[1].tasks.len(), 2);
    let wave_1_ids: Vec<&str> = schedule.waves[1]
        .tasks
        .iter()
        .map(|t| t.task_id.as_str())
        .collect();
    assert!(wave_1_ids.contains(&"TASK-002"));
    assert!(wave_1_ids.contains(&"TASK-003"));

    assert_eq!(schedule.waves[2].tasks.len(), 1);
    assert_eq!(schedule.waves[2].tasks[0].task_id, "TASK-004");
    assert_eq!(
        schedule.waves[2].tasks[0].slice_kind,
        TaskSliceKind::WideRefactorContract
    );
}
