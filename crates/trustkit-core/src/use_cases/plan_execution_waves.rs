// plan_execution_waves use case.
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle VI: Ubiquitous Language & Focused Modular Clean Code (< 250 LOC).

use crate::entities::{GateStage, TaskNode, WaveSchedule};
use crate::errors::DomainViolation;
use crate::ports::{CycleEvidenceRepositoryPort, WaveExecutionPlannerPort};
use crate::special_cases::{ActiveCycleLookup, SpecLockState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanExecutionWavesRequest {
    pub tasks: Vec<TaskNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanExecutionWavesOutcome {
    pub wave_schedule: WaveSchedule,
}

pub fn enforce_non_empty_backlog(
    cycle_id: &str,
    total_tasks: usize,
) -> Result<(), DomainViolation> {
    if total_tasks == 0 {
        return Err(DomainViolation::EmptyTaskBacklogFault {
            cycle_id: cycle_id.to_string(),
            reason: "Circuit Breaker tripped at Gate 3: atomic task backlog is empty (0 tasks). Cannot advance to Gate 4, Gate 5, or Gate 6.".to_string(),
        });
    }
    Ok(())
}

pub fn guard_gate_transition_with_backlog(
    cycle_id: &str,
    target_gate: GateStage,
    total_tasks: usize,
) -> Result<(), DomainViolation> {
    match target_gate {
        GateStage::Gate4TasksImplemented
        | GateStage::Gate5GauntletCleared
        | GateStage::Gate6Certified => enforce_non_empty_backlog(cycle_id, total_tasks),
        GateStage::Gate0Discovery
        | GateStage::Gate1SpecDrafting
        | GateStage::Gate2SpecSealed
        | GateStage::Gate3WavePlanned => Ok(()),
    }
}

pub fn verify_cycle_has_atomic_tasks(
    cycle_id: &str,
    repo: &dyn CycleEvidenceRepositoryPort,
) -> Result<usize, DomainViolation> {
    let tasks = repo.list_atomic_tasks(cycle_id)?;
    enforce_non_empty_backlog(cycle_id, tasks.len())?;
    Ok(tasks.len())
}

pub fn execute(
    req: PlanExecutionWavesRequest,
    repo: &dyn CycleEvidenceRepositoryPort,
    planner: &dyn WaveExecutionPlannerPort,
) -> Result<PlanExecutionWavesOutcome, DomainViolation> {
    let active_cycle = match repo.find_active_cycle()? {
        ActiveCycleLookup::Found(cycle) => *cycle,
        ActiveCycleLookup::NoActiveCycle => {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "plan_execution_waves".to_string(),
                details: "No active SDD cycle found in state.db. Run 'trustkit init' and 'trustkit spec lock' first.".to_string(),
            });
        }
    };

    if !matches!(active_cycle.spec_seal_state, SpecLockState::Frozen { .. }) {
        return Err(DomainViolation::InfrastructureBoundaryFault {
            subsystem: "plan_execution_waves".to_string(),
            details: format!(
                "Specification is not frozen. Current state is {:?}. Run 'trustkit spec lock' first.",
                active_cycle.spec_seal_state
            ),
        });
    }

    let mut schedule = planner.plan_waves(&req.tasks)?;
    enforce_non_empty_backlog(&active_cycle.cycle_id, schedule.total_tasks)?;
    schedule.cycle_id = active_cycle.cycle_id.clone();

    repo.save_atomic_tasks(&active_cycle.cycle_id, &req.tasks, &schedule)?;

    Ok(PlanExecutionWavesOutcome {
        wave_schedule: schedule,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{
        ArtifactSeal, CycleStatus, ExecutionEvidence, ExecutionWave, GateStage, MethodologyProfile,
        SddCycle,
    };
    use crate::special_cases::IssueLinkage;
    use std::sync::Mutex;

    struct FakeRepo {
        active_cycle: Mutex<Option<SddCycle>>,
        saved_tasks: Mutex<Vec<TaskNode>>,
    }

    impl FakeRepo {
        fn new(cycle: Option<SddCycle>) -> Self {
            Self {
                active_cycle: Mutex::new(cycle),
                saved_tasks: Mutex::new(vec![]),
            }
        }
    }

    impl CycleEvidenceRepositoryPort for FakeRepo {
        fn save_cycle(&self, c: &SddCycle) -> Result<(), DomainViolation> {
            *self.active_cycle.lock().unwrap() = Some(c.clone());
            Ok(())
        }
        fn find_active_cycle(&self) -> Result<ActiveCycleLookup, DomainViolation> {
            match &*self.active_cycle.lock().unwrap() {
                Some(c) => Ok(ActiveCycleLookup::Found(Box::new(c.clone()))),
                None => Ok(ActiveCycleLookup::NoActiveCycle),
            }
        }
        fn save_artifact_seal(&self, _: &ArtifactSeal) -> Result<u64, DomainViolation> {
            Ok(1)
        }
        fn list_artifact_seals(&self, _: &str) -> Result<Vec<ArtifactSeal>, DomainViolation> {
            Ok(vec![])
        }
        fn append_evidence(&self, _: &ExecutionEvidence) -> Result<u64, DomainViolation> {
            Ok(1)
        }
        fn list_evidences_for_cycle(
            &self,
            _: &str,
        ) -> Result<Vec<ExecutionEvidence>, DomainViolation> {
            Ok(vec![])
        }
        fn update_cycle_spec_lock(&self, _: &str, _: &str, _: &str) -> Result<(), DomainViolation> {
            Ok(())
        }
        fn save_atomic_tasks(
            &self,
            _: &str,
            tasks: &[TaskNode],
            _: &WaveSchedule,
        ) -> Result<(), DomainViolation> {
            self.saved_tasks.lock().unwrap().extend_from_slice(tasks);
            Ok(())
        }
    }

    struct FakePlanner {
        should_fail_cycle: bool,
    }

    impl WaveExecutionPlannerPort for FakePlanner {
        fn plan_waves(&self, tasks: &[TaskNode]) -> Result<WaveSchedule, DomainViolation> {
            if self.should_fail_cycle {
                return Err(DomainViolation::CycleDetected {
                    cycle_path: vec![
                        "TASK-001".to_string(),
                        "TASK-002".to_string(),
                        "TASK-001".to_string(),
                    ],
                    message: "Dependencia circular detectada".to_string(),
                });
            }

            Ok(WaveSchedule {
                cycle_id: "PLANNER_TEMP".to_string(),
                total_tasks: tasks.len(),
                waves: vec![ExecutionWave {
                    wave_index: 1,
                    tasks: tasks.to_vec(),
                }],
            })
        }
    }

    fn sample_cycle(state: SpecLockState) -> SddCycle {
        SddCycle {
            cycle_id: "CYC-20261028-001".to_string(),
            issue_linkage: IssueLinkage::UnlinkedLocalCycle {
                directory_slug: "test".to_string(),
            },
            methodology_profile: MethodologyProfile::StandardGauntletV2,
            current_gate: GateStage::Gate2SpecSealed,
            lifecycle_status: CycleStatus::Active,
            gauntlet_attempt: 0,
            spec_dir: "specs/001".to_string(),
            spec_file_path: "specs/001/spec.md".to_string(),
            spec_seal_state: state,
            work_branch: "feat/test".to_string(),
            base_commit_sha: "0000000000000000000000000000000000000000".to_string(),
            pre_existing_dirty_files: vec![],
            created_at: "2026-10-28T10:00:00Z".to_string(),
            updated_at: "2026-10-28T10:00:00Z".to_string(),
        }
    }

    fn sample_frozen_state() -> SpecLockState {
        SpecLockState::Frozen {
            spec_path: "specs/001/spec.md".to_string(),
            blake3_digest: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                .to_string(),
        }
    }

    #[test]
    fn test_plan_waves_requires_frozen_spec() {
        let repo = FakeRepo::new(Some(sample_cycle(SpecLockState::DraftUnverified)));
        let planner = FakePlanner {
            should_fail_cycle: false,
        };
        let req = PlanExecutionWavesRequest { tasks: vec![] };

        let result = execute(req, &repo, &planner);
        assert!(matches!(
            result,
            Err(DomainViolation::InfrastructureBoundaryFault { ref details, .. })
                if details.contains("Specification is not frozen")
        ));
    }

    #[test]
    fn test_plan_waves_success_with_frozen_spec() {
        let repo = FakeRepo::new(Some(sample_cycle(sample_frozen_state())));
        let planner = FakePlanner {
            should_fail_cycle: false,
        };
        let task = TaskNode {
            task_id: "TASK-001".to_string(),
            title: "Task 1".to_string(),
            architectural_layer: "CORE".to_string(),
            depends_on: vec![],
            expected_files: vec!["file.rs".to_string()],
            target_test_file: "test.rs".to_string(),
        };
        let req = PlanExecutionWavesRequest { tasks: vec![task] };

        let outcome = execute(req, &repo, &planner).expect("Should succeed");
        assert_eq!(outcome.wave_schedule.cycle_id, "CYC-20261028-001");
        assert_eq!(outcome.wave_schedule.total_tasks, 1);
        assert_eq!(repo.saved_tasks.lock().unwrap().len(), 1);
    }

    #[test]
    fn test_plan_waves_propagates_cycle_detected() {
        let repo = FakeRepo::new(Some(sample_cycle(sample_frozen_state())));
        let planner = FakePlanner {
            should_fail_cycle: true,
        };
        let req = PlanExecutionWavesRequest { tasks: vec![] };

        let result = execute(req, &repo, &planner);
        assert!(matches!(result, Err(DomainViolation::CycleDetected { .. })));
    }
}
