//! Orquestracao de execucao, isolamento em worktrees e integracao com GitHub.

pub mod async_supervisor;
pub mod autonomous_wave_orchestrator;
pub mod builder_agent_dispatcher;
pub mod certification_reporter;
pub mod mcp_graph_dispatch;
pub mod mcp_server;
pub mod mcp_spec_dispatch;
pub mod mcp_terminal_dispatch;
pub mod pty_socket_adapter;
pub mod supervisor_runtime_selector;
pub mod task_synthesizer;
pub mod terminal_output_sanitizer;
pub mod visual_evidence_guard;
pub mod wave_board_dashboard_sse_client;
pub mod wave_board_http_server;
pub mod wave_board_lifecycle_shutdown;
pub mod wave_board_port_allocator;
pub mod wave_board_server_sse;
pub mod worktree_provider;

pub use async_supervisor::AsyncExecutionSupervisor;
pub use autonomous_wave_orchestrator::{
    AutonomousWaveOrchestrator, DefaultSemanticCodeReviewer, MultiWaveDependencies,
    MultiWaveExecutionOutcome, MultiWaveExecutionRequest, TaskExecutionSummary,
};
pub use builder_agent_dispatcher::BuilderAgentDispatcher;
pub use certification_reporter::CycleCertificationReporter;
pub use mcp_server::McpServer;
pub use pty_socket_adapter::PtySocketAdapter;
pub use supervisor_runtime_selector::{ResolvedRuntimeBackend, SupervisorRuntimeSelector};
pub use task_synthesizer::{SynthesisReport, TaskSynthesizer};
pub use visual_evidence_guard::VisualEvidenceGuard;
pub use worktree_provider::WorktreeIsolationProvider;
