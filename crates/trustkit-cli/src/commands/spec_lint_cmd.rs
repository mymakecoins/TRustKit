// Handler for `trustkit spec lint` (TASK-005).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::cli_contract::SpecLintArgs;
use crate::error_handler::DomainOutcome;
use crate::gate_presenter::{GatePresenter, JsonSpecLintResponse, JsonSpecLintViolation};
use std::fs;
use std::path::Path;
use trustkit_ast::{SpecLintVerdict, SpecLinter};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::CycleEvidenceRepositoryPort;
use trustkit_core::special_cases::ActiveCycleLookup;
use trustkit_db::sqlite_adapter::SqliteWalRepositoryAdapter;

pub fn execute(args: &SpecLintArgs, json_mode: bool) -> Result<DomainOutcome, DomainViolation> {
    let db_path = Path::new(".trustkit").join("state.db");
    execute_with_db(args, json_mode, &db_path)
}

pub fn execute_with_db(
    args: &SpecLintArgs,
    json_mode: bool,
    db_path: &Path,
) -> Result<DomainOutcome, DomainViolation> {
    let target_spec_path = match &args.file {
        Some(path) if !path.trim().is_empty() => path.clone(),
        _ => {
            if !db_path.exists() {
                return Err(DomainViolation::InfrastructureBoundaryFault {
                    subsystem: "spec_lint".to_string(),
                    details: "No active SDD cycle found in state.db and no --file flag provided."
                        .to_string(),
                });
            }
            let repo = SqliteWalRepositoryAdapter::open(db_path)?;
            match repo.find_active_cycle()? {
                ActiveCycleLookup::Found(cycle) => cycle.spec_file_path,
                ActiveCycleLookup::NoActiveCycle => {
                    return Err(DomainViolation::InfrastructureBoundaryFault {
                        subsystem: "spec_lint".to_string(),
                        details: "No active SDD cycle found in state.db.".to_string(),
                    });
                }
            }
        }
    };

    let content = fs::read_to_string(&target_spec_path).map_err(|err| {
        DomainViolation::InfrastructureBoundaryFault {
            subsystem: "filesystem".to_string(),
            details: format!(
                "Failed reading specification file '{}': {err}",
                target_spec_path
            ),
        }
    })?;

    let verdict = SpecLinter::lint_markdown(&content);

    let (json_verdict, json_violations) = match &verdict {
        SpecLintVerdict::CleanPass => ("PASS".to_string(), Vec::new()),
        SpecLintVerdict::ViolationsFound(violations) => {
            let mapped = violations
                .iter()
                .map(|v| JsonSpecLintViolation {
                    code: v.code.clone(),
                    message: v.message.clone(),
                    line: v.line,
                    remediation: v.remediation.clone(),
                })
                .collect();
            ("FAIL".to_string(), mapped)
        }
    };

    let response = JsonSpecLintResponse {
        verdict: json_verdict,
        spec_file: target_spec_path.clone(),
        violations_count: json_violations.len(),
        violations: json_violations,
        next_steps: Vec::new(),
    };

    let rendered = GatePresenter::render_spec_lint(&response, json_mode || args.json);

    match verdict {
        SpecLintVerdict::CleanPass => {
            println!("{rendered}");
            Ok(DomainOutcome::Success)
        }
        SpecLintVerdict::ViolationsFound(violations) => {
            eprintln!("{rendered}");
            let has_invariant_violation = violations
                .iter()
                .any(|v| v.code == "SPEC-006" || v.code == "SPEC-007");

            if has_invariant_violation {
                Err(DomainViolation::AstIntegrityOrInvariantViolation {
                    invariant_id: "SPEC-007".to_string(),
                    intercepted_command: "spec lint".to_string(),
                    reason: format!(
                        "Specification '{}' contains invariant violations (AI trailers or emojis)",
                        target_spec_path
                    ),
                    remediation:
                        "Remove AI co-authorship trailers and emojis from the specification"
                            .to_string(),
                })
            } else {
                Err(DomainViolation::SpecOrTraceabilityIncomplete {
                    reason: format!(
                        "Specification '{}' has {} structural lint violations",
                        target_spec_path,
                        violations.len()
                    ),
                    missing_artefact:
                        "Valid specification structure conforming to SpecLinter rules".to_string(),
                })
            }
        }
    }
}
