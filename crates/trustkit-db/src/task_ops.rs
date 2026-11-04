// Database operations for AtomicTask, TDD lifecycle and worktrees.
// Constitutional Principle II: Absolute Prohibition of NULL & Special Cases.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::row_mappers::{map_row_to_atomic_task, string_list_to_json};
use rusqlite::{Connection, params};
use std::collections::HashMap;
use trustkit_core::entities::{AtomicTask, TaskNode, WaveSchedule};
use trustkit_core::errors::DomainViolation;
use trustkit_core::special_cases::{TddLifecycleState, WorktreeAllocation};

pub fn save_atomic_tasks(
    conn: &mut Connection,
    cycle_id: &str,
    tasks: &[TaskNode],
    wave_schedule: &WaveSchedule,
) -> Result<(), DomainViolation> {
    let tx = conn
        .transaction()
        .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
            subsystem: "sqlite_adapter".into(),
            details: format!("Begin tx failed: {e}"),
        })?;
    let mut wave_map = HashMap::new();
    for wave in &wave_schedule.waves {
        for t in &wave.tasks {
            wave_map.insert(t.task_id.as_str(), wave.wave_index as i64);
        }
    }
    for task in tasks {
        let wave_index = wave_map.get(task.task_id.as_str()).copied().unwrap_or(1);
        tx.execute(
            "INSERT OR REPLACE INTO atomic_tasks (
                cycle_id, task_id, title, architectural_layer, wave_index,
                depends_on_json, expected_files_json, target_test_file,
                tdd_lifecycle_state, red_evidence_id, green_evidence_id,
                worktree_allocation, commit_sha
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'AWAITING_RED_PHASE', 0, 0, 'NOT_PROVISIONED', 'UNCOMMITTED')",
            params![
                cycle_id, task.task_id, task.title, task.architectural_layer,
                wave_index, string_list_to_json(&task.depends_on),
                string_list_to_json(&task.expected_files), task.target_test_file
            ],
        ).map_err(|e| DomainViolation::InfrastructureBoundaryFault {
            subsystem: "sqlite_adapter".into(),
            details: format!("Insert atomic task failed: {e}"),
        })?;
    }
    tx.execute(
        "UPDATE sdd_cycles SET current_gate = 3, updated_at = (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')) WHERE cycle_id = ?1",
        params![cycle_id],
    ).map_err(|e| DomainViolation::InfrastructureBoundaryFault {
        subsystem: "sqlite_adapter".into(),
        details: format!("Update cycle gate 3 failed: {e}"),
    })?;
    tx.commit()
        .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
            subsystem: "sqlite_adapter".into(),
            details: format!("Commit tx failed: {e}"),
        })?;
    Ok(())
}

pub fn list_atomic_tasks(
    conn: &Connection,
    cycle_id: &str,
) -> Result<Vec<AtomicTask>, DomainViolation> {
    let mut stmt = conn
        .prepare(
            "SELECT cycle_id, task_id, title, architectural_layer, wave_index,
                depends_on_json, expected_files_json, target_test_file,
                tdd_lifecycle_state, red_evidence_id, green_evidence_id,
                worktree_allocation, commit_sha
             FROM atomic_tasks WHERE cycle_id = ?1 ORDER BY wave_index ASC, task_id ASC",
        )
        .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
            subsystem: "sqlite_adapter".into(),
            details: format!("Prepare failed: {e}"),
        })?;
    let rows = stmt
        .query_map(params![cycle_id], map_row_to_atomic_task)
        .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
            subsystem: "sqlite_adapter".into(),
            details: format!("Query failed: {e}"),
        })?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
            subsystem: "sqlite_adapter".into(),
            details: format!("Row error: {e}"),
        })
}

pub fn find_atomic_task(
    conn: &Connection,
    cycle_id: &str,
    task_id: &str,
) -> Result<Option<AtomicTask>, DomainViolation> {
    let mut stmt = conn
        .prepare(
            "SELECT cycle_id, task_id, title, architectural_layer, wave_index,
                depends_on_json, expected_files_json, target_test_file,
                tdd_lifecycle_state, red_evidence_id, green_evidence_id,
                worktree_allocation, commit_sha
             FROM atomic_tasks WHERE cycle_id = ?1 AND task_id = ?2 LIMIT 1",
        )
        .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
            subsystem: "sqlite_adapter".into(),
            details: format!("Prepare failed: {e}"),
        })?;
    let mut rows = stmt.query(params![cycle_id, task_id]).map_err(|e| {
        DomainViolation::InfrastructureBoundaryFault {
            subsystem: "sqlite_adapter".into(),
            details: format!("Query failed: {e}"),
        }
    })?;
    if let Some(r) = rows
        .next()
        .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
            subsystem: "sqlite_adapter".into(),
            details: format!("Row error: {e}"),
        })?
    {
        let task = map_row_to_atomic_task(r).map_err(|e| {
            DomainViolation::InfrastructureBoundaryFault {
                subsystem: "sqlite_adapter".into(),
                details: format!("Map atomic task error: {e}"),
            }
        })?;
        Ok(Some(task))
    } else {
        Ok(None)
    }
}

pub fn update_task_tdd_phase(
    conn: &Connection,
    cycle_id: &str,
    task_id: &str,
    state: &TddLifecycleState,
) -> Result<(), DomainViolation> {
    let rows = match state {
        TddLifecycleState::AwaitingRedPhase => conn.execute(
            "UPDATE atomic_tasks
             SET tdd_lifecycle_state = 'AWAITING_RED_PHASE', red_evidence_id = 0, green_evidence_id = 0, commit_sha = 'UNCOMMITTED'
             WHERE cycle_id = ?1 AND task_id = ?2",
            params![cycle_id, task_id],
        ),
        TddLifecycleState::RedVerified { failing_evidence_id } => conn.execute(
            "UPDATE atomic_tasks
             SET tdd_lifecycle_state = 'RED_VERIFIED', red_evidence_id = ?1
             WHERE cycle_id = ?2 AND task_id = ?3",
            params![*failing_evidence_id as i64, cycle_id, task_id],
        ),
        TddLifecycleState::GreenVerified { red_evidence_id, green_evidence_id } => conn.execute(
            "UPDATE atomic_tasks
             SET tdd_lifecycle_state = 'GREEN_VERIFIED', red_evidence_id = ?1, green_evidence_id = ?2
             WHERE cycle_id = ?3 AND task_id = ?4",
            params![*red_evidence_id as i64, *green_evidence_id as i64, cycle_id, task_id],
        ),
        TddLifecycleState::Committed { commit_sha } => conn.execute(
            "UPDATE atomic_tasks
             SET tdd_lifecycle_state = 'COMMITTED', commit_sha = ?1
             WHERE cycle_id = ?2 AND task_id = ?3",
            params![commit_sha.as_str(), cycle_id, task_id],
        ),
    }.map_err(|e| DomainViolation::InfrastructureBoundaryFault {
        subsystem: "sqlite_adapter".into(),
        details: format!("Update task tdd phase failed: {e}"),
    })?;

    if rows == 0 {
        return Err(DomainViolation::InfrastructureBoundaryFault {
            subsystem: "sqlite_adapter".into(),
            details: format!("Task {task_id} not found in cycle {cycle_id}"),
        });
    }
    Ok(())
}

pub fn update_task_worktree(
    conn: &Connection,
    cycle_id: &str,
    task_id: &str,
    allocation: &WorktreeAllocation,
) -> Result<(), DomainViolation> {
    let (alloc_str, commit_sha) = match allocation {
        WorktreeAllocation::NotProvisioned => ("NOT_PROVISIONED".to_string(), None),
        WorktreeAllocation::Provisioned {
            worktree_path,
            branch_name,
        } => (format!("PROVISIONED:{worktree_path}:{branch_name}"), None),
        WorktreeAllocation::MergedAndPruned { commit_sha } => {
            (format!("MERGED:{commit_sha}"), Some(commit_sha.as_str()))
        }
    };

    let rows = match commit_sha {
        Some(sha) => conn.execute(
            "UPDATE atomic_tasks
             SET worktree_allocation = ?1, commit_sha = ?2
             WHERE cycle_id = ?3 AND task_id = ?4",
            params![alloc_str, sha, cycle_id, task_id],
        ),
        None => conn.execute(
            "UPDATE atomic_tasks
             SET worktree_allocation = ?1
             WHERE cycle_id = ?2 AND task_id = ?3",
            params![alloc_str, cycle_id, task_id],
        ),
    }
    .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
        subsystem: "sqlite_adapter".into(),
        details: format!("Update task worktree failed: {e}"),
    })?;

    if rows == 0 {
        return Err(DomainViolation::InfrastructureBoundaryFault {
            subsystem: "sqlite_adapter".into(),
            details: format!("Task {task_id} not found in cycle {cycle_id}"),
        });
    }
    Ok(())
}
