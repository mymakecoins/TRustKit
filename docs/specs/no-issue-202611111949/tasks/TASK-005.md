# TASK-005: Comandos CLI e Ferramentas MCP memory glossary e adr-check

> **Camada:** CLI
> **Estimativa:** L
> **Status:** Pendente

depends_on: ['TASK-001', 'TASK-002', 'TASK-003', 'TASK-004']
layer: CLI
Arquivos Esperados:
- crates/trustkit-cli/src/commands/memory_cmd.rs
- crates/trustkit-cli/src/mcp_server.rs
test_file: crates/trustkit-cli/tests/mcp_knowledge_tools_tests.rs

---

## Contexto
Expor os recursos de Glossario de Linguagem Ubiqua e Triagem de ADR tanto na interface CLI (`trustkit memory glossary` e `trustkit memory adr-check`) quanto no protocolo MCP nativo.

## Escopo
- Adicionar subcomandos `glossary add`, `glossary list` e `adr-check` sob `trustkit memory`.
- Registrar ferramentas correspondentes no servidor MCP nativo (`crates/trustkit-cli/src/mcp_server.rs`).
- Garantir saidas JSON e formatadas sem valores nulos e sem emojis.
- Implementar testes de ponta a ponta e MCP em `crates/trustkit-cli/tests/mcp_knowledge_tools_tests.rs`.
