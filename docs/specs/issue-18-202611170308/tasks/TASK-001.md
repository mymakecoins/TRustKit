# TASK-001: Auditoria e Suite de Testes E2E de Commits de Tarefas

layer: CLI
depends_on: []
target_test_file: crates/trustkit-cli/tests/task_complete_dirty_guard_tests.rs
expected_files:
  - crates/trustkit-cli/src/commands/task_cmd.rs
  - crates/trustkit-cli/tests/task_complete_dirty_guard_tests.rs

## Contexto
Auditoria e verificacao automatizada de rejeicao de conclusao de tarefas com arquivos pendentes no working tree, atendendo RN-001 e RF-001 no componente Node1.

## Criterios de Aceitacao (Gherkin)
- **Dado** uma tarefa atomica ativa em execucao com alteracoes locais pendentes no working tree
- **Quando** o comando `trustkit task complete` for executado sem que um commit Git tenha sido realizado
- **Entao** a operacao falha deterministicamente com violacao de dominio e o estado da tarefa permanece nao concluido no SQLite.

## Rastreabilidade
- Regra de Negocio: RN-001
- Requisito Funcional: RF-001
- Evidencia: EV-001-task-complete-dirty-rejected.json
