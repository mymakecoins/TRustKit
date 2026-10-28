// SQLite WAL repository adapter.
// Constitutional Principle II: Absolute Prohibition of NULL & Special Cases.
// Constitutional Principle IV: Resilient Error Handling & Zero-Panic Policy.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::migrations::initialize_database;
use crate::row_mappers::{
    map_row_to_artifact_seal, map_row_to_atomic_task, map_row_to_cycle, map_row_to_evidence,
    string_list_to_json,
};
use rusqlite::{Connection, params};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use trustkit_core::entities::{
    ArtifactCategory, ArtifactSeal, AtomicTask, CycleStatus, EvidenceKind, ExecutionEvidence,
    GateVerdict, MethodologyProfile, SddCycle, TaskNode, WaveSchedule,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::CycleEvidenceRepositoryPort;
use trustkit_core::special_cases::{
    ActiveCycleLookup, ContractUnlockState, IssueLinkage, SpecLockState,
};

pub struct SqliteWalRepositoryAdapter {
    conn: Mutex<Connection>,
}

impl SqliteWalRepositoryAdapter {
    pub fn open(db_path: &Path) -> Result<Self, DomainViolation> {
        if let Some(parent) = db_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let conn = Connection::open(db_path).map_err(|e| {
            DomainViolation::InfrastructureBoundaryFault {
                subsystem: "sqlite_adapter".into(),
                details: format!("Failed opening SQLite db: {e}"),
            }
        })?;
        initialize_database(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn lock_conn(&self) -> Result<std::sync::MutexGuard<'_, Connection>, DomainViolation> {
        self.conn
            .lock()
            .map_err(|_| DomainViolation::InfrastructureBoundaryFault {
                subsystem: "sqlite_adapter".into(),
                details: "Poisoned SQLite mutex".into(),
            })
    }

    pub fn list_atomic_tasks(&self, cycle_id: &str) -> Result<Vec<AtomicTask>, DomainViolation> {
        let conn = self.lock_conn()?;
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
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| {
            DomainViolation::InfrastructureBoundaryFault {
                subsystem: "sqlite_adapter".into(),
                details: format!("Row error: {e}"),
            }
        })
    }
}

impl CycleEvidenceRepositoryPort for SqliteWalRepositoryAdapter {
    fn save_cycle(&self, c: &SddCycle) -> Result<(), DomainViolation> {
        let conn = self.lock_conn()?;
        let (kind, num, url, board_id, item_id, col) = match &c.issue_linkage {
            IssueLinkage::LinkedIssue {
                issue_number,
                issue_url,
                project_board_id,
                project_item_id,
                board_column,
            } => (
                "LINKED_ISSUE",
                *issue_number as i64,
                issue_url.as_str(),
                project_board_id.as_str(),
                project_item_id.as_str(),
                board_column.as_str(),
            ),
            IssueLinkage::UnlinkedLocalCycle { .. } => (
                "UNLINKED_LOCAL_CYCLE",
                0i64,
                "UNLINKED_LOCAL_CYCLE",
                "UNLINKED_LOCAL_CYCLE",
                "UNLINKED_LOCAL_CYCLE",
                "UNLINKED_LOCAL_CYCLE",
            ),
        };
        let methodology = match &c.methodology_profile {
            MethodologyProfile::StandardGauntletV2 => "STANDARD_GAUNTLET_V2",
            MethodologyProfile::SpecKitIntegrated { .. } => "SPECKIT_INTEGRATED",
        };
        let status = match c.lifecycle_status {
            CycleStatus::Active => "ACTIVE",
            CycleStatus::Paused => "PAUSED",
            CycleStatus::CircuitBreakerTripped => "CIRCUIT_BREAKER_TRIPPED",
            CycleStatus::Certified => "CERTIFIED",
            CycleStatus::Merged => "MERGED",
        };
        let (lock_state, blake3) = match &c.spec_seal_state {
            SpecLockState::DraftUnverified => ("DRAFT_UNVERIFIED", "UNSEALED_DRAFT"),
            SpecLockState::Frozen { blake3_digest, .. } => ("FROZEN", blake3_digest.as_str()),
        };
        conn.execute(
            "INSERT OR REPLACE INTO sdd_cycles (
                cycle_id, issue_linkage_kind, issue_number, issue_url, project_board_id, project_item_id, board_column,
                methodology_profile, current_gate, lifecycle_status, gauntlet_attempt, spec_dir, spec_file_path,
                spec_lock_state, spec_blake3_digest, work_branch, base_commit_sha, pre_existing_dirty_json, created_at, updated_at
             ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20)",
            params![
                c.cycle_id, kind, num, url, board_id, item_id, col, methodology,
                c.current_gate.as_u8() as i64, status, c.gauntlet_attempt as i64,
                c.spec_dir, c.spec_file_path, lock_state, blake3, c.work_branch,
                c.base_commit_sha, string_list_to_json(&c.pre_existing_dirty_files), c.created_at, c.updated_at,
            ],
        ).map_err(|e| DomainViolation::InfrastructureBoundaryFault {
            subsystem: "sqlite_adapter".into(), details: format!("Insert cycle failed: {e}"),
        })?;
        Ok(())
    }

    fn find_active_cycle(&self) -> Result<ActiveCycleLookup, DomainViolation> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT cycle_id, issue_linkage_kind, issue_number, issue_url, project_board_id, project_item_id, board_column,
                    methodology_profile, current_gate, lifecycle_status, gauntlet_attempt, spec_dir, spec_file_path,
                    spec_lock_state, spec_blake3_digest, work_branch, base_commit_sha, pre_existing_dirty_json, created_at, updated_at
             FROM sdd_cycles WHERE lifecycle_status = 'ACTIVE' LIMIT 1"
        ).map_err(|e| DomainViolation::InfrastructureBoundaryFault { subsystem: "sqlite_adapter".into(), details: format!("Prepare failed: {e}") })?;
        let mut rows =
            stmt.query([])
                .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
                    subsystem: "sqlite_adapter".into(),
                    details: format!("Query failed: {e}"),
                })?;
        if let Some(r) = rows
            .next()
            .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
                subsystem: "sqlite_adapter".into(),
                details: format!("Row fetch failed: {e}"),
            })?
        {
            let cycle =
                map_row_to_cycle(r).map_err(|e| DomainViolation::InfrastructureBoundaryFault {
                    subsystem: "sqlite_adapter".into(),
                    details: format!("Map cycle error: {e}"),
                })?;
            Ok(ActiveCycleLookup::Found(Box::new(cycle)))
        } else {
            Ok(ActiveCycleLookup::NoActiveCycle)
        }
    }

    fn save_artifact_seal(&self, seal: &ArtifactSeal) -> Result<u64, DomainViolation> {
        let conn = self.lock_conn()?;
        let cat = match seal.artifact_category {
            ArtifactCategory::Spec => "SPEC",
            ArtifactCategory::Constitution => "CONSTITUTION",
            ArtifactCategory::PreExistingTest => "PRE_EXISTING_TEST",
            _ => "FOREIGN_ISSUE_EVIDENCE",
        };
        let (unlock_state, just) = match &seal.contract_unlock_state {
            ContractUnlockState::Sealed => ("SEALED", "NONE"),
            ContractUnlockState::UnlockedByHuman { justification } => {
                ("UNLOCKED_BY_HUMAN", justification.as_str())
            }
        };
        conn.execute(
            "INSERT OR REPLACE INTO artifact_seals (cycle_id, file_path, artifact_category, blake3_digest, ast_assertion_count, contract_unlock_state, unlock_justification)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![seal.cycle_id, seal.file_path, cat, seal.blake3_digest, seal.ast_assertion_count as i64, unlock_state, just],
        ).map_err(|e| DomainViolation::InfrastructureBoundaryFault { subsystem: "sqlite_adapter".into(), details: format!("Insert seal failed: {e}") })?;
        Ok(conn.last_insert_rowid() as u64)
    }

    fn list_artifact_seals(&self, cycle_id: &str) -> Result<Vec<ArtifactSeal>, DomainViolation> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT seal_id, cycle_id, file_path, artifact_category, blake3_digest, ast_assertion_count, contract_unlock_state, unlock_justification
             FROM artifact_seals WHERE cycle_id = ?1 ORDER BY seal_id ASC"
        ).map_err(|e| DomainViolation::InfrastructureBoundaryFault { subsystem: "sqlite_adapter".into(), details: format!("Prepare failed: {e}") })?;
        let rows = stmt
            .query_map(params![cycle_id], map_row_to_artifact_seal)
            .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
                subsystem: "sqlite_adapter".into(),
                details: format!("Query seals failed: {e}"),
            })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| {
            DomainViolation::InfrastructureBoundaryFault {
                subsystem: "sqlite_adapter".into(),
                details: format!("Seal row error: {e}"),
            }
        })
    }

    fn append_evidence(&self, ev: &ExecutionEvidence) -> Result<u64, DomainViolation> {
        let conn = self.lock_conn()?;
        let kind = match ev.evidence_kind {
            EvidenceKind::CmdExec => "CMD_EXEC",
            EvidenceKind::AstAudit => "AST_AUDIT",
            EvidenceKind::BoardSync => "BOARD_SYNC",
            EvidenceKind::SpecSeal => "SPEC_SEAL",
            _ => "CRITIC_VERDICT",
        };
        let verdict = match ev.verdict {
            GateVerdict::Pass => "PASS",
            GateVerdict::Fail => "FAIL",
            GateVerdict::Info => "INFO",
            _ => "BLOCKED",
        };
        conn.execute(
            "INSERT INTO execution_evidences (cycle_id, gate_stage, gauntlet_attempt, actor_role, evidence_kind, executed_command, exit_code, duration_ms, sanitized_stdout_tail, sanitized_stderr_tail, output_blake3_digest, verdict, recorded_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                ev.cycle_id, ev.gate_stage.as_u8() as i64, ev.gauntlet_attempt as i64, ev.actor_role,
                kind, ev.executed_command, ev.exit_code, ev.duration_ms as i64, ev.sanitized_stdout_tail,
                ev.sanitized_stderr_tail, ev.output_blake3_digest, verdict, ev.recorded_at,
            ],
        ).map_err(|e| DomainViolation::InfrastructureBoundaryFault { subsystem: "sqlite_adapter".into(), details: format!("Insert evidence failed: {e}") })?;
        Ok(conn.last_insert_rowid() as u64)
    }

    fn list_evidences_for_cycle(
        &self,
        cycle_id: &str,
    ) -> Result<Vec<ExecutionEvidence>, DomainViolation> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT evidence_id, cycle_id, gate_stage, gauntlet_attempt, actor_role, evidence_kind, executed_command,
                    exit_code, duration_ms, sanitized_stdout_tail, sanitized_stderr_tail, output_blake3_digest, verdict, recorded_at
             FROM execution_evidences WHERE cycle_id = ?1 ORDER BY evidence_id ASC"
        ).map_err(|e| DomainViolation::InfrastructureBoundaryFault { subsystem: "sqlite_adapter".into(), details: format!("Prepare failed: {e}") })?;
        let rows = stmt
            .query_map(params![cycle_id], map_row_to_evidence)
            .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
                subsystem: "sqlite_adapter".into(),
                details: format!("Query evidences failed: {e}"),
            })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| {
            DomainViolation::InfrastructureBoundaryFault {
                subsystem: "sqlite_adapter".into(),
                details: format!("Evidence row error: {e}"),
            }
        })
    }

    fn update_cycle_spec_lock(
        &self,
        cycle_id: &str,
        blake3_digest: &str,
        locked_state: &str,
    ) -> Result<(), DomainViolation> {
        let conn = self.lock_conn()?;
        let gate = if locked_state == "FROZEN" { 2i64 } else { 1i64 };
        conn.execute(
            "UPDATE sdd_cycles SET spec_lock_state = ?1, spec_blake3_digest = ?2, current_gate = ?3, updated_at = (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')) WHERE cycle_id = ?4",
            params![locked_state, blake3_digest, gate, cycle_id],
        ).map_err(|e| DomainViolation::InfrastructureBoundaryFault { subsystem: "sqlite_adapter".into(), details: format!("Update cycle spec lock failed: {e}") })?;
        Ok(())
    }

    fn save_atomic_tasks(
        &self,
        cycle_id: &str,
        tasks: &[TaskNode],
        wave_schedule: &WaveSchedule,
    ) -> Result<(), DomainViolation> {
        let mut conn = self.lock_conn()?;
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
                "INSERT OR REPLACE INTO atomic_tasks (cycle_id, task_id, title, architectural_layer, wave_index, depends_on_json, expected_files_json, target_test_file, tdd_lifecycle_state, red_evidence_id, green_evidence_id, worktree_allocation, commit_sha)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'AWAITING_RED_PHASE', 0, 0, 'NOT_PROVISIONED', 'UNCOMMITTED')",
                params![cycle_id, task.task_id, task.title, task.architectural_layer, wave_index, string_list_to_json(&task.depends_on), string_list_to_json(&task.expected_files), task.target_test_file],
            ).map_err(|e| DomainViolation::InfrastructureBoundaryFault { subsystem: "sqlite_adapter".into(), details: format!("Insert atomic task failed: {e}") })?;
        }
        tx.execute(
            "UPDATE sdd_cycles SET current_gate = 3, updated_at = (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')) WHERE cycle_id = ?1",
            params![cycle_id],
        ).map_err(|e| DomainViolation::InfrastructureBoundaryFault { subsystem: "sqlite_adapter".into(), details: format!("Update cycle gate 3 failed: {e}") })?;
        tx.commit()
            .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
                subsystem: "sqlite_adapter".into(),
                details: format!("Commit tx failed: {e}"),
            })?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "sqlite_adapter_tests.rs"]
mod tests;
