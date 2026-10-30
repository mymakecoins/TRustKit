// Domain Port traits.
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle V: Separation of Construction and Execution.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

pub use crate::port_dtos::*;

use crate::entities::{
    ArtifactSeal, AstAuditVerdict, AtomicTask, CertificationSummary, ContextSliceRequest,
    ContextSliceResult, CycleStatus, ExecutionEvidence, FileGraphManifest, GateStage,
    GraphHealthReport, GraphHealthVerdict, GraphTraversalOutcome, KnowledgeDocument,
    KnowledgeLookupOutcome, KnowledgeRebuildSummary, KnowledgeSearchOutcome, KnowledgeSearchQuery,
    KnowledgeWriteOutcome, SddCycle, ShortestPathOutcome, SpecificationV2, StructuralEdge,
    StructuralNode, TaskNode, TraversalMode, WaveSchedule, WikiLink,
};
use crate::errors::DomainViolation;
use crate::special_cases::{
    ActiveCycleLookup, BoardColumn, BoardSyncOutcome, IdentityVerification, TddLifecycleState,
    WorktreeAllocation,
};

pub trait CycleEvidenceRepositoryPort: Send + Sync {
    fn save_cycle(&self, cycle: &SddCycle) -> Result<(), DomainViolation>;
    fn find_active_cycle(&self) -> Result<ActiveCycleLookup, DomainViolation>;
    fn find_latest_cycle(&self) -> Result<Option<SddCycle>, DomainViolation> {
        match self.find_active_cycle()? {
            ActiveCycleLookup::Found(c) => Ok(Some(*c)),
            ActiveCycleLookup::NoActiveCycle => Ok(None),
        }
    }
    fn save_artifact_seal(&self, seal: &ArtifactSeal) -> Result<u64, DomainViolation>;
    fn list_artifact_seals(&self, cycle_id: &str) -> Result<Vec<ArtifactSeal>, DomainViolation>;
    fn append_evidence(&self, evidence: &ExecutionEvidence) -> Result<u64, DomainViolation>;
    fn list_evidences_for_cycle(&self, cycle_id: &str) -> Result<Vec<ExecutionEvidence>, DomainViolation>;
    fn update_cycle_spec_lock(&self, cycle_id: &str, blake3_digest: &str, locked_state: &str) -> Result<(), DomainViolation>;
    fn save_atomic_tasks(&self, cycle_id: &str, tasks: &[TaskNode], wave_schedule: &WaveSchedule) -> Result<(), DomainViolation>;

    fn update_task_tdd_phase(&self, _cycle_id: &str, _task_id: &str, _state: &TddLifecycleState) -> Result<(), DomainViolation> {
        Ok(())
    }
    fn update_task_worktree(&self, _cycle_id: &str, _task_id: &str, _allocation: &WorktreeAllocation) -> Result<(), DomainViolation> {
        Ok(())
    }
    fn find_atomic_task(&self, _cycle_id: &str, _task_id: &str) -> Result<Option<AtomicTask>, DomainViolation> {
        Ok(None)
    }
    fn list_atomic_tasks(&self, _cycle_id: &str) -> Result<Vec<AtomicTask>, DomainViolation> {
        Ok(vec![])
    }
    fn update_cycle_status(&self, _cycle_id: &str, _status: CycleStatus, _gauntlet_attempt: u8) -> Result<(), DomainViolation> {
        Ok(())
    }
    fn advance_cycle_gate(&self, _cycle_id: &str, _next_gate: GateStage) -> Result<(), DomainViolation> {
        Ok(())
    }
    fn list_certification_evidences(&self, cycle_id: &str) -> Result<Vec<ExecutionEvidence>, DomainViolation> {
        self.list_evidences_for_cycle(cycle_id)
    }
}

pub trait ArtifactHasherPort: Send + Sync {
    fn compute_file_blake3(&self, path: &str) -> Result<String, DomainViolation>;
    fn compute_bytes_blake3(&self, bytes: &[u8]) -> String;
}

pub trait ProjectBoardGatewayPort: Send + Sync {
    fn verify_active_identity(&self, expected_handle: &str) -> Result<IdentityVerification, DomainViolation>;
    fn transition_issue_column(&self, issue_number: u64, target: BoardColumn) -> Result<BoardSyncOutcome, DomainViolation>;
    fn create_pull_request(&self, _details: &PullRequestDetails) -> Result<PullRequestOutcome, DomainViolation> {
        Err(DomainViolation::InfrastructureBoundaryFault {
            subsystem: "project_board".to_string(),
            details: "create_pull_request not implemented".to_string(),
        })
    }
}

pub trait PrdSpecificationIngestorPort: Send + Sync {
    fn ingest_from_prd(&self, prd_path: &str, custom_slug: Option<&str>) -> Result<SpecificationV2, DomainViolation>;
}

pub trait SpecValidatorPort: Send + Sync {
    fn validate_specification(&self, spec_content: &str) -> Result<SpecValidationReport, DomainViolation>;
}

pub trait WaveExecutionPlannerPort: Send + Sync {
    fn plan_waves(&self, tasks: &[TaskNode]) -> Result<WaveSchedule, DomainViolation>;
}

pub trait AstAuditorPort: Send + Sync {
    fn audit_source_tree(&self, files: &[String], reference_seals: &[ArtifactSeal]) -> Result<AstAuditVerdict, DomainViolation>;
}

pub trait ContextSlicerPort: Send + Sync {
    fn slice_context(&self, request: &ContextSliceRequest) -> Result<ContextSliceResult, DomainViolation>;
}

pub trait WorktreeIsolationPort: Send + Sync {
    fn provision_worktree(&self, cycle_id: &str, task_id: &str, base_branch: &str) -> Result<WorktreeAllocation, DomainViolation>;
    fn check_worktree_status(&self, worktree_path: &str) -> Result<WorktreeStatus, DomainViolation>;
    fn merge_and_prune_worktree(&self, task_id: &str, worktree_path: &str, target_branch: &str) -> Result<String, DomainViolation>;
}

pub trait AsyncExecutionSupervisorPort: Send + Sync {
    fn execute_supervised(&self, request: &SupervisedExecutionRequest) -> Result<SupervisedExecutionOutcome, DomainViolation>;
}

pub trait VisualEvidencePort: Send + Sync {
    fn restore_foreign_screenshots(&self, active_issue_prefix: &str, evidence_dir: &str) -> Result<ForeignScreenshotInspection, DomainViolation>;
}

pub trait CycleCertificationPort: Send + Sync {
    fn render_gauntlet_report(&self, summary: &CertificationSummary, evidences: &[ExecutionEvidence]) -> Result<String, DomainViolation>;
    fn format_issue_completion_comment(&self, summary: &CertificationSummary, spec: &SpecificationV2) -> Result<String, DomainViolation>;
}

pub trait KnowledgeRepositoryPort: Send + Sync {
    fn write_document(&self, document: &KnowledgeDocument) -> Result<KnowledgeWriteOutcome, DomainViolation>;
    fn read_document(&self, slug: &str) -> Result<KnowledgeLookupOutcome, DomainViolation>;
    fn search_documents(&self, query: &KnowledgeSearchQuery) -> Result<KnowledgeSearchOutcome, DomainViolation>;
    fn list_documents(&self) -> Result<Vec<KnowledgeDocument>, DomainViolation>;
    fn query_adjacent_links(&self, slug: &str) -> Result<Vec<WikiLink>, DomainViolation> {
        let _ = slug;
        Ok(vec![])
    }
    fn rebuild_index(&self) -> Result<KnowledgeRebuildSummary, DomainViolation>;
}

pub trait StructuralGraphRepositoryPort: Send + Sync {
    fn detect_changed_files(
        &self,
        current_manifests: &[FileGraphManifest],
    ) -> Result<Vec<String>, DomainViolation>;
    fn save_graph_snapshot(
        &self,
        nodes: &[StructuralNode],
        edges: &[StructuralEdge],
        manifests: &[FileGraphManifest],
        force: bool,
    ) -> Result<GraphHealthVerdict, DomainViolation>;
    fn load_nodes(&self) -> Result<Vec<StructuralNode>, DomainViolation>;
    fn load_edges(&self) -> Result<Vec<StructuralEdge>, DomainViolation>;
    fn load_manifests(&self) -> Result<Vec<FileGraphManifest>, DomainViolation> {
        Ok(vec![])
    }
}

pub trait StructuralGraphAnalyzerPort: Send + Sync {
    fn extract_from_files(
        &self,
        _file_paths: &[String],
    ) -> Result<(Vec<StructuralNode>, Vec<StructuralEdge>), DomainViolation> {
        Ok((vec![], vec![]))
    }
    fn audit_graph_health(
        &self,
        nodes: &[StructuralNode],
        edges: &[StructuralEdge],
        top_god_nodes: usize,
    ) -> Result<GraphHealthReport, DomainViolation>;
    fn find_shortest_path(
        &self,
        nodes: &[StructuralNode],
        edges: &[StructuralEdge],
        source_id: &str,
        target_id: &str,
    ) -> Result<ShortestPathOutcome, DomainViolation>;
    fn slice_graph_context(
        &self,
        nodes: &[StructuralNode],
        edges: &[StructuralEdge],
        root_file: &str,
        mode: TraversalMode,
        max_hops: usize,
        max_line_budget: usize,
    ) -> Result<GraphTraversalOutcome, DomainViolation>;
}
