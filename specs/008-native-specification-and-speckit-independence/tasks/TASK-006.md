# TASK-006: Implementar Ferramentas MCP Nativas de Especificacao

> **Camada:** Orquestracao / Protocolo
> **Estimativa:** M
> **Status:** Pendente

depends_on: [TASK-005]
layer: ORCH
Arquivos Esperados:
- crates/trustkit-orch/src/mcp_spec_dispatch.rs
- crates/trustkit-orch/src/mcp_server.rs
test_file: crates/trustkit-orch/tests/mcp_spec_tools_tests.rs

---

## Contexto
O servidor MCP nativo do TRustKit (`trustkit mcp serve`) precisa disponibilizar ferramentas dedicadas de especificacao para que clientes agênticos (Claude Code, Antigravity, Cursor) possam obter templates, contexto e linters diretamente via JSON-RPC sem recorrer a ferramentas de terceiros.

## Escopo
Implementar em `crates/trustkit-orch/src/mcp_spec_dispatch.rs` e registrar em `crates/trustkit-orch/src/mcp_server.rs`:
- Ferramenta `trustkit_spec_template`: retorna o template estruturado e instrucoes constitutionais para um novo `spec.md`.
- Ferramenta `trustkit_spec_context`: recebe `{ "idea": "...", "hops": 2 }` e retorna os artigos wiki e slices AST correlacionados.
- Ferramenta `trustkit_spec_lint`: recebe `{ "spec_content": "..." }` ou `{ "spec_file": "..." }` e retorna a analise lexico-estrutural.
- Integracao das schemas JSON no handler `tools/list` do servidor MCP.

### Fora do escopo
- Mutacao de arquivos sem comando explicito.

## Dependencias
- TASK-005

## Premissas
- Protocolo JSON-RPC 2.0 estrito.
- Resposta a chamadas de ferramentas em menos de 5ms.
- 100% livre de valores `null`.

## Criterio de Aceite
- [ ] **Dado** uma requisicao MCP `tools/list`, **quando** o servidor responde, **entao** inclui `trustkit_spec_template`, `trustkit_spec_context` e `trustkit_spec_lint` com schemas completos de entrada e saida.
- [ ] **Dado** uma chamada `tools/call` para `trustkit_spec_lint` com texto de spec valido, **quando** despachada, **entao** retorna resultado limpo sem violacoes.

## Notas Tecnicas
- Seguir o design de `crates/trustkit-orch/src/mcp_graph_dispatch.rs`.

## Rastreabilidade
- Origem: RF-048, RN-006
