// Database operations for ArtifactSeal and ExecutionEvidence persistence.
// Constitutional Principle II: Absolute Prohibition of NULL & Special Cases.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::row_mappers::{map_row_to_artifact_seal, map_row_to_evidence};
use rusqlite::{Connection, params};
use trustkit_core::entities::{
    ArtifactCategory, ArtifactSeal, EvidenceKind, ExecutionEvidence, GateVerdict,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::special_cases::ContractUnlockState;

pub fn save_artifact_seal(conn: &Connection, seal: &ArtifactSeal) -> Result<u64, DomainViolation> {
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

pub fn list_artifact_seals(conn: &Connection, cycle_id: &str) -> Result<Vec<ArtifactSeal>, DomainViolation> {
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

pub fn append_evidence(conn: &Connection, ev: &ExecutionEvidence) -> Result<u64, DomainViolation> {
    let kind = match ev.evidence_kind {
        EvidenceKind::CmdExec => "CMD_EXEC",
        EvidenceKind::AstAudit => "AST_AUDIT",
        EvidenceKind::BoardSync => "BOARD_SYNC",
        EvidenceKind::SpecSeal => "SPEC_SEAL",
        EvidenceKind::CodeReview => "CODE_REVIEW",
        EvidenceKind::SecurityReview => "SECURITY_REVIEW",
        EvidenceKind::CriticVerdict => "CRITIC_VERDICT",
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

pub fn list_evidences_for_cycle(
    conn: &Connection,
    cycle_id: &str,
) -> Result<Vec<ExecutionEvidence>, DomainViolation> {
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

pub fn list_certification_evidences(
    conn: &Connection,
    cycle_id: &str,
) -> Result<Vec<ExecutionEvidence>, DomainViolation> {
    let mut stmt = conn.prepare(
        "SELECT evidence_id, cycle_id, gate_stage, gauntlet_attempt, actor_role, evidence_kind, executed_command,
                exit_code, duration_ms, sanitized_stdout_tail, sanitized_stderr_tail, output_blake3_digest, verdict, recorded_at
         FROM execution_evidences
         WHERE cycle_id = ?1
         ORDER BY recorded_at ASC, evidence_id ASC"
    ).map_err(|e| DomainViolation::InfrastructureBoundaryFault { subsystem: "sqlite_adapter".into(), details: format!("Prepare certification query failed: {e}") })?;
    let rows = stmt
        .query_map(params![cycle_id], map_row_to_evidence)
        .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
            subsystem: "sqlite_adapter".into(),
            details: format!("Query certification evidences failed: {e}"),
        })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| {
        DomainViolation::InfrastructureBoundaryFault {
            subsystem: "sqlite_adapter".into(),
            details: format!("Certification evidence row error: {e}"),
        }
    })
}
