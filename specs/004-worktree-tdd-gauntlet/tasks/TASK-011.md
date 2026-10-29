# TASK-011: Implementar Servidor MCP stdio para Ferramentas de SDD do Agente Hospedeiro

> **Camada:** Integracao  
> **Estimativa:** M  
> **Status:** Concluido  

---

## Contexto
O TRustKit adota a arquitetura "Hybrid Rust Core + Thin Skill". Para que agentes de IA hospedeiros (Claude Code, Antigravity, Cursor) possam invocar diretamente as ferramentas determinísticas sem passar pelo overhead de parses complexos de saída de terminal, o binário nativo deve expor opcionalmente um servidor Model Context Protocol (MCP) sobre `stdio`.

## Escopo
Implementar em `crates/trustkit-orch/src/mcp_server.rs`:
- Loop de comunicação JSON-RPC 2.0 sobre `stdin` e `stdout` conforme especificação MCP.
- Registro de ferramentas:
  - `trustkit_init`: inicialização de ciclo.
  - `trustkit_spec_lock`: validação e selagem da especificação.
  - `trustkit_wave_plan`: planejamento topológico de ondas.
  - `trustkit_wave_run`: despacho de onda em worktrees.
  - `trustkit_task_test`: execução fiscalizada de fase TDD (RED / GREEN).
  - `trustkit_gauntlet_run`: execução do Gauntlet Camadas 0 e 1.
  - `trustkit_done`: certificação e geração de relatório.
  - `trustkit_pr`: abertura de PR sob demanda.
- Respostas minificadas, sem `null` e sem códigos de escape ANSI.

### Fora do escopo
- Servidor HTTP ou SSE (foco exclusivo em `stdio` local para agentes hospedeiros).

## Dependencias
- TASK-002 — Requer as portas e casos de uso de domínio.

## Premissas
- O servidor MCP comunica-se exclusivamente via `stdin` e `stdout`, utilizando `stderr` para logs de diagnóstico.

## Criterio de Aceite
- [x] **Dado** uma mensagem de `initialize` MCP enviada via `stdin`, **quando** processada, **entao** o servidor responde com a lista completa de ferramentas SDD disponíveis.
- [x] **Dado** uma chamada de ferramenta `trustkit_spec_lock` via MCP, **quando** executada, **entao** o resultado retorna estruturado em formato JSON sem travar o agente hospedeiro.

## Notas Tecnicas
- Arquivo: `crates/trustkit-orch/src/mcp_server.rs`.
- Seguir o padrão de mensageria assíncrona do protocolo MCP.

## Rastreabilidade
- Origem: RF-006, RF-008, Secao 5.1.4 do PRD
