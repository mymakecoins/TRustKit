// Database operations for Feature planning and persistence.
// Constitutional Principle II: Absolute Prohibition of NULL & Special Cases.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::row_mappers::{json_to_string_list, string_list_to_json};
use rusqlite::{Connection, Row, params};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use trustkit_core::entities::{
    ArchitecturalTaskLayer, FeatureStatus, FeatureTaskBlueprint, MilestoneCapability,
    PlannedFeature, SourceCitation, TechnicalConstraint,
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
struct TaskBlueprintDto {
    task_id: String,
    layer: String,
    title: String,
    business_rule_id: String,
    functional_req_id: String,
    mermaid_node: String,
    target_test_file: String,
    evidence_artifact: String,
    depends_on: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct FeaturePayloadEnvelope {
    capabilities: Vec<CapabilityDto>,
    technical_constraints: Vec<TechnicalConstraintDto>,
    task_blueprints: Vec<TaskBlueprintDto>,
}

fn encode_payload(feature: &PlannedFeature) -> String {
    let capabilities = feature
        .capabilities
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

    let technical_constraints = feature
        .technical_constraints
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

    let task_blueprints = feature
        .task_blueprints
        .iter()
        .map(|bp| TaskBlueprintDto {
            task_id: bp.task_id.clone(),
            layer: bp.layer.as_str().to_string(),
            title: bp.title.clone(),
            business_rule_id: bp.business_rule_id.clone(),
            functional_req_id: bp.functional_req_id.clone(),
            mermaid_node: bp.mermaid_node.clone(),
            target_test_file: bp.target_test_file.clone(),
            evidence_artifact: bp.evidence_artifact.clone(),
            depends_on: bp.depends_on.clone(),
        })
        .collect();

    let env = FeaturePayloadEnvelope {
        capabilities,
        technical_constraints,
        task_blueprints,
    };
    serde_json::to_string(&env).unwrap_or_else(|_| "{}".to_string())
}

fn decode_payload(
    raw: &str,
) -> (
    Vec<MilestoneCapability>,
    Vec<TechnicalConstraint>,
    Vec<FeatureTaskBlueprint>,
) {
    let Ok(env) = serde_json::from_str::<FeaturePayloadEnvelope>(raw) else {
        return (Vec::new(), Vec::new(), Vec::new());
    };
    let caps = env
        .capabilities
        .into_iter()
        .map(|d| {
            MilestoneCapability::new(
                d.capability_id,
                d.title,
                d.description,
                SourceCitation::new(
                    d.citation_file,
                    d.citation_start,
                    d.citation_end,
                    d.citation_section,
                ),
            )
        })
        .collect();
    let cons = env
        .technical_constraints
        .into_iter()
        .map(|d| {
            TechnicalConstraint::new(
                d.constraint_id,
                d.title,
                d.description,
                SourceCitation::new(
                    d.citation_file,
                    d.citation_start,
                    d.citation_end,
                    d.citation_section,
                ),
            )
        })
        .collect();
    let bps = env
        .task_blueprints
        .into_iter()
        .map(|d| FeatureTaskBlueprint {
            task_id: d.task_id,
            layer: ArchitecturalTaskLayer::from_str(&d.layer)
                .unwrap_or(ArchitecturalTaskLayer::CoreAndDomain),
            title: d.title,
            business_rule_id: d.business_rule_id,
            functional_req_id: d.functional_req_id,
            mermaid_node: d.mermaid_node,
            target_test_file: d.target_test_file,
            evidence_artifact: d.evidence_artifact,
            depends_on: d.depends_on,
        })
        .collect();
    (caps, cons, bps)
}

pub fn map_row_to_feature(row: &Row<'_>) -> rusqlite::Result<PlannedFeature> {
    let feature_id: String = row.get(0)?;
    let milestone_id: String = row.get(1)?;
    let sequence_index: i64 = row.get(2)?;
    let title: String = row.get(3)?;
    let slug: String = row.get(4)?;
    let spec_dir: String = row.get(5)?;
    let spec_file_path: String = row.get(6)?;
    let goal: String = row.get(7)?;
    let dependencies_json: String = row.get(8)?;
    let payload_json: String = row.get(9)?;
    let status_raw: String = row.get(10)?;
    let linked_cycle_id: String = row.get(11)?;
    let created_at: String = row.get(12)?;
    let updated_at: String = row.get(13)?;

    let status = FeatureStatus::from_str(&status_raw).unwrap_or(FeatureStatus::Planned);
    let dependencies = json_to_string_list(&dependencies_json);
    let (capabilities, technical_constraints, task_blueprints) = decode_payload(&payload_json);

    Ok(PlannedFeature {
        feature_id,
        milestone_id,
        sequence_index: sequence_index as usize,
        title,
        slug,
        spec_dir,
        spec_file_path,
        goal,
        dependencies,
        capabilities,
        technical_constraints,
        task_blueprints,
        status,
        linked_cycle_id,
        created_at,
        updated_at,
    })
}

pub fn save_feature(conn: &Connection, f: &PlannedFeature) -> Result<(), DomainViolation> {
    let deps_json = string_list_to_json(&f.dependencies);
    let payload_json = encode_payload(f);
    conn.execute(
        "INSERT INTO milestone_features (
            feature_id, milestone_id, sequence_index, title, slug, spec_dir, spec_file_path,
            goal, dependencies_json, payload_json, status, linked_cycle_id, created_at, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
         ON CONFLICT(feature_id) DO UPDATE SET
            milestone_id = excluded.milestone_id,
            sequence_index = excluded.sequence_index,
            title = excluded.title,
            slug = excluded.slug,
            spec_dir = excluded.spec_dir,
            spec_file_path = excluded.spec_file_path,
            goal = excluded.goal,
            dependencies_json = excluded.dependencies_json,
            payload_json = excluded.payload_json,
            status = CASE
                WHEN milestone_features.status IN ('InProgress', 'Completed') THEN milestone_features.status
                ELSE excluded.status
            END,
            linked_cycle_id = CASE
                WHEN milestone_features.status IN ('InProgress', 'Completed') THEN milestone_features.linked_cycle_id
                ELSE excluded.linked_cycle_id
            END,
            updated_at = excluded.updated_at",
        params![
            f.feature_id,
            f.milestone_id,
            f.sequence_index as i64,
            f.title,
            f.slug,
            f.spec_dir,
            f.spec_file_path,
            f.goal,
            deps_json,
            payload_json,
            f.status.as_str(),
            f.linked_cycle_id,
            f.created_at,
            f.updated_at,
        ],
    )
    .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
        subsystem: "feature_ops".to_string(),
        details: format!("Failed to save feature {}: {e}", f.feature_id),
    })?;
    Ok(())
}

pub fn find_feature_by_id(
    conn: &Connection,
    feature_id: &str,
) -> Result<Option<PlannedFeature>, DomainViolation> {
    let mut stmt = conn
        .prepare(
            "SELECT feature_id, milestone_id, sequence_index, title, slug, spec_dir, spec_file_path,
                    goal, dependencies_json, payload_json, status, linked_cycle_id, created_at, updated_at
             FROM milestone_features WHERE feature_id = ?1",
        )
        .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
            subsystem: "feature_ops".to_string(),
            details: format!("Prepare find_feature_by_id failed: {e}"),
        })?;
    let mut rows = stmt
        .query_map(params![feature_id], map_row_to_feature)
        .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
            subsystem: "feature_ops".to_string(),
            details: format!("Query find_feature_by_id failed: {e}"),
        })?;
    match rows.next() {
        Some(Ok(f)) => Ok(Some(f)),
        Some(Err(e)) => Err(DomainViolation::InfrastructureBoundaryFault {
            subsystem: "feature_ops".to_string(),
            details: format!("Decode feature row failed: {e}"),
        }),
        None => Ok(None),
    }
}

pub fn list_features(
    conn: &Connection,
    milestone_id: Option<&str>,
) -> Result<Vec<PlannedFeature>, DomainViolation> {
    let sql_filtered = "SELECT feature_id, milestone_id, sequence_index, title, slug, spec_dir, spec_file_path,
                         goal, dependencies_json, payload_json, status, linked_cycle_id, created_at, updated_at
                  FROM milestone_features WHERE milestone_id = ?1 ORDER BY sequence_index ASC";
    let sql_all = "SELECT feature_id, milestone_id, sequence_index, title, slug, spec_dir, spec_file_path,
                         goal, dependencies_json, payload_json, status, linked_cycle_id, created_at, updated_at
                  FROM milestone_features ORDER BY milestone_id ASC, sequence_index ASC";

    let mut out = Vec::new();
    if let Some(m_id) = milestone_id {
        let mut stmt = conn.prepare(sql_filtered).map_err(|e| {
            DomainViolation::InfrastructureBoundaryFault {
                subsystem: "feature_ops".to_string(),
                details: format!("Prepare list_features failed: {e}"),
            }
        })?;
        let rows = stmt
            .query_map(params![m_id], map_row_to_feature)
            .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
                subsystem: "feature_ops".to_string(),
                details: format!("Query list_features failed: {e}"),
            })?;
        for r in rows {
            out.push(r.map_err(|e| DomainViolation::InfrastructureBoundaryFault {
                subsystem: "feature_ops".to_string(),
                details: format!("Decode feature row failed: {e}"),
            })?);
        }
    } else {
        let mut stmt =
            conn.prepare(sql_all)
                .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
                    subsystem: "feature_ops".to_string(),
                    details: format!("Prepare list_features all failed: {e}"),
                })?;
        let rows = stmt.query_map([], map_row_to_feature).map_err(|e| {
            DomainViolation::InfrastructureBoundaryFault {
                subsystem: "feature_ops".to_string(),
                details: format!("Query list_features all failed: {e}"),
            }
        })?;
        for r in rows {
            out.push(r.map_err(|e| DomainViolation::InfrastructureBoundaryFault {
                subsystem: "feature_ops".to_string(),
                details: format!("Decode feature row failed: {e}"),
            })?);
        }
    }
    Ok(out)
}

pub fn update_feature_status(
    conn: &Connection,
    feature_id: &str,
    status: FeatureStatus,
    linked_cycle_id: &str,
    updated_at: &str,
) -> Result<(), DomainViolation> {
    conn.execute(
        "UPDATE milestone_features SET status = ?1, linked_cycle_id = ?2, updated_at = ?3 WHERE feature_id = ?4",
        params![status.as_str(), linked_cycle_id, updated_at, feature_id],
    )
    .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
        subsystem: "feature_ops".to_string(),
        details: format!("Failed updating status for feature {feature_id}: {e}"),
    })?;
    Ok(())
}
