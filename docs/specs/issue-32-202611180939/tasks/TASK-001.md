# TASK-001: Publicador IPC de Mutacoes e Ingestao HTTP no Daemon

layer: ORCH
depends_on: []
target_test_file: crates/trustkit-orch/tests/wave_board_ipc_event_ingestion_tests.rs
expected_files:
  - crates/trustkit-orch/src/wave_board_event_publisher.rs
  - crates/trustkit-orch/src/wave_board_server_sse.rs
  - crates/trustkit-orch/tests/wave_board_ipc_event_ingestion_tests.rs

## Contexto
Implementar o publicador IPC `WaveBoardEventPublisher` e o endpoint HTTP interno `POST /api/events` em `WaveBoardServerSse` para recepcao e broadcast de mutacoes de ciclo e tarefas (`KanbanCardMutation`) ao barramento SSE, com desacoplamento gracioso quando o daemon nao estiver ativo, atendendo RN-001 e RF-001 no componente Node1.

## Criterios de Aceitacao (Gherkin)
- **Dado** o servidor do Wave Board ativo com endpoint de ingestao de eventos e canal broadcast SSE
- **Quando** um publicador IPC enviar uma mutacao de card ou o daemon nao estiver ativo no ambiente
- **Entao** o daemon ativo transmite o frame SSE de mutacao aos clientes conectados e a ausencia do daemon e ignorada graciosamente sem erros.

## Rastreabilidade
- Regra de Negocio: RN-001
- Requisito Funcional: RF-001
- Evidencia: EV-001-wave-board-ipc-event-ingestion.json
