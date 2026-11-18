# TASK-004: Integracao de Eventos nos Comandos CLI e Suite E2E

layer: CLI
depends_on: [TASK-003]
target_test_file: crates/trustkit-cli/tests/wave_board_realtime_cli_e2e_tests.rs
expected_files:
  - crates/trustkit-cli/src/commands/wave_board_cmd.rs
  - crates/trustkit-cli/src/commands/task_cmd.rs
  - crates/trustkit-cli/src/commands/wave_run_cmd.rs
  - crates/trustkit-cli/tests/wave_board_realtime_cli_e2e_tests.rs

## Contexto
Integrar a emissao automatica de mutacoes e a leitura do banco SQLite WAL nos comandos `tk board serve`, `tk task test`, `tk task complete`, `tk wave run`, `tk init`, `tk spec lock` e `tk wave plan`, validando o fluxo ponta a ponta de sincronizacao em tempo real e o desacoplamento gracioso quando o daemon estiver parado, atendendo RN-004 e RF-004 no componente Node4.

## Criterios de Aceitacao (Gherkin)
- **Dado** um ciclo SDD ativo executando transicoes via tk task test, tk task complete e tk wave run
- **Quando** o estado for persistido no SQLite com o Wave Board ativo ou parado
- **Entao** com o board ativo os eventos sao entregues em tempo real via SSE e com o board parado os comandos concluem normalmente com exit code 0.

## Rastreabilidade
- Regra de Negocio: RN-004
- Requisito Funcional: RF-004
- Evidencia: EV-004-wave-board-realtime-cli-e2e.json
