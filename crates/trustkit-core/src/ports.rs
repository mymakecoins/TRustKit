// Domain Port traits.
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle V: Separation of Construction and Execution.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::entities::{
    ArtifactSeal, AstAuditVerdict, ContextSliceRequest, ContextSliceResult, ExecutionEvidence,
    SddCycle, SpecificationV2, TaskNode, WaveSchedule,
};
use crate::errors::DomainViolation;
use crate::special_cases::{
    ActiveCycleLookup, BoardColumn, BoardSyncOutcome, IdentityVerification,
};

pub trait CycleEvidenceRepositoryPort: Send + Sync {
    fn save_cycle(&self, cycle: &SddCycle) -> Result<(), DomainViolation>;
    fn find_active_cycle(&self) -> Result<ActiveCycleLookup, DomainViolation>;
    fn save_artifact_seal(&self, seal: &ArtifactSeal) -> Result<u64, DomainViolation>;
    fn list_artifact_seals(&self, cycle_id: &str) -> Result<Vec<ArtifactSeal>, DomainViolation>;
    fn append_evidence(&self, evidence: &ExecutionEvidence) -> Result<u64, DomainViolation>;
    fn list_evidences_for_cycle(
        &self,
        cycle_id: &str,
    ) -> Result<Vec<ExecutionEvidence>, DomainViolation>;
    fn update_cycle_spec_lock(
        &self,
        cycle_id: &str,
        blake3_digest: &str,
        locked_state: &str,
    ) -> Result<(), DomainViolation>;
    fn save_atomic_tasks(
        &self,
        cycle_id: &str,
        tasks: &[TaskNode],
        wave_schedule: &WaveSchedule,
    ) -> Result<(), DomainViolation>;
}

pub trait ArtifactHasherPort: Send + Sync {
    fn compute_file_blake3(&self, path: &str) -> Result<String, DomainViolation>;
    fn compute_bytes_blake3(&self, bytes: &[u8]) -> String;
}

pub trait ProjectBoardGatewayPort: Send + Sync {
    fn verify_active_identity(
        &self,
        expected_handle: &str,
    ) -> Result<IdentityVerification, DomainViolation>;
    fn transition_issue_column(
        &self,
        issue_number: u64,
        target: BoardColumn,
    ) -> Result<BoardSyncOutcome, DomainViolation>;
}

pub trait PrdSpecificationIngestorPort: Send + Sync {
    fn ingest_from_prd(
        &self,
        prd_path: &str,
        custom_slug: Option<&str>,
    ) -> Result<SpecificationV2, DomainViolation>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecValidationReport {
    pub mermaid_valid: bool,
    pub gherkin_valid: bool,
    pub orphan_rules: Vec<String>,
    pub total_rules: usize,
    pub total_requirements: usize,
}

pub trait SpecValidatorPort: Send + Sync {
    fn validate_specification(
        &self,
        spec_content: &str,
    ) -> Result<SpecValidationReport, DomainViolation>;
}

pub trait WaveExecutionPlannerPort: Send + Sync {
    fn plan_waves(&self, tasks: &[TaskNode]) -> Result<WaveSchedule, DomainViolation>;
}

pub trait AstAuditorPort: Send + Sync {
    fn audit_source_tree(
        &self,
        files: &[String],
        reference_seals: &[ArtifactSeal],
    ) -> Result<AstAuditVerdict, DomainViolation>;
}

pub trait ContextSlicerPort: Send + Sync {
    fn slice_context(
        &self,
        request: &ContextSliceRequest,
    ) -> Result<ContextSliceResult, DomainViolation>;
}
