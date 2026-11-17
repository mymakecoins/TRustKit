// Run Execution Wave Use Case (TASK-004).
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::entities::{EvidenceKind, ExecutionEvidence, GateStage, GateVerdict};
use crate::errors::DomainViolation;
use crate::ports::{CycleEvidenceRepositoryPort, WorktreeIsolationPort};
use crate::special_cases::{ActiveCycleLookup, SpecLockState, WorktreeAllocation};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunExecutionWaveRequest {
    pub cycle_id: String,
    pub wave_index: u32,
    pub now_iso: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutedTaskReport {
    pub task_id: String,
    pub worktree_path: String,
    pub merged_commit_sha: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunExecutionWaveOutcome {
    pub cycle_id: String,
    pub wave_index: u32,
    pub tasks_executed: Vec<ExecutedTaskReport>,
}

pub fn validate_commit_message(msg: &str) -> Result<(), DomainViolation> {
    let trimmed = msg.trim();
    let parts: Vec<&str> = trimmed.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(DomainViolation::AstIntegrityOrInvariantViolation {
            invariant_id: "RN-013".to_string(),
            intercepted_command: "git commit".to_string(),
            reason: "Commit message must follow pattern '<type>: #<id> <descricao>'".to_string(),
            remediation: "Format commit message as: feat: #45 implementa x".to_string(),
        });
    }

    let commit_type = parts[0].trim();
    let valid_types = [
        "feat", "fix", "test", "refactor", "docs", "chore", "perf", "build", "ci", "style",
    ];
    if !valid_types.contains(&commit_type) {
        return Err(DomainViolation::AstIntegrityOrInvariantViolation {
            invariant_id: "RN-013".to_string(),
            intercepted_command: "git commit".to_string(),
            reason: format!(
                "Invalid commit type '{}'. Must be one of {:?}",
                commit_type, valid_types
            ),
            remediation: "Use valid conventional commit prefix like 'feat', 'fix', 'test'"
                .to_string(),
        });
    }

    let remainder = parts[1].trim();
    if !remainder.starts_with('#') {
        return Err(DomainViolation::AstIntegrityOrInvariantViolation {
            invariant_id: "RN-013".to_string(),
            intercepted_command: "git commit".to_string(),
            reason: "Commit description must start with issue number '#<id>'".to_string(),
            remediation: "Include issue number right after prefix, e.g. 'feat: #45 ...'"
                .to_string(),
        });
    }

    Ok(())
}

pub fn execute(
    req: RunExecutionWaveRequest,
    repo: &dyn CycleEvidenceRepositoryPort,
    worktree_port: &dyn WorktreeIsolationPort,
) -> Result<RunExecutionWaveOutcome, DomainViolation> {
    let cycle = match repo.find_active_cycle()? {
        ActiveCycleLookup::Found(c) => *c,
        ActiveCycleLookup::NoActiveCycle => {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "run_execution_wave".to_string(),
                details: format!("No active SDD cycle found for cycle_id '{}'", req.cycle_id),
            });
        }
    };

    match &cycle.spec_seal_state {
        SpecLockState::Frozen { .. } => (),
        SpecLockState::DraftUnverified => {
            return Err(DomainViolation::SpecOrTraceabilityIncomplete {
                reason: "Cannot execute wave: specification must be frozen (RN-007)".to_string(),
                missing_artefact: "FROZEN specification lock".to_string(),
            });
        }
    }

    let all_tasks = repo.list_atomic_tasks(&cycle.cycle_id)?;
    let wave_tasks: Vec<_> = all_tasks
        .into_iter()
        .filter(|t| t.wave_index == req.wave_index)
        .collect();

    if wave_tasks.is_empty() {
        return Err(DomainViolation::InfrastructureBoundaryFault {
            subsystem: "run_execution_wave".to_string(),
            details: format!(
                "No tasks found for wave index {} in cycle '{}'",
                req.wave_index, cycle.cycle_id
            ),
        });
    }

    let mut executed_reports = Vec::with_capacity(wave_tasks.len());

    for mut task in wave_tasks {
        if task.worktree_allocation.is_merged_and_pruned() {
            let sha = match &task.worktree_allocation {
                WorktreeAllocation::MergedAndPruned { commit_sha } => commit_sha.clone(),
                _ => String::new(),
            };
            executed_reports.push(ExecutedTaskReport {
                task_id: task.task_id,
                worktree_path: String::new(),
                merged_commit_sha: sha,
            });
            continue;
        }

        let allocation =
            worktree_port.provision_worktree(&cycle.cycle_id, &task.task_id, &cycle.work_branch)?;
        let (worktree_path, _branch_name) = match &allocation {
            WorktreeAllocation::Provisioned {
                worktree_path,
                branch_name,
                base_branch,
            } => {
                task.allocate_worktree(worktree_path, branch_name, base_branch)?;
                (worktree_path.clone(), branch_name.clone())
            }
            _ => {
                return Err(DomainViolation::InfrastructureBoundaryFault {
                    subsystem: "worktree_allocation".to_string(),
                    details: format!("Expected provisioned worktree for task {}", task.task_id),
                });
            }
        };

        repo.update_task_worktree(&cycle.cycle_id, &task.task_id, &task.worktree_allocation)?;

        let prov_evidence = ExecutionEvidence {
            evidence_id: 0,
            cycle_id: cycle.cycle_id.clone(),
            gate_stage: GateStage::Gate4TasksImplemented,
            gauntlet_attempt: 0,
            actor_role: format!("ORCH:WORKTREE_{}", task.task_id),
            evidence_kind: EvidenceKind::CmdExec,
            executed_command: format!("git worktree add {} {}", worktree_path, task.task_id),
            exit_code: 0,
            duration_ms: 10,
            sanitized_stdout_tail: format!("Provisioned worktree at {}", worktree_path),
            sanitized_stderr_tail: "".to_string(),
            output_blake3_digest: "".to_string(),
            verdict: GateVerdict::Pass,
            recorded_at: req.now_iso.clone(),
        };
        repo.append_evidence(&prov_evidence)?;

        let merged_sha = worktree_port.merge_and_prune_worktree(
            &task.task_id,
            &worktree_path,
            &cycle.work_branch,
        )?;
        task.merge_and_prune_worktree(&merged_sha)?;
        repo.update_task_worktree(&cycle.cycle_id, &task.task_id, &task.worktree_allocation)?;

        let merge_evidence = ExecutionEvidence {
            evidence_id: 0,
            cycle_id: cycle.cycle_id.clone(),
            gate_stage: GateStage::Gate4TasksImplemented,
            gauntlet_attempt: 0,
            actor_role: format!("ORCH:MERGE_{}", task.task_id),
            evidence_kind: EvidenceKind::CmdExec,
            executed_command: format!("git merge --ff-only {}", merged_sha),
            exit_code: 0,
            duration_ms: 10,
            sanitized_stdout_tail: format!(
                "Merged and pruned worktree {} -> {}",
                worktree_path, merged_sha
            ),
            sanitized_stderr_tail: "".to_string(),
            output_blake3_digest: "".to_string(),
            verdict: GateVerdict::Pass,
            recorded_at: req.now_iso.clone(),
        };
        repo.append_evidence(&merge_evidence)?;

        executed_reports.push(ExecutedTaskReport {
            task_id: task.task_id,
            worktree_path,
            merged_commit_sha: merged_sha,
        });
    }

    Ok(RunExecutionWaveOutcome {
        cycle_id: cycle.cycle_id,
        wave_index: req.wave_index,
        tasks_executed: executed_reports,
    })
}
