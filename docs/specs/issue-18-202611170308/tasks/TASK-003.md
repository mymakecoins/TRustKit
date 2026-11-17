# TASK-003: Validacao de Limpeza de Worktree em Merge and Prune

layer: CLI
depends_on:
  - TASK-002
target_test_file: crates/trustkit-cli/tests/task_complete_clean_commit_tests.rs
expected_files:
  - crates/trustkit-cli/src/commands/task_cmd.rs
  - crates/trustkit-core/src/use_cases/complete_task.rs
  - crates/trustkit-cli/tests/task_complete_clean_commit_tests.rs

## Contexto
Conclusao de tarefa bem-sucedida com commit atomico persistido no Git e comprovacao de working tree limpo, atendendo RN-003 e RF-003 no componente Node3.

## Criterios de Aceitacao (Gherkin)
- **Dado** uma tarefa atomica com todas as alteracoes devidamente commitadas e working tree limpo
- **Quando** o comando `trustkit task complete` for executado
- **Entao** o SHA do commit recente e registrado com sucesso no SQLite e o estado da tarefa avanca para Committed.

## Rastreabilidade
- Regra de Negocio: RN-003
- Requisito Funcional: RF-003
- Evidencia: EV-003-task-complete-clean-accepted.json
