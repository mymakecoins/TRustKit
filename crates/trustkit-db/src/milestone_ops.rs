// Database operations for Milestone planning and persistence.
// Constitutional Principle II: Absolute Prohibition of NULL & Special Cases.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::row_mappers::{json_to_string_list, string_list_to_json};
use rusqlite::{Connection, Row, params};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use trustkit_core::entities::{
    Milestone, MilestoneCapability, MilestoneStatus, SourceCitation, TechnicalConstraint,
};
use trustkit_core::errors::DomainViolation;

#[derive(Debug, Serialize, Deserialize)]
struct CapabilityDto {
    capability_id: String,
    title: String,
    description: String,
    citation_file: String,
    citation_start: usize,
    citation_end: usize,
    citation_section: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TechnicalConstraintDto {
    constraint_id: String,
    title: String,
    description: String,
    citation_file: String,
    citation_start: usize,
    citation_end: usize,
    citation_section: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MilestoneContentEnvelope {
    capabilities: Vec<CapabilityDto>,
    technical_constraints: Vec<TechnicalConstraintDto>,
}

fn encode_capabilities_and_constraints(
    caps: &[MilestoneCapability],
    cons: &[TechnicalConstraint],
) -> String {
    let cap_dtos = caps
        .iter()
        .map(|c| CapabilityDto {
            capability_id: c.capability_id.clone(),
            title: c.title.clone(),
            description: c.description.clone(),
            citation_file: c.citation.file_path.clone(),
            citation_start: c.citation.start_line,
            citation_end: c.citation.end_line,
            citation_section: c.citation.section_title.clone(),
        })
        .collect();

    let con_dtos = cons
        .iter()
        .map(|c| TechnicalConstraintDto {
            constraint_id: c.constraint_id.clone(),
            title: c.title.clone(),
            description: c.description.clone(),
            citation_file: c.citation.file_path.clone(),
            citation_start: c.citation.start_line,
            citation_end: c.citation.end_line,
            citation_section: c.citation.section_title.clone(),
        })
        .collect();

    let envelope = MilestoneContentEnvelope {
        capabilities: cap_dtos,
        technical_constraints: con_dtos,
    };

    serde_json::to_string(&envelope).unwrap_or_else(|_| "{}".to_string())
}

fn decode_capabilities_and_constraints(
    raw: &str,
) -> (Vec<MilestoneCapability>, Vec<TechnicalConstraint>) {
    if let Ok(envelope) = serde_json::from_str::<MilestoneContentEnvelope>(raw) {
        let caps = envelope
            .capabilities
            .into_iter()
            .map(|dto| {
                MilestoneCapability::new(
                    dto.capability_id,
                    dto.title,
                    dto.description,
                    SourceCitation::new(
                        dto.citation_file,
                        dto.citation_start,
                        dto.citation_end,
                        dto.citation_section,
                    ),
                )
            })
            .collect();

        let cons = envelope
            .technical_constraints
            .into_iter()
            .map(|dto| {
                TechnicalConstraint::new(
                    dto.constraint_id,
                    dto.title,
                    dto.description,
                    SourceCitation::new(
                        dto.citation_file,
                        dto.citation_start,
                        dto.citation_end,
                        dto.citation_section,
                    ),
                )
            })
            .collect();

        return (caps, cons);
    }

    if let Ok(caps_only) = serde_json::from_str::<Vec<CapabilityDto>>(raw) {
        let caps = caps_only
            .into_iter()
            .map(|dto| {
                MilestoneCapability::new(
                    dto.capability_id,
                    dto.title,
                    dto.description,
                    SourceCitation::new(
                        dto.citation_file,
                        dto.citation_start,
                        dto.citation_end,
                        dto.citation_section,
                    ),
                )
            })
            .collect();
        return (caps, Vec::new());
    }

    (Vec::new(), Vec::new())
}

pub fn map_row_to_milestone(row: &Row<'_>) -> rusqlite::Result<Milestone> {
    let milestone_id: String = row.get(0)?;
    let plan_id: String = row.get(1)?;
    let sequence_index: i64 = row.get(2)?;
    let title: String = row.get(3)?;
    let slug: String = row.get(4)?;
    let file_path: String = row.get(5)?;
    let goal: String = row.get(6)?;
    let dependencies_json: String = row.get(7)?;
    let capabilities_json: String = row.get(8)?;
    let status_raw: String = row.get(9)?;
    let created_at: String = row.get(10)?;
    let updated_at: String = row.get(11)?;

    let status = MilestoneStatus::from_str(&status_raw).unwrap_or(MilestoneStatus::Planned);
    let dependencies = json_to_string_list(&dependencies_json);
    let (capabilities, technical_constraints) =
        decode_capabilities_and_constraints(&capabilities_json);

    Ok(Milestone {
        milestone_id,
        plan_id,
        sequence_index: sequence_index as usize,
        title,
        slug,
        file_path,
        goal,
        dependencies,
        capabilities,
        technical_constraints,
        status,
        created_at,
        updated_at,
    })
}

pub fn save_milestone(conn: &Connection, milestone: &Milestone) -> Result<(), DomainViolation> {
    let deps_json = string_list_to_json(&milestone.dependencies);
    let caps_json = encode_capabilities_and_constraints(
        &milestone.capabilities,
        &milestone.technical_constraints,
    );

    conn.execute(
        "INSERT INTO milestones (
            milestone_id, plan_id, sequence_index, title, slug, file_path, goal,
            dependencies_json, capabilities_json, status, created_at, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
         ON CONFLICT(milestone_id) DO UPDATE SET
            plan_id = excluded.plan_id,
            sequence_index = excluded.sequence_index,
            title = excluded.title,
            slug = excluded.slug,
            file_path = excluded.file_path,
            goal = excluded.goal,
            dependencies_json = excluded.dependencies_json,
            capabilities_json = excluded.capabilities_json,
            status = excluded.status,
            updated_at = excluded.updated_at",
        params![
            milestone.milestone_id,
            milestone.plan_id,
            milestone.sequence_index as i64,
            milestone.title,
            milestone.slug,
            milestone.file_path,
            milestone.goal,
            deps_json,
            caps_json,
            milestone.status.as_str(),
            milestone.created_at,
            milestone.updated_at,
        ],
    )
    .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
        subsystem: "milestone_ops".to_string(),
        details: format!("Failed to save milestone {}: {e}", milestone.milestone_id),
    })?;

    Ok(())
}

pub fn find_milestone_by_id(
    conn: &Connection,
    milestone_id: &str,
) -> Result<Option<Milestone>, DomainViolation> {
    let mut stmt = conn
        .prepare(
            "SELECT milestone_id, plan_id, sequence_index, title, slug, file_path, goal,
                    dependencies_json, capabilities_json, status, created_at, updated_at
             FROM milestones WHERE milestone_id = ?1",
        )
        .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
            subsystem: "milestone_ops".to_string(),
            details: format!("Prepare find_milestone_by_id failed: {e}"),
        })?;

    let mut rows = stmt
        .query_map(params![milestone_id], map_row_to_milestone)
        .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
            subsystem: "milestone_ops".to_string(),
            details: format!("Query find_milestone_by_id failed: {e}"),
        })?;

    match rows.next() {
        Some(Ok(m)) => Ok(Some(m)),
        Some(Err(e)) => Err(DomainViolation::InfrastructureBoundaryFault {
            subsystem: "milestone_ops".to_string(),
            details: format!("Row mapping error for milestone {milestone_id}: {e}"),
        }),
        None => Ok(None),
    }
}

pub fn list_milestones(
    conn: &Connection,
    plan_id: Option<&str>,
) -> Result<Vec<Milestone>, DomainViolation> {
    let sql = match plan_id {
        Some(_) => {
            "SELECT milestone_id, plan_id, sequence_index, title, slug, file_path, goal,
                    dependencies_json, capabilities_json, status, created_at, updated_at
             FROM milestones WHERE plan_id = ?1 ORDER BY sequence_index ASC, milestone_id ASC"
        }
        None => {
            "SELECT milestone_id, plan_id, sequence_index, title, slug, file_path, goal,
                    dependencies_json, capabilities_json, status, created_at, updated_at
             FROM milestones ORDER BY sequence_index ASC, milestone_id ASC"
        }
    };

    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
            subsystem: "milestone_ops".to_string(),
            details: format!("Prepare list_milestones failed: {e}"),
        })?;

    let mapped = match plan_id {
        Some(p) => stmt.query_map(params![p], map_row_to_milestone),
        None => stmt.query_map([], map_row_to_milestone),
    };

    let rows = mapped.map_err(|e| DomainViolation::InfrastructureBoundaryFault {
        subsystem: "milestone_ops".to_string(),
        details: format!("Query list_milestones failed: {e}"),
    })?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
            subsystem: "milestone_ops".to_string(),
            details: format!("Collect list_milestones failed: {e}"),
        })
}

pub fn update_milestone_status(
    conn: &Connection,
    milestone_id: &str,
    status: MilestoneStatus,
    updated_at: &str,
) -> Result<(), DomainViolation> {
    let count = conn
        .execute(
            "UPDATE milestones SET status = ?1, updated_at = ?2 WHERE milestone_id = ?3",
            params![status.as_str(), updated_at, milestone_id],
        )
        .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
            subsystem: "milestone_ops".to_string(),
            details: format!("Update milestone status failed: {e}"),
        })?;

    if count == 0 {
        return Err(DomainViolation::InfrastructureBoundaryFault {
            subsystem: "milestone_ops".to_string(),
            details: format!("Milestone '{milestone_id}' not found for status update"),
        });
    }

    Ok(())
}
