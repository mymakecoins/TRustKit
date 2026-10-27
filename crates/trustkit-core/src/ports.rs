// Domain Port traits.
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle V: Separation of Construction and Execution.

use crate::entities::{ArtifactSeal, ExecutionEvidence, SddCycle};
use crate::errors::DomainViolation;
use crate::special_cases::{
    ActiveCycleLookup, BoardColumn, BoardSyncOutcome, IdentityVerification,
};

pub trait CycleEvidenceRepositoryPort {
    fn save_cycle(&self, cycle: &SddCycle) -> Result<(), DomainViolation>;
    fn find_active_cycle(&self) -> Result<ActiveCycleLookup, DomainViolation>;
    fn save_artifact_seal(&self, seal: &ArtifactSeal) -> Result<u64, DomainViolation>;
    fn list_artifact_seals(&self, cycle_id: &str) -> Result<Vec<ArtifactSeal>, DomainViolation>;
    fn append_evidence(&self, evidence: &ExecutionEvidence) -> Result<u64, DomainViolation>;
    fn list_evidences_for_cycle(
        &self,
        cycle_id: &str,
    ) -> Result<Vec<ExecutionEvidence>, DomainViolation>;
}

pub trait ArtifactHasherPort {
    fn compute_file_blake3(&self, path: &str) -> Result<String, DomainViolation>;
    fn compute_bytes_blake3(&self, bytes: &[u8]) -> String;
}

pub trait ProjectBoardGatewayPort {
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
