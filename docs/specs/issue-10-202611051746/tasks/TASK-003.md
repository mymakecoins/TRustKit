# TASK-003: Orquestrador de Multiplas Ondas em Worktrees com Loop TDD

> **Camada:** Orquestracao
> **Estimativa:** G
> **Status:** Pendente

depends_on: [TASK-002]
layer: ORCH
Arquivos Esperados:
- crates/trustkit-orch/src/autonomous_wave_orchestrator.rs
test_file: crates/trustkit-orch/tests/multi_wave_autonomous_orchestrator_tests.rs

---

## Contexto
O Gate 4 no modo autonomo deve iterar sobre todas as ondas identificadas pelo DAG. Para cada onda, deve provisionar o isolamento das worktrees, orquestrar para cada tarefa o ciclo RED (comprovacao de falha), disparar o agente/implementacao, registrar GREEN, executar review e fechar a tarefa com complete.

## Escopo
- Implementar `AutonomousWaveOrchestrator` em `trustkit-orch`.
- Suportar a iteracao sequencial por todas as ondas (ex: Wave 1, Wave 2, Wave 3).
- Integrar com o supervisor de tarefas e verificar que nenhuma tarefa pula o estagio RED.
