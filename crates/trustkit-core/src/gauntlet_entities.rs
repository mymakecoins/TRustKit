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

#[cfg(test)]
#[path = "gauntlet_entities_tests.rs"]
mod tests;
