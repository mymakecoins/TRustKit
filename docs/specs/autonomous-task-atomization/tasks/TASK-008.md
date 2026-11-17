# TASK-008: Merge Acumulativo Seguro de Worktrees sem Force Reset Destrutivo

layer: ORCH
depends_on: []
target_test_file: crates/trustkit-orch/tests/safe_worktree_merge_tests.rs
expected_files:
  - crates/trustkit-orch/src/worktree_provider.rs
  - crates/trustkit-orch/tests/safe_worktree_merge_tests.rs

## Contexto
Substituir no `WorktreeIsolationProvider::merge_and_prune_worktree` o comando destrutivo `git branch -f` e o merge silencioso por uma integracao acumulativa verificada que preserva todos os commits de multiplas tarefas na mesma onda e detecta conflitos de merge explicitamente.

## Criterios de Aceitacao (Gherkin)
- **Dado** multiplas worktrees de uma mesma onda que produziram commits independentes
- **Quando** `merge_and_prune_worktree` integrar cada worktree ao branch alvo
- **Entao** todos os commits das tarefas anteriores devem ser preservados no historico do branch sem sobrescrita.

## Rastreabilidade
- Regra de Negocio: RN-008
- Requisito Funcional: RF-008
- Evidencia: EV-008-safe-worktree-merge.json
