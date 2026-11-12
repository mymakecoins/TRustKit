# TASK-004: Validador de Sequencia Expand-Migrate-Contract no WavePlanner

layer: AST
depends_on: ["TASK-003"]
target_test_file: crates/trustkit-ast/tests/ast_facade_tests.rs
expected_files:
  - crates/trustkit-ast/src/ast_facade.rs
  - crates/trustkit-ast/tests/ast_facade_tests.rs

## Contexto
Validador de Sequencia Expand-Migrate-Contract no WavePlanner atendendo RN-004 e RF-004 no componente Node4.

## Criterios de Aceitacao (Gherkin)
- **Dado** um plano de tarefas onde uma tarefa `WideRefactorContract` nao declara dependencia de todos os lotes `WideRefactorMigrateBatch` do mesmo topico
- **Quando** o validador de topologia do Gate 3 e executado
- **Entao** o comando `trustkit wave plan` rejeita o grafo com erro de violacao topologica informando exatamente qual aresta bloqueante esta faltando.

## Rastreabilidade
- Regra de Negocio: RN-004
- Requisito Funcional: RF-004
- Evidencia: EV-004-contract-edge-validation.json
