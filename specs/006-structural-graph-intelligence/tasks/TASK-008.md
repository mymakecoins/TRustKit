# TASK-008: Expor Subcomandos CLI trustkit graph e Tools Nativas no Servidor MCP

> **Camada:** Interface CLI & Orquestracao MCP
> **Estimativa:** G
> **Status:** Concluido

depends_on: [TASK-007]
layer: CLI
Arquivos Esperados:
- crates/trustkit-cli/src/commands/graph_cmd.rs
- crates/trustkit-orch/src/mcp_server.rs
test_file: crates/trustkit-cli/tests/mcp_graph_tools_tests.rs

---

## Contexto
Para disponibilizar a inteligencia de grafo estrutural tanto a desenvolvedores humanos no terminal quanto a agentes autonomos via Model Context Protocol (MCP stdio), e necessario implementar o subcomando `trustkit graph`, estender `trustkit context slice` com as flags de travessia e registrar as tools `trustkit_graph_*` no servidor MCP.

## Escopo
Implementar:
- Em `crates/trustkit-cli/src/commands/graph_cmd.rs`: handlers para:
  - `trustkit graph build [--update] [--force] [--json]`
  - `trustkit graph health [--json]`
  - `trustkit graph path <SOURCE> <TARGET> [--json]`
  - `trustkit graph explain <NODE_ID> [--json]`
  - Suporte as flags `--mode <bfs|dfs>` e `--hops <N>` em `trustkit context slice`.
- Em `crates/trustkit-orch/src/mcp_server.rs`: registro e despacho das tools MCP `trustkit_graph_query`, `trustkit_graph_path`, `trustkit_graph_explain` e `trustkit_graph_health`.

### Fora do escopo
- Renderizadores HTML pesados com bibliotecas JavaScript externas.

## Dependencias
- TASK-007

## Premissas
- Tempo de resposta para consultas no grafo e chamadas MCP inferior a `5ms`.
- Payloads JSON estritamente livres de `null` e saida humana ASCII livre de emojis.
- Limite maximo de 250 linhas de codigo por arquivo.

## Criterio de Aceite
- [x] **Dado** a execucao de `trustkit graph build --json` seguida de `trustkit graph explain <NODE_ID> --json`, **entao** o CLI retorna `Exit Code: 0` com JSON estruturado sem campos `null`.
- [x] **Dado** uma requisicao JSON-RPC `tools/list` ao servidor MCP (`trustkit mcp`), **entao** as tools `trustkit_graph_query`, `trustkit_graph_path`, `trustkit_graph_explain` e `trustkit_graph_health` sao listadas e respondem a `tools/call` em `< 5ms`.

## Notas Tecnicas
- Conectar os novos adaptadores no Composition Root (`crates/trustkit-cli/src/bootstrap.rs`).

## Rastreabilidade
- Origem: RF-032, RN-008
