// Plain Old Rust Structs (PORS) Domain Entities.
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle VI: Ubiquitous Language & Focused Modular Clean Code (< 250 LOC).

use crate::special_cases::{
    ContractUnlockState, IssueLinkage, SpecLockState, TddLifecycleState,
    TraceVerificationState, WorktreeAllocation,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MethodologyProfile {
    StandardGauntletV2,
    SpecKitIntegrated { feature_dir: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateStage {
    Gate0Discovery = 0,
    Gate1SpecDrafting = 1,
    Gate2SpecSealed = 2,
    Gate3WavePlanned = 3,
    Gate4TasksImplemented = 4,
    Gate5GauntletCleared = 5,
    Gate6Certified = 6,
}

impl GateStage {
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0 => Some(Self::Gate0Discovery),
            1 => Some(Self::Gate1SpecDrafting),
            2 => Some(Self::Gate2SpecSealed),
            3 => Some(Self::Gate3WavePlanned),
            4 => Some(Self::Gate4TasksImplemented),
            5 => Some(Self::Gate5GauntletCleared),
            6 => Some(Self::Gate6Certified),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Gate0Discovery => "GATE_0_DISCOVERY",
            Self::Gate1SpecDrafting => "GATE_1_SPEC_DRAFTING",
            Self::Gate2SpecSealed => "GATE_2_SPEC_SEALED",
            Self::Gate3WavePlanned => "GATE_3_WAVE_PLANNED",
            Self::Gate4TasksImplemented => "GATE_4_TASKS_IMPLEMENTED",
            Self::Gate5GauntletCleared => "GATE_5_GAUNTLET_CLEARED",
            Self::Gate6Certified => "GATE_6_CERTIFIED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CycleStatus {
    Active,
    Paused,
    CircuitBreakerTripped,
    Certified,
    Merged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactCategory {
    Spec,
    Constitution,
    PreExistingTest,
    ForeignIssueEvidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceKind {
    CmdExec,
    AstAudit,
    BoardSync,
    SpecSeal,
    CriticVerdict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateVerdict {
    Pass,
    Fail,
    Info,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SddCycle {
    pub cycle_id: String,
    pub issue_linkage: IssueLinkage,
    pub methodology_profile: MethodologyProfile,
    pub current_gate: GateStage,
    pub lifecycle_status: CycleStatus,
    pub gauntlet_attempt: u8,
    pub spec_dir: String,
    pub spec_file_path: String,
    pub spec_seal_state: SpecLockState,
    pub work_branch: String,
    pub base_commit_sha: String,
    pub pre_existing_dirty_files: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactSeal {
    pub seal_id: u64,
    pub cycle_id: String,
    pub file_path: String,
    pub artifact_category: ArtifactCategory,
    pub blake3_digest: String,
    pub ast_assertion_count: u32,
    pub contract_unlock_state: ContractUnlockState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceabilityLink {
    pub trace_id: u64,
    pub cycle_id: String,
    pub business_rule_id: String,
    pub functional_req_id: String,
    pub task_id: String,
    pub mermaid_journey_node: String,
    pub playwright_spec_path: String,
    pub evidence_screenshot: String,
    pub verification_state: TraceVerificationState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomicTask {
    pub cycle_id: String,
    pub task_id: String,
    pub title: String,
    pub architectural_layer: String,
    pub wave_index: u32,
    pub depends_on: Vec<String>,
    pub expected_files: Vec<String>,
    pub target_test_file: String,
    pub tdd_lifecycle_state: TddLifecycleState,
    pub red_evidence_id: u64,
    pub green_evidence_id: u64,
    pub worktree_allocation: WorktreeAllocation,
    pub commit_sha: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionEvidence {
    pub evidence_id: u64,
    pub cycle_id: String,
    pub gate_stage: GateStage,
    pub gauntlet_attempt: u8,
    pub actor_role: String,
    pub evidence_kind: EvidenceKind,
    pub executed_command: String,
    pub exit_code: i32,
    pub duration_ms: u64,
    pub sanitized_stdout_tail: String,
    pub sanitized_stderr_tail: String,
    pub output_blake3_digest: String,
    pub verdict: GateVerdict,
    pub recorded_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gate_stage_conversions() {
        assert_eq!(GateStage::from_u8(0), Some(GateStage::Gate0Discovery));
        assert_eq!(GateStage::from_u8(6), Some(GateStage::Gate6Certified));
        assert_eq!(GateStage::from_u8(7), None);
        assert_eq!(GateStage::Gate0Discovery.label(), "GATE_0_DISCOVERY");
    }

    #[test]
    fn test_sdd_cycle_construction() {
        let cycle = SddCycle {
            cycle_id: "issue-45-202610271400".to_string(),
            issue_linkage: IssueLinkage::UnlinkedLocalCycle {
                directory_slug: "issue-45-202610271400".to_string(),
            },
            methodology_profile: MethodologyProfile::StandardGauntletV2,
            current_gate: GateStage::Gate0Discovery,
            lifecycle_status: CycleStatus::Active,
            gauntlet_attempt: 0,
            spec_dir: "specs/001".to_string(),
            spec_file_path: "specs/001/spec.md".to_string(),
            spec_seal_state: SpecLockState::DraftUnverified,
            work_branch: "feat/45-core".to_string(),
            base_commit_sha: "0123456789012345678901234567890123456789".to_string(),
            pre_existing_dirty_files: vec![],
            created_at: "2026-10-27T14:00:00Z".to_string(),
            updated_at: "2026-10-27T14:00:00Z".to_string(),
        };
        assert_eq!(cycle.gauntlet_attempt, 0);
    }
}
