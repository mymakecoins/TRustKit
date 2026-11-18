# TASK-003: Frontend Kanban Dinamico Reativo via EventSource SSE

layer: ORCH
depends_on: [TASK-002]
target_test_file: crates/trustkit-orch/tests/wave_board_dynamic_frontend_tests.rs
expected_files:
  - docs/prototypes/trustkit-kanban-dashboard.html
  - crates/trustkit-orch/src/wave_board_dashboard_sse_client.rs
  - crates/trustkit-orch/tests/wave_board_dynamic_frontend_tests.rs

## Contexto
Atualizar `docs/prototypes/trustkit-kanban-dashboard.html` e `WaveBoardDashboardClient` para consumir `EventSource('/events')` e `/api/snapshot`, limpar cards estaticos de demonstracao, renderizar dinamicamente os cards do ciclo ativo, processar mutacoes incrementais entre as colunas de Gates 0 a 6 e apresentar indicador de conectividade em tempo real, atendendo RN-003 e RF-003 no componente Node3.

## Criterios de Aceitacao (Gherkin)
- **Dado** o frontend do dashboard kanban conectado ao servidor via EventSource em /events
- **Quando** o evento de snapshot inicial ou mutacoes incrementais de fases TDD e gates forem recebidos
- **Entao** o frontend remove os cards estaticos de demonstracao, renderiza os cards reais, move os cards entre as colunas de Gates 0 a 6 e atualiza badges e contadores sem recarregar a pagina.

## Rastreabilidade
- Regra de Negocio: RN-003
- Requisito Funcional: RF-003
- Evidencia: EV-003-wave-board-dynamic-frontend.json
