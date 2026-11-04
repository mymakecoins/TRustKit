use super::*;
use crate::cli_contract::{
    DoctorArgs, DoneArgs, InitArgs, MergeArgs, PrArgs, PreToolArgs, SpecInitArgs, SpecLockArgs,
    StatusArgs, TaskArgs, TaskSubcommand, TaskTestArgs, TddPhaseArg, WavePlanArgs, WaveRunArgs,
};
use tempfile::tempdir;
use trustkit_core::ports::CycleEvidenceRepositoryPort;

#[test]
fn test_sync_bootstrap_wave_plan() {
    let cli = Cli {
        json: true,
        command: Command::Wave(WaveArgs {
            action: WaveSubcommand::Plan(WavePlanArgs {
                tasks_dir: Some("/non/existent/tasks".to_string()),
                json: true,
            }),
        }),
    };
    let res = FastSyncBootstrap::dispatch(&cli);
    assert!(res.is_err());
}

#[test]
fn test_sync_bootstrap_spec_lock() {
    let cli = Cli {
        json: true,
        command: Command::Spec(SpecArgs {
            action: SpecSubcommand::Lock(SpecLockArgs {
                spec: Some("/non/existent/spec.md".to_string()),
                json: true,
            }),
        }),
    };
    let res = FastSyncBootstrap::dispatch(&cli);
    assert!(res.is_err());
}

#[test]
fn test_sync_bootstrap_status() {
    let cli = Cli {
        json: false,
        command: Command::Status(StatusArgs { json: false }),
    };
    let res = FastSyncBootstrap::dispatch(&cli);
    assert!(res.is_ok());
}

#[test]
fn test_sync_bootstrap_spec_init() {
    let cli = Cli {
        json: true,
        command: Command::Spec(SpecArgs {
            action: SpecSubcommand::Init(SpecInitArgs {
                from_prd: "/non/existent/file.md".to_string(),
                name: None,
                output_dir: None,
                json: true,
            }),
        }),
    };
    let res = FastSyncBootstrap::dispatch(&cli);
    assert!(res.is_err());
}

#[test]
fn test_concurrent_bootstrap_init() {
    let cli = Cli {
        json: false,
        command: Command::Init(InitArgs {
            issue: None,
            from_prd: None,
            json: false,
        }),
    };
    let res = FullConcurrentBootstrap::dispatch(&cli);
    assert!(res.is_ok());
}

#[test]
fn test_sync_bootstrap_task_test_dispatches() {
    let cli = Cli {
        json: true,
        command: Command::Task(TaskArgs {
            action: TaskSubcommand::Test(TaskTestArgs {
                task_id: "TASK-001".to_string(),
                phase: TddPhaseArg::Green,
                json: true,
            }),
        }),
    };
    let res = FastSyncBootstrap::dispatch(&cli);
    assert!(res.is_err());
}

#[test]
fn test_sync_bootstrap_doctor_dispatches() {
    let cli = Cli {
        json: true,
        command: Command::Doctor(DoctorArgs { json: true }),
    };
    let res = FastSyncBootstrap::dispatch(&cli);
    let _ = res;
}

#[test]
fn test_sync_bootstrap_done_dispatches() {
    let cli = Cli {
        json: true,
        command: Command::Done(DoneArgs {
            json: true,
            ..Default::default()
        }),
    };
    let res = FastSyncBootstrap::dispatch(&cli);
    assert!(res.is_err());
}

#[test]
fn test_sync_bootstrap_merge_dispatches() {
    let cli = Cli {
        json: true,
        command: Command::Merge(MergeArgs {
            target: "develop".to_string(),
            json: true,
        }),
    };
    let res = FastSyncBootstrap::dispatch(&cli);
    assert!(res.is_err());
}

#[test]
fn test_sync_bootstrap_pr_dispatches() {
    let cli = Cli {
        json: true,
        command: Command::Pr(PrArgs {
            target: Some("develop".to_string()),
            json: true,
        }),
    };
    let res = FastSyncBootstrap::dispatch(&cli);
    assert!(res.is_err());
}

#[test]
fn test_full_concurrent_bootstrap_adapter_graph_assembly() {
    let dir = tempdir().expect("Failed to create temporary directory");
    let db_path = dir.path().join("state.db");

    let bootstrap = FullConcurrentBootstrap::new(dir.path(), &db_path);
    assert!(
        bootstrap.is_ok(),
        "FullConcurrentBootstrap adapter graph assembly must succeed"
    );

    let container = bootstrap.unwrap();
    // Validate all 6 concrete adapters are initialized and accessible
    assert!(container.repo.find_active_cycle().is_ok());
    assert!(
        !container
            .worktree_provider
            .repo_root()
            .as_os_str()
            .is_empty()
    );
    assert!(
        !container
            .visual_evidence_guard
            .repo_root()
            .as_os_str()
            .is_empty()
    );
    let _ = container.supervisor;
}

#[test]
fn test_dispatch_cli_fast_sync_path_without_tokio_runtime() {
    // Constitutional Requirement: hook pre-tool must not load tokio runtime
    assert!(
        tokio::runtime::Handle::try_current().is_err(),
        "Runtime must not be pre-loaded before command execution"
    );

    let cli = Cli {
        json: true,
        command: Command::Hook(HookArgs {
            action: HookSubcommand::PreTool(PreToolArgs {
                command: "cargo check".to_string(),
                json: true,
            }),
        }),
    };

    let res = dispatch_cli(&cli);
    assert!(res.is_ok(), "hook pre-tool must dispatch successfully");

    // After execution on FastSyncBootstrap, tokio runtime must still NOT exist on this thread
    assert!(
        tokio::runtime::Handle::try_current().is_err(),
        "FastSyncBootstrap must not allocate a tokio runtime"
    );
}

#[test]
fn test_dispatch_cli_routes_gates_4_5_6_to_full_concurrent() {
    let cli_wave_run = Cli {
        json: true,
        command: Command::Wave(WaveArgs {
            action: WaveSubcommand::Run(WaveRunArgs {
                wave: 1,
                runtime: "auto".to_string(),
                agent_cmd: "".to_string(),
                json: true,
            }),
        }),
    };
    let res = dispatch_cli(&cli_wave_run);
    assert!(res.is_err());

    let cli_doctor = Cli {
        json: true,
        command: Command::Doctor(DoctorArgs { json: true }),
    };
    let res = dispatch_cli(&cli_doctor);
    let _ = res;

    let cli_done = Cli {
        json: true,
        command: Command::Done(DoneArgs {
            json: true,
            ..Default::default()
        }),
    };
    let res = dispatch_cli(&cli_done);
    assert!(res.is_err());
}
