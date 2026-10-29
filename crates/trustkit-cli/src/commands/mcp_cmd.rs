// Handler for `trustkit mcp` (TASK-014).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::cli_contract::{McpArgs, McpSubcommand};
use crate::error_handler::DomainOutcome;
use std::io::{self, BufReader};
use trustkit_core::errors::DomainViolation;
use trustkit_orch::McpServer;

pub fn execute(args: &McpArgs, _json_mode: bool) -> Result<DomainOutcome, DomainViolation> {
    match &args.action {
        McpSubcommand::Serve(_) => execute_serve(),
    }
}

pub fn execute_serve() -> Result<DomainOutcome, DomainViolation> {
    let server = McpServer::new();
    let stdin = io::stdin();
    let stdout = io::stdout();

    let reader = BufReader::new(stdin.lock());
    let writer = stdout.lock();

    server
        .run_stdio(reader, writer)
        .map_err(|err| DomainViolation::InfrastructureBoundaryFault {
            subsystem: "mcp_server".to_string(),
            details: format!("MCP server runtime error: {err}"),
        })?;

    Ok(DomainOutcome::Success)
}
