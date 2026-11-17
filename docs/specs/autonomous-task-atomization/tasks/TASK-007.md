# TASK-007: Dispatch do Agente Construtor entre as Fases RED e GREEN

layer: ORCH
depends_on: ["TASK-005", "TASK-006"]
target_test_file: crates/trustkit-orch/tests/builder_agent_dispatch_tests.rs
expected_files:
  - crates/trustkit-orch/src/builder_agent_dispatcher.rs
  - crates/trustkit-orch/tests/builder_agent_dispatch_tests.rs

## Contexto
Introduzir no `AutonomousWaveOrchestrator` o passo de invocacao do `BuilderAgentDispatcherPort` imediatamente apos a comprovacao da fase RED e antes da afericao da fase GREEN, com isolamento de locks de compilacao do Cargo, permitindo a implementacao autonoma do codigo de producao na worktree.

## Criterios de Aceitacao (Gherkin)
- **Dado** que a tarefa atomica teve sua evidencia da fase RED validada e gravada no SQLite WAL
- **Quando** o `AutonomousWaveOrchestrator` prosseguir para a construcao da solucao
- **Entao** deve despachar o agente construtor na worktree isolada antes de executar o teste de verificacao da fase GREEN.

## Rastreabilidade
- Regra de Negocio: RN-007
- Requisito Funcional: RF-007
- Evidencia: EV-007-builder-agent-dispatch.json
