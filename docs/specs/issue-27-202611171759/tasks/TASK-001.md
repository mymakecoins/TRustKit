# TASK-001: Alocacao Automatica de Porta Livre e Gestao de Metadados

layer: ORCH
depends_on: []
target_test_file: crates/trustkit-orch/tests/wave_board_port_allocator_tests.rs
expected_files:
  - crates/trustkit-orch/src/wave_board_port_allocator.rs
  - crates/trustkit-orch/tests/wave_board_port_allocator_tests.rs

## Contexto
Alocacao Automatica de Porta Livre e Gestao de Metadados atendendo RN-001 e RF-001 no componente Node1.

## Criterios de Aceitacao (Gherkin)
- **Dado** o comando de inicializacao do board acionado sem configuracao manual de porta
- **Quando** o servico de deteccao de porta dinamicamente ligar o socket em 127.0.0.1 com porta efemera
- **Entao** o sistema aloca uma porta TCP livre, grava os metadados em .trustkit/wave-board.json e disponibiliza a URL final pronta para conexao.

## Rastreabilidade
- Regra de Negocio: RN-001
- Requisito Funcional: RF-001
- Evidencia: EV-001-wave-board-port-allocator.json
