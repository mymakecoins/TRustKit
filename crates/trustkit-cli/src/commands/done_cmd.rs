// Handler for `trustkit done` (TASK-014).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::cli_contract::DoneArgs;
use crate::error_handler::DomainOutcome;
use crate::gate_presenter::{GatePresenter, JsonDoneResponse};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::CycleEvidenceRepositoryPort;
use trustkit_core::spec_entities::SpecificationV2;
use trustkit_core::special_cases::ActiveCycleLookup;
use trustkit_core::use_cases::certify_cycle_and_report::{self, CertifyCycleRequest};
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
    _args: &DoneArgs,
    json_mode: bool,
    db_path: &Path,
) -> Result<DomainOutcome, DomainViolation> {
    if !db_path.exists() {
        return Err(DomainViolation::InfrastructureBoundaryFault {
            subsystem: "done".to_string(),
            details: "No active SDD cycle found in state.db. Run 'trustkit init' first.".to_string(),
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

    let cert_reporter = CycleCertificationReporter::new();
    let now_iso = current_timestamp_iso();

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

    // Persist gauntlet-report.md in the specification directory
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
