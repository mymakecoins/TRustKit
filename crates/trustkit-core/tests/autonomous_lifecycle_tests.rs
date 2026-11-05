// Unit tests for autonomous SDD lifecycle orchestration (TASK-002).
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use std::sync::Mutex;
use trustkit_core::entities::{
    ArtifactSeal, ExecutionEvidence, ExecutionWave, GateVerdict, SddCycle, TaskNode, WaveSchedule,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::{
    ArtifactHasherPort, CycleEvidenceRepositoryPort, SpecValidationReport, SpecValidatorPort,
    WaveExecutionPlannerPort,
};
use trustkit_core::special_cases::ActiveCycleLookup;
use trustkit_core::use_cases::run_autonomous_cycle::{self, AutonomousCycleRequest};

struct MockRepo {
    locks: Mutex<Vec<(String, String, String)>>,
    tasks_saved: Mutex<Vec<String>>,
}

impl MockRepo {
    fn new() -> Self {
        Self {
            locks: Mutex::new(Vec::new()),
            tasks_saved: Mutex::new(Vec::new()),
        }
    }
}

impl CycleEvidenceRepositoryPort for MockRepo {
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
        cycle_id: &str,
        blake3_digest: &str,
        locked_state: &str,
    ) -> Result<(), DomainViolation> {
        self.locks.lock().unwrap().push((
            cycle_id.to_string(),
            blake3_digest.to_string(),
            locked_state.to_string(),
        ));
        Ok(())
    }
    fn save_atomic_tasks(
        &self,
        cycle_id: &str,
        _tasks: &[TaskNode],
        _wave_schedule: &WaveSchedule,
    ) -> Result<(), DomainViolation> {
        self.tasks_saved.lock().unwrap().push(cycle_id.to_string());
        Ok(())
    }
}

struct MockHasher;
impl ArtifactHasherPort for MockHasher {
    fn compute_file_blake3(&self, _path: &str) -> Result<String, DomainViolation> {
        Ok("mock_blake3_file_digest".to_string())
    }
    fn compute_bytes_blake3(&self, _bytes: &[u8]) -> String {
        "mock_blake3_bytes_digest".to_string()
    }
}

struct MockValidator {
    should_fail: bool,
}
impl SpecValidatorPort for MockValidator {
    fn validate_specification(
        &self,
        _spec_content: &str,
    ) -> Result<SpecValidationReport, DomainViolation> {
        if self.should_fail {
            Err(DomainViolation::TraceabilityGap {
                orphan_rule: "RN-001".to_string(),
                message: "Syntactic or structural spec validation failure".to_string(),
            })
        } else {
            Ok(SpecValidationReport {
                mermaid_valid: true,
                gherkin_valid: true,
                orphan_rules: vec![],
                total_rules: 5,
                total_requirements: 5,
            })
        }
    }
}

struct MockPlanner;
impl WaveExecutionPlannerPort for MockPlanner {
    fn plan_waves(&self, tasks: &[TaskNode]) -> Result<WaveSchedule, DomainViolation> {
        Ok(WaveSchedule {
            cycle_id: "".to_string(),
            total_tasks: tasks.len(),
            waves: vec![ExecutionWave {
                wave_index: 1,
                tasks: tasks.to_vec(),
            }],
        })
    }
}

#[test]
fn test_task_002_nominal_autonomous_cycle_clears_gates_0_to_3() {
    let repo = MockRepo::new();
    let hasher = MockHasher;
    let validator = MockValidator { should_fail: false };
    let planner = MockPlanner;

    let req = AutonomousCycleRequest {
        issue: Some("10".to_string()),
        spec_path: None,
        target_branch: "develop".to_string(),
        runtime_mode: "auto".to_string(),
        agent_command: "".to_string(),
        now_iso: "2026-11-05T12:00:00Z".to_string(),
        timestamp_slug: "202611051200".to_string(),
        spec_content: "# Spec\n## 1. Product Trio\n## 2. Mermaid\n## 3. Gherkin\n".to_string(),
        tasks: vec![TaskNode {
            task_id: "TASK-001".to_string(),
            title: "Task 1".to_string(),
            architectural_layer: "CLI".to_string(),
            depends_on: vec![],
            expected_files: vec![],
            target_test_file: "".to_string(),
        }],
    };

    let outcome = run_autonomous_cycle::execute(req, &repo, &hasher, &validator, &planner)
        .expect("Nominal cycle should succeed");

    assert_eq!(outcome.cycle_id, "issue-10-202611051200");
    assert_eq!(outcome.gates_cleared, vec![0, 1, 2, 3]);
    assert_eq!(outcome.spec_digest, "mock_blake3_bytes_digest");
    assert_eq!(outcome.total_waves, 1);
    assert_eq!(outcome.total_tasks, 1);
    assert_eq!(outcome.overall_verdict, GateVerdict::Pass);
    assert_eq!(repo.locks.lock().unwrap().len(), 1);
    assert_eq!(repo.tasks_saved.lock().unwrap().len(), 1);
}

#[test]
fn test_task_002_spec_validation_failure_aborts_at_gate_1() {
    let repo = MockRepo::new();
    let hasher = MockHasher;
    let validator = MockValidator { should_fail: true };
    let planner = MockPlanner;

    let req = AutonomousCycleRequest {
        issue: None,
        spec_path: Some("docs/specs/test/spec.md".to_string()),
        target_branch: "develop".to_string(),
        runtime_mode: "auto".to_string(),
        agent_command: "".to_string(),
        now_iso: "2026-11-05T12:00:00Z".to_string(),
        timestamp_slug: "202611051200".to_string(),
        spec_content: "# Invalid Spec".to_string(),
        tasks: vec![],
    };

    let result = run_autonomous_cycle::execute(req, &repo, &hasher, &validator, &planner);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        DomainViolation::TraceabilityGap { .. }
    ));
}

#[test]
fn test_task_002_missing_issue_and_spec_fails_validation() {
    let repo = MockRepo::new();
    let hasher = MockHasher;
    let validator = MockValidator { should_fail: false };
    let planner = MockPlanner;

    let req = AutonomousCycleRequest {
        issue: None,
        spec_path: None,
        target_branch: "develop".to_string(),
        runtime_mode: "auto".to_string(),
        agent_command: "".to_string(),
        now_iso: "2026-11-05T12:00:00Z".to_string(),
        timestamp_slug: "202611051200".to_string(),
        spec_content: "some content".to_string(),
        tasks: vec![],
    };

    let result = run_autonomous_cycle::execute(req, &repo, &hasher, &validator, &planner);
    assert!(result.is_err());
}
