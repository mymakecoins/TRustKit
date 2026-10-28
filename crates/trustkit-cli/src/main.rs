// TRustKit CLI Main Entrypoint.
// Constitutional Principle IV: Resilient Error Handling & Zero-Panic Policy.
// Constitutional Principle V: Separation of Construction and Execution.

use clap::Parser;
use std::process::ExitCode;
use trustkit_cli::bootstrap::dispatch_cli;
use trustkit_cli::cli_contract::Cli;
use trustkit_cli::error_handler::CliBoundaryErrorHandler;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let json_mode =
        args.iter().any(|arg| arg == "--json") || std::env::var("TRUSTKIT_AGENT_MODE").is_ok();

    match Cli::try_parse() {
        Ok(cli) => {
            let result = dispatch_cli(&cli);
            let exit_code = CliBoundaryErrorHandler::handle_result(result, cli.is_json_mode());
            ExitCode::from(exit_code as u8)
        }
        Err(clap_err) => {
            if json_mode {
                eprintln!(
                    "{{\"verdict\":\"FAIL\",\"exit_code\":1,\"violation\":{{\"kind\":\"CLI_PARSE_ERROR\",\"reason\":\"{}\"}}}}",
                    clap_err.to_string().replace('"', "\\\"").replace('\n', " ")
                );
            } else {
                eprintln!("{clap_err}");
            }
            ExitCode::from(1)
        }
    }
}
