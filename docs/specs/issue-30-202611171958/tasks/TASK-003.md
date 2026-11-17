# TASK-003: Despachante de Comandos Wave Board e Bootstrap CLI

layer: CLI
depends_on: ["TASK-002"]
target_test_file: crates/trustkit-cli/tests/wave_board_command_dispatcher_tests.rs
expected_files:
  - crates/trustkit-cli/src/wave_board_command_dispatcher.rs
  - crates/trustkit-cli/tests/wave_board_command_dispatcher_tests.rs

## Contexto
Despachante de Comandos Wave Board e Bootstrap CLI atendendo RN-003 e RF-003 no componente Node3.

## Criterios de Aceitacao (Gherkin)
- **Dado** o comando board up acionado pelo operador
- **Quando** o despachante executar o comando
- **Entao** a porta TCP efemera e alocada, o processo daemon e disparado em segundo plano, os metadados sao gravados em .trustkit/wave-board.json e a resposta formatada e apresentada no terminal.

## Rastreabilidade
- Regra de Negocio: RN-003
- Requisito Funcional: RF-003
- Evidencia: EV-003-wave-board-command-dispatcher.json
