# TASK-001: Detalhes Estruturados de Provisionamento no WorktreeIsolationProvider

layer: ORCH
depends_on: []
target_test_file: crates/trustkit-orch/tests/worktree_provision_notification_tests.rs
expected_files:
  - crates/trustkit-orch/src/worktree_provider.rs
  - crates/trustkit-core/src/special_cases.rs
  - crates/trustkit-core/src/ports.rs
  - crates/trustkit-orch/tests/worktree_provision_notification_tests.rs

## Contexto
Estruturar o retorno e informacoes de provisionamento no WorktreeIsolationProvider detalhando a branch criada da tarefa, o caminho isolado da worktree e a branch base preservada, atendendo RN-001 e RF-001 no componente Node1.

## Criterios de Aceitacao (Gherkin)
- **Dado** o WorktreeIsolationProvider executando o provisionamento para uma tarefa
- **Quando** o comando git worktree add for concluido com sucesso
- **Entao** a alocacao retornada contem a branch criada, o diretorio isolado da worktree e a branch base preservada.

## Rastreabilidade
- Regra de Negocio: RN-001
- Requisito Funcional: RF-001
- Evidencia: EV-001-worktree-provision-notification.json
