// Clap v4 CLI argument definitions for Specs, Waves (Plan), AST Audit, Context, and Memory.
// Constitutional Principle V: Separation of Construction and Execution.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use clap::{Args, Subcommand};

#[derive(Args, Debug, PartialEq, Eq, Clone, Default)]
pub struct SpecPromptArgs {
    #[arg(long, help = "Feature idea or description in natural language")]
    pub idea: String,
    #[arg(
        long,
        default_value = "2",
        help = "Number of context hops for knowledge and AST search"
    )]
    pub context_hops: usize,
    #[arg(long, default_value = "4000", help = "Token budget for prompt output")]
    pub budget: usize,
    #[arg(
        long,
        help = "Generate 3 contrasting Design It Twice architectural briefings for parallel subagents"
    )]
    pub design_it_twice: bool,
    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}

#[derive(Args, Debug, PartialEq, Eq, Clone, Default)]
pub struct SpecLintArgs {
    #[arg(
        long,
        help = "Path to the specification file (defaults to active cycle spec)"
    )]
    pub file: Option<String>,
    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
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

#[derive(Args, Debug, PartialEq, Eq, Clone, Default)]
pub struct GauntletAstAuditArgs {
    #[arg(long, num_args = 1.., value_delimiter = ' ', help = "Explicit list of files to audit")]
    pub files: Vec<String>,

    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}

#[derive(Args, Debug, PartialEq, Eq, Clone)]
pub struct ContextArgs {
    #[command(subcommand)]
    pub action: ContextSubcommand,
}

#[derive(Subcommand, Debug, PartialEq, Eq, Clone)]
pub enum ContextSubcommand {
    #[command(
        name = "slice",
        about = "Slice AST context within budget for builder agents"
    )]
    Slice(ContextSliceArgs),
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

    #[arg(long, help = "Graph traversal mode: bfs or dfs")]
    pub mode: Option<String>,

    #[arg(long, help = "Maximum graph traversal hops from target")]
    pub hops: Option<usize>,

    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}

#[derive(Args, Debug, PartialEq, Eq, Clone)]
pub struct GraphArgs {
    #[command(subcommand)]
    pub action: GraphSubcommand,
}

#[derive(Subcommand, Debug, PartialEq, Eq, Clone)]
pub enum GraphSubcommand {
    #[command(
        name = "build",
        about = "Build or update the AST structural graph and emit GRAPH_REPORT.md"
    )]
    Build(GraphBuildArgs),
    #[command(
        name = "health",
        about = "Audit graph health, god nodes, loops and architectural boundaries"
    )]
    Health(GraphHealthArgs),
    #[command(
        name = "path",
        about = "Find shortest path between two graph nodes via BFS"
    )]
    Path(GraphPathArgs),
    #[command(
        name = "explain",
        about = "Explain structural connections and wiki relationships for a node"
    )]
    Explain(GraphExplainArgs),
}

#[derive(Args, Debug, PartialEq, Eq, Clone, Default)]
pub struct GraphBuildArgs {
    #[arg(
        long,
        help = "Incremental scan using BLAKE3 digests, only updating changed files"
    )]
    pub update: bool,

    #[arg(long, help = "Bypass Shrink-Guard if node count decreases")]
    pub force: bool,

    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}

#[derive(Args, Debug, PartialEq, Eq, Clone, Default)]
pub struct GraphHealthArgs {
    #[arg(
        long,
        default_value_t = 10,
        help = "Maximum number of God Nodes to rank"
    )]
    pub top_god_nodes: usize,

    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}

#[derive(Args, Debug, PartialEq, Eq, Clone, Default)]
pub struct GraphPathArgs {
    #[arg(help = "Source node ID or file path")]
    pub source: String,

    #[arg(help = "Target node ID or file path")]
    pub target: String,

    #[arg(long, help = "Consider edges undirected")]
    pub undirected: bool,

    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}

#[derive(Args, Debug, PartialEq, Eq, Clone, Default)]
pub struct GraphExplainArgs {
    #[arg(help = "Node identifier to explain")]
    pub node_id: String,

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
    #[command(
        name = "write",
        about = "Write and index a Markdown knowledge document into .trustkit/wiki/"
    )]
    Write(MemoryWriteArgs),
    #[command(
        name = "read",
        about = "Read a Markdown knowledge document by slug from .trustkit/wiki/"
    )]
    Read(MemoryReadArgs),
    #[command(
        name = "search",
        about = "Search indexed knowledge documents using FTS5 BM25 ranking"
    )]
    Search(MemorySearchArgs),
    #[command(
        name = "list",
        about = "List all Markdown knowledge documents in .trustkit/wiki/"
    )]
    List(MemoryListArgs),
    #[command(
        name = "rebuild",
        about = "Deterministically rebuild the SQLite FTS5 index and wikilink graph from .trustkit/wiki/"
    )]
    Rebuild(MemoryRebuildArgs),
    #[command(
        name = "glossary",
        about = "Manage ubiquitous domain glossary terms in .trustkit/wiki/glossary.md"
    )]
    Glossary(MemoryGlossaryArgs),
    #[command(
        name = "adr-check",
        about = "Evaluate architectural decision proposal against 3-criteria filter"
    )]
    AdrCheck(MemoryAdrCheckArgs),
}

#[derive(Args, Debug, PartialEq, Eq, Clone, Default)]
pub struct MemoryWriteArgs {
    #[arg(help = "Relative document slug (e.g. adrs/adr-002-storage-strategy)")]
    pub slug: String,

    #[arg(long, default_value = "Untitled", help = "Document title")]
    pub title: String,

    #[arg(
        long,
        value_delimiter = ',',
        help = "Comma-separated tags (e.g. adr,storage,sqlite)"
    )]
    pub tags: Vec<String>,

    #[arg(
        long,
        visible_alias = "body",
        default_value = "",
        help = "Markdown body content"
    )]
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

    #[arg(
        long,
        default_value_t = 10,
        help = "Maximum number of BM25 ranked hits to return"
    )]
    pub limit: usize,

    #[arg(
        long,
        value_delimiter = ',',
        help = "Optional comma-separated required tags filter"
    )]
    pub tags: Vec<String>,

    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}

#[derive(Args, Debug, PartialEq, Eq, Clone, Default)]
pub struct MemoryListArgs {
    #[arg(
        long,
        value_delimiter = ',',
        help = "Optional comma-separated required tags filter"
    )]
    pub tags: Vec<String>,

    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}

#[derive(Args, Debug, PartialEq, Eq, Clone, Default)]
pub struct MemoryRebuildArgs {
    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}

#[derive(Args, Debug, PartialEq, Eq, Clone)]
pub struct MemoryGlossaryArgs {
    #[command(subcommand)]
    pub action: MemoryGlossarySubcommand,
}

#[derive(Subcommand, Debug, PartialEq, Eq, Clone)]
pub enum MemoryGlossarySubcommand {
    #[command(name = "add", about = "Add or update a canonical term in the ubiquitous glossary")]
    Add(MemoryGlossaryAddArgs),
    #[command(name = "list", about = "List all canonical glossary terms")]
    List(MemoryGlossaryListArgs),
}

#[derive(Args, Debug, PartialEq, Eq, Clone, Default)]
pub struct MemoryGlossaryAddArgs {
    #[arg(long, help = "Canonical domain term")]
    pub term: String,

    #[arg(long, help = "Ubiquitous domain definition")]
    pub definition: String,

    #[arg(long, value_delimiter = ',', help = "Comma-separated rejected/ambiguous synonyms")]
    pub reject: Vec<String>,

    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}

#[derive(Args, Debug, PartialEq, Eq, Clone, Default)]
pub struct MemoryGlossaryListArgs {
    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}

#[derive(Args, Debug, PartialEq, Eq, Clone, Default)]
pub struct MemoryAdrCheckArgs {
    #[arg(long, help = "Proposed ADR title")]
    pub title: String,

    #[arg(long, help = "Context summary")]
    pub context: String,

    #[arg(long, help = "Is this decision hard to reverse?")]
    pub hard_to_reverse: bool,

    #[arg(long, help = "Is this decision surprising to a newcomer without context?")]
    pub surprising: bool,

    #[arg(long, help = "Are there genuine alternatives with real tradeoffs?")]
    pub tradeoff: bool,

    #[arg(long, help = "Emit null-free JSON output")]
    pub json: bool,
}

