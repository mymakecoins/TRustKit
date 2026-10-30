use std::sync::Mutex;
use trustkit_core::entities::{
    AppSecCategory, ArtifactSeal, EvidenceKind, ExecutionEvidence, ReviewSeverity, SddCycle,
    SecurityReviewDecision, SecurityReviewVerdict, SecuritySkipReason, SemanticReviewFinding,
    TaskNode, WaveSchedule,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::{CycleEvidenceRepositoryPort, SecurityDiffReviewerPort};
use trustkit_core::special_cases::ActiveCycleLookup;
use trustkit_core::use_cases::run_security_review::{RunSecurityReviewRequest, execute};

struct MockSecurityRepo {
    evidences: Mutex<Vec<ExecutionEvidence>>,
}

impl CycleEvidenceRepositoryPort for MockSecurityRepo {
    fn save_cycle(&self, _cycle: &SddCycle) -> Result<(), DomainViolation> {
        Ok(())
    }
    fn find_active_cycle(&self) -> Result<ActiveCycleLookup, DomainViolation> {
        Ok(ActiveCycleLookup::NoActiveCycle)
    }
    fn save_artifact_seal(&self, _seal: &ArtifactSeal) -> Result<u64, DomainViolation> {
        Ok(1)
    }
    fn list_artifact_seals(&self, _cycle_id: &str) -> Result<Vec<ArtifactSeal>, DomainViolation> {
        Ok(vec![])
    }
    fn append_evidence(&self, evidence: &ExecutionEvidence) -> Result<u64, DomainViolation> {
        let mut evs = self.evidences.lock().unwrap();
        let id = (evs.len() as u64) + 1;
        let mut cloned = evidence.clone();
        cloned.evidence_id = id;
        evs.push(cloned);
        Ok(id)
    }
    fn list_evidences_for_cycle(
        &self,
        _cycle_id: &str,
    ) -> Result<Vec<ExecutionEvidence>, DomainViolation> {
        Ok(self.evidences.lock().unwrap().clone())
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
        _tasks: &[TaskNode],
        _wave_schedule: &WaveSchedule,
    ) -> Result<(), DomainViolation> {
        Ok(())
    }
}

struct MockSecurityDiffReviewer {
    invoked: Mutex<bool>,
    candidates: Vec<SemanticReviewFinding>,
}

impl SecurityDiffReviewerPort for MockSecurityDiffReviewer {
    fn review_branch_diff(
        &self,
        _target_branch: &str,
    ) -> Result<Vec<SemanticReviewFinding>, DomainViolation> {
        *self.invoked.lock().unwrap() = true;
        Ok(self.candidates.clone())
    }
}

#[test]
fn test_run_security_review_filters_out_candidates_below_threshold_8() {
    let repo = MockSecurityRepo {
        evidences: Mutex::new(vec![]),
    };

    let candidates = vec![
        SemanticReviewFinding {
            file: "src/auth.rs".to_string(),
            line: 10,
            severity: ReviewSeverity::Low,
            category_or_angle: AppSecCategory::AuthAndAuthorization.as_str().to_string(),
            confidence: 6,
            summary: "Theoretical timing side-channel".to_string(),
            scenario: "Noise below threshold".to_string(),
            recommendation: "Ignore".to_string(),
        },
        SemanticReviewFinding {
            file: "src/log.rs".to_string(),
            line: 22,
            severity: ReviewSeverity::Medium,
            category_or_angle: AppSecCategory::DataAndPiiExposure.as_str().to_string(),
            confidence: 7,
            summary: "Speculative log field".to_string(),
            scenario: "Noise below threshold 8".to_string(),
            recommendation: "Ignore".to_string(),
        },
        SemanticReviewFinding {
            file: "src/custom_query.rs".to_string(),
            line: 42,
            severity: ReviewSeverity::High,
            category_or_angle: AppSecCategory::InputValidationInjection
                .as_str()
                .to_string(),
            confidence: 9,
            summary: "SQL Injection on search filter".to_string(),
            scenario: "Search param concatenated without prepared statement".to_string(),
            recommendation: "Use parameterized query".to_string(),
        },
    ];

    let reviewer = MockSecurityDiffReviewer {
        invoked: Mutex::new(false),
        candidates,
    };

    let req = RunSecurityReviewRequest {
        cycle_id: "issue-7".to_string(),
        target_branch: "develop".to_string(),
        min_confidence_threshold: 8,
        decision: SecurityReviewDecision::Execute,
        now_iso: "2026-10-30T10:05:00Z".to_string(),
    };

    let outcome = execute(req, &repo, &reviewer).expect("should execute review");
    assert!(*reviewer.invoked.lock().unwrap());
    assert_eq!(outcome.discarded_low_confidence_count, 2);
    assert_eq!(outcome.surviving_findings.len(), 1);
    assert_eq!(outcome.surviving_findings[0].confidence, 9);
    assert!(matches!(
        outcome.verdict,
        SecurityReviewVerdict::VulnerabilitiesDetected { .. }
    ));

    let evs = repo.list_evidences_for_cycle("issue-7").unwrap();
    assert_eq!(evs.len(), 1);
    assert_eq!(evs[0].evidence_kind, EvidenceKind::SecurityReview);
    assert_eq!(evs[0].actor_role, "CRITIC:SECURITY_REVIEW");
}

#[test]
fn test_run_security_review_skip_does_not_invoke_reviewer_and_records_skipped() {
    let repo = MockSecurityRepo {
        evidences: Mutex::new(vec![]),
    };
    let reviewer = MockSecurityDiffReviewer {
        invoked: Mutex::new(false),
        candidates: vec![],
    };

    let req = RunSecurityReviewRequest {
        cycle_id: "issue-7".to_string(),
        target_branch: "develop".to_string(),
        min_confidence_threshold: 8,
        decision: SecurityReviewDecision::Skip {
            reason: SecuritySkipReason::DeveloperPromptOptOut,
        },
        now_iso: "2026-10-30T10:05:00Z".to_string(),
    };

    let outcome = execute(req, &repo, &reviewer).expect("should record skip");
    assert!(!*reviewer.invoked.lock().unwrap());
    assert_eq!(
        outcome.verdict,
        SecurityReviewVerdict::SkippedByDeveloper {
            reason: SecuritySkipReason::DeveloperPromptOptOut,
        }
    );

    let evs = repo.list_evidences_for_cycle("issue-7").unwrap();
    assert_eq!(evs.len(), 1);
    assert_eq!(evs[0].evidence_kind, EvidenceKind::SecurityReview);
    assert!(evs[0]
        .sanitized_stdout_tail
        .contains("SkippedByDeveloper"));
}
