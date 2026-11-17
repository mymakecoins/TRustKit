# TASK-002: Servidor HTTP Local com Endpoint SSE e Entrega do Dashboard

layer: ORCH
depends_on: ["TASK-001"]
target_test_file: crates/trustkit-orch/tests/wave_board_server_sse_tests.rs
expected_files:
  - crates/trustkit-orch/src/wave_board_server_sse.rs
  - crates/trustkit-orch/tests/wave_board_server_sse_tests.rs

## Contexto
Servidor HTTP Local com Endpoint SSE e Entrega do Dashboard atendendo RN-002 e RF-002 no componente Node2.

## Criterios de Aceitacao (Gherkin)
- **Dado** o daemon do wave board em execucao local
- **Quando** o cliente web acessar a rota raiz ou se conectar ao endpoint de eventos /events
- **Entao** o servidor entrega a interface HTML do dashboard kanban e transmite notificacoes de mutacao de estado originadas no banco SQLite WAL via Server-Sent Events.

## Rastreabilidade
- Regra de Negocio: RN-002
- Requisito Funcional: RF-002
- Evidencia: EV-002-wave-board-server-sse.json
