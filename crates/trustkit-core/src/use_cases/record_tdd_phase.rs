// Record TDD Phase Use Case (TASK-003, TASK-006).
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::entities::{
    EvidenceKind, ExecutionEvidence, GateStage, GateVerdict, TddExecutionProof, TddPhase,
};
use crate::errors::DomainViolation;
use crate::ports::{
    AsyncExecutionSupervisorPort, CycleEvidenceRepositoryPort, SupervisedExecutionRequest,
};
use crate::special_cases::TddLifecycleState;

pub struct RedVerifier;

impl RedVerifier {
    pub fn verify(
        task_id: &str,
        exit_code: i32,
        stdout: &str,
        stderr: &str,
    ) -> Result<(), DomainViolation> {
        let combined = format!("{stdout}\n{stderr}");
        let combined_lower = combined.to_lowercase();

        if combined.contains("error[E")
            || combined_lower.contains("could not compile")
            || combined_lower.contains("compilation failed")
            || combined_lower.contains("no test target named")
            || combined_lower.contains("could not find `cargo.toml`")
            || combined_lower.contains("command not found")
            || combined_lower.contains("err_pnpm_no_script")
            || combined_lower.contains("missing script:")
            || combined_lower.contains("enoent")
        {
            let snippet = if !stderr.trim().is_empty() {
                stderr
                    .trim()
                    .lines()
                    .next()
                    .unwrap_or("test runner failure")
            } else {
                stdout
                    .trim()
                    .lines()
                    .next()
                    .unwrap_or("test runner failure")
            };
            return Err(DomainViolation::CompilationFailureInRedPhaseFault {
                task_id: task_id.to_string(),
                reason: format!(
                    "Test compilation or runner failure detected in RED phase: {snippet}"
                ),
            });
        }

        let has_passing_tests = combined
            .lines()
            .any(|l| l.contains("passed") && !l.contains("0 passed"));
        let has_failing_tests = combined
            .lines()
            .any(|l| l.contains("failed") && !l.contains("0 failed"));
        let has_executed_tests = has_passing_tests || has_failing_tests;

        if !has_executed_tests && combined.contains("0 passed; 0 failed") {
            return Err(DomainViolation::ZeroFilteredTestsFault {
                task_id: task_id.to_string(),
                reason: "Zero tests filtered or executed (0 passed; 0 failed)".to_string(),
            });
        }

        if exit_code == 0 {
            return Err(DomainViolation::TddLifecycleViolation {
                task_id: task_id.to_string(),
                reason: "Red phase requires a failing test execution (non-zero exit code)"
                    .to_string(),
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordTddPhaseRequest {
    pub cycle_id: String,
    pub task_id: String,
    pub phase: TddPhase,
    pub test_command: String,
    pub test_args: Vec<String>,
    pub working_dir: String,
    pub timeout_ms: u64,
    pub now_iso: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordTddPhaseOutcome {
    pub cycle_id: String,
    pub task_id: String,
    pub phase: TddPhase,
    pub new_state: TddLifecycleState,
    pub evidence_id: u64,
}

pub fn execute(
    req: RecordTddPhaseRequest,
    repo: &dyn CycleEvidenceRepositoryPort,
    supervisor: &dyn AsyncExecutionSupervisorPort,
) -> Result<RecordTddPhaseOutcome, DomainViolation> {
    let mut task = repo
        .find_atomic_task(&req.cycle_id, &req.task_id)?
        .ok_or_else(|| DomainViolation::TddLifecycleViolation {
            task_id: req.task_id.clone(),
            reason: format!(
                "Atomic task '{}' not found in cycle '{}'",
                req.task_id, req.cycle_id
            ),
        })?;

    // Pre-validation: MissingRedPhaseProof check (RN-002) before executing green command
    if req.phase == TddPhase::Green && !task.tdd_lifecycle_state.is_red_verified() {
        return Err(DomainViolation::TddLifecycleViolation {
            task_id: req.task_id,
            reason: "MissingRedPhaseProof: cannot transition to GreenVerified without prior RedVerified proof (RN-002)".to_string(),
        });
    }

    let exec_request = SupervisedExecutionRequest {
        command: req.test_command.clone(),
        args: req.test_args.clone(),
        working_dir: req.working_dir.clone(),
        timeout_ms: req.timeout_ms,
    };

    let outcome = supervisor.execute_supervised(&exec_request)?;

    if req.phase == TddPhase::Red {
        RedVerifier::verify(
            &req.task_id,
            outcome.exit_code,
            &outcome.stdout_tail,
            &outcome.stderr_tail,
        )?;
    }

    let is_phase_valid = match req.phase {
        TddPhase::Red => outcome.exit_code != 0,
        TddPhase::Green => outcome.exit_code == 0,
    };

    let evidence = ExecutionEvidence {
        evidence_id: 0,
        cycle_id: req.cycle_id.clone(),
        gate_stage: GateStage::Gate4TasksImplemented,
        gauntlet_attempt: 0,
        actor_role: format!("BUILDER:TDD_{}", req.phase.as_str()),
        evidence_kind: EvidenceKind::CmdExec,
        executed_command: format!("{} {}", req.test_command, req.test_args.join(" ")),
        exit_code: outcome.exit_code,
        duration_ms: outcome.duration_ms,
        sanitized_stdout_tail: outcome.stdout_tail,
        sanitized_stderr_tail: outcome.stderr_tail,
        output_blake3_digest: outcome.output_blake3_digest,
        verdict: if is_phase_valid {
            GateVerdict::Pass
        } else {
            GateVerdict::Fail
        },
        recorded_at: req.now_iso,
    };

    let evidence_id = repo.append_evidence(&evidence)?;
    let proof = TddExecutionProof::new(
        req.phase,
        outcome.exit_code,
        outcome.duration_ms,
        evidence_id,
    );

    match req.phase {
        TddPhase::Red => task.advance_red_phase(&proof)?,
        TddPhase::Green => task.advance_green_phase(&proof)?,
    }

    repo.update_task_tdd_phase(&req.cycle_id, &req.task_id, &task.tdd_lifecycle_state)?;

    Ok(RecordTddPhaseOutcome {
        cycle_id: req.cycle_id,
        task_id: req.task_id,
        phase: req.phase,
        new_state: task.tdd_lifecycle_state,
        evidence_id,
    })
}
