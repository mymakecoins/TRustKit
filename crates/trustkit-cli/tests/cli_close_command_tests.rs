// Integration tests for CLI close command and merge deprecation (TASK-003).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use clap::Parser;
use tempfile::tempdir;
use trustkit_cli::cli_contract::{Cli, CloseArgs, Command, MergeArgs};
use trustkit_cli::commands::{close_cmd, merge_cmd};
use trustkit_cli::error_handler::CliBoundaryErrorHandler;
use trustkit_core::entities::{CycleStatus, GateStage, MethodologyProfile, SddCycle};
use trustkit_core::ports::CycleEvidenceRepositoryPort;
use trustkit_core::special_cases::{ActiveCycleLookup, BoardColumn, IssueLinkage, SpecLockState};
use trustkit_db::sqlite_adapter::SqliteWalRepositoryAdapter;

fn setup_test_cycle(
    dir: &std::path::Path,
    status: CycleStatus,
) -> (std::path::PathBuf, SqliteWalRepositoryAdapter) {
    let db_path = dir.join("state.db");
    let repo = SqliteWalRepositoryAdapter::open(&db_path).expect("Failed to open test db");

    let cycle = SddCycle {
        cycle_id: "cycle-close-test".to_string(),
        issue_linkage: IssueLinkage::LinkedIssue {
            issue_number: 16,
            issue_url: "https://github.com/mymakecoins/TRustKit/issues/16".to_string(),
            project_board_id: "board-16".to_string(),
            project_item_id: "item-16".to_string(),
            board_column: BoardColumn::InProgress,
        },
        methodology_profile: MethodologyProfile::StandardGauntletV2,
        current_gate: match status {
            CycleStatus::Certified => GateStage::Gate6Certified,
            _ => GateStage::Gate4TasksImplemented,
        },
        lifecycle_status: status,
        gauntlet_attempt: 0,
        spec_dir: dir.to_str().unwrap().to_string(),
        spec_file_path: dir.join("spec.md").to_str().unwrap().to_string(),
        spec_seal_state: SpecLockState::Frozen {
            spec_path: dir.join("spec.md").to_str().unwrap().to_string(),
            blake3_digest: "0".repeat(64),
        },
        work_branch: "mymakecoins/refactor-redefinir-nome-e-sintaxe-do-comando-de".to_string(),
        base_commit_sha: "2b4b09f8380c37d76ee57d2ef819a16984d8fe65".to_string(),
        pre_existing_dirty_files: Vec::new(),
        created_at: "2026-11-12T00:00:00Z".to_string(),
        updated_at: "2026-11-12T00:00:00Z".to_string(),
    };

    repo.save_cycle(&cycle).expect("Failed to save cycle");
    (db_path, repo)
}

#[test]
fn test_parse_close_command_cli_contract() {
    let cli_close = Cli::try_parse_from(["trustkit", "close"]).expect("Failed to parse close");
    assert!(!cli_close.is_json_mode());
    assert_eq!(cli_close.command, Command::Close(CloseArgs { json: false }));

    let cli_close_json =
        Cli::try_parse_from(["trustkit", "close", "--json"]).expect("Failed to parse close --json");
    assert!(cli_close_json.is_json_mode());
    assert_eq!(
        cli_close_json.command,
        Command::Close(CloseArgs { json: true })
    );
}

#[test]
fn test_parse_legacy_merge_command_optional_target() {
    let cli_merge = Cli::try_parse_from(["trustkit", "merge"])
        .expect("Failed to parse legacy merge without target");
    assert!(!cli_merge.is_json_mode());
    assert_eq!(
        cli_merge.command,
        Command::Merge(MergeArgs {
            target: None,
            json: false
        })
    );
}

#[test]
fn test_merge_command_emits_deprecation_warning() {
    let dir = tempdir().expect("tempdir");
    let (db_path, _) = setup_test_cycle(dir.path(), CycleStatus::Active);

    let args = MergeArgs {
        target: Some("develop".to_string()),
        json: false,
    };

    // Calling merge_cmd emits deprecation warning constant
    assert!(merge_cmd::MERGE_DEPRECATION_WARNING.contains("trustkit merge"));
    assert!(merge_cmd::MERGE_DEPRECATION_WARNING.contains("trustkit close"));

    let res = merge_cmd::execute_with_db(&args, false, &db_path);
    // Cycle is Active (not Certified), so fails cleanly with boundary fault
    assert!(res.is_err());
}

#[test]
fn test_close_command_certified_cycle_success() {
    let dir = tempdir().expect("tempdir");
    let (db_path, repo) = setup_test_cycle(dir.path(), CycleStatus::Certified);

    let args = CloseArgs { json: false };
    let outcome = close_cmd::execute_with_db(&args, false, &db_path).expect("close should succeed");
    assert_eq!(outcome, trustkit_cli::error_handler::DomainOutcome::Success);

    // Verify cycle in state.db is now Merged (closed)
    let cycle_after = match repo.find_active_cycle().expect("find_active_cycle") {
        ActiveCycleLookup::Found(c) => c,
        ActiveCycleLookup::NoActiveCycle => panic!("Cycle should still be found"),
    };
    assert_eq!(cycle_after.lifecycle_status, CycleStatus::Merged);
}

#[test]
fn test_close_command_uncertified_cycle_rejected() {
    let dir = tempdir().expect("tempdir");
    let (db_path, _) = setup_test_cycle(dir.path(), CycleStatus::Active);

    let args = CloseArgs { json: false };
    let result = close_cmd::execute_with_db(&args, false, &db_path);
    assert!(result.is_err());

    let mut stderr_output = Vec::new();
    let exit_code =
        CliBoundaryErrorHandler::handle_result_with_writer(result, false, &mut stderr_output);
    assert_eq!(exit_code, 1);

    let err_str = String::from_utf8_lossy(&stderr_output);
    assert!(err_str.contains("cycle is not Certified"));
    assert!(err_str.contains("Gate 6 certification required"));
}
