# TASK-002: Servidor HTTP Local com Endpoint SSE e Dashboard

layer: ORCH
depends_on: ["TASK-001"]
target_test_file: crates/trustkit-orch/tests/wave_board_http_server_tests.rs
expected_files:
  - crates/trustkit-orch/src/wave_board_http_server.rs
  - crates/trustkit-orch/tests/wave_board_http_server_tests.rs

## Contexto
Servidor HTTP Local com Endpoint SSE e Dashboard atendendo RN-002 e RF-002 no componente Node2.

## Criterios de Aceitacao (Gherkin)
- **Dado** o servidor HTTP de wave board instanciado com o dashboard HTML
- **Quando** clientes HTTP requisitarem a rota raiz ou o endpoint de eventos /events
- **Entao** a rota raiz retorna o HTML com Content-Type text/html e a rota /events transmite eventos de mutacao via Server-Sent Events.

## Rastreabilidade
- Regra de Negocio: RN-002
- Requisito Funcional: RF-002
- Evidencia: EV-002-wave-board-http-server.json
