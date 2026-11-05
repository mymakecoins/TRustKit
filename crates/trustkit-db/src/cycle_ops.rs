// Database operations for SddCycle lifecycle and gates.
// Constitutional Principle II: Absolute Prohibition of NULL & Special Cases.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::row_mappers::{map_row_to_cycle, string_list_to_json};
use rusqlite::{Connection, params};
use trustkit_core::entities::{CycleStatus, GateStage, MethodologyProfile, SddCycle};
use trustkit_core::errors::DomainViolation;
use trustkit_core::special_cases::{ActiveCycleLookup, IssueLinkage, SpecLockState};

pub fn save_cycle(conn: &Connection, c: &SddCycle) -> Result<(), DomainViolation> {
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
        MethodologyProfile::StructuredSdd { .. } => "STRUCTURED_SDD",
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

pub fn find_active_cycle(conn: &Connection) -> Result<ActiveCycleLookup, DomainViolation> {
    let mut stmt = conn.prepare(
        "SELECT cycle_id, issue_linkage_kind, issue_number, issue_url, project_board_id, project_item_id, board_column,
                methodology_profile, current_gate, lifecycle_status, gauntlet_attempt, spec_dir, spec_file_path,
                spec_lock_state, spec_blake3_digest, work_branch, base_commit_sha, pre_existing_dirty_json, created_at, updated_at
         FROM sdd_cycles WHERE lifecycle_status IN ('ACTIVE', 'CERTIFIED', 'MERGED') ORDER BY created_at DESC LIMIT 1"
    ).map_err(|e| DomainViolation::InfrastructureBoundaryFault { subsystem: "sqlite_adapter".into(), details: format!("Prepare failed: {e}") })?;
    let mut rows = stmt
        .query([])
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

pub fn find_latest_cycle(conn: &Connection) -> Result<Option<SddCycle>, DomainViolation> {
    let mut stmt = conn.prepare(
        "SELECT cycle_id, issue_linkage_kind, issue_number, issue_url, project_board_id, project_item_id, board_column,
                methodology_profile, current_gate, lifecycle_status, gauntlet_attempt, spec_dir, spec_file_path,
                spec_lock_state, spec_blake3_digest, work_branch, base_commit_sha, pre_existing_dirty_json, created_at, updated_at
         FROM sdd_cycles ORDER BY created_at DESC LIMIT 1"
    ).map_err(|e| DomainViolation::InfrastructureBoundaryFault { subsystem: "sqlite_adapter".into(), details: format!("Prepare failed: {e}") })?;
    let mut rows = stmt
        .query([])
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
        Ok(Some(cycle))
    } else {
        Ok(None)
    }
}

pub fn update_cycle_spec_lock(
    conn: &Connection,
    cycle_id: &str,
    blake3_digest: &str,
    locked_state: &str,
) -> Result<(), DomainViolation> {
    let gate = if locked_state == "FROZEN" { 2i64 } else { 1i64 };
    conn.execute(
        "UPDATE sdd_cycles SET spec_lock_state = ?1, spec_blake3_digest = ?2, current_gate = ?3, updated_at = (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')) WHERE cycle_id = ?4",
        params![locked_state, blake3_digest, gate, cycle_id],
    ).map_err(|e| DomainViolation::InfrastructureBoundaryFault { subsystem: "sqlite_adapter".into(), details: format!("Update cycle spec lock failed: {e}") })?;
    Ok(())
}

pub fn update_cycle_status(
    conn: &Connection,
    cycle_id: &str,
    status: CycleStatus,
    gauntlet_attempt: u8,
) -> Result<(), DomainViolation> {
    let status_str = match status {
        CycleStatus::Active => "ACTIVE",
        CycleStatus::Paused => "PAUSED",
        CycleStatus::CircuitBreakerTripped => "CIRCUIT_BREAKER_TRIPPED",
        CycleStatus::Certified => "CERTIFIED",
        CycleStatus::Merged => "MERGED",
    };
    let rows = conn.execute(
        "UPDATE sdd_cycles SET lifecycle_status = ?1, gauntlet_attempt = ?2, updated_at = (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')) WHERE cycle_id = ?3",
        params![status_str, gauntlet_attempt as i64, cycle_id],
    ).map_err(|e| DomainViolation::InfrastructureBoundaryFault { subsystem: "sqlite_adapter".into(), details: format!("Update cycle status failed: {e}") })?;
    if rows == 0 {
        return Err(DomainViolation::InfrastructureBoundaryFault {
            subsystem: "sqlite_adapter".into(),
            details: format!("Cycle {cycle_id} not found"),
        });
    }
    Ok(())
}

pub fn advance_cycle_gate(
    conn: &Connection,
    cycle_id: &str,
    next_gate: GateStage,
) -> Result<(), DomainViolation> {
    let rows = conn.execute(
        "UPDATE sdd_cycles SET current_gate = ?1, updated_at = (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')) WHERE cycle_id = ?2",
        params![next_gate.as_u8() as i64, cycle_id],
    ).map_err(|e| DomainViolation::InfrastructureBoundaryFault { subsystem: "sqlite_adapter".into(), details: format!("Advance cycle gate failed: {e}") })?;
    if rows == 0 {
        return Err(DomainViolation::InfrastructureBoundaryFault {
            subsystem: "sqlite_adapter".into(),
            details: format!("Cycle {cycle_id} not found"),
        });
    }
    Ok(())
}
