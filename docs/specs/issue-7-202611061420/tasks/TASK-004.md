# TASK-004: Implementar Suite de Testes de Integracao para Help, Version e Headless

> **Camada:** CLI
> **Estimativa:** M
> **Status:** Pendente

depends_on: [TASK-002, TASK-003]
layer: CLI
Arquivos Esperados:
- crates/trustkit-cli/tests/cli_help_exit_code_tests.rs
- crates/trustkit-cli/tests/cli_gauntlet_headless_tests.rs
test_file: crates/trustkit-cli/tests/cli_help_exit_code_tests.rs

---

## Contexto
Para certificar a entrega e prevenir regressoes futuras no DoD da Issue #7, e necessario cobrir via testes de integracao a execucao real do binario com os parametros `--help`, `-h`, `--version` em subcomandos e execucao sem terminal interativo.

## Escopo
- Criar `crates/trustkit-cli/tests/cli_help_exit_code_tests.rs` cobrindo subcomandos raiz, `gauntlet`, `gauntlet run`, `init`, `spec`, etc.
- Criar `crates/trustkit-cli/tests/cli_gauntlet_headless_tests.rs` verificando ausencia de bloqueio de prompt em stdin nao conectado a terminal TTY.
