// SQLite WAL repository adapter.
// Constitutional Principle II: Absolute Prohibition of NULL & Special Cases.
// Constitutional Principle IV: Resilient Error Handling & Zero-Panic Policy.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::cycle_ops;
use crate::evidence_ops;
use crate::migrations::initialize_database;
use crate::task_ops;
use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;
use trustkit_core::entities::{
    ArtifactSeal, AtomicTask, CycleStatus, ExecutionEvidence, GateStage, SddCycle, TaskNode,
    WaveSchedule,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::CycleEvidenceRepositoryPort;
use trustkit_core::special_cases::{ActiveCycleLookup, TddLifecycleState, WorktreeAllocation};

pub struct SqliteWalRepositoryAdapter {
    pub(crate) conn: Mutex<Connection>,
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

    pub(crate) fn lock_conn(&self) -> Result<std::sync::MutexGuard<'_, Connection>, DomainViolation> {
        self.conn
            .lock()
            .map_err(|_| DomainViolation::InfrastructureBoundaryFault {
                subsystem: "sqlite_adapter".into(),
                details: "Poisoned SQLite mutex".into(),
            })
    }

    pub fn list_atomic_tasks(&self, cycle_id: &str) -> Result<Vec<AtomicTask>, DomainViolation> {
        let conn = self.lock_conn()?;
        task_ops::list_atomic_tasks(&conn, cycle_id)
    }

    pub fn find_atomic_task(&self, cycle_id: &str, task_id: &str) -> Result<Option<AtomicTask>, DomainViolation> {
        let conn = self.lock_conn()?;
        task_ops::find_atomic_task(&conn, cycle_id, task_id)
    }
}

impl CycleEvidenceRepositoryPort for SqliteWalRepositoryAdapter {
    fn save_cycle(&self, c: &SddCycle) -> Result<(), DomainViolation> {
        let conn = self.lock_conn()?;
        cycle_ops::save_cycle(&conn, c)
    }

    fn find_active_cycle(&self) -> Result<ActiveCycleLookup, DomainViolation> {
        let conn = self.lock_conn()?;
        cycle_ops::find_active_cycle(&conn)
    }

    fn find_latest_cycle(&self) -> Result<Option<SddCycle>, DomainViolation> {
        let conn = self.lock_conn()?;
        cycle_ops::find_latest_cycle(&conn)
    }

    fn save_artifact_seal(&self, seal: &ArtifactSeal) -> Result<u64, DomainViolation> {
        let conn = self.lock_conn()?;
        evidence_ops::save_artifact_seal(&conn, seal)
    }

    fn list_artifact_seals(&self, cycle_id: &str) -> Result<Vec<ArtifactSeal>, DomainViolation> {
        let conn = self.lock_conn()?;
        evidence_ops::list_artifact_seals(&conn, cycle_id)
    }

    fn append_evidence(&self, ev: &ExecutionEvidence) -> Result<u64, DomainViolation> {
        let conn = self.lock_conn()?;
        evidence_ops::append_evidence(&conn, ev)
    }

    fn list_evidences_for_cycle(
        &self,
        cycle_id: &str,
    ) -> Result<Vec<ExecutionEvidence>, DomainViolation> {
        let conn = self.lock_conn()?;
        evidence_ops::list_evidences_for_cycle(&conn, cycle_id)
    }

    fn update_cycle_spec_lock(
        &self,
        cycle_id: &str,
        blake3_digest: &str,
        locked_state: &str,
    ) -> Result<(), DomainViolation> {
        let conn = self.lock_conn()?;
        cycle_ops::update_cycle_spec_lock(&conn, cycle_id, blake3_digest, locked_state)
    }

    fn save_atomic_tasks(
        &self,
        cycle_id: &str,
        tasks: &[TaskNode],
        wave_schedule: &WaveSchedule,
    ) -> Result<(), DomainViolation> {
        let mut conn = self.lock_conn()?;
        task_ops::save_atomic_tasks(&mut conn, cycle_id, tasks, wave_schedule)
    }

    fn update_task_tdd_phase(
        &self,
        cycle_id: &str,
        task_id: &str,
        state: &TddLifecycleState,
    ) -> Result<(), DomainViolation> {
        let conn = self.lock_conn()?;
        task_ops::update_task_tdd_phase(&conn, cycle_id, task_id, state)
    }

    fn update_task_worktree(
        &self,
        cycle_id: &str,
        task_id: &str,
        allocation: &WorktreeAllocation,
    ) -> Result<(), DomainViolation> {
        let conn = self.lock_conn()?;
        task_ops::update_task_worktree(&conn, cycle_id, task_id, allocation)
    }

    fn find_atomic_task(
        &self,
        cycle_id: &str,
        task_id: &str,
    ) -> Result<Option<AtomicTask>, DomainViolation> {
        let conn = self.lock_conn()?;
        task_ops::find_atomic_task(&conn, cycle_id, task_id)
    }

    fn list_atomic_tasks(&self, cycle_id: &str) -> Result<Vec<AtomicTask>, DomainViolation> {
        let conn = self.lock_conn()?;
        task_ops::list_atomic_tasks(&conn, cycle_id)
    }

    fn update_cycle_status(
        &self,
        cycle_id: &str,
        status: CycleStatus,
        gauntlet_attempt: u8,
    ) -> Result<(), DomainViolation> {
        let conn = self.lock_conn()?;
        cycle_ops::update_cycle_status(&conn, cycle_id, status, gauntlet_attempt)
    }

    fn advance_cycle_gate(
        &self,
        cycle_id: &str,
        next_gate: GateStage,
    ) -> Result<(), DomainViolation> {
        let conn = self.lock_conn()?;
        cycle_ops::advance_cycle_gate(&conn, cycle_id, next_gate)
    }

    fn list_certification_evidences(
        &self,
        cycle_id: &str,
    ) -> Result<Vec<ExecutionEvidence>, DomainViolation> {
        let conn = self.lock_conn()?;
        evidence_ops::list_certification_evidences(&conn, cycle_id)
    }
}

#[cfg(test)]
#[path = "sqlite_adapter_tests.rs"]
mod tests;
