# TASK-008: Implementar Ferramentas MCP de Monitoramento e Controle de Terminais

> **Camada:** Orquestracao / MCP
> **Estimativa:** M
> **Status:** Pendente

depends_on: [TASK-006]
layer: ORCH
Arquivos Esperados:
- crates/trustkit-orch/src/mcp_terminal_dispatch.rs
- crates/trustkit-orch/src/mcp_server.rs
test_file: crates/trustkit-orch/tests/mcp_terminal_tools_tests.rs

---

## Contexto
Quando o TRustKit opera como servidor MCP (`trustkit mcp serve`) atendendo agentes de nivel superior (Claude Code, Antigravity, Cursor), esses agentes precisam ser capazes de inspecionar o progresso das tarefas da onda paralela, verificar se ha builders bloqueados por permissoes e enviar entradas para destravar a execucao diretamente via chamadas de ferramentas JSON-RPC 2.0.

## Escopo
Implementar em `crates/trustkit-orch/src/mcp_terminal_dispatch.rs` e registrar em `crates/trustkit-orch/src/mcp_server.rs`:
- Ferramenta `trustkit_wave_status`:
  - Retorna o status de todas as tarefas da onda ativa, runtime selecionado e contagem de terminais ativos/bloqueados/concluidos.
- Ferramenta `trustkit_terminal_inspect`:
  - Parametros: `task_id: String`, `lines: Option<usize>`.
  - Retorna o estado semantico atual (`Working`, `Blocked`, `Done`), o motivo do bloqueio (se houver) e o buffer recente sanitizado.
- Ferramenta `trustkit_terminal_send`:
  - Parametros: `task_id: String`, `input: String`.
  - Envia a resposta ou confirmacao requerida ao stdin do terminal da tarefa bloqueada para destravar a execucao.
- Validacao estrita de schemas de ferramentas sem propriedades `null`.

### Fora do escopo
- Streaming continuo de video ou graficos de terminal (apenas snapshots de texto sob demanda).

## Dependencias
- TASK-006

## Premissas
- Despacho em < 5ms no servidor Tokio.
- Sanitizacao automatica de qualquer texto retornado ao cliente MCP.

## Criterio de Aceite
- [ ] **Dado** uma chamada JSON-RPC 2.0 para `trustkit_terminal_inspect` com `task_id` valido, **quando** despachada, **entao** retorna payload estruturado com estado semantico e buffer recente.
- [ ] **Dado** uma chamada para `trustkit_terminal_send` para tarefa inexistente, **quando** executada, **entao** retorna erro estruturado JSON-RPC sem derrubar o servidor.

## Notas Tecnicas
- Seguir o padrao de `crates/trustkit-orch/src/mcp_spec_dispatch.rs` e `crates/trustkit-orch/src/mcp_server.rs`.

## Rastreabilidade
- Origem: RF-060, RN-008
