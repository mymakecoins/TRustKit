# TASK-003: Subcomando CLI update, Apresentador Dual-Audience e Paridade tk

layer: CLI
depends_on: [TASK-002]
target_test_file: crates/trustkit-cli/tests/cli_update_command_tests.rs
expected_files:
  - crates/trustkit-cli/src/cli_contract.rs
  - crates/trustkit-cli/src/bootstrap.rs
  - crates/trustkit-cli/src/commands/mod.rs
  - crates/trustkit-cli/src/commands/update_cmd.rs
  - crates/trustkit-cli/src/update_presenter.rs
  - crates/trustkit-cli/src/presenter.rs
  - crates/trustkit-cli/src/lib.rs
  - crates/trustkit-cli/tests/cli_update_command_tests.rs

## Contexto
Adicionar o subcomando `Update(UpdateArgs)` com flags `--check`, `--force` e `--json` em `cli_contract.rs`, implementar `update_presenter.rs` com saida Dual-Audience (TTY com `[INFO]`, `[UPDATE]`, `[PASS]` e rodape `NEXT STEP:` e JSON null-free com `next_steps`), registrar `update_cmd.rs` e despachar em `bootstrap.rs` com paridade estrita entre `trustkit update` e `tk update`, atendendo RN-003 e RF-003 no componente Node3.

## Criterios de Aceitacao (Gherkin)
- **Dado** os binarios `trustkit` e `tk` compilados com o subcomando `Update(UpdateArgs)`
- **Quando** o operador invocar `trustkit update` ou `tk update` com `--help`, `--check`, `--force` ou `--json`
- **Entao** ambos exibem ajuda consistente sem emojis, saida TTY padronizada com `NEXT STEP:` e payload JSON livre de `null` contendo `next_steps`.

## Rastreabilidade
- Regra de Negocio: RN-003
- Requisito Funcional: RF-003
- Evidencia: EV-003-cli-update-command.json
