// CLI Worktree Notification JSON Serialization Integration Tests (TASK-004 / RF-004 / RN-004).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use serde_json::Value;
use trustkit_cli::gate_presenter::{GatePresenter, JsonWaveRunResponse, WorktreeProvisionInfo};

#[test]
fn test_cli_worktree_notification_json_null_free_serialization() {
    let response = JsonWaveRunResponse {
        verdict: "PASS".to_string(),
        wave_index: 4,
        total_tasks: 1,
        successful_tasks: vec!["TASK-004".to_string()],
        failed_tasks: vec![],
        duration_ms: 110,
        runtime_used: "native".to_string(),
        provisioned_worktrees: vec![WorktreeProvisionInfo {
            task_id: "TASK-004".to_string(),
            branch_name: "worktree/TASK-004".to_string(),
            worktree_path: ".trustkit/worktrees/TASK-004".to_string(),
            base_branch: "develop".to_string(),
        }],
        next_steps: vec![],
    };

    let json_str = GatePresenter::render_wave_run(&response, true);

    // Absolute prohibition of NULL invariant verification
    assert!(
        !json_str.contains("null"),
        "JSON output must be strictly null-free under Constitutional Principle II"
    );

    let parsed: Value = serde_json::from_str(&json_str).expect("Valid JSON");
    assert_eq!(parsed["verdict"], "PASS");
    assert_eq!(parsed["wave_index"], 4);
    assert_eq!(parsed["total_tasks"], 1);

    let wts = parsed["provisioned_worktrees"]
        .as_array()
        .expect("provisioned_worktrees array");
    assert_eq!(wts.len(), 1);

    let wt0 = &wts[0];
    assert_eq!(wt0["task_id"], "TASK-004");
    assert_eq!(wt0["branch_name"], "worktree/TASK-004");
    assert_eq!(wt0["worktree_path"], ".trustkit/worktrees/TASK-004");
    assert_eq!(wt0["base_branch"], "develop");
}

#[test]
fn test_cli_worktree_notification_json_fallback_synthesis() {
    let response = JsonWaveRunResponse {
        verdict: "PASS".to_string(),
        wave_index: 4,
        total_tasks: 2,
        successful_tasks: vec!["TASK-004A".to_string(), "TASK-004B".to_string()],
        failed_tasks: vec![],
        duration_ms: 95,
        runtime_used: "native".to_string(),
        provisioned_worktrees: vec![],
        next_steps: vec![],
    };

    let json_str = GatePresenter::render_wave_run(&response, true);
    assert!(!json_str.contains("null"));

    let parsed: Value = serde_json::from_str(&json_str).expect("Valid JSON");
    let wts = parsed["provisioned_worktrees"]
        .as_array()
        .expect("provisioned_worktrees array");
    assert_eq!(wts.len(), 2);
    assert_eq!(wts[0]["task_id"], "TASK-004A");
    assert_eq!(wts[0]["branch_name"], "worktree/TASK-004A");
    assert_eq!(wts[1]["task_id"], "TASK-004B");
    assert_eq!(wts[1]["branch_name"], "worktree/TASK-004B");
}

#[test]
fn test_roundtrip_deserialization() {
    let response = JsonWaveRunResponse {
        verdict: "PASS".to_string(),
        wave_index: 4,
        total_tasks: 1,
        successful_tasks: vec!["TASK-004".to_string()],
        failed_tasks: vec![],
        duration_ms: 100,
        runtime_used: "native".to_string(),
        provisioned_worktrees: vec![WorktreeProvisionInfo {
            task_id: "TASK-004".to_string(),
            branch_name: "worktree/TASK-004".to_string(),
            worktree_path: ".trustkit/worktrees/TASK-004".to_string(),
            base_branch: "develop".to_string(),
        }],
        next_steps: vec![],
    };

    let json_str = GatePresenter::render_wave_run(&response, true);
    let roundtrip: JsonWaveRunResponse =
        serde_json::from_str(&json_str).expect("Deserialization roundtrip");
    assert_eq!(roundtrip.verdict, "PASS");
    assert_eq!(roundtrip.provisioned_worktrees.len(), 1);
    assert_eq!(
        roundtrip.provisioned_worktrees[0].branch_name,
        "worktree/TASK-004"
    );
}
