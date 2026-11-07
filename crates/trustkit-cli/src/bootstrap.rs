// Composition Root isolating object graph assembly and dependency injection.
// Constitutional Principle V: Separation of Construction and Execution.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use crate::cli_contract::{
    Cli, Command, ContextArgs, ContextSubcommand, GauntletArgs, GauntletSubcommand, HookArgs,
    HookSubcommand, SpecArgs, SpecSubcommand, WaveArgs, WaveSubcommand,
};
use crate::commands::{
    audit_security_cmd, auto_cmd, context_slice_cmd, doctor_cmd, done_cmd, gauntlet_ast_audit_cmd,
    gauntlet_run_cmd, graph_cmd, hook_cmd, init_cmd, mcp_cmd, memory_cmd, merge_cmd,
    milestones_cmd, pr_cmd, spec_init_cmd, spec_lint_cmd, spec_lock_cmd, spec_prompt_cmd,
    status_cmd, task_cmd, wave_plan_cmd, wave_run_cmd,
};
use crate::error_handler::DomainOutcome;
use std::path::Path;
use trustkit_core::errors::DomainViolation;
use trustkit_db::SqliteCycleRepository;
use trustkit_db::sqlite_adapter::SqliteWalRepositoryAdapter;
use trustkit_gh::{BoardSynchronizer, PrGateway, ProjectBoardSynchronizer, PullRequestGateway};
use trustkit_orch::{AsyncExecutionSupervisor, VisualEvidenceGuard, WorktreeIsolationProvider};

pub struct FastSyncBootstrap;

impl FastSyncBootstrap {
    pub fn dispatch(cli: &Cli) -> Result<DomainOutcome, DomainViolation> {
        let json_mode = cli.is_json_mode();
        match &cli.command {
            Command::Status(args) => status_cmd::execute(args, json_mode),
            Command::Hook(HookArgs { action }) => match action {
                HookSubcommand::PreTool(args) => hook_cmd::execute_pre_tool(args, json_mode),
            },
            Command::Spec(SpecArgs { action }) => match action {
                SpecSubcommand::Prompt(args) => spec_prompt_cmd::execute(args, json_mode),
                SpecSubcommand::Lint(args) => spec_lint_cmd::execute(args, json_mode),
                SpecSubcommand::Init(args) => spec_init_cmd::execute(args, json_mode),
                SpecSubcommand::Lock(args) => spec_lock_cmd::execute(args, json_mode),
            },
            Command::Wave(WaveArgs { action }) => match action {
                WaveSubcommand::Plan(args) => wave_plan_cmd::execute(args, json_mode),
                WaveSubcommand::Run(args) => wave_run_cmd::execute(args, json_mode),
            },
            Command::Gauntlet(GauntletArgs { action }) => match action {
                GauntletSubcommand::AstAudit(args) => {
                    gauntlet_ast_audit_cmd::execute(args, json_mode)
                }
                GauntletSubcommand::Run(args) => gauntlet_run_cmd::execute(args, json_mode),
            },
            Command::Context(ContextArgs { action }) => match action {
                ContextSubcommand::Slice(args) => context_slice_cmd::execute(args, json_mode),
            },
            Command::Task(args) => task_cmd::execute(args, json_mode),
            Command::Doctor(args) => doctor_cmd::execute(args, json_mode),
            Command::Done(args) => done_cmd::execute(args, json_mode),
            Command::Merge(args) => merge_cmd::execute(args, json_mode),
            Command::Pr(args) => pr_cmd::execute(args, json_mode),
            Command::Mcp(args) => mcp_cmd::execute(args, json_mode),
            Command::Memory(args) => memory_cmd::execute(args, json_mode),
            Command::Graph(args) => graph_cmd::execute(args, json_mode),
            Command::Audit(args) => audit_security_cmd::execute(args, json_mode),
            Command::Auto(args) => auto_cmd::execute(args, json_mode),
            Command::Milestones(args) => milestones_cmd::execute(args, json_mode),
            Command::Init(_) => Err(DomainViolation::InfrastructureBoundaryFault {
                subsystem: "bootstrap".to_string(),
                details: "Command 'init' requires FullConcurrentBootstrap path.".to_string(),
            }),
        }
    }
}

pub struct FullConcurrentBootstrap {
    pub repo: SqliteCycleRepository,
    pub worktree_provider: WorktreeIsolationProvider,
    pub supervisor: AsyncExecutionSupervisor,
    pub visual_evidence_guard: VisualEvidenceGuard,
    pub board_synchronizer: BoardSynchronizer,
    pub pr_gateway: PrGateway,
}

impl FullConcurrentBootstrap {
    /// Assembles all concrete infrastructure adapters into the composition root graph.
    pub fn new(repo_root: &Path, db_path: &Path) -> Result<Self, DomainViolation> {
        let repo = SqliteWalRepositoryAdapter::open(db_path)?;
        let worktree_provider = WorktreeIsolationProvider::new(repo_root);
        let supervisor = AsyncExecutionSupervisor::new();
        let visual_evidence_guard = VisualEvidenceGuard::new(repo_root);
        let board_synchronizer = ProjectBoardSynchronizer::new();
        let pr_gateway = PullRequestGateway::new();

        Ok(Self {
            repo,
            worktree_provider,
            supervisor,
            visual_evidence_guard,
            board_synchronizer,
            pr_gateway,
        })
    }

    /// Default initialization from repository root and .trustkit/state.db.
    pub fn init_default() -> Result<Self, DomainViolation> {
        let repo_root = Path::new(".");
        let db_path = repo_root.join(".trustkit").join("state.db");
        Self::new(repo_root, &db_path)
    }

    /// Dispatch commands requiring concurrent execution runtime or full adapter orchestration.
    pub fn dispatch(cli: &Cli) -> Result<DomainOutcome, DomainViolation> {
        let json_mode = cli.is_json_mode();
        match &cli.command {
            Command::Init(args) => {
                let runtime = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .map_err(|err| DomainViolation::InfrastructureBoundaryFault {
                        subsystem: "tokio_runtime".to_string(),
                        details: format!("Failed building async runtime: {err}"),
                    })?;

                runtime.block_on(async { init_cmd::execute(args, json_mode) })
            }
            Command::Wave(WaveArgs {
                action: WaveSubcommand::Run(args),
            }) => wave_run_cmd::execute(args, json_mode),
            Command::Task(args) => task_cmd::execute(args, json_mode),
            Command::Gauntlet(GauntletArgs {
                action: GauntletSubcommand::Run(args),
            }) => gauntlet_run_cmd::execute(args, json_mode),
            Command::Doctor(args) => doctor_cmd::execute(args, json_mode),
            Command::Done(args) => done_cmd::execute(args, json_mode),
            Command::Merge(args) => merge_cmd::execute(args, json_mode),
            Command::Pr(args) => pr_cmd::execute(args, json_mode),
            Command::Mcp(args) => mcp_cmd::execute(args, json_mode),
            _ => FastSyncBootstrap::dispatch(cli),
        }
    }
}

pub fn dispatch_cli(cli: &Cli) -> Result<DomainOutcome, DomainViolation> {
    match &cli.command {
        Command::Hook(_)
        | Command::Status(_)
        | Command::Spec(_)
        | Command::Context(_)
        | Command::Memory(_)
        | Command::Graph(_)
        | Command::Audit(_)
        | Command::Auto(_)
        | Command::Milestones(_)
        | Command::Wave(WaveArgs {
            action: WaveSubcommand::Plan(_),
        })
        | Command::Gauntlet(GauntletArgs {
            action: GauntletSubcommand::AstAudit(_),
        }) => FastSyncBootstrap::dispatch(cli),

        Command::Init(_)
        | Command::Wave(WaveArgs {
            action: WaveSubcommand::Run(_),
        })
        | Command::Task(_)
        | Command::Gauntlet(GauntletArgs {
            action: GauntletSubcommand::Run(_),
        })
        | Command::Doctor(_)
        | Command::Done(_)
        | Command::Merge(_)
        | Command::Pr(_)
        | Command::Mcp(_) => FullConcurrentBootstrap::dispatch(cli),
    }
}

#[cfg(test)]
#[path = "bootstrap_tests.rs"]
mod tests;
