// Clap v4 CLI command and flag definitions.
// Constitutional Principle V: Separation of Construction and Execution.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use clap::{Args, Parser, Subcommand};
use trustkit_core::errors::DomainViolation;

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
            Command::Board(args) => match &args.action {
                BoardSubcommand::Up(a) => a.json,
                BoardSubcommand::Down(a) => a.json,
                BoardSubcommand::Status(a) => a.json,
                BoardSubcommand::Serve(_) => false,
            },
            Command::Wave(WaveArgs { action }) => match action {
                WaveSubcommand::Plan(args) => args.json,
                WaveSubcommand::Run(args) => args.json,
                WaveSubcommand::Board(args) => match &args.action {
                    BoardSubcommand::Up(a) => a.json,
                    BoardSubcommand::Down(a) => a.json,
                    BoardSubcommand::Status(a) => a.json,
                    BoardSubcommand::Serve(_) => false,
                },
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
            Command::Close(args) => args.json,
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
                MemorySubcommand::Glossary(g) => match &g.action {
                    MemoryGlossarySubcommand::Add(a) => a.json,
                    MemoryGlossarySubcommand::List(a) => a.json,
                },
                MemorySubcommand::AdrCheck(a) => a.json,
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
            Command::Auto(args) => args.json,
            Command::Update(args) => args.json,
            Command::Milestones(MilestonesArgs { action }) => match action {
                MilestonesSubcommand::Plan(args) => args.json,
                MilestonesSubcommand::List(args) => args.json,
                MilestonesSubcommand::Inspect(args) => args.json,
            },
        }
    }
}

#[derive(Subcommand, Debug, PartialEq, Eq)]
pub enum Command {
    #[command(about = "Execute full autonomous SDD lifecycle from specification or issue")]
    Auto(AutoArgs),

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

    #[command(about = "Manage real-time wave kanban board background daemon")]
    Board(BoardArgs),

    #[command(about = "Execute atomic tasks under strict TDD phase verification (Gate 4)")]
    Task(TaskArgs),

    #[command(about = "Run Gauntlet verification layers (Gate 5)")]
    Gauntlet(GauntletArgs),

    #[command(about = "Diagnose cycle state and circuit breaker failure summaries")]
    Doctor(DoctorArgs),

    #[command(about = "Certify completed cycle, compile gauntlet report, and format issue comment")]
    Done(DoneArgs),

    #[command(about = "Close active SDD cycle and transition issue to ReadyForTest")]
    Close(CloseArgs),

    #[command(
        about = "Perform deterministic merge into target branch at Gate 6 (DEPRECATED: use 'trustkit close')"
    )]
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

    #[command(
        visible_alias = "milestone",
        about = "Plan, list and inspect project milestones"
    )]
    Milestones(MilestonesArgs),

    #[command(
        about = "Check, build and self-update trustkit and tk binaries from remote main branch"
    )]
    Update(UpdateArgs),
}

#[derive(Args, Debug, PartialEq, Eq, Default)]
pub struct InitArgs {
    #[arg(long, help = "GitHub issue number or URL")]
    pub issue: Option<String>,

    #[arg(long, help = "Path to source PRD or requirements specification")]
    pub from_prd: Option<String>,

    #[arg(long, help = "Path to milestone markdown file for targeted SDD cycle")]
    pub milestone: Option<String>,

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
    #[command(
        name = "board",
        about = "Manage real-time wave kanban board background daemon"
    )]
    Board(WaveBoardArgs),
}

#[derive(Args, Debug, PartialEq, Eq, Clone)]
pub struct BoardArgs {
    #[command(subcommand)]
    pub action: BoardSubcommand,
}

#[derive(Args, Debug, PartialEq, Eq, Clone)]
pub struct WaveBoardArgs {
    #[command(subcommand)]
    pub action: BoardSubcommand,
}

#[derive(Subcommand, Debug, PartialEq, Eq, Clone)]
pub enum BoardSubcommand {
    #[command(name = "up", about = "Start the real-time wave kanban board daemon")]
    Up(BoardUpArgs),
    #[command(name = "down", about = "Stop the real-time wave kanban board daemon")]
    Down(BoardDownArgs),
    #[command(
        name = "status",
        about = "Inspect status of the wave kanban board daemon"
    )]
    Status(BoardStatusArgs),
    #[command(name = "serve", about = "Run local HTTP/SSE board server", hide = true)]
    Serve(BoardServeArgs),
}

#[derive(Args, Debug, PartialEq, Eq, Clone)]
pub struct BoardUpArgs {
    #[arg(long, help = "Automatically open the board URL in the default browser")]
    pub open: bool,
    #[arg(long, help = "Emit deterministic null-free JSON output")]
    pub json: bool,
}

#[derive(Args, Debug, PartialEq, Eq, Clone)]
pub struct BoardDownArgs {
    #[arg(long, help = "Emit deterministic null-free JSON output")]
    pub json: bool,
}

#[derive(Args, Debug, PartialEq, Eq, Clone)]
pub struct BoardStatusArgs {
    #[arg(long, help = "Emit deterministic null-free JSON output")]
    pub json: bool,
}

#[derive(Args, Debug, PartialEq, Eq, Clone)]
pub struct BoardServeArgs {
    #[arg(long, help = "Port to listen on")]
    pub port: u16,
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
        visible_alias = "audit",
        about = "Execute Layer 0 syntactic AST invariant and anti-cheat audit"
    )]
    AstAudit(GauntletAstAuditArgs),
    #[command(
        name = "run",
        about = "Run concurrent verification gauntlet across Layer 0 and Layer 1 critics"
    )]
    Run(GauntletRunArgs),
}

#[derive(Args, Debug, PartialEq, Eq, Default, Clone)]
pub struct AutoArgs {
    #[arg(help = "Optional positional target: GitHub issue number/link or path to spec")]
    pub target_ref: Option<String>,

    #[arg(long, help = "GitHub issue number, link or ID")]
    pub issue: Option<String>,

    #[arg(long, help = "Path to local specification markdown file")]
    pub spec: Option<String>,

    #[arg(
        long,
        default_value = "develop",
        help = "Target branch for deterministic merge"
    )]
    pub target: String,

    #[arg(
        long,
        default_value = "auto",
        help = "Runtime execution backend (auto, pty, native)"
    )]
    pub runtime: String,

    #[arg(
        long,
        default_value = "",
        help = "Optional builder agent command to execute"
    )]
    pub agent_cmd: String,

    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}

impl AutoArgs {
    pub fn resolved_issue(&self) -> Option<String> {
        if let Some(ref iss) = self.issue {
            return Some(iss.clone());
        }
        if let Some(ref t) = self.target_ref {
            if !t.ends_with(".md") && !std::path::Path::new(t).is_file() {
                return Some(t.clone());
            }
        }
        None
    }

    pub fn resolved_spec(&self) -> Option<String> {
        if let Some(ref s) = self.spec {
            return Some(s.clone());
        }
        if let Some(ref t) = self.target_ref {
            if t.ends_with(".md") || std::path::Path::new(t).is_file() {
                return Some(t.clone());
            }
        }
        None
    }

    pub fn validate(&self) -> Result<(), DomainViolation> {
        if self.resolved_issue().is_none() && self.resolved_spec().is_none() {
            return Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "auto_cli".to_string(),
                details: "Either --issue or --spec must be provided for trustkit auto".to_string(),
            });
        }
        Ok(())
    }
}

#[derive(Args, Debug, PartialEq, Eq)]
pub struct MilestonesArgs {
    #[command(subcommand)]
    pub action: MilestonesSubcommand,
}

#[derive(Subcommand, Debug, PartialEq, Eq)]
pub enum MilestonesSubcommand {
    #[command(name = "plan", about = "Plan milestones DAG from PRD and ADR files")]
    Plan(MilestonesPlanArgs),

    #[command(name = "list", about = "List planned milestones")]
    List(MilestonesListArgs),

    #[command(name = "inspect", about = "Inspect milestone details and criteria")]
    Inspect(MilestonesInspectArgs),
}

#[derive(Args, Debug, PartialEq, Eq, Default)]
pub struct MilestonesPlanArgs {
    #[arg(long, help = "Path to PRD markdown file")]
    pub prd: String,

    #[arg(long, help = "Path to ADR markdown file")]
    pub adr: String,

    #[arg(
        long,
        default_value = "docs/milestones",
        help = "Output directory for milestone markdown files"
    )]
    pub output_dir: String,

    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}

#[derive(Args, Debug, PartialEq, Eq, Default)]
pub struct MilestonesListArgs {
    #[arg(long, help = "Optional plan ID to filter milestones")]
    pub plan_id: Option<String>,

    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}

#[derive(Args, Debug, PartialEq, Eq, Default)]
pub struct MilestonesInspectArgs {
    #[arg(help = "Milestone identifier (e.g. M01)")]
    pub milestone_id: String,

    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}

#[derive(Args, Debug, PartialEq, Eq, Default, Clone)]
pub struct UpdateArgs {
    #[arg(
        long,
        help = "Only check if newer commits exist on remote main without applying update"
    )]
    pub check: bool,

    #[arg(
        long,
        help = "Force rebuild and reinstallation even if local commit matches remote main"
    )]
    pub force: bool,

    #[arg(long, help = "Emit deterministic null-free JSON output")]
    pub json: bool,
}

#[cfg(test)]
#[path = "cli_contract_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "cli_contract_help_tests.rs"]
mod help_tests;
