# TASK-004: Ciclo de Vida Daemon e Encerramento Gracioso

layer: ORCH
depends_on: ["TASK-003"]
target_test_file: crates/trustkit-orch/tests/wave_board_lifecycle_daemon_tests.rs
expected_files:
  - crates/trustkit-orch/src/wave_board_lifecycle_daemon.rs
  - crates/trustkit-orch/tests/wave_board_lifecycle_daemon_tests.rs

## Contexto
Ciclo de Vida Daemon e Encerramento Gracioso atendendo RN-004 e RF-004 no componente Node4.

## Criterios de Aceitacao (Gherkin)
- **Dado** o daemon do wave board em execucao com metadados persistidos
- **Quando** o comando board down for acionado ou board status for consultado
- **Entao** o status exibe PID, porta, URL e tempo de atividade, e o encerramento envia sinal de desligamento ao processo e remove o arquivo de metadados.

## Rastreabilidade
- Regra de Negocio: RN-004
- Requisito Funcional: RF-004
- Evidencia: EV-004-wave-board-lifecycle-daemon.json
