# TASK-008: Expor Comandos CLI trustkit memory e Tools Nativas no Servidor MCP

> **Camada:** Interface CLI & Orquestracao MCP
> **Estimativa:** G
> **Status:** Concluida

depends_on: [TASK-005, TASK-006, TASK-007]
layer: CLI
Arquivos Esperados:
- crates/trustkit-cli/src/commands/memory_cmd.rs
- crates/trustkit-orch/src/mcp_server.rs
test_file: crates/trustkit-cli/tests/mcp_knowledge_tools_tests.rs

---

## Contexto
Para que tanto desenvolvedores humanos quanto agentes autonomos (via CLI ou protocolo MCP stdio) possam consultar, gravar, listar e reconstruir a memoria perene do repositorio, e necessario expor o subcomando `trustkit memory` e registrar as tools correspondentes no servidor MCP nativo.

## Escopo
Implementar:
- Em `crates/trustkit-cli/src/commands/memory_cmd.rs`: handlers para `trustkit memory write`, `trustkit memory read`, `trustkit memory search`, `trustkit memory list` e `trustkit memory rebuild` com suporte integral ao modo `--json` livre de `null`.
- Em `crates/trustkit-orch/src/mcp_server.rs`: registro e roteamento das tools `trustkit_knowledge_search`, `trustkit_knowledge_read`, `trustkit_knowledge_write` e `trustkit_knowledge_rebuild`.

### Fora do escopo
- Interface grafica web externa.

## Dependencias
- TASK-005
- TASK-006
- TASK-007

## Premissas
- Tempo de resposta para subcomandos locais e chamadas MCP inferior a `5ms`.
- Conformidade com os codigos de saida POSIX do TRustKit.

## Criterio de Aceite
- [x] **Dado** o servidor MCP ativo ou invocacao via CLI, **quando** `trustkit memory write` seguido de `trustkit memory search` sao executados, **entao** o documento e persistido em `.trustkit/wiki/` e retornado no payload JSON ranqueado sem valores nulos.
- [x] **Dado** a invocacao JSON-RPC `tools/list` no servidor MCP, **entao** as tools `trustkit_knowledge_*` constam com seus respectivos schemas JSON de entrada.

## Notas Tecnicas
- Arquivos alvo: `crates/trustkit-cli/src/commands/memory_cmd.rs` e `crates/trustkit-orch/src/mcp_server.rs`.

## Rastreabilidade
- Origem: RF-024, RN-008
