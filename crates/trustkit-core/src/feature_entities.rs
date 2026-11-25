// Feature Planning Entities (Level 2 Meso Decomposition from Milestones).
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::errors::DomainViolation;
use crate::milestone_entities::{MilestoneCapability, TechnicalConstraint};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureStatus {
    Planned,
    InProgress,
    Completed,
    Cancelled,
}

impl FeatureStatus {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "Planned",
            Self::InProgress => "InProgress",
            Self::Completed => "Completed",
            Self::Cancelled => "Cancelled",
        }
    }
}

impl std::fmt::Display for FeatureStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for FeatureStatus {
    type Err = DomainViolation;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "planned" => Ok(Self::Planned),
            "inprogress" | "in_progress" | "in-progress" => Ok(Self::InProgress),
            "completed" => Ok(Self::Completed),
            "cancelled" | "canceled" => Ok(Self::Cancelled),
            _ => Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "feature_status".to_string(),
                details: format!("Unrecognized feature status: '{s}'"),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchitecturalTaskLayer {
    SchemaAndContract,
    CoreAndDomain,
    ApiUiAndAdapters,
    TelemetryAuditAndE2e,
}

impl ArchitecturalTaskLayer {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::SchemaAndContract => "SCHEMA_AND_CONTRACT",
            Self::CoreAndDomain => "CORE_AND_DOMAIN",
            Self::ApiUiAndAdapters => "API_UI_AND_ADAPTERS",
            Self::TelemetryAuditAndE2e => "TELEMETRY_AUDIT_AND_E2E",
        }
    }

    pub const fn display_label(&self) -> &'static str {
        match self {
            Self::SchemaAndContract => "Schema e Contrato",
            Self::CoreAndDomain => "Core e Dominio",
            Self::ApiUiAndAdapters => "API, UI e Adaptadores",
            Self::TelemetryAuditAndE2e => "Telemetria, Auditoria e Testes E2E",
        }
    }
}

impl std::str::FromStr for ArchitecturalTaskLayer {
    type Err = DomainViolation;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_uppercase().as_str() {
            "SCHEMA_AND_CONTRACT" | "SCHEMA" | "CONTRACT" => Ok(Self::SchemaAndContract),
            "CORE_AND_DOMAIN" | "CORE" | "DOMAIN" => Ok(Self::CoreAndDomain),
            "API_UI_AND_ADAPTERS" | "API" | "UI" | "ADAPTERS" => Ok(Self::ApiUiAndAdapters),
            "TELEMETRY_AUDIT_AND_E2E" | "E2E" | "TELEMETRY" => Ok(Self::TelemetryAuditAndE2e),
            _ => Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "architectural_task_layer".to_string(),
                details: format!("Unrecognized architectural task layer: '{s}'"),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureTaskBlueprint {
    pub task_id: String,
    pub layer: ArchitecturalTaskLayer,
    pub title: String,
    pub business_rule_id: String,
    pub functional_req_id: String,
    pub mermaid_node: String,
    pub target_test_file: String,
    pub evidence_artifact: String,
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedFeature {
    pub feature_id: String,
    pub milestone_id: String,
    pub sequence_index: usize,
    pub title: String,
    pub slug: String,
    pub spec_dir: String,
    pub spec_file_path: String,
    pub goal: String,
    pub dependencies: Vec<String>,
    pub capabilities: Vec<MilestoneCapability>,
    pub technical_constraints: Vec<TechnicalConstraint>,
    pub task_blueprints: Vec<FeatureTaskBlueprint>,
    pub status: FeatureStatus,
    pub linked_cycle_id: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeaturePlan {
    pub milestone_id: String,
    pub milestone_title: String,
    pub features: Vec<PlannedFeature>,
    pub created_at: String,
}

impl FeaturePlan {
    pub fn find_feature(&self, id: &str) -> Option<&PlannedFeature> {
        self.features.iter().find(|f| f.feature_id == id)
    }

    pub fn total_tasks(&self) -> usize {
        self.features.iter().map(|f| f.task_blueprints.len()).sum()
    }

    pub fn validate_dag(&self) -> Result<(), DomainViolation> {
        for f in &self.features {
            for dep in &f.dependencies {
                if dep == &f.feature_id {
                    return Err(DomainViolation::CycleDetected {
                        cycle_path: vec![f.feature_id.clone(), dep.clone()],
                        message: format!("Self-dependency detected in feature {}", f.feature_id),
                    });
                }
            }
        }
        Ok(())
    }
}
