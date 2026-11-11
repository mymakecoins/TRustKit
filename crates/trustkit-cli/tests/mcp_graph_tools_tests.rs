// Integration tests for CLI `trustkit graph` and MCP `trustkit_graph_*` tools (TASK-008, RF-032).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use assert_cmd::Command;
use serde_json::{Value, json};
use std::fs;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use trustkit_ast::GraphHealthAuditor;
use trustkit_core::entities::{
    EdgeConfidence, EdgeRelationKind, FileGraphManifest, GraphNodeKind, StructuralEdge,
    StructuralNode,
};
use trustkit_core::ports::StructuralGraphRepositoryPort;
use trustkit_db::{GraphSqliteRepository, KnowledgeWikiFsRepository};
use trustkit_orch::McpServer;
use trustkit_orch::mcp_server::JsonRpcRequest;

#[test]
fn test_cli_graph_build_health_path_explain_and_context_slice_null_free_json() {
    let tmp = TempDir::new().expect("create tempdir");
    let src_dir = tmp.path().join("crates/my-crate/src");
    fs::create_dir_all(&src_dir).expect("create src dir");

    let lib_rs = src_dir.join("lib.rs");
    fs::write(
        &lib_rs,
        "pub struct UserModel;\npub fn fetch_user() -> UserModel { UserModel }\n",
    )
    .expect("write lib.rs");

    let svc_rs = src_dir.join("service.rs");
    fs::write(&svc_rs, "use crate::UserModel;\npub struct UserService;\n")
        .expect("write service.rs");

    // 1. trustkit graph build --json
    let build_out = Command::cargo_bin("trustkit")
        .expect("trustkit binary")
        .current_dir(tmp.path())
        .args(["graph", "build", "--json"])
        .output()
        .expect("run graph build");

    assert!(build_out.status.success(), "graph build should exit 0");
    let build_stdout = String::from_utf8(build_out.stdout).expect("utf8 stdout");
    assert!(
        !build_stdout.contains("null"),
        "JSON must be strictly null-free"
    );
    let build_json: Value = serde_json::from_str(&build_stdout).expect("valid json");
    assert_eq!(build_json["verdict"], "PASS");
    assert!(build_json["total_nodes"].as_u64().unwrap_or(0) >= 2);
    assert!(
        tmp.path()
            .join(".trustkit/wiki/graph/GRAPH_REPORT.md")
            .exists()
    );

    // 2. trustkit graph explain <NODE_ID> --json
    let explain_out = Command::cargo_bin("trustkit")
        .expect("trustkit binary")
        .current_dir(tmp.path())
        .args(["graph", "explain", "crates/my-crate/src/lib.rs", "--json"])
        .output()
        .expect("run graph explain");

    assert!(explain_out.status.success(), "graph explain should exit 0");
    let explain_stdout = String::from_utf8(explain_out.stdout).expect("utf8 stdout");
    assert!(
        !explain_stdout.contains("null"),
        "JSON must be strictly null-free"
    );
    let explain_json: Value = serde_json::from_str(&explain_stdout).expect("valid json");
    assert_eq!(explain_json["outcome"], "FOUND");
    assert_eq!(
        explain_json["node"]["node_id"],
        "crates/my-crate/src/lib.rs"
    );

    // 3. trustkit graph path <SOURCE> <TARGET> --json
    let path_out = Command::cargo_bin("trustkit")
        .expect("trustkit binary")
        .current_dir(tmp.path())
        .args([
            "graph",
            "path",
            "crates/my-crate/src/service.rs",
            "crates/my-crate/src/lib.rs",
            "--undirected",
            "--json",
        ])
        .output()
        .expect("run graph path");

    assert!(path_out.status.success(), "graph path should exit 0");
    let path_stdout = String::from_utf8(path_out.stdout).expect("utf8 stdout");
    assert!(
        !path_stdout.contains("null"),
        "JSON must be strictly null-free"
    );

    // 4. trustkit graph health --json
    let health_out = Command::cargo_bin("trustkit")
        .expect("trustkit binary")
        .current_dir(tmp.path())
        .args(["graph", "health", "--json"])
        .output()
        .expect("run graph health");

    assert!(health_out.status.success(), "graph health should exit 0");
    let health_stdout = String::from_utf8(health_out.stdout).expect("utf8 stdout");
    assert!(
        !health_stdout.contains("null"),
        "JSON must be strictly null-free"
    );
    let health_json: Value = serde_json::from_str(&health_stdout).expect("valid json");
    assert_eq!(health_json["verdict"], "PASS");

    // 5. trustkit context slice --file crates/my-crate/src/service.rs --mode bfs --hops 2 --budget 120 --json
    let slice_out = Command::cargo_bin("trustkit")
        .expect("trustkit binary")
        .current_dir(tmp.path())
        .args([
            "context",
            "slice",
            "--file",
            "crates/my-crate/src/service.rs",
            "--mode",
            "bfs",
            "--hops",
            "2",
            "--budget",
            "120",
            "--json",
        ])
        .output()
        .expect("run context slice");

    assert!(slice_out.status.success(), "context slice should exit 0");
    let slice_stdout = String::from_utf8(slice_out.stdout).expect("utf8 stdout");
    assert!(
        !slice_stdout.contains("null"),
        "JSON must be strictly null-free"
    );
    let slice_json: Value = serde_json::from_str(&slice_stdout).expect("valid json");
    assert_eq!(slice_json["status"], "CONTEXT_SLICED");
}

#[test]
fn test_mcp_tools_list_exposes_all_graph_tools_with_schemas() {
    let server = McpServer::new();
    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(201)),
        method: "tools/list".to_string(),
        params: None,
    };
    let resp = server.handle_request(&req).expect("tools/list response");
    let serialized = serde_json::to_string(&resp).expect("serialize response");
    assert!(
        !serialized.contains("null"),
        "tools/list JSON must be null-free"
    );

    let tools = resp
        .result
        .expect("result")
        .get("tools")
        .and_then(Value::as_array)
        .expect("tools array")
        .clone();
    for expected_tool in [
        "trustkit_graph_query",
        "trustkit_graph_path",
        "trustkit_graph_explain",
        "trustkit_graph_health",
    ] {
        let found = tools
            .iter()
            .find(|t| t["name"] == expected_tool)
            .unwrap_or_else(|| panic!("Missing MCP tool: {expected_tool}"));
        assert_eq!(found["inputSchema"]["type"], "object");
    }
}

#[test]
fn test_mcp_graph_tools_call_under_5ms_and_null_free() {
    let tmp = TempDir::new().expect("create tempdir");
    let g_repo = GraphSqliteRepository::open_from_root(tmp.path()).expect("open graph repo");
    let k_repo =
        KnowledgeWikiFsRepository::open_from_root(tmp.path()).expect("open knowledge repo");

    let nodes = vec![
        StructuralNode::new(
            "crates/core/lib.rs",
            "lib.rs",
            GraphNodeKind::CodeFile,
            "crates/core/lib.rs",
            "CORE",
        ),
        StructuralNode::new(
            "crates/cli/main.rs",
            "main.rs",
            GraphNodeKind::CodeFile,
            "crates/cli/main.rs",
            "CLI",
        ),
    ];
    let edges = vec![StructuralEdge::new(
        "crates/cli/main.rs",
        "crates/core/lib.rs",
        EdgeRelationKind::Imports,
        EdgeConfidence::Extracted,
        "crates/cli/main.rs",
    )];
    let manifests = vec![
        FileGraphManifest::new("crates/core/lib.rs", "blake3_1", 1, "2026-10-29T22:00:00Z"),
        FileGraphManifest::new("crates/cli/main.rs", "blake3_2", 1, "2026-10-29T22:00:00Z"),
    ];
    g_repo
        .save_graph_snapshot(&nodes, &edges, &manifests, true)
        .expect("save graph snapshot");

    let analyzer = Arc::new(GraphHealthAuditor::new());
    let server = McpServer::with_knowledge_repo(Arc::new(k_repo))
        .with_graph_repo(Arc::new(g_repo))
        .with_graph_analyzer(analyzer);

    // Call trustkit_graph_health in < 5ms
    let start_health = Instant::now();
    let health_req = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(json!(1)),
        method: "tools/call".into(),
        params: Some(json!({"name": "trustkit_graph_health", "arguments": {"top_god_nodes": 5}})),
    };
    let health_resp = server
        .handle_request(&health_req)
        .expect("health response")
        .result
        .expect("res");
    let health_duration = start_health.elapsed();
    assert!(
        health_duration < Duration::from_millis(5),
        "MCP call must complete in < 5ms, took {:?}",
        health_duration
    );
    assert_eq!(health_resp["isError"], false);
    let health_text = health_resp["content"][0]["text"].as_str().expect("text");
    assert!(!health_text.contains("null"));
    assert!(health_text.contains("\"verdict\":\"PASS\""));

    // Call trustkit_graph_explain in < 5ms
    let start_explain = Instant::now();
    let explain_req = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(json!(2)),
        method: "tools/call".into(),
        params: Some(
            json!({"name": "trustkit_graph_explain", "arguments": {"node_id": "crates/core/lib.rs"}}),
        ),
    };
    let explain_resp = server
        .handle_request(&explain_req)
        .expect("explain response")
        .result
        .expect("res");
    let explain_duration = start_explain.elapsed();
    assert!(
        explain_duration < Duration::from_millis(5),
        "MCP call must complete in < 5ms, took {:?}",
        explain_duration
    );
    assert_eq!(explain_resp["isError"], false);
    let explain_text = explain_resp["content"][0]["text"].as_str().expect("text");
    assert!(!explain_text.contains("null"));
    assert!(explain_text.contains("\"outcome\":\"FOUND\""));

    // Call trustkit_graph_path in < 5ms
    let start_path = Instant::now();
    let path_req = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(json!(3)),
        method: "tools/call".into(),
        params: Some(
            json!({"name": "trustkit_graph_path", "arguments": {"source": "crates/cli/main.rs", "target": "crates/core/lib.rs"}}),
        ),
    };
    let path_resp = server
        .handle_request(&path_req)
        .expect("path response")
        .result
        .expect("res");
    let path_duration = start_path.elapsed();
    assert!(
        path_duration < Duration::from_millis(5),
        "MCP call must complete in < 5ms, took {:?}",
        path_duration
    );
    assert_eq!(path_resp["isError"], false);
    let path_text = path_resp["content"][0]["text"].as_str().expect("text");
    assert!(!path_text.contains("null"));
    assert!(path_text.contains("\"outcome\":\"PATH_FOUND\""));

    // Call trustkit_graph_query in < 5ms
    let start_query = Instant::now();
    let query_req = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(json!(4)),
        method: "tools/call".into(),
        params: Some(json!({"name": "trustkit_graph_query", "arguments": {"query": "lib.rs"}})),
    };
    let query_resp = server
        .handle_request(&query_req)
        .expect("query response")
        .result
        .expect("res");
    let query_duration = start_query.elapsed();
    assert!(
        query_duration < Duration::from_millis(5),
        "MCP call must complete in < 5ms, took {:?}",
        query_duration
    );
    assert_eq!(query_resp["isError"], false);
    let query_text = query_resp["content"][0]["text"].as_str().expect("text");
    assert!(!query_text.contains("null"));
    assert!(query_text.contains("\"outcome\":\"RESULTS\""));
}

#[test]
fn test_cli_and_mcp_deep_modules_seam_analysis_and_design_it_twice_under_10ms() {
    let tmp = TempDir::new().expect("create tempdir");
    let src_dir = tmp.path().join("crates/my-crate/src");
    fs::create_dir_all(&src_dir).expect("create src dir");

    let wrapper_rs = src_dir.join("wrapper.rs");
    fs::write(
        &wrapper_rs,
        "use crate::adapter::Adapter;\npub fn delegate_op(a: &Adapter) -> bool { a.run() }\n",
    )
    .expect("write wrapper.rs");

    let adapter_rs = src_dir.join("adapter.rs");
    fs::write(
        &adapter_rs,
        "pub trait StoragePort { fn run(&self) -> bool; }\npub struct Adapter;\nimpl StoragePort for Adapter { fn run(&self) -> bool { true } }\n",
    )
    .expect("write adapter.rs");

    // 1. Build graph and check CLI `trustkit graph health --json`
    Command::cargo_bin("trustkit")
        .expect("trustkit binary")
        .current_dir(tmp.path())
        .args(["graph", "build", "--json"])
        .assert()
        .success();

    let health_out = Command::cargo_bin("trustkit")
        .expect("trustkit binary")
        .current_dir(tmp.path())
        .args(["graph", "health", "--json"])
        .output()
        .expect("run graph health");
    assert!(health_out.status.success());
    let health_stdout = String::from_utf8(health_out.stdout).expect("utf8");
    assert!(!health_stdout.contains("null"));
    let health_json: Value = serde_json::from_str(&health_stdout).expect("json");
    assert!(health_json["shallow_modules"].is_array());
    assert!(health_json["module_depth_metrics"].is_array());
    assert!(health_json["seam_classifications"].is_array());
    assert!(
        !health_json["shallow_modules"]
            .as_array()
            .unwrap()
            .is_empty(),
        "wrapper.rs must be reported in shallow_modules"
    );

    // 2. Check CLI `trustkit spec prompt --idea "..." --design-it-twice --json`
    let prompt_out = Command::cargo_bin("trustkit")
        .expect("trustkit binary")
        .current_dir(tmp.path())
        .args([
            "spec",
            "prompt",
            "--idea",
            "Motor de auditoria de modulos profundos",
            "--design-it-twice",
            "--json",
        ])
        .output()
        .expect("run spec prompt");
    assert!(prompt_out.status.success());
    let prompt_stdout = String::from_utf8(prompt_out.stdout).expect("utf8");
    assert!(!prompt_stdout.contains("null"));
    let prompt_json: Value = serde_json::from_str(&prompt_stdout).expect("json");
    let briefs = prompt_json["design_it_twice_briefs"]
        .as_array()
        .expect("design_it_twice_briefs array");
    assert_eq!(briefs.len(), 3);

    // 3. Check MCP `trustkit_graph_health` and `trustkit_spec_context` in < 10ms
    let g_repo = GraphSqliteRepository::open_from_root(tmp.path()).expect("open graph repo");
    let k_repo =
        KnowledgeWikiFsRepository::open_from_root(tmp.path()).expect("open knowledge repo");
    let server = McpServer::with_knowledge_repo(Arc::new(k_repo))
        .with_graph_repo(Arc::new(g_repo))
        .with_graph_analyzer(Arc::new(GraphHealthAuditor::new()));

    let start_mcp = Instant::now();
    let mcp_health_req = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(json!(501)),
        method: "tools/call".into(),
        params: Some(json!({"name": "trustkit_graph_health", "arguments": {"top_god_nodes": 5}})),
    };
    let mcp_health_resp = server
        .handle_request(&mcp_health_req)
        .expect("mcp health")
        .result
        .expect("res");
    assert!(start_mcp.elapsed() < Duration::from_millis(10));
    let mcp_health_text = mcp_health_resp["content"][0]["text"].as_str().unwrap();
    assert!(!mcp_health_text.contains("null"));
    assert!(mcp_health_text.contains("\"shallow_modules\""));
    assert!(mcp_health_text.contains("\"seam_classifications\""));

    let start_ctx = Instant::now();
    let mcp_ctx_req = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(json!(502)),
        method: "tools/call".into(),
        params: Some(json!({
            "name": "trustkit_spec_context",
            "arguments": {"idea": "Porta de persistencia transacional", "design_it_twice": true}
        })),
    };
    let mcp_ctx_resp = server
        .handle_request(&mcp_ctx_req)
        .expect("mcp spec context")
        .result
        .expect("res");
    assert!(start_ctx.elapsed() < Duration::from_millis(10));
    let mcp_ctx_text = mcp_ctx_resp["content"][0]["text"].as_str().unwrap();
    assert!(!mcp_ctx_text.contains("null"));
    assert!(mcp_ctx_text.contains("\"design_it_twice_briefs\""));
}
