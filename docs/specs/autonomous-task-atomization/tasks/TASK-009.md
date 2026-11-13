# TASK-009: Integracao E2E Completa de Atomizacao e Execucao no trustkit auto

layer: CLI
depends_on: ["TASK-002", "TASK-004", "TASK-007", "TASK-008"]
target_test_file: crates/trustkit-cli/tests/cli_auto_atomization_e2e_tests.rs
expected_files:
  - crates/trustkit-cli/src/commands/auto_cmd.rs
  - crates/trustkit-cli/tests/cli_auto_atomization_e2e_tests.rs

## Contexto
Conectar no handler `auto_cmd.rs` o pipeline completo: verificacao de tarefas existentes, acionamento automatico do atomizador de spec/issue quando ausentes, Circuit Breaker de backlog vazio, scaffolding RED, dispatch do construtor e merge seguro de ondas.

## Criterios de Aceitacao (Gherkin)
- **Dado** uma especificacao formal apontada em `trustkit auto --spec <path> --target develop --json` sem pasta `tasks/` previa
- **Quando** o comando autonomo for executado
- **Entao** deve atomizar as tarefas, planejar as ondas no Gate 3, executar o ciclo TDD no Gate 4 e homologar o ciclo com commits reais.

## Rastreabilidade
- Regra de Negocio: RN-009
- Requisito Funcional: RF-009
- Evidencia: EV-009-cli-auto-atomization-e2e.json
