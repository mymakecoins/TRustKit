# TASK-005: Integracao e Paridade Estrita dos Binarios trustkit e tk

layer: CLI
depends_on: ["TASK-004"]
target_test_file: crates/trustkit-cli/tests/wave_board_e2e_parity_tests.rs
expected_files:
  - crates/trustkit-cli/src/wave_board_e2e_parity.rs
  - crates/trustkit-cli/tests/wave_board_e2e_parity_tests.rs

## Contexto
Integracao e Paridade Estrita dos Binarios trustkit e tk atendendo RN-005 e RF-005 no componente Node5.

## Criterios de Aceitacao (Gherkin)
- **Dado** os comandos board e wave board executados via trustkit ou pelo alias tk
- **Quando** os comandos forem processados em modo TTY ou com a flag --json
- **Entao** ambos os binarios entregam comportamento identico com bloco NEXT STEP em TTY e JSON valido livre de valores nulos.

## Rastreabilidade
- Regra de Negocio: RN-005
- Requisito Funcional: RF-005
- Evidencia: EV-005-wave-board-e2e-parity.json
