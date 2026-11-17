# TASK-003: Mensagens Informativas em ASCII Puro no Modo TTY

layer: CLI
depends_on: ["TASK-002"]
target_test_file: crates/trustkit-cli/tests/cli_worktree_notification_tty_tests.rs
expected_files:
  - crates/trustkit-cli/src/gate_presenter.rs
  - crates/trustkit-cli/src/commands/wave_run_cmd.rs
  - crates/trustkit-cli/src/commands/auto_cmd.rs
  - crates/trustkit-cli/tests/cli_worktree_notification_tty_tests.rs

## Contexto
Renderizar mensagens informativas em ASCII puro ao usuario confirmando a criacao da branch, o checkout na worktree e a preservacao da branch do diretorio raiz, atendendo RN-003 e RF-003 no componente Node3.

## Criterios de Aceitacao (Gherkin)
- **Dado** a execucao de onda ou ciclo autonomo em terminal interativo
- **Quando** o provisionamento de worktree for apresentado
- **Entao** o terminal exibe as mensagens informativas em ASCII puro notificando a criacao da branch, o checkout no diretorio isolado e a preservacao da branch raiz.

## Rastreabilidade
- Regra de Negocio: RN-003
- Requisito Funcional: RF-003
- Evidencia: EV-003-cli-worktree-notification-tty.json
