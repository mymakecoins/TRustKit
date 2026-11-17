# TASK-003: Diagnostico Acionavel em Task Test Phase Red com Suite Aprovada

layer: CLI
depends_on:
  - TASK-001
  - TASK-002
target_test_file: crates/trustkit-cli/tests/cli_task_test_red_guidance_tests.rs
expected_files:
  - crates/trustkit-cli/src/commands/task_cmd.rs
  - crates/trustkit-cli/tests/cli_task_test_red_guidance_tests.rs

## Contexto
Refinamento da mensagem de erro e diagnostico em task test --phase red para que, quando a suite passar com codigo zero, o desenvolvedor receba explicacao remediadora detalhada sobre a necessidade de adicionar uma nova assercao falhando em target_test_file, atendendo RN-003 e RF-003 no componente Node3.

## Criterios de Aceitacao (Gherkin)
- **Dado** um target_test_file pre-existente cujos testes executados passam com codigo de saida zero
- **Quando** o comando trustkit task test TASK-XXX --phase red for executado
- **Entao** o erro TDD_VIOLATION inclui orientacao remediadora clara para adicionar um teste que comprove a falha do novo comportamento antes de prosseguir.

## Rastreabilidade
- Regra de Negocio: RN-003
- Requisito Funcional: RF-003
- Evidencia: EV-003-task-test-red-diagnostic.json
