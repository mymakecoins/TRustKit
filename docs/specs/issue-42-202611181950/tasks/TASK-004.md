# TASK-004: CLI AutoArgs Reconciliacao Interativa TTY e Resolucao Declarativa JSON

layer: CLI
depends_on: ["TASK-002", "TASK-003"]
target_test_file: crates/trustkit-cli/tests/cli_auto_reconciliation_tests.rs
expected_files:
  - crates/trustkit-cli/src/cli_auto_reconciliation.rs
  - crates/trustkit-cli/tests/cli_auto_reconciliation_tests.rs

## Contexto
CLI AutoArgs Reconciliacao Interativa TTY e Resolucao Declarativa JSON atendendo RN-004 e RF-004 no componente Node4.

## Criterios de Aceitacao (Gherkin)
- **Dado** a invocacao de `tk auto --issue <N>` ou `trustkit auto`
- **Quando** existirem ciclos ativos concorrentes em modo TTY interativo ou com as flags `--new`, `--close-merged` e `--reuse-cycle <ID>` em modo `--json`
- **Entao** o CLI reconcilia os ciclos sem travar a entrada padrao em modo autonomo, fecha ciclos mergeados quando solicitado e emite orientacao de Gate 1 no `next_steps` se a especificacao ainda nao estiver materializada.

## Rastreabilidade
- Regra de Negocio: RN-004
- Requisito Funcional: RF-004
- Evidencia: EV-004-cli-auto-reconciliation.json
