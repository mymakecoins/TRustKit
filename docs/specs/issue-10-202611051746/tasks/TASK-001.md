# TASK-001: Declarar Comando CLI e Argumentos trustkit auto

> **Camada:** CLI
> **Estimativa:** P
> **Status:** Pendente

depends_on: []
layer: CLI
Arquivos Esperados:
- crates/trustkit-cli/src/cli_contract.rs
- crates/trustkit-cli/src/commands/auto_cmd.rs
test_file: crates/trustkit-cli/tests/cli_auto_args_tests.rs

---

## Contexto
O comando `trustkit auto` precisa expor uma interface amigavel com suporte mutuo a `--issue <link|id>` e `--spec <caminho-local>`, alem das opcoes `--target`, `--runtime`, `--agent-cmd` e `--json`.

## Escopo
- Adicionar `AutoArgs` e variante `Command::Auto(AutoArgs)` em `cli_contract.rs`.
- Validar mutuamente a presenca de pelo menos um argumento (`--issue` ou `--spec`).
- Criar o esqueleto do comando em `crates/trustkit-cli/src/commands/auto_cmd.rs`.
