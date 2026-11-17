# TASK-004: Ciclo de Vida Daemon Encerramento Gracioso e Status Operacional

layer: ORCH
depends_on: ["TASK-003"]
target_test_file: crates/trustkit-orch/tests/wave_board_lifecycle_shutdown_tests.rs
expected_files:
  - crates/trustkit-orch/src/wave_board_lifecycle_shutdown.rs
  - crates/trustkit-orch/tests/wave_board_lifecycle_shutdown_tests.rs

## Contexto
Ciclo de Vida Daemon Encerramento Gracioso e Status Operacional atendendo RN-004 e RF-004 no componente Node4.

## Criterios de Aceitacao (Gherkin)
- **Dado** um servico de board previamente inicializado e com metadados persistidos
- **Quando** o comando wave board down for executado ou wave board status for consultado
- **Entao** o status informa PID, porta e tempo de atividade com precisao, e o encerramento envia sinal de desligamento gracioso removendo o arquivo de metadados.

## Rastreabilidade
- Regra de Negocio: RN-004
- Requisito Funcional: RF-004
- Evidencia: EV-004-wave-board-lifecycle-shutdown.json
