use std::sync::Mutex;
use trustkit_core::entities::{
    ArtifactSeal, AtomicTask, CodeReviewEffortLevel, EvidenceKind, ExecutionEvidence, ReviewAngle,
    ReviewSeverity, SddCycle, SemanticReviewFinding, TaskNode, WaveSchedule,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::{CycleEvidenceRepositoryPort, SemanticCodeReviewerPort};
use trustkit_core::special_cases::{ActiveCycleLookup, TddLifecycleState, WorktreeAllocation};
use trustkit_core::use_cases::review_task_code::{ReviewTaskCodeRequest, execute};

struct MockCodeReviewRepo {
    task: Option<AtomicTask>,
    evidences: Mutex<Vec<ExecutionEvidence>>,
}

impl CycleEvidenceRepositoryPort for MockCodeReviewRepo {
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
    fn find_atomic_task(
        &self,
        _cycle_id: &str,
        _task_id: &str,
    ) -> Result<Option<AtomicTask>, DomainViolation> {
        Ok(self.task.clone())
    }
}

struct MockSemanticReviewer {
    findings: Vec<SemanticReviewFinding>,
}

impl SemanticCodeReviewerPort for MockSemanticReviewer {
    fn review_task_diff(
        &self,
        _cycle_id: &str,
        _task_id: &str,
        _effort_level: CodeReviewEffortLevel,
    ) -> Result<Vec<SemanticReviewFinding>, DomainViolation> {
        Ok(self.findings.clone())
    }
}

#[test]
fn test_review_task_code_succeeds_on_green_task_and_applies_medium_cap() {
    let green_task = AtomicTask {
        cycle_id: "issue-7".to_string(),
        task_id: "TASK-003".to_string(),
        title: "Refactor review".to_string(),
        architectural_layer: "CORE".to_string(),
        wave_index: 2,
        depends_on: vec![],
        expected_files: vec!["src/lib.rs".to_string()],
        target_test_file: "tests/test.rs".to_string(),
        tdd_lifecycle_state: TddLifecycleState::GreenVerified {
            red_evidence_id: 1,
            green_evidence_id: 2,
        },
        red_evidence_id: 1,
        green_evidence_id: 2,
        worktree_allocation: WorktreeAllocation::NotProvisioned,
        commit_sha: "".to_string(),
    };

    let repo = MockCodeReviewRepo {
        task: Some(green_task),
        evidences: Mutex::new(vec![]),
    };

    let mut findings = Vec::new();
    for i in 1..=10 {
        findings.push(SemanticReviewFinding {
            file: "src/lib.rs".to_string(),
            line: i,
            severity: ReviewSeverity::Medium,
            category_or_angle: ReviewAngle::Simplification.as_str().to_string(),
            confidence: 8,
            summary: format!("Simplification candidate #{i}"),
            scenario: "Redundant branch".to_string(),
            recommendation: "Flatten conditional".to_string(),
        });
    }

    let reviewer = MockSemanticReviewer { findings };
    let req = ReviewTaskCodeRequest {
        cycle_id: "issue-7".to_string(),
        task_id: "TASK-003".to_string(),
        effort_level: CodeReviewEffortLevel::Medium,
        max_findings_override: 0,
        now_iso: "2026-10-30T10:00:00Z".to_string(),
    };

    let outcome = execute(req, &repo, &reviewer).expect("should succeed for green task");
    assert!(outcome.clean_pass);
    assert_eq!(outcome.findings.len(), 8);
    assert_eq!(outcome.recorded_evidence_id, 1);

    let recorded = repo.list_evidences_for_cycle("issue-7").unwrap();
    assert_eq!(recorded.len(), 1);
    assert_eq!(recorded[0].evidence_kind, EvidenceKind::CodeReview);
    assert_eq!(recorded[0].actor_role, "AUDITOR:CODE_REVIEW");
}

#[test]
fn test_review_task_code_fails_when_task_has_not_completed_green_phase() {
    let red_only_task = AtomicTask {
        cycle_id: "issue-7".to_string(),
        task_id: "TASK-003".to_string(),
        title: "Refactor review".to_string(),
        architectural_layer: "CORE".to_string(),
        wave_index: 2,
        depends_on: vec![],
        expected_files: vec!["src/lib.rs".to_string()],
        target_test_file: "tests/test.rs".to_string(),
        tdd_lifecycle_state: TddLifecycleState::RedVerified {
            failing_evidence_id: 1,
        },
        red_evidence_id: 1,
        green_evidence_id: 0,
        worktree_allocation: WorktreeAllocation::NotProvisioned,
        commit_sha: "".to_string(),
    };

    let repo = MockCodeReviewRepo {
        task: Some(red_only_task),
        evidences: Mutex::new(vec![]),
    };
    let reviewer = MockSemanticReviewer { findings: vec![] };

    let req = ReviewTaskCodeRequest {
        cycle_id: "issue-7".to_string(),
        task_id: "TASK-003".to_string(),
        effort_level: CodeReviewEffortLevel::Medium,
        max_findings_override: 0,
        now_iso: "2026-10-30T10:00:00Z".to_string(),
    };

    let res = execute(req, &repo, &reviewer);
    assert!(res.is_err());
    match res.unwrap_err() {
        DomainViolation::TddLifecycleViolation { task_id, .. } => {
            assert_eq!(task_id, "TASK-003");
        }
        other => panic!("Expected TddLifecycleViolation, got {other:?}"),
    }
}
