// Domain entities for Semantic Code Review (Gate 4) and Security Review (Gates 5 & 6).
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL & Special Cases.
// Constitutional Principle VI: Ubiquitous Language & Focused Modular Clean Code (< 250 LOC).

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CodeReviewEffortLevel {
    Low,
    Medium,
    High,
    Max,
}

impl CodeReviewEffortLevel {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "LOW",
            Self::Medium => "MEDIUM",
            Self::High => "HIGH",
            Self::Max => "MAX",
        }
    }

    pub const fn default_max_findings(&self) -> usize {
        match self {
            Self::Low => 4,
            Self::Medium => 8,
            Self::High => 10,
            Self::Max => 15,
        }
    }

    pub fn from_str_case_insensitive(value: &str) -> Option<Self> {
        match value.trim().to_uppercase().as_str() {
            "LOW" => Some(Self::Low),
            "MEDIUM" | "MED" => Some(Self::Medium),
            "HIGH" => Some(Self::High),
            "MAX" | "XHIGH" => Some(Self::Max),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReviewAngle {
    LogicCorrectness,
    RemovedBehavior,
    CrossFileContract,
    Reuse,
    Simplification,
    Efficiency,
    Altitude,
    ProjectConventions,
}

impl ReviewAngle {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::LogicCorrectness => "LOGIC_CORRECTNESS",
            Self::RemovedBehavior => "REMOVED_BEHAVIOR",
            Self::CrossFileContract => "CROSS_FILE_CONTRACT",
            Self::Reuse => "REUSE",
            Self::Simplification => "SIMPLIFICATION",
            Self::Efficiency => "EFFICIENCY",
            Self::Altitude => "ALTITUDE",
            Self::ProjectConventions => "PROJECT_CONVENTIONS",
        }
    }

    pub const fn is_correctness_angle(&self) -> bool {
        matches!(
            self,
            Self::LogicCorrectness | Self::RemovedBehavior | Self::CrossFileContract
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppSecCategory {
    InputValidationInjection,
    AuthAndAuthorization,
    CryptoAndSecrets,
    RemoteCodeExecutionXss,
    DataAndPiiExposure,
}

impl AppSecCategory {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::InputValidationInjection => "INPUT_VALIDATION_INJECTION",
            Self::AuthAndAuthorization => "AUTH_AND_AUTHORIZATION",
            Self::CryptoAndSecrets => "CRYPTO_AND_SECRETS",
            Self::RemoteCodeExecutionXss => "REMOTE_CODE_EXECUTION_XSS",
            Self::DataAndPiiExposure => "DATA_AND_PII_EXPOSURE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReviewSeverity {
    High,
    Medium,
    Low,
}

impl ReviewSeverity {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::High => "HIGH",
            Self::Medium => "MEDIUM",
            Self::Low => "LOW",
        }
    }

    pub const fn rank_weight(&self) -> u8 {
        match self {
            Self::High => 3,
            Self::Medium => 2,
            Self::Low => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecurityReviewMode {
    Prompt,
    Always,
    Disabled,
}

impl SecurityReviewMode {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Prompt => "PROMPT",
            Self::Always => "ALWAYS",
            Self::Disabled => "DISABLED",
        }
    }

    pub fn from_str_case_insensitive(value: &str) -> Option<Self> {
        match value.trim().to_uppercase().as_str() {
            "PROMPT" => Some(Self::Prompt),
            "ALWAYS" => Some(Self::Always),
            "DISABLED" | "NEVER" => Some(Self::Disabled),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CliSecurityReviewFlag {
    ForceExecute,
    ForceSkip,
    Unspecified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecuritySkipReason {
    CliFlagSkip,
    DeveloperPromptOptOut,
    HeadlessNonInteractiveDefault,
    PolicyDisabled,
}

impl SecuritySkipReason {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::CliFlagSkip => "CLI_FLAG_SKIP",
            Self::DeveloperPromptOptOut => "DEVELOPER_PROMPT_OPT_OUT",
            Self::HeadlessNonInteractiveDefault => "HEADLESS_NON_INTERACTIVE_DEFAULT",
            Self::PolicyDisabled => "POLICY_DISABLED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecurityReviewDecision {
    Execute,
    Skip { reason: SecuritySkipReason },
    PromptUser,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticReviewFinding {
    pub file: String,
    pub line: u32,
    pub severity: ReviewSeverity,
    pub category_or_angle: String,
    pub confidence: u8,
    pub summary: String,
    pub scenario: String,
    pub recommendation: String,
}

impl SemanticReviewFinding {
    pub fn meets_confidence_threshold(&self, threshold: u8) -> bool {
        self.confidence >= threshold
    }

    pub fn is_correctness_priority(&self) -> bool {
        matches!(
            self.category_or_angle.as_str(),
            "LOGIC_CORRECTNESS" | "REMOVED_BEHAVIOR" | "CROSS_FILE_CONTRACT"
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityReviewVerdict {
    CleanPass,
    SkippedByDeveloper {
        reason: SecuritySkipReason,
    },
    VulnerabilitiesDetected {
        findings: Vec<SemanticReviewFinding>,
    },
}

impl SecurityReviewVerdict {
    pub fn is_pass_or_skipped(&self) -> bool {
        matches!(self, Self::CleanPass | Self::SkippedByDeveloper { .. })
    }

    pub fn report_status_line(&self) -> String {
        match self {
            Self::CleanPass => "PASS (Confianca >= 8)".to_string(),
            Self::SkippedByDeveloper { .. } => "SKIPPED (Opt-out pelo desenvolvedor)".to_string(),
            Self::VulnerabilitiesDetected { findings } => {
                format!("FAIL ({} vulnerabilidades detectadas)", findings.len())
            }
        }
    }
}

#[cfg(test)]
#[path = "semantic_review_entities_tests.rs"]
mod tests;
