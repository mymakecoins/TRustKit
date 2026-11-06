# TASK-003: Garantir Execucao Headless Nao-Bloqueante em Gauntlet Run e Resolucao de Politica

> **Camada:** CLI
> **Estimativa:** M
> **Status:** Pendente

depends_on: [TASK-001]
layer: CLI
Arquivos Esperados:
- crates/trustkit-cli/src/commands/gauntlet_run_cmd.rs
test_file: crates/trustkit-cli/tests/cli_gauntlet_headless_tests.rs

---

## Contexto
Quando `trustkit gauntlet run` e executado por agentes autonomos sem terminal TTY associado ou com variavel de ambiente indicando execucao em lote, o comando nao pode travar solicitando confirmacao interativa no stdin. Alem disso, a configuracao em `trustkit.toml` deve ser localizada a partir da raiz do repositorio de trabalho.

## Escopo
- Corrigir a localizacao de `trustkit.toml` e `.trustkit/config.json` no repositorio raiz em `gauntlet_run_cmd.rs`.
- Garantir que em ambientes headless (`!is_interactive_tty` ou quando invocado sob agentes) o prompt de revisao de seguranca nao bloqueie a execucao, adotando o fallback por politica padrao ou skip automatico seguro.
