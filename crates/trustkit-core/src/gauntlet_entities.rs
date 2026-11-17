// Domain entities for Gauntlet V2 Swarm, Critics, TDD and Certification.
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle VI: Ubiquitous Language & Focused Modular Clean Code (< 250 LOC).

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TddPhase {
    Red,
    Green,
}

impl TddPhase {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Red => "RED",
            Self::Green => "GREEN",
        }
    }

    pub fn from_str_case_insensitive(value: &str) -> Option<Self> {
        match value.trim().to_uppercase().as_str() {
            "RED" => Some(Self::Red),
            "GREEN" => Some(Self::Green),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TddExecutionProof {
    pub phase: TddPhase,
    pub exit_code: i32,
    pub duration_ms: u64,
    pub evidence_id: u64,
}

impl TddExecutionProof {
    pub fn new(phase: TddPhase, exit_code: i32, duration_ms: u64, evidence_id: u64) -> Self {
        Self {
            phase,
            exit_code,
            duration_ms,
            evidence_id,
        }
    }

    pub fn validates_phase_expectation(&self) -> bool {
        match self.phase {
            TddPhase::Red => self.exit_code != 0,
            TddPhase::Green => self.exit_code == 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CriticKind {
    DomainAdversarial,
    ContractCompliance,
    StaticSecurity,
    PlaywrightVisual,
}

impl CriticKind {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::DomainAdversarial => "DOMAIN_ADVERSARIAL",
            Self::ContractCompliance => "CONTRACT_COMPLIANCE",
            Self::StaticSecurity => "STATIC_SECURITY",
            Self::PlaywrightVisual => "PLAYWRIGHT_VISUAL",
        }
    }

    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::DomainAdversarial => "Domain/Adversarial Critic",
            Self::ContractCompliance => "Contract Compliance Critic",
            Self::StaticSecurity => "Static Security Critic",
            Self::PlaywrightVisual => "Playwright Visual Critic",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CriticVerdict {
    Pass,
    Fail {
        reason: String,
        file: String,
        line: u32,
    },
}

impl CriticVerdict {
    pub fn pass() -> Self {
        Self::Pass
    }

    pub fn fail(reason: impl Into<String>, file: impl Into<String>, line: u32) -> Self {
        Self::Fail {
            reason: reason.into(),
            file: file.into(),
            line,
        }
    }

    pub const fn is_pass(&self) -> bool {
        matches!(self, Self::Pass)
    }

    pub const fn is_fail(&self) -> bool {
        matches!(self, Self::Fail { .. })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CriticFinding {
    pub kind: CriticKind,
    pub verdict: CriticVerdict,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CircuitBreakerState {
    Closed,
    Tripped {
        consecutive_failures: u8,
        diagnostic: String,
    },
}

impl CircuitBreakerState {
    pub const MAX_FAILURES: u8 = 5;

    pub const fn is_closed(&self) -> bool {
        matches!(self, Self::Closed)
    }

    pub const fn is_tripped(&self) -> bool {
        matches!(self, Self::Tripped { .. })
    }

    pub fn evaluate(consecutive_failures: u8, diagnostic: impl Into<String>) -> Self {
        if consecutive_failures >= Self::MAX_FAILURES {
            Self::Tripped {
                consecutive_failures,
                diagnostic: diagnostic.into(),
            }
        } else {
            Self::Closed
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CertificationSummary {
    pub cycle_id: String,
    pub layer0_clean_pass: bool,
    pub critic_findings: Vec<CriticFinding>,
    pub total_evidences_recorded: usize,
    pub circuit_breaker_state: CircuitBreakerState,
    pub certified_at: String,
}

impl CertificationSummary {
    pub fn is_fully_certified(&self) -> bool {
        self.layer0_clean_pass
            && self.circuit_breaker_state.is_closed()
            && !self.critic_findings.is_empty()
            && self.critic_findings.iter().all(|f| f.verdict.is_pass())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticBugLoopPlan {
    pub red_loop_command: String,
    pub minimized_repro: String,
    pub hypotheses: Vec<String>,
    pub debug_tag_prefix: String,
    pub seam_fix_strategy: String,
    pub cleanup_steps: Vec<String>,
}

impl DiagnosticBugLoopPlan {
    pub const fn phase_count(&self) -> usize {
        6
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.red_loop_command.trim().is_empty() {
            return Err("red_loop_command cannot be empty".to_string());
        }
        if self.minimized_repro.trim().is_empty() {
            return Err("minimized_repro cannot be empty".to_string());
        }
        if self.hypotheses.len() < 3 || self.hypotheses.len() > 5 {
            return Err("hypotheses count must be between 3 and 5".to_string());
        }
        if !self.debug_tag_prefix.starts_with("[DEBUG-") || !self.debug_tag_prefix.ends_with(']') {
            return Err("debug_tag_prefix must match [DEBUG-xxxx]".to_string());
        }
        if self.seam_fix_strategy.trim().is_empty() {
            return Err("seam_fix_strategy cannot be empty".to_string());
        }
        if self.cleanup_steps.is_empty() {
            return Err("cleanup_steps cannot be empty".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeDangerClassification {
    OneWayDoor,
    TwoWayDoor,
}

impl MergeDangerClassification {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::OneWayDoor => "OneWayDoor",
            Self::TwoWayDoor => "TwoWayDoor",
        }
    }

    pub const fn is_one_way(&self) -> bool {
        matches!(self, Self::OneWayDoor)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BlastRadiusLevel {
    Isolated,
    ModuleLocal,
    CrossCrate,
    SystemWide,
}

impl BlastRadiusLevel {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Isolated => "Isolated",
            Self::ModuleLocal => "ModuleLocal",
            Self::CrossCrate => "CrossCrate",
            Self::SystemWide => "SystemWide",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeDangerAssessment {
    pub classification: MergeDangerClassification,
    pub blast_radius: BlastRadiusLevel,
    pub reason: String,
    pub affected_files: Vec<String>,
}

impl MergeDangerAssessment {
    pub fn to_markdown_section(&self) -> String {
        format!(
            "## Merge Danger\n\n- Classification: {}\n- Blast Radius: {}\n- Details: {}\n- Files Evaluated: {}",
            self.classification.as_str(),
            self.blast_radius.as_str(),
            self.reason,
            self.affected_files.len()
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarnessRetroItem {
    pub category: String,
    pub severity: String,
    pub recommendation: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarnessRetroReport {
    pub cycle_id: String,
    pub items: Vec<HarnessRetroItem>,
}

impl HarnessRetroReport {
    pub fn new(cycle_id: impl Into<String>, items: Vec<HarnessRetroItem>) -> Self {
        Self {
            cycle_id: cycle_id.into(),
            items,
        }
    }

    pub fn to_markdown_section(&self) -> String {
        let mut section = String::from("## Retrospectiva de Harness\n\n");
        if self.items.is_empty() {
            section.push_str("- Nenhuma oportunidade de melhoria identificada no ciclo.\n");
        } else {
            for item in &self.items {
                section.push_str(&format!(
                    "- [{}] {}: {}\n",
                    item.severity, item.category, item.recommendation
                ));
            }
        }
        section
    }
}

#[cfg(test)]
#[path = "gauntlet_entities_tests.rs"]
mod tests;
