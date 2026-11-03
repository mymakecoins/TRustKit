// Tests for MCP terminal monitoring tools (TASK-008).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use serde_json::json;
use trustkit_orch::McpServer;

#[test]
fn test_mcp_terminal_tools_listed() {
    let tools = McpServer::list_tools();
    let names: Vec<String> = tools.into_iter().map(|t| t.name).collect();
    assert!(names.contains(&"trustkit_wave_status".to_string()));
    assert!(names.contains(&"trustkit_terminal_inspect".to_string()));
    assert!(names.contains(&"trustkit_terminal_send".to_string()));
}

#[test]
fn test_mcp_terminal_inspect_call() {
    let server = McpServer::new();
    let req = serde_json::from_value(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "trustkit_terminal_inspect",
            "arguments": {
                "task_id": "TASK-001",
                "lines": 20
            }
        }
    })).unwrap();

    let resp = server.handle_request(&req).expect("Should handle request");
    assert!(resp.error.is_none());
    let result = resp.result.expect("Should have result");
    let content = result["content"][0]["text"].as_str().unwrap();
    assert!(content.contains("TASK-001"));
}

#[test]
fn test_mcp_terminal_send_call() {
    let server = McpServer::new();
    let req = serde_json::from_value(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "trustkit_terminal_send",
            "arguments": {
                "task_id": "TASK-001",
                "input": "y\n"
            }
        }
    })).unwrap();

    let resp = server.handle_request(&req).expect("Should handle request");
    assert!(resp.error.is_none());
    let result = resp.result.expect("Should have result");
    let content = result["content"][0]["text"].as_str().unwrap();
    assert!(content.contains("delivered") || content.contains("acknowledged"));
}
