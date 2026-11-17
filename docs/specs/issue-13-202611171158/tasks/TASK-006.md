# TASK-006: Ferramenta MCP trustkit_wave_view

layer: ORCH
depends_on: ["TASK-002"]
target_test_file: crates/trustkit-orch/tests/mcp_wave_view_tool_tests.rs
expected_files:
  - crates/trustkit-orch/src/mcp_wave_view_tool.rs
  - crates/trustkit-orch/tests/mcp_wave_view_tool_tests.rs

## Contexto
Ferramenta MCP trustkit_wave_view atendendo RN-006 e RF-067 no componente Node4.

## Criterios de Aceitacao (Gherkin)
- **Dado** o servidor MCP nativo rodando via `trustkit mcp serve`
- **Quando** um agente de desenvolvimento ou orquestrador invocar a tool `trustkit_wave_view` via JSON-RPC 2.0
- **Entao** o servidor MCP retorna a estrutura completa das ondas do ciclo ativo com tempo de resposta inferior a 10ms.

## Rastreabilidade
- Regra de Negocio: RN-006
- Requisito Funcional: RF-067
- Evidencia: EV-006-mcp-wave-view-tool.json
