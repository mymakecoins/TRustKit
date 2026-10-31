// Integration tests for MCP spec tools (TASK-006).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use serde_json::json;
use std::time::Instant;
use trustkit_orch::mcp_server::{JsonRpcRequest, McpServer};

const VALID_SPEC: &str = r#"# Especificacao: 009-sample-feature

## 1. Product Trio

- **PM**: Regras de negocio definidas.
- **Tech Lead**: Arquitetura hexagonal.
- **Design / QA**: Criterios de aceitacao BDD.

## 2. Diagrama de Fluxo (Mermaid)

```mermaid
flowchart LR
    Node1[Entrada] --> Node2[Saida]
```

## 3. Criterios de Aceitacao (Gherkin)

### Cenario 1: Execucao Nominal

- **Dado** ambiente pronto
- **Quando** executa
- **Entao** conclui com sucesso

## 4. Matriz de Rastreabilidade

| Regra Negocio | Requisito Funcional | Tarefa | No Mermaid | Teste Unitario / Integracao | Evidencia / Verificacao |
| :--- | :--- | :--- | :--- | :--- | :--- |
| RN-001 | RF-001 | TASK-001 | Node1 | tests/unit.rs | EV-001.json |
"#;

#[test]
fn test_mcp_spec_template_tool_call() {
    let server = McpServer::new();
    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(101)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "trustkit_spec_template",
            "arguments": {
                "feature_slug": "010-oauth-flow",
                "topic_idea": "Integracao OAuth2 RFC 6749"
            }
        })),
    };

    let start = Instant::now();
    let resp = server.handle_request(&req).expect("valid response");
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_millis() < 5,
        "MCP tool dispatch exceeded 5ms budget: {:?}",
        elapsed
    );

    let result = resp.result.expect("result object");
    let content_text = result["content"][0]["text"].as_str().expect("text string");
    assert!(content_text.contains("\"verdict\":\"PASS\""));
    assert!(content_text.contains("010-oauth-flow"));
    assert!(content_text.contains("## 1. Product Trio"));
    assert!(content_text.contains("flowchart LR"));
}

#[test]
fn test_mcp_spec_context_tool_call() {
    let server = McpServer::new();
    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(102)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "trustkit_spec_context",
            "arguments": {
                "idea": "Autenticacao",
                "hops": 2,
                "max_snippets": 3
            }
        })),
    };

    let resp = server.handle_request(&req).expect("valid response");
    let result = resp.result.expect("result object");
    let content_text = result["content"][0]["text"].as_str().expect("text string");
    assert!(content_text.contains("\"verdict\":\"PASS\""));
    assert!(!content_text.contains(": null"));
}

#[test]
fn test_mcp_spec_lint_tool_call_nominal_and_violation() {
    let server = McpServer::new();

    // 1. Lint valido
    let valid_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(103)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "trustkit_spec_lint",
            "arguments": {
                "content": VALID_SPEC
            }
        })),
    };

    let resp_valid = server.handle_request(&valid_req).expect("valid response");
    let text_valid = resp_valid.result.unwrap()["content"][0]["text"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(text_valid.contains("\"verdict\":\"PASS\""));
    assert!(text_valid.contains("\"violations_count\":0"));

    // 2. Lint com violacao
    let invalid_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(104)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "trustkit_spec_lint",
            "arguments": {
                "content": "Spec vazia sem nada"
            }
        })),
    };

    let resp_invalid = server.handle_request(&invalid_req).expect("invalid response");
    let text_invalid = resp_invalid.result.unwrap()["content"][0]["text"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(text_invalid.contains("\"verdict\":\"FAIL\""));
    assert!(text_invalid.contains("SPEC-001"));
}
