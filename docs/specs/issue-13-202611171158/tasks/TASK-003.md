# TASK-003: Subcomando CLI trustkit wave view

layer: CLI
depends_on: ["TASK-002"]
target_test_file: crates/trustkit-cli/tests/cli_wave_view_cmd_tests.rs
expected_files:
  - crates/trustkit-cli/src/cli_wave_view_cmd.rs
  - crates/trustkit-cli/tests/cli_wave_view_cmd_tests.rs

## Contexto
Subcomando CLI trustkit wave view atendendo RN-003 e RF-064 no componente Node3.

## Criterios de Aceitacao (Gherkin)
- **Dado** que o desenvolvedor ou agente executa `trustkit wave view` no terminal
- **Quando** houver um ciclo ativo e tarefas registradas
- **Entao** o comando emite no stdout a saida textual formatada exatamente no padrao de ondas:

## Rastreabilidade
- Regra de Negocio: RN-003
- Requisito Funcional: RF-064
- Evidencia: EV-003-cli-wave-view-cmd.json
