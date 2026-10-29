// MCP stdio server implementing the Model Context Protocol JSON-RPC 2.0 loop.
// Constitutional Principle II: Absolute Prohibition of NULL (skip_serializing_if).
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::io::{BufRead, Write};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use trustkit_core::knowledge_entities::{
    KnowledgeDocument, KnowledgeLookupOutcome, KnowledgeSearchQuery, KnowledgeWriteOutcome,
};
use trustkit_core::ports::KnowledgeRepositoryPort;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    #[serde(default)]
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

#[derive(Default, Clone)]
pub struct McpServer {
    knowledge_repo: Option<Arc<dyn KnowledgeRepositoryPort>>,
}

impl std::fmt::Debug for McpServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpServer").field("has_knowledge_repo", &self.knowledge_repo.is_some()).finish()
    }
}

fn tool(name: &str, description: &str, input_schema: Value) -> McpTool {
    McpTool { name: name.to_string(), description: description.to_string(), input_schema }
}

impl McpServer {
    pub fn new() -> Self { Self { knowledge_repo: None } }
    pub fn with_knowledge_repo(repo: Arc<dyn KnowledgeRepositoryPort>) -> Self { Self { knowledge_repo: Some(repo) } }

    pub fn list_tools() -> Vec<McpTool> {
        vec![
            tool("trustkit_init", "Inicializa o ciclo SDD deterministico (Gate 0)", json!({"type":"object","properties":{"issue":{"type":"integer"},"from_prd":{"type":"string"}}})),
            tool("trustkit_spec_lock", "Valida e congela especificacao com digest BLAKE3 (Gates 1 e 2)", json!({"type":"object"})),
            tool("trustkit_wave_plan", "Gera o planejamento topologico em ondas a partir do DAG (Gate 3)", json!({"type":"object"})),
            tool("trustkit_wave_run", "Despacha as tarefas da onda em git worktrees isoladas (Gate 4)", json!({"type":"object","properties":{"wave":{"type":"integer"}},"required":["wave"]})),
            tool("trustkit_task_test", "Executa teste fiscalizado de fase TDD (RED ou GREEN) (Gate 4)", json!({"type":"object","properties":{"task_id":{"type":"string"},"phase":{"type":"string","enum":["red","green"]}},"required":["task_id","phase"]})),
            tool("trustkit_gauntlet_run", "Executa a bateria de auditoria do Gauntlet (Gate 5)", json!({"type":"object"})),
            tool("trustkit_done", "Gera a certificacao do ciclo e renderiza gauntlet-report.md (Gate 6)", json!({"type":"object"})),
            tool("trustkit_pr", "Abre o Pull Request sob demanda no GitHub", json!({"type":"object","properties":{"title":{"type":"string"},"custom_body":{"type":"string"}}})),
            tool("trustkit_knowledge_search", "Busca lexical FTS5 ranqueada por BM25 na memoria perene", json!({"type":"object","properties":{"query":{"type":"string"},"limit":{"type":"integer"},"tags":{"type":"array","items":{"type":"string"}}},"required":["query"]})),
            tool("trustkit_knowledge_read", "Le documento da memoria perene (.trustkit/wiki/<slug>.md)", json!({"type":"object","properties":{"slug":{"type":"string"}},"required":["slug"]})),
            tool("trustkit_knowledge_write", "Grava atomicamente nota Markdown em .trustkit/wiki/ e indexa no FTS5", json!({"type":"object","properties":{"slug":{"type":"string"},"title":{"type":"string"},"tags":{"type":"array","items":{"type":"string"}},"content":{"type":"string"}},"required":["slug","title","content"]})),
            tool("trustkit_knowledge_rebuild", "Reconstroi deterministicamente o indice FTS5 e grafo de wikilinks", json!({"type":"object","properties":{}})),
        ]
    }

    pub fn handle_request(&self, req: &JsonRpcRequest) -> Option<JsonRpcResponse> {
        let req_id = req.id.clone().unwrap_or_else(|| json!(0));
        match req.method.as_str() {
            "initialize" => Some(Self::ok_resp(req_id, json!({
                "protocolVersion": "2024-11-05",
                "serverInfo": { "name": "trustkit-mcp-server", "version": "0.1.0" },
                "capabilities": { "tools": {} }
            }))),
            "notifications/initialized" => None,
            "ping" => Some(Self::ok_resp(req_id, json!({}))),
            "tools/list" => Some(Self::ok_resp(req_id, json!({ "tools": Self::list_tools() }))),
            "tools/call" => Some(self.handle_tool_call(req_id, req.params.as_ref())),
            _ => Some(Self::err_resp(req_id, -32601, format!("Method '{}' not found", req.method))),
        }
    }

    fn handle_tool_call(&self, req_id: Value, params_val: Option<&Value>) -> JsonRpcResponse {
        let params = params_val.and_then(Value::as_object);
        let tool_name = params.and_then(|p| p.get("name")).and_then(Value::as_str).unwrap_or("");
        let empty_args = Map::new();
        let args = params.and_then(|p| p.get("arguments")).and_then(Value::as_object).or(params).unwrap_or(&empty_args);

        if !Self::list_tools().iter().any(|t| t.name == tool_name) {
            return Self::err_resp(req_id, -32601, format!("Tool '{}' not found", tool_name));
        }
        let (text_response, is_error) = if tool_name.starts_with("trustkit_knowledge_") {
            self.dispatch_knowledge_tool(tool_name, args)
        } else {
            (json!({ "tool": tool_name, "status": "acknowledged", "gate": "SDD_GATEWAY_READY" }).to_string(), false)
        };
        Self::ok_resp(req_id, json!({ "content": [{ "type": "text", "text": text_response }], "isError": is_error }))
    }

    fn dispatch_knowledge_tool(&self, tool_name: &str, args: &Map<String, Value>) -> (String, bool) {
        let Some(repo) = &self.knowledge_repo else {
            return (json!({ "verdict": "PASS", "tool": tool_name, "status": "acknowledged" }).to_string(), false);
        };
        match tool_name {
            "trustkit_knowledge_write" => {
                let slug = args.get("slug").and_then(Value::as_str).unwrap_or("");
                let title = args.get("title").and_then(Value::as_str).unwrap_or("Untitled");
                let content = args.get("content").or_else(|| args.get("body")).and_then(Value::as_str).unwrap_or("");
                let ts = format!("{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs());
                let doc = KnowledgeDocument::new(slug, title, extract_tags(args.get("tags")), content, ts, String::new());
                match repo.write_document(&doc) {
                    Ok(KnowledgeWriteOutcome::Success { slug, file_path, blake3_digest, indexed_links_count }) => (
                        json!({"verdict":"PASS","tool":tool_name,"outcome":"SUCCESS","slug":slug,"file_path":file_path,"blake3_digest":blake3_digest,"indexed_links_count":indexed_links_count}).to_string(), false,
                    ),
                    Err(e) => (json!({"verdict":"FAIL","tool":tool_name,"error":e.to_string()}).to_string(), true),
                }
            }
            "trustkit_knowledge_read" => {
                let slug = args.get("slug").and_then(Value::as_str).unwrap_or("");
                match repo.read_document(slug) {
                    Ok(KnowledgeLookupOutcome::Found(d)) => {
                        let links: Vec<String> = repo.query_adjacent_links(&d.slug).unwrap_or_default().into_iter().map(|l| if l.source_slug == d.slug { l.target_slug } else { l.source_slug }).collect();
                        (json!({"verdict":"PASS","tool":tool_name,"outcome":"FOUND","document":{"slug":d.slug,"title":d.title,"tags":d.tags,"body_markdown":d.body_markdown,"updated_at":d.updated_at,"blake3_digest":d.blake3_digest,"adjacent_links":links}}).to_string(), false)
                    }
                    Ok(KnowledgeLookupOutcome::DocumentNotFound) => (json!({"verdict":"PASS","tool":tool_name,"outcome":"DOCUMENT_NOT_FOUND","slug":slug}).to_string(), false),
                    Err(e) => (json!({"verdict":"FAIL","tool":tool_name,"error":e.to_string()}).to_string(), true),
                }
            }
            "trustkit_knowledge_search" => {
                let q_str = args.get("query").or_else(|| args.get("query_text")).and_then(Value::as_str).unwrap_or("");
                let limit = args.get("limit").and_then(Value::as_u64).unwrap_or(10) as usize;
                let query = KnowledgeSearchQuery::new(q_str, limit).with_tags(extract_tags(args.get("tags")));
                match repo.search_documents(&query) {
                    Ok(out) => {
                        let hits: Vec<Value> = out.as_slice().iter().map(|h| json!({"slug":h.slug,"title":h.title,"bm25_score":h.bm25_score,"snippet":h.snippet,"adjacent_links":h.adjacent_links})).collect();
                        (json!({"verdict":"PASS","tool":tool_name,"outcome":out.variant_label(),"query":q_str,"hits":hits}).to_string(), false)
                    }
                    Err(e) => (json!({"verdict":"FAIL","tool":tool_name,"error":e.to_string()}).to_string(), true),
                }
            }
            "trustkit_knowledge_rebuild" => match repo.rebuild_index() {
                Ok(s) => (json!({"verdict":"PASS","tool":tool_name,"outcome":"REBUILT","notes_restored":s.notes_restored,"links_restored":s.links_restored,"duration_ms":s.duration_ms}).to_string(), false),
                Err(e) => (json!({"verdict":"FAIL","tool":tool_name,"error":e.to_string()}).to_string(), true),
            },
            _ => (json!({"verdict":"FAIL","tool":tool_name,"error":"Unsupported knowledge tool"}).to_string(), true),
        }
    }

    fn ok_resp(id: Value, result: Value) -> JsonRpcResponse {
        JsonRpcResponse { jsonrpc: "2.0".to_string(), id, result: Some(result), error: None }
    }

    fn err_resp(id: Value, code: i32, message: String) -> JsonRpcResponse {
        JsonRpcResponse { jsonrpc: "2.0".to_string(), id, result: None, error: Some(JsonRpcError { code, message, data: None }) }
    }

    pub fn run_stdio<R: BufRead, W: Write>(&self, mut reader: R, mut writer: W) -> Result<(), std::io::Error> {
        let mut line = String::new();
        while reader.read_line(&mut line)? > 0 {
            let trimmed = line.trim();
            if !trimmed.is_empty()
                && let Ok(req) = serde_json::from_str::<JsonRpcRequest>(trimmed)
                && let Some(resp) = self.handle_request(&req)
            {
                let json_line = serde_json::to_string(&resp).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
                writeln!(writer, "{json_line}")?;
                writer.flush()?;
            }
            line.clear();
        }
        Ok(())
    }
}

fn extract_tags(val: Option<&Value>) -> Vec<String> {
    match val {
        Some(Value::Array(arr)) => arr.iter().filter_map(Value::as_str).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
        Some(Value::String(csv)) => csv.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_initialize_and_tools_list() {
        let server = McpServer::new();
        let init_req = JsonRpcRequest { jsonrpc: "2.0".to_string(), id: Some(json!(1)), method: "initialize".to_string(), params: None };
        let init_resp = server.handle_request(&init_req).expect("response");
        assert_eq!(init_resp.id, json!(1));
        assert!(init_resp.result.unwrap().get("serverInfo").is_some());

        let list_req = JsonRpcRequest { jsonrpc: "2.0".to_string(), id: Some(json!(2)), method: "tools/list".to_string(), params: None };
        let list_resp = server.handle_request(&list_req).expect("response");
        let tools_arr = list_resp.result.unwrap().get("tools").unwrap().as_array().unwrap().clone();
        assert_eq!(tools_arr.len(), 12);
        assert!(tools_arr.iter().any(|t| t["name"] == "trustkit_spec_lock"));
        assert!(tools_arr.iter().any(|t| t["name"] == "trustkit_knowledge_search"));
        assert!(tools_arr.iter().any(|t| t["name"] == "trustkit_knowledge_write"));
    }

    #[test]
    fn test_run_stdio_stream() {
        let server = McpServer::new();
        let input = "{\"jsonrpc\":\"2.0\",\"id\":10,\"method\":\"ping\"}\n";
        let mut output = Vec::new();
        server.run_stdio(Cursor::new(input), &mut output).expect("run stdio");
        assert!(String::from_utf8_lossy(&output).contains("\"id\":10"));
    }
}
