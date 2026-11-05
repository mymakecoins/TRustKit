// Domain entities representing SDD Gate stages, statuses and execution evidences.
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle VI: Ubiquitous Language & Focused Modular Clean Code (< 250 LOC).

use crate::special_cases::{ContractUnlockState, TraceVerificationState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MethodologyProfile {
    StandardGauntletV2,
    StructuredSdd { feature_dir: String },
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
    CodeReview,
    SecurityReview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateVerdict {
    Pass,
    Fail,
    Info,
    Blocked,
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
