# TASK-003: Novo Comando CLI trustkit close e Transicao de Governanca com Depreciacao de merge

layer: CLI
depends_on: ["TASK-001", "TASK-002"]
target_test_file: crates/trustkit-cli/tests/cli_close_command_tests.rs
expected_files:
  - crates/trustkit-cli/src/cli_contract.rs
  - crates/trustkit-cli/src/commands/close_cmd.rs
  - crates/trustkit-cli/tests/cli_close_command_tests.rs

## Contexto
Transicao de Estado SQLite e Board para ReadyForTest via CLI sem flag --target e aviso de depreciacao de merge atendendo RN-003 e RF-003 no componente Node3.

## Criterios de Aceitacao (Gherkin)
- **Dado** o comando legado `trustkit merge` executado pelo desenvolvedor
- **Quando** o comando for processado pelo parser da CLI
- **Entao** uma mensagem de aviso de depreciacao e emitida orientando a adocao imediata de `trustkit close`.

## Rastreabilidade
- Regra de Negocio: RN-003
- Requisito Funcional: RF-003
- Evidencia: EV-003-cli-deprecation-warning.json
