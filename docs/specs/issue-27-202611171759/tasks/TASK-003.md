# TASK-003: Comandos CLI Wave Board Up Down Status e Paridade com Alias tk

layer: CLI
depends_on: ["TASK-002"]
target_test_file: crates/trustkit-cli/tests/wave_board_cli_commands_tests.rs
expected_files:
  - crates/trustkit-cli/src/wave_board_cli_commands.rs
  - crates/trustkit-cli/tests/wave_board_cli_commands_tests.rs

## Contexto
Comandos CLI Wave Board Up Down Status e Paridade com Alias tk atendendo RN-003 e RF-003 no componente Node3.

## Criterios de Aceitacao (Gherkin)
- **Dado** o operador executando os subcomandos wave board up, down ou status no terminal
- **Quando** o comando for invocado indistintamente via trustkit ou pelo alias abreviado tk em modo texto ou com a flag --json
- **Entao** a resposta terminal apresenta equivalencia comportamental estrita com recomendacao de proximo passo em modo TTY e contrato JSON livre de valores nulos.

## Rastreabilidade
- Regra de Negocio: RN-003
- Requisito Funcional: RF-003
- Evidencia: EV-003-wave-board-cli-commands.json
