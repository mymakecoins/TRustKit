// Security review policy parser and decision resolver (TASK-002).
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL & Special Cases.
// Constitutional Principle IV: Resilient Error Handling & Zero-Panic Policy.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::entities::{
    CliSecurityReviewFlag, SecurityReviewDecision, SecurityReviewMode, SecuritySkipReason,
};
use crate::errors::DomainViolation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SecurityReviewPolicyConfig {
    pub mode: SecurityReviewMode,
    pub min_confidence_threshold: u8,
}

impl Default for SecurityReviewPolicyConfig {
    fn default() -> Self {
        Self {
            mode: SecurityReviewMode::Prompt,
            min_confidence_threshold: 8,
        }
    }
}

impl SecurityReviewPolicyConfig {
    pub fn new(
        mode: SecurityReviewMode,
        min_confidence_threshold: u8,
    ) -> Result<Self, DomainViolation> {
        if !(8..=10).contains(&min_confidence_threshold) {
            return Err(DomainViolation::AstIntegrityOrInvariantViolation {
                invariant_id: "RN-002".to_string(),
                intercepted_command: "security_review_policy".to_string(),
                reason: format!(
                    "min_confidence_threshold {} is out of allowed range 8..=10",
                    min_confidence_threshold
                ),
                remediation: "Set min_confidence_threshold between 8 and 10 in trustkit.toml."
                    .to_string(),
            });
        }
        Ok(Self {
            mode,
            min_confidence_threshold,
        })
    }
}

pub fn parse_security_review_policy(
    raw_content: &str,
) -> Result<SecurityReviewPolicyConfig, DomainViolation> {
    let mut mode = SecurityReviewMode::Prompt;
    let mut min_confidence_threshold: u8 = 8;

    for line in raw_content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }

        let clean =
            trimmed.trim_matches(|c: char| c == ',' || c == '{' || c == '}' || c.is_whitespace());

        let kv = clean.split_once('=').or_else(|| clean.split_once(':'));

        let Some((raw_key, raw_val)) = kv else {
            continue;
        };

        let key = raw_key
            .trim()
            .trim_matches(|c: char| c == '"' || c == '\'')
            .to_lowercase();
        let val_without_comment = raw_val.split('#').next().unwrap_or(raw_val).trim();
        let val = val_without_comment
            .trim_matches(|c: char| c == '"' || c == '\'' || c == ',' || c.is_whitespace());

        if key == "security_review_mode" || key == "security_review" {
            match SecurityReviewMode::from_str_case_insensitive(val) {
                Some(parsed_mode) => mode = parsed_mode,
                None => {
                    return Err(DomainViolation::AstIntegrityOrInvariantViolation {
                        invariant_id: "RN-002".to_string(),
                        intercepted_command: "security_review_policy".to_string(),
                        reason: format!("Invalid security_review_mode value '{val}'"),
                        remediation:
                            "Use 'prompt', 'always', or 'disabled' for security_review_mode."
                                .to_string(),
                    });
                }
            }
        } else if key == "min_confidence_threshold" || key == "min_confidence" {
            match val.parse::<u8>() {
                Ok(parsed_threshold) => {
                    min_confidence_threshold = parsed_threshold;
                }
                Err(_) => {
                    return Err(DomainViolation::AstIntegrityOrInvariantViolation {
                        invariant_id: "RN-002".to_string(),
                        intercepted_command: "security_review_policy".to_string(),
                        reason: format!("Invalid min_confidence_threshold integer '{val}'"),
                        remediation:
                            "Provide an integer between 8 and 10 for min_confidence_threshold."
                                .to_string(),
                    });
                }
            }
        }
    }

    SecurityReviewPolicyConfig::new(mode, min_confidence_threshold)
}

pub fn resolve_security_review_decision(
    cli_flag: CliSecurityReviewFlag,
    policy: &SecurityReviewPolicyConfig,
    is_interactive_tty: bool,
    is_json_mode: bool,
) -> SecurityReviewDecision {
    match cli_flag {
        CliSecurityReviewFlag::ForceExecute => SecurityReviewDecision::Execute,
        CliSecurityReviewFlag::ForceSkip => SecurityReviewDecision::Skip {
            reason: SecuritySkipReason::CliFlagSkip,
        },
        CliSecurityReviewFlag::Unspecified => match policy.mode {
            SecurityReviewMode::Always => SecurityReviewDecision::Execute,
            SecurityReviewMode::Disabled => SecurityReviewDecision::Skip {
                reason: SecuritySkipReason::PolicyDisabled,
            },
            SecurityReviewMode::Prompt => {
                if is_json_mode || !is_interactive_tty {
                    SecurityReviewDecision::Skip {
                        reason: SecuritySkipReason::HeadlessNonInteractiveDefault,
                    }
                } else {
                    SecurityReviewDecision::PromptUser
                }
            }
        },
    }
}
