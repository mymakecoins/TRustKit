# TASK-002: Blindagem de Conclusao de Tarefa com Working Tree Sujo

layer: ORCH
depends_on:
  - TASK-001
target_test_file: crates/trustkit-orch/tests/worktree_merge_dirty_guard_tests.rs
expected_files:
  - crates/trustkit-orch/src/worktree_provider.rs
  - crates/trustkit-orch/tests/worktree_merge_dirty_guard_tests.rs

## Contexto
Validacao de limpeza da worktree antes de merge_and_prune_worktree, impedindo a perda de arquivos e rejeitando worktree suja com DomainViolation, atendendo RN-002 e RF-002 no componente Node2.

## Criterios de Aceitacao (Gherkin)
- **Dado** uma worktree provisionada para uma tarefa atomica contendo arquivos modificados ou nao rastreados
- **Quando** o provedor de isolamento tentar executar `merge_and_prune_worktree` para a branch alvo
- **Entao** o procedimento falha imediatamente com erro semantico impedindo a perda de arquivos ou contaminacao da branch alvo.

## Rastreabilidade
- Regra de Negocio: RN-002
- Requisito Funcional: RF-002
- Evidencia: EV-002-worktree-merge-dirty-rejected.json
