// Handler for `trustkit gauntlet ast-audit` (TASK-014).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::cli_contract::GauntletAstAuditArgs;
use crate::error_handler::DomainOutcome;
use crate::next_step_presenter::{JsonNextStep, NextStepPresenter};
use serde::Serialize;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use trustkit_ast::TreeSitterAstAuditorAdapter;
use trustkit_core::entities::{AstAuditVerdict, NextStepResolver};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::CycleEvidenceRepositoryPort;
use trustkit_core::special_cases::ActiveCycleLookup;
use trustkit_core::use_cases::audit_ast_layer0::{self, AuditAstLayer0Request};
use trustkit_db::sqlite_adapter::SqliteWalRepositoryAdapter;

#[derive(Debug, Serialize)]
pub struct JsonAstAuditSuccessResponse {
    pub status: String,
    pub verdict: String,
    pub cycle_id: String,
    pub audited_files: Vec<String>,
    pub total_violations: usize,
    pub recorded_evidence_id: u64,
    pub next_steps: Vec<JsonNextStep>,
}

pub fn execute(
    args: &GauntletAstAuditArgs,
    json_mode: bool,
) -> Result<DomainOutcome, DomainViolation> {
    let db_path = Path::new(".trustkit").join("state.db");
    execute_with_db(args, json_mode, &db_path)
}

pub fn execute_with_db(
    args: &GauntletAstAuditArgs,
    json_mode: bool,
    db_path: &Path,
) -> Result<DomainOutcome, DomainViolation> {
    if !db_path.exists() {
        return Err(DomainViolation::InfrastructureBoundaryFault {
            subsystem: "gauntlet_ast_audit".to_string(),
            details: "No active SDD cycle found in state.db. Run 'trustkit init' first."
                .to_string(),
        });
    }

    let repo = SqliteWalRepositoryAdapter::open(db_path)?;
    let active_cycle = match repo.find_active_cycle()? {
        ActiveCycleLookup::Found(cycle) => *cycle,
        ActiveCycleLookup::NoActiveCycle => {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "gauntlet_ast_audit".to_string(),
                details: "No active SDD cycle found. Run 'trustkit init' first.".to_string(),
            });
        }
    };

    let files = if args.files.is_empty() {
        collect_workspace_source_files(&active_cycle.spec_dir)
    } else {
        args.files.clone()
    };

    let now_iso = current_timestamp_iso();
    let auditor = TreeSitterAstAuditorAdapter::new();

    let request = AuditAstLayer0Request {
        cycle_id: active_cycle.cycle_id.clone(),
        files: files.clone(),
        now_iso,
    };

    let outcome = audit_ast_layer0::execute(request, &repo, &auditor)?;

    match outcome.verdict {
        AstAuditVerdict::CleanPass { .. } => {
            let steps = NextStepResolver::for_ast_audit();
            if json_mode || args.json {
                let response = JsonAstAuditSuccessResponse {
                    status: "PASS".to_string(),
                    verdict: "CLEAN_PASS".to_string(),
                    cycle_id: outcome.cycle_id,
                    audited_files: files,
                    total_violations: 0,
                    recorded_evidence_id: outcome.recorded_evidence_id,
                    next_steps: NextStepPresenter::to_json_vec(&steps),
                };
                if let Ok(json_str) = serde_json::to_string_pretty(&response) {
                    println!("{json_str}");
                }
            } else {
                let mut out = format!(
                    "[PASS] AST Layer 0 CleanPass: 0 violations detected across {} files.",
                    files.len()
                );
                out.push_str(&NextStepPresenter::render_ascii_block(&steps));
                println!("{out}");
            }
            Ok(DomainOutcome::Success)
        }
        AstAuditVerdict::Rejected { violations } => {
            let details = format!(
                "AST Layer 0 rejected: {} invariant/anti-cheat violation(s) detected.",
                violations.len()
            );
            Err(DomainViolation::AstIntegrityOrInvariantViolation {
                invariant_id: "RN-007,RN-013,AntiCheat".to_string(),
                intercepted_command: "trustkit gauntlet ast-audit".to_string(),
                reason: details,
                remediation: "Eliminate forbidden emojis, remove AI trailers, and restore stripped test assertions."
                    .to_string(),
            })
        }
    }
}

fn collect_workspace_source_files(spec_dir: &str) -> Vec<String> {
    let mut files = Vec::new();
    let spec_path = Path::new(spec_dir);
    if let Ok(entries) = std::fs::read_dir(spec_path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if let Some(ext) = p.extension()
                && (ext == "rs" || ext == "ts" || ext == "tsx")
            {
                files.push(p.display().to_string());
            }
        }
    }
    files
}

fn current_timestamp_iso() -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}-01-01T00:00:00Z", 1970 + duration.as_secs() / 31_536_000)
}
