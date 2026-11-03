//! Orquestracao de execucao, isolamento em worktrees e integracao com GitHub.

pub mod async_supervisor;
pub mod certification_reporter;
pub mod mcp_graph_dispatch;
pub mod mcp_server;
pub mod mcp_spec_dispatch;
pub mod mcp_terminal_dispatch;
pub mod pty_socket_adapter;
pub mod supervisor_runtime_selector;
pub mod terminal_output_sanitizer;
pub mod visual_evidence_guard;
pub mod worktree_provider;

pub use async_supervisor::AsyncExecutionSupervisor;
pub use certification_reporter::CycleCertificationReporter;
pub use mcp_server::McpServer;
pub use pty_socket_adapter::PtySocketAdapter;
pub use supervisor_runtime_selector::{ResolvedRuntimeBackend, SupervisorRuntimeSelector};
pub use visual_evidence_guard::VisualEvidenceGuard;
pub use worktree_provider::WorktreeIsolationProvider;
