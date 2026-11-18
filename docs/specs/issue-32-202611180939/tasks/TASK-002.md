# TASK-002: Reconciliador de Snapshot Inicial via SQLite WAL

layer: ORCH
depends_on: [TASK-001]
target_test_file: crates/trustkit-orch/tests/wave_board_sqlite_snapshot_tests.rs
expected_files:
  - crates/trustkit-orch/src/wave_board_snapshot_builder.rs
  - crates/trustkit-orch/src/wave_board_server_sse.rs
  - crates/trustkit-orch/tests/wave_board_sqlite_snapshot_tests.rs

## Contexto
Implementar `WaveBoardSnapshotBuilder` para consolidar ciclos e tarefas reais armazenados no SQLite WAL (`.trustkit/state.db`) e expor o snapshot livre de valores nulos no endpoint `GET /api/snapshot` e no frame inicial `event: snapshot` de `GET /events`, atendendo RN-002 e RF-002 no componente Node2.

## Criterios de Aceitacao (Gherkin)
- **Dado** o daemon do Wave Board em execucao com ciclos e tarefas reais persistidos no SQLite WAL
- **Quando** um cliente requisitar o endpoint /api/snapshot ou conectar ao stream /events
- **Entao** o servidor retorna o snapshot consolidado livre de valores nulos contendo os cards reais, gates, fases TDD, hashes BLAKE3 e comandos recomendados.

## Rastreabilidade
- Regra de Negocio: RN-002
- Requisito Funcional: RF-002
- Evidencia: EV-002-wave-board-sqlite-snapshot.json
