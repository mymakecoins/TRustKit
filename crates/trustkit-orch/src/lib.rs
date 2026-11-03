//! TRustKit Orch: Concurrency orchestrator, worktree provisioning, supervisor, visual evidence guard and MCP server.

pub mod async_supervisor;
pub mod certification_reporter;
pub mod herdr_socket_adapter;
pub mod mcp_graph_dispatch;
pub mod mcp_server;
pub mod mcp_spec_dispatch;
pub mod visual_evidence_guard;
pub mod worktree_provider;

pub use async_supervisor::AsyncExecutionSupervisor;
pub use certification_reporter::CycleCertificationReporter;
pub use herdr_socket_adapter::HerdrSocketAdapter;
pub use mcp_server::McpServer;
pub use visual_evidence_guard::VisualEvidenceGuard;
pub use worktree_provider::WorktreeIsolationProvider;
