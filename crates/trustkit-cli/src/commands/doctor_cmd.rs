// Handler for `trustkit doctor` (TASK-014 & TASK-003).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::cli_contract::DoctorArgs;
use crate::error_handler::DomainOutcome;
use crate::gate_presenter::{GatePresenter, JsonDoctorResponse};
use std::fs;
use std::path::{Path, PathBuf};
use trustkit_core::entities::{
    CycleStatus, GateVerdict, HarnessExecutionPolicyConfig, parse_harness_execution_policy,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::CycleEvidenceRepositoryPort;
use trustkit_db::sqlite_adapter::SqliteWalRepositoryAdapter;
use trustkit_orch::WorktreeIsolationProvider;

pub fn resolve_state_db_path() -> PathBuf {
    let local = Path::new(".trustkit").join("state.db");
    if local.exists() {
        return local;
    }
    if let Ok(cwd) = std::env::current_dir() {
        for ancestor in cwd.ancestors() {
            let candidate = ancestor.join(".trustkit").join("state.db");
            if candidate.exists() {
                return candidate;
            }
        }
    }
    local
}

pub fn load_project_execution_policy(working_dir: &Path) -> HarnessExecutionPolicyConfig {
    let mut check_dirs = Vec::new();
    check_dirs.push(working_dir.to_path_buf());
    for ancestor in working_dir.ancestors() {
        if ancestor != working_dir && ancestor != Path::new("") {
            check_dirs.push(ancestor.to_path_buf());
        }
    }
    check_dirs.push(PathBuf::from("."));

    for dir in check_dirs {
        let toml_path = dir.join("trustkit.toml");
        if let Ok(raw) = fs::read_to_string(&toml_path)
            && let Ok(cfg) = parse_harness_execution_policy(&raw)
        {
            return cfg;
        }
        let json_path = dir.join(".trustkit").join("config.json");
        if let Ok(raw) = fs::read_to_string(&json_path)
            && let Ok(cfg) = parse_harness_execution_policy(&raw)
        {
            return cfg;
        }
    }
    HarnessExecutionPolicyConfig::default()
}

pub fn inspect_compilation_cache_diagnostic(
    sccache_available: bool,
    policy: &HarnessExecutionPolicyConfig,
) -> (String, String) {
    if !policy.caching.is_compilation_cache_enabled() {
        return (
            "DISABLED".to_string(),
            "Compilation cache: disabled via trustkit.toml policy.".to_string(),
        );
    }

    if sccache_available {
        (
            "SCCACHE_ACTIVE".to_string(),
            format!(
                "Compilation cache: sccache detected and active (RUSTC_WRAPPER=sccache, shared_target_dir='{}').",
                policy.caching.shared_target_dir
            ),
        )
    } else {
        (
            "SHARED_TARGET_FALLBACK".to_string(),
            format!(
                "Compilation cache: sccache not detected in PATH (fallback: shared target-dir '{}'). Install sccache via 'cargo install sccache' to accelerate multi-worktree compilation.",
                policy.caching.shared_target_dir
            ),
        )
    }
}

pub fn execute(args: &DoctorArgs, json_mode: bool) -> Result<DomainOutcome, DomainViolation> {
    let db_path = resolve_state_db_path();
    execute_with_db(args, json_mode, &db_path)
}

pub fn execute_with_db(
    args: &DoctorArgs,
    json_mode: bool,
    db_path: &Path,
) -> Result<DomainOutcome, DomainViolation> {
    let sccache_available = WorktreeIsolationProvider::detect_sccache_available();
    execute_with_db_and_sccache(args, json_mode, db_path, sccache_available)
}

pub fn execute_with_db_and_sccache(
    _args: &DoctorArgs,
    json_mode: bool,
    db_path: &Path,
    sccache_available: bool,
) -> Result<DomainOutcome, DomainViolation> {
    if !db_path.exists() {
        return Err(DomainViolation::InfrastructureBoundaryFault {
            subsystem: "doctor".to_string(),
            details: "No active SDD cycle found in state.db. Run 'trustkit init' first."
                .to_string(),
        });
    }

    let repo = SqliteWalRepositoryAdapter::open(db_path)?;
    let cycle = match repo.find_latest_cycle()? {
        Some(c) => c,
        None => {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "doctor".to_string(),
                details: "No active or recorded SDD cycle found. Run 'trustkit init' first."
                    .to_string(),
            });
        }
    };

    let is_tripped =
        cycle.lifecycle_status == CycleStatus::CircuitBreakerTripped || cycle.gauntlet_attempt >= 5;

    let evidences = repo.list_evidences_for_cycle(&cycle.cycle_id)?;
    let mut diagnostics = Vec::new();

    for ev in evidences.iter().rev() {
        if ev.verdict == GateVerdict::Fail {
            diagnostics.push(format!(
                "Failure in stage {:?} by actor {}: command `{}` exited with {}",
                ev.gate_stage, ev.actor_role, ev.executed_command, ev.exit_code
            ));
            if diagnostics.len() >= 5 {
                break;
            }
        }
    }

    if diagnostics.is_empty() {
        diagnostics.push("No failing evidence recorded in active cycle.".to_string());
    }

    let workspace_dir = db_path
        .parent()
        .and_then(|p| p.parent())
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let exec_policy = load_project_execution_policy(workspace_dir);
    let (cache_status, cache_diag) =
        inspect_compilation_cache_diagnostic(sccache_available, &exec_policy);
    diagnostics.push(cache_diag);

    let remediation = if is_tripped {
        "Circuit breaker tripped after 5 consecutive failures. Intervene manually to fix failing tests and reset status.".to_string()
    } else if !sccache_available && exec_policy.caching.is_compilation_cache_enabled() {
        "Cycle is healthy. Install sccache via 'cargo install sccache' to enable RUSTC_WRAPPER compilation caching, then continue TDD cycle or run gauntlet.".to_string()
    } else {
        "Cycle is healthy. Continue TDD cycle or run gauntlet.".to_string()
    };

    let verdict = if is_tripped { "BLOCKED" } else { "PASS" }.to_string();

    let response = JsonDoctorResponse {
        verdict,
        circuit_breaker_tripped: is_tripped,
        consecutive_failures: cycle.gauntlet_attempt,
        sccache_detected: sccache_available,
        compilation_cache_status: cache_status,
        batch_execution_enabled: exec_policy.batching.is_batching_active(),
        diagnostics,
        remediation: remediation.clone(),
        next_steps: Vec::new(),
    };

    println!("{}", GatePresenter::render_doctor(&response, json_mode));

    if is_tripped {
        Err(DomainViolation::CircuitBreakerTripped {
            cycle_id: cycle.cycle_id,
            consecutive_failures: cycle.gauntlet_attempt,
            remediation,
        })
    } else {
        Ok(DomainOutcome::Success)
    }
}
