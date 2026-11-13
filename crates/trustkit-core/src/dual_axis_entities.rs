// Domain entities for Dual-Axis Semantic Review (RN-001 / RF-001).
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::gate_entities::GateVerdict;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReviewAxis {
    StandardsAndSmells,
    SpecCompliance,
}

impl ReviewAxis {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::StandardsAndSmells => "STANDARDS_AND_SMELLS",
            Self::SpecCompliance => "SPEC_COMPLIANCE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FowlerCodeSmell {
    MysteriousName,
    DuplicatedCode,
    FeatureEnvy,
    DataClumps,
    PrimitiveObsession,
    RepeatedSwitches,
    ShotgunSurgery,
    DivergentChange,
    SpeculativeGenerality,
    MessageChains,
    MiddleMan,
    RefusedBequest,
}

impl FowlerCodeSmell {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::MysteriousName => "MysteriousName",
            Self::DuplicatedCode => "DuplicatedCode",
            Self::FeatureEnvy => "FeatureEnvy",
            Self::DataClumps => "DataClumps",
            Self::PrimitiveObsession => "PrimitiveObsession",
            Self::RepeatedSwitches => "RepeatedSwitches",
            Self::ShotgunSurgery => "ShotgunSurgery",
            Self::DivergentChange => "DivergentChange",
            Self::SpeculativeGenerality => "SpeculativeGenerality",
            Self::MessageChains => "MessageChains",
            Self::MiddleMan => "MiddleMan",
            Self::RefusedBequest => "RefusedBequest",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StandardsFindingKind {
    RepositoryStandardViolation {
        rule_id: String,
        description: String,
    },
    FowlerSmell {
        smell: FowlerCodeSmell,
        description: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpecComplianceDefect {
    MissingOrPartialRequirement,
    WrongImplementation,
    ScopeCreep,
}

impl SpecComplianceDefect {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::MissingOrPartialRequirement => "MissingOrPartialRequirement",
            Self::WrongImplementation => "WrongImplementation",
            Self::ScopeCreep => "ScopeCreep",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DualAxisFindingKind {
    StandardsAndSmells(StandardsFindingKind),
    SpecCompliance {
        defect: SpecComplianceDefect,
        requirement_reference: String,
        details: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DualAxisFinding {
    pub file: String,
    pub line: u32,
    pub kind: DualAxisFindingKind,
    pub summary: String,
    pub recommendation: String,
}

impl DualAxisFinding {
    pub fn axis(&self) -> ReviewAxis {
        match &self.kind {
            DualAxisFindingKind::StandardsAndSmells(_) => ReviewAxis::StandardsAndSmells,
            DualAxisFindingKind::SpecCompliance { .. } => ReviewAxis::SpecCompliance,
        }
    }

    pub fn is_scope_creep(&self) -> bool {
        matches!(
            &self.kind,
            DualAxisFindingKind::SpecCompliance {
                defect: SpecComplianceDefect::ScopeCreep,
                ..
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AxisReviewResult {
    pub axis: ReviewAxis,
    pub verdict: GateVerdict,
    pub findings: Vec<DualAxisFinding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DualAxisReviewReport {
    pub task_id: String,
    pub standards_axis: AxisReviewResult,
    pub spec_axis: AxisReviewResult,
}

impl DualAxisReviewReport {
    pub fn new(
        task_id: String,
        standards_findings: Vec<DualAxisFinding>,
        spec_findings: Vec<DualAxisFinding>,
    ) -> Self {
        let standards_verdict = if standards_findings.is_empty() {
            GateVerdict::Pass
        } else {
            GateVerdict::Fail
        };
        let spec_verdict = if spec_findings.is_empty() {
            GateVerdict::Pass
        } else {
            GateVerdict::Fail
        };

        Self {
            task_id,
            standards_axis: AxisReviewResult {
                axis: ReviewAxis::StandardsAndSmells,
                verdict: standards_verdict,
                findings: standards_findings,
            },
            spec_axis: AxisReviewResult {
                axis: ReviewAxis::SpecCompliance,
                verdict: spec_verdict,
                findings: spec_findings,
            },
        }
    }

    pub fn overall_verdict(&self) -> GateVerdict {
        if self.standards_axis.verdict == GateVerdict::Pass
            && self.spec_axis.verdict == GateVerdict::Pass
        {
            GateVerdict::Pass
        } else {
            GateVerdict::Fail
        }
    }

    pub fn is_pass(&self) -> bool {
        self.overall_verdict() == GateVerdict::Pass
    }
}
