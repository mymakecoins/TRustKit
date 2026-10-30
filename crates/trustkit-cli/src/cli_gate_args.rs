// Clap v4 CLI argument definitions for Gates 4, 5, and 6.
// Constitutional Principle V: Separation of Construction and Execution.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use clap::{Args, Subcommand, ValueEnum};
use serde::Serialize;
use trustkit_core::entities::{CliSecurityReviewFlag, CodeReviewEffortLevel};

#[derive(Args, Debug, PartialEq, Eq, Clone)]
pub struct TaskArgs {
    #[command(subcommand)]
    pub action: TaskSubcommand,
}

#[derive(Subcommand, Debug, PartialEq, Eq, Clone)]
pub enum TaskSubcommand {
    #[command(name = "test", about = "Execute test with TDD phase verification")]
    Test(TaskTestArgs),
    #[command(name = "review", about = "Run multi-angle semantic code review on task diff (Gate 4 Refactor)")]
    Review(TaskReviewArgs),
    #[command(name = "complete", about = "Complete atomic task after TDD verification")]
    Complete(TaskCompleteArgs),
}

#[derive(Args, Debug, PartialEq, Eq, Clone)]
pub struct TaskTestArgs {
    #[arg(help = "Atomic task identifier (e.g. TASK-001)")]
    pub task_id: String,

    #[arg(long, value_enum, help = "TDD cycle phase (red or green)")]
    pub phase: TddPhaseArg,

    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TddPhaseArg {
    #[value(name = "red")]
    Red,
    #[value(name = "green")]
    Green,
}

impl From<TddPhaseArg> for trustkit_core::gauntlet_entities::TddPhase {
    fn from(arg: TddPhaseArg) -> Self {
        match arg {
            TddPhaseArg::Red => Self::Red,
            TddPhaseArg::Green => Self::Green,
        }
    }
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CodeReviewEffortArg {
    #[value(name = "low")]
    Low,
    #[default]
    #[value(name = "medium")]
    Medium,
    #[value(name = "high")]
    High,
    #[value(name = "max")]
    Max,
}

impl From<CodeReviewEffortArg> for CodeReviewEffortLevel {
    fn from(arg: CodeReviewEffortArg) -> Self {
        match arg {
            CodeReviewEffortArg::Low => Self::Low,
            CodeReviewEffortArg::Medium => Self::Medium,
            CodeReviewEffortArg::High => Self::High,
            CodeReviewEffortArg::Max => Self::Max,
        }
    }
}

#[derive(Args, Debug, PartialEq, Eq, Clone)]
pub struct TaskReviewArgs {
    #[arg(help = "Atomic task identifier (e.g. TASK-001)")]
    pub task_id: String,

    #[arg(long, value_enum, default_value_t = CodeReviewEffortArg::Medium, help = "Review effort level (low, medium, high, max)")]
    pub effort: CodeReviewEffortArg,

    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}

#[derive(Args, Debug, PartialEq, Eq, Clone, Default)]
pub struct TaskCompleteArgs {
    #[arg(help = "Atomic task identifier (e.g. TASK-001)")]
    pub task_id: String,

    #[arg(long, value_enum, help = "Optional multi-angle code review effort level before completion")]
    pub code_review: Option<CodeReviewEffortArg>,

    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}

#[derive(Args, Debug, PartialEq, Eq, Clone, Default)]
pub struct WaveRunArgs {
    #[arg(long, help = "Topological execution wave index number")]
    pub wave: u32,

    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}

#[derive(Args, Debug, PartialEq, Eq, Clone, Default)]
pub struct GauntletRunArgs {
    #[arg(long, conflicts_with = "skip_security_review", help = "Force execution of specialized AppSec diff review (security-review)")]
    pub security_review: bool,

    #[arg(long, conflicts_with = "security_review", help = "Explicitly skip specialized AppSec diff review without prompting")]
    pub skip_security_review: bool,

    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}

impl GauntletRunArgs {
    pub const fn resolve_cli_security_flag(&self) -> CliSecurityReviewFlag {
        if self.security_review {
            CliSecurityReviewFlag::ForceExecute
        } else if self.skip_security_review {
            CliSecurityReviewFlag::ForceSkip
        } else {
            CliSecurityReviewFlag::Unspecified
        }
    }
}

#[derive(Args, Debug, PartialEq, Eq, Clone, Default)]
pub struct DoctorArgs {
    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}

#[derive(Args, Debug, PartialEq, Eq, Clone, Default)]
pub struct DoneArgs {
    #[arg(long, conflicts_with = "skip_security_review", help = "Force execution of specialized AppSec diff review (security-review)")]
    pub security_review: bool,

    #[arg(long, conflicts_with = "security_review", help = "Explicitly skip specialized AppSec diff review without prompting")]
    pub skip_security_review: bool,

    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}

impl DoneArgs {
    pub const fn resolve_cli_security_flag(&self) -> CliSecurityReviewFlag {
        if self.security_review {
            CliSecurityReviewFlag::ForceExecute
        } else if self.skip_security_review {
            CliSecurityReviewFlag::ForceSkip
        } else {
            CliSecurityReviewFlag::Unspecified
        }
    }
}

#[derive(Args, Debug, PartialEq, Eq, Clone)]
pub struct MergeArgs {
    #[arg(long, help = "Target branch for deterministic merge (e.g. develop)")]
    pub target: String,

    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}

#[derive(Args, Debug, PartialEq, Eq, Clone, Default)]
pub struct PrArgs {
    #[arg(long, help = "Target branch for the pull request (defaults to develop)")]
    pub target: Option<String>,

    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}

#[derive(Args, Debug, PartialEq, Eq, Clone)]
pub struct McpArgs {
    #[command(subcommand)]
    pub action: McpSubcommand,
}

#[derive(Subcommand, Debug, PartialEq, Eq, Clone)]
pub enum McpSubcommand {
    #[command(name = "serve", about = "Start TRustKit Model Context Protocol server over stdio")]
    Serve(McpServeArgs),
}

#[derive(Args, Debug, PartialEq, Eq, Clone, Default)]
pub struct McpServeArgs {
    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}

#[derive(Args, Debug, PartialEq, Eq, Clone)]
pub struct AuditArgs {
    #[command(subcommand)]
    pub action: AuditSubcommand,
}

#[derive(Subcommand, Debug, PartialEq, Eq, Clone)]
pub enum AuditSubcommand {
    #[command(
        name = "security",
        about = "Run decoupled 6-phase pre-release security audit and pentest ledger validation"
    )]
    Security(AuditSecurityArgs),
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AuditProfileArg {
    #[value(name = "quick")]
    Quick,
    #[default]
    #[value(name = "standard")]
    Standard,
    #[value(name = "deep")]
    Deep,
}

#[derive(Args, Debug, PartialEq, Eq, Clone, Default)]
pub struct AuditSecurityArgs {
    #[arg(long, value_enum, default_value_t = AuditProfileArg::Standard, help = "Audit depth profile (quick, standard, deep)")]
    pub profile: AuditProfileArg,

    #[arg(long, help = "Target repository or directory scope path")]
    pub scope: Option<String>,

    #[arg(long, help = "External or git-ignored directory for audit ledger artifacts")]
    pub output_dir: Option<String>,

    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}
