// Handler for `trustkit done` (TASK-014 & TASK-005).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::cli_contract::DoneArgs;
use crate::commands::gauntlet_run_cmd::{
    DefaultSecurityDiffReviewer, StdioInteractivePrompt, load_project_security_policy,
};
use crate::error_handler::DomainOutcome;
use crate::gate_presenter::{GatePresenter, JsonDoneResponse};
use std::fs;
use std::io::{self, IsTerminal};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use trustkit_core::entities::{
    CliSecurityReviewFlag, EvidenceKind, SecurityReviewDecision, SecurityReviewVerdict,
    SecuritySkipReason,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::{
    CycleEvidenceRepositoryPort, InteractivePromptPort, SecurityDiffReviewerPort,
};
use trustkit_core::security_review_policy::resolve_security_review_decision;
use trustkit_core::spec_entities::SpecificationV2;
use trustkit_core::special_cases::ActiveCycleLookup;
use trustkit_core::use_cases::certify_cycle_and_report::{self, CertifyCycleRequest};
use trustkit_core::use_cases::run_security_review::{self, RunSecurityReviewRequest};
use trustkit_db::sqlite_adapter::SqliteWalRepositoryAdapter;
use trustkit_orch::CycleCertificationReporter;

fn current_timestamp_iso() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{now}")
}

pub fn execute(args: &DoneArgs, json_mode: bool) -> Result<DomainOutcome, DomainViolation> {
    let db_path = Path::new(".trustkit").join("state.db");
    execute_with_db(args, json_mode, &db_path)
}

pub fn execute_with_db(
    args: &DoneArgs,
    json_mode: bool,
    db_path: &Path,
) -> Result<DomainOutcome, DomainViolation> {
    let is_tty = io::stdin().is_terminal() && !json_mode && !args.json;
    let prompt = StdioInteractivePrompt;
    let reviewer = DefaultSecurityDiffReviewer;
    execute_with_prompt(args, json_mode, db_path, is_tty, &prompt, &reviewer)
}

pub fn execute_with_prompt(
    args: &DoneArgs,
    json_mode: bool,
    db_path: &Path,
    is_interactive_tty: bool,
    prompt_port: &dyn InteractivePromptPort,
    security_reviewer: &dyn SecurityDiffReviewerPort,
) -> Result<DomainOutcome, DomainViolation> {
    if !db_path.exists() {
        return Err(DomainViolation::InfrastructureBoundaryFault {
            subsystem: "done".to_string(),
            details: "No active SDD cycle found in state.db. Run 'trustkit init' first."
                .to_string(),
        });
    }

    let repo = SqliteWalRepositoryAdapter::open(db_path)?;
    let active_cycle = match repo.find_active_cycle()? {
        ActiveCycleLookup::Found(cycle) => *cycle,
        ActiveCycleLookup::NoActiveCycle => {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "done".to_string(),
                details: "No active SDD cycle found. Run 'trustkit init' first.".to_string(),
            });
        }
    };

    let now_iso = current_timestamp_iso();
    let existing_evidences = repo.list_evidences_for_cycle(&active_cycle.cycle_id)?;
    let has_security_review_evidence = existing_evidences
        .iter()
        .any(|e| e.evidence_kind == EvidenceKind::SecurityReview);
    let cli_flag = args.resolve_cli_security_flag();

    if !has_security_review_evidence || cli_flag != CliSecurityReviewFlag::Unspecified {
        let policy = load_project_security_policy(Path::new(&active_cycle.spec_dir));
        let initial_decision = resolve_security_review_decision(
            cli_flag,
            &policy,
            is_interactive_tty,
            json_mode || args.json,
        );
        let final_decision = match initial_decision {
            SecurityReviewDecision::PromptUser => {
                if prompt_port.prompt_security_review_confirmation()? {
                    SecurityReviewDecision::Execute
                } else {
                    SecurityReviewDecision::Skip {
                        reason: SecuritySkipReason::DeveloperPromptOptOut,
                    }
                }
            }
            other => other,
        };

        let sec_outcome = run_security_review::execute(
            RunSecurityReviewRequest {
                cycle_id: active_cycle.cycle_id.clone(),
                target_branch: "develop".to_string(),
                min_confidence_threshold: policy.min_confidence_threshold,
                decision: final_decision,
                now_iso: now_iso.clone(),
            },
            &repo,
            security_reviewer,
        )?;

        if let SecurityReviewVerdict::VulnerabilitiesDetected { findings } = sec_outcome.verdict {
            let should_abort = if is_interactive_tty && !json_mode && !args.json {
                prompt_port.prompt_abort_on_vulnerabilities(&findings)?
            } else {
                true
            };
            if should_abort {
                return Err(DomainViolation::CriticSwarmOrPlaywrightFailed {
                    layer: "SecurityReview".to_string(),
                    attempt: active_cycle.gauntlet_attempt,
                    failure_summary: format!(
                        "{} high-confidence security vulnerability(ies) detected before Gate 6 certification",
                        findings.len()
                    ),
                });
            }
        }
    }

    let cert_reporter = CycleCertificationReporter::new();
    let spec = SpecificationV2 {
        cycle_id: active_cycle.cycle_id.clone(),
        feature_slug: active_cycle.cycle_id.clone(),
        spec_file_path: active_cycle.spec_file_path.clone(),
        business_rules: Vec::new(),
        functional_requirements: Vec::new(),
        mermaid_content: String::new(),
        gherkin_scenarios: Vec::new(),
        traceability_matrix: Vec::new(),
        lock_state: active_cycle.spec_seal_state.clone(),
    };

    let request = CertifyCycleRequest {
        cycle_id: active_cycle.cycle_id.clone(),
        spec,
        now_iso,
    };

    let outcome = certify_cycle_and_report::execute(request, &repo, &cert_reporter)?;

    let spec_dir_path = Path::new(&active_cycle.spec_dir);
    if !spec_dir_path.exists() {
        let _ = fs::create_dir_all(spec_dir_path);
    }

    let report_file_path = if active_cycle.spec_dir.is_empty() {
        Path::new("gauntlet-report.md").to_path_buf()
    } else {
        spec_dir_path.join("gauntlet-report.md")
    };

    fs::write(&report_file_path, &outcome.report_markdown).map_err(|e| {
        DomainViolation::InfrastructureBoundaryFault {
            subsystem: "done_report".to_string(),
            details: format!("Failed to write gauntlet-report.md: {e}"),
        }
    })?;

    let response = JsonDoneResponse {
        verdict: "PASS".to_string(),
        cycle_id: outcome.cycle_id,
        gate: "Gate6Certified".to_string(),
        report_file: report_file_path.to_string_lossy().to_string(),
        issue_comment: outcome.completion_comment,
    };

    println!("{}", GatePresenter::render_done(&response, json_mode));
    Ok(DomainOutcome::Success)
}
