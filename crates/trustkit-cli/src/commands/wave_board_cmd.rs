// Handler for `trustkit board` and `trustkit wave board` (TASK-003).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::cli_contract::{
    BoardDownArgs, BoardServeArgs, BoardStatusArgs, BoardSubcommand, BoardUpArgs,
};
use crate::commands::init_cmd::current_timestamps;
use crate::error_handler::DomainOutcome;
use crate::wave_board_cli_commands::{WaveBoardAction, WaveBoardCliResponse};
use std::fs;
use std::path::Path;
use trustkit_core::errors::DomainViolation;
use trustkit_db::sqlite_adapter::SqliteWalRepositoryAdapter;
use trustkit_orch::wave_board_lifecycle_shutdown::WaveBoardLifecycleManager;
use trustkit_orch::wave_board_port_allocator::{WaveBoardMetadata, WaveBoardPortAllocator};
use trustkit_orch::wave_board_server_sse::WaveBoardServerSse;

pub fn execute(
    subcommand: &BoardSubcommand,
    json_mode: bool,
) -> Result<DomainOutcome, DomainViolation> {
    match subcommand {
        BoardSubcommand::Up(args) => execute_up(args, json_mode),
        BoardSubcommand::Down(args) => execute_down(args, json_mode),
        BoardSubcommand::Status(args) => execute_status(args, json_mode),
        BoardSubcommand::Serve(args) => execute_serve(args),
    }
}

pub fn execute_up(args: &BoardUpArgs, json_mode: bool) -> Result<DomainOutcome, DomainViolation> {
    let meta_path = Path::new(".trustkit").join("wave-board.json");
    let (now_iso, _) = current_timestamps();

    let port = WaveBoardPortAllocator::allocate_free_port().map_err(|e| {
        DomainViolation::InfrastructureBoundaryFault {
            subsystem: "wave_board".to_string(),
            details: format!("Failed allocating ephemeral port: {e}"),
        }
    })?;

    let exe =
        std::env::current_exe().map_err(|e| DomainViolation::InfrastructureBoundaryFault {
            subsystem: "wave_board".to_string(),
            details: format!("Failed determining current binary path: {e}"),
        })?;

    let log_file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(".trustkit/wave-board.log")
        .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
            subsystem: "wave_board".to_string(),
            details: format!("Failed opening log file: {e}"),
        })?;
    let err_file =
        log_file
            .try_clone()
            .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
                subsystem: "wave_board".to_string(),
                details: format!("Failed cloning log file handle: {e}"),
            })?;

    let mut cmd = std::process::Command::new(exe);
    cmd.args(["board", "serve", "--port", &port.to_string()])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::from(log_file))
        .stderr(std::process::Stdio::from(err_file));

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }

    let child = cmd
        .spawn()
        .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
            subsystem: "wave_board".to_string(),
            details: format!("Failed spawning wave board background daemon: {e}"),
        })?;

    let pid = child.id();
    let meta = WaveBoardMetadata::new(pid, port, now_iso);
    meta.save_to_file(&meta_path)
        .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
            subsystem: "wave_board".to_string(),
            details: format!("Failed saving wave board metadata: {e}"),
        })?;

    if args.open {
        let _ = std::process::Command::new("xdg-open")
            .arg(&meta.url)
            .spawn();
    }

    let next_steps = vec![
        format!("Access kanban dashboard at {}", meta.url),
        "Run 'tk board status' to check daemon health".to_string(),
        "Run 'tk board down' to stop the background daemon".to_string(),
    ];

    let response = WaveBoardCliResponse::new(
        WaveBoardAction::Up,
        pid,
        port,
        &meta.url,
        "Wave board daemon started successfully",
        next_steps,
    );

    if json_mode || args.json {
        println!("{}", response.to_json_output());
    } else {
        println!("{}", response.to_tty_output());
    }

    Ok(DomainOutcome::Success)
}

pub fn execute_down(
    args: &BoardDownArgs,
    json_mode: bool,
) -> Result<DomainOutcome, DomainViolation> {
    let meta_path = Path::new(".trustkit").join("wave-board.json");

    if !meta_path.exists() {
        let response = WaveBoardCliResponse::new(
            WaveBoardAction::Down,
            0,
            0,
            "",
            "Wave board daemon is not running (no active session)",
            vec!["Run 'tk board up' to launch a new wave board session".to_string()],
        );
        if json_mode || args.json {
            println!("{}", response.to_json_output());
        } else {
            println!("{}", response.to_tty_output());
        }
        return Ok(DomainOutcome::Success);
    }

    let meta = WaveBoardMetadata::load_from_file(&meta_path)
        .unwrap_or_else(|_| WaveBoardMetadata::new(0, 0, String::new()));

    WaveBoardLifecycleManager::graceful_shutdown(&meta_path).map_err(|e| {
        DomainViolation::InfrastructureBoundaryFault {
            subsystem: "wave_board".to_string(),
            details: format!("Failed shutting down wave board daemon: {e}"),
        }
    })?;

    let response = WaveBoardCliResponse::new(
        WaveBoardAction::Down,
        meta.pid,
        meta.port,
        &meta.url,
        "Wave board daemon stopped successfully",
        vec!["Run 'tk board up' to launch a new wave board session".to_string()],
    );

    if json_mode || args.json {
        println!("{}", response.to_json_output());
    } else {
        println!("{}", response.to_tty_output());
    }

    Ok(DomainOutcome::Success)
}

pub fn execute_status(
    args: &BoardStatusArgs,
    json_mode: bool,
) -> Result<DomainOutcome, DomainViolation> {
    let meta_path = Path::new(".trustkit").join("wave-board.json");
    let report = WaveBoardLifecycleManager::inspect_status(&meta_path);

    let (pid, port, url, message, next_steps) = if report.is_running {
        (
            report.pid.unwrap_or(0),
            report.port.unwrap_or(0),
            report.url.unwrap_or_default(),
            format!(
                "Wave board daemon is active (uptime: {}s)",
                report.uptime_seconds
            ),
            vec![
                "Open the dashboard URL in your browser".to_string(),
                "Run 'tk board down' to terminate the session".to_string(),
            ],
        )
    } else {
        (
            0,
            0,
            String::new(),
            "Wave board daemon is stopped".to_string(),
            vec!["Run 'tk board up' to start the wave board daemon".to_string()],
        )
    };

    let response =
        WaveBoardCliResponse::new(WaveBoardAction::Status, pid, port, url, message, next_steps);

    if json_mode || args.json {
        println!("{}", response.to_json_output());
    } else {
        println!("{}", response.to_tty_output());
    }

    Ok(DomainOutcome::Success)
}

pub fn execute_serve(args: &BoardServeArgs) -> Result<DomainOutcome, DomainViolation> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
            subsystem: "tokio".to_string(),
            details: format!("Failed creating tokio runtime: {e}"),
        })?;

    runtime.block_on(async move {
        let html_path = Path::new("docs/prototypes/trustkit-kanban-dashboard.html");
        let html = if html_path.exists() {
            fs::read_to_string(html_path)
                .unwrap_or_else(|_| "<h1>TRustKit Wave Board</h1>".to_string())
        } else {
            "<h1>TRustKit Wave Board</h1>".to_string()
        };

        let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", args.port))
            .await
            .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
                subsystem: "wave_board".to_string(),
                details: format!("Failed binding to port {}: {e}", args.port),
            })?;

        let mut sse_server = WaveBoardServerSse::new(html);
        let db_path = Path::new(".trustkit").join("state.db");
        if let Ok(repo) = SqliteWalRepositoryAdapter::open(&db_path) {
            sse_server = sse_server.with_repository(std::sync::Arc::new(repo));
        }
        let server = std::sync::Arc::new(sse_server);
        server
            .serve(listener)
            .await
            .map_err(|e| DomainViolation::InfrastructureBoundaryFault {
                subsystem: "wave_board".to_string(),
                details: format!("Wave board server loop error: {e}"),
            })
    })?;

    Ok(DomainOutcome::Success)
}
