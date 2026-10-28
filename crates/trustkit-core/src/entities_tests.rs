// Unit tests for domain entities.
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use super::*;

#[test]
fn test_gate_stage_conversions() {
    assert_eq!(GateStage::from_u8(0), Some(GateStage::Gate0Discovery));
    assert_eq!(GateStage::from_u8(6), Some(GateStage::Gate6Certified));
    assert_eq!(GateStage::from_u8(7), None);
    assert_eq!(GateStage::Gate0Discovery.label(), "GATE_0_DISCOVERY");
}

#[test]
fn test_sdd_cycle_construction() {
    let cycle = SddCycle {
        cycle_id: "issue-45-202610271400".to_string(),
        issue_linkage: IssueLinkage::UnlinkedLocalCycle {
            directory_slug: "issue-45-202610271400".to_string(),
        },
        methodology_profile: MethodologyProfile::StandardGauntletV2,
        current_gate: GateStage::Gate0Discovery,
        lifecycle_status: CycleStatus::Active,
        gauntlet_attempt: 0,
        spec_dir: "specs/001".to_string(),
        spec_file_path: "specs/001/spec.md".to_string(),
        spec_seal_state: SpecLockState::DraftUnverified,
        work_branch: "feat/45-core".to_string(),
        base_commit_sha: "0123456789012345678901234567890123456789".to_string(),
        pre_existing_dirty_files: vec![],
        created_at: "2026-10-27T14:00:00Z".to_string(),
        updated_at: "2026-10-27T14:00:00Z".to_string(),
    };
    assert_eq!(cycle.gauntlet_attempt, 0);
}

#[test]
fn test_new_phase2_entities_construction() {
    let doc = UpstreamDocument {
        path: "specs/prd.md".to_string(),
        title: "Test PRD".to_string(),
        topic_slug: "test-prd".to_string(),
        business_rules: vec![BusinessRule {
            rule_id: "RN-001".to_string(),
            description: "Rule 1".to_string(),
        }],
        functional_requirements: vec![FunctionalRequirement {
            req_id: "RF-001".to_string(),
            description: "Req 1".to_string(),
        }],
    };
    assert_eq!(doc.business_rules.len(), 1);
    assert_eq!(doc.functional_requirements.len(), 1);

    let row = TraceabilityRow {
        business_rule_id: "RN-001".to_string(),
        functional_req_id: "RF-001".to_string(),
        task_id: "TASK-001".to_string(),
        mermaid_node: "A".to_string(),
        playwright_spec_path: "tests/e2e.spec.ts".to_string(),
        evidence_screenshot: "SC-001.png".to_string(),
    };
    assert_eq!(row.task_id, "TASK-001");

    let spec = SpecificationV2 {
        cycle_id: "cycle-1".to_string(),
        feature_slug: "test-prd".to_string(),
        spec_file_path: "specs/test-prd-spec.md".to_string(),
        business_rules: doc.business_rules.clone(),
        functional_requirements: doc.functional_requirements.clone(),
        mermaid_content: "flowchart TD; A-->B;".to_string(),
        gherkin_scenarios: vec![GherkinScenario {
            name: "Scenario 1".to_string(),
            given: vec!["context".to_string()],
            when: vec!["action".to_string()],
            then: vec!["outcome".to_string()],
        }],
        traceability_matrix: vec![row],
        lock_state: SpecLockState::DraftUnverified,
    };
    assert_eq!(spec.feature_slug, "test-prd");

    let task = TaskNode {
        task_id: "TASK-001".to_string(),
        title: "Task 1".to_string(),
        architectural_layer: "CORE".to_string(),
        depends_on: vec![],
        expected_files: vec!["crates/trustkit-core/src/entities.rs".to_string()],
        target_test_file: "tests/test.rs".to_string(),
    };
    let wave = ExecutionWave {
        wave_index: 1,
        tasks: vec![task],
    };
    let schedule = WaveSchedule {
        cycle_id: "cycle-1".to_string(),
        total_tasks: 1,
        waves: vec![wave],
    };
    assert_eq!(schedule.total_tasks, 1);
    assert_eq!(schedule.waves.len(), 1);
}
