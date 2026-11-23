// Integration tests for TASK-001: ToolCachingPolicy, BatchExecutionConfig and HarnessRetroEngine metrics.
// Constitutional Principle I: Strict Hexagonal Isolation (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use trustkit_core::entities::{
    BatchExecutionConfig, BatchGroupingMode, CompilationCacheBackend, EvidenceKind,
    ExecutionEvidence, GateStage, GateVerdict, HarnessRetroEngine, ToolCachingPolicy,
    parse_harness_execution_policy,
};

fn make_cmd_evidence(id: u64, cmd: &str, stdout: &str, duration_ms: u64) -> ExecutionEvidence {
    ExecutionEvidence {
        evidence_id: id,
        cycle_id: "issue-43-test".to_string(),
        gate_stage: GateStage::Gate4TasksImplemented,
        gauntlet_attempt: 0,
        actor_role: "BUILDER:TDD_GREEN".to_string(),
        evidence_kind: EvidenceKind::CmdExec,
        executed_command: cmd.to_string(),
        exit_code: 0,
        duration_ms,
        sanitized_stdout_tail: stdout.to_string(),
        sanitized_stderr_tail: String::new(),
        output_blake3_digest: format!("{:064x}", id),
        verdict: GateVerdict::Pass,
        recorded_at: "2026-11-23T15:00:00Z".to_string(),
    }
}

#[test]
fn test_tool_caching_policy_resolution_and_validation() {
    let default_policy = ToolCachingPolicy::default();
    assert_eq!(
        default_policy.compilation_cache,
        CompilationCacheBackend::Auto
    );
    assert!(default_policy.is_compilation_cache_enabled());
    assert_eq!(
        default_policy.resolve_rustc_wrapper(true),
        Some("sccache"),
        "Auto backend must resolve sccache wrapper when available"
    );
    assert_eq!(
        default_policy.resolve_rustc_wrapper(false),
        None,
        "Auto backend must not set sccache wrapper when sccache is absent"
    );
    assert_eq!(
        default_policy.resolve_cargo_target_dir(),
        Some(".trustkit/cache/target"),
        "Enabled compilation cache must expose shared cargo target dir"
    );

    let invalid = ToolCachingPolicy::new(CompilationCacheBackend::SharedTargetDir, "   ", true, true);
    assert!(
        invalid.is_err(),
        "Empty shared_target_dir must be rejected when compilation cache is enabled"
    );

    let disabled = ToolCachingPolicy::new(CompilationCacheBackend::Disabled, "", false, false)
        .expect("Disabled policy accepts empty target dir");
    assert!(!disabled.is_compilation_cache_enabled());
    assert_eq!(disabled.resolve_rustc_wrapper(true), None);
    assert_eq!(disabled.resolve_cargo_target_dir(), None);
}

#[test]
fn test_batch_execution_config_validation_and_activity() {
    let default_batch = BatchExecutionConfig::default();
    assert!(default_batch.is_batching_active());
    assert_eq!(default_batch.grouping_mode, BatchGroupingMode::PerCrate);
    assert_eq!(default_batch.max_batch_size, 8);
    assert!(default_batch.preserve_task_traceability);

    let zero_size = BatchExecutionConfig::new(true, BatchGroupingMode::PerWave, 0, true);
    assert!(zero_size.is_err(), "max_batch_size=0 must be rejected");

    let oversized = BatchExecutionConfig::new(true, BatchGroupingMode::PerWave, 65, true);
    assert!(oversized.is_err(), "max_batch_size>64 must be rejected");

    let untraceable = BatchExecutionConfig::new(true, BatchGroupingMode::PerCrate, 4, false);
    assert!(
        untraceable.is_err(),
        "Enabled batching without task traceability must be rejected"
    );
}

#[test]
fn test_parse_harness_execution_policy_from_toml() {
    let raw_toml = r#"
[governance.gauntlet]
security_review_mode = "prompt"
min_confidence_threshold = 8

[harness.execution]
compilation_cache = "sccache"
shared_target_dir = ".trustkit/cache/shared-rust"
ast_cache_enabled = true
coupling_cache_enabled = false
batch_execution_enabled = true
batch_grouping_mode = "per-wave"
max_batch_size = 12
"#;

    let parsed = parse_harness_execution_policy(raw_toml).expect("valid harness execution policy");
    assert_eq!(
        parsed.caching.compilation_cache,
        CompilationCacheBackend::Sccache
    );
    assert_eq!(
        parsed.caching.shared_target_dir,
        ".trustkit/cache/shared-rust"
    );
    assert!(parsed.caching.ast_analysis_cache_enabled);
    assert!(!parsed.caching.structural_coupling_cache_enabled);
    assert!(parsed.batching.enabled);
    assert_eq!(parsed.batching.grouping_mode, BatchGroupingMode::PerWave);
    assert_eq!(parsed.batching.max_batch_size, 12);

    let bad_toml = "compilation_cache = \"invalid-backend\"\n";
    assert!(parse_harness_execution_policy(bad_toml).is_err());
}

#[test]
fn test_harness_retro_engine_quantifies_cache_reuse_and_cycle_time() {
    let evidences = vec![
        make_cmd_evidence(
            1,
            "RUSTC_WRAPPER=sccache cargo test -p trustkit-core --test t1",
            "CACHE_HIT=true test result: ok",
            120,
        ),
        make_cmd_evidence(
            2,
            "cargo test -p trustkit-orch --target-dir .trustkit/cache/target --test t2 --test t3",
            "BATCH_EXEC=2 test result: ok",
            180,
        ),
        make_cmd_evidence(
            3,
            "RUSTC_WRAPPER=sccache cargo test -p trustkit-cli --test t4",
            "CACHE_HIT=true test result: ok",
            150,
        ),
        make_cmd_evidence(4, "cargo fmt --check", "ok", 50),
        make_cmd_evidence(
            5,
            "RUSTC_WRAPPER=sccache cargo clippy --workspace",
            "CACHE_HIT=true ok",
            250,
        ),
    ];

    let (report, metrics) =
        HarnessRetroEngine::analyze_events_with_metrics("issue-43-test", &evidences);

    assert_eq!(metrics.total_cmd_executions, 5);
    assert_eq!(metrics.cached_executions, 4);
    assert_eq!(metrics.batched_executions, 1);
    assert_eq!(metrics.cache_reuse_rate_pct, 80);
    assert_eq!(metrics.total_duration_ms, 750);
    assert_eq!(metrics.average_cycle_time_ms, 150);

    let econ_item = report
        .items
        .iter()
        .find(|i| i.category == "Economia de Ferramentas")
        .expect("Must include Economia de Ferramentas item");
    assert_eq!(econ_item.severity, "LOW");
    assert!(
        econ_item
            .recommendation
            .contains("taxa de reaproveitamento de 80%"),
        "Retro recommendation must reflect cache reuse rate: {}",
        econ_item.recommendation
    );
    assert!(
        econ_item
            .recommendation
            .contains("tempo medio de ciclo de 150ms"),
        "Retro recommendation must reflect average cycle time: {}",
        econ_item.recommendation
    );
}
