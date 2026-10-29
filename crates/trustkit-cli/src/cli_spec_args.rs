// Clap v4 CLI argument definitions for Specs, Waves (Plan), AST Audit, Context, and Memory.
// Constitutional Principle V: Separation of Construction and Execution.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use clap::{Args, Subcommand};

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

#[derive(Args, Debug, PartialEq, Eq, Clone)]
pub struct MemoryArgs {
    #[command(subcommand)]
    pub action: MemorySubcommand,
}

#[derive(Subcommand, Debug, PartialEq, Eq, Clone)]
pub enum MemorySubcommand {
    #[command(name = "write", about = "Write and index a Markdown knowledge document into .trustkit/wiki/")]
    Write(MemoryWriteArgs),
    #[command(name = "read", about = "Read a Markdown knowledge document by slug from .trustkit/wiki/")]
    Read(MemoryReadArgs),
    #[command(name = "search", about = "Search indexed knowledge documents using FTS5 BM25 ranking")]
    Search(MemorySearchArgs),
    #[command(name = "list", about = "List all Markdown knowledge documents in .trustkit/wiki/")]
    List(MemoryListArgs),
    #[command(name = "rebuild", about = "Deterministically rebuild the SQLite FTS5 index and wikilink graph from .trustkit/wiki/")]
    Rebuild(MemoryRebuildArgs),
}

#[derive(Args, Debug, PartialEq, Eq, Clone, Default)]
pub struct MemoryWriteArgs {
    #[arg(help = "Relative document slug (e.g. adrs/adr-002-storage-strategy)")]
    pub slug: String,

    #[arg(long, default_value = "Untitled", help = "Document title")]
    pub title: String,

    #[arg(long, value_delimiter = ',', help = "Comma-separated tags (e.g. adr,storage,sqlite)")]
    pub tags: Vec<String>,

    #[arg(long, visible_alias = "body", default_value = "", help = "Markdown body content")]
    pub content: String,

    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}

#[derive(Args, Debug, PartialEq, Eq, Clone, Default)]
pub struct MemoryReadArgs {
    #[arg(help = "Relative document slug to read")]
    pub slug: String,

    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}

#[derive(Args, Debug, PartialEq, Eq, Clone, Default)]
pub struct MemorySearchArgs {
    #[arg(help = "Lexical FTS5 search query string")]
    pub query: String,

    #[arg(long, default_value_t = 10, help = "Maximum number of BM25 ranked hits to return")]
    pub limit: usize,

    #[arg(long, value_delimiter = ',', help = "Optional comma-separated required tags filter")]
    pub tags: Vec<String>,

    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}

#[derive(Args, Debug, PartialEq, Eq, Clone, Default)]
pub struct MemoryListArgs {
    #[arg(long, value_delimiter = ',', help = "Optional comma-separated required tags filter")]
    pub tags: Vec<String>,

    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}

#[derive(Args, Debug, PartialEq, Eq, Clone, Default)]
pub struct MemoryRebuildArgs {
    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}
