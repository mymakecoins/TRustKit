# TASK-003: Classificador BlastRadius e TaskSliceKind

layer: AST
depends_on: ["TASK-002"]
target_test_file: crates/trustkit-ast/tests/wave_planner_tests.rs
expected_files:
  - crates/trustkit-ast/src/wave_planner.rs
  - crates/trustkit-ast/tests/wave_planner_tests.rs

## Contexto
Classificador BlastRadius e TaskSliceKind atendendo RN-003 e RF-003 no componente Node3.

## Criterios de Aceitacao (Gherkin)
- **Dado** um conjunto de tarefas anotadas ou estruturadas como `WideRefactorExpand`, `WideRefactorMigrateBatch` e `WideRefactorContract`
- **Quando** o `WaveExecutionPlanner` calcula o DAG de execucao
- **Entao** a tarefa `Expand` e alocada isoladamente na primeira onda, todas as tarefas `MigrateBatch` rodam em paralelo na onda subsequente bloqueadas pela `Expand`, e a tarefa `Contract` e alocada na onda final bloqueada por todos os lotes `MigrateBatch`.

## Rastreabilidade
- Regra de Negocio: RN-003
- Requisito Funcional: RF-003
- Evidencia: EV-003-expand-contract-dag-ordering.json
