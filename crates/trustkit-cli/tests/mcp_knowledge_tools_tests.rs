// Integration tests for CLI `trustkit memory` and MCP `trustkit_knowledge_*` tools (TASK-008).
// Constitutional Principle II: Absolute Prohibition of NULL & Special Cases.
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use assert_cmd::Command;
use serde_json::{Value, json};
use std::fs;
use std::sync::Arc;
use tempfile::TempDir;
use trustkit_db::KnowledgeWikiFsRepository;
use trustkit_orch::McpServer;
use trustkit_orch::mcp_server::JsonRpcRequest;

#[test]
fn test_cli_memory_write_read_search_list_and_rebuild_null_free_json() {
    let tmp = TempDir::new().expect("create tempdir");

    // 1. trustkit memory write
    let write_out = Command::cargo_bin("trustkit")
        .expect("trustkit binary")
        .current_dir(tmp.path())
        .args([
            "memory", "write", "adrs/adr-002-storage-strategy",
            "--title", "Estrategia de Armazenamento",
            "--tags", "adr,storage,sqlite",
            "--content", "Decisao de persistencia atomica WAL vinculada a [[specs/spec-005]] e [[adrs/adr-001]].",
            "--json",
        ])
        .output()
        .expect("run memory write");

    assert!(write_out.status.success(), "memory write should exit 0");
    let write_stdout = String::from_utf8(write_out.stdout).expect("utf8 stdout");
    assert!(!write_stdout.contains("null"), "JSON must be strictly null-free");
    let write_json: Value = serde_json::from_str(&write_stdout).expect("valid json");
    assert_eq!(write_json["verdict"], "PASS");
    assert_eq!(write_json["outcome"], "SUCCESS");
    assert_eq!(write_json["slug"], "adrs/adr-002-storage-strategy");
    assert_eq!(write_json["indexed_links_count"], 2);
    assert!(tmp.path().join(".trustkit/wiki/adrs/adr-002-storage-strategy.md").exists());

    // 2. trustkit memory read
    let read_out = Command::cargo_bin("trustkit")
        .expect("trustkit binary")
        .current_dir(tmp.path())
        .args(["memory", "read", "adrs/adr-002-storage-strategy", "--json"])
        .output()
        .expect("run memory read");

    assert!(read_out.status.success(), "memory read should exit 0");
    let read_stdout = String::from_utf8(read_out.stdout).expect("utf8 stdout");
    assert!(!read_stdout.contains("null"), "JSON must be strictly null-free");
    let read_json: Value = serde_json::from_str(&read_stdout).expect("valid json");
    assert_eq!(read_json["outcome"], "FOUND");
    assert_eq!(read_json["document"]["title"], "Estrategia de Armazenamento");
    assert_eq!(read_json["document"]["tags"], json!(["adr", "storage", "sqlite"]));

    // 3. trustkit memory search
    let search_out = Command::cargo_bin("trustkit")
        .expect("trustkit binary")
        .current_dir(tmp.path())
        .args(["memory", "search", "persistencia atomica WAL", "--limit", "5", "--json"])
        .output()
        .expect("run memory search");

    assert!(search_out.status.success(), "memory search should exit 0");
    let search_stdout = String::from_utf8(search_out.stdout).expect("utf8 stdout");
    assert!(!search_stdout.contains("null"), "JSON must be strictly null-free");
    let search_json: Value = serde_json::from_str(&search_stdout).expect("valid json");
    assert_eq!(search_json["outcome"], "RESULTS");
    let hits = search_json["hits"].as_array().expect("hits array");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0]["slug"], "adrs/adr-002-storage-strategy");
    assert!(hits[0]["adjacent_links"].as_array().expect("adj").contains(&json!("specs/spec-005")));

    // 4. trustkit memory list
    let list_out = Command::cargo_bin("trustkit")
        .expect("trustkit binary")
        .current_dir(tmp.path())
        .args(["memory", "list", "--json"])
        .output()
        .expect("run memory list");

    assert!(list_out.status.success(), "memory list should exit 0");
    let list_stdout = String::from_utf8(list_out.stdout).expect("utf8 stdout");
    assert!(!list_stdout.contains("null"), "JSON must be strictly null-free");
    let list_json: Value = serde_json::from_str(&list_stdout).expect("valid json");
    assert_eq!(list_json["outcome"], "RESULTS");
    assert_eq!(list_json["total"], 1);

    // 5. Corrupt/remove state.db and run trustkit memory rebuild
    fs::remove_file(tmp.path().join(".trustkit/state.db")).expect("remove state.db");
    let rebuild_out = Command::cargo_bin("trustkit")
        .expect("trustkit binary")
        .current_dir(tmp.path())
        .args(["memory", "rebuild", "--json"])
        .output()
        .expect("run memory rebuild");

    assert!(rebuild_out.status.success(), "memory rebuild should exit 0");
    let rebuild_stdout = String::from_utf8(rebuild_out.stdout).expect("utf8 stdout");
    assert!(!rebuild_stdout.contains("null"), "JSON must be strictly null-free");
    let rebuild_json: Value = serde_json::from_str(&rebuild_stdout).expect("valid json");
    assert_eq!(rebuild_json["outcome"], "REBUILT");
    assert_eq!(rebuild_json["notes_restored"], 1);
    assert_eq!(rebuild_json["links_restored"], 2);
}

#[test]
fn test_mcp_tools_list_exposes_all_knowledge_tools_with_schemas() {
    let server = McpServer::new();
    let req = JsonRpcRequest { jsonrpc: "2.0".to_string(), id: Some(json!(101)), method: "tools/list".to_string(), params: None };
    let resp = server.handle_request(&req).expect("tools/list response");
    let serialized = serde_json::to_string(&resp).expect("serialize response");
    assert!(!serialized.contains("null"), "tools/list JSON must be null-free");

    let tools = resp.result.expect("result").get("tools").and_then(Value::as_array).expect("tools array").clone();
    for expected_tool in [
        "trustkit_knowledge_search",
        "trustkit_knowledge_read",
        "trustkit_knowledge_write",
        "trustkit_knowledge_rebuild",
    ] {
        let found = tools.iter().find(|t| t["name"] == expected_tool).unwrap_or_else(|| panic!("Missing MCP tool: {expected_tool}"));
        assert_eq!(found["inputSchema"]["type"], "object");
    }
}

#[test]
fn test_mcp_knowledge_write_read_search_and_rebuild_roundtrip() {
    let tmp = TempDir::new().expect("create tempdir");
    let repo = KnowledgeWikiFsRepository::open_from_root(tmp.path()).expect("open knowledge repo");
    let server = McpServer::with_knowledge_repo(Arc::new(repo));

    // 1. Call trustkit_knowledge_write
    let write_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "trustkit_knowledge_write",
            "arguments": {
                "slug": "rules/branch-validation",
                "title": "Regras de Validacao de Branch",
                "tags": ["rules", "git", "governance"],
                "content": "Regras de validacao de branch deterministica vinculadas a [[specs/spec-005]]."
            }
        })),
    };
    let write_res = server.handle_request(&write_req).expect("write response").result.expect("result");
    assert_eq!(write_res["isError"], false);
    let write_text = write_res["content"][0]["text"].as_str().expect("text");
    assert!(!write_text.contains("null"));
    let write_payload: Value = serde_json::from_str(write_text).expect("parse write payload");
    assert_eq!(write_payload["outcome"], "SUCCESS");
    assert_eq!(write_payload["slug"], "rules/branch-validation");
    assert!(tmp.path().join(".trustkit/wiki/rules/branch-validation.md").exists());

    // 2. Call trustkit_knowledge_search
    let search_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(2)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "trustkit_knowledge_search",
            "arguments": { "query": "regras de validacao de branch", "limit": 5 }
        })),
    };
    let search_text = server.handle_request(&search_req).expect("search").result.expect("res")["content"][0]["text"].as_str().expect("text").to_string();
    assert!(!search_text.contains("null"));
    let search_payload: Value = serde_json::from_str(&search_text).expect("parse search payload");
    assert_eq!(search_payload["outcome"], "RESULTS");
    assert_eq!(search_payload["hits"][0]["slug"], "rules/branch-validation");

    // 3. Call trustkit_knowledge_read
    let read_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(3)),
        method: "tools/call".to_string(),
        params: Some(json!({ "name": "trustkit_knowledge_read", "arguments": { "slug": "rules/branch-validation" } })),
    };
    let read_text = server.handle_request(&read_req).expect("read").result.expect("res")["content"][0]["text"].as_str().expect("text").to_string();
    assert!(!read_text.contains("null"));
    let read_payload: Value = serde_json::from_str(&read_text).expect("parse read payload");
    assert_eq!(read_payload["outcome"], "FOUND");
    assert_eq!(read_payload["document"]["title"], "Regras de Validacao de Branch");

    // 4. Call trustkit_knowledge_rebuild
    let rebuild_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(4)),
        method: "tools/call".to_string(),
        params: Some(json!({ "name": "trustkit_knowledge_rebuild", "arguments": {} })),
    };
    let rebuild_text = server.handle_request(&rebuild_req).expect("rebuild").result.expect("res")["content"][0]["text"].as_str().expect("text").to_string();
    assert!(!rebuild_text.contains("null"));
    let rebuild_payload: Value = serde_json::from_str(&rebuild_text).expect("parse rebuild payload");
    assert_eq!(rebuild_payload["outcome"], "REBUILT");
    assert_eq!(rebuild_payload["notes_restored"], 1);
    assert_eq!(rebuild_payload["links_restored"], 1);
}
