# TASK-006: Implementar Orquestracao de Ondas Interativas no Gate 4

> **Camada:** Caso de Uso
> **Estimativa:** L
> **Status:** Pendente

depends_on: [TASK-005]
layer: CORE
Arquivos Esperados:
- crates/trustkit-core/src/use_cases/run_interactive_wave.rs
test_file: crates/trustkit-core/tests/run_interactive_wave_tests.rs

---

## Contexto
O Gate 4 do TRustKit governa a execucao paralela em Git Worktrees. Com a adicao do multiplexador de PTYs, o caso de uso deve coordenar o ciclo de vida completo de cada tarefa da onda: provisionar a worktree, instanciar o agente construtor interativo, monitorar seus estados semanticos (`Working`, `Blocked`, `Done`), sanitizar logs de auditoria e so entao proceder para a fase de testes e merge.

## Escopo
Implementar em `crates/trustkit-core/src/use_cases/run_interactive_wave.rs`:
- Estrutura `RunInteractiveWaveRequest`: `cycle_id: String`, `wave_index: u32`, `runtime_mode: RuntimeBackendMode`, `agent_command: String`, `now_iso: String`.
- Estrutura `RunInteractiveWaveOutcome`: `cycle_id: String`, `wave_index: u32`, `tasks_dispatched: Vec<InteractiveTaskReport>`, `overall_verdict: GateVerdict`.
- Fluxo de coordenacao do caso de uso:
  1. Validar selo criptografico `FROZEN` da especificacao (RN-007).
  2. Filtrar tarefas atômicas da onda solicitada.
  3. Para cada tarefa: provisionar Git Worktree via `WorktreeIsolationPort`.
  4. Despachar o terminal interativo via `TerminalMultiplexerPort` configurado para o diretório da worktree.
  5. Monitorar estado via polling periodico ou notificacoes ate `Done` ou limite de timeout.
  6. Caso o agente entre em `Blocked`, registrar o evento e permitir que orquestradores superiores intervenham.
  7. Apos `Done`, sanitizar buffer de terminal e gravar evidencia de auditoria no SQLite WAL.
  8. Executar merge deterministico e limpeza de worktree.

### Fora do escopo
- Decisao autonoma sobre perguntas de negocio do agente (deve escalar via estado `Blocked`).

## Dependencias
- TASK-005

## Premissas
- Isolamento estrito entre tarefas concorrentes da mesma onda.
- Gravacao transacional de cada evidencia de transicao de estado.

## Criterio de Aceite
- [ ] **Dado** uma onda com 2 tarefas e provedor de multiplexacao mock retornando `Done`, **quando** o caso de uso e executado, **entao** ambas as worktrees sao alocadas, terminais concluidos, buffers sanitizados e merged com veredito `PASS`.
- [ ] **Dado** um agente que reporta `AgentSemanticState::Blocked`, **quando** monitorado, **entao** o caso de uso suspende a conclusao daquela tarefa sem corromper as demais tarefas paralelas da onda.

## Notas Tecnicas
- Seguir o isolamento de `crates/trustkit-core/src/use_cases/run_execution_wave.rs`.

## Rastreabilidade
- Origem: RF-058, RN-006
