// Integration tests for TASK-002: Compilation cache in worktrees, command batching and AST wave cache.
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use tempfile::tempdir;
use trustkit_core::entities::{
    AstAuditVerdict, BatchExecutionConfig, BatchGroupingMode, CompilationCacheBackend,
    EdgeConfidence, EdgeRelationKind, GraphNodeKind, StructuralEdge, StructuralNode,
    ToolCachingPolicy,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::{
    AsyncExecutionSupervisorPort, SupervisedExecutionOutcome, SupervisedExecutionRequest,
    WorktreeIsolationPort,
};
use trustkit_core::special_cases::WorktreeAllocation;
use trustkit_orch::{
    AstWaveAnalysisCache, BuilderAgentDispatcher, TaskCommandSpec, WaveBatchCommandPlanner,
    WorktreeIsolationProvider,
};

fn setup_git_repo() -> tempfile::TempDir {
    let dir = tempdir().expect("tempdir");
    let path = dir.path();

    Command::new("git")
        .args(["init", "-b", "develop"])
        .current_dir(path)
        .output()
        .expect("git init");
    Command::new("git")
        .args(["config", "user.name", "Tester"])
        .current_dir(path)
        .output()
        .expect("git user");
    Command::new("git")
        .args(["config", "user.email", "test@test.local"])
        .current_dir(path)
        .output()
        .expect("git email");

    fs::write(path.join("init.txt"), "init content").expect("write init");
    Command::new("git")
        .args(["add", "init.txt"])
        .current_dir(path)
        .output()
        .expect("git add");
    Command::new("git")
        .args(["commit", "-m", "chore: #43 initial"])
        .current_dir(path)
        .output()
        .expect("git commit");

    dir
}

#[test]
fn test_worktree_provider_provisions_shared_compilation_cache_and_env() {
    let repo_dir = setup_git_repo();
    let policy = ToolCachingPolicy::new(
        CompilationCacheBackend::Auto,
        ".trustkit/cache/target",
        true,
        true,
    )
    .expect("valid policy");

    let provider =
        WorktreeIsolationProvider::new(repo_dir.path()).with_caching_policy(policy.clone());

    let envs_with_sccache = provider.resolve_compilation_cache_env(true);
    assert!(
        envs_with_sccache
            .iter()
            .any(|(k, v)| k == "RUSTC_WRAPPER" && v == "sccache"),
        "Must configure RUSTC_WRAPPER=sccache when sccache is available"
    );
    assert!(
        envs_with_sccache
            .iter()
            .any(|(k, v)| k == "CARGO_TARGET_DIR" && v.ends_with(".trustkit/cache/target")),
        "Must configure shared CARGO_TARGET_DIR"
    );

    let alloc = provider
        .provision_worktree("cycle-43", "TASK-001", "develop")
        .expect("provision worktree");

    let wt_path = match alloc {
        WorktreeAllocation::Provisioned { worktree_path, .. } => worktree_path,
        _ => panic!("Expected Provisioned worktree"),
    };

    assert!(
        repo_dir.path().join(".trustkit/cache/target").exists(),
        "Shared target cache dir must be created during worktree provisioning"
    );

    let status = provider
        .check_worktree_status(&wt_path)
        .expect("worktree status");
    assert!(
        status.is_clean,
        "Worktree with linked target cache must remain clean in git status"
    );

    let dispatcher = BuilderAgentDispatcher::new().with_caching_policy(policy);
    assert_eq!(
        dispatcher.resolve_target_dir("TASK-001"),
        ".trustkit/cache/target"
    );
}

struct RecordingSupervisor {
    calls: std::sync::Mutex<Vec<SupervisedExecutionRequest>>,
    fail_combined: bool,
    failing_task_arg: String,
}

impl RecordingSupervisor {
    fn new(fail_combined: bool, failing_task_arg: &str) -> Self {
        Self {
            calls: std::sync::Mutex::new(Vec::new()),
            fail_combined,
            failing_task_arg: failing_task_arg.to_string(),
        }
    }
}

impl AsyncExecutionSupervisorPort for RecordingSupervisor {
    fn execute_supervised(
        &self,
        request: &SupervisedExecutionRequest,
    ) -> Result<SupervisedExecutionOutcome, DomainViolation> {
        self.calls.lock().unwrap().push(request.clone());
        let test_flags = request.args.iter().filter(|a| *a == "--test").count();
        if self.fail_combined && test_flags > 1 {
            return Ok(SupervisedExecutionOutcome {
                exit_code: 101,
                duration_ms: 40,
                stdout_tail: "combined batch failed".to_string(),
                stderr_tail: "".to_string(),
                output_blake3_digest: "fail_digest".to_string(),
            });
        }
        if !self.failing_task_arg.is_empty()
            && request.args.iter().any(|a| a == &self.failing_task_arg)
        {
            return Ok(SupervisedExecutionOutcome {
                exit_code: 101,
                duration_ms: 25,
                stdout_tail: format!("test {} failed", self.failing_task_arg),
                stderr_tail: "".to_string(),
                output_blake3_digest: "single_fail_digest".to_string(),
            });
        }
        Ok(SupervisedExecutionOutcome {
            exit_code: 0,
            duration_ms: 30,
            stdout_tail: "test result: ok".to_string(),
            stderr_tail: "".to_string(),
            output_blake3_digest: "pass_digest".to_string(),
        })
    }
}

#[test]
fn test_wave_batch_command_planner_groups_per_crate_and_preserves_traceability() {
    let specs = vec![
        TaskCommandSpec {
            task_id: "TASK-001".to_string(),
            crate_name: "trustkit-core".to_string(),
            wave_index: 1,
            request: SupervisedExecutionRequest {
                command: "cargo".to_string(),
                args: vec![
                    "test".to_string(),
                    "-p".to_string(),
                    "trustkit-core".to_string(),
                    "--test".to_string(),
                    "policy_a_tests".to_string(),
                ],
                working_dir: ".".to_string(),
                timeout_ms: 60_000,
            },
        },
        TaskCommandSpec {
            task_id: "TASK-002".to_string(),
            crate_name: "trustkit-core".to_string(),
            wave_index: 1,
            request: SupervisedExecutionRequest {
                command: "cargo".to_string(),
                args: vec![
                    "test".to_string(),
                    "-p".to_string(),
                    "trustkit-core".to_string(),
                    "--test".to_string(),
                    "policy_b_tests".to_string(),
                ],
                working_dir: ".".to_string(),
                timeout_ms: 60_000,
            },
        },
        TaskCommandSpec {
            task_id: "TASK-003".to_string(),
            crate_name: "trustkit-orch".to_string(),
            wave_index: 1,
            request: SupervisedExecutionRequest {
                command: "cargo".to_string(),
                args: vec![
                    "test".to_string(),
                    "-p".to_string(),
                    "trustkit-orch".to_string(),
                    "--test".to_string(),
                    "orch_tests".to_string(),
                ],
                working_dir: ".".to_string(),
                timeout_ms: 60_000,
            },
        },
    ];

    let config = BatchExecutionConfig::new(true, BatchGroupingMode::PerCrate, 4, true).unwrap();
    let batches = WaveBatchCommandPlanner::plan_batches(&specs, &config);

    assert_eq!(
        batches.len(),
        2,
        "Three tasks across two crates must be grouped into 2 crate batches"
    );
    assert_eq!(
        batches[0].task_ids,
        vec!["TASK-001".to_string(), "TASK-002".to_string()]
    );
    assert!(
        batches[0]
            .combined_request
            .args
            .contains(&"policy_a_tests".to_string())
            && batches[0]
                .combined_request
                .args
                .contains(&"policy_b_tests".to_string()),
        "Combined request must include both test targets"
    );

    let supervisor = RecordingSupervisor::new(false, "");
    let outcomes = WaveBatchCommandPlanner::execute_batches(&batches, &supervisor)
        .expect("batched execution succeeds");

    assert_eq!(
        outcomes.len(),
        3,
        "Must return individual outcome entry for each of the 3 tasks"
    );
    assert_eq!(
        supervisor.calls.lock().unwrap().len(),
        2,
        "Supervisor must only be invoked 2 times instead of 3"
    );
    assert!(
        outcomes[0].1.stdout_tail.contains("BATCH_EXEC=2"),
        "Batched outcome must record BATCH_EXEC marker for HarnessRetroEngine"
    );
}

#[test]
fn test_wave_batch_command_planner_fallback_isolates_failing_task() {
    let specs = vec![
        TaskCommandSpec {
            task_id: "TASK-001".to_string(),
            crate_name: "trustkit-core".to_string(),
            wave_index: 1,
            request: SupervisedExecutionRequest {
                command: "cargo".to_string(),
                args: vec![
                    "test".to_string(),
                    "-p".to_string(),
                    "trustkit-core".to_string(),
                    "--test".to_string(),
                    "good_test".to_string(),
                ],
                working_dir: ".".to_string(),
                timeout_ms: 60_000,
            },
        },
        TaskCommandSpec {
            task_id: "TASK-002".to_string(),
            crate_name: "trustkit-core".to_string(),
            wave_index: 1,
            request: SupervisedExecutionRequest {
                command: "cargo".to_string(),
                args: vec![
                    "test".to_string(),
                    "-p".to_string(),
                    "trustkit-core".to_string(),
                    "--test".to_string(),
                    "bad_test".to_string(),
                ],
                working_dir: ".".to_string(),
                timeout_ms: 60_000,
            },
        },
    ];

    let config = BatchExecutionConfig::default();
    let batches = WaveBatchCommandPlanner::plan_batches(&specs, &config);
    let supervisor = RecordingSupervisor::new(true, "bad_test");
    let outcomes =
        WaveBatchCommandPlanner::execute_batches(&batches, &supervisor).expect("fallback works");

    assert_eq!(outcomes.len(), 2);
    assert_eq!(outcomes[0].0, "TASK-001");
    assert_eq!(outcomes[0].1.exit_code, 0);
    assert_eq!(outcomes[1].0, "TASK-002");
    assert_eq!(outcomes[1].1.exit_code, 101);
}

#[test]
fn test_ast_wave_analysis_cache_reuses_unmodified_files_across_waves() {
    let mut cache = AstWaveAnalysisCache::new(ToolCachingPolicy::default());
    let ast_call_count = AtomicUsize::new(0);
    let coupling_call_count = AtomicUsize::new(0);

    let file_path = "crates/trustkit-core/src/lib.rs";
    let content_v1 = "pub mod entities;\npub fn clean() {}\n";

    // Wave 1: initial AST analysis and coupling extraction (cache miss)
    let verdict_w1 = cache
        .get_or_analyze_ast(file_path, content_v1, || {
            ast_call_count.fetch_add(1, Ordering::SeqCst);
            Ok(AstAuditVerdict::CleanPass { violations: vec![] })
        })
        .expect("wave 1 ast");
    assert!(verdict_w1.is_pass());

    let (_nodes_w1, edges_w1) = cache
        .get_or_extract_structural_coupling(file_path, content_v1, || {
            coupling_call_count.fetch_add(1, Ordering::SeqCst);
            Ok((
                vec![StructuralNode::new(
                    file_path,
                    "lib.rs",
                    GraphNodeKind::CodeFile,
                    file_path,
                    "CORE",
                )],
                vec![StructuralEdge::new(
                    file_path,
                    "crates/trustkit-core/src/entities.rs",
                    EdgeRelationKind::Imports,
                    EdgeConfidence::Extracted,
                    file_path,
                )],
            ))
        })
        .expect("wave 1 coupling");
    assert_eq!(edges_w1.len(), 1);
    assert_eq!(cache.cache_misses(), 2);
    assert_eq!(cache.cache_hits(), 0);

    // Wave 2: unmodified file (cache hit!)
    let verdict_w2 = cache
        .get_or_analyze_ast(file_path, content_v1, || {
            ast_call_count.fetch_add(1, Ordering::SeqCst);
            Ok(AstAuditVerdict::CleanPass { violations: vec![] })
        })
        .expect("wave 2 ast");
    assert!(verdict_w2.is_pass());

    let (_nodes_w2, edges_w2) = cache
        .get_or_extract_structural_coupling(file_path, content_v1, || {
            coupling_call_count.fetch_add(1, Ordering::SeqCst);
            Ok((vec![], vec![]))
        })
        .expect("wave 2 coupling");
    assert_eq!(edges_w2.len(), 1);
    assert_eq!(ast_call_count.load(Ordering::SeqCst), 1);
    assert_eq!(coupling_call_count.load(Ordering::SeqCst), 1);
    assert_eq!(cache.cache_hits(), 2);
    assert_eq!(cache.reuse_rate_pct(), 50);

    // Wave 3: modified file (cache invalidation -> miss)
    let content_v2 = "pub mod entities;\npub fn updated() {}\n";
    let _ = cache
        .get_or_analyze_ast(file_path, content_v2, || {
            ast_call_count.fetch_add(1, Ordering::SeqCst);
            Ok(AstAuditVerdict::CleanPass { violations: vec![] })
        })
        .expect("wave 3 ast");
    assert_eq!(ast_call_count.load(Ordering::SeqCst), 2);
    assert_eq!(cache.cache_misses(), 3);
}
