// CLI Worktree Notification TTY Integration Tests (TASK-003 / RF-003 / RN-003).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use trustkit_cli::gate_presenter::{GatePresenter, JsonWaveRunResponse, WorktreeProvisionInfo};

#[test]
fn test_cli_worktree_notification_tty_rendering() {
    let response = JsonWaveRunResponse {
        verdict: "PASS".to_string(),
        wave_index: 3,
        total_tasks: 1,
        successful_tasks: vec!["TASK-003".to_string()],
        failed_tasks: vec![],
        duration_ms: 120,
        runtime_used: "native".to_string(),
        provisioned_worktrees: vec![WorktreeProvisionInfo {
            task_id: "TASK-003".to_string(),
            branch_name: "worktree/TASK-003".to_string(),
            worktree_path: ".trustkit/worktrees/TASK-003".to_string(),
            base_branch: "develop".to_string(),
        }],
        next_steps: vec![],
    };

    let rendered = GatePresenter::render_wave_run(&response, false);

    assert!(rendered.contains("[INFO] Branch 'worktree/TASK-003' criada com sucesso."));
    assert!(
        rendered.contains(
            "[INFO] Checkout realizado no diretorio isolado: .trustkit/worktrees/TASK-003"
        )
    );
    assert!(rendered.contains("[INFO] Branch ativa do repositorio raiz preservada: 'develop'"));
    assert!(rendered.contains("[PASS] Wave 3 executed: 1/1 tasks passed in 120ms"));

    // Verify pure ASCII (no emojis or non-ASCII characters)
    assert!(
        rendered.is_ascii(),
        "Rendered output must be pure ASCII without emojis"
    );
}

#[test]
fn test_cli_worktree_notification_tty_fallback_from_successful_tasks() {
    let response = JsonWaveRunResponse {
        verdict: "PASS".to_string(),
        wave_index: 3,
        total_tasks: 1,
        successful_tasks: vec!["TASK-003".to_string()],
        failed_tasks: vec![],
        duration_ms: 80,
        runtime_used: "native".to_string(),
        provisioned_worktrees: vec![],
        next_steps: vec![],
    };

    let rendered = GatePresenter::render_wave_run(&response, false);
    assert!(rendered.contains("[INFO] Branch 'worktree/TASK-003' criada com sucesso."));
    assert!(
        rendered.contains(
            "[INFO] Checkout realizado no diretorio isolado: .trustkit/worktrees/TASK-003"
        )
    );
    assert!(rendered.contains("[INFO] Branch ativa do repositorio raiz preservada: 'develop'"));
    assert!(rendered.is_ascii());
}

#[test]
fn test_render_worktree_provision_notification_helper() {
    let formatted = GatePresenter::render_worktree_provision_notification(
        "worktree/TASK-XYZ",
        ".trustkit/worktrees/TASK-XYZ",
        "feature-123",
    );
    assert_eq!(
        formatted,
        "[INFO] Branch 'worktree/TASK-XYZ' criada com sucesso.\n[INFO] Checkout realizado no diretorio isolado: .trustkit/worktrees/TASK-XYZ\n[INFO] Branch ativa do repositorio raiz preservada: 'feature-123'"
    );
    assert!(formatted.is_ascii());
}
