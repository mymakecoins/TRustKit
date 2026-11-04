// Handler for `trustkit hook pre-tool`.
// Constitutional Principle IV: Resilient Error Handling & Zero-Panic Policy.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::cli_contract::PreToolArgs;
use crate::error_handler::DomainOutcome;
use crate::presenter::Presenter;
use trustkit_core::errors::DomainViolation;
use trustkit_core::use_cases::inspect_pre_tool_command::{self, InspectPreToolCommandRequest};

pub fn execute_pre_tool(
    args: &PreToolArgs,
    json_mode: bool,
) -> Result<DomainOutcome, DomainViolation> {
    let req = InspectPreToolCommandRequest {
        command: args.command.clone(),
    };

    let outcome = inspect_pre_tool_command::execute(req)?;
    let rendered = Presenter::render_hook_pre_tool(&outcome.evaluated_command, json_mode);
    println!("{rendered}");

    Ok(DomainOutcome::Success)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_pre_tool_safe_command() {
        let args = PreToolArgs {
            command: "cargo test".to_string(),
            json: true,
        };
        let res = execute_pre_tool(&args, true);
        assert!(res.is_ok());
    }

    #[test]
    fn test_execute_pre_tool_blocked_command() {
        let args = PreToolArgs {
            command: "git stash".to_string(),
            json: true,
        };
        let res = execute_pre_tool(&args, true);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().posix_exit_code(), 4);
    }
}
