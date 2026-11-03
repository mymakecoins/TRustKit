use super::*;

fn is_emoji(ch: char) -> bool {
    matches!(
        ch as u32,
        0x1F600..=0x1F64F
            | 0x1F300..=0x1F5FF
            | 0x1F680..=0x1F6FF
            | 0x1F700..=0x1F77F
            | 0x1F780..=0x1F7FF
            | 0x1F800..=0x1F8FF
            | 0x1F900..=0x1F9FF
            | 0x1FA00..=0x1FA6F
            | 0x1FA70..=0x1FAFF
            | 0x2600..=0x26FF
            | 0x2700..=0x27BF
    )
}

fn assert_no_emojis(text: &str) {
    for ch in text.chars() {
        assert!(!is_emoji(ch), "Forbidden emoji found: '{ch}' in text: {text}");
    }
}

fn assert_no_null(text: &str) {
    assert!(!text.contains("null"), "JSON contains forbidden 'null': {text}");
}

#[test]
fn test_render_task_test_dual_mode() {
    let payload = JsonTaskTestResponse {
        verdict: "PASS".to_string(),
        task_id: "TASK-001".to_string(),
        phase: "RED".to_string(),
        exit_code: 1,
        duration_ms: 120,
        evidence_id: 42,
        status: "RED_VERIFIED".to_string(),
    };

    let json = GatePresenter::render_task_test(&payload, true);
    assert_no_null(&json);
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid json");
    assert_eq!(parsed["verdict"], "PASS");
    assert_eq!(parsed["task_id"], "TASK-001");

    let tty = GatePresenter::render_task_test(&payload, false);
    assert_no_emojis(&tty);
    assert!(tty.contains("[PASS] Task TASK-001 test --phase RED"));
}

#[test]
fn test_render_task_complete_dual_mode() {
    let payload = JsonTaskCompleteResponse {
        verdict: "PASS".to_string(),
        task_id: "TASK-001".to_string(),
        new_state: "GREEN_VERIFIED".to_string(),
        timestamp: "2026-10-28T12:00:00Z".to_string(),
    };

    let json = GatePresenter::render_task_complete(&payload, true);
    assert_no_null(&json);
    let tty = GatePresenter::render_task_complete(&payload, false);
    assert_no_emojis(&tty);
}

#[test]
fn test_render_wave_run_dual_mode() {
    let payload = JsonWaveRunResponse {
        verdict: "PASS".to_string(),
        wave_index: 1,
        total_tasks: 2,
        successful_tasks: vec!["TASK-001".to_string(), "TASK-002".to_string()],
        failed_tasks: vec![],
        duration_ms: 450,
        runtime_used: "native".to_string(),
    };

    let json = GatePresenter::render_wave_run(&payload, true);
    assert_no_null(&json);
    let tty = GatePresenter::render_wave_run(&payload, false);
    assert_no_emojis(&tty);
}

#[test]
fn test_render_gauntlet_run_dual_mode() {
    let payload = JsonGauntletRunResponse {
        verdict: "PASS".to_string(),
        layer0_status: "CLEAN_PASS".to_string(),
        layer1_domain_status: "PASS".to_string(),
        layer1_contract_status: "PASS".to_string(),
        layer1_build_status: "PASS".to_string(),
        layer1_e2e_status: "PASS".to_string(),
        critics_passed: 4,
        critics_total: 4,
        circuit_breaker_tripped: false,
    };

    let json = GatePresenter::render_gauntlet_run(&payload, true);
    assert_no_null(&json);
    let tty = GatePresenter::render_gauntlet_run(&payload, false);
    assert_no_emojis(&tty);
}

#[test]
fn test_render_doctor_dual_mode() {
    let payload = JsonDoctorResponse {
        verdict: "FAIL".to_string(),
        circuit_breaker_tripped: true,
        consecutive_failures: 5,
        diagnostics: vec!["Critic build failed".to_string()],
        remediation: "Inspect compiler errors".to_string(),
    };

    let json = GatePresenter::render_doctor(&payload, true);
    assert_no_null(&json);
    let tty = GatePresenter::render_doctor(&payload, false);
    assert_no_emojis(&tty);
}

#[test]
fn test_render_done_dual_mode() {
    let payload = JsonDoneResponse {
        verdict: "PASS".to_string(),
        cycle_id: "004-cycle".to_string(),
        gate: "GATE-6".to_string(),
        report_file: "specs/004/gauntlet-report.md".to_string(),
        issue_comment: "Cycle certified successfully.".to_string(),
    };

    let json = GatePresenter::render_done(&payload, true);
    assert_no_null(&json);
    let tty = GatePresenter::render_done(&payload, false);
    assert_no_emojis(&tty);
}

#[test]
fn test_render_merge_and_pr_dual_mode() {
    let merge_payload = JsonMergeResponse {
        verdict: "PASS".to_string(),
        cycle_id: "004-cycle".to_string(),
        target_branch: "develop".to_string(),
        commit_hash: "a1b2c3d".to_string(),
        board_column: "AWAIT_STAGING".to_string(),
    };
    let json_merge = GatePresenter::render_merge(&merge_payload, true);
    assert_no_null(&json_merge);
    let tty_merge = GatePresenter::render_merge(&merge_payload, false);
    assert_no_emojis(&tty_merge);

    let pr_payload = JsonPrResponse {
        verdict: "PASS".to_string(),
        cycle_id: "004-cycle".to_string(),
        pr_url: "https://github.com/org/repo/pull/42".to_string(),
        pr_number: 42,
        target_branch: "develop".to_string(),
        board_column: "IN_REVIEW".to_string(),
    };
    let json_pr = GatePresenter::render_pr(&pr_payload, true);
    assert_no_null(&json_pr);
    let tty_pr = GatePresenter::render_pr(&pr_payload, false);
    assert_no_emojis(&tty_pr);
}

#[test]
fn test_render_mcp_serve_dual_mode() {
    let payload = JsonMcpServeResponse {
        status: "RUNNING".to_string(),
        transport: "stdio".to_string(),
        server_name: "trustkit-mcp".to_string(),
        version: "0.1.0".to_string(),
    };
    let json = GatePresenter::render_mcp_serve(&payload, true);
    assert_no_null(&json);
    let tty = GatePresenter::render_mcp_serve(&payload, false);
    assert_no_emojis(&tty);
}
