// Composition Root isolating object graph assembly and dependency injection.
// Constitutional Principle V: Separation of Construction and Execution.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::cli_contract::{Cli, Command, HookArgs, HookSubcommand};
use crate::commands::{hook_cmd, init_cmd, status_cmd};
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
        Command::Hook(_) | Command::Status(_) => FastSyncBootstrap::dispatch(cli),
        Command::Init(_) => FullConcurrentBootstrap::dispatch(cli),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli_contract::{InitArgs, StatusArgs};

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
