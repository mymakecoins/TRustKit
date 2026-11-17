// TRustKit CLI Main Entrypoint.
// Constitutional Principle IV: Resilient Error Handling & Zero-Panic Policy.
// Constitutional Principle V: Separation of Construction and Execution.

use clap::{CommandFactory, FromArgMatches};
use std::process::ExitCode;
use trustkit_cli::bootstrap::dispatch_cli;
use trustkit_cli::cli_contract::Cli;
use trustkit_cli::error_handler::CliBoundaryErrorHandler;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let json_mode =
        args.iter().any(|arg| arg == "--json") || std::env::var("TRUSTKIT_AGENT_MODE").is_ok();

    let bin_name = args
        .first()
        .and_then(|a| std::path::Path::new(a).file_name())
        .and_then(|f| f.to_str())
        .unwrap_or("trustkit");

    let is_tk = bin_name == "tk" || bin_name.starts_with("tk");
    let mut cmd = Cli::command();
    if is_tk {
        cmd = cmd.name("tk").bin_name("tk");
    }

    let parse_result = cmd
        .try_get_matches_from_mut(&args)
        .and_then(|matches| Cli::from_arg_matches(&matches));

    match parse_result {
        Ok(cli) => {
            let result = dispatch_cli(&cli);
            let exit_code = CliBoundaryErrorHandler::handle_result(result, cli.is_json_mode());
            ExitCode::from(exit_code as u8)
        }
        Err(clap_err) => {
            if !clap_err.use_stderr() {
                print!("{clap_err}");
                ExitCode::from(0)
            } else if json_mode {
                let code = clap_err.exit_code();
                eprintln!(
                    "{{\"verdict\":\"FAIL\",\"exit_code\":{code},\"violation\":{{\"kind\":\"CLI_PARSE_ERROR\",\"reason\":\"{}\"}}}}",
                    clap_err.to_string().replace('"', "\\\"").replace('\n', " ")
                );
                ExitCode::from(code as u8)
            } else {
                eprint!("{clap_err}");
                ExitCode::from(clap_err.exit_code() as u8)
            }
        }
    }
}
