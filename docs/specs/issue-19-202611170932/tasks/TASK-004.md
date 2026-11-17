# TASK-004: Serializacao Estruturada e Null-Free no Modo JSON

layer: CLI
depends_on: ["TASK-003"]
target_test_file: crates/trustkit-cli/tests/cli_worktree_notification_json_tests.rs
expected_files:
  - crates/trustkit-cli/src/gate_presenter.rs
  - crates/trustkit-cli/src/commands/wave_run_cmd.rs
  - crates/trustkit-cli/tests/cli_worktree_notification_json_tests.rs

## Contexto
Incluir campos explicitos de branch e worktree path no payload JSON de resposta sem valores nulos, atendendo RN-004 e RF-004 no componente Node4.

## Criterios de Aceitacao (Gherkin)
- **Dado** a execucao de onda com a flag --json
- **Quando** o payload de resposta for serializado
- **Entao** o JSON contem os dados estruturados de provisionamento sem nenhum valor nulo.

## Rastreabilidade
- Regra de Negocio: RN-004
- Requisito Funcional: RF-004
- Evidencia: EV-004-cli-worktree-notification-json.json
