// Tests for dual-audience presenter.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use super::*;
use trustkit_core::entities::{CycleStatus, GateStage, MethodologyProfile, SddCycle};
use trustkit_core::special_cases::{
    ActiveCycleLookup, BoardColumn, BoardSyncOutcome, IssueLinkage, SpecLockState,
};
use trustkit_core::use_cases::initialize_cycle::InitializeCycleOutcome;

fn make_dummy_cycle(id: &str) -> SddCycle {
    SddCycle {
        cycle_id: id.to_string(),
        issue_linkage: IssueLinkage::UnlinkedLocalCycle {
            directory_slug: id.to_string(),
        },
        methodology_profile: MethodologyProfile::StandardGauntletV2,
        current_gate: GateStage::Gate0Discovery,
        lifecycle_status: CycleStatus::Active,
        gauntlet_attempt: 0,
        spec_dir: format!("docs/specs/{id}"),
        spec_file_path: format!("docs/specs/{id}/spec.md"),
        spec_seal_state: SpecLockState::DraftUnverified,
        work_branch: "feat/test".to_string(),
        base_commit_sha: "0".repeat(40),
        pre_existing_dirty_files: vec!["dirty.txt".to_string()],
        created_at: "2026-10-27T14:15:00Z".to_string(),
        updated_at: "2026-10-27T14:15:00Z".to_string(),
    }
}

#[test]
fn test_render_init_json_has_zero_null_and_next_steps() {
    let cycle = make_dummy_cycle("no-issue-202610271415");
    let outcome = InitializeCycleOutcome {
        cycle,
        sync_outcome: BoardSyncOutcome::SkippedUnlinkedCycle,
    };
    let rendered = Presenter::render_init(&outcome, true);
    assert!(!rendered.contains(": null"));
    assert!(rendered.contains("UNLINKED_LOCAL_CYCLE"));
    assert!(rendered.contains("\"next_steps\""));
    assert!(rendered.contains("trustkit spec prompt"));
}

#[test]
fn test_render_init_ascii_has_no_emojis_and_next_step_footer() {
    let mut cycle = make_dummy_cycle("issue-45-202610271415");
    cycle.issue_linkage = IssueLinkage::LinkedIssue {
        issue_number: 45,
        issue_url: "url".into(),
        project_board_id: "b".into(),
        project_item_id: "i".into(),
        board_column: BoardColumn::InProgress,
    };
    let outcome = InitializeCycleOutcome {
        cycle,
        sync_outcome: BoardSyncOutcome::Synchronized {
            column: BoardColumn::InProgress,
        },
    };
    let rendered = Presenter::render_init(&outcome, false);
    assert!(rendered.contains("[PASS] [GATE-0]"));
    assert!(rendered.contains("NEXT STEP:"));
    assert!(rendered.is_ascii());
}

#[test]
fn test_render_hook_pre_tool_json_and_ascii() {
    let json_out = Presenter::render_hook_pre_tool("cargo test", true);
    assert!(
        json_out.contains("\"verdict\": \"PASS\"") || json_out.contains("\"verdict\":\"PASS\"")
    );
    assert!(!json_out.contains(": null"));

    let ascii_out = Presenter::render_hook_pre_tool("cargo test", false);
    assert!(ascii_out.contains("[PASS] Command approved: cargo test"));
    assert!(ascii_out.is_ascii());
}

#[test]
fn test_render_status_active_cycle_json_zero_null_and_next_steps() {
    let cycle = make_dummy_cycle("issue-45-202610271415");
    let lookup = ActiveCycleLookup::Found(Box::new(cycle));
    let out = Presenter::render_status(&lookup, true);
    assert!(!out.contains(": null"));
    assert!(out.contains("ACTIVE_CYCLE"));
    assert!(out.contains("issue-45-202610271415"));
    assert!(out.contains("\"next_steps\""));
}

#[test]
fn test_render_status_no_active_cycle_json_zero_null_and_next_steps() {
    let lookup = ActiveCycleLookup::NoActiveCycle;
    let out = Presenter::render_status(&lookup, true);
    assert!(!out.contains(": null"));
    assert!(out.contains("NO_ACTIVE_CYCLE"));
    assert!(out.contains("\"next_steps\""));
    assert!(out.contains("trustkit init --issue <ISSUE_ID>"));
}

#[test]
fn test_render_status_active_ascii_has_no_emojis_and_next_step() {
    let cycle = make_dummy_cycle("issue-45-202610271415");
    let lookup = ActiveCycleLookup::Found(Box::new(cycle));
    let out = Presenter::render_status(&lookup, false);
    assert!(out.is_ascii());
    assert!(out.contains("[PASS] [GATE-0] Active Cycle Status"));
    assert!(out.contains("NEXT STEP:"));
}

#[test]
fn test_render_status_no_active_ascii_has_no_emojis_and_next_step() {
    let lookup = ActiveCycleLookup::NoActiveCycle;
    let out = Presenter::render_status(&lookup, false);
    assert!(out.is_ascii());
    assert!(out.contains("[PASS] No active cycle found."));
    assert!(out.contains("NEXT STEP:"));
}

#[test]
fn test_render_close_dual_mode_null_free_and_no_emojis() {
    let res = crate::gate_presenter::JsonCloseResponse {
        verdict: "PASS".to_string(),
        cycle_id: "issue-16-202611121758".to_string(),
        board_column: "ReadyForTest".to_string(),
        next_steps: Vec::new(),
    };

    let json_out = Presenter::render_close(&res, true);
    assert!(!json_out.is_empty());
    assert!(!json_out.contains(": null"));
    assert!(
        json_out.contains("\"verdict\":\"PASS\"") || json_out.contains("\"verdict\": \"PASS\"")
    );
    assert!(json_out.contains("ReadyForTest"));
    assert!(json_out.contains("trustkit pr --target develop"));

    let ascii_out = Presenter::render_close(&res, false);
    assert!(ascii_out.is_ascii());
    assert!(ascii_out.contains("[PASS] SDD cycle governance closed"));
    assert!(ascii_out.contains("ReadyForTest"));
    assert!(ascii_out.contains("trustkit pr --target develop"));
}
