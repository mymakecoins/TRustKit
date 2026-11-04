// Clap v4 CLI command and flag definitions.
// Constitutional Principle V: Separation of Construction and Execution.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use clap::{Args, Parser, Subcommand};

pub use crate::cli_gate_args::*;
pub use crate::cli_spec_args::*;

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
                SpecSubcommand::Prompt(args) => args.json,
                SpecSubcommand::Lint(args) => args.json,
                SpecSubcommand::Init(args) => args.json,
                SpecSubcommand::Lock(args) => args.json,
            },
            Command::Wave(WaveArgs { action }) => match action {
                WaveSubcommand::Plan(args) => args.json,
                WaveSubcommand::Run(args) => args.json,
            },
            Command::Gauntlet(GauntletArgs { action }) => match action {
                GauntletSubcommand::AstAudit(args) => args.json,
                GauntletSubcommand::Run(args) => args.json,
            },
            Command::Context(ContextArgs { action }) => match action {
                ContextSubcommand::Slice(args) => args.json,
            },
            Command::Task(TaskArgs { action }) => match action {
                TaskSubcommand::Test(args) => args.json,
                TaskSubcommand::Review(args) => args.json,
                TaskSubcommand::Complete(args) => args.json,
            },
            Command::Doctor(args) => args.json,
            Command::Done(args) => args.json,
            Command::Merge(args) => args.json,
            Command::Pr(args) => args.json,
            Command::Mcp(McpArgs { action }) => match action {
                McpSubcommand::Serve(args) => args.json,
            },
            Command::Memory(MemoryArgs { action }) => match action {
                MemorySubcommand::Write(a) => a.json,
                MemorySubcommand::Read(a) => a.json,
                MemorySubcommand::Search(a) => a.json,
                MemorySubcommand::List(a) => a.json,
                MemorySubcommand::Rebuild(a) => a.json,
            },
            Command::Graph(GraphArgs { action }) => match action {
                GraphSubcommand::Build(a) => a.json,
                GraphSubcommand::Health(a) => a.json,
                GraphSubcommand::Path(a) => a.json,
                GraphSubcommand::Explain(a) => a.json,
            },
            Command::Audit(AuditArgs { action }) => match action {
                AuditSubcommand::Security(a) => a.json,
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

    #[command(about = "Execute atomic tasks under strict TDD phase verification (Gate 4)")]
    Task(TaskArgs),

    #[command(about = "Run Gauntlet verification layers (Gate 5)")]
    Gauntlet(GauntletArgs),

    #[command(about = "Diagnose cycle state and circuit breaker failure summaries")]
    Doctor(DoctorArgs),

    #[command(about = "Certify completed cycle, compile gauntlet report, and format issue comment")]
    Done(DoneArgs),

    #[command(about = "Perform deterministic merge into target branch at Gate 6")]
    Merge(MergeArgs),

    #[command(about = "Open human-directed Pull Request to target branch")]
    Pr(PrArgs),

    #[command(about = "TRustKit Model Context Protocol (MCP) server lifecycle")]
    Mcp(McpArgs),

    #[command(about = "Context slicing utilities for builder agent prompts")]
    Context(ContextArgs),

    #[command(about = "Manage and query persistent project knowledge memory (.trustkit/wiki)")]
    Memory(MemoryArgs),

    #[command(about = "Structural graph intelligence, AST relationships and health checks")]
    Graph(GraphArgs),

    #[command(about = "Decoupled pre-release homologation and repository security audits")]
    Audit(AuditArgs),
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
        name = "prompt",
        about = "Generate structured spec prompt and skeleton for a feature idea"
    )]
    Prompt(SpecPromptArgs),
    #[command(
        name = "lint",
        about = "Validate specification markdown against structural and constitutional invariants"
    )]
    Lint(SpecLintArgs),
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
    #[command(
        name = "run",
        about = "Execute tasks in topological wave within isolated worktrees"
    )]
    Run(WaveRunArgs),
}

#[derive(Args, Debug, PartialEq, Eq)]
pub struct GauntletArgs {
    #[command(subcommand)]
    pub action: GauntletSubcommand,
}

#[derive(Subcommand, Debug, PartialEq, Eq)]
pub enum GauntletSubcommand {
    #[command(
        name = "ast-audit",
        about = "Execute Layer 0 syntactic AST invariant and anti-cheat audit"
    )]
    AstAudit(GauntletAstAuditArgs),
    #[command(
        name = "run",
        about = "Run concurrent verification gauntlet across Layer 0 and Layer 1 critics"
    )]
    Run(GauntletRunArgs),
}

#[cfg(test)]
#[path = "cli_contract_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "cli_contract_help_tests.rs"]
mod help_tests;
