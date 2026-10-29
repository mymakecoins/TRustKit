// Clap v4 CLI argument definitions for Gates 4, 5, and 6.
// Constitutional Principle V: Separation of Construction and Execution.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use clap::{Args, Subcommand, ValueEnum};
use serde::Serialize;

#[derive(Args, Debug, PartialEq, Eq, Clone)]
pub struct TaskArgs {
    #[command(subcommand)]
    pub action: TaskSubcommand,
}

#[derive(Subcommand, Debug, PartialEq, Eq, Clone)]
pub enum TaskSubcommand {
    #[command(name = "test", about = "Execute test with TDD phase verification")]
    Test(TaskTestArgs),
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

#[derive(Args, Debug, PartialEq, Eq, Clone)]
pub struct TaskCompleteArgs {
    #[arg(help = "Atomic task identifier (e.g. TASK-001)")]
    pub task_id: String,

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
    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}

#[derive(Args, Debug, PartialEq, Eq, Clone, Default)]
pub struct DoctorArgs {
    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}

#[derive(Args, Debug, PartialEq, Eq, Clone, Default)]
pub struct DoneArgs {
    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
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
