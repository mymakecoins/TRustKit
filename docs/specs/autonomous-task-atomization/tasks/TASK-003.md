# TASK-003: Circuit Breaker EmptyTaskBacklogFault contra Ciclos Vazios no Gate 3

layer: CORE
depends_on: []
target_test_file: crates/trustkit-core/tests/empty_backlog_circuit_breaker_tests.rs
expected_files:
  - crates/trustkit-core/src/use_cases/plan_execution_waves.rs
  - crates/trustkit-core/tests/empty_backlog_circuit_breaker_tests.rs

## Contexto
Implementar no dominio central o Circuit Breaker que impede terminantemente que um ciclo SDD avance para o Gate 4 (Execucao de Ondas), Gate 5 (Gauntlet) ou Gate 6 (Certificacao/Merge) quando o backlog de tarefas atomicas resultar vazio (`total_tasks == 0`).

## Criterios de Aceitacao (Gherkin)
- **Dado** um ciclo ativo que apos a leitura e tentativa de atomizacao possua zero tarefas atomicas
- **Quando** o motor avaliar a transicao do Gate 3 para o Gate 4
- **Entao** deve emitir a violacao `EmptyTaskBacklogFault` e abortar imediatamente o fluxo com codigo de saida nao-zero.

## Rastreabilidade
- Regra de Negocio: RN-003
- Requisito Funcional: RF-003
- Evidencia: EV-003-circuit-breaker-empty-tasks.json
