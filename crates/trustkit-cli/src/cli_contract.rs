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
            Command::Spec(SpecArgs { action }) => match action {
                SpecSubcommand::Init(args) => args.json,
                SpecSubcommand::Lock(args) => args.json,
            },
            Command::Wave(WaveArgs { action }) => match action {
                WaveSubcommand::Plan(args) => args.json,
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

    #[command(about = "Manage and govern Specification V2 lifecycle (Gates 1 & 2)")]
    Spec(SpecArgs),

    #[command(about = "Plan and organize execution waves from atomic tasks (Gate 3)")]
    Wave(WaveArgs),
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
    #[command(
        name = "pre-tool",
        about = "Intercept and evaluate shell commands before execution"
    )]
    PreTool(PreToolArgs),
}

#[derive(Args, Debug, PartialEq, Eq)]
pub struct PreToolArgs {
    #[arg(long, help = "Shell command string to evaluate")]
    pub command: String,

    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}

#[derive(Args, Debug, PartialEq, Eq)]
pub struct SpecArgs {
    #[command(subcommand)]
    pub action: SpecSubcommand,
}

#[derive(Subcommand, Debug, PartialEq, Eq)]
pub enum SpecSubcommand {
    #[command(
        name = "init",
        about = "Initialize a Spec V2 skeleton from upstream PRD or ADR"
    )]
    Init(SpecInitArgs),

    #[command(
        name = "lock",
        about = "Validate and cryptographically seal specification at Gate 2"
    )]
    Lock(SpecLockArgs),
}

#[derive(Args, Debug, PartialEq, Eq, Clone, Default)]
pub struct SpecLockArgs {
    #[arg(
        long,
        help = "Path to the specification file (defaults to active cycle spec)"
    )]
    pub spec: Option<String>,

    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}

#[derive(Args, Debug, PartialEq, Eq, Clone, Default)]
pub struct SpecInitArgs {
    #[arg(long, help = "Path to upstream PRD or ADR Markdown file")]
    pub from_prd: String,

    #[arg(long, help = "Custom name/slug for the feature specification")]
    pub name: Option<String>,

    #[arg(long, help = "Output directory for the generated specification file")]
    pub output_dir: Option<String>,

    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}

#[derive(Args, Debug, PartialEq, Eq)]
pub struct WaveArgs {
    #[command(subcommand)]
    pub action: WaveSubcommand,
}

#[derive(Subcommand, Debug, PartialEq, Eq)]
pub enum WaveSubcommand {
    #[command(
        name = "plan",
        about = "Construct DAG and plan topological execution waves"
    )]
    Plan(WavePlanArgs),
}

#[derive(Args, Debug, PartialEq, Eq, Clone, Default)]
pub struct WavePlanArgs {
    #[arg(
        long,
        help = "Path to directory containing atomic task definitions (TASK-NNN.md)"
    )]
    pub tasks_dir: Option<String>,

    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}

#[cfg(test)]
#[path = "cli_contract_tests.rs"]
mod tests;
