# TASK-004: Verificacao Integrada de Ciclo Limpo e Integridade de Merge

layer: CLI
depends_on:
  - TASK-003
target_test_file: crates/trustkit-cli/tests/e2e_task_lifecycle_commit_integrity_tests.rs
expected_files:
  - crates/trustkit-cli/tests/e2e_task_lifecycle_commit_integrity_tests.rs

## Contexto
Suite de testes E2E do ciclo completo de vida de tarefas (Red -> Green -> Commit -> Complete -> Merge/Close) sem arquivos orfaos, atendendo RN-004 e RF-004 no componente Node4.

## Criterios de Aceitacao (Gherkin)
- **Dado** uma suite de testes de integracao simulando o ciclo de vida completo de tarefas
- **Quando** todas as fases TDD, commits atomicos e merges forem processados
- **Entao** todos os testes passam sem deixar arquivos orfaos e a branch de trabalho reflete com precisao o historico linear.

## Rastreabilidade
- Regra de Negocio: RN-004
- Requisito Funcional: RF-004
- Evidencia: EV-004-e2e-commit-integrity-cycle.json
