// Gate 6 Done & PR Consolidation Verifier (TASK-005, RN-005, RF-005).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use trustkit_core::errors::DomainViolation;

pub struct Gate6ReportConsolidator;

impl Gate6ReportConsolidator {
    pub fn verify_report_integrity(report_content: &str) -> Result<(), DomainViolation> {
        if !report_content.contains("## Merge Danger") {
            return Err(DomainViolation::AstIntegrityOrInvariantViolation {
                invariant_id: "RF-005".to_string(),
                intercepted_command: "trustkit done".to_string(),
                reason: "gauntlet-report.md missing mandated '## Merge Danger' section".to_string(),
                remediation: "Ensure MergeDangerCalculator injects Merge Danger into report"
                    .to_string(),
            });
        }

        if !report_content.contains("## Retrospectiva de Harness") {
            return Err(DomainViolation::AstIntegrityOrInvariantViolation {
                invariant_id: "RF-005".to_string(),
                intercepted_command: "trustkit done".to_string(),
                reason: "gauntlet-report.md missing mandated '## Retrospectiva de Harness' section"
                    .to_string(),
                remediation: "Ensure HarnessRetroEngine injects retrospective into report"
                    .to_string(),
            });
        }

        if !report_content.is_ascii() {
            return Err(DomainViolation::AstIntegrityOrInvariantViolation {
                invariant_id: "CONSTITUTION-VII".to_string(),
                intercepted_command: "trustkit done".to_string(),
                reason: "gauntlet-report.md contains non-ASCII characters or prohibited emojis"
                    .to_string(),
                remediation: "Convert all report text to pure ASCII with zero emojis".to_string(),
            });
        }

        Ok(())
    }
}
