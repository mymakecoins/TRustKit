// Composition Root isolating object graph assembly and dependency injection.
// Constitutional Principle V: Separation of Construction and Execution.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::cli_contract::{
    Cli, Command, ContextArgs, ContextSubcommand, GauntletArgs, GauntletSubcommand, HookArgs,
    HookSubcommand, SpecArgs, SpecSubcommand, WaveArgs, WaveSubcommand,
};
use crate::commands::{
    context_slice_cmd, gauntlet_ast_audit_cmd, hook_cmd, init_cmd, spec_init_cmd, spec_lock_cmd,
    status_cmd, wave_plan_cmd,
};
use crate::error_handler::DomainOutcome;
use trustkit_core::errors::DomainViolation;

pub struct FastSyncBootstrap;

impl FastSyncBootstrap {
    pub fn dispatch(cli: &Cli) -> Result<DomainOutcome, DomainViolation> {
        let json_mode = cli.is_json_mode();
        match &cli.command {
            Command::Status(args) => status_cmd::execute(args, json_mode),
            Command::Hook(HookArgs { action }) => match action {
                HookSubcommand::PreTool(args) => hook_cmd::execute_pre_tool(args, json_mode),
            },
            Command::Spec(SpecArgs { action }) => match action {
                SpecSubcommand::Init(args) => spec_init_cmd::execute(args, json_mode),
                SpecSubcommand::Lock(args) => spec_lock_cmd::execute(args, json_mode),
            },
            Command::Wave(WaveArgs { action }) => match action {
                WaveSubcommand::Plan(args) => wave_plan_cmd::execute(args, json_mode),
            },
            Command::Gauntlet(GauntletArgs { action }) => match action {
                GauntletSubcommand::AstAudit(args) => {
                    gauntlet_ast_audit_cmd::execute(args, json_mode)
                }
            },
            Command::Context(ContextArgs { action }) => match action {
                ContextSubcommand::Slice(args) => context_slice_cmd::execute(args, json_mode),
            },
            Command::Init(_) => Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "bootstrap".to_string(),
                details: "Command 'init' requires FullConcurrentBootstrap path.".to_string(),
            }),
        }
    }
}

pub struct FullConcurrentBootstrap;

impl FullConcurrentBootstrap {
    pub fn dispatch(cli: &Cli) -> Result<DomainOutcome, DomainViolation> {
        let json_mode = cli.is_json_mode();
        match &cli.command {
            Command::Init(args) => {
                let runtime = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .map_err(|err| DomainViolation::InfrastructureBoundaryFault {
                        subsystem: "tokio_runtime".to_string(),
                        details: format!("Failed building async runtime: {err}"),
                    })?;

                runtime.block_on(async { init_cmd::execute(args, json_mode) })
            }
            _ => FastSyncBootstrap::dispatch(cli),
        }
    }
}

pub fn dispatch_cli(cli: &Cli) -> Result<DomainOutcome, DomainViolation> {
    match &cli.command {
        Command::Hook(_)
        | Command::Status(_)
        | Command::Spec(_)
        | Command::Wave(_)
        | Command::Gauntlet(_)
        | Command::Context(_) => FastSyncBootstrap::dispatch(cli),
        Command::Init(_) => FullConcurrentBootstrap::dispatch(cli),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli_contract::{InitArgs, SpecInitArgs, SpecLockArgs, StatusArgs, WavePlanArgs};

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
}
