// Tests for gauntlet ast-audit and context slice command handlers (TASK-014).

use std::fs::File;
use std::io::Write;
use tempfile::tempdir;
use trustkit_cli::cli_contract::{ContextSliceArgs, GauntletAstAuditArgs};
use trustkit_cli::commands::{context_slice_cmd, gauntlet_ast_audit_cmd};
use trustkit_cli::error_handler::DomainOutcome;
use trustkit_core::entities::{CycleStatus, GateStage, MethodologyProfile, SddCycle};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::CycleEvidenceRepositoryPort;
use trustkit_core::special_cases::{IssueLinkage, SpecLockState};
use trustkit_db::sqlite_adapter::SqliteWalRepositoryAdapter;

fn setup_test_db(dir: &std::path::Path) -> std::path::PathBuf {
    let db_path = dir.join("state.db");
    let repo = SqliteWalRepositoryAdapter::open(&db_path).expect("Failed to open test db");

    let cycle = SddCycle {
        cycle_id: "test-cycle-ast".to_string(),
        issue_linkage: IssueLinkage::UnlinkedLocalCycle {
            directory_slug: "test-cycle-ast".to_string(),
        },
        methodology_profile: MethodologyProfile::StandardGauntletV2,
        current_gate: GateStage::Gate3WavePlanned,
        lifecycle_status: CycleStatus::Active,
        gauntlet_attempt: 0,
        spec_dir: dir.to_str().unwrap().to_string(),
        spec_file_path: dir.join("spec.md").to_str().unwrap().to_string(),
        spec_seal_state: SpecLockState::DraftUnverified,
        work_branch: "feat/003-ast".to_string(),
        base_commit_sha: "abcd".to_string(),
        pre_existing_dirty_files: Vec::new(),
        created_at: "2026-10-28T00:00:00Z".to_string(),
        updated_at: "2026-10-28T00:00:00Z".to_string(),
    };

    repo.save_cycle(&cycle).expect("Failed to save cycle");
    db_path
}

#[test]
fn test_context_slice_cmd_execution() {
    let dir = tempdir().expect("Failed to create tempdir");
    let file_path = dir.path().join("model.rs");
    let mut file = File::create(&file_path).expect("Failed to create file");
    writeln!(
        file,
        r#"
pub struct User {{
    pub id: u64,
}}
impl User {{
    pub fn id(&self) -> u64 {{
        self.id
    }}
}}
"#
    )
    .unwrap();

    let args = ContextSliceArgs {
        file: Some(file_path.to_str().unwrap().to_string()),
        files: Vec::new(),
        budget: Some(20),
        symbols: vec!["User".to_string()],
        json: true,
    };

    let outcome = context_slice_cmd::execute(&args, true).expect("Execution failed");
    assert_eq!(outcome, DomainOutcome::Success);
}

#[test]
fn test_gauntlet_ast_audit_cmd_clean_pass() {
    let dir = tempdir().expect("Failed to create tempdir");
    let db_path = setup_test_db(dir.path());

    let clean_file = dir.path().join("clean.rs");
    let mut file = File::create(&clean_file).expect("Failed to create file");
    writeln!(file, "pub fn add(a: i32, b: i32) -> i32 {{ a + b }}").unwrap();

    let args = GauntletAstAuditArgs {
        files: vec![clean_file.to_str().unwrap().to_string()],
        json: true,
    };

    let outcome = gauntlet_ast_audit_cmd::execute_with_db(&args, true, &db_path)
        .expect("Audit execution failed");
    assert_eq!(outcome, DomainOutcome::Success);
}

#[test]
fn test_gauntlet_ast_audit_cmd_rejection() {
    let dir = tempdir().expect("Failed to create tempdir");
    let db_path = setup_test_db(dir.path());

    let emoji_file = dir.path().join("dirty.rs");
    let mut file = File::create(&emoji_file).expect("Failed to create file");
    writeln!(file, "// rocket 🚀\npub fn clean() {{}}").unwrap();

    let args = GauntletAstAuditArgs {
        files: vec![emoji_file.to_str().unwrap().to_string()],
        json: true,
    };

    let result = gauntlet_ast_audit_cmd::execute_with_db(&args, true, &db_path);
    match result {
        Err(DomainViolation::AstIntegrityOrInvariantViolation { invariant_id, .. }) => {
            assert!(invariant_id.contains("RN-007") || invariant_id.contains("AST"));
        }
        _ => panic!("Expected AstIntegrityOrInvariantViolation"),
    }
}
