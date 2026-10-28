// Unit tests for AuditAstLayer0UseCase (TASK-011).
// Constitutional Principle III: Test-First & Anti-Cheat.

use trustkit_core::entities::{
    ArtifactSeal, AstAuditVerdict, AstViolation, ExecutionEvidence,
    SddCycle, TaskNode, WaveSchedule,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::{AstAuditorPort, CycleEvidenceRepositoryPort};
use trustkit_core::special_cases::ActiveCycleLookup;
use trustkit_core::use_cases::audit_ast_layer0::{self, AuditAstLayer0Request};

struct MockRepo {
    evidences: std::sync::Mutex<Vec<ExecutionEvidence>>,
}

impl CycleEvidenceRepositoryPort for MockRepo {
    fn save_cycle(&self, _cycle: &SddCycle) -> Result<(), DomainViolation> { Ok(()) }
    fn find_active_cycle(&self) -> Result<ActiveCycleLookup, DomainViolation> {
        Ok(ActiveCycleLookup::NoActiveCycle)
    }
    fn save_artifact_seal(&self, _seal: &ArtifactSeal) -> Result<u64, DomainViolation> { Ok(1) }
    fn list_artifact_seals(&self, _cycle_id: &str) -> Result<Vec<ArtifactSeal>, DomainViolation> { Ok(vec![]) }
    fn append_evidence(&self, evidence: &ExecutionEvidence) -> Result<u64, DomainViolation> {
        self.evidences.lock().unwrap().push(evidence.clone());
        Ok(1)
    }
    fn list_evidences_for_cycle(&self, _cycle_id: &str) -> Result<Vec<ExecutionEvidence>, DomainViolation> {
        Ok(self.evidences.lock().unwrap().clone())
    }
    fn update_cycle_spec_lock(&self, _cycle_id: &str, _digest: &str, _state: &str) -> Result<(), DomainViolation> { Ok(()) }
    fn save_atomic_tasks(&self, _c: &str, _t: &[TaskNode], _w: &WaveSchedule) -> Result<(), DomainViolation> { Ok(()) }
}

struct MockAuditor {
    clean: bool,
}

impl AstAuditorPort for MockAuditor {
    fn audit_source_tree(
        &self,
        files: &[String],
        _reference_seals: &[ArtifactSeal],
    ) -> Result<AstAuditVerdict, DomainViolation> {
        if self.clean {
            Ok(AstAuditVerdict::CleanPass { violations: vec![] })
        } else {
            Ok(AstAuditVerdict::Rejected {
                violations: vec![AstViolation::ForbiddenEmoji {
                    file_path: files.first().cloned().unwrap_or_default(),
                    line: 1,
                    character: "⛔".to_string(),
                }],
            })
        }
    }
}

#[test]
fn test_audit_ast_layer0_use_case_clean_pass() {
    let repo = MockRepo { evidences: std::sync::Mutex::new(vec![]) };
    let auditor = MockAuditor { clean: true };

    let req = AuditAstLayer0Request {
        cycle_id: "cycle-1".to_string(),
        files: vec!["src/lib.rs".to_string()],
        now_iso: "2026-10-28T12:00:00Z".to_string(),
    };

    let outcome = audit_ast_layer0::execute(req, &repo, &auditor)
        .expect("use case execution should succeed");

    assert!(outcome.verdict.is_pass());
    assert_eq!(outcome.total_violations, 0);
    assert_eq!(repo.evidences.lock().unwrap().len(), 1);
}

#[test]
fn test_audit_ast_layer0_use_case_rejected() {
    let repo = MockRepo { evidences: std::sync::Mutex::new(vec![]) };
    let auditor = MockAuditor { clean: false };

    let req = AuditAstLayer0Request {
        cycle_id: "cycle-1".to_string(),
        files: vec!["src/lib.rs".to_string()],
        now_iso: "2026-10-28T12:00:00Z".to_string(),
    };

    let outcome = audit_ast_layer0::execute(req, &repo, &auditor)
        .expect("use case execution should return rejected verdict");

    assert!(!outcome.verdict.is_pass());
    assert_eq!(outcome.total_violations, 1);
    assert_eq!(repo.evidences.lock().unwrap().len(), 1);
}
