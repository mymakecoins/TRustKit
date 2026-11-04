// lock_specification use case.
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle VI: Ubiquitous Language & Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Cryptographic Immutability ("The Event Log Proves").

use crate::entities::{
    ArtifactCategory, ArtifactSeal, CycleStatus, GateStage, MethodologyProfile, SddCycle,
};
use crate::errors::DomainViolation;
use crate::ports::{ArtifactHasherPort, CycleEvidenceRepositoryPort, SpecValidatorPort};
use crate::special_cases::{ActiveCycleLookup, ContractUnlockState, IssueLinkage, SpecLockState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockSpecificationRequest {
    pub spec_path: Option<String>,
    pub file_content: String,
    pub now_iso: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockSpecificationOutcome {
    pub cycle_id: String,
    pub spec_file_path: String,
    pub spec_lock_state: String,
    pub blake3_digest: String,
    pub locked_at: String,
    pub rules_count: usize,
    pub requirements_count: usize,
    pub tasks_count: usize,
    pub e2e_tests_count: usize,
}

pub fn execute(
    req: LockSpecificationRequest,
    repo: &dyn CycleEvidenceRepositoryPort,
    hasher: &dyn ArtifactHasherPort,
    validator: &dyn SpecValidatorPort,
) -> Result<LockSpecificationOutcome, DomainViolation> {
    let active_cycle_opt = match repo.find_active_cycle()? {
        ActiveCycleLookup::Found(cycle) => Some(*cycle),
        ActiveCycleLookup::NoActiveCycle => None,
    };

    let (cycle_id, spec_file_path) = match (req.spec_path, active_cycle_opt) {
        (Some(path), Some(cycle)) => (cycle.cycle_id, path),
        (Some(path), None) => {
            let standalone_id = "CYC-STANDALONE".to_string();
            let parent_dir = std::path::Path::new(&path)
                .parent()
                .and_then(|p| p.to_str())
                .unwrap_or("specs");

            let standalone_cycle = SddCycle {
                cycle_id: standalone_id.clone(),
                issue_linkage: IssueLinkage::UnlinkedLocalCycle {
                    directory_slug: "standalone".to_string(),
                },
                methodology_profile: MethodologyProfile::StandardGauntletV2,
                current_gate: GateStage::Gate1SpecDrafting,
                lifecycle_status: CycleStatus::Active,
                gauntlet_attempt: 0,
                spec_dir: parent_dir.to_string(),
                spec_file_path: path.clone(),
                spec_seal_state: SpecLockState::DraftUnverified,
                work_branch: "main".to_string(),
                base_commit_sha: "0000000000000000000000000000000000000000".to_string(),
                pre_existing_dirty_files: vec![],
                created_at: req.now_iso.clone(),
                updated_at: req.now_iso.clone(),
            };

            repo.save_cycle(&standalone_cycle)?;
            (standalone_id, path)
        }
        (None, Some(cycle)) => (cycle.cycle_id, cycle.spec_file_path),
        (None, None) => {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "lock_specification".to_string(),
                details: "No active SDD cycle found and no --spec flag provided.".to_string(),
            });
        }
    };

    let report = validator.validate_specification(&req.file_content)?;

    let blake3_digest = hasher.compute_bytes_blake3(req.file_content.as_bytes());

    let seal = ArtifactSeal {
        seal_id: 0,
        cycle_id: cycle_id.clone(),
        file_path: spec_file_path.clone(),
        artifact_category: ArtifactCategory::Spec,
        blake3_digest: blake3_digest.clone(),
        ast_assertion_count: report.total_rules as u32,
        contract_unlock_state: ContractUnlockState::Sealed,
    };

    repo.save_artifact_seal(&seal)?;
    repo.update_cycle_spec_lock(&cycle_id, &blake3_digest, "FROZEN")?;

    Ok(LockSpecificationOutcome {
        cycle_id,
        spec_file_path,
        spec_lock_state: "FROZEN".to_string(),
        blake3_digest,
        locked_at: req.now_iso,
        rules_count: report.total_rules,
        requirements_count: report.total_requirements,
        tasks_count: report.total_rules,
        e2e_tests_count: report.total_rules,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{ExecutionEvidence, TaskNode, WaveSchedule};
    use crate::ports::SpecValidationReport;
    use std::sync::Mutex;

    struct FakeRepo {
        active_cycle: Option<SddCycle>,
        saved_cycles: Mutex<Vec<SddCycle>>,
        seals: Mutex<Vec<ArtifactSeal>>,
        locked: Mutex<Option<(String, String, String)>>,
    }

    impl CycleEvidenceRepositoryPort for FakeRepo {
        fn save_cycle(&self, cycle: &SddCycle) -> Result<(), DomainViolation> {
            let mut list = self.saved_cycles.lock().unwrap();
            list.push(cycle.clone());
            Ok(())
        }
        fn find_active_cycle(&self) -> Result<ActiveCycleLookup, DomainViolation> {
            Ok(match &self.active_cycle {
                Some(c) => ActiveCycleLookup::Found(Box::new(c.clone())),
                None => ActiveCycleLookup::NoActiveCycle,
            })
        }
        fn save_artifact_seal(&self, seal: &ArtifactSeal) -> Result<u64, DomainViolation> {
            let mut list = self.seals.lock().unwrap();
            list.push(seal.clone());
            Ok(list.len() as u64)
        }
        fn list_artifact_seals(&self, _: &str) -> Result<Vec<ArtifactSeal>, DomainViolation> {
            Ok(self.seals.lock().unwrap().clone())
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
        fn update_cycle_spec_lock(
            &self,
            cycle_id: &str,
            blake3_digest: &str,
            locked_state: &str,
        ) -> Result<(), DomainViolation> {
            let mut l = self.locked.lock().unwrap();
            *l = Some((
                cycle_id.to_string(),
                blake3_digest.to_string(),
                locked_state.to_string(),
            ));
            Ok(())
        }
        fn save_atomic_tasks(
            &self,
            _: &str,
            _: &[TaskNode],
            _: &WaveSchedule,
        ) -> Result<(), DomainViolation> {
            Ok(())
        }
    }

    struct FakeHasher;
    impl ArtifactHasherPort for FakeHasher {
        fn compute_file_blake3(&self, _: &str) -> Result<String, DomainViolation> {
            Ok("fake_hash_64_chars_long_0123456789abcdef0123456789abcdef0123456789a".to_string())
        }
        fn compute_bytes_blake3(&self, _: &[u8]) -> String {
            "fake_hash_64_chars_long_0123456789abcdef0123456789abcdef0123456789a".to_string()
        }
    }

    struct FakeValidator {
        should_fail_gap: bool,
    }
    impl SpecValidatorPort for FakeValidator {
        fn validate_specification(&self, _: &str) -> Result<SpecValidationReport, DomainViolation> {
            if self.should_fail_gap {
                return Err(DomainViolation::TraceabilityGap {
                    orphan_rule: "RN-002".to_string(),
                    message: "Gap in RN-002".to_string(),
                });
            }
            Ok(SpecValidationReport {
                mermaid_valid: true,
                gherkin_valid: true,
                orphan_rules: vec![],
                total_rules: 2,
                total_requirements: 2,
            })
        }
    }

    #[test]
    fn test_lock_spec_success_standalone() {
        let repo = FakeRepo {
            active_cycle: None,
            saved_cycles: Mutex::new(vec![]),
            seals: Mutex::new(vec![]),
            locked: Mutex::new(None),
        };
        let hasher = FakeHasher;
        let validator = FakeValidator {
            should_fail_gap: false,
        };

        let req = LockSpecificationRequest {
            spec_path: Some("specs/feat/spec.md".to_string()),
            file_content: "valid spec content".to_string(),
            now_iso: "2026-10-28T10:00:00Z".to_string(),
        };

        let outcome = execute(req, &repo, &hasher, &validator).expect("lock should succeed");
        assert_eq!(outcome.cycle_id, "CYC-STANDALONE");
        assert_eq!(outcome.spec_lock_state, "FROZEN");
        assert_eq!(outcome.rules_count, 2);
        assert_eq!(repo.saved_cycles.lock().unwrap().len(), 1);
        assert_eq!(repo.seals.lock().unwrap().len(), 1);
        assert_eq!(repo.locked.lock().unwrap().as_ref().unwrap().2, "FROZEN");
    }

    #[test]
    fn test_lock_spec_propagates_gap_error() {
        let repo = FakeRepo {
            active_cycle: None,
            saved_cycles: Mutex::new(vec![]),
            seals: Mutex::new(vec![]),
            locked: Mutex::new(None),
        };
        let hasher = FakeHasher;
        let validator = FakeValidator {
            should_fail_gap: true,
        };

        let req = LockSpecificationRequest {
            spec_path: Some("specs/feat/spec.md".to_string()),
            file_content: "gap spec content".to_string(),
            now_iso: "2026-10-28T10:00:00Z".to_string(),
        };

        let res = execute(req, &repo, &hasher, &validator);
        match res {
            Err(DomainViolation::TraceabilityGap { orphan_rule, .. }) => {
                assert_eq!(orphan_rule, "RN-002");
            }
            other => panic!("Expected TraceabilityGap, got: {other:?}"),
        }
    }
}
