// query_cycle_status use case.
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL & Special Cases.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::errors::DomainViolation;
use crate::ports::CycleEvidenceRepositoryPort;
use crate::special_cases::ActiveCycleLookup;

pub fn execute(
    repo: &dyn CycleEvidenceRepositoryPort,
) -> Result<ActiveCycleLookup, DomainViolation> {
    repo.find_active_cycle()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{
        ArtifactSeal, CycleStatus, ExecutionEvidence, GateStage, MethodologyProfile, SddCycle,
    };
    use crate::special_cases::{ActiveCycleLookup, IssueLinkage, SpecLockState};
    use std::sync::Mutex;

    struct StubCycleRepository {
        active_cycle: Mutex<ActiveCycleLookup>,
    }

    impl StubCycleRepository {
        fn with_active(cycle: SddCycle) -> Self {
            Self {
                active_cycle: Mutex::new(ActiveCycleLookup::Found(Box::new(cycle))),
            }
        }

        fn without_active() -> Self {
            Self {
                active_cycle: Mutex::new(ActiveCycleLookup::NoActiveCycle),
            }
        }
    }

    impl CycleEvidenceRepositoryPort for StubCycleRepository {
        fn save_cycle(&self, _cycle: &SddCycle) -> Result<(), DomainViolation> {
            Ok(())
        }

        fn find_active_cycle(&self) -> Result<ActiveCycleLookup, DomainViolation> {
            Ok(self.active_cycle.lock().unwrap().clone())
        }

        fn save_artifact_seal(&self, _seal: &ArtifactSeal) -> Result<u64, DomainViolation> {
            Ok(1)
        }

        fn list_artifact_seals(
            &self,
            _cycle_id: &str,
        ) -> Result<Vec<ArtifactSeal>, DomainViolation> {
            Ok(vec![])
        }

        fn append_evidence(&self, _evidence: &ExecutionEvidence) -> Result<u64, DomainViolation> {
            Ok(1)
        }

        fn list_evidences_for_cycle(
            &self,
            _cycle_id: &str,
        ) -> Result<Vec<ExecutionEvidence>, DomainViolation> {
            Ok(vec![])
        }

        fn update_cycle_spec_lock(
            &self,
            _cycle_id: &str,
            _blake3_digest: &str,
            _locked_state: &str,
        ) -> Result<(), DomainViolation> {
            Ok(())
        }

        fn save_atomic_tasks(
            &self,
            _cycle_id: &str,
            _tasks: &[crate::entities::TaskNode],
            _wave_schedule: &crate::entities::WaveSchedule,
        ) -> Result<(), DomainViolation> {
            Ok(())
        }
    }

    #[test]
    fn test_query_cycle_status_found_active_cycle() {
        let cycle = SddCycle {
            cycle_id: "issue-45-202610271415".to_string(),
            issue_linkage: IssueLinkage::UnlinkedLocalCycle {
                directory_slug: "no-issue-202610271415".to_string(),
            },
            methodology_profile: MethodologyProfile::StandardGauntletV2,
            current_gate: GateStage::Gate0Discovery,
            lifecycle_status: CycleStatus::Active,
            gauntlet_attempt: 0,
            spec_dir: "docs/specs/issue-45-202610271415".to_string(),
            spec_file_path: "docs/specs/issue-45-202610271415/spec.md".to_string(),
            spec_seal_state: SpecLockState::DraftUnverified,
            work_branch: "feat/45-core".to_string(),
            base_commit_sha: "0123456789abcdef0123456789abcdef01234567".to_string(),
            pre_existing_dirty_files: vec!["dirty.txt".to_string()],
            created_at: "2026-10-27T14:15:00Z".to_string(),
            updated_at: "2026-10-27T14:15:00Z".to_string(),
        };

        let repo = StubCycleRepository::with_active(cycle.clone());
        let outcome = execute(&repo).expect("Should return ActiveCycleLookup");

        match outcome {
            ActiveCycleLookup::Found(active) => {
                assert_eq!(active.cycle_id, "issue-45-202610271415");
                assert_eq!(active.current_gate, GateStage::Gate0Discovery);
                assert_eq!(active.lifecycle_status, CycleStatus::Active);
                assert_eq!(active.work_branch, "feat/45-core");
                assert_eq!(
                    active.pre_existing_dirty_files,
                    vec!["dirty.txt".to_string()]
                );
            }
            ActiveCycleLookup::NoActiveCycle => panic!("Expected Found variant"),
        }
    }

    #[test]
    fn test_query_cycle_status_no_active_cycle() {
        let repo = StubCycleRepository::without_active();
        let outcome = execute(&repo).expect("Should return ActiveCycleLookup");

        assert_eq!(outcome, ActiveCycleLookup::NoActiveCycle);
    }
}
