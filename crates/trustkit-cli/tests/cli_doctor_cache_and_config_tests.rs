// Integration tests for TASK-003: Doctor compilation cache diagnostics, trustkit.toml config and documentation parity.
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;
use trustkit_cli::cli_contract::DoctorArgs;
use trustkit_cli::commands::{doctor_cmd, init_cmd};
use trustkit_core::entities::{
    BatchGroupingMode, CompilationCacheBackend, CycleStatus, GateStage, MethodologyProfile,
    SddCycle,
};
use trustkit_core::ports::CycleEvidenceRepositoryPort;
use trustkit_core::special_cases::{IssueLinkage, SpecLockState};
use trustkit_db::sqlite_adapter::SqliteWalRepositoryAdapter;

fn workspace_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .ancestors()
        .find(|p| p.join("Cargo.lock").exists())
        .unwrap_or(manifest_dir)
        .to_path_buf()
}

fn setup_active_cycle_db(dir: &Path) -> PathBuf {
    let tk_dir = dir.join(".trustkit");
    fs::create_dir_all(&tk_dir).expect("create .trustkit");
    let db_path = tk_dir.join("state.db");
    let repo = SqliteWalRepositoryAdapter::open(&db_path).expect("open db");

    let cycle = SddCycle {
        cycle_id: "issue-43-doctor".to_string(),
        issue_linkage: IssueLinkage::UnlinkedLocalCycle {
            directory_slug: "issue-43-doctor".to_string(),
        },
        methodology_profile: MethodologyProfile::StandardGauntletV2,
        current_gate: GateStage::Gate4TasksImplemented,
        lifecycle_status: CycleStatus::Active,
        gauntlet_attempt: 0,
        spec_dir: dir.to_string_lossy().to_string(),
        spec_file_path: dir.join("spec.md").to_string_lossy().to_string(),
        spec_seal_state: SpecLockState::DraftUnverified,
        work_branch: "feat/43-cache".to_string(),
        base_commit_sha: "a".repeat(40),
        pre_existing_dirty_files: vec![],
        created_at: "2026-11-23T15:00:00Z".to_string(),
        updated_at: "2026-11-23T15:00:00Z".to_string(),
    };
    repo.save_cycle(&cycle).expect("save cycle");
    db_path
}

#[test]
fn test_scaffold_trustkit_toml_includes_harness_execution_cache_and_batch_settings() {
    let dir = tempdir().expect("tempdir");
    init_cmd::scaffold_trustkit_toml(dir.path()).expect("scaffold trustkit.toml");
    let content = fs::read_to_string(dir.path().join("trustkit.toml")).expect("read toml");

    assert!(
        content.contains("[harness.execution]"),
        "trustkit.toml must contain [harness.execution] section"
    );
    assert!(
        content.contains("compilation_cache = \"auto\""),
        "trustkit.toml must configure compilation_cache"
    );
    assert!(
        content.contains("batch_grouping_mode = \"per-crate\""),
        "trustkit.toml must configure batch_grouping_mode"
    );

    let policy = doctor_cmd::load_project_execution_policy(dir.path());
    assert_eq!(
        policy.caching.compilation_cache,
        CompilationCacheBackend::Auto
    );
    assert!(policy.batching.enabled);
    assert_eq!(policy.batching.grouping_mode, BatchGroupingMode::PerCrate);
}

#[test]
fn test_doctor_reports_sccache_status_and_guides_installation_when_absent() {
    let dir = tempdir().expect("tempdir");
    init_cmd::scaffold_trustkit_toml(dir.path()).expect("scaffold toml");
    let policy = doctor_cmd::load_project_execution_policy(dir.path());

    let (status_absent, diag_absent) =
        doctor_cmd::inspect_compilation_cache_diagnostic(false, &policy);
    assert_eq!(status_absent, "SHARED_TARGET_FALLBACK");
    assert!(
        diag_absent.contains("sccache not detected")
            && diag_absent.contains("cargo install sccache"),
        "Doctor diagnostic must guide sccache installation when absent: {diag_absent}"
    );

    let (status_present, diag_present) =
        doctor_cmd::inspect_compilation_cache_diagnostic(true, &policy);
    assert_eq!(status_present, "SCCACHE_ACTIVE");
    assert!(
        diag_present.contains("sccache detected and active")
            && diag_present.contains("RUSTC_WRAPPER=sccache"),
        "Doctor diagnostic must confirm active sccache when detected: {diag_present}"
    );

    let db_path = setup_active_cycle_db(dir.path());
    let args = DoctorArgs { json: true };
    let res = doctor_cmd::execute_with_db_and_sccache(&args, true, &db_path, false);
    assert!(res.is_ok());
}

#[test]
fn test_documentation_parity_for_cache_and_batching_across_readmes_and_manuals() {
    let root = workspace_root();
    let docs = [
        "README.md",
        "README.en.md",
        "docs/manual/MANUAL_DO_USUARIO.md",
        "docs/manual/manual.html",
        "docs/manuals/user-guide.md",
        "docs/manuals/user-guide.html",
    ];

    for rel_path in docs {
        let full_path = root.join(rel_path);
        assert!(full_path.exists(), "{rel_path} must exist");
        let content = fs::read_to_string(&full_path).expect("read doc file");
        assert!(
            content.contains("sccache"),
            "{rel_path} must document sccache compilation caching"
        );
        assert!(
            content.contains("ToolCachingPolicy") || content.contains("compilation_cache"),
            "{rel_path} must document ToolCachingPolicy or compilation_cache"
        );
        assert!(
            content.contains("BatchExecutionConfig") || content.contains("batch_grouping_mode"),
            "{rel_path} must document BatchExecutionConfig or batch_grouping_mode"
        );
    }
}
