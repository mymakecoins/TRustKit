// Milestone Planning Entities
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::errors::DomainViolation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MilestoneStatus {
    Planned,
    InProgress,
    Completed,
    Cancelled,
}

impl MilestoneStatus {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "Planned",
            Self::InProgress => "InProgress",
            Self::Completed => "Completed",
            Self::Cancelled => "Cancelled",
        }
    }
}

impl std::fmt::Display for MilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for MilestoneStatus {
    type Err = DomainViolation;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "planned" => Ok(Self::Planned),
            "inprogress" | "in_progress" | "in-progress" => Ok(Self::InProgress),
            "completed" => Ok(Self::Completed),
            "cancelled" | "canceled" => Ok(Self::Cancelled),
            _ => Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "milestone_status".to_string(),
                details: format!("Unrecognized milestone status: '{s}'"),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceCitation {
    pub file_path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub section_title: String,
}

impl SourceCitation {
    pub fn new(
        file_path: impl Into<String>,
        start_line: usize,
        end_line: usize,
        section_title: impl Into<String>,
    ) -> Self {
        Self {
            file_path: file_path.into(),
            start_line,
            end_line,
            section_title: section_title.into(),
        }
    }

    pub fn format_citation(&self) -> String {
        if self.start_line == self.end_line {
            format!("{}#L{}", self.file_path, self.start_line)
        } else {
            format!("{}#L{}-L{}", self.file_path, self.start_line, self.end_line)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MilestoneCapability {
    pub capability_id: String,
    pub title: String,
    pub description: String,
    pub citation: SourceCitation,
}

impl MilestoneCapability {
    pub fn new(
        capability_id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
        citation: SourceCitation,
    ) -> Self {
        Self {
            capability_id: capability_id.into(),
            title: title.into(),
            description: description.into(),
            citation,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TechnicalConstraint {
    pub constraint_id: String,
    pub title: String,
    pub description: String,
    pub citation: SourceCitation,
}

impl TechnicalConstraint {
    pub fn new(
        constraint_id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
        citation: SourceCitation,
    ) -> Self {
        Self {
            constraint_id: constraint_id.into(),
            title: title.into(),
            description: description.into(),
            citation,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Milestone {
    pub milestone_id: String,
    pub plan_id: String,
    pub sequence_index: usize,
    pub title: String,
    pub slug: String,
    pub file_path: String,
    pub goal: String,
    pub dependencies: Vec<String>,
    pub capabilities: Vec<MilestoneCapability>,
    pub technical_constraints: Vec<TechnicalConstraint>,
    pub status: MilestoneStatus,
    pub created_at: String,
    pub updated_at: String,
}

impl Milestone {
    pub fn has_dependencies(&self) -> bool {
        !self.dependencies.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MilestonePlan {
    pub plan_id: String,
    pub prd_path: String,
    pub adr_path: String,
    pub milestones: Vec<Milestone>,
    pub created_at: String,
}

impl MilestonePlan {
    pub fn find_milestone(&self, id: &str) -> Option<&Milestone> {
        self.milestones.iter().find(|m| m.milestone_id == id)
    }

    pub fn total_capabilities(&self) -> usize {
        self.milestones.iter().map(|m| m.capabilities.len()).sum()
    }

    pub fn validate_dag(&self) -> Result<(), DomainViolation> {
        for m in &self.milestones {
            for dep in &m.dependencies {
                if dep == &m.milestone_id {
                    return Err(DomainViolation::CycleDetected {
                        cycle_path: vec![m.milestone_id.clone(), dep.clone()],
                        message: format!(
                            "Self-dependency detected in milestone {}",
                            m.milestone_id
                        ),
                    });
                }
            }
        }
        Ok(())
    }
}
