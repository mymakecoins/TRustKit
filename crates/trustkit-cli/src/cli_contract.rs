// Clap v4 CLI command and flag definitions.
// Constitutional Principle V: Separation of Construction and Execution.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug, PartialEq, Eq)]
#[command(
    name = "trustkit",
    author = "TRustKit Contributors",
    version,
    about = "Deterministic SDD Lifecycle Foundation, Fast Hooks & State Persistence",
    long_about = None
)]
pub struct Cli {
    #[arg(long, global = true, help = "Emit deterministic null-free JSON output")]
    pub json: bool,

    #[command(subcommand)]
    pub command: Command,
}

impl Cli {
    pub fn is_json_mode(&self) -> bool {
        if self.json || std::env::var("TRUSTKIT_AGENT_MODE").is_ok() {
            return true;
        }

        match &self.command {
            Command::Init(args) => args.json,
            Command::Status(args) => args.json,
            Command::Hook(HookArgs { action }) => match action {
                HookSubcommand::PreTool(args) => args.json,
            },
        }
    }
}

#[derive(Subcommand, Debug, PartialEq, Eq)]
pub enum Command {
    #[command(about = "Initialize or resume an SDD cycle at Gate 0")]
    Init(InitArgs),

    #[command(about = "Query active cycle status and health diagnostics")]
    Status(StatusArgs),

    #[command(about = "Hook lifecycle commands for pre/post tool execution")]
    Hook(HookArgs),
}

#[derive(Args, Debug, PartialEq, Eq, Default)]
pub struct InitArgs {
    #[arg(long, help = "GitHub issue number or URL")]
    pub issue: Option<String>,

    #[arg(long, help = "Path to source PRD or requirements specification")]
    pub from_prd: Option<String>,

    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}

#[derive(Args, Debug, PartialEq, Eq, Default)]
pub struct StatusArgs {
    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}

#[derive(Args, Debug, PartialEq, Eq)]
pub struct HookArgs {
    #[command(subcommand)]
    pub action: HookSubcommand,
}

#[derive(Subcommand, Debug, PartialEq, Eq)]
pub enum HookSubcommand {
    #[command(name = "pre-tool", about = "Intercept and evaluate shell commands before execution")]
    PreTool(PreToolArgs),
}

#[derive(Args, Debug, PartialEq, Eq)]
pub struct PreToolArgs {
    #[arg(long, help = "Shell command string to evaluate")]
    pub command: String,

    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_init_with_issue() {
        let args = ["trustkit", "init", "--issue", "45", "--json"];
        let cli = Cli::try_parse_from(args).expect("parse should succeed");
        assert!(cli.is_json_mode());
        assert_eq!(
            cli.command,
            Command::Init(InitArgs {
                issue: Some("45".to_string()),
                from_prd: None,
                json: true,
            })
        );
    }

    #[test]
    fn test_parse_status() {
        let args = ["trustkit", "status"];
        let cli = Cli::try_parse_from(args).expect("parse should succeed");
        assert!(!cli.is_json_mode());
        assert_eq!(cli.command, Command::Status(StatusArgs { json: false }));
    }

    #[test]
    fn test_parse_hook_pre_tool() {
        let args = [
            "trustkit",
            "hook",
            "pre-tool",
            "--command",
            "git add .",
            "--json",
        ];
        let cli = Cli::try_parse_from(args).expect("parse should succeed");
        assert!(cli.is_json_mode());
        match cli.command {
            Command::Hook(HookArgs {
                action: HookSubcommand::PreTool(pre_tool),
            }) => {
                assert_eq!(pre_tool.command, "git add .");
                assert!(pre_tool.json);
            }
            _ => panic!("Expected hook pre-tool command"),
        }
    }
}
