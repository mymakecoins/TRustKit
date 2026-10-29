// Clap v4 CLI argument definitions for Specs, Waves (Plan), AST Audit, and Context.
// Constitutional Principle V: Separation of Construction and Execution.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use clap::Args;

#[derive(Args, Debug, PartialEq, Eq, Clone, Default)]
pub struct SpecLockArgs {
    #[arg(long, help = "Path to the specification file (defaults to active cycle spec)")]
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

#[derive(Args, Debug, PartialEq, Eq, Clone, Default)]
pub struct WavePlanArgs {
    #[arg(long, help = "Path to directory containing atomic task definitions (TASK-NNN.md)")]
    pub tasks_dir: Option<String>,

    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}

#[derive(Args, Debug, PartialEq, Eq, Clone, Default)]
pub struct GauntletAstAuditArgs {
    #[arg(long, num_args = 1.., value_delimiter = ' ', help = "Explicit list of files to audit")]
    pub files: Vec<String>,

    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}

#[derive(Args, Debug, PartialEq, Eq, Clone, Default)]
pub struct ContextSliceArgs {
    #[arg(long, help = "Target file path to slice")]
    pub file: Option<String>,

    #[arg(long, num_args = 1.., value_delimiter = ' ', help = "Multiple target file paths to slice")]
    pub files: Vec<String>,

    #[arg(long, help = "Line or token budget limit")]
    pub budget: Option<usize>,

    #[arg(long, num_args = 1.., value_delimiter = ',', help = "Symbol filters to preserve")]
    pub symbols: Vec<String>,

    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}
