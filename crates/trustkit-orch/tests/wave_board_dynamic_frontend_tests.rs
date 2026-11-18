// Integration tests for Dynamic Reactive Kanban Frontend and EventSource Client (TASK-003 / RF-003 / RN-003).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use std::fs;
use std::path::Path;
use trustkit_orch::wave_board_dashboard_sse_client::{
    KanbanCardMutation, WaveBoardDashboardClient,
};
use trustkit_orch::wave_board_snapshot_builder::{
    BoardMetricsSummary, BoardSnapshot, KanbanCardSnapshot,
};

#[test]
fn test_task_003_dashboard_client_replaces_static_cards_with_snapshot_and_moves_columns() {
    let mut client = WaveBoardDashboardClient::new();

    // Seed static demo card #008
    client.apply_mutation(KanbanCardMutation {
        task_id: "#008-sdd-auto".to_string(),
        previous_column: "GATE 0: DISCOVERY".to_string(),
        new_column: "GATE 0: DISCOVERY".to_string(),
        tdd_phase: "DRAFT".to_string(),
        timestamp_iso: "2026-11-18T09:00:00Z".to_string(),
    });
    assert_eq!(client.card_count(), 1);

    let snapshot = BoardSnapshot {
        cycle_id: "issue-32-202611180939".to_string(),
        current_gate: 4,
        metrics: BoardMetricsSummary {
            active_cycles: 1,
            total_cards: 2,
            active_worktrees: 1,
            gauntlet_attempts: 0,
        },
        cards: vec![
            KanbanCardSnapshot {
                card_id: "issue-32-TASK-001".to_string(),
                cycle_id: "issue-32-202611180939".to_string(),
                task_id: "TASK-001".to_string(),
                title: "Publicador IPC".to_string(),
                gate: 4,
                column_index: 4,
                gate_label: "GATE 4: TASKS IMPLEMENTED".to_string(),
                status_label: "Em Desenvolvimento (TDD)".to_string(),
                priority: "PRIMARY".to_string(),
                tdd_phase: "COMMITTED".to_string(),
                blake3_seal: "7eda7cb0".to_string(),
                worktree_branch: "mymakecoins/32".to_string(),
                wave_label: "Wave 1/4".to_string(),
                gauntlet_attempt: 0,
                next_step_command: "tk gauntlet run --json".to_string(),
            },
            KanbanCardSnapshot {
                card_id: "issue-32-TASK-002".to_string(),
                cycle_id: "issue-32-202611180939".to_string(),
                task_id: "TASK-002".to_string(),
                title: "Snapshot SQLite".to_string(),
                gate: 3,
                column_index: 3,
                gate_label: "GATE 3: WAVE PLANNED".to_string(),
                status_label: "Pronto para Execucao (Planejado)".to_string(),
                priority: "PRIMARY".to_string(),
                tdd_phase: "AWAITING_RED".to_string(),
                blake3_seal: "7eda7cb0".to_string(),
                worktree_branch: "mymakecoins/32".to_string(),
                wave_label: "Wave 2/4".to_string(),
                gauntlet_attempt: 0,
                next_step_command: "tk task test TASK-002 --phase red --json".to_string(),
            },
        ],
    };

    client.load_snapshot(&snapshot);
    assert!(
        client.find_card("#008-sdd-auto").is_none(),
        "Static demo card must be cleared after loading snapshot"
    );
    assert_eq!(client.card_count(), 2);
    assert_eq!(client.column_card_count("GATE 4: TASKS IMPLEMENTED"), 1);
    assert_eq!(client.column_card_count("GATE 3: WAVE PLANNED"), 1);

    // Apply incremental SSE mutation moving TASK-002 to Gate 4 RED and then GREEN
    client.apply_mutation(KanbanCardMutation {
        task_id: "TASK-002".to_string(),
        previous_column: "GATE 3: WAVE PLANNED".to_string(),
        new_column: "GATE 4: TASKS IMPLEMENTED".to_string(),
        tdd_phase: "RED".to_string(),
        timestamp_iso: "2026-11-18T09:50:00Z".to_string(),
    });
    assert_eq!(client.column_card_count("GATE 3: WAVE PLANNED"), 0);
    assert_eq!(client.column_card_count("GATE 4: TASKS IMPLEMENTED"), 2);
    assert_eq!(
        client.find_card("TASK-002").expect("TASK-002").tdd_phase,
        "RED"
    );

    client.apply_mutation(KanbanCardMutation {
        task_id: "TASK-002".to_string(),
        previous_column: "GATE 4: TASKS IMPLEMENTED".to_string(),
        new_column: "GATE 5: GAUNTLET CLEARED".to_string(),
        tdd_phase: "GREEN".to_string(),
        timestamp_iso: "2026-11-18T09:51:00Z".to_string(),
    });
    assert_eq!(client.column_card_count("GATE 4: TASKS IMPLEMENTED"), 1);
    assert_eq!(client.column_card_count("GATE 5: GAUNTLET CLEARED"), 1);
}

#[test]
fn test_task_003_dashboard_html_contains_reactive_eventsource_and_snapshot_reconciliation() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let html_path = Path::new(manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .expect("Workspace root")
        .join("docs/prototypes/trustkit-kanban-dashboard.html");

    let html = fs::read_to_string(&html_path).expect("Failed reading dashboard HTML");
    assert!(
        WaveBoardDashboardClient::verify_dashboard_html_reactive_contract(&html),
        "Dashboard HTML must contain EventSource('/events'), /api/snapshot reconciliation, dynamic column rendering, and connection status indicator"
    );
}
