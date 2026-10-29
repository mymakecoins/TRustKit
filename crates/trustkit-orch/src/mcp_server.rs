// MCP stdio server implementing the Model Context Protocol JSON-RPC 2.0 loop.
// Constitutional Principle II: Absolute Prohibition of NULL (skip_serializing_if).
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{BufRead, Write};

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

#[derive(Debug, Default, Clone)]
pub struct McpServer;

impl McpServer {
    pub fn new() -> Self {
        Self
    }

    pub fn list_tools() -> Vec<McpTool> {
        vec![
            McpTool {
                name: "trustkit_init".to_string(),
                description: "Inicializa o ciclo SDD deterministico com ou sem issue vinculada (Gate 0)".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "issue": { "type": "integer", "description": "Numero da issue GitHub opcional" },
                        "from_prd": { "type": "string", "description": "Caminho do arquivo PRD opcional" }
                    }
                }),
            },
            McpTool {
                name: "trustkit_spec_lock".to_string(),
                description: "Valida criterios Gherkin, Mermaid e Matriz de Rastreabilidade e congela com digest BLAKE3 (Gates 1 e 2)".to_string(),
                input_schema: json!({ "type": "object" }),
            },
            McpTool {
                name: "trustkit_wave_plan".to_string(),
                description: "Gera o planejamento topologico de execucao em ondas a partir do DAG de tarefas atomicas (Gate 3)".to_string(),
                input_schema: json!({ "type": "object" }),
            },
            McpTool {
                name: "trustkit_wave_run".to_string(),
                description: "Despacha as tarefas da onda em git worktrees efemeras isoladas (Gate 4)".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "wave": { "type": "integer", "description": "Indice da onda de tarefas" }
                    },
                    "required": ["wave"]
                }),
            },
            McpTool {
                name: "trustkit_task_test".to_string(),
                description: "Executa teste fiscalizado de fase TDD (RED ou GREEN) com registro imutavel no SQLite (Gate 4)".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "task_id": { "type": "string" },
                        "phase": { "type": "string", "enum": ["red", "green"] }
                    },
                    "required": ["task_id", "phase"]
                }),
            },
            McpTool {
                name: "trustkit_gauntlet_run".to_string(),
                description: "Executa a bateria de auditoria do Gauntlet (Camada 0 AST e Camada 1 Swarm de Criticos) (Gate 5)".to_string(),
                input_schema: json!({ "type": "object" }),
            },
            McpTool {
                name: "trustkit_done".to_string(),
                description: "Gera a certificacao do ciclo, renderiza o gauntlet-report.md e formata o comentario de conclusao (Gate 6)".to_string(),
                input_schema: json!({ "type": "object" }),
            },
            McpTool {
                name: "trustkit_pr".to_string(),
                description: "Abre o Pull Request sob demanda no GitHub com rastreabilidade completa e move card para In review".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "title": { "type": "string" },
                        "custom_body": { "type": "string" }
                    }
                }),
            },
        ]
    }

    pub fn handle_request(&self, req: &JsonRpcRequest) -> Option<JsonRpcResponse> {
        let req_id = req.id.clone().unwrap_or(Value::Null);

        match req.method.as_str() {
            "initialize" => Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: req_id,
                result: Some(json!({
                    "protocolVersion": "2024-11-05",
                    "serverInfo": {
                        "name": "trustkit-mcp-server",
                        "version": "0.1.0"
                    },
                    "capabilities": {
                        "tools": {}
                    }
                })),
                error: None,
            }),
            "notifications/initialized" => None,
            "ping" => Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: req_id,
                result: Some(json!({})),
                error: None,
            }),
            "tools/list" => Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: req_id,
                result: Some(json!({
                    "tools": Self::list_tools()
                })),
                error: None,
            }),
            "tools/call" => {
                let params = req.params.as_ref().and_then(|p| p.as_object());
                let tool_name = params
                    .and_then(|p| p.get("name"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("");

                let tools = Self::list_tools();
                if let Some(tool) = tools.iter().find(|t| t.name == tool_name) {
                    let text_response = json!({
                        "tool": tool.name,
                        "status": "acknowledged",
                        "gate": "SDD_GATEWAY_READY"
                    }).to_string();

                    Some(JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: req_id,
                        result: Some(json!({
                            "content": [
                                {
                                    "type": "text",
                                    "text": text_response
                                }
                            ],
                            "isError": false
                        })),
                        error: None,
                    })
                } else {
                    Some(JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: req_id,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32601,
                            message: format!("Tool '{}' not found", tool_name),
                            data: None,
                        }),
                    })
                }
            }
            _ => Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: req_id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: format!("Method '{}' not found", req.method),
                    data: None,
                }),
            }),
        }
    }

    pub fn run_stdio<R: BufRead, W: Write>(&self, mut reader: R, mut writer: W) -> Result<(), std::io::Error> {
        let mut line = String::new();
        while reader.read_line(&mut line)? > 0 {
            let trimmed = line.trim();
            if !trimmed.is_empty()
                && let Ok(req) = serde_json::from_str::<JsonRpcRequest>(trimmed)
                && let Some(resp) = self.handle_request(&req)
            {
                let json_line = serde_json::to_string(&resp).map_err(|e| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, e)
                })?;
                writeln!(writer, "{}", json_line)?;
                writer.flush()?;
            }
            line.clear();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_initialize_and_tools_list() {
        let server = McpServer::new();
        let init_req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "initialize".to_string(),
            params: None,
        };

        let init_resp = server.handle_request(&init_req).expect("response");
        assert_eq!(init_resp.id, json!(1));
        assert!(init_resp.result.unwrap().get("serverInfo").is_some());

        let list_req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(2)),
            method: "tools/list".to_string(),
            params: None,
        };

        let list_resp = server.handle_request(&list_req).expect("response");
        let tools_arr = list_resp.result.unwrap().get("tools").unwrap().as_array().unwrap().clone();
        assert_eq!(tools_arr.len(), 8);
        assert!(tools_arr.iter().any(|t| t["name"] == "trustkit_spec_lock"));
    }

    #[test]
    fn test_run_stdio_stream() {
        let server = McpServer::new();
        let input = "{\"jsonrpc\":\"2.0\",\"id\":10,\"method\":\"ping\"}\n";
        let mut output = Vec::new();

        server.run_stdio(Cursor::new(input), &mut output).expect("run stdio");
        let output_str = String::from_utf8_lossy(&output);
        assert!(output_str.contains("\"id\":10"));
    }
}
